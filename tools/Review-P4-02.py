"""Independent raw-record, ELF and inventory review. Never accepts a pass string alone."""
import argparse
import collections
import copy
import hashlib
import json
from pathlib import Path
import struct
import re
import tomllib

ROOT = Path(__file__).resolve().parents[1]
ROLES = ['supervisor', 'compositor', 'input', 'shell', 'editor', 'update', 'policy']
HEAPS = [2, 48, 2, 16, 64, 32, 4]
MIB = 1048576

def need(ok, reason):
    if not ok:
        raise ValueError(reason)

def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))

def raw(path, build, session):
    records = []
    for line in Path(path).read_text(encoding='utf-8').splitlines():
        if not line.startswith('m1.'):
            continue
        words = line.split()
        fields = {}
        for word in words[1:]:
            k, v = word.split('=', 1)
            need(k not in fields and v, 'duplicate/empty field')
            fields[k] = v
        need(fields['build'] == build and fields['session'] == session, 'record identity')
        need(fields['version'] == '5' and fields['session_origin'] == 'host', 'record version')
        need(int(fields['seq']) == len(records), 'record sequence')
        need(fields['outcome'] == 'ok' and fields['emergency'] == 'false', 'failed guest record')
        records.append({'event': words[0], 'fields': fields})
    need(records, 'missing records')
    return records

def rows(records, event):
    return [r['fields'] for r in records if r['event'] == event]

def profile():
    actual=read(ROOT/'config/p4-02-runtime.json')
    expected={'schema_version':1,'story':'P4-02','status':'fixed experimental runtime inputs; acceptance tracked in evidence',
        'processes':16,'threads_per_process':1,'lifetime_units':24,'reserved_lifetime_units':3,'retries_per_role':2,
        'heap_caps_mib':dict(zip(ROLES,HEAPS)),'per_role_overhead_cap_bytes':2*MIB,'process_limit_bytes':128*MIB,
        'active_staging_limit_bytes':384*MIB,'recovery_reserve_bytes':16*MIB,'tables_per_bank':128,'table_banks':2,
        'fixed_runtime_limit_bytes':32*MIB,'heap_allocation_slots':512,'heap_max_alignment':4096,
        'private_handles_per_role':3,'live_handle_limit_per_role':12,'live_mapping_limit_per_role':16,
        'ipc_queue_grants_per_role':0,'device_grants_per_role':0,'restart_backoff_ns':20000000,
        'watchdog_owner_residence_ns':1000000000,'watchdog_clock':'elapsed-hpet-owner-residence-including-kernel',
        'heap_cycles_per_owner':100,'reload_process_launches':0,'refund_lifetime_credits':False,'guest_deadline_ms':30000,
        'sdk_contract_sha256':'debe57866ed308f46ff2f48f5065e739c7457d0f28d0e8dccc93d42abd4c891b',
        'kernel_profile':'kernel-stress','kernel_opt_level':3,'bootstrap_handle_rights':[256,256,257],'restart_admission_deferral_ns':320000000,'diagnostic_policy':'desktop-wait-summary-v1','verified_boot_plan_slots':3}
    def same(a,b):
        if type(a) is not type(b): return False
        if isinstance(a,dict): return a.keys()==b.keys() and all(same(a[k],b[k]) for k in a)
        if isinstance(a,list): return len(a)==len(b) and all(same(x,y) for x,y in zip(a,b))
        return a==b
    need(same(actual,expected),'runtime profile shape/type/value drift')
    cargo=tomllib.loads((ROOT/'Cargo.toml').read_text())
    need(type(cargo['profile']['kernel-stress']['opt-level']) is int and cargo['profile']['kernel-stress']['opt-level']==3, 'actual Cargo optimization profile')
    need(digest(ROOT/'contracts/sdk-v1.json')==actual['sdk_contract_sha256'],'frozen SDK drift')
    return actual

def check(records, test):
    need(not rows(records,'m1.desktop_bound'),'failed runtime bound')
    launches = rows(records, 'm1.desktop_launch')
    need(len(launches) == (12 if test else 7), 'launch count')
    need([v['role'] for v in (launches[3:10] if test else launches)] == ROLES, 'trusted graph order')
    for unit, launch in enumerate(launches, 1):
        role = ROLES.index(launch['role'])
        need(int(launch['principal']) == 400 + role and launch['authority_session'] == '1001', 'role binding')
        need(launch['grants'] == 'private-memory' and int(launch['heap']) == HEAPS[role] * MIB, 'role capacity/authority')
        need(int(launch['image_stack_tables_cap']) == 2 * MIB and int(launch['units']) == unit, 'prepaid units')
        need(int(launch['mode']) == (1 if test and unit <= 10 else unit-9 if test else 0), 'trusted fault mode')
    live={};generations={};unit=0
    for record in records:
        v=record['fields'];event=record['event']
        if event=='m1.desktop_launch':
            role=v['role'];pid=int(v['process']);generation=int(v['generation'])
            need(role not in live and generation>generations.get(pid,0),'stale/duplicate role generation')
            generations[pid]=generation;live[role]=(pid,generation);unit+=1
        elif event in ['m1.desktop_reap','m1.desktop_enter']:
            role=v['role']
            need(live.get(role)==(int(v['process']),int(v['generation'])),'role owner/entry/reap mismatch')
            if event=='m1.desktop_reap':
                need(int(v['units'])==unit,'reap refunded or fabricated lifetime units')
                del live[role]
    need(len(live)==(0 if test else 7),'surviving/missing owner')
    inventory = rows(records, 'm1.desktop_inventory')
    need(len(inventory) == 1 and inventory[0]['phase'] == 'admitted', 'inventory count')
    inv = inventory[0]
    need(inv['spaces'] == '7' and inv['units'] == ('10' if test else '7') and int(inv['reserve']) == 16 * MIB, 'admission/reserve')
    need(int(inv['frames'])*4096 == int(inv['committed']), 'physical ownership count')
    need(int(inv['committed']) >= sum(HEAPS)*MIB and int(inv['committed']) < 182*MIB, 'backing capacity')
    need(int(inv['kernel_stacks']) == 7*65536 and int(inv['fixed_tables']) >= 16*2*128*4096, 'fixed/stacks counted')
    caps=rows(records,'m1.desktop_cap_inventory')
    need(len(caps)==1 and all(caps[0][k]==v for k,v in {'owners':'7','objects':'21','handles':'21','mappings':'0','backing':'0','queues':'0','device_grants':'0','process_rights':'256','thread_rights':'256','timer_rights':'257'}.items()),'actual role handle/object/queue authority')
    memory=rows(records,'m1.runtime_memory')
    need(len(memory)==1,'missing runtime storage')
    m=memory[0]
    need(all(m[k]==str(v) for k,v in {'dynamic_ceiling':182*MIB,'user_ceiling':128*MIB,'spaces':16,'allocation_slots':256,'mappings_per_space':128,'tables_per_bank':128,'banks':2}.items()),'backend profile mismatch')
    need(int(m['metadata_bytes'])+int(m['table_storage_bytes'])+int(m['kernel_stack_storage'])<=32*MIB,'fixed runtime budget')
    need(len(rows(records, 'm1.ready')) == 1 and len(rows(records, 'm1.boot_complete')) == 1, 'boot completion')
    if not test:
        ready = rows(records, 'm1.ready')[0]
        need(ready['roles'] == '7' and ready['acceptance_observers'] == 'false', 'ordinary runtime readiness')
        need(not rows(records, 'm1.desktop_unwind') and not rows(records, 'm1.desktop_reap'), 'ordinary fault hooks')
        entries=rows(records,'m1.desktop_enter')
        need(len(entries)==7 and {v['role'] for v in entries}==set(ROLES) and all(v['cpl']=='3' and v['acceptance_observers']=='false' for v in entries), 'ordinary CPL3 dispatch progress')
        return
    unwind = rows(records, 'm1.desktop_unwind')
    need([v['phase'] for v in unwind] == ['1','2','3'], 'partial-start cases')
    need(all(v['units'] == '0' and v['frames'] == '0' and v['authority'] == 'none' and int(v['reserve']) == 16*MIB for v in unwind), 'partial-start settlement')
    need([v['role'] for v in launches[:3]] == ROLES[:3] and [v['role'] for v in launches[10:]] == ['editor','editor'], 'hidden launch')
    reaps = rows(records, 'm1.desktop_reap')
    need(len(reaps) == 12 and all(v['refunded'] == 'false' for v in reaps), 'reap/credit count')
    need([v['role'] for v in reaps[:3]] == ROLES[:3] and all(v['result']=='513' for v in reaps[:3]), 'prepared graph unwind')
    abort = rows(records, 'm1.desktop_abort')
    need(len(abort)==1 and abort[0]['prepared']=='3' and abort[0]['failed_role']=='shell' and abort[0]['units']=='3' and abort[0]['frames']=='0' and abort[0]['refunded']=='false' and int(abort[0]['reserve'])==16*MIB, 'graph abort settlement')
    outcomes = collections.defaultdict(list)
    for v in reaps[3:]:
        outcomes[v['role']].append(int(v['result']))
    need(dict(outcomes) == {r: ([42,90,768] if r == 'editor' else [91] if r == 'update' else [0]) for r in ROLES}, 'lifecycle outcomes')
    need(all(v['token'] == 'desktop-heap-100-settled' for v in rows(records, 'm1.user_log')), 'unexpected desktop assertion/diagnostic')
    logs = [v for v in rows(records, 'm1.user_log') if v['token'] == 'desktop-heap-100-settled']
    need(collections.Counter(int(v['principal']) for v in logs) == collections.Counter({400+i:3 if i==4 else 1 for i in range(7)}), 'real heap workload completion')
    hog_begin=next(i for i,r in enumerate(records) if r['event']=='m1.desktop_launch' and r['fields']['role']=='editor' and r['fields']['mode']=='3')
    hog_end=next(i for i,r in enumerate(records) if r['event']=='m1.desktop_watchdog')
    need(sum(r['event']=='m1.wait_resume' and r['fields']['thread']=='0' for r in records[hog_begin:hog_end])>=2, 'healthy control parked/woke beside actual CPU hog')
    watchdog = rows(records, 'm1.desktop_watchdog')
    need(len(watchdog) == 1 and watchdog[0]['role'] == 'editor' and int(watchdog[0]['residence_without_wait_ns']) >= 1000000000 and watchdog[0]['limit_ns'] == '1000000000', 'hung owner bound')
    settled = rows(records, 'm1.desktop_settled')
    need(len(settled) == 1, 'missing settlement')
    end = settled[0]
    need(all(end[k] == '0' for k in ['spaces','allocations','objects','handles','frames','hidden_launches']), 'unsettled inventory')
    need(end['units'] == '12' and int(end['control']) > 10 and end['editor_ends'] == '3' and end['heap_cycles_per_owner'] == '100' and end['observers'] == 'false', 'completion accounting')
    need(end['ready_limit_ns']=='320000000' and 0 <= int(end['ready_max_ns']) <= 320000000, 'ready-delay bound')
    need(int(end['reserve']) == 16*MIB and int(end['elapsed_ns']) < 25000000000, 'completion bound')

def elf(path):
    b = Path(path).read_bytes()
    need(b[:7] == b'\x7fELF\x02\x01\x01' and len(b) <= MIB, 'user ELF identity/size')
    need(struct.unpack_from('<HHI',b,16) == (2,62,1), 'user ELF target')
    ph = struct.unpack_from('<Q',b,32)[0]
    size, count = struct.unpack_from('<HH',b,54)
    need(size == 56 and 0 < count <= 16, 'user ELF program headers')
    pages = 0
    loads = 0
    for n in range(count):
        kind, flags, off, address, physical, files, memory, align = struct.unpack_from('<IIQQQQQQ', b, ph+n*size)
        if kind != 1:
            need(kind not in (2,3), 'dynamic/interpreter segment')
            continue
        need(flags in (4,5,6) and files <= memory and off+files <= len(b), 'user ELF permissions/span')
        need(address >= 0x400000 and address+memory < 0x40000000, 'image/heap overlap')
        pages += ((address % 4096 + memory + 4095)//4096)
        loads += 1
    need(loads > 0, 'empty ELF')
    return {'sha256':digest(path), 'load_pages':pages, 'load_segments':loads}

def review(directory, test, verify_source=True):
    declared=profile()
    directory = Path(directory)
    prefix = 'desktop-runtime' if test else 'cold-boot-a'
    packaged = directory / (prefix+'-package')
    run = directory / (prefix+'-run')
    result = read(directory/'result.json')
    case = read(run/'case-result.json')
    manifest = read(directory/'build-manifest.json')
    image = read(packaged/'iso-manifest.json')
    inputs = read(directory/'build-inputs.json')
    build_command=read(directory/'kernel-build.command.json')['arguments']
    need(inputs['profile']==declared['kernel_profile'] and build_command[build_command.index('--profile')+1]==declared['kernel_profile'],'kernel optimization profile')
    need(build_command[build_command.index('--features')+1]==('desktop-test' if test else 'desktop-runtime'),'actual kernel features')
    need(result['status'] == case['status'] == 'passed' and case['cleanup_reaped'] and not case['timed_out'], 'guest runner result')
    need(case['deadline_ms'] == 30000 and manifest['flavor'] == ('desktop-test' if test else 'normal'), 'flavor/deadline')
    need(case['emulator_test_exit'] == (33 if test else None), 'exit behavior')
    need(digest(packaged/'continuum.iso') == image['iso_sha256'] == case['iso_sha256'], 'actual ISO hash')
    need(digest(directory/'kernel.elf') == manifest['kernel_sha256'] == image['kernel_sha256'], 'actual kernel hash')
    need(manifest['build_id'] == image['build_id'] == case['build_id'], 'build correlation')
    if verify_source:
        for name, expected in inputs['sources'].items():
            need(digest(ROOT/name) == expected, 'stale source: '+name)
    records = raw(run/'serial.log', manifest['build_id'], image['host_session_id'])
    check(records, test)
    images = [elf(directory/(name+'.elf')) for name in ['user_a','user_b','service']]
    for name in ['user_a','user_b','service']:
        text=(directory/(name+'-llvm-objdump.stdout.log')).read_text()
        instructions=0
        for line in text.splitlines():
            match=re.match(r'^\s*[0-9a-f]+:\s+([a-z][a-z0-9]*)\b(.*)',line)
            if not match: continue
            op,operands=match.groups(); instructions+=1
            need(not re.search(r'%(?:[xyz]mm|mm[0-7]|st\b|k[0-7])',operands), 'desktop integer payload register contract')
            need(not op.startswith(('v','f')) and op not in ['cli','sti','hlt','wrmsr','rdmsr','xgetbv','xsetbv','xsave','xrstor','ldmxcsr','stmxcsr'], 'unsupported desktop user instruction')
        need(instructions>100, 'missing user disassembly')
        need(not (directory/(name+'-llvm-nm.stdout.log')).read_text().strip(), 'unresolved user symbol')
    inv = rows(records,'m1.desktop_inventory')[0]
    expected = sum(HEAPS)*MIB + sum((images[i%3]['load_pages']+19)*4096 for i in range(7))
    need(int(inv['committed']) == expected, 'independent ELF/backing reconciliation')
    need(int(inv['allocations']) == sum(images[i%3]['load_segments']+5 for i in range(7)), 'allocation reconciliation')
    memory=rows(records,'m1.runtime_memory')[0]
    per_role_fixed=(int(memory['metadata_bytes'])+int(memory['table_storage_bytes']))//16+65536
    need(all((image['load_pages']+19)*4096+per_role_fixed<=2*MIB for image in images),'image/BSS/stacks/table overhead envelope')
    mutations = 0
    if test:
        # Mutate every field whose value closes an acceptance criterion. The
        # independent checker must reject omissions, surplus owners and drift.
        for index, record in enumerate(records):
            if not record['event'].startswith('m1.desktop_'):
                continue
            for field in record['fields']:
                if field in ['version','build','session','session_origin','seq','stage','outcome','emergency']:
                    continue
                candidate = copy.deepcopy(records)
                candidate[index]['fields'][field] = '999999999'
                try:
                    check(candidate,True)
                except (ValueError,KeyError):
                    mutations += 1
                else:
                    # Some observations are informational, never converted to
                    # an asserted criterion by counting them as a negative.
                    pass
        for token in ['desktop-check-0000000001', 'unexpected-desktop-diagnostic']:
            candidate = copy.deepcopy(records)
            extra = copy.deepcopy(next(r for r in candidate if r['event'] == 'm1.user_log'))
            extra['fields']['token'] = token
            extra['fields']['total'] = str(len(token))
            candidate.insert(-2, extra)
            for sequence, record in enumerate(candidate):
                record['fields']['seq'] = str(sequence)
            try:
                check(candidate, True)
            except ValueError:
                mutations += 1
            else:
                raise ValueError('accepted extra assertion/diagnostic')
        need(mutations >= 120, 'insufficient reviewer negatives')
    return {'schema_version':1,'story':'P4-02','status':'passed','review':'local Codex self-review with separate raw/ELF oracle; not external approval','test':test,'build_id':manifest['build_id'],'kernel_sha256':manifest['kernel_sha256'],'iso_sha256':image['iso_sha256'],'serial_sha256':digest(run/'serial.log'),'images':images,'inventory':inv,'settled':rows(records,'m1.desktop_settled'),'reviewer_negatives':mutations,'source_hashes_verified':verify_source}

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--run', required=True)
    parser.add_argument('--ordinary', action='store_true')
    parser.add_argument('--development', action='store_true')
    args = parser.parse_args()
    report = review(args.run, not args.ordinary, not args.development)
    Path(args.run, 'p4-02-review.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))
