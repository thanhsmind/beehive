---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Workspace protocol, dispatch receipts, brief discipline

## Summary

A host project can keep a root `WORKSPACE_PROTOCOL.md` with its human
decision boundaries, merge owner and deploy owner. `bee protocol check`
reports its state, `bee protocol init` writes a valid starter file, and
`bee doctor` shows one advisory row. Every `bee dispatch prepare` record
now carries a digest of the prompt it built and of the protocol file, and
`bee dispatch verify` tells a leader later whether either changed. The
worker prompt forbids option menus, the advisor prompt forbids leaning, and
a leaning `--purpose` on a reviewer or advisor dispatch gets a warning.

Mode: `standard` — 2 risk flags: public contracts (two new verbs, new keys
on the prepare record and output, prompt text), multi-domain (verbs, doctor,
dispatch, prompts, doctrine). Why this is the least workflow that protects
the work: nothing here gates or refuses new work, so the risk is a wrong
parse or a changed prepare output; unit tests on fixtures catch both.

Playbook: `skills/bee-planning/playbooks/feature.md` — cited, not copied.

## Requirements (from CONTEXT.md)

- D1: root `WORKSPACE_PROTOCOL.md`, `KEY: value` lines anywhere, empty value takes `- item` lists, unknown keys kept.
- D2: states missing / unreadable / invalid / valid; MERGE_OWNER, DEPLOY_OWNER, HUMAN_DECISION_BOUNDARIES (≥1 item) required.
- D3: test commands stay in `commands.test`; a differing non-empty FULL_TEST is an issue.
- D4: `bee protocol check` (exit 0 always) and `bee protocol init` (refuses on an existing file; template is valid).
- D5: advisory doctor row `workspace-protocol`, never blocking.
- D6: leader doctrine reads the protocol, excerpts boundaries into briefs; workers never read it.
- D7: every prepare record and output carries `prompt_sha256` and `protocol_sha256`.
- D8: `bee dispatch verify --dispatch-id <id> [--prompt-file]` reports protocol and prompt states.
- D9: worker-cell prompt forbids option menus.
- D10: advisor prompt carries the no-leaning line.
- D11: reviewer/advisor `--purpose` runs `lint_brief`, warn-only `purpose_leaning`.
- D12: no "do not narrow review dimensions" rule.

## Load-bearing claims

Labels: `read` (file opened at that line), `ran` (command run, output held). Evidence is a verbatim substring.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The prepare record is appended in one function to dispatch.jsonl | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1364 | pub(crate) fn append_prepare_record(root: &Path, record: &Map<String, Value>) |
| 2 | The record map is built at one site with dispatch_id | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:3239 | record.insert("dispatch_id".into(), Value::String(dispatch_id.clone())); |
| 3 | The `&str` hasher prepare already uses is sha256_hex in reservations | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:3236 | let brief_sha256 = brief.map(crate::verbs::reservations::sha256_hex); |
| 4 | Herding and cli payloads carry the text in `stdin`, not `prompt` | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2727 | payload.insert("stdin".into(), Value::String(pane_prompt.clone())); |
| 5 | dispatch subverbs route in one match | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:4061 | "prepare" => run_dispatch_prepare(flags, use_json, t0), |
| 6 | The leaning rule has one home | read | packages/bee-rs/crates/bee/src/verbs/drivers/brief_lint.rs:309 | pub(crate) fn lint_brief(brief: &str) -> Result<(), Value> { |
| 7 | Doctor already has advisory rows | read | packages/bee-rs/crates/bee/src/doctor.rs:584 | "status": "advisory" |
| 8 | The command catalog payload is hand-edited, no generator | read | docs/knowledge/areas/workflow-state/gates.md:866 | `src/generated/registry_payload.json` (hand-edited — no generator exists in |
| 12 | lint_brief also runs a four-section shape arm; the stem arm is its own fn | read | packages/bee-rs/crates/bee/src/verbs/drivers/brief_lint.rs:245 | fn first_leaning_stem_outside_tagged(lines: &[&str], fences: &Fences) -> Option<&'static str> { |
| 9 | The reviewer prompt already forbids leaning | read | packages/bee/prompts/reviewer.md:4 | The brief never carries the orchestrator's own verdict, leaning or expected findings. |
| 10 | Prepare records today carry no prompt digest | ran | tail -3 .bee/logs/dispatch.jsonl | "dispatch_id":"c7db8699-1e53-4d72-a787-718fee386269","kind":"gather","cell":null |
| 11 | The leaning guard must not refuse on --purpose | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:3948 | The LEANING GUARD runs here too, on the resolved brief bytes and NOTHING |

## Discovery

The leader read the prepare record path (claims 1-4, 11), the dispatch
subverb router (5), the leaning rule (6), the doctor row shape (7), the
registry home (8) and the prompt files (9), and read today's dispatch log
(10). The slp sources are `paseo-pi-team/templates/WORKSPACE_PROTOCOL.example.md`
and `cli/lib/workspace-protocol.mjs` (four states, report-only),
`paseo-room/protocol/workspace-protocol.template.md` (brief discipline) and
the paseo-doctrine-downstream admission pilot (digest receipts).

## Approach

One new module owns the protocol: parse, state, digest, template. Doctor
and prepare call it; nothing else parses the file. Prepare adds two keys at
the one record site and the envelope, so every kind gets them. `dispatch
verify` reads the log newest-first and recomputes. The prompts and the
doctrine change as text. The purpose lint reuses `lint_brief` and only
adds an output key.

Rejected: keeping the protocol in `.bee/config.json` (the owner edits the
file by hand, and Paseo reads the root file); refusing a dispatch when the
protocol is missing (unproven in paseo, would red every host); a second
leaning rule list for purposes (two lists drift).

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Parser misreads paseo files | MEDIUM | wsp-1 | fixtures from paseo-pi-team and paseo-room formats |
| Prepare output changes break callers | MEDIUM | wsp-2 | existing prepare tests stay green; new keys only add |
| Purpose lint refuses a needed consult | LOW | wsp-2 | test: a leaning purpose still builds the payload |
| Prompt byte tests | LOW | wsp-3 | prompt-render tests green |

Waves: wsp-1 and wsp-3 run in parallel (no file overlap; the registry payload
is hand-edited, so only wsp-3 runs `bee dev regen`, for its prompt and
doctrine copies). wsp-2 follows wsp-1: it calls wsp-1's digest helper and
edits the same router and registry payload. wsp-4 documents last.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "094d64e24e68c0b6fabaed19ae6619f3ac5e32bac9e9da2ab51728125c724d9b",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"wsp-1 and wsp-2 add Rust verbs and prepare keys."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Tests ride each code cell; the live proof is the leader's."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"wsp-3 changes prompts and doctrine; wsp-4 writes the concept."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"conditional","role":"read","condition":"a worker report needs a multi-file check","reason":"Discovery is done; gathers only on demand."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Review is user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the plan consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Standard plan check."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Three seats on standard work."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Three seats on standard work."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Standard plan check."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Standard plan check."},
    {"stage":"design","classification":"not-applicable","role":"design","reason":"No rendered UI changes."}
  ]
}
```

## Shape

Epic map. Outcome: a leader knows the project's human boundaries, can prove
a brief did not change, and briefs carry no menus and no leaning.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Protocol | parse, check, init, doctor row | boundaries live in prose today | wsp-1 | unit tests on fixtures |
| Receipts | prepare digests, dispatch verify, purpose lint | no proof a brief stayed the same | wsp-2 | prepare tests + live verify |
| Discipline | prompts, doctrine | menus and leaning in briefs | wsp-3 | prompt tests, pointer check |
| Record | knowledge concept | the concept must match | wsp-4 | pointer check |

One slice; every cell is current.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| wsp-1 | Add the workspace protocol check, init and doctor row | verbs/protocol.rs (new), verbs/mod.rs, verbs/drivers/close.rs, doctor.rs, generated/registry_payload.json, tests/registry_contracts.rs | — | `bee protocol check` prints the state; `bee protocol init` writes a valid file; doctor shows the row | cargo test protocol; doctor; registry |
| wsp-3 | Forbid option menus in worker briefs and leaning in advisor briefs | packages/bee/prompts/worker-cell.md, packages/bee/prompts/advisor.md, packages/bee/LEADER.md, skills/bee-hive/references/gates-and-delegation.md | — | the worker prompt and the leader doctrine carry the new rules | cargo test prompt |
| wsp-2 | Record prompt and protocol digests at dispatch and add dispatch verify | verbs/drivers/prepare.rs, verbs/drivers/close.rs, verbs/drivers/brief_lint.rs, verbs/drivers/tests.rs, generated/registry_payload.json, tests/registry_contracts.rs | wsp-1 | prepare output has `prompt_sha256` and `protocol_sha256`; `bee dispatch verify` reports same/changed | cargo test prepare; dispatch; registry |
| wsp-4 | Document the workspace protocol and dispatch receipts | docs/knowledge/areas/workflow-state/workspace-protocol.md (new), docs/knowledge/index.md | wsp-1, wsp-2, wsp-3 | one concept names the file, the verbs and the receipt | pointer check |

Paths under `verbs/`, `doctor.rs` and `generated/` are in `packages/bee-rs/crates/bee/src/`; `tests/` is `packages/bee-rs/crates/bee/tests/`. The cell packet:

```json
[
  {
    "id": "wsp-1",
    "feature": "workspace-protocol",
    "title": "Add the workspace protocol check, init and doctor row",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D1", "D2", "D3", "D4", "D5"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/protocol.rs",
      "packages/bee-rs/crates/bee/src/verbs/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/tests/registry_contracts.rs"
    ],
    "read_first": [
      "docs/history/workspace-protocol/CONTEXT.md",
      "docs/knowledge/areas/workflow-state/dispatch.md",
      "packages/bee-rs/crates/bee/src/registry.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "/mnt/data/Projects/refs/slp/paseo-pi-team/templates/WORKSPACE_PROTOCOL.example.md",
      "/mnt/data/Projects/refs/slp/paseo-pi-team/cli/lib/workspace-protocol.mjs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/workflow-state/dispatch.md"],
    "action": "New module verbs/protocol.rs, the ONE home that reads WORKSPACE_PROTOCOL.md. Contract: pub(crate) `PROTOCOL_FILE = \"WORKSPACE_PROTOCOL.md\"`; `parse(text) -> BTreeMap<String, ProtocolValue>` per D1 (scalar or list); `protocol_state(root) -> ProtocolReport { state: missing|unreadable|invalid|valid, issues: Vec<String>, keys, sha256: Option<String> }` per D2 and D3 (FULL_TEST compared against .bee/config.json commands.test read through the existing config reader; string or array joined with \" && \"); `protocol_sha256(control_root) -> Option<String>` using crate::verbs::reservations::sha256_hex over the file bytes (None when missing or unreadable) — wsp-2 calls this; `TEMPLATE` per D4. Verbs: `bee protocol check [--json]` prints state, issues, keys, sha256; exit 0 for every state. `bee protocol init [--json]` writes TEMPLATE to <root>/WORKSPACE_PROTOCOL.md only when absent; an existing file returns a typed refusal {ok:false, type:\"refused\", reason:\"protocol_exists\", path, fix} and leaves the bytes untouched. Wire the verb group through the same router style as `dispatch` in close.rs and declare both verbs and their flags in generated/registry_payload.json (hand-edited, no generator — gates.md:866); if the top-level verb router lives outside close.rs, wire `protocol` there too and name the extra file as a deviation (the registry contract test must stay green; update its pins if it counts verbs). doctor.rs: one row `workspace-protocol` per D5 — missing/valid status `ok`, invalid/unreadable status `advisory` (assert that exact string) with issues and fix `bee protocol check`; never blocking. Tests: parse a paseo-pi-team style fenced file (lists, trailing # comment kept, unknown keys kept); parse a `- key: value` paseo-doctrine style file reads no keys and gives invalid with three issues; missing file is missing; non-UTF-8 bytes is unreadable; init then check is valid; init twice refuses and keeps the first bytes; init prints the review-and-edit next-step line; FULL_TEST equal is fine, different is one issue naming commands.test; a duplicate key gives a `duplicate key` issue and the last value; a scalar HUMAN_DECISION_BOUNDARIES gives an issue naming the `- item` list form; a `<...>` or ` | ` placeholder value gives a placeholder issue (so a copied unfilled paseo template is invalid); a list ends at a blank line; the doctor row always prints, ok with the init hint when missing, advisory and never blocking when invalid. Root: the control root (main checkout holding the bee store), the same root prepare uses. No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee protocol && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee doctor && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts",
    "must_haves": {
      "truths": [
        "bee protocol check reports missing, unreadable, invalid or valid and exits 0 for each",
        "a valid protocol needs MERGE_OWNER, DEPLOY_OWNER and at least one HUMAN_DECISION_BOUNDARIES item",
        "a FULL_TEST that differs from commands.test is an issue",
        "bee protocol init writes a file that checks valid and never overwrites an existing file",
        "the doctor row never blocks overall_status",
        "protocol_sha256(root) is the one digest helper"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/protocol.rs", "substantive": "parser, state, digest, template, verbs and tests"}
      ],
      "key_links": ["doctor.rs calls protocol_state", "registry_payload.json declares protocol check and protocol init"],
      "prohibitions": ["No gate or refusal of any other verb on protocol state", "No second parser of the file", "No code comments"]
    }
  },
  {
    "id": "wsp-3",
    "feature": "workspace-protocol",
    "title": "Forbid option menus in worker briefs and leaning in advisor briefs",
    "lane": "standard",
    "role": "docs",
    "deps": [],
    "decisions": ["D6", "D9", "D10", "D12"],
    "files": [
      "packages/bee/prompts/worker-cell.md",
      "packages/bee/prompts/advisor.md",
      "packages/bee/LEADER.md",
      "skills/bee-hive/references/gates-and-delegation.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/workspace-protocol/CONTEXT.md",
      "packages/bee/prompts/worker-cell.md",
      "packages/bee/prompts/advisor.md",
      "packages/bee/prompts/reviewer.md",
      "packages/bee/LEADER.md",
      "/mnt/data/Projects/refs/slp/paseo-room/protocol/workspace-protocol.template.md"
    ],
    "affects_skills": ["skills/bee-hive/references/gates-and-delegation.md"],
    "affects_specs": [],
    "action": "worker-cell.md: add one Contract bullet per D9 (never stop to offer the leader a menu of implementation options; choose inside the cell's scope and record it as a departure; a decision outside the scope is one short yes/no question through [PARKED] or [BLOCKED]). Put it where it renders for every shape, not inside a native_bookkeeping block. advisor.md: add, right after the first line, the reviewer.md line 4 sentence per D10. packages/bee/LEADER.md: in the section on starting a session or on dispatch, add the D6 rule in two or three ASD-STE100 sentences: when WORKSPACE_PROTOCOL.md exists at the repo root, read it in full before routing work; copy the boundaries a cell touches into that cell's brief; name each listed boundary the plan touches in the Gate 2 message (under bypass, in the auto-approval reason and the closing report); a boundary adds no new stop — the gate_bypass level decides as for every gate; workers never read the file; `bee protocol check` reports its state. gates-and-delegation.md Delegation contract: one short paragraph — a reviewer or advisor brief carries the artifact, the constraints and the open question, never the leader's preferred answer (prepare warns with purpose_leaning); a worker brief never asks for a menu. Then run the full regen chain `bee dev regen` (render-skill-trees, onboard --repo-root . --apply, release-manifest --write) once from the worktree so the vendored copies match, and commit the regenerated files with this cell. Run the prompt render tests and fix any byte pin that asserts the exact old prompt text by updating the expected text, never by weakening the assertion. No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee prompt && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "the rendered worker-cell prompt forbids an option menu in every shape",
        "the advisor prompt forbids the orchestrator's verdict, leaning or expected findings",
        "the leader doctrine names WORKSPACE_PROTOCOL.md, the excerpt rule, the Gate 2 naming rule with no new stop, and that workers never read it"
      ],
      "artifacts": [
        {"path": "packages/bee/prompts/worker-cell.md", "substantive": "the no-menu contract line"}
      ],
      "key_links": ["gates-and-delegation.md cites purpose_leaning"],
      "prohibitions": ["No rule to stop narrowing review lenses (D12)", "No code comments"]
    }
  },
  {
    "id": "wsp-2",
    "feature": "workspace-protocol",
    "title": "Record prompt and protocol digests at dispatch and add dispatch verify",
    "lane": "standard",
    "role": "code",
    "deps": ["wsp-1"],
    "decisions": ["D7", "D8", "D11"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/brief_lint.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/tests/registry_contracts.rs"
    ],
    "read_first": [
      "docs/history/workspace-protocol/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/protocol.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/brief_lint.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/workflow-state/dispatch.md"],
    "action": "D7: at the record build site in prepare.rs (near the dispatch_id insert, ~line 3239) insert `prompt_sha256` = sha256_hex of the text field the returned payload carries to the worker (`prompt` for Agent/spawn payloads, `stdin` for Bash herding/cli payloads, the codex message field where that is the carrier — compute it from the final payload map, not from `prompt_body`, so the digest equals what the worker gets) and `protocol_sha256` = crate::verbs::protocol::protocol_sha256(root) (null when None). Add both to the envelope beside dispatch_id. Every kind gets them. Keep brief_sha256 and every other key unchanged. D8: add `bee dispatch verify --dispatch-id <id> [--prompt-file <path>] [--json]` routed beside prepare in close.rs: read .bee/logs/dispatch.jsonl newest-first, take the first `source: prepare` line with that dispatch_id; protocol state: same/changed/appeared/removed/absent from recorded vs current protocol_sha256; prompt state: not_checked without --prompt-file, unrecorded when the record has no prompt_sha256, else match/mismatch over the file bytes; unknown id → {ok:false, type:\"refused\", reason:\"dispatch_not_found\", fix}. Declare it in registry_payload.json and add `prompt_sha256`/`protocol_sha256` to the prepare description there. D11: for kind reviewer and advisor with a non-empty --purpose and a --role that does not start with `hat-`, run ONLY the leaning-stem arm (`first_leaning_stem_outside_tagged` in brief_lint.rs, made pub(crate) or wrapped in a thin pub(crate) fn there; never the four-section shape arm, never a copied stem list); on a hit put `purpose_leaning` (the refusal's reason and fix) in the envelope, print it as one warning line in text mode, set `purpose_leaning: true` on the dispatch record, and build the payload exactly as without it; never refuse. Tests: prepare record and envelope carry a 64-hex prompt_sha256 equal to sha256 of payload prompt; the digest equals sha256 of `payload.prompt` for an Agent payload and of `payload.stdin` for a herding Bash payload; a round-trip test saves that field to a file and `dispatch verify --prompt-file` says match; protocol_sha256 null without the file and equal to the file digest with it (read from the control root); verify gives same, then changed after editing the file, removed after deleting it; prompt match and mismatch; verify also gives appeared (no file at prepare, file now) and absent (no file then or now); prompt unrecorded for a record without prompt_sha256 (hand-append one old-shape line in the test log); a non-UTF-8 --prompt-file is mismatch; verify run with a worktree cwd reads the control root's log; unknown id refuses with a fix naming the control root; a reviewer purpose that lint_brief rejects still returns a payload plus purpose_leaning; a clean one-line purpose such as `Plan-step hat wave over docs/history/x/plan.md` has no purpose_leaning key; a leaning purpose under `--role hat-risks` has no purpose_leaning key. Existing prepare tests stay green. No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee prepare && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee dispatch && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts",
    "must_haves": {
      "truths": [
        "every prepare record and envelope carries prompt_sha256 and protocol_sha256",
        "bee dispatch verify reports protocol same/changed/appeared/removed/absent and prompt match/mismatch/not_checked/unrecorded",
        "a leaning reviewer or advisor purpose yields purpose_leaning and never a refusal",
        "the --brief-file leaning refusal is unchanged"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "substantive": "digests, verify, purpose lint and tests"}
      ],
      "key_links": ["prepare calls protocol::protocol_sha256", "purpose lint calls the brief_lint stem arm only"],
      "prohibitions": ["No second hasher", "No second leaning rule list", "No refusal on --purpose", "No code comments"]
    }
  },
  {
    "id": "wsp-4",
    "feature": "workspace-protocol",
    "title": "Document the workspace protocol and dispatch receipts",
    "lane": "standard",
    "role": "docs",
    "deps": ["wsp-1", "wsp-2", "wsp-3"],
    "decisions": ["D1", "D2", "D3", "D4", "D5", "D6", "D7", "D8", "D9", "D10", "D11", "D12"],
    "files": [
      "docs/knowledge/areas/workflow-state/workspace-protocol.md",
      "docs/knowledge/index.md"
    ],
    "read_first": [
      "docs/history/workspace-protocol/CONTEXT.md",
      "docs/knowledge/areas/workflow-state/dispatch.md",
      "docs/knowledge/index.md"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/workflow-state/workspace-protocol.md"],
    "action": "Write one concept in the bee.area frontmatter style of its neighbours (ASD-STE100, short sentences): the root file and its key format, the four states and three required keys, why test commands stay in commands.test, check/init/doctor row, the leader readership rule, the prepare digests and dispatch verify with its states, the warn-only purpose lint and why it cannot refuse, the no-menu and no-leaning prompt lines, and D12. Cite the decision store id (`bee decisions search --text WORKSPACE_PROTOCOL`) as `workspace-protocol D1-D12`. Pointers: verbs/protocol.rs, verbs/drivers/prepare.rs, doctor.rs, the two prompts. Add the concept to docs/knowledge/index.md under workflow-state.",
    "verify": "rg -n 'WORKSPACE_PROTOCOL.md|prompt_sha256|dispatch verify|purpose_leaning' docs/knowledge/areas/workflow-state/workspace-protocol.md && rg -n 'workspace-protocol.md' docs/knowledge/index.md",
    "must_haves": {
      "truths": ["the concept names the file, the verbs, the receipt and the brief rules", "the index lists the concept"],
      "artifacts": [{"path": "docs/knowledge/areas/workflow-state/workspace-protocol.md", "substantive": "the concept"}],
      "key_links": ["the concept cites the decision"],
      "prohibitions": ["No code comments added anywhere"]
    }
  }
]
```

## Test matrix

| Dimension | Probe | Cell | Pass when |
|---|---|---|---|
| Empty/absent | no protocol file | wsp-1, wsp-2 | state missing; protocol_sha256 null; verify absent |
| Malformed input | non-UTF-8 file; paseo-doctrine `- key:` prose | wsp-1 | unreadable; invalid with three issues |
| Foreign format | paseo-pi-team fenced template | wsp-1 | keys and lists parse |
| Overwrite | init on an existing file | wsp-1 | refused, bytes kept |
| Drift | FULL_TEST differs from commands.test | wsp-1 | one issue |
| Unfilled template | paseo `human \| named-integration-owner` values | wsp-1 | placeholder issue, invalid |
| Round trip | saved payload field through verify --prompt-file | wsp-2 | match |
| Change after dispatch | protocol edited or deleted | wsp-2 | changed / removed |
| False fire | leaning purpose on an advisor consult | wsp-2 | payload built, purpose_leaning set |
| Regression | existing prepare, doctor, registry, prompt tests | each code cell | the cell's verify commands green |

## Open Questions

None.
