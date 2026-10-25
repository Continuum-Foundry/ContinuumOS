"""Independent, fail-closed ELF and LLVM audit for P2-07. Never executes the ELF."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import struct
import sys
if sys.flags.optimize: raise RuntimeError("Run the auditor without Python -O; assertions enforce audit checks.")
import subprocess

def inspect(data):
    def u(at, fmt):
        size = struct.calcsize(fmt)
        assert 0 <= at <= len(data)-size, 'truncated ELF'
        return struct.unpack_from('<'+fmt,data,at)[0]
    def span(at,size):
        assert 0 <= at <= len(data) and 0 <= size <= len(data)-at, 'out-of-file range'
        return data[at:at+size]
    assert span(0,7) == b'\x7fELF\x02\x01\x01', 'not ELF64 LE'
    assert u(16,'H') == 2 and u(18,'H') == 62 and u(20,'I') == 1, 'not x86-64 ET_EXEC'
    entry=u(24,'Q'); assert entry == 0x400000, 'wrong probe entry'
    ph, ps, pn = u(32,'Q'),u(54,'H'),u(56,'H')
    assert ps == 56 and 0 < pn <= 64, 'invalid program headers'
    span(ph,ps*pn)
    segments=[];ranges=[];executable=False
    for at in range(ph,ph+ps*pn,ps):
        kind,flags=u(at,'I'),u(at+4,'I')
        assert kind not in (2,3,7), 'dynamic/interpreter/TLS segment'
        assert kind != 0x6474e551 or not flags & 1, 'executable stack'
        if kind != 1: continue
        off,va,fs,ms,align=(u(at+n,'Q') for n in (8,16,32,40,48))
        assert flags in (4,5,6), 'invalid or WX load permissions'
        assert fs <= ms and va+ms <= 2**64, 'invalid load extent'
        assert align <= 1 or (align & (align-1) == 0 and off%align == va%align), 'bad alignment'
        span(off,fs)
        assert all(va+ms <= a or va >= b for a,b in ranges), 'overlapping loads'
        if ms: ranges.append((va,va+ms))
        executable |= bool(flags&1 and va <= entry < va+fs)
        segments.append(dict(address=hex(va),offset=off,file_bytes=fs,memory_bytes=ms,flags=flags,alignment=align))
    assert executable and {s['flags'] for s in segments} == {4,5,6}, 'missing separated RX/R/RW loads'
    sh,ss,sn,si=u(40,'Q'),u(58,'H'),u(60,'H'),u(62,'H')
    assert ss == 64 and 0 < sn < 65536 and si < sn, 'invalid sections'
    span(sh,ss*sn)
    names=sh+si*ss; strings=span(u(names+24,'Q'),u(names+32,'Q'))
    sections=[];symbols=0
    for at in range(sh,sh+ss*sn,ss):
        ni,kind,flags,off,size=u(at,'I'),u(at+4,'I'),u(at+8,'Q'),u(at+24,'Q'),u(at+32,'Q')
        assert ni < len(strings), 'invalid section name'
        name=strings[ni:].split(b'\0',1)[0].decode()
        if kind != 8: span(off,size)
        assert not flags&0x400, 'TLS section'
        assert flags&7 != 7, 'WX section'
        assert not (kind in (4,9) and size), 'unresolved relocations'
        assert kind not in (6,11), 'dynamic section'
        assert not (size and name == '.gcc_except_table'), 'exception landing-pad table'
        if kind==2:
            symbols+=1
            assert u(at+56,'Q')==24 and size%24==0, 'invalid symbols'
            for symbol in range(off,off+size,24):
                assert not (u(symbol,'I') and u(symbol+6,'H')==0), 'unresolved named symbol'
                assert u(symbol+4,'B')&15 != 6, 'TLS symbol'
        if flags&2: sections.append(dict(name=name,bytes=size,flags=hex(flags)))
    assert symbols, 'missing static symbol table'
    return dict(format='ELF64-LE',machine='x86_64',entry=hex(entry),segments=segments,allocated_sections=sections,unresolved_symbols=0,remaining_relocations=0,dynamic=False,tls=False,unwinding=False,executed=False)

def mutation_checks(data):
    ph=struct.unpack_from('<Q',data,32)[0]
    sh=struct.unpack_from('<Q',data,40)[0]
    sn=struct.unpack_from('<H',data,60)[0]
    sym=next(sh+i*64 for i in range(sn) if struct.unpack_from('<I',data,sh+i*64+4)[0]==2)
    symoff=struct.unpack_from('<Q',data,sym+24)[0]
    cases=[('wrong-machine',18,'H',3),('wrong-entry',24,'Q',0x500000),('wx',ph+4,'I',7),('dynamic',ph,'I',2),('interpreter',ph,'I',3),('tls-segment',ph,'I',7),('tls-section',sh+64+8,'Q',0x402),('relocations',sym+4,'I',4),('unresolved-symbol',symoff+24+6,'H',0)]
    result=[]
    for name,at,fmt,value in cases:
        b=bytearray(data);struct.pack_into('<'+fmt,b,at,value)
        try: inspect(b)
        except (AssertionError,ValueError,struct.error) as e: result.append(dict(case=name,rejected=True,reason=str(e)))
        else: raise AssertionError('audit accepted mutation '+name)
    for length in (0,16,63,len(data)//2):
        try: inspect(data[:length])
        except (AssertionError,ValueError,struct.error): result.append(dict(case='truncated-'+str(length),rejected=True))
        else: raise AssertionError('accepted truncation')
    return result

def main():
    p=argparse.ArgumentParser();p.add_argument('elf',type=Path);p.add_argument('output',type=Path);p.add_argument('--llvm',type=Path,required=True)
    a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True);data=a.elf.read_bytes();record=inspect(data)
    def run(tool,*args):
        r=subprocess.run([str(a.llvm/(tool+'.exe')),*args,str(a.elf)],capture_output=True,text=True,encoding='utf-8')
        assert r.returncode==0,r.stderr
        (a.output/(tool+'.txt')).write_text(r.stdout,encoding='utf-8')
        return r.stdout
    readobj=run('llvm-readobj','--file-headers','--program-headers','--sections','--relocations','--symbols','--unwind')
    # Rust's precompiled alloc shims retain passive CFI even under panic=abort.
    # Keep it visible; reject personality/LSDA augmentation instead of discarding it.
    augmentations=re.findall(r'augmentation: (\S+)',readobj)
    assert all(x in ('zR','R') for x in augmentations), 'personality/LSDA unwind metadata'
    undefined=run('llvm-nm','--undefined-only');assert not undefined.strip(),'LLVM unresolved symbols'
    symbols=run('llvm-nm','--defined-only','--demangle')
    assert not re.search(r'(std::|__imp_|pthread|_Unwind_|rust_eh_personality|__tls_get_addr|__cxa_)',symbols),'unexpected OS/unwind symbol'
    disassembly=run('llvm-objdump','--disassemble','--no-show-raw-insn','--demangle')
    instructions=[line for line in disassembly.splitlines() if re.match(r'^\s*[0-9a-f]+:\s',line)]
    fp=[line for line in instructions if re.search(r'\b([xyz]mm\d+|st\([0-7]\))\b',line) or re.search(r':\s+(f[a-z0-9]+|emms|vzeroupper|vzeroall|ldmxcsr|stmxcsr|xsave[a-z]*|xrstor[a-z]*)\b',line)]
    opcodes=Counter(re.search(r':\s+(\S+)',line)[1] for line in fp)
    probes={name: [line.strip() for line in symbols.splitlines() if name in line.replace('<','').replace('>','')][:5] for name in ('ui_probe_entry','ui_link::ARENA','__rust_alloc','custom_ui::scene','custom_ui::retained::Frame::draw','text_core::Engine::layout_cached','text_core::Engine','fontdue','rustybuzz','memcpy','memmove','memset','memcmp','slint_ui::paint','CandidateWindow','i_slint_renderer_software','libm','__addsf3','__mulsf3','__divsf3')}
    for name in ('ui_probe_entry','ui_link::ARENA','__rust_alloc','custom_ui::scene','text_core::Engine','fontdue','rustybuzz'):
        assert probes[name], 'real API not retained: '+name
    record.update(sha256=hashlib.sha256(data).hexdigest(),file_bytes=len(data),llvm_undefined_empty=True,unexpected_os_symbols=False,passive_cfi_sections=[s for s in record['allocated_sections'] if s['name'] in ('.eh_frame','.eh_frame_hdr')],cfi_augmentations=augmentations,runtime_unwinder_linked=False,fp_simd_instruction_count=len(fp),fp_simd_opcodes=dict(opcodes),fp_simd_excerpt=fp[:12],symbol_excerpts=probes,negative_checks=mutation_checks(data))
    (a.output/'audit.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'status':'passed','elf_sha256':record['sha256'],'fp_simd_instructions':len(fp),'negative_checks':len(record['negative_checks'])}))

if __name__=='__main__': main()
