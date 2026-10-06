import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ge4g_client/main.dart';
import 'package:ge4g_client/player.dart';

class FakeEngine implements EngineBridge {
  @override
  Map<String, dynamic> request(Map<String, dynamic> value) => {'ok': true};
  @override
  Uint8List frame(int session, int length) => Uint8List(length);
}

void main() {
  testWidgets('mobile library starts with an import action and GE4G branding', (
    tester,
  ) async {
    final root = Directory.systemTemp.createTempSync('ge4g-widget-');
    await tester.runAsync(() async {
      await tester.pumpWidget(
        GE4GApp(engine: FakeEngine(), dataDirectory: root),
      );
      await Future<void>.delayed(const Duration(milliseconds: 150));
    });
    await tester.pump();
    expect(find.text('GE4G'), findsOneWidget);
    expect(find.byKey(const Key('import-game')), findsOneWidget);
    expect(find.byKey(const Key('manage-games')), findsOneWidget);
    expect(find.text('YOUR GAMES / 보관함'), findsOneWidget);
    await tester.pumpWidget(const SizedBox());
    root.deleteSync(recursive: true);
  });
}
