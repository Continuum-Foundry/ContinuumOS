"""Serial P4-01 gate: current M3/M1, independent host plans and affected hosted M2."""
import argparse
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys
import time
import uuid

import m4_prerequisites as plan
import m4_reference as reference
import p3_acceptance as baseline

ROOT = plan.ROOT


def module(name, path):
    spec=importlib.util.spec_from_file_location(name,path)
    result=importlib.util.module_from_spec(spec);spec.loader.exec_module(result)
    return result


def review(path):
    path=baseline.local(path); report=baseline.read(path)
    baseline.require(report['story']=='P4-01' and report['status']=='passed','incomplete P4-01 gate')
    baseline.require(baseline.source_lock()==baseline.read(path.parent/'source-lock.json'),'P4 source changed')
    baseline.require(baseline.digest(path.parent/'source-lock.json')==report['source_lock_sha256'],'P4 source lock substitution')
    plan.validate(plan.read(plan.PROFILE));reference.profile(reference.load(plan.PROFILE))
    baseline.require([c['name'] for c in report['commands']]==['m3','host-plan','m2'],'missing/reordered baseline stage')
    for command in report['commands']:
        baseline.require(command['exit_code']==0 and not command['timed_out'] and command['cleanup_reaped'],'failed stage/cleanup')
        baseline.require(baseline.read(path.parent/(command['name']+'-command.json'))==command,'command substitution')
    # Reopen current raw guest/artifact reviewers, not just the aggregate string.
    m3=baseline.review(baseline.local(report['m3']))
    host_path=baseline.local(report['host_plan']);host=baseline.read(host_path)
    baseline.require(host['status']=='host-plan-passed' and host['profile_mutations']>100 and host['codec_cases']>1000,'incomplete host plan')
    baseline.require(host['scoped_deadline_negatives']==10,'missing scoped-deadline consumer negatives')
    baseline.require(all(r['status']=='passed' for r in host['rows']) and len(host['children'])==12,'incomplete host failures')
    for source,sha in host['inputs'].items():
        baseline.require(baseline.digest(ROOT/source)==sha,'stale host input: '+source)
    expected={'success':'complete','nonzero':'exited-nonzero','missing-final':'missing-final-record','timeout':'timeout'}
    for child in host['children']:
        current=baseline.local(child['path']);value=baseline.read(current)
        baseline.require(baseline.digest(current)==child['sha256'],'changed child report')
        if child['case'] in expected:
            nested_path=baseline.local(value['child_result']);nested=baseline.read(nested_path)
            baseline.require(baseline.digest(nested_path)==value['child_result_sha256'],'changed nested child')
            baseline.require(nested['outcome']==value['outcome']==expected[child['case']] and nested['cleanup']=='reaped' and nested['launched'],'child failure misclassified')
            baseline.require(nested['runner_sha256']==baseline.digest(ROOT/'tools/m3_runtime.py'),'changed child runner')
            if child['case']=='success':
                raw=plan.read(nested_path.parent/'stdout.log')
                baseline.require(raw==dict(version=1,event='complete',run_id=nested['run_id'],image_sha256=host['image_sha256']),'missing/changed completion')
            elif child['case']=='nonzero':
                baseline.require(nested['exit_code']==7,'unexpected child exit')
            elif child['case']=='missing-final':
                baseline.require(plan.read(nested_path.parent/'stdout.log')['event']=='started' and nested['exit_code']==0,'missing-final fixture changed')
            else:
                baseline.require(nested['deadline_ms']==100 and nested['exit_code']!=0,'negative host deadline changed')
        else:
            baseline.require(not value['launched'] and value['cleanup']=='not-launched' and value['outcome']=='validation-rejected','prelaunch fault escaped')
    devices=host_path.parent/'devices'
    for command in host['devices']['commands']:
        baseline.require(baseline.digest(devices/(command['name']+'.txt'))==command['sha256'],'changed device capability output')
    baseline.require(baseline.digest(host_path.parent/'plans.json')==host['plans_sha256'],'changed disabled-development plan')
    m2path=baseline.local(report['m2']);m2=baseline.read(m2path)
    hosted=module('p401_hosted',ROOT/'tools/Run-P2Acceptance.py')
    hosted.validate_index(m2,baseline.read(ROOT/'config/p2-13-acceptance.json'),m2path.parent.name,m2['artifact'])
    baseline.require(m2['status']=='passed' and hosted.snapshot()==baseline.read(m2path.parent/'tested-inputs.json'),'stale M2 source')
    result={'story':'P4-01','status':'passed','scope':'local Codex self-review of prerequisites/host planning; no guest desktop or new ABI acceptance',
            'acceptance_sha256':baseline.digest(path),'source_lock_sha256':report['source_lock_sha256'],
            'm3_review_sha256':baseline.digest(baseline.local(report['m3']).parent/'review.json'),
            'm3_rows':len(m3['matrix']),'m1':m3['runtime_regression'],
            'deadline_profile':m3['deadline_profile'],'ordinary_guest_deadline_ms':30000,'stress_guest_deadline_ms':60000,
            'host_plan_sha256':baseline.digest(host_path),'profile_mutations':host['profile_mutations'],'codec_cases':host['codec_cases'],
            'm2_rows':len(m2['rows']),'limits':host['limits']}
    baseline.save(path.parent/'review.json',result)
    return result


def run():
    plan.validate(plan.read(plan.PROFILE));reference.profile(reference.load(plan.PROFILE))
    folder=ROOT/'build/m4/P4-01'/f'acceptance-{time.time_ns()}-{uuid.uuid4().hex}';folder.mkdir(parents=True)
    locked=baseline.source_lock();baseline.save(folder/'source-lock.json',locked)
    (folder/'source.patch').write_bytes(subprocess.check_output(['git','diff','HEAD','--binary'],cwd=ROOT))
    result={'story':'P4-01','status':'running','source_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            'source_lock_sha256':baseline.digest(folder/'source-lock.json'),'profile_sha256':plan.digest(plan.PROFILE),'commands':[]}
    print('P4-01 local run: '+str(folder),flush=True)
    try:
        result['commands'].append(baseline.command(folder,'m3',['pwsh','-NoProfile','-File','tools/Test-P3-14.ps1'],10800))
        matches=re.findall(r'P3-14 acceptance: ([^\r\n]+)',(folder/'m3.stdout.log').read_text(encoding='utf-8-sig'))
        baseline.require(len(matches)==1,'missing/ambiguous M3 path');m3path=baseline.local(matches[0].strip());result['m3']=str(m3path)
        m3=baseline.read(m3path);stress=baseline.read(baseline.local(m3['runs']['stress_acceptance']))
        prior=baseline.read(baseline.local(stress['runs']['prior_acceptance']));pre=baseline.local(prior['runs']['prerequisites'])
        match=re.search(r'Evidence: ([^\r\n]+)',(pre/'m0.log').read_text(encoding='utf-8-sig'))
        baseline.require(match is not None,'missing fresh Phase 0 artifact')
        image=baseline.local(match[1].strip())/'clean-a/userland-link.elf'
        stress_artifact=baseline.local(stress['stress'][0]['artifacts'])
        result['commands'].append(baseline.command(folder,'host-plan',[sys.executable,'tools/Test-P4-01.py','--image',str(image),'--sha256',baseline.digest(image),'--stress-run',str(stress_artifact)],180))
        matches=re.findall(r'P4-01 host review: ([^\r\n]+)',(folder/'host-plan.stdout.log').read_text(encoding='utf-8-sig'))
        baseline.require(len(matches)==1,'missing host plan review');result['host_plan']=str(baseline.local(matches[0].strip()))
        result['commands'].append(baseline.command(folder,'m2',['pwsh','-NoProfile','-File','tools/Test-P2-13.ps1'],10800))
        matches=re.findall(r'P2-13 acceptance: ([^\r\n]+)',(folder/'m2.stdout.log').read_text(encoding='utf-8-sig'))
        baseline.require(len(matches)==1,'missing hosted M2 path');result['m2']=str(baseline.local(matches[0].strip())/'acceptance.json')
        baseline.require(baseline.source_lock()==locked,'implementation inputs changed during P4 gate')
        result['status']='passed'
    except BaseException as error:
        result['status']='interrupted' if isinstance(error,KeyboardInterrupt) else 'failed'
        result['error']=str(error)
        if isinstance(error,baseline.CommandFailure):result['commands'].append(error.result)
        raise
    finally:
        baseline.save(folder/'acceptance.json',result)
        print('P4-01 acceptance: '+str(folder/'acceptance.json'),flush=True)
    print(json.dumps(review(folder/'acceptance.json'),indent=2),flush=True)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    sub=parser.add_subparsers(dest='command',required=True)
    sub.add_parser('run');consumer=sub.add_parser('review');consumer.add_argument('--acceptance',type=Path,required=True)
    args=parser.parse_args()
    if args.command=='run':run()
    else:print(json.dumps(review(args.acceptance),indent=2))
