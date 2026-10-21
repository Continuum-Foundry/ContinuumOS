"""Independent host-only HarfBuzz oracle using the pinned QEMU distribution DLL.

Reads authored single-script corpus entries; emits glyphs, UTF-8 clusters and
font-unit advances/offsets. This does not share Rust shaping/layout code.
"""
import ctypes as c
import hashlib
import json
import os
from pathlib import Path
import sys

root = Path(__file__).resolve().parent.parent
dll_dir = root / 'work/tools/qemu'
cookie = os.add_dll_directory(str(dll_dir))
hb_path = dll_dir / 'libharfbuzz-0.dll'
hb = c.CDLL(str(hb_path))

def api(name, result, *args):
    fn = getattr(hb, name)
    fn.restype = result
    fn.argtypes = list(args)
    return fn

p = c.c_void_p
api('hb_version_string', c.c_char_p)
api('hb_blob_create', p, p, c.c_uint, c.c_int, p, p)
api('hb_face_create', p, p, c.c_uint)
api('hb_font_create', p, p)
api('hb_ot_font_set_funcs', None, p)
api('hb_buffer_create', p)
api('hb_buffer_add_utf8', None, p, c.c_char_p, c.c_int, c.c_uint, c.c_int)
api('hb_direction_from_string', c.c_int, c.c_char_p, c.c_int)
api('hb_script_from_string', c.c_uint32, c.c_char_p, c.c_int)
api('hb_buffer_set_direction', None, p, c.c_int)
api('hb_buffer_set_script', None, p, c.c_uint32)
api('hb_buffer_set_cluster_level', None, p, c.c_int)
api('hb_shape', None, p, p, p, c.c_uint)

class Info(c.Structure):
    _fields_ = [('codepoint', c.c_uint32), ('mask', c.c_uint32),
                ('cluster', c.c_uint32), ('var1', c.c_uint32), ('var2', c.c_uint32)]
class Position(c.Structure):
    _fields_ = [('advance_x', c.c_int32), ('advance_y', c.c_int32),
                ('offset_x', c.c_int32), ('offset_y', c.c_int32), ('var', c.c_uint32)]

api('hb_buffer_get_glyph_infos', c.POINTER(Info), p, c.POINTER(c.c_uint))
api('hb_buffer_get_glyph_positions', c.POINTER(Position), p, c.POINTER(c.c_uint))
for kind in ['blob', 'face', 'font', 'buffer']:
    api('hb_' + kind + '_destroy', None, p)

lock = json.loads((root / 'fixtures/m2/text-v1/fonts.lock.json').read_text(encoding='utf-8-sig'))
corpus = json.loads((root / 'fixtures/m2/text-v1/corpus.json').read_text(encoding='utf-8-sig'))
rows = []
for case in corpus['cases']:
    if 'oracle' not in case:
        continue
    spec = case['oracle']
    asset = lock['fonts'][spec['font']]
    data = (root / asset['path']).read_bytes()
    assert hashlib.sha256(data).hexdigest() == asset['sha256']
    raw = c.create_string_buffer(data)
    blob = hb.hb_blob_create(raw, len(data), 0, None, None)  # duplicate
    face = hb.hb_face_create(blob, 0)
    font = hb.hb_font_create(face)
    hb.hb_ot_font_set_funcs(font)
    buffer = hb.hb_buffer_create()
    text = case['text'].encode('utf-8')
    hb.hb_buffer_add_utf8(buffer, text, len(text), 0, len(text))
    hb.hb_buffer_set_direction(buffer, hb.hb_direction_from_string(spec['direction'].encode(), -1))
    hb.hb_buffer_set_script(buffer, hb.hb_script_from_string(spec['script'].encode(), -1))
    hb.hb_buffer_set_cluster_level(buffer, 0)
    hb.hb_shape(font, buffer, None, 0)
    count = c.c_uint()
    infos = hb.hb_buffer_get_glyph_infos(buffer, c.byref(count))
    positions = hb.hb_buffer_get_glyph_positions(buffer, c.byref(count))
    rows.append({'id': case['id'], 'font': spec['font'], 'glyphs': [
        {'id': infos[i].codepoint, 'cluster': infos[i].cluster,
         'advance': positions[i].advance_x,
         'offset': [positions[i].offset_x, positions[i].offset_y]}
        for i in range(count.value)]})
    for kind, obj in [('buffer', buffer), ('font', font), ('face', face), ('blob', blob)]:
        getattr(hb, 'hb_' + kind + '_destroy')(obj)

result = {'schema_version': 1, 'oracle': 'native HarfBuzz from pinned local QEMU inputs',
          'version': hb.hb_version_string().decode(),
          'dll_sha256': hashlib.sha256(hb_path.read_bytes()).hexdigest(),
          'corpus_sha256': hashlib.sha256((root / 'fixtures/m2/text-v1/corpus.json').read_bytes()).hexdigest(),
          'cases': rows}
out = json.dumps(result, ensure_ascii=False, indent=2) + '\n'
if len(sys.argv) == 3 and sys.argv[1] == '--verify':
    assert json.loads(Path(sys.argv[2]).read_text(encoding='utf-8')) == result, 'Independent shaping reference changed'
    print('Independent HarfBuzz reference matches all', len(rows), 'cases.')
elif len(sys.argv) == 2:
    Path(sys.argv[1]).write_text(out, encoding='utf-8')
else:
    raise SystemExit('usage: Reference-TextShape.py OUTPUT | --verify EXPECTED')
