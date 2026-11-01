"""Independent SDK, version/ownership, portable-runtime and source review."""
import argparse
import copy
import importlib.util
import json
from pathlib import Path
import shutil
import re

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('routes_review',ROOT/'tools/Review-P3-11.py')
routes=importlib.util.module_from_spec(spec);spec.loader.exec_module(routes)
require,read,save,digest=routes.require,routes.read,routes.save,routes.digest

def codegen(folder):
    text=(folder/'llvm-objdump.stdout.log').read_text()
    instructions=lambda t:re.findall(r'^\s*[0-9a-f]+:\s+(\S+)([^\n]*)',t,re.M)
    fp=lambda m,o:m.startswith(('f','v','xsave','xrstor')) or m in ('ldmxcsr','stmxcsr','emms') or re.search(r'%(?:[xyz]mm|st|mm|k[0-7])',o)
    require(sorted(m for m,o in instructions(text) if fp(m,o))==['fxrstor64','fxsave64'],'kernel FP outside context helpers')
    helpers={n:routes.cap.boundary.excerpt(text,n) for n in ('task_resume','task_save_fp','user_syscall','timer_vector')}
    require('fxrstor64' in helpers['task_resume'] and 'iretq' in helpers['task_resume'] and 'fxsave64' in helpers['task_save_fp'] and 'sysret' not in text,'context instruction closure')
    require(helpers['user_syscall'].count('pushq')==21 and helpers['user_syscall'].count('popq')==16,'syscall frame closure')
    allowed=set('movups movaps movupd movapd movsd movss movdqa movdqu movd movq pxor xorps xorpd andps andpd andnps andnpd orps orpd por pand pandn pcmpeqd pcmpeqb pshufd pshuflw pshufhw punpckldq punpcklqdq punpckhqdq pslldq psrldq psllq psrlq paddq paddd psubq psubd addsd subsd mulsd divsd sqrtsd comisd ucomisd cvtsi2sd cvttsd2si cvtsd2ss cvtss2sd unpcklpd unpckhpd movhlps movlhps shufps shufpd'.split())
    counts={}
    for item in read(folder/'user-images.json')['images']:
        name=item['name'];routes.cap.scheduler.user_elf(folder/(name+'.elf'))
        require(item['sha256']==digest(folder/(name+'.elf')) and not (folder/(name+'-llvm-nm.stdout.log')).read_text().strip(),'user artifact/symbol closure')
        body=(folder/(name+'-llvm-objdump.stdout.log')).read_text();simd=[]
        for m,o in instructions(body):
            require(not m.startswith(('v','f','xsave','xrstor')) and m not in ('cpuid','rdrand','rdseed','rdtsc','ldmxcsr','stmxcsr') and not re.search(r'%(?:[yz]mm|mm|k[0-7])',o),'unsupported CPU dispatch/instruction')
            if '%xmm' in o:require(m in allowed,'SSE2 instruction review');simd.append(m)
        require(set(simd)==set(item['simd_mnemonics']),'instruction manifest drift')
        counts[name]={m:simd.count(m) for m in sorted(set(simd))}
    return dict(helpers=helpers,simd_instruction_counts=counts)

def audit(rows):
    baseline=routes.audit(rows)
    by={}
    for event,fields in rows:by.setdefault(event,[]).append(fields)
    waits=by.get('m1.wait_settled',[])
    require(len(waits)==9 and all(f.get('records')==f.get('watches')=='0' for f in waits),'SDK wait/watch leak')
    roster={(int(f['wave']),int(f['actor'])):f for f in by['m1.route_admit']}
    finish={(int(f['wave']),int(f['actor'])):f for f in by['m1.route_end']}
    logs=by.get('m1.user_log',[]);tokens=[f['token'] for f in logs]
    require(not any('failed' in t or 'route-panic' in t or 'sdk-service-panic' in t or 'sdk-client-panic' in t for t in tokens),'user assertion failure')
    hellos={};transfers={};discards={};decisions={};states={};commits={}
    for f in logs:
        t=f['token'];p=t.split('-')
        require(f['offset']=='0' and int(f['total'])==len(t),'fragmented token')
        if t.startswith('sdk-hello-'):
            wave,actor,n,status=map(int,p[2:]);key=(wave,actor,n)
            require(key not in hellos and f['principal']=='101' and f['case']=='703','invalid client hello attribution')
            expected=14 if n in (3,4,8) or n==7 and actor==4 else 4 if n in (5,9,10) else 0
            require(status==expected,'version/feature/reserved outcome');hellos[key]=int(f['seq'])
            owner=roster[wave,3];peer=roster[wave,actor]
            prior=[x for x in by.get('m1.ipc_sender',[]) if x['receiver']==owner['process'] and x['generation']==owner['generation'] and int(x['seq'])<int(f['seq'])]
            require(prior and all(prior[-1][k]==peer[v] for k,v in [('sender','process'),('sender_generation','generation'),('principal','principal'),('authority_session','authority_session')]) and prior[-1]['bytes']=='64','hello without authenticated service IPC')
        elif t.startswith('sdk-transfer-') or t.startswith('sdk-discard-'):
            wave,actor=map(int,p[2:4]);target=transfers if p[1]=='transfer' else discards
            require((wave,actor) not in target and f['case']==str(700+actor) and f['principal']=='505','transfer attribution')
            target[wave,actor]=int(f['seq'])
        elif p[0]=='d':
            wave,case,op,status,epoch,revision,value,queued=map(int,p[1:]);require((wave,case) not in decisions and f['principal']=='303' and f['case']=='700','decision attribution')
            decisions[wave,case]=(status,revision,value,queued,epoch,int(f['seq']))
        elif p[0]=='state':
            wave,case,epoch,writer,published,policy=map(int,p[1:]);states[wave,case]=(epoch,writer,published,policy,int(f['seq']))
        elif p[0]=='commit':
            wave,case,request,actor,status,epoch,revision,value=map(int,p[1:]);commits[wave,case]=(actor,status,epoch,revision,value,int(f['seq']))
    require(set(hellos)=={(0,a,n) for a in (4,5) for n in range(1,11)},'incomplete independent interoperability')
    require(set(transfers)==set(discards)=={(0,a) for a in (4,5)},'missing accepted/rejected transfer ownership')
    for key,seq in transfers.items():require(seq<hellos[*key,6],'transfer not observed before reply')
    for key,seq in discards.items():require(seq<hellos[*key,8],'discard not observed before incompatible reply')
    require(len(decisions)==131,'incomplete route decisions')
    require([case for wave,case in decisions if wave==0]==list(routes.MATRIX),'route decision ordering')
    for case,expected in routes.MATRIX.items():require(decisions[0,case][:4]==expected,'route/policy semantic outcome '+str(case))
    require(states[0,250][1]==0 and states[0,251][1:3]==(5,100) and states[0,252][1:3]==(5,101),'fence/grant/publication separation')
    require(states[0,250][4]<states[0,251][4]<states[0,252][4]<commits[0,25][5],'old commit before writer fence')
    require(commits[0,25][1]==13 and commits[0,26][1]==3 and commits[0,27][1]==0,'stale writer/direct endpoint bypass')
    for case in (25,26,27):
        require(commits[0,case][5]<int(finish[0,4]['seq']) and commits[0,case][5]<int(finish[0,5]['seq']),'service lifetimes did not overlap')
    return dict(**baseline,version_exchanges=len(hellos),accepted_moves=len(transfers),rejected_protocol_moves_closed=len(discards))

def runtime_audit(rows):
    by={}
    for e,f in rows:by.setdefault(e,[]).append(f)
    require(len(by.get('m1.kernel_entry',[]))==1 and not any(e in by for e in ('m1.exception','m1.panic','m1.boot_rejected')),'runtime reset/fatal')
    ready=by.get('m1.route_ready',[])
    require(len(ready)==1 and all(ready[0].get(k)==v for k,v in dict(waves='1',tasks='6',settled='true',user_broker='true').items()),'runtime completion')
    starts=by.get('m1.route_admit',[]);ends=by.get('m1.route_end',[])
    require(len(starts)==len(ends)==6 and {f['actor'] for f in starts}=={f['actor'] for f in ends}==set(map(str,range(6))),'runtime lifecycle')
    require(len({f['private_root'] for f in starts})==6 and all(f['wave']=='9' and f['roster_readonly']=='true' for f in starts),'runtime isolation')
    finish={int(f['actor']):f for f in ends}
    for actor,f in finish.items():
        expected=98 if actor==2 else 2 if actor==3 else 0
        require(int(f['code'])==expected and f['vector']=='0' and int(f['dispatches'])>0,'runtime failure containment')
        start=next(x for x in starts if int(x['actor'])==actor)
        require((start['process'],start['generation'])==(f['process'],f['generation']),'runtime result substitution')
        if actor<2:require(int(f['preemptions'])>0,'math not preempted')
    settled=by.get('m1.route_settled',[])
    require(len(settled)==1 and all(settled[0][k]=='0' for k in ('objects','handles','mappings','spaces','allocations','committed','resident','tables','stacks','pins','frames')),'runtime resource leak')
    waits=by.get('m1.wait_settled',[])
    require(len(waits)==1 and waits[0].get('records')==waits[0].get('watches')=='0','runtime wait/watch leak')
    logs=by.get('m1.user_log',[]);tokens=[f['token'] for f in logs]
    require(not any('failed' in t or 'sdk-client-panic' in t for t in tokens),'runtime assertion')
    for actor in (0,1,5):
        matching=[f for f in logs if f['token'].startswith(f'sdk-runtime-{actor}-')]
        require(len(matching)==1 and int(matching[0]['token'].split('-')[3])>=20 and matching[0]['case']==str(800+actor),'portable/reclaim/memory/math workload')
    for token,case in [('sdk-syscalls-all-supported','800'),('sdk-panic-contained','802'),('sdk-oom-contained','803'),('sdk-startup-return','804')]:
        require(sum(f['token']==token and f['case']==case for f in logs)==1,'missing runtime hook '+token)
    for actor in (0,1,5):require(sum(f['token']=='sdk-heap-capacity-reused' and f['case']==str(800+actor) for f in logs)==1,'heap capacity/reuse')
    for actor in (0,1,4,5):require(sum(f['token']=='sdk-heap-unmapped' and f['case']==str(800+actor) for f in logs)==1,'heap backing not explicitly released')
    require(int(finish[5]['seq'])>int(finish[2]['seq']) and int(finish[5]['seq'])>int(finish[3]['seq']),'no healthy continuation after bounded failures')
    return dict(process_runs=6,exact_settlements=1,preemptions=[int(finish[a]['preemptions']) for a in (0,1)],runtime_hooks='nonrendering selected model, segmentation, libm, reclaim/zero/realloc/return/panic/OOM')

def runtime_mutations(rows):
    cases=[]
    for event in ('m1.route_admit','m1.route_end','m1.route_settled','m1.route_ready','m1.wait_settled'):
        cases.append(('missing '+event,[(e,f) for e,f in rows if e!=event]))
    for token in ('sdk-runtime-0-','sdk-runtime-1-','sdk-runtime-5-','sdk-syscalls-','sdk-panic-contained','sdk-oom-contained','sdk-startup-return','sdk-heap-capacity-reused','sdk-heap-unmapped'):
        cases.append(('missing '+token,[(e,f) for e,f in rows if not(e=='m1.user_log' and f['token'].startswith(token))]))
    for event,key,value in [('m1.route_end','preemptions','0'),('m1.route_end','code','99'),('m1.route_settled','frames','1'),('m1.wait_settled','records','1'),('m1.wait_settled','watches','1')]:
        changed=copy.deepcopy(rows)
        for e,f in changed:
            if e==event:f[key]=value
        cases.append((event+'.'+key,changed))
    for name,changed in cases:
        try:runtime_audit(changed)
        except (ValueError,KeyError,IndexError):continue
        raise ValueError('accepted runtime mutation: '+name)
    return [name for name,_ in cases]

def mutations(rows):
    cases=[]
    for event in ('m1.route_ready','m1.route_admit','m1.route_end','m1.route_settled','m1.ipc_sender','m1.wait_settled'):
        cases.append(('missing '+event,[(e,f) for e,f in rows if e!=event]))
    for prefix in ('sdk-hello-','sdk-transfer-','sdk-discard-','state-0-250-','state-0-251-','state-0-252-','commit-0-25-','commit-0-26-','commit-0-27-'):
        cases.append(('missing '+prefix,[(e,f) for e,f in rows if not(e=='m1.user_log' and f['token'].startswith(prefix))]))
    for event,field,value in [('m1.route_ready','tasks','59'),('m1.route_admit','principal','999'),('m1.route_end','code','99'),('m1.route_settled','frames','1'),('m1.ipc_sender','sender_generation','999'),('m1.wait_settled','records','1'),('m1.wait_settled','watches','1')]:
        changed=copy.deepcopy(rows)
        for e,f in changed:
            if e==event:f[field]=value
        cases.append((event+'.'+field,changed))
    for name,changed in cases:
        try:audit(changed)
        except (ValueError,KeyError,IndexError):continue
        raise ValueError('accepted evidence mutation: '+name)
    return [name for name,_ in cases]

def focused(folder,runtime=False):
    require(read(folder/'result.json')['status']=='passed','failed SDK run')
    manifest=read(folder/'build-manifest.json');require(manifest['flavor']==('sdk-runtime-test' if runtime else 'sdk-test') and manifest['kernel_sha256']==digest(folder/'kernel.elf'),'kernel identity')
    routes.memory.elf_audit(folder/'kernel.elf');instruction_review=codegen(folder)
    result,launch,case=routes.memory.lifecycle(folder/('sdk-runtime-run' if runtime else 'sdk-run'))
    require(result['emulator_test_exit']==33 and launch['fixture']=='routes','actual ISO execution')
    builds=read(folder/'sdk-builds.json');require(builds['status']=='passed' and builds['offline'] and len(builds['images'])==3 and builds['runtime']==runtime,'independent builds')
    require(builds['unchanged_client_sha256']==digest(folder/'user_a.elf'),'changed prior client')
    for image in builds['images']:
        name=image['file'];require(image['sha256']==digest(folder/name)==case['package']['payload']['boot/'+name]==digest(folder/('sdk-runtime-package/extracted payload/boot' if runtime else 'sdk-package/extracted payload/boot')/name),'ISO consumer artifact identity')
        clean=Path(image['root']);require(clean.is_relative_to(folder) and (clean/'out').is_dir(),'clean independent output')
        for item in image['inputs']:require(digest(clean/item['path'])==item['sha256'],'changed clean build input')
    require(len({i['root'] for i in builds['images']})==3 and builds['images'][1]['sha256']!=builds['images'][2]['sha256'],'service not rebuilt independently')
    raw=folder/('sdk-runtime-run/serial.log' if runtime else 'sdk-run/serial.log');rows=routes.channel.raw_records(raw.read_bytes(),manifest['build_id'],result['host_session_id'])
    return dict(status='passed',scope='focused only; final regression gate separate',audit=runtime_audit(rows) if runtime else audit(rows),mutations=runtime_mutations(rows) if runtime else mutations(rows),codegen=instruction_review,build_id=manifest['build_id'],kernel_sha256=manifest['kernel_sha256'],iso_sha256=result['iso_sha256'],serial_sha256=digest(raw),builds=builds)

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--sdk',type=Path);p.add_argument('--runtime',action='store_true');p.add_argument('--acceptance',type=Path);p.add_argument('--destination',type=Path);a=p.parse_args()
    if not a.acceptance:
        report=focused(a.sdk.resolve(),a.runtime);save(a.sdk/'sdk-review.json',report);print(json.dumps(report['audit'],indent=2));return
    acceptance=read(a.acceptance);require(acceptance['story']=='P3-12' and acceptance['status']=='passed' and len(acceptance['commands'])==19 and all(c['exit_code']==0 for c in acceptance['commands']),'incomplete final acceptance')
    runs={k:routes.local(v) for k,v in acceptance['runs'].items()}
    # Explicit projection of the 13 unchanged regression commands. This is a
    # fresh P3-11 regression review, never a replacement for the SDK review.
    projection=copy.deepcopy(acceptance);projection['story']='P3-11';projection['derived_scope']='P3-11 regression subset of P3-12';projection['parent_acceptance_sha256']=digest(a.acceptance)
    projection['commands']=[c for c in projection['commands'] if c['name'] not in ('sdk','sdk-runtime','sdk-codec-build','sdk-codecs','sdk-flavors','m2-portable')]
    projection['runs']={k:v for k,v in projection['runs'].items() if k not in ('sdk','sdk_runtime','codecs','flavors','portable')}
    projected=a.acceptance.parent/'regression-acceptance.json';save(projected,projection)
    regression,helpers,_,_,pre,m1,sources=routes.aggregate(projected)
    report=focused(runs['sdk']);report['runtime']=focused(runs['sdk_runtime'],True);require(report['builds']['unchanged_client_sha256']!=report['runtime']['builds']['unchanged_client_sha256'],'runtime artifact not separate');require(read(runs['sdk']/'build-inputs.json')['sources']==sources,'SDK/regression source mismatch')
    require(read(runs['sdk_runtime']/'build-inputs.json')['sources']==sources,'runtime/regression source mismatch')
    codecs=read(runs['codecs']/'conformance.json');require(codecs['status']=='passed' and codecs['cases']>=4871 and codecs['contract_sha256']==digest(ROOT/'contracts/sdk-v1.json') and codecs['reference_sha256']==digest(ROOT/'tools/sdk_contract.py'),'codec identity/coverage')
    require(codecs['semantic']['decoded_cases']==codecs['cases'] and codecs['semantic']['independent_producer_cases']>2000 and codecs['semantic']['paired_field_swap_rejected'] and codecs['semantic']['build']['source_sha256']==digest(ROOT/'tools/sdk_codec_consumer.rs'),'independent semantic/producer oracle')
    flavors=read(runs['flavors']/'flavors.json');require(len(flavors)==11 and all(f['exit_code']==0 for f in flavors),'narrow build flavors')
    portable=read(runs['portable']/'portable.json');require(portable['status']=='passed' and portable['guest_ui_executed'] is False and portable['elf_sha256']==digest(runs['portable']/'selected-ui.elf'),'portable selected-link regression')
    report['portable_regression']=portable
    report.update(story='P3-12',scope='final-source local acceptance',review='local Codex self-review; not external approval',acceptance_sha256=digest(a.acceptance),regression=regression,codecs=codecs,flavors=flavors,freeze_contract_sha256=digest(ROOT/'contracts/sdk-v1.json'),limitations=['one thread/process; x87/SSE2 only, masked SSE exceptions','no native x87 debug metadata or shared-process threads','no guest display/font/shaping/raster backend, dev transport, durable policy or state migration','supervision IDs remain experimental; no general SDK release'])
    report['reviewer_sha256']=digest(__file__)
    report['criterion_review']={
        'independent_builds':'pass: three fresh public-only locked/offline Cargo roots per ISO, declared SDK 0.1.0, separate old/new service artifacts and one unchanged client',
        'codec_semantics':'pass: independent reference classifications, decoded field values and independently supplied producer fields; canonical bytes and paired field-swap rejection',
        'compatibility_and_ownership':'pass: 20 old/new service exchanges, compatible minor/echo/features, explicit major/mandatory-feature/header and payload reserved/correlation rejection; two accepted immutable moves and two rejected application messages with received handles closed',
        'route_fences':'pass: independent public client/service artifacts execute the full nine-incarnation route/policy matrix; delayed old epoch and retained endpoint cannot bypass the writer fence',
        'runtime_hooks':'pass: separate six-process ISO, full heap/slot reuse, zero/copy/reallocation failure atomicity, selected model/segmentation/libm, live FP preemption, return/panic/fallible OOM, healthy continuation and exact settlement',
        'freeze_and_handoff':'pass: only the exact selected contract hash freezes; distinct time/epoch domains, manual authority ownership, current provider checks and M4/M5 obligations remain explicit'
    }
    report['unsafe_review']={
        'kernel':'registered user_fixture observer and existing checked dispatcher/return-state validation; public operations cannot select return IP/selectors/privileged flags; one CPU and original deadlines/quotas',
        'register_boundary':'raw pointer-bearing wrapper remains unsafe; R8/R9 zero, RAX/RDX/RCX/R11 and memory clobbers; explicit little-endian records rather than Rust layout; actual ELF and ISO payload identities checked',
        'allocator':'single owning thread with no callback/signal reentry; private RW 64 KiB object, source handle closed after map; 64-byte cells/32 slots/alignment <=4096; zero before return, matching deallocation, live-allocation shutdown denied; default allocate-copy-free realloc preserves old allocation on failure',
        'fp_and_dependencies':'kernel FP limited to eager fxsave64/fxrstor64 helpers; admitted user SSE2 instruction whitelist and no AVX/XSAVE/CPU dispatch or unresolved imports; selected model/segmentation/libm only execute in runtime guest',
        'authority':'numeric identities and startup/checkpoint bytes confer no authority; receive outputs usable only after success; transferred handles explicitly closed on protocol rejection; existing current-session/policy/revision/deadline and writer-fence rules retained',
        'limits':'trusted synthetic bootstrap; one thread/process, masked SSE exceptions, no shared-process threads/TLS/SMP, no full text/font/raster/display guest, durable policy, state migration, safe old-code retirement or general SDK release'
    }
    save(a.acceptance.parent/'review.json',report)
    if a.destination:
        dest=a.destination.resolve();require(dest.is_relative_to(ROOT/'evidence/phase-3/P3-12') and not dest.exists(),'bad evidence destination');dest.mkdir(parents=True)
        for name,value in [('review.json',report),('acceptance.json',acceptance),('prerequisites.json',pre),('m1-review.json',m1),('sdk-builds.json',report['builds']),('runtime-builds.json',report['runtime']['builds']),('runtime-build-manifest.json',read(runs['sdk_runtime']/'build-manifest.json')),('runtime-user-images.json',read(runs['sdk_runtime']/'user-images.json')),('runtime-launch-context.json',dict(result=read(runs['sdk_runtime']/'sdk-runtime-run/run-result.json'),launch=read(runs['sdk_runtime']/'sdk-runtime-run/launch-manifest.json'),package=read(runs['sdk_runtime']/'sdk-runtime-run/case-result.json')['package'])),('codecs.json',codecs),('build-inputs.json',read(runs['sdk']/'build-inputs.json')),('build-manifest.json',read(runs['sdk']/'build-manifest.json')),('user-images.json',read(runs['sdk']/'user-images.json')),('launch-context.json',dict(result=read(runs['sdk']/'sdk-run/run-result.json'),launch=read(runs['sdk']/'sdk-run/launch-manifest.json'),package=read(runs['sdk']/'sdk-run/case-result.json')['package']))]:save(dest/name,value)
        extra=[ROOT/'tools/Review-P3-12.py',ROOT/'tools/Test-P3-12.ps1',ROOT/'tools/Check-P3-12Flavors.ps1',ROOT/'tools/Check-P3-12Portable.ps1',ROOT/'tools/sdk_contract.py',ROOT/'tools/sdk_codec_consumer.rs',ROOT/'tools/Build-P3-12CodecConsumer.ps1']
        save(dest/'source-provenance.json',dict(source_commit=acceptance['source_commit'],dirty_input_hashes={**sources,**{str(p.relative_to(ROOT)).replace('\\','/'):digest(p) for p in extra}}))
        shutil.copyfile(runs['m1']/'kernel-host/check.stdout.log',dest/'host-tests.txt')
        raw=(runs['sdk']/'sdk-run/serial.log').read_bytes();part=b'';index=0
        for line in raw.splitlines(keepends=True):
            if len(part)+len(line)>200000:(dest/f'sdk-serial-{index}.txt').write_bytes(part);part=b'';index+=1
            part+=line
        if part:(dest/f'sdk-serial-{index}.txt').write_bytes(part)
        raw=(runs['sdk_runtime']/'sdk-runtime-run/serial.log').read_bytes();part=b'';index=0
        for line in raw.splitlines(keepends=True):
            if len(part)+len(line)>200000:(dest/f'runtime-serial-{index}.txt').write_bytes(part);part=b'';index+=1
            part+=line
        if part:(dest/f'runtime-serial-{index}.txt').write_bytes(part)
        require(sum(p.stat().st_size for p in dest.rglob('*') if p.is_file())<4*1024*1024,'evidence budget')
    print(json.dumps(dict(status='passed',audit=report['audit'],codecs=codecs),indent=2))
if __name__=='__main__':main()
