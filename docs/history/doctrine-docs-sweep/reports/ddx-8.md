# ddx-8 — parity and pointer check of the doctrine and docs sweep

Checked at HEAD `ad6300be6`, after the ddx-1 to ddx-7 commits (d0552054b,
880a3e724, 5399ad719, 3584d45b9, 26b518542, fca3a0f5e, 428ed2d79, 04f8e7bed),
the two follow-ups (1604e1cbf ddx-1, 6570fff86 ddx-6) and the wave-close regen
(ad6300be6). The truths come from the cell list in `plan.md`. The files were
read directly. No cap report was read.

**Verdict: BLOCKED on one truth.** The verify passes. The ddx-1 truth that
names `skills/bee-planning/playbooks/session/steer-recipes.md` does not hold
as written: 1604e1cbf moved the page to
`skills/bee-planning/references/steer-recipes.md`, away from the path D1 locks.
Every other truth holds.

## Verify

The cell verify, run as written, exits 0:

```
PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity --test agents_block_render_parity --test rule_index_parity --test class_playbook_parity && .bee/bin/bee knowledge check && .bee/bin/bee dev release-manifest --check
```

Result line per command:

| Command | Result line |
|---|---|
| `cargo test … --test agents_block_render_parity` | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| `cargo test … --test class_playbook_parity` | `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| `cargo test … --test pointer_integrity` | `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| `cargo test … --test rule_index_parity` | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| `.bee/bin/bee knowledge check` | `knowledge check: 330 concept(s) in 351 file(s), 0 OKF error(s), 0 profile error(s), 605 profile warning(s) — OK` |
| `.bee/bin/bee dev release-manifest --check` | `release_manifest --check: 990 file(s) match stored manifest, 1 unhashed artifact(s) present` |

The first attempt at this cell (commit ace4c3aeb) was red in two places:
`class_playbook_parity` refused `playbooks/session/steer-recipes.md` as an
unlisted situation, and `release-manifest --check` was waiting on the regen.
1604e1cbf cleared the first by moving the page out of `playbooks/session/`.
ad6300be6 cleared the second.

## Truths

### ddx-1

| Truth | Held? | Evidence |
|---|---|---|
| feature.md has a numbered step that writes the caller's tutorial first for a package or API other code calls and checks the result against it | held | `skills/bee-planning/playbooks/feature/feature.md:34` step 4: "write the caller's tutorial first … Check the design against that tutorial before any delegate writes code, and again at verify" |
| skills/bee-planning/playbooks/session/steer-recipes.md maps one-line steer prompts to bee situations and principles | **not held as written** | `playbooks/session/` holds no `steer-recipes.md`; 1604e1cbf renamed it unchanged (similarity 100%) to `skills/bee-planning/references/steer-recipes.md`. The content holds there: a 22-row "Say \| Routes to" table, and every situation file it names exists. The truth and CONTEXT.md D1 both name the old path |
| session/index.md links steer-recipes.md | held | `index.md:18` "see `bee-planning/references/steer-recipes.md`"; pointer_integrity passes |

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
| no file outside docs/history and CREATION-LOG.md names trace-and-provenance.md | held | `rg -l trace-and-provenance --hidden` (excluding docs/history, CREATION-LOG.md, .git) finds only `.bee/` state records (decisions, backlog, archived cells, ledgers) and `docs/backlog.md`, the generated backlog view naming the PBI. After the regen no rendered skill tree names it |
| pointer_integrity passes | held | `test result: ok. 10 passed` |

### ddx-6

| Truth | Held? | Evidence |
|---|---|---|
| .bee/expertise/review.md holds only the reviewer side | held | headings: Where to look, What a finding is, Severity calibration, Severity is a spent signal, Adversarial reading, The 5-Layer rubric, Truth Table Test, CRUD Lifecycle check, Claims-table audit, Verify before reporting, Label uncertainty exactly, Evidence standards, Scope discipline, Style versus substance, Reader load |
| review-author.md holds the author side | held | headings: Asking for a review, Receiving a finding, Reviewing your own work; listed in `.bee/expertise/INDEX.md:21` |
| every referrer that cites an author-side section points to review-author.md | held | every `expertise/review.md` referrer in skills, docs and prepare.rs cites a reviewer section (Claims-table audit, CRUD, Reader load, Verify before reporting, finding quality); none cites an author section |
| pointer_integrity passes | held | `test result: ok. 10 passed` |

`expertise/review.md`, `expertise/review-author.md` and `expertise/INDEX.md`
match their `.bee/expertise/` copies byte for byte (`diff` is empty), since 6570fff86
made the same split in the source. `review-author.md` also opens with a
`## Where to look` section aimed at the author.

### ddx-7

| Truth | Held? | Evidence |
|---|---|---|
| contract.md covers ui vr gate/diff/accept, ui ds soul check/init, ui ds a11y | held | sections `## ui vr gate`, `## ui vr diff`, `## ui vr accept`, `## ui ds soul` (invocations `init`, `check`, `factory`), `## ui ds a11y` |
| every ui command named in the references is covered | held | `rg -o "\bui …"` over `packages/bee/design/bee-design/references` finds 40 distinct command stems; each has a `##` section in contract.md, or for `ui knowledge check` and `ui init` an entry under "Not for host work" with its reason |

## What would unblock

One choice for the leader, both keep the verify green:

1. Keep the page at `skills/bee-planning/references/steer-recipes.md` and
   amend D1 and the ddx-1 truth to that path (record the amendment with
   `bee decisions log`). The page calls itself "a lookup for the owner, not a
   situation", so references/ fits it better than `playbooks/session/`.
2. Move it back to `playbooks/session/` and list it under `## Situations`
   in `session/index.md`, which keeps D1 as written but files a lookup page as
   a situation.

Leaning: option 1.
