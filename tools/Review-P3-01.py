"""Validate P3-01 final identities and retain a compact local acceptance bundle."""
import argparse
import json
import os
from pathlib import Path
import shutil

import m3_runtime as runtime


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", type=Path, required=True)
    parser.add_argument("--destination", type=Path, required=True)
    args = parser.parse_args()
    root = runtime.ROOT

    def read(path):
        return json.loads(Path(path).read_text(encoding="utf-8-sig"))

    def local(value):
        return Path(value.replace("${WORKSPACE}", str(root)))

    def safe(value):
        if isinstance(value, dict):
            return {key: safe(item) for key, item in value.items()}
        if isinstance(value, list):
            return [safe(item) for item in value]
        if isinstance(value, str):
            for source, target in ((str(root), "${WORKSPACE}"), (root.as_posix(), "${WORKSPACE}"),
                                   (os.environ.get("USERPROFILE", "__NO_PROFILE__"), "${USERPROFILE}")):
                value = value.replace(source, target)
            return value
        return value

    acceptance = read(args.run / "acceptance.json")
    runtime.require(acceptance["status"] == "passed", "acceptance failed")
    for path, digest in acceptance["inputs"].items():
        runtime.require(runtime.digest(root / path) == digest, "stale final source: " + path)
    baseline = local(acceptance["m1"])
    review = read(baseline / "review.json")
    runtime.require(runtime.digest(baseline / "review.json") == acceptance["m1_review_sha256"], "changed M1 review")
    runtime.require(review["review_tool_sha256"] == runtime.digest(root / "tools/Review-P1-07.ps1"), "stale M1 reviewer")
    runtime.require(review["status"] == "passed" and len(review["launches"]) == 42, "incomplete M1")
    pre_dir = local(acceptance["prerequisites"])
    runtime.require(runtime.digest(pre_dir / "prerequisites.json") == acceptance["prerequisites_sha256"], "changed prerequisites")
    for item in acceptance["lifecycle"]:
        path = root / item["path"] / "result.json"
        runtime.require(runtime.digest(path) == item["result_sha256"], "changed lifecycle result")
        runtime.require(item["result"]["cleanup"] == "reaped", "unreaped child")
    destination = args.destination.resolve()
    runtime.require(destination.is_relative_to(root / "evidence/phase-3/P3-01"), "wrong evidence root")
    runtime.require(not destination.exists(), "never overwrite an existing evidence bundle")
    destination.mkdir(parents=True)
    for name in ("acceptance.json", "hook-gap-map.json", "runtime-input-inventory.json"):
        runtime.save(destination / name, safe(read(args.run / name)))
    runtime.save(destination / "prerequisites.json", safe(read(pre_dir / "prerequisites.json")))
    runtime.save(destination / "m1-review.json", safe(review))
    runtime.save(destination / "m1-result.json", safe(read(baseline / "result.json")))
    sources = read(baseline / "suite-source-inputs.json")
    for path, digest in sources.items():
        runtime.require(runtime.digest(root / path) == digest, "changed baseline source: " + path)
    runtime.save(destination / "baseline-source-inputs.json", sources)
    launch_index = []
    launch_dir = destination / "launches"
    launch_dir.mkdir()
    for index, launch in enumerate(review["launches"]):
        folder = local(launch["directory"])
        serial = folder / "serial.log"
        shutil.copyfile(serial, launch_dir / f"{index:02d}-serial.txt")
        result = read(folder / "run-result.json")
        manifest = read(folder / "launch-manifest.json")
        runtime.require(result["cleanup_reaped"] and result["deadline_ms"] == 30000 and
                        (not result["timed_out"] or launch["outcome"] == "broken-before-entry"),
                        "baseline lifecycle mismatch")
        runtime.save(launch_dir / f"{index:02d}-context.json", safe({
            "result": result, "launch": manifest,
            "package": read(folder / "case-result.json")["package"],
            "serial_sha256": runtime.digest(serial),
            "host_logs": {name: {"sha256": runtime.digest(folder / name),
                                 "text": (folder / name).read_text(encoding="utf-8", errors="strict")}
                          for name in ("qemu.stdout.log", "qemu.stderr.log")}}))
        launch_index.append({"number": index, "fixture": launch["fixture"], "outcome": launch["outcome"],
                             "serial_sha256": runtime.digest(serial)})
    runtime.save(destination / "review.json", {
        "schema_version": 1, "status": "passed", "review": "local Codex self-review; not independent approval",
        "review_tool_sha256": runtime.digest(__file__), "acceptance_rows": len(acceptance["rows"]),
        "m1_rows": 33, "m1_launches": launch_index,
        "criteria": {"current_prerequisites_baseline": "passed", "draft_runtime_SDK": "passed",
                     "scheduling_IRQ_FP_deadline_owners": "passed", "locked_limits_measurements": "passed",
                     "bounded_host_runner_negatives_cleanup": "passed"},
        "guest_runtime_implemented": False, "ABI_frozen": False, "desktop_targets_accepted": False})
    # The authored README/development findings and final checksum manifest are
    # added after review. Every retained member must be staged before hygiene.
    files = list(destination.rglob("*"))
    files = [p for p in files if p.is_file()]
    runtime.require(len(files) < 195 and sum(p.stat().st_size for p in files) < 4 * 1024 * 1024 and
                    all(p.stat().st_size <= 256 * 1024 for p in files), "compact evidence budget exceeded")
    print(destination)


if __name__ == "__main__":
    main()
