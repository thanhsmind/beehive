---
type: grilling
status: closed
claimed-by:
blocked-by: none
---

## Question

bee workers today execute exactly one cell and do not argue. Where
does a worker's Dissent land (nearest: `bee cells escalate`, cap
concerns), what OBLIGATES the orchestrator to answer one-of-three
(accept + log / reject with reasoning / escalate a rung), and what
does StopAndAsk look like mid-cell (nearest: waiting-on question,
blocked cell)? Boundary signals to write into worker instructions:
contract/API changes, trading data quality or UX for a technical
target, new dependencies.

## Answer

User half (D 4b7aa303): blocker dissent pauses the RELATED slice only,
and the orchestrator is obligated to a logged one-of-three response
(accept / reject with reasoning / escalate a rung) before that slice
resumes. Mechanism half (D a2affcba): dissent lands on the cell via a
new CLI record {target, claim, alternative, severity}; the obligation
is enforced the judge-debt way — `bee close`/`bee worktree merge`
refuse while a dissent lacks its verdict; blocker severity rides the
existing blocked-status machinery; StopAndAsk = the herding
round-mailbox shape plus options[]+leaning on the [BLOCKED] form; no
live mid-flight Q&A channel. Key findings: consider-grade dissent has
NO carrier today (cap report has no concerns key, worker prose is
read-banned), and `bee cells escalate` already means model-tier — the
SLP escalate verb must not reuse that name. Findings:
docs/history/research/slp-dissent-surfaces.md.
