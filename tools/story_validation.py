"""Agent-selected story checks. Full milestone gates keep their existing policy.

Selection is a reviewed engineering decision, not inferred from file names.
Run from Enter-Dev.ps1 for builds/guests. All artifacts remain workspace-local.
"""
import argparse
import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time

import p3_acceptance as base
import evidence_retention as retention

ROOT = base.ROOT
require, read, save, digest = base.require, base.read, base.save, base.digest
PS = ['pwsh', '-NoProfile', '-File']
RUNTIME = {
    'memory': ('owned-memory', 'P3-02'),
    'loader': ('static-loader', 'P3-03'),
    'boundary': ('user-boundary', 'P3-04'),
    'scheduler': ('scheduler', 'P3-05'),
    'capabilities': ('capabilities', 'P3-06'),
    'channels': ('channels', 'P3-07'),
    'waits': ('waits', 'P3-08'),
    'supervisor': ('supervisor', 'P3-09'),
    'policy': ('policy', 'P3-10'),
    'routes': ('routes', 'P3-11'),
    'sdk': ('sdk', 'P3-12'),
    'sdk-runtime': ('sdk-runtime', 'P3-12'),
}


def catalog():
    checks = {
        'docs': (['node', 'audit/check-docs.cjs'], 120),
        'governance': (['node', 'audit/check-governance.cjs'], 120),
        # Existing index audit opens many historical blobs; its old direct
        # invocation was unbounded. This host bound does not alter guest limits.
        'hygiene': (['node', 'audit/check-repository-hygiene.cjs'], 1800),
        'runner-tests': ([sys.executable, 'tools/Test-StoryValidation.py'], 60),
        'retention-tests': ([sys.executable, 'tools/Test-EvidenceRetention.py'], 60),
        'child-runner-tests': ([sys.executable, 'tools/Test-P3-14Runner.py'], 60),
        'hygiene-tests': (['node', '--test', 'audit/check-repository-hygiene.test.cjs'], 180),
        'prerequisites': (PS + ['tools/Recheck-P2Prerequisites.ps1'], 1800),
        'kernel-host': (['cargo', 'test', '--locked', '--offline', '-p', 'continuum-kernel', '--lib'], 180),
        'desktop-host': (['cargo', 'test', '--locked', '--offline', '-p', 'continuum-kernel', '--lib', '--features', 'desktop-runtime'], 180),
        'display-host': (['cargo', 'test', '--locked', '--offline', '-p', 'continuum-kernel', '--lib', '--features', 'display-test'], 180),
        'boot-metadata': (['cargo','xtask','test','--suite','m1','--case','boot-metadata'],600),
        'scanout-memory': (['cargo','xtask','test','--suite','m1','--case','memory-ownership'],600),
        'scanout-guard': (['cargo','xtask','test','--suite','m1','--case','guard-fault'],600),
        'device-host': (['cargo', 'test', '--locked', '--offline', '-p', 'continuum-kernel', '--lib', '--features', 'device-test'], 180),
        'xtask-host': (['cargo', 'test', '--locked', '--offline', '-p', 'xtask', '--bin', 'xtask', 'm1::'], 300),
        'timer': (['cargo', 'xtask', 'test', '--suite', 'm1', '--case', 'timer-progress'], 600),
        'full-m1': (PS + ['tools/Test-P1-07.ps1'], 10800),
        'full-m2': (PS + ['tools/Test-P2-13.ps1'], 10800),
        'full-m3': (PS + ['tools/Test-P3-14.ps1'], 10800),
        'portable': (PS + ['tools/Check-P3-12Portable.ps1', '-Output', '{output}/portable'], 600),
    }
    for key, package in (('ui-host', 'custom-ui'), ('text-host', 'text-core'), ('editor-host', 'editor-model')):
        checks[key] = (['cargo', 'test', '--locked', '--offline', '-p', package, '--lib'], 180)
    for key, (case, _) in RUNTIME.items():
        checks[key] = (['cargo', 'xtask', 'test', '--suite', 'm1', '--case', case], 600)
    return checks


# Coverage means the unchanged aggregate runner AND reviewer require this check.
# It is not inferred merely because a suite happens to build similar code.
COVERS = {'full-m1': {'kernel-host'},
          'full-m3': {'prerequisites', 'full-m1', 'kernel-host', 'portable', *RUNTIME}}
FAMILIES = ('M1', 'M2', 'M3', 'prerequisites')


def validate_manifest(manifest):
    require(manifest['scope'] == 'story',
            'story runner cannot certify milestone acceptance')
    retention.policy(manifest)
    require(re.fullmatch(r'P\d+-\d+|[A-Za-z0-9][A-Za-z0-9_-]*', manifest['story']), 'invalid story')
    impact = manifest['impact']
    for key in ('subsystems', 'consumers', 'rationale'):
        require(bool(impact[key]), 'missing impact ' + key)
    selected = manifest['selected']
    ids = [entry['id'] for entry in selected]
    require(ids and len(ids) == len(set(ids)), 'empty/duplicate selection')
    for entry in selected:
        require(entry['id'] in catalog() and bool(entry['reason'].strip()), 'unknown/unexplained check')
    covered = set(ids)
    for key in ids:
        covered.update(COVERS.get(key, set()))
    for family in FAMILIES:
        key = 'prerequisites' if family == 'prerequisites' else 'full-' + family.lower()
        require(key in covered or bool(manifest['not_selected'].get(family, '').strip()),
                'missing omission rationale: ' + family)
    for family in ('m1', 'm2', 'm3'):
        require(type(impact['broad_' + family]) is bool, 'broad impact must be explicit')
        if impact['broad_' + family] or family.upper() in impact['uncertain_families']:
            require('full-' + family in covered, 'broad/uncertain impact requires full-' + family)
    require(set(impact['uncertain_families']) <= set(FAMILIES), 'unknown uncertain family')
    require(type(impact['shared_boundary_scheduler']) is bool, 'shared boundary impact must be explicit')
    if impact['shared_boundary_scheduler']:
        require({'boundary', 'scheduler', 'capabilities'} <= covered, 'missing shared-boundary guests')
        # The P4 runner always executes ordinary boot and its separate flavor matrix.
        require(manifest['story'] in ('P4-02', 'P4-03', 'P4-04'), 'shared-boundary story needs an ordinary/flavor adapter')
    if manifest['story'] == 'P4-03':
        require({'kernel-host', 'device-host', 'xtask-host', 'memory', 'timer', 'boundary', 'scheduler', 'capabilities'} <= covered,
                'missing P4-03 affected coverage')
    if manifest['story'] == 'P4-04':
        require({'kernel-host','display-host','xtask-host','prerequisites','boot-metadata','scanout-memory','scanout-guard','memory','timer','boundary','scheduler','capabilities'} <= covered,
                'missing P4-04 affected coverage')
    if manifest['story'] == 'P4-02':
        require({'kernel-host', 'desktop-host'} <= covered, 'missing P4-02 host tests')
    for item in manifest.get('historical_references', []):
        require(bool(item['reason']), 'unexplained historical reference')
    return manifest


def schedule(manifest):
    """Expand commands, removing only explicitly covered duplicate checks."""
    validate_manifest(manifest)
    ids = [entry['id'] for entry in manifest['selected']]
    covered = {key: next((other for other in ids if key in COVERS.get(other, set())), None) for key in ids}
    return [key for key in ids if not covered[key]], {key: value for key, value in covered.items() if value}



# Counts are actual launches inside each focused xtask case, not command counts.
# Full M3 remains deliberately unknown here: it owns nested suites and negatives.
HOST_CHECKS = {'docs', 'governance', 'hygiene', 'runner-tests', 'retention-tests', 'child-runner-tests',
               'hygiene-tests', 'prerequisites', 'kernel-host', 'desktop-host',
               'portable', 'ui-host', 'text-host', 'editor-host', 'device-host', 'display-host', 'xtask-host'}
AUDIT_CHECKS = {'docs', 'governance', 'hygiene'}
VM_LAUNCHES = {key: 1 for key in RUNTIME}
VM_LAUNCHES.update(boundary=4, scheduler=3, **{'full-m1': 42, 'full-m2': 0, 'full-m3': None,
                                             'boot-metadata':4,'scanout-memory':1,'scanout-guard':1,'display-720':1,'display-1080':1,'display-flavors':0,'ordinary': 1, 'flavors': 0, 'device-0': 1, 'device-flavors': 0, 'timer': 1})


def execution_plan(manifest, checks=None):
    selected, covered = schedule(manifest)
    checks = list(selected if checks is None else checks)
    require(len(checks) == len(set(checks)), 'duplicate planned check')
    counts = {}
    for key in checks:
        if re.fullmatch(r'desktop-[0-4]', key):
            counts[key] = 1
        elif key in VM_LAUNCHES:
            counts[key] = VM_LAUNCHES[key]
        else:
            require(key in HOST_CHECKS and key in catalog(), 'missing launch classification: ' + key)
            counts[key] = 0
    full = [key for key in checks if key.startswith('full-')]
    unknown = [key for key, count in counts.items() if count is None]
    total = None if unknown else sum(counts.values())
    budget = manifest.get('execution_budget')
    if budget is None:
        require(not full and total == 0, 'guest/full checks require an explicit execution_budget')
        budget = {'max_vm_launches': 0, 'allow_full_suites': False}
    maximum = budget['max_vm_launches']
    require(type(budget['allow_full_suites']) is bool, 'invalid full-suite budget flag')
    require(maximum is None or type(maximum) is int and maximum >= 0, 'invalid VM launch ceiling')
    require(not full or budget['allow_full_suites'], 'full suites require explicit budget opt-in')
    if maximum is None:
        require(full and budget['allow_full_suites'] and bool(budget.get('full_suite_reason', '').strip()),
                'unbounded count requires a reason for the explicit full suite')
    else:
        require(not unknown, 'unknown full-suite count cannot fit a bounded VM budget')
        require(total <= maximum, f'VM launch budget exceeded: {total} > {maximum}')
    plan = {'scope': 'story', 'execute': checks, 'covered_by': covered,
            'command_groups': len(checks), 'vm_launches': total, 'vm_launches_by_check': counts,
            'unknown_launch_counts': unknown, 'full_suites': full,
            'separate_metadata_builds': 18 if 'display-flavors' in checks else 17 if 'device-flavors' in checks else 16 if 'flavors' in checks else 0,
            'execution_budget': copy.deepcopy(budget)}
    if manifest['schema_version'] == 2:
        plan['retention'] = copy.deepcopy(retention.policy(manifest))
    return plan


def source_snapshot():
    # Full file set captures staged, unstaged, new, deleted and renamed inputs.
    names = subprocess.check_output(['git', 'ls-files', '-z', '--cached', '--others', '--exclude-standard'], cwd=ROOT).decode().split('\0')
    result = {name: digest(ROOT / name) if (ROOT / name).is_file() else None
              for name in sorted(set(names) - {''})}
    # Index-sensitive hygiene checks must be invalidated even if working bytes
    # are unchanged when a file is staged or unstaged.
    result['@git-index'] = hashlib.sha256(subprocess.check_output(
        ['git', 'ls-files', '--stage', '-z'], cwd=ROOT)).hexdigest()
    result['@source-commit'] = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    return result



def implementation_inputs(snapshot):
    """Code/config/assets stay strict; prose/evidence/index are rechecked by audits.

    Build inputs must not be stored in docs prose or retained evidence. This is
    not a per-check dependency guess: any implementation/tool/schema change still
    invalidates reuse, including new and deleted files.
    """
    governance = {'AGENTS.md', 'CLAUDE.md', 'README.md', 'EXPORT_CONTROL.md'}
    return {name: sha for name, sha in snapshot.items() if
            name not in governance and name not in ('@git-index', '@source-commit')
            and not name.startswith('evidence/')
            and not (name.startswith('docs/') and name.endswith('.md'))}


def snapshot_digest(snapshot):
    return hashlib.sha256(json.dumps(snapshot, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def context_snapshot(manifest=None):
    """Fingerprint actual executables, pinned guest tools, compiler and runner state."""
    ids = {e['id'] for e in manifest['selected']} if manifest else set(catalog())
    builds = bool(ids & {*RUNTIME, 'full-m1', 'full-m2', 'full-m3', 'kernel-host',
                         'desktop-host', 'device-host', 'xtask-host', 'timer', 'portable', 'ui-host', 'text-host', 'editor-host'})
    builds |= manifest is not None and manifest['story'] in ('P4-02', 'P4-03', 'P4-04')
    paths = {Path(sys.executable)}
    programs = {'git', *[catalog()[key][0][0] for key in ids]}
    if builds:
        programs.update(('cargo', 'rustc', 'pwsh', 'node', 'python'))
    for program in programs:
        path = shutil.which(program)
        if path:
            paths.add(Path(path).resolve())
    for name in ('RUSTC_WRAPPER',):
        if os.environ.get(name):
            paths.add(Path(os.environ[name]).resolve())
    if builds and shutil.which('rustc'):
        sysroot = Path(subprocess.check_output(['rustc', '--print', 'sysroot'], cwd=ROOT, text=True).strip())
        # Includes driver DLLs, LLVM/linker and both host/guest standard libraries.
        for directory in ('bin', 'lib', 'libexec'):
            paths.update(p for p in (sysroot / directory).rglob('*') if p.is_file())
    for item in read(ROOT / 'config/m1-inputs.lock.json')['files'] if builds else []:
        override = os.environ.get(item.get('override') or '')
        path = Path(override) if override else ROOT / item['path']
        paths.add(path.resolve())
    for name in (('work/toolchains/cargo/config.toml', 'work/windows-dev-signing/compiler.json',
                  'work/windows-dev-signing/admin-enforce.json') if builds else ()):
        path = ROOT / name
        paths.add(path)
    env = {key: hashlib.sha256(value.encode()).hexdigest() for key, value in os.environ.items() if
           key.startswith(('CARGO_', 'RUST', 'CONTINUUM_')) or key in ('PATH', 'INCLUDE', 'LIB', 'LIBPATH')}
    return {'files': {str(p): digest(p) if p.is_file() else None for p in sorted(paths)}, 'environment': env}


def tree_hashes(path):
    path = Path(path)
    require(path.resolve().is_relative_to(ROOT), 'artifact outside workspace')
    return {str(p.resolve()): digest(p) for p in sorted(path.rglob('*')) if p.is_file()}


def command_evidence(check):
    """Schema-2 commands bind both output streams and their execution record."""
    stdout = Path(check['stdout'])
    expected = [stdout, stdout.parent / (check['id'] + '.stderr.log'),
                stdout.parent / (check['id'] + '-command.json')]
    require(all(str(p.resolve()) in check['artifact_hashes'] for p in expected),
            'missing command stdout/stderr/execution evidence')


def artifact_path(text, prefix):
    matches = re.findall(re.escape(prefix) + r'([^\r\n]+)', text)
    require(len(matches) == 1, 'missing/ambiguous artifact path: ' + prefix)
    path = Path(matches[0].strip()).resolve()
    require(path.is_relative_to(ROOT), 'artifact outside workspace')
    return path


def review_check(key, stdout, output):
    """Reopen actual artifacts; a command's exit code alone cannot pass a guest."""
    if key in RUNTIME:
        import runtime_focus_review
        path = artifact_path(stdout, 'M1 artifacts: ')
        result = runtime_focus_review.review(key, path)
        return result, [path]
    if key == 'full-m3':
        path = artifact_path(stdout, 'P3-14 acceptance: ')
        return base.review(path), aggregate_artifacts(path)
    if key == 'full-m1':
        path = artifact_path(stdout, 'M1 artifacts: ')
        base.command(output, 'm1-review', PS + ['tools/Review-P1-07.ps1', '-Run', str(path)], 600)
        result = read(path / 'review.json')
        require(result['status'] == 'passed', 'M1 reviewer failed')
        return result, [path]
    if key == 'full-m2':
        path = artifact_path(stdout, 'P2-13 acceptance: ')
        spec = importlib.util.spec_from_file_location('m2_gate', ROOT / 'tools/Run-P2Acceptance.py')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        result = read(path / 'acceptance.json')
        require(result['status'] == 'passed', 'M2 acceptance incomplete')
        module.validate_index(result, read(ROOT / 'config/p2-13-acceptance.json'), path.name, result['artifact'])
        negatives = read(path / 'harness-negatives.json')
        require(len(negatives) == 7 and all(n['rejected'] for n in negatives), 'missing M2 negatives')
        # Every referenced output is included in the immutable run evidence.
        outputs = [Path(p) for row in result['rows'] for p in row['evidence']]
        return result, [path, *[p if p.is_dir() else p.parent for p in outputs]]
    if key == 'prerequisites':
        path = artifact_path(stdout, 'Prerequisite evidence: ')
        result = read(path / 'prerequisites.json')
        require(result['status'] == 'passed' and result['offline'] and len(result['commands']) == 10
                and all(c['exit_code'] == 0 for c in result['commands']), 'incomplete prerequisites')
        return result, [path]
    if key == 'portable':
        path = output / 'portable'
        result = read(path / 'portable.json')
        require(result['status'] == 'passed' and result['guest_ui_executed'] is False
                and result['elf_sha256'] == digest(path / 'selected-ui.elf'), 'portable artifact mismatch')
        return result, [path]
    return {'status': 'passed', 'scope': 'host command'}, []


def aggregate_artifacts(path):
    """M3 stores artifact directories in its child acceptance records."""
    aggregate = read(path)
    stress_path = base.local(aggregate['runs']['stress_acceptance'])
    stress = read(stress_path)
    runtime_path = base.local(stress['runs']['prior_acceptance'])
    runtime = read(runtime_path)
    return [path.parent, stress_path.parent, runtime_path.parent,
            *[base.local(v) for v in runtime['runs'].values()],
            base.local(stress['runs']['public_supervision']), base.local(stress['runs']['flavors']),
            *[base.local(e['artifacts']) for e in stress['stress']]]


class Session:
    """Serial journal. Resume references old successes without overwriting failures."""
    def __init__(self, manifest, folder, resume=None, snapshot=None, context=None, planned_checks=None):
        self.started = time.monotonic()
        self.manifest = copy.deepcopy(validate_manifest(manifest))
        plan = execution_plan(self.manifest, planned_checks)
        self.folder = Path(folder)
        self.folder.mkdir(parents=True, exist_ok=False)
        self.snapshot_fn = snapshot or source_snapshot
        self.context_fn = context or (lambda: context_snapshot(self.manifest))
        self.sources = self.snapshot_fn()
        self.implementation = implementation_inputs(self.sources)
        self.context = self.context_fn()
        self.report = {'schema_version': 1, 'story': manifest['story'], 'scope': 'story',
                       'status': 'running', 'manifest': self.manifest, 'source_snapshot': self.sources,
                       'implementation_inputs': self.implementation,
                       'context': self.context, 'checks': [], 'commands': [], 'plan': plan,
                       'source_commit': self.sources.get('@source-commit'),
                       'not_selected': manifest['not_selected'],
                       'historical_references': manifest.get('historical_references', [])}
        self.previous = {}
        if resume:
            previous = read(resume)
            require(previous['manifest'] == self.manifest and implementation_inputs(previous['source_snapshot']) == self.implementation
                    and previous['context'] == self.context, 'resume inputs changed; start a fresh run')
            require(previous['status'] in ('failed', 'interrupted', 'passed'), 'resume requires a finished or interrupted attempt')
            self.previous = {c['id']: c for c in previous['checks'] if c['status'] == 'fresh-pass'}
            self.report['resumed_from'] = str(Path(resume).resolve())
        self.persist()

    def persist(self):
        save(self.folder / 'acceptance.json', self.report)

    def check_storage(self):
        policy = retention.policy(self.manifest)
        if policy is None:
            return  # Preserve existing journals/plans; no retrospective migration.
        paths = {p for check in self.report['checks'] for p in check['artifact_hashes']}
        paths.update(str(p) for p in self.folder.rglob('*') if p.is_file())
        self.report['working_storage'] = retention.storage(paths, policy['working_set_budget_bytes'])

    def check(self, key, args, limit, reviewer=None, execute=None):
        self.check_storage()
        current = self.snapshot_fn()
        require(implementation_inputs(current) == self.implementation, 'source changed during story validation')
        require(key in self.report['plan']['execute'], 'check was not in the bounded plan')
        require(key not in {r['id'] for r in self.report['checks']}, 'duplicate check id')
        prior = self.previous.get(key)
        if key in AUDIT_CHECKS and prior and prior.get('audit_snapshot_sha256') != snapshot_digest(current):
            prior = None
        # Output-directory-dependent metadata/portable commands are cheap and
        # deliberately rerun in the new attempt rather than repointing old files.
        if any(str(self.folder) in a for a in args):
            prior = None
        if prior:
            if self.manifest['schema_version'] == 2:
                command_evidence(prior)
            require(prior['arguments'] == args and prior['deadline_seconds'] == limit, 'resume command changed')
            require(bool(prior['artifact_hashes']), 'resume lacks retained artifacts')
            require(all(Path(p).is_file() and digest(p) == sha for p, sha in prior['artifact_hashes'].items()), 'resume artifact changed/missing')
            stdout = Path(prior['stdout']).read_text(encoding='utf-8-sig')
            result, _ = reviewer(stdout) if reviewer else ({'status': 'passed'}, [])
            require(result.get('status') == 'passed', 'focused reviewer did not pass')
            record = copy.deepcopy(prior)
            record.update(review=result, resumed=True)
        else:
            result = (execute or base.command)(self.folder, key, args, limit)
            self.report['commands'].append(result)
            require(result['exit_code'] == 0 and not result['timed_out'] and result['cleanup_reaped'], 'failed selected command')
            stdout_path = self.folder / (key + '.stdout.log')
            stdout = stdout_path.read_text(encoding='utf-8-sig')
            review, artifacts = reviewer(stdout) if reviewer else ({'status': 'passed', 'scope': 'host command'}, [])
            require(review.get('status') == 'passed', 'focused reviewer did not pass')
            hashes = {str(stdout_path.resolve()): digest(stdout_path)}
            if self.manifest['schema_version'] == 2:
                for path in (self.folder / (key + '.stderr.log'), self.folder / (key + '-command.json')):
                    require(path.is_file(), 'missing command stderr/execution evidence')
                    hashes[str(path.resolve())] = digest(path)
            for artifact in artifacts:
                hashes.update(tree_hashes(artifact))
            record = {'id': key, 'status': 'fresh-pass', 'arguments': args, 'deadline_seconds': limit,
                      'elapsed_ms': result['elapsed_ms'], 'stdout': str(stdout_path.resolve()),
                      'command': result, 'review': review, 'artifact_hashes': hashes, 'resumed': False}
        after = self.snapshot_fn()
        require(implementation_inputs(after) == self.implementation, 'source changed during story validation')
        if key in AUDIT_CHECKS:
            require(current == after, 'audit inputs changed during check')
            record['audit_snapshot_sha256'] = snapshot_digest(current)
        self.report['checks'].append(record)
        self.persist()
        self.check_storage()
        self.persist()
        print(f"Story check {key}: fresh-pass elapsed_ms={record['elapsed_ms']} resumed={record['resumed']}", flush=True)
        return stdout, record['review']

    def selected(self):
        ids, covered = schedule(self.manifest)
        self.report['covered_by'] = covered
        for key in ids:
            if key in {c['id'] for c in self.report['checks']}:
                continue
            args, limit = catalog()[key]
            args = [a.replace('{output}', str(self.folder)) for a in args]
            self.check(key, args, limit, lambda text, k=key: review_check(k, text, self.folder))

    def finish(self, expected):
        self.check_storage()
        require([c['id'] for c in self.report['checks']] == expected, 'missing/reordered selected checks')
        require(self.report['plan'] == execution_plan(self.manifest, expected), 'execution plan changed')
        current = self.snapshot_fn()
        require(implementation_inputs(current) == self.implementation and self.context_fn() == self.context, 'source/tool context changed')
        require(all(c.get('audit_snapshot_sha256') == snapshot_digest(current) for c in self.report['checks'] if c['id'] in AUDIT_CHECKS), 'audit inputs changed; resume to rerun host audits')
        self.report['status'] = 'passed'
        self.report['command_elapsed_ms'] = sum(c['elapsed_ms'] for c in self.report['checks'] if not c['resumed'])
        self.report['elapsed_ms'] = round((time.monotonic() - self.started) * 1000)
        self.persist()


def review_report(path, expected):
    report = read(path)
    validate_manifest(report['manifest'])
    require(report.get('plan') == execution_plan(report['manifest'], expected), 'changed/missing execution plan')
    require(report['status'] == 'passed' and report['scope'] == 'story', 'incomplete/wrong validation scope')
    current = source_snapshot()
    require(report.get('implementation_inputs') == implementation_inputs(report['source_snapshot']) == implementation_inputs(current)
            and report['context'] == context_snapshot(report['manifest']), 'stale story validation')
    require(all(c.get('audit_snapshot_sha256') == snapshot_digest(current) for c in report['checks'] if c['id'] in AUDIT_CHECKS), 'stale host audits')
    require(report['story'] == report['manifest']['story'], 'story identity mismatch')
    require(report['not_selected'] == report['manifest']['not_selected']
            and report['historical_references'] == report['manifest'].get('historical_references', []),
            'omission/history report differs from selection')
    require([c['id'] for c in report['checks']] == expected, 'missing/reordered selected check')
    require(report.get('covered_by', {}) == schedule(report['manifest'])[1], 'invalid aggregate coverage')
    for check in report['checks']:
        if report['manifest']['schema_version'] == 2:
            command_evidence(check)
        require(check['status'] == 'fresh-pass' and check['review'].get('status') == 'passed'
                and bool(check['artifact_hashes']), 'selected check did not pass')
        require(all(Path(p).is_file() and digest(p) == sha for p, sha in check['artifact_hashes'].items()), 'changed/missing retained artifact')
        command = check['command']
        require(command['arguments'] == check['arguments'] and command['exit_code'] == 0
                and command['deadline_seconds'] == check['deadline_seconds']
                and not command['timed_out'] and command['cleanup_reaped'], 'failed/substituted selected command')
        if check['id'] in catalog():
            args, limit = catalog()[check['id']]
            # A resumed command can refer to the previous attempt output root.
            output = str(Path(check['stdout']).parent)
            require(check['arguments'] == [a.replace('{output}', output) for a in args]
                    and check['deadline_seconds'] == limit, 'selected command differs from catalog')
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['plan', 'run', 'review'])
    parser.add_argument('--manifest', required=True)
    parser.add_argument('--resume')
    parser.add_argument('--acceptance')
    args = parser.parse_args()
    manifest = validate_manifest(read(args.manifest))
    ids, covered = schedule(manifest)
    if args.action == 'plan':
        print(json.dumps({'scope': 'story', 'execute': ids, 'covered_by': covered,
                          'commands': {k: catalog()[k][0] for k in ids},
                          'not_selected': manifest['not_selected'],
                          'impact': manifest['impact'], 'plan': execution_plan(manifest)}, indent=2))
        return
    require(manifest['story'] not in ('P4-02', 'P4-03', 'P4-04'), 'use the story adapter for mandatory device/runtime checks')
    if args.action == 'review':
        report = review_report(args.acceptance, ids)
        require(report['manifest'] == manifest, 'selection differs from requested manifest')
        for c in report['checks']:
            review_check(c['id'], Path(c['stdout']).read_text(encoding='utf-8-sig'), Path(args.acceptance).parent)
        print('Story validation reviewed; no milestone acceptance claimed')
        return
    print(json.dumps(execution_plan(manifest), indent=2), flush=True)
    folder = ROOT / 'build/story-validation' / manifest['story'] / str(time.time_ns())
    session = Session(manifest, folder, args.resume)
    try:
        session.selected()
        session.finish(ids)
    except BaseException as error:
        session.report.update(status='interrupted' if isinstance(error, KeyboardInterrupt) else 'failed', error=str(error))
        if isinstance(error, base.CommandFailure):
            session.report['commands'].append(error.result)
        session.persist()
        raise
    finally:
        print('Story acceptance: ' + str(folder / 'acceptance.json'), flush=True)


if __name__ == '__main__':
    main()
