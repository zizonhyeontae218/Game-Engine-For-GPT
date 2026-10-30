import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:ge4g_native/p5_native.dart';

import 'touch_controls.dart';

/// A view of native state and pixels. Every battle command executes in Rust.
class PentominoScreen extends StatefulWidget {
  final PentominoBridge? bridge;
  final Map<String, dynamic>? config;
  final String? captureDirectory;
  const PentominoScreen({
    super.key,
    this.bridge,
    this.config,
    this.captureDirectory,
  });

  @override
  State<PentominoScreen> createState() => _PentominoScreenState();
}

class _PentominoScreenState extends State<PentominoScreen> {
  final _captureKey = GlobalKey();
  PentominoBridge? _bridge;
  Map<String, dynamic>? _config, _observation, _saved;
  int? _session;
  int _action = 0, _target = 0, _width = 1, _height = 1;
  ui.Image? _image;
  String? _error, _message;
  bool _busy = true;

  List<Map<String, dynamic>> get _fighters =>
      (_config!['fighters'] as List).cast<Map<String, dynamic>>();
  List<Map<String, dynamic>> get _actions =>
      (_config!['actions'] as List).cast<Map<String, dynamic>>();
  Map<String, dynamic> get _battle =>
      _observation!['battle'] as Map<String, dynamic>;
  bool get _finished => _observation != null && _battle['winner'] != -1;

  @override
  void initState() {
    super.initState();
    _start();
  }

  Future<void> _start() async {
    if (mounted) setState(() => _busy = true);
    int? candidate;
    try {
      _bridge ??= widget.bridge ?? PentominoNative();
      _config ??=
          widget.config ??
          jsonDecode(await rootBundle.loadString('assets/pentomino_p5.json'))
              as Map<String, dynamic>;
      if (!mounted) return;
      final created = _bridge!.request({
        'op': 'create',
        'config': _config,
        'seed': 7,
        'mode': 'linked',
      }) as Map<String, dynamic>;
      candidate = created['session'] as int;
      final observation = _bridge!.request({
        'op': 'observe',
        'session': candidate,
      }) as Map<String, dynamic>;
      final frame = _bridge!.request({
        'op': 'frame',
        'session': candidate,
      }) as Map<String, dynamic>;
      final image = await _decode(frame);
      if (!mounted) {
        image.dispose();
        _bridge!.request({'op': 'close', 'session': candidate});
        return;
      }
      if (_session != null) {
        _bridge!.request({'op': 'close', 'session': _session});
      }
      final oldImage = _image;
      setState(() {
        _session = candidate;
        _observation = observation;
        _image = image;
        _width = frame['width'] as int;
        _height = frame['height'] as int;
        _action = 0;
        _target = _defaultTarget();
        _error = null;
        _message = null;
        _busy = false;
      });
      oldImage?.dispose();
      if (widget.captureDirectory != null) await _capture('start');
    } catch (error) {
      if (candidate != null && candidate != _session) {
        _bridge?.request({'op': 'close', 'session': candidate});
      }
      if (mounted) {
        setState(() {
          _error = '$error';
          _busy = false;
        });
      }
    }
  }

  Future<ui.Image> _decode(Map<String, dynamic> frame) {
    final done = Completer<ui.Image>();
    final width = frame['width'] as int, height = frame['height'] as int;
    final bytes = Uint8List.fromList((frame['rgba'] as List).cast<int>());
    if (width < 1 || height < 1 || bytes.length != width * height * 4) {
      throw StateError('Invalid native frame.');
    }
    ui.decodeImageFromPixels(
      bytes,
      width,
      height,
      ui.PixelFormat.rgba8888,
      done.complete,
    );
    return done.future;
  }

  int _defaultTarget() {
    final actor = _battle['turn'] as int;
    final relation = _actions[_action]['target'];
    if (relation != 'enemy') return actor;
    return _fighters.indexWhere((f) => f['team'] != _fighters[actor]['team']);
  }

  Future<void> _refresh(Map<String, dynamic> observation) async {
    final frame = _bridge!.request({
      'op': 'frame',
      'session': _session,
    }) as Map<String, dynamic>;
    final image = await _decode(frame);
    if (!mounted) {
      image.dispose();
      return;
    }
    final oldImage = _image;
    setState(() {
      _observation = observation;
      _image = image;
      _width = frame['width'] as int;
      _height = frame['height'] as int;
      _target = _defaultTarget();
    });
    oldImage?.dispose();
  }

  Future<void> _act() async {
    setState(() {
      _busy = true;
      _error = null;
      _message = null;
    });
    try {
      final result = _bridge!.request({
        'op': 'act',
        'session': _session,
        'command': {
          'id': (_battle['last_command_id'] as int) + 1,
          'actor': _battle['turn'],
          'action': _action,
          'target': _target,
        },
      }) as Map<String, dynamic>;
      await _refresh(result);
    } catch (error) {
      if (mounted) setState(() => _error = '$error');
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  void _save() {
    try {
      final save = _bridge!.request({
        'op': 'save',
        'session': _session,
      }) as Map<String, dynamic>;
      setState(() {
        _saved = save;
        _message = '이번 실행에 임시 저장했습니다.';
        _error = null;
      });
    } catch (error) {
      setState(() => _error = '$error');
    }
  }

  Future<void> _restore() async {
    setState(() {
      _busy = true;
      _error = null;
      _message = null;
    });
    try {
      _bridge!.request({'op': 'restore', 'session': _session, 'save': _saved});
      await _refresh(
        _bridge!.request({'op': 'observe', 'session': _session})
            as Map<String, dynamic>,
      );
      if (mounted) setState(() => _message = '저장한 전투로 돌아왔습니다.');
    } catch (error) {
      if (mounted) setState(() => _error = '$error');
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _capture(String label) async {
    SchedulerBinding.instance.scheduleFrame();
    await SchedulerBinding.instance.endOfFrame;
    if (!mounted) return;
    final boundary =
        _captureKey.currentContext!.findRenderObject() as RenderRepaintBoundary;
    final image = await boundary.toImage(pixelRatio: 1);
    final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
    image.dispose();
    final dir = Directory(widget.captureDirectory!);
    await dir.create(recursive: true);
    await File('${dir.path}/pentomino-$label-ui.png')
        .writeAsBytes(bytes!.buffer.asUint8List());
    await File('${dir.path}/pentomino-$label-state.json')
        .writeAsString(jsonEncode(_observation));
  }

  @override
  void dispose() {
    if (_session != null) {
      _bridge?.request({'op': 'close', 'session': _session});
    }
    _image?.dispose();
    super.dispose();
  }

  String _actionLabel(Map<String, dynamic> action) => switch (action['id']) {
    'strike' || 'attack' => '공격',
    'heal' => '회복',
    'poison' => '독',
    'regen' => '재생',
    'rest' || 'recover' => '자원 회복',
    _ => action['id'] as String,
  };

  Widget _roster() {
    final states = (_battle['fighters'] as List).cast<Map<String, dynamic>>();
    return Column(
      children: [
        for (var i = 0; i < _fighters.length; i++)
          Container(
            width: double.infinity,
            padding: const EdgeInsets.symmetric(vertical: 12, horizontal: 16),
            decoration: BoxDecoration(
              color: !_finished && _battle['turn'] == i ? acid : paper,
              border: const Border(bottom: BorderSide(color: ink)),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Expanded(
                      child: Text(
                        '${_fighters[i]['id']} / 팀 ${_fighters[i]['team']}',
                        style: const TextStyle(fontWeight: FontWeight.w900),
                      ),
                    ),
                    Text('HP ${states[i]['hp']} / ${_fighters[i]['hp']}'),
                  ],
                ),
                const SizedBox(height: 6),
                LinearProgressIndicator(
                  value: (states[i]['hp'] as int) / (_fighters[i]['hp'] as int),
                  color: ink,
                  backgroundColor: Colors.black12,
                  minHeight: 4,
                ),
                const SizedBox(height: 6),
                Text(
                  '자원 ${states[i]['resource']} / ${_fighters[i]['resource']}'
                  '${_status(states[i], 'poison', '독')}${_status(states[i], 'regen', '재생')}',
                ),
              ],
            ),
          ),
      ],
    );
  }

  String _status(Map<String, dynamic> fighter, String key, String label) {
    final status = fighter[key] as Map<String, dynamic>;
    return status['turns'] == 0 ? '' : ' · $label ${status['turns']}턴';
  }

  Widget _commands() => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      Text(
        _finished
            ? (_battle['winner'] == -2 ? '무승부' : '팀 ${_battle['winner']} 승리')
            : '${_fighters[_battle['turn'] as int]['id']}의 차례',
        key: const Key('pentomino-turn'),
        style: const TextStyle(fontSize: 26, fontWeight: FontWeight.w900),
      ),
      const SizedBox(height: 16),
      DropdownButtonFormField<int>(
        key: ValueKey('pentomino-action-$_action'),
        initialValue: _action,
        decoration: const InputDecoration(labelText: '행동'),
        items: [
          for (var i = 0; i < _actions.length; i++)
            DropdownMenuItem(
              value: i,
              child: Text(
                '${_actionLabel(_actions[i])} · 자원 ${_actions[i]['cost']}',
              ),
            ),
        ],
        onChanged: _busy || _finished
            ? null
            : (value) => setState(() {
                _action = value!;
                _target = _defaultTarget();
              }),
      ),
      const SizedBox(height: 12),
      DropdownButtonFormField<int>(
        key: ValueKey('pentomino-target-$_target'),
        initialValue: _target,
        decoration: const InputDecoration(labelText: '대상'),
        items: [
          for (var i = 0; i < _fighters.length; i++)
            DropdownMenuItem(
              value: i,
              child: Text(_fighters[i]['id'] as String),
            ),
        ],
        onChanged: _busy || _finished
            ? null
            : (value) => setState(() => _target = value!),
      ),
      const SizedBox(height: 16),
      FilledButton(
        key: const Key('pentomino-execute'),
        onPressed: _busy || _finished ? null : _act,
        child: Text(_busy ? '진행 중…' : '행동 실행'),
      ),
      const SizedBox(height: 12),
      const Text('각 팀의 차례를 직접 조작하세요.'),
      const SizedBox(height: 20),
      Wrap(
        spacing: 8,
        runSpacing: 8,
        children: [
          OutlinedButton(
            onPressed: _busy ? null : _save,
            child: const Text('임시 저장'),
          ),
          OutlinedButton(
            onPressed: _busy || _saved == null ? null : _restore,
            child: const Text('임시 복원'),
          ),
          OutlinedButton(
            key: const Key('pentomino-restart'),
            onPressed: _busy ? null : _start,
            child: const Text('새 전투'),
          ),
        ],
      ),
    ],
  );

  @override
  Widget build(BuildContext context) => RepaintBoundary(
    key: _captureKey,
    child: Scaffold(
      backgroundColor: paper,
      appBar: AppBar(
        backgroundColor: paper,
        title: const Text(
          'PENTOMINO / pre1P6',
          style: TextStyle(fontWeight: FontWeight.w900, fontSize: 18),
        ),
      ),
      body: SafeArea(
        child: _observation == null
            ? Center(
                child: _error == null
                    ? const CircularProgressIndicator()
                    : Padding(
                        padding: const EdgeInsets.all(24),
                        child: Column(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            SelectableText(_error!),
                            const SizedBox(height: 16),
                            OutlinedButton(
                              onPressed: _busy ? null : _start,
                              child: const Text('다시 시작'),
                            ),
                          ],
                        ),
                      ),
              )
            : LayoutBuilder(
                builder: (context, bounds) => SingleChildScrollView(
                  padding: EdgeInsets.all(bounds.maxWidth < 600 ? 16 : 24),
                  child: Center(
                    child: ConstrainedBox(
                      constraints: const BoxConstraints(maxWidth: 1040),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          Text(
                            'TURN ${_battle['command_count']}',
                            style: const TextStyle(
                              fontSize: 14,
                              fontWeight: FontWeight.w900,
                            ),
                          ),
                          const SizedBox(height: 12),
                          if (bounds.maxWidth >= 800)
                            Row(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Expanded(flex: 3, child: _stage()),
                                const SizedBox(width: 24),
                                Expanded(flex: 2, child: _commands()),
                              ],
                            )
                          else ...[
                            _stage(),
                            const SizedBox(height: 24),
                            _commands(),
                          ],
                          if (_error != null)
                            Padding(
                              padding: const EdgeInsets.only(top: 16),
                              child: SelectableText(
                                _error!,
                                style: TextStyle(
                                  color: Theme.of(context).colorScheme.error,
                                ),
                              ),
                            ),
                          if (_message != null)
                            Padding(
                              padding: const EdgeInsets.only(top: 16),
                              child: Text(_message!),
                            ),
                        ],
                      ),
                    ),
                  ),
                ),
              ),
      ),
    ),
  );

  Widget _stage() => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      DecoratedBox(
        decoration: BoxDecoration(
          color: ink,
          border: Border.all(color: ink, width: 2),
        ),
        child: AspectRatio(
          aspectRatio: _width / _height,
          child: _image == null
              ? const SizedBox.shrink()
              : RawImage(
                  image: _image,
                  fit: BoxFit.contain,
                  filterQuality: FilterQuality.none,
                ),
        ),
      ),
      const SizedBox(height: 12),
      _roster(),
    ],
  );
}
