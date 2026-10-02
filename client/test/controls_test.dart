import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ge4g_client/controls.dart';
import 'package:ge4g_client/control_store.dart';
import 'package:ge4g_client/touch_controls.dart';
import 'package:flutter/material.dart';

String get layouts => File('assets/default_layouts.json').readAsStringSync();
String get bindings => File('assets/default_bindings.json').readAsStringSync();
GameControls defaults() =>
    GameControls.parse(layouts, bindings, 'demo.basement');
void main() {
  test(
    'simultaneous touches, keyboard and joystick merge; release is per pointer',
    () {
      final router = InputRouter(defaults());
      router.button('z', 1, true);
      router.button('z', 2, true);
      router.button('x', 3, true);
      router.joystick(1, -.8);
      router.key('KeyC', true);
      expect(router.input['interact'], true);
      expect(router.input['right'], true);
      expect(router.input['up'], true);
      expect(router.input['actions'], ['c', 'x']);
      router.button('z', 1, false);
      expect(router.input['interact'], true);
      router.button('z', 2, false);
      expect(router.input['interact'], false);
      router.activate(defaults());
      expect(router.input['actions'], isEmpty);
      expect(router.input['right'], false);
      router.dispose();
    },
  );
  test('strict schemas reject mismatched games/profiles/actions and retain semantics', () {
    expect(
      () => GameControls.parse(layouts, bindings, 'another.game'),
      throwsFormatException,
    );
    final data = jsonDecode(layouts);
    data['active_profile'] = 'missing';
    expect(
      () => GameControls.parse(jsonEncode(data), bindings, 'demo.basement'),
      throwsFormatException,
    );
    final bad = jsonDecode(bindings);
    bad['profiles']['default']['buttons']['z'] = ['bad action'];
    expect(
      () => GameControls.parse(layouts, jsonEncode(bad), 'demo.basement'),
      throwsFormatException,
    );
  });
  test('profile selection, external JSON reload, invalid retention and game isolation persist', () async {
    final root = Directory.systemTemp.createTempSync('ge4g-controls-');
    final store = ControlStore(Directory('${root.path}/one'), 'demo.basement');
    final other = ControlStore(Directory('${root.path}/two'), 'demo.other');
    try {
      await store.initialize(layouts, bindings);
      await other.initialize(
        layouts,
        bindings.replaceAll('demo.basement', 'demo.other'),
      );
      await store.select('left_handed');
      expect(store.current.activeId, 'left_handed');
      expect(other.current.activeId, 'default');
      final valid = store.current;
      await store.layoutsFile.writeAsString('{bad');
      await store.reload();
      expect(identical(store.current, valid), true);
      expect(store.error, isNotNull);
      final edit = jsonDecode(valid.bindingsText);
      edit['profiles']['left_handed']['buttons']['z'] = ['space'];
      await store.layoutsFile.writeAsString(valid.layoutsText);
      await store.bindingsFile.writeAsString(jsonEncode(edit));
      await store.reload();
      expect(store.error, isNull);
      expect(store.current.activeBindings.buttons['z'], ['space']);
      store.dispose();
      final restored = ControlStore(
        Directory('${root.path}/one'),
        'demo.basement',
      );
      await restored.initialize(layouts, bindings);
      expect(restored.current.activeId, 'left_handed');
      expect(restored.current.activeBindings.buttons['z'], ['space']);
      restored.dispose();
    } finally {
      other.dispose();
      root.deleteSync(recursive: true);
    }
  });
  testWidgets(
    'real touch widgets support button chords and joystick cancellation',
    (tester) async {
      final router = InputRouter(defaults());
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              width: 600,
              height: 400,
              child: TouchControls(router: router),
            ),
          ),
        ),
      );
      final z = await tester.startGesture(
        tester.getCenter(find.text('Z')),
        pointer: 11,
      );
      final x = await tester.startGesture(
        tester.getCenter(find.text('X')),
        pointer: 12,
      );
      expect(router.input['interact'], true);
      expect(router.input['actions'], ['x']);
      await z.cancel();
      expect(router.input['interact'], false);
      expect(router.input['actions'], ['x']);
      await x.up();
      expect(router.input['actions'], isEmpty);
      final stick = find.byType(Joystick);
      final center = tester.getCenter(stick);
      final move = await tester.startGesture(
        center + const Offset(30, 0),
        pointer: 13,
      );
      expect(router.input['right'], true);
      final second = await tester.startGesture(
        center - const Offset(30, 0),
        pointer: 14,
      );
      expect(router.input['right'], true); // first pointer owns the stick
      await move.cancel();
      expect(router.input['right'], false);
      await second.up();
      await tester.pumpWidget(const SizedBox());
      router.dispose();
    },
  );
}
