import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:archive/archive.dart';
import 'package:crypto/crypto.dart';
import 'package:path/path.dart' as p;

import 'control_store.dart';
import 'controls.dart';

String packagePath(String value) {
  if (value.length > 240 ||
      value.contains('\\') ||
      value.contains(':') ||
      value.contains('\u0000') ||
      value.startsWith('/') ||
      value
          .split('/')
          .any((part) => part.isEmpty || part == '.' || part == '..')) {
    throw FormatException('unsafe package path: $value');
  }
  return value;
}

class InstalledGame {
  final String id, name, version, digest;
  final Directory directory;
  InstalledGame(this.id, this.name, this.version, this.digest, this.directory);
  String get project => p.join(directory.path, 'game', 'ge4g.toml');
  Future<String> get layouts =>
      File(p.join(directory.path, 'controls', 'layouts.json')).readAsString();
  Future<String> get bindings =>
      File(p.join(directory.path, 'controls', 'bindings.json')).readAsString();
  Map<String, dynamic> toJson() => {
    'id': id,
    'name': name,
    'version': version,
    'digest': digest,
  };
}

/// Only validated data packages are imported; the executable runtime is bundled by Flutter.
class GameLibrary {
  final Directory root;
  final void Function(String project) validate;
  GameLibrary(this.root, this.validate);
  Directory controls(String id) => Directory(
    p.join(root.path, 'settings', identifier(id, 'game_id'), 'controls'),
  );
  File save(String id) =>
      File(p.join(root.path, 'saves', identifier(id, 'game_id'), 'save.json'));
  File get _index => File(p.join(root.path, 'library.json'));
  final List<String> warnings = [];
  final Map<String, int> _leases = {};
  Future<void> _tail = Future.value();
  void retain(InstalledGame game) => _leases.update(
    game.directory.path,
    (count) => count + 1,
    ifAbsent: () => 1,
  );
  void release(InstalledGame game) {
    final count = _leases[game.directory.path] ?? 0;
    if (count <= 1) {
      _leases.remove(game.directory.path);
    } else {
      _leases[game.directory.path] = count - 1;
    }
  }

  Future<T> _locked<T>(Future<T> Function() operation) {
    final done = Completer<void>();
    final previous = _tail;
    _tail = done.future;
    return (() async {
      await previous;
      try {
        return await operation();
      } finally {
        done.complete();
      }
    })();
  }

  Future<void> _writeGames(List<InstalledGame> games) => atomicText(
    _index,
    jsonEncode({
      'schema_version': 1,
      'games': games.map((game) => game.toJson()).toList(),
    }),
  );
  Future<List<InstalledGame>> list() => _locked(_readGames);
  Future<List<InstalledGame>> _readGames() async {
    if (!await _index.exists()) return [];
    final json = object(jsonDecode(await _index.readAsString()), 'library');
    version(json, 'library');
    final games = <InstalledGame>[];
    final ids = <String>{};
    var changed = false;
    for (final value in json['games'] as List) {
      try {
        final game = object(value, 'game');
        final id = identifier(game['id'], 'game_id');
        final digest = text(game['digest'], 'digest');
        if (!RegExp(r'^[0-9a-f]{64}$').hasMatch(digest) || !ids.add(id)) {
          throw const FormatException('invalid/duplicate installed identity');
        }
        final directory = Directory(p.join(root.path, 'games', id, digest));
        if (!await directory.exists() ||
            !await File(p.join(directory.path, 'game', 'ge4g.toml')).exists()) {
          throw FileSystemException('설치 파일이 없습니다', directory.path);
        }
        games.add(
          InstalledGame(
            id,
            text(game['name'], 'name'),
            text(game['version'], 'version'),
            digest,
            directory,
          ),
        );
      } catch (failure) {
        warnings.add('보관함의 사용할 수 없는 항목을 제거했습니다: $failure');
        changed = true;
      }
    }
    if (changed) await _writeGames(games);
    return games;
  }

  Future<void> reorder(int oldIndex, int newIndex) => _locked(() async {
    final games = await _readGames();
    if (oldIndex < 0 ||
        oldIndex >= games.length ||
        newIndex < 0 ||
        newIndex > games.length) {
      throw RangeError('invalid library order');
    }
    if (newIndex > oldIndex) newIndex--;
    final moved = games.removeAt(oldIndex);
    games.insert(newIndex, moved);
    await _writeGames(games);
  });
  Future<void> remove(
    String id, {
    bool allData = false,
    Future<void> Function(String id)? release,
  }) => _locked(() async {
    identifier(id, 'game_id');
    final games = await _readGames();
    await release?.call(id);
    final installed = Directory(p.join(root.path, 'games', id));
    if (_leases.keys.any((path) => p.isWithin(installed.path, path))) {
      throw StateError('실행 중인 게임을 먼저 닫으세요');
    }
    // Quarantine content before changing the index; an index failure rolls it back.
    Directory? tomb;
    if (await installed.exists()) {
      tomb = Directory(
        '${installed.path}.delete-${DateTime.now().microsecondsSinceEpoch}',
      );
      await installed.rename(tomb.path);
    }
    try {
      await _writeGames(games.where((game) => game.id != id).toList());
    } catch (_) {
      if (tomb != null) await tomb.rename(installed.path);
      rethrow;
    }
    if (tomb != null) await tomb.delete(recursive: true);
    if (allData) {
      for (final directory in [
        Directory(p.join(root.path, 'saves', id)),
        Directory(p.join(root.path, 'settings', id)),
      ]) {
        if (await directory.exists()) await directory.delete(recursive: true);
      }
    }
  });
  Future<void> _collect(String id, String active) async {
    final directory = Directory(p.join(root.path, 'games', id));
    if (!await directory.exists()) return;
    await for (final entry in directory.list(followLinks: false)) {
      if (entry is Directory &&
          p.basename(entry.path) != active &&
          RegExp(r'^[0-9a-f]{64}$').hasMatch(p.basename(entry.path)) &&
          !_leases.containsKey(entry.path)) {
        try {
          await entry.delete(recursive: true);
        } catch (failure) {
          warnings.add('이전 게임 파일 정리 실패: $failure');
        }
      }
    }
  }

  Future<void> collectOrphans() => _locked(() async {
    final games = await _readGames();
    final directory = Directory(p.join(root.path, 'games'));
    if (!await directory.exists()) return;
    await for (final entry in directory.list(followLinks: false)) {
      if (entry is! Directory) continue;
      final id = p.basename(entry.path);
      try {
        identifier(id, 'game_id');
      } on FormatException {
        continue;
      }
      final active =
          games.where((game) => game.id == id).firstOrNull?.digest ?? '';
      await _collect(id, active);
    }
  });
  Future<void> _copyTree(Directory source, Directory target) async {
    if (!await source.exists()) return;
    await target.create(recursive: true);
    await for (final entry in source.list(
      recursive: true,
      followLinks: false,
    )) {
      final destination = p.join(
        target.path,
        p.relative(entry.path, from: source.path),
      );
      if (entry is Directory) {
        await Directory(destination).create(recursive: true);
      }
      if (entry is File) {
        await File(destination).parent.create(recursive: true);
        await entry.copy(destination);
      }
      if (entry is Link) {
        throw const FileSystemException('user data symlinks are not supported');
      }
    }
  }

  Future<InstalledGame> importFile(
    File file, {
    Future<void> Function(InstalledGame game, InstalledGame? previous)?
    activate,
    Future<void> Function(String id)? release,
  }) async {
    if (await file.length() > 64 * 1024 * 1024) {
      throw const FormatException('package exceeds 64 MiB compressed');
    }
    return importBytes(
      await file.readAsBytes(),
      activate: activate,
      release: release,
    );
  }

  Future<InstalledGame> importBytes(
    Uint8List bytes, {
    Future<void> Function(InstalledGame game, InstalledGame? previous)?
    activate,
    Future<void> Function(String id)? release,
  }) =>
      _locked(() => _importBytes(bytes, activate: activate, release: release));
  Future<InstalledGame> _importBytes(
    Uint8List bytes, {
    Future<void> Function(InstalledGame game, InstalledGame? previous)?
    activate,
    Future<void> Function(String id)? release,
  }) async {
    if (bytes.length > 64 * 1024 * 1024) {
      throw const FormatException('package exceeds 64 MiB compressed');
    }
    // Inspect the central directory before ZipDecoder can inflate a symlink.
    final directory = ZipDirectory()..read(InputMemoryStream(bytes));
    if (directory.fileHeaders.isEmpty || directory.fileHeaders.length > 4096) {
      throw const FormatException('package needs 1..4096 files');
    }
    final names = <String>{};
    var total = 0;
    for (final header in directory.fileHeaders) {
      final name = packagePath(header.filename);
      if (!names.add(name.toLowerCase())) {
        throw FormatException('duplicate package path $name');
      }
      final mode = (header.externalFileAttributes >> 16) & 0xf000;
      if (mode != 0 && mode != 0x8000 ||
          header.generalPurposeBitFlag & 1 != 0 ||
          ![0, 8].contains(header.compressionMethod)) {
        throw FormatException('unsupported package entry $name');
      }
      if (header.uncompressedSize > 16 * 1024 * 1024 ||
          header.uncompressedSize < 0) {
        throw FormatException('package entry too large: $name');
      }
      if (header.file?.filename != name ||
          header.file?.uncompressedSize != header.uncompressedSize) {
        throw FormatException('inconsistent ZIP header: $name');
      }
      total += header.uncompressedSize;
      if (total > 256 * 1024 * 1024) {
        throw const FormatException('package exceeds 256 MiB extracted');
      }
    }
    final archive = ZipDecoder().decodeBytes(bytes);
    final files = <String, Uint8List>{};
    for (final entry in archive) {
      final data = entry.readBytes();
      if (!entry.isFile ||
          entry.isSymbolicLink ||
          data == null ||
          data.length != entry.size) {
        throw FormatException('invalid package entry ${entry.name}');
      }
      files[entry.name] = data;
    }
    if (!files.containsKey('bundle.json')) {
      throw const FormatException('missing bundle.json');
    }
    final manifest = object(
      jsonDecode(utf8.decode(files['bundle.json']!)),
      'bundle',
    );
    if (![1, 2, 3].contains(manifest['schema_version'])) {
      throw const FormatException('unsupported game package schema');
    }
    if ([2, 3].contains(manifest['schema_version']) &&
        manifest['game_schema'] != 2) {
      throw const FormatException('FlatLand package requires game_schema 2');
    }
    fields(manifest, {
      'schema_version',
      'game_id',
      'name',
      'version',
      'engine_abi',
      if (manifest['schema_version'] == 3) 'features',
      if ([2, 3].contains(manifest['schema_version'])) 'game_schema',
      'project',
      'layouts',
      'bindings',
      'files',
    }, 'bundle');
    if (manifest['schema_version'] == 3) {
      final features = manifest['features'];
      if (features is! List ||
          features.any(
            (f) => ![
              'combat',
              'inventory',
              'quests',
              'event_scenes',
              'turn_battle',
              'planes',
              'atlas',
              'music',
              'lua_rng',
              'step_walk',
              'battle_stage',
              'view_projection',
              'billboard_projection',
              'persistent_combatants',
              'battle_fx',
              'cutscene_bubbles',
              'solid_buildings',
              'contact_ordering',
              'building_presentation',
              'entity_defaults',
            ].contains(f),
          )) {
        throw const FormatException('unsupported required engine feature');
      }
    }
    if (manifest['engine_abi'] != 1) {
      throw const FormatException('unsupported engine ABI');
    }
    if (manifest['project'] != 'game/ge4g.toml' ||
        manifest['layouts'] != 'controls/layouts.json' ||
        manifest['bindings'] != 'controls/bindings.json') {
      throw const FormatException('unsupported bundle entry points');
    }
    final hashes = object(manifest['files'], 'bundle.files');
    if (hashes.length != files.length - 1 ||
        hashes.containsKey('bundle.json')) {
      throw const FormatException('package manifest file count mismatch');
    }
    for (final entry in hashes.entries) {
      packagePath(entry.key);
      final content = files[entry.key];
      if (content == null ||
          sha256.convert(content).toString() != entry.value) {
        throw FormatException('package hash mismatch: ${entry.key}');
      }
    }
    for (final required in [
      'game/ge4g.toml',
      'controls/layouts.json',
      'controls/bindings.json',
    ]) {
      if (!files.containsKey(required)) {
        throw FormatException('missing $required');
      }
    }
    final id = identifier(manifest['game_id'], 'game_id');
    GameControls.parse(
      utf8.decode(files['controls/layouts.json']!),
      utf8.decode(files['controls/bindings.json']!),
      id,
    );
    final digest = sha256.convert(bytes).toString();
    final destination = Directory(p.join(root.path, 'games', id, digest));
    await root.create(recursive: true);
    final stagingRoot = await Directory(p.join(root.path, 'staging'))
        .create(recursive: true);
    final staging = await stagingRoot.createTemp('import-');
    try {
      for (final entry in files.entries) {
        final file = File(p.joinAll([staging.path, ...entry.key.split('/')]));
        await file.parent.create(recursive: true);
        await file.writeAsBytes(entry.value, flush: true);
      }
      validate(p.join(staging.path, 'game', 'ge4g.toml'));
      final game = InstalledGame(
        id,
        text(manifest['name'], 'name'),
        text(manifest['version'], 'version'),
        digest,
        destination,
      );
      final games = await _readGames();
      final position = games.indexWhere((item) => item.id == id);
      final previous = position < 0 ? null : games[position];
      final next = [...games];
      if (position < 0) {
        next.add(game);
      } else {
        next[position] = game;
      }
      await destination.parent.create(recursive: true);
      final created = !await destination.exists();
      if (created) await staging.rename(destination.path);
      Directory? backup;
      var switched = false;
      var opened = false;
      var preserveBackup = false;
      try {
        await release?.call(id);
        if (_leases.keys.any(
          (path) => p.isWithin(destination.parent.path, path),
        )) {
          throw StateError('실행 중인 게임을 먼저 닫으세요');
        }
        if (activate != null) {
          backup = await stagingRoot.createTemp('rollback-');
          for (final kind in ['saves', 'settings']) {
            await _copyTree(
              Directory(p.join(root.path, kind, id)),
              Directory(p.join(backup.path, kind)),
            );
          }
        }
        await _writeGames(next);
        switched = true;
        await activate?.call(game, previous);
        opened = true;
        // No active session can use the retired digest. Cleanup errors are warnings,
        // never a reason to roll back after old content has already been retired.
        try {
          await _collect(id, digest);
        } catch (failure) {
          warnings.add('이전 게임 파일 정리 실패: $failure');
        }
        return game;
      } catch (failure) {
        if (!opened) {
          try {
            if (switched) await _writeGames(games);
            if (backup != null && switched) {
              for (final kind in ['saves', 'settings']) {
                final target = Directory(p.join(root.path, kind, id));
                if (await target.exists()) await target.delete(recursive: true);
                await _copyTree(Directory(p.join(backup.path, kind)), target);
              }
            }
            if (created &&
                !_leases.containsKey(destination.path) &&
                await destination.exists()) {
              await destination.delete(recursive: true);
            }
          } catch (rollbackFailure) {
            preserveBackup = true;
            throw StateError(
              '업데이트 실패: $failure; 복구 중 파일 오류: $rollbackFailure. 원본 데이터 보관: ${backup?.path}',
            );
          }
        }
        rethrow;
      } finally {
        if (!preserveBackup && backup != null && await backup.exists()) {
          await backup.delete(recursive: true);
        }
      }
    } finally {
      if (await staging.exists()) await staging.delete(recursive: true);
    }
  }
}
