"""Independent ELF, loaded-byte/table and trusted-launch evidence review."""
import argparse
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import shutil
import struct

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("memory_review", ROOT / "tools/Review-P3-02.py")
memory = importlib.util.module_from_spec(spec)
spec.loader.exec_module(memory)
require, read, save, digest, local = memory.require, memory.read, memory.save, memory.digest, memory.local
PAGE, NX, MASK = 4096, 1 << 63, 0x000FFFFFFFFFF000
STARTUP, RETURN, STACK, STACK_BYTES = 0x7FFD0000, 0x7FFD1000, 0x7FFE0000, 65536


def expected_pages(path):
    raw = path.read_bytes()
    require(raw[:7] == b"\x7fELF\2\1\1" and raw[7] in (0,3) and raw[8:16] == bytes(8), "user ELF identity")
    kind, machine, version, entry, ph, sh, flags, eh, ps, pn, ss, sn, si = struct.unpack_from("<HHIQQQIHHHHHH", raw, 16)
    require((kind, machine, version, flags, eh, ps, ss) == (2, 62, 1, 0, 64, 56, 64), "user ELF headers")
    require(entry == 0x400000 and 0 < pn <= 16 and 0 < sn <= 128 and si < sn
            and ph + pn * ps <= len(raw) and sh + sn * ss <= len(raw), "user ELF tables")
    pages, permissions, loads = {}, {}, []
    for n in range(pn):
        k, f, off, va, pa, fs, ms, align = struct.unpack_from("<IIQQQQQQ", raw, ph + n * ps)
        require(k == 1, "fixture has unexpected runtime segment")
        require(f in (4, 5, 6) and fs <= ms and off + fs <= len(raw) and ms > 0
                and align == 4096 and va == pa and va % align == off % align, "user LOAD")
        base, end = va // PAGE * PAGE, (va + ms + PAGE - 1) // PAGE * PAGE
        require(0x10000 <= base < end <= STARTUP, "user LOAD bounds")
        content = bytearray(end - base)
        content[va-base:va-base+fs] = raw[off:off+fs]
        for address in range(base, end, PAGE):
            require(address not in pages, "rounded LOAD overlap")
            pages[address] = bytes(content[address-base:address-base+PAGE])
            permissions[address] = f
        loads.append(dict(address=va, offset=off, file_bytes=fs, memory_bytes=ms, flags=f))
    require(len(loads) == 3 and [s['flags'] for s in loads] == [5, 4, 6]
            and loads[-1]['memory_bytes'] - loads[-1]['file_bytes'] >= 8193, "fixture split/BSS")
    require(any(s['flags'] == 5 and s['address'] <= entry < s['address'] + s['file_bytes'] for s in loads), "entry file-backed RX")
    for n in range(sn):
        _, k, f, _, off, size, _, _, _, entsize = struct.unpack_from("<IIQQQQIIQQ", raw, sh+n*ss)
        require(k not in (4, 6, 9, 11, 14, 15, 16, 19) and f & 0x400 == 0, "user runtime section obligation")
        require(k == 8 or off+size <= len(raw), "user section outside file")
        if k == 2:
            require(entsize == 24 and size % 24 == 0, "symbol table size")
            for p in range(off, off+size, 24):
                name, info, _, index, _, _ = struct.unpack_from("<IBBHQQ", raw, p)
                require(not (name and info >> 4 and index == 0), "undefined nonlocal user symbol")
    startup = bytearray(PAGE)
    startup[:8] = b"CNTM3RT0"
    struct.pack_into("<II", startup, 8, 0, 64)
    struct.pack_into("<I", startup, 32, PAGE)
    pages[STARTUP], permissions[STARTUP] = bytes(startup), 4
    pages[RETURN], permissions[RETURN] = b"\x0f\x0b" + bytes(PAGE-2), 5
    for address in range(STACK, STACK+STACK_BYTES, PAGE):
        content = bytearray(PAGE)
        if address == STACK+STACK_BYTES-PAGE:
            struct.pack_into("<Q", content, PAGE-8, RETURN)
        pages[address], permissions[address] = bytes(content), 6
    return pages, permissions, dict(sha256=digest(path), bytes=len(raw), entry=entry, loads=loads, commitment=len(pages)*PAGE)


def audit(rows, images):
    events = {}
    for event, fields in rows:
        events.setdefault(event, []).append(fields)
    image_rows = events.get('m1.loader_image', [])
    require(len(image_rows) == 3 and {int(f['image']) for f in image_rows} == {0, 1, 2}, "three loaded images required")
    physical, roots, audit_images = set(), set(), []
    for i, (pages, permissions, elf) in enumerate(images):
        item = next(f for f in image_rows if int(f['image']) == i)
        root = int(item['root'], 0)
        require(root % PAGE == 0 and root not in roots, "private roots")
        roots.add(root)
        require(item['digest'] == elf['sha256'] and int(item['bytes']) == elf['bytes']
                and int(item['entry'], 0) == elf['entry'] and int(item['commitment']) == elf['commitment']
                and int(item['rsp'], 0) == STACK+STACK_BYTES-8 and int(item['startup'], 0) == STARTUP
                and item['state'] == 'staged', "loaded image metadata/startup")
        tables = {}
        for f in events.get('m1.loader_table', []):
            if int(f['image']) == i:
                key = int(f['table']), int(f['index'])
                require(key not in tables and key[0] < 64 and key[1] < 512, "table duplicate/bounds")
                tables[key] = int(f['raw'], 0)
        leaves, visited, used = {}, set(), set()
        def walk(table, level, prefix):
            require(table not in visited and 0 <= table < 64, "table cycle/alias/range")
            visited.add(table)
            entries = [(n, word) for (t, n), word in tables.items() if t == table]
            require(entries, "missing raw table")
            for index, word in entries:
                used.add((table, index))
                address = prefix | index << (12 + (level-1)*9)
                require(word & 0x85 == 5 and (level != 4 or index < 256), "user U/S/present/huge")
                if level > 1:
                    require(word & 0xFFF == 7 and word & NX == 0, "ancestor permissions")
                    child = ((word & MASK) - root) // PAGE
                    require((word & MASK) >= root and (word & MASK) % PAGE == 0, "child backing")
                    walk(child, level-1, address)
                else:
                    require(address not in leaves, "leaf duplicate")
                    leaves[address] = word
        walk(0, 4, 0)
        require(set(tables) == used and set(leaves) == set(pages), "unexpected/missing mapped page or unreachable table")
        observed = [f for f in events.get('m1.loader_page', []) if int(f['image']) == i]
        require(len(observed) == len(pages) and len({int(f['address'], 0) for f in observed}) == len(pages), "page records")
        for f in observed:
            address, leaf, alias = (int(f[k], 0) for k in ('address', 'leaf', 'alias'))
            require(address in pages and (leaf & ~0x60) == (leaves[address] & ~0x60), "page record/table disagreement")
            flags = permissions[address]
            require(leaf & 7 == (7 if flags == 6 else 5) and bool(leaf & NX) == (flags != 5), "RX/R/RW/NX permission")
            require(alias & MASK == leaf & MASK and alias & (NX|7) == NX|1, "supervisor writable or user alias")
            backing = leaf & MASK
            require(backing not in physical, "shared physical backing")
            require(any(int(f['kind']) == 0 and int(f['base'],0) <= backing and backing+PAGE <= int(f['base'],0)+int(f['bytes'],0)
                        for f in events.get('m1.memory_region',[])), 'backing outside ordinary RAM')
            require(not any(backing < int(f['base'],0)+int(f['bytes'],0) and int(f['base'],0) < backing+PAGE
                            for f in events.get('m1.reservation',[])), 'reserved physical backing')
            physical.add(backing)
            require(f['sha256'] == hashlib.sha256(pages[address]).hexdigest(), "loaded content/BSS/startup/stack mismatch")
        audit_images.append(dict(image=i, **elf, pages=len(pages), table_pages=len(visited)))
    binding = events.get('m1.loader_binding', [])
    require(len(binding) == 4, "binding count")
    bindings = {int(f['image']): f for f in binding}
    require(set(bindings) == {0, 1, 2, 3}, "binding identity")
    for i, principal, session, service, grants in [(0,101,1001,'false',1),(1,202,2002,'false',1),(2,303,3003,'true',3),(3,202,2002,'false',1)]:
        f = bindings[i]
        require((int(f['principal']), int(f['authority_session']), f['service'], int(f['grants']), int(f['ceiling']))
                == (principal, session, service, grants, 67108864), "trusted identity/ceiling")
        require(int(f['generation']) > 0 and f['source'] == ('reuse' if i == 3 else 'bootstrap'), "binding source/generation")
    require(len({bindings[i]['slot'] for i in (0,1,2)}) == 3 and bindings[3]['slot'] == bindings[0]['slot']
            and int(bindings[3]['generation']) > int(bindings[0]['generation']) and bindings[3]['stale_denied'] == 'true', "stale/reused binding")
    final = events.get('m1.loader_ready', [])
    require(len(final) == 1, "missing loader completion")
    f = final[0]
    require(f['images'] == '3' and f['malformed'] == '19' and f['authority_edits'] == '15' and f['frames'] == '0'
            and f['user_execution'] == 'false' and all(f[k] == 'true' for k in ('digest_rejected','stale_denied','settled')), "loader completion/settlement")
    rejected = events.get('m1.loader_rejected', [])
    allocations = [f for f in rejected if f['case'] == 'allocation']
    malformed = [f for f in rejected if f['case'] == 'malformed']
    count = images[0][2]['commitment']//PAGE
    require(int(final[0]['allocation_failures']) == count and len(allocations) == count
            and {int(f['quota']) for f in allocations} == set(range(count)), "allocation failure coverage")
    require(len(malformed) == 18 and {int(f['mutation']) for f in malformed} == set(range(18)), "malformed coverage")
    require(all(f['settled'] == 'true' and f['field'] == 'frames' and f['reason'] == 'exhausted' for f in allocations), "allocation rejection/cleanup")
    require(all(f['settled'] == 'true' and f['field'] == 'elf' for f in malformed), "malformed rejection/cleanup")
    return dict(images=audit_images, private_pages=len(physical), allocation_failures=count, malformed_cases=19, trusted_bindings=4, settled_frames=0)


def mutation_checks(rows, images):
    tests = []
    for name, event, key, value in [('wrong-page','m1.loader_page','sha256','0'*64),('writable-alias','m1.loader_page','alias',hex(NX|3)),('forged-principal','m1.loader_binding','principal','999'),('stale-reuse','m1.loader_binding','generation','0'),('false-settlement','m1.loader_ready','frames','1')]:
        changed = copy.deepcopy(rows)
        next(f for e,f in changed if e == event)[key] = value
        try: audit(changed, images)
        except (ValueError, KeyError): tests.append(dict(case=name, status='rejected'))
        else: raise ValueError('review accepted ' + name)
    for event in ('m1.loader_image','m1.loader_table','m1.loader_page','m1.loader_binding','m1.loader_rejected','m1.loader_ready'):
        try: audit([(e,f) for e,f in rows if e != event], images)
        except (ValueError, KeyError): tests.append(dict(case='missing-'+event, status='rejected'))
        else: raise ValueError('review accepted missing ' + event)
    for name in ('writable-RX-table','forged-startup','stale-reuse-generation'):
        changed = copy.deepcopy(rows)
        if name == 'writable-RX-table':
            f = next(f for e,f in changed if e == 'm1.loader_table' and f['image'] == '0' and int(f['raw'],0)&7 == 5 and not int(f['raw'],0)&NX)
            f['raw'] = hex(int(f['raw'],0)|2)
            f = next(f for e,f in changed if e == 'm1.loader_page' and f['image'] == '0' and int(f['address'],0) == 0x400000)
            f['leaf'] = hex(int(f['leaf'],0)|2)
        elif name == 'forged-startup':
            next(f for e,f in changed if e == 'm1.loader_page' and int(f['address'],0) == STARTUP)['sha256'] = 'f'*64
        else:
            generation = next(f['generation'] for e,f in changed if e == 'm1.loader_binding' and f['image'] == '0')
            next(f for e,f in changed if e == 'm1.loader_binding' and f['image'] == '3')['generation'] = generation
        try: audit(changed, images)
        except (ValueError, KeyError): tests.append(dict(case=name, status='rejected'))
        else: raise ValueError('review accepted ' + name)
    return tests


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--acceptance', type=Path, required=True)
    parser.add_argument('--destination', type=Path)
    args = parser.parse_args()
    acceptance = read(args.acceptance)
    require(acceptance['story'] == 'P3-03' and acceptance['status'] == 'passed' and len(acceptance['commands']) == 5
            and all(c['exit_code'] == 0 for c in acceptance['commands']), 'incomplete acceptance')
    pre, baseline, owned, loader = (local(acceptance['runs'][key]) for key in ('prerequisites','m1','memory','loader'))
    prerequisites = read(pre/'prerequisites.json')
    require(prerequisites['status'] == 'passed' and prerequisites['offline'] and len(prerequisites['commands']) == 10
            and all(c['exit_code'] == 0 for c in prerequisites['commands']), 'prerequisites')
    sources = read(baseline/'suite-source-inputs.json')
    pre_sources = {item['path']:item['sha256'] for item in read(pre/'source-inputs.json')}
    require(all(pre_sources.get(p) == sha == digest(ROOT/p) for p,sha in sources.items()), 'stale prerequisites/final sources')
    require(all(read(run/'build-inputs.json')['sources'] == sources for run in (owned,loader)), 'different guest/baseline sources')
    m1 = read(baseline/'review.json')
    require(m1['status'] == 'passed' and len(m1['launches']) == 42 and len(read(baseline/'result.json')['stages']['rows']) == 33
            and m1['review_tool_sha256'] == digest(ROOT/'tools/Review-P1-07.ps1'), 'M1 acceptance/review')
    for item in m1['launches']:
        memory.lifecycle(local(item['directory']), item['outcome'] != 'broken-before-entry')
    mf = read(loader/'build-manifest.json')
    require(mf['flavor'] == 'static-loader-test' and digest(loader/'kernel.elf') == mf['kernel_sha256'], 'loader kernel')
    memory.elf_audit(loader/'kernel.elf')
    run = loader/'static-loader-run'
    result, launch, case = memory.lifecycle(run)
    require(result['emulator_test_exit'] == 33 and result['build_id'] == mf['build_id'] and launch['fixture'] == 'static-loader', 'loader exit/identity')
    rows = memory.records((run/'serial.log').read_bytes(), mf['build_id'], result['host_session_id'])
    names = ['user_a','user_b','service']
    images = [expected_pages(loader/f'{n}.elf') for n in names]
    descriptors = read(loader/'user-images.json')
    require(descriptors['trusted_descriptors_sha256'] == digest(loader/'trusted-images.rs'), 'changed trusted descriptor')
    extracted = loader/'static-loader-package/extracted payload'
    for n, image, descriptor in zip(names,images,descriptors['images']):
        require(image[2]['sha256'] == descriptor['sha256'] == digest(extracted/f'boot/{n}.elf')
                == case['package']['payload'][f'boot/{n}.elf'], 'image/ISO/descriptor digest')
        require(not (loader/f'{n}-llvm-nm.stdout.log').read_text().strip(), 'undefined user symbol')
        command = read(loader/f'{n}-build.command.json')['arguments']
        require(all(flag in command for flag in ('--target=x86_64-unknown-none','code-model=small','relocation-model=static','no-redzone=yes','panic=abort','opt-level=1',n)), 'user compiler flags')
    observed = audit(rows,images)
    mutations = mutation_checks(rows,images)
    owned_result, _, _ = memory.lifecycle(owned/'owned-memory-run')
    owned_mf = read(owned/'build-manifest.json')
    owned_rows = memory.records((owned/'owned-memory-run/serial.log').read_bytes(), owned_mf['build_id'], owned_result['host_session_id'])
    memory_audit = memory.table_audit(owned_rows)
    host = (baseline/'kernel-host/check.stdout.log').read_text(encoding='utf-8-sig')
    require('28 passed; 0 failed' in host and all(f'user_image::tests::{name} ... ok' in host for name in ('parser_mutation_matrix','transactional_allocation_and_table_failures','loaded_bytes_permissions_startup_and_authority_reuse','digest_and_parse_fail_without_context_or_frames','generation_wrap_retires_process_slot','sha_known_vectors')), 'loader host test coverage')
    report = dict(schema_version=1,story='P3-03',status='passed',review='local Codex self-review; not independent approval',reviewer_sha256=digest(__file__),acceptance_sha256=digest(args.acceptance),
                  source_commit=acceptance['source_commit'],loader_build_id=mf['build_id'],kernel_sha256=mf['kernel_sha256'],iso_sha256=result['iso_sha256'],serial_sha256=digest(run/'serial.log'),
                  audit=observed,memory_audit=memory_audit,reviewer_mutations=mutations,prerequisite_checks=10,m1_rows=33,m1_launches=42,kernel_host_tests=28,
                  user_execution=False,public_launch_API=False,desktop_targets_accepted=False)
    save(args.acceptance.parent/'review.json',report)
    if args.destination:
        dest = args.destination.resolve()
        require(dest.is_relative_to(ROOT/'evidence/phase-3/P3-03') and not dest.exists(), 'invalid/existing destination')
        dest.mkdir(parents=True)
        for name,value in [('review.json',report),('acceptance.json',acceptance),('prerequisites.json',prerequisites),('baseline-source-inputs.json',sources),('m1-review.json',m1),('m1-result.json',read(baseline/'result.json')),('loader-build-inputs.json',read(loader/'build-inputs.json')),('loader-build-manifest.json',mf),('user-images.json',descriptors)]: save(dest/name,value)
        shutil.copyfile(baseline/'kernel-host/check.stdout.log',dest/'host-tests.txt')
        (dest/'user-codegen').mkdir()
        for n in names:
            save(dest/'user-codegen'/f'{n}-command.json',read(loader/f'{n}-build.command.json'))
            disassembly=(loader/f'{n}-llvm-objdump.stdout.log').read_text()
            entry='<fixture_entry>:'
            require(entry in disassembly,'missing user entry disassembly')
            excerpt=entry+disassembly.split(entry,1)[1].split('\n\n',1)[0]+'\n'
            (dest/'user-codegen'/f'{n}-entry.txt').write_text(excerpt,encoding='utf-8')
        provenance = dict(sources)
        for p in ('tools/Test-P3-03.ps1','tools/Review-P3-03.py','tools/Review-P3-02.py','contracts/runtime-v0.json','config/m3-profiles.json','docs/06-contracts/runtime-v0.md','docs/05-delivery/phase-3/static-loader-runbook.md','docs/04-decisions/0019-static-images-and-trusted-launch.md','audit/check-docs.cjs'): provenance[p]=digest(ROOT/p)
        save(dest/'source-provenance.json',dict(source_commit=acceptance['source_commit'],dirty_input_hashes=provenance))
        launches = [(local(i['directory']),f'{n:02d}') for n,i in enumerate(m1['launches'])]+[(owned/'owned-memory-run','memory'),(run,'loader')]
        (dest/'launches').mkdir()
        for folder,label in launches:
            shutil.copyfile(folder/'serial.log',dest/'launches'/f'{label}-serial.txt')
            save(dest/'launches'/f'{label}-context.json',dict(result=read(folder/'run-result.json'),launch=read(folder/'launch-manifest.json'),package=read(folder/'case-result.json')['package'],serial_sha256=digest(folder/'serial.log')))
        files = [p for p in dest.rglob('*') if p.is_file()]
        require(len(files)<190 and sum(p.stat().st_size for p in files)<4*1024*1024 and all(p.stat().st_size<=256*1024 for p in files), 'evidence budget')
        print(dest)
    print(json.dumps(report,indent=2))


if __name__ == '__main__': main()
