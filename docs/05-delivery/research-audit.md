# Research artifact audit

Date: 2026-10-17. Revision 2 reconciles the reviewed plan and user-approved direction.
Scope: this documentation library and its draft contract fixtures.
This is **not OS implementation evidence**.

## Delivered material

- End-to-end architecture and product scope, requirements, feature/support matrix,
  source catalogue and language/dependency/prior-art evaluations.
- Kernel, memory/process, IPC/capability, boot/device, graphics/UI, storage/network,
  security, application/runtime, accessibility/text, lifecycle and performance specs.
- Live-update classes, transaction protocol, state/compatibility rules and failure recovery.
- Sixteen proposed ADRs with alternatives, consequences and evidence gates.
- Slint/custom comparison, typed actions/context, optional inference and separate boot recovery.
- Gated roadmap, ordered backlog, risks/experiments, requirement/test traceability
  and shared Claude/Codex/other-agent handoff.
- Draft package/update JSON schemas and explicitly synthetic example manifests.
- Root `AGENTS.md` and `CLAUDE.md` point to the same documentation.

## Checks performed

Automated checks parse all JSON, verify local Markdown link targets/source anchors,
look for leaked internal browser citation IDs, check requirement R01–R19 coverage in
the validation matrix, count/inspect ADR status and record document inventory.
`work/audit-docs.cjs` contains the local audit implementation.

PowerShell **7.6.5** `Test-Json` validated both supplied positive fixtures against
the JSON Schema 2020-12 draft schemas. Nine negative mutations were rejected:
malformed digest, missing format version, unknown package field, excessive memory,
duplicate capability, unsupported update format, empty component set, negative epoch
and oversized checkpoint declaration. Results are saved in
`work/contract-validation-results.json` in the authoring workspace and included in
the packaged handoff's `audit/` directory.

The initial local npm/Ajv scratch installation returned EPERM. The existing built-in
validator completed the checks, so no installation or approval escalation was needed.
This tooling fallback does not leave the research deliverable blocked.

Manual consistency review checked the boot-to-desktop-to-update-to-recovery path,
stable-core dependencies, distinction between guest CPU/GPU support, writer fencing,
pre/post-decision outcomes, unsaved state, irreversible effects, persistence modes,
version/ABI boundaries, draft schema semantics and future-agent instructions.
The original revision-2 audit checked the then-current 40-task dependency graph, optional AI's exclusion from
core-release dependencies, toolkit buffer/scene boundaries, memory profiles and ISO gates.
The 2026-10-21 installer/accounts amendment expanded the checker to 44 tasks
and requires B41–B44 in B35's release dependency closure while retaining optional
AI's exclusion. This does not change the historical validation results above.
The subsequent approved desktop/platform expansion makes the current checker
validate 60 distinct tasks, 22 milestone IDs, the feature matrix and M8/M14/PC
release closures with optional research excluded from VM releases. The dated
Windows/browser sources are planning research, not an executed guest port or
acceptance result. Original source/evidence reviews above remain historical.

## Limits

Link checks validate local targets, not perpetual availability of upstream web pages.
Sources have explicit O/I access and failure limits in the catalogue. Schema checks
do not validate cryptographic authentication, actual digest bytes, path safety,
state migrations, authority or compatibility graphs. The example packages have no
executable payloads. No code was compiled or booted and no UI benchmark was run.

The directory is not currently a Git repository. No commits, branches or PRs were
created. No VM or physical disk was changed. Tool availability outside PATH and WSL
distribution state were not established.

## Packaging

Canonical authoring files are the workspace's root `docs/`, `AGENTS.md` and `CLAUDE.md`.
The downloadable archive is a snapshot containing those files, audit results and
the documentation validation scripts. It excludes scratch tool caches and generated
OS binaries (there are none). The output overview explains how to start.

Treat the accompanying machine-readable inventory as the exact file/word/link count
for the packaged snapshot. Counts exclude future implementation and do not indicate
that any proposed feature exists yet.
