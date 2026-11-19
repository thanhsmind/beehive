# ddx-8 — parity and pointer check of the doctrine and docs sweep

Checked at HEAD `04f8e7bed` (ddx-1 to ddx-7 committed: d0552054b, 880a3e724,
5399ad719, 3584d45b9, 26b518542, fca3a0f5e, 428ed2d79, 04f8e7bed). The truths
come from the cell list in `plan.md`; the files were read directly, no cap
report was read.

**Verdict: BLOCKED.** The verify fails in two places: `class_playbook_parity`
refuses `steer-recipes.md` (ddx-1), and `release-manifest --check` fails
because the wave-close `bee dev regen` has not run yet.

## Verify

The cell verify, run as written:

```
PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity --test agents_block_render_parity --test rule_index_parity --test class_playbook_parity && .bee/bin/bee knowledge check && .bee/bin/bee dev release-manifest --check
```

cargo stops at the first failing test binary, so the four tests were re-run
with `--no-fail-fast` to get a result for each, and the two `bee` commands
were run on their own:

| Command | Result line |
|---|---|
| `cargo test … --test agents_block_render_parity` | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| `cargo test … --test class_playbook_parity` | `test result: FAILED. 8 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out` |
| `cargo test … --test pointer_integrity` | `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| `cargo test … --test rule_index_parity` | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| `.bee/bin/bee knowledge check` | `knowledge check: 330 concept(s) in 351 file(s), 0 OKF error(s), 0 profile error(s), 605 profile warning(s) — OK` |
| `.bee/bin/bee dev release-manifest --check` | `release_manifest --check: FAIL (1 missing, 4 added, 23 changed)` |

The failing test, `every_situation_file_matches_its_index`:

```
session: …/skills/bee-planning/playbooks/session/steer-recipes.md is not listed in …/skills/bee-planning/playbooks/session/index.md, so no router can reach it
```

The fence treats every file in `playbooks/session/` other than `index.md` as a
situation and wants a `- \`<name>\` — <triggers> — delivers: <deliverable>`
line under `## Situations`. ddx-1 linked the page with a prose line under the
list instead, and the page calls itself "a lookup for the owner, not a
situation".

The manifest mismatch is the expected state before the wave-close regen: the
one missing entry is `skills/bee-researching/references/trace-and-provenance.md`
(deleted by ddx-5), the four added are `.bee/expertise/review-author.md`,
`skills/bee-planning/playbooks/session/steer-recipes.md`,
`skills/bee-researching/references/provenance.md` and
`skills/bee-researching/references/trace.md`, and the 23 changed are files the
wave edited. The plan orders the regen at wave close but also puts this check
in ddx-8, which depends only on ddx-1 to ddx-7, so ddx-8 cannot pass until
after the regen.

## Truths

### ddx-1

| Truth | Held? | Evidence |
|---|---|---|
| feature.md has a numbered step that writes the caller's tutorial first for a package or API other code calls and checks the result against it | held | `skills/bee-planning/playbooks/feature/feature.md:34` step 4: "write the caller's tutorial first … Check the design against that tutorial before any delegate writes code, and again at verify" |
| steer-recipes.md maps one-line steer prompts to bee situations and principles | held | 22-row table "Say \| Routes to"; every situation file it names exists (`session/*.md`, `bugfix/bug-fix.md`, `perf/perf-issue.md`) |
| session/index.md links steer-recipes.md | held as written, but it fails the fence | `index.md:18` "see `steer-recipes.md`" is a prose line; `class_playbook_parity` needs a `## Situations` entry (see Verify) |

### ddx-2

| Truth | Held? | Evidence |
|---|---|---|
| autonomous-run.md checks the four preconditions before the loop runs | held | step 1, lines 5–10: done once by hand, the agent has the tools, every stage can stop the line, repeated failures already became checks; "stop on any no" |
| role-bootstrap.md has the same check as a pre-flight step | held | `### 4. Pre-flight — the four trust preconditions` (lines 50–62), after `### 3` bypass-level check; later steps renumbered 5–7 |

### ddx-3

| Truth | Held? | Evidence |
|---|---|---|
| LEADER.md: the leader triages every proposed PBI (build, park or drop); only money, publishing or irreversible calls go to the owner, decision-ready | held | `packages/bee/LEADER.md:128-136` "## Triage proposed PBIs" |
| owner-brief-contract.md row 5: Your call carries the leader's verdict, not an open choice | held | row 5 (line 61): "The leader's verdict (`build`, `park` or `drop`) … never an open choice for the owner" |
| bee-shaping first triage: the leader decides and records the verdict | held | `skills/bee-shaping/SKILL.md:117-120` Qualify step 5 "The leader decides and records the verdict itself" |

`packages/bee/AGENTS.block.md` holds no triage text (`rg -i "triage|proposed pbi"`
finds nothing), so leaving it unchanged matches the cell action.

### ddx-4

| Truth | Held? | Evidence |
|---|---|---|
| routing-and-contracts.md § Question Format states the policy | held | line 310: "ask one question at a time when an answer can reframe the scope … Ask a numbered batch in one round when the questions are independent of each other" |
| shaping-reference.md and bee-wayfinding/SKILL.md point to that section and carry no conflicting rule | held | `shaping-reference.md:26-27` and `bee-wayfinding/SKILL.md:65-66` point to "Question Format"; wayfinding's frontier is defined as questions askable "without guessing an answer not yet heard", which is the independent-batch case. `shaping-reference.md:117` "confirm one at a time" is about implied decisions, not batching |

### ddx-5

| Truth | Held? | Evidence |
|---|---|---|
| trace-and-provenance.md is gone; trace.md and provenance.md hold its halves | held | `skills/bee-researching/references/` lists `trace.md`, `provenance.md`, no `trace-and-provenance.md` |
| bee-how loads only trace.md; bee-why only provenance.md | held | `bee-how/SKILL.md:18` loads `trace.md`; `bee-why/SKILL.md:18` loads `provenance.md` |
| no file outside docs/history and CREATION-LOG.md names trace-and-provenance.md | held in the source trees | `rg trace-and-provenance skills packages docs .bee/expertise` (excluding docs/history and CREATION-LOG.md) finds only `docs/backlog.md:31`, the PBI title in the generated backlog view. The rendered copies (`.claude/`, `.agents/`, `.opencode/`, `.codex-plugin/`, `.claude-plugin/`) still name it until the regen; `.bee/` state records keep it as history |
| pointer_integrity passes | held | `test result: ok. 10 passed` |

### ddx-6

| Truth | Held? | Evidence |
|---|---|---|
| review.md holds only the reviewer side | held | headings: Where to look, What a finding is, Severity calibration, Severity is a spent signal, Adversarial reading, The 5-Layer rubric, Truth Table Test, CRUD Lifecycle check, Claims-table audit, Verify before reporting, Label uncertainty exactly, Evidence standards, Scope discipline, Style versus substance, Reader load |
| review-author.md holds the author side | held | headings: Asking for a review, Receiving a finding, Reviewing your own work; listed in `.bee/expertise/INDEX.md:21` |
| every referrer that cites an author-side section points to review-author.md | held | every `expertise/review.md` referrer in skills, docs and prepare.rs cites a reviewer section (Claims-table audit, CRUD, Reader load, Verify before reporting, finding quality); none cites an author section |
| pointer_integrity passes | held | `test result: ok. 10 passed` |

### ddx-7

| Truth | Held? | Evidence |
|---|---|---|
| contract.md covers ui vr gate/diff/accept, ui ds soul check/init, ui ds a11y | held | sections `## ui vr gate`, `## ui vr diff`, `## ui vr accept`, `## ui ds soul` (invocations `init`, `check`, `factory`), `## ui ds a11y` |
| every ui command named in the references is covered | held | `rg -o "\bui …"` over `packages/bee/design/bee-design/references` finds 40 distinct command stems; each has a `##` section in contract.md, or for `ui knowledge check` and `ui init` an entry under "Not for host work" with its reason |

## What would unblock

1. ddx-1 follow-up: make the fence and the page agree. D1 locks the path
   `playbooks/session/steer-recipes.md`, so the fix that keeps D1 is to add a
   `- \`steer-recipes\` — <triggers> — delivers: <deliverable>` line under
   `## Situations` in `session/index.md`, replacing the prose line, and drop
   "not a situation" from the page. Moving the page out of `playbooks/session/`
   would reopen D1.
2. Run `bee dev regen` at wave close, then re-run this verify. Until then
   `release-manifest --check` fails by design.
