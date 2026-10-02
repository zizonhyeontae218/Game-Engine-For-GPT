import 'dart:math' as math;

import 'package:flutter/material.dart';

import 'controls.dart';

const paper = Color(0xfff2efe5),
    ink = Color(0xff151515),
    acid = Color(0xffdcff3f);

class TouchControls extends StatelessWidget {
  final InputRouter router;
  const TouchControls({super.key, required this.router});
  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: router,
    builder: (context, _) => LayoutBuilder(
      builder: (context, constraints) {
        final profile = router.controls.active;
        final short = math.min(constraints.maxWidth, constraints.maxHeight);
        if (short < 48) return const SizedBox.shrink();
        Rect position(double x, double y, double width, double height) {
          final w = width.clamp(48.0, constraints.maxWidth),
              h = height.clamp(48.0, constraints.maxHeight);
          return Rect.fromLTWH(
            (x * constraints.maxWidth - w / 2).clamp(
              0.0,
              constraints.maxWidth - w,
            ),
            (y * constraints.maxHeight - h / 2).clamp(
              0.0,
              constraints.maxHeight - h,
            ),
            w,
            h,
          );
        }

        final stick = profile.joystick;
        final size = (short * stick.size).clamp(math.min(96.0, short), short).toDouble();
        final rect = position(stick.x, stick.y, size, size);
        return Stack(
          children: [
            Positioned.fromRect(
              rect: rect,
              child: Joystick(key: ValueKey(router.controls), router: router),
            ),
            for (final button in profile.buttons)
              Positioned.fromRect(
                rect: position(
                  button.x,
                  button.y,
                  short * button.width,
                  short * button.height,
                ),
                child: Semantics(
                  label: '${button.label} 게임 버튼',
                  button: true,
                  child: Listener(
                    key: ValueKey('${router.controls.hashCode}/${button.id}'),
                    behavior: HitTestBehavior.opaque,
                    onPointerDown: (event) =>
                        router.button(button.id, event.pointer, true),
                    onPointerUp: (event) =>
                        router.button(button.id, event.pointer, false),
                    onPointerCancel: (event) =>
                        router.button(button.id, event.pointer, false),
                    child: Container(
                      alignment: Alignment.center,
                      decoration: BoxDecoration(
                        color: router.isPressed(button.id)
                            ? acid
                            : paper.withValues(alpha: .93),
                        border: Border.all(color: ink, width: 3),
                        boxShadow: const [
                          BoxShadow(color: ink, offset: Offset(4, 4)),
                        ],
                      ),
                      child: Text(
                        button.label,
                        style: const TextStyle(
                          color: ink,
                          fontWeight: FontWeight.w900,
                          fontSize: 18,
                        ),
                      ),
                    ),
                  ),
                ),
              ),
          ],
        );
      },
    ),
  );
}

class Joystick extends StatefulWidget {
  final InputRouter router;
  const Joystick({super.key, required this.router});
  @override
  State<Joystick> createState() => _JoystickState();
}

class _JoystickState extends State<Joystick> {
  int? pointer;
  void move(PointerEvent event, double size) {
    if (event.pointer != pointer) return;
    final delta = event.localPosition - Offset(size / 2, size / 2);
    final radius = size * .35;
    final length = delta.distance;
    final normalized = length > radius ? delta / length : delta / radius;
    widget.router.joystick(normalized.dx, normalized.dy);
  }

  void end(PointerEvent event) {
    if (event.pointer != pointer) return;
    pointer = null;
    widget.router.joystick(0, 0);
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, constraints) {
      final size = constraints.maxWidth;
      return Semantics(
        label: '이동 조이스틱',
        child: Listener(
          behavior: HitTestBehavior.opaque,
          onPointerDown: (event) {
            if (pointer == null) {
              pointer = event.pointer;
              move(event, size);
            }
          },
          onPointerMove: (event) => move(event, size),
          onPointerUp: end,
          onPointerCancel: end,
          child: Container(
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: paper.withValues(alpha: .85),
              border: Border.all(color: ink, width: 3),
            ),
            child: Stack(
              alignment: Alignment.center,
              children: [
                const Icon(Icons.add, color: ink, size: 70),
                Transform.translate(
                  offset:
                      Offset(widget.router.stickX, widget.router.stickY) *
                      size *
                      .25,
                  child: Container(
                    width: size * .38,
                    height: size * .38,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: acid,
                      border: Border.all(color: ink, width: 3),
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      );
    },
  );
}
