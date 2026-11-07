# Phase 3 aggregate acceptance

P3-14's historical strict full gate passes at its original limits; see below.
The current P4-01 recheck uses the explicitly owner-authorized
[revision-2 scoped TCG stress profile](stress-deadline-profile-v2.md): only the two
long stress flavors use 60000 ms. Ordinary/SDK/public-supervision guests retain
30000 ms; all counts, guest-local bounds and negative deadlines remain unchanged.
Current passes must not be called original-limit acceptance. The earlier
owner-accepted host exception also remains historical evidence.
Keep the host awake. Run serially in the existing
approved Windows development signer/xorriso context; do not change power/security
settings. Full generated files remain local under `build/m3/P3-14/` and `work/`.

```powershell
pwsh -NoProfile -File tools/Test-P3-14.ps1
python tools/p3_acceptance.py review --acceptance <local-acceptance.json>
# Only after review; destination must not already exist:
python tools/p3_acceptance.py review --acceptance <local-acceptance.json> --destination evidence/phase-3/P3-14/<date>
```

[Fixed inputs](../../../config/p3-14-acceptance.json) lock the commands, ordinary
30000-ms and scoped long-stress 60000-ms deadlines, deliberate 1-ms negative deadline and 23 matrix
rows. The runner fingerprints all implementation/tool/config/contract/fixture
inputs, including untracked files, before execution and rejects any added, missing
or changed input afterward. It records the base commit, dirty patch, exact
compiler/tool locks, SDK hash, artifacts and ISO payloads through the nested gates.
Documentation and evidence can be prepared during execution without changing the
measured implementation. Final documentation hashes are recorded at promotion.

The runner executes its host lifecycle tests and a fresh complete P3-13 gate with
no regression/prerequisite reuse. P3-13 runs the bounded stress guests first, so
a stress deadline failure is reported before the longer regression suite. A pass
still requires the complete P3-12 gate: ten Phase 0/
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

The live native observer reads only appended serial bytes and retains parser
state between polls. It buffers partial lines and applies the same field,
sequence, stage and resource checks to every completed record. Truncation,
malformed records and output-limit violations fail closed. The 20-ms polling
sleep, log caps, readiness policy and original deadlines remain unchanged. Final
assertions reopen and validate the complete log and export its full record tree.
Reports include live read-byte counts and read/parse time. Differential tests cover
chunk boundaries and invalid input; the opt-in retained-log benchmark compares
full-prefix polling with incremental parsing. This measures host parser work,
not guest CPU or desktop performance. Changed host source still requires fresh
final-source acceptance.

An early host-only lifecycle preflight runs success, nonzero, missing-final and
timeout children before the expensive guest suite; the later complete P3-01 gate
remains mandatory. Child reports separate process-creation and post-creation
time. Nested runners stream stage timestamps and artifact paths as work starts
and finishes. To diagnose only the ten stress guests and public supervision:

```powershell
pwsh -NoProfile -File tools/Test-P3-13.ps1 -StressOnly
```

This command produces `diagnostic-passed`, which the final acceptance reviewer
rejects as a substitute for the complete gate. Supplied runtime reuse references
are validated before any guest launch; final P3-14 still forbids such reuse.

The supervision fixture's diagnostic-rate assertion observes guest clock values
around each call. Its bounded oracle accounts for a call crossing an elapsed
one-second quota boundary and requires a witnessed quota rejection. It still
rejects a fifth accepted call in a known unchanged window and premature quota
errors. The ordinary and public-SDK fixtures share this oracle; the kernel's
four-record allowance, guest limits and independent resource review are unchanged.

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

P3-14 was closed on 2026-11-04 by explicit owner acceptance with a legacy host-lifecycle exception. In that earlier evidence, guest/runtime regressions, complete M1 review (33 rows/42 launches), ten stress guests and five aggregate failure probes pass local Codex self-review. The legacy P3-01 Python child missed its original one-second startup bound; the owner waived it for that earlier closure only. That run did not pass the strict gate; the subsequent strict result is recorded below. No failed check is labeled passed, and no deadline is extended.

See the [evidence, failed fixture and explicit owner exception](../../../evidence/phase-3/P3-14/2026-11-04/README.md). M4 guest UI/reload, M5 migration and safe retirement, M6 durability, M7 accounts/installation and all desktop efficiency targets remain open. Only the existing exact SDK subset is frozen; broader ADRs remain Proposed.


## Strict M3 acceptance - 2026-11-06

The fresh full P3-14 gate and independent aggregate reviewer pass all 23
matrix rows and five deliberate failures at the original 30000-ms guest,
1000-ms normal host-fixture and 1-ms negative guest limits. No historical
regression or prerequisite reuse and no exception are used for this result.
M3's declared isolated-runtime/interface exit passes local Codex self-review;
M4 guest desktop/live assets is next. Guest UI/reload, M5 migration/retirement,
durability, real accounts and desktop efficiency targets remain open.

See the [strict acceptance and runner repair evidence](../../../evidence/phase-3/P3-14/2026-11-06/README.md). The earlier
failed/interrupted runs and owner exception remain recorded as history; only
the existing exact SDK subset is frozen. This is not human/security certification.
