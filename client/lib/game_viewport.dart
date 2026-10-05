import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/material.dart';

/// Fit the canonical frame independently of orientation flags and touch controls.
Rect fittedGameRect(Size available, Size source) {
  final scale = math.min(
    available.width / source.width,
    available.height / source.height,
  );
  final width = source.width * scale, height = source.height * scale;
  return Rect.fromLTWH(
    (available.width - width) / 2,
    available.width < available.height ? 0 : (available.height - height) / 2,
    width,
    height,
  );
}

class GameViewport extends StatelessWidget {
  final ui.Image? image;
  final Size source;
  const GameViewport({super.key, required this.image, required this.source});
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, constraints) {
      final rect = fittedGameRect(constraints.biggest, source);
      return Stack(
        children: [
          Positioned.fromRect(
            rect: rect,
            child: SizedBox(
              key: const Key('game-frame'),
              child: image == null
                  ? const Center(child: CircularProgressIndicator())
                  : RawImage(
                      image: image,
                      filterQuality: FilterQuality.none,
                      fit: BoxFit.fill,
                    ),
            ),
          ),
        ],
      );
    },
  );
}
