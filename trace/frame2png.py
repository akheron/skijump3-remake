"""Convert frame dumps (64000 bytes of pixels + 768 bytes of 6-bit palette) to PNG."""
import struct, sys, zlib

def png(path, w, h, rows):
    def chunk(t, d):
        c = struct.pack('>I', len(d)) + t + d
        return c + struct.pack('>I', zlib.crc32(t + d) & 0xffffffff)
    raw = b''.join(b'\0' + r for r in rows)
    with open(path, 'wb') as f:
        f.write(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 2, 0, 0, 0))
                + chunk(b'IDAT', zlib.compress(raw, 6)) + chunk(b'IEND', b''))

def convert(src, dst, scale=2):
    d = open(src, 'rb').read()
    pix, pal = d[:64000], d[64000:64768]
    rgb = [bytes((min(255, pal[i * 3] * 4), min(255, pal[i * 3 + 1] * 4), min(255, pal[i * 3 + 2] * 4))) for i in range(256)]
    rows = []
    for y in range(200):
        row = b''.join(rgb[p] * scale for p in pix[y * 320:(y + 1) * 320])
        rows.extend([row] * scale)
    png(dst, 320 * scale, 200 * scale, rows)

if __name__ == '__main__':
    convert(sys.argv[1], sys.argv[2])
