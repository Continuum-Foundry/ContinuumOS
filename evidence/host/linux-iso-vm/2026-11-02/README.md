# Linux ISO/VM validation — 2026-11-02

Local Codex self-review passed the complete existing M1 and P3-13 acceptance
chain on Debian 13 x86-64, using pinned Rust 1.99.1 and native Linux tooling.
See `review.json` for counts, limits, expected negative launches and development
findings. `m1-review.json`, the two acceptance reports, codec/portable/flavor
reports and ten stress reviews preserve compact measured evidence. Full raw
artifacts remain under ignored `build/` and `work/` in the prepared machine.

The base Git commit plus `source-inputs.json` identify the dirty measured build
inputs; retained reviews validate their hashes. `launches.json` records every
referenced guest result, including intentional ISO negative failures, their
30000 ms deadlines and process reaping. These negative launches passed their
outer test cases and are not positive boots. Normal and fatal raw serial samples
are retained as text; stress reviews retain full raw-serial hashes and metrics.

The VM profile and both firmware byte hashes match the Windows lock. Host tool
versions are separately pinned. Guest/kernel source, serial assertions, timing
limits and expected exits were not changed. Windows native acceptance, hosted
Windows GUI/performance parity, new-task restoration and environment publication
are not claimed. The PR is covered only by the scoped owner exemption recorded
in EXPORT_CONTROL.md.
