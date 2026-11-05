# Plan: agent-shell-rename

## Summary

Rename the five rendered bee agent shells after hive roles and add a sixth
for the advisor kind: `bee-build`→`bee-worker`, `bee-design`→`bee-comb`,
`bee-gather`→`bee-forager`, `bee-extract`→`bee-scout`,
`bee-review`→`bee-guard`, plus new `bee-elder` so an advisor-kind dispatch
names a bee shell instead of borrowing `general-purpose`. Two disjoint
cells, one wave: asr-1 moves the code, templates and tests; asr-2 moves the
prose and the knowledge concept. Playbook:
`skills/bee-planning/playbooks/feature.md` — step 6 (statechart) does not
apply: no event-order flow changes. Lane `standard`: two flags (public
contracts — the agent names are what hosts and the model guard speak;
multi-domain — Rust, templates, docs, site), 12+ product files.

SMALLER PATH check: PASS. The names live in three closed tables
(`AGENT_ROLES_BY_NAME`, `ROLE_AGENTS`, `AGENT_OPENCODE_PERMISSION_DENY`),
one `match` in `prompt.rs`, one in `agent_job_summary`, and the template
file names. Everything else derives from those through the one resolver
(pattern `20260831-prose-that-recommends-a-shape-drifts`). No alias layer:
the old names are retired outright and the onboarding plan removes their
stale rendered files, so no host keeps a shell bee no longer knows.

Decision: `e8ffde50-2e43-499e-86ee-a1d2eece215b` (the approved name set).

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | The agent→role table is one closed const in templates.rs | read | packages/bee-rs/crates/bee/src/onboard/templates.rs:437 | ("bee-build", &["generation"]), |
| 2 | The role→agent table is one closed const in guard.rs | read | packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs:182 | ("generation", "bee-gather"), |
| 3 | A role with no rendered agent falls to general-purpose in prepare | read | packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs:316 | agent_for_role(role).unwrap_or("general-purpose") |
| 4 | prepare names the claude subagent through that total lookup | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2478 | _ => pinned_agent_type(marker_role), |
| 5 | Embedded agent bodies are keyed by name in prompt.rs | read | packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs:55 | "bee-build" => AGENT_BUILD, |
| 6 | The onboarding plan walks templates only, so a renamed-away file is never removed | read | packages/bee-rs/crates/bee/src/onboard/agents.rs:215 | for tmpl_name in list_template_agents(engine) { |
| 7 | The opencode plan walks the same template list | read | packages/bee-rs/crates/bee/src/onboard/agents.rs:251 | for tmpl_name in list_template_agents(engine) { |
| 8 | The model guard repairs general-purpose onto the pinned agent for a role that has one | read | packages/bee-rs/crates/bee/src/hooks/model_guard.rs:1338 | if let (false, Some("general-purpose"), Some(pinned)) = |
| 9 | doctor names the design agent file by path | read | packages/bee-rs/crates/bee/src/doctor.rs:269 | const DESIGN_AGENT_REL: &str = ".claude/agents/bee-design.md"; |
| 10 | OpenCode permission profiles are keyed by agent name | read | packages/bee-rs/crates/bee/src/onboard/templates.rs:484 | ("bee-build", &["task", "todowrite", "webfetch", "websearch", "lsp"]), |
| 11 | The regen chain re-renders agent files through onboard --apply | ran | .bee/bin/bee dev regen --help | render-skill-trees, then onboard --repo-root . --apply, then release-manifest --write |
| 12 | prepare pins the claude shell by KIND, and only the fallback arm consults the role table | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2475 | "cell" => "bee-build", |
| 13 | A seat role is its own canonical name, so the role table never answers for hat-*/lane-* | read | packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs:289 | let key = canonical_role(role); |
| 14 | apply already deletes any remove_*_agent_file item the plan emits | read | packages/bee-rs/crates/bee/src/onboard/apply.rs:505 | join_rel(repo_root, rel) |

## Discovery

- Chesterton's fence on the advisor: `guard.rs:305-313` says bee renders
  no advisor agent on purpose because "an advisor's model comes from the
  advisor slot rather than from an agent file". That reason expired with
  agent-model-unpin D1/D2 (`onboard/agents.rs:219-222`): agent files now
  render unconditionally and the dispatch payload's model param is the
  authority. A `bee-elder` shell therefore carries prompt and tool
  contract only, never a model pin — the fence can come down.
- Seat roles (`hat-*`, `lane-*`) dispatch as `--kind advisor` with the
  marker naming the resolved seat, and a seat is its own canonical name
  (claim 13), so a `ROLE_AGENTS` row alone reaches `bee-elder` only for the
  bare advisor role. The shell is therefore pinned by KIND in prepare
  (claim 12: an `"advisor" => "bee-elder"` arm beside the `cell` and
  `gather` arms). On the guard side `bee-elder` + `[bee-tier: hat-value]`
  takes the marker + model-param branch (`model_guard.rs:1275`, then the
  strict-equality check) and is allowed; the general-purpose repair fires
  under `[bee-tier: advisor]`, and asr-1 extends it to a seat marker whose
  prompt carries `[bee-kind: advisor]`. asr-1 adds the tests that prove
  all three.
- Herding panes for a non-cell dispatch are prefixed with the pinned
  shell's embedded body (`prepare.rs:2489-2494`). Today an advisor pane
  gets no body (general-purpose has none); after this change it gets the
  elder contract in front of its brief. That is intended — the contract is
  the same read-only digest brief the pane needs — and is recorded as part
  of decision `e8ffde50` and in the test matrix.
- Old-name habits: a bare `subagent_type: "bee-build"` lands in
  `bare-denied` (`model_guard.rs:1634-1639`), whose FIX lists the live
  shells. asr-1 adds a one-line rename hint from the retired-name table
  ("bee-build is now bee-worker") to that refusal, and denies a marker
  dispatch whose `subagent_type` is a retired name instead of letting the
  host fail with its own "agent not found".
- Retired rendered files are removed unconditionally on `--apply`
  (`apply.rs:796-810` already deletes any `remove_*_agent_file` item).
  bee has always overwritten its rendered agent files byte-for-byte on
  sync, so a hand edit was never preserved; the plan item carries
  `replacement` so the apply line reads "removed retired shell X (now Y)".
- On OpenCode `bee-elder` renders only when `team.opencode.advisor` names
  a model: `AGENT_TIER_DEFAULTS_OPENCODE` seeds no advisor, and an
  opencode agent file with no model is not a file (`agents.rs:138-141`).
  The deny row is still declared so a host that sets the slot gets it.
- The wave-close regen MUST run the freshly built binary
  (`cargo build --release`, then that `bee dev regen`), never the vendored
  `.bee/bin/bee` from 2.74: the name tables are compiled in, and the old
  binary would render nothing for the new templates and remove nothing.
- Cheap reality touch: `rg` over `packages/bee-rs/crates/bee/src` shows 19
  files naming the old shells; 11 of them are tests. `bee-design` also
  names the design SKILL and its packs (`bee-design-*`) — those stay; only
  the agent (`.claude/agents/bee-design.md`, template, tables) renames.
- Regen is a wave-barrier obligation: both cells edit sources only; the
  leader runs `bee dev regen` once at wave close, which re-renders
  `.claude/agents/`, `.opencode/agent/`, `AGENTS.md` and the manifest.

## Approach

One rename, one new shell, no compatibility alias. A retired-name list in
`templates.rs` lets the onboarding plan remove the five old rendered files
on any host (claude and opencode trees) so an upgrade leaves no shell the
guard would refuse by name. The advisor kind pins `bee-elder` on claude
through the same `ROLE_AGENTS` row every other shell uses.

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 (this slice) | names in code, templates, tests, prose, knowledge | user approved the set | `bee dispatch prepare --kind advisor` returns `bee-elder`; `--kind cell` returns `bee-worker` | release 2.75 |

## Cells — current slice (preview)

```json
[
  {
    "id": "asr-1",
    "feature": "agent-shell-rename",
    "title": "Rename the rendered agent shells and add bee-elder for the advisor kind",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["e8ffde50-2e43-499e-86ee-a1d2eece215b"],
    "files": [
      "packages/bee/agents/bee-worker.md.tmpl",
      "packages/bee/agents/bee-comb.md.tmpl",
      "packages/bee/agents/bee-forager.md.tmpl",
      "packages/bee/agents/bee-scout.md.tmpl",
      "packages/bee/agents/bee-guard.md.tmpl",
      "packages/bee/agents/bee-elder.md.tmpl",
      "packages/bee/agents/bee-build.md.tmpl",
      "packages/bee/agents/bee-design.md.tmpl",
      "packages/bee/agents/bee-gather.md.tmpl",
      "packages/bee/agents/bee-extract.md.tmpl",
      "packages/bee/agents/bee-review.md.tmpl",
      "packages/bee-rs/crates/bee/src/onboard/templates.rs",
      "packages/bee-rs/crates/bee/src/onboard/agents.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/wave_run.rs",
      "packages/bee-rs/crates/bee/src/hooks/model_guard.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs",
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/verbs/status_full/store.rs",
      "packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs",
      "packages/bee-rs/crates/bee/tests/installer_contracts.rs",
      "packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/src/verbs/reviews.rs"
    ],
    "read_first": [
      "docs/history/agent-shell-rename/plan.md",
      "packages/bee-rs/crates/bee/src/onboard/templates.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs",
      "packages/bee-rs/crates/bee/src/onboard/agents.rs",
      "packages/bee/agents/bee-review.md.tmpl"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md"],
    "regen_obligation_ack": "wave-barrier",
    "action": "Rename the five agent templates with git mv (bee-build→bee-worker, bee-design→bee-comb, bee-gather→bee-forager, bee-extract→bee-scout, bee-review→bee-guard) and update each file's `name:` and every self-reference in its body. Add packages/bee/agents/bee-elder.md.tmpl modelled on bee-review.md.tmpl: read-only tools (Read, Grep, Glob), description 'Advisor seat for the bee Delegation contract — a session-class consult, a hat seat, or a blind lane; reached only through bee dispatch prepare', body = the advisor contract (read-only, digest with recommendation/confidence/falsifier, no state writes, no decisions). In onboard/templates.rs rename every row of AGENT_ROLES_BY_NAME and AGENT_OPENCODE_PERMISSION_DENY, add (\"bee-elder\", &[\"advisor\"]) and a bee-elder deny row equal to bee-scout's, and add `pub const RETIRED_AGENT_NAMES: &[(&str, &str)]` holding the five (old, new) pairs. In onboard/agents.rs make compute_agent_file_plan and compute_opencode_agent_file_plan also emit remove_agent_file / remove_opencode_agent_file for every retired path that exists (search `for tmpl_name in list_template_agents`), each item carrying `replacement: <new name>`; apply.rs already deletes such items (apply.rs:796-810) and needs no change — only make its printed line name the replacement when the item carries one. In verbs/drivers/guard.rs rename every ROLE_AGENTS row (the array type grows to 6), add (\"advisor\", \"bee-elder\") as the LAST row, rename agent_job_summary arms (add bee-elder: 'advises read-only from a fixed seat and reports (never writes, never decides)'), and rewrite the doc text that says bee renders no advisor agent. In verbs/drivers/prompt.rs rename the include_str! paths and match arms and add AGENT_ELDER. In verbs/drivers/prepare.rs add an `\"advisor\" => \"bee-elder\"` arm to the pinned_type match (search `\"cell\" if design_role =>`) so every --kind advisor dispatch on claude — bare advisor role AND seat roles such as hat-value or lane-2 — returns subagent_type bee-elder while the marker still names the RESOLVED role; add a test for each in drivers/tests.rs, and one asserting a herding-shaped advisor slot now prefixes the pane prompt with the elder body. In hooks/model_guard.rs: (1) extend the general-purpose repair so a prompt carrying `[bee-kind: advisor]` with a seat marker is repaired onto bee-elder; (2) in the bare-denied refusal, when subagent_type is a retired name, prepend one line 'subagent_type \"bee-build\" was renamed to \"bee-worker\"' from RETIRED_AGENT_NAMES; (3) deny a marker dispatch whose subagent_type is a retired name with that same rename line (reason `retired-agent-type`) instead of allowing it through to the host's own not-found error. Drop `advisor` from the loop in the existing test `a_role_with_no_rendered_agent_is_never_repaired_to_undefined` (it now has a rendered agent). Add tests: `[bee-tier: hat-value]` + bee-elder allowed; `[bee-tier: advisor]` + general-purpose repaired onto bee-elder; `[bee-tier: hat-value]` + general-purpose + `[bee-kind: advisor]` repaired onto bee-elder; bare `bee-build` denied with the rename line; `[bee-tier: code]` + `bee-build` denied retired-agent-type. In doctor.rs point DESIGN_AGENT_REL and its messages and tests at .claude/agents/bee-comb.md. In generated/registry_payload.json rename the bee-extract mention in the dispatch.prepare --role description (this file is hand-maintained, not regenerated). In verbs/reviews.rs change the test git email bee-review@example.com to bee-guard@example.com. Sweep every remaining old name in the listed files (tests, fixtures, FIX strings such as hook_local.rs 'dispatch a bee-extract worker'). Do NOT touch the bee-design skill or bee-design-* packs anywhere (skills.rs, doctor DESIGN_SKILL, design/). No code comments; rewrite the touched `//`/`///` lines that explain the tables so they name the new shells, never add new ones. Do not run bee dev regen or onboard --apply: the leader runs the regen chain once at wave close.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml",
    "must_haves": {
      "truths": [
        "bee dispatch prepare --runtime claude --kind cell returns subagent_type bee-worker; --kind gather returns bee-forager; --kind gather --role extraction returns bee-scout; --kind reviewer returns bee-guard; a design-role cell returns bee-comb",
        "bee dispatch prepare --runtime claude --kind advisor returns subagent_type bee-elder, and --kind advisor --role hat-value also returns bee-elder with [bee-tier: hat-value]",
        "The model guard allows subagent_type bee-elder under a seat marker, repairs general-purpose onto bee-elder under [bee-tier: advisor] and under a seat marker with [bee-kind: advisor], and denies a retired name (bare or under a marker) with a one-line rename hint",
        "On a host that still holds .claude/agents/bee-build.md (or any retired name), bee onboard plans remove_agent_file for it with replacement named; same for .opencode/agent/",
        "A herding-shaped advisor slot's pane prompt is prefixed with the bee-elder body",
        "`rg -n 'bee-build|bee-gather|bee-extract|bee-review\\b' packages/` returns nothing; bee-design remains only as the skill and pack names",
        "The full cargo suite is green"
      ],
      "artifacts": [
        {"path": "packages/bee/agents/bee-elder.md.tmpl", "substantive": "a read-only advisor shell with the advisor contract"},
        {"path": "packages/bee-rs/crates/bee/src/onboard/templates.rs", "substantive": "renamed tables, bee-elder rows, RETIRED_AGENT_NAMES"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs", "substantive": "renamed ROLE_AGENTS with the advisor row"}
      ],
      "key_links": [
        "prepare.rs pinned_type match pins bee-elder on --kind advisor; ROLE_AGENTS carries the advisor row for the guard's repair and role_for_agent",
        "onboard/agents.rs plan removes RETIRED_AGENT_NAMES files with their replacement named",
        "model_guard.rs reads RETIRED_AGENT_NAMES for the rename hint"
      ],
      "prohibitions": [
        "No alias that keeps an old agent name accepted",
        "No change to role names or team.<runtime> config shape",
        "No rename of the bee-design skill or bee-design-* packs",
        "No bee dev regen or onboard --apply inside the cell"
      ]
    }
  },
  {
    "id": "asr-2",
    "feature": "agent-shell-rename",
    "title": "Move the prose, the site guide and the knowledge concept to the new shell names",
    "lane": "standard",
    "role": "docs",
    "deps": [],
    "decisions": ["e8ffde50-2e43-499e-86ee-a1d2eece215b"],
    "files": [
      "skills/bee-swarming/references/swarming-reference.md",
      "skills/bee-hive/references/gates-and-delegation.md",
      "site/guide/vi/giao-viec.html",
      "site/guide/vi/config.html",
      "site/guide/vi/tu-vung.html",
      "docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md",
      "docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md",
      "docs/knowledge/areas/bee-herding/handing-a-foreign-agent-its-brief.md",
      "docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md",
      "docs/knowledge/areas/workflow-state/overview.md",
      "docs/knowledge/areas/onboarding/onboarding-design-packs-the-design-role-and-the-design-gate.md",
      "docs/product-description/delegation/dispatch.md",
      "docs/product-description/delegation/workers.md",
      "docs/product-description/glossary.md",
      "docs/product-description/goal.md",
      "docs/product-description/reviews/reviewing.md",
      "docs/product-description/verification/areas.md"
    ],
    "read_first": [
      "docs/history/agent-shell-rename/plan.md",
      "docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md",
      "site/guide/vi/giao-viec.html"
    ],
    "affects_skills": [
      "skills/bee-swarming/references/swarming-reference.md",
      "skills/bee-hive/references/gates-and-delegation.md"
    ],
    "affects_specs": [
      "docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md",
      "docs/knowledge/areas/bee-herding/handing-a-foreign-agent-its-brief.md"
    ],
    "regen_obligation_ack": "wave-barrier",
    "action": "In every listed file replace the agent shell names: bee-build→bee-worker, bee-gather→bee-forager, bee-extract→bee-scout, bee-review→bee-guard, and the AGENT bee-design→bee-comb (the bee-design SKILL and bee-design-* packs keep their names — read each hit and decide by context; `.claude/agents/bee-design.md` and `packages/bee/agents/bee-design.md.tmpl` are the agent). Where prose says the advisor runs as general-purpose or that bee renders no advisor agent, say it runs as bee-elder; in docs/knowledge/areas/bee-herding/handing-a-foreign-agent-its-brief.md (search `embedded_agent_body`) record that an advisor pane is now prefixed with the bee-elder body. Do not edit AGENTS.md at the repo root, .claude/agents/ or .opencode/agent/ (the regen chain renders them at wave close). In docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md add one short section 'The six rendered shells' listing each shell, the job it serves, whether it writes, and the decision id e8ffde50-2e43-499e-86ee-a1d2eece215b; keep historical decision text that quotes old names as-is when it is a quote. In site/guide/vi/giao-viec.html make the shell table (search `Bốn agent`) the ONE home for all six names in Vietnamese with each one's job and tool set: fix the heading count (four→six), drop the stale 'một model cố định' sentence (agent files no longer pin a model), and fix the claim that bee-build is the only writer (bee-worker and bee-comb both write). In site/guide/vi/tu-vung.html (search `Một trong bốn`) keep one glossary row per name but link each to the giao-viec table instead of repeating the descriptions. Leave docs/history/** and docs/knowledge/patterns/** untouched (history quotes). Write through the bee-technical-writing skill; no new bee terms the reader does not need.",
    "verify": "rg -n 'bee-build|bee-gather|bee-extract|bee-review\\b' skills site/guide docs/knowledge/areas docs/product-description — expected: zero hits",
    "must_haves": {
      "truths": [
        "rg for the four old non-design names over skills/, site/guide/, docs/knowledge/areas/ and docs/product-description/ returns nothing",
        "model-roles-and-escalation.md names the six shells and cites the decision",
        "giao-viec.html is the one home of the six names; tu-vung.html links there; the 'four' and 'only writer' claims are gone",
        "The bee-design skill and packs are still named bee-design everywhere"
      ],
      "artifacts": [
        {"path": "docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md", "substantive": "a 'The six rendered shells' section"},
        {"path": "site/guide/vi/giao-viec.html", "substantive": "a six-row shell table"}
      ],
      "key_links": ["tu-vung.html rows link to the giao-viec.html shell table", "handing-a-foreign-agent-its-brief.md names the bee-elder pane prefix"],
      "prohibitions": ["No edit to AGENTS.md, .claude/agents/, .opencode/agent/, docs/history/, docs/knowledge/patterns/", "No rename of the bee-design skill"]
    }
  }
]
```

## Test matrix

| Case | Proof |
|---|---|
| each dispatch kind names its new shell | drivers/tests.rs cases (asr-1) |
| advisor kind and seat roles name bee-elder | drivers/tests.rs + model_guard.rs tests (asr-1) |
| retired rendered files are removed by the onboarding plan | onboard/agents.rs tests (asr-1) |
| a retired name typed by habit is denied with a rename hint | model_guard.rs tests (asr-1) |
| a herding advisor pane is prefixed with the bee-elder body | drivers/tests.rs (asr-1) |
| prose carries no old name | rg verify line (asr-2) |
| user-visible path | after regen: `bee dispatch prepare --runtime claude --kind advisor --role hat-value --json` shows bee-elder; `--kind cell` shows bee-worker (`green:live` at close) |

## Open Questions

(none)

## Out of scope

- Any alias keeping the old names alive.
- Renaming roles (`code`, `read`, `review`, …) or the `team.<runtime>` shape.
- The bee-design skill family.
- Hosts' own `.claude/agents/` files not rendered by bee.
