import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ge4g_client/cutscene_bubble.dart';
import 'package:ge4g_client/game_library.dart';
import 'package:ge4g_client/game_manager.dart';
import 'package:ge4g_client/player.dart';

import 'library_test.dart' as fixture;

class UnusedEngine implements EngineBridge {
  @override
  Map<String, dynamic> request(Map<String, dynamic> request) => {'ok': true};
  @override
  Uint8List frame(int session, int length) => Uint8List(length);
}

Future<void> settleIo(WidgetTester tester) async {
  // Called inside one real-async scope: storage and callback futures share it.
  // Finish menu transitions before waiting for the manager's IO.
  await tester.pump(const Duration(milliseconds: 500));
  for (var attempt = 0; attempt < 200; attempt++) {
    await tester.pump(const Duration(milliseconds: 25));
    await Future<void>.delayed(const Duration(milliseconds: 25));
    if (find.byType(LinearProgressIndicator).evaluate().isEmpty) break;
  }
  await tester.pump();
  expect(find.byType(LinearProgressIndicator), findsNothing);
  await tester.pumpAndSettle();
}

void main() {
  for (final size in [const Size(390, 844), const Size(800, 360)]) {
    for (final speaker in ['안내인', null]) {
      testWidgets(
        'bubble speaker $speaker at $size is semantic, never invented',
        (tester) async {
          tester.view.physicalSize = size;
          tester.view.devicePixelRatio = 1;
          addTearDown(tester.view.resetPhysicalSize);
          addTearDown(tester.view.resetDevicePixelRatio);
          await tester.pumpWidget(
            MaterialApp(
              home: CutsceneBubble(
                bubble: {
                  'id': 'line',
                  'actor': 'internal_entity_id',
                  'text': '여기가 바람항 공방이야.',
                  'speaker': ?speaker,
                  'screen_anchor': [160, 160],
                },
                source: const Size(320, 240),
                onAdvance: () {},
                onSave: () {},
              ),
            ),
          );
          await tester.pumpAndSettle();
          expect(
            find.byKey(const Key('bubble-speaker')),
            speaker == null ? findsNothing : findsOneWidget,
          );
          if (speaker != null) {
            expect(find.text(speaker), findsOneWidget);
          }
          expect(find.text('이야기'), findsNothing);
          expect(find.text('internal_entity_id'), findsNothing);
          expect(tester.takeException(), isNull);
        },
      );
    }
  }
  for (final full in [false, true]) {
    testWidgets(
      'manager reorder and explicit ${full ? 'full' : 'normal'} delete confirmation',
      (tester) async {
        await tester.runAsync(() async {
          final root = Directory.systemTemp.createTempSync('ge4g-manager-');
          final library = GameLibrary(root, (_) {});
          final player = Player(UnusedEngine(), library);
          await library.importBytes(fixture.package(id: 'a', name: '게임 A'));
          await library.importBytes(fixture.package(id: 'b', name: '게임 B'));
          final save = library.save('b')..parent.createSync(recursive: true);
          save.writeAsStringSync('progress');
          library.controls('b').createSync(recursive: true);
          await tester.pumpWidget(
            MaterialApp(
              home: GameManager(library: library, player: player),
            ),
          );
          await Future<void>.delayed(const Duration(milliseconds: 150));
          await settleIo(tester);
          expect(find.text('GAME MANAGER / 게임 관리'), findsOneWidget);
          expect(find.textContaining('b\n'), findsOneWidget);
          await tester.tap(find.byKey(const ValueKey('manage-b')));
          await tester.pumpAndSettle();
          await tester.tap(find.text('위로 이동'));
          await settleIo(tester);
          expect((await library.list()).map((g) => g.id), ['b', 'a']);
          await tester.tap(find.byKey(const ValueKey('manage-b')));
          await tester.pumpAndSettle();
          await tester.tap(find.text(full ? '게임 및 데이터 모두 삭제' : '게임 삭제'));
          await tester.pumpAndSettle();
          expect(save.existsSync(), true);
          expect(
            find.textContaining(full ? '보관된 이전 저장' : '저장 데이터와 조작 설정은 보관'),
            findsOneWidget,
          );
          await tester.tap(find.text('취소'));
          await tester.pumpAndSettle();
          expect((await library.list()).length, 2);
          await tester.tap(find.byKey(const ValueKey('manage-b')));
          await tester.pumpAndSettle();
          await tester.tap(find.text(full ? '게임 및 데이터 모두 삭제' : '게임 삭제'));
          await tester.pumpAndSettle();
          await tester.tap(find.byKey(const Key('confirm-delete')));
          await settleIo(tester);
          expect(find.text('게임 B'), findsNothing);
          expect((await library.list()).single.id, 'a');
          expect(save.existsSync(), !full);
          expect(library.controls('b').existsSync(), !full);
          expect(tester.takeException(), isNull);
          await tester.pumpWidget(const SizedBox());
          player.dispose();
          root.deleteSync(recursive: true);
        });
      },
    );
  }
}
