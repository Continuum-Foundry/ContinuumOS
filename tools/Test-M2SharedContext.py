"""Reject semantic drift while preserving the original UI build-context pins."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('m2_gate',ROOT/'tools/Run-P2Acceptance.py')
gate=importlib.util.module_from_spec(spec);spec.loader.exec_module(gate)


def main():
    rows=[]
    old={p:subprocess.check_output(['git','-c','protocol.allow=never','show',gate.FROZEN_CONTEXT_REVISION+':'+p],cwd=ROOT,
                                 timeout=10,env={**os.environ,'GIT_NO_LAZY_FETCH':'1'})
         for p in ('.cargo/config.toml','Cargo.toml')}
    current={p:(ROOT/p).read_bytes() for p in old}
    for path in old:
        gate.shared_context_equivalent(path,current[path],old[path])
        gate.shared_context_equivalent(path,current[path]+b'\n# context-only comment\n',old[path])
        rows.append({'name':'reviewed-current-'+path,'status':'passed'})
    config=current['.cargo/config.toml'].replace(b'\r\n',b'\n')
    manifest=current['Cargo.toml'].replace(b'\r\n',b'\n')
    variants=[
        ('alias','.cargo/config.toml',config.replace(b'run --locked --package',b'run --package')),
        ('offline-false','.cargo/config.toml',config.replace(b'offline = true',b'offline = false')),
        ('offline-type','.cargo/config.toml',config.replace(b'offline = true',b'offline = 1')),
        ('global-target','.cargo/config.toml',config+b'\n[build]\ntarget="x86_64-unknown-none"\n'),
        ('extra-alias','.cargo/config.toml',config.replace(b'[alias]',b'[alias]\nunreviewed="build"')),
        ('m2-profile','Cargo.toml',manifest.replace(b'[profile.m2-comparison]\ninherits = "release"\nopt-level = 3',b'[profile.m2-comparison]\ninherits = "release"\nopt-level = 1')),
        ('workspace','Cargo.toml',manifest.replace(b'members = ["tools/xtask"',b'members = ["unreviewed", "tools/xtask"',1)),
        ('edition','Cargo.toml',manifest.replace(b'edition = "2024"',b'edition = "2021"')),
        ('kernel-profile','Cargo.toml',manifest.replace(b'[profile.kernel-stress]\ninherits = "kernel-dev"\nopt-level = 3',b'[profile.kernel-stress]\ninherits = "kernel-dev"\nopt-level = 2')),
        ('kernel-profile-type','Cargo.toml',manifest.replace(b'[profile.kernel-stress]\ninherits = "kernel-dev"\nopt-level = 3',b'[profile.kernel-stress]\ninherits = "kernel-dev"\nopt-level = 3.0')),
        ('extra-profile','Cargo.toml',manifest+b'\n[profile.unreviewed]\ninherits="dev"\n'),
        ('malformed','Cargo.toml',b'[broken')]
    for name,path,data in variants:
        gate.require(data != current[path].replace(b'\r\n',b'\n'),'ineffective negative fixture: '+name)
        try:gate.shared_context_equivalent(path,data,old[path])
        except (ValueError,KeyError):rows.append({'name':name,'status':'rejected'})
        else:raise ValueError('accepted semantic drift: '+name)
    for name,call in [
        ('unrelated-source',lambda:gate.shared_context_equivalent('crates/slint-ui/src/lib.rs',b'',b'')),
        ('wrong-archive-pin',lambda:gate.archived_shared_context({'path':'Cargo.toml','sha256':'0'*64},manifest))]:
        try:call()
        except ValueError:rows.append({'name':name,'status':'rejected'})
        else:raise ValueError('accepted context fault: '+name)
    selection=gate.read(ROOT/'legal/ui-selection.json')
    context=gate.review_pinned_inputs(selection)
    gate.require({row['path'] for row in context}==set(old),'unexpected current context delta')
    print(json.dumps({'status':'passed','historical_pins_unchanged':True,'negative_cases':len(variants)+2,
                      'rows':rows,'shared_context':context},indent=2))


if __name__=='__main__':main()
