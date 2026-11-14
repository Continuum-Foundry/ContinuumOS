# P4-04 checked guest framebuffer acceptance

Status: passed local Codex self-review, 2026-11-14, for the declared single-output
synchronous software-copy backend. This is selected story acceptance, not an M4
milestone exit, external approval or a usable guest desktop.

Source: clean `main` fast-forwarded with `git pull --ff-only` to `33f0869`, followed
by the exact working inputs in [source-lock.json](source-lock.json).
[Selection](selection.json) records consumers, impact and omissions before testing.
[Acceptance](acceptance.json) and [independent aggregate review](aggregate-review.json)
identify 21 command groups, 20 selected VM launches and 18 separate metadata builds;
no full suites were selected. Ordinary 30000-ms guest and 320-ms local ready bounds
are unchanged. Each display guest preserves all nine admitted owner runs and
900 heap cycles, bounded restarts and exact physical settlement.

## Criterion review

| Criterion | Reviewed result |
| --- | --- |
| Exact independently bootable ISO at 720p and 1080p | Captured actual QEMU output matches a separately generated CPU reference at every pixel; initial displayed capture is entirely black. Color bars, corners, borders and two-sided clipping identify channel order and placement. |
| Metadata, extent, format and reservations | 110 ordinary and 110 display-feature kernel host tests pass, including checked overflow/missing output, reservation overlap, unsupported format, 24/32-bit channel conversion and guarded padded stride. Actual guest scanout is framebuffer-owned supervisor RW/NX/UC, with no WB alias. |
| Authority, source release and stale generations | Actual CPL3 calls bind acquisition/query to the trusted display-role incarnation. Every other entered role denies all five operations; stale generation, user capture/mapping, invalid source and row capacity deny. Source is overwritten immediately after each successful copy; captured pixels retain the copied data. |
| Mapping/owner cleanup and serial recovery | Injected two-page mapping failure rolls back completely. Owner death revokes the grant; runtime teardown returns the tested dynamic pool to baseline. Unsupported output keeps serial boot diagnostics; the serial-only ISO passes. |
| Affected regressions | Ordinary runtime, four independent metadata ISO boots, scanout-memory, actual guard fault, owned memory, timer, four boundary guests, three scheduler cases, capabilities and independent SDK/runtime all pass. Ordinary/boundary/scheduler/capability/display feature subsets build separately among the 18 flavors. |
| Independent oracle | Complete raw identities/sequences, exact kernel/ISO hashes, mapping records, authority/lifetimes, blank/output pixels and QMP acknowledgements are re-read independently. Five missing-criterion mutations per display mode are rejected. |
| Host harness and orchestration | 19 affected M1 host tests and 28 current selection tests pass. Explicit native-child and no-ISO QMP diagnostics remain ignored in ordinary discovery; lifecycle parent explicitly invokes its native child. The original P4-04 orchestration pair separately passes [27 tests](original-selection-tests.txt). |
| Prerequisites | Fresh signing/compiler/Phase 0/tool checks, clean linking and source-path-with-spaces checks pass in the approved execution context. |

Both display modes use build ID
`5b5daaacf803274194b9d9f3c7ae9ee49f281dc6b6bc77e3e2220d7fdb46d73f` and kernel SHA-256
`aad1aeb3a50951ebcc4f02d1790f54cf03fc93edc54de9785e012e186549ae61`.

| Mode | Exact ISO SHA-256 | Displayed PPM / independent CPU reference SHA-256 |
| --- | --- | --- |
| 1280x720, pitch 5120 | `d7ceb57f7bd17c069abbe0f6ec873762f2f9f990e58161fd68fd62430746bd8a` | `99421c7640c36db7d6e385526ebb7f98c3bd234f13612433d64421032c10a1e2` |
| 1920x1080, pitch 7680 | `57e48b94c782e50bfd9dfdbf728f66f75a5c695a9df0764d702391a3dc6b1e57` | `5a1d91c0773ca3f07029cb94b72da031fff4524792c1314b516545a3b0b8e81d` |

Both observed formats are 32-bit with RGB shifts 16/8/0, physical base
0x80000000, one output. [720p capture](display-720-display.png) and
[1080p capture](display-1080-display.png) are lossless PNG conversions of QEMU PPMs;
initial blank captures are retained separately. [Normalization](normalization.json)
binds original and retained bytes. [Serial selection](serial-selection.json) states
the explicit excerpt rules and sequence gaps; complete raw streams stay local.

## Boundary and limitations

The temporary trusted kernel mediator accepts one checked output up to 1920x1080,
pitch up to 8192 and validated 24/32-bit RGB channel layouts. The actual optical
matrix above qualifies 32-bit output; other layouts have guarded host conversion
coverage only. Entire page-rounded backing must remain framebuffer-owned. Install
the scanout mapping before runtime process roots snapshot upper PML4 slots.

User callers pass explicit integer arguments through the checked syscall boundary.
A complete row is first copied into at most 7680 private kernel bytes, then clipped
and converted with volatile stores. No user pointer survives. Return after MFENCE
permits source reuse and proves synchronous CPU-copy completion. A screenshot proves
QEMU displayed bytes at capture time. Neither result proves page-flip, atomic-frame
presentation, VSync, tear freedom, monitor acknowledgement or scanout latency.
Alpha is ignored; no color-management/composition pipeline is accepted.

Scanout and capture remain unavailable to apps. Role/incarnation/generation checks
use the P4-03 broker; supervisor mapping lifetime is boot-reserved and owner grants
revoke at teardown. Internal operations 0x4100–0x4104 remain unfrozen. Only the
previously accepted SDK subset is frozen. No GPU/IOMMU, physical input, selected
guest toolkit, compositor, live reload or 256 MiB cold-idle result is accepted.
P4-05 receives this backend; P4-05–P4-19 remain Not started.

## Development findings and source attribution

[Historical findings](development-findings.json) retain failed/interrupted attempts:
late scanout mapping caused a syscall-root page fault; QMP burst requests failed;
the old PowerShell child lifecycle fixture missed its unchanged host observation
bound; config-after-heap launch staging duplicated a large table rebuild and
exceeded the unchanged guest-ready bound. Fixes establish mappings before root
snapshot, use response-matched sequential QMP, use an already signed native child
for lifecycle behavior, and map config before heap. Counts, memory sizes, retries
and deadlines remain. No timeout alone is called a root-cause proof.

The Nov9 partial full run is interrupted, not acceptance. On Nov14 strict resume
rejected the relocated PowerShell context and concurrent unrelated retention-tool
changes. This final selected gate ran fresh; no bridge/bypass reused that old run.
The other owner-authorized chat was explicitly paused before this gate. Its changes
remain preserved and unstaged; [attribution](concurrent-inputs.json) identifies
unrelated and mixed files. P4-04 keeps its schema-1 selection and is not migrated.

[Staging proof](staging-input-proof.json) distinguishes tested working inputs from
the intended P4-04 index: guest/build inputs agree exactly or by recorded Git text
normalization; three unrelated new host-policy inputs are excluded. Two mixed host
orchestration files stage their original P4-04 variants, independently unit-tested,
while retention extensions stay unstaged. Full gate host tests ran the compatible
current working variants. This is not a clean-checkout reproducibility claim.
[Local source reconstruction](local-source-archive.json) preserves complete raw
implementation inputs and both orchestration variants; the final intended index
patch remains local beside it. No logs, source trees, binaries or ISOs are uploaded.

## Reproduction and final closeout

Use the [framebuffer runbook](../../../../docs/05-delivery/phase-4/framebuffer-runbook.md).
Preview `tools/Test-P4-04.ps1 -Plan`, execute serially in the approved signer/xorriso
context and independently review the resulting journal with
`tools/p4_display_acceptance.py review --acceptance <journal>` in the same
Enter-Dev environment. Keep the host awake without changing host settings.

Full original journal: `build/m4/P4-04/acceptance-1794635215585403600/acceptance.json`. Full raw logs,
ISOs/ELFs, CPU PPM references, QMP streams and failures remain local under its
referenced `build/m1`, `build/m4` and `work/acceptance` paths. The compact bundle
retains criterion evidence and hashes within 4 MiB, 200 files and 256 KiB/file.
Final prose/evidence/staging changes rerun host audits and independently re-review
unchanged guests through strict source/context/artifact-bound resume. The
[closeout method](closeout-method.json) identifies the local final-journal locator
without a circular tracked-evidence mutation after the final audit snapshot.
No commit, push, publication or later-story implementation is performed.
