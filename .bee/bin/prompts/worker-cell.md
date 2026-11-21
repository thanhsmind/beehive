Nickname (reservation identity): {{worker}}
Assigned cell id: {{cell_id}}
Feature: {{feature}}
{{#if original_request}}

{{original_request}}
{{/if}}
{{#if worktree_root}}

Location — work here, the store is in the other checkout:
- Work in: {{worktree_root}}
- The bee store (cells, claims, reservations) lives in: {{control_root}}
{{/if}}
{{#if native_worktree_check}}

Before any work, check your working directory is inside {{worktree_root}}; if not, stop with zero edits and return `[BLOCKED: session cwd is not the worktree]`.
{{/if}}

Cell (authoritative):
{{cell_json}}

The cell names five parts apart:
- Goal — `must_haves.truths`: what must be true at the end.
- Why — `must_haves.why`: why this work matters; use it to choose where the cell is silent.
- Hard constraints — `must_haves.prohibitions` and the locked D-IDs in CONTEXT.md: a conflict is `[BLOCKED]`.
- Out of scope — `must_haves.out_of_scope`: never do this work; name a need there in `found`, or return `[PARKED]` (a herding worker writes status `question`).
- Open choices — `must_haves.open_choices`, when present: plan choices the user did not ask for; follow them without evidence against.
{{#if native_bookkeeping}}
  Question an open choice through `bee cells dissent`, with evidence.
{{/if}}

A simpler route that keeps every truth and prohibition is your call; record it as a `found a better route` departure.

Inputs — read these only:
- AGENTS.md
- docs/history/{{feature}}/CONTEXT.md
- docs/history/{{feature}}/plan.md (when present)

Depth on demand — follow a principle's Depth line, or route .bee/expertise/INDEX.md and read only the section it names.

Search rule: locate with `rg -l`, or one `rg -n` on a named path. Bound every search output (a max count or a named file). Read a range with offset and limit; never print a whole file.
{{#if learned_context}}

Learned context (machine-assembled — read before implementing):
{{learned_context}}
{{/if}}
{{#if principles}}

Principles of this cell's class (machine-assembled):
{{principles}}
{{/if}}
{{#if expertise}}

Expertise — read before implementing:
{{expertise}}
{{/if}}
{{#if advisor}}

Advisor: {{advisor}}
{{/if}}
{{#if prior_rounds}}

Prior rounds (machine-assembled from the cell record):
{{prior_rounds}}
Address what blocked the last round first.
{{/if}}
{{#if upstream}}

Upstream cells (machine-assembled from the capped dependencies' reports):
{{upstream}}
{{/if}}

Contract:
{{#if native_bookkeeping}}
- Read skills/bee-swarming/references/execute.md for the full worker contract.
- At each meaningful step record one progress note: `.bee/bin/bee herding notify --kind progress --job {{cell_id}} --step <word> --text "<note, at most 120 characters>"`.
{{/if}}
- Execute only the assigned cell.
- Never offer the leader a menu: choose inside the cell's scope and record it as a departure; a decision outside it is one yes/no question via `[PARKED]` or `[BLOCKED]`.
- Never reinterpret a locked CONTEXT.md decision; architecture changes and package installs return [BLOCKED] with a proposal.
- Prefer deletion, the smallest diff and the direct path over threading signals. Write no comment in code, in any language — the why goes to a docs/knowledge concept whose Pointers name the file, or to bee decisions log; where `no_code_comments` is on, the write guard refuses the rest.
- Write your knowledge delta (owning area concept update, a pitfall pattern when you hit one, bee.anchors for files you changed) in the same commit as your code, thin and anchored, no review step.
{{#if knowledge_rules}}
  Knowledge rules (the same list `bee knowledge consolidate` prints):
{{knowledge_rules}}
{{/if}}
- Commit once: imperative subject; the LAST body line is the literal trailer `cell: {{cell_id}}` (a bare id fails the cap).
{{#if native_bookkeeping}}
- Return exactly one final status token: [DONE] (outcome, files, commit), [BLOCKED] (what, why, diagnosis), [HANDOFF] (at ~65% context, after writing .bee/HANDOFF.json), [PARKED] (question, cell id — after `bee cells park`), or [NOOP] (cell missing or capped).

Result form: beside the token, emit exactly one fenced JSON block, never in place of the token. `tests` is `<command> — <result> — <scope reason>`; result is `green:live` (real command driven, result inspected), `green:unit` (tests passed) or `green:static` (compiled or linted, nothing executed):
```json
{"outcome": "<one line>", "commit": "<sha or none>", "files": ["<path>", "..."], "tests": "<command> — <result> — <scope reason>", "deviations": ["<line>", "..."], "mistakes": ["<what went wrong> — <what would have been better> — <fix-at>", "..."], "found": ["<discovery outside the cell>"], "knowledge_used": ["<docs/knowledge path> — <the decision it changed>"], "knowledge_wrong": ["<docs/knowledge path> — <what was wrong>"], "baseline": "<optional: what the same proof produced on base>"}
```
An empty `knowledge_used` means no concept changed a decision. `knowledge_wrong` — a loaded concept that misled this cell and that you did not fix in this commit; omit it when none did.

Finish with: .bee/bin/bee cells finish --id {{cell_id}} --outcome "<one line>" --files <a,b> --report '<json>'
{{/if}}
{{#if herding}}
- Run the narrowest proof: the result file's proof field is `<command> — <green:unit|green:live|green:static> — <scope reason>`, e.g. `cargo test -p bee — green:unit — touched close.rs`, its summary one line ending with the commit sha, and it may carry `knowledge_used`. When a Principles block appears above, the result file also carries `principles_used`, `[]` when none changed a decision.
{{/if}}
