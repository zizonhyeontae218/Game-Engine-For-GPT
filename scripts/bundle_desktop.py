#!/usr/bin/env python3
"""Distribute each desktop game with its Flutter client and native runtime."""
import argparse
import json
from pathlib import Path
import shutil
import tarfile
import zipfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--client', type=Path, required=True, help='Flutter release bundle / Release directory')
    parser.add_argument('--game', type=Path, required=True)
    parser.add_argument('--platform', choices=['windows', 'linux'], required=True)
    parser.add_argument('--out', type=Path, required=True, help='New output directory; existing contents are never overwritten')
    args = parser.parse_args()
    executable = 'ge4g_client.exe' if args.platform == 'windows' else 'ge4g_client'
    native = 'ge4g_client.dll' if args.platform == 'windows' else 'lib/libge4g_client.so'
    required = [executable, native, 'data/flutter_assets/AssetManifest.bin', 'flutter_windows.dll' if args.platform == 'windows' else 'lib/libflutter_linux_gtk.so']
    for name in required:
        if not (args.client / name).is_file():
            parser.error(f'Incomplete built Flutter client: missing {name}')
    if args.out.exists():
        parser.error(f'Output already exists: {args.out}')
    with zipfile.ZipFile(args.game) as source:
        manifest = json.loads(source.read('bundle.json'))
        if manifest['schema_version'] not in (1,2) or manifest['engine_abi'] != 1:
            parser.error('Unsupported game package version')
    shutil.copytree(args.client, args.out)
    shutil.copy2(Path(__file__).resolve().parents[1] / 'LICENSE', args.out / 'LICENSE')
    shutil.copy2(Path(__file__).resolve().parents[1] / 'packaging/arch/ge4g.png', args.out / 'ge4g.png')
    (args.out / 'data').mkdir(exist_ok=True)
    shutil.copy2(args.game, args.out / 'data/basement_game.ge4g')
    (args.out / 'client_mode.json').write_text(json.dumps({'schema_version': 1, 'mode': 'embedded', 'game': 'data/basement_game.ge4g', 'allow_library': False}, indent=2) + '\n')
    (args.out / 'PLAY.txt').write_text('GE4G / GameEngineForGPT — Basement / FlatLand\n\nStart ge4g_client' + ('.exe' if args.platform == 'windows' else '') + '. Your game starts directly.\nJoystick / WASD / arrows: move. Z / E: interact. X, C, Space: game actions.\nUse the in-game control editor to switch or edit per-game JSON profiles.\nSaves and controls live in your user application data directory.\n')
    # Ordinary Linux desktop dependencies are supplied by Arch; the engine is embedded.
    if args.platform == 'linux':
        (args.out / 'ge4g.desktop').write_text('[Desktop Entry]\nType=Application\nName=' + manifest['name'].replace('\n', ' ') + '\nExec=ge4g_client\nIcon=ge4g\nTerminal=false\nCategories=Game;\n')
        (args.out / 'ge4g_client').chmod(0o755)
        archive = args.out.with_suffix('.tar.gz')
        with tarfile.open(archive, 'w:gz') as target:
            target.add(args.out, arcname=args.out.name)
    else:
        archive = args.out.with_suffix('.zip')
        with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED) as target:
            for path in sorted(args.out.rglob('*')):
                if path.is_file():
                    target.write(path, arcname=f'{args.out.name}/{path.relative_to(args.out).as_posix()}')
    print(json.dumps({'ok': True, 'mode': 'embedded', 'game_id': manifest['game_id'], 'directory': str(args.out), 'archive': str(archive)}))

if __name__ == '__main__':
    main()
