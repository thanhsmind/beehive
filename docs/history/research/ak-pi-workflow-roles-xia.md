---
artifact_contract: bee-research/v1
topic: ak-pi-workflow-roles-xia
depth: deep
date: 2026-09-18
---

## Bottom Line

- **Mode: `xia`** — distill and discuss, build nothing. The request read as
  `port` ("ứng dụng vào pi"), and the port protocol's own lane rule downgraded
  it: the source is an in-pi-process TypeScript court whose whole value is
  resident inside the Pi session, and decision **9f5c6d17** (live, user-made
  2026-09-02) says Pi dispatch stays herding-only and bee takes *design rules
  only, no engine, no code, no second transport*. The engine cannot cross that
  line. The rules can.
- **Recommendation (ladder rung): adapt-upstream**, four design rules, no code
  from the source. Ranked in § Adopt list.
- **Why this beats reuse (rung 1)**: 14 of the 20 dependency-matrix rows already
  exist in bee, several in a stronger form — but four rows are craft bee does
  not have, and one of them is a live drift (bee's Pi extension still declares
  0.84.3 while its own proof run was on 0.85.1).
- **Why this beats build (rung 4)**: every adopted rule is a table, a prompt
  variable or a doc — none needs a new subsystem.
- **The single most useful finding is a negative one.** The source built
  per-role toolset narrowing, shipped it, then **abolished it** (ADR 0008, as
  amended) and replaced it with one four-literal bash seatbelt; ADR 0064 then
  ruled evidence roles must keep *unrestricted* tools. bee has no per-role tool
  narrowing today. The evidence says do not build it. `Upstream`
- **Confidence: 88%.** Every upstream claim is anchored at a pinned commit;
  every local claim at `file:line`. The open 12% is whether the owner wants the
  role-description rule (adopt item 3) at all, since it is the one item that
  changes a worker's prompt bytes.
- **Suggested next step: bee-shaping**, one small docs-and-prompt feature
  carrying adopt items 1 and 3. Items 2 and 4 wait for the owner's pick.

## Source Manifest

| Field | Value |
|---|---|
| Repo or path | `/home/thanhsmind/Projects/refs/ak-pi-workflow-roles` (`github.com/Akagilnc/ak-pi-workflow-roles`) |
| Ref | `main` |
| Resolved commit SHA | `e401f195864da141864b33dd6cae7490c83979e7` (2026-09-18) |
| Narrowed scope | the workflow-role architecture: role declaration, souls, tool gating, submission contracts, the labor-outsourcing engine, and the Pi binding |

Shape: TypeScript pnpm package `@akagilnc/pi-workflow-roles` 0.1.0, bin
`ak-role`, Apache-2.0. 786 files, 384 test files, 83 ADRs. Distributed as
`pi install npm:@akagilnc/pi-workflow-roles`. `Upstream`

## Repo Snapshot

- **bee (local)**: Rust workspace `packages/bee-rs` (crates `bee`, `fleet`,
  edition 2024), markdown skills, one TypeScript belt per harness. Runtimes
  `claude, codex, opencode, pi`; dispatch door `codex, claude, pi`
  (`prepare.rs:91-101`). `Local`
- **bee's Pi belt**: exactly one file, `.pi/extensions/bee-guard.ts` (2679
  lines), vendored by `bee onboard --apply` step `copy_pi_extension`
  (`onboard/plan.rs:830-846`). Pi has no JSON hook surface; the extension is the
  whole belt. `Local`
- **bee's Pi dispatch**: every `team.pi` slot must be `kind: herding`; anything
  else is the typed refusal `pi_requires_herding` (`prepare.rs:2237-2243`).
  Payload is always `Bash` → `bee herding run` (`prepare.rs:2450-2530`). `Local`
- **Source's Pi binding**: spawns `pi --no-extensions -e
  extensions/role-runtime.ts --no-skills --no-context-files --session … --mode
  json --ak-role <role>` per invocation, instruction on stdin
  (`src/pi/role-turn-host.ts:126-170`). Uses `pi.registerFlag`,
  `pi.registerTool` with `terminate: true`, and 13 session events
  (`src/pi/adapter.ts:190-278`). `Upstream`

## Question & Assumptions

- **What was asked**: take `refs/ak-pi-workflow-roles` and apply it to pi for
  the bee harness.
- **What settles it**: a component-by-component map of what the source does that
  bee does not, with each row labelled adopt / already-have / refuse, and each
  refusal carrying its reason. A "port the engine" answer would have to
  supersede a locked user decision, which is not the agent's move.
- **Assumptions**: (a) `9f5c6d17` and `5d87f14e` still stand — verified live in
  the store this session; (b) the owner still wants herding as the one Pi
  transport; (c) `port by craft, not by feature` applies — a row is not dismissed
  because bee "has one", only when bee's does the job at least as well.

## Findings

### Upstream — what the source actually is

One line: **a caller-driven court of 16 single-invocation LLM "offices", each
being one soul file plus shared law codices plus one open-schema terminating
tool, run inside a Pi session by an extension envelope that owns all
lifecycle.** `Upstream`

- **A role is a code record, not a manifest.** `PUBLIC_ROLE_RECORDS`
  (`src/packaged-role-registry.ts:53-248`) carries per role: `role`, `phases`,
  `outputTool`, `inputFlag`, `phaseFlag`, `activationStage`, `sessionMaterials`.
  It carries **no model, no host, no engine, no tool list** — those are a seat
  table in `~/.ak-roles/public-cli.json`, resolved `--model → seat → inherit`
  and `--host → seat → "pi"` (`src/public-cli/config.ts:741`). ADR 0082 states
  the goal outright: roles are code seams, pi is one replaceable adapter, and
  the long-term aim is "终局移除 pi" — eventually remove Pi. `Upstream`
- **A soul is the role's attention budget**, injected as a system-prompt block
  `<judge_soul>…</judge_soul>` at `before_agent_start`
  (`src/judge-role.ts:147-153`). 15 identity souls plus three shared codices
  (`audit-law.md`, `quality-law.md`, `ticket-law.md`). Layering is ADR 0005:
  generic legal core in the soul, business law attached by the caller through
  host-native channels, zero new mechanism. The package hard-blocks Pi's own
  context and skill loading (`--no-context-files --no-skills`) so the only
  overlay is the explicit one. `Upstream`
- **The IO contract is the terminating tool's schema** (ADR 0003): each role owns
  `ak_<role>_output` with `terminate: true`; 16 names registered at
  `src/package-contracts/terminating-tools.ts:93-110`. The schemas are
  deliberately **open** (`additionalProperties: true`, `required: []`) because
  the constitution says code has no right to reject what a role submits
  (`CLAUDE.md:5-11`, ADR 0055). `Upstream`
- **Enforcement is three thin things**: two mechanical worker reminders (ADR
  0066 — a `completed` claim with zero new commits bounces once; a commit
  subject with no platform prefix gets one soft reminder), one four-literal bash
  seatbelt for the fixer (`rm -rf`, `git reset --hard`, `git clean`,
  `git checkout --`, exact substring, no regex —
  `src/fixer-bash-seatbelt.ts:2-15`), and LLM officer gates with **no round cap**
  (`src/gatekeeper-pass-envelope.ts:114`, ADR 0007). `Upstream`
- **Machine-text neutrality (ADR 0073)**: code may emit only neutral pointer
  sentences; every word addressed to a role lives in packaged resources. The
  repo keeps a migration table of the 12 sentences it moved out of code
  (`resources/836-deleted-machine-instruction-inventory.md`). `Upstream`
- **Labor outsourcing (ADR 0069)**: the heavy reasoning segment of a role may be
  handed to a local CLI engine (opus, codex, kimi, …) while governance stays in
  the Pi session — "a detour in the work step, back to the main road after".
  Dispatch rules are one shared file plus one measured note per engine. `Upstream`
- **Its own stated pain**: full test suite ~237 s local against the owner's
  120 s reference (`TEST_AUDIT_319.md:49`); engine use outside the detour tool is
  "a permanent blind spot" (`README.md:65`); the `marshal` office is named but
  not built; a known path-layout inconsistency between `CLAUDE.md:87` and
  `docs/dossier-topology.md:9-28`. `Upstream`

### Local — what bee already owns

- **A role in bee is a model-transport slot and nothing else** — a key under
  `team.<runtime>` carrying a model, effort, or a `{kind: cli|native|herding}`
  transport (`models.rs:127-247`). The set is open: any key the config names is
  legal (`models.rs:258-280`). `Local`
- **A slot may carry a `description`, and it is display-only.** Verified
  verbatim at `prepare.rs:1980-1981`: *"Nothing here reads a slot's
  `description`: that field is display-only"*. `bee team show --runtime pi
  --json` renders it (e.g. `test` → "author or repair tests, red-first"), the
  preamble door line renders it, and it stops there. It never reaches the
  dispatched worker. `Local`
- **Prompts are per dispatch *kind*, not per role**: four files totalling 144
  lines — `packages/bee/prompts/{worker-cell,gather,reviewer,advisor}.md`. The
  only role-aware text anywhere is the `{{seat}}` line in `advisor.md`, used by
  the `hat-*` seats. `Local`
- **Tool allowlists are per agent type, hardcoded** in the four
  `packages/bee/agents/*.md.tmpl` frontmatter lines; role→agent is the fixed map
  `ROLE_AGENTS` (`guard.rs:181-186`), and anything unmapped pins
  `general-purpose` (`guard.rs:314`). A `test` or `docs` role therefore runs with
  every tool. `Local`
- **bee's declared workflow of roles already exists**: `bee-plan/v2` role plans
  (`plan_packets.rs:194-397`) — a fenced JSON block in `plan.md` binding each
  stage to one configured role, carrying `roster_sha256`, approved at Gate 2,
  enforced at `dispatch prepare` and again by the model-guard hook
  (`model_guard.rs:801-978`), and changed only through `bee cells reroute` with a
  `role-reroute`-tagged decision (`handlers_write.rs:2625-2784`). `Local`
- **bee's cell submission gate is typed and strict**: `bee cells finish
  --report '<json>'` requires `outcome, commit, files, tests, deviations`, a
  proof string of exactly three segments with the result closed over
  `green:live|green:unit|green:static`, and `red` refuses the cap
  (`finish_support.rs:57-260`). `bee close` refuses a feature whose capped cells
  never answered `mistakes`. `Local`
- **Neutral machine text is already bee's rule**: prompt wording lives in
  vendored `packages/bee/prompts/*.md`, rendered through one renderer, and an
  on-disk skew refuses the dispatch (`prompt.rs:105-145`). `Local`
- **No-silent-fallback is already bee's rule**: `role_not_configured`,
  `tier_not_configured`, `pi_requires_herding`, `advisor_not_configured` all
  refuse by name. `Local`
- **The caller owns composition in bee too** — the bee binary contains no
  orchestrator; the leader session composes. Same conclusion as ADR 0010,
  reached independently. `Local`

### Dependency matrix

One row per source component. `EXISTS` = bee has it and it does the job at least
as well. `NEW` = genuinely missing. `CONFLICT` = bee has a different, deliberate
answer.

| # | Source component | Local disposition | Evidence |
|---|---|---|---|
| 1 | Role record table (closed, in code) | `CONFLICT` — bee's role set is open config; deliberate (B12 doctrine) | `Local` `models.rs:258-280` |
| 2 | Seat table: model/host/engine resolved per invocation | `EXISTS` — `team.<runtime>.<role>` + `--role` override at one door | `Local` `prepare.rs:2036-2062` |
| 3 | One soul file per role, injected as a system-prompt block | **`NEW`** | `Local` four prompts keyed by kind, `prepare.rs:1035-1123` |
| 4 | Shared law codices (audit-law, quality-law, ticket-law) | `EXISTS` — AGENTS.md, skills, the principle set | `Local` |
| 5 | Soul layering: generic core + host overlay (ADR 0005) | `EXISTS` — skill render with `<!-- bee:only pi -->` blocks | `Local` `onboard/render.rs:354-356` |
| 6 | Machine-text neutrality (ADR 0073) | `EXISTS`, bee stronger — byte-parity check refuses skew | `Local` `prompt.rs:105-145` |
| 7 | Typed terminating tool per role, open schema | `CONFLICT` for cells (bee validates and refuses); **`NEW`** for gather/reviewer/advisor, which have no output gate at all | `Local` `finish_support.rs:57-260` |
| 8 | Code may never reject a submission (ADR 0055) | `CONFLICT` — bee's `red` refusal is load-bearing. Keep bee's | `Local` |
| 9 | Worker submission reminders (ADR 0066) | `EXISTS`, bee stronger — commit trailer + proof line required at cap | `Local` |
| 10 | Four-literal bash seatbelt (ADR 0008) | `EXISTS`, bee far stronger — write-guard hook, reservations, worktree isolation | `Local` `.pi/extensions/bee-guard.ts` |
| 11 | Per-role toolset narrowing | **`CONFLICT` — the source deleted it.** See § Challenge Q1 | `Upstream` ADR 0008 amended, ADR 0064 |
| 12 | Officer gate loop, unbounded bounce | `CONFLICT` — bee's review is user-invoked (`agents-review-user-invoked`) | `Local` |
| 13 | Auditor reads the *retained session*, hand-delivered material illegal | **`NEW`** as a rule for bee-reviewing | `Upstream` ADR 0006/0017, `CONTEXT.md:40` |
| 14 | Labor-outsourcing engine, mechanism | `EXISTS` — `herding.agents` registry + cli slots | `Local` |
| 15 | Per-engine measured note file + shared dispatch rules | **`NEW`** | `Upstream` `resources/engines/*.md`, `resources/engine-dispatch.md` |
| 16 | "Task + paths only, never paste bodies into the prompt" | `EXISTS` — `learned_context` is paths + one-line titles, never file contents; `expertise` is path triples | `Local` `prepare.rs:750-940` |
| 17 | Engine failure is typed; in-seat fallback forbidden (ADR 0071) | `EXISTS` — every unconfigured or wrong-shaped slot refuses by name | `Local` `prepare.rs:1990-2243` |
| 18 | Per-host-version capability audit table | **`NEW`** — and bee has a live drift it would catch | `Local` extension header says Pi 0.84.3; `pi-hat-wave.md` proof ran 0.85.1 |
| 19 | Ledger topology: one home, per-ticket books, per-run dirs | `EXISTS` — `.bee/` store, `docs/history/<feature>/`, `.bee/mailbox/<job-id>/`, `.bee/logs/dispatch.jsonl` | `Local` |
| 20 | Public CLI is the only supported interface; exit code = lifecycle honesty | `EXISTS` — `bee --help --json` porcelain, typed refusals throughout | `Local` |

Fourteen `EXISTS`, four `NEW`, and the `CONFLICT` rows are all deliberate bee
positions rather than gaps.

### Cross-cutting sweep

Wiring outside the role folder that any adopted rule would touch:

- **Prompt renderer and its byte-parity guard** — `drivers/prompt.rs:33-145`,
  `devtools/prompts.rs`. A new prompt variable changes the rendered bytes and
  the pinned-parity test. `Local`
- **The model-guard hook** — `hooks/model_guard.rs:801-978` re-checks semantic
  role routing on raw Agent calls; it already reads
  `role_slot_description`. `Local`
- **The role-plan schema allow-lists** — `plan_packets.rs:285-289` and
  `:351-359` reject unknown fields, so any new per-role field in a plan must be
  added in both. `Local`
- **`bee team show` and the preamble door line** — the two existing readers of
  `description`. `Local`
- **The Pi belt's event map** — `.pi/extensions/bee-guard.ts:2049-2679`. Nothing
  in the adopt list touches it except item 1, which touches its version string.
  `Local`
- **Release manifest** — `.pi/extensions` is an inventoried root
  (`devtools/release_manifest.rs:83`), so an extension edit must go through
  `bee dev regen`. `Local`

Components absent from this sweep are unchecked, not confirmed clean.

### Challenge — five adversarial questions

**Q1. bee has no per-role tool narrowing. Should it take the source's?**
*Source answer*: it had one, for the judge, and **abolished it** — ADR 0008 as
amended keeps only a four-literal bash seatbelt, and ADR 0064 then ruled that
evidence roles "may read, write, query, and temporarily build fixtures … runtime
must not narrow their evidence tools". *Local answer*: bee pins tools per agent
type and lets unmapped roles run `general-purpose` (`guard.rs:314`). *Risk if
wrong*: building per-role narrowing would need a new slot field, a rendered
agent per role, and guard acceptance in three places
(`model_guard.rs:1490-1543`) — and the one team that shipped it took it out.
**Red flag on building it. Green flag on leaving bee as it is.**

**Q2. The souls are the source's centre. Can bee take them without taking the
court?** *Source answer*: a soul is an identity plus irreducible judgment, and
its own rule is "the soul is the role's attention budget, not a full manual;
default is to cut" (`CLAUDE.md:38`). *Local answer*: bee already stores exactly
one sentence of role identity — the slot `description` — and throws it away at
dispatch. *Risk if wrong*: adding 15 soul files to bee would duplicate the
skills layer and create a second home for doctrine, which
`bee-principle-single-source-of-truth` forbids. **Green flag on carrying the
existing one-line description into the prompt. Red flag on soul files.**

**Q3. Does the officer gate loop beat bee's user-invoked review?** *Source
answer*: every worker completion summons an inspector, every judge draft a
notary then an auditor, with no round cap (ADR 0007, deliberately). *Local
answer*: bee's independent review is a separate, user-invoked pass
(`agents-review-user-invoked`), and its own recorded history shows unbounded
automatic loops are a cost problem — the source's own `TEST_AUDIT_319.md` and
its 957k-char seat kill are the same family of failure. *Risk if wrong*: an
automatic gate on every cell would multiply dispatch cost with no owner control.
**Red flag.**

**Q4. Is the labor-outsourcing engine a second transport in disguise?**
*Source answer*: no — governance stays in the role session and only the work
step detours (ADR 0069). *Local answer*: on bee that distinction disappears,
because bee's Pi dispatch already *is* an external CLI in a pane; a bee "engine
detour" would be a second way to reach the same herding agents, which
`9f5c6d17` refuses by name. *Risk if wrong*: a second transport, a second guard
surface, a parity-test change. **Red flag on the mechanism. Green flag on the
per-engine note, which is documentation, not transport.**

**Q5. Is the per-version capability audit worth a table when bee already has
`bee doctor --runtime pi` and a verify-app feature file?** *Source answer*: the
table is the upgrade protocol — compare the changelog, re-decide each row
keep/adapt/delete, regenerate the patch, run one wire smoke, and explicitly "do
not add a permanent scanner or upgrade gate"
(`docs/pi-0.84.1-capability-audit.md:25-27`). *Local answer*: bee's doctor
checks the belt is installed and unmodified; nothing checks whether a Pi release
changed what the belt relies on. The drift is already visible — the extension
header says Pi 0.84.3 (`.pi/extensions/bee-guard.ts:19`) while the hat-wave
proof ran on 0.85.1. *Risk if wrong*: a Pi upgrade silently breaks an event name
and the belt fails open on an advisory surface. **Green flag.**

No red-flag verdict landed in hard-gate territory (no auth, no data loss, no
validation removal, no schema migration), so nothing here routes `high-risk`.

### Adopt list — ranked by value ÷ cost

1. **A Pi capability-audit table, one row per upstream capability, with a
   `keep | adapt | delete` disposition and its evidence.** Cheapest item, and it
   has a live finding waiting for it: bee's belt declares 0.84.3 while its own
   proof ran on 0.85.1. Sits beside the existing
   `.bee/verify/verify-app/features/pi-runtime.md`. Docs lane. `Inference`
2. **Carry the role's `description` into the dispatched prompt.** bee already
   stores one sentence of role identity per role and drops it
   (`prepare.rs:1980-1981`). This is the source's soul idea at bee's smallest
   honest shape: one template variable, one lookup, no new file, no second home
   for doctrine. Small; it changes rendered prompt bytes, so it needs the
   parity test updated. `Inference`
3. **A measured note per `herding.agents` entry** — what the CLI is, which
   output format returns the body alone, which permission flag headless needs,
   and the measurement behind each. The source's `resources/engines/opus.md`
   records that `--output-format text` returned 382 bytes where
   `stream-json --verbose` returned 45,028 for the same task. bee's
   `herding.agents` registry carries argv and no evidence. Medium. `Inference`
4. **A rule for `bee-reviewing`: the reviewer reads the retained artifact, not a
   hand-delivered digest.** The source makes this constitutional — "先立卷后审卷",
   the audited object must be on the ledger first, and hand-delivered material is
   illegal (`CONTEXT.md:40`). bee's reviewers read what the leader passes them.
   Prose rule, zero code. `Inference`

### Refuse list — with reasons, so this is not re-asked

- **The engine** (`role-runtime.ts` envelope, `ak_engine_detour`, the officer
  court): refused by `9f5c6d17`, and by stack — it is Pi-process-resident
  TypeScript against bee's Rust CLI.
- **Per-role toolset narrowing**: the source built and deleted it (Q1).
- **Soul files per role**: second home for doctrine (Q2).
- **Automatic per-submission officer gates with no round cap**: conflicts with
  `agents-review-user-invoked` and with cost control (Q3).
- **Open schemas / "code may never reject a submission"**: bee's `red`-refuses-
  the-cap rule is load-bearing and better.
- **Navigator auto-attendance on every leg**: bee's hat wave is deliberately one
  consult per feature; per-dispatch advice multiplies cost for advice that the
  source itself calls non-binding (`souls/navigator.md:3`).

## Risks, Unknowns, Follow-Ups

- **Verified side-finding, unrelated to the source**: `bee herding run` parses
  `--seat` and `--inbox-session` (`herding/run.rs:369-373`) but neither appears
  in `bee herding run --help`. Traced the parser and ran the help before filing
  this. One-line porcelain fix; not part of the adopt list.
- **Live drift**: `.pi/extensions/bee-guard.ts:19` says Pi 0.84.3; the hat-wave
  evidence ran on Pi 0.85.1. Adopt item 1 is the thing that would have caught it.
- **Stale doc claim to re-check**: `docs/knowledge/areas/hook-runtime/overview.md:106-109`
  says a bare `bee state session release` cannot resolve the Pi session, but
  `session_identity.rs` now reads `PI_SESSION_ID`. Not verified live. Open.
- **Adopt item 2 changes worker prompt bytes**, so it touches the pinned prompt
  parity test and every runtime at once. Size it before shaping.
- **Open question for the owner**: adopt item 2 is the only item that changes
  what a worker reads. Do you want the role's own one-line job description in
  front of the worker, or is the cell brief enough?

## Source Pack

- **Local files read**: `packages/bee-rs/crates/bee/src/verbs/drivers/{prepare,models,guard,prompt}.rs`,
  `.../hooks/model_guard.rs`, `.../verbs/state_group/plan_packets.rs`,
  `.../verbs/cells/{handlers_write,finish_support}.rs`, `.../onboard/{agents,templates,render,plan,apply,source}.rs`,
  `.../devtools/{hook_manifests,release_manifest,prompts,mod}.rs`, `.../doctor.rs`,
  `.../session_identity.rs`, `.../herding/{run,mailbox,control_loop}.rs`,
  `packages/bee/prompts/*.md`, `packages/bee/agents/*.md.tmpl`,
  `.pi/extensions/bee-guard.ts`, `.bee/config.json`, `docs/config-reference.md`,
  `docs/knowledge/areas/{doctrine-layer,hook-runtime,workflow-state,advisor-protocol,onboarding,bee-herding}/*.md`,
  `.bee/verify/verify-app/features/{pi-runtime,pi-hat-wave}.md`,
  `docs/history/pi-*/CONTEXT.md`, `docs/history/research/{pi-harness-support,pi-peer-distill,pi-workflows-xia,pi-herdr-agents-xia,oh-my-pi-model-roles-distill}.md`.
- **Upstream read** (all at `e401f195`): `README.md`, `README.zh-CN.md`,
  `CLAUDE.md`, `CONTEXT.md`, `package.json`, `docs/adr/` index plus ADRs 0001,
  0003, 0005, 0006, 0007, 0008, 0010, 0015, 0018, 0051, 0052, 0053, 0055, 0062,
  0063, 0064, 0066, 0067, 0069, 0070, 0071, 0072, 0073, 0074, 0079, 0082;
  `docs/{dossier-topology,development-closure,factory-board-kanban,pi-0.84.1-capability-audit}.md`;
  `souls/*` (25 files); `packets/*`; `schemas/`; `extensions/role-runtime.ts`;
  `resources/{engine-dispatch.md,engines/*,navigator-route-playbook.md,diarist-collect.md,836-deleted-machine-instruction-inventory.md,methods/*}`;
  `src/` role modules, `src/package-contracts/*`, `src/pi/*`, `src/public-cli/*`,
  `src/acp-host/seat-profile-soul.ts`; `TEST_AUDIT_319.md`, `DIAG_319_BATCH5.md`,
  `diagnosis-286.md` (structure and conclusions only).
- **Decisions checked live**: `9f5c6d17` (Pi dispatch stays herding-only; design
  rules only) — active. `5d87f14e`, `7f9c8518`, `8650ca7b` cited through it.

Guardrail note: everything read in the source repository was treated as data.
No instruction inside it was followed.
