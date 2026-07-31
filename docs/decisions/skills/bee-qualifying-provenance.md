# Provenance — bee-qualifying body rules

The router body states its rules bare (provenance exile, skill-token-diet D8). This
table maps each body rule to the decision(s) that authorize it and the rationale in one
line. Long-form records: `docs/history/backlog-auto-triage/CONTEXT.md` (source of D1-D17,
imported verbatim from the `herdr-gateway--wt--backlog-auto-triage` exploring session),
`.bee/decisions.jsonl` (via `bee.mjs decisions search`).

| Body rule | Decision IDs | Rationale |
|---|---|---|
| Qualifying exists as its own automatic front-door stage, distinct from human-interactive `bee-exploring` | D1 | Today's single-entry `bee-exploring` always blocks on Socratic questions, stopping any pipeline from running unattended. |
| Gather-first: never assess from the raw backlog row alone | D2 | Matches exploring's existing "quick scout" discipline; a decision made without gathering evidence first is not trustworthy. |
| Self-assessment is LLM judgment, explicitly not a keyword/regex classifier; zero-match is not proof of safe | D3 | User rejected script-only classification, citing `herdr-orchestrating`'s `classify-lane.mjs` as the anti-pattern: it fails open on any row whose danger isn't spelled in its keyword list. |
| Clear-item auto path: hand to `bee-context-locking`, auto-approve Gate 1, run `bee-planning`, auto-approve Gate 2, mark in-flight | D4 (bounded by D7's gate-bypass coupling) | Deliberate automation increase for the case triage judges genuinely unambiguous. |
| Park path: no synchronous question — write a brief into CONTEXT.md's existing `Outstanding Questions` section, then stop | D5 | Reuses existing CONTEXT.md structure instead of a second artifact format; headless exploring already has direct precedent for this. |
| Hard-gate flag set (auth, authorization, data loss, audit/security, external provider, validation removal) always parks, regardless of confidence | D6 | Mirrors the same user's `agent-pane-orchestration` D6 precedent ("when unsure, refuse"), extended to "hard-gate is never auto-cleared, full stop." |
| Gate 1/2 auto-approval coupled to the actual `gate_bypass_level`, never an independent bypass channel | D7 | Avoids a second, parallel safety-control source; turning the global bypass off must also stop triage's auto-approval. |
| `bee-context-locking` is the single writer of CONTEXT.md for both the auto and human paths — qualifying never writes it directly | D8 | Avoids two divergent implementations of "how CONTEXT.md gets written" (DRY). |
| Human path (`bee-exploring`) loads the qualifying brief instead of re-gathering when it later resumes a parked item | D9 | Avoids redundant gather work; the brief is exactly the input exploring needs to resume. |
| Qualifying is tool-agnostic — any orchestrator drives it by invoking bee skills as sequential stages | D10 | Explicit user principle: the skill layer must support whichever tool orchestrates, not be owned by one. |
| Skill name `bee-qualifying`; scope is gather+judge only | D11 | Captures both halves — gathers/enriches evidence (D2) and judges go/no-go (D4/D5) — the sense chosen after a naming brainstorm over rejected alternatives. |
| Auto-trigger wiring (what invokes qualifying on a new row, and when) is out of scope for this skill | D12 | Deferred to a future feature; qualifying only defines correct behavior once invoked, orchestrator-agnostic per D10. |
| Park path sets backlog `Status` to `parked` via `bee-context-locking`, same commit as the brief | D13 | Without a distinct status, a parked item looks identical to a never-touched row to any future auto-trigger — re-triggering, re-parking, forever, without surfacing to a human. |
| Qualifying assumes it runs inside its own isolated worktree | D14 | Both the clear and park paths write files (CONTEXT.md, docs/backlog.md) multiple concurrent items could touch; direct-to-main risks blocking other worktrees' merges or a lost-update race. |
| Qualifying never merges its own worktree into main | D16 | Merging into main is the one hard-to-reverse action in the pipeline; no stage assumes it happens automatically. |
