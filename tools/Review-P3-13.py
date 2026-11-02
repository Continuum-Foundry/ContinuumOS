"""Independent counts, outcomes, inventory arithmetic and raw HPET sample review.

Consumes observed guest records, never imports the Rust fixture's expected matrix.
All full artifacts remain local. A failed assertion prevents evidence promotion.
"""
import argparse
import copy
import csv
import hashlib
import importlib.util
import json
import math
import shutil
import statistics
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BOUND = 320_000_000

def prior_reviewer():
    spec=importlib.util.spec_from_file_location('sdk_review',ROOT/'tools/Review-P3-12.py')
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module

def artifact_review(folder, flavor):
    previous=prior_reviewer()
    manifest=read(folder/'build-manifest.json')
    require(manifest['flavor']==flavor and read(folder/'build-inputs.json')['profile']=='kernel-stress' and manifest['kernel_sha256']==digest(folder/'kernel.elf'),'stress kernel identity/profile')
    previous.routes.memory.elf_audit(folder/'kernel.elf')
    codegen=previous.codegen(folder)
    run=next(p for p in folder.iterdir() if p.is_dir() and p.name.endswith('-run'))
    result,launch,case=previous.routes.memory.lifecycle(run)
    require(result['emulator_test_exit']==33 and launch['fixture']==('supervisor' if flavor=='stress-supervisor-test' else 'routes'),'stress ISO execution')
    builds=read(folder/'stress-builds.json')
    require(builds['status']=='passed' and builds['public_sdk'] and '--locked' in builds['arguments'] and '--offline' in builds['arguments'],'independent stress build')
    clean=folder/'public-stress'
    require((clean/'out').is_dir() and not (clean/'kernel').exists(),'private stress build input')
    for item in builds['inputs']:require(digest(clean/item['path'])==item['sha256'],'changed public build input')
    metadata=read(folder/'stress-public-metadata.json')
    canonical=Path(builds['canonical_root'])
    require(hashlib.sha256(json.dumps(builds['cache_inputs'],separators=(',',':'),ensure_ascii=False).encode()).hexdigest()==builds['cache_key'],'invalid public cache key')
    require(canonical==ROOT/'build/m3/P3-13/public-cache'/builds['cache_key']/'public-stress','unkeyed public build reuse')
    require(builds['cache_inputs']['compiler-backend']==read(folder/'build-inputs.json')['rustc_sha256'],'public/kernel compiler mismatch')
    for name,sha in builds['cache_inputs'].items():
        if not name.startswith('compiler-'):require(digest(ROOT/name)==sha,'stale public artifact cache input')
    require({p['name'] for p in metadata['packages']}=={'continuum-rt','sdk-stress'} and all(not p['source'] and Path(p['manifest_path']).is_relative_to(canonical) for p in metadata['packages']),'private stress dependency')
    package=folder/(run.name.removesuffix('-run')+'-package')/'extracted payload/boot'
    for name in ('user_a.elf','user_b.elf','service.elf'):
        require(digest(folder/name)==builds['sha256']==case['package']['payload']['boot/'+name]==digest(package/name)==digest(canonical.parent/name),'stress ISO payload substitution')
    return dict(build_id=manifest['build_id'],kernel_sha256=manifest['kernel_sha256'],user_sha256=builds['sha256'],codegen=codegen,launch=launch)

def require(condition, message):
    if not condition:
        raise ValueError(message)

def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))

def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def save(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')

def distribution(values):
    require(values and all(0 <= n <= BOUND for n in values), 'missing/out-of-bound HPET samples')
    ordered = sorted(values)
    return {'samples': len(values), **{f'p{p}_ns': ordered[math.ceil(len(values)*p/100)-1] for p in (50,95,99)}, 'max_ns': max(values)}

def rows_from_serial(path):
    result = []
    for line in Path(path).read_text().splitlines():
        if not line.startswith('m1.'):
            continue
        fields = line.split()
        require(len(line.encode())+1 <= 512, 'oversized record')
        row = dict(word.split('=',1) for word in fields[1:])
        require(len(row) == len(fields)-1, 'duplicate field')
        result.append((fields[0],row))
    require(result and [int(f['seq']) for _,f in result] == list(range(len(result))), 'record loss/reorder')
    require(len({f['build'] for _,f in result}) == len({f['session'] for _,f in result}) == 1, 'hidden guest identity change')
    require(sum(e=='m1.kernel_entry' for e,_ in result)==1 and result[-1][0]=='m1.ready', 'missing/reset guest')
    require(not any(e in ('m1.exception','m1.panic','m1.boot_rejected') for e,_ in result), 'guest fatal')
    return result

def decode_unsigned(raw):
    require(0<len(raw)<=512,'inventory byte bound')
    values=[];value=shift=width=0
    for byte in raw:
        require(shift<64 and (shift!=63 or byte<=1),'inventory u64 overflow')
        value|=(byte&127)<<shift;width+=1
        if byte&128:shift+=7;continue
        require(width==1 or byte!=0,'noncanonical inventory integer')
        values.append(value);value=shift=width=0
    require(width==0,'truncated inventory integer')
    return values

def expand_inventory(rows):
    schema=read(ROOT/'config/p3-13-inventory.json')
    require(schema['schema_version']==2 and [len(schema[k]) for k in ('memory','objects','backing','owner')]==[9,11,10,7],'inventory schema drift')
    result=[];pending=None;chunks=[]
    for event,f in rows:
        if event!='m1.stress_inventory':
            require(pending is None,'interleaved inventory packet');result.append((event,f));continue
        require(f['schema']=='2' and 1<=int(f['parts'])<=8 and 0<len(f['raw'])<=128 and len(f['raw'])%2==0 and f['raw']==f['raw'].lower(),'inventory chunk shape')
        if pending is None:
            require(f['batch']=='0','inventory starts mid-packet');pending=f;chunks=[]
        require(all(f[k]==pending[k] for k in ('wave','phase','parts','schema','build','session')) and int(f['batch'])==len(chunks),'inventory loss/reorder')
        chunks.append(bytes.fromhex(f['raw']))
        if len(chunks)<int(f['parts']):require(len(chunks[-1])==64,'short intermediate inventory chunk');continue
        values=decode_unsigned(b''.join(chunks));owners=values[0]
        require(owners<=3 and len(values)==31+owners*7,'inventory field/owner count')
        common={k:v for k,v in pending.items() if k not in ('batch','parts','schema','raw')};at=1
        for family in ('memory','objects','backing'):
            fields=schema[family];result.append(('m1.stress_'+family,{**common,**dict(zip(fields,map(str,values[at:at+len(fields)])))}));at+=len(fields)
        actors=[]
        for _ in range(owners):
            fields=schema['owner'];row={**common,**dict(zip(fields,map(str,values[at:at+7])))};at+=7
            actors.append(int(row['actor']));result.append(('m1.stress_owner',row))
        require(actors==sorted(set(actors)) and all(a in range(3) for a in actors),'inventory owner order')
        pending=None
    require(pending is None,'missing inventory final chunk')
    return result

def audit(rows):
    by = {}
    for event, fields in rows:
        by.setdefault(event, []).append(fields)
    complete = by.get('m1.route_ready', [])
    require(len(complete)==1 and all(complete[0].get(k)==v for k,v in {'waves':'100','tasks':'201','settled':'true','stress':'true','user_broker':'true'}.items()), 'stress completion')
    cycles=by.get('m1.stress_cycle',[])
    require([int(f['wave']) for f in cycles]==list(range(100)), 'missing/duplicate lifecycle wave')
    previous=0
    for wave,f in enumerate(cycles):
        require(int(f['scenario'])==wave%10 and f['seed']=='20261025', 'scenario/seed drift')
        require(all(f[k]==v for k,v in {'launched':'2','reaped':'2','supervision':'separate','launch_denials':'1'}.items()), 'lifecycle/admission profile settlement')
        before,after=int(f['control_before']),int(f['control_after'])
        require(after>before>=previous, 'starved/reset healthy control')
        previous=after
        require(all(int(f[k])>0 for k in ('dispatch_ns','reap_ns','inventory_ns')) and sum(int(f[k]) for k in ('dispatch_ns','reap_ns','inventory_ns'))<=int(f['work_ns']),'overlapping/missing HPET phase spans')
    ends=by.get('m1.stress_end',[])
    require(len(ends)==201, 'lost lifecycle')
    admits=by.get('m1.stress_admit',[]);require(len(admits)==201,'missing trusted startup')
    roster={(int(f['wave']),int(f['actor'])):f for f in admits};require(len(roster)==201,'duplicate startup binding')
    control_root=roster[0,0]['private_root']
    for wave in range(100):
        pair=[roster[wave,a] for a in (1,2)]
        require(len({control_root,*[f['private_root'] for f in pair]})==3,'shared private page tables')
        for actor,f in zip((1,2),pair):
            require(f['principal']=='303' and f['authority_session']==str((3003 if actor==1 else 5005)+wave) and f['service']==str(actor==2).lower() and int(f['generation'])==wave+1 and f['bootstrap']=='true','trusted binding/reuse')
    seen=set()
    for f in ends:
        wave,actor=int(f['wave']),int(f['actor'])
        admitted=roster[(0,0) if actor==0 else (wave,actor)]
        require((f['process'],f['generation'])==(admitted['process'],admitted['generation']),'retargeted worker lifetime')
        require((wave,actor) not in seen, 'duplicate reap');seen.add((wave,actor))
        if actor==0:
            require(wave==100 and int(f['progress'])>0, 'healthy control lifetime')
            code,vector=0,0
        else:
            require(0<=wave<100 and actor in (1,2), 'unknown worker')
            require(int(f['progress'])==(1100 if wave==0 else 100), 'warm-up/measured work count')
            if actor==1 and (wave%10 in (6,8,9) or wave%20==12):code,vector=768,0
            elif actor==1 and wave%10==3:code,vector=270,14
            elif actor==2 and wave%10==6:code,vector=262,6
            else:code,vector=0,0
            if actor==1 and wave%10==4:require(int(f['preemptions'])>0, 'cooperative CPU hog')
        require((int(f['code']),int(f['vector']))==(code,vector) and int(f['dispatches'])>0, 'unexpected process outcome')
    require(seen=={(w,a) for w in range(100) for a in (1,2)}|{(100,0)}, 'missing worker')
    kills=by.get('m1.stress_kill',[]);require(len(kills)==35,'missing combined termination boundary')
    kill_keys=set()
    for f in kills:
        wave=int(f['wave']);cause=f['cause'];require((wave,cause) not in kill_keys and f['code']=='768','duplicate kill boundary');kill_keys.add((wave,cause))
        if cause=='copy':
            require(wave%10 in (8,9) and f['operation']==str(10 if wave%10==8 else 14) and f['status']==f['pins']=='0','operation interrupted before quiescence/commit')
            require(f['mappings']=='2' if wave%10==8 else f['queued_only']=='4096','missing committed map/move before reap')
        elif cause=='wait':require(wave%20==12 and f['blocked']=='true','not actually parked')
        elif cause=='broker':require(wave%10==6,'wrong broker fault scope')
        else:require(False,'unknown termination boundary')
    memory=by.get('m1.stress_memory',[])
    objects=by.get('m1.stress_objects',[])
    backing=by.get('m1.stress_backing',[])
    keys={ (0,'empty'),(100,'final')}|{(w,p) for w in range(100) for p in ('staged','settled')}|{(w,p) for w in range(100) if w%10 in (3,6,8,9) for p in ('client-dead','server-dead')}
    require(len(memory)==len(objects)==len(backing)==len(keys), 'missing resource time series')
    indexed=[]
    for family in (memory,objects,backing):
        item={(int(f['wave']),f['phase']):f for f in family}
        require(len(item)==len(keys) and set(item)==keys, 'resource sample keys')
        indexed.append(item)
    mm,oo,bb=indexed
    owners={}
    for f in by.get('m1.stress_owner',[]):
        key=int(f['wave']),f['phase'];actor=int(f['actor'])
        require((key,actor) not in owners, 'duplicate owner charge')
        admitted=roster[(0,0) if actor==0 else (key[0],actor)]
        require((f['process'],f['generation'])==(admitted['process'],admitted['generation']),'retargeted physical owner attribution')
        owners[key,actor]=f
    steady=None
    for key,m in mm.items():
        o,b=oo[key],bb[key]
        require(int(m['frames'])*4096==int(m['resident']) and int(m['free'])+int(m['resident'])==int(m['pool']), 'physical pool arithmetic')
        require(int(b['private'])+int(b['shared'])+int(b['queued_only'])==int(o['object_backing']), 'shared backing double count')
        require(int(b['nonobject'])+int(o['object_backing'])==int(m['resident']), 'unaccounted resident bytes')
        require(sum(int(f['resident']) for (sample,_),f in owners.items() if sample==key)+int(b['orphan'])==int(m['resident']), 'unaccounted physical owner/orphan bytes')
        require(sum(int(f['charged']) for (sample,_),f in owners.items() if sample==key)+int(b['orphan'])==int(m['committed']),'unaccounted committed owner/orphan bytes')
        require(int(m['resident'])+int(m['foreign_commitment'])==int(m['committed']) and int(m['foreign_commitment'])<=int(o['mapped_attribution']),'physical backing/foreign quota charge mismatch')
        require(all(int(f['requested'])<=int(f['resident'])<=int(f['charged'])<=1048576 for (sample,_),f in owners.items() if sample==key),'owner requested/rounded/charged quota mismatch')
        require(sum(int(f['object_references']) for (sample,_),f in owners.items() if sample==key)==int(b['private'])+2*int(b['shared']),'object reference attribution disagrees with private/two-owner shared backing')
        require(o['pins']=='0', 'surviving pin')
        if key[1]=='settled':
            sample=tuple((name,f[name]) for f in (m,o,b) for name in sorted(f) if name not in ('wave','phase','seq'))
            if steady is None:steady=sample
            require(sample==steady, 'settled monotonic growth/step')
            require(m['spaces']=='1' and b['wait_records']==b['wait_watches']==b['ready_records']=='0', 'surviving worker/wait')
    for key in ((0,'empty'),(100,'final')):
        require(all(mm[key][k]=='0' for k in ('spaces','allocations','committed','resident','frames')), 'final backing leak')
        require(all(oo[key][k]=='0' for k in ('objects','handles','mappings','queued_messages','queued_bytes','queued_handles','active_tables','stacks','pins')), 'final object leak')
        require(bb[key]['wait_records']==bb[key]['wait_watches']==bb[key]['ready_records']==bb[key]['timers']==bb[key]['events']=='0', 'final pending work')
    batches={};decisions={};expiries={};oom={};control=[];costs={};diagnostics=[];timer_lateness={};cancel_races={};work_phases={};faults={}
    for f in by.get('m1.user_log',[]):
        token=f['token'];require(token not in ('stress-failed','stress-panic'), 'user assertion/panic')
        require(f['offset']=='0' and int(f['total'])==len(token), 'fragmented fixture trace')
        parts=token.split('-')
        if parts[0] in ('ipc62','oom','timerlat','cancelrace','workphase','fault'):
            wave=int(parts[1]);require(0<=wave<100 and f['case']==str(900+wave%10) and f['principal']=='303','client observation retargeted')
        elif parts[0] in ('routepack','routecycle','expiry'):
            wave=int(parts[1]);require(0<=wave<100 and f['case']=='930' and f['principal']=='303','provider observation retargeted')
        elif parts[0]=='control':require(f['case']=='920' and f['principal']=='404','healthy control observation retargeted')
        elif parts[0] in ('clockcost','diagcost','diagbench'):require(f['case']=='900' and f['principal']=='303','cost observation retargeted')
        if parts[0]=='workphase':
            wave=int(parts[1]);require(len(parts)==5 and wave not in work_phases and all(0<int(v)<30000000000 for v in parts[2:]),'missing/invalid client phase spans');work_phases[wave]=tuple(map(int,parts[2:]))
        if parts[0]=='fault':
            wave,scenario=int(parts[1]),int(parts[2]);require(len(parts)==4 and parts[3]=='ok' and wave not in faults and scenario==wave%10,'fault completion drift');faults[wave]=scenario
        if parts[0]=='cancelrace':
            wave,status=map(int,parts[1:]);require(wave not in cancel_races and wave%20==2 and status==8,'cancellation registration race');cancel_races[wave]=status
        if parts[0]=='timerlat':
            wave,deadline,resumed=map(int,parts[1:]);require(wave not in timer_lateness and resumed>=deadline,'timer fired early/duplicate');timer_lateness[wave]=resumed-deadline
        if parts[0] in ('clockcost','diagcost'):
            require(parts[0] not in costs and int(parts[2])>0,'missing/duplicate observer cost');costs[parts[0]]={'calls':int(parts[1]),'guest_elapsed_ns':int(parts[2])}
        if parts[0]=='diagbench':diagnostics.append(int(parts[1]))
        if parts[0]=='control':control.append(tuple(map(int,parts[1:])))
        if parts[0]=='expiry':
            wave,deadline,observed=map(int,parts[1:]);require(wave not in expiries and observed>=deadline,'queued grant did not cross actual HPET expiry');expiries[wave]=observed-deadline
        if parts[0]=='oom':
            _,wave,error,capacity,reused=parts;wave=int(wave);require(wave not in oom and wave%10 in (0,7) and (error,capacity,reused)==('11','65536','reused'),'heap pressure/recovery');oom[wave]=int(capacity)
        if parts[0]=='ipc62':
            _,wave,batch,raw=parts;key=(int(wave),int(batch))
            alphabet='0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz'
            require(key not in batches and len(raw)%5==0 and all(c in alphabet for c in raw),'duplicate/malformed latency batch')
            decoded=[]
            for i in range(0,len(raw),5):
                value=0
                for digit in raw[i:i+5]:value=value*62+alphabet.index(digit)
                decoded.append(value)
            batches[key]=decoded
        if parts[0]=='routepack':
            require(len(parts)==3 and len(parts[2])==56,'malformed route outcomes')
            wave=int(parts[1]);values=bytes.fromhex(parts[2])
            for step in range(7):
                status,revision,value,queued=values[step*4:step*4+4];key=(wave,step)
                require(key not in decisions,'duplicate route decision')
                require((status,revision,value,queued)==([0,0,13,13,7,3,7][step],1,7,0),'stale writer/authority/commit outcome')
                decisions[key]=status
    require(set(decisions)=={(w,s) for w in range(100) for s in range(7)}, 'missing route/fence cycle')
    require(set(expiries)==set(range(100)) and set(oom)=={w for w in range(100) if w%10 in (0,7)},'missing expiry/heap fault class')
    require(set(timer_lateness)=={w for w in range(100) if w%20==2 or w%10==6},'missing timer deadline observations')
    require(set(cancel_races)==set(range(2,100,20)),'missing registered cancellation interleaving')
    require(len(control)==1 and control[0][0]==int(next(f for f in ends if f['actor']=='0')['progress']),'missing healthy work checksum')
    count,expected=control[0];a,c,value=1664525,1013904223,20261025;mask=(1<<64)-1
    while count:
        if count&1:value=(a*value+c)&mask
        c=(a*c+c)&mask;a=(a*a)&mask;count>>=1
    require(value==expected,'healthy control work corrupted')
    require(set(costs)=={'clockcost','diagcost'} and costs['clockcost']['calls']==1024 and costs['diagcost']['calls']==32,'observer cost probes')
    shares=by.get('m1.stress_share',[]);require(len(shares)==100 and {int(f['wave']) for f in shares}==set(range(100)) and all(f['shared']=='4096' and f['object_backing']=='4096' and f['counted_once']=='true' for f in shares),'missing real cross-owner shared backing')
    require(set(batches)=={(w,b) for w in range(100) for b in range(7)}, 'missing IPC round trips')
    ipc=[]
    for wave in range(100):
        for batch in range(7):
            values=batches[wave,batch];require(len(values)==(4 if batch==6 else 16), 'wrong latency batch size');ipc.extend(values)
    require(len(ipc)==10000, 'wrong measured IPC count')
    require(set(work_phases)==set(range(100)) and set(faults)=={w for w in range(100) if w%10 not in (8,9) and w%20!=12},'missing client phase/fault completions')
    require(all(work_phases[w][0]>=sum(ipc[w*100:(w+1)*100]) for w in range(100)),'IPC intervals exceed client echo span')
    metrics={};raw_metrics={}
    for f in by.get('m1.stress_samples',[]):
        kind=f['kind'];batch=int(f['batch']);raw=f['raw'];values=[int(raw[i:i+8],16) for i in range(0,len(raw),8)]
        raw_metrics.setdefault(kind,{})[batch]=values
    for f in by.get('m1.stress_metric',[]):
        kind=f['kind'];require(kind not in metrics and kind in ('dispatch','wake'), 'duplicate/unknown metric')
        batches=raw_metrics[kind];require(sorted(batches)==list(range(len(batches))), 'metric loss')
        values=[v for batch in sorted(batches) for v in batches[batch]]
        require(f['stride']=='32' and f['limit']=='4096' and len(values)==int(f['samples'])==int(f['observed'])//32<=4096, 'metric sample counts/overflow')
        require(max(values)<=int(f['maximum_ns'])<=int(f['bound_ns'])==BOUND, 'fairness/wake bound')
        metrics[kind]={**distribution(values),'observed':int(f['observed']),'maximum_all_ns':int(f['maximum_ns'])}
    require(set(metrics)=={'dispatch','wake'}, 'missing scheduling measurements')
    peaks=by.get('m1.stress_peaks',[]);bounds=by.get('m1.stress_bounds',[]);idle=by.get('m1.stress_idle',[])
    capacity=by.get('m1.stress_capacity',[]);storage=by.get('m1.stress_storage',[])
    cache=by.get('m1.stress_cache',[])
    cpu=by.get('m1.stress_cpu',[])
    require(len(cpu)==1 and all(cpu[0][k]=='0' for k in ('fs_base','gs_base','kernel_gs_base')) and int(cpu[0]['cr4'],0)&(1<<16)==0 and cpu[0]['tls']=='false' and cpu[0]['segment_policy']=='constant_zero','unsupported mutable segment-base profile')
    require(len(cache)==1 and cache[0]['entries']=='3' and 0<int(cache[0]['bytes'])<=4096 and int(cache[0]['verification_ns'])>0 and all(cache[0][k]==v for k,v in dict(immutable='true',writable_aliases='0',retained='true',charged='kernel_image').items()),'unexplained/unverified retained image cache')
    require(len(capacity)==len(storage)==1 and int(capacity[0]['ram_capacity'])==2147483648 and int(capacity[0]['eligible'])+int(capacity[0]['reserved_capacity'])==2147483648 and 0<int(capacity[0]['kernel_image'])<=int(capacity[0]['reserved_capacity']),'RAM/reservation partition')
    require(capacity[0]['clock']=='hpet' and 0<int(capacity[0]['resolution_ns'])<=100 and capacity[0]['quantum_ns']=='5000000','guest clock/profile correlation')
    require(0<int(storage[0]['fixed_budget'])<=int(storage[0]['fixed_ceiling'])==16777216 and int(storage[0]['samples'])>=32768,'unaccounted precharged capacity')
    require(len(peaks)==len(bounds)==len(idle)==1, 'missing peak/idle records')
    require(diagnostics==(list(range(32)) if peaks[0]['traced']=='true' else []),'paired diagnostic output profile')
    interval=by.get('m1.stress_interval',[]);require(len(interval)==1 and int(interval[0]['end_ns'])>int(interval[0]['start_ns']) and interval[0]['warmup']=='1000' and interval[0]['measured']=='10000','missing active interval')
    active_ns=int(interval[0]['end_ns'])-int(interval[0]['start_ns'])
    require(0<int(peaks[0]['masked_ns'])<=BOUND and 0<int(peaks[0]['messages'])<=128, 'IRQ/queue bound')
    require(peaks[0]['method']=='adjacent_resume_sti' and int(peaks[0]['segments'])>100 and 0<int(peaks[0]['masked_body_ns'])<=int(peaks[0]['masked_ns']) and max(int(f['committed']) for f in mm.values())<=int(peaks[0]['committed'])<=67108864,'masked boundary/peak commitment accounting')
    require(int(bounds[0]['peak_shared'])>=4096 and int(bounds[0]['pins_deferred'])==20 and int(bounds[0]['peak_waits'])>0 and int(bounds[0]['peak_timers'])>0, 'unexercised transient class')
    require(idle[0]['requested_halts']=='64' and int(idle[0]['halts'])>=64 and int(idle[0]['ticks'])>=64 and int(idle[0]['elapsed_ns'])>=320000000, 'idle/settle evidence')
    exported=by.get('m1.stress_export',[]);require(len(exported)==1 and exported[0]['interruptible']=='true' and int(exported[0]['elapsed_ns'])>0,'unaccounted sample export')
    return {'worker_lifecycles':200,'route_cycles':100,'route_decisions':700,'ipc':distribution(ipc),'ipc_serial_rate_per_second':10000*1e9/sum(ipc),'active_guest_elapsed_ns':active_ns,'active_round_trip_rate_including_warmup_per_second':11000*1e9/active_ns,'observer_cost_probes':costs,'timer_lateness':distribution(list(timer_lateness.values())),'queued_session_expiry_observation_lateness':distribution(list(expiries.values())),'scheduling':metrics,'peaks':peaks[0],'bounds':bounds[0],'idle':idle[0],'retained_cache':cache[0],'client_phase_spans_ns':work_phases},ipc,indexed,raw_metrics

def review_run(folder, output):
    folder=Path(folder).resolve();output=Path(output);output.mkdir(parents=True,exist_ok=True)
    result=read(folder/'result.json');require(result['status']=='passed', 'failed host/guest run')
    run=next(p for p in folder.iterdir() if p.is_dir() and p.name.endswith('-run'))
    launch=read(run/'run-result.json');require(launch['status']=='passed' and launch['cleanup_reaped'] and not launch['timed_out'] and launch['deadline_ms']==30000 and launch['iso_unchanged_after_run'], 'guest lifecycle/timeout')
    raw=rows_from_serial(run/'serial.log')
    require(any(e=='m1.stress_inventory' for e,_ in raw) and not any(e in ('m1.stress_memory','m1.stress_objects','m1.stress_backing','m1.stress_owner') for e,_ in raw),'missing encoded inventory source')
    rows=expand_inventory(raw);report,ipc,indexed,raw_metrics=audit(rows)
    report['artifacts']=artifact_review(folder,'stress-trace' if report['peaks']['traced']=='true' else 'stress-test')
    require(rows[0][1]['build']==report['artifacts']['build_id'] and rows[0][1]['session']==launch['host_session_id'],'serial artifact/session identity')
    report['build']=rows[0][1]['build'];report['session']=rows[0][1]['session'];report['iso_sha256']=launch['iso_sha256'];report['serial_sha256']=digest(run/'serial.log');report['host_elapsed_ms']=launch['elapsed_host_ms']
    with (output/'ipc.csv').open('w',newline='') as stream:
        writer=csv.writer(stream);writer.writerow(['sample','guest_elapsed_ns']);writer.writerows(enumerate(ipc))
    for name,family in zip(('memory','objects','backing'),indexed):
        fields=[k for k in next(iter(family.values())) if k not in ('build','session','session_origin','version','stage','outcome','emergency')]
        with (output/(name+'.csv')).open('w',newline='') as stream:
            writer=csv.DictWriter(stream,fields,extrasaction='ignore');writer.writeheader();writer.writerows(family.values())
    for event in ('m1.stress_owner','m1.stress_admit','m1.stress_end','m1.stress_cycle','m1.stress_share'):
        family=[f for e,f in rows if e==event];require(family,'missing retained raw family')
        fields=[k for k in family[0] if k not in ('build','session','session_origin','version','stage','outcome','emergency')]
        with (output/(event.removeprefix('m1.stress_')+'.csv')).open('w',newline='') as stream:
            writer=csv.DictWriter(stream,fields,extrasaction='ignore');writer.writeheader();writer.writerows(family)
    with (output/'scheduling.csv').open('w',newline='') as stream:
        writer=csv.writer(stream);writer.writerow(['kind','sample','guest_elapsed_ns'])
        for kind,batches in raw_metrics.items():
            writer.writerows((kind,i,v) for i,v in enumerate(v for batch in sorted(batches) for v in batches[batch]))
    save(output/'review.json',report)
    return report

def review_supervisor(folder,output):
    folder=Path(folder).resolve();output=Path(output);output.mkdir(parents=True,exist_ok=True)
    require(read(folder/'result.json')['status']=='passed','public supervision run failed')
    manifest=read(folder/'build-manifest.json')
    require(manifest['flavor']=='stress-supervisor-test' and digest(folder/'kernel.elf')==manifest['kernel_sha256'],'supervision artifact identity')
    result=read(folder/'sdk-stress-supervisor-run/run-result.json')
    require(result['status']=='passed' and result['cleanup_reaped'] and not result['timed_out'] and result['deadline_ms']==30000,'supervision guest lifecycle')
    spec=importlib.util.spec_from_file_location('supervision_review',ROOT/'tools/Review-P3-09.py')
    previous=importlib.util.module_from_spec(spec);spec.loader.exec_module(previous)
    rows=rows_from_serial(folder/'sdk-stress-supervisor-run/serial.log')
    report={'status':'passed','audit':previous.audit(rows),'mutations_rejected':len(previous.mutations(rows)),'build_id':manifest['build_id'],'kernel_sha256':manifest['kernel_sha256'],'iso_sha256':result['iso_sha256'],'serial_sha256':digest(folder/'sdk-stress-supervisor-run/serial.log'),'artifacts':artifact_review(folder,'stress-supervisor-test')}
    require(report['audit']['admissions']==29,'public supervision admission count changed')
    save(output/'review.json',report)
    return report

def mutation_check(rows):
    encoded=rows
    rows=expand_inventory(rows)
    cases=[]
    def field(event,key,value):
        changed=copy.deepcopy(rows)
        next(f for e,f in changed if e==event)[key]=value
        return changed
    for event,key,value in (
        ('m1.route_ready','waves','99'),('m1.route_ready','settled','false'),
        ('m1.stress_cycle','control_after','0'),('m1.stress_cycle','seed','0'),
        ('m1.stress_cycle','reaped','1'),('m1.stress_cycle','supervision','unchecked'),
        ('m1.stress_cycle','launch_denials','0'),('m1.stress_end','code','99'),
        ('m1.stress_end','vector','14'),('m1.stress_memory','resident','4096'),
        ('m1.stress_memory','free','0'),('m1.stress_objects','pins','1'),
        ('m1.stress_backing','shared','4096'),('m1.stress_backing','wait_records','1'),
        ('m1.stress_owner','resident','4096'),('m1.stress_owner','process','999'),
        ('m1.stress_owner','charged','0'),('m1.stress_memory','foreign_commitment','4096'),
        ('m1.stress_metric','maximum_ns','320000001'),('m1.stress_metric','samples','1'),
        ('m1.stress_peaks','masked_ns','320000001'),('m1.stress_bounds','peak_shared','0'),
        ('m1.stress_bounds','pins_deferred','0'),('m1.stress_idle','ticks','0')):
        cases.append((f'{event}/{key}',field(event,key,value)))
    for event in ('m1.stress_cycle','m1.stress_end','m1.stress_memory','m1.stress_metric'):
        changed=copy.deepcopy(rows);changed.pop(next(i for i,(e,_) in enumerate(changed) if e==event));cases.append((f'missing/{event}',changed))
    for key,value in (('entries','0'),('bytes','0'),('immutable','false'),('writable_aliases','1')):
        cases.append((f'cache/{key}',field('m1.stress_cache',key,value)))
    for prefix in ('ipc62-','routepack-'):
        changed=copy.deepcopy(rows);i=next(i for i,(e,f) in enumerate(changed) if e=='m1.user_log' and f['token'].startswith(prefix));changed.pop(i);cases.append((f'missing/{prefix}',changed))
        changed=copy.deepcopy(rows);f=next(f for e,f in changed if e=='m1.user_log' and f['token'].startswith(prefix));f['token']=prefix+'0-0-ffffffff';f['total']=str(len(f['token']));cases.append((f'corrupt/{prefix}',changed))
    accepted=[]
    for label,changed in cases:
        try:audit(changed)
        except (ValueError,KeyError,IndexError):continue
        accepted.append(label)
    require(not accepted,'accepted evidence mutations: '+', '.join(accepted))
    codec_mutations=0
    for bad in (b'',b'\x80',b'\x80\x00',b'\x81\x00',b'\xff'*9+b'\x02',b'\x80'*10+b'\x00'):
        try:decode_unsigned(bad)
        except ValueError:codec_mutations+=1;continue
        raise ValueError('accepted invalid inventory integer')
    if any(e=='m1.stress_inventory' for e,_ in encoded):
        for key,value in (('batch','1'),('parts','9'),('schema','3'),('raw','80')):
            changed=copy.deepcopy(encoded);next(f for e,f in changed if e=='m1.stress_inventory')[key]=value
            try:expand_inventory(changed)
            except (ValueError,KeyError):codec_mutations+=1;continue
            raise ValueError('accepted corrupted inventory packet '+key)
    require(decode_unsigned(bytes.fromhex('007f8001ffffffffffffffffff01'))==[0,127,128,(1<<64)-1],'inventory decoder golden vector')
    return {'mutations':len(cases)+codec_mutations,'rejected':len(cases)+codec_mutations,'accepted':accepted}

def aggregate(path, destination=None):
    path=Path(path).resolve();acceptance=read(path);out=path.parent
    require(acceptance['story']=='P3-13' and acceptance['status']=='passed' and len(acceptance['commands'])==25 and all(c['exit_code']==0 for c in acceptance['commands']),'incomplete final stress gate')
    require(acceptance['native_observer_profile']=='release' and digest(acceptance['native_observer']['path'])==acceptance['native_observer']['sha256'],'native observer identity/profile')
    require(all('--release' in c['arguments'] and '--locked' in c['arguments'] and '--offline' in c['arguments'] for c in acceptance['commands'] if c['program']=='cargo'),'unoptimized/unlocked stress observer')
    require(digest(ROOT/'contracts/sdk-v1.json')=='debe57866ed308f46ff2f48f5065e739c7457d0f28d0e8dccc93d42abd4c891b','changed frozen SDK contract')
    prior=Path(acceptance['runs']['prior_acceptance']);previous=read(prior.parent/'review.json')
    require(previous['story']=='P3-12' and previous['scope']=='final-source local acceptance' and previous['acceptance_sha256']==digest(prior) and previous['regression']['kernel_host_tests']==95,'missing final runtime regression review')
    prior_runs=read(prior)['runs'];sources=read(Path(prior_runs['sdk'])/'build-inputs.json')['sources']
    if 'regression_reuse' in acceptance:
        reuse=acceptance['regression_reuse'];manifest=Path(reuse['source_manifest'])
        require(reuse['acceptance_sha256']==digest(prior) and reuse['source_manifest_sha256']==digest(manifest) and read(manifest)==sources and reuse['validated_files']==len(sources),'invalid regression source reuse proof')
        require(all(digest(ROOT/name)==sha for name,sha in sources.items()),'changed reused regression input')
    entries=acceptance['stress'];require(len(entries)==10 and {(e['trace'],e['repetition']) for e in entries}=={(t,i) for t in ('off','on') for i in range(5)},'stress repetition/profile loss')
    reports=[];samples={'off':[],'on':[]};sessions=set();builds={'off':set(),'on':set()};identities=set();mutations=[];artifact_audits={}
    for entry in entries:
        folder=Path(entry['artifacts']);output=Path(entry['review'])
        require(read(folder/'build-inputs.json')['sources']==sources,'stress/regression source mismatch')
        report=review_run(folder,output);run=next(p for p in folder.iterdir() if p.is_dir() and p.name.endswith('-run'))
        mutation=mutation_check(rows_from_serial(run/'serial.log'));save(output/'mutation-check.json',mutation);mutations.append(mutation)
        require(report['session'] not in sessions and report['peaks']['traced']==str(entry['trace']=='on').lower(),'duplicate guest/profile mismatch');sessions.add(report['session'])
        builds[entry['trace']].add(report['build']);identities.add(report['artifacts']['user_sha256'])
        artifact_audits.setdefault(report['artifacts']['kernel_sha256'],report['artifacts'].pop('codegen'))
        with (output/'ipc.csv').open() as stream:samples[entry['trace']].extend(int(r['guest_elapsed_ns']) for r in csv.DictReader(stream))
        reports.append(dict(trace=entry['trace'],repetition=entry['repetition'],**report))
    require(all(len(v)==1 for v in builds.values()) and builds['off']!=builds['on'] and len(identities)==1,'unpaired source/artifact profiles')
    supervisor_folder=Path(acceptance['runs']['public_supervision'])
    require(read(supervisor_folder/'build-inputs.json')['sources']==sources,'supervision/regression source mismatch')
    supervisor=review_supervisor(supervisor_folder,out/'public-supervision-review')
    require(supervisor['artifacts']['user_sha256'] in identities,'different public supervision fixture')
    flavors=read(Path(acceptance['runs']['flavors'])/'flavors.json')
    require(len(flavors)==14 and all(f['exit_code']==0 for f in flavors),'separate flavor matrix incomplete')
    profile=read(ROOT/'config/p3-13-stress.json')
    require(profile['guest_deadline_ms']==30000 and profile['kernel_opt_level']==profile['public_fixture_opt_level']==3 and profile['repetitions']==5 and profile['retained_object_tolerance']==profile['retained_frame_tolerance']==0 and profile['verified_image_cache_entries']==3 and profile['verified_image_cache_budget_bytes']==4096,'changed experiment bounds')
    groups={t:[r for r in reports if r['trace']==t] for t in ('off','on')}
    timing={t:dict(ipc=distribution(samples[t]),worker_lifecycles=sum(r['worker_lifecycles'] for r in groups[t]),route_cycles=sum(r['route_cycles'] for r in groups[t]),active_guest_elapsed_ns=[r['active_guest_elapsed_ns'] for r in groups[t]],host_elapsed_ms=[r['host_elapsed_ms'] for r in groups[t]]) for t in groups}
    require(all(g['worker_lifecycles']==1000 and g['route_cycles']==500 and g['ipc']['samples']==50000 for g in timing.values()),'fixed P3-01 experiment counts')
    paired=[]
    for i in range(5):
        off=next(r for r in groups['off'] if r['repetition']==i);on=next(r for r in groups['on'] if r['repetition']==i)
        paired.append(dict(repetition=i,active_guest_elapsed_ratio=on['active_guest_elapsed_ns']/off['active_guest_elapsed_ns'],ipc_p50_ratio=on['ipc']['p50_ns']/off['ipc']['p50_ns'],ipc_p99_ratio=on['ipc']['p99_ns']/off['ipc']['p99_ns'],diagnostic_cost_off_ns=off['observer_cost_probes']['diagcost']['guest_elapsed_ns'],diagnostic_cost_on_ns=on['observer_cost_probes']['diagcost']['guest_elapsed_ns']))
    report=dict(story='P3-13',status='passed',scope='final-source local acceptance',review='local Codex self-review; not external approval',source_commit=acceptance['source_commit'],acceptance_sha256=digest(path),reviewer_sha256=digest(__file__),runtime_regression_review_sha256=digest(prior.parent/'review.json'),runtime_regression=previous['regression'],contract_sha256=digest(ROOT/'contracts/sdk-v1.json'),profile=profile,profiles=timing,paired_observer_overhead=paired,median_active_guest_elapsed_ratio=statistics.median(p['active_guest_elapsed_ratio'] for p in paired),public_supervision=supervisor,flavors=flavors,guests=reports,mutation_checks=mutations,limitations=['TCG one-vCPU 2GiB capacity profile; no desktop memory/CPU/input acceptance','mandatory timing/outcome instrumentation remains in both profiles; sampled dispatch/wake quantiles have complete maxima but not every tail sample','HPET elapsed intervals and host process elapsed are separate; no CPU/RSS claim','synthetic ephemeral routes, manual trusted bootstrap, experimental supervision IDs; no reload/durability/security certification; M3 exit remains open'])
    report['criterion_review']={
        'continuous_faults':'pass: ten distinct continuous guests, 100 waves each, 200 worker lifecycles and 100 route cycles per guest; expected process-local faults and healthy control checksums',
        'reclamation':'pass: every staged/death/settled resource row independently reconciled; all 100 settled baselines identical within each guest and final frame/object/wait inventories zero',
        'physical_accounting':'pass: eligible/free/resident partition, physical backing counted once, private/shared/queued holds, owner/orphan attribution, fixed image/table/sample capacity and launch refusal',
        'timing':'pass: 100000 measured IPC samples plus 10000 warmups, fixed-stride dispatch/wake samples with complete maxima, HPET expiry/timer lateness, 64 idle halts per guest and paired observer cost',
        'regressions':'pass: fresh P3-12 complete runtime gate, 95 host tests, separate public supervision and 14 separate metadata flavors; fixed deadlines/quotas and failed development provenance retained'
    }
    report['unsafe_review']={
        'boundary':'registered user_fixture hooks; checked dispatcher and common return validation retained; public operations cannot select return IP/selectors/privileged flags',
        'copy_and_reap':'real nested timer interrupt while map/transfer copy pin is live requests deferred death; return observer asserts successful commit and zero pins before trusted reap; no registry borrow or kernel continuation crosses task switch',
        'measurement':'single CPU, IF-clear bounded static observer state; fixed-capacity buffers reject overflow; mandatory queue peaks remain observed on inline traffic and ownership scans occur on transfer/ownership changes; snapshots and trace samples are read-only',
        'public_fixture':'only frozen SDK and packaged pure policy/routes code in isolated locked/offline build; little-endian test records and kernel-authenticated receive identities; explicit handles closed or reaped; all actual user ELFs reviewed for admitted x87/SSE2 baseline',
        'scope':'trusted synthetic bootstrap and experimental supervision IDs are explicit test facilities; not a general SDK extension, thread model, hostile-code certification, guest UI or reload result'
    }
    report['artifact_audits']=artifact_audits
    report['runtime_regression']={k:previous['regression'][k] for k in ('story','status','scope','kernel_host_tests','prerequisite_checks','m1_rows','m1_launches','acceptance_sha256','reviewer_sha256')}
    save(out/'review.json',report)
    if destination:
        dest=Path(destination).resolve();require(dest.is_relative_to(ROOT/'evidence/phase-3/P3-13') and not dest.exists(),'invalid/existing evidence destination');dest.mkdir(parents=True)
        for name,value in [('review.json',report),('acceptance.json',acceptance),('profile.json',profile),('inventory-schema.json',read(ROOT/'config/p3-13-inventory.json')),('source-provenance.json',dict(source_commit=acceptance['source_commit'],build_inputs=sources,review_inputs={str(p.relative_to(ROOT)).replace(chr(92),'/'):digest(p) for p in [Path(__file__),ROOT/'tools/Test-P3-13.ps1',ROOT/'tools/Check-P3-13Flavors.ps1']})),('development-findings.json',read(ROOT/'work/p3-13-draft/development-findings.json'))]:save(dest/name,value)
        save(dest/'runtime-regression-review.json',previous)
        save(dest/'m1-review.json',read(Path(prior_runs['m1'])/'review.json'))
        save(dest/'prerequisites.json',read(Path(prior_runs['prerequisites'])/'prerequisites.json'))
        for entry in entries:
            label=f"{entry['trace']}-{entry['repetition']}";review=Path(entry['review']);target=dest/label;target.mkdir()
            names=['ipc.csv','cycle.csv','mutation-check.json']
            if entry['repetition']==0:names+=['memory.csv','objects.csv','backing.csv','owner.csv','admit.csv','end.csv','share.csv','scheduling.csv']
            for name in names:shutil.copyfile(review/name,target/name)
        selected=[]
        for entry in (next(e for e in entries if e['trace']==t and e['repetition']==0) for t in ('off','on')):
            folder=Path(entry['artifacts']);run=next(p for p in folder.iterdir() if p.is_dir() and p.name.endswith('-run'))
            lines=(run/'serial.log').read_text().splitlines()
            selection=[line for line in lines if line.startswith(('m1.stress_capacity ','m1.stress_storage ','m1.stress_cache ','m1.stress_peaks ','m1.stress_bounds ','m1.stress_idle ','m1.stress_interval ','m1.route_ready ')) or line.startswith('m1.stress_inventory ') and (' wave=0 ' in line or ' wave=100 ' in line)]
            (dest/f"selected-{entry['trace']}.txt").write_text('\n'.join(selection)+'\n')
            selected.append(dict(trace=entry['trace'],selection='capacity/storage/summary plus wave0 and final resource records; other records remain local',full_serial_sha256=digest(run/'serial.log'),selected_records=len(selection),full_records=len(rows_from_serial(run/'serial.log'))))
        save(dest/'trace-selection.json',selected)
        failure_selection=[]
        for index,finding in enumerate(read(ROOT/'work/p3-13-draft/development-findings.json')['failed_attempts']):
            directory=Path(finding.get('directory',''))
            traces=list(directory.glob('*-run/serial.log')) if directory.is_dir() else []
            if not traces and directory.is_dir():traces=list(directory.glob('kernel-host/check.stderr.log'))
            for part,trace in enumerate(traces):
                expected=finding.get('serial_sha256' if trace.name=='serial.log' else 'stderr_sha256')
                require(expected is None or digest(trace)==expected,'changed retained failed trace')
                lines=trace.read_text(errors='replace').splitlines()
                # Keep raw final records and any explicit rejection/panic location.
                indices=sorted(set(range(max(0,len(lines)-6),len(lines)))|{i for i,line in enumerate(lines) if any(token in line for token in ('panic-line-','stress-panic','m1.boot_rejected ','STATUS_STACK_OVERFLOW','0xc00000fd'))})
                target=f'failure-{index:02d}-{part}.txt'
                (dest/target).write_text('\n'.join(lines[i] for i in indices)+'\n')
                source=next((directory/name for name in ('build-inputs.json','suite-source-inputs.json') if (directory/name).is_file()),None)
                failure_selection.append(dict(finding=index,path=target,full_path=str(trace),full_sha256=digest(trace),source_manifest=str(source) if source else None,source_manifest_sha256=digest(source) if source else None,full_lines=len(lines),selected_lines=[i+1 for i in indices],selection='last six raw lines plus explicit panic/rejection/stack-overflow lines; complete artifact remains local'))
        save(dest/'failure-selection.json',failure_selection)
        files=[p for p in dest.rglob('*') if p.is_file()];require(len(files)<=200 and max(p.stat().st_size for p in files)<=256*1024 and sum(p.stat().st_size for p in files)<=4*1024*1024,'evidence budget exceeded')
    return report

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--run');parser.add_argument('--output');parser.add_argument('--supervisor',action='store_true');parser.add_argument('--acceptance');parser.add_argument('--destination');args=parser.parse_args()
    if args.acceptance:
        aggregate(args.acceptance,args.destination)
    elif args.supervisor:
        review_supervisor(args.run,args.output)
    else:
        review_run(args.run,args.output)
        run=next(p for p in Path(args.run).iterdir() if p.is_dir() and p.name.endswith('-run'))
        save(Path(args.output)/'mutation-check.json',mutation_check(rows_from_serial(run/'serial.log')))
