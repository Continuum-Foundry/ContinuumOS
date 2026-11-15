# P4-06 input runbook

Status: [passed local selected-story self-review](../../../evidence/phase-4/P4-06/final-1794706382942464300/README.md), 2026-11-14.
No M4 exit, external certification or desktop efficiency acceptance.

The selected platform remains one-vCPU Q35/TCG, pinned workspace tools and an
independently packaged ISO. `input-test` attaches modern VirtIO keyboard at PCI
slot 4 and absolute tablet at slot 5. No dev-transport device is attached. The
existing temporary kernel mediator owns supervisor UC MMIO mappings, IRQ/INTx,
fixed descriptors and exclusive DMA backing. Devices receive no app pointers.
Policy, US ASCII mapping, physical key/modifier state, geometry hit testing,
focus/capture and bounded per-surface event queues execute in the compositor's
CPL3 process. This combined broker is an experimental extraction boundary; the
separate input role currently supplies ordinary lifecycle/control workload.

## Commands and current limits

Dot-source `tools/Enter-Dev.ps1`; use the approved Windows signer/xorriso context.
Keep the host awake; do not change power or security settings. Preview first:

```powershell
python tools/p4_input_acceptance.py plan
pwsh -File tools/Test-P4-06.ps1 -Plan
pwsh -File tools/Test-P4-06.ps1
```

The complete selected story plan has 24 command groups, 17 VM launches and 22
separate metadata builds, with no full suites. `-Plan` and `-Probe` are also
available through `tools/Test-P4-06.ps1`; a development probe does not close the story.
The 30000-ms guest deadline, inherited lifecycle counts and 320-ms local bounds
are unchanged. Development probes do not run all selected story regressions and
cannot close P4-06. Preserve failed/interrupted journals and exact ISO/ELF bytes.
Full payload stays local under `build/m4/P4-06/` and `build/m1/input-*/`.
No deletion is authorized. Schema-2 storage/capsule limits are declared in
`config/p4-06-validation.json`; initial storage inventory is local at
`work/p4-06/storage-inventory.json`.

The driver negotiates VERSION_1 only, eight writable eight-byte event buffers
per device and an empty status queue. Packed/indirect/event-index descriptors,
LED updates, wheel/relative-pointer devices, hot-plug, compose/IME and persistent
regional settings are absent. Initial printable mapping is US ASCII letters,
digits, punctuation, space, Tab and Enter; Ctrl/Alt suppress text. Physical key
codes remain distinct from text; unsupported device events cancel rather than
becoming arbitrary widget actions. Protected login authority is P4-10's gate.

## Experimental bytes and authority

Internal operation 0x4300 copies one 32-byte raw event to the trusted compositor:
sequence:u64, guest elapsed ns:u64, device index:u64, followed by explicit VirtIO
kind:u16/code:u16/value:i32 little-endian bytes. Operation 0x4301 copies a trusted
32-byte peer identity followed by a 96-byte CIN0 event into a one-slot app inbox.
Only the compositor can publish. Operation 0x4302 receives through the current
trusted shell/editor process/session binding, consumes after successful checked
copy, and exposes no source pointer. Operation 0x4303 is compositor-only device
reset management; DMA release requires actual reset completion. These operations
are experimental and do not extend the frozen SDK. The source codecs define the
current byte offsets; no native Rust layout is serialized.

Compositor surface snapshots supply ownership, visibility, clip, z-order and
geometry revision. Payload-provided process/widget IDs cannot select a physical
input recipient. The initial output is the accepted 720p pixel profile; broader
logical scale/paint agreement remains P4-11. Portable tests of rational coordinate
conversion do not prove scaled rendering. No toolkit, editing or reload is
accepted by this input result.

## Deterministic and manual replay

The host observer waits for explicit synthetic `input-inject-N` checkpoints and
uses QMP `input-send-event`, entering the actual VirtIO HID path. It presses both
Shift keys, releases the left Shift, types A, releases the remaining keys, then
presses inside the second surface and drags/releases outside under capture.
Clients independently decode explicit event offsets and report assertion
checkpoints. The independent reviewer reopens complete raw guest records and QMP
requests/acknowledgements. Direct model calls remain separate host tests.

For manual replay, launch the same retained ISO
with the recorded device/firmware/CPU arguments and a visible QEMU display,
keeping its serial diagnostics independent. At each checkpoint supply only the
requested synthetic key or pointer event; compare physical/event ordering and
focus/capture/cancel outcomes to the journal. Do not use passwords or personal
text. This manual procedure is a development aid, not an acceptance substitute.

The input fixture suppresses redundant wait-call records while retaining every
actual wait resume/latency, runtime bound, physical input event, fault, lifecycle
and settlement. Its invalid compositor-copy destination probe stops after one
actual denied copy; ordinary surface fixtures are unchanged. The earlier timeout
retains 612 wait-call records and 172 transport denials and remains a non-pass.
This diagnostic reduction changes no guest deadline, lifecycle/heap count or
ready/cleanup bound.

## Current extended fixture and recovery boundary

The final native fixture confirms 104 raw samples (82 keyboard, 22 tablet), all
nine entered owner runs/900 heap cycles, captured secondary-button down outside
the window, real captured-owner exit, stale-release suppression, 33-report flood,
deliberately unreleased physical Shift at device reset and four complete 720p
frames. Execution completes in 16741 ms host elapsed under the unchanged 30000-ms
guest bound. Independent review distinguishes 41 publications from 38 app receipts,
including 8 key, 2 text and 5 button events. Overflow explicitly discards 32 broker
events; a pending one-slot kernel inbox is independently superseded by cancellation.
Twenty-three missing-criterion mutations reject. The ordinary ISO separately
passes real-device readiness without synthetic policy or key/text logs.
A source report is committed only on SYN_REPORT. Complete visible completion
batches reject duplicate heads, malformed lengths and jumps before reuse. Failed
user copies consume no descriptor. The private per-surface queue is 32 events plus
a reserved cancellation slot. Cancellation reports the discarded model queue
count; a copied inbox can independently contain one superseded event. Kernel
publication and actual successful app receipt are recorded separately in test
builds, so overwriting an unread event cannot masquerade as consumption.

Raw timestamps describe guest acquisition, not the instant a physical key moved.
Frame start/end and acquisition use the same elapsed clock for overlap review.
Every 16-row cooperation point samples bounded input before yielding. These are
correctness/progress observations, not input latency, VSync or CPU-target passes.

Unconfirmed held keys/buttons and incomplete packets expire after two seconds;
other device activity cannot renew their deadlines. This is a conservative
experimental gesture limit, not an unlimited held-key/drag guarantee. Invalid raw
streams and fresh broker/old device sequence mismatches cause explicit reset and
resynchronization. Driver reset must complete before backing is reclaimed;
timeout retains/quarantines backing and fails the owning service's bounded path.
The combined compositor/input broker must be rebound after its process dies.

Upstream transport loss is distinct from the acquired OS queue. The
[VirtIO 1.2 input specification](https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html)
allows complete reports to be dropped under buffer shortage. The inspected
[upstream QEMU sender](https://github.com/qemu/qemu/blob/master/hw/input/virtio-input.c)
also drops a report when descriptors are unavailable. That moving source is not
the pinned binary or a build input; the abbreviated pinned source URL was not
available. The controlled native fixture waits for acquired report checkpoints
and independently counts every injected event. It does not establish unlimited
upstream event retention or protection against a malicious device. Missing-release
cancellation, OS queue overflow and device teardown are separate checked outcomes.

The ordinary ISO now builds `input-runtime` and attaches the two scoped devices.
It exposes no development transport. Test-only key/text publication/receipt logs
are absent in ordinary builds. The 64 KiB of DMA backing is explicitly admitted in
addition to process backing, without spending the existing 16 MiB recovery reserve.
A completed input service teardown must settle both device backing and the legacy
process/object/frame inventories. No new public SDK family is frozen.

## CIN0 bytes and unsafe ownership review

All fields are little-endian. CIN0 magic occupies 0–3, version u16 at 4 is zero,
length u16 at 6 is 96. Seven u64 fields at 8,16,24,32,40,48,56 carry sequence,
guest-acquisition ns, input epoch, surface, input-scene, output and focus epoch. Kind/code u16 occupy 64/66; value/x/y i32 occupy 68/72/76;
modifiers and Unicode scalar u32 occupy 80/84; 88–95 must be zero. The explicit
source codec and literal byte tests are authoritative. Cancel is a reserved event
slot with reason in code and discarded broker count in value; consumers clear
all held state and reject stale activation. The broader proposed contract remains
unfrozen.

DMA uses two exclusive 32 KiB supervisor-owned runs. Descriptor addresses never
come from apps. Full visible batches validate lengths, unique heads and ownership
before republishing. Checked user-copy failure consumes nothing. Acquire/release
fences order ring visibility; actual device reset completes before release, and
timeout fails with backing retained/quarantined. Fixed ring metadata is single
CPU/syscall-owned; IRQ only acknowledges and increments atomic counters. CPL3
broker BSS UnsafeCell and its fallible queues have one owning execution thread,
no signal/callback reentry and no borrow across user execution. This assumption
must be revisited for threads/SMP. Kernel inboxes copy bytes and retain no app
pointer. Successful final reset verifies zero DMA retention/grants and legacy
frame/object/process inventories; this is not IOMMU confinement.

The exact selected impact, omissions, failures, source reconstruction and verified
capsule are linked from the acceptance review. Full original ISO/ELF payload stays
local for active exact-artifact review/resume. Owner defers deletion. Final prose
and index changes rerun host audits; unchanged guest artifacts are re-reviewed.
