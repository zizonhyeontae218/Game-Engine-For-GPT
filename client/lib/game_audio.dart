import 'dart:async';

import 'package:audioplayers/audioplayers.dart';

/// Device playback is an adapter; completion never advances gameplay.
class GameAudio {
  final List<AudioPlayer> _voices = [];
  int _next = 0;
  AudioPlayer? _music;
  String? _musicPath;
  bool _musicPaused = false;
  Future<void> _musicQueue = Future.value();
  bool muted = false;
  bool _disposed = false;
  (String?, double, bool, bool)? _desiredMusic;
  Future<void> play(String path, {double volume = .4}) async {
    if (muted || _disposed) return;
    if (_voices.isEmpty) _voices.addAll(List.generate(4, (_) => AudioPlayer()));
    final voice = _voices[_next++ % _voices.length];
    await voice.stop();
    await voice.play(DeviceFileSource(path), volume: volume);
  }

  Future<void> syncMusic(
    String? path,
    double volume, {
    bool suspended = false,
  }) {
    final desired = (path, volume, suspended, muted);
    if (_desiredMusic == desired) return _musicQueue;
    _desiredMusic = desired;
    final task = _musicQueue.catchError((Object _) {}).then((_) async {
      if (_disposed) return;
      if (path == null) {
        await _music?.stop();
        _musicPath = null;
        _musicPaused = false;
        return;
      }
      if (muted || suspended) {
        await _music?.pause();
        _musicPaused = true;
        return;
      }
      _music ??= AudioPlayer();
      if (_musicPath != path) {
        await _music!.stop();
        await _music!.setReleaseMode(ReleaseMode.loop);
        await _music!.play(DeviceFileSource(path), volume: volume);
        _musicPath = path;
        _musicPaused = false;
      } else {
        await _music!.setVolume(volume);
        if (_musicPaused) {
          await _music!.resume();
          _musicPaused = false;
        }
      }
    });
    _musicQueue = task;
    return task;
  }

  void pause() {
    _desiredMusic = null;
    _musicPaused = true;
    unawaited(_music?.pause().catchError((Object _) {}) ?? Future.value());
    for (final voice in _voices) {
      unawaited(voice.stop().catchError((Object _) {}));
    }
  }

  void dispose() {
    _disposed = true;
    unawaited(_music?.dispose().catchError((Object _) {}) ?? Future.value());
    _music = null;
    for (final voice in _voices) {
      unawaited(voice.dispose().catchError((Object _) {}));
    }
    _voices.clear();
  }
}
