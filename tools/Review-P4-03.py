"""Independent raw PCI/queue/IRQ/lifetime and CPL3 grant oracle."""
import argparse
import copy
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('p402_device_base', ROOT/'tools/Review-P4-02.py')
runtime = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runtime)
need, read, digest, rows = runtime.need, runtime.read, runtime.digest, runtime.rows


def check(records):
    runtime.check(records, True)
    admission=rows(records,'m1.device_admission')
    need(len(admission)==1 and all(admission[0][k]==v for k,v in dict(devices='3',bytes='98304',fourth_denied='true',failed_backing_released='true',active_dma='false').items())
         and admission[0]['before_frames']==admission[0]['after_frames'], 'actual admission exhaustion/unwind')
    discovery = rows(records, 'm1.device_discovered')
    need(len(discovery) == 3, 'device count')
    for slot, row in enumerate(discovery):
        need(int(row['slot']) == slot and int(row['bdf']) == (4+slot)*8, 'root bus/device binding')
        need(int(row['id']) == (0x1052 if slot < 2 else 0x1043), 'modern device ID')
        base, size = int(row['base']), int(row['bytes'])
        need(base > 0 and base % 4096 == 0 and 4096 <= size <= 65536
             and size & (size-1) == 0 and base+size <= 2**52, 'BAR bounds')
        need(int(row['irq']) == 20+slot and row['uc'] == 'true' and row['user'] == 'false'
             and row['iommu'] == 'false', 'mapping/IRQ/trust policy')
        for old in discovery[:slot]:
            need(base >= int(old['base'])+int(old['bytes']) or int(old['base']) >= base+size, 'BAR overlap')
    authority = rows(records, 'm1.device_authority')
    feedback = rows(records, 'm1.device_feedback')
    need([(r['slot'],r['led_advertised'],r['kind'],r['unsupported_led_denied'],r['semantic_claim']) for r in feedback]
         == [('0','true','17','false','false'),('1','false','0','true','false'),('0','true','17','false','false')], 'advertised feedback / tablet rejection')
    bars = rows(records, 'm1.device_bar')
    need(len(bars) == 3, 'raw BAR values')
    for slot, bar in enumerate(bars):
        need(int(bar['slot']) == slot and int(bar['low']) & 7 == 4
             and (int(bar['high']) * 2**32 + (int(bar['low']) & ~15)) == int(discovery[slot]['base']), '64-bit BAR decode')
    need(len(authority) == 3 and [r['slot'] for r in authority] == ['0','1','2'], 'authority cases')
    for row in authority:
        need(all(row[k] == 'true' for k in ['forged','stale','overlap','over_rights','duplicate_irq','cancelled','all_denied']), 'authority negative')
    feature = rows(records, 'm1.device_features')
    need([r['slot'] for r in feature] == ['0','0','1','1','2','0'], 'negotiation phases')
    need(all(int(r['offered']) & 2**32 and int(r['selected']) == 2**32
             and r['required_unknown_denied'] == 'true' for r in feature), 'feature subset')
    queues = rows(records, 'm1.device_queue')
    need([(r['slot'],r['queue']) for r in queues] == [(str(s),str(q)) for s in [0,0,1,1,0] for q in [0,1]], 'queue setup matrix')
    for row in queues:
        base, q = int(row['base']), int(row['queue'])
        need(int(row['size']) == 8 and int(row['bytes']) == 32768 and base > 0 and base % 4096 == 0, 'DMA admission')
        need([int(row[k]) for k in ['desc','avail','used','payload']] == [base+q*16384+off for off in [0,4096,8192,12288]], 'split ring addresses')
        need(int(row['notify_offset']) % 2 == 0, 'notify alignment')
        need(all(base+32768 <= int(r['base']) or int(r['base'])+int(r['bytes']) <= base for r in discovery), 'DMA/MMIO overlap')
    used = rows(records, 'm1.device_used')
    need([(r['slot'],r['irq_expected'],r['lost_irq_poll']) for r in used] == [(str(s),irq,lost) for s in [0,1] for irq,lost in [('true','false'),('false','true')]], 'IRQ/missing-IRQ matrix')
    need(all(r['entries'] == '4' and r['malformed_denied'] == 'true' and r['chain_negatives'] == '5' and r['irq_latch'] == 'true'
             and (int(r['irqs']) > 0 if r['irq_expected'] == 'true' else r['irqs'] == '0') for r in used), 'real queue/IRQ delivery')
    entries = rows(records, 'm1.device_used_entry')
    need(len(entries) == 16, 'used-entry count')
    for start in range(0, 16, 4):
        batch = entries[start:start+4]
        need(sorted(int(r['head']) for r in batch) == list(range(4)), 'used duplicate/missing head')
        need(all(r['queue'] == '1' and r['capacity'] == '8' and 0 <= int(r['bytes']) <= 8 for r in batch), 'used length bounds')
    quarantine = rows(records, 'm1.device_quarantine')
    need(len(quarantine) == 2 and [r['slot'] for r in quarantine] == ['0','1'], 'quarantine count')
    need(all(r['devices'] == '1' and r['bytes'] == '32768' and r['cap_bytes'] == '16777216'
             and r['cap_devices'] == '3' and r['released'] == 'false' for r in quarantine), 'bounded retained DMA')
    resets = rows(records, 'm1.device_reset')
    need(len(resets) == 14, 'reset phases')
    failed = [r for r in resets if r['complete'] == 'false']
    need(len(failed) == 2 and all(r['retained'] == 'true' and r['timeout_ns'] == '100000000' for r in failed), 'reset timeout ownership')
    need(all(r['retained'] == 'false' and r['status'] == '0' and r['masked'] == 'true' for r in resets if r['complete'] == 'true'), 'actual reset completion')
    ready = rows(records, 'm1.device_ready')
    need(len(ready) == 1 and all(ready[0][k] == v for k,v in dict(devices='3',queue_setups='8',used_entries='16',quarantine_probes='2',retained_bytes='0',iommu='false').items()), 'pre-user device settlement')
    baseline = ready[0]['frames']
    settled = rows(records, 'm1.device_settled')
    need(len(settled) == 4 and [(r['slot'],r['masked']) for r in settled] == [('0','false'),('0','true'),('1','false'),('1','true')], 'reset-under-load matrix')
    need(all(r['retained_bytes'] == '0' and r['frames'] == baseline and r['reset_before_release'] == 'true' for r in settled), 'exact physical reset settlement')
    # Independently derive each expected CPL3 call from trusted launch identity.
    calls = rows(records, 'm1.device_call')
    all_expected = []
    aborted = {(r['role'],r['process'],r['generation']) for r in rows(records,'m1.desktop_reap') if r['result']=='513'}
    need(len(aborted)==3, 'pre-entry transactional rollback coverage')
    for launch in rows(records, 'm1.desktop_launch'):
        role = runtime.ROLES.index(launch['role'])
        owner = (str(role), str(int(launch['process'])+1), launch['generation'])
        actual = [r for r in calls if (r['role'],r['process'],r['generation']) == owner]
        if (launch['role'],launch['process'],launch['generation']) in aborted:
            need(not actual, 'rolled-back owner executed device calls')
            continue
        expected = []
        for slot in range(3):
            entitled = role == (2 if slot < 2 else 5)
            if entitled:
                expected += [(0x4000,slot,0,0,False),(0x4001,slot,0,4,False),
                             (0x4001,slot,int(discovery[slot]['bytes']),4,True),
                             (0x4001,slot,1,4,True),(0x4001,slot,2**64-1,1,True),
                             (0x4002,slot,1,0,True),(0x4003,slot,0,0,slot == 0)]
                if slot != 0: expected += [(0x4001,slot,0,4,True)]
            else:
                expected += [(0x4000,slot,0,0,True),(0x4001,slot,0,4,True),(0x4002,slot,1,0,True)]
        expected += [(0x4000,2**64-1,0,0,True)]
        need(all(r['denied'] in ('true','false') for r in actual), 'invalid decision boolean')
        observed = [(int(r['op']),int(r['slot']),int(r['offset']),int(r['width']),r['denied'] == 'true') for r in actual]
        need(observed == expected, 'CPL3 authority outcome/coverage')
        all_expected += actual
    need(len(all_expected) == len(calls) == 104, 'unbound/missing CPL3 caller')
    lease, death, final = [rows(records, e) for e in ['m1.device_lease','m1.device_owner_dead','m1.device_final']]
    need(len(lease) == len(death) == len(final) == 1, 'driver-owner lifetime')
    need(lease[0]['slot'] == '0' and lease[0]['bytes'] == '32768' and lease[0]['outstanding'] == '8' and lease[0]['device_owned'] == 'true', 'published lease')
    need(death[0]['reset_complete'] == 'true' and death[0]['released_bytes'] == '32768' and death[0]['pending_eventq'] == 'true', 'death reset-before-reclaim')
    need(death[0]['eventq_submitted']=='4' and 0<=int(death[0]['eventq_used'])<4, 'actual pending receive buffers at owner death')
    input_launch = next(r for r in rows(records,'m1.desktop_launch') if r['role'] == 'input' and (r['role'],r['process'],r['generation']) not in aborted)
    need(int(death[0]['process']) == int(input_launch['process'])+1 and death[0]['generation'] == input_launch['generation'], 'death process binding')
    need(all(final[0][k] == v for k,v in dict(grants='0',dma_bytes='0',leases='0',frames=baseline).items()), 'final inventory')
    need(int(lease[0]['seq']) < int(death[0]['seq']) < int(final[0]['seq']), 'death ordering')
    for r in quarantine:
        after = [s for s in resets if s['slot'] == r['slot'] and s['complete'] == 'true' and int(s['seq']) > int(r['seq'])]
        release = next(s for s in settled if s['slot'] == r['slot'] and s['masked'] == 'true')
        need(after and int(after[0]['seq']) < int(release['seq']), 'quarantine reset fence')


def review(directory):
    directory = Path(directory)
    run, package = directory/'devices-run', directory/'devices-package'
    case, image, manifest, inputs = [read(p) for p in [run/'case-result.json',package/'iso-manifest.json',directory/'build-manifest.json',directory/'build-inputs.json']]
    need(read(directory/'result.json')['status'] == case['status'] == 'passed'
         and case['cleanup_reaped'] and not case['timed_out'] and case['deadline_ms'] == 30000
         and case['emulator_test_exit'] == 33, 'device guest execution')
    need(inputs['profile'] == 'kernel-stress' and manifest['flavor'] == 'device-test', 'optimized device flavor')
    command = read(directory/'kernel-build.command.json')['arguments']
    need(command[command.index('--features')+1] == 'device-test' and command[command.index('--profile')+1] == 'kernel-stress', 'actual feature/profile')
    need(digest(directory/'kernel.elf') == manifest['kernel_sha256'] == image['kernel_sha256'], 'kernel bytes')
    need(digest(package/'continuum.iso') == image['iso_sha256'] == case['iso_sha256'], 'ISO bytes')
    need(manifest['build_id'] == image['build_id'] == case['build_id'], 'build identity')
    for name, expected in inputs['sources'].items(): need(digest(ROOT/name) == expected, 'stale source: '+name)
    launch = read(run/'launch-manifest.json')
    args = launch['arguments']
    for device in ['virtio-keyboard-pci,disable-legacy=on,addr=4','virtio-tablet-pci,disable-legacy=on,addr=5','virtio-serial-pci,disable-legacy=on,addr=6','virtserialport,chardev=p403,name=org.continuum.dev']:
        need(args.count(device) == 1 and args[args.index(device)-1] == '-device', 'actual device launch matrix')
    need('null,id=p403' in args and 'file:${WORKSPACE}' not in args, 'bounded device backend')
    records = runtime.raw(run/'serial.log', manifest['build_id'], image['host_session_id'])
    check(records)
    images = [runtime.elf(directory/(n+'.elf')) for n in ['user_a','user_b','service']]
    for n in ['user_a','user_b','service']:
        need('device_probe' in read(directory/(n+'-build.command.json'))['arguments'], 'missing independent CPL3 probe')
    mutations = 0
    important = {'m1.device_ready','m1.device_final','m1.device_quarantine','m1.device_used','m1.device_used_entry','m1.device_call','m1.device_owner_dead','m1.device_authority','m1.device_queue','m1.device_features','m1.device_feedback','m1.device_bar','m1.device_admission'}
    for index, record in enumerate(records):
        if record['event'] not in important: continue
        candidate = copy.deepcopy(records); candidate.pop(index)
        try: check(candidate)
        except (ValueError,KeyError,StopIteration): mutations += 1
        else: raise ValueError('accepted missing criterion: '+record['event'])
    result = dict(schema_version=1,story='P4-03',status='passed',review='local Codex self-review; no external certification',
                  build_id=manifest['build_id'],kernel_sha256=manifest['kernel_sha256'],iso_sha256=image['iso_sha256'],
                  serial_sha256=digest(run/'serial.log'),reviewer_negatives=mutations,images=images,
                  device_records=[r for r in records if r['event'].startswith('m1.device_')])
    return result


if __name__ == '__main__':
    p = argparse.ArgumentParser(); p.add_argument('--run', required=True)
    args = p.parse_args(); result = review(args.run)
    (Path(args.run)/'device-review.json').write_text(json.dumps(result,indent=2)+'\n')
    print('P4-03 device review passed')
