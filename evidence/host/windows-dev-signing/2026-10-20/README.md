# Windows development signing — incomplete rollout

The owner authorized private local development signing and a replacement App
Control policy on this Windows installation. Branch: `codex/p1-07-phase-1-acceptance`;
base source commit: `a5c7c9c`. `source-inputs.json` hashes the final prepared source
revision accompanying this bundle. Some script guards/compiler integration were
added after the initial privileged deployments and have not run end to end.

Measured results:

- Non-exportable RSA 3072 code-signing key created in the current-user software
  key store, with public certificate trusted only by this user's root store.
- Initial SAC blocked a freshly compiled locally signed probe even though its
  Authenticode signature verified. The original P1-07 blocker remains immutable.
- Captured existing active policy files, inventory, SAC state and Defender state
  locally before deploying an audit policy. No custom-policy idle audit denials
  were observed; this short check did not exercise the full toolchain.
- Replacement base `{3898A096-AC42-483C-AA0C-8CE5F2898A7F}` version `1.0.0.1`
  activated with audit mode absent before consumer SAC enforcement retired.
  Other system policies remain present. No reboot or antivirus exclusion occurred.
- A previously compiled signed probe then executed with exact arguments and exit
  code 23. Outside-path rejection passed. Controlled modification of signed PE
  section bytes produced `HashMismatch`; the corrupted copy was not executed.
- Compatibility failure: custom enforcement blocked the pinned unsigned Cargo
  executable and Rust compiler DLL. Exact-hash supplemental rules were prepared
  for 180 current tool files without modifying their pinned bytes. It contains only
  user-mode hash rules, no directory or publisher-wide authorization.
- **Windows canceled the subsequent UAC elevation for installing this supplement.**
  The supplemental policy is not installed, compiler execution remains blocked,
  and the new compiler wrapper/clean-build tests remain uncompiled/untested. No
  complete M1 gate rerun or P1-07 acceptance is claimed.

The private key, public certificate, original policy XML/binaries, full diagnostic
logs, host probes and rollback snapshot remain local in ignored
`work/windows-dev-signing/` and `build/windows-dev-signing/`. This bundle retains
compact policy inventories, prepared-policy/input hashes, signature results and
selected relevant Code Integrity events. JSON state is sanitized for workspace
paths; private key material and full tool/source copies are excluded.

Next: the owner decides whether to retry Windows elevation for the reviewed
`Set-DevMachinePolicy.ps1 -Action Tools` helper. After deployment, initialize the
compiler wrapper, run both signing preflights and rerun/review all P1-07 rows.
See the [setup and recovery runbook](../../../../docs/05-delivery/windows-development-signing.md).
