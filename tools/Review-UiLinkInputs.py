"""Verify actual offline P2-07 dependency/source/notice inputs, separately by target graph."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
if sys.flags.optimize: raise RuntimeError("Run the input auditor without Python -O.")
import tarfile
import tomllib

root=Path(__file__).resolve().parent.parent
def digest(data): return hashlib.sha256(data).hexdigest()
def main():
    p=argparse.ArgumentParser();p.add_argument('output',type=Path);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
    manifest='experiments/ui-link/Cargo.toml'
    def command(*args): return subprocess.check_output(['cargo',*args],cwd=root).decode('utf-8')
    lock=tomllib.loads((root/'experiments/ui-link/Cargo.lock').read_text())
    pins={(p['name'],p['version']):p.get('checksum') for p in lock['package']}
    previous=tomllib.loads(subprocess.check_output(['git','show','HEAD:Cargo.lock'],cwd=root).decode())
    old={(p['name'],p['version']):p['checksum'] for p in previous['package'] if 'checksum' in p}
    assert all(old.get(k)==v for k,v in pins.items() if v), 'new or changed registry pin'
    results={};verified={};files=0
    for candidate,features in [('custom',[]),('slint-aot',['--features','slint-aot'])]:
        common=['--locked','--offline','--manifest-path',manifest,*features]
        meta=json.loads(command('metadata',*common,'--format-version','1','--filter-platform','x86_64-unknown-none'))
        packages={(p['name'],p['version']):p for p in meta['packages']}
        graphs={}
        for label,edges in [('runtime','normal,no-proc-macro'),('build','normal,build')]:
            raw=command('tree',*common,'--target','x86_64-unknown-none','-e',edges,'--prefix','none','--format','{p}|{f}','--no-dedupe')
            (a.output/(candidate+'-'+label+'-tree.txt')).write_text(raw.replace(str(root),'${WORKSPACE}'),encoding='utf-8')
            graph={}
            for line in raw.splitlines():
                match=re.match(r'([^ ]+) v([^ |]+).*?\|(.*)',line);assert match,line
                n,v,f=match.groups();graph.setdefault((n,v),set()).update(filter(None,f.replace(' (*)','').split(',')))
            graphs[label]=graph
        for (name,version),features in graphs['runtime'].items():
            assert not name.startswith(('winit','i-slint-backend-winit','i-slint-backend-linuxkms','wgpu','fontique','parley','i-slint-compiler','slint-macros')),(name,features)
            # A no_std declaration alone cannot prevent accidental std through features.
            assert 'std' not in features,(name,version,features)
        rows=[]
        for key in sorted(graphs['build']):
            package=packages[key]
            if key not in verified and package['source']:
                source=Path(package['manifest_path']).parent
                archive=next((root/'work/toolchains/cargo/registry/cache').glob(f'*/{key[0]}-{key[1]}.crate'))
                assert digest(archive.read_bytes())==pins[key],key
                count=0
                with tarfile.open(archive) as stream:
                    for member in stream.getmembers():
                        if not member.isfile(): continue
                        relative=Path(*Path(member.name).parts[1:]);assert not relative.is_absolute() and '..' not in relative.parts
                        assert (source/relative).read_bytes()==stream.extractfile(member).read(),(key,str(relative))
                        count+=1
                files+=count
                notices=[]
                for file in sorted(source.rglob('*')):
                    if file.is_file() and file.name.upper().startswith(('LICENSE','COPYING','NOTICE')):
                        notices.append(dict(path=file.relative_to(source).as_posix(),sha256=digest(file.read_bytes())))
                verified[key]=dict(registry_checksum=pins[key],archive_and_expanded_source_verified=True,source_files=count,notices=notices)
            rows.append(dict(name=key[0],version=key[1],normal_runtime=key in graphs['runtime'],normal_features=sorted(graphs['runtime'].get(key,[])),build_features=sorted(graphs['build'][key]),license=package['license'],rust_version=package['rust_version'],build_script=any('custom-build' in t['kind'] for t in package['targets']),proc_macro=any('proc-macro' in t['kind'] for t in package['targets']),**verified.get(key,{})))
        results[candidate]=dict(normal_runtime_packages=len(graphs['runtime']),active_build_packages=len(rows),packages=rows)
    report=dict(schema_version=1,status='passed',target='x86_64-unknown-none',registry_pins_match_prior_root_lock=True,unique_source_files_verified=files,lock_sha256=digest((root/'experiments/ui-link/Cargo.lock').read_bytes()),root_lock_sha256=digest((root/'Cargo.lock').read_bytes()),candidates=results,scope='normal edges and host compiler/build inputs; final ELF audit separately establishes linked symbol closure; this is not a release SBOM',distribution_cleared=False)
    (a.output/'inputs.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:{x:v[x] for x in ('normal_runtime_packages','active_build_packages')} for k,v in results.items()}))
if __name__=='__main__':main()
