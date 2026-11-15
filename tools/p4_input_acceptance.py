"""Counted P4-06 new behavior and affected regressions; never an M4 exit."""
import argparse,importlib.util,json,time
from pathlib import Path
import p3_acceptance as base
import story_validation as v
import p4_runtime_acceptance as runtime
import p4_device_acceptance as device
import p4_display_acceptance as display
EARLY=['kernel-host','device-host','surface-host','xtask-host','input-oracle-tests','runner-tests','retention-tests','prerequisites']
def oracle():
    spec=importlib.util.spec_from_file_location('p406',base.ROOT/'tools/Review-P4-06.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m

def checks(m,probe=False):
    return EARLY+(['input']if probe else ['input','input-flavors','ordinary','devices','surface-720','display-720']+[x for x in v.schedule(m)[0]if x not in EARLY])

def guest(text):
    folder=v.artifact_path(text,'M1 artifacts: ');return oracle().review(folder),[folder]
def ordinary(text):
    folder=v.artifact_path(text,'M1 artifacts: ');return oracle().ordinary(folder),[folder]
def surface(text):
    folder=v.artifact_path(text,'M1 artifacts: ');return oracle().optical.review(folder,'surface-720'),[folder]
def flavors(folder):
    actual=base.read(folder/'flavors.json');names=runtime.FLAVORS+['device-test','display-test','surface-runtime','surface-test','input-runtime','input-test']
    base.require([x['flavor']for x in actual]==names and all(x['exit_code']==0 for x in actual),'separate flavor matrix')
    for row in actual:
        args=['check','--locked','--offline','-p','continuum-kernel','--bin','continuum-kernel','--target','x86_64-unknown-none','--profile','kernel-dev']
        if row['flavor']!='ordinary-minimal':args+=['--features',row['flavor']]
        base.require(row['arguments']==args,'substituted flavor command')
    return dict(status='passed',directory=str(folder),flavors=actual),[folder]

# One recorded reviewer-only correction, not a standing source/reuse exception.
REPAIR_ANCHOR='774ad7e14471749e3735a699a80b18fbaac7ded2de238a9e0f77b2772e72d8ee'
REPAIR_FILES={'tools/Review-P4-06.py','tools/Test-P4-06.py','tools/p4_input_acceptance.py'}
def repair_changes(a,b):
    changed={p for p in set(a)|set(b)if a.get(p)!=b.get(p)}
    base.require(changed==REPAIR_FILES,'repair changed implementation outside the exact three reviewer/runner files')
    return changed
def verify_repair(previous,current):
    base.require(base.digest(previous)==REPAIR_ANCHOR,'wrong reviewer-repair anchor')
    prior=base.read(previous)
    base.require(prior['status']=='failed' and prior['error']=='independent client binaries' and prior['story']=='P4-06','wrong reviewer failure')
    base.require(prior['manifest']==current['manifest'] and prior['context']==current['context'] and prior['source_commit']==current['source_commit'],'repair manifest/context/base changed')
    a,b=prior['implementation_inputs'],current['implementation_inputs'];changed=repair_changes(a,b)
    entries=[]
    for name in sorted(changed):
        blob=base.ROOT/'work/p4-06/source-overrides'/a[name]
        base.require(blob.is_file() and base.digest(blob)==a[name],'missing original reviewer source bytes')
        entries.append(dict(path=name,old_sha256=a[name],new_sha256=b[name],original_blob=str(blob)))
    return prior,dict(status='passed',scope='one reviewer/runner-only compatibility correction; no guest implementation, command, manifest, bounds or tool-context change',previous_acceptance=str(Path(previous).resolve()),previous_sha256=REPAIR_ANCHOR,changed_inputs=entries,reason='Ordinary images may have identical bytes; require byte-distinct independently linked fixture clients only. Old failed aggregate is preserved.')
def seed_repair(session,previous):
    prior,proof=verify_repair(previous,session.report)
    # Changed orchestration/oracle host checks rerun. Every carried guest or build
    # still passes normal per-command hash checks and independent artifact review.
    for c in prior['checks']:
        if c['id']in {'input-oracle-tests','runner-tests'}:continue
        base.require(all(Path(p).is_file() and base.digest(p)==sha for p,sha in c['artifact_hashes'].items()),'original carried artifact changed')
        session.previous[c['id']]=c
    command=next(c for c in prior['commands']if c['name']=='ordinary')
    base.require(command==base.read(Path(previous).parent/'ordinary-command.json') and command['arguments']==['cargo','xtask','run','--profile','vm-baseline'] and command['exit_code']==0 and not command['timed_out'] and command['cleanup_reaped'] and command['deadline_seconds']==600,'ordinary original command did not complete successfully')
    stdout=Path(previous).parent/'ordinary.stdout.log';text=stdout.read_text(encoding='utf-8-sig');result,artifacts=ordinary(text)
    hashes={str(p.resolve()):base.digest(p)for p in [stdout,stdout.parent/'ordinary.stderr.log',stdout.parent/'ordinary-command.json']}
    for artifact in artifacts:hashes.update(v.tree_hashes(artifact))
    session.previous['ordinary']=dict(id='ordinary',status='fresh-pass',arguments=command['arguments'],deadline_seconds=600,elapsed_ms=command['elapsed_ms'],stdout=str(stdout.resolve()),command=command,review=result,artifact_hashes=hashes,resumed=False,adopted_after_explicit_reviewer_repair=True)
    proof['ordinary_adoption']=dict(original_command_sha256=base.digest(stdout.parent/'ordinary-command.json'),stdout_sha256=base.digest(stdout),artifact_count=len(hashes),review=result,hash_observation='Original command/case/ISO/ELF/source identities verified; full artifact hashes recorded at this fresh corrected review. The failed aggregate had stopped before recording an ordinary pass.')
    session.report.update(resumed_from=str(Path(previous).resolve()),reviewer_repair=proof);session.persist()

def gate(manifest,resume,probe=False,repair=None):
    m=v.validate_manifest(base.read(manifest));planned=checks(m,probe);plan=v.execution_plan(m,planned);print(json.dumps(plan,indent=2),flush=True)
    folder=base.ROOT/'build/m4/P4-06'/(( 'probe-'if probe else'acceptance-')+str(time.time_ns()));s=v.Session(m,folder,resume,planned_checks=planned);s.report['plan']=plan;s.persist()
    if repair:base.require(resume is None and not probe,'repair is a separate final-source attempt');seed_repair(s,repair)
    try:
        for k in EARLY:
            args,limit=v.catalog()[k];s.check(k,args,limit,lambda text,k=k:v.review_check(k,text,folder))
        text,_=s.check('input',['cargo','xtask','test','--suite','m1','--case','input'],600,guest)
        if not probe:
            desc=v.artifact_path(text,'M1 artifacts: ');out=Path(s.previous['input-flavors']['review']['directory'])if 'input-flavors'in s.previous else folder/'flavors'
            s.check('input-flavors',v.PS+['tools/Check-P4-06Flavors.ps1','-Descriptors',str(desc),'-Output',str(out)],900,lambda text:flavors(out))
            s.check('ordinary',['cargo','xtask','run','--profile','vm-baseline'],600,ordinary)
            s.check('devices',['cargo','xtask','test','--suite','m1','--case','devices'],600,device.device_review)
            s.check('surface-720',['cargo','xtask','test','--suite','m1','--case','surface-720'],600,surface)
            s.check('display-720',['cargo','xtask','test','--suite','m1','--case','display-720'],600,lambda text:display.display_review(text,720))
            for k in v.schedule(m)[0]:
                if k in EARLY:continue
                args,limit=v.catalog()[k];s.check(k,args,limit,lambda text,k=k:device.timer_review(text)if k=='timer'else v.review_check(k,text,folder))
        s.finish(planned)
        if probe:s.report['scope']='development probe; not complete story acceptance';s.persist()
    except BaseException as e:
        s.report.update(status='interrupted'if isinstance(e,KeyboardInterrupt)else'failed',error=str(e));s.persist();raise
    finally:print('P4-06 acceptance: '+str(folder/'acceptance.json'),flush=True)

def review(path):
    report=base.read(path);planned=checks(report['manifest']);v.review_report(path,planned)
    if report.get('reviewer_repair'):verify_repair(report['reviewer_repair']['previous_acceptance'],report)
    for c in report['checks']:
        k=c['id'];text=Path(c['stdout']).read_text(encoding='utf-8-sig')
        if k=='input':base.require(c['arguments']==['cargo','xtask','test','--suite','m1','--case','input']and c['deadline_seconds']==600,'substituted input command');guest(text)
        elif k=='input-flavors':
            desc=v.artifact_path(Path(next(x for x in report['checks']if x['id']=='input')['stdout']).read_text(encoding='utf-8-sig'),'M1 artifacts: ');out=Path(c['review']['directory']);base.require(c['arguments']==v.PS+['tools/Check-P4-06Flavors.ps1','-Descriptors',str(desc),'-Output',str(out)]and c['deadline_seconds']==900,'substituted metadata command');flavors(out)
        elif k=='ordinary':base.require(c['arguments']==['cargo','xtask','run','--profile','vm-baseline']and c['deadline_seconds']==600,'substituted ordinary command');ordinary(text)
        elif k=='devices':base.require(c['arguments']==['cargo','xtask','test','--suite','m1','--case','devices']and c['deadline_seconds']==600,'substituted devices command');device.device_review(text)
        elif k=='surface-720':base.require(c['arguments']==['cargo','xtask','test','--suite','m1','--case',k]and c['deadline_seconds']==600,'substituted surface command');surface(text)
        elif k=='display-720':display.display_review(text,720)
        elif k=='timer':device.timer_review(text)
        else:v.review_check(k,text,Path(path).parent)
    result=dict(status='passed',story='P4-06',acceptance_sha256=base.digest(path),plan=report['plan']);base.save(Path(path).parent/'review.json',result);print(json.dumps(result,indent=2));return result
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('action',choices=['plan','probe-plan','probe','run','review']);p.add_argument('--manifest',default='config/p4-06-validation.json');p.add_argument('--resume');p.add_argument('--reviewer-repair');p.add_argument('--acceptance');a=p.parse_args()
    if a.action in ('plan','probe-plan'):
        m=v.validate_manifest(base.read(a.manifest));print(json.dumps(v.execution_plan(m,checks(m,a.action=='probe-plan')),indent=2))
    elif a.action=='review':review(a.acceptance)
    else:gate(a.manifest,a.resume,a.action=='probe',a.reviewer_repair)
