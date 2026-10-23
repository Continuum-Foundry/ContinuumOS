"""Independent P2-08 sample validation, nearest-rank aggregation and compact export.

Reads only local artifacts. Does not infer guest or native display performance.
"""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys


def read(path):
    return json.loads(Path(path).read_text(encoding="utf-8-sig"))


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write(path, value):
    def clean(item):
        if isinstance(item, str):
            return item.replace(str(Path.cwd()), "${WORKSPACE}").replace(str(Path.cwd()).replace("\\", "/"), "${WORKSPACE}")
        if isinstance(item, list):
            return [clean(v) for v in item]
        if isinstance(item, dict):
            return {k: clean(v) for k, v in item.items()}
        return item
    Path(path).write_text(json.dumps(clean(value), indent=2) + "\n", encoding="utf-8")


def ranks(values):
    values = sorted(values)
    assert values and all(isinstance(n, (float, int)) and math.isfinite(n) and n >= 0 for n in values)
    return {"n": len(values), **{f"p{p}": values[math.ceil(len(values) * p / 100) - 1] for p in (50, 95, 99)}, "max": values[-1]}


def table(path, fields, rows):
    with Path(path).open("w", newline="", encoding="utf-8") as file:
        writer = csv.DictWriter(file, fieldnames=fields)
        writer.writeheader()
        writer.writerows({key: row.get(key) for key in fields} for row in rows)


FRAME_FIELDS = ["n", "configuration", "workload", "variant", "layout_ns", "raster_ns", "submission_ns", "frame_ns", "input_to_ram_submission_ns", "trace_ns", "total_ns", "scheduled_lateness_ns", "missed_60hz_deadline", "changed_pixels", "submitted_damage_pixels", "allocations", "allocated_bytes", "surface_bytes", "rgb_bytes", "audit_previous_bytes", "glyph_cache_bytes", "glyph_cache_entries", "scratch_peak_bytes", "shape_work", "realized", "touched", "rgba_sha256"]


def validate_frames(frames, trace):
    assert len(frames) == 300
    for n, row in enumerate(frames):
        assert (row["n"], row["configuration"], row["workload"], row["variant"]) == (n, n % 6, n // 6 % 6, n // 36 % 5)
        assert row["frame_ns"] >= row["layout_ns"] + row["raster_ns"] + row["submission_ns"]
        assert row["input_to_ram_submission_ns"] >= row["frame_ns"]
        assert row["missed_60hz_deadline"] == (row["frame_ns"] > 16_666_667)
        pixels = 1280 * 720 if n % 6 < 3 else 1920 * 1080
        assert row["submitted_damage_pixels"] == pixels
        assert 0 <= row["realized"] <= 64
        if trace:
            assert 0 <= row["changed_pixels"] <= pixels
            assert row["allocations"] > 0 and row["allocated_bytes"] > 0
            assert len(row["rgba_sha256"]) == 64
        else:
            assert row["changed_pixels"] is None and row["allocations"] is None and row["rgba_sha256"] is None
    assert len({(r["configuration"], r["workload"], r["variant"]) for r in frames}) == 180


def main(source, destination):
    source, destination = Path(source).resolve(), Path(destination).resolve()
    destination.mkdir(parents=True, exist_ok=True)
    acceptance = read(source / "acceptance.json")
    assert acceptance["status"] == "passed" and not acceptance["measurements_skipped"]
    assert all(c["passed"] and c["cleanup"] == "reaped" and not c["timed_out"] for c in acceptance["commands"])
    assert acceptance["binaries"]["v1"]["sha256"] != acceptance["binaries"]["v2"]["sha256"]
    for binary in acceptance["binaries"].values():
        assert digest(binary["path"]) == binary["sha256"]
        binary["path"] = str(Path(binary["path"]).relative_to(Path.cwd())).replace("\\", "/")
    prereq = read(acceptance["prerequisite"]["path"])
    assert digest(acceptance["prerequisite"]["path"]) == acceptance["prerequisite"]["sha256"]
    assert prereq["status"] == "passed" and len(prereq["commands"]) == 10 and all(r["exit_code"] == 0 for r in prereq["commands"])
    acceptance["prerequisite"]["path"] = str(Path(acceptance["prerequisite"]["path"]).resolve().relative_to(Path.cwd())).replace("\\", "/")
    metrics = ["layout_ns", "raster_ns", "submission_ns", "frame_ns", "input_to_ram_submission_ns", "total_ns", "scheduled_lateness_ns"]
    results, cells, run_index, accounting = [], [], [], []
    for candidate in ("custom", "slint"):
        modes = {}
        for trace in ("off", "on"):
            all_frames = []
            for repetition in range(5):
                name = f"{candidate}-{trace}-{repetition}"
                directory = source / name
                result = read(directory / "result.json")
                frames = read(directory / "frames.json")
                memory = read(directory / "process.json")
                assert result["status"] == "passed" and result["trace"] == (trace == "on") and result["repetition"] == repetition
                assert result["executable_sha256"] == acceptance["binaries"]["v1"]["sha256"]
                assert result["protocol_sha256"] == acceptance["protocol_sha256"]
                assert result["warmup_ms"] >= 5000 and result["active_ns"] >= 30_000_000_000 and result["idle_ns"] >= 30_000_000_000
                assert result["warmup_frames"] == 50
                assert result["idle_render_requests"] == 0
                validate_frames(frames, trace == "on")
                assert len(memory) == 600
                assert [m["phase"] for m in memory] == ["active"] * 300 + ["idle"] * 300
                assert all(a["process"]["cpu_100ns"] <= b["process"]["cpu_100ns"] for a, b in zip(memory, memory[1:]))
                rows = [{"phase": m["phase"], "ms": m["ms"], **m["process"]} for m in memory]
                table(destination / (name + "-frames.csv"), FRAME_FIELDS, frames)
                table(destination / (name + "-process.csv"), list(rows[0]), rows)
                write(destination / (name + ".json"), result)
                idle_cpu_ns = (result["idle_after"]["cpu_100ns"] - result["idle_before"]["cpu_100ns"]) * 100
                results.append({"run": name, "cold_process_repetition": repetition == 0, "frames": {m: ranks([r[m] for r in frames]) for m in metrics}, "missed_60hz": sum(r["missed_60hz_deadline"] for r in frames), "active_cpu_ns": (memory[299]["process"]["cpu_100ns"] - memory[0]["process"]["cpu_100ns"]) * 100, "active_cpu_scope": "after first frame through last frame; warm-up and first frame excluded", "idle_cpu_ns": idle_cpu_ns, "idle_single_core_percent": idle_cpu_ns / result["idle_ns"] * 100, "working_set": ranks([r["process"]["working_set"] for r in memory]), "private_commit": ranks([r["process"]["private_commit"] for r in memory]), "peak_working_set": max(r["process"]["peak_working_set"] for r in memory), "peak_commit": max(r["process"]["peak_commit"] for r in memory), "active_overrun_ns": max(0, result["active_ns"] - 30_000_000_000), "observer_timer_wakes": 300, "os_scheduler_wakeups": None})
                unique = [next(r for r in frames if r["configuration"] == c) for c in range(6)]
                accounting.append({"run": name, "named_retained_buffer_capacity_bytes": {key: sum(r[key] for r in unique) for key in ("surface_bytes", "rgb_bytes", "audit_previous_bytes")}, "frame_allocations": sum(r["allocations"] for r in frames) if trace == "on" else None, "frame_requested_allocation_bytes": sum(r["allocated_bytes"] for r in frames) if trace == "on" else None, "frame_allocation_calls": ranks([r["allocations"] for r in frames]) if trace == "on" else None, "frame_requested_bytes": ranks([r["allocated_bytes"] for r in frames]) if trace == "on" else None, "glyph_cache_peak_payload_bytes": max(r["glyph_cache_bytes"] for r in frames), "frame_reported_scratch_peak_bytes": max(r["scratch_peak_bytes"] for r in frames), "submitted_damage_pixels": sum(r["submitted_damage_pixels"] for r in frames), "observed_changed_pixels": sum(r["changed_pixels"] for r in frames) if trace == "on" else None, "active_private_growth_bytes": memory[299]["process"]["private_commit"] - memory[0]["process"]["private_commit"], "active_working_set_growth_bytes": memory[299]["process"]["working_set"] - memory[0]["process"]["working_set"], "idle_private_growth_bytes": result["idle_after"]["private_commit"] - result["idle_before"]["private_commit"], "idle_working_set_growth_bytes": result["idle_after"]["working_set"] - result["idle_before"]["working_set"], "trace_work_ns": ranks([r["trace_ns"] for r in frames]), "limits": "allocation traffic covers frame function only; cache payload and scratch are subcomponent counters, not whole-process live heap; changed pixels do not reduce submitted full damage"})
                run_index.append({"run": name, "local_frames_sha256": digest(directory / "frames.json"), "local_process_sha256": digest(directory / "process.json"), "local_result_sha256": digest(directory / "result.json")})
                all_frames.extend({**r, "repetition": repetition} for r in frames)
            for config in range(6):
                for workload in range(6):
                    for variant in range(5):
                        sample = [r for r in all_frames if (r["configuration"], r["workload"], r["variant"]) == (config, workload, variant)]
                        cells.append({"candidate": candidate, "trace": trace, "configuration": config, "workload": workload, "variant": variant, "n": len(sample), "frame_p50_ns": ranks([r["frame_ns"] for r in sample])["p50"], "frame_p95_ns": ranks([r["frame_ns"] for r in sample])["p95"], "frame_p99_ns": ranks([r["frame_ns"] for r in sample])["p99"], "missed_60hz": sum(r["missed_60hz_deadline"] for r in sample)})
            modes[trace] = {"all_processes": {m: ranks([r[m] for r in all_frames]) for m in metrics}, "warm_process_repetitions_1_to_4": {m: ranks([r[m] for r in all_frames if r["repetition"] > 0]) for m in metrics}, "missed_60hz": sum(r["missed_60hz_deadline"] for r in all_frames), "n": len(all_frames)}
        write(destination / (candidate + "-distributions.json"), modes)
        reload = read(source / (candidate + "-reload") / "result.json")
        assert reload["status"] == "passed" and len(reload["cycles"]) == 100
        assert reload["owner_before"]["checkpoint_sha256"] == reload["owner_after"]["checkpoint_sha256"]
        assert all(r["owner_pid"] == reload["owner_pid"] and r["state"]["checkpoint_sha256"] == reload["owner_before"]["checkpoint_sha256"] for r in reload["cycles"])
        assert all(a["monotonic_ns"] < b["monotonic_ns"] for a, b in zip(reload["cycles"], reload["cycles"][1:]))
        helpers = reload["helpers"]
        assert [h["launch"]["success"] for h in helpers] == [True, True, False, False, False]
        assert all(h["launch"]["cleanup"] == "reaped" for h in helpers)
        assert helpers[-1]["launch"]["timed_out"]
        assert reload["failed_build"]["cleanup"] == "reaped" and reload["failed_build"]["exit"] != 0
        assert reload["failed_build"]["owner_checkpoint_sha256"] == reload["owner_before"]["checkpoint_sha256"]
        assert all(reload[k] for k in ("invalid_tokens_rejected", "invalid_resources_rejected", "active_composition_gesture_blocks_replacement", "removed_focus_cancelled", "failed_returned_checkpoint_rejected"))
        assert not (source / (candidate + "-reload") / "must-not-launch").exists()
        assert "injected candidate startup failure" in (source / (candidate + "-reload") / "start-failure" / "stderr.log").read_text()
        assert "restore:" in (source / (candidate + "-reload") / "restore-failure" / "stderr.log").read_text()
        assert [h["launch"]["result"]["variant"] for h in helpers[:2]] == [1, 2]
        assert all(h["owner_pid"] == reload["owner_pid"] and h["launch"]["pid"] != reload["owner_pid"] for h in helpers)
        for index, variant in enumerate(("v1", "v2")):
            launch = helpers[index]["launch"]
            child = launch["result"]
            assert child["pid"] == launch["pid"]
            assert child["build_sha256"] == acceptance["binaries"][variant]["sha256"]
            assert child["restored"]["checkpoint_sha256"] == reload["owner_before"]["checkpoint_sha256"]
            assert child["after"]["document"] == ("Draft: Continuum\n" if index == 0 else "Draft: CONTINUUM\n")
            assert child["after"]["selection"] == [16, 16]
            assert child["after"]["window"] == reload["owner_before"]["window"]
            assert child["after"]["focus"] == reload["owner_before"]["focus"]
            assert child["first_frame"]["rgba_sha256"] != child["changed_frame"]["rgba_sha256"]
        assert helpers[0]["launch"]["result"]["first_frame"]["rgba_sha256"] == helpers[1]["launch"]["result"]["first_frame"]["rgba_sha256"]
        assert helpers[0]["launch"]["result"]["changed_frame"]["rgba_sha256"] != helpers[1]["launch"]["result"]["changed_frame"]["rgba_sha256"]
        table(destination / (candidate + "-reload-cycles.csv"), ["n", "kind", "pause_ns", "owner_pid", "monotonic_ns", "checkpoint_sha256", "rgba_sha256", "scene_state"], [{**r, "checkpoint_sha256": r["state"]["checkpoint_sha256"], "rgba_sha256": r["frame"]["rgba_sha256"], "scene_state": json.dumps({k: r["state"][k] for k in ("selection", "focus", "window", "scroll_row", "dirty")}, separators=(",", ":"))} for r in reload["cycles"]])
        reload["cycle_pause_ns"] = ranks([r["pause_ns"] for r in reload.pop("cycles")])
        write(destination / (candidate + "-reload.json"), reload)
    table(destination / "cells.csv", list(cells[0]), cells)
    write(destination / "runs.json", results)
    write(destination / "accounting.json", accounting)
    paired = []
    by_name = {r["run"]: r for r in results}
    for candidate in ("custom", "slint"):
        for repetition in range(5):
            off, on = (by_name[f"{candidate}-{trace}-{repetition}"] for trace in ("off", "on"))
            paired.append({"candidate": candidate, "repetition": repetition, "frame_p50_delta_ns_on_minus_off": on["frames"]["frame_ns"]["p50"] - off["frames"]["frame_ns"]["p50"], "total_p50_delta_ns_on_minus_off": on["frames"]["total_ns"]["p50"] - off["frames"]["total_ns"]["p50"], "active_cpu_delta_ns_on_minus_off": on["active_cpu_ns"] - off["active_cpu_ns"], "peak_commit_delta_bytes_on_minus_off": on["peak_commit"] - off["peak_commit"]})
    write(destination / "paired-overhead.json", {"pairs": paired, "interpretation": "observed sequential paired deltas, not silently subtracted; power/load/cache and retention variation may dominate and can produce negative deltas; trace-off retains common phase timers, sample vectors and disabled allocator branch"})
    write(destination / "local-artifacts.json", {"root": str(source.relative_to(Path.cwd())).replace("\\", "/"), "runs": run_index, "source_inputs_sha256": digest(source / "source-inputs.json")})
    write(destination / "acceptance.json", acceptance)
    write(destination / "prerequisites.json", prereq)
    write(destination / "compiled-inputs.json", read(source / "source-inputs.json"))
    print(f"Validated 20 runs, 6000 frames, 12000 process samples, 200 resource cycles, 10 helpers: {destination}")


def self_test():
    import copy
    assert ranks([1, 2, 3, 4, 100]) == {"n": 5, "p50": 3, "p95": 100, "p99": 100, "max": 100}
    frames = []
    for n in range(300):
        frames.append({"n": n, "configuration": n % 6, "workload": n // 6 % 6, "variant": n // 36 % 5, "layout_ns": 1, "raster_ns": 2, "submission_ns": 3, "frame_ns": 6, "input_to_ram_submission_ns": 7, "missed_60hz_deadline": False, "submitted_damage_pixels": 1280 * 720 if n % 6 < 3 else 1920 * 1080, "realized": 16, "changed_pixels": 1, "allocations": 1, "allocated_bytes": 4, "rgba_sha256": "0" * 64})
    validate_frames(frames, True)
    mutations = {"n": 999, "configuration": 99, "workload": 99, "variant": 99, "frame_ns": 1, "input_to_ram_submission_ns": 1, "missed_60hz_deadline": True, "submitted_damage_pixels": 1, "realized": 65, "changed_pixels": -1, "allocations": 0, "allocated_bytes": 0, "rgba_sha256": "short"}
    rejected = []
    for key, value in mutations.items():
        changed = copy.deepcopy(frames)
        changed[0][key] = value
        try:
            validate_frames(changed, True)
        except AssertionError:
            rejected.append(key)
    assert len(rejected) == len(mutations)
    for changed, mode in ((frames[:-1], True), (frames, False)):
        try:
            validate_frames(changed, mode)
        except AssertionError:
            rejected.append("truncation-or-mode")
    assert len(rejected) == 15
    print(json.dumps({"status": "passed", "nearest_rank_vector": True, "rejected_sample_mutations": rejected}))


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        self_test()
    else:
        main(*sys.argv[1:])
