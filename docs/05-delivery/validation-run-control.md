# Bounded story validation and failure recovery

Applies to routine story closeout. Full milestone/release gates and their fixed
counts, limits and evidence requirements remain mandatory at their own exits.
See [validation policy](validation.md#change-based-story-validation).

## What P4-02 exposed

P4-02 was implemented while recursive M1/M3 gates repeatedly rebuilt and booted
older cases. Several attempts completed the new feature but failed later in host
startup, inherited reviewers or old stress cases. Shared-checkout edits invalidated
another attempt. One long attempt overlapped recorded lid-triggered standby;
other timing variation remains unexplained. A timeout alone is not a diagnosis.
The owner ultimately requested bounded affected-check validation. Failed and
interrupted aggregates remain non-passes in the retained story evidence.

| Failure mode | Required prevention |
| --- | --- |
| Every story recursively repeats older milestones | Record new behavior and affected consumers; select leaf checks. Complete gates need explicit broad/uncertain impact and budget opt-in. |
| Command counts hide multiple guest launches | Preview the registered launch count per case, mandatory repetitions, metadata builds and total before execution. Unknown full-suite counts are explicit. |
| A late failure restarts everything | Preserve per-check successes and failed attempts. Resume unchanged checks with artifact hashes and independent re-review; execute failed or invalidated work only. |
| Final prose/evidence/staging invalidates successful guests | These inputs trigger host audit re-execution, while unchanged implementation and tool context permit guest reuse. |
| Two chats mutate one checkout | Use one active writer during validation. Reuse an isolated suitable checkout when needed, and prove its signer/ISO-tool context before a long gate. Do not assume a Windows signer works in a new path. |
| An obsolete reviewer fails after guest execution | Run the cheap host orchestration tests first. Diagnose the actual artifact/check mismatch before rebuilding guests; preserve the failure and exact reviewer repair. |
| Sleep or a host timeout spoils a long run | Keep long gates serial in the approved context; advise the owner to keep the host awake. Do not change power/security settings or guest deadlines. |

## Before the first guest

1. Finish implementation and review its affected consumers. Preserve unrelated
   edits. Choose one checkout and keep its implementation stable through validation.
2. Write a story selection and execution budget. Run its read-only `plan` command.
   Report command groups, actual registered VM launches and metadata builds. The
   P4-02 selected plan is 17 command groups, 16 VM launches and 16 metadata builds;
   two host suites are groups containing many unit tests. These numbers are not a
   template for the next story and are not an elapsed-time prediction.
3. Guest-bearing story plans require `execution_budget.max_vm_launches` and
   `allow_full_suites`. A bounded plan cannot silently gain a full suite or an
   unclassified new command. Unknown full-suite internals need explicit full-suite
   opt-in, a reason and a declared unbounded count; never invent a precise total.
4. Run relevant fast host/orchestration checks before expensive guests. Then run
   the selected commands serially. The plan is persisted and independently checked.
   Launch ceilings validate registered case counts; they are not a runtime process
   quota and must be maintained when a case gains internal launches.

## At the first failure

Stop and preserve the report. Classify build/host preflight, guest assertion or
bound, artifact review, and source/context drift separately. Read the failing
record and compare affected inputs; do not infer the cause from a timeout.

Repair or diagnose the failing check and select the minimum justified rerun. Do
not restart the complete milestone cascade simply because its parent failed.
Repeated unexplained failures require investigation, not retries until green.
A required failing check remains open; counts and deadlines are not lowered to
close it. Broaden selection only when evidence changes the impact assessment.

`--resume` creates a new attempt and preserves the old record. It may carry
completed checks forward only with identical implementation, manifest, tool/config
context, commands and artifact hashes, followed by focused re-review. It can also
start from a passed attempt when finishing prose/evidence/staging. Documentation,
governance and hygiene audits carry full snapshot digests and rerun when those
inputs change. They must match the final snapshot; staging requires a final index
hygiene check. Reviewer/code/config/asset changes still invalidate automatic reuse.
A reviewer-only compatibility correction needs explicit artifact/input proof;
never silently relabel an old failed aggregate as passed.

Execution inputs retain all code, schemas, configs, assets, build tools and new or
deleted implementation paths. Only retained `evidence/`, `docs/*.md` prose and root
agent/README/export-control prose are separated for host audits. Do not place
compiled code, build fixtures or protocol inputs in those excluded locations.
Documentation JSON and Markdown contracts outside docs remain execution inputs.

## PR closeout

Run the relevant host audits after final documentation and staging. Keep the
original guest acceptance bundle immutable when later host-only improvements are
added; record their scope and tests separately and prove guest inputs unchanged.
Preserve pre-commit and push guards. An owner PR exemption is scoped to its exact
reviewed payload and destination, not a standing upload or test waiver.
