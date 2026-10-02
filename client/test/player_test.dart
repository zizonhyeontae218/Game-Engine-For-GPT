import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ge4g_client/game_library.dart';
import 'package:ge4g_client/player.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  test('live profile and JSON edits release held native inputs before remapping; pause never catches up', () async {
    final root = Directory.systemTemp.createTempSync('ge4g-live-');
    final engine = NativeEngine();
    final library = GameLibrary(root, (path) {
      engine.request({'op': 'validate', 'project': path});
    });
    final player = Player(engine, library);
    try {
      final game = await library.importFile(File('../dist/basement-demo.ge4g'));
      await player.open(game);
      player.input!.joystick(1, 0);
      player.input!.button('x', 7, true);
      player.tick(Duration.zero);
      player.tick(const Duration(microseconds: 16667));
      var snapshot = engine.request({
        'op': 'observe',
        'session': player.session,
      })['snapshot'];
      expect(snapshot['tick'], 1);
      expect(
        (snapshot['events'] as List).any(
          (e) => e['kind'] == 'action_pressed' && e['data']['action'] == 'x',
        ),
        true,
      );
      await player.store!.select('left_handed');
      expect(player.input!.input['right'], false);
      expect(player.input!.input['actions'], isEmpty);
      snapshot = engine.request({
        'op': 'observe',
        'session': player.session,
      })['snapshot'];
      expect(
        (snapshot['events'] as List).any(
          (e) => e['kind'] == 'action_released' && e['data']['action'] == 'x',
        ),
        true,
      );
      final bindings = jsonDecode(player.store!.current.bindingsText);
      bindings['profiles']['left_handed']['buttons']['z'] = ['right'];
      await player.store!.apply(
        player.store!.current.layoutsText,
        jsonEncode(bindings),
      );
      player.input!.button('z', 9, true);
      expect(player.input!.input['right'], true);
      expect(player.input!.input['interact'], false);
      final valid = player.store!.current;
      await expectLater(
        player.store!.apply('{broken', valid.bindingsText),
        throwsFormatException,
      );
      expect(identical(player.store!.current, valid), true);
      expect(player.input!.input['right'], true);
      player.setPaused(true);
      expect(player.input!.input['right'], false);
      player.tick(const Duration(hours: 1));
      expect(player.status['tick'], 1);
      player.setPaused(false);
      player.tick(const Duration(hours: 1));
      player.tick(const Duration(hours: 1, microseconds: 16667));
      expect(player.status['tick'], 2);
      var attemptedLoadingTick = false;
      void duringLoading() {
        if (player.loading && player.session != null && !attemptedLoadingTick) {
          attemptedLoadingTick = true;
          player.tick(const Duration(hours: 3));
          player.tick(const Duration(hours: 3, microseconds: 16667));
        }
      }

      player.addListener(duringLoading);
      final pendingFrame = player.refreshFrame();
      final restarted = player.open(game);
      expect(player.loading, true);
      expect(player.session, isNull);
      await restarted;
      await pendingFrame;
      player.removeListener(duringLoading);
      expect(attemptedLoadingTick, true);
      expect(player.loading, false);
      expect(player.status['tick'], 0);
      expect(player.image, isNotNull);
      player.setPaused(true);
      player.tick(const Duration(hours: 2));
      expect(player.image, isNotNull);
      expect(player.error, isNull);
    } finally {
      player.dispose();
      // Allow an already-decoding frame to be disposed without publishing to a closed player.
      await Future<void>.delayed(const Duration(milliseconds: 50));
      root.deleteSync(recursive: true);
    }
  });
}
