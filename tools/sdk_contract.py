"""Independent integer/byte reference, fixtures and generated-binding audit.

Never imports or calls the production codec. Production executable is an external
consumer of the same adversarial bytes; this reference determines the outcomes.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / 'contracts/sdk-v1.json'

def generated():
    c = json.loads(CONTRACT.read_text())
    runtime = json.loads((ROOT/'contracts/runtime-v0.json').read_text())
    assert all(runtime['syscall']['dispatch_ids'][k] == v for k,v in c['syscalls'].items())
    assert c['errors'] == runtime['syscall']['error_ids']
    return '// Generated from contracts/sdk-v1.json by tools/sdk_contract.py. Do not hand edit.\n' + ''.join(
        f'pub const {k.upper()}: u64 = {v};\n' for k,v in (c['syscalls']|c['errors']).items())

def reference(kind, b):
    sizes = {'service_request':16,'startup':64,'discovery':32,'map':32,'attachment':24,'send':32,'receive':48,'metadata':64,'wait':32,'wait_item':32,'timer':24,'attempt':24,'context':40,'route':160}
    if kind == 'header':
        if not 48 <= len(b) <= 256: return 4
        protocol,major,minor,op,features,length,handles,reserved,request,route,policy = struct.unpack('<IHHIIIHHQQQ',b[:48])
        if protocol!=0x53444b or major!=1 or features & ~1: return 14
        return 0 if reserved==0 and length==len(b)-48 and handles<=4 and op in (1,2,3) else 4
    if kind not in sizes: return 12
    if kind=='startup':
        return 0 if len(b)==64 and b[:8]==b'CNTM3RT0' and b[8:24]==struct.pack('<IIQ',0,64,0) and b[32:40]==struct.pack('<II',4096,0) and b[48:]==bytes(16) else 14
    if len(b)!=sizes[kind]: return 4
    w=struct.unpack('<'+'Q'*(len(b)//8),b)
    if kind=='service_request':return 0 if w[1]==0 else 4
    if kind=='discovery':return 0 if w[:2]==(0,4096) else 14
    if kind=='map':return 0 if w[2]>0 and all(x%4096==0 for x in w[:3]) and w[3]<=2 and w[0]>=65536 and w[0]+w[2]<=0x80000000 and w[1]+w[2]<=2**64-1 else 4
    if kind=='attachment':
        h,rights,mode,res=struct.unpack('<QQII',b)
        return 0 if h and rights and rights&~4095==0 and mode<=1 and res==0 else 4
    if kind=='timer':return 0 if w[0]<2**64-1 and w[2]==0 else 4
    if kind=='wait_item':return 0 if w[0] and w[1]<=3 and w[2]<=256 and w[3]<=4 and (w[1]==3 or w[2:]==(0,0)) else 4
    if kind=='send':return 0 if w[1]<=256 and w[3]<=4 else 4
    if kind=='metadata':return 0 if w[0]<=256 and w[1]<=4 and w[6]<=1 and w[7]==0 else 4
    if kind=='wait':return 0 if 1<=w[1]<=8 else 4
    if kind=='context':return 0 if w[4]<=1 else 4
    return 0

def fixtures():
    words=lambda *v:struct.pack('<'+'Q'*len(v),*v)
    start=b'CNTM3RT0'+struct.pack('<IIQQIIQ',0,64,0,0,4096,0,0)+bytes(16)
    good={'service_request':words(1,0),'startup':start,'discovery':words(0,4096,101,1001),'map':words(0x60000000,0,4096,0),'attachment':struct.pack('<QQII',0x100000001,53,0,0),'send':words(0x400000,64,0,0),'receive':words(0x600000,256,0x610000,4,0x620000,64),'metadata':words(64,1,3,1,101,1001,0,0),'wait':words(0x600000,1,100000000,0),'wait_item':words(0x100000001,3,160,1),'timer':words(1000000,0,0),'attempt':words(0x600000,10000000,0),'context':words(3,1,101,1001,0),'route':words(*range(20))}
    for minor in (0,1,65535):
        for op in (1,2,3):
            good[f'header-{minor}-{op}']=struct.pack('<IHHIIIHHQQQ',0x53444b,1,minor,op,0,16,0,0,1,2,3)+bytes(16)
    rows=[]
    for name,b in good.items():
        kind=name.split('-')[0]
        rows.append((name,kind,b))
        for length in range(len(b)):
            rows.append((f'{name}-truncated-{length}',kind,b[:length]))
        rows.append((name+'-trailing',kind,b+b'\0'))
        for at in range(len(b)):
            for bit in (1,128,255):
                mutation=bytearray(b);mutation[at]^=bit
                rows.append((f'{name}-byte-{at}-{bit}',kind,bytes(mutation)))
    for rights in (0,1,53,4095,4096,2**64-1):
        for mode in (0,1,2,2**32-1):rows.append((f'attachment-{rights}-{mode}','attachment',struct.pack('<QQII',1,rights,mode,0)))
    rows.append(('unknown-record','unknown',b''))
    return [{'name':n,'kind':k,'hex':b.hex(),'status':reference(k,b)} for n,k,b in rows]

def fields(kind,b):
    # Independent semantic decoder. No production implementation is imported.
    if kind=='startup':return [struct.unpack_from('<Q',b,i)[0] for i in (24,40)]
    if kind=='header':
        v=list(struct.unpack('<IHHIIIHHQQQ',b[:48]));del v[7];return v
    if kind=='attachment':return list(struct.unpack('<QQII',b))
    return list(struct.unpack('<'+'Q'*(len(b)//8),b))

def check_decoded(row,line):
    status,canonical,decoded=line.split();assert int(status)==row['status'],row['name']
    if row['status']==0:
        assert canonical==row['hex'],row['name']
        assert list(map(int,decoded.split(',')))==fields(row['kind'],bytes.fromhex(row['hex'])),row['name']+' semantic fields'

def check_produced(row,line):
    status,canonical,_=line.split();assert status=='0' and canonical==row['hex'],row['name']+' independent producer bytes'

def semantic_check(rows,production,output):
    built=subprocess.run(['pwsh','-NoProfile','-File',str(ROOT/'tools/Build-P3-12CodecConsumer.ps1'),'-Production',str(production.resolve()),'-Output',str(output.resolve())],capture_output=True,text=True,timeout=60,creationflags=subprocess.CREATE_NO_WINDOW)
    (output/'semantic-builder.stdout.log').write_text(built.stdout,encoding='utf-8');(output/'semantic-builder.stderr.log').write_text(built.stderr,encoding='utf-8')
    assert built.returncode==0,built.stderr
    producers=[]
    # Every valid mutation supplies independent field values to the production
    # encoder, rather than decoding/re-encoding through the same implementation.
    for row in rows:
        if row['status'] or row['kind']=='startup':continue
        b=bytes.fromhex(row['hex']);w=fields(row['kind'],b)
        producers.append((row,'encode-'+row['kind']+' '+','.join(map(str,w))+' '+(b[48:].hex() if row['kind']=='header' else '')))
    lines=''.join(f"{r['kind']} {r['hex']}\n" for r in rows)+''.join(line+'\n' for _,line in producers)
    run=subprocess.run([str((output/'semantic-consumer.exe').resolve())],input=lines,capture_output=True,text=True,timeout=30,creationflags=subprocess.CREATE_NO_WINDOW)
    (output/'semantic.stdout.log').write_text(run.stdout,encoding='utf-8');(output/'semantic.stderr.log').write_text(run.stderr,encoding='utf-8');assert run.returncode==0,run.stderr
    observed=run.stdout.splitlines();assert len(observed)==len(rows)+len(producers)
    for row,line in zip(rows,observed):check_decoded(row,line)
    for (row,_),line in zip(producers,observed[len(rows):]):check_produced(row,line)
    # A matching route/policy swap would survive a round trip. Explicitly show
    # that the independent semantic and producer comparisons reject that change.
    header=next(r for r in rows if r['name']=='header-0-1');b=bytes.fromhex(header['hex']);expected=fields('header',b);swapped=expected.copy();swapped[-2:]=reversed(swapped[-2:]);assert swapped!=expected
    altered=bytearray(b);altered[32:40],altered[40:48]=b[40:48],b[32:40];assert bytes(altered)!=b and reference('header',bytes(altered))==0
    for check,line in [(check_decoded,'0 '+header['hex']+' '+','.join(map(str,swapped))),(check_produced,'0 '+bytes(altered).hex()+' -')]:
        try:check(header,line)
        except AssertionError:pass
        else:raise AssertionError('paired encoder/decoder field-swap escaped independent comparison')
    return {'decoded_cases':len(rows),'independent_producer_cases':len(producers),'paired_field_swap_rejected':True,'build':json.loads((output/'semantic-build.json').read_text(encoding='utf-8-sig'))}

def main():
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--production',type=Path);p.add_argument('--generate',action='store_true');a=p.parse_args()
    dest=ROOT/'sdk/continuum-rt/src/generated.rs'
    if a.generate:dest.write_text(generated())
    assert dest.read_text()==generated(),'Generated bindings drift'
    assert (ROOT/'sdk/continuum-rt/contract.json').read_bytes()==CONTRACT.read_bytes(),'Packaged contract drift'
    rows=fixtures();a.output.mkdir(parents=True,exist_ok=True)
    packaged=[r for r in rows if 'truncated' not in r['name'] and '-byte-' not in r['name'] and 'trailing' not in r['name']]
    assert json.loads((ROOT/'sdk/continuum-rt/fixtures.json').read_text())==packaged,'Packaged byte fixtures drift'
    (a.output/'fixtures.json').write_text(json.dumps(rows,indent=2)+'\n')
    if not a.production:return
    lines=''.join(f"{r['kind']} {r['hex']}\n" for r in rows)
    result=subprocess.run([str(a.production.resolve())],input=lines,text=True,capture_output=True,timeout=30,creationflags=subprocess.CREATE_NO_WINDOW)
    (a.output/'production.stdout.log').write_text(result.stdout);(a.output/'production.stderr.log').write_text(result.stderr)
    assert result.returncode==0,result.stderr
    observed=result.stdout.splitlines();assert len(observed)==len(rows)
    for row,line in zip(rows,observed):
        status,canonical=line.split();assert int(status)==row['status'],row['name']
        if row['status']==0:assert canonical==row['hex'],row['name']
    # Production and reference classifications must agree on every byte mutation.
    rejected=0
    for row in rows:
        if row['status']!=0:rejected+=1
    semantic=semantic_check(rows,a.production,a.output)
    report={'status':'passed','cases':len(rows),'rejections':rejected,'canonical_successes':len(rows)-rejected,'contract_sha256':hashlib.sha256(CONTRACT.read_bytes()).hexdigest(),'production_sha256':hashlib.sha256(a.production.read_bytes()).hexdigest(),'fixtures_sha256':hashlib.sha256((a.output/'fixtures.json').read_bytes()).hexdigest(),'reference_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
    report['semantic']=semantic
    (a.output/'conformance.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
if __name__=='__main__':main()
