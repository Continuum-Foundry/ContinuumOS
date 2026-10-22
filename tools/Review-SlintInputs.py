"""Offline review of exact Slint build inputs, active feature graphs and source bytes.

Cargo metadata can unify host/target/conditional features. Use cargo tree to label
actual selected normal edges separately from build/proc-macro edges. Neither tree
is called an artifact-level SBOM or proof of guest linkage.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tarfile
import tomllib

parser=argparse.ArgumentParser()
parser.add_argument('output',type=Path)
parser.add_argument('--base',default='34dc796')
args=parser.parse_args()
root=Path(__file__).resolve().parent.parent
out=args.output.resolve();out.mkdir(parents=True,exist_ok=True)
def command(*argv):
    return subprocess.check_output(argv,cwd=root).decode('utf-8')
def digest(data):return hashlib.sha256(data).hexdigest()
def write(name,value):
    (out/name).write_text(json.dumps(value,indent=2,ensure_ascii=True)+'\n',encoding='utf-8')
base=tomllib.loads(command('git','show',args.base+':Cargo.lock'))
lock=tomllib.loads((root/'Cargo.lock').read_text(encoding='utf-8'))
probe=tomllib.loads((root/'experiments/dependencies/Cargo.lock').read_text(encoding='utf-8'))
index=lambda x:{(p['name'],p['version']):p for p in x['package']}
before,pins,previous=map(index,[base,lock,probe])
added={k:v for k,v in pins.items() if k not in before}
assert not set(before)-set(pins),'Existing dependency pin removed'
assert all(k in previous and p.get('checksum')==previous[k].get('checksum') for k,p in added.items()),'Unreviewed novel pin'
meta=json.loads(command('cargo','metadata','--locked','--offline','--format-version','1','--features','slint-candidate','--filter-platform','x86_64-pc-windows-msvc'))
metadata={(p['name'],p['version']):p for p in meta['packages']}
def tree(edges):
    raw=command('cargo','tree','--locked','--offline','-p','xtask','--features','slint-candidate','--target','x86_64-pc-windows-msvc','-e',edges,'--prefix','none','--format','{p}|{f}','--no-dedupe')
    result={}
    for line in raw.splitlines():
        match=re.match(r'([^ ]+) v([^ |]+).*?\|(.*)',line)
        assert match,line
        name,version,features=match.groups()
        result.setdefault((name,version),set()).update(filter(None,features.replace(' (*)','').split(',')))
    return raw,result
normal,runtime=tree('normal,no-proc-macro')
full,all_inputs=tree('normal,build')
(out/'runtime-tree.txt').write_text(normal.replace(str(root),'${WORKSPACE}'),encoding='utf-8')
(out/'build-tree.txt').write_text(full.replace(str(root),'${WORKSPACE}'),encoding='utf-8')
# Disabled backend/host font dependencies must never enter the renderer's runtime closure.
for name,version in runtime:
    assert not name.startswith(('winit','i-slint-backend-winit','i-slint-backend-linuxkms','wgpu','fontique','parley','i-slint-compiler','slint-macros')),(name,version)
for key,features in runtime.items():
    if key[0] in ('slint','i-slint-core','i-slint-renderer-software'):
        assert 'std' not in features,(key,features)
rows=[];checked_files=0
for key in sorted(all_inputs):
    p=metadata[key]
    row={'name':key[0],'version':key[1],'normal_runtime':key in runtime,'normal_features':sorted(runtime.get(key,[])),'build_graph_features':sorted(all_inputs[key]),'scope':'normal runtime and/or host compiler dependency; not guest link evidence','license':p['license'],'source':p['source']}
    if p['source']:
        archive=next((root/'work/toolchains/cargo/registry/cache').glob(f'*/{key[0]}-{key[1]}.crate'))
        row['registry_checksum']=pins[key]['checksum']
        assert digest(archive.read_bytes())==row['registry_checksum'],key
        source=Path(p['manifest_path']).parent
        count=0
        with tarfile.open(archive) as stream:
            for member in stream.getmembers():
                if not member.isfile():continue
                relative=Path(*Path(member.name).parts[1:])
                assert not relative.is_absolute() and '..' not in relative.parts
                target=source/relative
                assert target.is_file() and target.read_bytes()==stream.extractfile(member).read(),(key,str(relative))
                count+=1
        row['archive_and_expanded_source_verified']=True
        row['source_file_count']=count;checked_files+=count
    rows.append(row)
write('dependencies.json',rows)
write('input-review.json',{'schema_version':1,'status':'passed','base':args.base,'added_lock_entries':len(added),'all_added_pins_match_prior_phase_0_probe':True,'existing_pins_removed':False,'offline':True,'active_packages':len(rows),'normal_runtime_packages':len(runtime),'source_files_verified':checked_files,'lock_sha256':digest((root/'Cargo.lock').read_bytes()),'runtime_feature_source':'cargo tree normal,no-proc-macro; metadata union is deliberately not treated as a runtime feature graph','guest_compile_or_link_tested':False,'distribution_cleared':False})
print(f'Offline Slint input review passed: {len(rows)} packages, {checked_files} source files; {out}')
