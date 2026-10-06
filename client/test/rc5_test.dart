import 'dart:io';

import 'package:ge4g_client/game_library.dart';
import 'package:ge4g_client/player.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ge4g_client/cutscene_bubble.dart';
import 'package:ge4g_client/game_popup.dart';

void main() {
  if (Platform.isWindows) {
    test('completed story cannot reopen as a live bubble; ordinary dialogue history remains', () async {
      final dir = Directory.systemTemp.createTempSync('ge4g-story-');
      final engine = NativeEngine();
      final library = GameLibrary(dir, (path) {
        engine.request({'op': 'validate', 'project': path});
      });
      final player = Player(engine, library);
      player.audio.muted = true;
      try {
        final game = await library.importFile(
          File('../dist/flatland-harbor.ge4g'),
        );
        await player.open(game);
        player.choose('continue');
        player.command([
          {'op': 'event_scene', 'event': 'tour'},
        ]);
        player.tick(Duration.zero);
        for (var i = 1; i <= 16; i++) {
          player.tick(Duration(microseconds: i * 16667));
        }
        expect(player.popup?['kind'], 'bubble');
        expect(player.popup?['screen_foot'], isA<List>());
        for (var i = 0; i < 3; i++) {
          player.choose('continue');
        }
        expect(player.popup, isNull);
        player.showPopup();
        expect(player.popupVisible, false);
        player.command([
          {'op': 'say', 'text': '일반 대화 기록'},
        ]);
        player.dismissPopup();
        player.showPopup();
        expect(player.popupVisible, true);
        expect(player.popup?['text'], '일반 대화 기록');
      } finally {
        player.dispose();
        dir.deleteSync(recursive: true);
      }
    });
  }
  for (final size in [const Size(390, 844), const Size(800, 360)]) {
    testWidgets(
      'three actor bubbles advance by tap, save is separate, normal dialogue remains $size',
      (tester) async {
        tester.view.physicalSize = size;
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        var line = 0, saves = 0;
        await tester.pumpWidget(
          MaterialApp(
            home: StatefulBuilder(
              builder: (context, setState) => line < 3
                  ? CutsceneBubble(
                      bubble: {
                        'id': 'line-$line',
                        'actor': 'guide',
                        'text': '이야기 ${line + 1}',
                        'screen_anchor': [160, 160],
                      },
                      source: const Size(320, 240),
                      onAdvance: () => setState(() => line++),
                      onSave: () => saves++,
                    )
                  : GamePopup(text: '일반 대화', onClose: () {}),
            ),
          ),
        );
        for (var i = 0; i < 3; i++) {
          expect(find.text('이야기 ${i + 1}'), findsOneWidget);
          await tester.tap(find.byKey(const Key('bubble-save')));
          expect(line, i);
          expect(saves, i + 1);
          await tester.tapAt(Offset(size.width / 2, size.height - 24));
          await tester.pumpAndSettle();
          expect(tester.takeException(), isNull);
        }
        expect(find.text('일반 대화'), findsOneWidget);
        expect(find.text('MESSAGE / 대화'), findsOneWidget);
      },
    );
  }
}
