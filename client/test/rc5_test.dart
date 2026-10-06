import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ge4g_client/cutscene_bubble.dart';
import 'package:ge4g_client/game_popup.dart';

void main() {
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
