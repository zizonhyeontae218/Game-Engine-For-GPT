import 'dart:async';

import 'package:audioplayers/audioplayers.dart';

/// Device playback is an adapter; completion never advances gameplay.
class GameAudio {
  final List<AudioPlayer> _voices = [];
  int _next = 0;
  bool muted = false;
  Future<void> play(String path) async {
    if (muted) return;
    if (_voices.isEmpty) _voices.addAll(List.generate(4, (_) => AudioPlayer()));
    final voice = _voices[_next++ % _voices.length];
    await voice.stop();
    await voice.play(DeviceFileSource(path), volume: .4);
  }

  void pause() {
    for (final voice in _voices) {
      unawaited(voice.stop().catchError((Object _) {}));
    }
  }

  void dispose() {
    for (final voice in _voices) {
      unawaited(voice.dispose().catchError((Object _) {}));
    }
    _voices.clear();
  }
}
