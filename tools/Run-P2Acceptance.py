"""Bounded P2-13 orchestrator. Full runs stay local; no network operations."""
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]


def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))


def save(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def require(ok, why):
    if not ok:
        raise ValueError(why)


def snapshot():
    paths = subprocess.check_output(['git', 'ls-files', '--cached', '--others', '--exclude-standard'], cwd=ROOT).decode().splitlines()
    return [{'path': p, 'sha256': digest(ROOT / p)} for p in sorted(set(paths))
            if (ROOT / p).is_file() and (p.startswith(('crates/', 'tools/', 'experiments/', 'kernel/', 'config/', 'fixtures/', 'legal/', 'licenses/', '.cargo/', 'docs/06-contracts/schemas/'))
                                        or (p.startswith('audit/') and Path(p).suffix in ('.cjs', '.ps1'))
                                        or p in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml'))]


def validate_index(record, protocol, run_id, artifact):
    require(record['run_id'] == run_id, 'stale run')
    require(record['profile'] == protocol['host_profile'], 'profile mismatch')
    require(record['artifact'] == artifact, 'build mismatch')
    require(len(record['rows']) == len(protocol['required_rows']), 'missing/duplicate row')
    require({r['name'] for r in record['rows']} == set(protocol['required_rows']), 'wrong matrix')
    require(all(r['status'] == 'passed' and r['evidence'] for r in record['rows']), 'failed/skipped/unbacked row')
    require(all(Path(p).exists() for r in record['rows'] for p in r['evidence']), 'missing evidence output')
    require(record['cleanup'] == 'reaped', 'unreaped helper')


class Runner:
    def __init__(self, run, protocol):
        self.run, self.protocol, self.commands = run, protocol, []

    def step(self, name, args, expected=None, timeout=None, env=None):
        start = time.monotonic()
        log = self.run / (name + '.log')
        row = dict(name=name, arguments=list(map(str, args)), timeout_seconds=timeout or self.protocol['step_timeout_seconds'])
        with log.open('w', encoding='utf-8') as stream:
            process = subprocess.Popen(row['arguments'], cwd=ROOT, env=env, stdout=stream, stderr=subprocess.STDOUT,
                                       creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
            row['pid'] = process.pid
            try:
                code = process.wait(timeout=row['timeout_seconds'])
                row.update(exit_code=code, timed_out=False)
            except subprocess.TimeoutExpired:
                # Terminate only this owned process and its descendants.
                if os.name == 'nt':
                    killed = subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'], capture_output=True, timeout=30)
                    row['tree_kill_exit_code'] = killed.returncode
                else:
                    process.kill()
                process.wait(timeout=30)
                row.update(exit_code=process.returncode, timed_out=True)
        row.update(elapsed_seconds=time.monotonic() - start, cleanup='reaped', log_sha256=digest(log))
        output = log.read_text(encoding='utf-8-sig', errors='replace')
        row['passed'] = not row['timed_out'] and ((row['exit_code'] == 0) if expected is None else (row['exit_code'] != 0 and expected in output))
        self.commands.append(row)
        save(self.run / 'commands.json', self.commands)
        require(row['passed'], 'step failed: ' + name)
        print('Completed: ' + name, flush=True)
        return output


def output_directory(output, marker):
    matches = re.findall(r'^' + re.escape(marker) + r'(.+)$', output, re.MULTILINE)
    require(len(matches) == 1, 'missing/ambiguous output: ' + marker)
    path = Path(matches[0].strip()).resolve()
    require(path.is_relative_to(ROOT / 'build'), 'output outside build/')
    return path


def review_selection(inputs):
    selection = read(ROOT / 'legal/ui-selection.json')
    require(selection['decision'] == 'provisional-custom' and not selection['distribution_cleared'], 'unreviewed selection')
    prior = {p['name']: p for p in selection['packages']}
    current = {p['name']: p for p in inputs['candidates']['custom']['packages']}
    require(prior.keys() == current.keys(), 'changed selected closure')
    for name, old in prior.items():
        for key in ('version', 'normal_features', 'build_features', 'registry_checksum', 'license'):
            require(old[key] == current[name].get(key), 'changed dependency: ' + name + '/' + key)
        for notice in old['notices']:
            require(digest(ROOT / notice['path']) == notice['sha256'], 'notice changed')
    for row in selection['frozen_comparison'] + selection['input_pins'] + selection['additional_notices']:
        raw = (ROOT / row['path']).read_bytes()
        if row.get('normalization') == 'lf':
            raw = raw.replace(b'\r\n', b'\n')
        require(hashlib.sha256(raw).hexdigest() == row['sha256'], 'frozen input changed: ' + row['path'])
    manifests = []
    for story, day in [('P2-06', '2026-10-21'), ('P2-07', '2026-10-22'), ('P2-08', '2026-10-22'), ('P2-09', '2026-10-23')]:
        directory = ROOT / 'evidence/phase-2' / story / day
        manifest = read(directory / 'SHA256SUMS.json')
        for row in manifest['files']:
            require(digest(directory / row['path']) == row['sha256'], 'historical evidence changed')
        manifests.append(dict(story=story, files=len(manifest['files']), sha256=digest(directory / 'SHA256SUMS.json')))
    # The P2-09 chosen-source pins are historical. Final source is separately
    # hashed and linked here; never rewrite the old decision's evidence.
    return dict(status='passed', packages=len(current), preserved_evidence=manifests,
                chosen_source='P2-09 historical; final source separately rebuilt', distribution_cleared=False,
                axes=['integration', 'quality', 'text-accessibility', 'reload', 'efficiency', 'authoring', 'evolution', 'distribution'])


def main():
    protocol = read(ROOT / 'config/p2-13-acceptance.json')
    run = ROOT / 'build/m2/P2-13' / ('acceptance-' + str(time.time_ns()))
    run.mkdir(parents=True)
    print('P2-13 run: ' + str(run), flush=True)
    save(run / 'protocol.json', protocol)
    before = snapshot()
    save(run / 'tested-inputs.json', before)
    runner = Runner(run, protocol)
    ps = ['pwsh', '-NoProfile', '-File']
    try:
        negative_run = run / 'negative-processes'
        negative_run.mkdir()
        negative_runner = Runner(negative_run, protocol)
        for name, code, timeout in [('early-exit', 'raise SystemExit(7)', 10), ('timeout', 'import time; time.sleep(30)', 1)]:
            try:
                negative_runner.step(name, [sys.executable, '-c', code], timeout=timeout)
            except ValueError:
                require(negative_runner.commands[-1]['cleanup'] == 'reaped', 'negative process not reaped')
            else:
                raise ValueError('accepted process failure: ' + name)
        pre = output_directory(runner.step('prerequisites', ps + ['tools/Recheck-P2Prerequisites.ps1']), 'Prerequisite evidence: ')
        prereq = read(pre / 'prerequisites.json')
        require(prereq['status'] == 'passed' and len(prereq['commands']) == 10 and all(r['exit_code'] == 0 for r in prereq['commands']), 'prerequisite coverage')
        # Existing signer permits host artifacts only below build/cargo/.
        # A unique absent subdirectory still supplies a genuinely clean build.
        target = ROOT / 'build/cargo' / ('p2-13-' + run.name)
        perf = output_directory(runner.step('selected-acceptance', ps + ['tools/Test-P2-12.ps1', '-Prerequisites', str(pre), '-TargetDirectory', str(target)],
                                            timeout=protocol['performance_timeout_seconds']), 'P2-12 acceptance: ')
        acceptance = read(perf / 'acceptance.json')
        exe = target / 'm2-comparison/xtask.exe'
        artifact = digest(exe)
        require(artifact == acceptance['signed_executable_sha256'], 'wrong final host artifact')
        for native in ('native-reload', 'native-idle'):
            preview = read(Path(acceptance['outputs'][native]) / 'native-preview.json')
            require(preview['executable_sha256'] == artifact, 'native replay used a different executable')
        outputs = {}
        for name, marker in [('ui-fixtures', 'Shared UI fixture checks passed: '), ('editor-model', 'Document action checks passed: '), ('text', 'Text conformance checks passed: ')]:
            paths = []
            for repetition in range(2):
                directory = output_directory(runner.step(name + '-' + str(repetition), [str(exe), 'test', '--suite', name]), marker)
                require(read(directory / 'result.json')['status'] == 'passed', 'suite did not pass')
                paths.append(directory)
            require(digest(paths[0] / 'result.json') == digest(paths[1] / 'result.json'), 'nondeterministic suite: ' + name)
            outputs[name] = str(paths[0])
        runner.step('text-oracle', ['python', 'tools/Reference-TextShape.py', '--verify', 'fixtures/m2/text-v1/shaping-reference.json'])
        editor = read(Path(outputs['editor-model']) / 'result.json')
        require(len(editor['entries']) == 4 and len(editor['negative_cases']) == 56, 'action coverage')
        for entry in editor['entries']:
            runner.step('checkpoint-' + entry['entry'], ps + ['tools/Test-EditorCheckpoint.ps1', '-Path', str(Path(outputs['editor-model']) / (entry['entry'] + '.checkpoint')), '-Fixture', 'fixtures/m2/actions-v1/replay.json'])
        text = read(Path(outputs['text']) / 'result.json')
        require((len(text['observations']), len(text['composition']), len(text['negative'])) == (45, 6, 14), 'text coverage')
        for case, expected in [('success', 'exited-zero'), ('nonzero', 'exited-nonzero'), ('timeout', 'timeout')]:
            directory = output_directory(runner.step('lifecycle-' + case, [str(exe), 'hosted', '--config', 'config/m2-profiles.json', '--profile', 'hd', '--scale', '1.0', '--case', case],
                                                    expected=None if case == 'success' else 'Hosted lifecycle evidence: '), 'Hosted lifecycle evidence: ')
            result = read(directory / 'result.json')
            require(result['outcome'] == expected and result['cleanup'] == 'reaped' and result['executable_sha256'] == artifact, 'lifecycle result mismatch')
        env = os.environ.copy()
        env['CARGO_TARGET_DIR'] = str(run / 'link-target')
        env['CONTINUUM_FONT_DIR'] = str(ROOT / 'work/text-inputs')
        env['CARGO_TARGET_X86_64_UNKNOWN_NONE_RUSTFLAGS'] = '--emit=dep-info,llvm-ir,link'
        require(not any(os.environ.get(k) for k in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'CARGO_TARGET_X86_64_UNKNOWN_NONE_RUSTFLAGS')), 'inherited Rust flags')
        runner.step('probe-format', ['cargo', 'fmt', '--manifest-path', 'experiments/ui-link/Cargo.toml', '--check'])
        runner.step('link-inputs', ['python', 'tools/Review-UiLinkInputs.py', str(run / 'inputs')], env=env)
        selection = review_selection(read(run / 'inputs/inputs.json'))
        save(run / 'selection-review.json', selection)
        base = ['cargo', 'rustc', '--locked', '--offline', '--manifest-path', 'experiments/ui-link/Cargo.toml', '--target', protocol['target'], '--release', '--bin', 'ui-link']
        link = ['--', '-C', 'code-model=small', '-C', 'relocation-model=static', '-C', 'link-arg=-T' + str(ROOT / 'experiments/ui-link/link.ld'), '-C', 'link-arg=--no-undefined']
        runner.step('selected-link', base + link, env=env)
        elf = run / 'link-target/x86_64-unknown-none/release/ui-link'
        sysroot = subprocess.check_output(['rustc', '--print', 'sysroot'], cwd=ROOT).decode().strip()
        llvm = Path(sysroot) / 'lib/rustlib/x86_64-pc-windows-msvc/bin'
        runner.step('link-audit', ['python', 'tools/Review-UiLink.py', str(elf), str(run / 'elf-audit'), '--llvm', str(llvm)])
        audit = read(run / 'elf-audit/audit.json')
        for symbol in ('custom_ui::retained::Frame::draw', 'text_core::Engine::layout_cached'):
            require(audit['symbol_excerpts'][symbol], 'final selected path discarded: ' + symbol)
        for feature, reason in [('missing-hook', 'undefined symbol: p2_missing_present_hook'), ('host-feature', "can't find crate for `std`")]:
            runner.step(feature, base + ['--features', feature] + link, expected=reason, env=env)
        for name, args in [('docs', ['node', 'audit/check-docs.cjs']), ('governance', ['node', 'audit/check-governance.cjs']), ('contracts', ps + ['audit/validate-contracts.ps1'])]:
            runner.step(name, args)
        require(snapshot() == before, 'source/input set changed during acceptance')
        require(digest(exe) == artifact, 'host artifact changed during acceptance')
        evidence = {
            'prerequisites-clean-artifacts': [str(pre), str(perf), str(run / 'elf-audit/audit.json')],
            'comparison-selection': [str(run / 'selection-review.json')],
            'renderer-fixtures': [acceptance['outputs']['corpus']],
            'keyboard-pointer': [acceptance['outputs']['selected']],
            'text': [outputs['text']], 'semantics-readability': [acceptance['outputs']['selected']],
            'typed-actions-context': [outputs['editor-model']],
            **{name: [acceptance['outputs']['reload'], acceptance['outputs']['native-reload']] for name in ('token-reload', 'layout-reload', 'failed-update', 'reader-retirement')},
            'repeated-reload': [acceptance['outputs']['stability']],
            'damage-idle': [str(perf / 'review/damage-review.json'), str(perf / 'review/native-idle.json')],
            'metrics': [str(perf / 'review/performance-review.json'), str(perf / 'review/metrics.json')],
            'harness-cleanup': [str(run / 'commands.json')], 'documentation-regressions': [str(run / 'commands.json')]
        }
        record = dict(schema_version=1, status='passed', run_id=run.name, profile=protocol['host_profile'], artifact=artifact,
                      source_base=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip(),
                      rows=[dict(name=name, status='passed', evidence=evidence[name]) for name in protocol['required_rows']],
                      cleanup='reaped', performance=str(perf), outputs=outputs, link_sha256=audit['sha256'], guest=False)
        validate_index(record, protocol, run.name, artifact)
        negatives = []
        for name, mutate in [('stale', lambda r: r.update(run_id='stale')), ('build', lambda r: r.update(artifact='0' * 64)),
                             ('profile', lambda r: r.update(profile='debug')), ('missing', lambda r: r['rows'].pop()),
                             ('skipped', lambda r: r['rows'][0].update(status='skipped')), ('unreaped', lambda r: r.update(cleanup='alive')),
                             ('missing-result', lambda r: r['rows'][0].update(evidence=[str(run / 'nonexistent-result.json')]))]:
            bad = copy.deepcopy(record)
            mutate(bad)
            try:
                validate_index(bad, protocol, run.name, artifact)
            except ValueError as error:
                negatives.append(dict(case=name, rejected=True, reason=str(error)))
            else:
                raise ValueError('accepted harness mutation: ' + name)
        save(run / 'harness-negatives.json', negatives)
        save(run / 'acceptance.json', record)
        print('P2-13 acceptance: ' + str(run), flush=True)
    except Exception as error:
        save(run / 'failure.json', dict(status='failed', reason=str(error), commands_completed=len(runner.commands)))
        raise


if __name__ == '__main__':
    main()
