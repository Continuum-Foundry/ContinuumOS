"""Bounded final-source M3 gate and independent aggregate evidence review.

Run through Test-P3-14.ps1 in the existing Windows signer/xorriso context.
Full logs, artifacts and patches remain local. No historical-run reuse.
"""
import argparse
import copy
import ctypes
from datetime import datetime, timezone
from ctypes import wintypes
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
CONFIG = ROOT / 'config/p3-14-acceptance.json'

# Each row names independently reviewed evidence, rather than inferring product
# features from a blanket pass string. The final story and config use these IDs.
MATRIX = {
    'current-prerequisites-clean-optical-iso': ['prerequisites.json', 'm1-review.json'],
    'memory-ownership-allocation': ['sdk-review.json:regression.memory_regression'],
    'static-elf-admission': ['sdk-review.json:regression.loader_regression', 'kernel host malformed-image tests'],
    'trusted-launch-child-ceiling': ['sdk-review.json:regression.loader_regression', 'sdk-review.json:regression.supervisor_regression'],
    'user-entry-syscall-boundary': ['raw four-guest boundary regression and instruction audit'],
    'user-fault-containment': ['raw four-guest boundary regression', 'sdk-review.json:runtime.audit'],
    'preemption-cpu-isolation': ['sdk-review.json:regression.scheduler_regression'],
    'handles-memory-objects': ['sdk-review.json:regression.capabilities_regression'],
    'ipc-bytes-transfer': ['sdk-review.json:regression.channels_regression'],
    'caller-provenance-after-delegation': ['sdk-review.json:regression.channels_regression', 'sdk-review.json:regression.policy_regression'],
    'waits-deadlines-cancellation': ['sdk-review.json:regression.waits_regression'],
    'deadlock-backpressure-exhaustion': ['sdk-review.json:regression.waits_regression', 'stress-review.json:criterion_review'],
    'supervision-teardown': ['sdk-review.json:regression.supervisor_regression', 'stress-review.json:public_supervision'],
    'cross-owner-same-user-authority': ['sdk-review.json:regression.policy_regression'],
    'scoped-policy-confused-deputy': ['sdk-review.json:regression.policy_regression'],
    'reserved-privileged-operations': ['sdk-review.json:regression.policy_regression'],
    'brokered-invalidation-restart': ['sdk-review.json:regression.policy_regression', 'sdk-review.json:audit'],
    'proxy-route-writer-fence': ['sdk-review.json:audit', 'sdk-review.json:criterion_review.route_fences'],
    'route-transition-faults': ['sdk-review.json:regression.audit'],
    'independent-sdk-version-conformance': ['sdk-artifacts.json:independent_builds', 'sdk-review.json:codecs', 'sdk-review.json:audit'],
    'selected-runtime-hooks': ['runtime-artifacts.json', 'sdk-review.json:runtime.audit'],
    'sustained-lifecycle-accounting': ['stress-review.json:profiles', 'stress-review.json:criterion_review'],
    'regression-evidence-integrity': ['m1-review.json', 'sdk-review.json:portable_regression', 'negative-probes.json', 'source-lock.json'],
}


class CommandFailure(ValueError):
    def __init__(self, message, result):
        super().__init__(message)
        self.result = result


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    import hashlib
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))


def save(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')


def load_reviewer(story):
    spec = importlib.util.spec_from_file_location('p314_review_' + story, ROOT / f'tools/Review-{story}.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def local(path):
    path = Path(str(path).replace('${WORKSPACE}', str(ROOT))).resolve()
    require(path.is_relative_to(ROOT), 'path outside workspace')
    return path


def source_lock():
    # Include untracked implementation files and detect additions/deletions as
    # well as changed bytes. Prose/evidence may be prepared during a long run.
    paths = subprocess.check_output(['git', 'ls-files', '--cached', '--others', '--exclude-standard'], cwd=ROOT, text=True).splitlines()
    prefixes = ('kernel/', 'crates/', 'sdk/', 'experiments/', 'tools/', 'config/', 'contracts/', 'fixtures/', 'audit/', '.cargo/')
    names = ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml')
    return {p: digest(ROOT / p) for p in sorted(set(paths)) if
            (p.startswith(prefixes) or p in names) and (ROOT / p).is_file()}


class ChildJob:
    """Windows kill-on-close job owns the command and all descendants.

    Nested jobs are supported by this Windows profile. Failure to assign fails
    closed. Each job is private; no unrelated host process is enumerated/killed.
    """
    def __init__(self):
        require(os.name == 'nt', 'this gate requires the approved Windows context')
        self.api = ctypes.WinDLL('kernel32', use_last_error=True)
        self.api.CreateJobObjectW.argtypes = (ctypes.c_void_p, wintypes.LPCWSTR)
        self.api.CreateJobObjectW.restype = wintypes.HANDLE
        self.api.SetInformationJobObject.argtypes = (wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD)
        self.api.AssignProcessToJobObject.argtypes = (wintypes.HANDLE, wintypes.HANDLE)
        self.api.TerminateJobObject.argtypes = (wintypes.HANDLE, wintypes.UINT)
        self.api.CloseHandle.argtypes = (wintypes.HANDLE,)
        # JOBOBJECT_EXTENDED_LIMIT_INFORMATION, native Windows pointer layout.
        class Basic(ctypes.Structure):
            _fields_ = [('process_time', ctypes.c_int64), ('job_time', ctypes.c_int64),
                        ('flags', wintypes.DWORD), ('minimum', ctypes.c_size_t),
                        ('maximum', ctypes.c_size_t), ('active', wintypes.DWORD),
                        ('affinity', ctypes.c_size_t), ('priority', wintypes.DWORD),
                        ('scheduling', wintypes.DWORD)]
        class Limits(ctypes.Structure):
            _fields_ = [('basic', Basic), ('io', ctypes.c_uint64 * 6),
                        ('process_memory', ctypes.c_size_t), ('job_memory', ctypes.c_size_t),
                        ('peak_process', ctypes.c_size_t), ('peak_job', ctypes.c_size_t)]
        self.handle = self.api.CreateJobObjectW(None, None)
        require(self.handle, 'CreateJobObject failed')
        limits = Limits()
        limits.basic.flags = 0x2000  # JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
        if not self.api.SetInformationJobObject(self.handle, 9, ctypes.byref(limits), ctypes.sizeof(limits)):
            self.close()
            raise OSError(ctypes.get_last_error(), 'SetInformationJobObject failed')

    def assign(self, process):
        if not self.api.AssignProcessToJobObject(self.handle, int(process._handle)):
            process.kill()
            process.wait()
            raise OSError(ctypes.get_last_error(), 'AssignProcessToJobObject failed')

    def close(self):
        if self.handle:
            self.api.TerminateJobObject(self.handle, 1)
            self.api.CloseHandle(self.handle)
            self.handle = None


def command(run, name, args, timeout):
    started = time.monotonic()
    started_utc = datetime.now(timezone.utc).isoformat()
    print(f'M3 stage start name={name} utc={started_utc} artifacts={run}', flush=True)
    job = ChildJob()
    process = None
    timed_out = False
    try:
        with (run / (name + '.stdout.log')).open('wb') as stdout, (run / (name + '.stderr.log')).open('wb') as stderr:
            process = subprocess.Popen(args, cwd=ROOT, stdout=stdout, stderr=stderr,
                                       creationflags=subprocess.CREATE_NO_WINDOW)
            job.assign(process)
            with (run / (name + '.stdout.log')).open('rb') as live:
                pending = b''
                while True:
                    pending += live.read()
                    lines = pending.split(b'\n')
                    pending = lines.pop()
                    for line in lines:
                        if line.startswith(b'M3 stage '):
                            print(line.decode('utf-8', errors='replace'), flush=True)
                    remaining = timeout - (time.monotonic() - started)
                    if remaining <= 0:
                        timed_out = True
                        job.close()
                        process.wait(timeout=10)
                        break
                    try:
                        process.wait(timeout=min(0.25, remaining))
                        # Drain final progress records after process exit.
                        for line in (pending + live.read()).splitlines():
                            if line.startswith(b'M3 stage '):
                                print(line.decode('utf-8', errors='replace'), flush=True)
                        break
                    except subprocess.TimeoutExpired:
                        pass
    finally:
        job.close()
        if process is not None:
            process.wait(timeout=10)
    result = dict(name=name, arguments=args, exit_code=process.returncode,
                  started_utc=started_utc, finished_utc=datetime.now(timezone.utc).isoformat(),
                  timed_out=timed_out, deadline_seconds=timeout,
                  cleanup_reaped=True, elapsed_ms=round((time.monotonic() - started) * 1000))
    save(run / (name + '-command.json'), result)
    print(f'M3 stage finish name={name} exit={process.returncode} elapsed_ms={result["elapsed_ms"]} artifacts={run}', flush=True)
    if timed_out or result['exit_code'] != 0:
        raise CommandFailure(f'{name} failed; see {run}', result)
    return result


def replay(folder, sdk_folder, name, deadline_ms, check):
    """Execute exact accepted runtime ISO with fresh vars and serial paths."""
    folder.mkdir()
    original = sdk_folder / 'sdk-runtime-run'
    launch = read(original / 'launch-manifest.json')
    case = read(original / 'case-result.json')
    args = [a.replace('${WORKSPACE}', str(ROOT)).replace('\\', '/') for a in launch['arguments']]
    args = [a.replace(str(original), str(folder)) for a in args]
    # Sanitized paths use forward slashes on some report writers.
    args = [a.replace(original.as_posix(), folder.as_posix()) for a in args]
    require(any(str(folder / 'serial.log') in a or (folder / 'serial.log').as_posix() in a for a in args), 'serial replay path was not replaced')
    lock = read(ROOT / 'config/m1-inputs.lock.json')
    # Use exact preflight-reviewed executable and copied firmware template.
    pins = {item['id']: item for item in lock['files']}
    qemu = local(pins['qemu']['path'])
    require(digest(qemu) == pins['qemu']['sha256'], 'changed QEMU')
    # The original run may have changed NVRAM; use the locked pristine template.
    vars_source = local(pins['firmware_vars']['path'])
    require(digest(vars_source) == launch['vars_template_sha256'], 'changed firmware template')
    shutil.copyfile(vars_source, folder / 'fresh firmware vars.fd')
    iso_arg = next(a for a in args if 'continuum.iso' in a)
    iso_path = Path(iso_arg.split('file=', 1)[1].split(',', 1)[0]) if 'file=' in iso_arg else Path(iso_arg)
    require(digest(iso_path) == case['package']['iso_sha256'], 'changed replay ISO')
    save(folder / 'launch.json', dict(arguments=args, executable_sha256=digest(qemu),
                                     iso_sha256=digest(iso_path), deadline_ms=deadline_ms))
    job = ChildJob()
    started = time.monotonic()
    process = None
    timed_out = False
    error = None
    try:
        with (folder / 'qemu.stdout.log').open('wb') as stdout, (folder / 'qemu.stderr.log').open('wb') as stderr:
            process = subprocess.Popen([str(qemu), *args], stdout=stdout, stderr=stderr, creationflags=subprocess.CREATE_NO_WINDOW)
            job.assign(process)
            try:
                process.wait(timeout=deadline_ms / 1000)
            except subprocess.TimeoutExpired:
                timed_out = True
            if timed_out:
                error = 'runner deadline exceeded'
            else:
                require(process.returncode == 33, 'replay emulator exit mismatch')
                check(folder / 'serial.log', launch['build_id'], launch['host_session_id'])
    except ValueError as exc:
        error = str(exc)
    finally:
        job.close()
        if process is not None:
            process.wait(timeout=10)
    result = dict(name=name, status='rejected' if error else 'passed', error=error,
                  timed_out=timed_out, cleanup_reaped=True, launched=True,
                  exit_code=process.returncode, deadline_ms=deadline_ms,
                  elapsed_ms=round((time.monotonic() - started) * 1000))
    if (folder / 'serial.log').exists():
        result['serial_sha256'] = digest(folder / 'serial.log')
    save(folder / 'result.json', result)
    return result


def negatives(run, sdk_folder):
    sdk = load_reviewer('P3-12')
    folder = run / 'negative-probes'
    folder.mkdir()
    results = []
    def reject(name, function):
        try:
            function()
        except ValueError as error:
            results.append(dict(name=name, status='rejected', error=str(error), launched=False, cleanup_reaped=True))
            return
        raise ValueError('accepted deliberate fault: ' + name)

    broken = folder / 'broken-user.elf'
    content = bytearray((sdk_folder / 'user_a.elf').read_bytes())
    content[:4] = b'BAD!'
    broken.write_bytes(content)
    reject('broken-user-image', lambda: sdk.routes.cap.scheduler.user_elf(broken))
    reject('artifact-hash', lambda: require(digest(sdk_folder / 'user_a.elf') == '0' * 64, 'user artifact hash mismatch'))

    def audit(raw, build, session, fault=None):
        rows = sdk.routes.channel.raw_records(raw.read_bytes(), build, session)
        if fault == 'missing-completion-record':
            # Fault adapter drops the required record AFTER a real guest run.
            # Original serial bytes are retained separately, never overwritten.
            rows = [(e, f) for e, f in rows if e != 'm1.route_ready']
        if fault == 'expected-process-outcome':
            end = next(f for e, f in rows if e == 'm1.route_end' and f['actor'] == '2')
            require(end['code'] == '97', 'deliberately wrong process outcome: expected panic code 97')
        sdk.runtime_audit(rows)

    control = replay(folder / 'control', sdk_folder, 'control', 30000, audit)
    require(control['status'] == 'passed', 'negative probe control failed')
    for name in ('expected-process-outcome', 'missing-completion-record', 'runner-timeout'):
        result = replay(folder / name, sdk_folder, name, 1 if name == 'runner-timeout' else 30000,
                        lambda raw, build, session, name=name: audit(raw, build, session, name))
        require(result['status'] == 'rejected' and result['cleanup_reaped'], 'fault accepted or child leaked: ' + name)
        require(result['timed_out'] == (name == 'runner-timeout'), 'fault misclassified: ' + name)
        results.append(result)
    report = dict(status='passed', control=control, cases=results,
                  adapter='wrong expected panic code / dropped completion apply to independent consumer after real ISO replay; prelaunch ELF/hash faults launch no VM; timeout kills/reaps real QEMU')
    save(run / 'negative-probes.json', report)
    return report


def review_negatives(run, sdk_folder, negative):
    """Reopen actual failure inputs/serial; reject forged pass summaries."""
    sdk = load_reviewer('P3-12')
    folder = run / 'negative-probes'
    try:
        sdk.routes.cap.scheduler.user_elf(folder / 'broken-user.elf')
    except ValueError:
        pass
    else:
        raise ValueError('broken image no longer rejects')
    require(digest(sdk_folder / 'user_a.elf') != '0'*64, 'invalid hash mutation setup')
    original = read(sdk_folder / 'sdk-runtime-run/launch-manifest.json')
    for case in [negative['control'], *negative['cases'][2:]]:
        probe = folder / case['name']
        launch = read(probe / 'launch.json')
        require(launch['iso_sha256'] == original['iso_sha256'], 'negative replay artifact drift')
        require(launch['deadline_ms'] == case['deadline_ms'], 'negative replay deadline drift')
        if case['name'] == 'runner-timeout':
            require(case['timed_out'] and case['deadline_ms'] == 1 and case['exit_code'] != 33, 'timeout misclassified')
            continue
        require(not case['timed_out'] and case['exit_code'] == 33 and case['deadline_ms'] == 30000, 'non-timeout failure misclassified')
        rows = sdk.routes.channel.raw_records((probe / 'serial.log').read_bytes(), original['build_id'], original['host_session_id'])
        # Every non-timeout original stream is valid before the fault adapter.
        sdk.runtime_audit(rows)
        if case['name'] == 'missing-completion-record':
            try:
                sdk.runtime_audit([(e,f) for e,f in rows if e != 'm1.route_ready'])
            except ValueError:
                pass
            else:
                raise ValueError('missing completion accepted')
        elif case['name'] == 'expected-process-outcome':
            require(next(f for e,f in rows if e == 'm1.route_end' and f['actor'] == '2')['code'] == '98', 'panic oracle setup changed')
        else:
            require(case['name'] == 'control', 'unknown negative case')


def run_gate():
    config = read(CONFIG)
    require(config['required_matrix'] == list(MATRIX), 'missing/unknown/reordered matrix scenario')
    require(config['regression_command'] == ['pwsh', '-NoProfile', '-File', 'tools/Test-P3-13.ps1'], 'unexpected regression command')
    require(config['guest_deadline_ms'] == 30000 and config['negative_timeout_ms'] == 1, 'changed deadlines')
    run = ROOT / 'build/m3/P3-14' / ('acceptance-' + str(time.time_ns()))
    run.mkdir(parents=True)
    locked = source_lock()
    save(run / 'source-lock.json', locked)
    (run / 'source.patch').write_bytes(subprocess.check_output(['git', 'diff', '--binary'], cwd=ROOT))
    (run / 'source-status.txt').write_bytes(subprocess.check_output(['git', 'status', '--short'], cwd=ROOT))
    report = dict(schema_version=1, story='P3-14', status='running',
                  source_commit=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  source_lock_sha256=digest(run / 'source-lock.json'), config=copy.deepcopy(config),
                  commands=[], runs={})
    print('P3-14 local run: ' + str(run), flush=True)
    try:
        report['commands'].append(command(run, 'runner-tests', [sys.executable, 'tools/Test-P3-14Runner.py'], 30))
        report['commands'].append(command(run, 'host-lifecycle-preflight', [sys.executable, 'tools/Test-M3HostLifecycle.py'], 30))
        # Force one fresh public stress build; later repetitions may share its
        # exact source/compiler-keyed immutable artifact. Preserve old caches.
        cache = (ROOT / 'build/m3/P3-13/public-cache').resolve()
        backup = (run / 'previous-public-cache').resolve()
        require(cache.is_relative_to(ROOT / 'build/m3/P3-13') and backup.is_relative_to(run), 'cache move outside local build area')
        if cache.exists():
            shutil.move(str(cache), str(backup))
        report['fresh_stress_cache'] = True
        report['commands'].append(command(run, 'runtime-stress', config['regression_command'], config['regression_deadline_seconds']))
        output = (run / 'runtime-stress.stdout.log').read_text(encoding='utf-8-sig')
        matches = re.findall(r'P3-13 acceptance: ([^\r\n]+)', output)
        require(len(matches) == 1, 'missing/ambiguous fresh regression path')
        stress_path = local(matches[0].strip())
        stress = read(stress_path)
        require('regression_reuse' not in stress, 'historical reuse forbidden at M3 exit')
        report['runs']['stress_acceptance'] = str(stress_path)
        prior_path = local(stress['runs']['prior_acceptance'])
        prior = read(prior_path)
        require('prerequisite_reuse' not in prior, 'historical prerequisites forbidden')
        pre = local(prior['runs']['prerequisites'])
        m1 = local(prior['runs']['m1'])
        report['commands'].append(command(run, 'prerequisite-lifecycle', [sys.executable, 'tools/Test-P3-01.py', '--prerequisites', str(pre), '--m1', str(m1)], config['prerequisite_review_deadline_seconds']))
        report['runs']['prerequisite_acceptance'] = (run / 'prerequisite-lifecycle.stdout.log').read_text().strip().splitlines()[-1]
        load_reviewer('P3-13').aggregate(stress_path)
        report['negative_probes'] = negatives(run, local(prior['runs']['sdk_runtime']))
        require(source_lock() == locked, 'implementation source set changed during acceptance')
        report['status'] = 'passed'
    except KeyboardInterrupt:
        report.update(status='interrupted', error='owner interrupted acceptance')
        raise
    except Exception as error:
        if isinstance(error, CommandFailure):
            report['commands'].append(error.result)
        report.update(status='failed', error=str(error))
        raise
    finally:
        save(run / 'acceptance.json', report)
        print('P3-14 acceptance: ' + str(run / 'acceptance.json'), flush=True)
    review(run / 'acceptance.json')


def review(path, destination=None):
    path = local(path)
    acceptance = read(path)
    require(acceptance['story'] == 'P3-14' and acceptance['status'] == 'passed', 'incomplete P3-14 gate')
    require(acceptance['config'] == read(CONFIG), 'changed acceptance configuration')
    require(read(CONFIG)['required_matrix'] == list(MATRIX), 'missing/unknown matrix scenario')
    locked = read(path.parent / 'source-lock.json')
    require(digest(path.parent / 'source-lock.json') == acceptance['source_lock_sha256'] and locked == source_lock(), 'stale acceptance source')
    require([c['name'] for c in acceptance['commands']] == ['runner-tests', 'host-lifecycle-preflight', 'runtime-stress', 'prerequisite-lifecycle'] and all(c['exit_code'] == 0 and not c['timed_out'] and c['cleanup_reaped'] for c in acceptance['commands']), 'missing/failed required command')
    stress_path = local(acceptance['runs']['stress_acceptance'])
    stress = read(stress_path)
    require('regression_reuse' not in stress, 'historical stress regression reuse')
    prior_path = local(stress['runs']['prior_acceptance'])
    prior = read(prior_path)
    require('prerequisite_reuse' not in prior, 'historical prerequisite reuse')
    expected_prior = ['prerequisites', 'm1', 'm1-review', 'owned-memory', 'static-loader',
                      'user-boundary', 'scheduler', 'capabilities', 'channels', 'waits',
                      'supervisor', 'policy', 'routes', 'sdk', 'sdk-runtime',
                      'sdk-codec-build', 'sdk-codecs', 'sdk-flavors', 'm2-portable']
    require([c['name'] for c in prior['commands']] == expected_prior, 'missing/substituted runtime command')
    expected_stress = ['public-supervision', 'public-supervision-review']
    for repetition in range(5):
        for trace in ('off', 'on'):
            label = f'stress-{trace}-{repetition}'
            expected_stress += [label, label+'-review']
            if trace == 'off' and repetition == 0:
                expected_stress += ['stress-flavors']
    expected_stress += ['runtime-regressions', 'runtime-regression-review']
    require([c['name'] for c in stress['commands']] == expected_stress, 'missing/substituted stress command')
    # Full existing independent raw-byte, artifact, inventory, authority and
    # codegen review. A pass string alone is never the aggregate oracle.
    stress_review = load_reviewer('P3-13').aggregate(stress_path)
    sdk_review = read(prior_path.parent / 'review.json')
    prerequisite_path = local(acceptance['runs']['prerequisite_acceptance']) / 'acceptance.json'
    prerequisite = read(prerequisite_path)
    require(prerequisite['status'] == 'passed' and len(prerequisite['rows']) == 42
            and len({r['name'] for r in prerequisite['rows']}) == 42
            and all(r['status'] == 'passed' for r in prerequisite['rows']), 'incomplete prerequisite/lifecycle review')
    negative = read(path.parent / 'negative-probes.json')
    require(negative == acceptance['negative_probes'] and negative['status'] == 'passed' and negative['control']['status'] == 'passed', 'negative control mismatch')
    require([c['name'] for c in negative['cases']] == read(CONFIG)['required_negative_cases'] and all(c['status'] == 'rejected' and c['cleanup_reaped'] for c in negative['cases']), 'missing/accepted negative case')
    for case in [negative['control'], *negative['cases'][2:]]:
        folder = path.parent / 'negative-probes' / case['name']
        require(read(folder / 'result.json') == case, 'negative lifecycle substituted')
        if 'serial_sha256' in case:
            require(digest(folder / 'serial.log') == case['serial_sha256'], 'changed negative raw evidence')
    review_negatives(path.parent, local(prior['runs']['sdk_runtime']), negative)
    # These rows map to independently reviewed scenarios, not inferred product
    # features. Detailed prior reviewers retain their exact per-case oracles.
    matrix = [dict(scenario=name, status='passed', evidence=MATRIX[name]) for name in read(CONFIG)['required_matrix']]
    report = dict(story='P3-14', status='passed', review='local Codex self-review; not independent human/security approval',
                  source_commit=acceptance['source_commit'], acceptance_sha256=digest(path),
                  reviewer_sha256=digest(__file__), source_lock_sha256=digest(path.parent / 'source-lock.json'),
                  sdk_contract_sha256=digest(ROOT / 'contracts/sdk-v1.json'), matrix=matrix,
                  prerequisite_rows=len(prerequisite['rows']), negative_probes=negative,
                  stress_review_sha256=digest(stress_path.parent / 'review.json'),
                  sdk_review_sha256=digest(prior_path.parent / 'review.json'),
                  runtime_regression={k:sdk_review['regression'][k] for k in ('kernel_host_tests', 'prerequisite_checks', 'm1_rows', 'm1_launches')}, sdk_audit=sdk_review['audit'],
                  runtime_hooks=sdk_review['runtime']['audit'], codecs=sdk_review['codecs'],
                  stress_metrics={k:v for k,v in stress_review.items() if k not in ('runtime_regression', 'guests', 'artifact_audits')},
                  unsafe_review=dict(sdk=sdk_review['unsafe_review'], stress=stress_review.get('unsafe_review', {}),
                                     host='private kill-on-close Windows jobs contain command descendants; real replay uses exact ISO, fresh locked vars, no host shares/network/disks; all spawned QEMU processes waited'),
                  limits=['one vCPU, one thread/process, eager x87/SSE2 with masked SSE exceptions',
                          'ephemeral synthetic identities/policy/routes; surviving direct mappings end at owner teardown',
                          'only exact SDK contract subset frozen; supervision/later desktop families experimental',
                          'no guest UI/input/render/reload, state migration, safe old-code retirement, durability or real accounts',
                          '256 MiB desktop cold-idle and desktop latency targets remain open'])
    require(report['sdk_contract_sha256'] == read(CONFIG)['sdk_contract_sha256'], 'frozen contract changed')
    save(path.parent / 'review.json', report)
    if destination:
        promote(local(destination), path, report, stress_path, prior_path, locked)
    print(json.dumps(dict(status='passed', matrix_rows=len(matrix), negative_cases=len(negative['cases'])), indent=2))
    return report


def promote(dest, path, report, stress_path, prior_path, locked):
    require(dest.is_relative_to(ROOT / 'evidence/phase-3/P3-14') and not dest.exists(), 'bad/existing evidence destination')
    dest.mkdir(parents=True)
    for name, value in [('review.json', report), ('acceptance.json', read(path)), ('source-lock.json', locked),
                        ('runtime-acceptance.json', read(prior_path)), ('stress-acceptance.json', read(stress_path)),
                        ('negative-probes.json', read(path.parent / 'negative-probes.json'))]:
        save(dest / name, value)
    prior = read(prior_path)
    for key, filename in [('sdk', 'sdk-artifacts.json'), ('sdk_runtime', 'runtime-artifacts.json')]:
        folder = local(prior['runs'][key])
        run = folder / ('sdk-run' if key == 'sdk' else 'sdk-runtime-run')
        save(dest / filename, dict(build=read(folder / 'build-manifest.json'), images=read(folder / 'user-images.json'),
                                   independent_builds=read(folder / 'sdk-builds.json'),
                                   launch=read(run / 'launch-manifest.json'), result=read(run / 'case-result.json')))
    m1 = local(prior['runs']['m1'])
    for source, name in [(m1 / 'review.json', 'm1-review.json'),
                         (local(prior['runs']['prerequisites']) / 'prerequisites.json', 'prerequisites.json'),
                         (prior_path.parent / 'review.json', 'sdk-review.json'),
                         (stress_path.parent / 'review.json', 'stress-review.json')]:
        shutil.copyfile(source, dest / name)
    selections = []
    serials = [local(prior['runs']['sdk']) / 'sdk-run/serial.log',
               local(prior['runs']['sdk_runtime']) / 'sdk-runtime-run/serial.log']
    for entry in read(stress_path)['stress']:
        serials.append(next(local(entry['artifacts']).glob('*-run/serial.log')))
    for index, serial in enumerate(serials):
        lines = serial.read_bytes().splitlines(keepends=True)
        indices = sorted(set([0, *range(max(0, len(lines)-6), len(lines))]))
        name = f'selected-{index:02d}.txt'
        (dest / name).write_bytes(b''.join(lines[i] for i in indices))
        selections.append(dict(file=name, source=str(serial), source_sha256=digest(serial), total_lines=len(lines), selected_lines=[i+1 for i in indices]))
    save(dest / 'trace-selection.json', selections)
    for item in (path.parent / 'negative-probes').glob('*/launch.json'):
        save(dest / ('negative-' + item.parent.name + '.json'), dict(launch=read(item), result=read(item.parent / 'result.json')))
    # Enforce the retention budget before anything is staged.
    files = [p for p in dest.rglob('*') if p.is_file()]
    require(len(files) <= 200 and sum(p.stat().st_size for p in files) <= 4*1024*1024 and max(p.stat().st_size for p in files) <= 256*1024, 'compact evidence budget exceeded')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['run', 'review'])
    parser.add_argument('--acceptance')
    parser.add_argument('--destination')
    args = parser.parse_args()
    if args.action == 'run':
        run_gate()
    else:
        require(args.acceptance, 'review requires acceptance path')
        review(args.acceptance, args.destination)
