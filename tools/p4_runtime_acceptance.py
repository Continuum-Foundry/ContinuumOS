"""P4-02 story validation: mandatory new behavior plus recorded regressions."""
import argparse
import importlib.util
import json
from pathlib import Path
import time

import p3_acceptance as base
import story_validation as validation

ROOT = base.ROOT
DEFAULT_MANIFEST = ROOT / 'config/p4-02-validation.json'
FLAVORS = ['ordinary-minimal', 'desktop-runtime', 'desktop-test', 'user-boundary-test',
           'scheduler-test', 'capability-test', 'channel-test', 'wait-test', 'supervisor-test',
           'policy-test', 'route-test', 'sdk-test', 'sdk-runtime-test', 'stress-test',
           'stress-trace', 'stress-supervisor-test']


def focused():
    spec = importlib.util.spec_from_file_location('p402_review', ROOT / 'tools/Review-P4-02.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def guest_review(text, test):
    path = validation.artifact_path(text, 'M1 artifacts: ')
    return focused().review(path, test), [path]


def flavor_review(output):
    flavors = base.read(output / 'flavors.json')
    base.require([v['flavor'] for v in flavors] == FLAVORS
                 and all(v['exit_code'] == 0 for v in flavors), 'missing/failed separate flavor')
    return {'status': 'passed', 'directory': str(output), 'flavors': flavors}, [output]


def expected_checks(manifest):
    selected, _ = validation.schedule(manifest)
    early = [k for k in ('kernel-host', 'desktop-host') if k in selected]
    return early + ['desktop-0', 'flavors', 'desktop-1', 'desktop-2', 'desktop-3',
                    'desktop-4', 'ordinary'] + [k for k in selected if k not in early]


def require_implementation():
    # The validation-policy PR can land before the separate P4-02 kernel story.
    # A preview is useful on that baseline; an acceptance attempt must fail
    # before launching commands when the implementation/oracle is absent.
    dependencies = ['config/p4-02-runtime.json', 'kernel/src/desktop.rs',
                    'kernel/src/desktop_guest.rs', 'kernel/src/reclaiming_heap.rs',
                    'experiments/user-images/desktop.rs', 'tools/Review-P4-02.py',
                    'tools/Check-P4-02Flavors.ps1']
    missing = [name for name in dependencies if not (ROOT / name).is_file()]
    cargo = (ROOT / 'kernel/Cargo.toml').read_text() if (ROOT / 'kernel/Cargo.toml').is_file() else ''
    if 'desktop-runtime' not in cargo or 'desktop-test' not in cargo:
        missing.append('kernel desktop-runtime/desktop-test features')
    base.require(not missing, 'P4-02 implementation prerequisites missing; this adapter does not implement '
                 'or accept the kernel story: ' + ', '.join(missing))


def run_gate(manifest_path=DEFAULT_MANIFEST, resume=None):
    manifest = validation.validate_manifest(base.read(manifest_path))
    base.require(manifest['story'] == 'P4-02', 'wrong story selection')
    require_implementation()
    run = ROOT / 'build/m4/P4-02' / ('acceptance-' + str(time.time_ns()))
    session = validation.Session(manifest, run, resume)
    selected, _ = validation.schedule(manifest)
    try:
        for key in ('kernel-host', 'desktop-host'):
            if key in selected:
                args, limit = validation.catalog()[key]
                session.check(key, args, limit)
        for repetition in range(5):
            text, _ = session.check('desktop-' + str(repetition),
                                   ['cargo', 'xtask', 'test', '--suite', 'm1', '--case', 'desktop-runtime'],
                                   600, lambda text: guest_review(text, True))
            if repetition == 0:
                path = validation.artifact_path(text, 'M1 artifacts: ')
                output = run / 'flavors'
                session.check('flavors', validation.PS + ['tools/Check-P4-02Flavors.ps1',
                              '-Descriptors', str(path), '-Output', str(output)], 600,
                              lambda text: flavor_review(output))
        session.check('ordinary', ['cargo', 'xtask', 'run', '--profile', 'vm-baseline'],
                      600, lambda text: guest_review(text, False))
        session.selected()
        session.finish(expected_checks(manifest))
    except BaseException as error:
        session.report.update(status='interrupted' if isinstance(error, KeyboardInterrupt) else 'failed', error=str(error))
        if isinstance(error, base.CommandFailure):
            session.report['commands'].append(error.result)
        session.persist()
        raise
    finally:
        print('P4-02 acceptance: ' + str(run / 'acceptance.json'), flush=True)


def review(path):
    path = Path(path).resolve()
    original = base.read(path)
    base.require(original['story'] == 'P4-02', 'wrong story')
    report = validation.review_report(path, expected_checks(original['manifest']))
    for check in report['checks']:
        key = check['id']
        text = Path(check['stdout']).read_text(encoding='utf-8-sig')
        if key.startswith('desktop-') and key != 'desktop-host':
            base.require(check['arguments'] == ['cargo', 'xtask', 'test', '--suite', 'm1', '--case', 'desktop-runtime']
                         and check['deadline_seconds'] == 600, 'substituted desktop guest')
            guest_review(text, True)
        elif key == 'ordinary':
            base.require(check['arguments'] == ['cargo', 'xtask', 'run', '--profile', 'vm-baseline']
                         and check['deadline_seconds'] == 600, 'substituted ordinary boot')
            guest_review(text, False)
        elif key == 'flavors':
            output = Path(check['review']['directory'])
            first = next(c for c in report['checks'] if c['id'] == 'desktop-0')
            descriptors = validation.artifact_path(Path(first['stdout']).read_text(encoding='utf-8-sig'), 'M1 artifacts: ')
            base.require(check['arguments'] == validation.PS + ['tools/Check-P4-02Flavors.ps1',
                         '-Descriptors', str(descriptors), '-Output', str(output)]
                         and check['deadline_seconds'] == 600, 'substituted flavor command')
            flavor_review(output)
        else:
            validation.review_check(key, text, path.parent)
    result = {'schema_version': 2, 'story': 'P4-02', 'status': 'passed', 'scope': 'story',
              'review': 'local Codex self-review; not external approval or milestone acceptance',
              'acceptance_sha256': base.digest(path), 'guest_repetitions': 5,
              'metadata_flavors': 16, 'ordinary_without_acceptance_observers': True,
              'selected_regressions': [e['id'] for e in report['manifest']['selected']],
              'not_selected': report['not_selected'], 'covered_by': report['covered_by']}
    base.save(path.parent / 'review.json', result)
    print(json.dumps(result, indent=2))
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['run', 'review', 'plan'])
    parser.add_argument('--manifest', type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument('--resume', type=Path)
    parser.add_argument('--acceptance', type=Path)
    args = parser.parse_args()
    if args.action == 'run':
        run_gate(args.manifest, args.resume)
    elif args.action == 'plan':
        manifest = validation.validate_manifest(base.read(args.manifest))
        print(json.dumps({'scope': 'story', 'checks': expected_checks(manifest),
                          'covered_by': validation.schedule(manifest)[1], 'impact': manifest['impact']}, indent=2))
    else:
        base.require(args.acceptance, 'review requires acceptance path')
        review(args.acceptance)
