"""Final bounded offline regression plus sequential focused profiling.

The baseline is an explicitly preserved pre-optimization signed executable.
Full raw records and binaries stay local under build/.
"""
from importlib.machinery import SourceFileLoader
import os
from pathlib import Path
import sys
import time

base = SourceFileLoader('p2acceptance', str(Path(__file__).with_name('Run-P2Acceptance.py'))).load_module()
ROOT, read, save, digest, require = base.ROOT, base.read, base.save, base.digest, base.require


def main():
    baseline = Path(sys.argv[1]).resolve()
    require(baseline.is_relative_to(ROOT / 'build/cargo') and baseline.is_file(), 'preserved baseline required under build/cargo')
    run = ROOT / 'build/m2/performance-remediation' / ('acceptance-' + str(time.time_ns()))
    run.mkdir(parents=True)
    print('UI remediation run: ' + str(run), flush=True)
    p = read(ROOT / 'config/ui-performance-remediation.json')
    save(run / 'protocol.json', p)
    before = base.snapshot()
    save(run / 'tested-inputs.json', before)
    runner = base.Runner(run, dict(step_timeout_seconds=p['timeout_seconds']))
    ps = ['pwsh', '-NoProfile', '-File']
    try:
        output = runner.step('m1-regression', ps + ['tools/Test-P1-07.ps1'], timeout=5400)
        m1 = base.output_directory(output, 'M1 artifacts: ')
        result = read(m1 / 'result.json')
        require(result['status'] == 'passed' and result['stages']['launch_count'] == 42 and len(result['stages']['rows']) == 33, 'M1 regression matrix')
        hosted = base.output_directory(runner.step('hosted-regression', ps + ['tools/Test-P2-13.ps1'], timeout=5400), 'P2-13 acceptance: ')
        acceptance = read(hosted / 'acceptance.json')
        require(acceptance['status'] == 'passed' and len(acceptance['rows']) == 16, 'hosted regression matrix')
        exe = ROOT / 'build/cargo' / ('p2-13-' + hosted.name) / 'm2-comparison/xtask.exe'
        require(digest(exe) == acceptance['artifact'], 'candidate binary identity')
        index = dict(m1=str(m1), hosted=str(hosted), baseline_sha256=digest(baseline), candidate_sha256=digest(exe),
                     baseline=str(run / 'baseline'), candidate=str(run / 'candidate'), tails=[])
        # No concurrent builds/profilers launched by this runner. Host scheduling
        # remains uncontrolled; report each fixed cell and all misses.
        for label, binary in [('baseline', baseline), ('candidate', exe)]:
            runner.step('focused-' + label, [str(binary), 'ui-focused', 'diagnostic', index[label]])
        for repetition in range(p['tail_repetitions']):
            directory = run / ('tails-' + str(repetition))
            runner.step('tails-' + str(repetition), [str(exe), 'ui-focused', 'tails', str(directory)])
            index['tails'].append(str(directory))
        save(run / 'runs.json', index)
        runner.step('profile-review', [sys.executable, 'tools/Review-UiPerformanceRemediation.py', str(run)])
        require(base.snapshot() == before, 'source/input set changed during final run')
        require(digest(exe) == index['candidate_sha256'] and digest(baseline) == index['baseline_sha256'], 'profile binary changed')
        save(run / 'acceptance.json', dict(status='passed', runs=index, cleanup='reaped', source_guard='matching', guest=False))
        print('UI remediation acceptance: ' + str(run), flush=True)
    except Exception as error:
        save(run / 'failure.json', dict(status='failed', reason=str(error), commands_completed=len(runner.commands)))
        raise


if __name__ == '__main__':
    main()
