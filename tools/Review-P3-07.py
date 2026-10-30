"""Independent raw channel/provenance, artifact and final-regression review."""
import argparse
import copy
import importlib.util
import json
from pathlib import Path
import re
import shutil

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('cap', ROOT/'tools/Review-P3-06.py')
cap = importlib.util.module_from_spec(spec); spec.loader.exec_module(cap)
memory, loader, boundary, scheduler = cap.memory, cap.loader, cap.boundary, cap.scheduler
require, read, save, digest, local = memory.require, memory.read, memory.save, memory.digest, memory.local


def raw_records(raw, build, session):
    require(raw.endswith(b'\n') and len(raw) <= 2*1024*1024, 'incomplete/oversized serial')
    rows=[]
    for line in raw.splitlines():
        if not line.startswith(b'm1.'): continue
        require(len(line)<=512, 'oversized record')
        words=line.decode('ascii').split(); pairs=[w.split('=',1) for w in words[1:]]
        require(all(len(p)==2 for p in pairs), 'bad field')
        f=dict(pairs)
        require(len(f)==len(pairs), 'duplicate field')
        require(f['build']==build and f['session']==session and f['version']=='5' and f['session_origin']=='host'
                and f['seq']==str(len(rows)) and f['outcome']=='ok' and f['emergency']=='false', 'identity/order/fatal')
        rows.append((words[0],f))
    require(rows[0][0]=='m1.kernel_entry' and rows[-1][0]=='m1.ready', 'incomplete guest')
    return rows


def audit(rows):
    by={}
    for e,f in rows: by.setdefault(e,[]).append(f)
    require(len(by.get('m1.kernel_entry',[]))==1 and not any(e in by for e in ('m1.exception','m1.panic','m1.boot_rejected','m1.user_fault')), 'fatal/reset')
    ready=by.get('m1.cap_ready',[]);require(len(ready)==1,'missing completion');ready=ready[0]
    require(all(ready.get(k)==v for k,v in dict(waves='2',tasks='6',settled='true',user_execution='true').items()),'incomplete experiment')
    require(0<int(ready['fixed_bytes'])<=16777216 and 0<int(ready['switches'])<2048,'precharge/dispatch bound')
    admits=by.get('m1.ipc_admit',[]);ends=by.get('m1.cap_reap',[])
    require(len(admits)==len(ends)==len(by.get('m1.task_start',[]))==6,'incomplete lifecycle')
    bindings={}
    for f in admits:
        module=int(f['module']);p=int(f['principal']);s=int(f['authority_session'])
        require((p,s)==((202,2002) if module==1 else (101,1001)), 'launch binding')
        key=(f['process'],f['generation']);require(key not in bindings,'reused process binding')
        bindings[key]=(f['principal'],f['authority_session'])
    for f in ends:
        require(f['result']==f['expected']=='0' and (f['principal'],f['authority_session'])==bindings[(f['process'],f['generation'])], 'failed/rebound owner')
    settled=[f for f in by.get('m1.cap_inventory',[]) if f.get('phase')=='settled']
    require(len(settled)==2,'missing exact settlements')
    for f in settled:
        for k in ('owners','objects','handles','mappings','backing','mapped','frames','spaces','allocations','committed','resident','tables','stacks','pins'):
            require(f[k]=='0','unsettled '+k)
    senders=by.get('m1.ipc_sender',[]);require(senders,'missing trusted metadata')
    delegated=set()
    for f in senders:
        require((f['principal'],f['authority_session'])==bindings[(f['sender'],f['sender_generation'])], 'forged historical sender')
        require((f['receiver'],f['generation']) in bindings,'unknown receiver')
        require(0<=int(f['bytes'])<=4096 and 0<=int(f['handles'])<=8,'receive bounds')
        if f['bytes']=='64': delegated.add((f['sender'],f['receiver'],f['principal']))
    require({('0','1','101'),('1','0','202'),('1','2','202'),('2','1','101')}<=delegated,'missing cross-principal/same-user delegation')
    # Queued final memory must be received after its sender was reaped.
    for f in [f for f in senders if f['bytes']=='64' and f['handles']=='1']:
        require(any(e['process']==f['sender'] and e['generation']==f['sender_generation'] and int(e['seq'])<int(f['seq']) for e in ends),'receive did not follow sender death')
    require(len([f for f in senders if f['bytes']=='64' and f['handles']=='1'])==2,'missing orphan queue transfer')
    calls=by.get('m1.ipc_call',[]);require(calls,'missing transport calls')
    errors={(int(f['operation']),int(f['status'])) for f in calls}
    require({(14,1),(14,3),(14,4),(14,6),(14,9),(15,4),(15,5),(15,6),(15,11)}<=errors,'missing typed failure coverage')
    require(any(f['operation']=='14' and f['status']=='6' and f['messages']=='8' for f in calls),'no count exhaustion')
    require(any(f['operation']=='14' and f['status']=='6' and f['bytes']=='16384' for f in calls),'no byte exhaustion')
    # Inventory is global: the full 32-slot caller plus two peers with three grants each.
    require(any(f['operation']=='15' and f['status']=='11' and f['handles']=='38' for f in calls),'no destination table exhaustion')
    for f in calls:
        require(f['status']=='0' or f['value']=='0','error leaked scalar')
        require(0<=int(f['pairs'])<=8 and 0<=int(f['messages'])<=128 and 0<=int(f['bytes'])<=262144 and 0<=int(f['held'])<=256 and 0<=int(f['objects'])<=128 and 0<=int(f['handles'])<=512,'transport inventory bound')
    previous=None; preserved=0
    inventory_keys=('pairs','messages','bytes','held','objects','handles')
    for e,f in rows:
        if e=='m1.ipc_call':
            state=tuple(f[k] for k in inventory_keys)
            if f['status']!='0' and previous is not None:
                require(state==previous,'rejected operation changed inventory');preserved+=1
            previous=state
        elif e in ('m1.ipc_admit','m1.cap_reap','m1.cap_inventory') or e=='m1.cap_call' and f['operation'] not in ('4','6'):
            previous=None
    require(preserved>=10,'insufficient observable rollback checks')
    require(len([f for f in by.get('m1.user_log',[]) if f.get('token')=='channel-queued-peer-death-pass'])==1,'missing queued receiver process death')
    for token in ('channel-negatives-pass','channel-client-pass','channel-service-pass','channel-delegate-pass'):
        require(len([f for f in by.get('m1.user_log',[]) if f.get('token')==token])==2,'missing '+token)
    require(not any(f.get('token')=='channel-failed' for f in by.get('m1.user_log',[])),'guest assertion failure')
    return dict(tasks=6,waves=2,syscalls=len(calls),receives=len(senders),preserved_failure_inventories=preserved,fixed_bytes=int(ready['fixed_bytes']),switches=int(ready['switches']),errors=sorted(errors))


def mutations(rows):
    cases=[]
    for event in ('m1.cap_ready','m1.ipc_admit','m1.cap_reap','m1.cap_inventory','m1.ipc_sender','m1.ipc_call','m1.user_log'):
        cases.append(('missing '+event,[(e,f) for e,f in rows if e!=event]))
    for event,key,value in [('m1.ipc_sender','principal','999'),('m1.ipc_sender','sender_generation','999'),('m1.cap_reap','result','99'),('m1.cap_inventory','objects','99'),('m1.cap_ready','fixed_bytes','16777217'),('m1.ipc_call','held','999')]:
        changed=copy.deepcopy(rows)
        for e,f in changed:
            if e==event: f[key]=value
        cases.append((event+'.'+key,changed))
    for name,changed in cases:
        try: audit(changed)
        except (ValueError,KeyError): continue
        raise ValueError('accepted mutation: '+name)
    return [name for name,_ in cases]


def focused(folder):
    require(read(folder/'result.json')['status']=='passed','failed focused result')
    mf=read(folder/'build-manifest.json');require(mf['flavor']=='channel-test' and mf['kernel_sha256']==digest(folder/'kernel.elf'),'wrong kernel')
    memory.elf_audit(folder/'kernel.elf');helpers=cap.codegen(folder)
    helpers['user_copy']=boundary.excerpt((folder/'llvm-objdump.stdout.log').read_text(),'user_copy')
    run=folder/'channels-run';result,launch,case=memory.lifecycle(run)
    require(result['emulator_test_exit']==33 and launch['fixture']=='channels' and result['build_id']==mf['build_id'],'launch identity/exit')
    for item in read(folder/'user-images.json')['images']:
        require(item['sha256']==digest(folder/(item['name']+'.elf'))==case['package']['payload']['boot/'+item['name']+'.elf']==digest(folder/'channels-package/extracted payload/boot'/(item['name']+'.elf')),'actual ISO module')
    raw=(run/'serial.log').read_bytes();rows=raw_records(raw,mf['build_id'],result['host_session_id'])
    return dict(audit=audit(rows),mutations=mutations(rows),build_id=mf['build_id'],kernel_sha256=mf['kernel_sha256'],iso_sha256=result['iso_sha256'],serial_sha256=digest(run/'serial.log')),helpers


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--channels',type=Path);p.add_argument('--acceptance',type=Path);p.add_argument('--destination',type=Path);args=p.parse_args()
    a=read(args.acceptance) if args.acceptance else None
    folder=local(a['runs']['channels']) if a else args.channels.resolve()
    observed,helpers=focused(folder)
    report=dict(schema_version=1,story='P3-07',status='passed',review='local Codex self-review, not external approval',scope='focused guest/artifact audit',reviewer_sha256=digest(__file__),**observed)
    if a:
        require(a['story']=='P3-07' and a['status']=='passed' and len(a['commands'])==9 and all(c['exit_code']==0 for c in a['commands']),'incomplete aggregate')
        runs={k:local(v) for k,v in a['runs'].items()};pre=read(runs['prerequisites']/'prerequisites.json')
        require(pre['status']=='passed' and pre['offline'] and len(pre['commands'])==10 and all(c['exit_code']==0 for c in pre['commands']),'prerequisites')
        sources=read(runs['m1']/'suite-source-inputs.json');pre_sources={i['path']:i['sha256'] for i in read(runs['prerequisites']/'source-inputs.json')}
        require(all(pre_sources.get(path)==sha==digest(ROOT/path) for path,sha in sources.items()),'stale final-source prerequisites')
        require(all(read(runs[k]/'build-inputs.json')['sources']==sources for k in ('memory','loader','boundary','scheduler','capabilities','channels')),'different regression sources')
        m1=read(runs['m1']/'review.json');require(m1['status']=='passed' and len(m1['launches'])==42 and len(read(runs['m1']/'result.json')['stages']['rows'])==33 and m1['review_tool_sha256']==digest(ROOT/'tools/Review-P1-07.ps1'),'incomplete M1')
        for launch in m1['launches']:
            memory.lifecycle(local(launch['directory']),launch['outcome']!='broken-before-entry')
        host=(runs['m1']/'kernel-host/check.stdout.log').read_text(encoding='utf-8-sig');require('47 passed; 0 failed' in host and host.count('objects::tests::channels::')==8,'missing host coverage')
        old_objects,_=cap.focused(runs['capabilities']);report['capability_regression']=old_objects['audit'];report['mutations']+=old_objects['mutations']
        for key,fixture in [('memory','owned-memory'),('loader','static-loader')]:
            run=runs[key]/(fixture+'-run');r,_,_=memory.lifecycle(run);rows=memory.records((run/'serial.log').read_bytes(),read(runs[key]/'build-manifest.json')['build_id'],r['host_session_id'])
            report[key+'_regression']=memory.table_audit(rows) if key=='memory' else loader.audit(rows,[loader.expected_pages(runs[key]/(n+'.elf')) for n in ('user_a','user_b','service')])
        for group,suffix in enumerate('abcd'):
            run=runs['boundary']/('user-boundary-'+suffix+'-run');r,_,_=memory.lifecycle(run);rows=memory.records((run/'serial.log').read_bytes(),read(runs['boundary']/'build-manifest.json')['build_id'],r['host_session_id']);boundary.audit(rows,group);report['mutations']+=boundary.mutation_checks(rows,group)
        cap.boundary_codegen(runs['boundary'])
        for suffix,trace in [('on',True),('off',False)]:
            run=runs['scheduler']/('scheduler-'+suffix+'-run');r,_,_=memory.lifecycle(run);rows=memory.records((run/'serial.log').read_bytes(),read(runs['scheduler']/'build-manifest.json')['build_id'],r['host_session_id']);scheduler.audit(rows,trace,True);report['mutations']+=scheduler.mutations(rows,trace,True)
        scheduler.codegen(runs['scheduler'])
        run=runs['scheduler']/'scheduler-cpu-run';r,l,_=memory.lifecycle(run);rows=memory.records((run/'serial.log').read_bytes(),read(runs['scheduler']/'build-manifest.json')['build_id'],r['host_session_id']);require('-sse2' in l['arguments'][l['arguments'].index('-cpu')+1] and any(e=='m1.scheduler_cpu_rejected' and f['user_entries']=='0' and not int(f['cpuid_edx'],0)&(1<<26) for e,f in rows),'real CPU rejection')
        report.update(scope='complete final-source story acceptance',source_commit=a['source_commit'],acceptance_sha256=digest(args.acceptance),kernel_host_tests=47,prerequisite_checks=10,m1_rows=33,m1_launches=42,boundary_process_runs=36,scheduler_tasks=60)
        report['criterion_review']={
            'bounded_codecs':'pass: independent literal byte fixture, malformed envelope matrix, real ring-3 encoder/service decoder and typed mandatory-feature rejection',
            'atomic_authority':'pass: attenuated memory/endpoint moves and duplicates; source invalidation only after enqueue; copy/sizing/table failure inventories preserved; host nonce boundaries',
            'trusted_provenance':'pass: independent replay of process-generation/principal/session launch bindings and delegated sender/receiver tuples; payload claims 999/777 grant no identity',
            'bounded_admission':'pass: actual guest count/byte/table/buffer failures plus host attachment quota, pair/table/nonce admission and duplicate-source/rights rejection',
            'teardown':'pass: actual queued receiver process death and receipt after sender death; host queued memory and cross-pair cycles; exact object/backing/frame/table/stack/pin settlements',
        }
        report['unsafe_review']={
            'copies':'bounded input records copied once; output spans checked for overflow/overlap; existing effective-page checks, recovery and balanced pins; partial output bytes unspecified on error',
            'commit':'one CPU, IF-clear registry; output callback finishes before all receiver installations/dequeue; no fallible resource reservation after commit; no acknowledgement syscall',
            'ownership':'queues retain their own attenuated references; bounded root tracing removes unreachable cycles; supervisor root/stack precedes process removal and backend destruction',
            'identity':'only trusted launcher supplies historical process-generation/principal/session binding; payload and endpoint origin cannot choose sender identity',
            'limits':'one owner thread; no SMP, waits/deadlines, policy/session liveness enforcement, generic exactly-once, universal revoke, independent frozen SDK or guest UI',
        }
        save(args.acceptance.parent/'review.json',report)
        if args.destination:
            dest=args.destination.resolve();require(dest.is_relative_to(ROOT/'evidence/phase-3/P3-07') and not dest.exists(),'invalid/existing destination');dest.mkdir(parents=True)
            for name,value in [('review.json',report),('acceptance.json',a),('prerequisites.json',pre),('m1-review.json',m1),('build-inputs.json',read(folder/'build-inputs.json')),('build-manifest.json',read(folder/'build-manifest.json')),('user-images.json',read(folder/'user-images.json'))]:save(dest/name,value)
            shutil.copyfile(runs['m1']/'kernel-host/check.stdout.log',dest/'host-tests.txt')
            reviewers=[ROOT/'tools'/('Review-P3-0'+str(n)+'.py') for n in range(2,7)]
            save(dest/'source-provenance.json',dict(source_commit=a['source_commit'],dirty_input_hashes={**sources,**{str(p.relative_to(ROOT)).replace('\\','/'):digest(p) for p in [Path(__file__),ROOT/'tools/Test-P3-07.ps1',ROOT/'tools/Review-P1-07.ps1',ROOT/'docs/05-delivery/phase-3/channels-runbook.md',*reviewers]}}))
            # Keep all channel raw records in bounded chunks with full raw hash.
            raw=(folder/'channels-run/serial.log').read_bytes();chunks=[];part=b''
            for line in raw.splitlines(keepends=True):
                if len(part)+len(line)>200000:chunks.append(part);part=b''
                part+=line
            if part:chunks.append(part)
            for i,part in enumerate(chunks):(dest/f'channels-serial-{i}.txt').write_bytes(part)
            run=folder/'channels-run';save(dest/'launch-context.json',dict(result=read(run/'run-result.json'),launch=read(run/'launch-manifest.json'),package=read(run/'case-result.json')['package']))
            (dest/'codegen').mkdir()
            for name in ('user_syscall','user_copy','task_resume'):
                (dest/'codegen'/(name+'.txt')).write_text(helpers[name])
            require(sum(p.stat().st_size for p in dest.rglob('*') if p.is_file())<4*1024*1024,'bundle budget')
    print(json.dumps(report,indent=2))

if __name__=='__main__':main()
