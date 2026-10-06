import 'dart:math' as math;

import 'package:flutter/material.dart';

import 'game_viewport.dart';
import 'touch_controls.dart';

/// Screen anchor comes from the canonical native projection, never Dart gameplay.
class CutsceneBubble extends StatelessWidget {
  final Map<String, dynamic> bubble;
  final Size source;
  final VoidCallback onAdvance;
  final VoidCallback onSave;
  const CutsceneBubble({
    super.key,
    required this.bubble,
    required this.source,
    required this.onAdvance,
    required this.onSave,
  });
  @override
  Widget build(BuildContext context) => TweenAnimationBuilder<double>(
    tween: Tween(begin: 0.0, end: 1.0),
    duration: const Duration(milliseconds: 160),
    curve: Curves.easeOut,
    builder: (context, opacity, child) =>
        Opacity(opacity: opacity, child: child),
    child: LayoutBuilder(
      builder: (context, c) {
        final rect = fittedGameRect(
          Size(c.maxWidth, math.max(1, c.maxHeight - 44)),
          source,
        );
        final anchor =
            bubble['screen_anchor'] as List? ??
            [source.width / 2, source.height / 2];
        final x = rect.left + (anchor[0] as num) * rect.width / source.width;
        final y =
            44 + rect.top + (anchor[1] as num) * rect.height / source.height;
        final foot = bubble['screen_foot'] as List?;
        final footY = foot == null
            ? y + 44
            : 44 + rect.top + (foot[1] as num) * rect.height / source.height;
        final width = math.min(320.0, math.max(1.0, c.maxWidth - 24));
        final left = (x - width / 2)
            .clamp(12.0, math.max(12, c.maxWidth - width - 12))
            .toDouble();
        final below = y < 220;
        return GestureDetector(
          key: const Key('cutscene-tap'),
          behavior: HitTestBehavior.opaque,
          onTap: onAdvance,
          child: Stack(
            children: [
              Positioned(
                left: left,
                top: below
                    ? (footY + 16)
                          .clamp(48.0, math.max(48, c.maxHeight - 220))
                          .toDouble()
                    : null,
                bottom: below
                    ? null
                    : (c.maxHeight - y + 12)
                          .clamp(16.0, math.max(16, c.maxHeight - 48))
                          .toDouble(),
                width: width,
                child: AnimatedSwitcher(
                  duration: const Duration(milliseconds: 180),
                  transitionBuilder: (child, animation) => FadeTransition(
                    opacity: animation,
                    child: SlideTransition(
                      position:
                          Tween<Offset>(
                            begin: const Offset(0, .06),
                            end: Offset.zero,
                          ).animate(
                            CurvedAnimation(
                              parent: animation,
                              curve: Curves.easeOut,
                            ),
                          ),
                      child: child,
                    ),
                  ),
                  child: Material(
                    key: ValueKey(bubble['id']),
                    color: paper,
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(12),
                      side: const BorderSide(color: ink, width: 2),
                    ),
                    child: Padding(
                      padding: const EdgeInsets.all(16),
                      child: Column(
                        mainAxisSize: MainAxisSize.min,
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            bubble['speaker'] as String? ?? '이야기',
                            style: const TextStyle(fontWeight: FontWeight.bold),
                          ),
                          const SizedBox(height: 8),
                          ConstrainedBox(
                            constraints: const BoxConstraints(maxHeight: 140),
                            child: SingleChildScrollView(
                              child: Text(bubble['text'] as String),
                            ),
                          ),
                          const SizedBox(height: 10),
                          const Text(
                            '탭하여 계속 ▾',
                            style: TextStyle(fontSize: 12),
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
              ),
              Positioned(
                left: x.clamp(8.0, math.max(8, c.maxWidth - 16)).toDouble(),
                top: ((below ? footY + 4 : y - 12))
                    .clamp(44.0, math.max(44, c.maxHeight - 12))
                    .toDouble(),
                child: Icon(
                  below ? Icons.arrow_drop_up : Icons.arrow_drop_down,
                  color: ink,
                  size: 16,
                ),
              ),
              Positioned(
                top: 0,
                right: 0,
                child: IconButton(
                  key: const Key('bubble-save'),
                  tooltip: '이 장면 저장',
                  onPressed: onSave,
                  icon: const Icon(Icons.save_outlined, color: acid),
                ),
              ),
            ],
          ),
        );
      },
    ),
  );
}
