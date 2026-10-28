"""Independent P3-05 queue, CPU-context, artifact and regression review."""
import argparse
import copy
import importlib.util
import json
from pathlib import Path
import re
import shutil
import struct

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('boundary', ROOT/'tools/Review-P3-04.py')
boundary = importlib.util.module_from_spec(spec)
spec.loader.exec_module(boundary)
memory, loader = boundary.memory, boundary.loader
require, read, save, digest, local = memory.require, memory.read, memory.save, memory.digest, memory.local
MODES = list(range(1, 11))
EXPECTED = {0:768, 1:0, 2:272, 3:1024, 4:262, 5:269, 6:0, 7:0, 8:768, 9:0, 10:512}


def audit(rows, trace, detailed=False):
    require(sum(e=='m1.kernel_entry' for e,f in rows)==1, 'reset')
    by = {}
    for event, fields in rows: by.setdefault(event, []).append(fields)
    require(not any(e in by for e in ('m1.exception','m1.panic','m1.boot_rejected')), 'kernel failure')
    policy, ready = by.get('m1.scheduler_policy', []), by.get('m1.scheduler_ready', [])
    require(len(policy)==len(ready)==1, 'missing policy/completion')
    policy, ready = policy[0], ready[0]
    require(all(policy.get(k)==v for k,v in dict(slots='32',owners='16',threads_per_owner='1',priority='equal',quantum_ns='5000000',fairness_quanta='64',fp='x87-sse-sse2',xsave='false',avx='false',unsupported_denied='true',trace=str(trace).lower()).items()),'policy mismatch')
    require(ready['waves']=='10' and ready['tasks']=='30' and ready['settled']=='true' and ready['idle']=='true' and ready['trace']==str(trace).lower(), 'incomplete run')
    for key in ('max_masked_upper_ns','max_body_ns','max_wait_ns'):
        require(0<int(ready[key])<320_000_000, 'fairness/masking bound '+key)
    require(int(ready['max_body_ns'])<=int(ready['max_masked_upper_ns']),'body exceeds conservative bound')
    require(int(ready['nested_irqs'])>=int(ready['pin_irqs'])>0 and ready['observer_samples']=='100' and int(ready['observer_total_ns'])>0,'missing IRQ/observer test')
    cpu=by.get('m1.scheduler_cpu',[])
    require(len(cpu)==1,'missing CPU/memory policy')
    cpu=cpu[0]
    require(int(cpu['cpuid_edx'],0)&((7<<24)|1)==((7<<24)|1) and int(cpu['cr0'],0)&0x2e==0x2a and int(cpu['cr4'],0)&((3<<9)|(1<<18)|(1<<16)|(1<<22)|(1<<23))==3<<9,'CPU control policy')
    require(cpu['missing_feature_denials']=='4' and cpu['required_feature_denials']=='1' and 0<int(cpu['fixed_bytes'])<=16777216,'CPU denials/fixed budget')
    starts, ends, settlements = (by.get(e,[]) for e in ('m1.task_start','m1.task_end','m1.task_settlement'))
    require(len(starts)==len(ends)==30 and len(settlements)==10,'missing lifecycle')
    previous = [0,0,0]
    total = 0
    for wave, mode in enumerate(MODES):
        ss=starts[wave*3:wave*3+3]
        ee=[e for e in ends if int(e['wave'])==wave]
        require(len(ee)==3 and {int(e['thread']) for e in ee}=={0,1,2},'wave owners')
        require(len({s['root'] for s in ss})==len({s['stack'] for s in ss})==3,'shared stack/root')
        for index,s in enumerate(ss):
            require(int(s['thread'])==int(s['process'])==index and int(s['principal'])==[101,202,303][index] and int(s['mode'])==[0,0,mode][index] and s['fresh']=='true','trusted owner binding')
            require(int(s['generation'])>previous[index], 'process reuse generation')
            previous[index]=int(s['generation'])
            require(int(s['root'],0)%4096==0 and int(s['stack'],0)>=0xffff800000000000,'owned root/stack')
            e=next(e for e in ee if int(e['thread'])==index)
            require(e['generation']==s['generation'] and int(e['result'])==EXPECTED[int(s['mode'])], 'context/fault result')
            require(int(s['seq'])<int(e['seq'])<int(settlements[wave]['seq']),'lifecycle order')
            if index<2:
                require(e['dispatches']=='16' and int(e['preemptions'])>=15 and int(e['progress'])>0, 'non-yielding hog failed')
            elif mode in (2,4,5): require(int(e['vector'])==EXPECTED[mode]-256,'wrong FP/privilege fault')
            if detailed:
                require(int(e['elapsed_ns'])>0 and int(e['runnable_ns'])>=0,'task timing')
                masks=int(e['mxcsr'],0)&0x1f80
                require(masks!=0x1f80 if int(s['mode'])==3 else masks==0x1f80,'SSE control outcome')
            total+=int(e['dispatches'])
        require(all(settlements[wave].get(k)==v for k,v in dict(wave=str(wave),settled='true',frames='0',spaces='0',pins='0',allocations='0',committed='0',resident='0',mappings='0',tables='0',stacks='0').items()), 'resource leak')
        control=next(e for e in ee if e['thread']=='2')
        if mode in (2,3,4,5,8,10):
            require(any(int(e['seq'])>int(control['seq']) for e in ee if e['thread']!='2'), 'no surviving progress')
        switches=[f for f in by.get('m1.task_switch',[]) if int(f['wave'])==wave]
        if trace:
            queue=[0,1,2]; counters=[0,0,0]; progress=[0,0,0]
            for f in switches:
                thread=int(f['thread']); require(queue and queue.pop(0)==thread,'round-robin order')
                counters[thread]+=1
                require(int(f['dispatches'])==counters[thread] and int(f['progress'])>=progress[thread] and int(f['elapsed_ns'])>0,'dispatch counter/progress')
                progress[thread]=int(f['progress'])
                if f['done']=='false': queue.append(thread)
                else: require(f['done']=='true' and any(e['thread']==str(thread) and e['dispatches']==f['dispatches'] and e['result']==f['result'] for e in ee),'dispatch termination')
            if detailed:
                for i in range(3):
                    e=next(e for e in ee if e['thread']==str(i))
                    require(sum(int(f['elapsed_ns']) for f in switches if f['thread']==str(i))==int(e['elapsed_ns']) and sum(int(f['wait_ns']) for f in switches if f['thread']==str(i))==int(e['runnable_ns']),'dispatch/runnable timing reconciliation')
            require(not queue and counters==[int(next(e for e in ee if e['thread']==str(i))['dispatches']) for i in range(3)],'missing switches')
        else: require(not switches,'trace-off contains switch observer')
    require(int(ready['switches'])==total,'switch reconciliation')
    if detailed and trace:
        require(max(int(f['wait_ns']) for f in by['m1.task_switch'])==int(ready['max_wait_ns']),'runnable maximum reconciliation')
    faults=by.get('m1.user_fault',[])
    require(len(faults)==3 and {int(f['vector']) for f in faults}=={6,13,16} and all(f['cs']=='0x33' and f['ss']=='0x3b' for f in faults),'actual CPL3 x87/privilege faults')
    return dict(tasks=30,waves=10,switches=total,non_yielding_hogs=20,**{k:int(ready[k]) for k in ('max_masked_upper_ns','max_body_ns','max_wait_ns','nested_irqs','pin_irqs','observer_total_ns')})


def mutations(rows, trace, detailed=False):
    results=[]
    for event in ('m1.scheduler_policy','m1.scheduler_ready','m1.task_start','m1.task_end','m1.task_settlement','m1.user_fault')+ (('m1.task_switch',) if trace else ()):
        try: audit([(e,f) for e,f in rows if e!=event],trace,detailed)
        except (ValueError,KeyError,IndexError,StopIteration): results.append('missing '+event)
        else: raise ValueError('accepted missing '+event)
    for event,key,value in [('m1.task_start','principal','999'),('m1.task_end','result','99'),('m1.task_end','preemptions','0'),('m1.task_end','progress','0'),('m1.task_settlement','frames','1'),('m1.scheduler_ready','max_wait_ns','320000001'),('m1.scheduler_ready','pin_irqs','0'),('m1.scheduler_policy','priority','boosted')]:
        changed=copy.deepcopy(rows); next(f for e,f in changed if e==event)[key]=value
        try: audit(changed,trace,detailed)
        except (ValueError,KeyError): results.append(event+'/'+key)
        else: raise ValueError('accepted mutation '+event+'/'+key)
    return results


def user_elf(path):
    raw=path.read_bytes()
    require(raw[:7]==b'\x7fELF\2\1\1' and len(raw)<=1048576,'user ELF identity/size')
    kind,machine,version,entry,ph,sh,flags,eh,ps,pn,ss,sn,si=struct.unpack_from('<HHIQQQIHHHHHH',raw,16)
    require((kind,machine,version,flags,eh,ps,ss)==(2,62,1,0,64,56,64) and entry==0x400000,'user ELF architecture/entry')
    require(0<pn<=16 and 0<sn<=128 and si<sn and ph+pn*ps<=len(raw) and sh+sn*ss<=len(raw),'user ELF table bounds')
    loads=[]; entry_rx=False
    for n in range(pn):
        k,f,off,va,pa,fs,ms,align=struct.unpack_from('<IIQQQQQQ',raw,ph+n*ps)
        require(k==1 and f in (4,5,6) and fs<=ms and off+fs<=len(raw) and ms>0 and align==4096 and va==pa and va%align==off%align,'user ELF static W^X LOAD')
        base,end=va//4096*4096,(va+ms+4095)//4096*4096
        require(0x10000<=base<end<=0x7ffd0000 and all(end<=a or base>=b for a,b,_ in loads),'user ELF overlap/range')
        loads.append((base,end,f))
        entry_rx |= f==5 and va<=entry<va+fs
    require([f for _,_,f in loads]==[5,4,6] and entry_rx,'user ELF split/entry backing')
    for n in range(sn):
        _,k,f,_,off,size,_,_,_,entsize=struct.unpack_from('<IIQQQQIIQQ',raw,sh+n*ss)
        require(k not in (4,6,9,11,14,15,16,19) and f&0x400==0 and (k==8 or off+size<=len(raw)),'user ELF runtime obligation')
        if k==2:
            require(entsize==24 and size%24==0,'user ELF symbols')
            for p in range(off,off+size,24):
                name,info,_,index,_,_=struct.unpack_from('<IBBHQQ',raw,p)
                require(not(name and info>>4 and index==0),'undefined user symbol')
    return loads


def codegen(folder):
    kernel=(folder/'llvm-objdump.stdout.log').read_text()
    helpers={n:boundary.excerpt(kernel,n) for n in ('task_save_fp','task_resume','user_leave','user_syscall','timer_vector','exception_common')}
    require('fxsave64' in helpers['task_save_fp'] and 'fxrstor64' in helpers['task_resume'] and 'clts' in helpers['task_save_fp'], 'missing eager FP state')
    require('rep' in helpers['task_resume'] and 'iretq' in helpers['task_resume'] and 'sysret' not in kernel,'resume frame')
    require(helpers['user_syscall'].count('pushq')==21 and helpers['user_syscall'].count('popq')==16,'syscall scratch/frame balance')
    require(helpers['timer_vector'].count('pushq')==16 and helpers['timer_vector'].count('popq')==15 and '%cr0' in helpers['timer_vector'],'IRQ TS/GPR balance')
    instructions=lambda t: re.findall(r'^\s*[0-9a-f]+:\s+(\S+)([^\n]*)',t,re.M)
    def fp(m,o): return m.startswith(('f','v','xsave','xrstor')) or m in ('ldmxcsr','stmxcsr','emms') or re.search(r'%(?:[xyz]mm|st|mm|k[0-7])',o)
    actual=[m for m,o in instructions(kernel) if fp(m,o)]
    require(sorted(actual)==['fxrstor64','fxsave64'],'kernel FP outside narrow helpers')
    images=read(folder/'user-images.json')
    require(images['scheduler'] and images['required_cpu_features']==7,'missing image CPU manifest')
    inventories={}
    allowed={'fxsave64','fxrstor64','fninit','fldcw','fldz','fdivrp','fwait','wait','ldmxcsr','pxor','divss','vxorps'}
    for name,item in zip(('user_a','user_b','service'), images['images']):
        user_elf(folder/(name+'.elf'))
        require(item['sha256']==digest(folder/(name+'.elf')) and not (folder/(name+'-llvm-nm.stdout.log')).read_text().strip(),'image symbols/hash')
        text=(folder/(name+'-llvm-objdump.stdout.log')).read_text()
        found=[m for m,o in instructions(text) if fp(m,o)]
        require(set(found)<=allowed and {'fxsave64','fxrstor64','fdivrp','divss','vxorps'}<=set(found),'unreviewed user instructions')
        spinner=boundary.excerpt(text,'spinner')
        require(spinner.count('syscall')==1 and 'pause' not in spinner and all('%'+r in spinner for r in ('rax','rcx','rdx','rbx','rbp','rsi','rdi','r8','r9','r10','r11','r12','r13','r14','r15')),'hog sentinel/exit path')
        inventories[name]={m:found.count(m) for m in sorted(set(found))}
        helpers[name+'-spinner']=spinner
    return helpers,inventories


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--acceptance',type=Path)
    parser.add_argument('--scheduler',type=Path,help='focused raw audit only')
    parser.add_argument('--destination',type=Path)
    args=parser.parse_args()
    a=read(args.acceptance) if args.acceptance else None
    folder=local(a['runs']['scheduler']) if a else args.scheduler.resolve()
    require(read(folder/'result.json')['status']=='passed','scheduler run failed')
    mf=read(folder/'build-manifest.json')
    require(mf['kernel_sha256']==digest(folder/'kernel.elf'),'kernel hash')
    memory.elf_audit(folder/'kernel.elf')
    audits=[]; rejected=[]; launches=[]
    for suffix,trace in [('on',True),('off',False)]:
        run=folder/('scheduler-'+suffix+'-run')
        result,launch,case=memory.lifecycle(run)
        require(result['emulator_test_exit']==33 and launch['fixture']==result['fixture']=='scheduler-'+suffix and result['build_id']==launch['build_id']==mf['build_id'],'guest exit/identity')
        argv=launch['arguments']
        require(argv[argv.index('-cpu')+1]=='qemu64-v1,+apic' and argv[argv.index('-accel')+1]=='tcg' and argv[argv.index('-machine')+1]=='pc-q35-11.1,hpet=on','scheduler VM profile')
        rows=memory.records((run/'serial.log').read_bytes(),mf['build_id'],result['host_session_id'])
        audits.append(audit(rows,trace,bool(a))); rejected+=mutations(rows,trace,bool(a)); launches.append((run,'scheduler-'+suffix))
        for item in read(folder/'user-images.json')['images']:
            require(case['package']['payload']['boot/'+item['name']+'.elf']==item['sha256'],'ISO image mismatch')
    require(read(launches[0][0]/'run-result.json')['host_session_id']!=read(launches[1][0]/'run-result.json')['host_session_id'],'reused boot session')
    run=folder/'scheduler-cpu-run'
    result,launch,case=memory.lifecycle(run)
    require(result['emulator_test_exit']==33 and '-sse2' in launch['arguments'][launch['arguments'].index('-cpu')+1] and launch['vm']['cpu'].endswith(',-sse2'),'CPU negative profile')
    rows=memory.records((run/'serial.log').read_bytes(),mf['build_id'],result['host_session_id'])
    denials=[f for e,f in rows if e=='m1.scheduler_cpu_rejected']
    require(len(denials)==1 and denials[0]['user_entries']=='0' and not int(denials[0]['cpuid_edx'],0)&(1<<26) and not any(e=='m1.task_start' for e,f in rows),'missing real CPU denial')
    launches.append((run,'scheduler-cpu'))
    helpers,instructions=codegen(folder)
    if a:
        require(helpers['exception_common'].count('pushq')==16 and helpers['exception_common'].count('popq')==15 and '%cr0' in helpers['exception_common'],'exception TS guard/frame balance')
    report=dict(schema_version=1,story='P3-05',status='passed',review='local Codex self-review; independent parsers, not external approval',reviewer_sha256=digest(__file__),build_id=mf['build_id'],kernel_sha256=mf['kernel_sha256'],audits=audits,reviewer_mutations=rejected,instructions=instructions,preemption=True,native_sse_trap_verified=False,sse_unmask_policy="terminate at next kernel boundary",public_memory_rights=False,desktop_targets_accepted=False)
    report['unsafe_review']={
        'scope':'local Codex source/assembly review and independently parsed execution/artifact evidence',
        'entry':'160-byte integer/IRET continuation; per-owner CR3, guarded syscall stack and TSS RSP0; trusted selectors and reset bases; no SYSRET or suspended Rust stack',
        'irq':'all GPRs and original CR0 retained; TS set before Rust; LAPIC acknowledged before fixed-buffer capture; no allocation, queue operations, logging or memory borrows in IRQ',
        'fp':'aligned private 512-byte architectural state; fresh zero payload/default controls; only two kernel FX helpers; no kernel FP arithmetic, OSXSAVE, FSGSBASE or PKRU state',
        'copy':'live-pin timer delivery only marks pending; no switch until copy unpins and syscall completes; precise copy recovery remains bounded',
        'teardown':'active CR3 captures page-table-bank publication; return to retained supervisor stack/root before any owned memory or stack is reclaimed',
        'limits':'one owner thread per process, equal priority, no blocking kernel continuations/SMP/TLS; unmasked SSE unsupported with bounded policy kill; hardware #XM and nonzero x87 debug metadata not execution-validated on TCG'}
    report['iso_sha256']=[read(run/'run-result.json')['iso_sha256'] for run,_ in launches]
    report['serial_sha256']=[digest(run/'serial.log') for run,_ in launches]
    report['criterion_review']={
        'non_yielding_progress':'pass: two ring-3 hogs per wave, 16 dispatches each, plus control progress; raw FIFO rotation and fixed 64-quantum bounds',
        'context_and_reuse':'pass for declared context: integer/arithmetic/DF/stack/private-VA sentinels and distinct x87/XMM/MXCSR patterns; fresh/reused slots initialize payloads and controls',
        'bounded_policy':'pass: equal-priority policy with no public elevation API, fixed queue/owner admission, trusted compute/copy-return kill, empty-queue halt, kernel IRQ deferral',
        'features_and_faults':'pass for supported subset: actual instruction audit, real CPU rejection without SSE2, native x87 #MF; unmasked SSE controls explicitly unsupported and terminated at a boundary, not credited as #XM',
        'containment_and_regressions':'pass requires final-source M1 plus owned-memory, loader and P3-04 raw audits; offending control tasks leave other owners progressing with exact settlements'}
    report['scope']='complete final-source story acceptance' if a else 'focused scheduler/CPU audit only; aggregate story regressions not reviewed by this invocation'
    if a:
        require(a['status']=='passed' and a['story']=='P3-05' and len(a['commands'])==7 and all(c['exit_code']==0 for c in a['commands']),'incomplete acceptance')
        pre,baseline,owned,staged,old=(local(a['runs'][k]) for k in ('prerequisites','m1','memory','loader','boundary'))
        prerequisites=read(pre/'prerequisites.json')
        require(prerequisites['status']=='passed' and len(prerequisites['commands'])==10 and all(c['exit_code']==0 for c in prerequisites['commands']),'prerequisites')
        sources=read(baseline/'suite-source-inputs.json')
        pre_sources={v['path']:v['sha256'] for v in read(pre/'source-inputs.json')}
        require(all(pre_sources.get(p)==h==digest(ROOT/p) for p,h in sources.items()),'stale final sources')
        require(all(read(p/'build-inputs.json')['sources']==sources for p in (owned,staged,old,folder)),'regression source mismatch')
        m1=read(baseline/'review.json')
        require(m1['status']=='passed' and len(m1['launches'])==42 and len(read(baseline/'result.json')['stages']['rows'])==33,'M1 gate')
        host=(baseline/'kernel-host/check.stdout.log').read_text()
        require('34 passed; 0 failed' in host and 'cpu_gate_and_pristine_state ... ok' in host,'host tests')
        for item in m1['launches']:
            run=local(item['directory']); memory.lifecycle(run,item['outcome']!='broken-before-entry')
            launches.append((run,'m1-'+str(len(launches))))
        result,_,_=memory.lifecycle(owned/'owned-memory-run')
        report['memory_audit']=memory.table_audit(memory.records((owned/'owned-memory-run/serial.log').read_bytes(),read(owned/'build-manifest.json')['build_id'],result['host_session_id']))
        result,_,_=memory.lifecycle(staged/'static-loader-run')
        report['loader_audit']=loader.audit(memory.records((staged/'static-loader-run/serial.log').read_bytes(),read(staged/'build-manifest.json')['build_id'],result['host_session_id']),[loader.expected_pages(staged/(n+'.elf')) for n in ('user_a','user_b','service')])
        for group,suffix in enumerate('abcd'):
            run=old/('user-boundary-'+suffix+'-run'); result,_,_=memory.lifecycle(run)
            rows=memory.records((run/'serial.log').read_bytes(),read(old/'build-manifest.json')['build_id'],result['host_session_id'])
            boundary.audit(rows,group); report['reviewer_mutations']+=boundary.mutation_checks(rows,group)
            launches.append((run,'boundary-'+suffix))
        launches += [(owned/'owned-memory-run','memory'),(staged/'static-loader-run','loader')]
        report.update(source_commit=a['source_commit'],prerequisite_checks=10,m1_rows=33,m1_launches=42,kernel_host_tests=34,acceptance_sha256=digest(args.acceptance))
        save(args.acceptance.parent/'review.json',report)
        if args.destination:
            dest=args.destination.resolve()
            require(dest.is_relative_to(ROOT/'evidence/phase-3/P3-05') and not dest.exists(),'existing/invalid destination')
            dest.mkdir(parents=True)
            for name,value in [('acceptance.json',a),('review.json',report),('prerequisites.json',prerequisites),('m1-review.json',m1),('m1-result.json',read(baseline/'result.json')),('scheduler-build-inputs.json',read(folder/'build-inputs.json')),('scheduler-build-manifest.json',mf),('user-images.json',read(folder/'user-images.json')),('source-provenance.json',dict(source_commit=a['source_commit'],dirty_input_hashes=sources))]:save(dest/name,value)
            shutil.copyfile(baseline/'kernel-host/check.stdout.log',dest/'host-tests.txt')
            (dest/'codegen').mkdir()
            for name,text in helpers.items():(dest/'codegen'/(name+'.txt')).write_text(text)
            for name in ('user_a','user_b','service'):
                save(dest/'codegen'/(name+'-command.json'),read(folder/(name+'-build.command.json')))
            shutil.copyfile(folder/'runtime-ids.rs',dest/'codegen/runtime-ids.txt')
            provenance=dict(sources)
            for name in ('tools/Test-P3-05.ps1','tools/Review-P3-05.py','tools/Review-P3-04.py','tools/Review-P3-03.py','tools/Review-P3-02.py','docs/05-delivery/phase-3/scheduler-runbook.md','docs/04-decisions/0021-preemptive-owner-threads-and-eager-fp.md','docs/06-contracts/runtime-v0.md'):
                provenance[name]=digest(ROOT/name)
            save(dest/'source-provenance.json',dict(source_commit=a['source_commit'],dirty_input_hashes=provenance))
            (dest/'launches').mkdir()
            for run,label in launches:
                shutil.copyfile(run/'serial.log',dest/'launches'/(label+'-serial.txt'))
                save(dest/'launches'/(label+'-context.json'),dict(result=read(run/'run-result.json'),launch=read(run/'launch-manifest.json'),package=read(run/'case-result.json')['package'],serial_sha256=digest(run/'serial.log')))
            files=[p for p in dest.rglob('*') if p.is_file()]
            require(len(files)<190 and sum(p.stat().st_size for p in files)<4*1024*1024 and all(p.stat().st_size<=256*1024 for p in files),'evidence budget')
    print(json.dumps(report,indent=2))


if __name__=='__main__':main()
