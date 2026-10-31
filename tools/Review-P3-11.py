"""Independent P3-11 raw route/fence/outcome, provenance and regression review."""
import argparse
import copy
import importlib.util
import json
import shutil
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('previous',ROOT/'tools/Review-P3-10.py')
previous=importlib.util.module_from_spec(spec);spec.loader.exec_module(previous)
supervision=previous.supervision
waits,channel,cap,memory=previous.waits,previous.channel,previous.cap,previous.memory
require,read,save,digest,local=previous.require,previous.read,previous.save,previous.digest,previous.local

# Independently enumerated case expectations: status, authoritative revision,
# counter value, queue occupancy. Never generated from the Rust state machine.
MATRIX={
1:(3,0,0,0),2:(0,0,0,0),58:(9,0,0,0),3:(13,0,0,0),4:(0,0,0,0),59:(9,0,0,0),5:(0,0,0,0),
6:(3,0,0,0),7:(14,0,0,0),8:(14,0,0,0),9:(3,0,0,0),54:(14,0,0,0),
10:(0,1,1,0),11:(0,1,1,0),57:(0,1,1,0),12:(3,1,1,0),13:(0,2,2,0),14:(13,2,2,0),15:(13,2,2,0),
16:(0,2,2,1),56:(6,2,2,1),17:(0,2,2,2),18:(0,2,2,3),19:(0,2,2,4),20:(11,2,2,4),21:(8,2,2,3),
22:(0,3,3,2),23:(0,4,4,1),24:(0,5,5,0),25:(13,5,5,0),26:(3,5,5,0),
51:(0,5,5,0),52:(13,5,5,0),27:(0,6,6,0),28:(3,6,6,0),29:(13,7,7,0),
31:(7,7,7,0),32:(3,7,7,0),33:(0,7,7,0),34:(13,7,7,0),35:(0,8,8,0),36:(7,8,8,0),37:(7,8,8,0)}
ORDER=list(MATRIX)

def audit(rows):
    by={}
    for e,f in rows:by.setdefault(e,[]).append(f)
    ready=by.get('m1.route_ready',[])
    require(len(ready)==1 and all(ready[0].get(k)==v for k,v in dict(waves='9',tasks='54',settled='true',user_broker='true').items()),'missing completion')
    require(0<int(ready[0]['fixed_bytes'])<=16777216,'fixed runtime bounds')
    require(not any(e in by for e in ('m1.exception','m1.panic','m1.boot_rejected')),'kernel fatal')
    admits=by.get('m1.route_admit',[]);ends=by.get('m1.route_end',[])
    require(len(admits)==len(ends)==54,'missing actor lifecycle')
    roster={};finish={}
    victims={1:0,2:0,3:0,4:0,5:4,6:2,7:0,8:2}
    for wave in range(9):
        aa=[f for f in admits if int(f['wave'])==wave];ee=[f for f in ends if int(f['wave'])==wave]
        require({int(f['actor']) for f in aa}=={int(f['actor']) for f in ee}==set(range(6)),'missing actor')
        require(len({f['private_root'] for f in aa})==6 and all(int(f['private_root'])>0 for f in aa),'shared/private root')
        for f in aa:
            actor=int(f['actor']);roster[(wave,actor)]=f
            require(f['principal']==str([303,404,505,101,505,505][actor]) and f['authority_session']==str([3003,4004,5005,1001,5005,5005][actor]+wave),'wrong trusted principal/session')
            require(f['service']==str(actor!=3).lower() and f['roster_readonly']=='true','wrong trusted class')
            if wave:require(int(f['generation'])>int(roster[(wave-1,actor)]['generation']),'reused process generation')
        for f in ee:
            actor=int(f['actor']);finish[(wave,actor)]=f;r=roster[(wave,actor)]
            require((f['process'],f['generation'])==(r['process'],r['generation']),'retargeted lifetime')
            fault=actor==victims.get(wave)
            require(int(f['code'])==(262 if fault else 0) and int(f['vector'])==(6 if fault else 0) and int(f['dispatches'])>0,'wrong actor outcome')
    decisions={};states={};commits={};lost={};replays={};expired=[];replies={};retentions=[];wires={};corrupts={}
    for f in by.get('m1.user_log',[]):
        token=f['token'];require('route-failed' not in token and 'route-panic' not in token,'user failure')
        parts=token.split('-')
        if parts[0] not in ('d','state','commit','lost','replay','expired','direct','admin','reply','retention','wire','corrupt'):continue
        require(f['offset']=='0' and int(f['total'])==len(token),'fragmented diagnostic')
        if parts[0]=='d':
            wave,case,op,status,epoch,rev,value,queued=map(int,parts[1:]);key=(wave,case)
            require(key not in decisions and f['principal']=='303' and f['case']=='700','duplicate/non-authority decision')
            require(int(roster[(wave,0)]['seq'])<int(f['seq'])<int(finish[(wave,0)]['seq']),'decision outside authority lifetime')
            decisions[key]=dict(operation=op,status=status,epoch=epoch,revision=rev,value=value,queued=queued,seq=int(f['seq']))
        elif parts[0]=='state':
            wave,case,epoch,writer,published,policy=map(int,parts[1:]);require((wave,case) not in states,'duplicate state')
            require(writer in (0,4,5),'unknown writer');states[(wave,case)]=(epoch,writer,published,policy,int(f['seq']))
        elif parts[0]=='commit':
            wave,case,request,service,status,epoch,rev,value=map(int,parts[1:]);require((wave,case) not in commits,'duplicate commit')
            prior=[x for x in by.get('m1.ipc_sender',[]) if int(x['seq'])<int(f['seq']) and x['receiver']==roster[(wave,0)]['process'] and x['generation']==roster[(wave,0)]['generation']]
            require(prior,'commit without service IPC');sender=prior[-1];r=roster[(wave,service)]
            require((sender['sender'],sender['sender_generation'],sender['principal'],sender['authority_session'])==(r['process'],r['generation'],r['principal'],r['authority_session']),'commit substituted service sender')
            require((wave==5 or int(f['seq'])<int(finish[(wave,4)]['seq'])) and int(f['seq'])<int(finish[(wave,5)]['seq']),'services not concurrently alive')
            commits[(wave,case)]=(request,service,status,epoch,rev,value,int(f['seq']))
        elif parts[0]=='corrupt':
            wave,case,expected,received=map(int,parts[1:]);require((wave,case) not in corrupts,'duplicate corrupt reply');require(f['case']=='702' and f['principal']=='505','corruption not in proxy');corrupts[(wave,case)]=(expected,received,int(f['seq']))
        elif parts[0]=='wire':
            wave,case,expected,received=map(int,parts[1:]);require((wave,case) not in wires,'duplicate wire reply')
            owner=roster[(wave,0)];proxy=roster[(wave,2)]
            prior=[x for x in by.get('m1.ipc_sender',[]) if int(x['seq'])<int(f['seq']) and x['receiver']==owner['process'] and x['generation']==owner['generation']]
            require(prior and (prior[-1]['sender'],prior[-1]['sender_generation'],prior[-1]['principal'],prior[-1]['authority_session'])==(proxy['process'],proxy['generation'],proxy['principal'],proxy['authority_session']),'wire reply not received from actual proxy')
            wires[(wave,case)]=(expected,received,int(f['seq']),int(prior[-1]['seq']))
        elif parts[0]=='reply':
            wave,case,status=map(int,parts[1:]);require((wave,case) not in replies,'duplicate reply');replies[(wave,case)]=status
        elif parts[0]=='lost':
            wave,request,rev,value=map(int,parts[1:]);require(wave not in lost,'duplicate lost result');lost[wave]=(request,rev,value,int(f['seq']))
        elif parts[0]=='replay':
            wave,hash_=map(int,parts[1:]);require(f['principal']=='404','replay not administrator fixture');replays[wave]=hash_
        elif parts[0]=='retention':
            start,end=map(int,parts[1:]);require(end-start>=1_000_000_000,'result retention not elapsed');retentions.append(int(f['seq']))
        elif parts[0]=='expired':
            start,deadline,end=map(int,parts[1:]);require(start<deadline<=end,'no actual elapsed expiry');expired.append((start,deadline,end,int(f['seq'])))
    require([c for w,c in decisions if w==0]==ORDER,'incomplete/reordered decision matrix')
    for case,expect in MATRIX.items():
        d=decisions[(0,case)];require(tuple(d[k] for k in ('status','revision','value','queued'))==expect,'matrix outcome '+str(case))
        require(d['epoch']==(101 if ORDER.index(case)>=ORDER.index(25) else 100),'wrong route epoch')
    require(replies[(0,25)]==13 and replies[(0,29)]==13,'missing stale/correlation reply rejection')
    require(set(corrupts)=={(0,29)} and corrupts[(0,29)][:2]==(12,13) and wires[(0,29)][:2]==(12,13),'wrong correlation did not cross actual IPC')
    require(corrupts[(0,29)][2]<wires[(0,29)][3]<wires[(0,29)][2]<decisions[(0,29)]['seq'],'corrupt/send/receive/reject ordering')
    expected_requests={10:1,13:3,22:5,23:6,24:7,25:8,26:9,27:10,28:11,29:12,31:13,32:14,35:15,36:16}
    require({c for w,c in commits if w==0}==set(expected_requests),'missing authoritative commit attempts')
    for case,id_ in expected_requests.items():
        request,service,status,epoch,rev,value,seq=commits[(0,case)]
        require(request==id_ and service==(4 if case<=26 else 5),'wrong request/instance')
        require(status==(0 if case==29 else MATRIX[case][0]) and (rev,value)==MATRIX[case][1:3],'wrong mutation outcome')
    require(states[(0,250)][:3]==(101,0,100) and states[(0,251)][:3]==(101,5,100) and states[(0,252)][:3]==(101,5,101),'fence/grant/publication not separated')
    require(states[(0,250)][4]<states[(0,251)][4]<states[(0,252)][4]<commits[(0,25)][6]<commits[(0,26)][6]<commits[(0,27)][6],'delayed/retained writer ordering')
    for wave in range(1,9):
        e=100+wave*100
        common={c:MATRIX[c] for c in (1,2,58,3,4,59,5,10)};common[53]=(3,0,0,0)
        extras={1:{},2:{20:(0,1,1,0)},3:{20:(0,1,1,0),21:(0,1,1,0)},4:{20:(0,1,1,0),21:(0,1,1,0),22:(0,1,1,0)},5:{40:(9,1,1,0),42:(0,1,1,0),43:(0,1,1,0),44:(0,2,2,0)},6:{40:(9,1,1,0),42:(0,1,1,0),43:(0,1,1,0),44:(9,1,1,0)},7:{},8:{40:(9,2,2,0),41:(0,2,2,0)}}[wave]
        expected={**common,**extras};require({c for w,c in decisions if w==wave}==set(expected),'incomplete death matrix '+str(wave))
        for case,outcome in expected.items():
            d=decisions[(wave,case)];require(tuple(d[k] for k in ('status','revision','value','queued'))==outcome,'death outcome '+str((wave,case)))
        if wave in (1,2,3,4):
            s=states[(wave,0)]
            require(s[:3]=={1:(e,4,e),2:(e+1,0,e),3:(e+1,5,e),4:(e+1,5,e+1)}[wave],'wrong authority cut state')
            require(lost[wave][:3]==(0,1,1),'lost precommit state')
        elif wave==7:require(lost[wave][:3]==(2,2,2) and commits[(wave,40)][2:6]==(0,e,2,2),'lost commit must remain unknown')
        if wave in (5,6):require(not any(w==wave and c==40 for w,c in commits),'failed peer performed mutation')
        if wave==8:require(commits[(wave,40)][2:6]==(0,e,2,2),'reply loss erased commit')
    require(set(lost)=={1,2,3,4,7} and set(replays)==set(range(1,9)) and len(set(replays.values()))==1,'missing cuts/literal replay')
    literal=[0]*20
    for k,v in {0:53,1:10,3:1,4:1,5:0x525445,6:1,8:100,9:100,10:1,11:2,12:2**64-1,13:1,14:3,15:1,16:101,17:1001}.items():literal[k]=v
    hash_=14695981039346656037
    for byte in b''.join(v.to_bytes(8,'little') for v in literal):hash_=((hash_^byte)*1099511628211)&(2**64-1)
    require(all(v==hash_ for v in replays.values()),'different stale authority replay bytes')
    # Every origin-bearing admission/cancellation/dispatch receives real client IPC
    # before any service forwarding; the immutable payload cannot replace metadata.
    for (wave,case),d in decisions.items():
        if d['operation'] in (9,10,11,12,13):
            r=roster[(wave,3)];owner=roster[(wave,0)]
            previous_seq=max([int(roster[(wave,0)]['seq'])]+[x['seq'] for (w,c),x in decisions.items() if w==wave and x['seq']<d['seq']])
            actual=[f for f in by.get('m1.ipc_sender',[]) if previous_seq<int(f['seq'])<d['seq'] and f['receiver']==owner['process'] and f['generation']==owner['generation'] and f['sender']==r['process'] and f['sender_generation']==r['generation'] and f['principal']=='101' and f['authority_session']==r['authority_session'] and f['bytes']=='160']
            require(actual,'decision lacks actual current caller IPC')
    require(len(expired)==1 and len(retentions)==1 and decisions[(0,51)]['seq']<decisions[(0,52)]['seq'] and retentions[0]<decisions[(0,52)]['seq'],'missing elapsed queued/result expiry')
    require(commits[(0,31)][6]>expired[0][3],'expiry not before commit')
    owner=roster[(0,0)]
    arms=[f for f in by.get('m1.wait_call',[]) if f['process']==owner['process'] and f['generation']==owner['generation'] and f['operation']=='22' and f['status']=='0']
    require(len(arms)==2,'missing actual timer arms')
    require(expired[0][0]<=int(arms[1]['observed_ns'])<expired[0][1] and int(arms[1]['seq'])<expired[0][3],'expiry timer not armed before elapsed fence')
    for arm,completion in zip(arms,(retentions[0],expired[0][3])):
        require(any(f['process']==owner['process'] and f['generation']==owner['generation'] and f['operation']=='24' and f['pending']=='true' and int(arm['seq'])<int(f['seq'])<completion for f in by.get('m1.wait_call',[])),'timer not actually parked')

    # Transport edge 3->4 is the retained endpoint; 2->4/5 are proxy paths.
    for left,right in ((3,4),(2,4),(2,5)):
        l=roster[(0,left)];r=roster[(0,right)]
        require(any(f['sender']==l['process'] and f['sender_generation']==l['generation'] and f['receiver']==r['process'] and f['generation']==r['generation'] and f['bytes']=='160' for f in by.get('m1.ipc_sender',[])),'missing actual route/direct path')
    for wave in range(9):
        for actor in range(2,6):
            if actor==victims.get(wave):continue
            matching=[f for f in by.get('m1.user_log',[]) if f['token']==f'direct-{wave}-{actor}-survived']
            require(len(matching)==1,'missing surviving direct mapping')
            f=matching[0];r=roster[(wave,actor)]
            require(f['case']==str(700+actor) and f['principal']==r['principal'] and int(r['seq'])<int(f['seq'])<int(finish[(wave,actor)]['seq']),'direct mapping attributed to wrong actor')
            if victims.get(wave)==0:require(int(f['seq'])>int(finish[(wave,0)]['seq']),'direct mapping not observed after authority death')
        for actor in (0,4,2):
            if victims.get(wave)==actor:
                require(any(e=='m1.user_fault' and f['case']==str(700+actor) and f['vector']=='6' and int(roster[(wave,actor)]['seq'])<int(f['seq'])<int(finish[(wave,actor)]['seq']) for e,f in rows),'missing actual #UD')
    settled=by.get('m1.route_settled',[]);require(len(settled)==9,'missing settlements')
    for f in settled:require(all(f.get(k)=='0' for k in ('objects','handles','mappings','spaces','allocations','committed','resident','tables','stacks','pins','frames')),'resource leak')
    return dict(incarnations=9,process_runs=54,fixed_bytes=int(ready[0]['fixed_bytes']),decisions=len(decisions),mutation_attempts=len(commits),successful_commits=sum(v[2]==0 for v in commits.values()),faults=8,exact_settlements=9,decision_matrix=[dict(wave=w,case=c,**v) for (w,c),v in decisions.items()],cut_states={str(w):states[(w,40 if w==7 else 0)][:4] for w in (1,2,3,4,7)})

def mutations(rows):
    cases=[]
    for event in ('m1.route_admit','m1.route_end','m1.route_settled','m1.route_ready','m1.ipc_sender','m1.user_fault','m1.wait_call'):
        cases.append(('remove '+event,[(e,f) for e,f in rows if e!=event]))
    for event,field,value in [('m1.route_admit','generation','999'),('m1.route_admit','principal','999'),('m1.route_admit','private_root','0'),('m1.route_admit','roster_readonly','false'),('m1.route_end','code','99'),('m1.route_end','vector','99'),('m1.route_settled','frames','1'),('m1.route_ready','settled','false'),('m1.ipc_sender','sender_generation','999')]:
        changed=copy.deepcopy(rows)
        for e,f in changed:
            if e==event:f[field]=value
        cases.append((event+'.'+field,changed))
    for case in MATRIX:
        cases.append(('missing case '+str(case),[(e,f) for e,f in rows if not(e=='m1.user_log' and f.get('token','').startswith('d-0-'+str(case)+'-'))]))
        changed=copy.deepcopy(rows)
        for e,f in changed:
            if e=='m1.user_log' and f.get('token','').startswith('d-0-'+str(case)+'-'):
                parts=f['token'].split('-');parts[4]=str(int(parts[4])+1);f['token']='-'.join(parts);f['total']=str(len(f['token']))
        cases.append(('wrong case status '+str(case),changed))
    for prefix in ('state-0-250-','state-0-251-','state-0-252-','commit-0-25-','commit-0-26-','commit-0-27-','expired-','retention-','reply-0-25-','reply-0-29-','wire-0-29-','corrupt-','replay-','lost-','direct-'):
        cases.append(('remove '+prefix,[(e,f) for e,f in rows if not(e=='m1.user_log' and f.get('token','').startswith(prefix))]))
    for name,changed in cases:
        try:audit(changed)
        except (ValueError,KeyError,IndexError):continue
        raise ValueError('accepted mutation: '+name)
    return [name for name,_ in cases]

def focused(folder):
    require(read(folder/'result.json')['status']=='passed','failed focused result')
    mf=read(folder/'build-manifest.json');require(mf['flavor']=='route-test' and mf['kernel_sha256']==digest(folder/'kernel.elf'),'kernel identity')
    memory.elf_audit(folder/'kernel.elf');helpers=cap.codegen(folder)
    r,l,c=memory.lifecycle(folder/'routes-run')
    require(r['emulator_test_exit']==33 and l['fixture']=='routes' and r['build_id']==mf['build_id'],'launch identity')
    for image in read(folder/'user-images.json')['images']:
        name=image['name']+'.elf'
        require(image['sha256']==digest(folder/name)==c['package']['payload']['boot/'+name]==digest(folder/'routes-package/extracted payload/boot'/name),'ISO module identity')
    raw=folder/'routes-run/serial.log';rows=channel.raw_records(raw.read_bytes(),mf['build_id'],r['host_session_id'])
    return dict(audit=audit(rows),mutations=mutations(rows),build_id=mf['build_id'],kernel_sha256=mf['kernel_sha256'],iso_sha256=r['iso_sha256'],serial_sha256=digest(raw)),helpers

def aggregate(path):
    a=read(path)
    require(a['story']=='P3-11' and a['status']=='passed' and len(a['commands'])==13 and all(c['exit_code']==0 for c in a['commands']),'incomplete aggregate')
    runs={k:channel.local(v) for k,v in a['runs'].items()}
    pre=read(runs['prerequisites']/'prerequisites.json')
    require(pre['status']=='passed' and pre['offline'] and len(pre['commands'])==10 and all(c['exit_code']==0 for c in pre['commands']),'prerequisites')
    sources=read(runs['m1']/'suite-source-inputs.json')
    if 'prerequisite_reuse' in a:
        reuse=a['prerequisite_reuse'];manifest=channel.local(reuse['source_manifest'])
        require(reuse['source_manifest_sha256']==digest(manifest) and reuse['validated_files']==len(sources) and read(manifest)==sources,'invalid prerequisite reference/source proof')
    pre_sources={i['path']:i['sha256'] for i in read(runs['prerequisites']/'source-inputs.json')}
    require(all(pre_sources.get(name)==sha==digest(ROOT/name) for name,sha in sources.items()),'stale source/prerequisites')
    require(all(read(runs[k]/'build-inputs.json')['sources']==sources for k in ('memory','loader','boundary','scheduler','capabilities','channels','waits','supervisor','policy','routes')),'different regression sources')
    m1=read(runs['m1']/'review.json')
    require(m1['status']=='passed' and len(m1['launches'])==42 and len(read(runs['m1']/'result.json')['stages']['rows'])==33 and m1['review_tool_sha256']==digest(ROOT/'tools/Review-P1-07.ps1'),'incomplete M1')
    for launch in m1['launches']:memory.lifecycle(channel.local(launch['directory']),launch['outcome']!='broken-before-entry')
    host=(runs['m1']/'kernel-host/check.stdout.log').read_text(encoding='utf-8-sig')
    require('93 passed; 0 failed' in host and host.count('wait::tests::')==9 and host.count('objects::tests::waiting::')==7 and host.count('objects::tests::channels::')==10,'host wait/atomicity coverage')
    wait_observed,_=waits.focused(runs['waits'])
    supervisor_observed,_=supervision.focused(runs['supervisor'])
    supervisor_folder=runs['supervisor'];smf=read(supervisor_folder/'build-manifest.json');sr=read(supervisor_folder/'supervisor-run/run-result.json')
    supervisor_rows=channel.raw_records((supervisor_folder/'supervisor-run/serial.log').read_bytes(),smf['build_id'],sr['host_session_id'])
    parked=[f for e,f in supervisor_rows if e=='m1.supervisor_end' and f['parked']=='true']
    require(len(parked)==4,'supervisor parked teardown coverage')
    for end in parked:
        require(any(e=='m1.supervisor_call' and f['operation']=='31' and f['target']==end['token'] and f['status']=='0' and f['value']=='4' and int(f['seq'])<int(end['seq']) for e,f in supervisor_rows),'missing explicit blocked-state barrier')

    prior,helpers=previous.focused(runs['policy'])
    report,_=focused(runs['routes'])
    report['policy_regression']=prior['audit'];report['mutations']+=prior['mutations']
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
    report.update(schema_version=1,story='P3-11',status='passed',scope='complete final-source story acceptance',review='local Codex self-review, not external approval',reviewer_sha256=digest(__file__),source_commit=a['source_commit'],acceptance_sha256=digest(path),kernel_host_tests=93,prerequisite_checks=10,m1_rows=33,m1_launches=42)
    report['criterion_review']={
        'routing':'pass: authenticated control, separate writer grant/publication, compatible proxy path, bounded queue/results, unavailable/incompatible errors',
        'fence':'pass: two live private services, delayed old-epoch and retained direct-endpoint writes reject; only resource authority commits',
        'requests':'pass: pending/result dedupe, reorder/substitution/retention/cancellation, expiry and correlated stale reply rejection',
        'authorization':'pass: current exact sender/origin/service, finite policy/session grants, invalidation and literal stale-incarnation replay',
        'death':'pass: authority cuts around fence/grant/publication/commit/reply, service and proxy deaths, explicit surviving-owner rejection or declared unknown outcome, fresh bootstrap and exact settlements'
    }
    report['unsafe_review']={
        'kernel':'registered fixture hooks only; unchanged public dispatch and return-state validation; one CPU, IF-clear kernel transitions; no new assembly',
        'authority':'CPL3 bounded allocation-free routing/policy/resource owner; only commit changes counter; no service RW mapping to resource; exact generation identities and monotonically fenced epochs',
        'outcomes':'ephemeral resource/result loss after authority death is explicitly unknown, never blindly replayed; cancel after commit does not undo effect; high-water marks survive result expiry within incarnation',
        'direct_authority':'retained service endpoint survives switches but fails mandatory mutation fence; independent private RW mappings survive broker death until process teardown',
        'limits':'trusted synthetic bootstrap and fixture-controlled failure schedule, no durable activation/state migration/old-code retirement, no general discovery/SDK freeze/SMP/guest UI'
    }
    save(path.parent/'review.json',report)
    return report,helpers,a,runs,pre,m1,sources

def promote(dest,report,helpers,a,runs,pre,m1,sources):
    require(dest.is_relative_to(ROOT/'evidence/phase-3/P3-11') and not dest.exists(),'invalid/existing destination')
    dest.mkdir(parents=True)
    folder=runs['routes'];run=folder/'routes-run'
    for name,value in [('review.json',report),('acceptance.json',a),('prerequisites.json',pre),('m1-review.json',m1),('build-inputs.json',read(folder/'build-inputs.json')),('build-manifest.json',read(folder/'build-manifest.json')),('user-images.json',read(folder/'user-images.json')),('launch-context.json',dict(result=read(run/'run-result.json'),launch=read(run/'launch-manifest.json'),package=read(run/'case-result.json')['package']))]:save(dest/name,value)
    shutil.copyfile(runs['m1']/'kernel-host/check.stdout.log',dest/'host-tests.txt')
    extra=[Path(__file__),ROOT/'tools/Test-P3-11.ps1',ROOT/'tools/Check-P3-11Flavors.ps1',ROOT/'audit/check-docs.cjs',ROOT/'docs/05-delivery/phase-3/routes-runbook.md',ROOT/'docs/04-decisions/0027-ephemeral-service-routes-and-mandatory-writer-fences.md',*[ROOT/'tools'/f'Review-P3-{n:02d}.py' for n in range(2,11)]]
    save(dest/'source-provenance.json',dict(source_commit=a['source_commit'],dirty_input_hashes={**sources,**{str(p.relative_to(ROOT)).replace(chr(92),'/'):digest(p) for p in extra}}))
    raw=(run/'serial.log').read_bytes();parts=[];part=b''
    for line in raw.splitlines(keepends=True):
        if len(part)+len(line)>200000:parts.append(part);part=b''
        part+=line
    if part:parts.append(part)
    for i,part in enumerate(parts):(dest/f'routes-serial-{i}.txt').write_bytes(part)
    (dest/'codegen').mkdir()
    for name in ('user_syscall','task_resume'):(dest/'codegen'/(name+'.txt')).write_text(helpers[name])
    require(sum(p.stat().st_size for p in dest.rglob('*') if p.is_file())<4*1024*1024,'evidence budget')




if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--routes',type=Path);parser.add_argument('--acceptance',type=Path);parser.add_argument('--destination',type=Path)
    args=parser.parse_args()
    if args.acceptance:
        report,helpers,a,runs,pre,m1,sources=aggregate(args.acceptance.resolve())
        if args.destination:promote(args.destination.resolve(),report,helpers,a,runs,pre,m1,sources)
    else:
        require(args.routes is not None,'routes or acceptance required')
        report,_=focused(args.routes.resolve())
        report=dict(schema_version=1,story='P3-11',status='passed',scope='focused probe; not final acceptance',**report)
    print(json.dumps(report,indent=2))
