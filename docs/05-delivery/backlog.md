# Ordered engineering backlog

Status: B01/B02 have [Phase 0 evidence](phase-0.md); B36/B38 have preliminary
reviews only. Runtime tasks remain unstarted. Owner column means a responsibility, not an assigned
agent/person. Read milestone gates before picking work. `→` dependencies must pass;
tasks within a gate can share infrastructure without implying simultaneous agents.

M1 is decomposed into [seven local Phase 1 stories](phase-1/README.md), retaining
the Bxx identifiers below. The stories expose B03/B04's minimal serial bootstrap
and separate B06's M1 timer baseline from its later scheduling/context work.

| ID | Owner / task | Depends on | Definition of done | Requirements |
| --- | --- | --- | --- | --- |
| B01 | Build: inventory host and candidate tools | Docs | Paths, versions, licenses, host constraints and fallback report | R03,R16,R18 |
| B02 | Build: pin toolchain and target/link configs | B01 | Minimal independent kernel/userland link experiments; syscall ABI remains M3 | R03,R14,R16 |
| B03 | Platform: validate QEMU/OVMF/ISO recipe | B01,B02,B38 | Exact ISO cold-boots through declared firmware/optical path | R01,R02 |
| B04 | Kernel: serial/boot-info diagnostics | B02,B03 | Checked metadata, fixed logging, build/boot IDs | R01,R17 |
| B05 | Kernel: memory ownership and exceptions | B04 | Reserved ranges, allocators, guards, controlled faults | R01,R10 |
| B06 | Kernel: timer/preemption/context switch | B05 | Register/interrupt/FP policy tests, monotonic uptime | R01,R10 |
| B07 | UI: selected portable toolkit/raster integration | B02,B36 | Chosen candidate, common surface/input boundary and reference scenes | R08,R09 |
| B08 | UI: layout, stable IDs and semantic tree | B07 | Incremental layout and keyboard/focus fixtures | R08,R15 |
| B09 | UI: text shaping/edit model | B07,B08 | Grapheme/bidi/shaping/fallback corpus and editor checkpoint | R07,R15 |
| B10 | UI: token/layout live swap on host | B08,B09 | Preserved text/focus, invalid data rejected | R04,R08 |
| B11 | Kernel: static ELF user loader and syscalls | B05,B06 | Validated entry, user-mode tasks, malformed ELF tests | R03,R10,R14 |
| B12 | Kernel: rights-bearing handles and IPC | B11 | Bounded channels, transfer/error/wait fixtures | R10,R14 |
| B13 | Services: supervisor, quotas and routes | B12 | Crash containment, epochs, bounded queues, stale writer tests | R06,R10,R14 |
| B14 | Platform: dev transport adapter | B03,B12 | Bounded framed byte stream with actual host/guest transfer | R13,R17 |
| B15 | Security: session authority and plan validation | B13,B14 | Replay, bad credential, permissions and malformed-plan tests | R10,R13,R14 |
| B16 | Graphics: guest surfaces/compositor/input | B06,B08,B12 | Hosted scenes appear in guest; focus/capture and buffer lifetime tests | R08,R09,R15 |
| B17 | Updates: guest L0/L1 publish | B10,B15,B16 | Visible no-reboot asset/layout update and rejection evidence | R04,R13,R17 |
| B18 | State: checkpoint codecs/migration fixtures | B09,B13 | Identity/adjacent migration, resource inventory, corrupt/oversize failures | R07,R14 |
| B19 | Updates: candidate prepare and quiescence | B15,B18 | Restricted startup, bounded drain/cancel, safe abort | R05,R06,R12 |
| B20 | Updates: activation/fencing/retirement | B19 | One writer, coherent route root, no reachable old code freed | R05,R06,R12 |
| B21 | QA: first product reload demo | B17,B20,B37 | UI+behavior retain text/selection/window; same boot ID; 100 cycles | R04,R05,R07,R09,R19 |
| B22 | Storage: block/DMA/flush implementation | B05,B12 | Descriptor/reset/flush tests; documented device durability contract | R10,R11 |
| B23 | Storage: record store/packages/generations | B22 | Torn-write recovery, hash validation and bounded GC | R11,R12,R16 |
| B24 | Updates: durable decision/recovery | B20,B23 | Faults before/after every record/fence/publish/ACK; recovery table passes | R06,R11,R12 |
| B25 | Services: settings replacement under load | B24 | No lost accepted writes, dedupe and compatibility tests | R06,R07,R14 |
| B26 | Product: file browser/save/autosave | B09,B16,B23 | Honest save status, path authority, recovery of unsaved buffer | R08,R11,R15 |
| B27 | Runtime: bounded Wasm spike | B12,B18,B20 | Fuel/memory/import limits and native comparison | R05,R09,R10 |
| B28 | Networking: stack and TLS integration | B12,B22-style DMA support | Controlled packets, clocks/entropy/provider negative tests | R10,R13 |
| B29 | Graphics: VirtIO GPU feasibility | B16, profiling | Exact guest/host backend path and measured benefit | R08,R09 |
| B30 | Platform: SMP experiment | B06,B12,B24 | TLB/ordering/multicore update tests; independent performance evidence | R09,R10,R12 |
| B31 | Accessibility: native assistive consumer | B08,B09,B16,audio as needed | Keyboard/semantic actions, announced focus/value, no screen-reader overclaim | R15 |
| B32 | Quality: benchmark/fault test harness | B04; grows with gates | Repeatable workloads, trace metadata, deterministic faults | R09,R12,R17 |
| B33 | Release: rescue and offline generation boot | B23,B24,B25,B40 | Last good selection, failed boot limits, schema-aware recovery | R02,R12 |
| B34 | Release: deterministic packaging and provenance | B02,B23,B33 | Two clean builds compared, SBOM/notices, symbols/checksums | R02,R16 |
| B35 | Product: experimental release review | B21,B25,B26,B32,B34 | AI-free core requirements/limitations and evidence complete | R01–R19 |
| B36 | UI: Slint/custom comparison | B02 | Comparable scenes/metrics; guest compile/reload/text/semantics/license record; confirm at M4 | R04,R08,R09,R15 |
| B37 | SDK: typed actions and scoped context | B08,B13 | UI/automation parity; rights/revision/expiry/dedupe/reload tests | R10,R14,R19 |
| B38 | Boot: compare loader and ISO paths | B01,B02 | Viable pinned loader and actual ISO recipe; image types distinguished | R01,R02,R16 |
| B39 | Optional AI: small-model CPU service | B06,B21,B37 | Supported task, scoped proposals, quota/crash/unload tests and memory accounting | Optional; R09,R10,R19 if enabled |
| B40 | Recovery: boot attempts and known-good health | B23,B24 | Durable retry/health/fallback tests; current data preserved | R02,R11,R12 |

## Task working format

Each implementation issue records the relevant ADR/spec, exact inputs, output paths,
bounded scope, negative cases, evidence needed, and which contract/version it changes.
Write a small vertical slice before broad abstractions. Do not implement B27–B31
merely because they are interesting if earlier gates remain broken.
B39 is optional after the first state-preserving guest demo; no core-release task
depends on inference. B36 is a bounded comparison, not two complete toolkits.

For B28 the dependency phrase means the networking driver's equivalent DMA/queue
mechanism must exist; it does not require adopting block-specific code. Replace
the phrase with real component dependencies when implementation layout is known.

## First coding session

Re-run B01/B02 using the [Phase 0 commands](phase-0.md), then start B03/B38's actual
ISO path in M1. Report exact missing tools before installation/system changes.
Hosted UI feasibility can continue if VM setup is externally blocked; begin with
B36 rather than claiming B07 complete. Preserve the objective; a host-only
demo is not an OS. B38 resolves packaging before B03 claims ISO success.
