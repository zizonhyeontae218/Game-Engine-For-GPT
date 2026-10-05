import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ge4g_client/battle_screen.dart';
import 'package:ge4g_client/controls.dart';

void main() {
  test(
    'town preset selects one axis, keeps keyboard chords and releases touches',
    () {
      final router = InputRouter(
        GameControls.parse(
          File('../examples/flatland_nuvema/controls/layouts.json')
              .readAsStringSync(),
          File('../examples/flatland_nuvema/controls/bindings.json')
              .readAsStringSync(),
          'demo.flatland.nuvema',
        ),
      );
      router.joystick(.5, .8);
      expect(router.input['down'], true);
      expect(router.input['right'], false);
      router.joystick(.8, .5);
      expect(router.input['right'], true);
      expect(router.input['down'], false);
      router.joystick(
        .59,
        .60,
      ); // near-diagonal jitter retains horizontal intent
      expect(router.input['right'], true);
      expect(router.input['down'], false);
      router.joystick(.5, .8);
      expect(router.input['down'], true);
      router.key('KeyZ', true);
      expect(router.input['interact'], true);
      router.clear();
      expect(router.input['interact'], false);
      expect(router.input['right'], false);
    },
  );
  for (final size in [
    const Size(390, 844),
    const Size(800, 360),
    const Size(640, 320),
  ]) {
    testWidgets('dedicated battle moves, bag and save fit $size at 2x text', (
      tester,
    ) async {
      tester.view.physicalSize = size;
      tester.view.devicePixelRatio = 1;
      tester.platformDispatcher.textScaleFactorTestValue = 2;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
      String? chosen;
      int saves = 0;
      await tester.pumpWidget(
        MaterialApp(
          home: BattleScreen(
            battle: const {
              'turn': 2,
              'text': '새싹이의 잎새 바람! 상대에게 12 피해.',
              'result': null,
              'fighters': [
                {'id': 'hero', 'name': '새싹이', 'hp': 18, 'hp_max': 32},
                {
                  'id': 'rival',
                  'name': '풀숲 친구',
                  'enemy': true,
                  'hp': 24,
                  'hp_max': 36,
                },
              ],
              'options': [
                {
                  'id': 'move:burst:rival',
                  'text': '잎새 바람',
                  'pp': 9,
                  'group': 'moves',
                  'target': 'rival',
                },
                {'id': 'guard', 'text': '방어', 'group': 'moves'},
                {'id': 'item_potion', 'text': '회복약 ×5', 'group': 'items'},
              ],
            },
            image: null,
            source: const Size(320, 240),
            onChoose: (id) => chosen = id,
            onSave: () => saves++,
            onRotate: () {},
          ),
        ),
      );
      await tester.ensureVisible(
        find.byKey(const Key('battle-move:burst:rival')),
      );
      await tester.tap(find.byKey(const Key('battle-move:burst:rival')));
      expect(chosen, 'move:burst:rival');
      await tester.ensureVisible(find.text('가방 / BAG'));
      await tester.tap(find.text('가방 / BAG'));
      await tester.pump();
      await tester.ensureVisible(find.byKey(const Key('battle-item_potion')));
      await tester.tap(find.byKey(const Key('battle-item_potion')));
      await tester.pump();
      expect(chosen, 'item_potion');
      await tester.ensureVisible(find.text('전투 저장 / SAVE'));
      await tester.tap(find.text('전투 저장 / SAVE'));
      expect(saves, 1);
      expect(tester.takeException(), isNull);
    });
  }
}
