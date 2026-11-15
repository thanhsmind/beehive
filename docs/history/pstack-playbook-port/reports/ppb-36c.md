# ppb-36c: check of the pstack verification-skill merge in ppb-36

Checked against commit `7cc3065dc` (ppb-36). The sources are the pstack skills
`create-verification-skill` (`SKILL.md` and `references/feature-map-example/`)
and `maintain-verification-skill` (`SKILL.md`) under
`/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/`. The bee
side is `skills/bee-verifying/` and `skills/bee-verify-upkeep/`. No file other
than this report was edited.

Status key: **P** present in the bee skill; **L** left out, with the bee home or
the bee difference named; **M** missing.

## Verdict

The merge holds. Every source rule is present, or left out with a named bee home
or bee difference. None is missing. The split matches
`port-notes/merge-ppb-36.md`.

- **pstack-only tool names:** none. `rg -i 'cursor|graphite|bugbot|poteto|subagent_type|model:'`
  over `skills/bee-verifying/` and `skills/bee-verify-upkeep/` finds no match
  (exit 1). No `.cursor/` path, no PR wording and no pstack slash name
  (`/create-verification-skill`, `/maintain-verification-skill`) remains in
  either `SKILL.md`. The `disable-model-invocation: true` frontmatter key is a
  shared skill-format key, not a pstack tool.
- **bee rules removed:** none. `git diff 7cc3065dc~1 7cc3065dc -- skills/` adds
  only the two `LICENSE.pstack` files. Neither `SKILL.md` nor
  `references/` under `skills/` changed, so every bee rule is still there:
  the constant `.bee/verify/verify-app/` source path, the render rule, the
  `bash <path>` helper rule and its reason, the fast and full drive contract,
  the exit-code contract, the `commands.test` ban, When a run is due, Entry
  points and Record the run.
- **Other checks:** both `LICENSE.pstack` files are byte-identical to the pstack
  `LICENSE`. `SKILL.md` and `LICENSE.pstack` of both skills match the source byte
  for byte in all five rendered trees (`.claude-plugin`, `.codex-plugin`,
  `.agents`, `.opencode`, `.claude`). The three files in
  `skills/bee-verifying/references/feature-map-example/` are byte-identical to
  the pstack ones. Descriptions are unchanged at 273 (bee-verifying) and 274
  (bee-verify-upkeep) characters, under 300.

## create-verification-skill → bee-verifying

### Frame

| Source rule | Status | Note |
|---|---|---|
| Frontmatter name and description | L | bee differs: bee routes by its own description, which names the constant `verify-app` and the upkeep split |
| `disable-model-invocation: true` | P | |
| Every serious project needs a scripted way to drive the real app and prove behavior | P | intro |
| Output is a project-local skill at `.cursor/skills/verify-<app>/` | L | bee differs: constant source path `.bee/verify/verify-app/`, rendered to every runtime ("Where the generated skill lives") |
| Write for the next agent, read cold mid-task | P | intro |

### 1. Interview the repo, not the user

| Source rule | Status | Note |
|---|---|---|
| Answer from the codebase; ask the user only what you cannot observe | P | |
| Surface: pick the primary one, note the rest | P | |
| Run: prefer the repo's own dev command; note ports, env, seed data, auth | P | |
| Drive: existing harnesses first, then a generic recipe | P | |
| Observe: what evidence can be captured | P | |
| Isolate: if two instances cannot run side by side, say so; refuse to double-drive | P | |
| Broken checkout: fix it or report precisely before generating | P | |
| Irrelevant missing asset may be created as marked scaffolding and removed in cleanup | P | |

### 2. Generate the skill

| Source rule | Status | Note |
|---|---|---|
| YAML frontmatter with name and a description naming app, surface, when to use; without it the skill never registers | P | name is the constant `verify-app`; the description carries the identity |
| Every section grounded in the interview, no placeholders | P | |
| Launch: exact command, readiness signal, teardown | P | |
| Launch for a short-lived CLI/TUI: build once, one isolated PTY or tmux session per drive | P | |
| Doctor: one read-only "worth driving?" check, run first when anything looks off | P | |
| Drive: real selectors and commands, stable handles over coordinates | P | bee adds the fast and full scopes |
| Evidence: real user path, not setters or test-only endpoints | P | |
| Evidence: capture action and resulting state | P | |
| Evidence: verify side effects alongside what is visible | P | |
| Evidence: mocks only where a production boundary already isolates | P | |
| Evidence: observe what a dry-run actually skips | P | |
| Cleanup: never kill by process name; kill what you started | P | |
| Cleanup keeps the evidence at a named location | P | |
| Helpers: executable, invocation shown in the body; no reverse-engineering | P | bee's longer rule adds the `bash <path>` prefix and the 0644-render reason |

### 3. Seed the feature map

| Source rule | Status | Note |
|---|---|---|
| `features/README.md` plus one file per feature, top 3 to 5 | P | path under `.bee/verify/verify-app/` |
| Follow `references/feature-map-example/` | P | the three example files are byte-identical |
| Each file: what, how to reach, how to drive, observable end state | P | |
| The four H2s | P | |
| The map is the maintained source; one convenient entry point is incomplete | P | |

### 4. Prove the generated skill

| Source rule | Status | Note |
|---|---|---|
| Run it end to end once: launch, doctor, drive ONE feature, evidence, cleanup | P | |
| After cleanup, confirm evidence still exists | P | |
| Run cleanup after every failed iteration | P | |
| A never-executed skill is a draft | P | bee adds the `bee onboard --verify-only --apply` render step |

### 5. Offer the maintenance loop

| Source rule | Status | Note |
|---|---|---|
| Point the user at the maintenance skill | P | as section 6, naming `bee-verify-upkeep` instead of `/maintain-verification-skill` (D2 swap) |
| Suggest a cadence only if asked | P | |

## maintain-verification-skill → bee-verify-upkeep

### Frame and outcomes

| Source rule | Status | Note |
|---|---|---|
| Frontmatter name and description | L | bee differs: own description, upkeep split from bee-verifying |
| `disable-model-invocation: true` | P | |
| A feature map rots; upkeep loop for a generated or any mapped verification skill | P | names `bee-verifying` |
| Unit of rigor is the feature: every feature from source and live | P | |
| Pick one outcome and say which | P | |
| clean: full coverage, nothing to ship, no branch, no PR | P | "No worktree, no cell" (bee counterpart) |
| changed: one PR of proven corrections | P | "one worktree and one cell" |
| blocked: say exactly what blocked it | P | |

### Edit scope

| Source rule | Status | Note |
|---|---|---|
| Edit only the verification skill's own directory | P | narrowed to the source under `.bee/verify/verify-app/`, plus the re-render rule |
| Never edit product code; drift vs regression | P | |

### Pass

| Source rule | Status | Note |
|---|---|---|
| 0. Locate the target (usually `.cursor/skills/verify-*/`) | L | bee differs: the path is a constant, so there is one candidate; step 0 "Skill present" checks it |
| 0. Several candidates → ask which one | L | bee differs: cannot happen with the constant path |
| 0. None → stop and point at the create skill, never invent a target | P | step 0 routes to `bee-verifying` |
| 1. Index hygiene, no generated inventory | P | |
| 2. Source wave: one read-only subagent per feature, concurrent | P | |
| 2. Children never drive or edit; return shape | P | |
| 3. Reconcile: every feature has a summary; merge recipes | P | |
| 3. Spot-check cited drift; don't re-prove clean claims | P | |
| 3. Sweep churn for missing surfaces, concrete source path required | P | |
| 4. Live pass required even when source looks clean | P | |
| 4. Coordinator owns driving; the skill's Launch section decides the model | P | |
| 4. Exercise every feature at least once | P | |
| 4. Invariant 1: doctor before first drive, per fresh session, after failure; reset when doctor can't see it | P | |
| 4. Invariant 2: evidence survives every cleanup, checked | P | |
| 4. Invariant 3: nothing outlives its drive; shared instance cleans residue only | P | |
| 4. Doctor failure from skill drift: fix and retry once, restart only what the fix invalidated | P | |
| 4. `verified-unreachable` needs the concrete prerequisite and the route; omitted prerequisite is drift | P | |
| 4. A harness fix is re-driven live before it ships | P | |
| 4. Final teardown after the last drive, evidence stays | P | |
| 5. Triage: doc drift → fix | P | |
| 5. Triage: harness gap → fix, same helpers rule as generation | P | with the `bash <path>` reason |
| 5. Triage: product gap → record for the user, keep out of the PR | P | "keep it out of this change" |
| 6. Ship or stop: changed = one PR, re-read every changed file first | P | step 7: one worktree and one cell, re-render, `bee worktree merge` |
| 6. Clean or blocked: no PR, report outcome and coverage honestly | P | step 7, "no worktree" |
| Concise run notes in a scratch location, not committed | P | |

## Counts

68 source rules checked. P: 62. L: 6. M: 0.

The six L rows are the two frontmatter descriptions, the `.cursor/` output path,
and the three Locate-the-target rows that the constant path makes moot; for the
last, the "none → stop" part is P. The port notes name all of them.

## Suspicions that failed verification

- The upstream summary names commit `1788ccfd9` for ppb-36. That commit is the
  ppb-35c technical-writing check and touches only its report;
  `git log --format='%h %(trailers:key=cell,valueonly)'` shows `7cc3065dc` as the
  ppb-36 commit.
- The ppb-36 commit diff shows +136 and +189 lines for
  `.claude/skills/bee-verify-upkeep/SKILL.md` and
  `.claude/skills/bee-verifying/SKILL.md`. That looked like a rewrite, but the
  previous `.claude/skills` copies were 11-line Skillstation hub stubs (a
  `skill_get` pointer, no rules), and the render replaced them with the full
  source. Both now match `skills/` byte for byte, so no rule was added or
  dropped.
- `disable-model-invocation` matches the `model:` part of the scan pattern by
  eye, not by `rg` (the pattern needs `model:` with the colon right after).
  It is a shared skill-format key, not a pstack-only tool.

## Side note

The ppb-36 commit subject reads "Check the pstack verification-skill merge…",
the wording of a check cell, though it is the merge (trailer `cell: ppb-36`).
It changes nothing in this verdict.
