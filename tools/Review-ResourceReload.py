"""Independent consumer of P2-11 state, pixel, lifecycle and timestamp evidence."""
import hashlib
import json
import sys
from pathlib import Path
import struct
import zlib
import binascii

def load(p):
    return json.loads(Path(p).read_text(encoding='utf-8-sig'))

def require(ok, message):
    if not ok:
        raise AssertionError(message)

def ppm(path):
    magic, geometry, maximum, pixels=path.read_bytes().split(b'\n',3)
    w,h=map(int,geometry.split())
    require(magic==b'P6' and maximum==b'255' and len(pixels)==w*h*3,'PPM encoding')
    return w,h,pixels

def png_rgba(path):
    data=path.read_bytes(); require(data[:8]==b'\x89PNG\r\n\x1a\n','PNG signature')
    offset=8; compressed=bytearray(); header=None
    while offset<len(data):
        size=struct.unpack_from('!I',data,offset)[0];kind=data[offset+4:offset+8]
        value=data[offset+8:offset+8+size]
        crc=struct.unpack_from('!I',data,offset+8+size)[0]
        require(binascii.crc32(kind+value)&0xffffffff==crc,'PNG CRC')
        if kind==b'IHDR': header=struct.unpack('!IIBBBBB',value)
        if kind==b'IDAT': compressed.extend(value)
        offset+=12+size
    w,h,depth,color,compression,filtering,interlace=header
    require(depth==8 and color in (2,6) and compression==filtering==interlace==0,'PNG format')
    channels=4 if color==6 else 3;stride=w*channels
    raw=zlib.decompress(compressed);require(len(raw)==h*(stride+1),'PNG payload bound')
    previous=bytearray(stride);output=bytearray()
    def paeth(a,b,c):
        p=a+b-c;pa,pb,pc=abs(p-a),abs(p-b),abs(p-c)
        return a if pa<=pb and pa<=pc else b if pb<=pc else c
    for y in range(h):
        at=y*(stride+1);kind=raw[at];row=bytearray(raw[at+1:at+1+stride])
        require(kind<=4,'PNG filter')
        for x in range(stride):
            a=row[x-channels] if x>=channels else 0;b=previous[x];c=previous[x-channels] if x>=channels else 0
            predictor=[0,a,b,(a+b)//2,paeth(a,b,c)][kind]
            row[x]=(row[x]+predictor)&255
        if channels==4: output.extend(row)
        else:
            for x in range(0,stride,3):output.extend(row[x:x+3]);output.append(255)
        previous=row
    return w,h,output

reload_dir, native_dir = map(Path, sys.argv[1:3])
r, n = load(reload_dir/'result.json'), load(native_dir/'native-reload.json')
require(r['status'] == n['status'] == 'passed', 'Harness outcome')
fields = ['document', 'document_sha256', 'revision', 'model_revision',
          'checkpoint_sha256', 'selection', 'dirty', 'undo_len', 'redo_len',
          'document_scroll', 'scroll_row', 'search', 'focus', 'window']
for field in fields:
    require(r['before'][field] == r['after'][field], f'Logical continuity: {field}')
require(r['before']['undo_len'] > 0 and r['before']['redo_len'] > 0, 'History fixture empty')
require(r['before']['dirty'] and r['before']['scroll_row'] == 537, 'Unsaved/scroll fixture')
require(r['checkpoint_exact'] and r['provider_address_stable'], 'Owner continuity')
require(r['before_pixels']['rgba_sha256'] != r['after_pixels']['rgba_sha256'], 'Pixel change')
for filename, result in [('before.ppm',r['before_pixels']),('after.ppm',r['after_pixels'])]:
    raw=(reload_dir/filename).read_bytes()
    require(hashlib.sha256(raw).hexdigest()==result['image_sha256'], f'Capture identity: {filename}')
    w,h,pixels=ppm(reload_dir/filename)
    require((w,h)==(1280,720),'Surface geometry')
items=r['semantics_after']['items']
status=next(i for i in items if i['id']=='sem.status')
require(status['parent']=='sem.sidebar','Structural semantic parent')
require(any(i['name']=='Reloaded settings' for i in items),'Semantic label publication')
cases={row['case']: row for row in r['negative']}
expected={'withheld-readers-headroom','truncated','version','unknown','stale','oversized',
          'duplicate-id','cycle','missing-font','missing-image','corrupt-asset','font-version',
          'numeric-overflow','nonfinite','no-focus','missing-shell','bad-action',
          'render-failure','failed-builder-preserves-output','failed-build'}
require(set(cases)==expected,'Missing negative cases')
for row in cases.values():
    require(bool(row['reason']),'Missing precise reason')
require(cases['withheld-readers-headroom']['old_buffer_preserved'] and
        cases['withheld-readers-headroom']['reclaimed_on_release'],'Ownership release')
require('Exhausted' in cases['render-failure']['reason'],'Stage paint pressure not exercised')
require(' queued' in r['queued']['document'],'Ordered queued text')
require(r['composition']['composition_active'],'Compatible composition cancelled')
require(not r['removed']['composition_active'] and r['removed']['focus']=='search','Removed focus policy')
for key in ['drag','removed','overflow']:
    require(r[key]['pressed_keys']==r[key]['pressed_buttons']==0 and not r[key]['capture'],f'Stuck input: {key}')
require(r['overflow']['dropped']==257 and r['coalesced']==1,'Queue/save accounting')
require(r['query']['live_generations']==1,'Resource roots retained after release')
published=[row for row in r['trace'] if row['outcome']=='published']
for row in published:
    times=[row[k] for k in ['receive_us','validated_us','staged_us','published_us']]
    require(times==sorted(times),'Publication phase clock ordering')
    require(row['semantic_revision']>0 and row['stage_bytes']<=16*1024*1024,'Staging accounting')
require(published[0]['first_frame_us']>=published[0]['published_us'],'First completed host frame')
require(n['lifecycle_before']==n['lifecycle_after'],'Native process/window lifecycle')
require(n['bounds_before']==n['bounds_after'],'Native geometry continuity')
for field in fields:
    require(n['before']['state'][field]==n['after']['state'][field],f'Native state: {field}')
require(n['after']['host_time_us']>n['before']['host_time_us'],'Native monotonic clock')
require(n['before']['resources']['active_generation']==7 and n['after']['resources']['active_generation']==8,'Actual native generation')
require(n['partial']['resources']['status']['outcome']=='rejected' and
        n['retry']['resources']['status']['outcome']=='rejected','Native partial/retry outcomes')
require(n['partial']['state']['checkpoint_sha256']==n['before']['state']['checkpoint_sha256'],'Partial-write state')
require(n['partial']['capture']['rgba_sha256']==n['before']['capture']['rgba_sha256'],'Partial-write pixels')
require(n['after']['selected_semantics']['scene_revision']==n['after']['scene_revision'],'Mixed frame/semantic revisions')
require(n['after']['resources']['status']['semantic_revision']==n['after']['scene_revision'],'Mixed staged/visible revisions')
require(n['final']['state']['revision']==n['before']['state']['revision']+1 and
        ' continued' in n['final']['state']['document'],'Continued native edit')
for tag in ['before','after']:
    w,h,rgba=png_rgba(native_dir/f'native-{tag}.png')
    require((w,h)==(1280,720),'Native image geometry')
    require(hashlib.sha256(rgba).hexdigest()==n[tag]['capture']['rgba_sha256'],f'Native presented pixels: {tag}')
require(n['before']['capture']['rgba_sha256']!=n['after']['capture']['rgba_sha256'],'Native visible change')
report={'status':'passed','logical_fields':fields,'negative_cases':len(cases),'publications':len(published),
        'native_generation':[7,8],'native_pid':n['lifecycle_before']['worker_pid'],
        'native_same_window':True,'pixel_identities_checked':4,'semantics_separate':True,
        'native_completed_build_to_submitted_ms':n['native_completed_build_to_submitted_ms'],
        'scope':'ephemeral hosted; Form.Update is not a scanout timestamp; no performance acceptance'}
(reload_dir/'independent-review.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
print(json.dumps(report,indent=2))
