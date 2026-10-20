# Validation strategy and requirement traceability

Status: acceptance plan with [Phase 0 compile/tooling evidence](phase-0.md).
Host validator/doctor checks, clean freestanding links and dependency compilation
have run. P1-02 adds minimal original-kernel entry from the actual UEFI ISO, bounded
emulator exits and negative boot/cleanup checks; see its
[runbook](phase-1/minimal-iso-runbook.md) and [evidence](../../evidence/phase-1/P1-02/2026-10-18/README.md).
P1-03 adds checked owned descriptors, per-launch guest identity, resource
summaries and bounded rejection cases; see its
[runbook](phase-1/boot-metadata-runbook.md) and
[review](../../evidence/phase-1/P1-03/2026-10-18/README.md).
The corrected matrix passes 18 host tests and 17 VM launches: seven positive
metadata boots, eight expected metadata rejections and two negative ISO boots.
All processes are reaped. This remains a bounded subset of T01/T02/T12 supporting
R01/R03/R17/R18 and preparatory R14 discipline, not process ABI/security completion.
Every result must include source/build IDs, configuration and raw evidence.

P1-04 extends T01/T02 and R01/R03/R17 with the
[fatal exception matrix](phase-1/exception-diagnostics-runbook.md): actual invalid
opcode, non-present read, Rust panic and an invalid opcode with the logger gate
held. Exact symbolized contexts and exit 37 are required. P1-05 adds T03 memory
ownership, zeroing/reuse, checked mapping and actual guard/RO/NX/double-fault cases;
see its [runbook and acceptance status](phase-1/memory-ownership-runbook.md).
This extends R01/R03/R17/R18 and early R10 groundwork. Process containment,
interrupt/timekeeping and broader M1 acceptance remain later gates.

For M1's bounded subset, use the [Phase 1 story acceptance matrix](phase-1/07-phase-1-acceptance.md).
Its later guest gates remain unrun. [P1-01 host prerequisites](phase-1/prerequisites-runbook.md)
pass version/capability/hash checks and rejection tests; this adds no ISO boot result.

## Requirement matrix

| Requirement | Test families | Passing evidence |
| --- | --- | --- |
| R01 original OS | T01 boot/provenance | Original kernel and declared upstream boundaries |
| R02 ISO | T01, T12 release | Exact ISO cold boot offline without host shares |
| R03 language | T02 toolchain/ABI | Freestanding builds; audited assembly/first-party C inventory |
| R04 live resources | T05 UI, T07 reload | Scene updates, preserved work, unchanged kernel boot ID/uptime |
| R05 live app logic | T07 | New behavior/build instance, preserved logical state, safe old retirement |
| R06 service updates | T06 IPC, T07, T08 failure | Fenced writers, coherent routes, accepted request preservation |
| R07 continuity limits | T07, T08 | Schema rejection, migration fixtures, honest partial-continuity outcomes |
| R08 exceptional UI | T05, T10 visual review | Reference scenes, interaction review, scale/focus/clipping checks |
| R09 efficiency | T09 workload | Frame distributions, memory/idle/peak/reload costs on declared profiles |
| R10 isolation | T03 memory, T04 device, T06, T11 security | User faults contained, permissions/limits enforced, DMA trust stated |
| R11 persistent files | T04, T08 storage | Acknowledged durable writes survive modeled faults |
| R12 recovery | T07, T08, T12 | Known outcome after every transition; last good/rescue path |
| R13 dev boundary | T11 | Bad authority/replay/host-command requests rejected; release disabled |
| R14 contracts | T02, T06, T07 | Independent clients/codecs, versions and migration compatibility fixtures |
| R15 accessibility/text | T05, T10 | Keyboard/semantic/scaling/script corpus; exact support limitations |
| R16 delivery | T12 | Locks, provenance, license inventory and explained clean-build comparison |
| R17 diagnostics | T01,T07,T08,T11 | Accurate visible/error states, correlated traces, no false success |
| R18 maintainability | T13 documentation | Links, schemas, traceability and status/evidence consistency |
| R19 typed actions | T06,T07,T14 | UI/automation parity, scoped context, expected revisions and reload-safe authority |

## Test families

**T01 — Boot and recovery:** repeated cold ISO boot; malformed/missing boot response;
controlled user/kernel exceptions; rescue generation selection; no network/shared-folder
dependency. Readiness markers and serial output are mandatory; screenshots supplement them.

**T02 — Toolchain/loader/ABI:** kernel/user code models, section alignment, W^X/NX,
entry/return registers, invalid ELF ranges/segments, unsupported relocations, panic
policy and FP/SIMD save/restore. Compare independently built SDK clients.

**T03 — Memory/process isolation:** invalid user pointer, unmap race, guard fault,
cross-process mapping denial, resource exhaustion, process death and cleanup. Validate
zeroed pages and register state so one process cannot read remnants of another.

**T04 — Devices/storage path:** bad descriptors/features, DMA lifetime/reset under
load, lost interrupts, block error/timeout, flush failure, torn/reordered write model.
Separate VM kill tests from a real fault-injecting block model.

**T05 — UI interaction/rendering:** deterministic paint/layout scenes, old/new damage,
occlusion/alpha/filter invalidation, buffer lifetime, idle pacing, keyboard/focus/capture,
gesture interruption and text composition. Keep screenshot and semantic assertions separate.

**T06 — IPC/authority:** malformed frames/messages, length arithmetic, queue saturation,
deadlines/cancel/peer death, rights attenuation, stale handles, handle-transfer rollback,
route epochs, duplicate/reordered requests and independent protocol versions.

**T07 — Live update:** L0/L1/L2/L3 separately; compiler failure; corrupt transfer;
wrong base/schema/protocol; quiescence timeout; candidate start/import failure;
headroom shortage; active-input reload; incompatible dependency graph; 100-cycle leak test.
Prove changed behavior and unchanged kernel boot identity. Keep an external host trace
of VM lifecycle as additional evidence that reset/snapshot restore did not fake continuity.

**T08 — Update/persistence faults:** deterministic failure before/after checkpoint,
prepare record, decision flush, writer fence, route publication, health record and ACK.
Crash old/new/coordinator, disconnect host, fill disk, fail flush. Check single writer,
accepted state revision, dedupe history, coherent generation and available recovery blobs.
For irreversible effects require forward-recovery tests, not a fictional rollback pass.

**T09 — Performance:** fixed scripts/seeds for idle, drag/resize, editor scroll/type,
large list, overlap, effects over changing content and concurrent file work. Use optimized
builds, warm/cold distinction, p50/p95/p99, missed deadlines, resource peaks and trace overhead.
Never equate frame submission with display-photon latency.
Cold idle targets 256 MiB including bounded caches; VM capacity is not an idle
allowance. Optional inference adds separate warm-idle/active, load/unload and
weights/attention/scratch/device/shared-memory/peak accounting.

**T10 — Accessibility/international text:** semantic names/actions/focus, keyboard-only
workflows, contrast/non-color states, reduced motion, high scale, bidi, graphemes,
shaping/fallback and long translations. A semantic tree pass is not a screen-reader pass.

**T11 — Security:** replay/invalid session, malformed packages/fonts/images, capability
escalation, excessive imports, guest-directed host commands/paths, dev endpoint in release,
entropy not-ready and TLS/certificate failures once networking exists. Perform threat-specific
review before handling valuable credentials or exposing public services.

**T12 — Release:** compare two clean builds, verify package/ISO checksums, boot the exact
output offline, validate notices/SBOM/provenance, run rescue/last-good selection and
confirm the support matrix matches actual test evidence.

**T13 — Documentation/contracts:** local links/anchors, JSON parsing/schema fixtures,
all requirement IDs covered, ADR status/evidence, no proposed command represented as
existing, and source citations for externally derived claims.

**T14 — Typed actions:** compare button/shortcut/automation invocation of the same
operation; reject fabricated context, stale revision/preview, extra authority and
unsupported versions. Test duplicate requests and action-provider replacement.

**T15 — Optional inference:** exact supported model/operators/backend, FP/SIMD,
quotas, invalid proposals, malicious context, worker crash and unload/reload. Measure
editor responsiveness and all memory profiles. This is not a core-release gate.

Revision-2 T05 also compares Slint/custom target configuration and records text/effect/
semantic limits. T01/T12 require the actual ISO and boot-attempt/health/fallback cases;
live transaction tests cannot substitute for boot recovery.

## Test placement and limits

Pure state machines, codecs, layout and storage models run on host. Guest integration
tests validate privilege transitions, IRQ/device behavior and actual rendering. Use
property/fuzz tests for untrusted parsers and migration bounds. Loom/Miri are helpful
within supported models; they cannot prove full hardware correctness.

Do not create implementation-mirroring tests for trivial cosmetic changes. Focus new
tests on behavior, boundaries and previously uncertain risks. Once relevant checks
pass, do not repeat unrelated suites without a reason.

## Evidence bundle layout

Proposed `evidence/<run-id>/`: `config.json`, input revision/locks, artifact hashes,
serial log, guest events, host lifecycle log, metrics, screenshots, fault script/seed,
assertion results and limitations. Include exact test/harness revision. Any manual
intervention is recorded. Redact secrets and document content before sharing.
