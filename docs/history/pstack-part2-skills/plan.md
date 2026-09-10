---
artifact_contract: bee-plan/v1
mode: standard
---

# Plan: pstack-part2-skills

Mode: `standard` — 1 risk flag: multi-domain
Why this is the least workflow that protects the work: nine new skill trees under
`.claude/skills/`, no Rust code, no existing behavior changed — the risk is a ported
skill that tells an agent to hand-pick a model or read a Cursor path that does not
exist here, so the ceremony that pays is a shared adaptation contract plus one
mechanical de-pstack check.

## Requirements (from CONTEXT.md)

No prior CONTEXT.md — the ask arrived clear. The decisions this plan locks:

- D1: Port nine pstack skills into `.claude/skills/`, keeping their pstack names:
  `unslop`, `technical-writing`, `teach`, `how`, `why`, `recall`, `arena`,
  `architect`, `prototype`.
- D2: Do NOT port the `multi-phase-plan` playbook. `bee-planning` plus cells and
  slices already own that ground; a second plan format would break
  `bee-principle-single-source-of-truth`.
- D3: Every subagent dispatch inside a ported skill goes through the ONE bee door,
  `.bee/bin/bee dispatch prepare`. No hand-picked subagent type, no model
  parameter, no model name in prose.
- D4: `why` gains bee's own record as a source (`bee decisions search`,
  `docs/history/<feature>/CONTEXT.md`, `docs/knowledge/`, `docs/history/learnings/`)
  and loses the six MCP source playbooks (Linear, Notion, Slack, Datadog, Sentry,
  Databricks) — none of those MCPs are configured in this repo.
- D5: `recall` reads Claude Code transcripts, not Cursor ones, and reads bee state
  before it mines transcripts.
- D6: Every ported skill keeps `disable-model-invocation: true`, so it is
  user-invoked only and never competes with bee's own router.
- D7: Install to `.claude/skills/` only, tracked in git — the same home commit
  9400e829 used for the pstack part-1 verification trio. Not `skills/` (that tree is
  the bee plugin's own contract, synced by `onboard --apply`), not `.agents/skills/`.

## Load-bearing claims

Labels: `read` = I opened the file at the anchor. `ran` = I ran the command and kept
its output. `guessed` = no evidence yet. No row may stay `guessed` past the gate.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The pstack skills exist at these paths and are the source of the port | ran | `git clone --depth 1 https://github.com/cursor/plugins.git` then `ls pstack-src/pstack/skills/` | `architect`, `arena`, `how`, `recall`, `teach`, `technical-writing`, `unslop`, `why` all listed |
| 2 | `.claude/skills/` already holds non-bee skills, tracked in git, and they survive onboarding | ran | `git ls-files .claude/skills/create-verification-skill` | `.claude/skills/create-verification-skill/SKILL.md` |
| 3 | `onboard --apply` syncs only `bee-*` into `.claude/skills/`, so a non-bee dir is never overwritten | read | `README.md:621` | "by default it syncs the bee skill set into the host repo's own managed roots (`<repo>/.claude/skills/bee-*` …) from this repo's `skills/` tree" |
| 4 | `bee dispatch prepare` returns the tool and payload to run, plus an economics record | ran | `.bee/bin/bee dispatch prepare --runtime claude --kind gather --role extraction --json` | `{"tool":"Bash","payload":{"command":".bee/bin/bee herding run --task-file - --json --agent \"agy-flash\" --ceiling 1800"}, "economics":{"logical_tier":"extraction"}}` |
| 5 | Claude Code transcripts live one flat `.jsonl` per session under a slug dir, not the Cursor nested form | ran | `ls ~/.claude/projects/-home-thanhsmind-Projects-goglbe-beehive/*.jsonl \| wc -l` | `500`; sample `…/01146a14-5d0c-4cfa-ad8f-cf27bbaa44c5.jsonl` |
| 6 | bee's decision store is searchable from the CLI and is the repo's real "why" record | ran | `.bee/bin/bee decisions search --help` and `bee orient` | `decisions: 1914 active`; `--text (str) — Whitespace-separated search terms (case-insensitive, OR-matched, ranked by hit count)` |
| 7 | The declared test suite does not cover `.claude/skills/**` | read | `.bee/config.json` `commands.test` | `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` |
| 8 | pstack's `multi-phase-plan` playbook is pstack-infrastructure-bound and would duplicate bee-planning | read | `pstack/skills/poteto-mode/playbooks/multi-phase-plan.md` | "Run `node pstack/skills/poteto-mode/scripts/check-plan.mjs <plan.md>`"; "Ten lanes on `grok-4.6-fast-xhigh` at the PR head" |
| 9 | `why`'s six non-git source playbooks each target an MCP this repo does not have | read | `pstack/skills/why/references/source-playbook.md:5-13` | rows for Linear, Notion, Slack, Datadog, Sentry, Databricks |

## Discovery

I cloned `cursor/plugins` into the session scratchpad and read all nine candidate
skills plus their reference files end to end (about 1,500 lines). Two findings
changed the shape. First, `why` carries eight source playbooks, six of which name
MCPs that are not configured here — porting them whole would ship six dead
investigators, so the port keeps `code-archaeology.md` and `epistemics.md` and adds
one new `bee-store.md` playbook for bee's own record. Second, `multi-phase-plan`
hard-codes pstack's own scripts, agent types, and PR-stack tooling, and its job is
already bee-planning's; it is out of scope rather than adapted.

## Approach

Recommended path: one adaptation contract, five parallel cells, disjoint file trees.

The adaptation contract every cell applies to every file it writes:

1. **Dispatch through the door.** Replace every hand-picked subagent type, model
   parameter, and readonly flag with a `.bee/bin/bee dispatch prepare` call, then
   "run exactly the tool and payload it returns". Map the pstack roles: an explorer
   or investigator is a gather dispatch, a judge or cross-judge is a reviewer
   dispatch, a design candidate runner or synthesizer is an advisor dispatch.
2. **Strip the Cursor and pstack furniture.** No `~/.cursor/**`, no
   `pstack/skills/**` paths, no pstack agent type names, no `cursor-team-kit`, no
   `/deslop`, no model names.
3. **Point cross-references at what exists here.** A named sibling skill keeps its
   name. Where bee already owns the ground, point at the bee skill instead.
4. **Keep the frontmatter shape**: `name`, `description`, `disable-model-invocation: true`.
5. **Add one provenance line** at the top of each `SKILL.md` body: ported from
   pstack (`cursor/plugins`, `pstack/skills/<name>`), adapted for bee.
6. **Write to `.claude/skills/<name>/`** and nowhere else.

Rejected alternatives:

- Copy the nine skills verbatim. Rejected: `bee dispatch prepare` is a boundary rule
  in AGENTS.md, and a verbatim `how` would tell a worker to hand-pick a model — the
  model-guard hook refuses that, so the skill would be dead on arrival.
- Rewrite them as `bee-*` skills folded into `skills/`. Rejected: that changes the
  bee plugin's published contract for a set of skills this repo wants locally, and
  commit 9400e829 already set the precedent for the other home.
- Port `multi-phase-plan` too. Rejected per D2.

Risk map:

| Component | Risk | Proof needed |
|---|---|---|
| `unslop`, `technical-writing` | LOW — pure prose rules, no dispatch | frontmatter parses, no pstack token |
| `teach`, `prototype` | LOW — thin, call other skills by name | link targets resolve |
| `how`, `arena` | MEDIUM — dispatch rewiring | no hand-picked model survives the grep |
| `recall` | MEDIUM — transcript paths must be real here | the glob returns files |
| `why` | MEDIUM — source set changes, new bee-store playbook | the playbook's commands run |
| `architect` | MEDIUM — depends on `arena`, `how`, `why` all landing | cross-links resolve |

## Shape

Slice 1 is the whole feature. Five cells, no shared files, all parallel.

| Cell | Writes | Why grouped |
|---|---|---|
| A | `.claude/skills/unslop/`, `.claude/skills/technical-writing/` | The two writing skills cite each other; one writer keeps them consistent |
| B | `.claude/skills/how/`, `.claude/skills/why/`, `.claude/skills/teach/` | `teach` sits on `how` and `why`; the dispatch rewiring must read the same way across all three |
| C | `.claude/skills/arena/`, `.claude/skills/architect/` | `architect` Phase B runs `arena`; one writer owns both sides of that handoff |
| D | `.claude/skills/recall/` | Standalone; cites `why` by name only |
| E | `.claude/skills/prototype/` | Standalone; the playbook becomes a skill |

## Test matrix

`.claude/skills/**` is markdown an agent loads, not Rust the suite compiles (claim 7).
The declared `commands.test` does not reach it, so the proof is a parity and pointer
check, run over the nine new trees:

- **Happy path** — every new `SKILL.md` has YAML frontmatter that parses and carries
  `name`, `description`, and `disable-model-invocation: true`.
- **Edge case** — every relative markdown link inside the nine trees resolves to a
  file that exists.
- **Error path** — the de-pstack scan returns zero hits over the nine trees, for the
  token set named in the adaptation contract (Cursor paths, pstack paths, pstack
  agent type names, `cursor-team-kit`, `/deslop`, and any model name).

Every cell runs all three over its own trees and records the output on its cap. The
leader runs them once more over all nine before close.

## Open Questions

(none)

## Out of scope

- The `multi-phase-plan` playbook (D2).
- `why`'s six MCP source playbooks (D4).
- The other 39 pstack skills, including the whole `principle-*` set and `poteto-mode`
  itself — bee has its own principle set and its own router.
- Any change to `skills/`, `.agents/skills/`, `AGENTS.md`, or `CLAUDE.md`.
- Any change to the bee Rust binary.
