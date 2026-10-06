import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:archive/archive.dart';
import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ge4g_client/game_library.dart';
import 'package:ge4g_client/player.dart';
import 'package:ge4g_native/ge4g_native.dart';

import 'library_test.dart' as fixture;

Uint8List revision(Uint8List source, String marker, String version) {
  final entries = <String, Uint8List>{
    for (final f in ZipDecoder().decodeBytes(source)) f.name: f.readBytes()!,
  };
  entries['game/ge4g.toml'] = Uint8List.fromList([
    ...entries['game/ge4g.toml']!,
    ...utf8.encode('\n# $marker\n'),
  ]);
  final manifest = jsonDecode(
    utf8.decode(entries.remove('bundle.json')!),
  ) as Map<String, dynamic>;
  manifest['version'] = version;
  manifest['files'] = {
    for (final e in entries.entries) e.key: sha256.convert(e.value).toString(),
  };
  final archive = Archive();
  for (final e in entries.entries) {
    archive.addFile(ArchiveFile.bytes(e.key, e.value));
  }
  archive.addFile(ArchiveFile.string('bundle.json', jsonEncode(manifest)));
  return Uint8List.fromList(ZipEncoder().encode(archive));
}

class RejectOpen implements EngineBridge {
  final NativeEngine actual = NativeEngine();
  @override
  Map<String, dynamic> request(Map<String, dynamic> value) {
    if (value['op'] == 'open' &&
        File(value['project'] as String)
            .readAsStringSync()
            .contains('# reject open')) {
      throw const NativeFailure(
        'deliberate open failure',
        code: 'invalid_project',
      );
    }
    return actual.request(value);
  }

  @override
  Uint8List frame(int session, int length) => actual.frame(session, length);
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  test('stable identity, duplicate import, order restart/update and stale index repair', () async {
    final root = Directory.systemTemp.createTempSync('ge4g-final-order-');
    final library = GameLibrary(root, (_) {});
    try {
      final a = await library.importBytes(fixture.package(id: 'a'));
      await library.importBytes(fixture.package(id: 'b'));
      final c = await library.importBytes(fixture.package(id: 'c'));
      await library.reorder(2, 0);
      final restarted = GameLibrary(root, (_) {});
      expect((await restarted.list()).map((g) => g.id), ['c', 'a', 'b']);
      await restarted.importBytes(
        fixture.package(id: 'a', projectText: 'revisionB', version: '0.2.0'),
      );
      expect(a.directory.existsSync(), false);
      final before = (await restarted.list()).map((g) => g.toJson()).toList();
      await restarted.importBytes(
        fixture.package(id: 'a', projectText: 'revisionB', version: '0.2.0'),
      );
      expect((await restarted.list()).map((g) => g.toJson()).toList(), before);
      expect(Directory('${root.path}/games/a').listSync().length, 1);
      c.directory.deleteSync(recursive: true);
      expect((await restarted.list()).map((g) => g.id), ['a', 'b']);
      expect(restarted.warnings.single, contains('설치 파일이 없습니다'));
      expect((await GameLibrary(root, (_) {}).list()).map((g) => g.id), [
        'a',
        'b',
      ]);
    } finally {
      root.deleteSync(recursive: true);
    }
  });
  test(
    'failed validation/open rolls back active pointer, all user data and order',
    () async {
      final root = Directory.systemTemp.createTempSync('ge4g-final-rollback-');
      final library = GameLibrary(root, (_) {});
      try {
        final a = await library.importBytes(fixture.package(id: 'a'));
        await library.importBytes(fixture.package(id: 'b'));
        final save = library.save('a')..parent.createSync(recursive: true);
        save.writeAsStringSync('original save');
        final settings = File('${library.controls('a').path}/layouts.json')
          ..parent.createSync(recursive: true);
        settings.writeAsStringSync('original controls');
        await expectLater(
          GameLibrary(
            root,
            (_) => throw StateError('invalid project'),
          ).importBytes(fixture.package(id: 'a', projectText: 'bad')),
          throwsStateError,
        );
        await expectLater(
          library.importBytes(
            fixture.package(id: 'a', projectText: 'B'),
            activate: (game, old) async {
              save.writeAsStringSync('changed save');
              settings.writeAsStringSync('changed settings');
              throw StateError('new open failed');
            },
          ),
          throwsStateError,
        );
        expect((await library.list()).map((g) => g.id), ['a', 'b']);
        expect((await library.list()).first.digest, a.digest);
        expect(a.directory.existsSync(), true);
        expect(save.readAsStringSync(), 'original save');
        expect(settings.readAsStringSync(), 'original controls');
        expect(Directory('${root.path}/games/a').listSync().length, 1);
      } finally {
        root.deleteSync(recursive: true);
      }
    },
  );
  test('normal deletion preserves user data, reinstall reuses it; full deletion removes archives/settings', () async {
    final root = Directory.systemTemp.createTempSync('ge4g-final-delete-');
    final library = GameLibrary(root, (_) {});
    try {
      final game = await library.importBytes(fixture.package());
      final save = library.save(game.id)..parent.createSync(recursive: true);
      save.writeAsStringSync('save');
      final settings = library.controls(game.id)..createSync(recursive: true);
      File('${settings.path}/layouts.json').writeAsStringSync('settings');
      final archived = File('${save.parent.path}/archive/old.json')
        ..parent.createSync(recursive: true);
      archived.writeAsStringSync('older save');
      library.retain(game);
      await expectLater(library.remove(game.id), throwsStateError);
      expect((await library.list()).length, 1);
      library.release(game);
      await library.remove(game.id);
      expect(await library.list(), isEmpty);
      expect(game.directory.parent.existsSync(), false);
      expect(save.readAsStringSync(), 'save');
      expect(settings.existsSync(), true);
      final installed = await library.importBytes(fixture.package());
      expect(installed.digest, game.digest);
      expect(save.readAsStringSync(), 'save');
      await library.remove(game.id, allData: true);
      expect(await library.list(), isEmpty);
      expect(game.directory.parent.existsSync(), false);
      expect(save.parent.existsSync(), false);
      expect(settings.parent.existsSync(), false);
      expect(archived.existsSync(), false);
    } finally {
      root.deleteSync(recursive: true);
    }
  });
  if (Platform.isWindows) {
    test('native revision A -> B archives exact old save, opens fresh, preserves controls; duplicate resumes', () async {
      final root = Directory.systemTemp.createTempSync('ge4g-final-native-');
      final engine = NativeEngine();
      final library = GameLibrary(
        root,
        (p) => engine.request({'op': 'validate', 'project': p}),
      );
      final player = Player(engine, library)..audio.muted = true;
      try {
        final bytes = File('../dist/flatland-harbor.ge4g').readAsBytesSync();
        final a = await player.installBytes(revision(bytes, 'A', '0.2.0-rc.5'));
        player.choose('continue');
        await player.store!.select('left_handed');
        await player.save();
        final saved = library.save(a.id).readAsBytesSync();
        final layouts = File('${library.controls(a.id).path}/layouts.json')
            .readAsStringSync();
        final bBytes = revision(bytes, 'B', '0.2.0');
        final b = await player.installBytes(bBytes);
        expect((await library.list()).single.digest, b.digest);
        expect(player.session, isNotNull);
        expect(player.archivedSave, true);
        expect(player.status['tick'], 0);
        expect(player.message, contains('이전 저장은 보관'));
        expect(library.save(a.id).existsSync(), false);
        final archive = Directory('${library.save(a.id).parent.path}/archive')
            .listSync()
            .whereType<File>()
            .single;
        expect(archive.readAsBytesSync(), saved);
        expect(a.directory.existsSync(), false);
        expect(
          File('${library.controls(a.id).path}/layouts.json')
              .readAsStringSync(),
          layouts,
        );
        expect(player.store!.current.activeId, 'left_handed');
        player.choose('continue');
        await player.save();
        final newSave = library.save(a.id).readAsBytesSync();
        final compatibleBytes = revision(bytes, 'B', '0.2.0+game-update');
        await player.installBytes(compatibleBytes);
        expect(player.resumedSave, true);
        expect(player.archivedSave, false);
        expect(b.directory.existsSync(), false);
        await player.installBytes(compatibleBytes);
        expect(player.resumedSave, true);
        expect(player.archivedSave, false);
        expect(library.save(a.id).readAsBytesSync(), newSave);
        expect((await library.list()).length, 1);
        expect(Directory('${root.path}/games/${a.id}').listSync().length, 1);
        expect(player.store!.current.activeId, 'left_handed');
        await player.removeGame(a.id);
        expect(player.session, isNull);
        expect(player.game, isNull);
        expect(library.save(a.id).readAsBytesSync(), newSave);
        await player.installBytes(compatibleBytes);
        expect(player.resumedSave, true);
        expect(player.store!.current.activeId, 'left_handed');
        await player.removeGame(a.id, allData: true);
        expect(library.save(a.id).parent.existsSync(), false);
        expect(library.controls(a.id).parent.existsSync(), false);
      } finally {
        await player.closeAndWait();
        player.dispose();
        root.deleteSync(recursive: true);
      }
    });
    test(
      'native open failure and corrupt save never masquerade as stale revision',
      () async {
        final root = Directory.systemTemp.createTempSync('ge4g-final-failure-');
        final engine = RejectOpen();
        final library = GameLibrary(
          root,
          (p) => engine.actual.request({'op': 'validate', 'project': p}),
        );
        final player = Player(engine, library)..audio.muted = true;
        try {
          final source = File('../dist/flatland-harbor.ge4g').readAsBytesSync();
          final a = await player.installBytes(
            revision(source, 'A', '0.2.0-rc.5'),
          );
          await player.save();
          final save = library.save(a.id).readAsBytesSync();
          await expectLater(
            player.installBytes(revision(source, 'reject open', '0.2.0')),
            throwsA(isA<NativeFailure>()),
          );
          expect((await library.list()).single.digest, a.digest);
          expect(a.directory.existsSync(), true);
          expect(library.save(a.id).readAsBytesSync(), save);
          await player.open(a, load: true, rethrowFailure: true);
          expect(player.resumedSave, true);
          await player.closeAndWait();
          library.save(a.id).writeAsStringSync('{broken');
          await expectLater(
            player.installBytes(revision(source, 'B', '0.2.0')),
            throwsA(isA<NativeFailure>()),
          );
          expect((await library.list()).single.digest, a.digest);
          expect(library.save(a.id).readAsStringSync(), '{broken');
          expect(
            Directory('${library.save(a.id).parent.path}/archive').existsSync(),
            false,
          );
        } finally {
          await player.closeAndWait();
          player.dispose();
          root.deleteSync(recursive: true);
        }
      },
    );
  }
}
