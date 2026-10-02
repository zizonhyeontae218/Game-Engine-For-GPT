import 'dart:convert';

import 'package:flutter/material.dart';

import 'control_store.dart';
import 'controls.dart';
import 'touch_controls.dart';

class ControlEditor extends StatefulWidget {
  final ControlStore store;
  const ControlEditor({super.key, required this.store});
  @override
  State<ControlEditor> createState() => _ControlEditorState();
}

class _ControlEditorState extends State<ControlEditor> {
  late final TextEditingController layouts, bindings;
  String? error;
  bool busy = false;
  @override
  void initState() {
    super.initState();
    layouts = TextEditingController(text: widget.store.current.layoutsText);
    bindings = TextEditingController(text: widget.store.current.bindingsText);
  }

  @override
  void dispose() {
    layouts.dispose();
    bindings.dispose();
    super.dispose();
  }

  void refresh() {
    layouts.text = widget.store.current.layoutsText;
    bindings.text = widget.store.current.bindingsText;
    setState(() => error = null);
  }

  Future<void> apply() async {
    setState(() {
      busy = true;
      error = null;
    });
    try {
      await widget.store.apply(layouts.text, bindings.text);
    } catch (failure) {
      if (mounted) setState(() => error = '$failure');
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> change(
    String type,
    String? button,
    String field,
    double value,
  ) async {
    try {
      final current = GameControls.parse(
        layouts.text,
        bindings.text,
        widget.store.gameId,
      );
      final json = current.layoutsJson;
      final profile = (json['profiles'] as List)
          .cast<Map<String, dynamic>>()
          .firstWhere((p) => p['id'] == current.activeId);
      if (type == 'joystick') {
        (profile['joystick'] as Map)[field] = value;
      } else {
        ((profile['buttons'] as List).cast<Map<String, dynamic>>().firstWhere(
          (b) => b['id'] == button,
        ))[field] = value;
      }
      layouts.text = const JsonEncoder.withIndent('  ').convert(json);
      await apply();
    } catch (failure) {
      if (mounted) setState(() => error = '$failure');
    }
  }

  Widget slider(
    String label,
    double value,
    double min,
    double max,
    void Function(double) end,
  ) => _LiveSlider(label: label, value: value, min: min, max: max, onEnd: end);
  @override
  Widget build(BuildContext context) {
    final current = widget.store.current, active = current.active;
    return Material(
      color: paper,
      child: SafeArea(
        child: ListView(
          padding: EdgeInsets.fromLTRB(
            20,
            20,
            20,
            20 + MediaQuery.viewInsetsOf(context).bottom,
          ),
          children: [
            Row(
              children: [
                const Expanded(
                  child: Text(
                    'CONTROL LAB / 실시간 편집',
                    style: TextStyle(fontSize: 20, fontWeight: FontWeight.w900),
                  ),
                ),
                IconButton(
                  tooltip: '닫기',
                  onPressed: () => Navigator.pop(context),
                  icon: const Icon(Icons.close),
                ),
              ],
            ),
            Text('${widget.store.gameId} · ${active.name}'),
            const SizedBox(height: 10),
            const Text(
              '게임은 계속 실행됩니다. 슬라이더를 놓거나 JSON을 적용하면 즉시 반영됩니다. 편집 중 게임 입력은 해제됩니다.',
            ),
            slider(
              '조이스틱 X',
              active.joystick.x,
              0,
              1,
              (v) => change('joystick', null, 'x', v),
            ),
            slider(
              '조이스틱 Y',
              active.joystick.y,
              0,
              1,
              (v) => change('joystick', null, 'y', v),
            ),
            slider(
              '조이스틱 크기',
              active.joystick.size,
              .2,
              .65,
              (v) => change('joystick', null, 'size', v),
            ),
            slider(
              '데드 존',
              active.joystick.deadZone,
              0,
              .8,
              (v) => change('joystick', null, 'dead_zone', v),
            ),
            for (final button in active.buttons)
              ExpansionTile(
                key: ValueKey('${current.hashCode}/${button.id}'),
                title: Text(
                  '${button.label} → ${current.activeBindings.buttons[button.id]!.join(', ')}',
                ),
                children: [
                  slider(
                    '${button.label} X',
                    button.x,
                    0,
                    1,
                    (v) => change('button', button.id, 'x', v),
                  ),
                  slider(
                    '${button.label} Y',
                    button.y,
                    0,
                    1,
                    (v) => change('button', button.id, 'y', v),
                  ),
                  slider(
                    '너비',
                    button.width,
                    .1,
                    .6,
                    (v) => change('button', button.id, 'width', v),
                  ),
                  slider(
                    '높이',
                    button.height,
                    .1,
                    .6,
                    (v) => change('button', button.id, 'height', v),
                  ),
                  TextFormField(
                    initialValue: current.activeBindings.buttons[button.id]!
                        .join(', '),
                    decoration: const InputDecoration(
                      labelText: '액션 매핑 (쉼표로 구분)',
                      helperText: 'left, right, up, down, interact 또는 게임 액션',
                    ),
                    onFieldSubmitted: (value) async {
                      try {
                        final json = object(
                          jsonDecode(bindings.text),
                          'bindings',
                        );
                        (json['profiles'][current.activeId]['buttons']
                            as Map)[button.id] = value
                            .split(',')
                            .map((s) => s.trim())
                            .where((s) => s.isNotEmpty)
                            .toList();
                        bindings.text = const JsonEncoder.withIndent('  ')
                            .convert(json);
                        await apply();
                      } catch (failure) {
                        if (mounted) setState(() => error = '$failure');
                      }
                    },
                  ),
                ],
              ),
            const SizedBox(height: 16),
            const Text(
              'LAYOUTS.JSON / 배치·프로필',
              style: TextStyle(fontWeight: FontWeight.bold),
            ),
            TextField(
              key: const Key('layouts-json'),
              controller: layouts,
              minLines: 5,
              maxLines: 12,
              autocorrect: false,
              enableSuggestions: false,
              style: const TextStyle(fontFamily: 'monospace', fontSize: 12),
            ),
            const SizedBox(height: 16),
            const Text(
              'BINDINGS.JSON / 이 게임의 버튼·키 매핑',
              style: TextStyle(fontWeight: FontWeight.bold),
            ),
            TextField(
              key: const Key('bindings-json'),
              controller: bindings,
              minLines: 5,
              maxLines: 12,
              autocorrect: false,
              enableSuggestions: false,
              style: const TextStyle(fontFamily: 'monospace', fontSize: 12),
            ),
            if (error != null)
              Padding(
                padding: const EdgeInsets.symmetric(vertical: 12),
                child: Text(
                  '적용 실패 · 기존 설정 유지\n$error',
                  style: const TextStyle(color: Colors.red),
                ),
              ),
            Wrap(
              spacing: 12,
              runSpacing: 12,
              children: [
                FilledButton(
                  key: const Key('apply-controls'),
                  onPressed: busy ? null : apply,
                  child: Text(busy ? '저장 중…' : 'JSON 적용'),
                ),
                OutlinedButton(
                  onPressed: refresh,
                  child: const Text('현재 설정 다시 읽기'),
                ),
                OutlinedButton(
                  onPressed: () async {
                    try {
                      final json = object(jsonDecode(layouts.text), 'layouts');
                      final mapping = object(
                        jsonDecode(bindings.text),
                        'bindings',
                      );
                      final profiles = json['profiles'] as List;
                      final copy = object(
                        jsonDecode(
                          jsonEncode(
                            profiles.firstWhere(
                              (p) => p['id'] == json['active_profile'],
                            ),
                          ),
                        ),
                        'profile',
                      );
                      var count = profiles.length + 1;
                      var id = 'custom_$count';
                      while (profiles.any((p) => p['id'] == id)) {
                        id = 'custom_${++count}';
                      }
                      copy['id'] = id;
                      copy['name'] = 'CUSTOM $count';
                      profiles.add(copy);
                      mapping['profiles'][id] = jsonDecode(
                        jsonEncode(mapping['profiles'][json['active_profile']]),
                      );
                      json['active_profile'] = id;
                      layouts.text = jsonEncode(json);
                      bindings.text = jsonEncode(mapping);
                      await apply();
                      refresh();
                    } catch (failure) {
                      if (mounted) setState(() => error = '$failure');
                    }
                  },
                  child: const Text('프로필 복제'),
                ),
              ],
            ),
            const SizedBox(height: 20),
            SelectableText(
              '배치: ${widget.store.layoutsFile.path}\n매핑: ${widget.store.bindingsFile.path}',
              style: const TextStyle(fontSize: 11),
            ),
          ],
        ),
      ),
    );
  }
}

class _LiveSlider extends StatefulWidget {
  final String label;
  final double value, min, max;
  final void Function(double) onEnd;
  const _LiveSlider({
    required this.label,
    required this.value,
    required this.min,
    required this.max,
    required this.onEnd,
  });
  @override
  State<_LiveSlider> createState() => _LiveSliderState();
}

class _LiveSliderState extends State<_LiveSlider> {
  late double value;
  @override
  void initState() {
    super.initState();
    value = widget.value;
  }

  @override
  void didUpdateWidget(_LiveSlider old) {
    super.didUpdateWidget(old);
    if (old.value != widget.value) value = widget.value;
  }

  @override
  Widget build(BuildContext context) => Row(
    children: [
      SizedBox(
        width: 125,
        child: Text('${widget.label}\n${value.toStringAsFixed(2)}'),
      ),
      Expanded(
        child: Slider(
          value: value,
          min: widget.min,
          max: widget.max,
          onChanged: (v) => setState(() => value = v),
          onChangeEnd: widget.onEnd,
        ),
      ),
    ],
  );
}
