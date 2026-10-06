import 'battle_screen.dart';

import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

import 'control_editor.dart';
import 'controls.dart';
import 'game_library.dart';
import 'player.dart';
import 'touch_controls.dart';
import 'game_popup.dart';
import 'cutscene_bubble.dart';
import 'manual_orientation.dart';
import 'game_viewport.dart';

final rc5CaptureKey = GlobalKey();

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
    builder: (context, child) =>
        RepaintBoundary(key: rc5CaptureKey, child: child!),
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
  final orientation = ManualOrientation();
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
    orientation.addListener(changed);
    unawaited(orientation.initialize());
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
      if (widget.arguments.contains('--rc5-evidence')) unawaited(rc5Evidence());
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
    if (editing ||
        player?.session == null ||
        player!.paused ||
        player!.popupVisible) {
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
    orientation.removeListener(changed);
    orientation.dispose();
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
                  'FLATLAND\nPLAYER / 0.2',
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
                    TextButton(
                      onPressed: busy
                          ? null
                          : () async {
                              await player!.open(game);
                              focus.requestFocus();
                            },
                      child: const Text('새 게임'),
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
  Future<void> playSettings() async {
    final live = player!, previousPause = player!.paused;
    live.setPaused(true);
    editing = true;
    final action = await showModalBottomSheet<String>(
      context: context,
      isScrollControlled: true,
      useSafeArea: true,
      builder: (sheetContext) => StatefulBuilder(
        builder: (sheetContext, update) => FractionallySizedBox(
          heightFactor: .9,
          child: ListView(
            padding: const EdgeInsets.all(16),
            children: [
              Row(
                children: [
                  Expanded(
                    child: Text(
                      live.game!.name,
                      style: const TextStyle(fontWeight: FontWeight.w900),
                    ),
                  ),
                  IconButton(
                    tooltip: '닫기',
                    onPressed: () => Navigator.pop(sheetContext),
                    icon: const Icon(Icons.close),
                  ),
                ],
              ),
              DropdownButton<String>(
                key: const Key('profile-select'),
                isExpanded: true,
                value: live.store!.current.activeId,
                items: live.store!.current.profiles
                    .map(
                      (p) => DropdownMenuItem(value: p.id, child: Text(p.name)),
                    )
                    .toList(),
                onChanged: (value) async {
                  if (value == null) return;
                  try {
                    await live.store!.select(value);
                    if (sheetContext.mounted) update(() {});
                  } catch (error) {
                    if (mounted) setState(() => failure = '$error');
                  }
                },
              ),
              for (final entry in const <(String, String, IconData)>[
                ('edit', '조작 편집', Icons.tune),
                ('preset', '게임 기본 프리셋 적용', Icons.restore),
                ('inventory', '인벤토리 / 장비', Icons.backpack_outlined),
                ('quests', '퀘스트', Icons.assignment_outlined),
                ('save', '저장', Icons.save_outlined),
                ('load', '불러오기', Icons.folder_open),
                ('restart', '새 게임 / 재시작', Icons.replay),
                ('dialogue', '대화 다시 보기', Icons.chat_bubble_outline),
                ('sound', '소리 켜기 / 끄기', Icons.volume_up_outlined),
                ('debug', '충돌 영역 표시', Icons.border_outer),
              ])
                ListTile(
                  key: entry.$1 == 'edit' ? const Key('edit-controls') : null,
                  leading: Icon(entry.$3),
                  title: Text(entry.$2),
                  onTap: () => Navigator.pop(sheetContext, entry.$1),
                ),
              if (!embedded)
                ListTile(
                  leading: const Icon(Icons.grid_view),
                  title: const Text('보관함'),
                  onTap: () => Navigator.pop(sheetContext, 'library'),
                ),
              ExpansionTile(
                title: const Text('디버그 상태'),
                children: [
                  SelectableText(
                    'scene ${live.status['scene']} · tick ${live.status['tick']}\n${live.status['state']}\n${live.status['last_action'] ?? ''}',
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
    editing = false;
    if (!mounted || player != live || live.session == null) return;
    live.setPaused(previousPause);
    try {
      switch (action) {
        case 'edit':
          await edit();
        case 'preset':
          await live.store!.resetPreset();
        case 'inventory':
          await gameJournal(false);
        case 'quests':
          await gameJournal(true);
        case 'save':
          await live.save();
        case 'load':
          await live.open(live.game!, load: true);
        case 'restart':
          await live.open(live.game!);
        case 'dialogue':
          live.showPopup();
        case 'sound':
          live.toggleSound();
        case 'debug':
          live.toggleDebug();
        case 'library':
          live.close();
          setState(() {});
      }
    } catch (error) {
      if (mounted) setState(() => failure = '$error');
    }
    if (mounted) focus.requestFocus();
  }

  Future<void> gameJournal(bool quests) async {
    final live = player!;
    final previous = live.paused;
    live.setPaused(true);
    await showModalBottomSheet<void>(
      context: context,
      isScrollControlled: true,
      backgroundColor: paper,
      builder: (context) => StatefulBuilder(
        builder: (context, update) {
          final systems = live.status['systems'] as Map? ?? {};
          final catalog = live.status['catalog'] as Map? ?? {};
          final owned = (systems[quests ? 'quests' : 'inventory'] as Map? ?? {})
              .entries
              .toList();
          final definitions =
              catalog[quests ? 'quests' : 'items'] as Map? ?? {};
          return SafeArea(
            child: ConstrainedBox(
              constraints: BoxConstraints(
                maxHeight: MediaQuery.sizeOf(context).height * .8,
              ),
              child: ListView(
                shrinkWrap: true,
                padding: const EdgeInsets.all(16),
                children: [
                  Text(
                    quests ? 'QUESTS / 퀘스트' : 'INVENTORY / 인벤토리',
                    style: const TextStyle(
                      fontWeight: FontWeight.w900,
                      fontSize: 20,
                    ),
                  ),
                  const Divider(thickness: 3, color: ink),
                  if (owned.isEmpty) const Text('아직 없습니다.'),
                  for (final entry in owned)
                    Card(
                      child: Padding(
                        padding: const EdgeInsets.all(12),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              '${(definitions[entry.key] as Map?)?['name'] ?? entry.key}',
                              style: const TextStyle(
                                fontWeight: FontWeight.bold,
                              ),
                            ),
                            if (quests) ...[
                              Text(
                                {
                                      'inactive': '미수락',
                                      'active': '진행 중',
                                      'completed': '완료',
                                      'failed': '실패',
                                    }[(entry.value as Map)['status']] ??
                                    '',
                              ),
                              for (final objective
                                  in ((entry.value as Map)['objectives']
                                              as Map? ??
                                          {})
                                      .entries)
                                Text(
                                  '${(definitions[entry.key] as Map?)?['objective_labels']?[objective.key] ?? objective.key}: ${objective.value} / ${(definitions[entry.key] as Map?)?['objectives']?[objective.key] ?? '?'}',
                                ),
                            ] else ...[
                              Text('수량 ${entry.value}'),
                              if ((entry.value as int) > 0 &&
                                  (definitions[entry.key] as Map?)?['usable'] ==
                                      true)
                                FilledButton(
                                  onPressed: () {
                                    live.command([
                                      {'op': 'use', 'item': entry.key},
                                    ]);
                                    update(() {});
                                  },
                                  child: const Text('사용'),
                                ),
                              if ((entry.value as int) > 0 &&
                                  (definitions[entry.key] as Map?)?['slot'] !=
                                      null)
                                OutlinedButton(
                                  onPressed: () {
                                    live.command([
                                      {
                                        'op': 'equip',
                                        'item': entry.key,
                                        'slot': definitions[entry.key]['slot'],
                                      },
                                    ]);
                                    update(() {});
                                  },
                                  child: const Text('장착'),
                                ),
                            ],
                          ],
                        ),
                      ),
                    ),
                  if (live.message != null) Text(live.message!),
                  OutlinedButton(
                    onPressed: () => Navigator.pop(context),
                    child: const Text('닫기'),
                  ),
                ],
              ),
            ),
          );
        },
      ),
    );
    if (mounted && player == live) live.setPaused(previous);
  }

  Widget playView() {
    final live = player!;
    final state = live.status['state'] as Map? ?? {};
    final hp = (live.status['actors'] as Map?)?['player']?['hp'];
    final power = (live.status['timers'] as Map?)?['power'] as int? ?? 0;
    final issue = live.store!.error ?? live.error ?? failure ?? live.message;
    return Stack(
      children: [
        Positioned.fill(
          top: 44,
          child: Stack(
            children: [
              Positioned.fill(
                child: GameViewport(
                  image: live.image,
                  source: Size(
                    (live.status['width'] as int? ?? 320).toDouble(),
                    (live.status['height'] as int? ?? 200).toDouble(),
                  ),
                ),
              ),
              Positioned.fill(child: TouchControls(router: live.input!)),
              if (live.paused)
                const Center(
                  child: ColoredBox(
                    color: ink,
                    child: Padding(
                      padding: EdgeInsets.all(12),
                      child: Text(
                        'PAUSED / 일시 정지',
                        style: TextStyle(color: acid),
                      ),
                    ),
                  ),
                ),
            ],
          ),
        ),
        Positioned(
          top: 0,
          left: 0,
          right: 0,
          height: 44,
          child: ColoredBox(
            color: ink,
            child: Row(
              children: [
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    '${hp == null ? 'GE4G' : '♥ $hp'}  ${state['game.score'] == null ? live.game!.name : 'SCORE ${state['game.score']}'}${power > 0 ? '  ⚡${(power / 60).ceil()}s' : ''}${live.status['view_label'] == null ? '' : ' / ${live.status['view_label']}'}',
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(
                      color: acid,
                      fontSize: 13,
                      fontWeight: FontWeight.w900,
                    ),
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
                IconButton(
                  key: const Key('manual-rotation'),
                  tooltip: orientation.landscape ? '세로모드' : '가로모드',
                  onPressed: () async {
                    live.release();
                    try {
                      await orientation.toggle();
                    } catch (error) {
                      if (mounted) setState(() => failure = '회전 실패: $error');
                    }
                  },
                  icon: Icon(
                    orientation.landscape
                        ? Icons.stay_current_portrait
                        : Icons.stay_current_landscape,
                    color: acid,
                  ),
                ),
                IconButton(
                  key: const Key('play-settings'),
                  tooltip: '게임 메뉴',
                  onPressed: playSettings,
                  icon: const Icon(Icons.menu, color: acid),
                ),
              ],
            ),
          ),
        ),
        if (issue != null)
          Positioned(
            left: 8,
            right: 8,
            bottom: 8,
            child: Material(
              color: paper,
              child: Row(
                children: [
                  Expanded(
                    child: Padding(
                      padding: const EdgeInsets.all(8),
                      child: Text(
                        issue,
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                  ),
                  IconButton(
                    tooltip: '알림 닫기',
                    onPressed: () => setState(() {
                      failure = null;
                      live.message = null;
                    }),
                    icon: const Icon(Icons.close),
                  ),
                ],
              ),
            ),
          ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    body: SafeArea(
      child: Stack(
        children: [
          Positioned.fill(
            child: Focus(
              focusNode: focus,
              autofocus: true,
              onKeyEvent: keyboard,
              onFocusChange: (hasFocus) {
                if (!hasFocus) player?.release();
              },
              child: !ready || (player?.loading ?? false)
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
          if (player?.popupVisible ?? false)
            Positioned.fill(
              child: player!.popup?['kind'] == 'battle'
                  ? BattleScreen(
                      battle: player!.popup!,
                      image: player!.image,
                      source: Size(
                        (player!.status['width'] as int).toDouble(),
                        (player!.status['height'] as int).toDouble(),
                      ),
                      onChoose: (choice) {
                        player!.choose(choice);
                        focus.requestFocus();
                      },
                      onSave: () {
                        player!.save();
                      },
                      onRotate: () {
                        orientation.toggle();
                      },
                    )
                  : player!.popup?['kind'] == 'bubble'
                  ? CutsceneBubble(
                      bubble: player!.popup!,
                      source: Size(
                        (player!.status['width'] as int).toDouble(),
                        (player!.status['height'] as int).toDouble(),
                      ),
                      onAdvance: () {
                        player!.choose('continue');
                        focus.requestFocus();
                      },
                      onSave: () {
                        player!.save();
                      },
                    )
                  : GamePopup(
                      text: player!.popup!['text'] as String,
                      choices: (player!.popup!['options'] as List? ?? [])
                          .whereType<Map>()
                          .map((e) => Map<String, dynamic>.from(e))
                          .toList(),
                      onChoose: (choice) {
                        player!.choose(choice);
                        focus.requestFocus();
                      },
                      onSave: () {
                        player!.save();
                      },
                      onClose: () {
                        player!.dismissPopup();
                        focus.requestFocus();
                      },
                    ),
            ),
        ],
      ),
    ),
  );

  /// Windows release acceptance: real Flutter UI + canonical native frames.
  Future<void> rc5Evidence() async {
    try {
      if (!embedded || player?.session == null) {
        throw StateError('embedded game required');
      }
      ticker.stop();
      final live = player!;
      await live.open(live.game!);
      live.setPaused(false);
      if (widget.arguments.contains('--capture-landscape')) {
        await orientation.toggle();
      }
      final output = Directory(Platform.environment['GE4G_SMOKE_OUTPUT']!);
      await output.create(recursive: true);
      void request(Map<String, dynamic> data) {
        live.status = live.engine.request({'session': live.session, ...data});
        live.popup = live.status['waiting'] is Map
            ? Map<String, dynamic>.from(live.status['waiting'])
            : null;
        live.popupVisible = live.popup != null;
      }

      Future<void> capture(String name) async {
        await Future<void>.delayed(const Duration(milliseconds: 100));
        await live.refreshFrame();
        if (mounted) setState(() {});
        SchedulerBinding.instance.scheduleFrame();
        await SchedulerBinding.instance.endOfFrame;
        await Future<void>.delayed(const Duration(milliseconds: 100));
        final boundary =
            rc5CaptureKey.currentContext!.findRenderObject()
                as RenderRepaintBoundary;
        final uiImage = await boundary.toImage(pixelRatio: 1);
        final uiBytes = await uiImage.toByteData(
          format: ui.ImageByteFormat.png,
        );
        await File(p.join(output.path, '$name-ui.png'))
            .writeAsBytes(uiBytes!.buffer.asUint8List());
        uiImage.dispose();
        final frame = await live.image!.toByteData(
          format: ui.ImageByteFormat.png,
        );
        await File(p.join(output.path, '$name-frame.png'))
            .writeAsBytes(frame!.buffer.asUint8List());
        final snapshot = live.engine.request({
          'op': 'observe',
          'session': live.session,
        })['snapshot'];
        await File(p.join(output.path, '$name-status.json'))
            .writeAsString(jsonEncode(live.status), encoding: utf8);
        await File(p.join(output.path, '$name.json'))
            .writeAsString(jsonEncode(snapshot), encoding: utf8);
      }

      request({'op': 'choose', 'choice': 'continue'});
      request({
        'op': 'command',
        'actions': [
          {
            'op': 'move',
            'entity': 'player',
            'at': [256, 176],
          },
        ],
      });
      await capture('01-top');
      final before = live.engine.request({
        'op': 'observe',
        'session': live.session,
      })['snapshot']['entities'];
      request({
        'op': 'command',
        'actions': [
          {'op': 'view', 'mode': 'depth'},
        ],
      });
      await capture('02-depth');
      final after = live.engine.request({
        'op': 'observe',
        'session': live.session,
      })['snapshot']['entities'];
      if (jsonEncode(before) != jsonEncode(after)) {
        throw StateError('view mutated actors');
      }
      request({
        'op': 'command',
        'actions': [
          {'op': 'event_scene', 'event': 'battle'},
        ],
      });
      await capture('03-battle-before');
      request({'op': 'choose', 'choice': 'move:scratch:rival'});
      for (final entry in <String, int>{
        '04-battle-effect': 12,
        '05-battle-impact': 12,
        '06-battle-hp': 12,
        '07-battle-settle': 72,
      }.entries) {
        var remaining = entry.value;
        while (remaining > 0) {
          final ticks = remaining > 15 ? 15 : remaining;
          request({'op': 'advance', 'ticks': ticks});
          remaining -= ticks;
        }
        await capture(entry.key);
      }
      await live.save();
      final saved = await library!
          .save(live.game!.id)
          .readAsString(encoding: utf8);
      await File(p.join(output.path, 'battle-save.json'))
          .writeAsString(saved, encoding: utf8);
      request({'op': 'choose', 'choice': 'move:pulse:rival'});
      request({'op': 'advance', 'ticks': 12});
      await capture('08-projectile');
      request({'op': 'skip'});
      request({'op': 'choose', 'choice': 'move:scratch:rival'});
      request({'op': 'skip'});
      request({'op': 'choose', 'choice': 'battle_continue'});
      await live.save();
      await capture('09-persistent-save');
      request({
        'op': 'command',
        'actions': [
          {'op': 'event_scene', 'event': 'battle'},
        ],
      });
      await capture('10-persistent-encounter');
      request({'op': 'choose', 'choice': 'guard'});
      request({'op': 'advance', 'ticks': 12});
      await capture('11-guard');
      request({'op': 'skip'});
      request({'op': 'choose', 'choice': 'item_potion'});
      request({'op': 'advance', 'ticks': 12});
      await capture('12-heal');
      request({'op': 'skip'});
      while (live.status['waiting']['result'] == null) {
        request({'op': 'choose', 'choice': 'move:scratch:rival'});
        request({'op': 'skip'});
      }
      request({'op': 'choose', 'choice': 'battle_continue'});
      request({
        'op': 'command',
        'actions': [
          {'op': 'view', 'mode': 'top'},
          {
            'op': 'move',
            'entity': 'player',
            'at': [256, 160],
          },
        ],
      });
      await capture('13-top-shadow');
      request({
        'op': 'command',
        'actions': [
          {'op': 'view', 'mode': 'depth'},
          {
            'op': 'move',
            'entity': 'player',
            'at': [256, 144],
          },
        ],
      });
      await capture('14-occlusion-behind');
      request({
        'op': 'command',
        'actions': [
          {
            'op': 'move',
            'entity': 'player',
            'at': [256, 176],
          },
        ],
      });
      await capture('15-occlusion-front');
      request({
        'op': 'command',
        'actions': [
          {
            'op': 'move',
            'entity': 'player',
            'at': [336, 272],
          },
        ],
      });
      for (final mode in ['top', 'depth', 'alternate']) {
        request({
          'op': 'command',
          'actions': [
            {'op': 'view', 'mode': mode},
          ],
        });
        for (var i = 0; i < 4; i++) {
          request({
            'op': 'advance',
            'ticks': 15,
            'input': {'down': true},
          });
        }
        final snapshot = live.engine.request({
          'op': 'observe',
          'session': live.session,
        })['snapshot'];
        final entity = (snapshot['entities'] as List).firstWhere(
          (e) => e['id'] == 'player',
        );
        if (entity['position']['x'] != 336 * 60 ||
            entity['position']['y'] != 272 * 60) {
          throw StateError('roof access $mode');
        }
        await capture('16-roof-$mode');
      }
      request({
        'op': 'command',
        'actions': [
          {
            'op': 'move',
            'entity': 'player',
            'at': [256, 176],
          },
          {'op': 'view', 'mode': 'depth'},
          {'op': 'event_scene', 'event': 'tour'},
        ],
      });
      request({'op': 'advance', 'ticks': 12});
      for (var line = 1; line <= 3; line++) {
        if (live.status['waiting']?['kind'] != 'bubble') {
          throw StateError('story line $line missing');
        }
        await capture('17-bubble-$line');
        final box =
            rc5CaptureKey.currentContext!.findRenderObject() as RenderBox;
        final point = box.localToGlobal(
          Offset(box.size.width / 2, box.size.height - 24),
        );
        GestureBinding.instance.handlePointerEvent(
          PointerDownEvent(pointer: 77, position: point),
        );
        GestureBinding.instance.handlePointerEvent(
          PointerUpEvent(pointer: 77, position: point),
        );
        await Future<void>.delayed(const Duration(milliseconds: 220));
      }
      if (live.status['waiting'] != null) {
        throw StateError('story tap failed to return');
      }
      for (var i = 0; i < 12; i++) {
        request({'op': 'advance', 'ticks': 1});
      }
      await live.save();
      final beforeLoad = live.status['systems'];
      await live.open(live.game!, load: true);
      if (jsonEncode(live.status['systems']['combatants']) !=
              jsonEncode(beforeLoad['combatants']) ||
          live.status['systems']['view'] != beforeLoad['view']) {
        throw StateError('save persistence regression');
      }
      await capture('18-restored');
      await File(p.join(output.path, 'result.json')).writeAsString(
        jsonEncode({
          'ok': true,
          'game_id': live.game!.id,
          'view_entities_equal': true,
          'platform': Platform.operatingSystem,
          'physical_device_acceptance': false,
        }),
        encoding: utf8,
      );
      exit(0);
    } catch (error) {
      stderr.writeln('RC5 rendered acceptance failed: $error');
      exit(1);
    }
  }

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
      live.status = live.engine.request({
        'op': 'replay',
        'session': live.session,
        'replay': replay,
      });
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
