import 'dart:async';
import 'dart:convert';

import 'package:flutter/services.dart';

import 'dart:ui' as ui;

import 'package:flutter/foundation.dart';
import 'package:ge4g_native/ge4g_native.dart';

import 'control_store.dart';
import 'controls.dart';
import 'game_library.dart';
import 'game_audio.dart';

import 'package:path/path.dart' as p;

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
  bool paused = false,
      debug = false,
      loading = false,
      _decoding = false,
      _disposed = false;
  Completer<void>? _frameIdle;
  String? error, message;
  final GameAudio audio = GameAudio();
  Map<String, dynamic>? popup;
  bool popupVisible = false;
  dynamic _seenPopup;
  int _audioCursor = 0;
  int _generation = 0;
  void _presentation() {
    final candidate = status['waiting'] is Map
        ? status['waiting']
        : status['popup'];
    if (candidate is Map && candidate['id'] != _seenPopup) {
      popup = Map<String, dynamic>.from(candidate);
      _seenPopup = candidate['id'];
      popupVisible = true;
      release();
    }
    if (candidate is Map && candidate['id'] == _seenPopup && popupVisible) {
      popup = Map<String, dynamic>.from(candidate);
    }
    if (candidate is! Map && popup?['kind'] == 'bubble') {
      popup = null;
      popupVisible = false;
    }
    final music = (status['systems'] as Map?)?['music'] as Map?;
    String? musicPath;
    if (music?['file'] is String && game != null) {
      final relative = packagePath(music!['file'] as String);
      musicPath = p.joinAll([
        game!.directory.path,
        'game',
        ...relative.split('/'),
      ]);
    }
    unawaited(
      audio
          .syncMusic(
            musicPath,
            ((music?['volume'] as num?)?.toDouble() ?? 40) / 100,
            suspended: paused,
          )
          .catchError((Object failure) {
            if (!_disposed) {
              message = '음악 재생 오류: $failure';
              notifyListeners();
            }
          }),
    );
    final cues = (status['audio'] as List? ?? []).whereType<Map>().toList()
      ..sort((a, b) => (a['id'] as int).compareTo(b['id'] as int));
    for (final cue in cues) {
      final id = cue['id'] as int;
      if (id <= _audioCursor || cue['file'] is! String || game == null) {
        continue;
      }
      final relative = packagePath(cue['file'] as String);
      final path = p.joinAll([
        game!.directory.path,
        'game',
        ...relative.split('/'),
      ]);
      unawaited(
        audio.play(path).catchError((Object failure) {
          if (!_disposed) {
            message = '사운드 재생 오류: $failure';
            notifyListeners();
          }
        }),
      );
    }
    _audioCursor = status['event_cursor'] as int? ?? _audioCursor;
  }

  void choose(String choice) {
    if (session == null) return;
    try {
      release();
      status = engine.request({
        'op': 'choose',
        'session': session,
        'choice': choice,
      });
      popupVisible = false;
      _presentation();
      refreshFrame();
    } catch (failure) {
      message = '선택 실패: $failure';
    }
    notifyListeners();
  }

  void command(List<Map<String, dynamic>> actions) {
    if (session == null) return;
    try {
      release();
      status = engine.request({
        'op': 'command',
        'session': session,
        'actions': actions,
      });
      _presentation();
      refreshFrame();
    } catch (failure) {
      message = '실행 실패: $failure';
    }
    notifyListeners();
  }

  void dismissPopup() {
    release();
    popupVisible = false;
    notifyListeners();
  }

  void showPopup() {
    if (popup == null) return;
    release();
    popupVisible = true;
    notifyListeners();
  }

  void toggleSound() {
    audio.muted = !audio.muted;
    if (audio.muted) audio.pause();
    _presentation();
    notifyListeners();
  }

  Duration? _lastElapsed;
  double _accumulator = 0;
  Player(this.engine, this.library);
  Future<void> open(InstalledGame candidate, {bool load = false}) async {
    close();
    loading = true;
    notifyListeners();
    final generation = _generation;
    final controls = ControlStore(library.controls(candidate.id), candidate.id);
    try {
      final legacyMappings = jsonDecode(
        await rootBundle.loadString('assets/default_bindings.json'),
      ) as Map<String, dynamic>;
      legacyMappings['game_id'] = candidate.id;
      await controls.initialize(
        await candidate.layouts,
        await candidate.bindings,
        legacyLayouts: await rootBundle.loadString(
          'assets/default_layouts.json',
        ),
        legacyBindings: jsonEncode(legacyMappings),
      );
      if (_disposed || generation != _generation) {
        controls.dispose();
        return;
      }
      final save = library.save(candidate.id);
      final loadSave = load && await save.exists();
      if (_disposed || generation != _generation) {
        controls.dispose();
        return;
      }
      final opened = engine.request({
        'op': 'open',
        'project': candidate.project,
        if (loadSave) 'load': save.path,
      });
      game = candidate;
      store = controls;
      input = InputRouter(controls.current);
      session = opened['session'] as int;
      status = opened;
      _audioCursor = status['event_cursor'] as int? ?? 0;
      _presentation();
      paused = false;
      error = null;
      store!.addListener(_controlsChanged);
      final pending = _frameIdle;
      if (pending != null) await pending.future;
      if (_disposed || generation != _generation) return;
      await refreshFrame();
      if (_disposed || generation != _generation) return;
      loading = false;
      notifyListeners();
    } catch (failure) {
      controls.dispose();
      loading = false;
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
    if (value) {
      audio.pause();
    } else {
      _presentation();
    }
    notifyListeners();
  }

  /// Fixed 60 Hz authoritative ticks; background gaps never advance the game.
  void tick(Duration elapsed) {
    if (session == null ||
        loading ||
        paused ||
        (popupVisible &&
            popup?['kind'] != 'battle' &&
            popup?['kind'] != 'bubble') ||
        error != null) {
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
        'input': popupVisible && popup?['kind'] == 'bubble'
            ? <String, dynamic>{}
            : input!.input,
      });
      _presentation();
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
    final ready = Completer<void>();
    _frameIdle = ready;
    final generation = _generation;
    final width = status['width'] as int, height = status['height'] as int;
    ui.ImmutableBuffer? buffer;
    ui.ImageDescriptor? descriptor;
    ui.Codec? codec;
    try {
      final pixels = engine.frame(session!, status['frame_bytes'] as int);
      buffer = await ui.ImmutableBuffer.fromUint8List(pixels);
      descriptor = ui.ImageDescriptor.raw(
        buffer,
        width: width,
        height: height,
        pixelFormat: ui.PixelFormat.rgba8888,
      );
      codec = await descriptor.instantiateCodec();
      final frame = await codec.getNextFrame();
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
      codec?.dispose();
      descriptor?.dispose();
      buffer?.dispose();
      _decoding = false;
      _frameIdle = null;
      ready.complete();
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
    audio.pause();
    popup = null;
    popupVisible = false;
    _seenPopup = null;
    _audioCursor = 0;
    debug = false;
    paused = false;
    loading = false;
    if (!_disposed) notifyListeners();
  }

  @override
  void dispose() {
    _disposed = true;
    close();
    audio.dispose();
    super.dispose();
  }
}
