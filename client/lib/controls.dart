import 'dart:convert';

import 'package:flutter/foundation.dart';

Map<String, dynamic> object(Object? value, String field) {
  if (value is! Map<String, dynamic>) {
    throw FormatException('$field must be an object');
  }
  return value;
}

void fields(Map<String, dynamic> value, Set<String> allowed, String field) {
  final unknown = value.keys.toSet().difference(allowed);
  if (unknown.isNotEmpty) {
    throw FormatException('$field has unknown fields: ${unknown.join(', ')}');
  }
}

String text(Object? value, String field, {int max = 96}) {
  if (value is! String || value.isEmpty || value.length > max) {
    throw FormatException('$field requires 1..$max characters');
  }
  return value;
}

String identifier(Object? value, String field) {
  final result = text(value, field);
  if (!RegExp(r'^[a-z0-9][a-z0-9_.-]{0,95}$').hasMatch(result)) {
    throw FormatException('$field must be a stable lowercase identifier');
  }
  return result;
}

double number(Object? value, String field, double min, double max) {
  if (value is! num ||
      !value.toDouble().isFinite ||
      value < min ||
      value > max) {
    throw FormatException('$field must be between $min and $max');
  }
  return value.toDouble();
}

void version(Map<String, dynamic> value, String field) {
  if (value['schema_version'] != 1) {
    throw FormatException(
      '$field: unsupported schema_version ${value['schema_version']} (expected 1)',
    );
  }
}

class TouchButton {
  final String id, label;
  final double x, y, width, height;
  TouchButton.fromJson(Map<String, dynamic> json)
    : id = identifier(json['id'], 'button.id'),
      label = text(json['label'], 'button.label', max: 12),
      x = number(json['x'], 'button.x', 0, 1),
      y = number(json['y'], 'button.y', 0, 1),
      width = number(json['width'], 'button.width', .1, .6),
      height = number(json['height'], 'button.height', .1, .6) {
    fields(json, {'id', 'label', 'x', 'y', 'width', 'height'}, 'button');
  }
  Map<String, dynamic> toJson() => {
    'id': id,
    'label': label,
    'x': x,
    'y': y,
    'width': width,
    'height': height,
  };
}

class JoystickLayout {
  final double x, y, size, deadZone;
  final bool cardinal;
  JoystickLayout.fromJson(Map<String, dynamic> json)
    : x = number(json['x'], 'joystick.x', 0, 1),
      y = number(json['y'], 'joystick.y', 0, 1),
      size = number(json['size'], 'joystick.size', .2, .65),
      deadZone = number(json['dead_zone'], 'joystick.dead_zone', 0, .8),
      cardinal = json['cardinal'] == true {
    fields(json, {'x', 'y', 'size', 'dead_zone', 'cardinal'}, 'joystick');
    if (json.containsKey('cardinal') && json['cardinal'] is! bool) {
      throw const FormatException('joystick.cardinal must be boolean');
    }
  }
  Map<String, dynamic> toJson() => {
    'x': x,
    'y': y,
    'size': size,
    'dead_zone': deadZone,
    if (cardinal) 'cardinal': true,
  };
}

class TouchProfile {
  final String id, name;
  final JoystickLayout joystick;
  final List<TouchButton> buttons;
  TouchProfile.fromJson(Map<String, dynamic> json, {bool allowEmpty = false})
    : id = identifier(json['id'], 'profile.id'),
      name = text(json['name'], 'profile.name'),
      joystick = JoystickLayout.fromJson(
        object(json['joystick'], 'profile.joystick'),
      ),
      buttons = _buttons(json['buttons'], allowEmpty) {
    fields(json, {'id', 'name', 'joystick', 'buttons'}, 'profile');
  }
  static List<TouchButton> _buttons(Object? value, bool allowEmpty) {
    if (value is! List || (!allowEmpty && value.isEmpty) || value.length > 16) {
      throw FormatException(
        'profile.buttons needs ${allowEmpty ? 0 : 1}..16 buttons',
      );
    }
    final buttons = value
        .map((b) => TouchButton.fromJson(object(b, 'button')))
        .toList(growable: false);
    if (buttons.map((b) => b.id).toSet().length != buttons.length) {
      throw const FormatException('duplicate button id');
    }
    return List.unmodifiable(buttons);
  }

  Map<String, dynamic> toJson() => {
    'id': id,
    'name': name,
    'joystick': joystick.toJson(),
    'buttons': buttons.map((b) => b.toJson()).toList(),
  };
}

class ProfileBindings {
  final Map<String, List<String>> joystick, buttons, keys;
  ProfileBindings.fromJson(Map<String, dynamic> json)
    : joystick = _actions(json['joystick'], 'bindings.joystick'),
      buttons = _actions(json['buttons'], 'bindings.buttons'),
      keys = _actions(json['keys'], 'bindings.keys') {
    fields(json, {'joystick', 'buttons', 'keys'}, 'profile bindings');
    if (!setEquals(joystick.keys.toSet(), {'left', 'right', 'up', 'down'})) {
      throw const FormatException('joystick bindings need left/right/up/down');
    }
    for (final key in keys.keys) {
      if (!RegExp(
        r'^(Key[A-Z]|Digit[0-9]|Arrow(Left|Right|Up|Down)|Space|Enter|Tab|ShiftLeft|ControlLeft|AltLeft)$',
      ).hasMatch(key)) {
        throw FormatException('unsupported keyboard code $key');
      }
    }
  }
  static Map<String, List<String>> _actions(Object? value, String field) {
    final json = object(value, field);
    if (json.length > 64) throw FormatException('$field exceeds 64 bindings');
    return Map.unmodifiable(
      json.map<String, List<String>>((key, value) {
        if (value is! List || value.length > 8) {
          throw FormatException(
            '$field.$key must be an array with up to 8 actions',
          );
        }
        final actions = value
            .map<String>((a) {
              final name = text(a, '$field.$key action', max: 64);
              if (!RegExp(r'^[A-Za-z0-9_.-]+$').hasMatch(name)) {
                throw FormatException('invalid action $name');
              }
              return name;
            })
            .toSet()
            .toList(growable: false);
        return MapEntry(key, List<String>.unmodifiable(actions));
      }),
    );
  }

  Map<String, dynamic> toJson() => {
    'joystick': joystick,
    'buttons': buttons,
    'keys': keys,
  };
}

class GameControls {
  final String gameId, activeId;
  final int layoutVersion;
  final List<TouchProfile> profiles;
  final Map<String, ProfileBindings> bindings;
  GameControls._(
    this.gameId,
    this.activeId,
    this.profiles,
    this.bindings,
    this.layoutVersion,
  );
  factory GameControls.parse(
    String layoutsText,
    String bindingsText,
    String gameId,
  ) {
    if (layoutsText.length > 1024 * 1024 || bindingsText.length > 1024 * 1024) {
      throw const FormatException('control JSON exceeds 1 MiB');
    }
    final layouts = object(jsonDecode(layoutsText), 'layouts');
    final mappings = object(jsonDecode(bindingsText), 'bindings');
    if (![1, 2].contains(layouts['schema_version'])) {
      throw const FormatException('unsupported layouts schema_version');
    }
    version(mappings, 'bindings');
    fields(layouts, {
      'schema_version',
      'active_profile',
      'profiles',
    }, 'layouts');
    fields(mappings, {'schema_version', 'game_id', 'profiles'}, 'bindings');
    if (mappings['game_id'] != gameId) {
      throw FormatException(
        'bindings belong to ${mappings['game_id']}, expected $gameId',
      );
    }
    final list = layouts['profiles'];
    if (list is! List || list.isEmpty || list.length > 16) {
      throw const FormatException('layouts needs 1..16 profiles');
    }
    final profiles = list
        .map(
          (p) => TouchProfile.fromJson(
            object(p, 'profile'),
            allowEmpty: layouts['schema_version'] == 2,
          ),
        )
        .toList(growable: false);
    final ids = profiles.map((p) => p.id).toSet();
    if (ids.length != profiles.length) {
      throw const FormatException('duplicate profile id');
    }
    final active = identifier(layouts['active_profile'], 'active_profile');
    if (!ids.contains(active)) {
      throw FormatException('unknown active profile $active');
    }
    final bindings = object(mappings['profiles'], 'bindings.profiles').map(
      (id, value) => MapEntry(
        id,
        ProfileBindings.fromJson(object(value, 'bindings profile')),
      ),
    );
    if (!setEquals(ids, bindings.keys.toSet())) {
      throw const FormatException('layout and mapping profile ids must match');
    }
    for (final profile in profiles) {
      final binding = bindings[profile.id]!;
      final actions =
          [
            ...binding.buttons.values,
            ...binding.joystick.values,
            ...binding.keys.values,
          ].expand((a) => a).toSet().difference({
            'left',
            'right',
            'up',
            'down',
            'interact',
          });
      if (actions.length > 32) {
        throw FormatException('${profile.id}: exceeds 32 named actions');
      }
      if (!setEquals(
        profile.buttons.map((b) => b.id).toSet(),
        bindings[profile.id]!.buttons.keys.toSet(),
      )) {
        throw FormatException(
          '${profile.id}: layout and mapping button ids must match',
        );
      }
    }
    return GameControls._(
      gameId,
      active,
      List.unmodifiable(profiles),
      Map.unmodifiable(bindings),
      layouts['schema_version'] as int,
    );
  }
  TouchProfile get active => profiles.firstWhere((p) => p.id == activeId);
  ProfileBindings get activeBindings => bindings[activeId]!;
  Map<String, dynamic> get layoutsJson => {
    'schema_version': layoutVersion,
    'active_profile': activeId,
    'profiles': profiles.map((p) => p.toJson()).toList(),
  };
  Map<String, dynamic> get bindingsJson => {
    'schema_version': 1,
    'game_id': gameId,
    'profiles': bindings.map((k, v) => MapEntry(k, v.toJson())),
  };
  String get layoutsText =>
      const JsonEncoder.withIndent('  ').convert(layoutsJson);
  String get bindingsText =>
      const JsonEncoder.withIndent('  ').convert(bindingsJson);
}

/// Pointer ownership, key state and joystick state merge into one normalized input.
class InputRouter extends ChangeNotifier {
  bool _stickHorizontal = true;
  GameControls controls;
  final Map<String, Set<int>> _buttons = {};
  final Set<String> _keys = {};
  double stickX = 0, stickY = 0;
  InputRouter(this.controls);
  bool isPressed(String id) => _buttons[id]?.isNotEmpty ?? false;
  void button(String id, int pointer, bool down) {
    if (down) {
      (_buttons[id] ??= {}).add(pointer);
    } else {
      _buttons[id]?.remove(pointer);
    }
    notifyListeners();
  }

  void key(String code, bool down) {
    down ? _keys.add(code) : _keys.remove(code);
    notifyListeners();
  }

  void joystick(double x, double y) {
    final fresh =
        stickX.abs() <= controls.active.joystick.deadZone &&
        stickY.abs() <= controls.active.joystick.deadZone;
    stickX = x.clamp(-1, 1);
    stickY = y.clamp(-1, 1);
    if (controls.active.joystick.cardinal) {
      if (fresh) {
        _stickHorizontal = stickX.abs() >= stickY.abs();
      } else if (_stickHorizontal && stickY.abs() > stickX.abs() * 1.12) {
        _stickHorizontal = false;
      } else if (!_stickHorizontal && stickX.abs() > stickY.abs() * 1.12) {
        _stickHorizontal = true;
      }
    }
    notifyListeners();
  }

  void clear() {
    _buttons.clear();
    _keys.clear();
    stickX = 0;
    stickY = 0;
    notifyListeners();
  }

  void activate(GameControls candidate) {
    _buttons.clear();
    _keys.clear();
    stickX = 0;
    stickY = 0;
    controls = candidate;
    notifyListeners();
  }

  Map<String, dynamic> get input {
    final binding = controls.activeBindings;
    final actions = <String>{};
    for (final entry in _buttons.entries) {
      if (entry.value.isNotEmpty) {
        actions.addAll(binding.buttons[entry.key] ?? const []);
      }
    }
    for (final key in _keys) {
      actions.addAll(binding.keys[key] ?? const []);
    }
    final dead = controls.active.joystick.deadZone;
    final sx = controls.active.joystick.cardinal && !_stickHorizontal
        ? 0.0
        : stickX;
    final sy = controls.active.joystick.cardinal && _stickHorizontal
        ? 0.0
        : stickY;
    if (sx < -dead) actions.addAll(binding.joystick['left']!);
    if (sx > dead) actions.addAll(binding.joystick['right']!);
    if (sy < -dead) actions.addAll(binding.joystick['up']!);
    if (sy > dead) actions.addAll(binding.joystick['down']!);
    const builtins = {'left', 'right', 'up', 'down', 'interact'};
    final named = actions.difference(builtins).toList()..sort();
    return {
      for (final name in builtins) name: actions.contains(name),
      'actions': named,
    };
  }
}
