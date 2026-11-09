"""Counted P4-03 new behavior and affected regressions; no milestone claim."""
import argparse
import importlib.util
import json
from pathlib import Path
import time
import p3_acceptance as base
import p4_runtime_acceptance as runtime
import story_validation as validation

ROOT = base.ROOT
EARLY = ['kernel-host','device-host','xtask-host','runner-tests','prerequisites']


def expected_checks(manifest):
    selected, _ = validation.schedule(manifest)
    return EARLY + ['device-0','device-flavors','ordinary'] + [k for k in selected if k not in EARLY]


def oracle():
    spec = importlib.util.spec_from_file_location('p403_oracle', ROOT/'tools/Review-P4-03.py')
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    return module


def device_review(text):
    path = validation.artifact_path(text, 'M1 artifacts: ')
    return oracle().review(path), [path]


def flavors(output):
    actual = base.read(output/'flavors.json')
    base.require([v['flavor'] for v in actual] == runtime.FLAVORS+['device-test']
                 and all(v['exit_code'] == 0 for v in actual), 'missing/failed flavor')
    for row in actual:
        expected=['check','--locked','--offline','-p','continuum-kernel','--bin','continuum-kernel','--target','x86_64-unknown-none','--profile','kernel-dev']
        if row['flavor'] != 'ordinary-minimal':expected += ['--features',row['flavor']]
        base.require(row['arguments'] == expected, 'substituted metadata arguments')
    return dict(status='passed',directory=str(output),flavors=actual), [output]


def timer_review(text):
    path = validation.artifact_path(text, 'M1 artifacts: ')
    manifest = base.read(path/'build-manifest.json'); run=path/'timer-progress-run'
    case = base.read(run/'case-result.json'); image=base.read(path/'timer-progress-package/iso-manifest.json')
    base.require(case['status'] == 'passed' and case['cleanup_reaped'] and not case['timed_out']
                 and case['deadline_ms'] == 30000 and case['emulator_test_exit'] == 33, 'timer guest')
    base.require(base.digest(path/'kernel.elf') == manifest['kernel_sha256'] == image['kernel_sha256']
                 and base.digest(path/'timer-progress-package/continuum.iso') == image['iso_sha256'] == case['iso_sha256'], 'timer artifacts')
    records = oracle().runtime.raw(run/'serial.log', manifest['build_id'], image['host_session_id'])
    progress=oracle().rows(records,'m1.timer_progress')
    base.require(len(progress) == 1 and progress[0]['registers'] == 'true', 'returning IRQ registers')
    samples=oracle().rows(records,'m1.clock_sample')
    base.require(len(samples)==100 and [int(s['index']) for s in samples]==list(range(100))
                 and all(s['registers']=='true' and s['continuation']=='true' for s in samples), 'timer sample coverage')
    base.require(all(int(a['ticks']) < int(b['ticks']) and int(a['uptime_ns']) < int(b['uptime_ns']) for a,b in zip(samples,samples[1:]))
                 and all(progress[0][k]=='100' for k in ['reads','changes','halts']), 'timer progress')
    return dict(status='passed',progress=progress), [path]


def gate(manifest_path, resume, probe=False):
    manifest = validation.validate_manifest(base.read(manifest_path))
    base.require(manifest['story'] == 'P4-03', 'wrong story')
    checks = ['kernel-host','device-host','xtask-host','runner-tests','device-0'] if probe else expected_checks(manifest)
    if probe: manifest['execution_budget']['max_vm_launches'] = 1
    plan = validation.execution_plan(manifest, checks)
    print(json.dumps(plan,indent=2),flush=True)
    folder = ROOT/'build/m4/P4-03'/(('probe-' if probe else 'acceptance-')+str(time.time_ns()))
    session=validation.Session(manifest,folder,resume,planned_checks=checks)
    session.report['plan']=plan; session.persist()
    try:
        for key in (EARLY[:-1] if probe else EARLY):
            args, limit = validation.catalog()[key]; session.check(key,args,limit)
        text,_=session.check('device-0',['cargo','xtask','test','--suite','m1','--case','devices'],600,device_review)
        if probe:
            session.report['covered_by'] = validation.schedule(manifest)[1]
            session.finish(checks)
            session.report['scope']='development-probe';session.persist()
            return
        descriptors=validation.artifact_path(text,'M1 artifacts: '); output=folder/'flavors'
        session.check('device-flavors',validation.PS+['tools/Check-P4-03Flavors.ps1','-Descriptors',str(descriptors),'-Output',str(output)],600,lambda text:flavors(output))
        session.check('ordinary',['cargo','xtask','run','--profile','vm-baseline'],600,lambda text:runtime.guest_review(text,False))
        for key in validation.schedule(manifest)[0]:
            if key in EARLY: continue
            args,limit=validation.catalog()[key]
            session.check(key,args,limit,lambda text,k=key:timer_review(text) if k == 'timer' else validation.review_check(k,text,folder))
        session.finish(expected_checks(manifest))
    except BaseException as error:
        session.report.update(status='interrupted' if isinstance(error,KeyboardInterrupt) else 'failed',error=str(error))
        if isinstance(error,base.CommandFailure): session.report['commands'].append(error.result)
        session.persist(); raise
    finally: print('P4-03 acceptance: '+str(folder/'acceptance.json'),flush=True)


def review(path):
    report=base.read(path)
    validation.review_report(path,expected_checks(report['manifest']))
    for check in report['checks']:
        key=check['id']; text=Path(check['stdout']).read_text(encoding='utf-8-sig')
        if key=='device-0':
            base.require(check['arguments']==['cargo','xtask','test','--suite','m1','--case','devices'] and check['deadline_seconds']==600,'substituted device command')
            device_review(text)
        elif key=='device-flavors':
            output=Path(check['review']['directory'])
            first=next(c for c in report['checks'] if c['id']=='device-0')
            descriptors=validation.artifact_path(Path(first['stdout']).read_text(encoding='utf-8-sig'),'M1 artifacts: ')
            base.require(check['arguments']==validation.PS+['tools/Check-P4-03Flavors.ps1','-Descriptors',str(descriptors),'-Output',str(output)] and check['deadline_seconds']==600,'substituted flavor command')
            flavors(output)
        elif key=='ordinary':
            base.require(check['arguments']==['cargo','xtask','run','--profile','vm-baseline'] and check['deadline_seconds']==600,'substituted ordinary command')
            runtime.guest_review(text,False)
        elif key=='timer':timer_review(text)
        else:validation.review_check(key,text,Path(path).parent)
    result=dict(schema_version=1,story='P4-03',status='passed',scope='story',acceptance_sha256=base.digest(path),plan=report['plan'])
    base.save(Path(path).parent/'review.json',result)
    print('P4-03 aggregate review passed')
    return result


if __name__ == '__main__':
    p=argparse.ArgumentParser();p.add_argument('action',choices=['plan','run','probe','review']);p.add_argument('--manifest',default='config/p4-03-validation.json');p.add_argument('--resume');p.add_argument('--acceptance')
    args=p.parse_args()
    if args.action == 'plan':
        m=validation.validate_manifest(base.read(args.manifest));print(json.dumps(validation.execution_plan(m,expected_checks(m)),indent=2))
    elif args.action=='review':review(args.acceptance)
    else: gate(args.manifest,args.resume,args.action=='probe')
