"""Independent P4-05 raw transport, lifetime and actual optical pixel oracle.

Does not import the guest codec, broker, renderer or client producer.
"""
import argparse, collections, importlib.util, json, copy
from pathlib import Path
import p3_acceptance as base
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('p405_runtime',ROOT/'tools/Review-P4-02.py')
runtime=importlib.util.module_from_spec(spec);spec.loader.exec_module(runtime)
need,rows=runtime.need,runtime.rows
SCENARIOS={'surface-720':1,'surface-1080':1,'surface-move':2,'surface-hide':3,'surface-remove':4,'surface-denied':5,'surface-death':6,'surface-failure':7,'surface-stall':8}

def pixels(w,h,scenario):
    # Separate rectangle-domain oracle, with independently authored literal colors,
    # placements and clip edges. No guest loops/stride/source buffer construction.
    data=bytearray(bytes((12,18,24))*w*h)
    for x0,y0,x1,y1,color in [(0,25,150,145,(180,40,20))]:
        for y in range(y0,y1):data[(y*w+x0)*3:(y*w+x1)*3]=bytes(color)*(x1-x0)
    if scenario not in (3,4,6):
        left,top=(248,164) if scenario==2 else (88,74)
        for y in range(top,top+108):
            for x in range(left,left+144):
                i=(y*w+x)*3
                # Premultiplied encoded-sRGB over opaque destination, exact integer rounding.
                rgb=[20+(data[i]*127+127)//255,90+(data[i+1]*127+127)//255,60+(data[i+2]*127+127)//255]
                data[i:i+3]=bytes(rgb)
    return bytes(data)

def runtime_baseline(records,scenario):
    baseline=copy.deepcopy([r for r in records if (r['event']!='m1.user_log' or r['fields']['token']=='desktop-heap-100-settled') and not (scenario==8 and r['event']=='m1.desktop_watchdog' and r['fields']['role']=='compositor')])
    changed_sessions=0
    for row in baseline:
        f=row['fields']
        if row['event']=='m1.desktop_launch' and f['role']=='editor' and f['mode']=='2':
            need(f['authority_session']=='2002','trusted synthetic second-session binding');f['authority_session']='1001';changed_sessions+=1
        if scenario>=7 and row['event']=='m1.desktop_reap' and f['role']=='compositor' and f['result']!='513':
            need(f['result']==('90' if scenario==7 else '768'),'actual compositor failure outcome');f['result']='0'
    need(changed_sessions==1,'second-session launch')
    return baseline

def check(records,w,h,scenario):
    allowed={'surface-transport-denied','surface-client-negatives-passed','surface-client-a','surface-client-b','surface-final-presented','surface-initial-presented','surface-inventory-zero','surface-owner-death-reclaimed','surface-wrong-session-denied','surface-compositor-cancelled'}
    logs=rows(records,'m1.user_log')
    need(all(r['token']=='desktop-heap-100-settled' or r['token'] in allowed for r in logs),'unexpected client assertion')
    # Preserve the complete existing nine-owner / 900-cycle acceptance assertions.
    baseline=runtime_baseline(records,scenario)
    runtime.check(baseline,True)
    counts=collections.Counter(r['token'] for r in logs)
    need(counts['surface-client-negatives-passed']==2 and counts['surface-client-a']==counts['surface-client-b']==1,'independent client completions')
    need(counts['surface-initial-presented']==1 and counts['surface-final-presented']==1 and counts['surface-inventory-zero']==(0 if scenario>=7 else 1),'presentation / private storage settlement')
    need(counts['surface-wrong-session-denied']==(0 if scenario>=7 else 1) and counts['surface-compositor-cancelled']==(2 if scenario==8 else 1 if scenario==7 else 0),'session and compositor death outcomes')
    need(counts['surface-owner-death-reclaimed']==(1 if scenario==6 else 0),'owner death mode')
    frames=rows(records,'m1.surface_frame');need(len(frames)==2 and [int(f['frame']) for f in frames]==[1,2] and all(int(f['rows'])==h and int(f['width'])==w and int(f['height'])==h and f['source_released']=='true' for f in frames),'complete initial and final frame copies')
    out=rows(records,'m1.display_output');need(len(out)==1,'display metadata')
    d=out[0];need((int(d['width']),int(d['height']))==(w,h),'output dimensions')
    need(all(d[k]==v for k,v in dict(bpp='32',red='16',green='8',blue='0',cache='uc',user='false',writable='true',nx='true',initial_clear='complete',fence='cpu-copy',vsync='false').items()),'checked scanout properties')
    launches=rows(records,'m1.desktop_launch')
    requests=rows(records,'m1.surface_request');responses=rows(records,'m1.surface_response')
    need(requests and len(requests)==len(responses)+(2 if scenario==8 else 1 if scenario==7 else 0),'complete request/reply/cancellation coverage')
    tickets={};seqs={};created={};status=collections.defaultdict(list)
    for q in requests:
        t=int(q['ticket']);need(t not in tickets,'duplicate request ticket');tickets[t]=q
        matching=[v for v in launches if int(v['process'])+1==int(q['process']) and v['generation']==q['incarnation']]
        need(len(matching)==1 and matching[0]['role']==['shell','editor'][int(q['client'])] and matching[0]['principal']==q['principal'] and matching[0]['authority_session']==q['authority_session'] and q['authority_session'] in ('1001','2002'),'trusted transport provenance')
        key=(q['process'],q['incarnation']);need(int(q['sequence'])>seqs.get(key,0),'replayed endpoint sequence');seqs[key]=int(q['sequence'])
        need(int(q['request'])==int(q['sequence'])+100 and 128<=int(q['bytes'])<=4096,'independent client request shape')
    seen=set()
    for r in responses:
        t=int(r['ticket']);need(t in tickets and t not in seen,'unmatched/duplicate response');seen.add(t);q=tickets[t]
        matching=[v for v in launches if int(v['process'])+1==int(r['process']) and v['generation']==r['incarnation']]
        need(len(matching)==1 and matching[0]['role']=='compositor','reply authority')
        need(r['request']==q['request'],'response correlation')
        k,c,st=int(q['kind']),int(q['client']),int(r['status']);status[c].append((k,st))
        if k==1 and st==0:created[c]=int(r['value'])
        if k==2 and st==0:need(int(q['surface'])==created[c],'cross-owner acquired surface')
    for c in range(2):
        pairs=collections.Counter(status[c])
        for pair,n in { (1,0):1,(2,0):3,(2,5):1,(2,3):1,(4,1):1,(4,3):1,(4,0):1,(4,4):1,(3,4):1,(6,4):1,(6,0):2,(6,3):2,(8,0):1,(9,0):(0 if scenario==8 or scenario==7 and c==0 else 1)}.items():need(pairs[pair]>=n,'missing accepted/denied lease transition '+str((c,pair)))
        need(pairs[(3,0)]==20,'complete independently submitted pixel chunks')
        need(pairs[(2,2)]+pairs[(2,3)]>=3,'forged/stale surface denials')
    if scenario==5:need(collections.Counter(status[1])[(5,1)]==1 and collections.Counter(status[1])[(4,1)]==2,'malformed damage/stride denials')
    if scenario<7:
        wrong=[q for q in requests if q['authority_session']=='2002'];need(len(wrong)==2 and {q['kind'] for q in wrong}=={'1','8'},'actual wrong-session request')
        need(all(next(r for r in responses if r['ticket']==q['ticket'])['status']=='2' for q in wrong),'wrong session accepted')
    else:
        cancelled=set(tickets)-seen;need(len(cancelled)==(2 if scenario==8 else 1) and all(tickets[t]['kind']=='9' for t in cancelled),'compositor cancellation identity')
        if scenario==8:
            watchdog=[r for r in rows(records,'m1.desktop_watchdog') if r['role']=='compositor'];need(len(watchdog)==1 and int(watchdog[0]['residence_without_wait_ns'])>=1000000000,'stalled compositor watchdog')
    denied=rows(records,'m1.surface_denied');need(denied,'transport denial evidence')
    for role in [0,2,5,6]:need({int(r['op']) for r in denied if r['role']==str(role)}==set(range(0x4200,0x4205)),'unentitled endpoint operation')
    dead=rows(records,'m1.surface_transport_dead');need(len(dead)==12 and dead[-1]['queued']=='0','bounded mailbox teardown')
    final=next(i for i,r in enumerate(records) if r['event']=='m1.user_log' and r['fields']['token']=='surface-final-presented')
    if scenario==6:
        first_editor=next(i for i,r in enumerate(records) if r['event']=='m1.desktop_reap' and r['fields']['role']=='editor')
        need(first_editor<final,'death did not precede exposed-region presentation')
    return {'requests':len(requests),'denials':len(denied),'descriptor':d}

def review(folder,label):
    folder=Path(folder);scenario=SCENARIOS[label];w,h=(1920,1080) if label=='surface-1080' else (1280,720)
    manifest=base.read(folder/'build-manifest.json');inputs=base.read(folder/'build-inputs.json');run=folder/(label+'-run');image=base.read(folder/(label+'-package')/'iso-manifest.json');case=base.read(run/'case-result.json')
    need(case['status']=='passed' and not case['timed_out'] and case['cleanup_reaped'] and case['deadline_ms']==30000 and case['emulator_test_exit']==33,'optical result')
    need(manifest['flavor']==image['build_flavor']=='surface-test','surface feature')
    need(base.digest(folder/'kernel.elf')==manifest['kernel_sha256']==image['kernel_sha256'],'kernel identity')
    need(base.digest(folder/(label+'-package')/'continuum.iso')==image['iso_sha256']==case['iso_sha256'],'exact optical ISO identity')
    for name,sha in inputs['sources'].items():need(base.digest(ROOT/name)==sha,'stale implementation '+name)
    images=base.read(folder/'user-images.json');need(images['surface_runtime'] is True and images['surface_scenario']==scenario,'trusted scenario')
    need(len(images['images'])==3 and len({i['sha256'] for i in images['images'][:2]})==2,'independent client binaries')
    elf_images=[]
    for name in ['user_a','user_b','service']:
        row=next(row for row in images['images'] if row['name']==name)
        need(base.digest(folder/(name+'.elf'))==row['sha256']==image['payload']['boot/'+name+'.elf'],'independently linked and optically packaged client bytes')
        elf_images.append(runtime.elf(folder/(name+'.elf')))

    records=runtime.raw(run/'serial.log',manifest['build_id'],image['host_session_id']);observations=check(records,w,h,scenario)
    inventory=rows(records,'m1.desktop_inventory')[0];memory=rows(records,'m1.runtime_memory')[0]
    expected=sum(runtime.HEAPS)*runtime.MIB+sum((elf_images[i%3]['load_pages']+19)*4096 for i in range(7))
    need(int(inventory['committed'])==expected and int(inventory['allocations'])==sum(elf_images[i%3]['load_segments']+5 for i in range(7)),'independent physical/allocation reconciliation')
    per_role_fixed=(int(memory['metadata_bytes'])+int(memory['table_storage_bytes']))//16+65536
    need(all((item['load_pages']+19)*4096+per_role_fixed<=2*runtime.MIB for item in elf_images),'actual role overhead ceiling')
    header=f'P6\n{w} {h}\n255\n'.encode();ppm=(run/'display.ppm').read_bytes();reference=header+pixels(w,h,scenario)
    need(ppm==reference,'actual QEMU pixels differ from independent composition/exposure reference')
    need((run/'display-blank.ppm').read_bytes()==header+bytes(w*h*3),'initial scanout clear')
    (folder/'surface-reference.ppm').write_bytes(reference)
    replies=[json.loads(line) for line in (run/'qemu.stdout.log').read_text().splitlines() if line.strip()]
    need(not any('error' in r for r in replies) and {r.get('id') for r in replies if 'return' in r}==set(range(7)),'QMP acknowledgements')
    mutations=0
    for event in ['m1.surface_request','m1.surface_response','m1.surface_denied','m1.surface_transport_dead','m1.display_output','m1.surface_frame']:
        try:check([r for r in records if r['event']!=event],w,h,scenario)
        except (ValueError,KeyError,StopIteration):mutations+=1
        else:raise ValueError('missing criterion accepted '+event)
    result=dict(status='passed',label=label,scenario=scenario,build_id=manifest['build_id'],kernel_sha256=manifest['kernel_sha256'],iso_sha256=image['iso_sha256'],capture_sha256=base.digest(run/'display.ppm'),reference_sha256=base.digest(folder/'surface-reference.ppm'),width=w,height=h,criterion_mutations=mutations,observations=observations,scope='experimental copied leases / full redraw / synchronous scanout; no VSync or desktop efficiency result')
    base.save(folder/'surface-review.json',result);return result
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('folder');p.add_argument('--case',required=True,choices=SCENARIOS);a=p.parse_args();print(json.dumps(review(a.folder,a.case),indent=2))
