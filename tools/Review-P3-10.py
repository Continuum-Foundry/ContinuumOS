"""Independent P3-10 IPC decision matrix, provenance, lifetime and artifact review."""
import argparse
import copy
import importlib.util
import json
import shutil
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('supervision',ROOT/'tools/Review-P3-09.py')
supervision=importlib.util.module_from_spec(spec);spec.loader.exec_module(supervision)
waits,channel,cap,memory=supervision.waits,supervision.channel,supervision.cap,supervision.memory
require,read,save,digest,local=channel.require,channel.read,channel.save,channel.digest,channel.local

# Independent expected outcomes: intentionally not derived from Rust policy code.
DENIED={1,2,6,7,8,9,10,13,14,17,18,22,23,25,27,31,41,42,45,46,47,48,49}
EXPIRED={35,37,39,40,51,55,56}
EXPECTED={i:(3 if i in DENIED else 7 if i in EXPIRED else 13 if i==33 else 16 if i==34 else 0) for i in range(1,57) if i!=52}
EXPECTED.update({57:14,60:0,61:0,62:0})

def audit(rows):
    by={}
    for e,f in rows:by.setdefault(e,[]).append(f)
    require(len(by.get('m1.kernel_entry',[]))==1 and not any(e in by for e in ('m1.exception','m1.panic','m1.boot_rejected')),'reset/kernel fatal')
    ready=by.get('m1.policy_ready',[])
    require(len(ready)==1 and all(ready[0].get(k)==v for k,v in dict(waves='2',tasks='12',settled='true',user_broker='true').items()),'missing completion')
    require(0<int(ready[0]['fixed_bytes'])<=16777216,'fixed runtime bounds')
    admits=by.get('m1.policy_admit',[]);ends=by.get('m1.policy_end',[])
    require(len(admits)==len(ends)==12,'missing lifecycle')
    roster={};finish={}
    for wave in (0,1):
        aa=[f for f in admits if int(f['wave'])==wave]
        require({int(f['actor']) for f in aa}==set(range(6)),'missing actor')
        require(len({f['private_root'] for f in aa})==6,'shared address root')
        for f in aa:
            actor=int(f['actor']);key=(wave,actor);roster[key]=f
            require(f['principal']==str([303,404,101,202,101,505][actor]) and f['authority_session']==str([3003,4004,1001,2002,1001,5005][actor]+wave),'wrong principal/session')
            require(f['service']==str(actor in (0,1,5)).lower() and f['roster_readonly']=='true','wrong trusted binding')
            if wave:require(int(f['generation'])>int(roster[(0,actor)]['generation']),'reused process generation')
        ee=[f for f in ends if int(f['wave'])==wave]
        require({int(f['actor']) for f in ee}==set(range(6)),'missing owner completion')
        for f in ee:
            actor=int(f['actor']);finish[(wave,actor)]=f
            require((f['process'],f['generation'])==(roster[(wave,actor)]['process'],roster[(wave,actor)]['generation']),'retargeted process')
            require(int(f['code'])==(262 if actor==0 else 0) and int(f['vector'])==(6 if actor==0 else 0) and int(f['dispatches'])>0,'wrong process outcome')
    tokens=[f for f in by.get('m1.user_log',[]) if f['principal']=='303' and f['case']=='600']
    decisions={};decision_sequences={};scopes={};queued={};closures=set()
    for f in tokens:
        token=f['token'];parts=token.split('-')
        require(f['offset']=='0' and int(f['total'])==len(token),'fragmented diagnostic')
        if parts[0]=='d':
            wave,case,actor,process,generation,command,status,value,epoch,revision=map(int,parts[1:])
            key=(wave,case);require(key not in decisions,'duplicate decision')
            expected=EXPECTED.get(case,(13 if wave==1 else 0) if case==52 else None)
            require(expected is not None and status==expected,'wrong matrix outcome '+str(key))
            r=roster[(wave,actor)]
            require((process,generation)==(int(r['process']),int(r['generation'])),'decision not current actor')
            require(int(r['seq'])<int(f['seq'])<int(finish[(wave,actor)]['seq']),'decision outside actor lifetime')
            require(epoch==100+100*wave+(case>=30 and case not in (52,60,61,62)),'wrong policy epoch')
            require(revision==(1 if case<28 or case in (52,60,61,62) else 2),'unexpected mutation/rollback')
            # Every policy decision follows an actual kernel-stamped receive.
            prior=[x for x in by.get('m1.ipc_sender',[]) if int(x['seq'])<int(f['seq']) and x['receiver']==roster[(wave,0)]['process'] and x['generation']==roster[(wave,0)]['generation']]
            require(prior,'decision without IPC')
            actual=prior[-1]
            require((actual['sender'],actual['sender_generation'],actual['principal'],actual['authority_session'])==(r['process'],r['generation'],r['principal'],r['authority_session']),'payload/creator substituted for sender')
            values={52:1,3:0,4:1,5:1,11:2,12:1,15:3,16:1,19:4,20:1,21:1,24:2,26:5,28:2,29:2,30:0,32:1,36:2,38:0,43:3,44:2,50:0,53:4,54:0,60:0,61:0,62:0}
            require(value==(values.get(case,0) if status==0 else 0),'wrong grant/ticket/commit result')
            decision_sequences[key]=int(f['seq'])
            decisions[key]=dict(actor=actor,command=command,status=status,value=value,epoch=epoch,revision=revision)
        elif parts[0]=='s':
            wave,case,kind,resource,owner,operation,grant=map(int,parts[1:]);key=(wave,case)
            require(key not in scopes,'duplicate scope');scopes[key]=dict(kind=kind,resource=resource,owner=owner,operation=operation,grant=grant)
        elif parts[0]=='queued':
            case,start,expiry,end=map(int,parts[1:]);require(start<expiry<=end,'request did not expire while queued');queued.setdefault(case,[]).append((start,expiry,end,int(f['seq'])))
        elif parts[0]=='broker':closures.add(int(parts[1]))
        else:require(False,'unexpected broker diagnostic '+token)
    require(len(decisions)==2*len(EXPECTED)+2 and scopes.keys()==decisions.keys(),'incomplete decision matrix')
    for wave in (0,1):
        require({case for w,case in decisions if w==wave}==set(EXPECTED)|{52},'missing matrix row')
        require(decisions[(wave,51)]['actor']==4 and decisions[(wave,50)]['actor']==2,'delegation not exercised')
        require(scopes[(wave,44)]['kind']==2 and all(scopes[(wave,i)]['kind']==i-42 for i in range(45,49)),'privileged scope matrix')
        require(any(f['sender']==roster[(wave,2)]['process'] and f['sender_generation']==roster[(wave,2)]['generation'] and f['receiver']==roster[(wave,4)]['process'] and f['generation']==roster[(wave,4)]['generation'] and f['handles']=='1' and f['bytes']=='1' for f in by.get('m1.ipc_sender',[])),'missing actual endpoint transfer')
        for actor in range(2,6):
            require(any(f['token']==f'direct-{wave}-{actor}-survived' and int(f['seq'])>int(finish[(wave,0)]['seq']) for f in by.get('m1.user_log',[])),'direct authority did not survive broker death')
        require(any(f['token']==f'admin-{wave}-peer-closed' for f in by.get('m1.user_log',[])),'no fail-closed peer death')
    for wave in (0,1):
        for case in set(EXPECTED)|{52}:
            scope=scopes[(wave,case)]
            expected_kind={41:2,42:3,43:2,44:2,45:3,46:4,47:5,48:6}.get(case,1)
            require(scope['kind']==expected_kind and scope['resource']==(99 if case==10 else 2 if case==14 else 1) and scope['owner']==(202 if case in (8,14) else 101),'wrong resource/owner matrix')
            operation=4 if 41<=case<=48 else 2 if case in (13,17,23,24,27,28,29,31) else 1
            require(scope['operation']==operation,'wrong operation matrix')
    require(closures=={0,1} and set(queued)=={37,55} and all(len(v)==2 for v in queued.values()),'missing queued expiry/restarts')
    literal=[0]*20
    for index,value in {0:52,1:3,2:2,3:1,4:1,5:1,6:101,8:2**64-1,9:1,10:1,11:100,12:2**64-1,13:2}.items():literal[index]=value
    fingerprint=14695981039346656037
    for byte in b''.join(w.to_bytes(8,'little') for w in literal):fingerprint=((fingerprint^byte)*1099511628211)&(2**64-1)
    for wave in (0,1):
        require(any(f['token']==f'replay-{wave}-{fingerprint}' and f['principal']=='101' and f['case']=='602' for f in by.get('m1.user_log',[])),'payload bytes were not replayed identically')
    for case,assignment in ((37,36),(55,54)):
        for wave,(start,expiry,end,seq) in enumerate(queued[case]):
            admin=roster[(wave,1)]
            samples=[f for f in by.get('m1.wait_call',[]) if f['process']==admin['process'] and f['generation']==admin['generation'] and f['operation']=='17' and f['status']=='0' and int(f['seq'])<decision_sequences[(wave,assignment)]]
            require(samples and expiry==int(samples[-1]['value'])+500000000,'lease timestamp not derived from actual elapsed sample')
            before=[f for f in by.get('m1.ipc_call',[]) if f['process']==roster[(wave,0)]['process'] and f['generation']==roster[(wave,0)]['generation'] and f['operation']=='16' and f['status']=='0' and f['value']=='160' and decision_sequences[(wave,assignment)]<int(f['seq'])<seq]
            require(before,'no actual queued request before elapsed fence')
    settled=by.get('m1.policy_settled',[]);require(len(settled)==2,'missing settlements')
    for f in settled:
        require(all(f.get(k)=='0' for k in ('objects','handles','mappings','spaces','allocations','committed','resident','tables','stacks','pins','frames')),'resource leak')
    return dict(incarnations=2,process_runs=12,fixed_bytes=int(ready[0]['fixed_bytes']),decisions=len(decisions),denials=sum(d['status']!=0 for d in decisions.values()),exact_settlements=2,decision_matrix=[dict(wave=w,case=c,**decisions[(w,c)],**scopes[(w,c)]) for w,c in sorted(decisions)])

def mutations(rows):
    cases=[]
    for event in ('m1.policy_admit','m1.policy_end','m1.policy_settled','m1.policy_ready','m1.ipc_sender','m1.user_log'):
        cases.append(('remove '+event,[(e,f) for e,f in rows if e!=event]))
    for event,field,value in [('m1.policy_admit','generation','999'),('m1.policy_admit','principal','999'),('m1.policy_admit','private_root','0'),('m1.policy_admit','roster_readonly','false'),('m1.policy_end','code','99'),('m1.policy_end','vector','0'),('m1.policy_settled','frames','1'),('m1.policy_ready','settled','false'),('m1.ipc_sender','sender_generation','999')]:
        changed=copy.deepcopy(rows)
        for e,f in changed:
            if e==event:f[field]=value
        cases.append((event+'.'+field,changed))
    for case in EXPECTED|{52:13}:
        cases.append(('missing matrix '+str(case),[(e,f) for e,f in rows if not(e=='m1.user_log' and f.get('token','').startswith(('d-0-'+str(case)+'-','d-1-'+str(case)+'-')))]))
    changed=copy.deepcopy(rows)
    for e,f in changed:
        if e=='m1.user_log' and f.get('token','').startswith('queued-'):
            fields=f['token'].split('-');fields[2]=fields[3];f['token']='-'.join(fields);f['total']=str(len(f['token']))
    cases.append(('expiry preceded queue admission',changed))
    for case in EXPECTED|{52:13}:
        changed=copy.deepcopy(rows)
        for e,f in changed:
            if e=='m1.user_log' and f.get('token','').startswith('d-1-'+str(case)+'-'):
                fields=f['token'].split('-');fields[8]=str(int(fields[8])+1);f['token']='-'.join(fields);f['total']=str(len(f['token']))
        cases.append(('wrong matrix result '+str(case),changed))
    for name,changed in cases:
        try:audit(changed)
        except (ValueError,KeyError,IndexError):continue
        raise ValueError('accepted mutation: '+name)
    return [name for name,_ in cases]

def focused(folder):
    require(read(folder/'result.json')['status']=='passed','failed focused result')
    mf=read(folder/'build-manifest.json');require(mf['flavor']=='policy-test' and mf['kernel_sha256']==digest(folder/'kernel.elf'),'kernel identity')
    memory.elf_audit(folder/'kernel.elf');helpers=cap.codegen(folder)
    r,l,c=memory.lifecycle(folder/'policy-run')
    require(r['emulator_test_exit']==33 and l['fixture']=='policy' and r['build_id']==mf['build_id'],'launch identity')
    for image in read(folder/'user-images.json')['images']:
        name=image['name']+'.elf'
        require(image['sha256']==digest(folder/name)==c['package']['payload']['boot/'+name]==digest(folder/'policy-package/extracted payload/boot'/name),'ISO module identity')
    raw=folder/'policy-run/serial.log';rows=channel.raw_records(raw.read_bytes(),mf['build_id'],r['host_session_id'])
    return dict(audit=audit(rows),mutations=mutations(rows),build_id=mf['build_id'],kernel_sha256=mf['kernel_sha256'],iso_sha256=r['iso_sha256'],serial_sha256=digest(raw)),helpers


def aggregate(path):
    a=read(path)
    require(a['story']=='P3-10' and a['status']=='passed' and len(a['commands'])==12 and all(c['exit_code']==0 for c in a['commands']),'incomplete aggregate')
    runs={k:channel.local(v) for k,v in a['runs'].items()}
    pre=read(runs['prerequisites']/'prerequisites.json')
    require(pre['status']=='passed' and pre['offline'] and len(pre['commands'])==10 and all(c['exit_code']==0 for c in pre['commands']),'prerequisites')
    sources=read(runs['m1']/'suite-source-inputs.json')
    if 'prerequisite_reuse' in a:
        reuse=a['prerequisite_reuse'];manifest=channel.local(reuse['source_manifest'])
        require(reuse['source_manifest_sha256']==digest(manifest) and reuse['validated_files']==len(sources) and read(manifest)==sources,'invalid prerequisite reference/source proof')
    pre_sources={i['path']:i['sha256'] for i in read(runs['prerequisites']/'source-inputs.json')}
    require(all(pre_sources.get(name)==sha==digest(ROOT/name) for name,sha in sources.items()),'stale source/prerequisites')
    require(all(read(runs[k]/'build-inputs.json')['sources']==sources for k in ('memory','loader','boundary','scheduler','capabilities','channels','waits','supervisor','policy')),'different regression sources')
    m1=read(runs['m1']/'review.json')
    require(m1['status']=='passed' and len(m1['launches'])==42 and len(read(runs['m1']/'result.json')['stages']['rows'])==33 and m1['review_tool_sha256']==digest(ROOT/'tools/Review-P1-07.ps1'),'incomplete M1')
    for launch in m1['launches']:memory.lifecycle(channel.local(launch['directory']),launch['outcome']!='broken-before-entry')
    host=(runs['m1']/'kernel-host/check.stdout.log').read_text(encoding='utf-8-sig')
    require('82 passed; 0 failed' in host and host.count('wait::tests::')==9 and host.count('objects::tests::waiting::')==7 and host.count('objects::tests::channels::')==10,'host wait/atomicity coverage')
    wait_observed,_=waits.focused(runs['waits'])
    supervisor_observed,_=supervision.focused(runs['supervisor'])
    supervisor_folder=runs['supervisor'];smf=read(supervisor_folder/'build-manifest.json');sr=read(supervisor_folder/'supervisor-run/run-result.json')
    supervisor_rows=channel.raw_records((supervisor_folder/'supervisor-run/serial.log').read_bytes(),smf['build_id'],sr['host_session_id'])
    parked=[f for e,f in supervisor_rows if e=='m1.supervisor_end' and f['parked']=='true']
    require(len(parked)==4,'supervisor parked teardown coverage')
    for end in parked:
        require(any(e=='m1.supervisor_call' and f['operation']=='31' and f['target']==end['token'] and f['status']=='0' and f['value']=='4' and int(f['seq'])<int(end['seq']) for e,f in supervisor_rows),'missing explicit blocked-state barrier')

    report,helpers=focused(runs['policy'])
    report['supervisor_regression']=supervisor_observed['audit'];report['mutations']+=supervisor_observed['mutations']
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
    report.update(schema_version=1,story='P3-10',status='passed',scope='complete final-source story acceptance',review='local Codex self-review, not external approval',reviewer_sha256=digest(__file__),source_commit=a['source_commit'],acceptance_sha256=digest(path),kernel_host_tests=82,prerequisite_checks=10,m1_rows=33,m1_launches=42)
    report['criterion_review']={
        'provenance':'pass: six isolated address roots per incarnation, A/B/A2/forwarder plus exact trusted administrator; kernel-stamped actual IPC and delegated endpoint sender; forged payload and stale generations deny',
        'sharing':'pass: cross-owner and second-app default denial; explicit B read succeeds, write/wrong-resource fail; child attenuation and no child escalation',
        'forwarding':'pass: exact caller-issued single-use tickets plus both caller/service grants; weak caller, weak service, substitution and replay deny',
        'liveness':'pass: finite at-most-60-second session leases, grant/context/deadline/revision checks; queued grant and session expiry, epoch invalidation between preview and execution; no mutation on denial',
        'scopes':'pass: denied synthetic app installer/account requests; narrow explicit disk scope succeeds and account/process/diagnostic/service scopes deny',
        'restart':'pass: actual broker #UD twice; peer closure and surviving direct mappings; fresh process/session/epoch, empty-policy denial before trusted restore, old process/context/checkpoint replay denial and exact settlement',
    }
    report['unsafe_review']={
        'kernel':'single CPU IF-clear; existing common syscall and return validation; copied 40-byte liveness query returns only exact admitted tuple existence, never authority; no new assembly or user-selected privileged return state',
        'broker':'bounded allocation-free CPL3 policy/provider, exact bootstrap administrator and read-only roster; request words explicitly encoded little endian, never native-layout casts; no document contents or credentials in diagnostics',
        'commit':'mandatory original/service binding, session, role, epoch, expiry, revision and context checks form the use-time authorization boundary; an admitted operation may finish after subsequent caller death/elapsed expiry; policy changes serialize with actions; no authorization cache or deferred kernel continuation',
        'teardown':'quiescent kernel root/stack; remove waits then registry/maps/queued refs before process memory; publish fresh roots after roster mapping; two exact settlements',
        'limits':'broker/provider co-located; out-of-band fixture administrator is bounded by kernel binding lifetime rather than ordinary subject leases; trusted synthetic launch fixture, no credentials, durable RBAC, universal direct revocation, real privileged services, independent frozen SDK, live replacement or guest UI',
    }
    save(path.parent/'review.json',report)
    return report,helpers,a,runs,pre,m1,sources


def promote(dest,report,helpers,a,runs,pre,m1,sources):
    require(dest.is_relative_to(ROOT/'evidence/phase-3/P3-10') and not dest.exists(),'invalid/existing destination')
    dest.mkdir(parents=True)
    folder=runs['policy'];run=folder/'policy-run'
    for name,value in [('review.json',report),('acceptance.json',a),('prerequisites.json',pre),('m1-review.json',m1),('build-inputs.json',read(folder/'build-inputs.json')),('build-manifest.json',read(folder/'build-manifest.json')),('user-images.json',read(folder/'user-images.json')),('launch-context.json',dict(result=read(run/'run-result.json'),launch=read(run/'launch-manifest.json'),package=read(run/'case-result.json')['package']))]:save(dest/name,value)
    shutil.copyfile(runs['m1']/'kernel-host/check.stdout.log',dest/'host-tests.txt')
    extra=[Path(__file__),ROOT/'tools/Test-P3-10.ps1',ROOT/'tools/Check-P3-10Flavors.ps1',ROOT/'audit/check-docs.cjs',ROOT/'docs/05-delivery/phase-3/policy-runbook.md',ROOT/'docs/04-decisions/0026-ephemeral-scoped-policy-and-exact-forwarding-tickets.md',*[ROOT/'tools'/('Review-P3-0'+str(n)+'.py') for n in range(2,10)]]
    save(dest/'source-provenance.json',dict(source_commit=a['source_commit'],dirty_input_hashes={**sources,**{str(p.relative_to(ROOT)).replace(chr(92),'/'):digest(p) for p in extra}}))
    raw=(run/'serial.log').read_bytes();parts=[];part=b''
    for line in raw.splitlines(keepends=True):
        if len(part)+len(line)>200000:parts.append(part);part=b''
        part+=line
    if part:parts.append(part)
    for i,part in enumerate(parts):(dest/f'policy-serial-{i}.txt').write_bytes(part)
    (dest/'codegen').mkdir()
    for name in ('user_syscall','task_resume'):(dest/'codegen'/(name+'.txt')).write_text(helpers[name])
    require(sum(p.stat().st_size for p in dest.rglob('*') if p.is_file())<4*1024*1024,'evidence budget')



if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--policy',type=Path);parser.add_argument('--acceptance',type=Path);parser.add_argument('--destination',type=Path)
    args=parser.parse_args()
    if args.acceptance:
        report,helpers,a,runs,pre,m1,sources=aggregate(args.acceptance.resolve())
        if args.destination:promote(args.destination.resolve(),report,helpers,a,runs,pre,m1,sources)
    else:
        require(args.policy is not None,'policy or acceptance required')
        report,_=focused(args.policy.resolve())
        report=dict(schema_version=1,story='P3-10',status='passed',scope='focused probe; not final acceptance',**report)
    print(json.dumps(report,indent=2))
