"""P4-02 original full serial gate; owner deferred new selection policy to next story."""
import argparse
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import time
import p3_acceptance as base

ROOT = base.ROOT

def focused():
    spec = importlib.util.spec_from_file_location('p402_full_review', ROOT/'tools/Review-P4-02.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def run_gate():
    run = ROOT/'build/m4/P4-02'/('acceptance-'+str(time.time_ns()))
    run.mkdir(parents=True)
    locked = base.source_lock()
    base.save(run/'source-lock.json', locked)
    report = {'schema_version':1, 'story':'P4-02', 'status':'running', 'commands':[], 'guests':[],
              'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
              'selection':'original complete serial gate; owner directs change-based PR policy to take effect next story (2026-11-08)'}
    def command(name, args, limit):
        result = base.command(run, name, args, limit)
        report['commands'].append(result)
        base.save(run/'acceptance.json', report)
        base.require(base.source_lock()==locked, 'source changed during P4-02 acceptance')
        return (run/(name+'.stdout.log')).read_text(encoding='utf-8-sig')
    def artifact(text, prefix):
        matches = re.findall(re.escape(prefix)+r'([^\r\n]+)', text)
        base.require(len(matches)==1, 'missing/ambiguous artifact path: '+prefix)
        return Path(matches[0].strip())
    try:
        command('kernel-host',['cargo','test','--locked','--offline','-p','continuum-kernel','--lib'],180)
        command('desktop-host',['cargo','test','--locked','--offline','-p','continuum-kernel','--lib','--features','desktop-runtime'],180)
        for i in range(5):
            path = artifact(command('desktop-'+str(i),['cargo','xtask','test','--suite','m1','--case','desktop-runtime'],600),'M1 artifacts: ')
            result = focused().review(path, True)
            base.save(run/('desktop-'+str(i)+'-review.json'),result)
            report['guests'].append({'directory':str(path),'review':result})
            if i==0:
                command('flavors',['pwsh','-NoProfile','-File','tools/Check-P4-02Flavors.ps1','-Descriptors',str(path),'-Output',str(run/'flavors')],600)
        path = artifact(command('ordinary',['cargo','xtask','run','--profile','vm-baseline'],600),'M1 artifacts: ')
        report['ordinary'] = {'directory':str(path),'review':focused().review(path,False)}
        base.save(run/'ordinary-review.json',report['ordinary']['review'])
        path = artifact(command('m3',['pwsh','-NoProfile','-File','tools/Test-P3-14.ps1'],10800),'P3-14 acceptance: ')
        report['m3_acceptance'] = str(path)
        base.review(path)
        base.require(base.source_lock()==locked,'source changed during P4-02 acceptance')
        report['status'] = 'passed'
    except KeyboardInterrupt:
        report.update(status='interrupted',error='acceptance interrupted; never counted as a pass')
        raise
    except Exception as error:
        if isinstance(error,base.CommandFailure): report['commands'].append(error.result)
        report.update(status='failed',error=str(error))
        raise
    finally:
        base.save(run/'acceptance.json',report)
        print('P4-02 acceptance: '+str(run/'acceptance.json'),flush=True)

def review(path):
    path = Path(path)
    report = base.read(path)
    names = ['kernel-host','desktop-host','desktop-0','flavors','desktop-1','desktop-2','desktop-3','desktop-4','ordinary','m3']
    base.require(report['status']=='passed' and report['story']=='P4-02','incomplete P4-02 full acceptance')
    base.require([c['name'] for c in report['commands']]==names and all(c['exit_code']==0 and not c['timed_out'] and c['cleanup_reaped'] for c in report['commands']),'incomplete/failed commands')
    base.require(len(report['guests'])==5 and base.read(path.parent/'source-lock.json')==base.source_lock(),'stale source or missing guest')
    for guest in report['guests']: focused().review(guest['directory'],True)
    focused().review(report['ordinary']['directory'],False)
    flavors = base.read(path.parent/'flavors/flavors.json')
    base.require(len(flavors)==16 and all(v['exit_code']==0 for v in flavors),'missing narrow build')
    base.review(report['m3_acceptance'])
    result = {'schema_version':1,'story':'P4-02','status':'passed','review':'local Codex self-review; not external approval',
              'acceptance_sha256':base.digest(path),'guest_repetitions':5,'metadata_flavors':16,
              'ordinary_without_acceptance_observers':True,'m3_acceptance':report['m3_acceptance'],
              'selection':report['selection']}
    base.save(path.parent/'review.json',result)
    print(json.dumps(result,indent=2))
    return result

if __name__=='__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action',choices=['run','review'])
    parser.add_argument('--acceptance')
    args = parser.parse_args()
    if args.action=='run': run_gate()
    else:
        base.require(args.acceptance,'review requires acceptance path')
        review(args.acceptance)
