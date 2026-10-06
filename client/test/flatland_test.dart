import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ge4g_client/game_library.dart';
import 'package:ge4g_client/main.dart';
import 'package:ge4g_client/manual_orientation.dart';
import 'package:ge4g_client/player.dart';
import 'package:ge4g_client/game_viewport.dart';

class PlayEngine implements EngineBridge {
  int releases = 0;
  @override
  Map<String, dynamic> request(Map<String, dynamic> value) {
    if (value['op'] == 'release') releases++;
    return {
      'ok': true,
      'session': 1,
      'width': 380,
      'height': 420,
      'frame_bytes': 638400,
      'tick': 0,
      'scene': 'maze',
      'state': <String, dynamic>{},
      'audio': [],
      'popup': {'id': 1, 'text': 'A separate dialogue popup'},
    };
  }

  @override
  Uint8List frame(int session, int length) => Uint8List(length);
}

void main() {
  test('frame fit uses all available space without orientation/control reservations', () {
    final rect = fittedGameRect(const Size(740, 316), const Size(380, 420));
    expect(rect.height, 316);
    expect(rect.width, closeTo(285.90476, .001));
    expect(rect.center.dx, 370);
    final portrait = fittedGameRect(const Size(360, 696), const Size(380, 420));
    expect(portrait.width, 360);
    expect(portrait.top, 0);
  });
  test(
    'orientation remains locked to one direction and changes only on toggle',
    () async {
      final calls = <List<DeviceOrientation>>[];
      final mode = ManualOrientation(
        apply: (value) async {
          calls.add(value);
        },
      );
      await mode.initialize();
      expect(mode.landscape, false);
      expect(calls.single, [DeviceOrientation.portraitUp]);
      await mode.toggle();
      expect(mode.landscape, true);
      expect(calls.last, [DeviceOrientation.landscapeLeft]);
      await mode.toggle();
      expect(mode.landscape, false);
      expect(calls.last, [DeviceOrientation.portraitUp]);
      mode.dispose();
    },
  );
  testWidgets(
    'FlatLand import, modal dismissal and manual layout survive window resizing',
    (tester) async {
      // Filesystem, session shutdown and decode callbacks need one real zone.
      await tester.runAsync(() async {
        final root = Directory.systemTemp.createTempSync('flatland-ui-');
        final engine = PlayEngine();
        final library = GameLibrary(root, (_) => {});
        await library.importFile(File('../dist/flatland-pacman.ge4g'));
        tester.view.devicePixelRatio = 1;
        tester.view.physicalSize = const Size(360, 740);
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        await tester.pumpWidget(GE4GApp(engine: engine, dataDirectory: root));
        await Future<void>.delayed(const Duration(milliseconds: 150));
        await tester.pump();
        await tester.ensureVisible(find.text('PLAY →'));
        await tester.pump();
        await tester.tap(find.text('PLAY →'));
        // Wait for controls IO and the first decoded frame, rather than a fixed
        // delay that may expire while the player correctly remains loading.
        for (var attempt = 0; attempt < 200; attempt++) {
          await Future<void>.delayed(const Duration(milliseconds: 25));
          await tester.pump();
          if (find.text('A separate dialogue popup').evaluate().isNotEmpty) {
            break;
          }
        }
        expect(find.text('A separate dialogue popup'), findsOneWidget);
        await tester.tap(find.byKey(const Key('close-game-popup')));
        await tester.pump();
        expect(find.text('A separate dialogue popup'), findsNothing);
        expect(find.byTooltip('가로모드'), findsOneWidget);
        expect(tester.getSize(find.byKey(const Key('game-frame'))).width, 360);
        expect(find.text('SPACE'), findsNothing);
        expect(find.text('Z'), findsNothing);
        expect(find.textContaining('ACTION:'), findsNothing);
        tester.view.physicalSize = const Size(740, 360);
        await tester.pump();
        expect(
          find.byTooltip('가로모드'),
          findsOneWidget,
        ); // Resize does not select landscape.
        final frame = tester.getSize(find.byKey(const Key('game-frame')));
        expect(frame.height, 316);
        expect(frame.width, closeTo(316 * 380 / 420, .01));
        final releases = engine.releases;
        await tester.tap(find.byKey(const Key('manual-rotation')));
        await tester.pump();
        expect(find.byTooltip('세로모드'), findsOneWidget);
        expect(engine.releases, greaterThan(releases));
        tester.view.physicalSize = const Size(360, 740);
        await tester.pump();
        expect(find.byTooltip('세로모드'), findsOneWidget);
        tester.platformDispatcher.textScaleFactorTestValue = 2;
        addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
        await tester.pump();
        expect(tester.takeException(), isNull);
        await tester.tap(find.byKey(const Key('play-settings')));
        await tester.pump(const Duration(milliseconds: 500));
        expect(find.byKey(const Key('profile-select')), findsOneWidget);
        expect(find.text('게임 기본 프리셋 적용'), findsOneWidget);
        await tester.pumpWidget(const SizedBox());
        root.deleteSync(recursive: true);
      });
    },
  );
}
