"""Accept P3-01 using explicit fresh prerequisites/M1 runs and real child faults."""
import argparse
import copy
import json
import os
from pathlib import Path
import re
import struct
import subprocess
import sys
import time

import m3_runtime as runtime


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prerequisites", type=Path, required=True)
    parser.add_argument("--m1", type=Path, required=True)
    args = parser.parse_args()
    root = runtime.ROOT
    run = root / "build/m3/P3-01" / f"acceptance-{time.time_ns()}"
    run.mkdir(parents=True)
    rows = []

    def check(name, condition):
        runtime.require(condition, name)
        rows.append({"name": name, "status": "passed"})

    def rejects(name, function):
        try:
            function()
        except (ValueError, OSError):
            rows.append({"name": name, "status": "passed"})
            return
        raise ValueError("unexpected acceptance: " + name)

    try:
        pre = json.loads((args.prerequisites / "prerequisites.json").read_text(encoding="utf-8-sig"))
        check("ten-current-prerequisites", pre["status"] == "passed" and len(pre["commands"]) == 10
              and all(c["exit_code"] == 0 for c in pre["commands"]) and pre["offline"]
              and pre["signing"]["status"] == "passed")
        check("source-base-current", (args.prerequisites / "source-base.txt").read_text().strip() ==
              subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip())
        # Recheck executable/build/kernel inputs, not only the summary's pass string.
        # Prose and the new host runner do not change the compiled baseline.
        for item in json.loads((args.prerequisites / "source-inputs.json").read_text(encoding="utf-8-sig")):
            path = item["path"]
            if path.startswith(("kernel/", "crates/", "experiments/", "tools/xtask/", "tools/windows/")) or path in (
                    "Cargo.lock", "Cargo.toml", "rust-toolchain.toml", ".cargo/config.toml",
                    "config/m1-inputs.lock.json", "tools/Recheck-P2Prerequisites.ps1"):
                runtime.require(runtime.digest(root / path) == item["sha256"], "stale baseline input: " + path)
        rows.append({"name": "current-compiled-source-inputs", "status": "passed"})
        m0_log = (args.prerequisites / "m0.log").read_text(encoding="utf-8-sig")
        match = re.search(r"Evidence: ([^\r\n]+)", m0_log)
        runtime.require(match is not None, "fresh M0 evidence path absent")
        m0_dir = Path(match[1].strip())
        image = m0_dir / "clean-a/userland-link.elf"
        m0 = runtime.read_json(m0_dir / "m0-results.json")
        sha = runtime.digest(image)
        check("fresh-real-user-link-image", m0["status"] == "passed" and
              m0["build"]["artifacts"][1]["sha256"] == sha and image.stat().st_mtime_ns >=
              (args.prerequisites / "source-base.txt").stat().st_mtime_ns)
        m1 = json.loads((args.m1 / "result.json").read_text(encoding="utf-8-sig"))
        review = json.loads((args.m1 / "review.json").read_text(encoding="utf-8-sig"))
        check("complete-M1-baseline-and-raw-review", m1["status"] == "passed" and
              len(m1["stages"]["rows"]) == 33 and review["status"] == "passed" and len(review["launches"]) == 42)
        baseline_sources = json.loads((args.m1 / "suite-source-inputs.json").read_text(encoding="utf-8-sig"))
        check("current-M1-build-and-harness-inputs", all(runtime.digest(root / path) == sha
              for path, sha in baseline_sources.items()))
        # Review-P1-07 independently validates every raw record and current source.
        check("baseline-after-prerequisite-start", (args.m1 / "result.json").stat().st_mtime_ns >=
              (args.prerequisites / "source-base.txt").stat().st_mtime_ns)

        p = runtime.read_json(runtime.PROFILE)
        runtime.validate_profile(p)
        c = runtime.read_json(runtime.CONTRACT)
        runtime.validate_contract(c)
        locked_vm = json.loads((root / "config/m1-inputs.lock.json").read_text())["vm"]
        cpu_at = locked_vm["arguments"].index("-cpu")
        check("profile-matches-actual-M1-CPU", p["vm"] == {
            key: locked_vm[key] for key in p["vm"] if key != "cpu"} |
            {"cpu": locked_vm["arguments"][cpu_at + 1]})
        # Independent standard-library encodings, not a second call to generator.
        check("independent-startup-layout", struct.calcsize("<8sIIQQIIQ16s") == 64)
        check("independent-message-layout", struct.calcsize("<IHHIIIHHQQQ") == 48)
        runtime.generate(run / "draft-a.rs")
        runtime.generate(run / "draft-b.rs")
        generated = (run / "draft-a.rs").read_text()
        check("deterministic-generated-SDK", runtime.digest(run / "draft-a.rs") == runtime.digest(run / "draft-b.rs"))
        check("independent-startup-offsets", all(
            f"STARTUP_{name}_OFFSET: usize = {offset};" in generated for name, offset in (
                ("MAGIC", 0), ("REVISION", 8), ("LENGTH", 12), ("FEATURES", 16),
                ("BOOTSTRAP_HANDLE", 24), ("PAGE_BYTES", 32), ("RESERVED", 36),
                ("CLOCK_RESOLUTION_NS", 40), ("RESERVED_TAIL", 48))))
        check("independent-message-offsets", all(
            f"MESSAGE_{name}_OFFSET: usize = {offset};" in generated for name, offset in (
                ("PROTOCOL", 0), ("MAJOR", 4), ("MINOR", 6), ("OPERATION", 8),
                ("REQUIRED_FEATURES", 12), ("PAYLOAD_BYTES", 16), ("HANDLE_COUNT", 20),
                ("RESERVED", 22), ("REQUEST_ID", 24), ("ROUTE_EPOCH", 32), ("POLICY_EPOCH", 40))))
        compiled = subprocess.run(["rustc", "--edition=2024", "--crate-type", "lib",
                                   "--crate-name", "m3_draft_sdk", "--target", c["target"],
                                   str(run / "draft-a.rs"), "-o", str(run / "draft.rlib")],
                                  cwd=root, capture_output=True, timeout=30)
        (run / "sdk-compile.stdout.log").write_bytes(compiled.stdout)
        (run / "sdk-compile.stderr.log").write_bytes(compiled.stderr)
        check("generated-SDK-target-compiles", compiled.returncode == 0)
        for name, path, value in (
                ("profile", ["profile"], "enhanced"),
                ("CPU", ["vm", "cpu"], "host"), ("SMP", ["vm", "vcpus"], 2),
                ("capacity", ["vm", "ram_mib"], 4096), ("limits", ["limits", "message_bytes"], 4097),
                ("deadline", ["runner", "fixture_timeout_ms"], 0),
                ("type", ["vm", "vcpus"], True), ("clock", ["measurement", "guest_clock_gate"], "ticks")):
            altered = copy.deepcopy(p)
            if len(path) == 1:
                altered[path[0]] = value
            else:
                altered[path[0]][path[1]] = value
            config = run / ("bad-" + name + ".json")
            runtime.save(config, altered)
            directory, report = runtime.run(config, image, sha, "success")
            check("reject-" + name, report["outcome"] == "validation-rejected" and
                  not report["launched"] and report["cleanup"] == "not-launched")
        for name, altered in (("unknown-field", dict(p, unknown=1)),
                              ("missing-field", {k: v for k, v in p.items() if k != "limits"})):
            rejects(name, lambda altered=altered: runtime.validate_profile(altered))
        rejects("duplicate-JSON-key", lambda: json.loads('{"a":1,"a":2}', object_pairs_hook=runtime.no_duplicates))
        malformed = run / "malformed-JSON.json"
        malformed.write_text('{"broken":', encoding="utf-8")
        oversized = run / "oversized-JSON.json"
        oversized.write_bytes(b" " * 65537)
        bad_image = run / "not-ELF.bin"
        bad_image.write_bytes(b"x" * 64)
        for name, config, img, expected, tool in (
                ("missing-config", run / "missing.json", image, sha, None),
                ("malformed-config", malformed, image, sha, None),
                ("oversized-config", oversized, image, sha, None),
                ("missing-image", runtime.PROFILE, run / "missing.elf", sha, None),
                ("changed-image", runtime.PROFILE, image, "0" * 64, None),
                ("malformed-image", runtime.PROFILE, bad_image, runtime.digest(bad_image), None),
                ("missing-tool", runtime.PROFILE, image, sha, run / "missing.exe"),
                ("alternate-tool", runtime.PROFILE, image, sha, bad_image)):
            directory, report = runtime.run(config, img, expected, "success", tool)
            check("reject-" + name, report["outcome"] == "validation-rejected" and
                  not report["launched"] and report["cleanup"] == "not-launched")
        _, invalid_case = runtime.run(runtime.PROFILE, image, sha, "unknown")
        check("reject-unknown-case", not invalid_case["launched"] and invalid_case["outcome"] == "validation-rejected")
        lifecycle = []
        for case, outcome, exit_code in (("success", "complete", 0), ("nonzero", "exited-nonzero", 7),
                                         ("missing-final", "missing-final-record", 0), ("timeout", "timeout", None)):
            directory, report = runtime.run(runtime.PROFILE, image, sha, case)
            check("actual-child-" + case, report["outcome"] == outcome and report["launched"] and
                  report["cleanup"] == "reaped" and (exit_code is None or report["exit_code"] == exit_code))
            lifecycle.append({"path": directory.relative_to(root).as_posix(),
                              "result_sha256": runtime.digest(directory / "result.json"), "result": report})
        for case, expected_code, outcome in (("success", 0, "complete"), ("nonzero", 1, "exited-nonzero"),
                                               ("missing-final", 1, "missing-final-record"), ("timeout", 1, "timeout")):
            command = [sys.executable, str(root / "tools/m3_runtime.py"), "run", "--image", str(image),
                       "--sha256", sha, "--case", case]
            cli = subprocess.run(command, cwd=root, capture_output=True, text=True, timeout=10,
                                 creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0)
            (run / ("cli-" + case + ".stdout.log")).write_text(cli.stdout, encoding="utf-8")
            (run / ("cli-" + case + ".stderr.log")).write_text(cli.stderr, encoding="utf-8")
            directory = Path(cli.stdout.strip())
            report = runtime.read_json(directory / "result.json")
            check("documented-CLI-" + case, cli.returncode == expected_code and report["outcome"] == outcome and
                  report["cleanup"] == "reaped")
            lifecycle.append({"path": directory.relative_to(root).as_posix(),
                              "result_sha256": runtime.digest(directory / "result.json"), "result": report})
        # Preserve the byte identity of every hook, including frozen Slint gaps.
        hooks = runtime.read_json(root / "experiments/ui-link/hooks.json")
        hook_map = {"allocation": "P3-02/P3-09", "entry and memory": "P3-03/P3-04",
                    "memory and math": "P3-05", "failure": "P3-04/P3-09", "threading": "P3-05/P3-09",
                    "clock/event loop": "P3-08/M4", "single-thread global state": "P3-05/P3-09/M4"}
        gaps = [{"candidate": group, "hook": item["hook"], "owner": hook_map.get(item["hook"], "M4-or-later"),
                 "kernel_implemented": False, "obligation": item["guest_obligation"]}
                for group in ("common", "custom", "slint_aot") for item in hooks[group]]
        runtime.save(run / "hook-gap-map.json", {"hooks_sha256": runtime.digest(root / "experiments/ui-link/hooks.json"), "gaps": gaps})
        historical = []
        for relative in ("evidence/phase-1/P1-07/2026-10-20-acceptance",
                         "evidence/phase-2/P2-13/2026-10-24",
                         "evidence/phase-2/performance-remediation/2026-10-24"):
            folder = root / relative
            historical.append({"path": relative, "review_sha256": runtime.digest(folder / "README.md"),
                               "local_raw_log_count": sum(1 for _ in folder.rglob("*.log")),
                               "verification": "review read; historical raw bytes not reverified"})
        selected_audit_path = root / "evidence/phase-2/performance-remediation/2026-10-24/selected-elf-audit.json"
        selected_audit = json.loads(selected_audit_path.read_text(encoding="utf-8-sig"))
        runtime.save(run / "runtime-input-inventory.json", {
            "historical": historical, "availability_notice_sha256": runtime.digest(root / "evidence/README.md"),
            "compiler": m0["build"]["rustc"], "target": c["target"],
            "user_probe": {"sha256": sha, "code_model": "small", "relocation_model": "static", "executed": False},
            "selected_ui_probe": {"code_model": "small", "relocation_model": "static", "executed": False,
                                  "historical_elf_sha256": selected_audit["sha256"],
                                  "historical_audit_sha256": runtime.digest(selected_audit_path),
                                  "scope": "existing M2 final-link review only; no new UI build or execution",
                                  "arena_bytes": 67108864, "reclaims": False},
            "tool_lock_sha256": runtime.digest(root / "config/m1-inputs.lock.json"),
            "cargo_lock_sha256": runtime.digest(root / "Cargo.lock")})
        sources = ["config/m3-profiles.json", "contracts/runtime-v0.json", "tools/m3_runtime.py",
                   "tools/Test-P3-01.py", "tools/Review-P3-01.py", "docs/06-contracts/runtime-v0.md",
                   "docs/05-delivery/phase-3/runtime-prerequisites-runbook.md"]
        result = {"schema_version": 1, "status": "passed", "scope": "P3-01 prerequisites/contracts/host lifecycle only",
                  "source_base": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
                  "inputs": {p: runtime.digest(root / p) for p in sources}, "rows": rows,
                  "prerequisites": str(args.prerequisites).replace(str(root), "${WORKSPACE}"),
                  "prerequisites_sha256": runtime.digest(args.prerequisites / "prerequisites.json"),
                  "m1": str(args.m1).replace(str(root), "${WORKSPACE}"),
                  "m1_review_sha256": runtime.digest(args.m1 / "review.json"),
                  "image_sha256": sha, "generated_sdk_sha256": runtime.digest(run / "draft-a.rs"),
                  "lifecycle": lifecycle, "guest_runtime_executed": False, "abi_frozen": False}
        runtime.save(run / "acceptance.json", result)
    except Exception as error:
        runtime.save(run / "failure.json", {"status": "failed", "error": str(error), "rows": rows})
        print(run)
        raise
    print(run)


if __name__ == "__main__":
    main()
