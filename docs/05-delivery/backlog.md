# Ordered engineering backlog

Status: B01/B02 have [Phase 0 evidence](phase-0.md). P1-02 demonstrates B03/B38's
minimal Limine UEFI optical route. P1-03 completes B04's checked boot metadata,
fixed diagnostics and correlated identities on that profile; see its
[runbook](phase-1/boot-metadata-runbook.md) and
[acceptance review](../../evidence/phase-1/P1-03/2026-10-18/README.md).
Later runtime gates remain open. B36 has a P2-09 provisional custom decision;
M2 completion and M4 guest confirmation remain open.
P1-04 implements B05's fatal exception subset; see its
[runbook and acceptance status](phase-1/exception-diagnostics-runbook.md).
P1-05 implements frame ownership, page tables, guards and actual double-fault
integration; its [accepted memory review](../../evidence/phase-1/P1-05/2026-10-19/README.md)
closes B05's planned memory/exception gate on this profile. P1-06's [accepted timer baseline](../../evidence/phase-1/P1-06/2026-10-20/README.md) implements only B06's M1 interrupt/uptime subset. Preemption, task/FP context and scheduling remain M3; B06 is not Done.
P1-07 implements the aggregate M1/early-B32 harness and final readiness; its
[complete local review](../../evidence/phase-1/P1-07/2026-10-20-acceptance/README.md)
passes 33 rows and 42 launches, closing the declared experimental M1 gate.
B32/B06 remain partial. The general heap and user isolation remain later work.
Owner column means a responsibility, not an assigned
agent/person. Read milestone gates before picking work. `→` dependencies must pass;
tasks within a gate can share infrastructure without implying simultaneous agents.

M1 is decomposed into [seven local Phase 1 stories](phase-1/README.md), retaining
the Bxx identifiers below. The stories expose B03/B04's minimal serial bootstrap
and separate B06's M1 timer baseline from its later scheduling/context work.

M2 is decomposed into [thirteen local Phase 2 stories](phase-2/README.md).
P2-01's [local acceptance](../../evidence/phase-2/P2-01/2026-10-20/README.md) rechecks
B01/B02 and adds fixed profiles/bounded headless lifecycle infrastructure for B32;
P2-02's [local corpus acceptance](../../evidence/phase-2/P2-02/2026-10-21/README.md)
adds B07/B08 boundary design and B36 shared fixture inputs, without implementing
either candidate. P2-03's [accepted document/actions model](../../evidence/phase-2/P2-03/2026-10-21/README.md)
implements B09's logical state subset and M2 design/model of B37, with shared
entry adapters, scoped contexts, bounded retries and explicit checkpoints.
P2-04's [accepted text experiment](../../evidence/phase-2/P2-04/2026-10-21/README.md)
adds B09/B36's pinned fonts, shared text/composition corpus and bounded custom pipeline;
P2-05's [accepted custom candidate](../../evidence/phase-2/P2-05/2026-10-21/README.md)
adds B07/B08/B36's bounded custom layout/paint/input/semantic integration and host
instrumentation. Full redraw/copies and immediate standard transitions remain
comparison findings. P2-06
[adds Slint candidate evidence](../../evidence/phase-2/P2-06/2026-10-21/README.md)
for B07/B08/B36 with shared text/input/semantic adapters and explicit native
text/effect/clip/blend and distribution-term gaps.
P2-07/P2-08 now supply accepted final-link feasibility and hosted comparison.
P2-09 records the [provisional custom Rust direction](phase-2/ui-selection-review.md),
terms/freeze and remaining selected-core work. P2-10 now passes
[selected hosted-core acceptance](../../evidence/phase-2/P2-10/2026-10-23/README.md). P2-11 now passes [hosted resource reload acceptance](../../evidence/phase-2/P2-11/2026-10-23/README.md), completing B10's bounded continuity/publication slice. P2-12–P2-13 remain Not started; guest execution and efficiency/stability acceptance stay open.
P2-02 through P2-09 cover B36 and initial
B07/B08/B09/B37 boundaries, candidates, target links and decision. P2-10 completes
the hosted portions of B07/B08/B09; P2-11 covers B10, and P2-12/P2-13 cover M2's
B32 measurements/acceptance. B37 remains interface/model work until its B13 and
replacement dependencies pass; B07/B36 still need M4 guest confirmation. This
decomposition does not change the Bxx dependencies or mark later requirements Done.

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
| B11 | Kernel: static ELF user loader and syscalls | B05,B06 | Validated user entry and malformed ELF tests; trusted user/service principal, session and process binding; child launch cannot forge identity or exceed granted authority | R03,R10,R14 |
| B12 | Kernel: rights-bearing handles and IPC | B11 | Bounded channels, transfer/error/wait fixtures; trusted caller provenance, forged owner/role rejection, cross-owner denial, explicit sharing, same-user app least authority and unauthorized disk/account-operation rejection | R10,R14 |
| B13 | Services: supervisor, quotas, authority policy and routes | B12 | Crash containment, epochs, bounded queues, stale writer tests; scoped role-to-capability policy, owner checks, installer/account-administration grant ceilings, brokered invalidation and fail-closed restart; no universal direct-handle revocation claim | R06,R10,R14 |
| B14 | Platform: dev transport adapter | B03,B12 | Bounded framed byte stream with actual host/guest transfer | R13,R17 |
| B15 | Security: session authority and plan validation | B13,B14 | Replay, bad credential, permissions and malformed-plan tests; bind dev credentials to M3 caller/session context, reject role/owner spoofing, keep dev authority separate from desktop account administration | R10,R13,R14 |
| B16 | Graphics: guest surfaces/compositor/input | B06,B08,B12 | Hosted scenes appear in guest; focus/capture and buffer lifetime tests; trusted setup/login routing, restricted capture and sensitive-event exclusion from ordinary diagnostics, tested with synthetic input | R08,R09,R10,R15 |
| B17 | Updates: guest L0/L1 publish | B10,B15,B16 | Visible no-reboot asset/layout update and rejection evidence | R04,R13,R17 |
| B18 | State: checkpoint codecs/migration fixtures | B09,B13 | Identity/adjacent migration, resource inventory, corrupt/oversize failures; exclude credentials and privileged installer handles from ordinary app checkpoints | R07,R14 |
| B19 | Updates: candidate prepare and quiescence | B15,B18 | Restricted startup, bounded drain/cancel, safe abort; reject replacement while an active installation writer/flush owns its transaction, with modeled M5 coverage | R05,R06,R12 |
| B20 | Updates: activation/fencing/retirement | B19 | One writer, coherent route root, no reachable old code freed | R05,R06,R12 |
| B21 | QA: first product reload demo | B17,B20,B37 | UI+behavior retain text/selection/window; same boot ID; 100 cycles | R04,R05,R07,R09,R19 |
| B22 | Storage: block/DMA/flush implementation | B05,B12 | Descriptor/reset/flush tests; documented device durability contract; bounded geometry and explicit target identity for non-writing VM installation plans | R10,R11 |
| B23 | Storage: record store/packages/generations | B22 | Torn-write recovery, hash validation and bounded GC; select and experimentally validate boot/system/data layout, firmware discovery and complete offline payload contract; review entropy/credential-storage feasibility without claiming account implementation | R11,R12,R16 |
| B24 | Updates: durable decision/recovery | B20,B23 | Faults before/after every record/fence/publish/ACK; recovery table passes, including empty/interrupted initial-generation activation and honest resume/rescue outcomes | R06,R11,R12 |
| B25 | Services: settings replacement under load | B24 | No lost accepted writes, dedupe and compatibility tests | R06,R07,R14 |
| B26 | Product: file browser/save/autosave | B09,B16,B23 | Honest save status, path authority, recovery of unsaved buffer | R08,R11,R15 |
| B27 | Runtime: bounded Wasm spike (M20) | B12,B18,B20 | Exact-target interpreter/import integration; fuel/memory/stack limits, cancellation, state exchange and native comparison | R05,R09,R10 |
| B28 | Networking: stack and TLS integration | B12,B22-style DMA support | Controlled packets, clocks/entropy/provider negative tests | R10,R13 |
| B29 | Graphics: VirtIO GPU feasibility (M18) | B16, profiling | Exact guest/host backend, resource/fence/reset tests, measured benefit and maintained software fallback | R08,R09 |
| B30 | Platform: SMP experiment (M19) | B06,B12,B24 | Multicore context/scheduling, TLB/ordering/concurrent update/recovery tests; measured workload comparison | R09,R10,R12 |
| B31 | Accessibility: native assistive/text integration (M13) | B08,B09,B16,B55 | Richer text, keyboard/semantic actions and announced focus/value through validated speech/audio; exact language/device support and failure limits | R08,R15 |
| B32 | Quality: benchmark/fault test harness | B04; grows with gates | Repeatable workloads, trace metadata, deterministic faults | R09,R12,R17 |
| B33 | Release: rescue and offline generation boot | B23,B24,B25,B40 | Last good selection, failed boot limits, schema-aware rescue primitives preserving current data; B44 separately accepts their installed-disk integration | R02,R12 |
| B34 | Release: deterministic packaging and provenance | B02,B23,B33 | Two clean builds compared, SBOM/notices, symbols/checksums; exact offline installation/boot payloads inventoried; packaged installer lifecycle accepted by B44 before M8 | R02,R16 |
| B35 | Product: experimental release review | B21,B25,B26,B32,B34,B44 | AI-free core requirements/limitations and evidence complete, including B41–B44 account setup, graphical installation, repair and preserving reinstall on the exact release ISO | R01–R19 |
| B36 | UI: Slint/custom comparison | B02 | Comparable scenes/metrics; guest compile/reload/text/semantics/license record; confirm at M4 | R04,R08,R09,R15 |
| B37 | SDK: typed actions and scoped context | B08,B13 | UI/automation parity; trusted principal/session provenance and provider-side scoped grants; cross-owner, revoked-role, rights/revision/expiry/dedupe/reload tests | R10,R14,R19 |
| B38 | Boot: compare loader and ISO paths | B01,B02 | Viable pinned loader and actual ISO recipe; image types distinguished | R01,R02,R16 |
| B39 | Optional AI: small-model CPU service (M21) | B06,B21,B37 | Isolated named model/backend, typed scoped proposals, quota/escalation/crash/unload tests, cold/warm/active memory and unaffected desktop/reload path | Optional; R09,R10,R19 if enabled |
| B40 | Recovery: boot attempts and known-good health | B23,B24 | Durable retry/health/fallback tests; current data preserved; distinguish an incomplete first initialization with no known-good generation from recoverable later activation | R02,R11,R12 |
| B41 | Security: local accounts, credentials, sessions and scoped RBAC | B02,B13,B16,B23,B24,B37 | Reviewed exact-target entropy/credential design; durable account lifecycle, authorized scoped roles, secure login/logout/lock and recovery; cross-user isolation and interrupted first-account setup tests | R10,R11,R14,R15,R19 |
| B42 | Installation: offline backend and installed boot | B22,B23,B24,B33,B38,B40 | Narrowly authorized service validates target-bound plans/confirmation; implements offline fresh install, boot/system repair and preserving reinstall through coherent generations; progress/cancellation/durable outcomes; independent installed VM boot | R02,R10,R11,R12,R14,R17 |
| B43 | Product: graphical installer and first-run setup | B08,B09,B16,B37,B41,B42 | Unprivileged keyboard-complete Live/Install/Rescue flow with fresh/repair/preserving-reinstall operations; target confirmation, progress/errors; initial owner setup before ordinary desktop access; no plaintext secrets in logs/checkpoints | R08,R10,R11,R14,R15,R17,R19 |
| B44 | QA: installed lifecycle acceptance | B26,B33,B40,B41,B42,B43 | Exact packaged ISO installs offline; ISO-detached/fresh-firmware boot, account setup, persistent files/ownership, boot/system repair, preserving reinstall and interruption/rescue matrix; sentinel disks unchanged | R02,R10,R11,R12,R15,R16,R17 |
| B45 | Files: complete operations and pickers (M9) | B12,B16,B23,B26,B37,B41 | Folder/create/copy/move/rename/delete/trash, conflict choices, bounded progress/cancel and grant-bearing open/save pickers; interrupted/denied operations tested | R08,R10,R11,R14,R15,R17,R19 |
| B46 | Desktop: exchange, associations and notifications (M9) | B08,B16,B37,B41,B45 | Cross-app scoped clipboard/drag offers, default apps/file associations and bounded keyboard-accessible notifications; denied/stale offers and missing apps tested | R08,R10,R14,R15,R19 |
| B47 | Settings: time, locale and keyboard (M9) | B06,B16,B23,B25 | Persistent date/time, time zone, locale and keymap; wall-clock adjustment cannot corrupt monotonic deadlines or silently rename files | R08,R11,R14,R15 |
| B48 | Apps: native lifecycle and SDK delivery (M10) | B13,B23,B37,B41,B46 | Independent SDK app install/launch/update/uninstall, reviewed permission increases and explicit app-data retention with cleanup/quota tests | R08,R10,R11,R14,R19 |
| B49 | Updates: verified user delivery and trust lifecycle (M10) | B20,B24,B34,B40,B48 | Verified offline app/system bundles, publisher/trust lifecycle, interrupted activation recovery and honest reboot-required outcomes; separate from dev authority | R06,R10,R12,R14,R16,R17 |
| B50 | Data: file/settings backup and restore (M11) | B24,B26,B41,B45,B47 | Independent destination, coherent versioned export and verified fresh-install restore with explicit ownership mapping; exclude secrets/privileged policy; corrupt/interrupted restore tests | R10,R11,R12,R14,R17 |
| B51 | Administration: resource and service management (M11) | B13,B25,B32,B41,B48 | Scoped app/service stop/restart, CPU/memory/disk status and bounded diagnostic export; hung app, unsafe restart and unauthorized controls tested | R09,R10,R14,R17 |
| B52 | Lifecycle: shutdown and session recovery (M11) | B18,B24,B40,B41,B48,B50 | Bounded save/checkpoint/flush shutdown, explicit force consequences, compatible authorized session restore and crash-loop suppression | R07,R11,R12,R14,R17 |
| B53 | Networking: general stack, time and trust (M12) | B28,B41,B47,B49 | Virtual Ethernet/addressing/DHCP/DNS/TCP/TLS with trusted time/store policy, bounded resources and malformed/packet-loss/certificate/entropy tests | R09,R10,R13,R14,R17 |
| B54 | Product: connections and online-update client (M12) | B49,B51,B53 | Connection settings and downloads through the existing verifier/activation path; disconnect/retry and invalid metadata cannot bypass authority or weaken verification | R08,R10,R12,R14,R17 |
| B55 | Audio: service, devices and capture permissions (M13) | B06,B12,B16,B41 | Bounded playback/capture/mixing, device/format negotiation, explicit permissions/indicators, revocation and underrun/reset/failure cleanup | R08,R09,R10,R14,R15 |
| B56 | Release: usable VM desktop acceptance (M14) | B35,B45,B46,B47,B48,B49,B50,B51,B52,B54,B55,B31 | Exact ISO/installed M8–M13 workflow, backup/network/audio/accessibility/fault/resource matrix and clean-build/publication review; optional research excluded | R01–R19 |
| B57 | Hardware: named-PC feasibility and drivers (M15) | B56,B38,B22,B16 | Select one named UEFI profile and required device matrix before implementation; actual boot/storage/input/software display/network/audio/power qualification | R02,R08,R09,R10,R11,R17 |
| B58 | Release: named-PC installed lifecycle (M15) | B57,B42,B43,B44,B50,B52,B54,B55 | Profile-specific install/boot/accounts, preserving repair/reinstall, independent backup/rescue and poweroff evidence; VM evidence cannot substitute | R02,R08,R10,R11,R12,R15,R16,R17 |
| B59 | Research: Windows compatibility and probes (M16) | B02,B11,B12,B13,B16,B23,B41 | Sourced native-service versus Wine adaptation study and bounded x64 console/DLL/runtime/GUI probes; costs, denial/cleanup and supported-subset or stop/replan outcome | R03,R09,R10,R14,R17 |
| B60 | Research: browser feasibility and route decision (M17) | B53,B59,B16,B09,B41 | Native-port versus Windows route, dependency/render/network/isolation/update/memory audit and bounded probe findings; explicit later decision, no sandbox bypass | R08,R09,R10,R14,R15,R17 |

## Task working format

The [M3 identity/authority plan](../02-architecture/identity-and-authorization.md#m3-work-items-and-acceptance)
expands B11–B13's local acceptance cases and the B15/B37 handoff. There is no
separate P3 story decomposition yet. Synthetic principals and ephemeral role
fixtures do not imply implemented accounts. Full accounts and operational RBAC
have a mandatory M7 gate after M6 in [milestones](milestones.md#accounts-sessions-and-rbac--required-for-m8).
B41 implements it; decompose that gate before starting it.

M6's [installer-preparation plan](installer-preparation.md) expands existing
B22–B24/B40 and the later B33/B34 handoff. It does not add P6 stories or require
a complete installer for M6. Include its preparation cases when M6 is decomposed;
decompose mandatory M7 B41–B44 before implementation. These four newly added broad
tasks brought the installer-era backlog to 44; existing IDs are preserved. B44 makes all four
transitive prerequisites of B35/M8, while B39 remains optional. Detailed P3–P8
stories are created only at the owner's milestone decomposition request.

The approved desktop/platform expansion adds B45–B60 (60 total broad tasks) and
M9–M21. See the [feature matrix](desktop-feature-matrix.md) for owner, classification
and future acceptance. B56 is the M14 release root and B58 the named-PC root;
B27/B29/B30/B39/B59/B60 are optional research tracks excluded from B35 and B56.
Networking B28/B53 and assistive/audio B31/B55 are required for M14, not M8.
Detailed P9–P21 stories also wait for the owner's request before each milestone.

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
