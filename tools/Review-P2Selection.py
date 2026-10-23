"""Recheck P2-09 selection against retained evidence; no build/guest/legal approval."""
import copy
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
SELECTION = ROOT / 'legal/ui-selection.json'


def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def pinned_file(row):
    path = (ROOT / row['path']).resolve()
    require(path.is_relative_to(ROOT), 'path outside workspace')
    require(path.is_file(), 'missing pin: ' + row['path'])
    require(row.get('normalization') in [None, 'lf'], 'unknown pin normalization')
    raw = path.read_bytes()
    if row.get('normalization') == 'lf':
        raw = raw.replace(b'\r\n', b'\n')
    require(hashlib.sha256(raw).hexdigest() == row['sha256'], 'pin: ' + row['path'])


def paths_under(bases, suffixes=None):
    return {p.relative_to(ROOT).as_posix() for base in bases for p in (ROOT / base).rglob('*')
            if p.is_file() and (suffixes is None or p.suffix in suffixes)}


def validate(selection, baseline):
    require(selection['schema_version'] == 1, 'unknown selection version')
    require(selection['decision'] == 'provisional-custom', 'unreviewed toolkit')
    require(selection['adr_status'] == 'Proposed', 'unpassed ADR gate')
    for key in ['distribution_cleared', 'guest_confirmed', 'm2_complete']:
        require(selection[key] is False, 'unpassed gate: ' + key)
    require(selection['target'] == 'x86_64-unknown-none', 'target mismatch')
    require(selection['default_features'] == [], 'unreviewed default feature')
    require(selection['compiler'] == 'rustc 1.99.1 (58cf8f9c3 2026-10-13)', 'compiler mismatch')
    require(selection['profiles'] == {'host': 'm2-comparison', 'target': 'ui-link release opt-level=1 panic=abort codegen-units=1 no-LTO'}, 'profile mismatch')
    linked = next(p for p in read(ROOT / 'evidence/phase-2/P2-07/2026-10-22/acceptance.json')['artifacts'] if p['candidate'] == 'custom')
    require(selection['linked_artifact'] == linked, 'linked artifact mismatch')
    require(selection['next_story'] == 'P2-10', 'missing next bounded story')
    require(selection['completion_gates'] == ['P2-10', 'P2-11', 'P2-12', 'P2-13', 'M3', 'M4', 'M5'], 'missing completion gate')
    actual = {p['name']: p for p in selection['packages']}
    expected = {p['name']: p for p in baseline['candidates']['custom']['packages']}
    require(len(actual) == len(selection['packages']) == len(expected) == 32, 'package closure')
    require(actual.keys() == expected.keys(), 'different package closure')
    for name, row in actual.items():
        old = expected[name]
        for key in ['version', 'normal_features', 'build_features', 'registry_checksum', 'license']:
            require(row[key] == old.get(key), name + ': changed ' + key)
        choice = ('LicenseRef-ContinuumOS-Proprietary' if not old.get('registry_checksum') else
                  'Apache-2.0' if name == 'unicode-linebreak' else 'Zlib' if name == 'foldhash' else 'MIT')
        require(row['selected_license'] == choice, name + ': unreviewed terms')
        prefix = 'licenses/third-party/' + name + '-' + row['version'] + '/'
        notices = [{'path': prefix + n['path'], 'sha256': n['sha256']} for n in old.get('notices', [])]
        require(row['notices'] == notices, name + ': incomplete notices')
    expected_pins = {
        'input_pins': {'.cargo/config.toml', 'Cargo.toml', 'Cargo.lock', 'tools/xtask/Cargo.toml',
                       'experiments/ui-link/Cargo.lock', 'config/m2-profiles.json', 'config/p2-08-comparison.json',
                       'fixtures/m2/ui-v1/lock.json', 'fixtures/m2/text-v1/lock.json',
                       'fixtures/m2/text-v1/fonts.lock.json', 'fixtures/m2/text-v1/data.lock.json'},
        'chosen_source': paths_under(['crates/custom-ui', 'crates/text-core', 'crates/editor-model', 'experiments/ui-link'], {'.rs', '.toml', '.lock', '.ld'}),
        'frozen_comparison': paths_under(['crates/slint-ui', 'fixtures/m2/slint-v1']) | {
            'tools/Preview-Slint.ps1', 'tools/xtask/src/slint_candidate.rs', 'tools/Test-P2-06.ps1',
            'tools/Review-Slint.py', 'tools/Review-SlintInputs.py'},
        'additional_notices': {'licenses/third-party/noto-fonts/OFL.txt', 'licenses/third-party/noto-cjk/OFL.txt', 'licenses/third-party/unicode-data/LICENSE.txt'},
        'upstream_evidence': {'evidence/phase-2/P2-07/2026-10-22/SHA256SUMS.json', 'evidence/phase-2/P2-08/2026-10-22/SHA256SUMS.json'},
    }
    for key, expected_paths in expected_pins.items():
        require(selection[key], 'missing ' + key)
        require(len(selection[key]) == len(expected_paths) and {r['path'] for r in selection[key]} == expected_paths, 'incomplete pins: ' + key)
        for row in selection[key]:
            pinned_file(row)
    for row in actual.values():
        for notice in row['notices']:
            pinned_file(notice)
    require(selection['fonts'] == read(ROOT / 'fixtures/m2/text-v1/fonts.lock.json')['fonts'], 'font identity/order')
    for row in selection['fonts']:
        pinned_file(row)
        require((ROOT / row['path']).stat().st_size == row['bytes'], 'font length')


def main():
    require(len(sys.argv) == 3, 'usage: Review-P2Selection.py <prerequisites.json> <output-dir>')
    prereq_path, output = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
    require(output.is_relative_to(ROOT / 'build') or output.is_relative_to(ROOT / 'work'), 'output must stay local under build/ or work/')
    selection = read(SELECTION)
    baseline = read(ROOT / 'evidence/phase-2/P2-07/2026-10-22/inputs.json')
    upstream = []
    for manifest in selection['upstream_evidence']:
        pinned_file(manifest)
        directory = (ROOT / manifest['path']).parent
        rows = read(directory / 'SHA256SUMS.json')['files']
        for row in rows:
            pinned_file({'path': (directory / row['path']).relative_to(ROOT).as_posix(), 'sha256': row['sha256']})
        upstream.append({'manifest': manifest['path'], 'verified_files': len(rows)})
    validate(selection, baseline)
    audit = read(ROOT / 'evidence/phase-2/P2-07/2026-10-22/custom-elf-audit.json')
    require(audit['unresolved_symbols'] == audit['remaining_relocations'] == 0 and not audit['dynamic'], 'target closure')
    prior_source = read(ROOT / 'evidence/phase-2/P2-07/2026-10-22/compiled-inputs.json')['files']
    source_map = {row['path']: row['sha256'] for row in prior_source}
    for row in selection['chosen_source']:
        require(row['path'] in source_map, 'source missing from linked evidence: ' + row['path'])
        raw = (ROOT / row['path']).read_bytes()
        lf = raw.replace(b'\r\n', b'\n')
        variants = {hashlib.sha256(b).hexdigest() for b in [raw, lf, lf.replace(b'\n', b'\r\n')]}
        require(source_map[row['path']] in variants, 'selected source changed since final link: ' + row['path'])
    prereq = read(prereq_path)
    expected_commands = {'signing', 'compiler', 'host-metadata', 'doctor', 'doctor-json', 'build', 'm0', 'acceptance', 'dependencies', 'm1-doctor'}
    require(prereq['status'] == 'passed' and prereq['offline'] is True, 'prerequisites failed')
    commands = prereq['commands']
    require(len(commands) == 10 and {r['name'] for r in commands} == expected_commands, 'prerequisite coverage')
    require(all(r['exit_code'] == 0 for r in commands), 'prerequisite command failure')
    require(prereq['signing']['status'] == 'passed', 'signing failed')
    negatives = []
    mutations = [
        ('unknown version', lambda d: d.update(schema_version=2)),
        ('unreviewed toolkit', lambda d: d.update(decision='slint')),
        ('false ADR acceptance', lambda d: d.update(adr_status='Accepted')),
        ('false distribution clearance', lambda d: d.update(distribution_cleared=True)),
        ('false guest confirmation', lambda d: d.update(guest_confirmed=True)),
        ('false M2 completion', lambda d: d.update(m2_complete=True)),
        ('host target', lambda d: d.update(target='x86_64-pc-windows-msvc')),
        ('new feature', lambda d: d.update(default_features=['slint-candidate'])),
        ('wrong compiler', lambda d: d.update(compiler='unreviewed compiler')),
        ('wrong profile', lambda d: d['profiles'].update(host='debug')),
        ('wrong linked artifact', lambda d: d['linked_artifact'].update(sha256='0'*64)),
        ('missing M2 gate', lambda d: d['completion_gates'].remove('P2-11')),
        ('missing package', lambda d: d['packages'].pop()),
        ('wrong feature graph', lambda d: d['packages'][0].update(normal_features=[])),
        ('wrong license', lambda d: d['packages'][0].update(selected_license='GPL-3.0-only')),
        ('missing notice', lambda d: d['packages'][0]['notices'].pop()),
        ('changed frozen source', lambda d: d['frozen_comparison'][0].update(sha256='0'*64)),
        ('unsupported pin normalization', lambda d: d['frozen_comparison'][0].update(normalization='ignore')),
        ('missing frozen source', lambda d: d['frozen_comparison'].pop()),
        ('missing selected source', lambda d: d['chosen_source'].pop()),
        ('missing lock pin', lambda d: d['input_pins'].pop()),
        ('wrong font', lambda d: d['fonts'][0].update(sha256='0'*64)),
    ]
    for name, mutate in mutations:
        changed = copy.deepcopy(selection)
        mutate(changed)
        try:
            validate(changed, baseline)
        except ValueError as error:
            negatives.append({'case': name, 'rejected': True, 'reason': str(error)})
        else:
            raise ValueError('mutation accepted: ' + name)
    output.mkdir(parents=True, exist_ok=True)
    report = {
        'story': 'P2-09', 'status': 'passed', 'scope': 'Identity/closure/notice checks, not implementation or legal approval',
        'selection_sha256': digest(SELECTION), 'reviewer_sha256': digest(__file__),
        'linked_artifact': selection['linked_artifact'],
        'prerequisites_sha256': digest(prereq_path), 'prerequisite_commands': commands,
        'upstream_manifests': upstream, 'active_packages': 32, 'third_party_packages': 28,
        'source_match': 'P2-07 chosen files unchanged modulo CRLF/LF; no new target execution',
        'selected_source_files': len(selection['chosen_source']), 'frozen_comparison_files': len(selection['frozen_comparison']),
        'fonts_verified': len(selection['fonts']), 'negative_cases': negatives,
        'guest_executed': False, 'new_measurements': False, 'distribution_cleared': False,
    }
    (output / 'checks.json').write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    print(json.dumps({'status': 'passed', 'upstream': upstream, 'negative_cases': len(negatives), 'output': str(output)}))


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, OSError, TypeError) as error:
        sys.exit('P2-09 review failed: ' + str(error))
