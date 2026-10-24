"""Independent raw-record consumer for P2-12. No inferred guest success."""
import csv
import hashlib
import json
import math
import statistics
import sys
from pathlib import Path


def read(path):
    return json.loads(Path(path).read_text(encoding="utf-8-sig"))


def require(ok, why):
    if not ok:
        raise AssertionError(why)


def distribution(values):
    values = sorted(values)
    return {"count": len(values), **{f"p{q}": values[max(0, math.ceil(len(values) * q / 100) - 1)] for q in (50, 95, 99)}, "max": max(values)}


def save(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def export_csv(path, rows):
    with Path(path).open("w", newline="", encoding="utf-8") as stream:
        writer = csv.DictWriter(stream, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)


run = Path(sys.argv[1])
acceptance = read(run / "acceptance.json")
require(acceptance["status"] == "passed", "Harness did not pass")
p = read(run / "measure-0-off/protocol.json")
require((p["reload_cycles"], p["pixel_tolerance"], p["repetitions"], p["active_frames"], p["idle_ms"]) == (100, 0, 5, 300, 30000), "Changed experiment identity")
out = run / "review"
out.mkdir(exist_ok=True)
outputs = acceptance["outputs"]
damage = read(Path(outputs["damage"]) / "damage.json")
require(len(damage["cases"]) == 108 and damage["fragmentation_fallback"], "Missing damage fixtures")
for row in damage["cases"]:
    require(row["max_channel_error"] == 0, "Pixel tolerance changed")
    require(row["full_rgba_sha256"] == row["incremental"]["rgba_sha256"], "Pixel oracle disagrees")
    require(len(row["incremental"]["regions"]) <= 32, "Unbounded fragmentation")
    if row["case"] == "static-no-paint":
        require(not row["incremental"]["painted"], "Static damage repainted")
require(any(0 < r["incremental"]["damage_pixels"] < r["geometry"][0] * r["geometry"][1] for r in damage["cases"]), "No real incremental fixture")
save(out / "damage-review.json", {"status": "passed", "cases": len(damage["cases"]), "tolerance": 0,
     "partial_cases": sum(0 < r["incremental"]["damage_pixels"] < r["geometry"][0] * r["geometry"][1] for r in damage["cases"]),
     "full_render_baseline_ns": distribution([r["full_render_ns"] for r in damage["cases"]]),
     "oracle_scope": "same selected renderer, independent full framebuffer; no golden tolerance increase"})

blocks = []
all_frames = []
all_process = []
for rep in range(5):
    for mode in ("off", "on"):
        directory = run / f"measure-{rep}-{mode}"
        result = read(directory / "result.json")
        frames = read(directory / "frames.json")
        process = read(directory / "process.json")
        idle = read(directory / "idle-frames.json")
        require(result["status"] == "passed" and result["repetition"] == rep and result["trace"] == (mode == "on"), "Wrong block identity")
        require(result["executable_sha256"] == acceptance["signed_executable_sha256"], "Measurement executable differs")
        require(read(directory / "protocol.json") == p, "Protocol changed during run")
        require(len(frames) == 300 and len(process) > 0, "Missing raw samples")
        require([r["n"] for r in frames] == list(range(300)), "Dropped or reordered slots")
        for phase in ("active","idle"):
            samples=[r for r in process if r["phase"]==phase]
            require(samples and samples[0]["slot"]==1,"Missing initial process observation")
            for first,second in zip(samples,samples[1:]):
                require(second["elapsed_ns"]>first["elapsed_ns"] and second["slot"]-first["slot"]-1==first["missed_observations"],"Unreported process sampling gap")
            require(0 <= result[phase+"_ns"]-samples[-1]["elapsed_ns"] < 250000000,"Missing final process observations")
        require(sum(r["missed_observations"] for r in process)==result["process_sample_misses"],"Wrong process observation loss count")
        require(len(result["idle_phases"])==4,"Missing idle phases")
        for phase in result["idle_phases"]:
            require(phase["elapsed_ns"] >= (10000000000 if phase["phase"] in ("static","clock-caret") else 5000000000),"Short idle phase")
            require(phase["frames"] <= 6*(phase["elapsed_ns"]//500000000+1) if phase["phase"]=="clock-caret" else phase["frames"]==0,"Idle timer catch-up or hidden repaint")
        require(all(frames[n]["geometry_index"] == n % 6 and frames[n]["input"]["workload"] == (n // 6) % 6 for n in range(300)), "Workload changed")
        require(result["active_ns"] >= 30_000_000_000 and result["idle_ns"] >= 30_000_000_000, "Short measurement window")
        require(result["static_frames"] == result["hidden_frames"] == result["minimized_frames"] == 0, "Idle repainted")
        require(result["clock_caret_surface_wakes"] == len(idle) > 0, "Timers missing")
        require(all(r["phase"] == "clock-caret" and r["frame"]["painted"] for r in idle), "Timer did not repaint")
        require(all(r["cache_entries"] <= 128 and r["cache_bytes"] <= 1048576 and r["layout_cache_entries"] <= 128 and r["layout_cache_bytes"] <= 2097152 for r in frames), "Cache grew beyond bounds")
        for r in frames:
            w, h, scale = p["geometry"][r["geometry_index"]]
            require(r["damage_pixels"] <= w * h and len(r["regions"]) <= 32, "Bad damage extent")
            if mode == "on":
                require(r["raster_ns"] + r["compose_ns"] == r["raster_compose_ns"], "Phase accounting disagrees")
                require(r["input"]["state_ns"] + r["input"]["dispatch_ns"] <= r["input"]["dispatch_state_total_ns"], "Dispatch phase accounting exceeds interval")
        fields = ("layout_ns", "raster_compose_ns", "raster_ns", "compose_ns", "submission_ns", "frame_ns", "audit_ns", "input_to_ram_submission_ns", "lateness_ns", "allocations", "allocated_bytes", "rust_live", "rust_peak", "damage_pixels", "touched", "cache_entries", "cache_bytes", "layout_cache_entries", "layout_cache_bytes", "layout_cache_hits", "layout_cache_misses", "paint_scratch_peak")
        raw_frames = [{"slot": r["n"], "geometry": r["geometry_index"], "workload": r["input"]["workload"],
                       "input_ns": r["input"]["input_ns"], "dispatch_ns": r["input"]["dispatch_ns"], "state_ns": r["input"]["state_ns"], **{k: r[k] for k in fields}} for r in frames]
        export_csv(out / f"frames-{rep}-{mode}.csv", raw_frames)
        raw_process = [{"phase": r["phase"], "slot": r["slot"], "elapsed_ns":r["elapsed_ns"], "missed_observations":r["missed_observations"], **r["process"]} for r in process]
        export_csv(out / f"process-{rep}-{mode}.csv", raw_process)
        export_csv(out / f"idle-{rep}-{mode}.csv", [{"elapsed_ns": r["elapsed_ns"], "geometry": r["geometry_index"], "frame_ns": r["frame"]["frame_ns"], "damage_pixels": r["frame"]["damage_pixels"]} for r in idle])
        blocks.append({"repetition": rep, "trace": mode, "cold_process_repetition": rep == 0,
                       "frames": distribution([r["frame_ns"] for r in frames]), "active_ns": result["active_ns"],
                       "idle_ns": result["idle_ns"], "idle_cpu_ms": (result["idle_after"]["cpu_100ns"] - result["idle_before"]["cpu_100ns"]) / 10000,
                       "missed_60hz": sum(r["frame_ns"] > 16666667 for r in frames),
                       "peak_private_commit": max(r["process"]["private_commit"] for r in process),
                       "peak_working_set": max(r["process"]["working_set"] for r in process),
                       "clock_caret_surface_wakes": len(idle), "observer_wakes": result["observer_wakes"], "missed_process_observations":result["process_sample_misses"], "idle_phases":result["idle_phases"],
                       "static_hidden_minimized_frames": 0})
        all_frames.extend({**r, "rep": rep, "trace": mode} for r in frames)
        all_process.extend(process)

groups = []
for index, geometry in enumerate(p["geometry"]):
    for workload in range(6):
        for mode in ("off", "on"):
            values = [r for r in all_frames if r["geometry_index"] == index and r["input"]["workload"] == workload and r["trace"] == mode]
            groups.append({"geometry": geometry, "workload": p["workloads"][workload], "trace": mode,
                           "frames_ns": distribution([r["frame_ns"] for r in values]),
                           "raster_compose_ns": distribution([r["raster_compose_ns"] for r in values]),
                           "input_to_ram_ns": distribution([r["input_to_ram_submission_ns"] for r in values]),
                           "missed_60hz": sum(r["frame_ns"] > 16666667 for r in values),
                           "partial_frames": sum(0 < r["damage_pixels"] < geometry[0] * geometry[1] for r in values),
                           "no_paint_frames": sum(not r["painted"] for r in values)})

for width in (1280, 1920):
    phases=[]
    for index, geometry in enumerate(p["geometry"]):
        if geometry[0] != width:
            continue
        for workload in range(6):
            values=[r for r in all_frames if r["geometry_index"]==index and r["input"]["workload"]==workload and r["trace"]=="on"]
            phases.append({"geometry":geometry,"workload":p["workloads"][workload],
                "phase_ns":{k:distribution([r[k] for r in values]) for k in ("layout_ns","raster_ns","compose_ns","submission_ns")},
                "input_phase_ns":{k:distribution([r["input"][k] for r in values]) for k in ("input_ns","dispatch_ns","state_ns")}})
    save(out / f"phases-{width}.json",phases)

stability = []
for geometry in ("1280x720", "1920x1080"):
    directory = Path(outputs["stability"]) / geometry
    result = read(directory / "result.json")
    rows = [json.loads(line) for line in (directory / "cycles.jsonl").read_text(encoding="utf-8").splitlines()]
    require(result["status"] == "passed" and len(rows) == 105 and result["headroom_rejected"] and result["reader_release_recovery"], "Missing stability or headroom cases")
    opacity=read(directory / "opacity-negative.json")
    require(result["opacity_rejected"] and opacity["builder_rejected"] and opacity["api_rejected"],"Nonopaque generation accepted")
    require(opacity["active_before"]==opacity["active_after"] and opacity["scene_revision_before"]==opacity["scene_revision_after"] and opacity["checkpoint_before"]==opacity["checkpoint_after"] and opacity["frame_before"]==opacity["frame_after"],"Nonopaque failure changed active state")
    save(out / f"opacity-{geometry}.json",opacity)
    base = rows[4]
    require(len({r["checkpoint_sha256"] for r in rows}) == 1, "Checkpoint changed")
    for i, r in enumerate(rows):
        require(r["cycle"] == i and r["generation"] == 8 + i and r["owner_address_unchanged"], "Continuity or cycle identity failure")
        require(r["resources"]["live_generations"] == 1 and r["cache_entries"] <= 128 and r["cache_bytes"] <= 1048576 and r["layout_cache_entries"] <= 128 and r["layout_cache_bytes"] <= 2097152, "Retired generation or unbounded cache")
        if i >= 5:
            growth = {"rust_live": max(0, r["settled_rust_live"] - base["settled_rust_live"]),
                      "private_commit": max(0, r["settled_process"]["private_commit"] - base["settled_process"]["private_commit"]),
                      "working_set": max(0, r["settled_process"]["working_set"] - base["settled_process"]["working_set"])}
            require(growth == r["growth_from_baseline"], "Growth calculation changed")
            require(growth["rust_live"] <= 65536 and growth["private_commit"] <= 4194304 and growth["working_set"] <= 8388608, "Growth tolerance exceeded")
    measured = rows[5:]
    # Four live copies of the variable-length build ID: active payload, current
    # source Value, status and query status. Decimal growth is not a new UI root.
    normalized = [r["settled_rust_live"] - 4 * len(str(r["generation"])) for r in measured]
    require(len(set(normalized)) == 1, "Unexplained live Rust allocation growth after settling")
    raw = [{"cycle": r["cycle"], "generation": r["generation"], "checkpoint_sha256": r["checkpoint_sha256"],
            "live_generations": r["resources"]["live_generations"], "cache_entries": r["cache_entries"], "cache_bytes": r["cache_bytes"],
            "build_ns": r["build_ns"], "visible_ns": r["completed_build_to_ram_visible_ns"],
            "rust_live": r["settled_rust_live"], "rust_peak": r["peak_rust_live"], "private_commit": r["settled_process"]["private_commit"],
            "working_set": r["settled_process"]["working_set"], "peak_private_commit": r["peak_process"]["private_commit"], "peak_working_set": r["peak_process"]["working_set"]} for r in rows]
    export_csv(out / f"cycles-{geometry}.csv", raw)
    stability.append({"geometry": geometry, "cycles": 100, "warmup": 5, "headroom_rejected": True, "reader_recovery": True,
                      "completed_build_to_ram_visible_ns": distribution([r["completed_build_to_ram_visible_ns"] for r in measured]),
                      "build_ns": distribution([r["build_ns"] for r in measured]),
                      "max_growth": {k: max(r["growth_from_baseline"][k] for r in measured) for k in measured[0]["growth_from_baseline"]},
                      "final_minus_baseline_rust_live": measured[-1]["settled_rust_live"] - base["settled_rust_live"],
                      "normalized_rust_live_plateau": normalized[0],
                      "identity_growth_explanation": "four owned build-ID string copies; decimal generation width only; baseline process-counter Value allocated once after warmup",
                      "peak_rust_live": max(r["peak_rust_live"] for r in measured),
                      "checkpoint_sha256": result["checkpoint_sha256"]})

native = read(Path(outputs["native-idle"]) / "native-idle.json")
require(native["status"] == "passed", "Native idle did not pass")
for row in native["observations"]:
    require(row["elapsed_ms"] >= (10000 if row["phase"] in ("static", "caret") else 5000), "Native idle interval shortened")
    require(row["submitted_frames"] == 0 if row["phase"] != "caret" else row["submitted_frames"] > 0, "Native idle scheduling failure")
save(out / "native-idle.json", native)

overhead = []
for rep in range(5):
    off = next(b for b in blocks if b["repetition"] == rep and b["trace"] == "off")
    on = next(b for b in blocks if b["repetition"] == rep and b["trace"] == "on")
    overhead.append({"repetition": rep, "p50_frame_delta_ns": on["frames"]["p50"] - off["frames"]["p50"],
                     "p95_frame_delta_ns": on["frames"]["p95"] - off["frames"]["p95"],
                     "idle_cpu_delta_ms": on["idle_cpu_ms"] - off["idle_cpu_ms"],
                     "private_commit_peak_delta": on["peak_private_commit"] - off["peak_private_commit"]})
save(out / "metrics.json", {"blocks": blocks, "groups": groups, "instrumentation_pairs": overhead, "stability": stability,
     "accounting": {"host_process_peaks": {k: max(r["process"][k] for r in all_process) for k in ("peak_working_set", "peak_commit")},
                    "frame_and_rgb_bytes": sum(w * h * 7 for w, h, s in p["geometry"]),
                    "rust_live_peak": max(r["rust_peak"] for r in all_frames),
                    "font_source_bytes": sum(f["bytes"] for f in read("fixtures/m2/text-v1/fonts.lock.json")["fonts"]),
                    "unattributed": "Process includes mapped code, stacks, runtime/OS heaps, allocator retention, text font tables, trees/model/log records; Rust LIVE charges all Rust requested bytes, including these data owners. Known categories overlap, never summed as disjoint totals.",
                    "native_presenter": "separate worker and WinForms presenter counters in native-idle.json"},
     "targets": {"guest_memory_budget_bytes": 268435456, "guest_measured": False, "photon_latency_measured": False,
                 "hosted_1080p_p95_8ms_and_p99_16ms": "Compare each named geometry/workload group; misses remain open bounded optimization work.",
                 "reload_target_ns": 500000000},
     "limits": p["limits"]})
files = [run / "acceptance.json", Path(outputs["damage"]) / "damage.json"] + [run / f"measure-{r}-{m}" / f for r in range(5) for m in ("off", "on") for f in ("frames.json", "process.json", "idle-frames.json", "result.json")]
save(out / "raw-inputs.json", [{"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest(), "bytes": path.stat().st_size} for path in files])
save(out / "performance-review.json", {"status": "passed", "frame_samples": len(all_frames), "process_samples": len(all_process), "damage_cases": 108,
     "reload_cycles": 200, "native_idle": True, "guest": False, "method": "Separate Python consumer rechecks raw phase/count/pixel/state/resource outcomes; this is local self-review, not independent approval."})
print(f"P2-12 independent raw-record review passed: {out}")
