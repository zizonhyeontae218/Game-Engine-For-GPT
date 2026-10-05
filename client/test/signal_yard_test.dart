import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ge4g_client/game_library.dart';
import 'package:ge4g_client/game_popup.dart';
import 'package:ge4g_client/player.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  test('feature package, dialogue save/resume and choices use the native event controller', () async {
    final root = Directory.systemTemp.createTempSync('signal-yard-');
    final engine = NativeEngine();
    final library = GameLibrary(
      root,
      (path) => engine.request({'op': 'validate', 'project': path}),
    );
    final player = Player(engine, library);
    try {
      final game = await library.importFile(
        File('../dist/flatland-signal-yard.ge4g'),
      );
      await player.open(game);
      expect(player.status['game_schema'], 2);
      expect(player.status['waiting']['kind'], 'dialogue');
      player.choose('continue');
      expect(player.status['waiting']['kind'], 'choice');
      final waiting = player.status['waiting'];
      await player.save();
      await player.open(game, load: true);
      expect(player.status['waiting'], waiting);
      player.choose('accept');
      expect(player.status['systems']['inventory']['potion'], 3);
      expect(player.status['systems']['quests']['signal']['status'], 'active');
      player.choose('continue');
      expect(player.status['waiting'], isNull);
      expect(player.input!.input['actions'], isEmpty);
    } finally {
      player.dispose();
      await Future<void>.delayed(const Duration(milliseconds: 50));
      root.deleteSync(recursive: true);
    }
  });
  testWidgets(
    'choices and save remain reachable in narrow landscape with large text',
    (tester) async {
      tester.view.physicalSize = const Size(740, 360);
      tester.view.devicePixelRatio = 1;
      tester.platformDispatcher.textScaleFactorTestValue = 2;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
      String? selected;
      var saves = 0;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: GamePopup(
              text:
                  'TURN ARENA\n${List.filled(12, '대화와 턴제 전투 저장 테스트').join('\n')}',
              onClose: () {},
              choices: const [
                {'id': 'guard', 'text': '방어 / GUARD'},
                {'id': 'attack', 'text': '공격 / ATTACK'},
              ],
              onChoose: (id) => selected = id,
              onSave: () => saves++,
            ),
          ),
        ),
      );
      await tester.ensureVisible(find.byKey(const Key('choice-guard')));
      await tester.tap(find.byKey(const Key('choice-guard')));
      expect(selected, 'guard');
      await tester.ensureVisible(find.text('이 장면 저장 / SAVE'));
      await tester.tap(find.text('이 장면 저장 / SAVE'));
      expect(saves, 1);
      expect(tester.takeException(), isNull);
    },
  );
}
