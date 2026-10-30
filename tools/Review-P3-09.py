"""Independent P3-09 raw lifecycle, admission, teardown and artifact consumer."""
import argparse
import copy
import importlib.util
import json
import shutil
from collections import Counter
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('waits',ROOT/'tools/Review-P3-08.py')
waits=importlib.util.module_from_spec(spec);spec.loader.exec_module(waits)
channel,cap,memory=waits.channel,waits.cap,waits.memory
require,read,save,digest,local=channel.require,channel.read,channel.save,channel.digest,channel.local

def audit(rows):
    by={}
    known={};states={};remaining={};ended={};dispatched=Counter();roots=[];units={};threads={}
    parks={};killed_parks=0;requests={};denials=set();joins=[];last=0
    def stop_descendants(target):
        for token in known:
            at=token
            for _ in range(16):
                if at==target:
                    if states[token] in ('prepared','running'):states[token]='stopping'
                    break
                at=int(known[at]['parent'])
                if at==0:break
    for event,f in rows:
        by.setdefault(event,[]).append(f)
        if event=='m1.supervisor_admit':
            t=int(f['token']);p=int(f['parent']);require(t>last,'reused/nonmonotonic token');last=t
            require(t not in known and f['private_handles']=='true' and f['memory']=='1048576','admission bounds/grants')
            if p==0:
                roots.append(t);require(f['principal']=='303' and f['service']=='true' and f['grants']=='3','root binding')
                require(int(f['authority_session'])==3002+len(roots),'fresh bootstrap session')
                units[t]=8;remaining[t]=7
            else:
                require(p in known and states[p]=='running','unknown/nonrunning parent')
                require(f['principal']==known[p]['principal'] and f['authority_session']==known[p]['authority_session'] and f['service']=='false' and int(f['grants']) & ~int(known[p]['grants'])==0,'rebound child authority')
            known[t]=f;states[t]='prepared'
            key=(f['thread'],f['generation']);require(key not in threads,'reused thread generation');threads[key]=t
        elif event=='m1.supervisor_request':
            c=int(f['caller']);require(c in known and states[c]=='running','request from dead/unknown caller')
            requests[c]=f
        elif event=='m1.supervisor':
            t=int(f['token']);action=f['action'];require(t in known,'unknown lifecycle token')
            if action=='bootstrap':
                require(states[t]=='prepared' and known[t]['parent']=='0' and f['closed']=='false','bootstrap state')
                states[t]='running'
            elif action=='create':
                p=int(f['parent']);n=int(f['code']);q=requests[p]
                require(states[t]=='prepared' and int(known[t]['parent'])==p and n==int(q['units']) and 0<n<=remaining[p],'subtree credit bypass')
                require(q['threads']=='1' and q['priority']=='0' and q['inherited']=='0' and q['memory']=='1048576','structural escalation')
                remaining[p]-=n;units[t]=n;remaining[t]=n-1
            elif action=='start':
                c=int(f['parent']);require(states[t]=='prepared' and c in (roots[-1],int(known[t]['parent'])),'unauthorized start')
                states[t]='running'
            elif action=='terminate':
                c=int(f['parent']);require(c in (roots[-1],int(known[t]['parent'])) and states[t] in ('prepared','running','stopping','reaped'),'unauthorized kill')
                stop_descendants(t)
            elif action=='reap':
                require(states[t] in ('prepared','running','stopping') and t not in ended,'double/invalid reap')
                stop_descendants(t)
                states[t]='reaped';ended[t]=f
            elif action=='join':
                c=int(f['parent']);require(states[t]=='reaped' and c in (roots[-1],int(known[t]['parent'])),'early/unauthorized join')
                require(not any(int(v['parent'])==t and states[k]!='joined' for k,v in known.items()),'parent recycled with descendants')
                require((f['code'],f['vector'])==(ended[t]['code'],ended[t]['vector']),'join changed exit')
                states[t]='joined';joins.append(t)
            elif action=='admission-closed':
                require(t==roots[-1] and f['closed']=='true' and f['code']=='3','root failure not closed')
                stop_descendants(t)
            elif action=='restart':
                require(t==roots[-1] and f['closed']=='false','unauthorized restart permit')
            else:raise ValueError('unknown action '+action)
        elif event=='m1.supervisor_dispatch':
            t=int(f['token']);require(t in known and states[t]=='running' and f['validated']=='true','entry before start/after reap')
            require(all(f[k]==known[t][k] for k in ('process','generation','thread','mode')),'retargeted saved context')
            dispatched[t]+=1
        elif event=='m1.supervisor_call':
            c=int(f['caller']);require(c in known and states[c]=='running','call after death')
            op=int(f['operation']);status=int(f['status'])
            if op==27 and status:
                q=requests.get(c)
                if q:
                    b=known[c]
                    if q['principal']!=b['principal']:denials.add('principal')
                    if q['authority_session']!=b['authority_session']:denials.add('session')
                    for key,value in [('service','0'),('grants','1'),('memory','1048576'),('threads','1'),('priority','0'),('inherited','0')]:
                        if q[key]!=value:denials.add(key)
                    if status==11 and q['threads']=='1':denials.add('units')
            if status==13:require(int(f['target']) in joins,'stale result without joined token')
        elif event=='m1.wait_park':
            key=(f['thread'],f['generation']);require(key in threads and states[threads[key]]=='running' and key not in parks,'dead/unknown/double wait');parks[key]=f
        elif event=='m1.wait_wake':
            key=(f['thread'],f['generation']);require(key in parks and states[threads[key]]=='running','dead/unknown wake');parks.pop(key)
        elif event=='m1.wait_complete':
            key=(f['thread'],f['generation']);require(key in parks and states[threads[key]] in ('running','stopping'),'completion after destruction')
        elif event=='m1.wait_resume':
            key=(f['thread'],f['generation']);require(key in threads and states[threads[key]]=='running','resumed dead/unknown waiter')
        elif event=='m1.supervisor_end':
            t=int(f['token']);require(t in ended and states[t]=='reaped','end before resource reap')
            require(all(f[k]==known[t][k] for k in ('process','generation','thread','mode')),'end identity')
            require((f['code'],f['vector'])==(ended[t]['code'],ended[t]['vector']),'end outcome mismatch')
            require(int(f['dispatches'])==dispatched[t],'missing/extra dispatch')
            key=(f['thread'],f['generation'])
            if f['parked']=='true':require(key in parks and f['code']=='768','parked owner not terminated');parks.pop(key);killed_parks+=1
            else:require(key not in parks,'dangling wait')
            mode=int(f['mode'])
            expected={1000:0,1001:262,1002:1,1003:2,1005:768,1006:768,1007:768,1200:0,1201:262}
            if 500<=mode<=503:code=262 if mode==503 else 0
            elif 1100<=mode<=1104:code=0
            else:code=expected[mode]
            # A prepared normal child is intentionally killed without dispatch.
            if mode==1000 and dispatched[t]==0:code=768
            require(int(f['code'])==code and int(f['vector'])==(6 if code==262 else 0),'unexpected child/root outcome')
            if mode==1005:require(int(f['preemptions'])>0 and dispatched[t]>0,'CPU hog was not preempted')
        elif event=='m1.supervisor_settled':
            require(all(s in ('reaped','joined') for s in states.values()) and not parks,'live task/wait at epoch boundary')
            require(all(f[k]=='0' for k in ('objects','handles','mappings','committed','resident','spaces','allocations','tables','stacks','pins','frames','nodes')),'resource leak')
    require(len(roots)==4 and len(by.get('m1.supervisor_settled',[]))==4,'incomplete epochs')
    require(len(ended)==len(known)==len(by.get('m1.supervisor_end',[])) and not parks,'incomplete reaping')
    require({'principal','session','service','grants','memory','threads','priority','inherited','units'}<=denials,'missing ceiling/identity denials')
    require(killed_parks>=4,'missing blocked child/parent/root teardown')
    require(len([f for f in by['m1.supervisor'] if f['action']=='restart'])==2,'retry storm or missing retry')
    require(sum(f['operation']=='32' and f['status']=='11' for f in by['m1.supervisor_call'])==1,'missing quarantine')
    require(sum(f['action']=='admission-closed' for f in by['m1.supervisor'])==4,'admission reopened on death')
    tokens=[f['token'] for f in by.get('m1.user_log',[])]
    require('supervisor-failed' not in tokens and tokens.count('quota-healthy')==5 and tokens.count('root-failure-ready')==1 and tokens.count('panic')==1,'failed/missing user assertions')
    require(tokens.count('healthy-progress')==4,'healthy progress missing')
    caps={(int(f['operation']),int(f['status'])) for f in by.get('m1.cap_call',[])}
    require({(8,11),(10,11)}<=caps,'missing handle/mapping quota results')
    calls={(int(f['operation']),int(f['status'])) for f in by.get('m1.wait_call',[])}
    require({(19,11),(21,11)}<=calls,'missing object/timer quota results')
    require(any(f['operation']=='14' and f['status']=='11' for f in by.get('m1.ipc_call',[])),'missing message ceiling')
    ready=by.get('m1.supervisor_ready',[])
    require(len(ready)==1 and all(ready[0].get(k)==v for k,v in dict(waves='4',settled='true',user_execution='true',fail_closed='true').items()) and 0<int(ready[0]['fixed_bytes'])<=16777216,'missing/bounded completion')
    return dict(epochs=4,admissions=len(known),reaped=len(ended),joins=len(joins),dispatches=sum(dispatched.values()),parked_owner_kills=killed_parks,retries=2,quarantine=1,exact_settlements=4,fixed_bytes=int(ready[0]['fixed_bytes']))

def mutations(rows):
    cases=[]
    for event in ('m1.supervisor_admit','m1.supervisor','m1.supervisor_dispatch','m1.supervisor_end','m1.supervisor_settled','m1.supervisor_ready','m1.supervisor_call','m1.supervisor_request','m1.user_log'):
        cases.append(('remove '+event,[(e,f) for e,f in rows if e!=event]))
    for event,key,value in [('m1.supervisor_admit','token','1'),('m1.supervisor_admit','authority_session','3003'),('m1.supervisor_dispatch','generation','999'),('m1.wait_park','generation','999'),('m1.supervisor_end','code','99'),('m1.supervisor_end','parked','false'),('m1.supervisor_settled','frames','1'),('m1.supervisor_ready','fixed_bytes','16777217')]:
        changed=copy.deepcopy(rows)
        for e,f in changed:
            if e==event:f[key]=value
        cases.append((event+'.'+key,changed))
    for name,changed in cases:
        try:audit(changed)
        except (ValueError,KeyError):continue
        raise ValueError('accepted mutation: '+name)
    return [name for name,_ in cases]

def focused(folder):
    require(read(folder/'result.json')['status']=='passed','failed focused result')
    mf=read(folder/'build-manifest.json');require(mf['flavor']=='supervisor-test' and mf['kernel_sha256']==digest(folder/'kernel.elf'),'kernel identity')
    memory.elf_audit(folder/'kernel.elf');helpers=cap.codegen(folder)
    r,l,c=memory.lifecycle(folder/'supervisor-run')
    require(r['emulator_test_exit']==33 and l['fixture']=='supervisor' and r['build_id']==mf['build_id'],'launch identity')
    for image in read(folder/'user-images.json')['images']:
        name=image['name']+'.elf'
        require(image['sha256']==digest(folder/name)==c['package']['payload']['boot/'+name]==digest(folder/'supervisor-package/extracted payload/boot'/name),'actual ISO modules')
    raw=folder/'supervisor-run/serial.log';rows=channel.raw_records(raw.read_bytes(),mf['build_id'],r['host_session_id'])
    return dict(audit=audit(rows),mutations=mutations(rows),build_id=mf['build_id'],kernel_sha256=mf['kernel_sha256'],iso_sha256=r['iso_sha256'],serial_sha256=digest(raw)),helpers


def aggregate(path):
    a=read(path)
    require(a['story']=='P3-09' and a['status']=='passed' and len(a['commands'])==11 and all(c['exit_code']==0 for c in a['commands']),'incomplete aggregate')
    runs={k:channel.local(v) for k,v in a['runs'].items()}
    pre=read(runs['prerequisites']/'prerequisites.json')
    require(pre['status']=='passed' and pre['offline'] and len(pre['commands'])==10 and all(c['exit_code']==0 for c in pre['commands']),'prerequisites')
    sources=read(runs['m1']/'suite-source-inputs.json')
    if 'prerequisite_reuse' in a:
        reuse=a['prerequisite_reuse'];manifest=channel.local(reuse['source_manifest'])
        require(reuse['source_manifest_sha256']==digest(manifest) and reuse['validated_files']==len(sources) and read(manifest)==sources,'invalid prerequisite reference/source proof')
    pre_sources={i['path']:i['sha256'] for i in read(runs['prerequisites']/'source-inputs.json')}
    require(all(pre_sources.get(name)==sha==digest(ROOT/name) for name,sha in sources.items()),'stale source/prerequisites')
    require(all(read(runs[k]/'build-inputs.json')['sources']==sources for k in ('memory','loader','boundary','scheduler','capabilities','channels','waits','supervisor')),'different regression sources')
    m1=read(runs['m1']/'review.json')
    require(m1['status']=='passed' and len(m1['launches'])==42 and len(read(runs['m1']/'result.json')['stages']['rows'])==33 and m1['review_tool_sha256']==digest(ROOT/'tools/Review-P1-07.ps1'),'incomplete M1')
    for launch in m1['launches']:memory.lifecycle(channel.local(launch['directory']),launch['outcome']!='broken-before-entry')
    host=(runs['m1']/'kernel-host/check.stdout.log').read_text(encoding='utf-8-sig')
    require('73 passed; 0 failed' in host and host.count('wait::tests::')==9 and host.count('objects::tests::waiting::')==7 and host.count('objects::tests::channels::')==9,'host wait/atomicity coverage')
    wait_observed,_=waits.focused(runs['waits'])
    report,helpers=focused(runs['supervisor'])
    report['waits_regression']=wait_observed['audit'];report['mutations']+=wait_observed['mutations']
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
    report.update(schema_version=1,story='P3-09',status='passed',scope='complete final-source story acceptance',review='local Codex self-review, not external approval',reviewer_sha256=digest(__file__),source_commit=a['source_commit'],acceptance_sha256=digest(path),kernel_host_tests=73,prerequisite_checks=10,m1_rows=33,m1_launches=42)
    report['criterion_review']={
        'lifecycle':'pass: actual CPL3 create/start/observe/terminate/join; precise normal/fault/panic/OOM/kill outcomes and healthy progress beside preempted CPU hogs',
        'quotas':'pass: prepaid ancestor/descendant reservations; live handle/map and lifetime object/timer/pair/message/byte/backing budgets; retained transfers cannot refund credits; reserved control envelope',
        'authority':'pass: current caller principal/session, service/grant/memory/thread/priority/inheritance denials; direct parent/root control; monotonic tokens and stale-target denial after join',
        'teardown':'pass: prepared, running, parked, faulted, dead-parent and supervisor-death paths; no saved continuation or dispatch after destruction; four exact object/frame/table/stack/pin/wait/tree settlements',
        'restart':'pass: two retry permits, host boundary backoff checks, guest quarantine, epoch launch ceiling and admission closure on supervisor death; explicit trusted restore uses fresh session',
    }
    report['unsafe_review']={
        'entry':'unchanged shared return validation and eager FP; only trusted scalar startup IDs are new; no user-selected return IP, selectors, privileged flags or executable bytes',
        'admission':'one CPU IF-clear; derive and narrow authenticated context; loader/object preparation unwinds before publication; no user buffer/pin or Rust continuation retained',
        'reaping':'quiescent kernel root/stack; drop waits first, remove registry/maps/queued authority, destroy allocator/address-space/stack, clear task, publish Reaped; monotonic tokens and parent lineage outlive execution',
        'quotas':'lifetime budgets are deliberately conservative; no refunds before epoch settlement, including retained references; no production elastic service manager or hard CPU guarantee',
        'limits':'one thread/process, priority zero, eight units/epoch, immutable ISO images, no nonempty inheritance, no arbitrary install, SMP, persistent recovery, policy/session liveness, frozen SDK or guest UI',
    }
    save(path.parent/'review.json',report)
    return report,helpers,a,runs,pre,m1,sources


def promote(dest,report,helpers,a,runs,pre,m1,sources):
    require(dest.is_relative_to(ROOT/'evidence/phase-3/P3-09') and not dest.exists(),'invalid/existing destination')
    dest.mkdir(parents=True)
    folder=runs['supervisor'];run=folder/'supervisor-run'
    for name,value in [('review.json',report),('acceptance.json',a),('prerequisites.json',pre),('m1-review.json',m1),('build-inputs.json',read(folder/'build-inputs.json')),('build-manifest.json',read(folder/'build-manifest.json')),('user-images.json',read(folder/'user-images.json')),('launch-context.json',dict(result=read(run/'run-result.json'),launch=read(run/'launch-manifest.json'),package=read(run/'case-result.json')['package']))]:save(dest/name,value)
    shutil.copyfile(runs['m1']/'kernel-host/check.stdout.log',dest/'host-tests.txt')
    extra=[Path(__file__),ROOT/'tools/Test-P3-09.ps1',ROOT/'tools/Check-P3-09Flavors.ps1',ROOT/'audit/check-docs.cjs',ROOT/'docs/05-delivery/phase-3/supervision-runbook.md',ROOT/'docs/04-decisions/0025-prepaid-subtrees-and-quiescent-process-reaping.md',*[ROOT/'tools'/('Review-P3-0'+str(n)+'.py') for n in range(2,9)]]
    save(dest/'source-provenance.json',dict(source_commit=a['source_commit'],dirty_input_hashes={**sources,**{str(p.relative_to(ROOT)).replace(chr(92),'/'):digest(p) for p in extra}}))
    raw=(run/'serial.log').read_bytes();parts=[];part=b''
    for line in raw.splitlines(keepends=True):
        if len(part)+len(line)>200000:parts.append(part);part=b''
        part+=line
    if part:parts.append(part)
    for i,part in enumerate(parts):(dest/f'supervisor-serial-{i}.txt').write_bytes(part)
    (dest/'codegen').mkdir()
    for name in ('user_syscall','task_resume'):(dest/'codegen'/(name+'.txt')).write_text(helpers[name])
    require(sum(p.stat().st_size for p in dest.rglob('*') if p.is_file())<4*1024*1024,'evidence budget')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--supervisor',type=Path);parser.add_argument('--acceptance',type=Path);parser.add_argument('--destination',type=Path)
    args=parser.parse_args()
    if args.acceptance:
        report,helpers,a,runs,pre,m1,sources=aggregate(args.acceptance.resolve())
        if args.destination:promote(args.destination.resolve(),report,helpers,a,runs,pre,m1,sources)
    else:
        require(args.supervisor is not None,'supervisor or acceptance required')
        report,_=focused(args.supervisor.resolve())
        report=dict(schema_version=1,story='P3-09',status='passed',scope='focused probe; not final acceptance',**report)
    print(json.dumps(report,indent=2))
