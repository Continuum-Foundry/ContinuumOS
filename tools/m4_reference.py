"""Independent P4 host-only profile/byte consumer; does not import the producer."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def reject_unless(value, reason):
    if not value:
        raise ValueError(reason)


def pairs(entries):
    result = {}
    for key, value in entries:
        reject_unless(key not in result, 'duplicate JSON key')
        result[key] = value
    return result


def load(path):
    with Path(path).open('rb') as stream:
        data = stream.read(65537)
    reject_unless(len(data) <= 65536, 'profile too large')
    try:
        return json.loads(data, object_pairs_hook=pairs)
    except RecursionError as error:
        raise ValueError('profile nesting') from error


def profile(value):
    reject_unless(value['schema_version'] == 1 and value['story'] == 'P4-01', 'profile identity')
    lock = load(ROOT / 'config/m1-inputs.lock.json')['vm']
    cpu = lock['arguments'][lock['arguments'].index('-cpu') + 1]
    reject_unless(value['vm'] == dict(machine=lock['machine'], cpu=cpu, accelerator='tcg', vcpus=1, ram_mib=2048, hpet=True, nic='none', serial='diagnostics-only'), 'VM baseline')
    reject_unless(value['geometry'] == [[w, h, scale] for w, h in ((1280,720),(1920,1080)) for scale in (100,150,200)], 'geometry cells')
    reject_unless(value['effects'] == ['rich','reduced'] and len(value['workloads']) == 6, 'quality/workload coverage')
    measured = value['measurement']
    reject_unless(measured['repetitions'] * measured['frames_per_cell_per_repetition'] == 1000 and measured['repetitions'] == 5, 'sample counts')
    reject_unless(measured['warmup_frames'] == 50 and measured['warmup_ms'] == 5000 and measured['warmup_reloads'] == 5, 'warmups')
    reject_unless(measured['compatible_reloads_per_class_per_baseline'] == 100 and measured['fault_repetitions'] == 5 and measured['seed'] == 20261106, 'stress/fault counts')
    reject_unless(sum(measured['idle_phases_ms']) == measured['idle_ms'] == 30000 and measured['idle_ms'] // measured['observer_ms'] == value['idle']['observer_samples_per_30s'], 'idle intervals')
    reject_unless(measured['host_clock'] == 'perf_counter_ns' and measured['guest_clock'] == 'elapsed-hpet-ns' and measured['cross_domain_subtraction'] is False and measured['visible_observer_required'] is True, 'clock/visibility rules')
    reject_unless(value['targets'] == dict(cold_idle_bytes=256*1024**2, render_composition_p95_ns=8000000, render_composition_p99_ns=16000000, input_submit_p95_ns=25000000, build_visible_ns=500000000, refresh_hz=60), 'unchanged targets')
    settle = value['settlement']
    reject_unless(settle['quanta'] * settle['quantum_ns'] == settle['fairness_ns'] == settle['cleanup_ns'] == 320000000, 'fairness/cleanup bounds')
    reject_unless(all(settle[k] == 0 for k in ('unexplained_bytes','frames','objects','handles','maps','leases','retired_roots','hidden_launches','pixel_tolerance')), 'growth tolerances')
    roles = value['proposed_runtime']
    reject_unless(roles['implemented'] is False and roles['threads_per_process'] == 1 and roles['steady_roles'] == len(roles['heap_caps_mib']) == 7, 'proposed capacity status')
    reject_unless(1 + (roles['steady_roles'] - 1) * (1 + roles['retries_per_child']) <= roles['lifetime_units'] == 24 and roles['processes'] == 16 and roles['refund_lifetime_credits'] is False and roles['reload_process_launches'] == 0, 'lifetime budget')
    reject_unless(sum(roles['heap_caps_mib'].values()) == 168 and roles['active_staging_commit_bytes'] > 256*1024**2, 'ceilings distinct from cold target')
    b = value['bounds']
    reject_unless(b['surface_stride'] * b['surface_height'] == b['surface_bytes'] and b['surface_width'] * 4 <= b['surface_stride'], 'surface arithmetic')
    reject_unless(b['desktop_message_bytes'] == 4096 and b['dev_chunk_bytes'] <= b['dev_frame_bytes'] <= b['blob_bytes'] <= b['transaction_bytes'], 'transport bounds')
    reject_unless(value['devices']['dev_disabled_omits'] == ['controller','port','chardev','mutation-grant'] and value['devices']['diagnostic_serial_separate'] is True and value['devices']['iommu_claim'] is False, 'device trust/disabled plan')
    stress = load(ROOT / 'config/p3-13-stress.json')
    reject_unless(value['runner']['inherited_guest_ms'] == 30000 and value['runner']['stress_guest_ms'] == stress['guest_deadline_ms'] == 60000
                  and stress['deadline_profile'] == 'm3-stress-tcg-v2', 'scoped stress deadline')
    canonical = json.dumps(value, sort_keys=True, separators=(',', ':')).encode()
    reject_unless(hashlib.sha256(canonical).hexdigest() == '333b7fa2170877508d737d9f0fb97fe190a082fc5562fea6f016484798704f1a', 'unreviewed profile fields/types')
    return True


def integer(raw, at, size, signed=False):
    reject_unless(at + size <= len(raw), 'truncated integer')
    return int.from_bytes(raw[at:at+size], 'little', signed=signed)


def desktop(family, raw, sequence=1, resource=3, output=4):
    reject_unless(isinstance(raw, bytes) and 64 <= len(raw) <= 4096 and raw[:8] == b'CDU0\0\0\0\0', 'desktop header')
    reject_unless(integer(raw,8,4) == len(raw) and integer(raw,12,2) == 1 and integer(raw,14,2) == integer(raw,60,4) == 0, 'desktop version/length/flags')
    reject_unless(integer(raw,16,8) == sequence > 0 and integer(raw,32,8) == resource > 0 and integer(raw,40,8) == output > 0 and integer(raw,24,8) > 0 and integer(raw,48,8) > 0, 'desktop context')
    reject_unless(integer(raw,56,4) == len(raw) - 64, 'body length')
    layouts = {'surface': [8,8,4,4,4,4,8,4,4,4,4,8], 'input':[8,8,8,4,4,-4,-4,4,4], 'semantic':[8,8,8,-4,-4,4,4,4,4,4,4], 'output':[8,8,4,4,4,4,-4,-4]}
    reject_unless(family in layouts, 'family')
    values=[]; offset=64
    for width in layouts[family]:
        values.append(integer(raw,offset,abs(width),width<0)); offset += abs(width)
    tail=raw[offset:]
    reject_unless(family == 'semantic' or not tail, 'unexpected tail')
    if family == 'surface':
        lease,scene,w,h,stride,fmt,size,x,y,dw,dh,generation=values
        reject_unless(min(lease,scene,generation)>0 and 1<=w<=1920 and 1<=h<=1080 and fmt==1, 'surface type/identity')
        reject_unless(stride%4==0 and w*4<=stride<=8192 and size==stride*h<=8847360 and x+dw<=w and y+dh<=h, 'surface range')
    elif family == 'input':
        stamp,scene,focus,device,kind,x,y,code,mods=values
        reject_unless(min(stamp,scene,focus,device)>0 and kind in (1,2,3) and code<65536 and mods<256, 'input fields')
    elif family == 'semantic':
        node,parent,scene,x,y,w,h,role,state,actions,length=values
        reject_unless(node>0 and node!=parent and scene>0 and w<=1920*65536 and h<=1080*65536 and role in range(1,9) and state<65536 and actions<65536, 'semantic fields')
        reject_unless(length==len(tail)<=256 and 0 not in tail, 'semantic label')
        try: tail.decode('utf-8')
        except UnicodeError as error: raise ValueError('semantic encoding') from error
    else:
        identity,topology,w,h,num,den,x,y=values
        reject_unless(identity>0 and topology==1 and (w,h) in ((1280,720),(1920,1080)) and (num,den) in ((1,1),(3,2),(2,1)) and x==y==0, 'output fields')
    return tuple(values), tail


def dev(raw, sequence=1, development=False, authenticated=False):
    reject_unless(isinstance(raw,bytes) and 32<=len(raw)<=1048608 and raw[:8]==b'OSU1\x01\0\0\0', 'dev header')
    reject_unless(integer(raw,8,4)==32 and integer(raw,12,4)==len(raw)-32<=1048576 and integer(raw,16,8)==sequence>0 and integer(raw,26,2)==integer(raw,28,4)==0, 'dev bounds/flags/context')
    kind=integer(raw,24,2)
    reject_unless(1<=kind<=12 and development is True and (kind<3 or authenticated is True), 'dev authority model')
    return kind,raw[32:]
