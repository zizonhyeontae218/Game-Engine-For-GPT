import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:path/path.dart' as p;

import 'controls.dart';

Future<void> atomicText(File file, String text) async {
  await file.parent.create(recursive: true);
  final temporary = File(
    '${file.path}.$pid.${DateTime.now().microsecondsSinceEpoch}.tmp',
  );
  try {
    await temporary.writeAsString('$text\n', flush: true);
    await temporary.rename(file.path);
  } finally {
    if (await temporary.exists()) await temporary.delete();
  }
}

class ControlStore extends ChangeNotifier {
  final Directory directory;
  final String gameId;
  late GameControls current, packaged;
  String? error;
  StreamSubscription<FileSystemEvent>? _watch;
  Timer? _debounce, _poll;
  bool _writing = false, _disposed = false, _reloading = false;
  String _lastLayouts = '', _lastBindings = '';
  Future<void> _queue = Future.value();
  ControlStore(this.directory, this.gameId);
  File get layoutsFile => File(p.join(directory.path, 'layouts.json'));
  File get bindingsFile => File(p.join(directory.path, 'bindings.json'));
  Future<void> initialize(
    String defaultLayouts,
    String defaultBindings, {
    String? legacyLayouts,
    String? legacyBindings,
  }) async {
    final candidate = GameControls.parse(
      defaultLayouts,
      defaultBindings,
      gameId,
    );
    packaged = candidate;
    await directory.create(recursive: true);
    // Only migrate untouched generic layouts; retain user edits and invalid files.
    if (candidate.layoutVersion == 2 &&
        legacyLayouts != null &&
        legacyBindings != null &&
        await layoutsFile.exists() &&
        await bindingsFile.exists()) {
      try {
        final previous = GameControls.parse(
          await layoutsFile.readAsString(),
          await bindingsFile.readAsString(),
          gameId,
        );
        final legacy = GameControls.parse(
          legacyLayouts,
          legacyBindings,
          gameId,
        );
        final previousLayout = previous.layoutsJson..remove('active_profile');
        final legacyLayout = legacy.layoutsJson..remove('active_profile');
        if (jsonEncode(previousLayout) == jsonEncode(legacyLayout) &&
            previous.bindingsText == legacy.bindingsText) {
          await atomicText(bindingsFile, candidate.bindingsText);
          await atomicText(layoutsFile, candidate.layoutsText);
        }
      } on FormatException {
        // reload() reports invalid overrides while keeping the packaged preset valid.
      }
    }
    if (!await layoutsFile.exists()) {
      await atomicText(layoutsFile, candidate.layoutsText);
    }
    if (!await bindingsFile.exists()) {
      await atomicText(bindingsFile, candidate.bindingsText);
    }
    current = candidate;
    await reload();
    try {
      _watch = directory.watch().listen(
        (_) {
          _debounce?.cancel();
          _debounce = Timer(
            const Duration(milliseconds: 100),
            () => unawaited(reload()),
          );
        },
        onError: (Object failure) {
          debugPrint('Control watcher failed; polling continues: $failure');
        },
      );
    } on FileSystemException catch (failure) {
      debugPrint('Control watcher unavailable; polling continues: $failure');
      // Polling below is a real fallback where platform watchers are unavailable.
    }
    _poll = Timer.periodic(
      const Duration(seconds: 1),
      (_) => unawaited(reload()),
    );
  }

  Future<void> reload() async {
    if (_writing || _disposed || _reloading) return;
    _reloading = true;
    try {
      final layouts = await layoutsFile.readAsString();
      final mappings = await bindingsFile.readAsString();
      if (layouts == _lastLayouts && mappings == _lastBindings) {
        if (error != null) {
          error = null;
          notifyListeners();
        }
        return;
      }
      final candidate = GameControls.parse(layouts, mappings, gameId);
      if (_disposed || _writing) return;
      current = candidate;
      _lastLayouts = layouts;
      _lastBindings = mappings;
      error = null;
      notifyListeners();
    } catch (failure) {
      if (!_disposed) {
        error = '프로필 JSON 오류: $failure';
        notifyListeners();
      }
    } finally {
      _reloading = false;
    }
  }

  Future<void> apply(String layouts, String mappings) async {
    final candidate = GameControls.parse(layouts, mappings, gameId);
    final work = _queue.catchError((Object _) {}).then((_) async {
      if (_disposed) throw StateError('Control store is closed');
      _writing = true;
      try {
        await atomicText(bindingsFile, candidate.bindingsText);
        await atomicText(layoutsFile, candidate.layoutsText);
        if (_disposed) return;
        _lastLayouts = '${candidate.layoutsText}\n';
        _lastBindings = '${candidate.bindingsText}\n';
        current = candidate;
        error = null;
        notifyListeners();
      } finally {
        _writing = false;
      }
    });
    _queue = work;
    await work;
  }

  Future<void> resetPreset() =>
      apply(packaged.layoutsText, packaged.bindingsText);

  Future<void> select(String profile) async {
    final layouts = current.layoutsJson..['active_profile'] = profile;
    await apply(jsonEncode(layouts), current.bindingsText);
  }

  @override
  void dispose() {
    _disposed = true;
    _watch?.cancel();
    _debounce?.cancel();
    _poll?.cancel();
    super.dispose();
  }
}
