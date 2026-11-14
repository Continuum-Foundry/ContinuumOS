# P4-05 copied surfaces and guest composition acceptance

Status: passed local Codex self-review, 2026-11-14, for the declared copied-lease,
single-output user-mode compositor. This is selected story acceptance, not an M4
exit, external/security certification or a usable desktop OS.

Source: clean `main` switched and fast-forwarded to `c0f6e0b`, followed by the exact
working inputs in [input-lock.json](input-lock.json) and the exact-byte
[source reconstruction](source-reconstruction.json). [Staging proof](staging-input-proof.json)
compares every tested implementation path with the intended index; the [reconstruction
verification](source-reconstruction-verification.json) restores all 520 tested hashes
from base Git bytes and exact capsule overrides. The staging proof distinguishes
exact bytes from Git line-ending normalization. This is not a clean-checkout
bit-reproducibility claim. No unrelated edits were present.
[Selection](selection.json) records impact, consumers and omissions. The
[plan](plan.json) and [results](results.json) identify 28 command groups, 22 selected
VM launches (one unchanged 720p probe reused with exact source/context/artifact
verification and independent re-review), and 20 separate metadata builds. No full
milestone suites or deadline/count waiver is used. Guest limits remain 30000 ms,
ready/cleanup bounds remain 320 ms, and inherited heaps/retries/lifetime credits remain.

## Criterion and unsafe review

| Criterion | Reviewed result |
| --- | --- |
| Two independent clients and optical output | Separately linked client images, byte hashes bound to the extracted ISO payload, compose opaque/translucent RGBA8 snapshots. All nine actual QEMU final captures equal the independently authored CPU references at every pixel, at 720p and 1080p. Each case first presents overlap, then its final transition; all 18 complete frame copies and 13680 rows are observed. |
| Wire and current authority | Explicit CDU0 fields and response bytes; no Rust struct crosses the boundary. Literal-byte/truncation/format/extent tests supplement independent raw request/reply review. Actual trusted process/incarnation/principal/session attribution, forbidden endpoint operations, foreign surface IDs, stale output/buffer generations and real session-2002 create/frame requests deny. |
| Admission and rejected-frame preservation | Partial copies, malformed stride/damage, three-slot lease pressure and the distinct 24 MiB global byte ceiling deny before active snapshot publication. Two 8847360-byte buffers on an additional surface are admitted; a third denies. Rejected operations preserve the reference-correct last frame. Eight-surface and arithmetic/exhaustion bounds have portable adversarial coverage. |
| Writer/reader lifetimes | Copied request chunks and private compositor storage prevent retained writable client aliases; original bytes are overwritten immediately after submit. Writes/reuse and release of InUse leases deny. Cancel/released leases free once, duplicate releases deny, frame opportunities stay separate from lease release. The single owner excludes mutation during frame readers. |
| Movement, hide, removal and death | Full redraw clears old/new exposure and clips signed placement/local bounds with premultiplied encoded-sRGB source-over. Ordinary dead-client cleanup itself schedules a repair frame; the real owner-death case calls the same helper after actual kernel teardown. Padding never appears as pixels. |
| Failure and exact settlement | Real compositor panic and a CPU-stalled presentation terminate with the original panic/watchdog outcomes. Incomplete requests cancel; already copied replies survive until consumption/death. Serial/control continue. Nine optical cases retain all nine entered owner runs and 900 heap cycles each; physical frame/allocation/object/handle/wait inventories settle exactly. Private surface/lease bytes settle or disappear with full process teardown. |
| Independent review | Exact kernel, client and ISO hashes, raw identities/sequences, authority outcomes, complete frames, QMP acknowledgements and full pixel data are re-read. Six missing-criterion mutations per case reject. Separate exact-count review distinguishes global byte pressure from per-surface slots. Actual ELF loads independently reconcile admitted physical bytes and allocation counts. |
| Affected regressions | Ordinary service boot, existing checked scanout, owned memory, four syscall-boundary guests, three scheduler cases, capabilities, channels and independent SDK/runtime pass. The ordinary/surface kernel host suites each pass 116 tests; four surface oracle tests, affected host harness/runner/retention checks, fresh ten-check Phase 0/tool preflights and all 20 separate metadata flavors pass. |

Unsafe review: frame storage becomes a slice only after bounded successful private
allocation; it is released after synchronous readers finish. All other surface
storage uses fallible Vec reservation under the already admitted reclaiming heap.
The allocator has one owning user thread. Kernel mailboxes keep no user pointers,
copy through the existing checked boundary, consume nothing on failed copies and
never hold a borrow/pin across user execution. Their fixed metadata is separate
from general SDK object queues. Scanout remains P4-04's supervisor RW/NX/UC mapping;
its CPU-copy fence does not establish atomic-frame display or VSync. Existing
freestanding memory shims use ABI-compatible byte-pointer C signatures; the pinned
alloc link emits a declaration-type lint, with actual link/guest/ownership checks
passing. No compiler/security settings are changed to suppress it.

## Exact optical artifacts

| Case / review | Build ID | Exact ISO SHA-256 | Displayed PPM / CPU reference SHA-256 |
| --- | --- | --- | --- |
| [surface-720](surface-720-review.json) | `f4d331892ead4dab8206c6b5a69aa6566f9c306041c020c943cee558b0b914a9` | `133e1ecbe351aaebe7a31b25201ca6a2e4f9c2bf9e36527e1068ff9b07b81404` | `a3d9822bfcf41cae8c7c740399e798d7cd5dc9eb339726c8dbcd38b6f96d35f5` |
| [surface-1080](surface-1080-review.json) | `f4d331892ead4dab8206c6b5a69aa6566f9c306041c020c943cee558b0b914a9` | `0425ff8a16538a6c903950c5956282403800144953551ffd983aef2cd2dbed80` | `9978bc4a5230b43ea87d05605777471953f3ebdde82d144c61b9e7eeb99e0883` |
| [surface-move](surface-move-review.json) | `84cc406c209afc517b4491ae85986f71a240c446c12028cbaa2635d899fdbc71` | `ad64e44121722b52f04d09ee5987efaab8f3ffe40f5747048e0ed260ec120378` | `feb5e057f72e2de9c8cf90f21a74cd69fbe213cbf049218a734c1ef96ee776f0` |
| [surface-hide](surface-hide-review.json) | `e21fcb628da4cc2fd7e7676bf2d626e05cd3ed81dc249e1c0cd69a76478c26ca` | `c9e3261adc323790d25260264eb4b6515c11876dd3f79ee37f396f7ae4a259da` | `e1defe2e6c76b2385fa320bc4b6bb386010588b8b42a84edc5f10807bc5ae4fd` |
| [surface-remove](surface-remove-review.json) | `296de6b200de0f5bf2e44180cd94f723aa34d51f6a63dd700e007214716376d4` | `239e17de16090c9231889445bfd09ae3017c5471616f199fb51499a8983feb0f` | `e1defe2e6c76b2385fa320bc4b6bb386010588b8b42a84edc5f10807bc5ae4fd` |
| [surface-denied](surface-denied-review.json) | `ce66618919d594a0d5804be5a5b23b57dfdbbefe9687029fb61cfed1ea47b99b` | `d649aea2c575c9d06c19b2bda1c6e8861b5aa49ea8676e11ef759579591a9073` | `a3d9822bfcf41cae8c7c740399e798d7cd5dc9eb339726c8dbcd38b6f96d35f5` |
| [surface-death](surface-death-review.json) | `2cda04803d599f2d21945753bc63f0d22e80cb113f915704913cdcd19ceb4f4d` | `eef15fdbf788f69e17813ceecbcd6471edc613bf44219f52c3280cd6a69eb5b5` | `e1defe2e6c76b2385fa320bc4b6bb386010588b8b42a84edc5f10807bc5ae4fd` |
| [surface-failure](surface-failure-review.json) | `d8362893a43cbf941101b9b337d7366bb771e61a5ebc2394a85ab1615b9d3049` | `fba0eccb54c5bb03b3564e86097be73a25cba8082f1314a82639ae3cce2e16e6` | `a3d9822bfcf41cae8c7c740399e798d7cd5dc9eb339726c8dbcd38b6f96d35f5` |
| [surface-stall](surface-stall-review.json) | `7784f3b9aa9796475f556e394dc2e00ce69858ee98720e7dfffb958ef9a30f8f` | `88347281358a0f57fe35a5e06759d6a606e65628798d93b1d85e0de6bdd69789` | `a3d9822bfcf41cae8c7c740399e798d7cd5dc9eb339726c8dbcd38b6f96d35f5` |

Lossless PNGs are byte-preserving P6 conversions, with original/retained hashes
in [normalization](normalization.json). Complete original serial, QMP, CPU-reference
and PPM files are in the verified local capsule. [Serial selection](serial-selection.json)
identifies retained complete criterion groups and intentional sequence gaps;
excerpts are not represented as complete raw streams.

## Boundary and handoff

At most eight surfaces, three leases/surface, 24 MiB of surface backing, one full
output scratch allocation and one 4096-byte request/reply slot per admitted client.
All fit existing private heaps; no kernel allocation rights are granted to apps.
Only resource/output revision 1 and RGBA8 premultiplied input are admitted. Surface
and lease IDs occupy disjoint ranges seeded by the persistent display-grant
incarnation; stale IDs cannot identify a later compositor's objects. Counter and
epoch exhaustion deny. No new SDK or permanent desktop ABI is frozen.

The temporary copied endpoint adapter and scanout mediator are trusted kernel
mechanisms; all surface policy, storage, z-order, clips and composition execute in
CPL3. Shell/editor roles are the admitted clients; richer broker/session policy,
accounts and credentials remain their later gates. No source pointer or native
callback crosses the service boundary. Client death during a Reading transport
request has bounded portable coverage; real guest owner and compositor death prove
hardware teardown/exposure/cancellation. This is not universal concurrent-race or
revocation certification.

The baseline redraws fully and cooperates after 16 rows. Diagnostic frame intervals
include scheduling and mediated scanout; they are not renderer-thread CPU, VSync,
latency or efficiency-target passes. A mid-copy compositor failure may leave a
partial frame; clients need an explicit fresh binding after service failure.
There is no crash continuity or safe compositor code replacement claim. P4-06
receives bounded pixel geometry and output-revision binding and P4-07 receives the copied buffer API.
Physical input, selected guest toolkit/fonts/text, semantics, resource transport,
L0/L1 reload and the 256 MiB whole-desktop target remain open.

## Development findings and retention

[Development findings](development-findings.json) distinguish the oversized
non-loadable alloc debug sections rejected by the unchanged 1 MiB module limit,
the original per-row-yield deadline miss, host-context/stack diagnostics,
reviewer-only correction and the later ordinary death-repaint fix. Failed and
interrupted aggregates remain non-passes. Source debug stripping and a host size
check preserve the limit; 16-row cooperation preserves both complete frames and
all bounds. The reviewer initially confused setup unwind 513 with deliberate
compositor panic 90; its regression test and explicit unchanged-artifact proof are
retained. The final source was qualified after the ordinary repaint correction.

Use the [surface runbook](../../../../docs/05-delivery/phase-4/surfaces-runbook.md).
Full execution/review records and exact ISOs/ELFs remain at the local paths in
[results](results.json). The committed bundle stays within 4 MiB / 200 files /
256 KiB per file. A canonical compressed local capsule preserves complete selected
streams, source reconstruction, commands/context, reviews and meaningful failures;
its selection/verification and availability receipt are retained here. Capsule
integrity is not a replacement for original artifact re-review/resume.

Final prose/evidence/index closure reruns host audits and re-reviews unchanged
guests through strict resume. [Closeout method](closeout-method.json) identifies
the local final-journal locator without a circular tracked-evidence hash mutation.
Exact original payload is retained for active review/resume; no deletion is
performed under the owner deferral. No commit, push, publication or later-story
implementation is performed.
