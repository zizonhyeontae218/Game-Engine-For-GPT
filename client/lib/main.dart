import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

import 'control_editor.dart';
import 'controls.dart';
import 'game_library.dart';
import 'player.dart';
import 'touch_controls.dart';

Future<void> main(List<String> args) async {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(GE4GApp(arguments: args));
}

class GE4GApp extends StatelessWidget {
  final List<String> arguments;
  final EngineBridge? engine;
  final Directory? dataDirectory;
  const GE4GApp({
    super.key,
    this.arguments = const [],
    this.engine,
    this.dataDirectory,
  });
  @override
  Widget build(BuildContext context) => MaterialApp(
    title: 'GE4G / GameEngineForGPT',
    debugShowCheckedModeBanner: false,
    theme: ThemeData(
      useMaterial3: true,
      scaffoldBackgroundColor: paper,
      fontFamily: 'monospace',
      colorScheme: const ColorScheme.light(
        primary: ink,
        onPrimary: acid,
        secondary: acid,
        surface: paper,
        onSurface: ink,
        error: Color(0xffb52118),
      ),
      inputDecorationTheme: const InputDecorationTheme(
        border: OutlineInputBorder(borderRadius: BorderRadius.zero),
        filled: true,
        fillColor: Colors.white,
      ),
      filledButtonTheme: FilledButtonThemeData(
        style: FilledButton.styleFrom(
          shape: const RoundedRectangleBorder(),
          minimumSize: const Size(48, 48),
          textStyle: const TextStyle(fontWeight: FontWeight.w900),
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          shape: const RoundedRectangleBorder(),
          side: const BorderSide(color: ink, width: 2),
          minimumSize: const Size(48, 48),
        ),
      ),
      sliderTheme: const SliderThemeData(
        activeTrackColor: ink,
        thumbColor: ink,
        overlayColor: Color(0x22000000),
      ),
    ),
    home: ClientHome(
      arguments: arguments,
      engine: engine,
      dataDirectory: dataDirectory,
    ),
  );
}

class ClientHome extends StatefulWidget {
  final List<String> arguments;
  final EngineBridge? engine;
  final Directory? dataDirectory;
  const ClientHome({
    super.key,
    required this.arguments,
    this.engine,
    this.dataDirectory,
  });
  @override
  State<ClientHome> createState() => _ClientHomeState();
}

class _ClientHomeState extends State<ClientHome>
    with SingleTickerProviderStateMixin, WidgetsBindingObserver {
  Player? player;
  GameLibrary? library;
  List<InstalledGame> games = [];
  String? failure;
  bool ready = false,
      busy = false,
      embedded = false,
      editing = false,
      _wasPaused = false,
      _background = false;
  late final Ticker ticker;
  final focus = FocusNode(debugLabel: 'game input');
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    ticker = createTicker((elapsed) => player?.tick(elapsed))..start();
    unawaited(initialize());
  }

  Future<void> initialize() async {
    try {
      final engine = widget.engine ?? NativeEngine();
      final override = Platform.isLinux || Platform.isWindows
          ? Platform.environment['GE4G_CLIENT_DATA']
          : null;
      final root =
          widget.dataDirectory ??
          (override != null
              ? Directory(override)
              : await getApplicationSupportDirectory());
      final loaded = GameLibrary(
        root,
        (project) => engine.request({'op': 'validate', 'project': project}),
      );
      library = loaded;
      player = Player(engine, loaded)..addListener(changed);
      games = await loaded.list();
      if (Platform.isWindows || Platform.isLinux) {
        final executableRoot = p.dirname(Platform.resolvedExecutable);
        final configFile = File(p.join(executableRoot, 'client_mode.json'));
        if (await configFile.exists()) {
          final config = object(
            jsonDecode(await configFile.readAsString()),
            'client_mode',
          );
          version(config, 'client_mode');
          fields(config, {
            'schema_version',
            'mode',
            'game',
            'allow_library',
          }, 'client_mode');
          if (config['mode'] != 'embedded' ||
              config['allow_library'] != false) {
            throw const FormatException(
              'desktop distributions require embedded mode',
            );
          }
          embedded = true;
          final gamePath = packagePath(text(config['game'], 'game', max: 240));
          final game = await loaded.importFile(
            File(p.joinAll([executableRoot, ...gamePath.split('/')])),
          );
          await player!.open(game, load: true);
        }
      }
      if (!mounted) return;
      setState(() => ready = true);
      focus.requestFocus();
      if (widget.arguments.contains('--smoke-test')) unawaited(smoke());
    } catch (error) {
      if (mounted) {
        setState(() {
          failure = '$error';
          ready = true;
        });
      }
    }
  }

  void changed() {
    if (mounted) setState(() {});
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      if (_background) {
        _background = false;
        player?.setPaused(_wasPaused);
      }
    } else if (!_background) {
      _background = true;
      _wasPaused = player?.paused ?? false;
      player?.setPaused(true);
    } else {
      player?.release();
    }
  }

  @override
  void didChangeViewFocus(ui.ViewFocusEvent event) {
    if (event.state == ui.ViewFocusState.unfocused) player?.release();
  }

  Future<void> import() async {
    player?.release();
    final type = XTypeGroup(
      label: 'Basement game (.ge4g)',
      extensions: const ['ge4g'],
      uniformTypeIdentifiers: const ['public.data'],
    );
    try {
      final selected = await openFile(acceptedTypeGroups: [type]);
      if (selected == null || !mounted) return;
      setState(() {
        busy = true;
        failure = null;
      });
      if (await selected.length() > 64 * 1024 * 1024) {
        throw const FormatException('package exceeds 64 MiB');
      }
      final game = await library!.importBytes(await selected.readAsBytes());
      games = await library!.list();
      await player!.open(game, load: true);
      if (mounted) focus.requestFocus();
    } catch (error) {
      if (mounted) setState(() => failure = '불러오기 실패: $error');
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> edit() async {
    player!.release();
    editing = true;
    await showModalBottomSheet<void>(
      context: context,
      isScrollControlled: true,
      shape: const RoundedRectangleBorder(
        side: BorderSide(color: ink, width: 3),
      ),
      builder: (context) => FractionallySizedBox(
        heightFactor: .92,
        child: ControlEditor(store: player!.store!),
      ),
    );
    editing = false;
    player?.release();
    focus.requestFocus();
  }

  String? keyCode(LogicalKeyboardKey key) {
    final special = {
      LogicalKeyboardKey.arrowLeft: 'ArrowLeft',
      LogicalKeyboardKey.arrowRight: 'ArrowRight',
      LogicalKeyboardKey.arrowUp: 'ArrowUp',
      LogicalKeyboardKey.arrowDown: 'ArrowDown',
      LogicalKeyboardKey.space: 'Space',
      LogicalKeyboardKey.enter: 'Enter',
      LogicalKeyboardKey.tab: 'Tab',
      LogicalKeyboardKey.shiftLeft: 'ShiftLeft',
      LogicalKeyboardKey.controlLeft: 'ControlLeft',
      LogicalKeyboardKey.altLeft: 'AltLeft',
    };
    if (special.containsKey(key)) return special[key];
    final label = key.keyLabel.toUpperCase();
    if (RegExp(r'^[A-Z]$').hasMatch(label)) return 'Key$label';
    if (RegExp(r'^[0-9]$').hasMatch(label)) return 'Digit$label';
    return null;
  }

  KeyEventResult keyboard(FocusNode node, KeyEvent event) {
    if (editing || player?.session == null || player!.paused) {
      return KeyEventResult.ignored;
    }
    final code = keyCode(event.logicalKey);
    if (code == null ||
        !player!.input!.controls.activeBindings.keys.containsKey(code)) {
      return KeyEventResult.ignored;
    }
    player!.input!.key(code, event is! KeyUpEvent);
    return KeyEventResult.handled;
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    ticker.dispose();
    player?.removeListener(changed);
    player?.dispose();
    focus.dispose();
    super.dispose();
  }

  Widget block(Widget child, {Color color = paper}) => Container(
    padding: const EdgeInsets.all(16),
    decoration: BoxDecoration(
      color: color,
      border: Border.all(color: ink, width: 3),
      boxShadow: const [BoxShadow(color: ink, offset: Offset(5, 5))],
    ),
    child: child,
  );
  Widget libraryView() => Center(
    child: ConstrainedBox(
      constraints: const BoxConstraints(maxWidth: 880),
      child: ListView(
        padding: const EdgeInsets.all(24),
        children: [
          Row(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              const Expanded(
                child: Text(
                  'GE4G',
                  style: TextStyle(
                    fontSize: 68,
                    fontWeight: FontWeight.w900,
                    height: 1,
                  ),
                ),
              ),
              block(
                const Text(
                  'BASEMENT\nPLAYER / 0.1',
                  style: TextStyle(fontWeight: FontWeight.bold),
                ),
                color: acid,
              ),
            ],
          ),
          const SizedBox(height: 8),
          const Text('GameEngineForGPT', style: TextStyle(fontSize: 18)),
          const SizedBox(height: 32),
          block(
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'LOAD. PLAY. REWIRE.',
                  style: TextStyle(fontSize: 24, fontWeight: FontWeight.w900),
                ),
                const SizedBox(height: 12),
                const Text('Basement 게임을 불러오고, 나만의 조작 프로필로 플레이하세요.'),
                const SizedBox(height: 20),
                FilledButton.icon(
                  key: const Key('import-game'),
                  onPressed: busy || embedded ? null : import,
                  icon: const Icon(Icons.add),
                  label: Text(busy ? '게임 검증 중…' : '게임 불러오기 / IMPORT .GE4G'),
                ),
              ],
            ),
          ),
          const SizedBox(height: 32),
          const Text(
            'YOUR GAMES / 보관함',
            style: TextStyle(fontWeight: FontWeight.w900),
          ),
          const SizedBox(height: 12),
          if (games.isEmpty)
            const Text('첫 게임을 불러오세요.\n조이스틱 + Z / X / C / SPACE 기본 제공.'),
          for (final game in games)
            Padding(
              padding: const EdgeInsets.only(bottom: 16),
              child: block(
                Row(
                  children: [
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            game.name,
                            style: const TextStyle(
                              fontWeight: FontWeight.w900,
                              fontSize: 18,
                            ),
                          ),
                          Text('${game.id} / ${game.version}'),
                        ],
                      ),
                    ),
                    FilledButton(
                      onPressed: busy
                          ? null
                          : () async {
                              await player!.open(game, load: true);
                              focus.requestFocus();
                            },
                      child: const Text('PLAY →'),
                    ),
                  ],
                ),
              ),
            ),
          if (failure != null)
            Padding(
              padding: const EdgeInsets.only(top: 20),
              child: SelectableText(
                failure!,
                style: const TextStyle(color: Colors.red),
              ),
            ),
          if (player?.error != null)
            Padding(
              padding: const EdgeInsets.only(top: 20),
              child: Text(
                player!.error!,
                style: const TextStyle(color: Colors.red),
              ),
            ),
        ],
      ),
    ),
  );
  Widget playView() {
    final live = player!, controls = live.store!.current;
    final state = live.status['state'] as Map? ?? {};
    return Column(
      children: [
        Container(
          color: ink,
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          child: Row(
            children: [
              const Text(
                'GE4G',
                style: TextStyle(
                  color: acid,
                  fontWeight: FontWeight.w900,
                  fontSize: 22,
                ),
              ),
              const SizedBox(width: 14),
              Expanded(
                child: Text(
                  live.game!.name,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(color: paper),
                ),
              ),
              IconButton(
                tooltip: live.paused ? '계속' : '일시 정지',
                onPressed: () => live.setPaused(!live.paused),
                icon: Icon(
                  live.paused ? Icons.play_arrow : Icons.pause,
                  color: acid,
                ),
              ),
              if (!embedded)
                IconButton(
                  tooltip: '보관함',
                  onPressed: () {
                    live.close();
                    setState(() {});
                  },
                  icon: const Icon(Icons.grid_view, color: paper),
                ),
            ],
          ),
        ),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          child: Wrap(
            spacing: 10,
            runSpacing: 6,
            crossAxisAlignment: WrapCrossAlignment.center,
            children: [
              DropdownButton<String>(
                key: const Key('profile-select'),
                value: controls.activeId,
                items: controls.profiles
                    .map(
                      (profile) => DropdownMenuItem(
                        value: profile.id,
                        child: Text(profile.name),
                      ),
                    )
                    .toList(),
                onChanged: (value) async {
                  if (value == null) return;
                  try {
                    await live.store!.select(value);
                  } catch (error) {
                    if (mounted) setState(() => failure = '$error');
                  }
                  focus.requestFocus();
                },
              ),
              OutlinedButton(
                key: const Key('edit-controls'),
                onPressed: edit,
                child: const Text('조작 편집'),
              ),
              OutlinedButton(onPressed: live.save, child: const Text('저장')),
              OutlinedButton(
                onPressed: () => live.open(live.game!, load: true),
                child: const Text('불러오기'),
              ),
              OutlinedButton(
                onPressed: () => live.open(live.game!),
                child: const Text('재시작'),
              ),
              IconButton(
                tooltip: '충돌 영역 표시',
                onPressed: live.toggleDebug,
                icon: const Icon(Icons.border_outer),
              ),
            ],
          ),
        ),
        Expanded(
          child: LayoutBuilder(
            builder: (context, constraints) {
              final portrait = constraints.maxWidth < 600;
              Widget frame = Center(
                child: AspectRatio(
                  aspectRatio:
                      (live.status['width'] as int? ?? 320) /
                      (live.status['height'] as int? ?? 200),
                  child: Container(
                    decoration: BoxDecoration(
                      color: ink,
                      border: Border.all(color: ink, width: 3),
                    ),
                    child: live.image == null
                        ? const Center(
                            child: CircularProgressIndicator(color: acid),
                          )
                        : RawImage(
                            image: live.image,
                            filterQuality: FilterQuality.none,
                            fit: BoxFit.contain,
                          ),
                  ),
                ),
              );
              final touch = TouchControls(router: live.input!);
              return Stack(
                children: [
                  if (portrait)
                    Column(
                      children: [
                        Expanded(
                          child: Padding(
                            padding: const EdgeInsets.all(12),
                            child: frame,
                          ),
                        ),
                        SizedBox(
                          height: (constraints.maxHeight * .5)
                              .clamp(160, 300)
                              .toDouble(),
                          child: touch,
                        ),
                      ],
                    )
                  else
                    Positioned.fill(
                      child: Stack(
                        children: [
                          Padding(
                            padding: const EdgeInsets.all(12),
                            child: frame,
                          ),
                          Positioned.fill(child: touch),
                        ],
                      ),
                    ),
                  if (live.paused)
                    const Center(
                      child: ColoredBox(
                        color: ink,
                        child: Padding(
                          padding: EdgeInsets.all(16),
                          child: Text(
                            'PAUSED / 일시 정지',
                            style: TextStyle(
                              color: acid,
                              fontWeight: FontWeight.w900,
                            ),
                          ),
                        ),
                      ),
                    ),
                ],
              );
            },
          ),
        ),
        Container(
          width: double.infinity,
          color: acid,
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          child: Text(
            '${live.status['scene']} / TICK ${live.status['tick']}  ${state.entries.map((e) => '${e.key.split('.').last}:${e.value}').join('  ')}',
            maxLines: 2,
            style: const TextStyle(fontSize: 11, fontWeight: FontWeight.bold),
          ),
        ),
        if (live.status['dialogue'] != null ||
            live.status['last_action'] != null)
          Padding(
            padding: const EdgeInsets.all(8),
            child: Text(
              '${live.status['dialogue'] ?? ''} ${live.status['last_action'] == null ? '' : 'ACTION: ${live.status['last_action']}'}',
              maxLines: 2,
            ),
          ),
        if (live.message != null) Text(live.message!),
        if (live.store!.error != null || live.error != null || failure != null)
          Padding(
            padding: const EdgeInsets.all(8),
            child: Text(
              live.store!.error ?? live.error ?? failure!,
              maxLines: 3,
              style: const TextStyle(color: Colors.red),
            ),
          ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    body: SafeArea(
      child: Focus(
        focusNode: focus,
        autofocus: true,
        onKeyEvent: keyboard,
        onFocusChange: (hasFocus) {
          if (!hasFocus) player?.release();
        },
        child: !ready
            ? const Center(child: CircularProgressIndicator())
            : player?.session != null
            ? playView()
            : embedded
            ? Center(
                child: Padding(
                  padding: const EdgeInsets.all(24),
                  child: SelectableText(
                    'GE4G / 게임을 열 수 없습니다\n${player?.error ?? failure ?? ''}',
                  ),
                ),
              )
            : libraryView(),
      ),
    ),
  );
  Future<void> smoke() async {
    try {
      if (!embedded || player?.session == null) {
        throw StateError('smoke test requires an embedded desktop game');
      }
      final live = player!;
      live.setPaused(true);
      await live.open(live.game!);
      live.setPaused(true);
      final replay = object(
        jsonDecode(
          await File(
            p.join(
              live.game!.directory.path,
              'game',
              'replays',
              'journey.json',
            ),
          ).readAsString(),
        ),
        'replay',
      );
      for (var tick = 0; tick < replay['ticks']; tick++) {
        final actions = <String>{};
        for (final span in replay['inputs']) {
          if (tick >= span['start'] && tick < span['end']) {
            actions.addAll((span['actions'] as List).cast<String>());
          }
        }
        live.status = live.engine.request({
          'op': 'advance',
          'session': live.session,
          'ticks': 1,
          'input': {
            for (final name in ['left', 'right', 'up', 'down', 'interact'])
              name: actions.contains(name),
            'actions': <String>[],
          },
        });
      }
      await live.refreshFrame();
      await Future<void>.delayed(const Duration(milliseconds: 500));
      final output = Directory(
        Platform.environment['GE4G_SMOKE_OUTPUT'] ??
            p.join(library!.root.path, 'smoke'),
      );
      await output.create(recursive: true);
      await File(p.join(output.path, 'snapshot.json')).writeAsString(
        jsonEncode(
          live.engine.request({
            'op': 'observe',
            'session': live.session,
          })['snapshot'],
        ),
      );
      final png = await live.image!.toByteData(format: ui.ImageByteFormat.png);
      await File(p.join(output.path, 'frame.png'))
          .writeAsBytes(png!.buffer.asUint8List());
      await File(p.join(output.path, 'result.json')).writeAsString(
        jsonEncode({
          'ok': true,
          'mode': 'embedded',
          'tick': live.status['tick'],
          'scene': live.status['scene'],
          'game_id': live.game!.id,
        }),
      );
      exit(0);
    } catch (error) {
      stderr.writeln('GE4G client smoke failed: $error');
      exit(1);
    }
  }
}
