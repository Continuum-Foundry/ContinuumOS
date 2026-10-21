"""Lossless PGM-to-PNG contact sheet using only Python's standard library.

Rows: combining, ligature, mono spacing, Arabic, Hebrew, Indic prebase, Indic conjunct, mixed
bidi, CJK wrap. Columns: scale 1.0/1.5/2.0. No fonts or rasterizer are substituted.
"""
from pathlib import Path
import binascii
import struct
import sys
import zlib

directory = Path(sys.argv[1])
output = Path(sys.argv[2])
names = ['combining', 'ligature', 'mono-spacing', 'arabic', 'hebrew', 'indic-prebase',
         'indic-conjunct', 'mixed-bidi', 'cjk-wrap']
rows = []
for name in names:
    row = []
    for scale in [100, 150, 200]:
        raw = (directory / f'{name}-{scale}.pgm').read_bytes()
        magic, geometry, maximum, pixels = raw.split(b'\n', 3)
        width, height = map(int, geometry.split())
        assert magic == b'P5' and maximum == b'255' and len(pixels) == width * height
        row.append((width, height, pixels))
    rows.append(row)
cell_width = max(w for row in rows for w, _, _ in row) + 8
width = cell_width * 3
height = sum(max(h for _, h, _ in row) + 8 for row in rows)
canvas = bytearray(width * height)
top = 0
for row in rows:
    for column, (w, h, pixels) in enumerate(row):
        for y in range(h):
            start = (top + y) * width + column * cell_width
            canvas[start:start+w] = pixels[y*w:(y+1)*w]
    top += max(h for _, h, _ in row) + 8

def chunk(kind, data):
    return (struct.pack('!I', len(data)) + kind + data
            + struct.pack('!I', binascii.crc32(kind+data) & 0xffffffff))

scanlines = b''.join(b'\0' + canvas[y*width:(y+1)*width] for y in range(height))
png = b'\x89PNG\r\n\x1a\n'
png += chunk(b'IHDR', struct.pack('!IIBBBBB', width, height, 8, 0, 0, 0, 0))
png += chunk(b'tEXt', b'Description\0Rows: ' + ', '.join(names).encode() + b'; columns: 100,150,200')
png += chunk(b'IDAT', zlib.compress(scanlines)) + chunk(b'IEND', b'')
output.write_bytes(png)
print(output)
