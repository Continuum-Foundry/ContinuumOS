"""Independent P3-04 raw-trace, ELF/codegen, lifecycle and regression review."""
import argparse
import copy
import importlib.util
import json
from pathlib import Path
import re
import shutil

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('loader_review', ROOT/'tools/Review-P3-03.py')
loader = importlib.util.module_from_spec(spec)
spec.loader.exec_module(loader)
memory = loader.memory
require, read, save, digest, local = memory.require, memory.read, memory.save, memory.digest, memory.local
CASES = [0] + [v for case in range(1,17) for v in (case,0)]
OUTCOMES = {0:('exit',0),1:('fault',270),2:('fault',270),3:('fault',262),
            4:('fault',269),5:('fault',269),6:('exit',1),7:('exit',2),
            8:('return-state',512),9:('return-state',512),10:('fault',270),
            11:('fault',263),12:('return-state',512),13:('return-state',512),
            14:('return-state',512),15:('fault',269),16:('exit',2)}


def audit(rows, group):
    cases=[0]+[v for case in range(group*4+1,group*4+5) for v in (case,0)]
    require(sum(e == 'm1.kernel_entry' for e,f in rows) == 1
            and not any(e in ('m1.exception','m1.panic','m1.boot_rejected') for e,f in rows), 'reset or kernel-fatal')
    starts, ends, settled, logs, calls, faults, bad, irqs, ready = ([] for _ in range(9))
    target = {'m1.user_start':starts,'m1.user_end':ends,'m1.user_settlement':settled,
              'm1.user_log':logs,'m1.user_call':calls,'m1.user_fault':faults,
              'm1.user_bad_return':bad,'m1.user_irq':irqs,'m1.user_ready':ready}
    for e,f in rows:
        if e in target: target[e].append(f)
    require(len(starts) == len(ends) == len(settled) == 9 and len(ready) == 1, 'missing lifecycle records')
    previous_generation, previous_ticks = 0, 0
    for index,(case,s,e,t) in enumerate(zip(cases,starts,ends,settled)):
        require(int(s['index']) == int(t['index']) == index and all(int(v['case']) == case for v in (s,e,t)), 'case order')
        require(int(s['seq']) < int(e['seq']) < int(t['seq']) and
                (index == 8 or int(t['seq']) < int(starts[index+1]['seq'])), 'overlapping/reordered lifecycle')
        require(s['slot'] == e['slot'] == '0' and s['generation'] == e['generation']
                and int(s['generation']) > previous_generation, 'current process generation')
        principal, session = ('101','1001') if case == 0 else ('202','2002')
        require(all(v['principal'] == principal and v['authority_session'] == session for v in (s,e)), 'forged caller provenance')
        previous_generation = int(s['generation'])
        require(s['cpl'] == '3' and int(s['kernel_stack'],0) >= 0xffff800000000000
                and int(s['root'],0) % 4096 == 0, 'entry/owned stack/root')
        reason,result = OUTCOMES[case]
        require(e['reason'] == reason and int(e['result']) == int(t['result']) == result, 'wrong process outcome')
        require(t['settled'] == 'true' and t['pins'] == t['frames'] == '0', 'resource/pin leak')
        before,after = int(t['ticks_before']),int(t['ticks_after'])
        require(previous_ticks <= int(s['ticks']) == before <= after, 'boot uptime regression')
        previous_ticks = after
        within = lambda f: int(s['seq']) < int(f['seq']) < int(e['seq'])
        if case == 0:
            require(int(e['ip'],0)==0x7ffd100f, 'ordinary return did not use generated exit trampoline')
            require(any(f['token'] == 'healthy' and within(f) for f in logs), 'missing healthy replacement')
            require(any(within(f) and int(f['operation']) == 0 and f['status'] == '0' and f['value'] == '32' for f in calls), 'missing real discovery syscall')
        if reason == 'fault':
            fs = [f for f in faults if within(f)]
            require(len(fs) == 1 and int(fs[0]['vector']) == int(e['vector']) == result-256
                    and fs[0]['cs'] == '0x33' and fs[0]['ss'] == '0x3b', 'fault was not user CPL3')
            require(0x400000 <= int(fs[0]['ip'],0) < 0x7ffd1000, 'fault IP outside fixture')
            if case in (1,2,10):
                require(int(fs[0]['error'],0) & 4, 'page fault lacks user error bit')
                expected = {1:0x12345000,2:0x7ffdffff,10:0x7ffd0000}[case]
                require(int(fs[0]['address'],0) == expected, 'wrong user fault address')
                require(int(fs[0]['error'],0) & 3 == (3 if case == 10 else 0), 'wrong page fault access')
        if reason == 'return-state':
            fs = [f for f in bad if within(f)]
            require(len(fs) == 1 and fs[0]['cs'] == '0x33' and fs[0]['ss'] == '0x3b', 'missing rejected frame')
            if case == 8: require(int(fs[0]['sp'],0) == 0x800000000000, 'noncanonical SP fixture')
            if case == 9: require(int(fs[0]['sp'],0) == 0x12345000, 'unmapped SP fixture')
            if case == 12: require(int(fs[0]['ip'],0) == 0x800000000000, 'injected noncanonical IP')
            if case == 13: require(int(fs[0]['flags'],0) & 0x3000 == 0x3000, 'injected IOPL')
            if case == 14: require(int(fs[0]['flags'],0) & 0x40000, 'user AC')
    # Independently inspect observed error/value pairs and concrete pointer tests.
    def call(op, status, a=None, b=None, value=None):
        return any(int(f['operation']) == op and int(f['status']) == status
                   and (a is None or int(f['a'],0) == a) and (b is None or int(f['b']) == b)
                   and (value is None or int(f['value']) == value) for f in calls)
    for address,length in [(0,1),(0x7fffffff,2),(2**64-4,8),(0xffff800000000000,1),(0x60000000,1),(0x12345000,1)]:
        require(call(1,4,address,length,0), 'missing denied input span')
    for address in [0,0x7ffd0000,0x400000,0xffff800000000000,0x60000000,2**64-16]:
        require(call(0,4,address,32,0), 'missing denied output span')
    require(call(0,5,b=31,value=32) and call(1,0,2**64-1,0,0) and call(1,4,b=257), 'length/empty/capacity semantics')
    require(call(1,0,b=256,value=256), 'full bounded diagnostic missing')
    full=[f for f in logs if f.get('total')=='256']
    require([int(f['offset']) for f in full]==[0,96,192] and ''.join(f['token'] for f in full)=='a'*256, 'diagnostic fragments/truncation')
    require(call(2,4,2**64-1,16,0) and call(2,4,1,3,0), 'allocation overflow/alignment')
    require(call(2,11,33*1024*1024,16,0) and (group!=3 or call(2,10)), 'quota and injected physical allocator OOM')
    require(call(2**64-1,12,value=0) and call(4,12,value=0) and call(4,14,value=0), 'operation/features/version errors')
    require(call(3,4) and any(f['token'] == 'pointers-passed' for f in logs), 'free/pointer fixture')
    for f in calls:
        require(f['case'] in ('0','6','7','16'), 'unexpected completed offender syscall')
        require(int(f['flags'],0) & 0x202 == 0x202 and int(f['flags'],0) & ~0xed7 == 0, 'unsafe user return flags')
        require(int(f['entry_flags'],0) & (0x200|0x400|0x100|0x40000|0x4000) == 0, 'entry flag mask')
        if int(f['status']) not in (0,5): require(f['value'] == '0', 'error leaked return value')
    require(irqs and all(f['nested'] == 'true' and int(f['after']) > int(f['before']) for f in irqs), 'nested timer IRQ')
    require(any(int(f['flags'],0) & 0x400 and int(f['operation']) == 4 and f['status'] == '0' for f in calls), 'DF sentinel')
    inventories=[f for e,f in rows if e=='m1.user_inventory']
    require(len(inventories)==9 and all(int(f['index'])==i and all(f[k]=='0' for k in ('spaces','allocations','committed','resident','mappings','tables','stacks','pins')) for i,f in enumerate(inventories)), 'inventory settlement')
    runtime=[f for e,f in rows if e=='m1.user_runtime']
    require(len(runtime)==1 and 0<int(runtime[0]['extra_bytes'])<65536 and int(runtime[0]['fixed_total'])<=16777216 and all(runtime[0][k]=='true' for k in ('fp_ts','fp_osfxsr')) and runtime[0]['fp_em']=='false', 'runtime capacity/FP trap policy')
    injections=[f for e,f in rows if e=='m1.user_oom_injection']
    require(len(injections)==(1 if group==3 else 0) and all(f['additional_pages']=='256' and f['source']=='trusted-supervisor' for f in injections), 'bounded OOM injection')
    r=ready[0]
    require(r['runs'] == '9' and int(r['group'])==group and all(r[k] == 'true' for k in ('settled','user_execution','copy_recovery'))
            and int(r['user_irqs']) > 0 and int(r['calls']) == len(calls)+sum(OUTCOMES[case][0]!='fault' for case in cases), 'missing final execution/copy/IRQ proof')
    return dict(group=group,runs=9,healthy_runs=5,healthy_replacements=4,offenders=4,calls=len(calls),nested_irqs=len(irqs),user_irqs=int(r['user_irqs']),settled_frames=0)


def mutation_checks(rows,group):
    results=[]
    for event,key,value in [('m1.user_end','reason','kernel-fatal'),('m1.user_end','principal','999'),
                            ('m1.user_settlement','frames','1'),('m1.user_ready','user_irqs','0'),
                            ('m1.user_ready','copy_recovery','false'),('m1.user_call','entry_flags','0x602'),
                            ('m1.user_fault','cs','0x8'),('m1.user_start','generation','0')]:
        if not any(e==event for e,f in rows): continue
        changed=copy.deepcopy(rows)
        next(f for e,f in changed if e == event)[key]=value
        try: audit(changed,group)
        except (ValueError,KeyError): results.append(dict(case=event+'/'+key,status='rejected'))
        else: raise ValueError('review accepted altered '+event+'/'+key)
    for event in ['m1.user_start','m1.user_end','m1.user_settlement','m1.user_call','m1.user_fault','m1.user_bad_return','m1.user_irq','m1.user_ready','m1.user_inventory','m1.user_runtime']:
        if not any(e==event for e,f in rows): continue
        try: audit([(e,f) for e,f in rows if e != event],group)
        except (ValueError,KeyError): results.append(dict(case='missing '+event,status='rejected'))
        else: raise ValueError('review accepted missing '+event)
    return results


def excerpt(text,name):
    marker='<'+name+'>:'
    require(marker in text,'missing assembly symbol '+name)
    return marker+text.split(marker,1)[1].split('\n\n',1)[0]+'\n'


def codegen(folder):
    texts={}
    for name in ('user_a','user_b','service'):
        loader.expected_pages(folder/(name+'.elf')) # independent static ELF obligations
        require(not (folder/(name+'-llvm-nm.stdout.log')).read_text().strip(),'undefined runtime symbol')
        text=(folder/(name+'-llvm-objdump.stdout.log')).read_text()
        fp=excerpt(text,'user_fp_forbidden')
        require('pxor\t%xmm0, %xmm0' in fp,'missing deliberate #NM site')
        require(len(re.findall(r'%[xyz]mm\d+',text)) == 2,'unexpected user FP/SIMD code')
        require(not re.search(r'^\s*[0-9a-f]+:\s+(?:f[a-z]+|v[a-z]+|ldmxcsr|stmxcsr|xsave|xrstor)\s',text,re.MULTILINE),'implicit FP/SIMD instruction')
        texts[name]=text
        args=read(folder/(name+'-build.command.json'))['arguments']
        require(all(v in args for v in ('--target=x86_64-unknown-none','code-model=small','relocation-model=static','no-redzone=yes','panic=abort','opt-level=1',name)), 'user build obligations')
    kernel=(folder/'llvm-objdump.stdout.log').read_text()
    asm={name:excerpt(kernel,name) for name in ('user_enter','user_leave','user_syscall','user_copy','user_copy_read','user_copy_write','user_copy_failed')}
    require('sysret' not in kernel,'unsafe SYSRET path')
    require(asm['user_syscall'].count('pushq') == 20 and asm['user_syscall'].count('popq') == 15
            and 'iretq' in asm['user_syscall'] and 'cld' in asm['user_syscall'], 'syscall assembly layout')
    require(asm['user_enter'].count('pushq') == 11 and 'iretq' in asm['user_enter'], 'entry layout')
    require('movq\t%rax, %cr3' in asm['user_leave'] and 'retq' in asm['user_leave'], 'surviving stack/root return')
    return asm,texts


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--acceptance',type=Path,required=True)
    parser.add_argument('--destination',type=Path)
    args=parser.parse_args()
    a=read(args.acceptance)
    require(a['story']=='P3-04' and a['status']=='passed' and a['host_test_threads']==1 and len(a['commands'])==6 and all(c['exit_code']==0 for c in a['commands']),'incomplete acceptance')
    pre,baseline,owned,staged,boundary=(local(a['runs'][k]) for k in ('prerequisites','m1','memory','loader','boundary'))
    prerequisites=read(pre/'prerequisites.json')
    require(prerequisites['status']=='passed' and prerequisites['offline'] and len(prerequisites['commands'])==10 and all(c['exit_code']==0 for c in prerequisites['commands']),'prerequisites')
    sources=read(baseline/'suite-source-inputs.json')
    pre_sources={v['path']:v['sha256'] for v in read(pre/'source-inputs.json')}
    require(all(pre_sources.get(p)==h==digest(ROOT/p) for p,h in sources.items()),'stale prerequisite/source inputs')
    require(all(read(p/'build-inputs.json')['sources']==sources for p in (owned,staged,boundary)),'different regression sources')
    m1=read(baseline/'review.json')
    require(m1['status']=='passed' and len(m1['launches'])==42 and len(read(baseline/'result.json')['stages']['rows'])==33
            and m1['review_tool_sha256']==digest(ROOT/'tools/Review-P1-07.ps1'),'M1 gate')
    for item in m1['launches']: memory.lifecycle(local(item['directory']),item['outcome']!='broken-before-entry')
    mr,_,_=memory.lifecycle(owned/'owned-memory-run')
    memory_audit=memory.table_audit(memory.records((owned/'owned-memory-run/serial.log').read_bytes(),read(owned/'build-manifest.json')['build_id'],mr['host_session_id']))
    lr,_,_=memory.lifecycle(staged/'static-loader-run')
    loader_audit=loader.audit(memory.records((staged/'static-loader-run/serial.log').read_bytes(),read(staged/'build-manifest.json')['build_id'],lr['host_session_id']),[loader.expected_pages(staged/(n+'.elf')) for n in ('user_a','user_b','service')])
    mf=read(boundary/'build-manifest.json')
    require(mf['flavor']=='user-boundary-test' and mf['kernel_sha256']==digest(boundary/'kernel.elf'),'boundary kernel identity')
    memory.elf_audit(boundary/'kernel.elf')
    runs=[boundary/('user-boundary-'+suffix+'-run') for suffix in 'abcd']
    observed=[];mutations=[];results=[]
    for group,run in enumerate(runs):
        result,launch,case=memory.lifecycle(run)
        require(result['emulator_test_exit']==33 and result['build_id']==mf['build_id'] and launch['fixture']=='user-boundary-'+ 'abcd'[group],'guest outcome/identity')
        rows=memory.records((run/'serial.log').read_bytes(),mf['build_id'],result['host_session_id'])
        observed.append(audit(rows,group)); mutations+=mutation_checks(rows,group)
        results.append(result)
    require(len({r['host_session_id'] for r in results})==4,'fresh launch sessions')
    asm,user=codegen(boundary)
    descriptors=read(boundary/'user-images.json')
    for n,item in zip(('user_a','user_b','service'),descriptors['images']):
        require(item['sha256']==digest(boundary/(n+'.elf'))==digest(boundary/'user-boundary-d-package/extracted payload/boot'/(n+'.elf'))==case['package']['payload']['boot/'+n+'.elf'],'image/ISO identity')
    host=(baseline/'kernel-host/check.stdout.log').read_text(encoding='utf-8-sig')
    require('30 passed; 0 failed' in host and 'copy_permissions_include_every_hardware_ancestor ... ok' in host
            and 'rejects_privilege_noncanonical_and_reserved_flags ... ok' in host,'host regression coverage')
    report=dict(schema_version=1,story='P3-04',status='passed',review='local Codex self-review; independent parsers, not external approval',
                reviewer_sha256=digest(__file__),acceptance_sha256=digest(args.acceptance),source_commit=a['source_commit'],build_id=mf['build_id'],kernel_sha256=mf['kernel_sha256'],
                iso_sha256=[r['iso_sha256'] for r in results],serial_sha256=[digest(r/'serial.log') for r in runs],audit=observed,memory_audit=memory_audit,loader_audit=loader_audit,reviewer_mutations=mutations,
                prerequisite_checks=10,m1_rows=33,m1_launches=42,kernel_host_tests=30,user_execution=True,preemption=False,public_memory_rights=False,desktop_targets_accepted=False)
    report['unsafe_review']={
        'scope':'local Codex source/assembly review with independently parsed guest and codegen evidence',
        'entry':'Reviewed 160-byte syscall frame, SysV alignment, trusted stack/current, FMASK, fresh segment bases and validated IRETQ; no SYSRET',
        'copy':'Reviewed complete-span software/effective hardware permission checks, balanced generation pins, IRQ exclusion and exactly two range-qualified recovery sites',
        'teardown':'Reviewed surviving kernel root/stack restoration before TSS reset and process/frame reap',
        'staging':'Reviewed exclusive unmapped allocation bounds and page-batched RW/NX to RO/NX alias restoration with invalidation',
        'fatal_regressions':'Complete M1 kernel fatal, guard, IST and returning-interrupt gate required',
        'limits':'One CPU and no preemption; user FP/SIMD blocked by TS; no public memory rights, guest UI or general supervisor acceptance'}
    save(args.acceptance.parent/'review.json',report)
    if args.destination:
        dest=args.destination.resolve()
        require(dest.is_relative_to(ROOT/'evidence/phase-3/P3-04') and not dest.exists(),'invalid/existing destination')
        dest.mkdir(parents=True)
        for name,value in [('review.json',report),('acceptance.json',a),('prerequisites.json',prerequisites),('baseline-source-inputs.json',sources),('m1-review.json',m1),('m1-result.json',read(baseline/'result.json')),('boundary-build-inputs.json',read(boundary/'build-inputs.json')),('boundary-build-manifest.json',mf),('user-images.json',descriptors)]: save(dest/name,value)
        shutil.copyfile(baseline/'kernel-host/check.stdout.log',dest/'host-tests.txt')
        (dest/'codegen').mkdir()
        for name,text in asm.items(): (dest/'codegen'/(name+'.txt')).write_text(text,encoding='utf-8')
        for n,text in user.items():
            save(dest/'codegen'/(n+'-command.json'),read(boundary/(n+'-build.command.json')))
            (dest/'codegen'/(n+'-entry.txt')).write_text(excerpt(text,'fixture_entry')+excerpt(text,'register_check')+excerpt(text,'user_fp_forbidden'),encoding='utf-8')
        shutil.copyfile(boundary/'runtime-ids.rs',dest/'codegen/runtime-ids.txt')
        provenance=dict(sources)
        for p in ['tools/Test-P3-04.ps1','tools/Review-P3-04.py','tools/Review-P3-03.py','tools/Review-P3-02.py','docs/05-delivery/phase-3/syscall-boundary-runbook.md','docs/04-decisions/0020-user-entry-and-fault-boundary.md','docs/06-contracts/runtime-v0.md']:
            provenance[p]=digest(ROOT/p)
        save(dest/'source-provenance.json',dict(source_commit=a['source_commit'],dirty_input_hashes=provenance))
        launches=[(local(i['directory']),f'{n:02d}') for n,i in enumerate(m1['launches'])]+[(owned/'owned-memory-run','memory'),(staged/'static-loader-run','loader'),]+[(r,'boundary-'+suffix) for r,suffix in zip(runs,'abcd')]
        (dest/'launches').mkdir()
        for folder,label in launches:
            shutil.copyfile(folder/'serial.log',dest/'launches'/(label+'-serial.txt'))
            save(dest/'launches'/(label+'-context.json'),dict(result=read(folder/'run-result.json'),launch=read(folder/'launch-manifest.json'),package=read(folder/'case-result.json')['package'],serial_sha256=digest(folder/'serial.log')))
        files=[p for p in dest.rglob('*') if p.is_file()]
        require(len(files)<190 and sum(p.stat().st_size for p in files)<4*1024*1024 and all(p.stat().st_size<=256*1024 for p in files),'evidence budget')
        print(dest)
    print(json.dumps(report,indent=2))


if __name__=='__main__': main()
