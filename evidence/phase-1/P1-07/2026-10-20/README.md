# P1-07 local review — 2026-10-20

**Blocked. P1-07 is not Done and M1 remains incomplete.** The aggregate harness,
ordered final readiness and deterministic packaging controls are implemented on
`codex/p1-07-phase-1-acceptance`. Codex performed this local self-review; it is not
independent approval. No push, PR, remote build or release occurred.

The first full gate, `suite-1792532218089714300-19176`, passed nine prerequisite/
regression rows, both clean normal kernel builds, five consecutive normal boots
and the second clean build's exact ISO boot. It then **failed** its whole-ISO
comparison. The difference was generated GPT disk/partition GUIDs and their CRCs,
in addition to the intentional host session fields. Kernel bytes were identical.
The original failed result and its diagnostic byte ranges are retained under
`development/`; its full images, logs and disassembly remain local.

The recipe now fixes volume/file dates and the experimental image's GPT GUID.
Repackaging the two original staging trees into separate scratch images produced
no remaining byte difference after normalizing the session fields. This isolated
comparison is **not a new boot** or a passing full gate.

The corrected full gate, `suite-1792532726225181200-52940`, stopped in M0 because
Windows Smart App Control refused the rebuilt host unit-test executable (error
4551). The same failure occurred outside the sandbox. Code Integrity events
3077/3033 identify `VerifiedAndReputableDesktop`, policy GUID
`{0283ac0f-fff1-49ae-ada1-8a933130cad6}`. This is separate from xorriso's documented
MSYS2 sandbox restriction. See [approval request](approval-request.txt) and the
selected diagnostic fields under `development/`.

The owner selected arranging approval under the existing policy. **Execution has
not been approved.** No signing certificate was available in either inspected My
store; read-only CiTool policy enumeration returned access denied. Microsoft
documents no per-app exception for Smart App Control and a trusted signing route.
No policy, certificate store or security setting was changed; the blocked binary
was not moved or modified. No material was sent to a signing or approval service.
[Microsoft FAQ](https://support.microsoft.com/en-us/windows/security/threat-malware-protection/smart-app-control-frequently-asked-questions),
[developer guidance](https://learn.microsoft.com/windows/apps/develop/smart-app-control/overview).

## Retained selection and limits

The [review](review.json) and [result](result.json) describe **independent guest
regressions only** on the corrected recipe, not a passing full M1 suite. All
23 scenarios passed their declared assertions across 36 distinct launches:
14 reached readiness, nine were expected fatal outcomes, eleven were expected
rejections, and two were deliberately broken-before-entry failures. Every selected
VM was reaped. These categories do not count expected failures as successful boots.
Each selected launch retains original raw serial bytes and a compact context containing
its package manifest, exact ISO/firmware hashes, argv, process result and host
stdout/stderr. One input/artifact inventory per build flavor avoids repeated dumps.
Full generated artifacts stay under ignored `build/m1/`; no source tree or binary
is copied into Git.

Source base: `0c3933b` (`Untrack ignored historical logs and source copies`), dirty.
The retained input inventories hash all kernel/xtask build inputs, toolchain,
lock/config and story scripts. Current guest build source hashes and artifact
bytes are checked by `tools/Review-P1-07.ps1 -Partial`. Earlier checks under
`checks/` belong to the **first, failed full suite** before the GPT recipe change;
their 14 xtask + 14 kernel-library tests are not current-source host acceptance.
The current-source host tests did not execute. Preserve this distinction.

Current formatting, documentation, schema-fixture and governance checks passed;
their results are retained under `checks/current-*`. The final process snapshot
found zero QEMU processes. The compact frozen bundle has 168 files and about
2.5 MB, within the 4 MiB/200-file/256-KiB-per-file retention budget.

Representative corrected ordinary normal run:

- Build ID: `918eeef20c20431c495d68ef2d843f09e360f8136d28e951047cbcebc9cac44e`.
- Kernel SHA-256: `c221b029eb3264e740f4d1f2c40d42e6d7f4540e1816543f1087f6d55364fcdb`.
- ISO SHA-256: `4edb835d815ac0e4963ad1225de9095c7fcac2ae1d72dd1bb9f011292c4ea74f`.

That exact ISO reached final readiness and continued idle before bounded host
cleanup. It is a corrected single normal boot, not the pending five-boot gate.

| Story matrix row | Result and evidence |
| --- | --- |
| Five normal cold boots | Passed in the first failed suite, with distinct sessions/fresh vars, ordered readiness and exit 33. Corrected final gate must repeat them; not accepted as final-source evidence |
| Second clean build | First-suite normal ELF bytes match exactly; both actual ISOs boot. Initial GPT differences failed comparison. Scratch fixed-GUID comparison passes; corrected complete gate pending |
| Missing/corrupt packaging inputs | First-suite prerequisite rejection records pass before tool/VM execution; retained under `checks/` |
| Broken loader/kernel configuration | Independent guest cases require bounded real failure, no entry/ready claim and reaped QEMU; raw logs retained |
| Boot metadata rejection | Eight independent expected field/reason outcomes, exit 35 and no later readiness |
| Invalid opcode/page fault/panic | Real expected fatal contexts and exit 37; ELF diagnostic symbols retained |
| Memory ownership/exhaustion | Reservation, zeroing/reuse and bounded exhaustion assertions retained |
| Guard/write/NX/emergency stack | Real fault diagnostics, including actual double-fault/IST entry; no reset counts as success |
| Timer/masking/clock | Delivered ticks, calibrated conversion, monotonic samples and distinct fresh-epoch sessions retained |
| Idle continuation | Real interrupt/IRETQ and halt continuation/register assertions retained |
| Harness negatives | First-suite 14 host tests pass missing/stale IDs, missing/duplicate readiness, unexpected exits/timeouts, cleanup and unexplained ISO changes. Current rebuilt host binary is blocked |
| Regression/docs/contracts | First-suite M0, dependency, doctor, schema and documentation checks pass; final metadata audits rerun after this review. This does not override blocked host execution |

## Acceptance criteria review

| Criterion | Conclusion |
| --- | --- |
| Every row has final raw evidence and result | **Not satisfied**: the corrected full gate cannot execute its host tests; corrected final clean comparison/five-boot matrix remains pending |
| Final exact ISOs and pinned configuration | **Not satisfied for the final gate**: first-suite six exact ISOs boot, and corrected independent guest images boot, but no completed corrected aggregate selection exists |
| Fixed timeout, no hidden extension | Satisfied for recorded attempts: `vm.deadline_ms=30000` is locked before launch; no deadline changed or automatic VM retry occurred |
| Honest outcome/lifecycle distinctions | Independently reviewed guest cases distinguish ready/test exit, expected fatal/rejection, broken-before-entry failure and host idle cleanup; timeouts never count as ready boots |
| Reviewable identity/notices/unsafe boundary | Source/input hashes, ELF audits/symbols, retained assembly excerpts and launch contexts are reviewable. No dependency or assembly implementation changed; the existing emulator I/O exit is reachable from explicit normal-kernel `acceptance-exit` boot configuration only after readiness |
| Criteria satisfied and consistent handoff | **Blocked**. Story/index/handoff retain incomplete M1 status and identify the actual host execution blocker |

The [runbook](../../../../docs/05-delivery/phase-1/acceptance-runbook.md) documents
the v5 readiness extension, normal-versus-test configuration, fixed timestamps/GPT
identity, strict ISO comparison and reproducible full command. Normal images use
no Cargo test features; ordinary normal boots continue into idle. Emulator exit
is not physical shutdown. Host sessions are correlation, not authentication or
reset detection. QEMU uses `-no-reboot`, fresh logs/vars, one core, 2 GiB, TCG,
Q35/OVMF, no network/share/user disk/snapshot/direct-kernel boot.

The kernel remains an experimental boot/memory/timer baseline: no general heap,
user processes/isolation, scheduler/FP context, desktop/rendering, persistent files,
live update, physical-platform or Secure Boot result. Delivered-tick uptime can
lag elapsed time. The 256 MiB cold-idle desktop target remains unmeasured. B06/B32
remain partial and all ADRs remain Proposed.

## Resume

An authorized trusted signing/execution path must first resolve the host block.
Do not infer permission from the owner's request alone. Re-run:

```powershell
. ./tools/Enter-Dev.ps1
pwsh -NoProfile -File tools/Test-P1-07.ps1
```

Then review all 33 rows/42 launches, source identities, both exact clean artifacts,
five consecutive normal boots and cleanup. Retain a **new** dated/run-specific
bundle and explicitly accept P1-07 before closing M1 or moving the handoff to M2.
The current frozen blocked bundle must not be rewritten into a passing narrative.

`evidence-index.json` distinguishes original and retained hashes. Serial logs are
verbatim; host text is path-sanitized. `SHA256SUMS.json` freezes only the selected
files actually retained in Git. Development observations explicitly distinguish
raw logs from transcribed early tool errors; full local artifacts remain available.
