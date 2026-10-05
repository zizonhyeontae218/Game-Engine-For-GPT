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
  Future<List<InstalledGame>> list() async {
    if (!await _index.exists()) return [];
    final json = object(jsonDecode(await _index.readAsString()), 'library');
    version(json, 'library');
    return (json['games'] as List).map((value) {
      final game = object(value, 'game');
      final id = identifier(game['id'], 'game_id');
      final digest = text(game['digest'], 'digest');
      if (!RegExp(r'^[0-9a-f]{64}$').hasMatch(digest)) {
        throw const FormatException('invalid installed digest');
      }
      return InstalledGame(
        id,
        text(game['name'], 'name'),
        text(game['version'], 'version'),
        digest,
        Directory(p.join(root.path, 'games', id, digest)),
      );
    }).toList();
  }

  Future<InstalledGame> importFile(File file) async {
    if (await file.length() > 64 * 1024 * 1024) {
      throw const FormatException('package exceeds 64 MiB compressed');
    }
    return importBytes(await file.readAsBytes());
  }

  Future<InstalledGame> importBytes(Uint8List bytes) async {
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
      await destination.parent.create(recursive: true);
      if (!await destination.exists()) await staging.rename(destination.path);
      final game = InstalledGame(
        id,
        text(manifest['name'], 'name'),
        text(manifest['version'], 'version'),
        digest,
        destination,
      );
      final games = await list();
      games.removeWhere((item) => item.id == id);
      games.add(game);
      await atomicText(
        _index,
        jsonEncode({
          'schema_version': 1,
          'games': games.map((item) => item.toJson()).toList(),
        }),
      );
      return game;
    } finally {
      if (await staging.exists()) await staging.delete(recursive: true);
    }
  }
}
