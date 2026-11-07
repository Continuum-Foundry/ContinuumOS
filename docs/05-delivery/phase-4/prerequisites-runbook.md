# P4-01 prerequisite and desktop planning gate

Status: Done for the [reviewed P4-01 subset](../../../evidence/phase-4/P4-01/final-1794022944926085700/README.md). This is a prerequisite/profile experiment, not a guest
desktop, driver, new SDK freeze or performance result. Read the
[story](01-prerequisites-profiles-and-contract-plan.md) and
[strict M3 review](../../../evidence/phase-3/P3-14/2026-11-06/README.md).

## Source and execution discipline

The complete implemented gate is:

```powershell
pwsh -NoProfile -File tools/Test-P4-01.ps1
python tools/p4_acceptance.py review --acceptance <fresh-P4-01-acceptance.json>
```

It independently validates [fixed host profiles](../../../config/m4-profiles.json)
before running complete fresh M3/M1, host plan/lifecycle checks with that run's
actual Phase 0 image, then affected hosted M2. M3/M2 have separate 10800-second
outer bounds. The explicitly owner-authorized [revision-2 stress profile](../phase-3/stress-deadline-profile-v2.md)
uses 60000 ms only for the two long M3 stress flavors; ordinary/SDK/public-supervision
guests remain 30000 ms and all guest-local/negative limits stay unchanged. Private kill-on-close jobs
own launched stage descendants and are closed/reaped on failures. No stage skip,
historical reuse or automatic retry is provided. The P4 reviewer reopens current
source locks, M3 raw/artifact reviews, host child completion/exit/timeout records,
device-help hashes and M2's sixteen-row evidence index. Full output stays under
`build/m4/P4-01/`. Failed/interrupted runs cannot pass review.

Focused host-plan reproduction uses an actual Phase 0 user ELF and its exact hash:

```powershell
. ./tools/Enter-Dev.ps1
python tools/m4_prerequisites.py validate
python tools/Test-P4-01.py --image <userland-link.elf> --sha256 <exact-sha256> --stress-run <fresh-stress-artifact-directory>
python tools/m4_prerequisites.py devices --output <new-local-output-directory>
```

The host plan is `host-plan-passed`, not full P4 acceptance. It checks every
profile leaf mutation, unknown/missing fields, duplicate/malformed/oversized JSON,
literal independent byte fixtures and deterministic malformed packets. Real
fixed Python child cases cover success, nonzero exit, missing completion and
timeout; missing/wrong tools, malformed/missing/changed artifacts and invalid
profiles reject before launch. The existing M3 lifecycle helper retains 1000-ms
normal and 100-ms negative limits, null stdin, bounded output, fixed interpreter
and exact completion identity. It executes no user ELF and spawns no descendants.
The independent deadline consumer also rejects ten mutations of real stress
launch metadata: wrong bounds, profile name/hash, source hash, original deadline
or build/package/input flavor, including forged stress metadata on an ordinary
guest. These are consumer fault fixtures; they are not additional VM runs.

Planned device deltas explicitly select standard VGA with requested dimensions,
modern PCI VirtIO keyboard/tablet and, only when enabled, a two-port-capacity
VirtIO serial controller with exactly one named port at index 1 and loopback-only
socket backend. Dev-disabled deltas contain no controller/port/chardev/mutation
grant. This validates the host plan, not guest device discovery or enforcement.

Start from safely updated `main`; preserve unrelated edits. Record the source
commit, binary dirty patch and hashes of tracked and untracked implementation
inputs, including the owner's Windows signing edits. Never run policy installers
as part of this gate. Use the existing enrolled signer/xorriso execution context,
locked/offline Cargo and workspace-local tools. Keep the host awake; do not change
power or security settings. Run acceptance serially, retain failed/interrupted
attempts, and keep all bounds fixed except the explicitly named revision-2 long-stress profile.

The first baseline command already exists:

```powershell
pwsh -NoProfile -File tools/Test-P3-14.ps1
python tools/p3_acceptance.py review --acceptance <fresh-local-acceptance.json>
```

It includes ten fresh Phase 0/signing/compiler/doctor prerequisites, complete M1
(33 rows/42 optical launches), current M3 guest/SDK/portable regressions, separate
build flavors, bounded host failures and the strict aggregate negative probes.
No historical prerequisite/regression reuse is permitted. The aggregate's outer
10800-second bound does not itself extend any guest deadline. The named stress
profile separately admits 60000 ms for its two flavors; ordinary guests stay
30000 ms. Preserve the 1000-ms
normal host fixture or deliberately shorter failure deadlines. Host preflight
timeout, guest timeout, expected rejection and interrupted run are distinct.

The affected hosted baseline also has an existing command:

```powershell
pwsh -NoProfile -File tools/Test-P2-13.ps1
```

Run it after the M3 command, never concurrently. This includes selected portable
linking, text/action/corpus/semantic/damage/reload/native tests. Its result cannot
accept guest rendering. Historical local-only logs are availability findings,
not fresh results. Full logs/source patches/binaries stay in ignored `build/`
and `work/`; only a reviewed compact bundle is eligible for retention.

## Current source audit and hook ownership

The original [hook inventory](../../../experiments/ui-link/hooks.json) describes
compile probes at P2-07 time; its `implemented_in_kernel: false` fields are
historical, not an audit of the current M3 implementation. The following map
separates implemented foundations from the missing selected-UI integration.

| Hook / source | Current implementation and limitation | Owning story and oracle |
| --- | --- | --- |
| Allocation: `sdk/continuum-rt/src/allocator.rs`, `kernel/src/user_entry.rs` | SDK allocator reuses a 64 KiB backing object, 64-byte cells, 32 live records, alignment at most 4096. Lower-level allocation has 32 slots and a 32 MiB heap ceiling. Neither is the hosted 64 MiB bump arena or a UI-sized reclaiming allocator. | P4-02: fragmentation/alignment/realloc/OOM/zero-reuse workloads and exact guest inventory settlement |
| Entry/image: `kernel/src/user_image.rs`, `owned_memory.rs`, `runtime_memory.rs` | Static ELF admission is at most 1 MiB and eight load segments; process commitment 64 MiB, 64 KiB user stack, two precharged table banks. Guest adapter is `Runtime<16,256,128,64>`: spaces, allocation records, mappings, pages per table bank. Selected fonts and renderer cannot be assumed to fit the small SDK image. | P4-02/P4-07: independent ELF audits and real font/image/backing admission, partial-launch unwind |
| Objects/IPC: `kernel/src/objects.rs`, `objects/channels.rs`, `wait.rs` | 16 owners, 128 objects, 32 handles and public mappings per owner; 32 MiB created/mapped-byte ceilings. Eight channel pairs with eight messages/16 KiB per direction; wait set has eight items. Planned larger M3 limits are not admitted capacities. | P4-02/P4-05/P4-12: pressure, immutable transfer, backpressure, cancellation and exact reference settlements |
| Supervision: `kernel/src/supervisor.rs`, `supervisor_guest.rs` | Eight prepaid epoch units including control; at most seven child admissions, 1 MiB backing/unit, four lifetime messages and timers/unit, two bounded retries, fixture call bound 2048. Physical reclaim does not refund launch/message credits. This is an ephemeral test service graph. | P4-02: finite ordinary desktop graph/restarts/control reserve, long-lived message admission, no hidden helper launches |
| Trusted identity/policy: `kernel/src/policy.rs`, `routes.rs`, public SDK | Trusted caller/session and scoped use-time checks exist for the admitted synthetic ephemeral protocol. They supply no real login, durable policy or development authentication. Ordinary startup must not install an acceptance observer. | P4-02/P4-10/P4-13: trusted bootstrap grants, protected-focus denials and dev-to-admin escalation rejection |
| CPU/math: `kernel/src/scheduling.rs`, `user_boundary.rs`, `sdk/continuum-rt` | One execution thread/process, eager private x87/SSE state, masked SSE exceptions. AVX/XSAVE/SMP, native unmasked #XM and x87 debug metadata are not accepted. Kernel excludes FP. | P4-02/P4-07: exact ELF/disassembly/CPU baseline and preempted renderer/text regressions |
| Fonts/text: `crates/text-core`, `fixtures/m2/text-v1`, `experiments/ui-link` | Hosted allowlisted immutable fonts/shaping/raster/cache pipeline and compile-link embeddings exist. M3 executes nonrendering model/grapheme/libm work only. No guest font package/lease/raster path. | P4-07: independent text/pixel corpus, immutable font lifetime, parser/OOM isolation, no host fallback |
| Clock/event loop: `kernel/src/timer.rs`, `wait.rs`, `crates/custom-ui` | Elapsed HPET nanoseconds and bounded waits exist; delivered-interrupt uptime can lag. Selected Controller receives explicit timestamps; there is no integrated guest UI wake/pacing loop. | P4-02/P4-07/P4-17: monotonic clock, bounded callbacks, idle/caret/clock accounting and instrumentation pairing |
| Physical input/actions: `crates/editor-model`, `crates/custom-ui` | Hosted typed actions, ordering, cancellation, focus/capture and composition policies exist. No hardware keyboard/pointer driver or guest dispatcher. | P4-03/P4-06/P4-08/P4-09: physical injection, reset/overflow, stale geometry, authority and logical checkpoint oracle |
| Presentation: `kernel/src/boot.rs`, `crates/custom-ui/src/retained.rs` | Boot framebuffer descriptors are checked and reserved; selected renderer draws owned RAM pixels. No app surface broker, display mapping, compositor or actual guest presentation is accepted. | P4-03/P4-04/P4-05/P4-07: range/cache/format negatives, independent guest pixel capture, lease ownership, full-versus-damage parity |
| Semantics/output: selected Scene/Controller and draft UI contract | Hosted stable IDs/semantic bounds/actions exist. No guest semantic exporter or frozen output/scale codec. | P4-09/P4-11: independent semantic/action and paint/input transform agreement, stale output-generation rejection |
| Failure: SDK panic/exit, supervisor and fixture hooks | Isolated failure/reap exists under accepted test profiles; a panic=abort or probe spin is not desktop recovery. | P4-02/P4-16: ordinary control progress, bounded restart/quarantine and update-fault continuity |
| External services: clipboard/IME/files/network/entropy/assistive consumers | No selected guest backend. Unsupported requests must fail explicitly; no std/host fallback. | P4-08/P4-10/P4-13 define minimal rejection/privacy/entropy requirements; M6/M7/M9/M13 own durable files, accounts, exchange and native assistive services |
| Dev bytes/session/publication | Hosted resource loader and M3 synthetic routes are groundwork only; no actual dev device, authentication, guest blob validation or publication. | P4-12 through P4-16: transport fault matrix, reviewed auth, bounded staging, real visible L0/L1 and unchanged process/boot identity |

Slint remains frozen comparison material, not the selected guest path. Its platform,
timers, event-loop/proxy, single-thread globals, native-service defaults and AOT
feature split in the original hook inventory stay unsupported guest obligations
if a future selection review reopens it. P4-07/P4-19 must explicitly review such a
change; no Slint defaults count as implemented services.

## Proposed service-budget handoff to P4-02

These are experiment ceilings to validate, not implemented capacity increases:
seven steady roles (supervisor, compositor/display, input, shell, editor,
update/staging, policy/session), at most 16 simultaneous processes, one execution
thread each. Admit 24 prepaid lifetime launch units per desktop epoch, with at
most two retries for each of six children (19 units including initial control).
Do not refund credits after reap, and do not spawn a process for L0/L1 reload.

Proposed per-role heap caps in MiB are respectively 2, 48, 2, 16, 64, 32 and 4.
Their sum is 168 MiB of *ceilings*, not allocated cold usage. Reserve 16 MiB for
control/recovery separately from staging. Propose at most 128 MiB commitment per
process and 384 MiB combined active/staging commitment, with a separate unchanged
256 MiB whole-guest cold-idle target. Shared backing is charged once globally and
to each mapping authority for admission. Count kernel/static tables, stacks,
images/BSS, font backing, display/front/back buffers, old/new resources, queues,
parser scratch and caches; no category disappears because it is statically linked.
P4-02 must reconcile these ceilings with actual images and zero-on-reuse accounting
before accepting any capacity change. P4-14 owns old/new staging headroom admission.

## Device experiment and trust plan

Use the pinned Q35/TCG/OVMF baseline with one vCPU and 2 GiB capacity. The existing
launcher supplies standard VGA (`-vga std`), a boot framebuffer and separate
file-backed diagnostic serial. It disables guest networking, monitor and parallel
port, omits GDB/QMP, uses fresh firmware vars and a read-only standalone ISO.
Current boot metadata proves neither requested 720p/1080p mode availability nor
presentation: P4-04 must request and verify exact modes/stride/masks and capture
the actual guest artifact. Firmware fallback to another mode must fail that cell.

Candidate physical input is modern PCI VirtIO keyboard plus absolute tablet;
candidate dev bytes use modern PCI VirtIO serial with one named port. Fresh
bounded capability help from the hash-verified pinned executable confirms
`virtio-keyboard-pci`, `virtio-tablet-pci`, `virtio-serial-pci` and `virtserialport`.
The PCI device help exposes `disable-legacy`; VGA help exposes `xres`, `yres`
and `vgamem_mb`. Full help stays in `work/p4-01-device-inventory/`. P4-03
still has to discover actual PCI IDs/BARs/capabilities/IRQ resources in the guest,
negotiate VERSION_1 with a declared feature allowlist and prove queues/reset.
Device names in QEMU help alone do not pass those experiments.

The dev-enabled host backend is a loopback-only socket chardev, dynamically
allocated local port, no guest NIC or host filesystem share. P4-12 must bind/check
the endpoint and record exact args; P4-13 authenticates peers independently of
loopback location. No guest-controlled host command or pathname is permitted.
Dev-disabled launch omits the serial device/port/chardev entirely and bootstrap
grants no dev mutation authority. Diagnostic COM1 remains a separate output-only
recovery route; it never carries resource updates or typed user input.

P4-03 initially owns narrowly documented kernel mechanisms for PCI discovery,
MMIO/port grant checks, IRQ mask/ack and DMA ordering. User processes own toolkit,
shell, parsers and publication. Device/driver code and QEMU are trusted for DMA;
there is no IOMMU confinement claim. Private CPU page mappings do not constrain
bus mastering. Pin/charge buffers until stop/drain/reset completes; on timeout,
retain a bounded quarantine and deny further admission. Extraction into isolated
drivers requires later explicit experiments rather than an assumed safe port.

Required negative oracles are: forged/stale/overlapping grants; unknown mandatory
features; malformed/looping/out-of-range descriptor chains; capacity exhaustion;
lost/duplicate IRQ; device death/reset under load; absent/unexpected dev port;
partial/oversize/reordered frames; replay/tamper/expired authority; injected input
into protected focus; parser death/OOM; stale lease/output/resource generations.
P4-03/04/06/12/13 own actual device tests. P4-01 validates the profile and draft
boundary rejection rules only. No driver or cryptographic construction is accepted.

## Clocks and measurement interpretation

Guest durations use elapsed HPET nanoseconds from the accepted M3 clock service.
Record clock resolution, monotonicity and observer cost. Host durations use
`perf_counter_ns`; never subtract guest timestamps from host timestamps. Record
host completed-build/send/commit-observation timestamps in the same host clock
domain; record guest input-arrival/present-submission in the guest domain. A
matching visible-frame observation is required for host-to-visible latency;
an ACK alone proves no pixels. Render+composition CPU percentiles need a validated
per-owner CPU clock excluding preempted time; until available, elapsed and batched
CPU observations remain separate and CPU-target compliance is unresolved.

Keep 720p and 1080p, 100/150/200 percent scales, rich/reduced effects, all text
fixtures and six hosted workload families. Warm up 50 frames plus five seconds;
measure five independent repetitions of 200 frames per cell (1000 samples),
trace off/on paired and reported separately. Use nearest-rank quantiles and keep
raw distributions, missed frame periods and instrumentation overhead. Warm up
five reloads then measure 100 compatible L0 and 100 L1 changes per baseline;
no hidden restarts. Idle is 30 seconds: ten static, ten clock/caret, five hidden,
five minimized, with 100-ms observer samples. Static/hidden/minimized paints are
zero; clock/caret deadlines stay counted. Guest frame/input tests and reload/stress
have separate future outer run bounds. The inherited 30000-ms ordinary guest
bound remains; only the separately versioned long M3 stress profile uses 60000 ms.

Settle for 64 five-ms quanta (320 ms), requiring exact zero unexplained retained
frames/objects/handles/maps/leases/roots/launches and zero unclassified byte growth.
Explicitly bounded warmed caches may plateau at their declared caps; classify
their initial growth separately. Fairness/control wake and cleanup deadline is
320 ms; CPU clock validation and serial independent watchdog are required before
measuring it. Individual resource transaction deadline is 5000 ms, with unchanged
sub-500-ms completed-build-to-visible *target*. Faults include every applicable
transition, five repetitions each, deterministic seed 20261106. Performance misses
remain findings with follow-up owners; correctness/boundedness misses block handoff.

The [initial candidate](profiles-draft.json) remains historical planning. The
[fixed experiment profile](../../../config/m4-profiles.json) now has producer and
independent semantic validators plus an exact canonical SHA-256 lock
`333b7fa2170877508d737d9f0fb97fe190a082fc5562fea6f016484798704f1a`.
This locks host experiments, not implemented guest capacities or new ABI IDs.
Proposed runtime ceilings remain explicitly unimplemented until P4-02 proves
admission/reclamation. The [literal codec fixtures](../../../fixtures/m4/desktop-v0/fixtures.json)
exercise the [draft boundary](../../06-contracts/desktop-v0.md) through independent
byte-offset consumers; later stories still own real service/client exchanges.
