"""Independent raw P3-06 rights/lifetime, hardware-fault and regression review."""
import argparse
import copy
import importlib.util
import json
from pathlib import Path
import re
import shutil

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('scheduler', ROOT/'tools/Review-P3-05.py')
scheduler = importlib.util.module_from_spec(spec)
spec.loader.exec_module(scheduler)
boundary, memory, loader = scheduler.boundary, scheduler.memory, scheduler.loader
require, read, save, digest, local = memory.require, memory.read, memory.save, memory.digest, memory.local
BASIC, MAP, READ, WRITE, EXECUTE, CONTROL = 7, 16, 32, 64, 128, 256


def audit(rows):
    by = {}
    for e, f in rows: by.setdefault(e, []).append(f)
    require(sum(e == 'm1.kernel_entry' for e, _ in rows) == 1, 'guest reset')
    require(not any(e in by for e in ('m1.exception', 'm1.panic', 'm1.boot_rejected')), 'kernel fatal')
    ready = by.get('m1.cap_ready', [])
    require(len(ready) == 1, 'missing completion')
    ready = ready[0]
    require(all(ready.get(k) == v for k, v in dict(waves='6', tasks='18', settled='true', user_execution='true').items()), 'incomplete experiment')
    require(0 < int(ready['fixed_bytes']) <= 16777216 and 0 < int(ready['pin_irqs']) <= int(ready['nested_irqs']), 'fixed storage/IRQ evidence')
    starts, admits, ends = (by.get(e, []) for e in ('m1.task_start', 'm1.cap_admit', 'm1.cap_reap'))
    require(len(starts) == len(admits) == len(ends) == 18, 'missing owner lifecycle')
    require(len(by.get('m1.cap_inventory',[]))==30,'missing inventories')
    require(len(by.get('m1.cap_pin', [])) == len(by.get('m1.cap_map', [])) == 6, 'missing shared pin/map evidence')
    for f in by['m1.cap_pin']:
        require(all(f.get(k) == '6' for k in ('unmap', 'protect', 'close', 'termination')) and all(f.get(k) == 'true' for k in ('unchanged', 'irq_pending', 'unpinned')), 'copy interleave outcome')
    for f in by['m1.cap_map']:
        require(f['base'] == str(0x50000000), 'shared map base')
        require(all(int(f['e'+str(n)], 0) & 7 == 7 for n in range(3)), 'ancestor user permission')
        require(any(int(c['case'])==int(f['case']) and c['operation']=='10' and c['status']=='0' and c['b']==f['memory'] and c['c']==f['base'] and c.get('x')=='0' and int(c['seq'])==int(f['seq'])+1 for c in by.get('m1.cap_call',[])),'unrelated shared table record')
        leaf, alias = int(f['e3'], 0), int(f['alias'], 0)
        require(leaf & 7 == 5 and leaf >> 63 == 1 and alias & 7 == 1 and alias >> 63 == 1, 'shared R/NX or supervisor alias escalation')
        require(leaf & 0x000fffffffff_f000 == alias & 0x000fffffffff_f000, 'alias backing mismatch')

    previous = [0, 0, 0]
    nonces = set()
    calls = by.get('m1.cap_call', [])
    denied = {}
    for wave in range(6):
        ss, aa = starts[wave*3:wave*3+3], admits[wave*3:wave*3+3]
        ee = [f for f in ends if int(f['wave']) == wave]
        inventories=[f for f in by['m1.cap_inventory'] if int(f['wave'])==wave]
        require(sorted(f['phase'] for f in inventories)==['admitted','reap','reap','reap','settled'],'missing wave inventory')
        require({int(f['task']) for f in ee} == {0, 1, 2}, 'wave owner completion')
        require(len({s['root'] for s in ss}) == len({s['stack'] for s in ss}) == 3, 'shared private root/stack')
        tables, objects, mappings = {}, {}, {0: [], 1: [], 2: []}
        active = set(range(3))

        def install(task, token, obj, rights):
            require(token != 0 and 1 <= token & 0xffffffff <= 32, 'invalid handle encoding')
            require(token >> 32 not in nonces and token not in tables[task], 'resurrected handle nonce')
            require(all(h & 0xffffffff != token & 0xffffffff for h in tables[task]), 'live slot collision')
            nonces.add(token >> 32)
            tables[task][token] = (obj, rights)

        for task, (s, a) in enumerate(zip(ss, aa)):
            require(int(s['thread']) == int(s['process']) == task and int(s['principal']) == [101, 202, 101][task] and int(s['mode']) == 100+wave, 'trusted binding/same-user fixture')
            require(int(s['generation']) > previous[task] and s['fresh'] == 'true', 'stale process identity')
            previous[task] = int(s['generation'])
            end = next(f for f in ee if int(f['task']) == task)
            expected = 270 if task == 1 and wave else 0
            require(int(end['result']) == int(end['expected']) == expected and end['generation'] == s['generation'] and int(end['principal']) == [101, 202, 101][task] and int(end['authority_session']) == [1001, 2002, 1001][task], 'owner/fault result')
            require(int(s['seq']) < int(end['seq']), 'reap before entry')
            require(int(a['module'])==task,'wrong admission module')
            tables[task] = {}
            p, t = int(a['process']), int(a['thread'])
            objects[p], objects[t] = (1, 0, 0), (2, 0, 0)
            install(task, p, p, BASIC | 8 | MAP | CONTROL)
            install(task, t, t, BASIC | CONTROL)
            h, foreign = int(a['grant']), int(a['foreign'])
            if task == 0:
                objects[h] = (3, READ, 4096)
                install(task, h, h, BASIC | MAP | READ)
            elif task == 1:
                require(foreign == int(aa[0]['grant']) and a['explicit_read'] == 'true', 'untrusted sharing admission')
                install(task, h, foreign, BASIC | MAP | READ)
            else: require(h == 0 and foreign == int(aa[0]['grant']) and a['explicit_read'] == 'false', 'same-user ambient grant')

        def live_objects():
            refs = {o for table in tables.values() for o, _ in table.values()}
            refs |= {m['obj'] for ms in mappings.values() for m in ms}
            refs |= {int(aa[t][key]) for t in active for key in ('process', 'thread')}
            return refs

        wave_rows = [(e, f) for e, f in rows if (e == 'm1.cap_call' and int(f['case']) == 100+wave) or (e in ('m1.cap_reap', 'm1.cap_inventory') and int(f['wave']) == wave)]
        for event, f in wave_rows:
            if event == 'm1.cap_call':
                task, op, status, value, a, b, c = (int(f[k]) for k in ('process', 'operation', 'status', 'value', 'a', 'b', 'c'))
                require(task in active and int(f['generation']) == previous[task], 'call after death/stale process')
                table = tables[task]
                h = table.get(a)
                if status: require(value == 0, 'error leaked scalar')
                if op in (6, 7, 8):
                    expected = 1 if h is None else 3 if h[1] & {6:1, 7:4, 8:2}[op] == 0 else 3 if op == 8 and b & ~h[1] else 11 if op == 8 and len(table) == 32 else 0
                    require(status == expected, 'handle status/type/rights mismatch')
                    if not status:
                        obj, rights = h
                        if op == 6: require(value == objects[obj][0] << 32 | rights, 'query disclosed unexpected identity/rights')
                        elif op == 7: del table[a]
                        else: install(task, value, obj, b)
                elif op == 9:
                    expected = 1 if h is None else 2 if objects[h[0]][0] != 1 else 3 if h[1] & 8 == 0 else 4 if b == 0 or b % 4096 or b > 65536 or c not in (READ, READ | WRITE, READ | EXECUTE) else 11 if len(table) == 32 else 0
                    require(status == expected, 'memory create admission/status mismatch')
                    if not status:
                        objects[value] = (3, c, b)
                        install(task, value, value, BASIC | MAP | c)
                elif op == 10:
                    mem = table.get(b)
                    requested = 'x' in f
                    x, offset, length = (int(f[k]) for k in ('x','d','n')) if requested else (-1,0,0)
                    right = {0:READ,1:READ|WRITE,2:READ|EXECUTE}.get(x,0)
                    expected = 4 if not requested or not right else 1 if h is None else 2 if objects[h[0]][0] != 1 else 3 if h[1]&MAP == 0 else 1 if mem is None else 2 if objects[mem[0]][0] != 3 else 3 if mem[1]&(MAP|right) != MAP|right else 4 if length == 0 or offset%4096 or length%4096 or offset+length > objects[mem[0]][2] else 11 if len(mappings[task]) == 32 else 4 if c%4096 or c<0x10000 or c+length>0x80000000 or any(c<m['base']+m['bytes'] and m['base']<c+length for m in mappings[task]) else 0
                    require(status == expected, 'map range/type/rights/quota status mismatch')
                    if not status:
                        mappings[task].append(dict(obj=mem[0],base=c,bytes=length,ceiling=mem[1]&objects[mem[0]][1]))
                elif op in (11,12):
                    mapping = next((m for m in mappings[task] if m['base']==b),None)
                    right = {0:READ,1:READ|WRITE,2:READ|EXECUTE}.get(c,0)
                    expected = 4 if op==12 and not right else 1 if h is None else 2 if objects[h[0]][0]!=1 else 3 if h[1]&MAP==0 else 4 if mapping is None else 3 if op==12 and mapping['ceiling']&right!=right else 0
                    require(status == expected, 'unmap/protect target/mapping ceiling status mismatch')
                    if op==11 and not status: mappings[task].remove(mapping)
                elif op in (4, 5):
                    control_handle = table.get(a if op == 4 else b)
                    expected = 1 if control_handle is None else 2 if objects[control_handle[0]][0] != (2 if op == 4 else 1) else 3 if control_handle[1] & CONTROL == 0 else 0
                    require(status == expected, 'control rights/type mismatch')
                    if op == 5 and not status: require(a == 0, 'user assertion failure')
                else: require(op in (2, 3) and status == 12, 'legacy allocation unexpectedly available')
                denied[(op, status)] = denied.get((op, status), 0)+1
            elif event == 'm1.cap_reap':
                task = int(f['task'])
                require(task in active, 'duplicate reap')
                active.remove(task); tables[task] = {}; mappings[task] = []
            else:
                refs = live_objects()
                require(int(f['owners']) == len(active) and int(f['objects']) == len(refs) and int(f['handles']) == sum(map(len, tables.values())) and int(f['mappings']) == sum(map(len, mappings.values())), 'independent reference inventory mismatch')
                require(int(f['backing']) == sum(objects[o][2] for o in refs if objects[o][0] == 3) and int(f['mapped']) == sum(m['bytes'] for ms in mappings.values() for m in ms), 'backing/mapped charge mismatch')
                if f['phase'] == 'settled': require(all(int(f[k]) == 0 for k in ('owners','objects','handles','mappings','backing','mapped','frames','spaces','allocations','committed','resident','tables','stacks','pins')), 'resource leak')
        require(not active and not live_objects(), 'incomplete teardown')
        tokens = [f for f in by.get('m1.user_log', []) if int(f['case']) == 100+wave]
        require(sum(f['token'] == 'same-user-denied' for f in tokens) == 1 and sum(f['token'] == 'creator-close' for f in tokens) == 1 and sum(f['token'] == 'shared-after-close' for f in tokens) == 1, 'missing positive/negative execution token')
        if wave == 0:
            require(any(f['token'] == 'zero-reuse' for f in tokens) and any(f['token'] == 'rights-quotas-pass' for f in tokens), 'missing zero/quota workflow')
            shared = next(f for f in tokens if f['token'] == 'shared-after-close')
            require(int(shared['seq']) > int(next(f for f in ee if f['task'] == '0')['seq']), 'no post-creator-death shared read')

    faults = by.get('m1.user_fault', [])
    require(len(faults) == 5, 'missing hardware faults')
    for case, address, error in [(101,0x50000000,7),(102,0x50000000,21),(103,0xffffffff80000000,5),(104,0x60000000,4),(105,0x60000000,6)]:
        f = next(f for f in faults if int(f['case']) == case)
        require(int(f['vector']) == 14 and int(f['address'],0) == address and int(f['error'],0) == error and f['cs'] == '0x33' and f['ss'] == '0x3b', 'wrong isolation fault')
        if case >= 104:
            issuer = next(e for e in ends if int(e['wave']) == case-100 and e['task'] == '0')
            require(int(f['seq']) < int(issuer['seq']), 'private owner was already dead')
    for pair in [(6,1),(6,3),(7,1),(7,3),(8,3),(8,11),(9,1),(9,2),(9,3),(9,4),(9,11),(10,1),(10,2),(10,3),(10,4),(10,11),(12,3),(2,12),(3,12)]:
        require(denied.get(pair, 0) > 0, 'missing required exact denial '+str(pair))
    return dict(tasks=18,waves=6,hardware_faults=5,unique_handle_installations=len(nonces),syscalls=len(calls),pin_interleavings=6,denials={str(k):v for k,v in denied.items() if k[1]},fixed_bytes=int(ready['fixed_bytes']),switches=int(ready['switches']))


def mutations(rows):
    rejected = []
    for event in ('m1.cap_ready','m1.cap_admit','m1.cap_reap','m1.cap_inventory','m1.cap_call','m1.cap_pin','m1.cap_map','m1.user_fault','m1.user_log'):
        try: audit([(e,f) for e,f in rows if e != event])
        except (ValueError,KeyError,IndexError,StopIteration): rejected.append('missing '+event)
        else: raise ValueError('accepted missing '+event)
    for event,key,value in [('m1.cap_call','status','99'),('m1.cap_reap','principal','999'),('m1.cap_reap','result','99'),('m1.cap_inventory','objects','99'),('m1.cap_inventory','backing','0'),('m1.cap_map','e3','0x7'),('m1.cap_pin','close','0'),('m1.user_fault','error','0'),('m1.cap_ready','fixed_bytes','16777217')]:
        changed=copy.deepcopy(rows); next(f for e,f in changed if e==event)[key]=value
        try: audit(changed)
        except (ValueError,KeyError,IndexError,StopIteration): rejected.append(event+'.'+key)
        else: raise ValueError('accepted mutation '+event+'.'+key)
    for op,key,value in [(10,'x','1'),(10,'c',str(0xffffffff80000000)),(10,'d',str(2**64-4096)),(10,'n','0'),(8,'b',str(2**64-1))]:
        changed=copy.deepcopy(rows)
        next(f for e,f in changed if e=='m1.cap_call' and int(f['operation'])==op and f['status']=='0')[key]=value
        try: audit(changed)
        except (ValueError,KeyError,IndexError,StopIteration): rejected.append('successful operation '+str(op)+'.'+key)
        else: raise ValueError('accepted operation mutation '+str(op)+'.'+key)
    return rejected


def codegen(folder):
    text=(folder/'llvm-objdump.stdout.log').read_text()
    helpers={n:boundary.excerpt(text,n) for n in ('task_resume','task_save_fp','user_syscall','user_leave','timer_vector','exception_common')}
    fp=lambda m,o: m.startswith(('f','v','xsave','xrstor')) or m in ('ldmxcsr','stmxcsr','emms') or re.search(r'%(?:[xyz]mm|st|mm|k[0-7])',o)
    instructions=lambda t: re.findall(r'^\s*[0-9a-f]+:\s+(\S+)([^\n]*)',t,re.M)
    require(sorted(m for m,o in instructions(text) if fp(m,o))==['fxrstor64','fxsave64'],'unexpected kernel FP')
    require('fxrstor64' in helpers['task_resume'] and 'fxsave64' in helpers['task_save_fp'] and 'iretq' in helpers['task_resume'] and 'sysret' not in text,'context instruction closure')
    require(helpers['user_syscall'].count('pushq')==21 and helpers['user_syscall'].count('popq')==16,'syscall frame closure')
    for name,item in zip(('user_a','user_b','service'),read(folder/'user-images.json')['images']):
        scheduler.user_elf(folder/(name+'.elf'))
        require(item['sha256']==digest(folder/(name+'.elf')) and not (folder/(name+'-llvm-nm.stdout.log')).read_text().strip(),'user artifact/symbol closure')
        require(not any(fp(m,o) for m,o in instructions((folder/(name+'-llvm-objdump.stdout.log')).read_text())),'unexpected user FP')
    return helpers


# The historical P3-04 reviewer predates P3-05's scratch push/pop and TS guard.
def boundary_codegen(folder):
    texts={}
    for name in ('user_a','user_b','service'):
        loader.expected_pages(folder/(name+'.elf')) # independent static ELF obligations
        require(not (folder/(name+'-llvm-nm.stdout.log')).read_text().strip(),'undefined runtime symbol')
        text=(folder/(name+'-llvm-objdump.stdout.log')).read_text()
        fp=boundary.excerpt(text,'user_fp_forbidden')
        require('pxor\t%xmm0, %xmm0' in fp,'missing deliberate #NM site')
        require(len(re.findall(r'%[xyz]mm\d+',text)) == 2,'unexpected user FP/SIMD code')
        require(not re.search(r'^\s*[0-9a-f]+:\s+(?:f[a-z]+|v[a-z]+|ldmxcsr|stmxcsr|xsave|xrstor)\s',text,re.MULTILINE),'implicit FP/SIMD instruction')
        texts[name]=text
        args=read(folder/(name+'-build.command.json'))['arguments']
        require(all(v in args for v in ('--target=x86_64-unknown-none','code-model=small','relocation-model=static','no-redzone=yes','panic=abort','opt-level=1',name)), 'user build obligations')
    kernel=(folder/'llvm-objdump.stdout.log').read_text()
    asm={name:boundary.excerpt(kernel,name) for name in ('user_enter','user_leave','user_syscall','user_copy','user_copy_read','user_copy_write','user_copy_failed')}
    require('sysret' not in kernel,'unsafe SYSRET path')
    require(asm['user_syscall'].count('pushq') == 21 and asm['user_syscall'].count('popq') == 16
            and 'iretq' in asm['user_syscall'] and 'cld' in asm['user_syscall'], 'syscall assembly layout')
    require('%cr0' in asm['user_syscall'], 'missing P3-05 TS guard')
    require(asm['user_enter'].count('pushq') == 11 and 'iretq' in asm['user_enter'], 'entry layout')
    require('movq\t%rax, %cr3' in asm['user_leave'] and 'retq' in asm['user_leave'], 'surviving stack/root return')
    return asm,texts


def focused(folder):
    contract=read(ROOT/'contracts/runtime-v0.json')['objects_boundary']
    require(contract['rights']==dict(QUERY=1,DUPLICATE=2,CLOSE=4,CREATE=8,MAP=16,READ=32,WRITE=64,EXECUTE=128,CONTROL=256),'contract/runtime rights drift')
    require(contract['map_request_fields']==[['base',8],['offset',8],['bytes',8],['access',8]] and contract['access']==dict(Read=0,Write=1,Execute=2),'map encoding/access drift')
    profile=read(ROOT/'config/m3-profiles.json')['objects_execution_subset']
    require((profile['owners'],profile['objects_global'],profile['handles_per_process'],profile['public_mappings_per_process'],profile['memory_call_bytes'])==(16,128,32,32,65536),'profile/registry drift')
    require(read(folder/'result.json')['status']=='passed','failed focused run')
    mf=read(folder/'build-manifest.json')
    require(mf['flavor']=='capability-test' and digest(folder/'kernel.elf')==mf['kernel_sha256'],'wrong kernel identity')
    memory.elf_audit(folder/'kernel.elf')
    run=folder/'capabilities-run'; result,launch,case=memory.lifecycle(run)
    require(result['emulator_test_exit']==33 and launch['fixture']=='capabilities' and result['build_id']==mf['build_id'],'launch/exit identity')
    argv=launch['arguments']
    require(argv[argv.index('-cpu')+1]=='qemu64-v1,+apic' and argv[argv.index('-accel')+1]=='tcg','CPU/accelerator profile')
    for item in read(folder/'user-images.json')['images']:
        require(case['package']['payload']['boot/'+item['name']+'.elf']==item['sha256']==digest(folder/'capabilities-package/extracted payload/boot'/(item['name']+'.elf')),'actual ISO user payload')
    rows=memory.records((run/'serial.log').read_bytes(),mf['build_id'],result['host_session_id'])
    return dict(audit=audit(rows),mutations=mutations(rows),build_id=mf['build_id'],kernel_sha256=mf['kernel_sha256'],iso_sha256=result['iso_sha256'],serial_sha256=digest(run/'serial.log')),codegen(folder)


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--acceptance',type=Path);p.add_argument('--capabilities',type=Path);p.add_argument('--destination',type=Path)
    args=p.parse_args();a=read(args.acceptance) if args.acceptance else None
    folder=local(a['runs']['capabilities']) if a else args.capabilities.resolve()
    observed,helpers=focused(folder)
    report=dict(schema_version=1,story='P3-06',status='passed',review='local Codex self-review; independently parsed evidence, not external approval',reviewer_sha256=digest(__file__),scope='focused guest/artifact audit',**observed)
    launches=[(folder/'capabilities-run','capabilities')]
    if a:
        require(a['story']=='P3-06' and a['status']=='passed' and len(a['commands'])==8 and all(c['exit_code']==0 for c in a['commands']),'incomplete aggregate')
        pre,baseline,owned,staged,old,scheduled=(local(a['runs'][k]) for k in ('prerequisites','m1','memory','loader','boundary','scheduler'))
        pre_report=read(pre/'prerequisites.json')
        require(pre_report['status']=='passed' and pre_report['offline'] and len(pre_report['commands'])==10 and all(c['exit_code']==0 for c in pre_report['commands']),'prerequisites')
        sources=read(baseline/'suite-source-inputs.json');pre_sources={i['path']:i['sha256'] for i in read(pre/'source-inputs.json')}
        require(all(pre_sources.get(path)==sha==digest(ROOT/path) for path,sha in sources.items()),'stale final-source prerequisite/build')
        require(all(read(f/'build-inputs.json')['sources']==sources for f in (owned,staged,old,scheduled,folder)),'different regression sources')
        m1=read(baseline/'review.json')
        require(m1['status']=='passed' and len(m1['launches'])==42 and len(read(baseline/'result.json')['stages']['rows'])==33 and m1['review_tool_sha256']==digest(ROOT/'tools/Review-P1-07.ps1'),'incomplete M1')
        for n,item in enumerate(m1['launches']):
            run=local(item['directory']);memory.lifecycle(run,item['outcome']!='broken-before-entry');launches.append((run,'m1-'+str(n)))
        host=(baseline/'kernel-host/check.stdout.log').read_text(encoding='utf-8-sig')
        require('39 passed; 0 failed' in host and all(name+' ... ok' in host for name in ('objects::tests::rights_types_stale_cross_process_and_nonce_exhaustion','objects::tests::immutable_mapping_outlives_creator_and_all_handles_with_pin_barriers','objects::tests::object_slot_retirement_and_page_table_failure_unwind','objects::tests::closing_self_handles_preserves_admitted_owner_lifecycle')),'host boundary/failure coverage')
        mr,_,_=memory.lifecycle(owned/'owned-memory-run')
        report['memory_audit']=memory.table_audit(memory.records((owned/'owned-memory-run/serial.log').read_bytes(),read(owned/'build-manifest.json')['build_id'],mr['host_session_id']))
        lr,_,_=memory.lifecycle(staged/'static-loader-run')
        report['loader_audit']=loader.audit(memory.records((staged/'static-loader-run/serial.log').read_bytes(),read(staged/'build-manifest.json')['build_id'],lr['host_session_id']),[loader.expected_pages(staged/(n+'.elf')) for n in ('user_a','user_b','service')])
        for group,suffix in enumerate('abcd'):
            run=old/('user-boundary-'+suffix+'-run');r,_,_=memory.lifecycle(run)
            rows=memory.records((run/'serial.log').read_bytes(),read(old/'build-manifest.json')['build_id'],r['host_session_id']);boundary.audit(rows,group);report['mutations']+=boundary.mutation_checks(rows,group);launches.append((run,'boundary-'+suffix))
        for suffix,trace in [('on',True),('off',False)]:
            run=scheduled/('scheduler-'+suffix+'-run');r,_,_=memory.lifecycle(run)
            rows=memory.records((run/'serial.log').read_bytes(),read(scheduled/'build-manifest.json')['build_id'],r['host_session_id']);scheduler.audit(rows,trace,True);report['mutations']+=scheduler.mutations(rows,trace,True);launches.append((run,'scheduler-'+suffix))
        run=scheduled/'scheduler-cpu-run';r,l,_=memory.lifecycle(run)
        rows=memory.records((run/'serial.log').read_bytes(),read(scheduled/'build-manifest.json')['build_id'],r['host_session_id'])
        require('-sse2' in l['arguments'][l['arguments'].index('-cpu')+1] and any(e=='m1.scheduler_cpu_rejected' and f['user_entries']=='0' and not int(f['cpuid_edx'],0)&(1<<26) for e,f in rows),'real CPU rejection')
        scheduler.codegen(scheduled);boundary_codegen(old)
        launches.extend([(run,'scheduler-cpu'),(owned/'owned-memory-run','memory'),(staged/'static-loader-run','loader')])
        report.update(scope='complete final-source story acceptance',source_commit=a['source_commit'],acceptance_sha256=digest(args.acceptance),prerequisite_checks=10,m1_rows=33,m1_launches=42,kernel_host_tests=39,scheduler_regression_tasks=60,boundary_regression_runs=36)
        report['criterion_review']={
            'unforgeable_local_lookup':'pass: independent replay of unique installation nonces, stale/foreign/type/rights errors; host-forced nonce/object exhaustion cannot wrap',
            'attenuation_and_query':'pass: subset duplication, required QUERY/DUPLICATE/CLOSE and CONTROL, explicit scalar kind/rights, supervisor lifecycle refs through self-close',
            'guest_memory_enforcement':'pass: shared ancestor/leaf/alias entries R/NX; shared write/execute, kernel read, live foreign-private read/write hardware faults; legacy pointer calls unsupported',
            'independent_lifetimes':'pass: creator-death and handle-close survival, live-pin IRQ and blocked close/unmap/protect/remove, exact final cleanup and zero reuse',
            'precharge_and_revocation_limits':'pass: full handles/maps and host physical/table/object failure rollback; duplicates/maps intentionally survive issuer close/death; no universal revoke',
        }
        report['unsafe_review']={'single_cpu':'IF-clear structural borrows; hard IRQ only marks pending; no user reference or pin crosses dispatch','copies':'32-byte metadata copied once, effective page checks, balanced pins and existing bounded fault recovery','mapping':'inactive-bank build before CR3 publication; no fallible registry installation after publish; immutable handle/backing-derived ceiling retained by mapping','teardown':'supervisor root/stack before registry and space removal; mapping subset rebuild needs no frames; held backing detached and globally charged','authority':'trusted launch only; same principal/session cannot mint grants; no kernel pointer in object query','limits':'one owner thread, no SMP/public device/IRQ/DMA/general child supervisor/IPC transfer/SDK freeze/UI; direct grants are not revocable by source close'}
        save(args.acceptance.parent/'review.json',report)
        if args.destination:
            dest=args.destination.resolve();require(dest.is_relative_to(ROOT/'evidence/phase-3/P3-06') and not dest.exists(),'invalid/existing destination');dest.mkdir(parents=True)
            for name,value in [('acceptance.json',a),('review.json',report),('prerequisites.json',pre_report),('m1-review.json',m1),('m1-result.json',read(baseline/'result.json')),('objects-build-inputs.json',read(folder/'build-inputs.json')),('objects-build-manifest.json',read(folder/'build-manifest.json')),('user-images.json',read(folder/'user-images.json'))]:save(dest/name,value)
            shutil.copyfile(baseline/'kernel-host/check.stdout.log',dest/'host-tests.txt')
            provenance=dict(sources)
            for name in ('tools/Test-P3-06.ps1','tools/Review-P3-06.py','tools/Review-P3-05.py','tools/Review-P3-04.py','tools/Review-P3-03.py','tools/Review-P3-02.py','docs/05-delivery/phase-3/handles-memory-runbook.md','docs/04-decisions/0022-retained-memory-objects-and-handle-ceilings.md','docs/06-contracts/runtime-v0.md'):
                provenance[name]=digest(ROOT/name)
            save(dest/'source-provenance.json',dict(source_commit=a['source_commit'],dirty_input_hashes=provenance))
            (dest/'codegen').mkdir();(dest/'launches').mkdir()
            for name,text in helpers.items():(dest/'codegen'/(name+'.txt')).write_text(text)
            shutil.copyfile(folder/'runtime-ids.rs',dest/'codegen/runtime-ids.txt')
            for run,label in launches:
                shutil.copyfile(run/'serial.log',dest/'launches'/(label+'-serial.txt'))
                save(dest/'launches'/(label+'-context.json'),dict(result=read(run/'run-result.json'),launch=read(run/'launch-manifest.json'),package=read(run/'case-result.json')['package'],serial_sha256=digest(run/'serial.log')))
            files=[f for f in dest.rglob('*') if f.is_file()];require(len(files)<190 and sum(f.stat().st_size for f in files)<4*1024*1024 and all(f.stat().st_size<=256*1024 for f in files),'compact evidence budget')
    print(json.dumps(report,indent=2))


if __name__=='__main__':main()
