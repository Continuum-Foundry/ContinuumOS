# Phase 3 aggregate acceptance

P3-14 is Done by owner acceptance with one legacy host-lifecycle exception.
The strict full gate has not passed; the default command below retains every
original check and deadline for future runs. Keep the host awake. Run serially in the existing
approved Windows development signer/xorriso context; do not change power/security
settings. Full generated files remain local under `build/m3/P3-14/` and `work/`.

```powershell
pwsh -NoProfile -File tools/Test-P3-14.ps1
python tools/p3_acceptance.py review --acceptance <local-acceptance.json>
# Only after review; destination must not already exist:
python tools/p3_acceptance.py review --acceptance <local-acceptance.json> --destination evidence/phase-3/P3-14/<date>
```

[Fixed inputs](../../../config/p3-14-acceptance.json) lock the commands, original
30,000-ms positive guest deadline, deliberate 1-ms negative deadline and 23 matrix
rows. The runner fingerprints all implementation/tool/config/contract/fixture
inputs, including untracked files, before execution and rejects any added, missing
or changed input afterward. It records the base commit, dirty patch, exact
compiler/tool locks, SDK hash, artifacts and ISO payloads through the nested gates.
Documentation and evidence can be prepared during execution without changing the
measured implementation. Final documentation hashes are recorded at promotion.

The runner executes its host lifecycle tests and a fresh complete P3-13 gate with
no regression/prerequisite reuse. P3-13 calls the complete P3-12 gate: ten Phase 0/
signing/doctor prerequisites, all 33 M1 rows/42 optical launches, 95 kernel host
tests, separate memory/loader/boundary/scheduler/handle/channel/wait/supervision/
policy/route/SDK/runtime guests, independent codecs, eleven metadata flavors and
affected model/selected portable final-link checks. It then runs separate public
supervision, fourteen metadata flavors and five repetitions of both continuous
stress profiles. Seeds, samples, quotas and timing bounds remain in the existing
locked M3/P3-13 profiles. Every kernel uses a fresh Cargo target directory; SDK
clients/services use separate public-only locked/offline roots. The aggregate
preserves the previous stress cache in its local run directory, forcing one fresh
public stress build before exact source/compiler-keyed reuse across repetitions.

The additional P3-01 run checks current prerequisites/contracts and real bounded
host-child failures. Independent reviewers reopen the raw serial, ELF/disassembly,
extracted ISO images, build identities, authority decisions and resource series.
This is local Codex self-review, not independent human or security approval.

The live native observer uses the same strict parser as final review with owned
JSON record-tree capture disabled during polling. Every current snapshot still
runs all field, sequence, stage and resource checks; the 20-ms polling sleep,
log caps, readiness policy and deadlines are unchanged. Final assertions and
export still build and consume the complete record tree. Host tests compare both
paths over malformed records, complete prefixes and partial-record boundaries.
On one captured 931,175-byte stress log, 30 release-mode checks took 506.194 ms
with record capture and 116.052 ms without it. These are host-parser observations,
not guest CPU or desktop performance results. The changed host source requires a
fresh final-source regression gate; the earlier passing runtime subset alone
does not accept this observer revision.

Five deliberate failures exercise the aggregate consumer. A malformed user ELF
and wrong artifact hash reject before launch. Three fresh executions of the exact
runtime ISO supply a passing control, a deliberately wrong expected panic outcome
and a consumer adapter that drops its required completion record. Original serial
bytes remain untouched and retained; this is a missing-record consumer test, not
a claim that the guest itself omitted a record. Another real QEMU launch hits the
explicit negative 1-ms deadline. Every launched process is waited/reaped through
a private Windows kill-on-close job; the host test also verifies descendant
termination on timeout. The original ISO's packaged host session is intentionally
reused by these diagnostic replays; it is correlation, not a fresh-boot credential.

Every missing/skipped/nonzero/timed-out positive stage fails without retrying.
The aggregate command has a separate 10,800-second outer bound for the entire
serial suite; it does not extend any per-probe deadline. Host preflight failures
and guest timeouts retain distinct logs and classifications. Failed or interrupted
attempts remain local findings and never count as acceptance.

After the final review, promote only compact outcomes, identities, selected raw
records, metrics and explicit unsafe/remaining-authority review. Retain complete
logs, ISOs, user images, source patches and time series locally. Add documentation
hashes and a manifest covering only the promoted files; stage the reviewed task
payload separately from the owner's display/signing changes, then run repository
hygiene, governance and documentation checks.

M3 covers isolated runtime/interfaces only. M4 guest display/input/render/resources
and authenticated L0/L1 reload; M5 state migration, behavior transfer and safe old
process retirement; M6 durability; M7 real accounts/installation remain separate.
Only the exact existing SDK subset is frozen. One vCPU/one thread per process,
masked SSE exceptions, unverified native x87 debug metadata and ephemeral synthetic
policy/routes remain declared limits. The 256 MiB cold-idle desktop and desktop
frame/input targets remain open.

## P3-14 owner-accepted closure

P3-14 is Done by explicit owner acceptance with a legacy host-lifecycle exception. All current guest/runtime regressions, complete M1 review (33 rows/42 launches), ten stress guests and five aggregate failure probes pass local Codex self-review. The legacy P3-01 Python child misses its original one-second startup bound; that fixture is waived for this story closure only. The strict full gate and M3 exit remain open. No failed check is labeled passed, and no deadline is extended.

See the [evidence, failed fixture and explicit owner exception](../../../evidence/phase-3/P3-14/2026-11-04/README.md). M4 guest UI/reload, M5 migration and safe retirement, M6 durability, M7 accounts/installation and all desktop efficiency targets remain open. Only the existing exact SDK subset is frozen; broader ADRs remain Proposed.
