"""Independent P3-08 raw wait/clock/lifecycle and artifact review."""
import argparse
import copy
import importlib.util
import json
import shutil
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('channel', ROOT/'tools/Review-P3-07.py')
channel = importlib.util.module_from_spec(spec); spec.loader.exec_module(channel)
memory, cap = channel.memory, channel.cap
require, read, digest = channel.require, channel.read, channel.digest


def audit(rows):
    by = {}
    outstanding = {}
    completed = {}
    parks = wakes = kills = 0
    for event, f in rows:
        by.setdefault(event, []).append(f)
        if event == 'm1.wait_park':
            key = (f['thread'], f['generation'])
            require(key not in outstanding, 'double registration')
            require(1 <= int(f['items']) <= 8 and int(f['now_ns']) >= 0, 'wait bounds')
            outstanding[key] = f
            parks += 1
        elif event == 'm1.wait_wake':
            key = (f['thread'], f['generation'])
            require(key in outstanding and key in completed, 'unknown or unlatched/double wake')
            latched=completed.pop(key)
            require(all(f[k]==latched[k] for k in ('since_ns','now_ns','elapsed_ns','status','value')),'completion changed before admission')
            old = outstanding.pop(key)
            require(f['since_ns'] == old['now_ns'] and int(f['elapsed_ns']) == int(f['now_ns'])-int(f['since_ns']) >= 0, 'wake clock/order')
            status = int(f['status'])
            require(status in (0, 7, 8, 9) and (status == 0 or f['value'] == '0'), 'wake outcome')
            if status == 7:
                require(int(old['deadline_ns']) <= int(f['now_ns']), 'early timeout')
            if status == 8:
                require(old['cancel'] == 'true', 'unsolicited cancellation')
            wakes += 1
        elif event == 'm1.wait_complete':
            key=(f['thread'],f['generation']);require(key in outstanding and key not in completed,'unknown/double completion');completed[key]=f
            require(f['status'] in ('0','7','8','9') and (f['status']=='0' or f['value']=='0'),'latched outcome')
        elif event == 'm1.wait_settled':
            require(not outstanding and not completed and f['records'] == f['watches'] == '0', 'abandoned waits')
        elif event == 'm1.wait_kill':
            key=(f['thread'],f['generation']);require(key in outstanding and key not in completed and f['result']=='768','kill did not remove pending wait');outstanding.pop(key);kills+=1
    require(not outstanding and parks == wakes+kills and wakes>0 and kills==1, 'missing completion/termination')
    ready = by.get('m1.cap_ready', [])
    require(len(ready) == 1 and ready[0]['settled'] == 'true' and ready[0]['tasks'] == '18', 'missing lifecycle')
    require(0 < int(ready[0]['fixed_bytes']) <= 16777216 and int(ready[0]['switches']) < 2048, 'resource bound')
    require(len(by.get('m1.cap_reap', [])) == 18 and all(f['result'] == f['expected'] and f['result'] in ('0','768') for f in by['m1.cap_reap']) and sum(f['result']=='768' for f in by['m1.cap_reap'])==1, 'failed task')
    admits=by.get('m1.ipc_admit',[]);starts=by.get('m1.task_start',[])
    require(len(admits)==len(starts)==18,'incomplete trusted admission')
    bindings={}
    for f in admits:
        key=(f['process'],f['generation']);require(key not in bindings,'reused launch identity')
        require((f['principal'],f['authority_session'])==(('202','2002') if f['module']=='1' else ('101','1001')),'launch principal/session')
        bindings[key]=(f['principal'],f['authority_session'])
    threads={(f['thread'],f['generation']):(f['process'],f['generation']) for f in starts}
    require(len(threads)==18 and all(threads[k] in bindings for k in threads),'thread/process binding')
    for event,f in rows:
        if event in ('m1.wait_call','m1.wait_delay'):
            require((f['process'],f['generation']) in bindings,'wait from unknown generation')
        if event in ('m1.wait_park','m1.wait_complete','m1.wait_wake','m1.wait_resume','m1.wait_kill'):
            require((f['thread'],f['generation']) in threads,'wake/reap retargeted thread')
        if event=='m1.cap_reap':require(bindings[(f['process'],f['generation'])]==(f['principal'],f['authority_session']),'rebound teardown identity')
        if event=='m1.wait_kill':require(threads[(f['thread'],f['generation'])]==(f['process'],f['generation']) and bindings[(f['process'],f['generation'])][0]==f['principal'],'kill attribution')
    for killed in by.get('m1.wait_kill',[]):
        require(not any(e in ('m1.wait_park','m1.wait_complete','m1.wait_wake','m1.wait_resume') and f['thread']==killed['thread'] and f['generation']==killed['generation'] and int(f['seq'])>int(killed['seq']) for e,f in rows),'terminated owner resumed')
    settlements = [f for f in by.get('m1.cap_inventory', []) if f.get('phase') == 'settled']
    require(len(settlements) == 6, 'missing object settlements')
    for f in settlements:
        require(all(f[k] == '0' for k in ('owners','objects','handles','mappings','backing','mapped','frames','spaces','allocations','committed','resident','tables','stacks','pins')), 'object/memory leak')
    clocks = by.get('m1.wait_clock', [])
    require(len(clocks) == 1, 'missing clock comparison')
    clock = clocks[0]
    require(clock['source'] == 'hpet' and clock['irq_off'] == 'true' and clock['ticks_before'] == clock['ticks_after'], 'delivered-tick conflation')
    require(int(clock['after_ns'])-int(clock['before_ns']) >= 30000000 and 0 < int(clock['resolution_ns']) <= 100 and clock['observer_samples'] == '100' and int(clock['observer_total_ns']) > 0, 'elapsed/observer evidence')
    calls = by.get('m1.wait_call', [])
    errors = {(int(f['operation']), int(f['status'])) for f in calls}
    require({(24,7),(24,8),(24,1),(24,2),(24,4),(20,3),(20,4)} <= errors, 'missing typed wait coverage')
    require(sum(f['operation']=='24' and f['status']=='4' for f in calls)>=54,'missing empty/oversized/duplicate set denials')
    require({(25,6),(25,8),(25,0)}<=errors,'missing deadline/cancel-checked pressure attempts')
    infos={}
    for f in by.get('m1.cap_call',[]):
        if f['operation']=='6':infos.setdefault((f['process'],f['generation'],f['a']),[]).append(f)
    retained=[]
    for key,observed in infos.items():
        live=[f for f in observed if f['status']=='0' and int(f['value'])>>32==5]
        gone=[f for f in observed if f['status']=='1']
        attempts=[f for f in calls if (f['process'],f['generation'])==key[:2] and f['operation']=='25']
        if len(live)>=2 and gone and attempts:
            require(int(live[0]['seq'])<int(live[1]['seq'])<int(gone[-1]['seq']),'transfer settlement order')
            require(any(f['status']=='8' and int(f['seq'])<int(live[1]['seq']) for f in attempts),'cancel changed source before acknowledgement')
            require(any(f['status']=='0' and int(live[1]['seq'])<int(f['seq'])<int(gone[-1]['seq']) for f in attempts),'move was not an accepted enqueue')
            retained.append(key)
    require(len(retained)==1,'missing exact retained-then-moved authority')
    tokens = [f['token'] for f in by.get('m1.user_log', [])]
    require(tokens.count('wait-local-ok') == 18 and tokens.count('wait-role-ok') == 17 and 'wait-failed' not in tokens, 'guest assertions')
    for token,count in {'wait-pressure-sender-ok':1,'wait-pressure-receiver-ok':1,'wait-pressure-control-ok':1,'wait-cycle-control-ok':1,'wait-cycle-two-ended':2,'wait-cycle-three-ended':3,'wait-victim-ready':1,'wait-peer-death-ok':1,'wait-healthy-control-ok':1,'wait-cancel-delivery-ok':1,'wait-boundaries-ok':1,'wait-pulse-reset-ok':1,'wait-pulse-cancel-ok':1}.items():require(tokens.count(token)==count,'missing '+token)
    boundaries={(f['action'],f['phase']) for f in by.get('m1.wait_boundary',[])}
    require(boundaries=={(str(a),str(p)) for a in range(70,74) for p in range(3)}|{('74','1'),('75','2')},'registration boundary coverage')
    delayed=by.get('m1.wait_delay',[]);require(len(delayed)==1,'missing delayed IRQ experiment')
    d=delayed[0];require(d['ticks_before']==d['ticks_after'] and d['irq_off']=='true' and int(d['after_ns'])-int(d['before_ns'])>=30000000,'elapsed extension while IRQ-off')
    require(any(f['operation']=='24' and f['status']=='7' and f['process']==d['process'] and f['generation']==d['generation'] and int(f['seq'])>int(d['seq']) and int(f['deadline_ns'])<int(d['after_ns'])<=int(f['observed_ns']) for f in calls),'delayed elapsed timeout not confirmed')
    require(len(by.get('m1.wait_settled', [])) == 6 and int(by['m1.wait_settled'][-1]['halts']) > 0, 'missing idle halt')
    resumes=by.get('m1.wait_resume',[]);require(len(resumes)==wakes,'missing resumption observation')
    require(Counter((f['thread'],f['generation'],f['ready_ns']) for f in resumes)==Counter((f['thread'],f['generation'],f['now_ns']) for f in by['m1.wait_wake']),'missing/duplicate dispatch observation')
    latencies=[]
    for f in resumes:
        value=int(f['resume_ns'])-int(f['ready_ns']);require(value==int(f['wake_latency_ns'])>=0,'wake dispatch clock');latencies.append(value)
    return dict(tasks=18, parks=parks, wakes=wakes, kills=kills, calls=len(calls), fixed_bytes=int(ready[0]['fixed_bytes']), clock=clock, wake_dispatch_max_ns=max(latencies), errors=sorted(errors))


def mutations(rows):
    cases = []
    for event in ('m1.wait_park','m1.wait_complete','m1.wait_wake','m1.wait_settled','m1.wait_clock','m1.wait_kill','m1.wait_resume','m1.wait_boundary','m1.wait_delay','m1.ipc_admit','m1.cap_ready','m1.cap_reap','m1.user_log'):
        cases.append(('missing '+event, [(e,f) for e,f in rows if e != event]))
    for event, key, value in [('m1.wait_wake','generation','999'),('m1.wait_wake','status','99'),('m1.wait_wake','since_ns','0'),('m1.wait_settled','records','1'),('m1.wait_clock','source','delivered-ticks'),('m1.wait_clock','ticks_after','999'),('m1.cap_ready','fixed_bytes','16777217'),('m1.cap_reap','result','99')]:
        changed = copy.deepcopy(rows)
        for e, f in changed:
            if e == event: f[key] = value
        cases.append((event+'.'+key, changed))
    for name, changed in cases:
        try: audit(changed)
        except (ValueError,KeyError): continue
        raise ValueError('accepted mutation: '+name)
    return [name for name, _ in cases]


def focused(folder):
    require(read(folder/'result.json')['status'] == 'passed', 'failed focused run')
    manifest = read(folder/'build-manifest.json')
    require(manifest['flavor'] == 'wait-test' and manifest['kernel_sha256'] == digest(folder/'kernel.elf'), 'kernel identity')
    memory.elf_audit(folder/'kernel.elf')
    helpers = cap.codegen(folder)
    run = folder/'waits-run'
    result, launch, case = memory.lifecycle(run)
    require(result['emulator_test_exit'] == 33 and launch['fixture'] == 'waits' and result['build_id'] == manifest['build_id'], 'launch/exit')
    for item in read(folder/'user-images.json')['images']:
        name = item['name']+'.elf'
        require(item['sha256'] == digest(folder/name) == case['package']['payload']['boot/'+name] == digest(folder/'waits-package/extracted payload/boot'/name), 'actual ISO module')
    raw = (run/'serial.log').read_bytes()
    rows = channel.raw_records(raw, manifest['build_id'], result['host_session_id'])
    return dict(audit=audit(rows), mutations=mutations(rows), build_id=manifest['build_id'], kernel_sha256=manifest['kernel_sha256'], iso_sha256=result['iso_sha256'], serial_sha256=digest(run/'serial.log')), helpers


def aggregate(path):
    a=read(path)
    require(a['story']=='P3-08' and a['status']=='passed' and len(a['commands'])==10 and all(c['exit_code']==0 for c in a['commands']),'incomplete aggregate')
    runs={k:channel.local(v) for k,v in a['runs'].items()}
    pre=read(runs['prerequisites']/'prerequisites.json')
    require(pre['status']=='passed' and pre['offline'] and len(pre['commands'])==10 and all(c['exit_code']==0 for c in pre['commands']),'prerequisites')
    sources=read(runs['m1']/'suite-source-inputs.json')
    if 'prerequisite_reuse' in a:
        reuse=a['prerequisite_reuse'];manifest=channel.local(reuse['source_manifest'])
        require(reuse['source_manifest_sha256']==digest(manifest) and reuse['validated_files']==len(sources) and read(manifest)==sources,'invalid prerequisite reference/source proof')
    pre_sources={i['path']:i['sha256'] for i in read(runs['prerequisites']/'source-inputs.json')}
    require(all(pre_sources.get(name)==sha==digest(ROOT/name) for name,sha in sources.items()),'stale source/prerequisites')
    require(all(read(runs[k]/'build-inputs.json')['sources']==sources for k in ('memory','loader','boundary','scheduler','capabilities','channels','waits')),'different regression sources')
    m1=read(runs['m1']/'review.json')
    require(m1['status']=='passed' and len(m1['launches'])==42 and len(read(runs['m1']/'result.json')['stages']['rows'])==33 and m1['review_tool_sha256']==digest(ROOT/'tools/Review-P1-07.ps1'),'incomplete M1')
    for launch in m1['launches']:memory.lifecycle(channel.local(launch['directory']),launch['outcome']!='broken-before-entry')
    host=(runs['m1']/'kernel-host/check.stdout.log').read_text(encoding='utf-8-sig')
    require('64 passed; 0 failed' in host and host.count('wait::tests::')==9 and host.count('objects::tests::waiting::')==7 and host.count('objects::tests::channels::')==9,'host wait/atomicity coverage')
    report,helpers=focused(runs['waits'])
    for name,reviewer in [('capabilities',cap),('channels',channel)]:
        observed,_=reviewer.focused(runs[name]);report[name+'_regression']=observed.get('audit',observed);report['mutations']+=observed.get('mutations',[])
    scheduled=runs['scheduler'];mf=read(scheduled/'build-manifest.json')
    require(read(scheduled/'result.json')['status']=='passed' and mf['kernel_sha256']==digest(scheduled/'kernel.elf'),'scheduler artifact')
    memory.elf_audit(scheduled/'kernel.elf');channel.scheduler.codegen(scheduled)
    report['scheduler_regression']=[]
    for suffix,trace in [('on',True),('off',False)]:
        run=scheduled/('scheduler-'+suffix+'-run');r,_,_=memory.lifecycle(run)
        rows=memory.records((run/'serial.log').read_bytes(),mf['build_id'],r['host_session_id'])
        report['scheduler_regression'].append(channel.scheduler.audit(rows,trace,True));report['mutations']+=channel.scheduler.mutations(rows,trace,True)
    run=scheduled/'scheduler-cpu-run';r,l,_=memory.lifecycle(run)
    rows=memory.records((run/'serial.log').read_bytes(),mf['build_id'],r['host_session_id'])
    require('-sse2' in l['arguments'][l['arguments'].index('-cpu')+1] and any(e=='m1.scheduler_cpu_rejected' and f['user_entries']=='0' and not int(f['cpuid_edx'],0)&(1<<26) for e,f in rows) and not any(e=='m1.task_start' for e,f in rows),'real CPU feature rejection')
    for key,fixture in [('memory','owned-memory'),('loader','static-loader')]:
        run=runs[key]/(fixture+'-run');r,_,_=memory.lifecycle(run)
        rows=memory.records((run/'serial.log').read_bytes(),read(runs[key]/'build-manifest.json')['build_id'],r['host_session_id'])
        report[key+'_regression']=memory.table_audit(rows) if key=='memory' else channel.loader.audit(rows,[channel.loader.expected_pages(runs[key]/(n+'.elf')) for n in ('user_a','user_b','service')])
    for group,suffix in enumerate('abcd'):
        run=runs['boundary']/('user-boundary-'+suffix+'-run');r,_,_=memory.lifecycle(run)
        rows=memory.records((run/'serial.log').read_bytes(),read(runs['boundary']/'build-manifest.json')['build_id'],r['host_session_id'])
        channel.boundary.audit(rows,group);report['mutations']+=channel.boundary.mutation_checks(rows,group)
    cap.boundary_codegen(runs['boundary'])
    report.update(schema_version=1,story='P3-08',status='passed',scope='complete final-source story acceptance',review='local Codex self-review, not external approval',reviewer_sha256=digest(__file__),source_commit=a['source_commit'],acceptance_sha256=digest(path),kernel_host_tests=64,prerequisite_checks=10,m1_rows=33,m1_launches=42)
    report['criterion_review']={
        'registration':'pass: raw three-boundary signal/enqueue/close and cancel/delivery injections; checked bounds/types/duplicates; host maximum-record and stale-generation matrix; FIFO admission',
        'elapsed_deadlines':'pass: checked HPET epoch, hardware resolution and observer cost; actual 30-ms IRQ-off comparisons and expired wait; host wrap/fraction/overflow/coalescing',
        'authority_races':'pass: full-queue timed/cancelled move retains exact source handle before later accepted move; copy-before-commit with final admission check; immutable confirmed completion',
        'progress_teardown':'pass: real two/three-party request cycles, independent control progress, parked owner killed without resume, peer-close wake and following healthy wave; exact six settlements',
        'idle_regressions':'pass: actual empty-runnable halt/wake, raw dispatch-preparation latency with observer cost; complete M1/timer and separate memory/loader/boundary/scheduler/capability/channel regression flavors',
    }
    report['unsafe_review']={
        'parking':'only copied handles and exact generations survive; no user spans, pins, registry/runtime borrows or kernel stack continuation; shared validated return before capture',
        'exclusion':'one CPU IF-clear; recursive registry/runtime domains assert; capture/idle require released domains; forced nested-copy IRQs preserved in old registered fixtures',
        'completion':'one immutable wait completion; termination drops wait before registry/address-space destruction; dead owner cannot return to CPL3; wake modifies status/value only',
        'clock':'validated UC HPET only, checked delta/u128 conversion and fail-closed overflow; delivered uptime epoch unchanged',
        'limits':'one owner thread per process; no SMP, wall-clock, async executor, hard-real-time/display/input latency, automatic universal deadlock detection, general public supervision or independent SDK freeze',
    }
    channel.save(path.parent/'review.json',report)
    return report,helpers,a,runs,pre,m1,sources


def promote(dest,report,helpers,a,runs,pre,m1,sources):
    require(dest.is_relative_to(ROOT/'evidence/phase-3/P3-08') and not dest.exists(),'invalid/existing evidence destination')
    dest.mkdir(parents=True)
    folder=runs['waits'];run=folder/'waits-run'
    for name,value in [('review.json',report),('acceptance.json',a),('prerequisites.json',pre),('m1-review.json',m1),('build-inputs.json',read(folder/'build-inputs.json')),('build-manifest.json',read(folder/'build-manifest.json')),('user-images.json',read(folder/'user-images.json')),('launch-context.json',dict(result=read(run/'run-result.json'),launch=read(run/'launch-manifest.json'),package=read(run/'case-result.json')['package']))]:channel.save(dest/name,value)
    shutil.copyfile(runs['m1']/'kernel-host/check.stdout.log',dest/'host-tests.txt')
    extra=[Path(__file__),ROOT/'tools/Test-P3-08.ps1',ROOT/'audit/check-docs.cjs',ROOT/'docs/05-delivery/phase-3/waits-runbook.md',ROOT/'docs/04-decisions/0024-elapsed-clock-and-bounded-wait-completion.md',*[ROOT/'tools'/('Review-P3-0'+str(n)+'.py') for n in range(2,8)]]
    channel.save(dest/'source-provenance.json',dict(source_commit=a['source_commit'],dirty_input_hashes={**sources,**{str(p.relative_to(ROOT)).replace('\\','/'):digest(p) for p in extra}}))
    raw=(run/'serial.log').read_bytes();parts=[];part=b''
    for line in raw.splitlines(keepends=True):
        if len(part)+len(line)>200000:parts.append(part);part=b''
        part+=line
    if part:parts.append(part)
    for i,part in enumerate(parts):(dest/f'waits-serial-{i}.txt').write_bytes(part)
    (dest/'codegen').mkdir()
    for name in ('user_syscall','task_resume'):(dest/'codegen'/(name+'.txt')).write_text(helpers[name])
    require(sum(p.stat().st_size for p in dest.rglob('*') if p.is_file())<4*1024*1024,'evidence budget')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--waits', type=Path)
    parser.add_argument('--acceptance', type=Path)
    parser.add_argument('--destination', type=Path)
    args = parser.parse_args()
    if args.acceptance:
        report,helpers,a,runs,pre,m1,sources=aggregate(args.acceptance.resolve())
        if args.destination:promote(args.destination.resolve(),report,helpers,a,runs,pre,m1,sources)
    else:
        require(args.waits is not None,'waits or acceptance required')
        report,_=focused(args.waits.resolve())
        report=dict(schema_version=1,story='P3-08',status='passed',scope='focused wait probe; not full story acceptance',**report)
    print(json.dumps(report,indent=2))
