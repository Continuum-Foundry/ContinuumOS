"""Independent optical-framebuffer and display-output oracle. No guest renderer import."""
import argparse, copy, importlib.util, json
from pathlib import Path
import p3_acceptance as base
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('p404_runtime',ROOT/'tools/Review-P4-02.py')
runtime=importlib.util.module_from_spec(spec);spec.loader.exec_module(runtime)
need,rows=runtime.need,runtime.rows

def pixels(w,h):
    data=bytearray(w*h*3)
    bars=[(255,0,0),(0,255,0),(0,0,255),(17,34,51),(128,64,192),(0,0,0)]
    for y in range(h):
        for x in range(w):
            if min(x,y,w-1-x,h-1-y)<8: rgb=(255,255,255)
            elif y<40 and x<40: rgb=(255,0,255)
            elif y<40 and x>=w-40: rgb=(0,255,255)
            elif y>=h-40 and x<40: rgb=(255,255,0)
            elif y>=h-40 and x>=w-40: rgb=(255,128,0)
            else: rgb=bars[x*6//w]
            data[(y*w+x)*3:(y*w+x+1)*3]=bytes(rgb)
    for x in [0,1,w-2,w-1]: data[((h//2)*w+x)*3:((h//2)*w+x+1)*3]=bytes([11,22,33])
    return bytes(data)

def check(records,w,h):
    runtime.check(records,True)
    out=rows(records,'m1.display_output');need(len(out)==1,'display descriptor')
    d=out[0];need((int(d['width']),int(d['height']))==(w,h),'actual output dimensions')
    need(d['bpp']=='32' and d['red']=='16' and d['green']=='8' and d['blue']=='0','actual channel format')
    need(int(d['pitch'])>=w*4 and int(d['bytes'])==int(d['pitch'])*h,'actual pitch extent')
    need(all(d[k]==v for k,v in dict(cache='uc',user='false',writable='true',nx='true',initial_clear='complete',padding='zero',fence='cpu-copy',vsync='false').items()),'mapping/copy contract')
    need(not rows(records,'m1.display_unavailable'),'output unavailable')
    unwind=rows(records,'m1.display_map_unwind');need(len(unwind)==1 and unwind[0]['pages']=='2' and unwind[0]['retained']=='0','mapping unwind')
    pattern=rows(records,'m1.display_pattern');need(len(pattern)==1,'completed pattern')
    p=pattern[0];need(p['rows']==str(h) and p['width']==str(w) and p['source_released']=='true' and p['scanout_observed']=='false','copy completion')
    calls=rows(records,'m1.display_call'); launches=rows(records,'m1.desktop_launch')
    for c in calls:
        matching=[v for v in launches if int(v['process'])+1==int(c['process']) and v['generation']==c['generation']]
        need(len(matching)==1 and runtime.ROLES[int(c['role'])]==matching[0]['role'],'trusted caller incarnation')
        if c['role']!='1':need(c['accepted']=='false','app device/capture access')
    for role in range(7):
        rc=[c for c in calls if c['role']==str(role)]
        if role!=1:need({int(c['op']) for c in rc}==set(range(0x4100,0x4105)),'all role operation denials')
    accepted=[c for c in calls if c['accepted']=='true']
    need([int(c['op']) for c in accepted]==[0x4100,0x4101],'display acquisition/query')
    need(len([c for c in calls if c['role']=='1' and c['accepted']=='false'])==5,'stale/capture/map/source/capacity denials')
    release=rows(records,'m1.display_release');need(len(release)==1 and all(release[0][k]==p[k] for k in ['process','generation']) and release[0]['completed']=='true','owner release')
    return d

def review(folder,w,h):
    folder=Path(folder);manifest=base.read(folder/'build-manifest.json')
    label=f'display-{720 if h==720 else 1080}'
    run=folder/(label+'-run');package=folder/(label+'-package');image=base.read(package/'iso-manifest.json');case=base.read(run/'case-result.json')
    need(case['status']=='passed' and not case['timed_out'] and case['cleanup_reaped'] and case['deadline_ms']==30000 and case['emulator_test_exit']==33,'optical guest result')
    need(base.digest(folder/'kernel.elf')==manifest['kernel_sha256']==image['kernel_sha256'],'kernel identity')
    need(base.digest(package/'continuum.iso')==image['iso_sha256']==case['iso_sha256'],'exact ISO identity')
    need(manifest['flavor']=='display-test' and image['build_flavor']=='display-test','test flavor')
    records=runtime.raw(run/'serial.log',manifest['build_id'],image['host_session_id'])
    descriptor=check(records,w,h)
    ppm=(run/'display.ppm').read_bytes();header=f'P6\n{w} {h}\n255\n'.encode()
    need(ppm.startswith(header),'QEMU capture dimensions/format')
    blank=(run/'display-blank.ppm').read_bytes();need(blank==header+bytes(w*h*3),'initial displayed clear/background')
    reference=pixels(w,h);need(ppm[len(header):]==reference,'display pixels differ from independent CPU reference')
    (folder/'cpu-reference.ppm').write_bytes(header+reference)
    qmp=(run/'qemu.stdout.log').read_text() if (run/'qemu.stdout.log').exists() else ''
    replies=[json.loads(line) for line in qmp.splitlines() if line.strip()]
    need(not any("error" in r for r in replies) and {r.get("id") for r in replies if "return" in r}=={0,1,2,3,4,5,6},"QMP capture acknowledgement")
    # Generic raw-record criterion removal audit.
    negatives=0
    for event in ['m1.display_output','m1.display_map_unwind','m1.display_pattern','m1.display_release','m1.display_call']:
        try:check([r for r in records if r['event']!=event],w,h)
        except (AssertionError,ValueError,KeyError):negatives+=1
        else:raise AssertionError('missing record mutation accepted: '+event)
    result=dict(status='passed',build_id=manifest['build_id'],kernel_sha256=manifest['kernel_sha256'],iso_sha256=image['iso_sha256'],capture_sha256=base.digest(run/'display.ppm'),blank_capture_sha256=base.digest(run/'display-blank.ppm'),reference_sha256=base.digest(folder/'cpu-reference.ppm'),width=w,height=h,descriptor=descriptor,criterion_mutations=negatives,scope='CPU reference / synchronous copy / QEMU displayed pixels; no scanout timing or VSync')
    base.save(folder/'display-review.json',result);return result

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('folder');p.add_argument('--width',type=int,required=True);p.add_argument('--height',type=int,required=True);a=p.parse_args();print(json.dumps(review(a.folder,a.width,a.height),indent=2))
