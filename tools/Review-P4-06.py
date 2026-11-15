"""Independent physical HID, receipt, pixel, authority and settlement oracle.
No guest input parser, wire codec, broker or renderer is imported.
"""
import argparse, collections, copy, importlib.util, json
from pathlib import Path
import p3_acceptance as base

def module(name,file):
    spec=importlib.util.spec_from_file_location(name,base.ROOT/file);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
r=module('p406_runtime','tools/Review-P4-02.py')
optical=module('p406_optical','tools/Review-P4-05.py')
need,rows=r.need,r.rows
NORMAL_KEYS=[(42,1),(54,1),(42,0),(30,1),(30,1),(30,0),(54,0)]
KEYS=NORMAL_KEYS+[(30,1)]*32+[(30,0),(42,1)]
TABLET=[(3,0,3075),(3,1,4558),(1,272,1),(0,0,0),
        (3,0,12810),(3,1,18230),(1,273,1),(1,272,0),(0,0,0),
        (3,0,3075),(3,1,4558),(1,272,1),(0,0,0),(1,272,0),(1,273,0),(0,0,0),
        (3,0,7174),(3,1,8660),(1,272,1),(0,0,0),(1,272,0),(0,0,0)]

def keyboard(raw):
    actual=[(int(x['kind']),int(x['code']),int(x['value'])) for x in raw if x['device']=='0']
    expected=[item for code,value in KEYS for item in [(1,code,value),(0,0,0)]]
    need(actual==expected,'physical normal/repeat/flood/missing-release keyboard reports')

def check(records):
    raw=rows(records,'m1.input_raw');published=rows(records,'m1.input_delivery');received=rows(records,'m1.input_received')
    tokens=[x['token'] for x in rows(records,'m1.user_log')]
    need(all('input-inject-'+str(n) in tokens for n in range(14)),'missing physical injection checkpoint')
    need([x for x in tokens if x.startswith('input-flood-')]==['input-flood-%02d'%n for n in range(34)],'missing bounded source flood checkpoints')
    required=['input-client-passed','input-client-pointer-passed','input-render-pressure-passed','input-overflow-passed','input-stale-release-passed','input-owner-exiting','input-owner-death-passed','input-device-reset-passed','input-reset-held-key-passed','input-inventory-zero','input-wrong-session-denied']
    need(all(tokens.count(t)==1 for t in required),'missing/duplicate client, failure or reset assertions')
    allowed=set(required)|{'desktop-heap-100-settled','surface-client-a','surface-client-b','surface-transport-denied','surface-initial-presented','surface-final-presented'}|{'input-inject-'+str(n) for n in range(14)}|{'input-flood-%02d'%n for n in range(34)}
    need(all(t in allowed for t in tokens),'unexpected assertion/diagnostic')
    keyboard(raw)
    need([(int(x['kind']),int(x['code']),int(x['value']))for x in raw if x['device']=='1']==TABLET,'physical tablet press/drag/stale/death releases')
    for device,count in [('0',82),('1',22)]:
        source=[x for x in raw if x['device']==device]
        need([int(x['sequence']) for x in source]==list(range(1,count+1)),'physical raw source sequence/count')
        need(all(0<=int(x['head'])<8 for x in source) and any(int(x['irq'])>0 for x in source),'checked heads and real device IRQ progress')
    need([int(x['ns']) for x in raw]==sorted(int(x['ns']) for x in raw),'raw monotonic acquisition clock')
    pub={(x['process'],x['incarnation'],x['sequence']):x for x in published}
    need(len(pub)==len(published),'duplicate publication')
    for x in received:
        need({k:v for k,v in pub.get((x['process'],x['incarnation'],x['sequence']),{}).items() if k!='seq'}=={k:v for k,v in x.items()if k!='seq'},'receipt without exact copied publication')
    for identity in {(x['process'],x['incarnation'])for x in received}:
        events=[x for x in received if (x['process'],x['incarnation'])==identity]
        need(all(int(a['sequence'])<int(b['sequence']) and int(a['ns'])<=int(b['ns']) for a,b in zip(events,events[1:])),'receipt sequence/clock')
    key=[x for x in received if x['kind']=='1']
    need([(int(x['code']),int(x['value']),int(x['modifiers']))for x in key]==[(42,1,1),(54,1,1),(42,0,1),(30,1,1),(30,2,1),(30,0,1),(54,0,0),(42,1,1)],'consumed focused key/modifier/repeat stream')
    text=[x for x in received if x['kind']=='2']
    need([(x['text'],x['value'],x['modifiers'])for x in text]==[('65','1','1'),('65','2','1')],'consumed independent uppercase/repeat text')
    need(all(x['process']==key[0]['process'] for x in text+key),'keyboard focus destination')
    buttons=[x for x in received if x['kind']=='4']
    need([(int(x['code']),int(x['value']),int(x['x']),int(x['y']),int(x['scene']))for x in buttons]==[(272,1,40,30,2),(273,1,420,330,2),(272,0,420,330,2),(272,1,40,30,2),(272,1,40,30,3)],'captured coordinates and suppression of stale/dead-owner releases')
    need(len({(x['process'],x['incarnation'],x['surface']) for x in buttons})==1 and buttons[0]['process']!=key[0]['process'],'pointer window authority')
    need(any(x['kind']=='3' and (x['x'],x['y'])==('420','330') for x in received),'actual captured motion')
    need(all(x['authority_session']=='1001' and x['output']=='1' and int(x['epoch'])>0 and int(x['focus'])>0 for x in published),'session/output/focus admission')
    cancels=[x for x in received if x['kind']=='5']
    need(any(x['code']=='1' and x['value']=='32' and x['modifiers']=='0' and x['process']==key[0]['process'] for x in cancels),'explicit model queue overflow/drop count')
    reset=[x for x in cancels if x['code']=='6'];need(len(reset)==1 and reset[0]['process']==key[-1]['process'] and reset[0]['modifiers']=='0' and int(reset[0]['sequence'])>int(key[-1]['sequence']),'actual reset cancels deliberately unreleased physical Shift')
    dead=next(x for x in rows(records,'m1.desktop_reap') if x['role']=='editor')
    need(dead['result']=='42' and int(dead['process'])+1==int(buttons[-1]['process']) and dead['generation']==buttons[-1]['incarnation'],'actual capture-owner exit')
    need(not any(x['process']==buttons[-1]['process'] and x['incarnation']==buttons[-1]['incarnation'] and int(x['seq'])>int(dead['seq']) for x in received),'delivery after owner death')
    need(len(rows(records,'m1.input_stopped'))==2 and len(rows(records,'m1.input_ready'))==4,'actual two-device reset and final release')
    need(all(x['reset_before_release']=='true' and x['dma_bytes']=='0' and x['grants']=='0' for x in rows(records,'m1.input_stopped')),'DMA release before reset or residual device grants')
    denied=rows(records,'m1.input_denied')
    need(any(x['authority_session']=='2002' for x in denied) and all(any(x['role']==str(role) and x['op']==str(op)for x in denied)for role in (3,4)for op in (0x4300,0x4301,0x4303)),'trusted injection/session denials')
    requests=rows(records,'m1.surface_request');responses={x['ticket']:x for x in rows(records,'m1.surface_response')}
    creates={responses[x['ticket']]['value']:x for x in requests if x['kind']=='1' and responses[x['ticket']]['status']=='0'}
    for event in published:
        owner=creates.get(event['surface']);need(owner is not None and all(event[k]==owner[k] for k in ['process','incarnation','authority_session']),'trusted created surface destination binding')
    captures=[x for x in requests if x['kind']=='11'];need(sum(responses[x['ticket']]['status']=='0' for x in captures)==3 and sum(responses[x['ticket']]['status']=='2' for x in captures)>=6,'capture lifetime, foreign owner and stale-scene denial')
    frames=rows(records,'m1.surface_frame');need(len(frames)==4 and [int(x['frame'])for x in frames]==[1,2,3,4] and all((x['rows'],x['width'],x['height'],x['source_released'])==('720','1280','720','true')for x in frames),'four complete scanout frames')
    # Prove keyboard input advanced during actual row copying, not merely beside an idle renderer.
    need(any(x['device']=='0' and int(frames[1]['started_ns'])<=int(x['ns'])<=int(frames[1]['finished_ns']) for x in raw),'actual keyboard acquisition during frame row copying')
    need(any(x['code']=='3' for x in cancels),'geometry/death explicit cancellation')
    return dict(raw_events=len(raw),published=len(published),received=len(received),key_events=len(key),text_events=len(text),button_events=len(buttons),frames=len(frames),model_overflow_dropped=32)

def clients(elves,test):
    if test:need(elves[0]['sha256']!=elves[1]['sha256'],'independent fixture client binaries')

def common(folder,prefix,features,test):
    r.profile()
    folder=Path(folder);run=folder/(prefix+'-run');pack=folder/(prefix+'-package')
    manifest=base.read(folder/'build-manifest.json');image=base.read(pack/'iso-manifest.json');case=base.read(run/'case-result.json');inputs=base.read(folder/'build-inputs.json')
    args=base.read(folder/'kernel-build.command.json')['arguments']
    need(args[args.index('--features')+1]==features and inputs['profile']=='kernel-stress','actual kernel features/profile')
    need(case['status']=='passed' and case['deadline_ms']==30000 and not case['timed_out'] and case['cleanup_reaped'] and case['emulator_test_exit']==(33 if test else None),'actual bounded guest completion')
    need(manifest['flavor']==('input-test' if test else 'normal') and base.digest(pack/'continuum.iso')==image['iso_sha256']==case['iso_sha256'] and base.digest(folder/'kernel.elf')==manifest['kernel_sha256']==image['kernel_sha256'],'exact ISO/kernel identity')
    need(manifest['build_id']==image['build_id']==case['build_id'] and case['iso_unchanged_after_run'],'build identity/immutable ISO')
    for name,expected in inputs['sources'].items():need(base.digest(base.ROOT/name)==expected,'stale input '+name)
    images=base.read(folder/'user-images.json');elves=[]
    for name in ['user_a','user_b','service']:
        entry=next(x for x in images['images'] if x['name']==name);actual=base.digest(folder/(name+'.elf'))
        need(actual==entry['sha256']==image['payload']['boot/'+name+'.elf']==base.digest(pack/'extracted payload/boot'/(name+'.elf')),'actual separately linked/extracted client bytes')
        command=base.read(folder/(name+'-build.command.json'))['arguments'];need('input_runtime' in command and 'surface_runtime' in command,'actual user input backend link')
        need(not (folder/(name+'-llvm-nm.stdout.log')).read_text().strip(),'unresolved client symbol');elves.append(r.elf(folder/(name+'.elf')))
    clients(elves,test)
    records=r.raw(run/'serial.log',manifest['build_id'],image['host_session_id'])
    inv=rows(records,'m1.desktop_inventory')[0]
    expected=sum(r.HEAPS)*r.MIB+sum((elves[i%3]['load_pages']+19)*4096 for i in range(7))
    need(int(inv['committed'])==expected and int(inv['allocations'])==sum(elves[i%3]['load_segments']+5 for i in range(7)),'ELF/physical/allocation reconciliation')
    need(int(inv['frames'])*4096==expected+65536,'independent 64 KiB DMA accounting')
    projection=copy.deepcopy([x for x in records if x['event']!='m1.user_log' or x['fields']['token']=='desktop-heap-100-settled'])
    for x in projection:
        f=x['fields']
        if x['event']=='m1.desktop_inventory':f['frames']=str(int(f['frames'])-16)
        if x['event']=='m1.desktop_launch' and f['role']=='editor' and f['mode']=='2':need(f['authority_session']=='2002','trusted synthetic second session');f['authority_session']='1001'
    r.check(projection,test)
    launch=base.read(run/'launch-manifest.json')['arguments'];need(all(x in launch for x in ['virtio-keyboard-pci,disable-legacy=on,addr=4','virtio-tablet-pci,disable-legacy=on,addr=5']) and not any('virtio-serial' in x for x in launch),'actual scoped input devices/no dev transport')
    return folder,run,manifest,image,records

def review(folder):
    folder,run,manifest,image,records=common(folder,'input','input-test',True);observations=check(records)
    reference=b'P6\n1280 720\n255\n'+optical.pixels(1280,720,3)
    need((run/'display.ppm').read_bytes()==reference,'actual final pixels after capture-owner death')
    need((run/'display-blank.ppm').read_bytes()==b'P6\n1280 720\n255\n'+bytes(1280*720*3),'actual initial clear')
    (folder/'input-reference.ppm').write_bytes(reference)
    requests=[json.loads(x)for x in (run/'qmp-requests.jsonl').read_text().splitlines()];replies=[json.loads(x)for x in (run/'qemu.stdout.log').read_text().splitlines()]
    ids=list(range(100,114))+list(range(200,233));actual=[x['id']for x in requests if x['execute']=='input-send-event']
    need(sorted(actual)==ids and len(actual)==len(set(actual)),'actual 47 QMP input injections')
    need(not any('error'in x for x in replies) and all(any(x.get('id')==i and 'return'in x for x in replies)for i in ids),'QMP injection acknowledgements')
    mutations=0
    for event in ['m1.input_raw','m1.input_received','m1.input_delivery','m1.input_denied','m1.input_ready','m1.input_stopped','m1.surface_frame','m1.desktop_reap']:
        try:check([x for x in records if x['event']!=event])
        except (ValueError,KeyError,IndexError,StopIteration):mutations+=1
        else:raise ValueError('missing criterion accepted '+event)
    for field in ['authority_session','sequence','ns','epoch','surface','scene','output','focus','kind','code','value','x','y','modifiers','text']:
        index=next(i for i,x in enumerate(records)if x['event']=='m1.input_received' and x['fields']['kind']=='4');candidate=copy.deepcopy(records);candidate[index]['fields'][field]='999999'
        try:check(candidate)
        except (ValueError,KeyError,IndexError,StopIteration):mutations+=1
        else:raise ValueError('tampered receipt accepted '+field)
    result=dict(status='passed',scope='physical input, private routing and exact teardown; no toolkit/reload/efficiency acceptance',build_id=manifest['build_id'],iso_sha256=image['iso_sha256'],capture_sha256=base.digest(run/'display.ppm'),observations=observations,criterion_mutations=mutations)
    base.save(folder/'input-review.json',result);return result

def ordinary(folder):
    folder,run,manifest,image,records=common(folder,'cold-boot-a','input-runtime',False)
    need(len(rows(records,'m1.input_ready'))==2 and not rows(records,'m1.input_raw') and not rows(records,'m1.input_delivery') and not rows(records,'m1.input_received'),'ordinary driver admission and diagnostic privacy')
    result=dict(status='passed',scope='ordinary input-runtime readiness; no synthetic test policy or key/text diagnostic stream',build_id=manifest['build_id'],iso_sha256=image['iso_sha256']);base.save(folder/'input-ordinary-review.json',result);return result
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('folder');p.add_argument('--ordinary',action='store_true');a=p.parse_args();print(json.dumps(ordinary(a.folder)if a.ordinary else review(a.folder),indent=2))
