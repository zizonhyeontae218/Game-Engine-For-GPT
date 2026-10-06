import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:archive/archive.dart';
import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ge4g_client/game_library.dart';
import 'package:ge4g_native/ge4g_native.dart';

Uint8List package({
  String id = 'demo.basement',
  bool badHash = false,
  String? extra,
  String projectText = 'test project',
  String version = '0.1.0',
  String name = 'Test game',
}) {
  final files = <String, Uint8List>{
    'game/ge4g.toml': Uint8List.fromList(utf8.encode(projectText)),
    'controls/layouts.json': File('assets/default_layouts.json')
        .readAsBytesSync(),
    'controls/bindings.json': Uint8List.fromList(
      utf8.encode(
        File('assets/default_bindings.json')
            .readAsStringSync()
            .replaceAll('demo.basement', id),
      ),
    ),
    ?extra: Uint8List(1),
  };
  final manifest = {
    'schema_version': 1,
    'game_id': id,
    'name': name,
    'version': version,
    'engine_abi': 1,
    'project': 'game/ge4g.toml',
    'layouts': 'controls/layouts.json',
    'bindings': 'controls/bindings.json',
    'files': {
      for (final entry in files.entries)
        entry.key: badHash ? 'wrong' : sha256.convert(entry.value).toString(),
    },
  };
  final archive = Archive();
  for (final entry in files.entries) {
    archive.addFile(ArchiveFile.bytes(entry.key, entry.value));
  }
  archive.addFile(ArchiveFile.string('bundle.json', jsonEncode(manifest)));
  return Uint8List.fromList(ZipEncoder().encode(archive));
}

void main() {
  test('data imports validate before activation, preserve per-game directories and reject malicious packages', () async {
    final root = Directory.systemTemp.createTempSync('ge4g-import-');
    var validated = 0;
    final library = GameLibrary(root, (project) {
      expect(File(project).readAsStringSync(), 'test project');
      validated++;
    });
    try {
      final game = await library.importBytes(package());
      expect(validated, 1);
      expect(File(game.project).existsSync(), true);
      expect((await library.list()).single.id, 'demo.basement');
      await library.importBytes(package(id: 'another.game'));
      expect((await library.list()).length, 2);
      expect(
        library.controls(game.id).path,
        isNot(library.controls('another.game').path),
      );
      expect(
        library.save(game.id).path,
        isNot(library.save('another.game').path),
      );
      await expectLater(
        library.importBytes(package(badHash: true)),
        throwsFormatException,
      );
      for (final path in [
        '../escape',
        '/absolute',
        'game\\escape',
        'game/../escape',
        'game/C:escape',
        'GAME/ge4g.toml',
      ]) {
        await expectLater(
          library.importBytes(package(extra: path)),
          throwsFormatException,
        );
      }
      expect((await library.list()).length, 2);
      expect(validated, 2);
      final rejected = GameLibrary(root, (_) => throw StateError('bad scene'));
      await expectLater(
        rejected.importBytes(package(id: 'invalid.game')),
        throwsStateError,
      );
      expect((await library.list()).length, 2);
    } finally {
      root.deleteSync(recursive: true);
    }
  });
  test('actual Dart FFI runs the packaged demo replay and renders canonical pixels', () async {
    final native = BasementNative();
    final root = Directory.systemTemp.createTempSync('ge4g-native-');
    final library = GameLibrary(root, (project) {
      native.request({'op': 'validate', 'project': project});
    });
    int? session;
    try {
      final bundle = File('../dist/basement-demo.ge4g');
      expect(
        bundle.existsSync(),
        true,
        reason: 'Run scripts/pack_game.py before Flutter acceptance tests',
      );
      final game = await library.importFile(bundle);
      final opened = native.request({'op': 'open', 'project': game.project});
      session = opened['session'] as int;
      final replay = jsonDecode(
        File('${game.directory.path}/game/replays/journey.json')
            .readAsStringSync(),
      );
      Map<String, dynamic> state = opened;
      for (var tick = 0; tick < 160; tick++) {
        final actions = <String>{};
        for (final span in replay['inputs']) {
          if (tick >= span['start'] && tick < span['end']) {
            actions.addAll((span['actions'] as List).cast<String>());
          }
        }
        state = native.request({
          'op': 'advance',
          'session': session,
          'ticks': 1,
          'input': {
            for (final key in ['left', 'right', 'up', 'down', 'interact'])
              key: actions.contains(key),
          },
        });
      }
      final snapshot = native.request({
        'op': 'observe',
        'session': session,
      })['snapshot'];
      expect(snapshot['scene'], 'room_b');
      expect(snapshot['tick'], 160);
      expect(snapshot['state']['demo.npc.spoken'], true);
      expect(snapshot['state']['demo.room_b.entered'], true);
      expect(
        sha256.convert(native.frame(session, state['frame_bytes'])).toString(),
        '38cc4352e7160dcf9104cbe19acd709e36d8165989b3c99d8b6611f0f76e932b',
      );
      final save = library.save(game.id);
      save.parent.createSync(recursive: true);
      native.request({'op': 'save', 'session': session, 'path': save.path});
      native.request({'op': 'close', 'session': session});
      session = native.request({
        'op': 'open',
        'project': game.project,
        'load': save.path,
      })['session'];
      expect(
        native.request({
          'op': 'observe',
          'session': session,
        })['snapshot']['state']['demo.npc.spoken'],
        true,
      );
    } finally {
      if (session != null) native.request({'op': 'close', 'session': session});
      root.deleteSync(recursive: true);
    }
  });
}
