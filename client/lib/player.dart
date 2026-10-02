import 'dart:ui' as ui;

import 'package:flutter/foundation.dart';
import 'package:ge4g_native/ge4g_native.dart';

import 'control_store.dart';
import 'controls.dart';
import 'game_library.dart';

abstract interface class EngineBridge {
  Map<String, dynamic> request(Map<String, dynamic> request);
  Uint8List frame(int session, int length);
}

class NativeEngine implements EngineBridge {
  final BasementNative native = BasementNative();
  @override
  Map<String, dynamic> request(Map<String, dynamic> request) =>
      native.request(request);
  @override
  Uint8List frame(int session, int length) => native.frame(session, length);
}

class Player extends ChangeNotifier {
  final EngineBridge engine;
  final GameLibrary library;
  InstalledGame? game;
  ControlStore? store;
  InputRouter? input;
  int? session;
  Map<String, dynamic> status = {};
  ui.Image? image;
  bool paused = false, debug = false, _decoding = false, _disposed = false;
  String? error, message;
  int _generation = 0;
  Duration? _lastElapsed;
  double _accumulator = 0;
  Player(this.engine, this.library);
  Future<void> open(InstalledGame candidate, {bool load = false}) async {
    close();
    final generation = _generation;
    final controls = ControlStore(library.controls(candidate.id), candidate.id);
    try {
      await controls.initialize(
        await candidate.layouts,
        await candidate.bindings,
      );
      if (_disposed || generation != _generation) {
        controls.dispose();
        return;
      }
      final save = library.save(candidate.id);
      final opened = engine.request({
        'op': 'open',
        'project': candidate.project,
        if (load && await save.exists()) 'load': save.path,
      });
      game = candidate;
      store = controls;
      input = InputRouter(controls.current);
      session = opened['session'] as int;
      status = opened;
      paused = false;
      error = null;
      store!.addListener(_controlsChanged);
      await refreshFrame();
      notifyListeners();
    } catch (failure) {
      controls.dispose();
      error = '게임을 열 수 없습니다: $failure';
      notifyListeners();
    }
  }

  void _controlsChanged() {
    final current = store!;
    if (!identical(input!.controls, current.current)) {
      release();
      input!.activate(current.current);
    }
    notifyListeners();
  }

  void release() {
    input?.clear();
    _lastElapsed = null;
    _accumulator = 0;
    if (session != null) {
      try {
        engine.request({'op': 'release', 'session': session});
      } catch (failure) {
        error = '입력 해제 오류: $failure';
      }
    }
  }

  void setPaused(bool value) {
    release();
    paused = value;
    notifyListeners();
  }

  /// Fixed 60 Hz authoritative ticks; background gaps never advance the game.
  void tick(Duration elapsed) {
    if (session == null || paused || error != null) {
      _lastElapsed = elapsed;
      return;
    }
    final previous = _lastElapsed;
    _lastElapsed = elapsed;
    if (previous == null) return;
    _accumulator +=
        (elapsed - previous).inMicroseconds.clamp(0, 133333) / 1000000;
    final ticks = (_accumulator * 60).floor().clamp(0, 8);
    if (ticks == 0) return;
    _accumulator -= ticks / 60;
    try {
      status = engine.request({
        'op': 'advance',
        'session': session,
        'ticks': ticks,
        'input': input!.input,
      });
      refreshFrame();
      notifyListeners();
    } catch (failure) {
      release();
      paused = true;
      error = '실행 오류: $failure';
      notifyListeners();
    }
  }

  Future<void> refreshFrame() async {
    if (_decoding || session == null) return;
    _decoding = true;
    final generation = _generation;
    final width = status['width'] as int, height = status['height'] as int;
    try {
      final pixels = engine.frame(session!, status['frame_bytes'] as int);
      final buffer = await ui.ImmutableBuffer.fromUint8List(pixels);
      final descriptor = ui.ImageDescriptor.raw(
        buffer,
        width: width,
        height: height,
        pixelFormat: ui.PixelFormat.rgba8888,
      );
      final codec = await descriptor.instantiateCodec();
      final frame = await codec.getNextFrame();
      codec.dispose();
      descriptor.dispose();
      buffer.dispose();
      if (_disposed || generation != _generation) {
        frame.image.dispose();
        return;
      }
      final old = image;
      image = frame.image;
      old?.dispose();
      notifyListeners();
    } catch (failure) {
      if (!_disposed && generation == _generation) {
        error = '화면 표시 오류: $failure';
        notifyListeners();
      }
    } finally {
      _decoding = false;
    }
  }

  Future<void> save() async {
    if (game == null || session == null) return;
    try {
      final path = library.save(game!.id);
      await path.parent.create(recursive: true);
      engine.request({'op': 'save', 'session': session, 'path': path.path});
      message = '저장 완료';
    } catch (failure) {
      message = '저장 실패: $failure';
    }
    notifyListeners();
  }

  void toggleDebug() {
    if (session == null) return;
    try {
      debug = !debug;
      status = engine.request({
        'op': 'debug',
        'session': session,
        'enabled': debug,
      });
      refreshFrame();
    } catch (failure) {
      error = '$failure';
    }
    notifyListeners();
  }

  void close() {
    release();
    _generation++;
    if (session != null) {
      try {
        engine.request({'op': 'close', 'session': session});
      } catch (failure) {
        error = '게임 종료 오류: $failure';
      }
    }
    session = null;
    store?.removeListener(_controlsChanged);
    store?.dispose();
    store = null;
    input?.dispose();
    input = null;
    game = null;
    image?.dispose();
    image = null;
    status = {};
    debug = false;
    paused = false;
  }

  @override
  void dispose() {
    close();
    _disposed = true;
    super.dispose();
  }
}
