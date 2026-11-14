"""Counted P4-05 story acceptance. Never a milestone gate."""
import argparse, importlib.util, json, time
from pathlib import Path
import p3_acceptance as base
import story_validation as v
import p4_runtime_acceptance as runtime
import p4_display_acceptance as display
spec=importlib.util.spec_from_file_location('p405_oracle',base.ROOT/'tools/Review-P4-05.py');oracle=importlib.util.module_from_spec(spec);spec.loader.exec_module(oracle)
EARLY=['kernel-host','surface-host','surface-oracle-tests','xtask-host','runner-tests','retention-tests','prerequisites']
CASES=list(oracle.SCENARIOS)

def checks(m,probe=False):return (EARLY[:-1] if probe else EARLY)+(['surface-720'] if probe else [CASES[0],'surface-flavors']+CASES[1:]+['ordinary','display-720']+[k for k in v.schedule(m)[0] if k not in EARLY])
def ordinary(text):
    folder=v.artifact_path(text,'M1 artifacts: ')
    return runtime.focused().review(folder,False,expected_features='surface-runtime'),[folder]
def guest(label,text):
    folder=v.artifact_path(text,'M1 artifacts: ');return oracle.review(folder,label),[folder]
def flavors(folder):
    data=base.read(folder/'flavors.json');base.require([r['flavor'] for r in data]==runtime.FLAVORS+['device-test','display-test','surface-runtime','surface-test'] and all(r['exit_code']==0 for r in data),'separate flavor matrix')
    return dict(status='passed',directory=str(folder),flavors=data),[folder]
def gate(manifest,resume,probe=False):
    m=v.validate_manifest(base.read(manifest));planned=checks(m,probe);plan=v.execution_plan(m,planned);print(json.dumps(plan,indent=2),flush=True)
    folder=base.ROOT/'build/m4/P4-05'/(('probe-' if probe else 'acceptance-')+str(time.time_ns()));s=v.Session(m,folder,resume,planned_checks=planned);s.report['plan']=plan;s.persist()
    try:
        for key in (EARLY[:-1] if probe else EARLY):
            args,limit=v.catalog()[key];s.check(key,args,limit,lambda text,key=key:v.review_check(key,text,folder))
        for label in (['surface-720'] if probe else CASES):
            text,_=s.check(label,['cargo','xtask','test','--suite','m1','--case',label],600,lambda text,label=label:guest(label,text))
            if not probe and label==CASES[0]:
                desc=v.artifact_path(text,'M1 artifacts: ');out=folder/'flavors'
                s.check('surface-flavors',v.PS+['tools/Check-P4-05Flavors.ps1','-Descriptors',str(desc),'-Output',str(out)],600,lambda text:flavors(out))
        if probe:s.finish(planned);s.report['scope']='development-probe';s.persist();return
        s.check('ordinary',['cargo','xtask','run','--profile','vm-baseline'],600,ordinary)
        s.check('display-720',['cargo','xtask','test','--suite','m1','--case','display-720'],600,lambda text:display.display_review(text,720))
        for key in v.schedule(m)[0]:
            if key in EARLY:continue
            args,limit=v.catalog()[key];s.check(key,args,limit,lambda text,key=key:v.review_check(key,text,folder))
        s.finish(planned)
    except BaseException as e:
        s.report.update(status='interrupted' if isinstance(e,KeyboardInterrupt) else 'failed',error=str(e));s.persist();raise
    finally:print('P4-05 acceptance: '+str(folder/'acceptance.json'),flush=True)
def review(path):
    r=base.read(path);v.review_report(path,checks(r['manifest']))
    for c in r['checks']:
        k=c['id'];text=Path(c['stdout']).read_text(encoding='utf-8-sig')
        if k in CASES:
            base.require(c['arguments']==['cargo','xtask','test','--suite','m1','--case',k] and c['deadline_seconds']==600,'substituted surface command');guest(k,text)
        elif k=='surface-flavors':
            first=next(row for row in r['checks'] if row['id']==CASES[0]);desc=v.artifact_path(Path(first['stdout']).read_text(encoding='utf-8-sig'),'M1 artifacts: ');out=Path(c['review']['directory']);base.require(c['arguments']==v.PS+['tools/Check-P4-05Flavors.ps1','-Descriptors',str(desc),'-Output',str(out)] and c['deadline_seconds']==600,'substituted flavor command');flavors(out)
        elif k=='ordinary':
            base.require(c['arguments']==['cargo','xtask','run','--profile','vm-baseline'] and c['deadline_seconds']==600,'substituted ordinary command');ordinary(text)
        elif k=='display-720':display.display_review(text,720)
        else:v.review_check(k,text,Path(path).parent)
    result=dict(status='passed',story='P4-05',acceptance_sha256=base.digest(path),plan=r['plan']);base.save(Path(path).parent/'review.json',result);print(json.dumps(result,indent=2));return result
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('action',choices=['plan','probe-plan','probe','run','review']);p.add_argument('--manifest',default='config/p4-05-validation.json');p.add_argument('--resume');p.add_argument('--acceptance');a=p.parse_args()
    if a.action in ('plan','probe-plan'):
        m=v.validate_manifest(base.read(a.manifest));print(json.dumps(v.execution_plan(m,checks(m,a.action=='probe-plan')),indent=2))
    elif a.action=='review':review(a.acceptance)
    else:gate(a.manifest,a.resume,a.action=='probe')
