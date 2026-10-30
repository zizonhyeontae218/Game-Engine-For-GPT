#!/usr/bin/env python3
"""Public ctypes P5 consumer. No engine source imports or simulated battle engine."""
import argparse
import ctypes
import hashlib
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('library', type=Path)
    parser.add_argument('--config', type=Path, default=Path(__file__).with_name('game.json'))
    parser.add_argument('--out', type=Path, default=Path('/tmp/ge4g-p5-consumer'))
    args = parser.parse_args()
    lib = ctypes.CDLL(str(args.library.resolve()))
    lib.ge4g_p5_v1_abi_version.restype = ctypes.c_uint32
    assert lib.ge4g_p5_v1_abi_version() == 1
    lib.ge4g_p5_v1_request.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
    lib.ge4g_p5_v1_request.restype = ctypes.c_void_p
    lib.ge4g_p5_v1_string_free.argtypes = [ctypes.c_void_p]
    config = json.loads(args.config.read_text())
    checks = []

    def call(op, expected=True, **fields):
        data = json.dumps({'version': 1, 'command': {'op': op, **fields}}).encode()
        buf = ctypes.create_string_buffer(data)
        ptr = lib.ge4g_p5_v1_request(buf, len(data))
        assert ptr, 'null response'
        try:
            response = json.loads(ctypes.string_at(ptr))
        finally:
            lib.ge4g_p5_v1_string_free(ptr)
        assert response['ok'] is expected, response
        return response['result'] if expected else response['error']

    def create(mode='linked'):
        return call('create', config=config, seed=7, mode=mode)['session']

    session = create()
    commands = []

    def act(actor, action, target):
        command = {'id': len(commands) + 1, 'actor': actor, 'action': action, 'target': target}
        observed = call('act', session=session, command=command)
        commands.append(command)
        return observed

    initial = call('observe', session=session)
    assert initial['battle']['turn'] == 0
    assert [f['hp'] for f in initial['battle']['fighters']] == [32, 30]
    poisoned = act(0, 1, 1)
    assert poisoned['battle']['fighters'][0]['resource'] == 9
    assert poisoned['battle']['fighters'][1]['poison'] == {'power': 3, 'turns': 3}
    after = act(1, 0, 0)
    assert [f['hp'] for f in after['battle']['fighters']] == [27, 27]
    assert after['battle']['fighters'][1]['poison']['turns'] == 2
    healed = act(0, 2, 0)
    assert healed['battle']['fighters'][0]['hp'] == 32
    assert healed['battle']['fighters'][0]['resource'] == 6
    checks.append('hand-calculated damage, cost, healing, poison timing')

    before = call('save', session=session)
    pixels = call('frame', session=session)
    assert hashlib.sha256(bytes(pixels['rgba'])).hexdigest() == pixels['sha256']
    for command in [commands[-1], {'id': 4, 'actor': 1, 'action': 0, 'target': 1},
                    {'id': 4, 'actor': 0, 'action': 0, 'target': 1}]:
        call('act', expected=False, session=session, command=command)
        assert call('save', session=session) == before
        assert call('frame', session=session)['sha256'] == pixels['sha256']
    checks.append('duplicate, wrong target and wrong actor reject atomically')

    call('gameplay', session=session, enabled=False)
    detached = call('save', session=session)
    call('act', expected=False, session=session,
         command={'id': 4, 'actor': 1, 'action': 0, 'target': 0})
    assert call('save', session=session) == detached
    restored = create('headless')
    call('restore', session=restored, save=detached)
    assert call('observe', session=restored)['battle'] == healed['battle']
    call('gameplay', session=session, enabled=True)
    call('gameplay', session=restored, enabled=True)
    assert call('save', session=session)['core_save2'] == call('save', session=restored)['core_save2']
    checks.append('real rule removal, detached restore and reinstall preserve authority')

    while True:
        state = call('observe', session=session)['battle']
        if state['winner'] != -1:
            break
        assert len(commands) < 100
        actor = state['turn']
        act(actor, 0, 1 - actor)
    final = call('save', session=session)
    replay = create('headless')
    for command in commands:
        call('act', session=replay, command=command)
    # Lifecycle differs deliberately, so compare authoritative battle state.
    assert call('observe', session=replay)['battle'] == call('observe', session=session)['battle']
    call('restore', session=restored, save=final)
    assert call('save', session=restored)['core_save2'] == final['core_save2']
    checks.append('victory, full save/restore and deterministic public command replay')

    corrupted = dict(final)
    corrupted['core_save2'] = final['core_save2'].replace('"tick":', '"tick":9', 1)
    call('restore', expected=False, session=session, save=corrupted)
    assert call('save', session=session) == final
    checks.append('failed restore preserves live canonical save')
    for handle in (session, restored, replay):
        call('close', session=handle)
        call('observe', expected=False, session=handle)
    checks.append('close invalidates all live handles')
    args.out.mkdir(parents=True, exist_ok=True)
    result = {'ok': True, 'checks': checks, 'commands': len(commands),
              'winner': state['winner'], 'initial_frame_sha256': pixels['sha256'],
              'library_sha256': hashlib.sha256(args.library.read_bytes()).hexdigest(),
              'external_certification': 'UNVERIFIED - internal public API reproducibility only'}
    (args.out / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
    (args.out / 'final-save.json').write_text(json.dumps(final, indent=2) + '\n')
    (args.out / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
    print(json.dumps(result))


if __name__ == '__main__':
    main()
