import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ge4g_client/pentomino_screen.dart';
import 'package:ge4g_native/p5_native.dart';

final _config = <String, dynamic>{
  'fighters': [
    {'id': 'blue', 'team': 0, 'hp': 20, 'resource': 6},
    {'id': 'red', 'team': 1, 'hp': 20, 'resource': 6},
  ],
  'actions': [
    {'id': 'strike', 'target': 'enemy', 'cost': 0},
    {'id': 'heal', 'target': 'ally', 'cost': 2},
  ],
};

class _Bridge implements PentominoBridge {
  final commands = <Map<String, dynamic>>[];
  bool ended = false;
  @override
  dynamic request(Map<String, dynamic> command) {
    commands.add(command);
    switch (command['op']) {
      case 'create':
        return {'session': 10};
      case 'frame':
        return {
          'width': 2,
          'height': 1,
          'rgba': [48, 144, 255, 255, 255, 80, 72, 255],
        };
      case 'act':
        ended = true;
        return _observation();
      case 'observe':
        return _observation();
      case 'close':
        return null;
      default:
        throw StateError('Unexpected command');
    }
  }

  Map<String, dynamic> _observation() => {
    'battle': {
      'turn': 0,
      'winner': ended ? 0 : -1,
      'command_count': ended ? 1 : 0,
      'last_command_id': ended ? 1 : 0,
      'fighters': [
        for (final hp in [20, ended ? 0 : 20])
          {
            'hp': hp,
            'resource': 6,
            'poison': {'power': 0, 'turns': 0},
            'regen': {'power': 0, 'turns': 0},
          },
      ],
    },
  };
}

void main() {
  testWidgets('selection forwards one native command; native outcome ends UI', (
    tester,
  ) async {
    final bridge = _Bridge();
    await tester.pumpWidget(
      MaterialApp(
        home: PentominoScreen(bridge: bridge, config: _config),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('blue의 차례'), findsOneWidget);
    await tester.ensureVisible(find.byKey(const Key('pentomino-execute')));
    await tester.tap(find.byKey(const Key('pentomino-execute')));
    await tester.pumpAndSettle();
    expect(bridge.commands.singleWhere((c) => c['op'] == 'act')['command'], {
      'id': 1,
      'actor': 0,
      'action': 0,
      'target': 1,
    });
    expect(find.text('팀 0 승리'), findsOneWidget);
    expect(
      tester
          .widget<FilledButton>(find.byKey(const Key('pentomino-execute')))
          .onPressed,
      isNull,
    );
    await tester.pumpWidget(const SizedBox.shrink());
    expect(bridge.commands.last['op'], 'close');
    expect(tester.takeException(), isNull);
  });

  testWidgets('narrow and wide layouts stay usable without overflow', (
    tester,
  ) async {
    final key = GlobalKey();
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    for (final size in [const Size(390, 844), const Size(1100, 800)]) {
      tester.view.physicalSize = size;
      await tester.pumpWidget(
        MaterialApp(
          home: RepaintBoundary(
            key: key,
            child: PentominoScreen(
              key: ValueKey(size),
              bridge: _Bridge(),
              config: _config,
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('blue의 차례'), findsOneWidget);
      expect(tester.takeException(), isNull);
      if (Platform.environment['GE4G_LAYOUT_CAPTURE'] case final String path) {
        final boundary =
            key.currentContext!.findRenderObject() as RenderRepaintBoundary;
        final image = await boundary.toImage(pixelRatio: 1);
        final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
        image.dispose();
        await Directory(path).create(recursive: true);
        await File('$path/layout-${size.width.toInt()}.png')
            .writeAsBytes(bytes!.buffer.asUint8List());
      }
      await tester.pumpWidget(const SizedBox.shrink());
    }
  });
}
