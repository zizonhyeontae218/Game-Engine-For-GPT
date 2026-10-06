import 'package:flutter/material.dart';

import 'game_library.dart';
import 'player.dart';
import 'touch_controls.dart';

/// Non-embedded library management. Content identity and transactions stay in storage.
class GameManager extends StatefulWidget {
  final GameLibrary library;
  final Player player;
  const GameManager({super.key, required this.library, required this.player});
  @override
  State<GameManager> createState() => _GameManagerState();
}

class _GameManagerState extends State<GameManager> {
  List<InstalledGame> games = [];
  bool busy = true;
  String? message;
  @override
  void initState() {
    super.initState();
    refresh();
  }

  Future<void> refresh() async {
    try {
      final loaded = await widget.library.list();
      if (mounted) {
        setState(() {
          games = loaded;
          busy = false;
        });
      }
    } catch (failure) {
      if (mounted) {
        setState(() {
          message = '$failure';
          busy = false;
        });
      }
    }
  }

  Future<void> move(int oldIndex, int newIndex) async {
    if (busy) return;
    setState(() => busy = true);
    try {
      await widget.library.reorder(oldIndex, newIndex);
    } catch (failure) {
      message = '순서 변경 실패: $failure';
    }
    await refresh();
  }

  Future<void> remove(InstalledGame game, bool allData) async {
    if (busy) return;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(allData ? '게임 및 데이터 모두 삭제' : '게임 삭제'),
        content: Text(
          allData
              ? '${game.name}의 게임 파일, 저장 데이터와 보관된 이전 저장, 조작 설정을 모두 삭제합니다. 되돌릴 수 없습니다.'
              : '${game.name}의 설치 파일을 삭제합니다. 저장 데이터와 조작 설정은 보관합니다.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('취소'),
          ),
          FilledButton(
            key: const Key('confirm-delete'),
            onPressed: () => Navigator.pop(context, true),
            child: const Text('삭제'),
          ),
        ],
      ),
    );
    if (confirmed != true || !mounted) return;
    setState(() => busy = true);
    try {
      await widget.player.removeGame(game.id, allData: allData);
      message = widget.player.message;
    } catch (failure) {
      message = '삭제 실패: $failure';
    }
    await refresh();
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    backgroundColor: paper,
    appBar: AppBar(
      backgroundColor: ink,
      foregroundColor: acid,
      title: const Text('GAME MANAGER / 게임 관리'),
    ),
    body: Column(
      children: [
        const Padding(
          padding: EdgeInsets.all(16),
          child: Text('손잡이를 끌어 순서를 바꾸세요. 저장과 조작 설정은 게임 ID로 유지됩니다.'),
        ),
        if (busy)
          const LinearProgressIndicator(color: acid, backgroundColor: ink),
        if (message != null)
          Padding(padding: const EdgeInsets.all(12), child: Text(message!)),
        Expanded(
          child: ReorderableListView.builder(
            key: const Key('game-order'),
            buildDefaultDragHandles: false,
            itemCount: games.length,
            onReorderItem: (oldIndex, newIndex) =>
                move(oldIndex, newIndex > oldIndex ? newIndex + 1 : newIndex),
            itemBuilder: (context, index) {
              final game = games[index];
              return Container(
                key: ValueKey(game.id),
                margin: const EdgeInsets.fromLTRB(16, 4, 16, 12),
                decoration: BoxDecoration(
                  color: paper,
                  border: Border.all(color: ink, width: 2),
                ),
                child: ListTile(
                  leading: ReorderableDragStartListener(
                    index: index,
                    enabled: !busy,
                    child: const Icon(Icons.drag_handle, color: ink),
                  ),
                  title: Text(
                    game.name,
                    style: const TextStyle(fontWeight: FontWeight.w900),
                  ),
                  subtitle: Text(
                    '${game.version}\n${game.id}\n${game.digest.substring(0, 12)}',
                  ),
                  isThreeLine: true,
                  trailing: PopupMenuButton<String>(
                    enabled: !busy,
                    key: ValueKey('manage-${game.id}'),
                    onSelected: (action) {
                      if (action == 'up') {
                        move(index, index - 1);
                      } else if (action == 'down') {
                        move(index, index + 2);
                      } else {
                        remove(game, action == 'all');
                      }
                    },
                    itemBuilder: (context) => [
                      if (index > 0)
                        const PopupMenuItem(value: 'up', child: Text('위로 이동')),
                      if (index < games.length - 1)
                        const PopupMenuItem(
                          value: 'down',
                          child: Text('아래로 이동'),
                        ),
                      const PopupMenuItem(
                        value: 'delete',
                        child: Text('게임 삭제'),
                      ),
                      const PopupMenuItem(
                        value: 'all',
                        child: Text('게임 및 데이터 모두 삭제'),
                      ),
                    ],
                  ),
                ),
              );
            },
          ),
        ),
        if (!busy && games.isEmpty)
          const Padding(
            padding: EdgeInsets.all(24),
            child: Text('설치한 게임이 없습니다.'),
          ),
      ],
    ),
  );
}
