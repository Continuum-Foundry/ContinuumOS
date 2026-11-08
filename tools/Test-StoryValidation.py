"""Synthetic orchestration/evidence tests; no compiler, signer or QEMU launches."""
import copy
import contextlib
import io
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import p3_acceptance as base
import story_validation as validation
import p4_runtime_acceptance as p4
import runtime_focus_review


def selection(story='policy-update', checks=('docs',)):
    return {'schema_version': 1, 'story': story, 'scope': 'story',
            'impact': {'subsystems': ['validation'], 'consumers': ['agents'],
                       'rationale': 'Focused tooling change.', 'broad_m1': False,
                       'broad_m2': False, 'broad_m3': False,
                       'shared_boundary_scheduler': False, 'uncertain_families': []},
            'selected': [{'id': key, 'reason': 'Affected behavior.'} for key in checks],
            'not_selected': {f: 'Unchanged unrelated subsystem.' for f in validation.FAMILIES},
            'historical_references': [],
            'execution_budget': {'max_vm_launches': 16, 'allow_full_suites': False}}


class SelectionTests(unittest.TestCase):
    def test_documentation_only_has_no_guest_or_tool_preflight(self):
        ids, covered = validation.schedule(selection())
        self.assertEqual(ids, ['docs'])
        self.assertEqual(covered, {})
        self.assertEqual(validation.catalog()['docs'][0], ['node', 'audit/check-docs.cjs'])

    def test_isolated_ui_has_no_m1_or_m3(self):
        ids, _ = validation.schedule(selection(checks=('ui-host', 'text-host')))
        self.assertEqual(ids, ['ui-host', 'text-host'])

    def test_broad_or_uncertain_requires_relevant_full_suite(self):
        for family in ('m1', 'm2', 'm3'):
            for uncertain in (False, True):
                manifest = selection()
                manifest['impact']['uncertain_families'] = [family.upper()] if uncertain else []
                manifest['impact']['broad_' + family] = not uncertain
                with self.assertRaisesRegex(ValueError, 'requires full-'):
                    validation.schedule(manifest)
                manifest['selected'].append({'id': 'full-' + family, 'reason': 'Broad dependency impact.'})
                self.assertIn('full-' + family, validation.schedule(manifest)[0])

    def test_full_m3_deduplicates_its_required_children(self):
        manifest = selection(checks=('prerequisites', 'kernel-host', 'memory', 'full-m1', 'full-m3'))
        ids, covered = validation.schedule(manifest)
        self.assertEqual(ids, ['full-m3'])
        self.assertEqual(covered['prerequisites'], 'full-m3')
        self.assertEqual(covered['kernel-host'], 'full-m1')

    def test_missing_reason_duplicate_unknown_scope_rejected(self):
        mutations = [lambda m: m.update(scope='milestone'),
                     lambda m: m['selected'].append(m['selected'][0]),
                     lambda m: m['selected'][0].update(id='skip-everything'),
                     lambda m: m['not_selected'].pop('M1'),
                     lambda m: m['selected'][0].update(reason='')]
        for mutate in mutations:
            manifest = selection()
            mutate(manifest)
            with self.assertRaises((ValueError, KeyError)):
                validation.validate_manifest(manifest)

    def test_p402_requires_hosts_and_shared_boundary_regressions(self):
        manifest = selection('P4-02')
        with self.assertRaisesRegex(ValueError, 'host tests'):
            validation.validate_manifest(manifest)
        manifest = selection('P4-02', ('kernel-host', 'desktop-host', 'boundary', 'scheduler', 'capabilities'))
        manifest['impact']['shared_boundary_scheduler'] = True
        validation.validate_manifest(manifest)
        manifest['selected'] = [e for e in manifest['selected'] if e['id'] != 'boundary']
        with self.assertRaisesRegex(ValueError, 'shared-boundary'):
            validation.validate_manifest(manifest)

    def test_actual_p402_selection_keeps_five_guests_and_ordinary(self):
        manifest = base.read(p4.DEFAULT_MANIFEST)
        checks = p4.expected_checks(manifest)
        self.assertEqual([c for c in checks if c.startswith('desktop-') and c != 'desktop-host'],
                         ['desktop-' + str(i) for i in range(5)])
        self.assertIn('ordinary', checks)
        self.assertIn('flavors', checks)
        self.assertNotIn('full-m3', checks)
        self.assertNotIn('full-m1', checks)
        self.assertIn('kernel-host', checks)
        self.assertTrue({'boundary', 'scheduler', 'capabilities', 'waits', 'sdk-runtime'} <= set(checks))

    def test_p402_adapter_requires_separate_implementation_before_execution(self):
        with tempfile.TemporaryDirectory() as folder, patch.object(p4, 'ROOT', Path(folder)), \
             patch.object(validation, 'Session') as session:
            with self.assertRaisesRegex(ValueError, 'implementation prerequisites missing'):
                p4.run_gate(p4.DEFAULT_MANIFEST)
            session.assert_not_called()


class BudgetTests(unittest.TestCase):
    def test_actual_plan_counts_every_nested_guest_before_execution(self):
        manifest = base.read(p4.DEFAULT_MANIFEST)
        plan = validation.execution_plan(manifest, p4.expected_checks(manifest))
        self.assertEqual(plan['vm_launches'], 16)
        self.assertEqual(plan['vm_launches_by_check']['boundary'], 4)
        self.assertEqual(plan['vm_launches_by_check']['scheduler'], 3)
        self.assertEqual(plan['separate_metadata_builds'], 16)
        self.assertEqual(plan['full_suites'], [])

    def test_ceiling_blocks_adapter_before_any_session_or_guest(self):
        manifest = base.read(p4.DEFAULT_MANIFEST)
        manifest['execution_budget']['max_vm_launches'] = 15
        with tempfile.TemporaryDirectory(dir=base.ROOT/'work') as folder:
            path = Path(folder)/'selection.json'
            base.save(path, manifest)
            with patch.object(p4, 'require_implementation'), patch.object(validation, 'Session') as session:
                with self.assertRaisesRegex(ValueError, 'budget exceeded'):
                    p4.run_gate(path)
                session.assert_not_called()

    def test_full_suite_cannot_sneak_into_bounded_story(self):
        for key in ('full-m1', 'full-m2', 'full-m3'):
            manifest = selection(checks=(key,))
            with self.assertRaisesRegex(ValueError, 'explicit budget opt-in'):
                validation.execution_plan(manifest)

    def test_unknown_full_count_must_be_explicit_and_unbounded(self):
        manifest = selection(checks=('full-m3',))
        manifest['execution_budget']['allow_full_suites'] = True
        with self.assertRaisesRegex(ValueError, 'unknown full-suite count'):
            validation.execution_plan(manifest)
        manifest['execution_budget'].update(max_vm_launches=None, full_suite_reason='Explicit broad enforcement change.')
        self.assertIsNone(validation.execution_plan(manifest)['vm_launches'])

    def test_guest_selection_requires_budget_but_prose_does_not(self):
        manifest = selection(checks=('boundary',))
        manifest.pop('execution_budget')
        with self.assertRaisesRegex(ValueError, 'explicit execution_budget'):
            validation.execution_plan(manifest)
        manifest['selected'] = [{'id': 'docs', 'reason': 'Prose only.'}]
        self.assertEqual(validation.execution_plan(manifest)['vm_launches'], 0)

    def test_new_unclassified_catalog_command_cannot_hide_launches(self):
        manifest = selection()
        with patch.object(validation, 'catalog', return_value={'docs': (['tool'], 1), 'future': (['guest-runner'], 1)}):
            manifest['selected'] = [{'id': 'future', 'reason': 'New command.'}]
            with self.assertRaisesRegex(ValueError, 'launch classification'):
                validation.execution_plan(manifest)

    def test_invalid_or_unexplained_unbounded_budget_rejects(self):
        for maximum in (-1, True, '16', None):
            manifest = selection()
            manifest['execution_budget']['max_vm_launches'] = maximum
            with self.assertRaises(ValueError):
                validation.execution_plan(manifest)


class JournalTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir=base.ROOT / 'work', prefix='story-validation-test-')
        self.root = Path(self.temp.name)
        self.sources = {'tracked': 'a', 'new-untracked': 'b', 'deleted': None}
        self.context = {'compiler': 'one', 'config': 'one'}
        self.calls = []
        self.sink = contextlib.redirect_stdout(io.StringIO())
        self.sink.__enter__()

    def tearDown(self):
        self.sink.__exit__(None, None, None)
        self.temp.cleanup()

    def session(self, name='run', resume=None):
        return validation.Session(selection(), self.root / name, resume,
                                  snapshot=lambda: copy.deepcopy(self.sources),
                                  context=lambda: copy.deepcopy(self.context))

    def execute(self, folder, key, args, limit):
        self.calls.append(key)
        (folder / (key + '.stdout.log')).write_text('synthetic success\n')
        return {'name': key, 'arguments': args, 'exit_code': 0, 'timed_out': False,
                'cleanup_reaped': True, 'elapsed_ms': 25, 'deadline_seconds': limit}

    def check(self, session, reviewer=None, execute=None):
        args, limit = validation.catalog()['docs']
        return session.check('docs', args, limit, reviewer=reviewer, execute=execute or self.execute)

    def review(self, session):
        with patch.object(validation, 'source_snapshot', return_value=self.sources), \
             patch.object(validation, 'context_snapshot', return_value=self.context):
            return validation.review_report(session.folder / 'acceptance.json', ['docs'])

    def test_success_missing_checks_and_tampered_report(self):
        session = self.session()
        with self.assertRaisesRegex(ValueError, 'missing/reordered'):
            session.finish(['docs'])
        self.check(session)
        session.report['covered_by'] = {}
        session.finish(['docs'])
        self.review(session)
        for mutate in (lambda r: r['checks'].clear(),
                       lambda r: r.update(scope='milestone'),
                       lambda r: r['checks'][0]['command'].update(exit_code=7),
                       lambda r: r['checks'][0]['review'].update(status='failed'),
                       lambda r: r.update(not_selected={'M1': 'passed'}),
                       lambda r: r['checks'][0].update(arguments=['fake']),
                       lambda r: r.update(covered_by={'docs': 'full-m3'})):
            record = copy.deepcopy(session.report)
            mutate(record)
            base.save(session.folder / 'acceptance.json', record)
            with self.assertRaises(ValueError):
                self.review(session)
        session.persist()
        (session.folder / 'docs.stdout.log').write_text('changed')
        with self.assertRaisesRegex(ValueError, 'artifact'):
            self.review(session)

    def test_resume_success_rechecks_oracle_preserves_failure(self):
        session = self.session()
        self.check(session)
        session.report.update(status='interrupted', error='original attempt')
        session.persist()
        old = session.folder / 'acceptance.json'
        original = old.read_bytes()
        resumed = self.session('resumed', old)
        reviews = []
        self.check(resumed, reviewer=lambda text: (reviews.append(text) or {'status': 'passed'}, []))
        resumed.report['covered_by'] = {}
        resumed.finish(['docs'])
        self.assertEqual(self.calls, ['docs'])
        self.assertEqual(len(reviews), 1)
        self.assertTrue(resumed.report['checks'][0]['resumed'])
        self.assertEqual(old.read_bytes(), original)
        self.review(resumed)

    def test_resume_rejects_source_tool_configuration_and_artifact_changes(self):
        session = self.session()
        self.check(session)
        session.report['status'] = 'failed'
        session.persist()
        old = session.folder / 'acceptance.json'
        self.sources['added'] = 'new'
        with self.assertRaisesRegex(ValueError, 'inputs changed'):
            self.session('changed-source', old)
        self.sources.pop('added')
        self.context['compiler'] = 'two'
        with self.assertRaisesRegex(ValueError, 'inputs changed'):
            self.session('changed-tool', old)
        self.context['compiler'] = 'one'
        self.context['config'] = 'two'
        with self.assertRaisesRegex(ValueError, 'inputs changed'):
            self.session('changed-config', old)
        self.context['config'] = 'one'
        (session.folder / 'docs.stdout.log').unlink()
        resumed = self.session('missing-artifact', old)
        with self.assertRaisesRegex(ValueError, 'artifact'):
            self.check(resumed)

    def test_source_change_inside_command_and_tool_change_at_finish_fail(self):
        session = self.session()
        def mutate(folder, key, args, limit):
            result = self.execute(folder, key, args, limit)
            self.sources['new'] = 'changed'
            return result
        with self.assertRaisesRegex(ValueError, 'source changed'):
            self.check(session, execute=mutate)
        self.sources.pop('new')
        other = self.session('other')
        self.check(other)
        self.context['compiler'] = 'changed'
        with self.assertRaisesRegex(ValueError, 'context changed'):
            other.finish(['docs'])

    def test_prose_evidence_and_index_changes_rerun_audits_not_guest_checks(self):
        manifest = selection(checks=('kernel-host', 'docs'))
        def make(name, resume=None):
            return validation.Session(manifest, self.root/name, resume,
                                      snapshot=lambda: copy.deepcopy(self.sources),
                                      context=lambda: copy.deepcopy(self.context))
        original = make('initial')
        for key in ('kernel-host', 'docs'):
            args, limit = validation.catalog()[key]
            original.check(key, args, limit, execute=self.execute)
        original.report['covered_by'] = {}
        original.finish(['kernel-host', 'docs'])
        path = original.folder/'acceptance.json'
        before = path.read_bytes()
        self.sources.update({'docs/story.md': 'new prose', 'evidence/final/report.json': 'new evidence', '@git-index': 'new staging'})
        resumed = make('host-audits-only', path)
        for key in ('kernel-host', 'docs'):
            args, limit = validation.catalog()[key]
            resumed.check(key, args, limit, execute=self.execute)
        resumed.report['covered_by'] = {}
        resumed.finish(['kernel-host', 'docs'])
        self.assertEqual(self.calls, ['kernel-host', 'docs', 'docs'])
        self.assertTrue(resumed.report['checks'][0]['resumed'])
        self.assertFalse(resumed.report['checks'][1]['resumed'])
        self.assertEqual(path.read_bytes(), before)
        with patch.object(validation, 'source_snapshot', return_value=self.sources), \
             patch.object(validation, 'context_snapshot', return_value=self.context):
            validation.review_report(resumed.folder/'acceptance.json', ['kernel-host', 'docs'])
            with self.assertRaisesRegex(ValueError, 'stale host audits'):
                validation.review_report(path, ['kernel-host', 'docs'])

    def test_contract_config_and_assets_remain_implementation_inputs(self):
        for name in ('docs/profile.json', 'contracts/protocol.md', 'fixtures/font.ttf', 'config/profile.json', 'tools/reviewer.py'):
            self.assertIn(name, validation.implementation_inputs({name: 'hash'}))

    def test_audit_change_mid_check_and_unplanned_check_reject(self):
        session = self.session('audit-change')
        def mutate(folder, key, args, limit):
            result = self.execute(folder, key, args, limit)
            self.sources['docs/story.md'] = 'changed during audit'
            return result
        with self.assertRaisesRegex(ValueError, 'audit inputs changed'):
            self.check(session, execute=mutate)
        with self.assertRaisesRegex(ValueError, 'bounded plan'):
            session.check('boundary', ['guest'], 1, execute=self.execute)

    def test_timeout_or_failed_oracle_never_passes(self):
        for index, change in enumerate(({'exit_code': 7}, {'timed_out': True}, {'cleanup_reaped': False})):
            session = self.session('failure-' + str(index))
            def failure(folder, key, args, limit):
                result = self.execute(folder, key, args, limit)
                result.update(change)
                return result
            with self.assertRaisesRegex(ValueError, 'failed selected'):
                self.check(session, execute=failure)
            self.assertFalse(session.report['checks'])
        session = self.session('oracle')
        def reject(text):
            raise ValueError('missing settlement')
        with self.assertRaisesRegex(ValueError, 'missing settlement'):
            self.check(session, reviewer=reject)
        self.assertFalse(session.report['checks'])
        session = self.session('oracle-failed-result')
        with self.assertRaisesRegex(ValueError, 'reviewer did not pass'):
            self.check(session, reviewer=lambda text: ({'status': 'failed'}, []))
        self.assertFalse(session.report['checks'])

    def test_p402_executes_and_reviews_selected_checks_without_nested_gate(self):
        manifest = selection('P4-02', ('kernel-host', 'desktop-host', 'boundary', 'scheduler', 'capabilities'))
        manifest['impact']['shared_boundary_scheduler'] = True
        manifest_path = self.root / 'selection.json'
        base.save(manifest_path, manifest)
        real_session = validation.Session
        def factory(selection, folder, resume):
            return real_session(selection, folder, resume, snapshot=lambda: self.sources,
                                context=lambda: self.context)
        def execute(folder, key, args, limit):
            result = self.execute(folder, key, args, limit)
            if key.startswith('desktop-') and key != 'desktop-host' or key == 'ordinary':
                artifact = self.root / ('artifact-' + key)
                artifact.mkdir()
                (artifact / 'bytes').write_text(key)
                (folder / (key + '.stdout.log')).write_text('M1 artifacts: ' + str(artifact) + '\n')
            if key == 'flavors':
                output = Path(args[args.index('-Output') + 1])
                output.mkdir()
                base.save(output / 'flavors.json', [{'flavor': f, 'exit_code': 0} for f in p4.FLAVORS])
            return result
        oracle = SimpleNamespace(review=lambda path, test: {'status': 'passed', 'test': test})
        with patch.object(p4, 'ROOT', self.root), \
             patch.object(p4, 'require_implementation'), \
             patch.object(validation, 'Session', side_effect=factory), \
             patch.object(base, 'command', side_effect=execute), \
             patch.object(p4, 'focused', return_value=oracle), \
             patch.object(validation, 'review_check', return_value=({'status': 'passed'}, [])), \
             patch.object(validation, 'source_snapshot', return_value=self.sources), \
             patch.object(validation, 'context_snapshot', return_value=self.context):
            p4.run_gate(manifest_path)
            acceptance = next(self.root.glob('build/m4/P4-02/*/acceptance.json'))
            report = p4.review(acceptance)
            self.assertEqual(report['scope'], 'story')
            self.assertEqual(self.calls, p4.expected_checks(manifest))
            self.assertNotIn('full-m3', self.calls)
            saved = base.read(acceptance)
            saved['checks'] = [c for c in saved['checks'] if c['id'] != 'desktop-4']
            base.save(acceptance, saved)
            with self.assertRaisesRegex(ValueError, 'missing/reordered'):
                p4.review(acceptance)

    def test_focused_runtime_rejects_stale_sources_and_changed_source_set(self):
        artifact = self.root / 'runtime'
        artifact.mkdir()
        kernel = self.root / 'kernel'
        kernel.mkdir()
        source = kernel / 'original.rs'
        source.write_text('original')
        base.save(artifact / 'build-inputs.json', {'sources': {'kernel/original.rs': base.digest(source)}})
        with patch.object(base, 'ROOT', self.root):
            source.write_text('changed')
            with self.assertRaisesRegex(ValueError, 'stale runtime source'):
                runtime_focus_review.review('memory', artifact)
            source.write_text('original')
            (kernel / 'added.rs').write_text('new')
            with self.assertRaisesRegex(ValueError, 'changed runtime source set'):
                runtime_focus_review.review('memory', artifact)


if __name__ == '__main__':
    unittest.main()
