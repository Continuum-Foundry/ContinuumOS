"""Independent P2-10 semantic/geometry assertions and dense rounded-edge oracle.

Never imports the Rust renderer or rewrites a golden. Complete images stay local.
"""
from pathlib import Path
import json, math, struct, sys, zlib, binascii

run = Path(sys.argv[1])
out = run / 'review'
out.mkdir(exist_ok=True)
read = lambda name: json.loads((run / name).read_text(encoding='utf-8-sig'))
result = read('result.json')
assert result['status'] == 'passed' and result['configurations'] == 30
assert result['semantic_keyboard_checkpoint_parity'] is True
assert result['selected_text']['text_cases'] == 45 and result['selected_text']['integrated_captures'] == 63
matrix = read('matrix.json')
workflows = read('workflows.json')
palettes = json.loads(Path('fixtures/m2/ui-v1/variants.json').read_text())['variants']
palettes = {v['id']: v['colors'] for v in palettes}

def ppm(path):
    magic, size, maximum, pixels = path.read_bytes().split(b'\n', 3)
    w, h = map(int, size.split())
    assert magic == b'P6' and maximum == b'255' and len(pixels) == w*h*3
    return w, h, pixels

def chunk(kind, data):
    return struct.pack('!I', len(data))+kind+data+struct.pack('!I', binascii.crc32(kind+data)&0xffffffff)

def png(path, w, h, rgb):
    raw = b''.join(b'\0'+rgb[y*w*3:(y+1)*w*3] for y in range(h))
    path.write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('!IIBBBBB',w,h,8,2,0,0,0))+chunk(b'IDAT',zlib.compress(raw))+chunk(b'IEND',b''))

assert len(matrix) == len(workflows) == 30
for case, flow in zip(matrix, workflows):
    assert case['id'] == flow['id']
    items = case['semantics']['items']
    byid = {n['id']: n for n in items}
    assert len(byid) == len(items) <= 1088
    assert sum(n['focus'] for n in items) == 1
    for n in items:
        assert n['parent'] is None or n['parent'] in byid
        assert all(math.isfinite(v) for v in n['bounds'])
        assert n['bounds'][2] >= 0 and n['bounds'][3] >= 0
    vw, vh = case['viewport'][2:]
    assert byid['sem.editor']['parent'] == 'sem.editor-window'
    assert byid['sem.editor']['bounds'] == [104,76,vw-332,vh-132]
    assert byid['sem.files-window']['bounds'] == [vw-200,8,192,min(vh-32,1488)]
    assert byid['sem.files-window']['count'] == 10000
    rows = [n for n in items if n['position'] is not None]
    assert len(rows) <= 64 and sum(n['selected'] for n in rows) == 1
    assert next(n for n in rows if n['selected'])['position'] == 21
    assert byid['sem.editor']['value'] == 'Draft: notes\n'
    assert byid['sem.search']['value'] == 'notes'
    assert byid['sem.reduced-motion']['checked'] and byid['sem.reduced-motion']['focus']
    assert 'settings.motion@1' in byid['sem.reduced-motion']['actions']
    assert case['work'] == {'measured':12,'arranged':12,'depth':3}
    width, height, rgb = ppm(run / (case['id']+'.ppm'))
    variant = case['id'].split('-',2)[2]
    colors = palettes[variant]
    assert list(rgb[:3]) == colors['surface'][:3]
    assert list(rgb[-3:]) == colors['surface'][:3]
    if case['id'] in ('1280x720-150-standard','1280x720-200-high-contrast'):
        png(out / (case['id']+'.png'),width,height,rgb)

texts = read('visual-text.json')
assert len(texts) == 21
authored = {
    1: ['C','a','f','e\u0301',' ','f','i',' ','f','f','i'],
    2: ['👩🏽‍💻',' ','🏳️‍🌈'],
    4: ['न','म','स्ते',' ','दु','नि','या'],
}
for case in texts:
    text = case['text']
    encoded = text.encode()
    for stop in case['stops']:
        encoded[:stop['byte']].decode()  # independently reject a UTF-8 interior stop
    if case['id'] in authored:
        graphemes = authored[case['id']]
        assert ''.join(graphemes) == text
        boundaries = {0}
        at = 0
        for g in graphemes:
            at += len(g.encode()); boundaries.add(at)
        assert all(s['byte'] in boundaries for s in case['stops'])
    for line in {s['line'] for s in case['stops']}:
        xs = [s['x'] for s in case['stops'] if s['line'] == line]
        assert xs == sorted(xs)

quality = []
for case in read('quality.json'):
    w,h,rgb = ppm(run / (case['id']+'.ppm'))
    scale = case['scale']/100
    x,y,rw,rh = case['rect']; radius = case['radius']
    def inside(px,py):
        if not (x <= px < x+rw and y <= py < y+rh): return False
        cx = min(max(px,x+radius),x+rw-radius)
        cy = min(max(py,y+radius),y+rh-radius)
        return math.hypot(px-cx,py-cy) <= radius
    errors=[]; edges=0
    for py in range(h):
        for px in range(w):
            corners = [inside((px+dx)/scale,(py+dy)/scale) for dx,dy in ((0,0),(1,0),(0,1),(1,1))]
            # Deep interior/exterior is exact. Dense integration only at edges.
            center = inside((px+.5)/scale,(py+.5)/scale)
            if all(corners) and center: expected=255
            elif not any(corners) and not center: expected=0
            else:
                edges+=1
                coverage=sum(inside((px+(sx+.5)/64)/scale,(py+(sy+.5)/64)/scale) for sy in range(64) for sx in range(64))
                expected=round(coverage*255/4096)
            actual=rgb[(py*w+px)*3:(py*w+px)*3+3]
            assert actual[0] == actual[1] == actual[2]
            error=abs(actual[0]-expected)
            assert error <= 16, (case['id'],px,py,actual[0],expected)
            if expected in (0,255): assert error == 0
            errors.append(error)
    quality.append({'id':case['id'],'max_channel_error':max(errors),'edge_pixels':edges,'status':'passed'})

report={'status':'passed','configurations':len(matrix),'semantic_snapshots':60,'authored_visual_text_cases':len(texts),'dense_edge_checks':quality,'tolerances':'unchanged edge channel <=16; exact flat/exterior pixels; 64x64 mathematical sampling'}
(out/'review.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
