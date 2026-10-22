# Browser feasibility and route decision

Research review: 2026-10-21. Status: sourced constraints and proposed M17/B60
experiments. No browser port, foreign browser execution or secure browsing result
exists. M8/M14 do not require browser delivery. See the
[Windows compatibility study](windows-compatibility-feasibility.md) and
[milestone gate](../05-delivery/milestones.md#m17--browser-feasibility-and-selection-gate).

## Findings and interpretation

Chromium's Windows sandbox relies on OS-provided security and specific process/
object mechanisms. Its documentation describes restricted tokens, jobs, desktops
and integrity/mitigation behavior. Executable startup therefore does not establish
the required isolation semantics on another kernel. A no-sandbox launch cannot
serve as browser acceptance. [S95](sources.md#s95)

Upstream build documentation lists maintained platform workflows; none establishes
a ContinuumOS backend. A native route requires its own host-hook/build inventory.
This is an inference about this project, not a claim that porting is impossible.
[S96](sources.md#s96)

The Windows route also depends on the actual API/runtime subset from M16. Review
that outcome rather than assuming any Windows browser will run once a simple EXE
works. A stop/replan result for Wine/EXE support still permits a native-engine study.
[S85](sources.md#s85), [S88](sources.md#s88)

## Route comparison

| Route | Required audit and probes | Decision boundary |
| --- | --- | --- |
| Native browser/engine port | Exact source/build tools; libc/runtime/thread/process hooks; IPC, files/fonts/input/surfaces, network/TLS/time/trust, executable-memory policy and crash handling | Compare a full maintained browser with an engine plus first-party shell; embedding an engine does not eliminate maintenance/security work |
| Windows browser through compatibility | Real M16 API/ABI results; DLL/runtime and graphics/media dependencies; Windows sandbox mechanism coverage mapped to native isolation | A successful launch/screenshot is insufficient; no compatibility-only security assertion |

Both routes need origin/profile isolation, secrets/cookies/storage handling,
permissions for downloads/clipboard/capture, updates and vulnerable-component
response, text/accessibility integration and measured memory. Record host and
guest build/runtime dependencies separately. Do not substitute a host browser,
remote web desktop or another kernel for a guest browser.

## M17 procedure and acceptance

B60 depends on B53/B59/B16/B09/B41. Begin with a named workload and exact supported
scope; pin candidate versions before experiments rather than adopting a moving
research URL as a dependency. Keep downloads/ports subject to their own scoped
implementation task; this plan introduces no executable fixture or engine source.

- Compare the two routes using source-backed host/API/resource inventories and
  actual M16 results. Mark all missing hooks, runtime components and maintenance owners.
- Define bounded compile/link/runtime probes that distinguish engine initialization,
  static content, text/input, networking and process isolation. Preserve failures;
  no one probe is evidence of full browsing support.
- Exercise malicious/untrusted-input boundaries with declared test content and
  denied file/network/cross-profile access. A credible sandbox architecture and
  negative tests are required; bypassing it makes the proposed production route fail.
- Report cold/warm/active memory, CPU/frame/input costs and process-crash recovery
  against the OS budgets. A browser's active footprint is not silently excluded
  from accounting or treated as the baseline desktop cold-idle measurement.
- Record exact rights/notices, third-party runtime/build and update dependencies,
  unsupported features and the security maintenance burden before choosing a route.

Exit is a reviewed route/subset proposal or stop/replan outcome, followed by an
explicit product decision and separately decomposed browser-delivery milestone if
approved. It does not promise a date, a full browser, general Windows compatibility
or that a browser becomes a dependency of M8/M14. Retain the original kernel,
software fallback and independent ISO boot path throughout.
