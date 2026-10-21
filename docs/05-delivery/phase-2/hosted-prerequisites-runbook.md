# P2-01 — Hosted prerequisites and lifecycle runner

This is host experiment infrastructure, with a deterministic headless fixture.
It does not open a window, render a scene, select a toolkit or update the guest.
P1-07's [accepted baseline](../../../evidence/phase-1/P1-07/2026-10-20-acceptance/README.md)
remains the ISO/VM evidence boundary. Its historical raw evidence availability is
described in [evidence/README](../../../evidence/README.md).

## Commands and local output

Use PowerShell 7 from the repository root, with the already enrolled development
signer. No script here installs tools, enrolls a key or changes host policy.
Cargo stays offline. If sandbox certificate-store access or xorriso is blocked,
retain that failure and run in the established local execution context. Do not
use an unsigned bypass or silently renew enrollment.

```powershell
. ./tools/Enter-Dev.ps1
pwsh -NoProfile -File tools/Recheck-P2Prerequisites.ps1
pwsh -NoProfile -File tools/Test-P2-01.ps1

# Optimized and diagnostic runs use distinct Cargo profiles and binaries.
cargo run --locked -p xtask --profile m2-comparison -- hosted --config config/m2-profiles.json --profile hd --scale 1.0 --case success
cargo run --locked -p xtask --profile m2-diagnostic -- hosted --config config/m2-profiles.json --profile full-hd --scale 1.5 --case success
```

The recheck runs fresh signing/compiler preflights, both doctor forms, feasibility
build, M0 clean/link comparison, Test-M0 (including independent LLVM inspection,
source/output paths with spaces and bad overrides), all five dependency probes,
and M1 doctor against its pinned inputs. It retains separate command exits,
arguments, elapsed time, CPU/OS, expiry, runner-config hash and cached facilities.
Full logs, source patch and all source-input hashes stay under ignored
`build/m2/P2-01/prerequisites-<timestamp>/`; nested M0/probe runs keep their normal
local paths. A nonzero command or signing failure leaves the gate incomplete.
The script is an explicit local acceptance sequence; its existing Phase 0
commands retain their existing timeout behavior. It is not the bounded scene runner.

`Test-P2-01.ps1` builds both profiles and exercises the complete resolution/scale
matrix, paths with spaces, deliberate nonzero exit, forced timeout, invalid
profile/scale/size/deadline, unavailable config/input, changed checksum and unknown
fields. Each invocation writes a new `build/m2/P2-01/run-<nanoseconds>-<pid>/`.
The acceptance index is under `build/m2/P2-01/acceptance-<timestamp>/`.
Negative runner commands return nonzero; acceptance succeeds only when their
manifests show the expected failure and cleanup.

## Locked profiles and measurement rules

[config/m2-profiles.json](../../../config/m2-profiles.json) is the version-1
measurement protocol and fixture-input lock. The runner rejects changed geometry,
unsupported scales/refresh, unknown fields, invalid deadlines and a changed protocol
before launching. Input paths resolve inside the workspace, are limited to 64 KiB
and require the recorded SHA-256. A missing input cannot reuse old success.

| Setting | Fixed value / interpretation |
| --- | --- |
| Physical pixels | `hd`: 1280×720; `full-hd`: 1920×1080, reported separately |
| Logical scale | 1.0, 1.5, 2.0; logical extent is physical extent divided by scale |
| Presentation budget | 60 Hz requested; actual host display/compositor cadence must be recorded by the future adapter; no VSync claim here |
| Comparison build | `m2-comparison`: release, opt-level 3, debug 1, LTO off, one codegen unit |
| Diagnostic build | `m2-diagnostic`: dev, opt-level 1, debug 2, assertions and overflow checks on |
| Clock | Rust `Instant`, with QPC/frequency imports verified in the actual Windows executable; process-relative nanoseconds; UTC labels identify runs only |
| Repetitions | Five independent runs per configuration; keep cold first run separate from warmed runs |
| Warm-up / windows | 5 s warm-up, 30 s active, 30 s idle, process samples every 100 ms; per-event/frame samples retained separately |
| Inputs | Seed 20261020; six named workload families in the JSON; shared scene/script/font hashes must be supplied by P2-02/P2-04 before either candidate is measured |
| Reload | 100 consecutive compatible cycles, fixed future corpus and settling rule; no reload is executed by P2-01 |
| Aggregation | Raw samples, nearest-rank p50/p95/p99, missed periods and event losses; no average-only or cross-machine clock subtraction |
| Instrumentation | Paired enabled/disabled runs with identical inputs, report CPU/frame/memory deltas; never silently subtract overhead |

The runner records 4096 raw timer-call pairs and their minimum/p50/p95 in every
run. These are a clock-call baseline, not UI instrumentation overhead, a timing
accuracy guarantee or guest performance. Actual adapter/tracing overhead remains
a prerequisite of P2-08's measurements. CPU time, working set and private commit
will describe only the named host process; retain system load/power/display state
with future measurements. No host counter certifies the 256 MiB guest cold-idle
target or the guest frame/input targets.

## Dependencies, facilities and rights

P2-01 adds **no dependency** and changes no Cargo lockfile. All executable runner
code is host-only in `xtask`; no new portable UI/domain or kernel code exists.
It uses Rust 1.99.1 `std`, existing serde_json 1.0.154 (default features) and sha2
0.11.1 (default features), with exact transitive packages/checksums in
[Cargo.lock](../../../Cargo.lock) and the [rights inventory](../../../legal/dependency-inventory.json).
Upstream declarations/notices and existing tool terms remain governed by
[THIRD_PARTY_NOTICES](../../../THIRD_PARTY_NOTICES.md). No release clearance follows.

The local cache inventory records each candidate manifest and notice-file hash.
Observed facilities include winit 0.30.13 / softbuffer 0.4.8 for a possible native
window/software presentation adapter, windows 0.62.2 and windows-sys 0.52.0/0.61.2,
Slint and its backends 1.18.1, tracing 0.1.44, image 0.25.10 and png 0.18.1.
**None is added, feature-selected or executed by this runner.** Cached presence
does not prove offline host graph closure or license suitability. Before use, its
owning candidate story must lock the exact host graph/features, preserve notices
and demonstrate native integration; a missing transitive input blocks that story.

Existing portable candidates retain Phase 0's exact features: tiny-skia 0.12.0
defaults off + no-std-float; rustybuzz 0.20.1 defaults off; fontdue 0.9.4 defaults
off + hashbrown; Slint 1.18.1 defaults off + compat-1-18, unsafe-single-threaded,
libm, renderer-software. This is guest library feasibility, not a hosted adapter
or guest final-link result. Slint's license choice remains open.

Tracing now means structured lifecycle JSON and raw clock samples, without a new
tracing dependency. No image comparison or capture tool is adopted: P2-02 fixes
pixel/semantic tolerances and P2-05/P2-06 implement the common surface/capture path.
There are no approved font/image assets in this fixture. Their provenance is a
separate P2-04 gate; this process test needs none. Do not fetch or infer font rights.

## Process and manifest contract

The parent validates first, then directly starts its own already signed executable
with a fixed `hosted-probe` fixture, null stdin, hidden window and file-backed
stdout/stderr. There is no shell, arbitrary command, network or descendant process.
The three fixture cases are success, deliberate nonzero exit, and a sleeping child.
The timeout fixture uses a fixed 100 ms deadline; normal configuration permits
100–10000 ms (default 1000). Parent checks every 5 ms, kills a timed-out child and
waits for it. This is a best-effort host deadline, not hard real time. Logs are
checked against 64 KiB; fixed fixture output is inherently bounded.

`result.json` records config/input/harness/manifest/lock and signed-executable
hashes, arguments, parent/child IDs, profile/scale, deadline, process-relative
elapsed time, exit code, outcome and cleanup separately. `not-launched` means
validation prevented launch; `reaped` requires a completed wait. `unconfirmed`
does not claim successful cleanup if a host process API fails; the owning guard
still attempts kill/wait on every return path. The acceptance index links exact
result hashes to build commands. Baseline manifests supply compiler, OS/CPU and
source commit/dirty-input identity. All inputs must agree before review.

## Review and handoff

Preserve accepted owned CR3/runtime/IST/exception mappings, ordinary-RAM-only
direct map, explicit ACPI/device cache policy and all version-5 timer/identity
boundaries. No kernel behavior is changed. A full M1 VM matrix is unnecessary
for this host infrastructure slice; doctor and existing host tests are rerun.
Final link still requires the allocator/platform work of P2-07.

P2-02 receives the profile lock and operational headless lifecycle runner. It
must define the common scene/surface/input corpus before candidate rendering,
and P2-04 must supply licensed font/text inputs. No toolkit decision, rendering,
live state continuity, isolation or guest performance result is implied.
