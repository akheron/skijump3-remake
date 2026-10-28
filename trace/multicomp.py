"""Build a key script that runs a cup past several waits for a key.

The human jumper never starts, so each jump of theirs times out and the
computer jumpers do the rest. The run stops at each screen that waits for a
key by polling, like a new hill screen; this inserts an Enter there and runs
again. The screen polls once for each frame, so the number of polls at the end
of the trace tells the frame where it starts.

The start keys file has the keys that start the cup and the first jump. The
default starts a World Cup. MARGIN (default 45000) is the number of frames
that a run goes on after the last key of the start keys or after the last wait.

Usage: multicomp.py <seed> <waits> <out keys file> [start keys file]
"""
import os
import subprocess
import sys

TRACE_DIR = os.path.dirname(os.path.abspath(__file__))
RUNS = os.environ.get('RUNS', os.path.join(TRACE_DIR, 'runs'))
SENTINEL = (999999, 13, 0)
WORLD_CUP_START = [(10, 13, 0), (20, 13, 0), (30, 13, 0)]


def run(keys, seed, maxframes, name):
    os.makedirs(RUNS, exist_ok=True)
    path = os.path.join(RUNS, f'{name}.keys')
    with open(path, 'w') as f:
        for k in keys:
            f.write('%d %d %d\n' % k)
    env = dict(os.environ, SJ3_MAXFRAMES=str(maxframes), SJ3_TIMEOUT='900')
    out = os.path.join(RUNS, name)
    subprocess.run([os.path.join(TRACE_DIR, 'run-sdl.sh'), path, out, str(seed)],
                   env=env, check=True, stdout=subprocess.DEVNULL)
    return os.path.join(out, 'run', 'Trace.txt')


def analyze(trace, maxframes):
    consumed = 0
    results = 0
    polls = 0
    for line in open(trace):
        context = line.split(' ', 2)[1]
        if context in ('Frame', 'End'):
            continue
        if context == 'KeyPressed':
            polls += 1
            continue
        polls = 0
        if context == 'Keypress':
            consumed += 1
        elif context == 'JumpResult':
            results += 1
    return consumed, maxframes - polls, results


def main():
    seed, waits, out_path = int(sys.argv[1]), int(sys.argv[2]), sys.argv[3]
    start = WORLD_CUP_START
    if len(sys.argv) > 4:
        start = [tuple(map(int, line.split())) for line in open(sys.argv[4]) if line.strip()]
    keys = start + [SENTINEL] * 12
    name = f'multi-{os.path.basename(out_path)}-s{seed}'
    margin = int(os.environ.get('MARGIN', '45000'))
    maxframes = max(k[0] for k in start if k != SENTINEL) + margin
    for wait in range(1, waits + 1):
        trace = run(keys, seed, maxframes, name)
        consumed, stall_frame, results = analyze(trace, maxframes)
        print(f'wait {wait}: {results} jumps, from frame {stall_frame}', flush=True)
        keys.insert(consumed, (stall_frame + 30, 13, 0))
        keys += [SENTINEL] * 12
        maxframes = stall_frame + margin
    with open(out_path, 'w') as f:
        for k in keys:
            f.write('%d %d %d\n' % k)
    print('maxframes', maxframes)


main()
