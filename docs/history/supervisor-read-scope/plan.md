---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: A supervising project reads the bee state of the projects it manages

## Summary

A repo can now say "I supervise the waggledance projects". When it does, a
session there may read the bee state and the knowledge folder of every
project waggledance has registered. It can read only. Source code, writes,
git and secret files in those projects stay blocked, and the block message
says which rule tripped and what is open. Two measurement commands take a
`--root` path so the supervisor can measure another project's store without
moving into it, and without writing into it.

Mode: `high-risk` — 3 risk flags: audit/security (a guard opens a read path), public-contracts (a new config key and two new flags), external-systems (the waggledance CLI).
Why this is the least workflow that protects the work: the change loosens a security guard for every project that opts in, so it needs the full hat wave and a test for each way the grant could leak.

## Requirements (from CONTEXT.md)

- D1: opt-in governed key `guards.supervises: "waggledance"`; other `guards.*` keys stay settable.
- D2: managed roots from `waggledance list --json`, only on the would-be refusal path, 2 s kill, never a binary inside this project, fail closed, no panic, no cache.
- D3: canonical root match or shared git common dir (symlinked roots, worktrees).
- D4: surface is `.bee/**` and `docs/knowledge/**` of the resolved root, minus secret-shaped files and `config.local.json`; directory-wide Grep refused when the directory holds one.
- D5: Read/Grep/Glob in the surface; Bash only with literal tokens, no newline, no `cd`, cwd in this project, heads cat/head/tail/wc/ls/jq, and the two bee report verbs as bare `bee` or this store's binary.
- D6: `--root <path>` on `bee knowledge report` and `bee report outcomes`; no byte of the target store changes.
- D7: with the key, a refusal names the rule, the surface, or the registry state; without it, byte-identical.
- C1: injected registry provider; tests (a)-(e) plus the leak cases.

## Load-bearing claims

Labels: `read` = opened at that line; `ran` = executed. Evidence is a verbatim substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The candidate loop sees each resolved other root, so the grant sits there | read | packages/bee-rs/crates/bee/src/hooks/write_guard/cross_project.rs:275 | `let Some(other) = scope.other_root(&raw, base) else { continue };` |
| 2 | The guard runs from the write-guard entry before any other read check | read | packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:241 | `if let Some(reason) = cross_project_denial(` |
| 3 | A secret-path predicate exists, and it checks a basename | read | packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs:192 | `pub(crate) fn is_secret_path(normalized: &str) -> bool {` |
| 4 | The governed view protects one guards key by name | read | packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs:1141 | `if let Some(allow) = root_obj.get("guards").and_then(\|g\| g.get("cross_project_allow")) {` |
| 5 | The tokenizer separators carry no newline | read | packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs:338 | `matches!(t, "&&" \| "\|\|" \| ";" \| "\|" \| "&")` |
| 6 | The tokenizer folds a newline into whitespace | read | packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs:296 | `if ch == ' ' \|\| ch == '\t' \|\| ch == '\n' \|\| ch == '\r' {` |
| 7 | The waggledance CLI prints the registry as JSON with root_path | ran | waggledance list --json | `{"projects":[{"id":"haw-dashboard","name":"haw-dashboard","root_path":"/mnt/data/Projects/anphabe/haw-dashboard","file_count":328}` |
| 8 | The registry holds stale /home roots that need canonicalising | ran | waggledance list --json (root_path values under /home) | `'/home/thanhsmind/Projects/goglbe/beehive'` |
| 9 | report outcomes writes timing into the store it resolved | read | packages/bee-rs/crates/bee/src/verbs/report_outcomes.rs:72 | `record_timing(&roots.root, CMD, t0, true);` |
| 10 | knowledge report resolves its root through the shared prelude | read | packages/bee-rs/crates/bee/src/verbs/knowledge/frame.rs:40 | `pub(crate) fn g_prelude(cmd: &'static str, json: bool, pre_json: bool, t0: Instant) -> Option<GPre> {` |
| 11 | Other guards keys are user toggles, so the whole subtree cannot be governed | read | docs/config-reference.md:437 | `(GH #71) — letting the agent keep its own memory` |

## Discovery

The leader read `cross_project.rs` in full, the knowledge section
"Cross-project refusal" in `docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md`,
and `docs/history/cross-project-guard/CONTEXT.md`. `waggledance list --json`
returns the registry in about 5 ms (36 projects, 8 with stale `/home/...`
roots), so no waggledance change is needed. The five-seat hat wave (reports
under `.bee/mailbox/job-1794403716*/report-1.md` and
`job-1794403763769-471931-1`) found the newline, expansion, `cd`, fake-`bee`,
sqlite3, directory-secret and `--root` timing leaks; D2, D4-D7 were
superseded to close them.

## Approach

Recommended path: a supervisor grant inside `cross_project.rs`, consulted in
the candidate loop only after a candidate resolved to another project (claim
1, D2), so a repo without the key and every in-repo call pay nothing. The
grant asks an injected provider for managed roots (C1); production runs
`waggledance list --json` with a 2 s kill. The shell grant is a strict
allowlist over literal tokens (D5). A separate cell adds `--root` (D6) and
moves timing off the target store; it shares no file with the guard.

Rejected: parse waggledance's `registry.db` (couples bee to its sqlite
schema); a daemon-exported registry file (needs a waggledance change, stale
when the daemon is down); a script that fills `guards.cross_project_allow`
(the hand list the owner refused, and it opens source reads); govern the
whole `guards` subtree (claim 11: it would lock user toggles); a Bash grant
of the two bee verbs only (drops `cat | jq`, which the spec names).

Risk map:
- A write or program run slips through the shell grant — HIGH — Lands in: sprs-1 — proof: refusal tests for newline, `$`, backtick, brace/glob path, redirect, `cd`, `tee`, `rm`, `git`, `sqlite3`, `rg`, mutating `bee`, a `/tmp/x/bee` head, and a mixed managed/unmanaged command.
- Secret content readable — HIGH — Lands in: sprs-1 — proof: `<m>/.bee/config.local.json`, `<m>/.bee/.env` Read refused; Grep `path=<m>/.bee` with `config.local.json` present refused.
- Provider hang, panic or failure becomes an allow — HIGH — Lands in: sprs-1 — proof: unit tests with an empty provider, and hook contract tests with a fake `waggledance` on PATH that exits 1, prints garbage, or sleeps past 2 s: exit 2 each.
- Repos without the key change behavior — MEDIUM — Lands in: sprs-1 — proof: byte-identical refusal text test with no key, and a test that a key value other than `waggledance` grants nothing.
- `--root` writes into the target store — HIGH — Lands in: sprs-2 — proof: a test that snapshots the target store's file list and bytes before and after both verbs.

Waves: sprs-1 and sprs-2 run in parallel (no shared files); sprs-3 waits for both. After merge the leader reinstalls `.bee/bin/bee` from the release build and checks the new refusal text in the installed binary before any live check.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"sprs-1 writes the guard grant and its tests; sprs-2 writes the --root flags."},
    {"stage":"writing","classification":"required","role":"writer","reason":"sprs-3 consolidates knowledge, the owner setup steps and delivery.md."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"High-risk: all five hat seats run the plan check."},
    {"stage":"hat-risks","classification":"required","role":"hat-risks","reason":"High-risk: a guard opens a read path."},
    {"stage":"hat-value","classification":"required","role":"hat-value","reason":"High-risk hat wave."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"High-risk hat wave; cites the SMALLER PATH check below."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"High-risk hat wave."},
    {"stage":"docs","classification":"not-applicable","role":"docs","reason":"sprs-1 carries the config-reference delta; sprs-3 the knowledge delta."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"Release runs through scripts/release.sh by the leader."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Each cell writes its own red-first tests."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"The leader read the guard during planning."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"No narrow lookup is left."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"design-ui","classification":"not-applicable","role":"design","reason":"No rendered UI."}
  ]
}
```

## Shape

Epic map, one slice. Playbook: `skills/bee-planning/playbooks/feature.md` (cited, not copied).

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Supervisor grant | read-only access to managed stores | owner decision 033faaf6 | 1 | guard tests (a)-(e) + leak tests + hook contract tests |
| Remote measurement | `--root` on two report verbs | spec item 5 | 1 | verb tests + byte-identical target store |

SMALLER PATH: a script that writes every registry root into `guards.cross_project_allow` is cheaper, but it is the hand-kept list the owner refused and it opens source reads too. A Bash grant of the two bee verbs only is cheaper, but drops `cat | jq`, which the spec names; the strict literal-token allowlist keeps it at a small, tested cost. Dropping D6 is cheaper, but the spec lists it. PASS.

Plan check (hat wave synthesis) — STRUCTURE: BLOCKERS from round 1 (newline, `cd`/root exception, fake `bee` head, sqlite3) are closed by D5 8452f29c; the `--root` write is closed by D6 a8d8ad76; the directory secret leak by D4 2c92af66; the provider hang by D2 5c498adf. WARNINGS applied: knowledge edit moved to sprs-3; `frame.rs` added to sprs-2; governed view builds a `guards` map holding either key; test names carry `cross_project_supervisor`. Rejected: govern the whole `guards` subtree (claim 11). Deferred: a `bee doctor` line showing the grant is active (not asked for). CELLS: cold pickup checked for each cell; no CRITICAL.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| sprs-1 | Let a supervising repo read the bee store and knowledge of its managed projects | cross_project.rs, guards.rs, tests.rs, hook_contracts.rs, verbs/config.rs, docs/config-reference.md | — | From a repo with `guards.supervises`, `Read <managed>/.bee/state.json` and `cat <managed>/.bee/x \| jq .` pass; `src/`, writes, secrets, tricks and unregistered repos are still refused, with a reason | `cargo test … --bin bee -- cross_project` and `--test hook_contracts cross_project_supervisor` |
| sprs-2 | Measure another project's store with --root | knowledge/report.rs, knowledge/frame.rs, report_outcomes.rs, generated/registry_payload.json | — | `bee knowledge report --root <other>` and `bee report outcomes --root <other>` print that project's numbers and leave its store untouched | `cargo test … --bin bee -- report_root` |
| sprs-3 | Consolidate this feature's knowledge and write its delivery | docs/knowledge/**, delivery.md | sprs-1, sprs-2 | the guard concept and delivery.md describe the grant and the owner setup steps | `.bee/bin/bee knowledge check` |

```json
[
  {
    "id": "sprs-1",
    "feature": "supervisor-read-scope",
    "title": "Let a supervising repo read the bee store and knowledge of its managed projects",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["902d1477", "5c498adf", "318dee0d", "2c92af66", "8452f29c", "06fe4cb5", "68db537b"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/cross_project.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs",
      "packages/bee-rs/crates/bee/tests/hook_contracts.rs",
      "packages/bee-rs/crates/bee/src/verbs/config.rs",
      "docs/config-reference.md"
    ],
    "read_first": [
      "docs/history/supervisor-read-scope/CONTEXT.md",
      "docs/history/cross-project-guard/CONTEXT.md",
      "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md",
      "docs/knowledge/patterns/20260714-a-fail-open-host-swallows-fail-closed-throws.md",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/cross_project.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs",
      "packages/bee-rs/crates/bee/src/verbs/config.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Implement docs/history/supervisor-read-scope/CONTEXT.md D1-D5, D7 and contract C1 in the cross-project guard; CONTEXT.md is the full rule text, follow it exactly. Inputs: the existing cross_project_denial arguments plus guards.supervises from .bee/config.json and .bee/config.local.json (same merge as cross_project_allow in Scope::load). Grant entry: in the candidate loop of cross_project_denial (anchor: `let Some(other) = scope.other_root(&raw, base) else { continue };`), only when guards.supervises is exactly the string \"waggledance\" and a candidate resolved to another root, ask a managed-roots provider once per call. Production provider (D2): resolve waggledance on the hook PATH, skip it when its canonical path lies inside this project, run `waggledance list --json` with a 2 s kill, take projects[].root_path; any failure yields a distinct 'registry unreadable' result, never a panic and never an unwrap on the spawn path. Managed match per D3, surface per D4 built from the resolved other root (so a worktree's own .bee counts). Read/Grep/Glob and Bash/exec grant rules exactly per D5, evaluated over the whole command: every other-project candidate must pass, and a single failing candidate or segment refuses the call. bee head check: bare `bee`, or a token whose canonical path equals <store root>/.bee/bin/bee of this project; segment shape must be `bee knowledge report ...` or `bee report outcomes ...`. Refusal text per D7: with no key, byte-identical to today; with the key, today's text plus one sentence naming the tripped rule and the readable surface, or that the registry was unreadable, or that the project is not in the registry. Keep cross_project_denial's signature; add a sibling entry point taking the provider (C1). Governed key (D1): make config_governed_view build the `guards` map when either cross_project_allow or supervises is present and carry both; make diff_governed_views refuse a change to guards.supervises like guards.cross_project_allow; leave other guards keys ungoverned. Red first, every new test name contains cross_project_supervisor. In write_guard/tests.rs with an injected provider: (a) Read <m>/.bee/state.json allowed; (b) Bash `echo x > <m>/.bee/state.json`, `cat <m>/.bee/state.json | tee x`, `git -C <m> status`, `cat <m>/.bee/state.json` followed by a newline and `rm <m>/.bee/state.json` all refused; (c) Read <m>/src/main.rs refused and stderr names the surface; (d) unregistered sibling refused with the not-in-registry sentence; (e) a provider root that is a symlink to <m> grants; plus: <m>/.bee/config.local.json and <m>/.bee/.env Read refused; Grep path=<m>/.bee with config.local.json present refused, Grep path=<m>/.bee/cells granted; `cat <m>/.bee/state.json | jq .` and `cat <m>/.bee/state.json 2>/dev/null` granted; `cat <m>/.bee/{state.json,../src/main.rs}`, `cat <m>/.bee/$x/state.json`, backtick, `cd <m>/.bee && cat state.json`, `sqlite3 -readonly <m>/.bee/x.db`, `rg x <m>/.bee/cells`, `/tmp/x/bee knowledge report --root <m>`, `bee cells add` with --root <m>, and a mixed `cat <m>/.bee/state.json <unmanaged>/x` refused; `bee knowledge report --root <m>` and `.bee/bin/bee report outcomes --root <m>/.bee` granted; Glob path=<m>/.bee pattern=../src/** refused; a git worktree of <m> granted; key absent gives byte-identical text; key value other than waggledance grants nothing; key set only in config.local.json grants; empty or failed provider gives the registry-unreadable sentence. In tests/hook_contracts.rs: run the real hook with a fake waggledance first on PATH that exits 1, prints garbage, or sleeps 5 s; each gives exit 2 within 4 s. In verbs/config.rs: a cross_project_supervisor test that bee config set guards.supervises is refused. Watch them fail before the fix. Document guards.supervises in docs/config-reference.md beside guards.cross_project_allow, including that only the user sets it by hand-editing .bee/config.json, citing docs/history/supervisor-read-scope/CONTEXT.md D-numbers, never raw decision ids. Do not edit docs/knowledge (sprs-3 owns it). No code comments. One commit.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- cross_project",
    "must_haves": {
      "truths": [
        "With guards.supervises set to waggledance, a Read of a managed project's .bee/state.json is allowed",
        "A write-shaped shell command, a git command, a mutating bee verb or a newline-joined second command on a managed project is refused",
        "A managed project's source outside .bee/ and docs/knowledge/ is refused, and the refusal names the readable surface",
        "A project absent from the waggledance registry is refused",
        "A symlinked registry root and a worktree of a managed repo both resolve to the managed project",
        "Secret-shaped files and config.local.json inside a managed .bee/ stay unreadable, a directory-wide Grep included",
        "A failing, garbage or hanging waggledance gives exit 2 with the registry-unreadable sentence",
        "Without guards.supervises every refusal text is byte-identical to before",
        "guards.supervises cannot be added or changed through bee config set or an agent edit"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/cross_project.rs", "substantive": "the supervisor grant, the registry provider and the shell allowlist"},
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs", "substantive": "red-first cross_project_supervisor tests (a)-(e) and the leak cases"},
        {"path": "packages/bee-rs/crates/bee/tests/hook_contracts.rs", "substantive": "fake-waggledance hook contract tests"}
      ],
      "key_links": [
        "cross_project_denial consults the supervisor grant only after a candidate resolved to another project",
        "config_governed_change_deny protects guards.supervises"
      ],
      "prohibitions": [
        "Never weaken the existing refusal for a repo without guards.supervises",
        "Never let an environment variable other than the hook PATH lookup of waggledance affect the grant",
        "Never parse waggledance's sqlite registry directly",
        "No code comments"
      ],
      "open_choices": ["Memoise the provider result within one hook call"]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "sprs-2",
    "feature": "supervisor-read-scope",
    "title": "Measure another project's store with --root",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["a8d8ad76"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/frame.rs",
      "packages/bee-rs/crates/bee/src/verbs/report_outcomes.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json"
    ],
    "read_first": [
      "docs/history/supervisor-read-scope/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/frame.rs",
      "packages/bee-rs/crates/bee/src/verbs/report_outcomes.rs",
      "packages/bee-rs/crates/bee/src/verbs/mod.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Implement docs/history/supervisor-read-scope/CONTEXT.md D6. Add an optional --root <path> flag to bee knowledge report (entry run_report in verbs/knowledge/report.rs, which takes its root through g_prelude in verbs/knowledge/frame.rs and rejects unknown keys through keys_known) and bee report outcomes (entry run in verbs/report_outcomes.rs, which resolves with resolve_store_root_worktree(&cwd) and then calls record_timing(&roots.root, CMD, t0, true)). With --root, resolve the store root from that path (a store root or any path inside one) exactly as each verb resolves it from the cwd today; without it, behavior is unchanged. With --root, no byte of the target store changes: record timing and any log row in the cwd store, or skip them when the cwd holds no store. A --root that is not inside a bee store exits with the verb's existing no-root error naming that path. If g_prelude needs a new parameter, keep every other caller's behavior unchanged. Register the flag in packages/bee-rs/crates/bee/src/generated/registry_payload.json for both verbs so bee <verb> --help lists it, and keep bee dev regen clean. Red first, test names containing report_root: knowledge report --root <fixture store> reads that store while the cwd is a different store; report outcomes --root <fixture store> counts that store's cells; --root <fixture>/.bee works; a non-store --root exits with the no-root error; after both verbs the fixture store's file list and every file's bytes are identical to before. No code comments. One commit.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- report_root",
    "must_haves": {
      "truths": [
        "bee knowledge report --root <path> measures the store at that path, not the cwd store",
        "bee report outcomes --root <path> counts the cells of the store at that path",
        "No byte of the --root store changes, timing included",
        "A --root that is not a bee store exits with the no-root error",
        "Without --root both verbs behave as before"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/report_outcomes.rs", "substantive": "--root resolution, timing kept off the target, report_root tests"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs", "substantive": "--root resolution and report_root tests"}
      ],
      "key_links": ["bee <verb> --help lists --root for both verbs"],
      "prohibitions": ["No write to the --root store", "No code comments"],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "sprs-3",
    "feature": "supervisor-read-scope",
    "title": "Consolidate this feature's knowledge and write its delivery",
    "lane": "high-risk",
    "role": "writer",
    "status": "open",
    "deps": ["sprs-1", "sprs-2"],
    "decisions": ["902d1477", "5c498adf", "318dee0d", "2c92af66", "8452f29c", "a8d8ad76", "06fe4cb5"],
    "files": [
      "docs/knowledge/**",
      "docs/history/supervisor-read-scope/delivery.md"
    ],
    "read_first": [
      "docs/history/supervisor-read-scope/CONTEXT.md",
      "docs/history/supervisor-read-scope/plan.md",
      "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Run the writer consolidation step on this feature: run `bee knowledge consolidate --feature supervisor-read-scope`. Extend the section 'Cross-project refusal' of docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md with the supervisor grant (D1-D5, D7) anchored to the code sprs-1 wrote, and add the --root flag (D6) to the concept that documents bee knowledge report or bee report outcomes. Include the owner setup steps: hand-edit guards.supervises into the supervising repo's .bee/config.json, refresh that repo's .bee/bin/bee to the release that carries it, then Read <managed>/.bee/state.json as a smoke check. Dedupe and trim, never restate code, and write docs/history/supervisor-read-scope/delivery.md listing the concepts changed and why, one line each. Cite CONTEXT.md D-numbers, never raw decision ids. Run `.bee/bin/bee knowledge check` green. One commit.",
    "verify": ".bee/bin/bee knowledge check",
    "must_haves": {
      "truths": [
        "area and pattern concepts describe the feature's changes, anchored to code",
        "docs/history/supervisor-read-scope/delivery.md lists every concept the feature changed"
      ],
      "artifacts": [
        {"path": "docs/history/supervisor-read-scope/delivery.md", "substantive": "the feature's delivery record"}
      ],
      "key_links": ["the delivery lands in the same merge as the code"],
      "prohibitions": ["No restating of code in area concepts", "No code comments"],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false}
  }
]
```

## Test matrix

High-risk: the applicable edge dimensions, each row in sprs-1 or sprs-2.

| Dimension | Probe | Pass when |
|---|---|---|
| Happy path | Read `<m>/.bee/state.json`, `cat <m>/.bee/state.json \| jq .` with the key set | exit 0, no stderr |
| Authorization boundary | Read `<m>/src/main.rs` | exit 2, stderr names `.bee/` and `docs/knowledge/` |
| Write path | `echo x > <m>/.bee/state.json`; `tee`; newline + `rm` | exit 2 |
| Command injection | `$x`, backtick, brace path, `sqlite3`, `rg`, `/tmp/x/bee` | exit 2 |
| Secrets | `<m>/.bee/config.local.json`, `<m>/.bee/.env`, Grep `path=<m>/.bee` | exit 2 |
| Registry failure | fake waggledance exits 1, prints garbage, sleeps 5 s | exit 2 within 4 s, registry-unreadable sentence |
| Unregistered | sibling repo not in the list | exit 2, not-in-registry sentence |
| Aliasing | symlinked registry root; git worktree of managed repo | exit 0 |
| Config tamper | `bee config set guards.supervises`; value other than waggledance | refused; no grant |
| Opt-out | no key | exit 2, byte-identical text |
| Remote measurement | `--root <store>` from another cwd | numbers of that store; target bytes unchanged |
| Bad input | `--root /nonexistent` | no-root error |

## Open Questions

(none)

## Out of scope

- Writes into managed projects (they stay with `waggledance_dispatch`).
- A supervisor other than waggledance.
- A `bee doctor` line that shows the grant is active (not asked for).
- Running the cross-project analysis itself (the owner's follow-up after release).
