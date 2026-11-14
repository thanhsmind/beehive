---
artifact_contract: bee-research/v1
topic: pstack-formal-verification-port
depth: standard
date: 2026-11-14
---

## Outcome

The owner picked the skillstation hub, not a bee pack (decision
`e9c80ef2`). An earlier decision, `d94edfa6`, already kept
`statechart` in the hub; this brief missed it. The two formal skills
are hub drafts tracked upstream, and bee's planning playbooks point at
them (`docs/history/formal-verification-pack/CONTEXT.md`). The pack
recommendation below is kept as the record of what was weighed.

## Bottom Line

- Recommendation (ladder rung): **adapt-upstream** (port mode).
  Bring in three pstack skills as bee skills:
  `create-formal-verification-skill`, its upkeep twin
  `maintain-formal-verification-skill`, and `statechart` with its
  TypeScript runner.
- Why this is the lightest credible path: bee has no formal check of
  any kind today. The upstream skills are MIT, small in prose (45, 60
  and 80 lines), and they already fit bee's own pattern: the earlier
  port turned `create-verification-skill` into `bee-verifying`, which
  writes a generated skill under `.bee/verify/` that `bee onboard
  --verify-only --apply` renders into every runtime.
- Why the next-best rung lost: there is nothing to reuse (rung 1) and
  no built-in (rung 2). Building a TLC-to-Mermaid tool from scratch
  (rung 4) repeats about 170 KB of tested upstream code for no gain.
- Confidence: **80%**. Prose is read in full at a pinned SHA. The
  statechart runner is measured but not run here: Java is not set up
  on this machine yet.
- Suggested next step: **bee-shaping** — two forks below need a
  locked answer.

## Source Manifest

| Field | Value |
|---|---|
| Repo or path | `https://github.com/cursor/plugins`, folder `pstack/` (a local clone at `/mnt/data/Projects/refs/pstack` is guarded as another project, so recon used GitHub) |
| Ref | `main` |
| Resolved commit SHA | `3dbd22ff12742fb28c9700b9760baa29ed41dc13` |
| Narrowed scope | `pstack/skills/create-formal-verification-skill/SKILL.md`, `pstack/skills/maintain-formal-verification-skill/SKILL.md`, `pstack/skills/statechart/{SKILL.md,references/render.md,scripts/**}` (tests listed, not read) |
| License | MIT, (c) 2026 Lauren Tan — `pstack/LICENSE` |

## Repo Snapshot

- Rust CLI (`packages/bee-rs`) plus Markdown skills in `skills/`,
  rendered into runtime skill homes by `bee dev regen` (`Local`).
- Third-party skill content already ships as opt-in packs under
  `packages/bee/design/` with `LICENSE.md`, `THIRD_PARTY_LICENSES.md`
  and `UPSTREAM.json` (pinned commit, excluded paths with reasons), and
  ships `.mjs` scripts (`bee-design-motion/additions/rehearse/`)
  (`Local`).
- Machine: Node 24.19, Bun 1.4.3, mise 2026.11.4; no Java configured
  (`Local`, command output).

## Findings

### What each source skill does (`Upstream`)

- **create-formal-verification-skill** — picks the code where a bug
  costs most and tests reach least, lets the question pick the tool
  (TLA+ default for ordering bugs; code-level verifier for per-input
  bugs; generated-input tests when no verifier exists), proves each
  property can fail on a broken variant, and writes a project-local
  verify skill with a coverage record and a **sync commit**. It must
  run that skill once on a scratch break before hand-off.
- **maintain-formal-verification-skill** — the drift loop. Four
  outcomes per check: in sync / updated / bug / blocked. Diffs covered
  code from the sync commit, classifies each change, updates the model
  from the code (never from the result), reruns, and replays every
  counterexample on the real code to split drift from bug. Edits only
  verification files.
- **statechart** — runs TLC on a TLA+ spec and a user-written view
  module (`state`, `step`, `group`, `colors`, `title`) and writes
  `chart.mmd` plus a report with the shortest path to each broken
  invariant. 500-state chart budget; 100k-state bound after the first
  break; TLC 1.7.4 and Java Temurin 25 pinned through mise, with
  `TLA2TOOLS`/`JAVA` overrides. Runner: about 30 TS files, about
  170 KB, needs Node 22.18+/23.6+, Bun 1.1.5+ or Deno 2+. Optional
  SVG via the npm package `pele@1.9.0`.

### Dependency matrix

| Component | Source | Local | Verdict | Label |
|---|---|---|---|---|
| Generator skill | `create-formal-verification-skill/SKILL.md` | none; nearest is `skills/bee-verifying` (UI/CLI driving, not proof) | NEW | `Local` |
| Upkeep skill | `maintain-formal-verification-skill/SKILL.md` | `skills/bee-verify-upkeep` keeps the feature map, not models | NEW | `Local` |
| Home of generated skills | `.cursor/skills/<name>/` | `.bee/verify/<name>/`, rendered by `bee onboard --verify-only --apply` ("Render only the host .bee/verify/ skills") | EXISTS — reuse, path differs | `Local` |
| Sync commit + covered files | in the generated SKILL.md | knowledge concepts carry `verified_commit` + `anchors`; `bee upkeep due` lists due work | EXISTS as a pattern; no reader for formal checks | `Local` / `Inference` |
| Statechart runner | `statechart/scripts/*.ts` | none | NEW — vendor | `Upstream` |
| TLC + Java install | mise, pinned | mise present; Java absent | NEW runtime dependency, opt-in | `Local` |
| Mermaid output | `chart.mmd` | `bee-design-charts` covers Mermaid diagram craft | EXISTS (adjacent) | `Local` |
| SVG render | `pele@1.9.0` (network install) | none | NEW, optional | `Upstream` |
| Third-party bookkeeping | MIT | `packages/bee/design/UPSTREAM.json` + `THIRD_PARTY_LICENSES.md` | EXISTS — reuse the shape | `Local` |

### Cross-cutting sweep

- `disable-model-invocation: true` on all three upstream skills: they
  run only when the user names them. bee skills self-route by
  description; keeping them user-invoked fits, since a model check is
  never a default step (`Inference`).
- The generator hard-links to `/statechart` and
  `/maintain-formal-verification-skill` by name. Renames must update
  both links (`Upstream`).
- The no-code-comments rule and its comment baseline apply to the
  vendored `.ts` files if they land in a scanned path (`Local`
  doctrine; whether the baseline scans `packages/bee/` is unchecked).
- `bee test` (cargo) does not run the runner's `node --test` suite; a
  vendored runner needs its own proof line or a CI step (`Inference`).
- The TLC download is a network fetch at first run: it must stay out
  of any hook or default path (`Inference`).

## Challenge

| Question | Source answer | Local answer | Risk if wrong | Flag |
|---|---|---|---|---|
| Core skill or opt-in pack? | Ships in a plugin users opt into | Design taste is opt-in by pick (design-os-in-bee D4) | Every host gets a Java-backed tool it never asked for | red unless opt-in |
| Where does a generated verify skill live? | `.cursor/skills/` | `.bee/verify/<name>/` is the rendered-source home | Skill seen by one runtime only | green — reuse |
| Vendor the TS runner or call it from pstack? | Shipped with the skill | Design packs vendor scripts with `UPSTREAM.json` | Runner breaks when pstack moves | green — vendor at the pinned SHA |
| Does a stale model get noticed? | Sync commit read only when someone runs maintain | `bee upkeep due` already lists due work | A passing check that describes dead code reads as proof | green if upkeep reads the sync commit (later slice) |
| Network and toolchain at first run? | mise installs TLC + Java | No Java here | A run fails offline or in CI | green — opt-in, user-invoked only, `TLA2TOOLS`/`JAVA` overrides kept |
| Do vendored comments break the comment rule? | Upstream `.ts` style unknown | Baseline reds on a rise | Suite red on import | unchecked — planning proof |

No red flag touches hard-gate territory (auth, data loss, schema,
external provider in a default path). Lane: **standard** (`Inference`).

## Open Questions for Shaping

1. **Pack or core.** Recommend an opt-in pack (`formal`), picked like
   the design packs: the generator and upkeep skills are prose-only
   and cheap, but the statechart runner pulls in Node + Java + TLC.
   Alternative: the two prose skills always install, statechart is the
   only pack.
2. **First slice.** Recommend slice 1 = the three skills plus the
   vendored runner; slice 2 = `bee upkeep due` reads the sync commit
   of each `.bee/verify/formal-*` skill and flags drift.
3. Do we then run the generator on bee itself (cell lifecycle, gate
   state machine, worktree merge) as the first real target, the way
   the user's quoted tip suggests? Recommend yes, as its own feature
   after the port lands.

## Source Pack

- Local: `.claude/skills/bee-researching/references/port-protocol.md`,
  `packages/bee/design/UPSTREAM.json`,
  `packages/bee/design/bee-design-charts/THIRD_PARTY_LICENSES.md`,
  `docs/knowledge/areas/onboarding/onboarding-design-packs-the-design-role-and-the-design-gate.md`,
  `skills/bee-verifying/SKILL.md`, `docs/history/research/pstack-distill.md`,
  `bee onboard --help`.
- Upstream: the four SKILL.md files and `render.md` above,
  `statechart/scripts/{statechart,package.json,run.ts}`, the tree
  listing at the pinned SHA, `pstack/LICENSE`.
