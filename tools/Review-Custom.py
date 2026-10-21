"""Inspect every candidate PPM and losslessly encode selected native pixels as PNG.

Independent flat-color/outer-clip checks; does not render fonts or replace goldens.
"""
from pathlib import Path
import binascii
import json
import struct
import sys
import zlib

run = Path(sys.argv[1])
output = Path(sys.argv[2])
output.mkdir(parents=True, exist_ok=True)
result = json.loads((run / 'result.json').read_text(encoding='utf-8-sig'))
variants = json.loads(Path('fixtures/m2/ui-v1/variants.json').read_text(encoding='utf-8-sig'))
palettes = {v['id']: v['colors'] for v in variants['variants']}

def ppm(path):
    magic, geometry, maximum, pixels = path.read_bytes().split(b'\n', 3)
    width, height = map(int, geometry.split())
    assert magic == b'P6' and maximum == b'255' and len(pixels) == width * height * 3
    return width, height, pixels

def chunk(kind, data):
    return struct.pack('!I', len(data)) + kind + data + struct.pack('!I', binascii.crc32(kind + data) & 0xffffffff)

def png(path, image):
    w, h, pixels = image
    rows = b''.join(b'\0' + pixels[y*w*3:(y+1)*w*3] for y in range(h))
    raw = b'\x89PNG\r\n\x1a\n'
    raw += chunk(b'IHDR', struct.pack('!IIBBBBB', w, h, 8, 2, 0, 0, 0))
    raw += chunk(b'IDAT', zlib.compress(rows)) + chunk(b'IEND', b'')
    path.write_bytes(raw)

rows = []
for observation in result['observations']:
    name = observation['id']
    data = observation['data']
    width, height, pixels = ppm(run / (name + '.ppm'))
    assert (width, height) == (data['width'], data['height'])
    suffix = name.split('-', 2)[2]
    variant = next(v for v in sorted(palettes, key=len, reverse=True) if suffix.startswith(v + '-'))
    colors = palettes[variant]
    scale = data['scale'] / 100
    def at(x, y):
        index = (int(y) * width + int(x)) * 3
        return list(pixels[index:index+3])
    assert at(0, 0) == colors['surface'][:3]
    assert at(width-1, height-1) == colors['surface'][:3]
    assert at(382*scale, 110*scale) == colors['panel'][:3]
    assert at(632*scale+1, 20*scale) == colors['surface'][:3]
    rows.append({'id': name, 'outer_clip_pixels': 3, 'panel_pixel': 1, 'status': 'passed'})

selected = {
    '1280x720-200-standard-initial': 'initial.png',
    '1280x720-200-standard-effects-toggle': 'reduced-effects.png',
    '1280x720-200-high-contrast-composition-commit': 'high-contrast.png',
    'text-node-3-200-editor': 'complex-editor.png',
    'changed-layout': 'changed-layout.png',
    'missing-icon': 'missing-icon.png',
}
for source, name in selected.items():
    png(output / name, ppm(run / (source + '.ppm')))

# A sheet uses crops of the actual pixels, without resampling. All scales appear.
cells = []
for case in ['arabic', 'indic-prebase', 'mixed-bidi', 'cjk-wrap']:
    cells.append([ppm(run / f'text-{case}-{scale}.ppm') for scale in [100, 150, 200]])
cell_width = max(w for row in cells for w, _, _ in row) + 8
sheet_width = cell_width * 3
sheet_height = sum(max(h for _, h, _ in row) + 8 for row in cells)
sheet = bytearray(sheet_width * sheet_height * 3)
top = 0
for row in cells:
    for column, (w, h, pixels) in enumerate(row):
        for y in range(h):
            dest = ((top+y) * sheet_width + column*cell_width) * 3
            sheet[dest:dest+w*3] = pixels[y*w*3:(y+1)*w*3]
    top += max(h for _, h, _ in row) + 8
png(output / 'text.png', (sheet_width, sheet_height, sheet))
(output / 'pixel-review.json').write_text(json.dumps({'status': 'passed', 'checks_per_capture': 4, 'captures': rows}, indent=2) + '\n', encoding='utf-8')
print(f'Independent pixel checks passed: {len(rows)} captures; PNGs: {output}')
