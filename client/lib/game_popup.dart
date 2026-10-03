import 'package:flutter/material.dart';

import 'touch_controls.dart';

class GamePopup extends StatelessWidget {
  final String text;
  final VoidCallback onClose;
  const GamePopup({super.key, required this.text, required this.onClose});
  @override
  Widget build(BuildContext context) => ColoredBox(
    color: const Color(0xbb000000),
    child: Center(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 520, maxHeight: 420),
          child: Container(
            decoration: BoxDecoration(
              color: paper,
              border: Border.all(color: ink, width: 3),
              boxShadow: const [BoxShadow(color: acid, offset: Offset(6, 6))],
            ),
            padding: const EdgeInsets.all(16),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                const Text(
                  'MESSAGE / 대화',
                  style: TextStyle(fontWeight: FontWeight.w900),
                ),
                const Divider(color: ink, thickness: 2),
                Flexible(
                  child: SingleChildScrollView(child: SelectableText(text)),
                ),
                const SizedBox(height: 12),
                FilledButton(
                  key: const Key('close-game-popup'),
                  autofocus: true,
                  onPressed: onClose,
                  child: const Text('확인 / CONTINUE'),
                ),
              ],
            ),
          ),
        ),
      ),
    ),
  );
}
