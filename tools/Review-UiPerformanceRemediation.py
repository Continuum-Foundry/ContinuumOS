"""Independent consumer for the fixed single-view diagnostic and repeated tails.

Elapsed distributions are not renderer-thread CPU or guest performance results.
"""
import collections
import json
import math
from pathlib import Path
import sys

from importlib.machinery import SourceFileLoader

base = SourceFileLoader('p2acceptance', str(Path(__file__).with_name('Run-P2Acceptance.py'))).load_module()
read, save, require, digest = base.read, base.save, base.require, base.digest


def key(row):
    return (*row['geometry'], row['reduced'], row['trace'], row['workload'])


def distribution(values):
    values = sorted(values)
    require(bool(values), 'empty distribution')
    return {f'p{q}': values[math.ceil(len(values) * q / 100) - 1] for q in (50, 95, 99)}


def consume(directory, mode, executable, protocol):
    result = read(directory / 'result.json')
    require(result['status'] == 'passed' and result['mode'] == mode, 'profile status/mode')
    require(result['executable_sha256'] == executable and not result['guest'], 'artifact/guest identity')
    require(read(directory / 'protocol.json') == protocol, 'changed runtime protocol')
    require(result['protocol_sha256'] == digest(directory / 'protocol.json') or
            result['protocol_sha256'] == digest(base.ROOT / 'config/ui-performance-remediation.json'), 'protocol bytes')
    geometry = protocol['geometry'] if mode == 'diagnostic' else [protocol['tail_primary_geometry']]
    expected = {(*g, reduced, trace, workload) for g in geometry for reduced in (False, True)
                for trace in (False, True) for workload in range(len(protocol['workloads']))}
    summaries = {key(r): r for r in result['cells']}
    require(len(summaries) == len(result['cells']) and summaries.keys() == expected, 'missing/duplicate summary cell')
    rows = collections.defaultdict(list)
    for line in (directory / 'frames.jsonl').read_text(encoding='utf-8').splitlines():
        row = json.loads(line)
        rows[key(row)].append(row)
    require(rows.keys() == expected, 'missing/extra raw cell')
    count = protocol['diagnostic_frames_per_cell' if mode == 'diagnostic' else 'tail_frames_per_primary_cell']
    for k, values in rows.items():
        require([r['sample'] for r in values] == list(range(count)), 'missing/duplicate/out-of-order samples')
        require(summaries[k]['samples'] == count, 'summary count')
        require(summaries[k]['frame_ns'] == distribution([r['frame_ns'] for r in values]), 'summary quantiles')
        require(all(r['frame_ns'] > 0 and r['paint_scratch_peak'] <= 64 * 1024 * 1024 for r in values), 'frame/scratch bounds')
        require(all(bool(r['rgba_sha256']) == k[4] for r in values), 'trace pixel identity unavailable')
    return summaries, rows


def review(run):
    index = read(run / 'runs.json')
    p = read(run / 'protocol.json')
    before, old_rows = consume(Path(index['baseline']), 'diagnostic', index['baseline_sha256'], p)
    after, new_rows = consume(Path(index['candidate']), 'diagnostic', index['candidate_sha256'], p)
    paired = []
    for k, current in after.items():
        old = before[k]
        require(all(old[n] == current[n] for n in ('final_rgba_sha256', 'checkpoint_sha256')), 'final pixel/state regression')
        if k[4]:
            require([r['rgba_sha256'] for r in old_rows[k]] == [r['rgba_sha256'] for r in new_rows[k]], 'per-frame pixel regression')
        paired.append(dict(geometry=list(k[:3]), reduced=k[3], trace=k[4], workload=k[5], samples=old['samples'],
                           before_frame_ns=old['frame_ns'], after_frame_ns=current['frame_ns'],
                           before_batch_cpu_100ns=old['batch_cpu_100ns'], after_batch_cpu_100ns=current['batch_cpu_100ns'],
                           final_rgba_sha256=current['final_rgba_sha256'], checkpoint_sha256=current['checkpoint_sha256']))
    tails = collections.defaultdict(list)
    repeated = []
    require(len(index['tails']) == p['tail_repetitions'], 'tail repetitions')
    for directory in index['tails']:
        summaries, rows = consume(Path(directory), 'tails', index['candidate_sha256'], p)
        repeated.append(summaries)
        for k, values in rows.items():
            tails[k].extend(values)
    reports = []
    for k, rows in tails.items():
        require(len(rows) == p['tail_repetitions'] * p['tail_frames_per_primary_cell'] >= 1000, 'insufficient primary tail samples')
        require(len({(s[k]['final_rgba_sha256'], s[k]['checkpoint_sha256']) for s in repeated}) == 1, 'nondeterministic repeated pixel/state')
        reports.append(dict(geometry=list(k[:3]), reduced=k[3], trace=k[4], workload=k[5], samples=len(rows),
                            frame_ns=distribution([r['frame_ns'] for r in rows]),
                            input_to_ram_submission_ns=distribution([r['input_to_ram_submission_ns'] for r in rows]),
                            elapsed_60hz_exceedances=sum(r['frame_ns'] > 16666667 for r in rows),
                            input_proxy_25ms_exceedances=sum(r['input_to_ram_submission_ns'] > 25000000 for r in rows),
                            allocations=distribution([r['allocations'] for r in rows]) if k[4] else None,
                            scratch_peak_bytes=max(r['paint_scratch_peak'] for r in rows),
                            work_mean={n: sum(r['work'][n] for r in rows) / len(rows) for n in rows[0]['work']}))
    pairs = []
    for off in reports:
        if off['trace']:
            continue
        on = next(r for r in reports if r['reduced'] == off['reduced'] and r['workload'] == off['workload'] and r['trace'])
        pairs.append(dict(reduced=off['reduced'], workload=off['workload'],
                          trace_on_minus_off_ns={q: on['frame_ns'][q] - off['frame_ns'][q] for q in ('p50', 'p95', 'p99')}))
    probes = [read(Path(d) / 'cpu-clock-probe.json') for d in [index['candidate']] + index['tails']]
    deltas = [r['cpu_delta_100ns'] for probe in probes for r in probe['samples']]
    report = dict(status='passed', diagnostic_cells=len(paired), diagnostic_pixel_pairs=1152,
                  tail_samples=sum(len(r) for r in tails.values()), paired_diagnostic=paired, primary_tails=reports,
                  instrumentation_pairs=pairs, process_clock_probe=dict(samples=len(deltas), zero_deltas=deltas.count(0),
                  smallest_observed_positive_delta_100ns=min((x for x in deltas if x > 0), default=None)),
                  limits=['Single quiet sequential baseline/final diagnostic pair: scheduling and instrumentation differ; no universal speedup claim.',
                          'Baseline allocation counting was not enabled; its zero counts are unavailable and excluded.',
                          'Final instrumentation adds dispatch/CPU-probe/allocation clocks; trace-off pixels/timings and final paired overhead are reported without correction.',
                          'Process CPU is batched; per-frame clocks are elapsed. Text clocks overlap fills and must not be summed.',
                          'Tail CPU and input are host proxies; guest CPU, guest input and whole-guest 256 MiB targets remain unaccepted.'],
                  guest=False)
    save(run / 'profile-review.json', report)
    return report


if __name__ == '__main__':
    result = review(Path(sys.argv[1]).resolve())
    print(f"UI remediation profile review passed: {result['tail_samples']} repeated tail samples")
