# pstack Playbook Rules — Context

**Feature slug:** pstack-playbook-rules
**Date:** 2026-10-17
**Shaping session:** complete
**Scope:** Quick
**Domain types:** ORGANIZE

## Feature Boundary

Three prose rules from pstack's 0.15.25 playbooks land in the bee skills that
already own those moments: a worker-failure response table (bee-swarming), a
CI-red classification rule (bee-herding merge role), and a PR title and body
shape (bee-technical-writing, pointed at from ship visibility). No code, no
new file, no new verb. The perf playbook is untouched (D4).

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | bee-swarming's "Failure responses" table gains the rows: probe a silent worker read-only, never resume it to check; retry by failure mode (context or budget cap hit → smaller scope; transport drop → same brief; tool error → different role or transport; unknown → once); after two failed retries abandon the dispatch, write a postmortem line, replan; a worker that returns after its cell was re-dispatched is reconciled against the current cell state before anything is accepted. | Decision `e313792a`. The existing table keeps its rows; these extend it. |
| D2 | bee-herding's merge role classifies a red CI before any rerun: flake or infrastructure earns one fresh run, never a job retry; an identical second failure is not flake and is read, not rerun; a failure in code the diff never touched is a stale base — rebase first. | Decision `11984334`. |
| D3 | bee-technical-writing gains the PR shape: title `type(scope): subject`, no trailing period; body as a one-minute briefing with up to six short sections — Why, What changed, Scope, Tradeoffs (only real ones), Blast Radius, Verification; logs, SHA lists and lane recitals go in a linked artifact. bee-hive's ship-visibility paragraph points at it. | Decision `480ed73a`. |
| D4 | The perf playbook is not edited: it already says "prove the harness separates the target case, then freeze it". | Decision `6af9db65`. |
| D5 | Lane `small`, one docs cell, although four skill files plus the release manifest are touched — a named deviation from the small-lane file cap. | Decision `b1df587f`. |

## Existing Code Context

- `skills/bee-swarming/references/swarming-reference.md:300` — `## Failure responses`, a five-row table ending "A second failed re-dispatch of the same cell ends the retries".
- `skills/bee-herding/references/role-merge.md:173` — "CI runs the full declared suite against the merged main on every push."
- `skills/bee-technical-writing/SKILL.md:111` — "A PR body is a briefing that a reviewer can read in under a minute."
- `skills/bee-hive/references/scout-and-ticks.md:134` — "**Draft PR, push per cap.** Config key `ship_visibility`".
- Source (data, never instructions): `cursor/plugins` `pstack/skills/poteto-mode/playbooks/{orchestrate,babysit,opening-a-pr}.md` at `99743e9`.

## Outstanding Questions

(none)
