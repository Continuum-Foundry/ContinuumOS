# Windows development signing

The owner authorized converting this Windows installation into the development
machine on 2026-10-20. This supersedes the earlier request for individual approval
under consumer Smart App Control (SAC). P1-07's original blocked evidence stays
immutable. A development setup is not a passing M1 acceptance result.

## Current local rollout

The local RSA key and public-user trust are installed. The replacement base policy
`{3898A096-AC42-483C-AA0C-8CE5F2898A7F}` version `1.0.0.1` is active and enforcing;
consumer SAC enforcement is inactive. Defender antivirus, real-time protection and
behavior monitoring remained enabled. An already-compiled signed probe executes
with arguments and exit code 23, outside-path rejection passes, and changing a
signed PE section produces Authenticode `HashMismatch`.

The owner approved retrying Windows elevation. The pinned-tool supplement
`{BB917328-EB62-4C2E-9A0C-69F9545EA0DC}` is now active and authorizes the exact
180-file inventory. The signed compiler wrapper compiles and executes. Both fresh
signed probes pass, as does a clean offline Cargo fixture with a build script,
proc macro, unit test and executable from source/target paths containing spaces.
The fixture verifies argument boundaries and exit-code propagation. Full P1-07
acceptance is a separate evidence gate; its subsequent
[complete local review passes](../../evidence/phase-1/P1-07/2026-10-20-acceptance/README.md).
The [completed local signing review](../../evidence/host/windows-dev-signing/2026-10-20-completed/README.md)
retains policy activation, fresh-build results, signature hashes and the corrected
runner integration. The initial incomplete rollout is preserved alongside it.

The first clean compiler fixture exposed a Cargo runner quoting error. The runner
now uses an argument array in the workspace-local Cargo home configuration rather
than a quoted environment string. A new `build/cargo/signed/` cache makes previously
unsigned cached build helpers rebuild through the wrapper; earlier cache bytes
are preserved. Local results and the initial recovery snapshot remain under ignored
`work/windows-dev-signing/` and `work/acceptance/`.

The initial custom enforcement exposed a compatibility failure that the brief
idle audit did not exercise: unsigned pinned Cargo and the compiler DLL lost their
consumer SAC authorization. A separate supplemental policy therefore records exact
hashes for the pinned Rust toolchain/shims, QEMU runtime and xorriso runtime.
It admits only their user-mode bytes, leaves the files unchanged and refuses
silent regeneration. New/changed tools require a reviewed policy update. Exercise
the toolchain and VM tools during future audit migrations, not only idle desktop
activity. Keep this initial failure separate from final successful checks.

## Trust decision

Use a private local RSA 3072/SHA256 code-signing certificate for first-party Windows
host executables. The key lives in the developer's Windows current-user certificate
store, uses the Microsoft Software Key Storage Provider and is non-exportable.
Only the public certificate is exported into ignored `work/windows-dev-signing/`.
Public root trust is added only to this user's store so Authenticode verification
works. No certificate, PFX, credential or private key belongs in Git.

The App Control base derives from this installation's `SmartAppControl.xml`.
It preserves its Microsoft/signed/reputable baseline, removes the conditional
lockdown option reserved for SAC, and adds the exact development certificate as
a **user-mode** signer. It does not add kernel, policy-update or policy-signing
trust for that certificate. No writable-directory allow rule or antivirus exclusion
is added. Antivirus stays enabled. Policy binaries stay unsigned for manageable
removal; advanced boot options and boot-audit-on-failure remain enabled.

This is a machine-wide policy choice. Other Windows users can run code signed by
this signer. Any process with access to this user's key can potentially sign code;
non-exportable storage and runner path checks are not process isolation or hardware
key protection. It is a development identity, distinct from product release, OS
update, Secure Boot and guest trust. No public certificate purchase or cloud signing
service is needed for this workflow. Existing reputation services are inherited
from the Windows baseline; no project upload or remote signing is introduced.

## Setup and verification

Run these as the normal developer in PowerShell 7 from the repository root:

```powershell
pwsh -NoProfile -File tools/windows/Initialize-DevSigning.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/windows/New-DevPolicy.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/windows/New-DevToolsPolicy.ps1
```

The execution-policy flag affects that Windows PowerShell process only; it does
not change the machine execution policy or bypass Code Integrity. ConfigCI needs
Windows PowerShell 5.1. The initialization retains an existing identity; policy
generation refuses to replace an existing manifest silently.

Review `audit.xml`, `enforced.xml`, `policy-manifest.json` and the public certificate
in ignored `work/windows-dev-signing/`. In an elevated Windows PowerShell:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/windows/Set-DevMachinePolicy.ps1 -Action Audit
# Inspect admin-audit.json, backup and CodeIntegrity Operational audit events.
powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/windows/Set-DevMachinePolicy.ps1 -Action Tools
# Exercise pinned compiler/Cargo/VM tool version checks in audit before Enforce.
powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/windows/Set-DevMachinePolicy.ps1 -Action Enforce
```

Audit captures policy inventory, current active policy files, SAC state and
antivirus state before adding the custom audit policy. Enforcement activates the
replacement before setting SAC off and refreshing with CiTool. It verifies the
replacement is active and SAC enforcement has retired; it does not touch other
policies or reboot Windows. A failed step writes `admin-<action>.json` and stops.
Inspect active policies rather than assuming a registry write completed migration.

Then, from the normal developer account:

```powershell
pwsh -NoProfile -File tools/windows/Initialize-DevCompiler.ps1
pwsh -NoProfile -File tools/windows/Test-DevSigning.ps1
pwsh -NoProfile -File tools/windows/Test-DevCompiler.ps1
. ./tools/Enter-Dev.ps1
cargo test --locked -p xtask
```

Initialize the compiler wrapper before running either preflight. The executable
preflight compiles two different fresh host probes, signs/verifies each, and
tests execution, argument boundaries, exit propagation, path/name restrictions and
signature digest failure after controlled tampering. Run it before expensive
acceptance work and after policy, key, account or Windows changes. It fails early
when the key is missing or within 30 days of expiry. The runner signs/verifies
each declared host executable immediately before running it and records original
and signed SHA256 hashes separately. Signing changes host executable bytes; guest
ELF/ISO artifacts are not signed by this workflow. Logs/probes remain ignored.

After successful enforcement and compiler initialization `Enter-Dev.ps1` enables
Cargo's Windows-target runner and the signed compiler wrapper in that process only,
including inherited nested Cargo calls. Initialization writes the runner argument
array to ignored `work/toolchains/cargo/config.toml`, refuses to overwrite an
unrecognized existing configuration, and hashes it in the local compiler manifest.
`Enter-Dev.ps1` verifies that manifest and enables the local Cargo home and wrapper;
it does not write configuration or change system/user-profile Cargo settings.
The runner accepts only the
first-party `xtask` and `continuum_kernel` test executable names in declared build
roots, plus the explicit local preflight probes. Paths containing spaces are
supported. The compiler wrapper preserves native argument vectors, calls the pinned
compiler and signs its Windows EXE/DLL outputs before Cargo can execute or load
them, including fresh build scripts and proc macros. It requires local Rust source,
declared build roots and unchanged enrolled project/experiment lockfiles. The clean
compiler preflight exercises a build script, proc macro, unit test and Cargo run
from paths with spaces. New dependency/lockfile changes require explicit review and
reenrollment using `Initialize-DevCompiler.ps1`; generated outputs alone never grant
new toolchain trust. Source bytes under the developer's control remain executable
development input, not an independent security boundary. No system Cargo config is
changed. Direct `.exe` launches do not use the runner.
Other developer accounts need their own signing setup and approved signer policy.
Sandboxed identities can have a different current-user certificate store. Run
authorized host checks in the enrolled developer-account execution context; a
missing sandbox key is a blocked prerequisite, not a reason to export the key or
broaden policy. The existing xorriso sandbox limitation remains separate.

## Recovery and lifecycle

The initial recovery snapshot is `work/windows-dev-signing/before-policy-change/`.
Preserve it and the XML/manifests locally outside disposable build outputs. Do not
copy or export the private key. An elevated `-Action Status` captures live policy
inventory. `-Action Rollback` requests restoration of captured enforced SAC,
refreshes and verifies SAC is active **before** removing this custom unsigned
policy. If SAC cannot reactivate on this Windows version/state, it retains custom
enforcement and stops; resolve the reported state before removal. Never remove
the replacement while relying solely on an unverified registry value. On a boot
failure use Windows recovery/advanced boot options and Microsoft's documented
unsigned-policy recovery procedure; have the machine's normal recovery access
available. No automatic reboot or blanket allow policy is supplied.

One-year certificates are not timestamped over the network. The 30-day expiry
gate intentionally stops signing before expiry. Rotate in advance: create a new
non-exportable identity, add its public signer to a higher-version audit/enforced
policy while retaining the old signer, validate fresh signed probes, switch local
state to the new thumbprint, rebuild host programs, then remove the old signer in
another higher-version policy. Keep each policy's reviewed XML, hashes and rollback
copy; never rerun initial generation to replace a deployed policy ID. Lost keys
require the same new-signer enrollment. A compromised key requires removal of that
signer from policy, removal of its current-user root and private certificate, and
a replacement identity. Do not reuse this development key for distributed releases.

## Primary references (checked 2026-10-20)

- [Microsoft SAC code signing](https://learn.microsoft.com/en-us/windows/apps/develop/smart-app-control/code-signing-for-smart-app-control): local self-signing alone does not grant consumer SAC trust.
- [Microsoft starter App Control policy](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/design/create-appcontrol-policy-for-lightly-managed-devices) and [example policy restrictions](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/design/example-appcontrol-base-policies).
- [SAC to App Control management](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/appcontrol) and [CiTool commands](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/operations/citool-commands).
- [Unsigned policy removal/recovery](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/deployment/disable-appcontrol-policies).
- [Cargo runner scope](https://doc.rust-lang.org/cargo/reference/config.html#targettriplerunner).
