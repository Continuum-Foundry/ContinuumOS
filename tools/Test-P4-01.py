"""Exercise locked P4 planning inputs, independent draft codecs and real host faults."""
import argparse
import copy
import importlib.util
import json
from pathlib import Path
import random
import sys
import time
import uuid

import m4_codecs as codec
import m4_prerequisites as plan
import m4_reference as reference


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--image', type=Path, required=True)
    parser.add_argument('--sha256', required=True)
    parser.add_argument('--stress-run', type=Path, required=True)
    args = parser.parse_args()
    run = plan.ROOT / 'build/m4/P4-01' / f'host-{time.time_ns()}-{uuid.uuid4().hex}'
    run.mkdir(parents=True)
    rows = []
    report = {'story': 'P4-01', 'scope': 'host-plan-only; baseline M1/M3/M2 acceptance remains separate',
              'status': 'running', 'rows': rows, 'profile_sha256': plan.digest(plan.PROFILE), 'image_sha256': args.sha256}

    def check(name, condition):
        plan.require(condition, name)
        rows.append({'name': name, 'status': 'passed'})

    def rejects(function):
        try:
            function()
        except (ValueError, OSError, KeyError, TypeError, OverflowError):
            return True
        return False

    try:
        value = plan.read(plan.PROFILE)
        plan.validate(value); reference.profile(reference.load(plan.PROFILE))
        check('independent-profile-constraints', True)
        mutations = []
        def leaves(node, path=()):
            if isinstance(node, dict):
                for key, item in node.items():
                    yield from leaves(item, (*path, key))
            elif isinstance(node, list):
                for index, item in enumerate(node):
                    yield from leaves(item, (*path, index))
            else:
                yield path, node
        for path, old in leaves(value):
            bad = copy.deepcopy(value); parent = bad
            for key in path[:-1]: parent = parent[key]
            parent[path[-1]] = (not old) if type(old) is bool else old + 1 if type(old) is int else old + '-bad'
            plan.require(rejects(lambda: plan.validate(bad)) and rejects(lambda: reference.profile(bad)), 'profile mutation accepted: ' + str(path))
            mutations.append(list(path))
        for bad in ({**value, 'unexpected': 1}, {k:v for k,v in value.items() if k!='bounds'}):
            check('unknown-or-missing-profile-field', rejects(lambda: plan.validate(bad)) and rejects(lambda: reference.profile(bad)))
        check('all-profile-leaves-fail-closed', len(mutations) > 100)
        for name, data in [('duplicate', b'{"a":1,"a":2}'), ('malformed', b'{'), ('oversized', b' '*65537)]:
            path = run / (name + '.json'); path.write_bytes(data)
            check('profile-reader-' + name, rejects(lambda: plan.read(path)) and rejects(lambda: reference.load(path)))
        plans=[]
        for width,height in ((1280,720),(1920,1080)):
            for enabled in (False,True):
                item = plan.plan(value,width,height,enabled,port=49152 if enabled else None)
                text=' '.join(item['arguments'])
                check(f'device-plan-{width}-{enabled}',
                      ('virtio-serial-pci' in text)==enabled and ('-chardev' in text)==enabled
                      and ('127.0.0.1' in text)==enabled and item['bootstrap_development_authority']==enabled
                      and bool(item['device_grants'])==enabled and 'diagnostic_serial' in item)
                plans.append(item)
        for kwargs in ({'width':3840,'height':2160}, {'width':1280,'height':720,'development':False,'port':49152},
                       {'width':1280,'height':720,'development':True,'port':0}, {'width':1280,'height':720,'development':1,'port':49152}):
            check('reject-unsupported-device-plan', rejects(lambda: plan.plan(value,**kwargs)))
        spec=importlib.util.spec_from_file_location('p401_deadline_consumer',plan.ROOT/'tools/Review-P3-02.py')
        consumer=importlib.util.module_from_spec(spec);spec.loader.exec_module(consumer)
        actual=args.stress_run.resolve()
        plan.require(actual.is_relative_to(plan.ROOT),'stress artifact outside workspace')
        guest=next(p for p in actual.iterdir() if p.is_dir() and p.name.endswith('-run'))
        source={name:json.loads((path).read_text(encoding='utf-8-sig')) for name,path in {
            'run-result.json':guest/'run-result.json','launch-manifest.json':guest/'launch-manifest.json',
            'case-result.json':guest/'case-result.json','build-manifest.json':actual/'build-manifest.json',
            'build-inputs.json':actual/'build-inputs.json'}.items()}
        isolated=run/'deadline-consumer';(isolated/'guest').mkdir(parents=True)
        def write_deadline(files):
            for name,data in files.items():
                plan.save((isolated if name.startswith('build-') else isolated/'guest')/name,data)
        write_deadline(source)
        check('independent-scoped-deadline-control',consumer.lifecycle(isolated/'guest')[0]['deadline_ms']==60000)
        deadline_mutations=[
            ('result-bound','run-result.json',['deadline_ms'],60001),
            ('launch-bound','launch-manifest.json',['vm','deadline_ms'],30000),
            ('profile-name','launch-manifest.json',['deadline_profile'],'unknown'),
            ('original-bound','launch-manifest.json',['original_guest_deadline_ms'],60000),
            ('profile-hash','launch-manifest.json',['stress_profile_sha256'],'0'*64),
            ('build-profile-hash','build-inputs.json',['sources','config/p3-13-stress.json'],'0'*64),
            ('build-flavor','build-manifest.json',['flavor'],'route-test'),
            ('packaged-flavor','case-result.json',['package','build_flavor'],'route-test'),
            ('input-flavor','build-inputs.json',['flavor'],'route-test')]
        for name,file,path,replacement in deadline_mutations:
            altered=copy.deepcopy(source);parent=altered[file]
            for key in path[:-1]:parent=parent[key]
            parent[path[-1]]=replacement;write_deadline(altered)
            check('reject-deadline-'+name,rejects(lambda:consumer.lifecycle(isolated/'guest')))
        altered=copy.deepcopy(source)
        altered['build-manifest.json']['flavor']=altered['build-inputs.json']['flavor']=altered['case-result.json']['package']['build_flavor']='route-test'
        write_deadline(altered)
        check('reject-stress-bound-on-ordinary-guest',rejects(lambda:consumer.lifecycle(isolated/'guest')))
        write_deadline(source)
        fixtures = plan.read(plan.ROOT / 'fixtures/m4/desktop-v0/fixtures.json')
        cases = 0; negative_cases = []
        def paired(family, raw, name, must_reject=False):
            nonlocal cases
            outcomes=[]
            for decoder in (codec.decode,reference.desktop):
                try: outcomes.append(('accepted',decoder(family,raw)))
                except ValueError: outcomes.append(('rejected',None))
            plan.require(outcomes[0]==outcomes[1], 'independent codec disagreement: '+name)
            if must_reject:
                plan.require(outcomes[0][0]=='rejected','codec fault accepted: '+name)
                negative_cases.append(name)
            cases += 1
        for fixture in fixtures['cases']:
            family=fixture['family']; raw=bytes.fromhex(fixture['hex']); label=bytes.fromhex(fixture['label_hex'])
            encoded=codec.encode(family,fixture['values'],label=label,**fixtures['envelope'])
            check('literal-byte-fixture-'+family, encoded==raw and codec.decode(family,raw)==(tuple(fixture['values']),label))
            paired(family,raw,'literal-'+family)
            for length in range(len(raw)):
                paired(family,raw[:length],f'{family}-truncate-{length}',True)
            paired(family,raw+b'\0',family+'-trailing',True)
            for offset,size in ((4,2),(6,2),(8,4),(12,2),(14,2),(16,8),(24,8),(32,8),(40,8),(48,8),(56,4),(60,4)):
                bad=bytearray(raw)
                bad[offset:offset+size]=(0 if offset in (16,24,32,40,48) else 65535).to_bytes(size,'little')
                paired(family,bytes(bad),f'{family}-header-{offset}',True)
            swapped=bytearray(raw); swapped[32:40],swapped[40:48]=raw[40:48],raw[32:40]
            paired(family,bytes(swapped),family+'-resource-output-swap',True)
            # Complete-body malformed fields, authored as byte offsets rather
            # than invoking the producer's own field or validation tables.
            changes={'surface':[(80,4,0),(88,4,1),(92,4,2),(96,8,2**64-1),(104,4,2**32-1)],
                     'input':[(64,8,0),(92,4,9),(108,4,256)],
                     'semantic':[(64,8,0),(72,8,5),(104,4,99),(112,4,65536),(116,4,257)],
                     'output':[(72,8,2),(80,4,3840),(92,4,0),(96,4,1)]}
            for offset,size,number in changes[family]:
                bad=bytearray(raw);bad[offset:offset+size]=number.to_bytes(size,'little')
                paired(family,bytes(bad),f'{family}-body-{offset}',True)
            if family=='semantic':
                bad=bytearray(raw);bad[-1]=255;paired(family,bytes(bad),'semantic-invalid-UTF8',True)
            rng=random.Random(20261106)
            for _ in range(200):
                bad=bytearray(raw); offset=rng.randrange(len(raw));bad[offset]^=1<<rng.randrange(8)
                paired(family,bytes(bad),'deterministic-byte-mutation')
        frame=fixtures['dev']; raw=bytes.fromhex(frame['hex'])
        check('independent-dev-literal',codec.encode_dev(bytes.fromhex(frame['payload_hex']),sequence=frame['sequence'],kind=frame['kind'])==raw)
        check('independent-dev-consumer',codec.decode_dev(raw,development=True,authenticated=True)==reference.dev(raw,development=True,authenticated=True))
        for label,kwargs in [('disabled',{}),('unauthenticated',{'development':True}),('replay',{'development':True,'authenticated':True,'sequence':2})]:
            check('reject-dev-'+label,rejects(lambda: codec.decode_dev(raw,**kwargs)) and rejects(lambda: reference.dev(raw,**kwargs)))
        for at,size,number in ((4,2,2),(8,4,31),(12,4,1048577),(16,8,0),(24,2,13),(26,2,1),(28,4,1)):
            bad=bytearray(raw);bad[at:at+size]=number.to_bytes(size,'little')
            check('dev-frame-negative-'+str(at),rejects(lambda:codec.decode_dev(bytes(bad),development=True,authenticated=True)) and rejects(lambda:reference.dev(bytes(bad),development=True,authenticated=True)))
        for length in range(len(raw)):
            check('dev-truncation',rejects(lambda:codec.decode_dev(raw[:length],development=True,authenticated=True)) and rejects(lambda:reference.dev(raw[:length],development=True,authenticated=True)))
        children=[]
        for case,outcome in [('success','complete'),('nonzero','exited-nonzero'),('missing-final','missing-final-record'),('timeout','timeout')]:
            path,result=plan.fixture(plan.PROFILE,args.image,args.sha256,case)
            check('real-host-child-'+case,result['outcome']==outcome and result['launched'] and result['cleanup']=='reaped')
            children.append({'case':case,'path':str(path/'result.json'),'sha256':plan.digest(path/'result.json')})
        invalid=run/'invalid-profile.json';plan.save(invalid,{**value,'unknown':1})
        badimage=run/'bad.elf';badimage.write_bytes(b'x'*64)
        for name,profile,image,sha,case,tool in (
            ('profile',invalid,args.image,args.sha256,'success',None),
            ('missing-profile',run/'absent.json',args.image,args.sha256,'success',None),
            ('missing-image',plan.PROFILE,run/'absent.elf',args.sha256,'success',None),
            ('changed-image',plan.PROFILE,args.image,'0'*64,'success',None),
            ('malformed-image',plan.PROFILE,badimage,plan.digest(badimage),'success',None),
            ('missing-tool',plan.PROFILE,args.image,args.sha256,'success',run/'absent.exe'),
            ('alternate-tool',plan.PROFILE,args.image,args.sha256,'success',badimage),
            ('unknown-case',plan.PROFILE,args.image,args.sha256,'arbitrary',None)):
            path,result=plan.fixture(profile,image,sha,case,tool)
            check('prelaunch-reject-'+name,result['outcome']=='validation-rejected' and not result['launched'] and result['cleanup']=='not-launched')
            children.append({'case':name,'path':str(path/'result.json'),'sha256':plan.digest(path/'result.json')})
        device=plan.inventory(run/'devices')
        check('host-fixture-output-paths-unique',len({child['path'] for child in children})==12)
        plan.save(run/'plans.json',plans)
        report.update(status='host-plan-passed',profile_mutations=len(mutations),codec_cases=cases,codec_required_negatives=len(negative_cases),
                      scoped_deadline_negatives=len(deadline_mutations)+1,
                      children=children,devices=device,plans_sha256=plan.digest(run/'plans.json'),
                      inputs={p:plan.digest(plan.ROOT/p) for p in ['tools/m4_prerequisites.py','tools/m4_codecs.py','tools/m4_reference.py','tools/Test-P4-01.py','tools/Review-P3-02.py','config/p3-13-stress.json','config/m4-profiles.json','fixtures/m4/desktop-v0/fixtures.json']},
                      limits=['host byte/profile/lifecycle tests only','no actual guest desktop or driver acceptance','no cryptographic implementation','no newly frozen SDK ID'])
    except BaseException as error:
        report.update(status='failed',error=str(error));raise
    finally:
        plan.save(run/'result.json',report)
        print('P4-01 host review: '+str(run/'result.json'),flush=True)


if __name__=='__main__':
    main()
