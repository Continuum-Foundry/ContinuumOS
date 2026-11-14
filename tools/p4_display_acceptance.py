"""Counted display acceptance and selected regressions, never a milestone gate."""
import argparse, importlib.util, json, time
from pathlib import Path
import p3_acceptance as base
import story_validation as v
import p4_runtime_acceptance as runtime
import p4_device_acceptance as device
spec=importlib.util.spec_from_file_location('p404_oracle',base.ROOT/'tools/Review-P4-04.py');oracle=importlib.util.module_from_spec(spec);spec.loader.exec_module(oracle)
EARLY=['kernel-host','display-host','xtask-host','runner-tests','prerequisites']
def checks(m):return EARLY+['display-720','display-1080','display-flavors','ordinary']+[k for k in v.schedule(m)[0] if k not in EARLY]
def display_review(text,h):
    path=v.artifact_path(text,'M1 artifacts: ');return oracle.review(path,1280 if h==720 else 1920,h),[path]
def flavors(output):
    data=base.read(output/'flavors.json');base.require([r['flavor'] for r in data]==runtime.FLAVORS+['device-test','display-test'] and all(r['exit_code']==0 for r in data),'separate flavor matrix')
    return dict(status='passed',directory=str(output),flavors=data),[output]
def boot_leaf(path,key):
    need=base.require; path=Path(path)
    need(base.read(path/'result.json')['status']=='passed','boot leaf result')
    cases=sorted(path.glob('*-run/case-result.json'))
    need(len(cases)==(4 if key=='boot-metadata' else 1),'boot leaf case count')
    summaries=[]
    for casepath in cases:
        run=casepath.parent; case=base.read(casepath);fixture=case['fixture']
        build=path/'injection-kernel' if fixture=='serial-only' else path
        m=base.read(build/'build-manifest.json');inputs=base.read(build/'build-inputs.json')
        for name,sha in inputs['sources'].items():need(base.digest(base.ROOT/name)==sha,'stale boot leaf source '+name)
        image=base.read(path/(run.name[:-4]+'-package')/'iso-manifest.json')
        need(case['status']=='passed' and case['cleanup_reaped'] and not case['timed_out'] and case['deadline_ms']==30000,'boot leaf lifecycle')
        need(base.digest(build/'kernel.elf')==m['kernel_sha256']==image['kernel_sha256'],'boot leaf kernel identity')
        need(base.digest(path/(run.name[:-4]+'-package')/'continuum.iso')==image['iso_sha256']==case['iso_sha256'],'boot leaf optical identity')
        records=[]
        for line in (run/'serial.log').read_text().splitlines():
            if not line.startswith('m1.'):continue
            words=line.split(); f=dict(word.split('=',1) for word in words[1:])
            need(len(f)==len(words)-1 and f['build']==m['build_id'] and f['session']==image['host_session_id'] and f['version']=='5' and int(f['seq'])==len(records),'raw boot identity/sequence')
            need(f['outcome']=='ok' or (key=='scanout-guard' and words[0]=='m1.exception' and f['outcome']=='fatal'),'raw boot outcome')
            records.append(dict(event=words[0],fields=f))
        rows=oracle.rows; memory=rows(records,'m1.memory_ready');need(len(memory)==1,'owned memory checkpoint')
        need(all(memory[0][k]==v for k,v in dict(wp='true',nx='true',supervisor='true',image_audited='true',direct_audited='true',guards='6',aliases='0',reuse_zero='true').items()),'owned mapping properties')
        fatal=rows(records,'m1.exception')
        if key=='scanout-guard':
            armed=rows(records,'m1.fault_armed');need(len(fatal)==len(armed)==1 and not rows(records,'m1.ready') and case['emulator_test_exit']==37,'guard fatal terminal state')
            f=fatal[0];a=armed[0];need(f['vector']=='14' and int(f['error'],16)==0 and int(f['cs'],16)==8 and int(f['address'],16)==int(a['target'],16) and int(f['rip'],16)==m['inspection']['diagnostic_sites']['diagnostic_guard_site']==int(a['site'],16),'actual guard exception context')
        else:
            need(not fatal and len(rows(records,'m1.ready'))==1 and len(rows(records,'m1.boot_complete'))==1 and case['emulator_test_exit']==33,'positive boot terminal state')
        summary=rows(records,'m1.boot_info_validated');need(len(summary)==1,'owned boot descriptor checkpoint')
        if key=='boot-metadata':need(summary[0]['framebuffers']==('0' if fixture=='serial-only' else '1') and summary[0]['modules']==('1' if run.name=='module-run' else '0'),'optical resource variants')
        summaries.append(dict(fixture=fixture,session=image['host_session_id'],iso_sha256=image['iso_sha256'],kernel_sha256=m['kernel_sha256'],raw_records=len(records),memory=memory,fatal=fatal))
    if key=='boot-metadata':need(len({r['session'] for r in summaries})==4 and len({r['iso_sha256'] for r in summaries})==4,'fresh optical sessions')
    return dict(status='passed',scope='independent focused optical/page/exception review',cases=summaries),[path]

def leaf_review(key,text,folder):
    if key=='timer':return device.timer_review(text)
    if key in ('boot-metadata','scanout-memory','scanout-guard'):
        path=v.artifact_path(text,'M1 artifacts: ');return boot_leaf(path,key)
    return v.review_check(key,text,folder)
def gate(manifest,resume,probe=False):
    m=v.validate_manifest(base.read(manifest));planned=EARLY[:-1]+['display-720'] if probe else checks(m)
    if probe:m['execution_budget']['max_vm_launches']=1
    plan=v.execution_plan(m,planned);print(json.dumps(plan,indent=2),flush=True)
    folder=base.ROOT/'build/m4/P4-04'/(('probe-' if probe else 'acceptance-')+str(time.time_ns()));s=v.Session(m,folder,resume,planned_checks=planned);s.report['plan']=plan;s.persist()
    try:
        for key in (EARLY[:-1] if probe else EARLY):
            args,limit=v.catalog()[key];s.check(key,args,limit)
        for h in ([720] if probe else [720,1080]):
            text,_=s.check('display-'+str(h),['cargo','xtask','test','--suite','m1','--case','display-'+str(h)],600,lambda text,h=h:display_review(text,h))
        if probe:
            s.finish(planned);s.report['scope']='development-probe';s.persist();return
        desc=v.artifact_path(text,'M1 artifacts: ');out=folder/'flavors'
        s.check('display-flavors',v.PS+['tools/Check-P4-04Flavors.ps1','-Descriptors',str(desc),'-Output',str(out)],600,lambda text:flavors(out))
        s.check('ordinary',['cargo','xtask','run','--profile','vm-baseline'],600,lambda text:runtime.guest_review(text,False))
        for key in v.schedule(m)[0]:
            if key in EARLY:continue
            args,limit=v.catalog()[key];s.check(key,args,limit,lambda text,k=key:leaf_review(k,text,folder))
        s.finish(checks(m))
    except BaseException as e:
        s.report.update(status='interrupted' if isinstance(e,KeyboardInterrupt) else 'failed',error=str(e));s.persist();raise
    finally:print('P4-04 acceptance: '+str(folder/'acceptance.json'),flush=True)
def review(path):
    r=base.read(path);v.review_report(path,checks(r['manifest']))
    for c in r['checks']:
        k=c['id'];text=Path(c['stdout']).read_text(encoding='utf-8-sig')
        if k in ['display-720','display-1080']:
            base.require(c['arguments']==['cargo','xtask','test','--suite','m1','--case',k] and c['deadline_seconds']==600,'substituted display command');display_review(text,int(k.split('-')[1]))
        elif k=='display-flavors':
            first=next(row for row in r['checks'] if row['id']=='display-1080');desc=v.artifact_path(Path(first['stdout']).read_text(encoding='utf-8-sig'),'M1 artifacts: ');out=Path(c['review']['directory'])
            base.require(c['arguments']==v.PS+['tools/Check-P4-04Flavors.ps1','-Descriptors',str(desc),'-Output',str(out)] and c['deadline_seconds']==600,'substituted flavor command');flavors(out)
        elif k=='ordinary':
            base.require(c['arguments']==['cargo','xtask','run','--profile','vm-baseline'] and c['deadline_seconds']==600,'substituted ordinary command');runtime.guest_review(text,False)
        else:leaf_review(k,text,Path(path).parent)
    result=dict(status='passed',story='P4-04',acceptance_sha256=base.digest(path),plan=r['plan']);base.save(Path(path).parent/'review.json',result);print(json.dumps(result,indent=2));return result
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('action',choices=['plan','probe-plan','probe','run','review']);p.add_argument('--manifest',default='config/p4-04-validation.json');p.add_argument('--resume');p.add_argument('--acceptance');a=p.parse_args()
    if a.action in ['plan','probe-plan']:
        m=v.validate_manifest(base.read(a.manifest));planned=checks(m)
        if a.action=='probe-plan':m['execution_budget']['max_vm_launches']=1;planned=EARLY[:-1]+['display-720']
        print(json.dumps(v.execution_plan(m,planned),indent=2))
    elif a.action=='review':review(a.acceptance)
    else:gate(a.manifest,a.resume,a.action=='probe')
