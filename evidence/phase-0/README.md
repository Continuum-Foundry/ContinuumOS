# Phase 0 evidence

Recorded on 2026-10-18 (America/New_York). These are compile/link and host-tooling
results, not an OS boot or runtime demonstration. See the
[runbook](../../docs/05-delivery/phase-0.md) for reproduction and limitations.

- `environment-report.json`: sanitized Windows/tool inventory; missing VM/ISO tools and blocked CIM access are explicit.
- `toolchain-manifest.json`: tested compiler/linker identity, input archive hashes and reviewed boot candidates.
- `m0-results.json`: two independent clean builds, artifact hashes and structural inspection.
- `acceptance-results.json` and `doctor-tests.json`: full source path with spaces and override tests.
- `dependency-report.json`: resolved host/guest dependency metadata and per-candidate compile/link outcomes.
- `logs/`: successful compile/inspection logs and dependency failures, including the initial corrected Slint pixel-type error.
- `SHA256SUMS.json`: hashes of retained evidence files, excluding this checksum index itself.

Regenerate deliberately with `tools/Record-Evidence.ps1` after running the checks;
it rejects stale build/dependency source hashes. Text evidence is normalized to LF
so its checksum index remains meaningful after a Git checkout.
The initial rustup permissions and host digest-formatting failures are retained as
explicitly labeled excerpts transcribed from their tool output.

Generated binaries and complete working directories remain ignored under `build/`
and `work/`. Rebuild them using the locked inputs; they are not release artifacts.
No kernel boot, runtime allocator, Slint backend, ISO, VM acceleration, guest
performance, process isolation or live reload is claimed. Rustup was installed
only inside the workspace; existing Visual Studio and SDK installations were reused.
