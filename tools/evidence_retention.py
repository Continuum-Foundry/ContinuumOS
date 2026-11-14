"""Bounded evidence capsules. No deletion, extraction, guest execution or acceptance.

Explicit file selection is an engineering review, never inferred from extensions.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import zipfile

ROOT = Path(__file__).resolve().parents[1]
LEGACY_STORIES = {'P4-02', 'P4-03', 'P4-04', 'validation-policy'}
KINDS = {'raw', 'review', 'provenance', 'failure', 'artifact'}


def need(condition, message):
    if not condition:
        raise ValueError(message)


def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))


def fingerprint(path):
    sha = hashlib.sha256()
    size = 0
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            sha.update(block)
            size += len(block)
    return sha.hexdigest(), size


def policy(manifest):
    version = manifest['schema_version']
    need(type(version) is int and version in (1, 2), 'unsupported story schema')
    if version == 1:
        need(manifest['story'] in LEGACY_STORIES,
             'new stories require schema_version 2 and a retention plan')
        return None
    value = manifest.get('retention', {})
    for key in ('working_set_budget_bytes', 'capsule_budget_bytes'):
        need(type(value.get(key)) is int and value[key] > 0, 'invalid retention ' + key)
    for key in ('cleanup_after', 'required_evidence', 'source_reconstruction'):
        need(isinstance(value.get(key), str) and bool(value[key].strip()), 'missing retention ' + key)
    # Budgets are selected before execution, not silently enlarged at closeout.
    return value


def storage(paths, budget):
    """Count actual path bytes (duplicate copies count); hashes alone cost no payload."""
    files = {Path(p).resolve() for p in paths}
    need(all(p.is_file() for p in files), 'working payload missing; cannot measure/reuse it')
    size = sum(p.stat().st_size for p in files)
    need(size <= budget, f'working evidence budget exceeded: {size} > {budget}')
    return {'files': len(files), 'bytes': size, 'budget_bytes': budget,
            'scope': 'journal and referenced check artifacts; shared caches/toolchains excluded'}


def local_file(name):
    path = (ROOT / name).resolve()
    need(path.is_relative_to(ROOT) and path.is_file(), 'evidence file outside workspace or missing')
    # Do not follow links/junctions outside the workspace (resolve above).
    return path


def validate_index(index):
    need(index['schema_version'] == 1 and index['scope'] == 'historical-evidence-capsule', 'wrong capsule scope')
    need(index['availability'] == 'selected-files-only; original payload required for fresh review/resume', 'wrong availability')
    need(type(index['budget_bytes']) is int and index['budget_bytes'] > 0, 'invalid capsule budget')
    need(index['source_reconstruction'].strip() and index['cleanup_after'].strip(), 'missing retention rationale')
    checks = index['checks']
    need(checks and len(checks) == len(set(checks)), 'empty/duplicate journal checks')
    covered = set()
    kinds = set()
    names = set()
    for row in index['files']:
        need(row['kind'] in KINDS and row['reason'].strip(), 'unexplained evidence selection')
        need(row['source'] not in names, 'duplicate selected file')
        names.add(row['source'])
        need(set(row['checks']) <= set(checks), 'unknown selected check')
        need(type(row['bytes']) is int and row['bytes'] >= 0, 'invalid evidence length')
        need(len(row['sha256']) == 64 and all(c in '0123456789abcdef' for c in row['sha256']), 'invalid evidence hash')
        if row['kind'] == 'raw':
            need(row.get('complete') is True, 'raw evidence must preserve a complete selected stream/sample file')
            covered.update(row['checks'])
        if row['kind'] == 'artifact':
            need(bool(row.get('artifact_reason', '').strip()), 'binary retention needs an explicit necessity')
        kinds.add(row['kind'])
    need(covered == set(checks), 'each executed check needs raw evidence')
    need({'raw', 'review', 'provenance'} <= kinds, 'missing raw/review/provenance evidence')
    need(index['failure_disposition'].strip(), 'record failure evidence or explain why none exists')
    need(index['journal_sha256'] in {r['sha256'] for r in index['files'] if r['kind'] == 'review'}, 'journal missing from capsule')


def bind_journal(index, journal):
    need(journal['story'] == index['story'] and journal['status'] == index['recorded_status']
         and [r['id'] for r in journal['checks']] == index['checks'], 'capsule journal mismatch')
    plan = policy(journal['manifest'])
    need(plan is not None and plan['capsule_budget_bytes'] == index['budget_bytes']
         and all(index[k] == plan[k] for k in ('cleanup_after', 'source_reconstruction', 'required_evidence')),
         'capsule differs from predeclared retention plan')
    checks = {c['id']: c for c in journal['checks']}
    for key, check in checks.items():
        if 'stdout' in check:
            # Keep both host streams, including stderr-only unit-test results.
            # Normalize separators so a capsule remains verifiable on another host.
            directory = check['stdout'].replace('\\', '/').rsplit('/', 1)[0]
            hashes = {p.replace('\\', '/'): sha for p, sha in check['artifact_hashes'].items()}
            retained = {r['sha256'] for r in index['files'] if r['kind'] == 'raw' and key in r['checks']}
            for suffix in ('.stdout.log', '.stderr.log'):
                path = directory + '/' + key + suffix
                need(path in hashes and hashes[path] in retained, 'missing complete command output stream')
    for row in index['files']:
        if row['kind'] != 'raw':
            continue
        for key in row['checks']:
            # Check bytes were actually bound into the execution journal.
            need(row['sha256'] in checks[key]['artifact_hashes'].values(), 'raw evidence not bound to executed check')


def selection(spec):
    journal_path = local_file(spec['acceptance'])
    journal = read(journal_path)
    need(journal['status'] in ('passed', 'failed', 'interrupted'), 'cannot compact a running journal')
    retained_policy = policy(journal['manifest'])
    need(retained_policy is not None, 'legacy journals require a separately reviewed migration; leave originals unchanged')
    rows = []
    for item in spec['files']:
        path = local_file(item['source'])
        sha, size = fingerprint(path)
        rows.append({**item, 'source': path.relative_to(ROOT).as_posix(), 'sha256': sha, 'bytes': size})
    sha, size = fingerprint(journal_path)
    rows.append({'source': journal_path.relative_to(ROOT).as_posix(), 'sha256': sha, 'bytes': size,
                 'kind': 'review', 'checks': [], 'reason': 'Original immutable execution/review journal.'})
    index = {'schema_version': 1, 'scope': 'historical-evidence-capsule', 'story': journal['story'],
             'recorded_status': journal['status'], 'journal_sha256': sha,
             'availability': 'selected-files-only; original payload required for fresh review/resume',
             'checks': [r['id'] for r in journal['checks']], 'files': rows,
             'budget_bytes': retained_policy['capsule_budget_bytes'],
             'cleanup_after': retained_policy['cleanup_after'],
             'source_reconstruction': retained_policy['source_reconstruction'],
             'required_evidence': retained_policy['required_evidence'],
             'failure_disposition': spec['failure_disposition']}
    validate_index(index)
    bind_journal(index, journal)
    return index


def create(spec, destination):
    index = selection(spec)
    destination = Path(destination).resolve()
    need(destination.is_relative_to(ROOT / 'work') or destination.is_relative_to(ROOT / 'build'),
         'capsules must stay in ignored workspace work/ or build/')
    need(not destination.exists(), 'capsule already exists; never overwrite evidence')
    destination.parent.mkdir(parents=True, exist_ok=True)
    # Exclusive create; a failed/oversized capsule remains for diagnosis, never a pass.
    with zipfile.ZipFile(destination, 'x', compression=zipfile.ZIP_DEFLATED, compresslevel=6) as archive:
        archive.writestr('retention.json', json.dumps(index, indent=2) + '\n')
        copied = set()
        for row in index['files']:
            if row['sha256'] not in copied:
                archive.write(local_file(row['source']), 'blobs/' + row['sha256'])
                copied.add(row['sha256'])
                need(archive.fp.tell() <= index['budget_bytes'], 'compressed capsule budget exceeded')
    return verify(destination)


def verify(path):
    """Verify retained bytes without opening original paths or claiming acceptance."""
    path = Path(path)
    with zipfile.ZipFile(path) as archive:
        index = json.loads(archive.read('retention.json'))
        validate_index(index)
        expected = {'retention.json', *['blobs/' + r['sha256'] for r in index['files']]}
        need(len(archive.namelist()) == len(expected) and set(archive.namelist()) == expected, 'unexpected/duplicate capsule members')
        lengths = {}
        for row in index['files']:
            sha = row['sha256']
            if sha not in lengths:
                observed = hashlib.sha256()
                size = 0
                with archive.open('blobs/' + sha) as stream:
                    for block in iter(lambda: stream.read(1024 * 1024), b''):
                        observed.update(block)
                        size += len(block)
                need(observed.hexdigest() == sha, 'changed capsule bytes')
                lengths[sha] = size
            need(lengths[sha] == row['bytes'], 'changed capsule length')
        journal = json.loads(archive.read('blobs/' + index['journal_sha256']))
        bind_journal(index, journal)
    need(path.stat().st_size <= index['budget_bytes'], 'compressed capsule budget exceeded')
    return {'status': 'integrity-verified', 'scope': index['scope'], 'story': index['story'],
            'recorded_status': index['recorded_status'], 'files': len(index['files']),
            'unique_blobs': len(lengths), 'compressed_bytes': path.stat().st_size,
            'availability': index['availability'], 'acceptance_revalidated': False,
            'originals_deleted': False}


def cleanup_plan(spec):
    """Preview a required post-merge cleanup; never remove or modify a file.

    Merge/consumer declarations require engineering review. This offline check
    does not query GitHub or discover other chats' active artifact dependencies.
    """
    merge = spec['merge']
    need(merge.get('state') == 'merged' and merge.get('merged_at')
         and re.fullmatch(r'[0-9a-f]{40}', merge.get('commit', ''))
         and merge.get('pr_url') and merge.get('verification_source'),
         'cleanup requires recorded verified PR merge, not merely an open/closed PR')
    capsule = local_file(spec['capsule'])
    need(fingerprint(capsule)[0] == spec['capsule_sha256'], 'capsule identity changed')
    verified = verify(capsule)
    need(spec.get('evidence_sufficiency_review', '').strip(), 'missing evidence sufficiency review')
    # Verify the compact bundle separately; the local capsule is not a Git member.
    manifest = local_file(spec['bundle_manifest'])
    protected = {capsule, manifest}
    for row in read(manifest)['files']:
        path = (manifest.parent / row['path']).resolve()
        need(path.is_relative_to(manifest.parent), 'bundle member escapes bundle')
        sha, size = fingerprint(path)
        need(sha == row['sha256'] and size == row['bytes'], 'compact evidence bundle changed')
        protected.add(path)
    need(len(protected) > 2, 'compact evidence bundle must contain evidence')
    rows, seen = [], set()
    for item in spec['files']:
        path = local_file(item['path'])
        need(path not in seen, 'duplicate cleanup candidate')
        seen.add(path)
        relative = path.relative_to(ROOT).as_posix()
        need(relative.startswith(('work/', 'build/')), 'cleanup candidates must be local work/build files')
        disposition = item['disposition']
        need(disposition in ('remove-after-merge', 'keep') and item['reason'].strip(), 'unexplained cleanup disposition')
        need(isinstance(item['consumers'], list), 'declare release/debug/review/resume consumers explicitly')
        if disposition == 'remove-after-merge':
            need(not item['consumers'], 'artifact still needed by a live consumer; keep it')
            need(path not in protected, 'cannot remove verified capsule or compact evidence')
            need(not relative.startswith(('work/tools/', 'work/toolchains/', 'build/cargo/',
                                          'work/windows-dev-signing/', 'build/windows-dev-signing/')),
                 'shared tools/caches/signing material require separate maintenance')
        sha, size = fingerprint(path)
        rows.append({**item, 'path': str(path), 'sha256': sha, 'bytes': size})
    need(rows, 'empty cleanup inventory; record an explicit no-artifacts disposition separately')
    return {'scope': 'post-merge-cleanup-preview', 'status': 'cleanup-required',
            'merge': merge, 'story': verified['story'], 'files': rows,
            'candidate_bytes': sum(r['bytes'] for r in rows if r['disposition'] == 'remove-after-merge'),
            'retained_bytes': sum(r['bytes'] for r in rows if r['disposition'] == 'keep'),
            'deletion_performed': False, 'cleanup_completed': False,
            'limits': 'Offline merge/consumer declarations need review; recheck hashes and live consumers before any later deletion.'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=('plan', 'pack', 'verify', 'cleanup-plan'))
    parser.add_argument('--selection')
    parser.add_argument('--capsule')
    args = parser.parse_args()
    if args.action == 'cleanup-plan':
        result = cleanup_plan(read(args.selection))
    elif args.action == 'verify':
        result = verify(args.capsule)
    elif args.action == 'pack':
        result = create(read(args.selection), args.capsule)
    else:
        result = selection(read(args.selection))
        result['selected_bytes_before_compression'] = sum(r['bytes'] for r in result['files'])
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
