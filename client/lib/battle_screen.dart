import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/material.dart';

import 'game_viewport.dart';
import 'touch_controls.dart';

/// Dedicated presentation of the authoritative event battle, never a dialogue.
class BattleScreen extends StatefulWidget {
  final Map<String, dynamic> battle;
  final ui.Image? image;
  final Size source;
  final ValueChanged<String> onChoose;
  final VoidCallback onSave, onRotate;
  const BattleScreen({
    super.key,
    required this.battle,
    required this.image,
    required this.source,
    required this.onChoose,
    required this.onSave,
    required this.onRotate,
  });
  @override
  State<BattleScreen> createState() => _BattleScreenState();
}

class _BattleScreenState extends State<BattleScreen> {
  bool bag = false;
  String? selectedTarget;
  double healthWidth = 220;
  Widget health(Map f) {
    final hp = ((f['display_hp'] ?? f['hp']) as num).toDouble();
    final max = (f['hp_max'] as num).toDouble();
    return Container(
      constraints: BoxConstraints(maxWidth: healthWidth),
      decoration: BoxDecoration(
        color: paper,
        border: Border.all(color: ink, width: 2),
      ),
      padding: const EdgeInsets.all(8),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            '${f['name']}',
            style: const TextStyle(fontWeight: FontWeight.w900),
          ),
          const SizedBox(height: 4),
          LinearProgressIndicator(
            value: max > 0 ? (hp / max).clamp(0, 1) : 0,
            minHeight: 8,
            color: hp * 3 < max ? Colors.red : acid,
            backgroundColor: ink,
          ),
          Text('HP ${hp.toInt()} / ${max.toInt()}'),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final fighters = (widget.battle['fighters'] as List)
        .whereType<Map>()
        .toList();
    final heroes = fighters.where((f) => f['enemy'] != true).toList();
    final enemies = fighters.where((f) => f['enemy'] == true).toList();
    final options = (widget.battle['options'] as List)
        .whereType<Map>()
        .toList();
    final targets = enemies.where((f) => (f['hp'] as num) > 0).toList();
    final target = targets.any((f) => f['id'] == selectedTarget)
        ? selectedTarget
        : targets.firstOrNull?['id'];
    final result = widget.battle['result'];
    final locked = widget.battle['presentation_locked'] == true;
    final moves = options
        .where(
          (o) =>
              o['group'] != 'items' &&
              (o['target'] == null || o['target'] == target),
        )
        .toList();
    final items = options.where((o) => o['group'] == 'items').toList();
    Widget commands() => Container(
      color: paper,
      padding: const EdgeInsets.all(12),
      child: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(
              result == null
                  ? 'TURN ${widget.battle['turn']} / ${bag ? '가방' : '기술'}'
                  : result == true
                  ? 'VICTORY / 승리'
                  : 'DEFEAT / 패배',
              style: const TextStyle(fontWeight: FontWeight.w900),
            ),
            const SizedBox(height: 8),
            Text(locked ? '공격 연출 중…' : widget.battle['text'] as String? ?? ''),
            const Divider(color: ink, thickness: 2),
            if (targets.length > 1)
              Wrap(
                spacing: 6,
                children: [
                  for (final f in targets)
                    ChoiceChip(
                      label: Text('${f['name']}'),
                      selected: target == f['id'],
                      onSelected: (_) =>
                          setState(() => selectedTarget = f['id'] as String),
                    ),
                ],
              ),
            LayoutBuilder(
              builder: (context, size) => Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  for (final o in bag && result == null ? items : moves)
                    SizedBox(
                      width: size.maxWidth > 300
                          ? (size.maxWidth - 8) / 2
                          : size.maxWidth,
                      child: FilledButton(
                        key: Key('battle-${o['id']}'),
                        onPressed: locked
                            ? null
                            : () {
                                setState(() => bag = false);
                                widget.onChoose(o['id'] as String);
                              },
                        child: Padding(
                          padding: const EdgeInsets.symmetric(vertical: 8),
                          child: Text(
                            '${o['text']}${o['pp'] == null ? '' : '\nPP ${o['pp']}'}',
                          ),
                        ),
                      ),
                    ),
                ],
              ),
            ),
            if (bag && items.isEmpty) const Text('사용할 아이템이 없습니다.'),
            if (result == null)
              TextButton(
                onPressed: locked ? null : () => setState(() => bag = !bag),
                child: Text(bag ? '← 기술' : '가방 / BAG'),
              ),
            OutlinedButton(
              onPressed: widget.onSave,
              child: const Text('전투 저장 / SAVE'),
            ),
          ],
        ),
      ),
    );
    Widget stage() => LayoutBuilder(
      builder: (context, c) {
        final rect = fittedGameRect(c.biggest, widget.source);
        healthWidth = (rect.width * .48).clamp(80, 220);
        return Stack(
          children: [
            Positioned.fill(
              child: GameViewport(image: widget.image, source: widget.source),
            ),
            Positioned(
              top: rect.top + 12,
              left: rect.left + 12,
              child: Column(children: [for (final f in enemies) health(f)]),
            ),
            Positioned(
              bottom: c.maxHeight - rect.bottom + 12,
              right: c.maxWidth - rect.right + 12,
              child: Column(children: [for (final f in heroes) health(f)]),
            ),
          ],
        );
      },
    );
    return ColoredBox(
      color: ink,
      child: SafeArea(
        child: Column(
          children: [
            SizedBox(
              height: 44,
              child: Row(
                children: [
                  const SizedBox(width: 12),
                  const Expanded(
                    child: Text(
                      'GE4G / BATTLE',
                      style: TextStyle(
                        color: acid,
                        fontWeight: FontWeight.w900,
                      ),
                    ),
                  ),
                  IconButton(
                    tooltip: '화면 회전',
                    onPressed: widget.onRotate,
                    icon: const Icon(Icons.screen_rotation, color: acid),
                  ),
                ],
              ),
            ),
            Expanded(
              child: LayoutBuilder(
                builder: (context, c) => c.maxWidth > c.maxHeight * 1.3
                    ? Row(
                        children: [
                          Expanded(flex: 3, child: stage()),
                          Expanded(flex: 2, child: commands()),
                        ],
                      )
                    : Column(
                        children: [
                          SizedBox(
                            height: math.min(
                              c.maxHeight * .58,
                              c.maxWidth / widget.source.aspectRatio,
                            ),
                            child: stage(),
                          ),
                          Expanded(child: commands()),
                        ],
                      ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
