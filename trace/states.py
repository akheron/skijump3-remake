"""Print decoded JumpState/LandState records: states.py Trace.txt [step] [fields...]"""
import struct, sys
path = sys.argv[1]; step = int(sys.argv[2]) if len(sys.argv) > 2 else 1
fields = sys.argv[3:]
n = 0
for line in open(path):
    p = line.split()
    if len(p) > 1 and p[1] in ('JumpState', 'LandState', 'JumpResult'):
        d = {}
        for kv in p[2:]:
            k, v = kv.split('=', 1); t, val = v.split(':', 1)
            d[k] = round(struct.unpack('>f', bytes.fromhex(val))[0], 3) if t == 'f32' else val
        if p[1] != 'JumpResult' and n % step: n += 1; continue
        n += 1
        print(p[1][:4], ' '.join(f'{k}={d[k]}' for k in (fields or d.keys()) if k in d))
