# P3-01 runtime prerequisite and host runner runbook

This gate rechecks existing code and fixes draft inputs for P3-02. No kernel or
Cargo dependency changes, user execution, frozen ABI, guest heap or performance
result is introduced. Read the [runtime draft](../../06-contracts/runtime-v0.md).

From PowerShell 7 at the workspace root, using the already enrolled signer:

```powershell
. ./tools/Enter-Dev.ps1
pwsh -NoProfile -File tools/Recheck-P2Prerequisites.ps1
pwsh -NoProfile -File tools/Test-P1-07.ps1
# Use the newly printed prerequisite and suite directories, never a historical pass.
pwsh -NoProfile -File tools/Review-P1-07.ps1 -Run <new-m1-suite>
python tools/Test-P3-01.py --prerequisites <new-prerequisite-directory> --m1 <new-m1-suite>
python tools/Review-P3-01.py --run <new-P3-acceptance-directory> --destination evidence/phase-3/P3-01/<new-run-id>
```

The reused prerequisite script writes under `build/m2/P2-01/prerequisites-*`;
that path is an implementation detail, not P2 evidence acceptance for this run.
It performs ten fresh signing/compiler/offline metadata/doctor/build/M0/negative
source-path/dependency checks. No install, enrollment, network override or host
policy change is performed. Existing xorriso/certificate-store sandbox restrictions
require the documented local execution context. An interrupted or failed run is
preserved and cannot satisfy this gate. A fresh M1 suite has all 33 rows/42 launches;
its review validates actual artifact bytes, identities, raw version-5 serial,
no-reboot/offline arguments and reaping. Existing M1 commands retain their fixed
30-second VM timeout and distinct ready/fatal/rejection/broken classes.

Run the prerequisite and M1 sequences serially: the existing host-child deadline
must not be consumed by a competing acceptance workload. P3-01 also fixes the
development signing test to corrupt a separate never-executed signed copy, since
Windows may retain a mapped section on the just-exited probe. Signature/digest,
argument/exit and tamper expectations remain unchanged; no policy/enrollment
change is made. Earlier host timeout/mapped-image failures remain local findings.

The P3 test uses the exact fresh Phase 0 user ELF referenced by the prerequisite's
`m0.log`, verifying that it is present and hashed. The host runner only checks
image identity/header/size; P3-03 owns full segment/loader validation. The image is
never executed by this host fixture. No VM is launched by `m3_runtime.py`.

```powershell
python tools/m3_runtime.py generate work/m3-runtime-draft.rs
python tools/m3_runtime.py run --image <fresh-userland-link.elf> --sha256 <actual-sha256> --case success
# Expected nonzero; each writes its distinct result.json and retains raw logs.
python tools/m3_runtime.py run --image <fresh-userland-link.elf> --sha256 <actual-sha256> --case missing-final
python tools/m3_runtime.py run --image <fresh-userland-link.elf> --sha256 <actual-sha256> --case nonzero
python tools/m3_runtime.py run --image <fresh-userland-link.elf> --sha256 <actual-sha256> --case timeout
```

Only the current Python interpreter and fixed script child are permitted, with
null stdin, hidden window, no shell or descendants. Each run has a fresh run ID,
exact config/contract/image/interpreter/runner hashes, arguments and parent/child
IDs. Normal fixture deadline is 1000 ms, timeout fault is 100 ms, polling 5 ms;
stdout/stderr are capped at 64 KiB. Kill/wait/reap runs on exceptional paths too.
`not-launched`, `reaped` and `unconfirmed` are separate cleanup outcomes; no process
API failure is a cleanup success. Nonzero exit, missing/mismatched final record,
timeout, output limit and validation rejection are distinct. Host fixture JSON
version 1 is separate from M1 serial version 5; no new guest serial version exists.

The test rejects unsupported CPU, SMP, capacity, timeout, unknown/missing fields,
wrong types, duplicate keys, unavailable/malformed/changed image and missing tool.
It exercises actual child early exit/missing completion/kill-and-wait, generated
field offsets against independent `struct` byte layouts, deterministic generation
and fresh baseline/source guards. Full logs and generated SDK stay ignored under
`build/m3/P3-01/` and `work/`; compact evidence retains reviewed results and hashes.

`m3-profiles.json` fixes Q35/TCG, `qemu64-v1,+apic`, one CPU, 2 GiB capacity and HPET
alongside explicit table/page/message quotas, stress counts, fairness, seed and
measurement rules. These are proposed runtime ceilings and experiments. P3-02
and later stories implement and test admission; P3-05 validates FP/CPU state,
P3-08 validates the proposed elapsed HPET clock. Every CPU/heap/IPC measurement
remains unmeasured here. Historical evidence availability follows
[evidence/README](../../../evidence/README.md).
