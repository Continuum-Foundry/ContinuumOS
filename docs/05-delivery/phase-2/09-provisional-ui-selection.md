# P2-09 — Record the provisional UI toolkit and integration decision

Issue type: Story. Epic: P2. Status: Not started. Owner: Unassigned.
Depends on: P2-08 Done. Suggested labels: `phase-2`, `decision`, `ui-toolkit`.
Backlog: B36 decision, B07 direction. Requirements: R08/R09/R15/R18; R04/R19 design.
Test families: T13 review of hosted T02/T05/T07/T09/T10 evidence.

## User story

As an implementation agent, I need one evidence-backed provisional UI direction
so subsequent work finishes a coherent core rather than maintaining two toolkits.

## Implementation scope

1. Review every comparison axis and requirement against P2-07/P2-08 evidence.
   Select a provisional toolkit/backend/feature/asset configuration, with explicit
   mandatory criteria, accepted prototype limits and bounded completion work.
2. Require actual freestanding final-link feasibility for the chosen path, a concrete
   guest surface/input/semantics integration plan, and supported routes for resources,
   layout and eventual compiled behavior replacement. A host interpreter is not a
   substitute for a guest-compatible path.
3. Resolve applicable distribution terms for the intended project/toolkit/API use
   before commitment. Identify exact licenses, notices and any required authorization;
   unresolved suitability blocks commitment rather than receiving an assumed clearance.
   Do not contact vendors or upload project details without separate authorization.
4. Update ADR-0011 and affected rendering/UI/action specifications with measured
   rationale, alternatives, remaining risks, evidence links and reconsideration gates.
   Keep ADRs Proposed until their full gates pass; M4 must still confirm the guest.
5. Freeze the unselected prototype as comparison/test material and define the chosen
   core's source/adapter ownership. List concrete remaining M2 cases for P2-10/P2-11
   and M3/M4/M5 dependencies without silently deferring M2's required behaviors.

## Acceptance criteria

- [ ] A recorded review names one provisional path with exact versions/features and
  evidence-backed tradeoffs across every comparison axis.
- [ ] Chosen target symbol closure, required scripts/semantics/effects and reload
  completion paths are credible; a failed mandatory criterion blocks the decision.
- [ ] Intended distribution terms and notices are resolved for commitment, with
  unresolved rights questions explicitly blocking adoption where applicable.
- [ ] Specs/ADR rationale, dependency inventory and completion checklist agree;
  architectural acceptance and M4 guest confirmation are not falsely marked complete.
- [ ] Neither two full production toolkits nor a changed core requirement is implied;
  the next bounded implementation story and any blockers are explicit.

## Tests and evidence

Review the full comparison, exact locks/feature graphs and missing-feature inventory.
Check links, traceability and notices consistency. If neither path meets mandatory
criteria, preserve a blocked decision and a bounded follow-up experiment; do not
choose a winner by reducing the corpus, dropping state continuity or switching to
a browser/Linux shell. Record reviewer identity and distinguish self-review from
independent review.

## Exclusions and handoff

No broad widget investment before this gate and no final guest toolkit acceptance.
P2-10 receives the selected implementation/configuration and an explicit M2 completion
list. Sources: [ADR-0011](../../04-decisions/0011-ui-authoring.md),
[ADR-0008](../../04-decisions/0008-rendering-path.md),
[reconciled direction](../../00-overview/reconciled-plan.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
