# P4 epic — Guest desktop and live assets

Issue type: Epic. Status: Not started. Owner: Graphics/Platform/Security/Quality.
Created: 2026-11-06. Milestone: M4. Labels: `phase-4`, `milestone-m4`.
These are repository-local, Jira-ready stories; no Jira issues have been created.

## Outcome and starting point

Boot the original OS from its standalone ISO, interact with the selected UI inside
hardware-isolated guest processes, and publish host-built L0/L1 resources into the
running desktop. Preserve unsaved editor work, selection, window and focus, prove
unchanged boot identity and continuous uptime, and reject invalid/unauthorized
updates without losing the working generation. Keep serial recovery independent.

Planning baseline: `main` at `1d934f1`, pulled with `git pull --ff-only` on
2026-11-06. Existing uncommitted display-roadmap and Windows-policy edits were
preserved. This plan incorporates M4/B61 from that display amendment without
claiming M22 implementation. All P4 stories start Not started.

Read the [strict M3 review](../../../evidence/phase-3/P3-14/2026-11-06/README.md),
[M2 review](../../../evidence/phase-2/P2-13/2026-10-24/README.md) and
[hosted performance findings](../../../evidence/phase-2/performance-remediation/2026-10-24/README.md).
M3 accepts an exact bounded SDK/runtime subset, including one thread per process,
masked SSE exceptions and ephemeral policy/routes. Its nonrendering portable work
does not supply fonts/raster/display/input, device authority or authenticated dev
transport. M2's custom Rust choice remains provisional until actual guest review.
The probe arena and hosted process measurements are not a production guest heap
or evidence of the 256 MiB whole-desktop target.

## Ordered stories

| ID | Story | Depends on | Backlog coverage | Status |
| --- | --- | --- | --- | --- |
| P4-01 | [Recheck prerequisites and lock the guest desktop plan](01-prerequisites-profiles-and-contract-plan.md) | P3-14 and P2-13 Done; Phase 0 recheck | B01/B02 follow-up, B32 setup | Not started |
| P4-02 | [Admit long-lived UI services and reclaiming heaps](02-desktop-runtime-capacity-and-lifecycle.md) | P4-01 | B13/B16 runtime follow-up | Not started |
| P4-03 | [Mediate device resources, IRQs and DMA lifetimes](03-device-authority-irqs-and-dma.md) | P4-02 | B14/B16 device foundation | Not started |
| P4-04 | [Present a checked guest framebuffer](04-framebuffer-mapping-and-presentation.md) | P4-03 | B16 display backend | Not started |
| P4-05 | [Broker surface leases and compose guest windows](05-surface-leases-and-compositor.md) | P4-04 | B16 compositor | Not started |
| P4-06 | [Deliver physical keyboard and pointer input](06-input-device-and-event-routing.md) | P4-03,P4-05 | B16 input | Not started |
| P4-07 | [Run selected UI, fonts and text in guest processes](07-selected-ui-fonts-and-guest-rendering.md) | P4-05,P4-06 | B07/B09/B16/B36 guest confirmation | Not started |
| P4-08 | [Integrate the minimal shell and editor workflow](08-shell-windows-and-editor-workflow.md) | P4-07 | B08/B16 shell | Not started |
| P4-09 | [Export semantics and checked typed actions](09-semantics-and-typed-action-routing.md) | P4-08 | B16, M4 B37 seed | Not started |
| P4-10 | [Protect setup/login input and diagnostic privacy](10-trusted-input-and-diagnostic-privacy.md) | P4-09 | B16 trusted-input subset | Not started |
| P4-11 | [Validate single-output display contract foundations](11-output-scale-and-geometry-contracts.md) | P4-10 | B61 | Not started |
| P4-12 | [Transfer bounded bytes over a dedicated dev channel](12-development-transport.md) | P4-03,P4-11 | B14 | Not started |
| P4-13 | [Authenticate development sessions and enforce authority](13-development-session-authentication.md) | P4-12 | B15 | Not started |
| P4-14 | [Stage and validate bounded resource plans](14-resource-transfer-and-plan-validation.md) | P4-13 | B15/B17 staging | Not started |
| P4-15 | [Publish coherent guest L0/L1 generations](15-guest-resource-and-layout-publication.md) | P4-11,P4-14 | B17 | Not started |
| P4-16 | [Prove continuity through update and transport faults](16-reload-continuity-and-failure-recovery.md) | P4-15 | B17/B32 failure matrix | Not started |
| P4-17 | [Measure and improve guest damage, pacing and latency](17-guest-performance-and-idle.md) | P4-16 | M4 B32, B16 efficiency | Not started |
| P4-18 | [Stress reloads and reconcile whole-guest resources](18-reload-stress-and-resource-accounting.md) | P4-17 | B17/B32 stability | Not started |
| P4-19 | [Run and review M4 acceptance](19-phase-4-acceptance.md) | P4-01 through P4-18 | M4 gate, B32 review | Not started |

Default execution order is the table order, accepting one story before advancing.
Dependencies are acceptance gates, not instructions to run agents concurrently.
P4-02/03 expose runtime/device work explicitly so P4-07 cannot borrow test-only
allocation, unrestricted kernel mappings or unimplemented driver hooks. P4-16 owns
transition faults, P4-17 owns measurements/remediation, P4-18 owns sustained stress;
P4-19 aggregates final-source evidence rather than deferring implementation to review.

## Fixed scope and defaults

- Original Rust kernel and isolated Rust services; one vCPU, one execution thread
  per process, software rendering, one output, workspace-local pinned inputs and
  independently buildable ISO. No guest compiler, host rendering fallback, GPU
  stack, SMP, foreign runtime or reboot disguised as reload.
- Prefer a checked boot-framebuffer display backend and a bounded VirtIO input/dev
  transport experiment. P4-01/03 must validate actual devices and select exact
  supported features before implementation. Preserve separate diagnostic serial.
  Document any temporary kernel driver, its trust and extraction handoff; toolkit,
  shell, parsers and resource publication belong in user processes.
- Physical/virtual page, table, image, heap, queue, lease and launch-credit limits
  are admitted explicitly. Increase experimental capacities only with checked
  accounting and regressions; do not silently alter frozen SDK semantics or M3
  deadlines. No allocator/fixture address shortcuts or direct app device authority.
- L0/L1 changes are validated data interpreted by already-running code. No new
  executable pages, behavior/helper substitution, app process replacement or state
  migration. M5 owns behavior replacement, reauthorization and safe code retirement.
  M4 still releases old data only after all scene/presentation readers finish.
- One development user and ephemeral state. Development credentials do not create
  a desktop identity or grant account/installer administration. Synthetic protected
  input tests seed M7; real credentials/accounts, disk installation and durability
  remain M6/M7. Clipboard/drag/pickers/notifications are interface seeds for M9.
- Retain 720p feasibility and 1080p guest profiles, plus declared scale/text/effect
  fixtures. B61 seeds explicit output generations and transforms. 4K, multi-output,
  live topology/mode switching and hot-plug remain M22; GPU acceleration remains M18.

## Measurements and scope of completion

P4-01 locks profile capacities, bounds, sample counts, seeds, clocks and independent
oracles before measurements. Preserve existing targets: 256 MiB cold idle including
kernel/services/apps/buffers/caches; 1080p/60 render+composition CPU p95 below 8 ms
and p99 below 16 ms; guest input-to-present-submission p95 below 25 ms; completed
host build to visible resource commit below 500 ms; at least 100 compatible reloads
without unexplained settled growth. Keep clock/caret timers in idle accounting.

M4 requires valid guest measurements and remediation of correctness/boundedness
failures. Target misses must have distributions, cause evidence or an explicitly
unresolved finding, and a next bounded task; they are not performance passes.
Do not silently narrow rich effects, enlarge budgets, extend inherited deadlines
or label all R08/R09/R15 complete. This preserves the existing M4 exit of guest
rendering metrics without claiming every later desktop quality target is achieved.

## Evidence and story completion

Use Not started, In progress, Blocked or Done consistently in story and index.
Done requires every checkbox, negative case, reproducible command/runbook and
recorded review. Planning creates no passing evidence or new ABI freeze. P4-01
specifies concrete commands during implementation; proposed command names are
not presented here as existing tools. Failed prerequisites block dependent work.

Record source commit plus dirty-input hashes, exact compiler/features/dependency
locks, SDK/codec versions, packaged image/font/resource/ISO hashes, device config,
launch arguments and measurement settings. Correlate boot/build/session/process,
surface/lease/output/resource generations, input sequences and transaction IDs.
Use synthetic documents/credentials and bounded diagnostics; do not retain secrets.
Screenshots supplement independent pixel, logical-state, semantic and authority
assertions. Prove the presented guest artifact, not just a RAM checksum. No reset,
snapshot restoration or changed VM PID may masquerade as continuous guest uptime.

Keep full generated runs under ignored `build/m4/<story-id>/<run-id>/` or `work/`.
Promote compact reviewed bundles to `evidence/phase-4/<story-id>/<run-id>/` only
after implementation, following [retention](../evidence-retention.md) and the
[historical evidence notice](../../../evidence/README.md). Preserve failures.
Run documentation, governance and repository-hygiene audits before retaining work.

Recheck Phase 0 before extensions. For shared boundary/scheduler changes validate
ordinary, user-boundary-test, scheduler-test and capability-test separately, plus
every affected later flavor. Existing xtask case commands build/package/execute;
do not invent a compile-only matrix command. Run affected M1/M2/M3 regressions
per change and complete M1/M3 plus affected M2 at final acceptance. Keep fixture
policy in registered user_fixture hooks. Run guest acceptance serially in the
approved signer/xorriso context, advise the owner to keep the host awake, and
distinguish host preflight timeouts from guest deadline failures. Interrupted runs
never pass. Do not change host power/security settings or assume a standing waiver.

## Traceability and handoff

| Existing work / gate | Owning stories and delivered boundary |
| --- | --- |
| B14 | P4-03/P4-12 actual bounded host/guest transport |
| B15 | P4-13 authority/authentication, P4-14 semantic plan validation |
| B16 | P4-02–P4-10 runtime/device/display/input/UI/shell/semantics/protected focus |
| B17 | P4-14–P4-16 staging/publication/recovery; P4-18 sustained reloads |
| B61 | P4-11 output/scale/geometry encodings and single-output guest fixtures |
| B07–B09/B36 | P4-07–P4-09 actual selected guest toolkit/text/input/semantics; P4-19 selection review |
| B32 | P4-01 profiles; P4-16 faults; P4-17 measurements; P4-18 stress; P4-19 aggregate |
| B37 / M5 | P4-09 checked UI/automation entry seed; M5 still owns replacement parity and checkpoints |
| M5 / B18–B21 | P4-19 exports stable logical IDs, data-generation/release rules, current authority, resource peaks and failure limits; no behavior-replacement claim |
| M7 / B41–B44 | P4-10 protected-input evidence and keyboard/visible-focus consumers; no real account or installer implementation |
| M9–M13 / M22 | P4-11 explicit output/scale seeds and limitations; no later display/service acceptance |

Epic requirements: M4 R04/R08/R09/R13/R17; bounded R03/R10/R14/R15/R18
and R19 groundwork. Test families: T01/T02/T03/T04/T05/T06/T07/T08/T09/T10/T11/T13,
M4 T14 action subset. See [M4](../milestones.md#m4--guest-desktop-and-live-assets),
[validation](../validation.md), [surface contract](../../06-contracts/ui-and-scene-protocol.md),
[wire draft](../../06-contracts/update-wire-format.md),
[security](../../02-architecture/security-and-trust.md) and
[display roadmap](../display-roadmap.md). This decomposition authorizes no Jira
creation, remote publication, physical-PC operation or host-policy change.
