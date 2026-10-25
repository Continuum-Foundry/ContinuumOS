"""Independently audit P3-02 ELF, raw tables, lifecycle and compact evidence."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import shutil
import struct

ROOT = Path(__file__).resolve().parents[1]
MASK = 0x000FFFFFFFFFF000
NX = 1 << 63
STACK = 0x7FFE0000


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def read(path):
    return json.loads(Path(path).read_text(encoding="utf-8-sig"))


def local(value):
    return Path(value.replace("${WORKSPACE}", str(ROOT)))


def safe(value):
    if isinstance(value, dict):
        return {key: safe(item) for key, item in value.items()}
    if isinstance(value, list):
        return [safe(item) for item in value]
    if isinstance(value, str):
        for source, target in ((str(ROOT), "${WORKSPACE}"), (ROOT.as_posix(), "${WORKSPACE}"),
                               (os.environ.get("USERPROFILE", "__NO_PROFILE__"), "${USERPROFILE}")):
            value = value.replace(source, target)
    return value


def save(path, value):
    Path(path).write_text(json.dumps(safe(value), indent=2) + "\n", encoding="utf-8")


def records(raw, build, session, failure=False):
    require(raw.endswith(b"\n") and len(raw) <= 256 * 1024, "incomplete/oversize serial")
    result = []
    for line in raw.splitlines():
        if not line.startswith(b"m1."):
            continue
        require(len(line) <= 512, "record too long")
        words = line.decode("ascii").split()
        pairs = [word.split("=", 1) for word in words[1:]]
        require(all(len(p) == 2 for p in pairs), "invalid record field")
        fields = dict(pairs)
        require(len(fields) == len(pairs), "duplicate field")
        require(fields["version"] == "5" and fields["build"] == build and fields["session"] == session
                and fields["session_origin"] == "host" and int(fields["seq"]) == len(result), "record identity/order")
        require(failure or fields["outcome"] == "ok" and fields["emergency"] == "false", "unexpected fatal/rejection")
        result.append((words[0], fields))
    require(result and result[0][0] == "m1.kernel_entry" and (failure or result[-1][0] == "m1.ready"), "missing entry/completion")
    return result


def table_audit(rows):
    by_event = {}
    for event, fields in rows:
        by_event.setdefault(event, []).append(fields)
    configs = by_event.get("m1.runtime_memory", [])
    require(len(configs) == 1, "missing/duplicate memory configuration")
    config = configs[0]
    expected = {"dynamic_ceiling": "16777216", "user_ceiling": "67108864", "spaces": "16",
                "allocation_slots": "256", "mappings_per_space": "128", "tables_per_bank": "64", "banks": "2", "initialized": "true"}
    require(all(config.get(k) == v for k, v in expected.items()), "wrong runtime configuration")
    storage = sum(int(config[k]) for k in ("metadata_bytes", "table_storage_bytes", "kernel_stack_storage"))
    require(storage <= 16777216 and int(config["kernel_guard_bytes"]) == 32 * 4096, "unbounded/unaccounted fixed storage")
    spaces = by_event.get("m1.runtime_space", [])
    require(len(spaces) == 2 and {int(s["space"]) for s in spaces} == {0, 1}, "missing private space snapshots")
    roots = [int(s["root"], 0) for s in spaces]
    require(roots[0] != roots[1] and all(p % 4096 == 0 for p in roots), "shared/unaligned roots")
    tables = {}
    leaves_by_space = {}
    walked_tables = {}
    for space in spaces:
        sid, root, used = int(space["space"]), int(space["root"], 0), int(space["used"])
        require(1 <= used <= 64, "table capacity")
        entries = {}
        for e in by_event.get("m1.runtime_entry", []):
            if int(e["space"]) != sid:
                continue
            key = (int(e["table"]), int(e["index"]))
            require(key not in entries and 0 <= key[0] < used and 0 <= key[1] < 512, "duplicate/out-of-range table entry")
            entries[key] = int(e["entry"], 0)
        require(entries and all(e != 0 for e in entries.values()), "empty/noncanonical dump")
        seen = set()
        leaves = {}

        def walk(table, level, base):
            require(table not in seen and table < used, "cycle/foreign/aliased table")
            seen.add(table)
            for (owner, index), entry in entries.items():
                if owner != table:
                    continue
                if level == 0 and index >= 256:
                    require(entry & 1 and not entry & 4, "user-accessible kernel mapping")
                    continue  # Shared supervisor tree is exhaustively audited by M1.
                require(entry & 7 == (5 if level == 3 and not entry & 2 else 7), "user permission level")
                require(entry & ~(MASK | NX | 0x67) == 0, "unsupported/huge/cache/global bits")
                address = base | (index << [39, 30, 21, 12][level])
                if level == 3:
                    require(0x10000 <= address < 0x80000000 and not (entry & 2 and not entry & NX), "range/WX leaf")
                    leaves[address] = entry
                else:
                    require(not entry & NX and entry & 7 == 7, "non-executable/non-user parent")
                    offset = (entry & MASK) - root
                    require(offset >= 0 and offset % 4096 == 0, "foreign parent")
                    walk(offset // 4096, level + 1, address)

        walk(0, 0, 0)
        require(seen == set(range(used)), "unreachable table allocation")
        expected_pages = set(range(STACK, STACK + 65536, 4096))
        if sid == 0:
            expected_pages |= set(range(0x400000, 0x404000, 4096))
        require(set(leaves) == expected_pages, "missing/extra/private/guard user mapping")
        for v, e in leaves.items():
            if v >= STACK:
                require(e & (NX | 7) == NX | 7, "stack must be user RW/NX")
            else:
                require(e & (NX | 7) == 5, "image must be user RX")
        tables[sid] = entries
        leaves_by_space[sid] = leaves
        walked_tables[sid] = used
    physical_sets = [{e & MASK for e in leaves_by_space[s].values()} for s in [0, 1]]
    require(not physical_sets[0] & physical_sets[1], "private backing aliases another process")
    usable=[(int(r["base"],0),int(r["base"],0)+int(r["bytes"],0)) for r in by_event.get("m1.memory_region",[]) if r["kind"]=="0"]
    reserved=[(int(r["base"],0),int(r["base"],0)+int(r["bytes"],0)) for r in by_event.get("m1.reservation",[])]
    kernel=[(int(r["base"],0),int(r["base"],0)+int(r["bytes"],0)) for r in by_event.get("m1.reservation",[]) if r["reason"]=="Kernel"]
    require(len(kernel)==1,"missing owned kernel reservation")
    for p in physical_sets[0] | physical_sets[1]:
        require(0<p<1<<32 and any(start<=p and p+4096<=end for start,end in usable)
                and not any(p<end and start<p+4096 for start,end in reserved),"user backing violates M1 reservation")
    for space in spaces:
        root=int(space["root"],0)
        require(kernel[0][0]<=root and root+int(space["used"])*4096<=kernel[0][1],"table root outside owned image")
    aliases = by_event.get("m1.runtime_alias", [])
    require(len(aliases) == 4, "missing physical alias audit")
    alias_pages = set()
    for alias in aliases:
        p, entry = int(alias["physical"], 0), int(alias["entry"], 0)
        require(p not in alias_pages and entry & MASK == p and entry & (NX | 7) == NX | 1, "writable/executable supervisor alias")
        require(alias["bytes"] == "4096", "wrong alias extent")
        alias_pages.add(p)
    require(alias_pages == {leaves_by_space[0][v] & MASK for v in range(0x400000, 0x404000, 4096)}, "wrong alias backing")
    for w in by_event.get("m1.runtime_walk", []):
        sid, table, index = int(w["space"]), int(w["table"]), int(w["index"])
        require(int(w["entry"], 0) == tables[sid].get((table, index), 0), "walk disagrees with complete dump")
        require(index == (int(w["virtual"], 0) >> [39, 30, 21, 12][int(w["level"])]) & 511, "walk index mismatch")
    stacks = by_event.get("m1.runtime_stack", [])
    require(len(stacks) == 2 and {int(s["space"]) for s in stacks} == {0, 1}, "missing kernel stacks")
    stack_bases = []
    for s in stacks:
        base = int(s["base"], 0)
        require(base % 4096 == 0 and base >= 0xFFFF800000000000 and s["bytes"] == "65536" and s["pages"] == "16"
                and int(s["guard_before"], 0) == int(s["guard_after"], 0) == 0 and s["zeroed"] == "true"
                and int(s["leaf"], 0) & (NX | 7) == NX | 3 and int(s["root_kernel"], 0) & 7 == 3, "kernel stack guard/privilege")
        stack_bases.append(base)
    require(abs(stack_bases[0] - stack_bases[1]) >= 73728, "overlapping kernel stacks")
    settlement = by_event.get("m1.runtime_settlement", [])
    require(len(settlement) == 2 and {s["cycle"] for s in settlement} == {"0", "999"}, "missing settlement")
    for s in settlement:
        require(all(s[k] == "0" for k in ("allocated", "spaces", "allocations", "mappings", "requested", "committed", "resident", "page_tables", "pins"))
                and s["zero_reuse"] == "true", "inventory leak")
    final = by_event.get("m1.runtime_memory_ready", [])
    require(len(final) == 1 and final[0]["cycles"] == "1000" and final[0]["frames"] == "0" and final[0]["user_execution"] == "false"
            and all(final[0][k] == "true" for k in ("real_cr3", "active_protect", "pins", "shared_busy", "zero_reuse", "rollback", "settled")), "missing/invalid memory completion")
    return {"fixed_kernel_storage_bytes": storage, "active_table_pages": walked_tables,
            "private_user_pages": {s: len(leaves_by_space[s]) for s in [0, 1]}, "cycles": 1000, "resident_after": 0}


def elf_audit(path):
    raw = path.read_bytes()
    require(raw[:7] == b"\x7fELF\x02\x01\x01", "ELF identity")
    kind, machine = struct.unpack_from("<HH", raw, 16)
    entry, phoff = struct.unpack_from("<QQ", raw, 24)
    phsize, count = struct.unpack_from("<HH", raw, 54)
    require((kind, machine, phsize) == (2, 62, 56) and count <= 16, "ELF architecture/header")
    loads = []
    for n in range(count):
        kind, flags, offset, virtual, physical, filesz, memsz, align = struct.unpack_from("<IIQQQQQQ", raw, phoff + n * phsize)
        require(kind not in (2, 3), "dynamic ELF")
        if kind != 1:
            continue
        require(flags in (4, 5, 6) and offset + filesz <= len(raw) and filesz <= memsz
                and align == 4096 and virtual >= 0xFFFFFFFF80000000 and virtual % 4096 == offset % 4096, "ELF segment bounds/permissions")
        loads.append({"address": virtual, "offset": offset, "file_size": filesz, "memory_size": memsz, "flags": flags})
    require(len(loads) == 3 and {s["flags"] for s in loads} == {4, 5, 6}, "ELF split W^X")
    for a, b in zip(loads, loads[1:]):
        require((a["address"] + a["memory_size"] + 4095) // 4096 <= b["address"] // 4096, "ELF alias overlap")
    require(any(s["flags"] == 5 and s["address"] <= entry < s["address"] + s["file_size"] for s in loads), "ELF entry RX")
    return {"entry": hex(entry), "loads": loads, "sha256": digest(path)}


def lifecycle(folder, expected_pass=True):
    result, launch, case = (read(folder / n) for n in ("run-result.json", "launch-manifest.json", "case-result.json"))
    require(result["cleanup_reaped"] and result["iso_unchanged_after_run"] and result["deadline_ms"] == 30000, "VM cleanup/deadline")
    require(launch["vm"]["vcpus"] == 1 and launch["vm"]["ram_mib"] == 2048 and launch["vm"]["deadline_ms"] == 30000, "VM capacity/profile")
    args = launch["arguments"]
    require("-no-reboot" in args and "-nic" in args and args[args.index("-nic") + 1] == "none"
            and sum(a == "-drive" for a in args) == 3, "VM offline/disk policy")
    require(not any(a in args for a in ("-kernel", "-snapshot", "-virtfs", "-fsdev", "-netdev", "-incoming", "-loadvm")), "forbidden VM transport")
    isos = [a for a in args if a.startswith("file=") and ",media=cdrom" in a]
    require(len(isos) == 1, "launched ISO missing")
    iso = local(isos[0][5:].split(",media=cdrom")[0])
    require(digest(iso) == result["iso_sha256"] == case["package"]["iso_sha256"], "changed launched ISO")
    if expected_pass:
        require(result["status"] == "passed" and not result["timed_out"], "failed guest")
    return result, launch, case


def mutation_checks(rows):
    tests = []
    for name, event, key, value in (
        ("writable-RX", "m1.runtime_entry", "entry", "0x3"),
        ("supervisor-alias-write", "m1.runtime_alias", "entry", hex(NX | 3)),
        ("guard-mapped", "m1.runtime_stack", "guard_before", "0x1003"),
        ("settlement-leak", "m1.runtime_settlement", "resident", "4096"),
        ("missing-cycles", "m1.runtime_memory_ready", "cycles", "999"),
        ("missing-table-pages", "m1.runtime_space", "used", "1"),
    ):
        mutated = copy.deepcopy(rows)
        if name == "writable-RX":
            target=next(f for e,f in mutated if e==event and int(f["entry"],0)&7==5 and not int(f["entry"],0)&NX)
            target[key]=hex(int(target[key],0)|2)
        elif name == "supervisor-alias-write":
            target=next(f for e,f in mutated if e==event)
            target[key]=hex(int(target[key],0)|2)
        else:
            next(f for e, f in mutated if e == event)[key] = value
        try:
            table_audit(mutated)
        except (ValueError, KeyError):
            tests.append({"case": name, "status": "rejected"})
        else:
            raise ValueError("reviewer accepted mutation: " + name)
    for event in ("m1.runtime_memory_ready", "m1.runtime_alias", "m1.runtime_space", "m1.runtime_entry", "m1.runtime_settlement"):
        try:
            table_audit([(e, f) for e, f in rows if e != event])
        except (ValueError, KeyError):
            tests.append({"case": "missing-" + event, "status": "rejected"})
        else:
            raise ValueError("reviewer accepted missing " + event)
    return tests


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--acceptance", type=Path, required=True)
    parser.add_argument("--destination", type=Path)
    args = parser.parse_args()
    acceptance = read(args.acceptance)
    require(acceptance["status"] == "passed" and len(acceptance["commands"]) == 4
            and all(c["exit_code"] == 0 for c in acceptance["commands"]), "incomplete acceptance")
    pre, baseline, memory = (local(acceptance["runs"][key]) for key in ("prerequisites", "m1", "memory"))
    prerequisites = read(pre / "prerequisites.json")
    require(prerequisites["status"] == "passed" and len(prerequisites["commands"]) == 10
            and all(c["exit_code"] == 0 for c in prerequisites["commands"]) and prerequisites["offline"], "prerequisites failed")
    source_inputs = read(baseline / "suite-source-inputs.json")
    pre_inputs={item["path"]:item["sha256"] for item in read(pre / "source-inputs.json")}
    require(all(pre_inputs.get(path)==sha for path,sha in source_inputs.items()), "stale prerequisite sources")
    require(read(memory / "build-inputs.json")["sources"] == source_inputs, "guest/M1 source mismatch")
    for path, sha in source_inputs.items():
        require(digest(ROOT / path) == sha, "changed tested source: " + path)
    m1 = read(baseline / "review.json")
    require(m1["review_tool_sha256"]==digest(ROOT / "tools/Review-P1-07.ps1"),"stale M1 reviewer")
    require(m1["status"] == "passed" and len(m1["launches"]) == 42 and len(read(baseline / "result.json")["stages"]["rows"]) == 33, "incomplete M1")
    manifest = read(memory / "build-manifest.json")
    elf = elf_audit(memory / "kernel.elf")
    require(elf["sha256"] == manifest["kernel_sha256"] and manifest["flavor"] == "runtime-memory-test", "wrong memory ELF")
    folder = memory / "owned-memory-run"
    result, launch, case = lifecycle(folder)
    require(result["emulator_test_exit"] == 33 and result["build_id"] == manifest["build_id"]
            and launch["fixture"] == "owned-memory", "wrong memory launch/exit")
    rows = records((folder / "serial.log").read_bytes(), manifest["build_id"], result["host_session_id"])
    audit = table_audit(rows)
    mutations = mutation_checks(rows)
    host_tests=(baseline / "kernel-host/check.stdout.log").read_text(encoding="utf-8-sig")
    require("22 passed; 0 failed" in host_tests and "run_failures_each_zero_stage_and_poisoned_physical_reuse ... ok" in host_tests
            and "every_parent_table_stage_and_mapping_quota_rolls_back ... ok" in host_tests, "missing host failure tests")
    for item in m1["launches"]:
        run = local(item["directory"])
        base_result, _, _ = lifecycle(run, item["outcome"] != "broken-before-entry")
        if item["outcome"] != "broken-before-entry":
            base_rows=records((run / "serial.log").read_bytes(),base_result["build_id"],base_result["host_session_id"],item["outcome"]!="ready")
            require(len(base_rows)==item["records"], "changed baseline serial record count")
            if item["outcome"]=="ready":
                require(sum(e=="m1.runtime_memory" for e,_ in base_rows)==1,"missing runtime initialization in M1 ready path")
    report = {"schema_version": 1, "story": "P3-02", "status": "passed", "review": "local Codex self-review; not independent approval",
              "reviewer_sha256": digest(__file__), "acceptance_sha256": digest(args.acceptance), "memory_build_id": manifest["build_id"],
              "memory_kernel_sha256": elf["sha256"], "memory_iso_sha256": result["iso_sha256"], "memory_serial_sha256": digest(folder / "serial.log"),
              "m1_rows": 33, "m1_launches": 42, "prerequisite_rows": 10, "kernel_host_tests":22, "host_tests_sha256":digest(baseline / "kernel-host/check.stdout.log"), "table_audit": audit, "reviewer_mutations": mutations,
              "criteria": {"owned_tables_and_guards": "passed", "allocation_reclamation": "passed", "failure_rollback": "passed",
                           "copy_and_shared_lifetimes": "passed", "M1_regression": "passed"},
              "user_execution": False, "public_memory_API": False, "desktop_targets_accepted": False}
    save(args.acceptance.parent / "review.json", report)
    if args.destination:
        dest = args.destination.resolve()
        require(dest.is_relative_to(ROOT / "evidence/phase-3/P3-02") and not dest.exists(), "invalid/existing evidence destination")
        dest.mkdir(parents=True)
        for name, value in (("review.json", report), ("acceptance.json", acceptance), ("prerequisites.json", prerequisites),
                            ("m1-review.json", m1), ("m1-result.json", read(baseline / "result.json")),
                            ("baseline-source-inputs.json", source_inputs), ("memory-build-manifest.json", manifest),
                            ("memory-build-inputs.json", read(memory / "build-inputs.json")), ("elf-audit.json", elf)):
            save(dest / name, value)
        shutil.copyfile(baseline / "kernel-host/check.stdout.log",dest / "host-tests.txt")
        provenance=dict(source_inputs)
        for path in ("tools/Test-P3-02.ps1","tools/Review-P3-02.py","config/m3-profiles.json","docs/06-contracts/runtime-v0.md",
                     "docs/05-delivery/phase-3/owned-memory-runbook.md","docs/04-decisions/0018-bounded-owned-memory.md","audit/check-docs.cjs"):
            provenance[path]=digest(ROOT / path)
        save(dest / "source-provenance.json",{"source_commit":acceptance["source_commit"],"dirty_input_hashes":provenance})
        launches = [(local(item["directory"]), f"{n:02d}") for n, item in enumerate(m1["launches"])] + [(folder, "memory")]
        (dest / "launches").mkdir()
        for run, label in launches:
            shutil.copyfile(run / "serial.log", dest / "launches" / f"{label}-serial.txt")
            save(dest / "launches" / f"{label}-context.json", {"result": read(run / "run-result.json"), "launch": read(run / "launch-manifest.json"),
                 "package": read(run / "case-result.json")["package"], "serial_sha256": digest(run / "serial.log"),
                 "host_logs": {name: {"sha256": digest(run / name), "text": (run / name).read_text(encoding="utf-8")} for name in ("qemu.stdout.log", "qemu.stderr.log")}})
        files = [p for p in dest.rglob("*") if p.is_file()]
        require(len(files) < 190 and sum(p.stat().st_size for p in files) < 4 * 1024 * 1024
                and all(p.stat().st_size <= 256 * 1024 for p in files), "evidence budget")
        print(dest)
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
