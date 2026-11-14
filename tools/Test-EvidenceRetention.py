"""Retention integrity, budgets and non-destructive behavior; no VM/toolchain."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile

import evidence_retention as r


def manifest():
    return {'schema_version': 2, 'story': 'future-story', 'retention': {
        'working_set_budget_bytes': 1024 * 1024, 'capsule_budget_bytes': 128 * 1024,
        'cleanup_after': 'After closeout and separately reviewed cleanup.',
        'required_evidence': 'Full outputs, review, provenance and failure disposition.',
        'source_reconstruction': 'Commit plus exact patch and untracked-input archive.'}}


class RetentionTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.mock = patch.object(r, 'ROOT', self.root)
        self.mock.start()
        (self.root / 'work').mkdir()
        (self.root / 'work/raw.log').write_text('complete samples\n' * 100)
        (self.root / 'work/provenance.patch').write_text('exact synthetic input patch')
        raw = r.fingerprint(self.root / 'work/raw.log')[0]
        self.journal = {'story': 'future-story', 'status': 'passed', 'manifest': manifest(),
                        'checks': [{'id': 'probe', 'artifact_hashes': {'original/raw.log': raw}}]}
        self.write_journal()
        self.spec = {'acceptance': 'work/acceptance.json', 'failure_disposition': 'No failed attempts in this synthetic case.',
                     'files': [{'source': 'work/raw.log', 'kind': 'raw', 'checks': ['probe'], 'complete': True,
                                'reason': 'All criterion samples and sequence records.'},
                               {'source': 'work/provenance.patch', 'kind': 'provenance', 'checks': [],
                                'reason': 'Exact tested dirty inputs.'}]}

    def tearDown(self):
        self.mock.stop()
        self.temp.cleanup()

    def write_journal(self):
        (self.root / 'work/acceptance.json').write_text(json.dumps(self.journal))

    def test_new_stories_cannot_silently_use_unbounded_legacy_policy(self):
        m = manifest()
        for key in m['retention']:
            changed = copy.deepcopy(m)
            changed['retention'].pop(key)
            with self.assertRaises(ValueError): r.policy(changed)
        m['schema_version'] = 1
        with self.assertRaisesRegex(ValueError, 'new stories'): r.policy(m)
        m['story'] = 'P4-04'
        self.assertIsNone(r.policy(m))
        m = manifest()
        m['retention']['working_set_budget_bytes'] = True
        with self.assertRaises(ValueError): r.policy(m)

    def test_working_budget_counts_copies_not_hashes_and_missing_artifacts_fail(self):
        a = self.root / 'work/a'; b = self.root / 'work/b'
        a.write_bytes(b'123'); b.write_bytes(b'123')
        self.assertEqual(r.storage([a, a, b], 6)['bytes'], 6)
        with self.assertRaisesRegex(ValueError, 'budget exceeded'): r.storage([a, b], 5)
        with self.assertRaisesRegex(ValueError, 'missing'): r.storage([a / 'missing'], 100)

    def test_capsule_is_portable_deduplicated_and_preserves_originals(self):
        original = {p: p.read_bytes() for p in (self.root / 'work').iterdir()}
        duplicate = self.root / 'work/duplicate.log'
        duplicate.write_bytes((self.root / 'work/raw.log').read_bytes())
        self.spec['files'].append({**self.spec['files'][0], 'source': 'work/duplicate.log'})
        capsule = self.root / 'work/capsule.zip'
        result = r.create(self.spec, capsule)
        self.assertEqual(result['unique_blobs'], result['files'] - 1)
        self.assertFalse(result['acceptance_revalidated'])
        for p, data in original.items(): self.assertEqual(p.read_bytes(), data)
        # Verification deliberately cannot access originals; it uses archived bytes.
        with patch.object(r, 'local_file', side_effect=AssertionError('opened original')):
            self.assertEqual(r.verify(capsule)['status'], 'integrity-verified')
        with self.assertRaisesRegex(ValueError, 'overwrite'): r.create(self.spec, capsule)

    def test_incomplete_unknown_unbound_and_missing_evidence_rejected(self):
        for mutate in (
            lambda s: s['files'][0].update(complete=False),
            lambda s: s['files'][0].update(checks=['invented']),
            lambda s: s['files'][0].update(source='work/provenance.patch'),
            lambda s: s['files'].pop(0),
            lambda s: s['files'].pop(1),
            lambda s: s.update(failure_disposition=''),
            lambda s: s['files'][0].update(source='../outside'),
        ):
            changed = copy.deepcopy(self.spec); mutate(changed)
            with self.assertRaises(ValueError): r.selection(changed)

    def test_real_command_requires_both_stdout_and_stderr(self):
        self.journal['checks'][0]['stdout'] = 'original/probe.stdout.log'
        sha = r.fingerprint(self.root / 'work/raw.log')[0]
        stderr = self.root / 'work/stderr.log'
        stderr.write_text('essential test results on stderr')
        self.journal['checks'][0]['artifact_hashes'] = {
            'original/probe.stdout.log': sha,
            'original/probe.stderr.log': r.fingerprint(stderr)[0]}
        self.write_journal()
        with self.assertRaisesRegex(ValueError, 'output stream'): r.selection(self.spec)
        self.spec['files'].append({'source': 'work/stderr.log', 'kind': 'raw',
                                   'checks': ['probe'], 'complete': True, 'reason': 'Full diagnostics.'})
        r.selection(self.spec)

    def test_budget_status_and_original_journal_are_not_rewritten(self):
        self.journal['status'] = 'failed'
        self.journal['manifest']['retention']['capsule_budget_bytes'] = 1
        self.write_journal()
        original = (self.root / 'work/acceptance.json').read_bytes()
        path = self.root / 'work/oversized.zip'
        with self.assertRaisesRegex(ValueError, 'budget exceeded'): r.create(self.spec, path)
        self.assertTrue(path.exists())
        self.assertEqual((self.root / 'work/acceptance.json').read_bytes(), original)
        self.journal['status'] = 'running'; self.write_journal()
        with self.assertRaisesRegex(ValueError, 'running'): r.selection(self.spec)

    def test_changed_bytes_and_extra_archive_members_rejected(self):
        path = self.root / 'work/capsule.zip'
        r.create(self.spec, path)
        with zipfile.ZipFile(path, 'a') as archive: archive.writestr('../unexpected', 'bad')
        with self.assertRaisesRegex(ValueError, 'members'): r.verify(path)
        clean = self.root / 'work/clean.zip'; r.create(self.spec, clean)
        corrupt = self.root / 'work/corrupt.zip'
        with zipfile.ZipFile(clean) as source, zipfile.ZipFile(corrupt, 'w') as target:
            for name in source.namelist():
                target.writestr(name, b'changed' if name.startswith('blobs/') else source.read(name))
        with self.assertRaisesRegex(ValueError, 'changed capsule'): r.verify(corrupt)

    def test_artifact_exception_and_destination_boundaries(self):
        self.spec['files'].append({'source': 'work/raw.log', 'kind': 'artifact', 'checks': [], 'reason': 'debug'})
        with self.assertRaises(ValueError): r.selection(self.spec)
        self.spec['files'].pop()
        with self.assertRaisesRegex(ValueError, 'ignored workspace'): r.create(self.spec, self.root / 'tracked.zip')

    def cleanup_spec(self):
        capsule = self.root / 'work/capsule.zip'
        r.create(self.spec, capsule)
        bundle = self.root / 'evidence/example'
        bundle.mkdir(parents=True)
        member = bundle / 'review.json'; member.write_text('{"status":"reviewed"}')
        sha, size = r.fingerprint(member)
        (bundle / 'SHA256SUMS.json').write_text(json.dumps({'files': [
            {'path': 'review.json', 'sha256': sha, 'bytes': size}]}))
        (self.root / 'work/temporary.iso').write_bytes(b'temporary tested ISO')
        (self.root / 'work/release.iso').write_bytes(b'exact release ISO')
        return {'merge': {'state': 'merged', 'pr_url': 'https://example.test/pull/1',
                          'commit': 'a' * 40, 'merged_at': '2026-11-14T12:00:00Z',
                          'verification_source': 'Synthetic merge fixture; no external PR claim.'},
                'capsule': 'work/capsule.zip', 'capsule_sha256': r.fingerprint(capsule)[0],
                'bundle_manifest': 'evidence/example/SHA256SUMS.json',
                'evidence_sufficiency_review': 'Fixture contains complete required evidence.',
                'files': [{'path': 'work/temporary.iso', 'disposition': 'remove-after-merge',
                           'reason': 'Temporary ISO; all validation/review consumers finished.', 'consumers': []},
                          {'path': 'work/release.iso', 'disposition': 'keep',
                           'reason': 'Exact release payload; revisit at release retirement.', 'consumers': ['release-1']}]}

    def test_post_merge_cleanup_preview_preserves_every_file(self):
        spec = self.cleanup_spec()
        before = {p: p.read_bytes() for p in self.root.rglob('*') if p.is_file()}
        result = r.cleanup_plan(spec)
        self.assertEqual(result['status'], 'cleanup-required')
        self.assertFalse(result['deletion_performed'])
        self.assertFalse(result['cleanup_completed'])
        self.assertGreater(result['candidate_bytes'], 0)
        self.assertGreater(result['retained_bytes'], 0)
        self.assertEqual(before, {p: p.read_bytes() for p in self.root.rglob('*') if p.is_file()})

    def test_cleanup_requires_merge_verified_evidence_and_no_live_consumer(self):
        spec = self.cleanup_spec()
        for mutate in (
            lambda s: s['merge'].update(state='open'),
            lambda s: s['merge'].update(state='closed'),
            lambda s: s['merge'].update(commit=''),
            lambda s: s.update(capsule_sha256='0' * 64),
            lambda s: s.update(evidence_sufficiency_review=''),
            lambda s: s['files'][0].update(consumers=['debug-issue-1']),
            lambda s: s['files'][0].update(path='work/capsule.zip'),
            lambda s: s['files'][0].update(path='evidence/example/review.json'),
        ):
            changed = copy.deepcopy(spec); mutate(changed)
            with self.assertRaises(ValueError): r.cleanup_plan(changed)
        (self.root / 'evidence/example/review.json').write_text('changed')
        with self.assertRaisesRegex(ValueError, 'bundle changed'): r.cleanup_plan(spec)

    def test_cleanup_protects_shared_tools_and_duplicate_paths(self):
        spec = self.cleanup_spec()
        tools = self.root / 'work/tools'; tools.mkdir()
        (tools / 'pinned.iso').write_bytes(b'firmware fixture')
        spec['files'][0]['path'] = 'work/tools/pinned.iso'
        with self.assertRaisesRegex(ValueError, 'shared tools'): r.cleanup_plan(spec)
        spec['files'][0]['disposition'] = 'keep'
        r.cleanup_plan(spec)
        spec['files'].append(copy.deepcopy(spec['files'][0]))
        with self.assertRaisesRegex(ValueError, 'duplicate'): r.cleanup_plan(spec)


if __name__ == '__main__':
    unittest.main()
