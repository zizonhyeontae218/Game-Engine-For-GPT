import 'dart:io';

import 'package:flutter/material.dart';
import 'package:ge4g_client/battle_screen.dart';

import 'package:flutter_test/flutter_test.dart';
import 'package:ge4g_client/game_library.dart';
import 'package:ge4g_native/ge4g_native.dart';
import 'package:ge4g_client/controls.dart';

void main() {
  test('cardinal newest direction wins and older held key resumes', () {
    final router = InputRouter(
      GameControls.parse(
        File('../examples/flatland_harbor/controls/layouts.json')
            .readAsStringSync(),
        File('../examples/flatland_harbor/controls/bindings.json')
            .readAsStringSync(),
        'demo.flatland.harbor',
      ),
    );
    router.key('ArrowRight', true);
    expect(router.input['direction'], [1, 0]);
    router.key('ArrowUp', true);
    expect(router.input['direction'], [0, -1]);
    router.key('ArrowUp', false);
    expect(router.input['direction'], [1, 0]);
    router.clear();
    expect(router.input.containsKey('direction'), false);
  });
  testWidgets(
    'battle locks input, interpolates display HP and preserves save',
    (tester) async {
      int chosen = 0, saves = 0;
      Map<String, dynamic> data(int shown, bool locked) => {
        'turn': 2,
        'text': '할퀴기',
        'presentation_locked': locked,
        'fighters': [
          {
            'id': 'hero',
            'name': '연두',
            'hp': 20,
            'display_hp': shown,
            'hp_max': 38,
          },
          {'id': 'rival', 'name': '구름', 'hp': 12, 'hp_max': 34, 'enemy': true},
        ],
        'options': [
          {
            'id': 'move:scratch:rival',
            'text': '할퀴기',
            'group': 'moves',
            'target': 'rival',
          },
        ],
      };
      Future<void> show(int hp, bool lock) => tester.pumpWidget(
        MaterialApp(
          home: BattleScreen(
            battle: data(hp, lock),
            image: null,
            source: const Size(320, 240),
            onChoose: (_) => chosen++,
            onSave: () => saves++,
            onRotate: () {},
          ),
        ),
      );
      await show(38, true);
      expect(
        tester
            .widget<FilledButton>(
              find.byKey(const Key('battle-move:scratch:rival')),
            )
            .onPressed,
        isNull,
      );
      expect(find.text('HP 38 / 38'), findsOneWidget);
      await show(29, true);
      expect(find.text('HP 29 / 38'), findsOneWidget);
      await show(20, false);
      expect(find.text('HP 20 / 38'), findsOneWidget);
      await tester.ensureVisible(
        find.byKey(const Key('battle-move:scratch:rival')),
      );
      await tester.tap(find.byKey(const Key('battle-move:scratch:rival')));
      expect(chosen, 1);
      await tester.ensureVisible(find.text('전투 저장 / SAVE'));
      await tester.tap(find.text('전투 저장 / SAVE'));
      expect(saves, 1);
    },
  );
  test('Windows Korean package import launches from Unicode storage', () async {
    if (!Platform.isWindows) return;
    final root = Directory.systemTemp.createTempSync('GE4G 한글 저장-');
    final native = BasementNative();
    int? session;
    try {
      final library = GameLibrary(
        root,
        (project) => native.request({'op': 'validate', 'project': project}),
      );
      final game = await library.importFile(File('../dist/한글 검증/바람항 공방.ge4g'));
      expect(game.name, 'FlatLand rc4 / 바람항 공방');
      final status = native.request({'op': 'open', 'project': game.project});
      session = status['session'] as int;
      expect(status['scene'], '항구');
      native.request({
        'op': 'choose',
        'session': session,
        'choice': 'continue',
      });
      native.request({
        'op': 'command',
        'session': session,
        'actions': [
          {'op': 'view', 'mode': 'depth'},
          {'op': 'goto', 'scene': 'inside_home', 'spawn': 'entry'},
        ],
      });
      expect(
        native.request({
          'op': 'advance',
          'session': session,
          'ticks': 0,
        })['systems']['view'],
        'depth',
      );
    } finally {
      if (session != null) native.request({'op': 'close', 'session': session});
      root.deleteSync(recursive: true);
    }
  });
}
