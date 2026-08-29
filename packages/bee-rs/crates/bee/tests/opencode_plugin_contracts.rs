// OpenCode guard-belt fixtures + the three-belt parity test (plan.md E3,
// cell oc-7).
//
// PROVENANCE / WHY FROM ZERO. `docs/06-runtime-integration.md:143` names a
// two-belt parity test that DOES NOT EXIST in this tree — it died with the
// Node runtime at the R6 cutover (commit 5c62cad0, `packages/bee/hooks/
// test_hook_contracts.mjs`'s Node-runtime half). There is nothing to port
// forward for the OpenCode belt specifically: OpenCode is a NEW third
// runtime (opencode-support D1), so this file is authored from zero, inside
// the cargo suite `commands.test` already runs (never a standalone script).
//
// WHAT THIS FILE PROVES, in two parts:
//
//   1. Fixture tests over the REAL `.opencode/plugins/bee-guard.ts`, run
//      under a real `node` subprocess (Node 24+ strips this file's erasable
//      TypeScript syntax natively — no build step, no ts-node). Each fixture
//      swaps in a STUB `.bee/bin/bee` binary that can deny (exit 2), allow
//      (exit 0), crash (any other nonzero exit), or be absent, and asserts
//      the plugin's two documented failure policies hold
//      (bee-guard.ts:14-32): BLOCKING surfaces throw on deny/crash/missing
//      (fail CLOSED); ADVISORY surfaces never throw, on any stub behavior
//      (fail OPEN, matching `docs/knowledge/patterns/20260714-a-fail-open-
//      host-swallows-fail-closed-throws.md`'s warning about NOT letting the
//      two mix). Node absence is a named, non-fatal skip — see
//      `node_or_skip`.
//
//   2. A verdict-shape parity test (`three_belt_parity_every_blocking_rule_
//      hits_helper_claude_codex_and_opencode` — name kept from its original
//      per-rule form, since it is cited by name in docs/knowledge/areas/
//      hook-runtime/catalog-projections-and-activation.md and docs/history/
//      opencode-support/discovery.md) that DERIVES the guard-rule inventory
//      from the catalog of record — the two checked-in, generated hook
//      manifests (`packages/bee/hooks/claude-hooks.json`, `packages/bee/
//      hooks/hooks.json`; both are `hook_manifests.rs`'s CATALOG rendered to
//      disk, kept honest by `hook_manifests_match_disk` in that module) —
//      and never a hand-authored list, per
//      `docs/knowledge/patterns/20260722-a-coverage-gate-derives-ground-
//      truth-it-never-compares-two-hand-lists.md`. The row set is (rule,
//      verdict SHAPE) pairs, not just rules: `emittable_shapes` derives
//      which of deny/repair/ask each BLOCKING rule (today `write-guard` and
//      `model-guard`) can actually put on the wire by scanning that rule's
//      own emit-path source, so a whole SHAPE — not only a whole rule — now
//      fails this suite by name if it goes missing on a belt. For every
//      such (rule, shape) pair, the test asserts FOUR independent signals
//      exist, failing by name (rule + shape + belt) if any is missing:
//        - HELPER level  — `bee hook <rule>` itself emits this shape for its
//          known-triggering payload (the shared FIRST belt every runtime's
//          translation layer calls into — plan.md's Approach section:
//          "helpers stay the FIRST belt on every runtime").
//        - CLAUDE belt   — this rule's own embedded fixture suite
//          (`hook_contracts.rs` for deny; `write_guard/tests.rs` or
//          `model_guard.rs`'s own test module for repair/ask) contains a
//          fixture using one of the claude-shaped tool names the catalog
//          says reaches this rule, proving this shape.
//        - CODEX belt    — the rule is wired under `PreToolUse` in the
//          file-shipped codex projection itself (no separate Codex
//          translation layer exists to fixture-test beyond that wiring, for
//          ANY shape — Codex's PreToolUse command execs `bee hook <rule>`
//          directly, byte for byte the same mechanism the helper-level
//          check already exercised for this exact shape; see bee-guard.ts's
//          own header comment on the matcher-only Codex difference).
//        - OPENCODE belt — `bee-guard.ts`'s `mapToolCall` actually routes at
//          least one real OpenCode tool to this rule (derived by parsing the
//          plugin's own switch statement, never hand-listed) AND
//          `runBlockingHook`'s own source still implements this shape (a
//          literal marker cross-check) — the actual deny/repair/ask/
//          unparseable PROOF for every such row lives in part 1 above; this
//          is the routing+implementation cross-check that keeps part 1 from
//          silently going vacuous if a row's `hook:` literal changes or a
//          shape's handling is deleted from the plugin. The fourth shape,
//          an UNPARSEABLE exit-0 verdict, is not producer-derived (bee never
//          emits invalid JSON of its own) and is checked once, globally, at
//          the end of the test — see the doc comment above
//          `emittable_shapes`.
//
// The design and any environment-skip behavior are recorded in
// `docs/history/opencode-support/discovery.md`.

use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

// ─── repo layout ────────────────────────────────────────────────────────────

/// `CARGO_MANIFEST_DIR` is `packages/bee-rs/crates/bee`; four `parent()`s
/// (crates, bee-rs, packages, repo root) reach the checkout root, the same
/// one `.opencode/plugins/bee-guard.ts` and the checked-in hook manifests
/// live under.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("CARGO_MANIFEST_DIR too shallow: {}", env!("CARGO_MANIFEST_DIR")))
        .to_path_buf()
}

fn bee_bin() -> PathBuf {
    assert_cmd::cargo::cargo_bin("bee")
}

/// The real plugin source, embedded at compile time — the same file
/// `docs/history/opencode-support/discovery.md` documents and this test
/// exercises live via `node`. Never copied or re-derived by hand.
const PLUGIN_SOURCE: &str = include_str!("../../../../../.opencode/plugins/bee-guard.ts");

/// `hook_contracts.rs`'s own source, embedded so the parity test can look
/// for the CLAUDE belt's deny fixtures by what they actually assert, rather
/// than trusting a hand-maintained pointer to a test name.
const HOOK_CONTRACTS_SOURCE: &str = include_str!("hook_contracts.rs");

/// `discovery.md`'s own text, embedded so the parity test can confirm any
/// ADVISORY hook the OpenCode plugin does not wire is a NAMED gap (already
/// written up by oc-6), never a silent omission.
const DISCOVERY_DOC: &str = include_str!("../../../../../docs/history/opencode-support/discovery.md");

// ─── verdict shapes: derived from the guard's own emit paths ──────────────
//
// A blocking rule's decision reaches its belt as one of several distinct
// WIRE SHAPES, not just "deny" vs "allow" — D6 (`.opencode/plugins/
// bee-guard.ts`'s own header comment, docs/history/opencode-support/
// discovery.md) names three the OpenCode belt must parse specially: an
// exit-0 verdict carrying `hookSpecificOutput.updatedInput` (a repair), one
// carrying `permissionDecision: "ask"` (bee's own "ask, never allow" —
// write_guard/main.rs:389-394), and exit-0 stdout that is non-empty but
// fails to parse at all (undecidable, and fail-closed on the blocking
// path). The parity test below used to prove only that a RULE reaches
// every belt — never that every SHAPE that rule can put on the wire does. A
// belt with a narrower return surface than another can honor the one shape
// it has a place for (deny) and let the rest go inert while a rule-only
// parity test stays green — the defect oc-8 actually found on the OpenCode
// belt (F2) before D6 closed it there. Widening the row set to (rule,
// shape) pairs closes the same gap for every belt, not only the one that
// already had it.
//
// Three of the four shapes are PRODUCER-side and are derived by scanning
// each blocking rule's own emit-path source for the literal markers that
// put the shape on the wire — never a hand-authored "these are the four
// shapes" list, so a rule that starts or stops emitting a shape changes the
// derived set on its own, per docs/knowledge/patterns/20260722-a-coverage-
// gate-derives-ground-truth-it-never-compares-two-hand-lists.md. The
// fourth, UNPARSEABLE, is not something bee itself ever emits
// (`jsjson::stringify` never produces invalid JSON) — it is a BELT-side
// parse-robustness requirement instead, asserted once, globally, wherever a
// belt in this repo actually parses bee's own stdout (see the dedicated
// block at the end of the parity test).

const DENY_SHAPE: &str = "deny (exit code 2)";
const REPAIR_SHAPE: &str = "exit-0 updatedInput repair";
const ASK_SHAPE: &str = "exit-0 permissionDecision ask";

/// `write_guard/main.rs`'s and `model_guard.rs`'s own source — the guard's
/// EMIT PATHS this file derives verdict shapes from. `write_guard/tests.rs`
/// doubles as write-guard's own CLAUDE-belt ask/repair-fixture proof source
/// below (the AskUserQuestion auto-fix fixtures live there, not in
/// `main.rs`, which holds only the emit path); `model_guard.rs`'s own
/// embedded `#[cfg(test)] mod tests` serves the same purpose for
/// model-guard's dispatch-repair fixtures, since this whole file (prod code
/// and tests together) is embedded as one string.
const WRITE_GUARD_MAIN_SOURCE: &str = include_str!("../src/hooks/write_guard/main.rs");
const MODEL_GUARD_SOURCE: &str = include_str!("../src/hooks/model_guard.rs");
const WRITE_GUARD_TESTS_SOURCE: &str = include_str!("../src/hooks/write_guard/tests.rs");

/// The verdict shapes rule `hook`'s own emit-path source can actually put
/// on the wire, derived by scanning that source for the literal markers
/// each shape requires — never a hand-authored per-rule list. Today
/// write-guard's AskUserQuestion auto-fix emits all three (its own comment
/// at write_guard/main.rs:389-394 is explicit both `updatedInput` and
/// `permissionDecision: "ask"` ride the SAME emission); model-guard's
/// dispatch-label repair carries `updatedInput` alone — "No
/// `permissionDecision` rides along" (model_guard.rs:137-141) — so
/// ASK_SHAPE is never derived for it. A future rule (or a future emission
/// on either existing rule) that adds or drops a marker changes this
/// function's output on its own, with no list to update by hand.
fn emittable_shapes(hook: &str) -> Vec<&'static str> {
    let source = match hook {
        "write-guard" => WRITE_GUARD_MAIN_SOURCE,
        "model-guard" => MODEL_GUARD_SOURCE,
        other => panic!(
            "no known emit-path source for BLOCKING rule \"{other}\" — add one to \
             `emittable_shapes` before trusting verdict-shape derivation for it"
        ),
    };
    let mut shapes = Vec::new();
    if source.contains("emit.code = 2") || source.contains("Ok((2,") {
        shapes.push(DENY_SHAPE);
    }
    if source.contains("\"updatedInput\".into()") {
        shapes.push(REPAIR_SHAPE);
    }
    if source.contains("Value::String(\"ask\".into())") {
        shapes.push(ASK_SHAPE);
    }
    assert!(
        !shapes.is_empty(),
        "BLOCKING rule \"{hook}\": emittable_shapes derivation found ZERO verdict shapes in its \
         own emit-path source — the markers likely drifted from the source"
    );
    shapes
}

// ─── catalog of record: derive the guard-rule inventory ────────────────────
//
// The catalog of record is the two checked-in, GENERATED hook manifests
// `packages/bee/hooks/claude-hooks.json` (Runtime::Claude) and
// `packages/bee/hooks/hooks.json` (Runtime::Codex) — both rendered from
// `hook_manifests.rs`'s `CATALOG` and drift-checked byte-for-byte against it
// by `hook_manifests_match_disk`. Reading them here (rather than reaching
// into the `devtools` module's private types from this black-box test) is
// itself the derivation: it can never drift from the catalog without that
// existing drift test going red first.

#[derive(Debug, Clone)]
struct CatalogRow {
    hook: String,
    /// True iff this hook is wired under a `PreToolUse` event WITH A MATCHER
    /// in EITHER projection. Both halves of that predicate are load-bearing,
    /// and both are DERIVED from the manifests' own data — never from a
    /// hand-authored name list, per `docs/knowledge/patterns/20260722-a-
    /// coverage-gate-derives-ground-truth-it-never-compares-two-hand-
    /// lists.md`, which this file already leans on for its rule inventory.
    ///
    /// `PreToolUse` is the only event a `tool.execute.before`-style belt can
    /// block on; every other event this catalog uses (`SessionStart`,
    /// `UserPromptSubmit`, `PostToolUse`, `SubagentStart`, `SubagentStop`,
    /// `PreCompact`, `Stop`) is advisory-only by construction.
    ///
    /// The MATCHER half is the codebase's own stated rule about this exact
    /// data, quoted from `devtools/hook_manifests.rs:166-167`:
    ///
    ///   // No matcher = every tool. Passive measurement only; this hook can
    ///   // never deny or block.
    ///
    /// So a matcher-less `PreToolUse` row is an OBSERVER wired on every
    /// tool, not a gate — registered-under-`PreToolUse` and can-block are
    /// two different facts, and only the second one is what the three-belt
    /// parity test means by BLOCKING. `activity` is today's instance
    /// (`hook_manifests.rs:104-111`: matcher-less on every event it appears
    /// on, by construction, and `hook_manifests.rs:638-641` actively asserts
    /// it stays matcher-less), but nothing here NAMES it: the next
    /// matcher-less `PreToolUse` hook is classified by the same derived
    /// predicate, with no list to remember to update.
    ///
    /// Both an ABSENT `matcher` key and an explicit JSON `null` arrive as
    /// `None` through `events_map`, so either spelling classifies the same
    /// way. In the real artifact the key is ABSENT — see the activity group
    /// at `packages/bee/hooks/claude-hooks.json:56-64`, which carries only
    /// `"hooks"`.
    blocking: bool,
    /// The `|`-split matcher tokens (e.g. `["Edit","Write",...]`) the CLAUDE
    /// projection uses to reach this hook under `PreToolUse`, empty when
    /// this hook is not wired under claude's `PreToolUse` at all OR is wired
    /// there matcher-less (the observer case above — an empty matcher list
    /// is exactly what makes such a row non-`blocking`).
    claude_pretooluse_matchers: Vec<String>,
    /// Same, for the CODEX projection.
    codex_pretooluse_matchers: Vec<String>,
}

impl CatalogRow {
    fn new(hook: &str) -> Self {
        CatalogRow {
            hook: hook.to_string(),
            blocking: false,
            claude_pretooluse_matchers: Vec::new(),
            codex_pretooluse_matchers: Vec::new(),
        }
    }
}

/// Pulls the hook NAME out of a rendered command string. Every rendered
/// command execs `<bin> hook <name>` at least once (the project-then-
/// main-worktree fallback chain repeats the same name twice) — mirrors the
/// same extraction `hook_manifests.rs`'s own
/// `projections_differ_only_where_approved` test performs on these exact
/// strings.
fn extract_hook_name(command: &str) -> Option<String> {
    let idx = command.find(" hook ")?;
    let rest = &command[idx + " hook ".len()..];
    let end = rest.find(|c: char| c == ';' || c.is_whitespace())?;
    Some(rest[..end].to_string())
}

/// `event name -> [(matcher, hook name)]` for one rendered projection JSON.
fn events_map(v: &Value) -> BTreeMap<String, Vec<(Option<String>, String)>> {
    let hooks_obj = v["hooks"].as_object().expect("projection JSON has no top-level \"hooks\" object");
    let mut map = BTreeMap::new();
    for (event, groups) in hooks_obj {
        let mut rows = Vec::new();
        for g in groups.as_array().unwrap_or_else(|| panic!("{event}: groups must be an array")) {
            let matcher = g.get("matcher").and_then(|m| m.as_str()).map(str::to_string);
            for h in g["hooks"].as_array().unwrap_or_else(|| panic!("{event}: group hooks must be an array")) {
                let command = h["command"].as_str().unwrap_or_else(|| panic!("{event}: hook command must be a string"));
                let name = extract_hook_name(command)
                    .unwrap_or_else(|| panic!("{event}: could not extract a hook name from command: {command}"));
                rows.push((matcher.clone(), name));
            }
        }
        map.insert(event.clone(), rows);
    }
    map
}

fn read_projection(rel: &str) -> Value {
    let path = repo_root().join(rel);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: not valid JSON: {e}", path.display()))
}

/// The derived catalog: hook name -> row. Built once per call from the two
/// checked-in projections — no hand-authored hook list anywhere in this
/// function.
fn derive_catalog() -> BTreeMap<String, CatalogRow> {
    let claude = events_map(&read_projection("packages/bee/hooks/claude-hooks.json"));
    let codex = events_map(&read_projection("packages/bee/hooks/hooks.json"));
    let mut rows: BTreeMap<String, CatalogRow> = BTreeMap::new();

    for (event, list) in &claude {
        for (matcher, name) in list {
            let row = rows.entry(name.clone()).or_insert_with(|| CatalogRow::new(name));
            if event == "PreToolUse" {
                // A matcher-less PreToolUse row is a passive OBSERVER wired
                // on every tool, never a gate — see `CatalogRow::blocking`
                // for the manifest comment that states it. `blocking` is set
                // INSIDE this arm so the predicate stays derived from the
                // manifest's own data, with no rule name written down here.
                if let Some(m) = matcher {
                    row.blocking = true;
                    row.claude_pretooluse_matchers = m.split('|').map(str::to_string).collect();
                }
            }
        }
    }
    for (event, list) in &codex {
        for (matcher, name) in list {
            let row = rows.entry(name.clone()).or_insert_with(|| CatalogRow::new(name));
            if event == "PreToolUse" {
                // Same derived predicate as the claude loop above.
                if let Some(m) = matcher {
                    row.blocking = true;
                    row.codex_pretooluse_matchers = m.split('|').map(str::to_string).collect();
                }
            }
        }
    }
    rows
}

// ─── the opencode plugin's own routing, derived from its source ───────────

/// Parses `bee-guard.ts`'s `mapToolCall` switch statement for every
/// `(OpenCode tool, bee hook)` pair it actually routes — the same mechanism
/// section (1)'s fixtures exercise, reused here so the parity test's
/// "opencode belt" claim can never drift from what the fixtures cover.
fn opencode_tool_hook_pairs() -> Vec<(String, String)> {
    let fn_start = PLUGIN_SOURCE
        .find("function mapToolCall")
        .expect("bee-guard.ts: mapToolCall not found — has the routing function been renamed?");
    let body = &PLUGIN_SOURCE[fn_start..];
    let switch_start = body
        .find("switch (tool)")
        .expect("bee-guard.ts: mapToolCall no longer switches on `tool` — routing derivation needs an update");
    let switch_body = &body[switch_start..];

    let mut pairs = Vec::new();
    for seg in switch_body.split("case \"").skip(1) {
        let tool_end = seg.find('"').expect("bee-guard.ts: unterminated `case \"...\"` tool literal");
        let tool = seg[..tool_end].to_string();
        if let Some(h) = seg.find("hook: \"") {
            let rest = &seg[h + "hook: \"".len()..];
            let hend = rest.find('"').expect("bee-guard.ts: unterminated `hook: \"...\"` literal");
            pairs.push((tool, rest[..hend].to_string()));
        }
        // A `default: return null` arm carries no `hook: "..."` literal, so
        // it is silently skipped rather than mis-parsed as a row.
    }
    assert!(
        pairs.len() >= 9,
        "bee-guard.ts: mapToolCall derivation found only {} routed tool cases — \
         expected at least 9 (write/edit/bash/apply_patch/read/grep/glob/question/task); \
         either the switch statement changed shape or this parser broke",
        pairs.len()
    );
    pairs
}

/// Parses every `runAdvisoryHook(directory, "<name>", ...)` call site —
/// the ADVISORY hooks the plugin actually wires (session-init,
/// prompt-context, state-sync, session-close, tools-logger today).
/// `codex-subagent-audit` and `chain-nudge` deliberately do not appear here;
/// see `advisory_gaps_the_plugin_does_not_wire_are_named_not_silent` below.
fn opencode_advisory_hooks() -> BTreeSet<String> {
    const MARKER: &str = "runAdvisoryHook(directory, \"";
    let mut set = BTreeSet::new();
    let mut idx = 0usize;
    while let Some(pos) = PLUGIN_SOURCE[idx..].find(MARKER) {
        let start = idx + pos + MARKER.len();
        let rest = &PLUGIN_SOURCE[start..];
        let end = rest.find('"').expect("bee-guard.ts: unterminated runAdvisoryHook name literal");
        set.insert(rest[..end].to_string());
        idx = start + end;
    }
    assert!(!set.is_empty(), "bee-guard.ts: found zero runAdvisoryHook call sites — advisory derivation broke");
    set
}

// ─── node availability (named, non-fatal skip) ─────────────────────────────
//
// "Node is absent" is not the only way this belt is unrunnable — a `node`
// that IS on PATH but too old to strip TypeScript natively (verified live:
// an ambient PATH with system node v18 ahead of nvm's v24 makes every
// harness spawn die with `ERR_UNKNOWN_FILE_EXTENSION` on the real
// `bee-guard.ts`, not a clean "not found") is functionally the same gap.
// This probes the REAL capability the harness needs — running a one-line
// `.ts` file directly, exactly how it loads the real plugin — rather than
// trusting a version number, so any node that cannot do that degrades to a
// named skip instead of a panic.

fn node_typescript_probe() -> Result<(), String> {
    let version_out = Command::new("node").arg("--version").output();
    let version = match version_out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => return Err("`node` not found on PATH".to_string()),
    };
    let dir = tempfile::tempdir().map_err(|e| format!("could not create a tempdir for the node/TS probe: {e}"))?;
    let probe = dir.path().join("probe.ts");
    std::fs::write(&probe, "const x: number = 1\nconsole.log(x)\n").map_err(|e| e.to_string())?;
    let out = Command::new("node")
        .arg(&probe)
        .output()
        .map_err(|e| format!("failed to spawn `node {}`: {e}", probe.display()))?;
    if out.status.success() && String::from_utf8_lossy(&out.stdout).trim() == "1" {
        Ok(())
    } else {
        Err(format!(
            "`node` ({version}) cannot run a minimal .ts file directly (needed to load \
             bee-guard.ts exactly the way OpenCode itself loads it — no build step) — stderr: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// The one opt-out surface for every environment-gated capability this
/// suite needs to prove REAL OpenCode-belt coverage (a TS-capable `node`
/// here; the installed `opencode` binary itself for the tool-registry
/// derivation further down). Unset — the default — an absent capability is
/// a FAIL, never a silent skip: a shell whose `node` predates v24 (system
/// node ahead of nvm on PATH, no override) used to yield 4 green tests and
/// zero enforcement coverage (F1) — that is strictly worse than a red
/// suite, because a green suite stops getting looked at. Set, the same
/// absence degrades to a named, visible SKIP for an environment that
/// deliberately carries neither: this repo's own CI is exactly that
/// environment today — `.github/workflows/ci.yml`'s R6-cutover comment
/// confirms the Node matrix was deleted outright, so `ubuntu-latest`'s
/// ambient `node` (if any) is not guaranteed TS-capable — recorded as a
/// real, intended behavior change for that pipeline in discovery.md's oc-9
/// section, not silently assumed away here.
const ALLOW_SKIP_ENV: &str = "BEE_OPENCODE_SUITE_ALLOW_SKIP";

fn env_allows_skip() -> bool {
    std::env::var_os(ALLOW_SKIP_ENV).is_some()
}

/// Every test that drives the real plugin under `node` opens with this. The
/// reason always reaches stderr first — cargo test only captures stdout, so
/// this line reaches the default (captured) output on a PASS *and* a FAIL
/// alike, never only on failure. What happens next depends on
/// `BEE_OPENCODE_SUITE_ALLOW_SKIP` (unset by default): unset, an absent or
/// TS-incapable `node` is a hard FAIL — a missing capability must never
/// report this test green with zero enforcement actually exercised (F1);
/// set, it is a named SKIP, matching `hook_contracts.rs`'s own `ran_native`
/// skip convention.
macro_rules! node_or_skip {
    ($test_name:expr) => {
        if let Err(reason) = node_typescript_probe() {
            let allow = env_allows_skip();
            eprintln!("{} (env-limited: {reason}) — {}", if allow { "SKIP" } else { "FAIL" }, $test_name);
            if allow {
                return;
            }
            panic!(
                "{}: a `node` capable of stripping TypeScript natively is required to prove real \
                 OpenCode enforcement coverage ({reason}) — refusing to report this test green \
                 with zero enforcement actually exercised. Set {ALLOW_SKIP_ENV}=1 to explicitly \
                 accept a degraded, unproven run in an environment that deliberately has no such \
                 node.",
                $test_name
            );
        }
    };
}

// ─── the node harness: drives the real plugin's exported hooks ────────────
//
// A tiny, generic driver (never checked in — written fresh into a tempdir
// per test run, exactly the way `hook_contracts.rs`'s `fixture()` writes its
// `.bee/onboarding.json` marker) that dynamically imports the REAL
// `bee-guard.ts` file by path, calls its default export to get the `Hooks`
// object, invokes exactly one named surface with a JSON payload read from
// stdin, and reports whether the call threw. Node 24's native TypeScript
// type-stripping runs the `.ts` file directly — no separate build step, no
// ts-node dependency.
const HARNESS_JS: &str = r#"
import { pathToFileURL } from "node:url";

const [, , pluginPath, directory, surface] = process.argv;

function readStdin() {
  return new Promise((resolve, reject) => {
    let data = "";
    process.stdin.setEncoding("utf8");
    process.stdin.on("data", (chunk) => { data += chunk; });
    process.stdin.on("end", () => resolve(data));
    process.stdin.on("error", reject);
  });
}

const payload = JSON.parse(await readStdin());
const mod = await import(pathToFileURL(pluginPath).href);
const hooks = await mod.default({ directory, worktree: directory, client: {}, $: {}, project: {}, app: {} });

const fn = hooks[surface];
if (typeof fn !== "function") {
  console.log(JSON.stringify({ threw: false, message: null, output: null, note: `surface ${surface} not registered` }));
  process.exit(0);
}

let result;
try {
  if (surface === "chat.message") {
    const input = { sessionID: payload.sessionID ?? "sess1", messageID: payload.messageID ?? "msg1" };
    const output = { parts: [], message: { id: payload.messageID ?? "msg1" } };
    await fn(input, output);
    result = { threw: false, message: null, output };
  } else if (surface === "event") {
    const input = { event: payload.event };
    await fn(input);
    result = { threw: false, message: null, output: null };
  } else {
    const input = { tool: payload.tool, sessionID: payload.sessionID ?? "sess1", callID: payload.callID ?? "call1", args: payload.args };
    const output = { args: payload.args ?? {} };
    await fn(input, output);
    result = { threw: false, message: null, output };
  }
} catch (err) {
  result = { threw: true, message: String(err && err.message ? err.message : err), output: null };
}
console.log(JSON.stringify(result));
"#;

struct HarnessResult {
    threw: bool,
    message: Option<String>,
    output: Option<Value>,
}

fn write_harness(dir: &Path) -> PathBuf {
    let path = dir.join("harness.mjs");
    std::fs::write(&path, HARNESS_JS).expect("failed to write the node harness");
    path
}

fn run_harness(harness: &Path, plugin: &Path, directory: &Path, surface: &str, payload: &Value) -> HarnessResult {
    let mut child = Command::new("node")
        .arg(harness)
        .arg(plugin)
        .arg(directory)
        .arg(surface)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to launch `node {}`: {e}", harness.display()));
    child
        .stdin
        .take()
        .unwrap()
        .write_all(payload.to_string().as_bytes())
        .expect("failed to write payload to the harness's stdin");
    let out: Output = child.wait_with_output().expect("node harness never exited");
    assert!(
        out.status.success(),
        "node harness (surface={surface}) exited non-zero: stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let last_line = stdout.lines().last().unwrap_or("").trim();
    let v: Value = serde_json::from_str(last_line).unwrap_or_else(|e| {
        panic!(
            "node harness (surface={surface}) did not print JSON on its last stdout line: {e}; \
             stdout={stdout} stderr={}",
            String::from_utf8_lossy(&out.stderr)
        )
    });
    HarnessResult {
        threw: v["threw"].as_bool().unwrap_or(false),
        message: v["message"].as_str().map(str::to_string),
        output: v.get("output").cloned().filter(|o| !o.is_null()),
    }
}

// ─── stub bee binaries: deny / allow / crash / absent ──────────────────────

enum StubBehavior {
    Deny(String),
    Allow,
    Crash,
    Missing,
    /// D6 (oc-8): an exit-0 verdict carrying a `hookSpecificOutput.
    /// updatedInput` repair — write-guard's `AskUserQuestion` header fix and
    /// model-guard's dispatch-label/`subagent_type` fix both ride this exact
    /// shape on the wire. The repaired field name is deliberately generic
    /// (`repairedField`) since the real repair target differs per rule; this
    /// fixture only proves `runBlockingHook` applies whatever `updatedInput`
    /// carries onto `output.args`, not a specific rule's own repair
    /// semantics.
    Repair,
    /// D6: an exit-0 verdict carrying `permissionDecision: "ask"` — bee's
    /// own "ask, never allow" verdict (write_guard/main.rs:389-394).
    Ask(String),
    /// D6: exit-0 stdout that is non-empty but not valid JSON — undecidable,
    /// and undecidable must stay fail-closed on the BLOCKING path.
    UnparseableVerdict,
}

/// Writes (or, for `Missing`, deliberately does NOT write) a stub
/// `.bee/bin/bee` under `root`. `root` must be OUTSIDE this checkout (a
/// fresh `tempfile::tempdir()` already is, since it defaults to the system
/// temp root) so the plugin's git-common-dir fallback can never accidentally
/// resolve to this repo's real binary and mask the scenario under test.
///
/// F3: every non-`Missing` stub now CAPTURES the exact stdin bee received
/// (to `last_stdin.json` next to the stub itself, via `$(dirname "$0")` —
/// stable regardless of which fixture tempdir this run uses) before
/// producing its verdict. The old `cat >/dev/null` swallowed stdin entirely,
/// so a field-name mistranslation in `mapToolCall` (a renamed, dropped, or
/// mis-shaped field) failed OPEN and stayed green — see
/// `read_captured_stdin` and
/// docs/knowledge/patterns/20260710-a-boundary-that-lists-field-names-will-
/// leak.md.
#[cfg(unix)]
fn write_stub_bee(root: &Path, behavior: &StubBehavior) {
    use std::os::unix::fs::PermissionsExt;
    let StubBehavior::Missing = behavior else {
        let bin_dir = root.join(".bee").join("bin");
        std::fs::create_dir_all(&bin_dir).expect("failed to create .bee/bin");
        let capture = "cat > \"$(dirname \"$0\")/last_stdin.json\"";
        let script = match behavior {
            StubBehavior::Deny(reason) => format!("#!/bin/sh\n{capture}\necho \"{reason}\" >&2\nexit 2\n"),
            StubBehavior::Allow => format!("#!/bin/sh\n{capture}\nexit 0\n"),
            StubBehavior::Crash => format!("#!/bin/sh\n{capture}\necho \"stub crash\" >&2\nexit 17\n"),
            StubBehavior::Repair => {
                let stdout =
                    json!({"hookSpecificOutput": {"updatedInput": {"repairedField": "repaired-value"}}}).to_string();
                format!("#!/bin/sh\n{capture}\nprintf '%s' '{stdout}'\nexit 0\n")
            }
            StubBehavior::Ask(reason) => {
                let stdout =
                    json!({"hookSpecificOutput": {"permissionDecision": "ask", "permissionDecisionReason": reason}})
                        .to_string();
                format!("#!/bin/sh\n{capture}\nprintf '%s' '{stdout}'\nexit 0\n")
            }
            StubBehavior::UnparseableVerdict => format!("#!/bin/sh\n{capture}\nprintf '%s' 'not-json{{{{{{'\nexit 0\n"),
            StubBehavior::Missing => unreachable!(),
        };
        let path = bin_dir.join("bee");
        std::fs::write(&path, script).expect("failed to write the stub bee binary");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .expect("failed to make the stub bee binary executable");
        return;
    };
}

/// Reads back the payload the stub `bee` binary actually received on stdin
/// (see `write_stub_bee`'s capture line) — the ground truth F3's assertions
/// compare against, never the `output.args` OpenCode sees (which is on the
/// OTHER side of the translation this test exists to check).
fn read_captured_stdin(root: &Path) -> Value {
    let path = root.join(".bee").join("bin").join("last_stdin.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("stub bee never captured stdin at {}: {e}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("captured stdin at {} was not valid JSON: {e}\ntext={text}", path.display()))
}

/// The REAL `output.args` shape a live OpenCode session sends per tool
/// (field names verified live against discovery.md's field-shape tables:
/// oc-2's write/edit/bash table, oc-6's glob/grep/question/task table, and
/// oc-3's apply_patch `patchText` finding), paired with the EXACT bee-shaped
/// `tool_input` object `mapToolCall`'s translation is supposed to produce,
/// and the exact `tool_name` literal it emits. Hand-authored domain
/// knowledge, the same way `helper_deny_payload` below already hand-authors
/// its own known-denying SHAPE per rule — deriving the expected translation
/// from the same source under test would prove nothing (F3): a field this
/// table gets wrong is a bug in THIS test, a field `mapToolCall` gets wrong
/// is the defect this test exists to catch, and only a fixed, independent
/// expectation can tell the two apart.
fn opencode_call_fixture(tool: &str) -> (Value, Value, &'static str) {
    match tool {
        "write" => (
            json!({"filePath": "/tmp/oc-fixture/target.txt", "content": "hello"}),
            json!({"file_path": "/tmp/oc-fixture/target.txt", "content": "hello"}),
            "Write",
        ),
        "edit" => (
            json!({"filePath": "/tmp/oc-fixture/target.txt", "oldString": "a", "newString": "b"}),
            json!({"file_path": "/tmp/oc-fixture/target.txt", "old_string": "a", "new_string": "b"}),
            "Edit",
        ),
        "bash" => (json!({"command": "ls -la /tmp"}), json!({"command": "ls -la /tmp"}), "Bash"),
        "apply_patch" => (
            json!({"patchText": "*** Begin Patch\n*** Add File: x.txt\n+hi\n*** End Patch"}),
            json!({"patch": "*** Begin Patch\n*** Add File: x.txt\n+hi\n*** End Patch"}),
            "apply_patch",
        ),
        "read" => (
            json!({"filePath": "/tmp/oc-fixture/target.txt", "offset": 5, "limit": 100}),
            json!({"file_path": "/tmp/oc-fixture/target.txt", "offset": 5, "limit": 100}),
            "Read",
        ),
        "grep" => (
            json!({"path": "/tmp/oc-fixture", "pattern": "needle", "include": "*.rs"}),
            json!({"path": "/tmp/oc-fixture", "pattern": "needle", "include": "*.rs"}),
            "Grep",
        ),
        "glob" => (
            json!({"path": "/tmp/oc-fixture", "pattern": "*.rs"}),
            json!({"path": "/tmp/oc-fixture", "pattern": "*.rs"}),
            "Glob",
        ),
        "question" => {
            let q = json!({
                "questions": [{
                    "question": "Pick one",
                    "header": "Choice",
                    "options": [{"label": "A", "description": "desc a"}],
                    "multiple": false,
                }]
            });
            (q.clone(), q, "AskUserQuestion")
        }
        "task" => {
            let t = json!({
                "description": "probe dispatch",
                "prompt": "[bee-tier: generation] probe",
                "subagent_type": "bee-build",
            });
            (t.clone(), t, "Task")
        }
        "lsp" => (
            // operation/line/character are real args OpenCode's lsp tool
            // sends but mapToolCall does not forward (bee-guard.ts's "lsp"
            // case comment) — kept here so this row also proves the
            // translation drops them rather than leaking extra fields into
            // tool_input.
            json!({"filePath": "/tmp/oc-fixture/target.txt", "operation": "hover", "line": 10, "character": 4}),
            json!({"file_path": "/tmp/oc-fixture/target.txt"}),
            "Read",
        ),
        "list" => (
            json!({"path": "/tmp/oc-fixture"}),
            json!({"path": "/tmp/oc-fixture"}),
            "Glob",
        ),
        other => panic!(
            "no payload fixture defined for OpenCode tool \"{other}\" — mapToolCall routes it but \
             this test's field-shape table (opencode_call_fixture) does not cover it yet; add a row \
             before trusting payload coverage for this tool (F3 / docs/knowledge/patterns/20260710-a-\
             boundary-that-lists-field-names-will-leak.md)"
        ),
    }
}

/// The full payload `bee-guard.ts`'s `runBlockingHook` sends on stdin for a
/// `tool.execute.before` call — `hook_event_name`/`session_id`/`cwd` are the
/// same for every mapped row; `tool_name`/`tool_input` are per-row (see
/// `opencode_call_fixture`).
fn expected_opencode_payload(session_id: &str, cwd: &Path, tool_name: &str, tool_input: &Value) -> Value {
    json!({
        "hook_event_name": "PreToolUse",
        "session_id": session_id,
        "cwd": cwd.to_string_lossy(),
        "tool_name": tool_name,
        "tool_input": tool_input,
    })
}

// ═════════════════════════════════════════════════════════════════════════
// PART 1 — fixture tests: deny / allow / crash / missing, per BLOCKING
// mapped row; never-throws, per ADVISORY surface.
// ═════════════════════════════════════════════════════════════════════════

#[cfg(unix)]
#[test]
fn every_blocking_mapped_row_denies_allows_crashes_and_reports_a_missing_binary() {
    node_or_skip!("every_blocking_mapped_row_denies_allows_crashes_and_reports_a_missing_binary");

    let harness_dir = tempfile::tempdir().expect("tempdir for the harness script");
    let harness = write_harness(harness_dir.path());
    let plugin = repo_root().join(".opencode/plugins/bee-guard.ts");

    let pairs = opencode_tool_hook_pairs();
    let mut failures: Vec<String> = Vec::new();

    for (tool, hook) in &pairs {
        let (args, expected_tool_input, expected_tool_name) = opencode_call_fixture(tool);
        let session_id = format!("sess-{tool}");
        let call_payload = json!({"tool": tool, "sessionID": session_id, "args": args});

        // (a) deny — a distinct, per-row reason must reach the thrown Error.
        {
            let fx = tempfile::tempdir().expect("tempdir");
            let reason = format!("stub-deny for {hook} via {tool}");
            write_stub_bee(fx.path(), &StubBehavior::Deny(reason.clone()));
            let r = run_harness(&harness, &plugin, fx.path(), "tool.execute.before", &call_payload);
            if !(r.threw && r.message.as_deref().is_some_and(|m| m.contains(&reason))) {
                failures.push(format!(
                    "{tool} -> {hook}: DENY did not throw the stub's reason (threw={}, message={:?})",
                    r.threw, r.message
                ));
            }
        }

        // (b) allow — no throw, output.args passes through unchanged, AND
        // (F3) the EXACT payload bee received matches the translated
        // field-name shape — never the old generic probe body, which would
        // stay green even if mapToolCall renamed, dropped, or mis-shaped a
        // field, tool_name, cwd, or session_id.
        {
            let fx = tempfile::tempdir().expect("tempdir");
            write_stub_bee(fx.path(), &StubBehavior::Allow);
            let r = run_harness(&harness, &plugin, fx.path(), "tool.execute.before", &call_payload);
            let args_after = r.output.as_ref().and_then(|o| o.get("args")).cloned();
            if r.threw || args_after.as_ref() != Some(&args) {
                failures.push(format!(
                    "{tool} -> {hook}: ALLOW must not throw and must pass args through unchanged \
                     (threw={}, args_after={:?}, args_expected={args})",
                    r.threw, args_after
                ));
            }
            let captured = read_captured_stdin(fx.path());
            let expected = expected_opencode_payload(&session_id, fx.path(), expected_tool_name, &expected_tool_input);
            if captured != expected {
                failures.push(format!(
                    "{tool} -> {hook}: payload bee received does not match the translated shape \
                     (F3) — expected {expected}, got {captured}"
                ));
            }
        }

        // (c) crash — a nonzero, non-2 exit must still throw (fail closed).
        {
            let fx = tempfile::tempdir().expect("tempdir");
            write_stub_bee(fx.path(), &StubBehavior::Crash);
            let r = run_harness(&harness, &plugin, fx.path(), "tool.execute.before", &call_payload);
            if !(r.threw && r.message.as_deref().is_some_and(|m| m.contains("did not return a verdict"))) {
                failures.push(format!(
                    "{tool} -> {hook}: CRASH must throw a \"did not return a verdict\" Error \
                     (threw={}, message={:?})",
                    r.threw, r.message
                ));
            }
        }

        // (d) missing binary — no .bee/bin/bee anywhere reachable must throw.
        {
            let fx = tempfile::tempdir().expect("tempdir");
            write_stub_bee(fx.path(), &StubBehavior::Missing);
            let r = run_harness(&harness, &plugin, fx.path(), "tool.execute.before", &call_payload);
            if !(r.threw && r.message.as_deref().is_some_and(|m| m.contains("could not find the bee binary"))) {
                failures.push(format!(
                    "{tool} -> {hook}: MISSING BINARY must throw a \"could not find the bee binary\" Error \
                     (threw={}, message={:?})",
                    r.threw, r.message
                ));
            }
        }

        // (e) D6 repair — exit-0 verdict carrying `hookSpecificOutput.
        // updatedInput`: must not throw, and `output.args` must gain the
        // repaired field via `Object.assign` (oc-8's F2 fix).
        {
            let fx = tempfile::tempdir().expect("tempdir");
            write_stub_bee(fx.path(), &StubBehavior::Repair);
            let r = run_harness(&harness, &plugin, fx.path(), "tool.execute.before", &call_payload);
            let args_after = r.output.as_ref().and_then(|o| o.get("args")).cloned();
            let repaired_ok = args_after
                .as_ref()
                .and_then(|a| a.get("repairedField"))
                .and_then(Value::as_str)
                == Some("repaired-value");
            if r.threw || !repaired_ok {
                failures.push(format!(
                    "{tool} -> {hook}: D6 repair must not throw and must apply updatedInput onto \
                     output.args (threw={}, args_after={:?})",
                    r.threw, args_after
                ));
            }
        }

        // (f) D6 ask — exit-0 verdict carrying `permissionDecision: "ask"`:
        // must throw, carrying the reason verbatim. write-guard's own "ask,
        // never allow" (write_guard/main.rs:389-394) is bee's DOMINANT
        // enforcement path for a repaired AskUserQuestion call — treating it
        // as an allow would silently defeat it on OpenCode specifically
        // (oc-8's F2 finding).
        {
            let fx = tempfile::tempdir().expect("tempdir");
            let reason = format!("stub-ask for {hook} via {tool}");
            write_stub_bee(fx.path(), &StubBehavior::Ask(reason.clone()));
            let r = run_harness(&harness, &plugin, fx.path(), "tool.execute.before", &call_payload);
            if !(r.threw && r.message.as_deref().is_some_and(|m| m.contains(&reason))) {
                failures.push(format!(
                    "{tool} -> {hook}: D6 ask verdict must throw the stub's reason (threw={}, message={:?})",
                    r.threw, r.message
                ));
            }
        }

        // (g) D6 unparseable — exit-0 stdout that is non-empty but not valid
        // JSON: undecidable, and undecidable stays fail-closed on the
        // BLOCKING path, never a silent allow.
        {
            let fx = tempfile::tempdir().expect("tempdir");
            write_stub_bee(fx.path(), &StubBehavior::UnparseableVerdict);
            let r = run_harness(&harness, &plugin, fx.path(), "tool.execute.before", &call_payload);
            if !(r.threw && r.message.as_deref().is_some_and(|m| m.contains("could not parse"))) {
                failures.push(format!(
                    "{tool} -> {hook}: D6 unparseable exit-0 verdict must throw a \"could not \
                     parse\" Error (threw={}, message={:?})",
                    r.threw, r.message
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "one or more BLOCKING mapped rows failed their deny/allow/crash/missing fixtures:\n{}",
        failures.join("\n")
    );
}

#[cfg(unix)]
#[test]
fn advisory_surfaces_never_throw_regardless_of_the_bee_binarys_behavior() {
    node_or_skip!("advisory_surfaces_never_throw_regardless_of_the_bee_binarys_behavior");

    let harness_dir = tempfile::tempdir().expect("tempdir for the harness script");
    let harness = write_harness(harness_dir.path());
    let plugin = repo_root().join(".opencode/plugins/bee-guard.ts");

    // Every surface the plugin wires an ADVISORY hook onto (bee-guard.ts):
    // chat.message covers session-init + prompt-context; the three `event`
    // shapes cover state-sync (file.edited, session.idle) and session-close
    // (session.idle, session.deleted); tool.execute.after covers
    // tools-logger.
    let surfaces: Vec<(&str, Value)> = vec![
        ("chat.message", json!({"sessionID": "s1", "messageID": "m1"})),
        ("event", json!({"event": {"type": "file.edited", "properties": {"file": "x.txt"}}})),
        ("event", json!({"event": {"type": "session.idle", "properties": {"sessionID": "s1"}}})),
        ("event", json!({"event": {"type": "session.deleted", "properties": {"info": {"id": "s1"}}}})),
        ("tool.execute.after", json!({"tool": "write", "args": {"filePath": "/tmp/x", "content": "hi"}})),
    ];

    let scenarios = [
        StubBehavior::Deny("stub-deny-advisory".to_string()),
        StubBehavior::Allow,
        StubBehavior::Crash,
        StubBehavior::Missing,
    ];

    let mut failures: Vec<String> = Vec::new();
    for (surface, payload) in &surfaces {
        for behavior in &scenarios {
            let fx = tempfile::tempdir().expect("tempdir");
            write_stub_bee(fx.path(), behavior);
            let r = run_harness(&harness, &plugin, fx.path(), surface, payload);
            if r.threw {
                failures.push(format!(
                    "surface {surface} threw ({:?}) even though every hook it reaches is ADVISORY \
                     (must swallow and log, never throw)",
                    r.message
                ));
            }
        }
    }

    assert!(failures.is_empty(), "advisory surfaces must never throw:\n{}", failures.join("\n"));

    // Non-vacuity: confirm the surfaces above really do reach the plugin's
    // ADVISORY hooks (derived from its own source), not an empty no-op.
    let wired = opencode_advisory_hooks();
    for expected in ["session-init", "prompt-context", "state-sync", "session-close", "tools-logger"] {
        assert!(
            wired.contains(expected),
            "expected bee-guard.ts to wire \"{expected}\" via runAdvisoryHook, but the derived set was {wired:?}"
        );
    }
}

#[cfg(not(unix))]
#[test]
fn opencode_plugin_fixtures_skip_on_non_unix() {
    eprintln!(
        "SKIP (env-limited: the stub bee binaries in this suite are unix shebang scripts) \
         — opencode plugin fixture tests"
    );
}

// ═════════════════════════════════════════════════════════════════════════
// PART 2 — the three-belt parity test.
// ═════════════════════════════════════════════════════════════════════════

/// A minimal fixture repo the NATIVE hooks accept as a bee root — same shape
/// as `hook_contracts.rs`'s own `fixture()` (the `.bee/onboarding.json`
/// install marker every hook's activation probe reads since the R6
/// cutover), reproduced here so the HELPER-level check below is
/// self-contained and never depends on another test binary's internals.
struct HelperFixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

fn helper_fixture() -> HelperFixture {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dunce::canonicalize(dir.path()).expect("canonicalize tempdir");
    std::fs::create_dir_all(root.join(".bee").join("logs")).unwrap();
    std::fs::create_dir_all(root.join(".bee").join("cells")).unwrap();
    std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
    std::fs::write(
        root.join(".bee").join("state.json"),
        serde_json::to_string_pretty(&json!({
            "phase": "swarming",
            "mode": "standard",
            "feature": "demo",
            "approved_gates": { "context": true, "shape": true, "execution": true, "review": false }
        }))
        .unwrap()
            + "\n",
    )
    .unwrap();
    HelperFixture { _dir: dir, root }
}

fn run_helper_hook(hook: &str, stdin: &[u8], cwd: &Path) -> Output {
    let mut child = Command::new(bee_bin())
        .args(["hook", hook])
        .current_dir(cwd)
        .env("BEE_HOOK_NO_DELEGATE", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to launch `bee hook {hook}`: {e}"));
    child.stdin.take().unwrap().write_all(stdin).ok();
    child.wait_with_output().unwrap_or_else(|e| panic!("`bee hook {hook}` never exited: {e}"))
}

/// The known-denying payload SHAPE per BLOCKING rule — the same domain
/// knowledge `hook_contracts.rs`'s own two "decision reaches the host"
/// tests already encode (a direct edit to `.bee/state.json` for
/// write-guard; a bare Agent/Task dispatch with no tier marker for
/// model-guard). Only the TOOL NAME is taken from the derived catalog row
/// rather than hand-typed.
fn helper_deny_payload(hook: &str, claude_tool_name: &str, root: &Path) -> Value {
    match hook {
        "write-guard" => json!({
            "tool_name": claude_tool_name,
            "tool_input": { "file_path": ".bee/state.json" },
            "cwd": root.to_string_lossy(),
        }),
        "model-guard" => json!({
            "tool_name": claude_tool_name,
            "tool_input": { "prompt": "implement the widget", "description": "some description" },
            "cwd": root.to_string_lossy(),
        }),
        other => panic!("no known deny-payload shape for BLOCKING rule \"{other}\" — add one before trusting this parity check"),
    }
}

/// For each `#[test] fn ... { ... }` block in `hook_contracts.rs`'s embedded
/// source, true iff some block both names one of `tool_names` as a
/// `"tool_name"` literal AND asserts a `code(&out), 2` / `code(out), 2` deny.
fn claude_belt_deny_fixture_exists(tool_names: &[String]) -> bool {
    let idxs: Vec<usize> = HOOK_CONTRACTS_SOURCE.match_indices("#[test]").map(|(i, _)| i).collect();
    for (n, &start) in idxs.iter().enumerate() {
        let end = idxs.get(n + 1).copied().unwrap_or(HOOK_CONTRACTS_SOURCE.len());
        let block = &HOOK_CONTRACTS_SOURCE[start..end];
        let names_a_tool =
            tool_names.iter().any(|t| block.contains(&format!("\"tool_name\": \"{t}\"")));
        let asserts_deny = block.contains("code(&out), 2") || block.contains("code(out), 2");
        if names_a_tool && asserts_deny {
            return true;
        }
    }
    false
}

/// General form of `claude_belt_deny_fixture_exists`: true iff some
/// `#[test] fn ... { ... }` block in `source` both names one of
/// `tool_names` as a `"tool_name"` literal — either JSON-string spacing
/// style, since `hook_contracts.rs`/`model_guard.rs` write `"tool_name":
/// "X"` (with a space) and `write_guard/tests.rs`'s AskUserQuestion
/// fixtures write `"tool_name":"X"` (without one) — AND contains every one
/// of `markers`.
fn claude_belt_test_block_has(source: &str, tool_names: &[String], markers: &[&str]) -> bool {
    let idxs: Vec<usize> = source.match_indices("#[test]").map(|(i, _)| i).collect();
    for (n, &start) in idxs.iter().enumerate() {
        let end = idxs.get(n + 1).copied().unwrap_or(source.len());
        let block = &source[start..end];
        let names_a_tool = tool_names.iter().any(|t| {
            block.contains(&format!("\"tool_name\": \"{t}\"")) || block.contains(&format!("\"tool_name\":\"{t}\""))
        });
        let has_markers = markers.iter().all(|m| block.contains(m));
        if names_a_tool && has_markers {
            return true;
        }
    }
    false
}

/// Whether CLAUDE-belt fixture coverage exists for (hook, shape). Reuses
/// `claude_belt_deny_fixture_exists` (and its `hook_contracts.rs` proof
/// surface) for DENY_SHAPE unchanged; for the other two shapes, hand-authored
/// knowledge of WHICH embedded source carries a given rule's OWN fixtures —
/// `write_guard/tests.rs` for write-guard's ask/repair, `model_guard.rs`'s
/// own embedded test module for model-guard's repair — since which file a
/// rule's fixtures live in is not itself derivable from the emit path.
fn claude_belt_shape_fixture_exists(hook: &str, shape: &str, tool_names: &[String]) -> bool {
    if shape == DENY_SHAPE {
        return claude_belt_deny_fixture_exists(tool_names);
    }
    let source = match hook {
        "write-guard" => WRITE_GUARD_TESTS_SOURCE,
        "model-guard" => MODEL_GUARD_SOURCE,
        other => panic!("no known CLAUDE-belt fixture source for BLOCKING rule \"{other}\""),
    };
    if shape == REPAIR_SHAPE {
        claude_belt_test_block_has(source, tool_names, &["updatedInput"])
    } else if shape == ASK_SHAPE {
        claude_belt_test_block_has(source, tool_names, &["permissionDecision", "\"ask\""])
    } else {
        panic!("claude_belt_shape_fixture_exists: no fixture-search wired for shape \"{shape}\"");
    }
}

/// The known-payload SHAPE per (BLOCKING rule, verdict shape) at HELPER
/// level. DENY_SHAPE defers to `helper_deny_payload` unchanged. The other
/// two are domain knowledge no catalog row carries (mirroring
/// `helper_deny_payload`'s own precedent): write-guard's AskUserQuestion
/// long-header auto-fix (`write_guard/tests.rs`'s own
/// `ask_long_header_is_auto_fixed`) carries BOTH REPAIR_SHAPE and ASK_SHAPE
/// in one emission; model-guard's own `marker_plus_param_agreement_rules`
/// param/tier disagreement repairs with no `permissionDecision` at all —
/// `emittable_shapes` never derives ASK_SHAPE for model-guard, so that
/// combination never reaches this function.
fn helper_shape_payload(hook: &str, shape: &str, claude_tool: &str, root: &Path) -> Value {
    if shape == DENY_SHAPE {
        return helper_deny_payload(hook, claude_tool, root);
    }
    match (hook, shape) {
        (h, s) if h == "write-guard" && (s == REPAIR_SHAPE || s == ASK_SHAPE) => json!({
            "tool_name": "AskUserQuestion",
            "tool_input": { "questions": [{
                "question": "q",
                "header": "Worktree switch",
                "options": [{"label": "A", "description": "x"}, {"label": "B", "description": "y"}],
            }]},
            "cwd": root.to_string_lossy(),
        }),
        (h, s) if h == "model-guard" && s == REPAIR_SHAPE => json!({
            "tool_name": claude_tool,
            "tool_input": { "prompt": "[bee-tier: generation] go", "model": "opus" },
            "cwd": root.to_string_lossy(),
        }),
        (h, s) => panic!(
            "no known payload shape for BLOCKING rule \"{h}\" / verdict shape \"{s}\" — add one to \
             `helper_shape_payload` before trusting this parity check"
        ),
    }
}

/// Runs `bee hook <hook>` at HELPER level against `shape`'s known-triggering
/// payload and returns `Some(gap message)` (naming rule, shape, and belt) if
/// the emission does not actually carry that shape.
fn helper_shape_gap(hook: &str, shape: &str, claude_tool: &str, root: &Path) -> Option<String> {
    let payload = helper_shape_payload(hook, shape, claude_tool, root);
    let out = run_helper_hook(hook, payload.to_string().as_bytes(), root);
    let code = out.status.code().unwrap_or(-1);
    if code == 42 {
        return Some(format!(
            "{hook} / {shape} / helper belt: `bee hook {hook}` still delegates to Node under \
             BEE_HOOK_NO_DELEGATE — the native decision path was never reached"
        ));
    }
    if shape == DENY_SHAPE {
        return if code != 2 {
            Some(format!(
                "{hook} / {shape} / helper belt: expected exit 2 (deny) for the known-denying \
                 payload, got exit {code} stderr={}",
                String::from_utf8_lossy(&out.stderr)
            ))
        } else {
            None
        };
    }
    if code != 0 {
        return Some(format!(
            "{hook} / {shape} / helper belt: expected exit 0 (a verdict-carrying allow) for the \
             known-triggering payload, got exit {code} stderr={}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: Value = match serde_json::from_str(stdout.trim()) {
        Ok(v) => v,
        Err(e) => {
            return Some(format!(
                "{hook} / {shape} / helper belt: exit-0 stdout was not valid JSON ({e}): {stdout}"
            ))
        }
    };
    let hso = &parsed["hookSpecificOutput"];
    let has_shape = if shape == REPAIR_SHAPE {
        !hso["updatedInput"].is_null()
    } else if shape == ASK_SHAPE {
        hso["permissionDecision"].as_str() == Some("ask")
    } else {
        panic!("helper_shape_gap: no verdict-shape marker check wired for \"{shape}\"");
    };
    if has_shape {
        None
    } else {
        Some(format!(
            "{hook} / {shape} / helper belt: exit-0 verdict did not carry the expected shape: {parsed}"
        ))
    }
}

/// The literal marker in `bee-guard.ts`'s own `runBlockingHook` that proves
/// it still implements `shape` — each marker is the exact
/// conditional/property-access line D6's own doc comment cites for that
/// shape (bee-guard.ts: `err?.status === 2` for deny, `hso.updatedInput` for
/// repair, `hso.permissionDecision === "ask"` for ask).
fn opencode_belt_shape_marker(shape: &str) -> &'static str {
    if shape == DENY_SHAPE {
        "err?.status === 2"
    } else if shape == REPAIR_SHAPE {
        "hso.updatedInput"
    } else if shape == ASK_SHAPE {
        "hso.permissionDecision === \"ask\""
    } else {
        panic!(
            "opencode_belt_shape_marker: no marker wired for shape \"{shape}\" — add one before \
             trusting this cross-check"
        );
    }
}

#[test]
fn three_belt_parity_every_blocking_rule_hits_helper_claude_codex_and_opencode() {
    let catalog = derive_catalog();
    let blocking: Vec<&CatalogRow> = catalog.values().filter(|r| r.blocking).collect();
    assert!(
        blocking.len() >= 2,
        "derived catalog found only {} BLOCKING rule(s) — expected at least write-guard and \
         model-guard; the derivation likely broke: {:?}",
        blocking.len(),
        catalog.keys().collect::<Vec<_>>()
    );

    let opencode_pairs = opencode_tool_hook_pairs();
    let opencode_hooks: BTreeSet<&str> = opencode_pairs.iter().map(|(_, h)| h.as_str()).collect();

    let mut gaps: Vec<String> = Vec::new();

    for row in &blocking {
        let hook = row.hook.as_str();
        let claude_tool = row
            .claude_pretooluse_matchers
            .first()
            .unwrap_or_else(|| panic!("BLOCKING rule \"{hook}\" has no claude PreToolUse matcher to derive a tool name from"));

        // The row set widens from RULES to (rule, verdict SHAPE) pairs here
        // — see the doc comment above `emittable_shapes`. A whole shape
        // (not just a whole rule) can now go missing on one belt and fail
        // this suite by name.
        for shape in emittable_shapes(hook) {
            // HELPER level: `bee hook <hook>` itself emits this shape for
            // its known-triggering payload.
            {
                let fx = helper_fixture();
                if let Some(gap) = helper_shape_gap(hook, shape, claude_tool, &fx.root) {
                    gaps.push(gap);
                }
            }

            // CLAUDE belt: the rule is wired under PreToolUse in claude's
            // file-shipped projection (already true by construction of
            // `blocking`, but checked per-projection here since `blocking`
            // is a union) AND this rule's own embedded fixture suite proves
            // a Claude-shaped tool_name literal reaches this shape.
            if row.claude_pretooluse_matchers.is_empty() {
                gaps.push(format!("{hook} / {shape} / claude belt: not wired under PreToolUse in claude-hooks.json"));
            } else if !claude_belt_shape_fixture_exists(hook, shape, &row.claude_pretooluse_matchers) {
                gaps.push(format!(
                    "{hook} / {shape} / claude belt: no fixture proves a Claude-shaped tool_name \
                     literal (one of {:?}) reaches this shape",
                    row.claude_pretooluse_matchers
                ));
            }

            // CODEX belt: the rule is wired under PreToolUse in codex's
            // file-shipped projection. Codex has no separate translation
            // layer to fixture-test beyond that wiring, for ANY shape —
            // its PreToolUse command execs `bee hook <rule>` directly, byte
            // for byte the same call the HELPER check above already
            // exercised for this exact shape.
            if row.codex_pretooluse_matchers.is_empty() {
                gaps.push(format!(
                    "{hook} / {shape} / codex belt: not wired under PreToolUse in \
                     packages/bee/hooks/hooks.json"
                ));
            }

            // OPENCODE belt: bee-guard.ts's mapToolCall actually routes some
            // real tool to this hook (as before) AND runBlockingHook's own
            // source still implements this shape — the live
            // deny/repair/ask/unparseable PROOF lives in
            // `every_blocking_mapped_row_denies_allows_crashes_and_reports_a_missing_binary`;
            // this is the routing+implementation cross-check that keeps
            // that proof from silently going vacuous if the shape's own
            // handling were ever deleted from the plugin.
            if !opencode_hooks.contains(hook) {
                gaps.push(format!(
                    "{hook} / {shape} / opencode belt: bee-guard.ts's mapToolCall routes no tool \
                     to this hook (derived pairs: {opencode_pairs:?})"
                ));
            } else if !PLUGIN_SOURCE.contains(opencode_belt_shape_marker(shape)) {
                gaps.push(format!(
                    "{hook} / {shape} / opencode belt: bee-guard.ts's runBlockingHook no longer \
                     implements this shape (marker {:?} not found)",
                    opencode_belt_shape_marker(shape)
                ));
            }
        }
    }

    // UNPARSEABLE_SHAPE: see the doc comment above `emittable_shapes` — not
    // producer-derived (bee's own `jsjson::stringify` never emits invalid
    // JSON), so it is not part of any rule's `emittable_shapes` row set and
    // is checked once, globally, rather than per rule.
    eprintln!(
        "verdict-shape parity: an unparseable exit-0 verdict is a NAMED EXCLUSION for the helper, \
         claude, and codex belts — bee's own `jsjson::stringify` never emits invalid JSON, so \
         there is nothing to parse at the point of emission, and the claude/codex belts' stdout is \
         parsed by their closed-source host applications, outside this repo. Asserted only where a \
         belt in this repo actually parses bee's own stdout: the opencode belt \
         (bee-guard.ts's runBlockingHook, live-proven by \
         every_blocking_mapped_row_denies_allows_crashes_and_reports_a_missing_binary's \
         StubBehavior::UnparseableVerdict scenario)."
    );
    if !PLUGIN_SOURCE.contains("could not parse") {
        gaps.push(
            "unparseable exit-0 verdict / opencode belt: bee-guard.ts's runBlockingHook no longer \
             throws a \"could not parse\" Error on invalid exit-0 verdict JSON"
                .to_string(),
        );
    }

    assert!(
        gaps.is_empty(),
        "verdict-shape parity gap(s), naming the rule, the shape, and the belt that missed it:\n{}",
        gaps.join("\n")
    );
}

/// True iff `name` and one of `markers` co-occur on the SAME LINE of
/// `discovery.md` — never merely "both appear somewhere in the document".
///
/// F5: the previous version of this check ANDed a per-rule `contains(rule)`
/// with a DOCUMENT-GLOBAL `contains("NAMED EXCLUSION") ||
/// contains("Deferred")` — but both marker literals are always present
/// SOMEWHERE in this file (on `codex-subagent-audit`'s and `chain-nudge`'s
/// own rows), so ANY rule name mentioned anywhere in the document passed,
/// whether or not that specific mention was actually tagged as a gap. Every
/// gap this file documents today is already written with its name and its
/// marker on the SAME line (each is one markdown table row, e.g.
/// `| codex-subagent-audit | n/a | NAMED EXCLUSION — ... |`), so scoping to
/// one line is a real narrowing, not a cosmetic one — see
/// docs/knowledge/patterns/20260722-a-coverage-gate-derives-ground-truth-it-
/// never-compares-two-hand-lists.md.
fn discovery_doc_names_as_a_gap(name: &str, markers: &[&str]) -> bool {
    DISCOVERY_DOC.lines().any(|line| line.contains(name) && markers.iter().any(|m| line.contains(m)))
}

/// The parity test above only requires BLOCKING rules to hit all three
/// belts. ADVISORY rules the OpenCode plugin does not wire
/// (`codex-subagent-audit`, `chain-nudge`) are allowed to be missing — but
/// ONLY if that gap is already a NAMED, documented exclusion, never a
/// silent one. This is the coverage-gate half of the contract: any future
/// ADVISORY hook the catalog gains that the plugin does not wire, and that
/// discovery.md does not name (on that rule's own line — F5), fails here
/// rather than shipping silently.
#[test]
fn advisory_gaps_the_plugin_does_not_wire_are_named_not_silent() {
    let catalog = derive_catalog();
    let advisory_rules: Vec<&str> = catalog.values().filter(|r| !r.blocking).map(|r| r.hook.as_str()).collect();
    assert!(
        advisory_rules.len() >= 2,
        "derived catalog found too few ADVISORY rules to be a meaningful check: {advisory_rules:?}"
    );

    let wired = opencode_advisory_hooks();
    let mut unnamed_gaps: Vec<String> = Vec::new();
    for rule in advisory_rules {
        if wired.contains(rule) {
            continue; // covered live by `advisory_surfaces_never_throw_regardless_of_the_bee_binarys_behavior`
        }
        // Not wired — must be named in discovery.md ON ITS OWN LINE (F5),
        // not silently dropped and not merely co-mentioned with an unrelated
        // gap's marker elsewhere in the document.
        if !discovery_doc_names_as_a_gap(rule, &["NAMED EXCLUSION", "Deferred"]) {
            unnamed_gaps.push(format!(
                "{rule}: not wired by bee-guard.ts's runAdvisoryHook AND not documented on its own \
                 line as a named gap in docs/history/opencode-support/discovery.md"
            ));
        }
    }

    assert!(
        unnamed_gaps.is_empty(),
        "silent ADVISORY coverage gap(s) — every hook the opencode belt does not wire must be a \
         NAMED exclusion in discovery.md:\n{}",
        unnamed_gaps.join("\n")
    );
}

// ═════════════════════════════════════════════════════════════════════════
// PART 3 — F4: the tool-registry coverage gate. `pairs.len() >= 9` (part 1
// above) and "at least one tool routes to this hook" (part 2 above) both
// only check mapToolCall's OWN claims about itself — a hand-authored floor
// compared against the hand-authored switch statement it is meant to police
// proves internal consistency, never coverage (docs/knowledge/patterns/
// 20260722-a-coverage-gate-derives-ground-truth-it-never-compares-two-hand-
// lists.md). oc-3 closed exactly this gap for `apply_patch`, which the
// installed binary registered under its own write-permission group while
// mapToolCall's `default: return null` arm let it through as a TypeScript-
// side allow. This section derives the REGISTERED tool inventory from the
// installed opencode binary itself, so that defect class cannot recur
// silently for the NEXT unmapped write- or read-capable tool.
// ═════════════════════════════════════════════════════════════════════════

/// Resolves the real, installed `opencode` binary this machine's PATH
/// points `opencode` at — never a hardcoded machine-specific path — via the
/// shell's own `command -v`, then canonicalized to follow the nvm-managed
/// symlink chain down to the real compiled executable (oc-1's Install
/// section: `~/.nvm/.../bin/opencode` -> `.../opencode-ai/bin/opencode.exe`).
/// Resolution is POSIX-shaped on purpose, and its limit is named rather than
/// papered over: it asks a POSIX shell for the binary and canonicalizes what
/// comes back, which follows the package manager's symlink to the real
/// executable whose bytes this derivation greps. On Windows that chain does
/// not hold — the shell answers with a POSIX-styled path to a `.cmd` shim,
/// which neither canonicalizes nor carries the bundled payload — so the
/// caller degrades to a NAMED skip there (see the call site) instead of
/// pretending to derive a registry from a batch script. The gate itself is
/// not weakened: the registry it derives is platform-independent content, and
/// Linux CI runs this hard-failing on every commit.
fn resolve_opencode_binary() -> Result<PathBuf, String> {
    let out = Command::new("sh")
        .arg("-c")
        .arg("command -v opencode")
        .output()
        .map_err(|e| format!("failed to run `command -v opencode`: {e}"))?;
    if !out.status.success() {
        return Err("`opencode` not found on PATH".to_string());
    }
    let raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if raw.is_empty() {
        return Err("`command -v opencode` produced no path".to_string());
    }
    let path = std::fs::canonicalize(&raw)
        .map_err(|e| format!("could not canonicalize \"{raw}\": {e}"))?;
    if !is_wrapper_script(&path) {
        return path_ok(path);
    }
    // The PATH entry is a wrapper, not the payload. Ask each version manager
    // that ships one to name the executable it actually execs.
    for manager in VERSION_MANAGER_RESOLVERS {
        let Some(candidate) = ask_resolver(manager) else { continue };
        if !is_wrapper_script(&candidate) {
            return path_ok(candidate);
        }
    }
    Err(format!(
        "the PATH entry {} is a WRAPPER SCRIPT, not the bundled executable this derivation greps, \
         and no version-manager resolver ({}) could name the real binary — install opencode-ai so \
         `command -v opencode` lands on the payload, or make the owning manager resolvable on PATH",
        path.display(),
        VERSION_MANAGER_RESOLVERS.join(", ")
    ))
}

/// Managers that install a shebang shim on PATH and answer `<manager> which
/// <tool>` with the executable it execs. Ordered, first usable answer wins;
/// an absent manager is skipped, never an error.
const VERSION_MANAGER_RESOLVERS: &[&str] = &["mise", "asdf", "volta"];

fn ask_resolver(manager: &str) -> Option<PathBuf> {
    let out = Command::new(manager).arg("which").arg("opencode").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if raw.is_empty() {
        return None;
    }
    std::fs::canonicalize(&raw).ok()
}

/// A wrapper is a text script standing in for the executable — a shebang
/// script (mise/asdf/volta shims, npm bin stubs) or a Windows `.cmd` batch
/// stub. Either way its bytes carry no bundled payload, so a derivation that
/// greps them reports zero tools and calls it a shape change. Named here
/// rather than papered over: the resolver above walks past it or says why it
/// could not.
fn is_wrapper_script(path: &Path) -> bool {
    if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("cmd") || e.eq_ignore_ascii_case("bat")) {
        return true;
    }
    let Ok(bytes) = std::fs::read(path) else { return false };
    bytes.starts_with(b"#!")
}

fn path_ok(path: PathBuf) -> Result<PathBuf, String> {
    Ok(path)
}

/// The resolver's own contract, pinned without needing an opencode install:
/// a shebang script and a `.cmd` stub are wrappers; an ELF payload is not.
#[test]
fn a_wrapper_script_is_never_mistaken_for_the_bundled_payload() {
    let tmp = tempfile::tempdir().expect("tempdir");

    let shim = tmp.path().join("opencode");
    std::fs::write(&shim, b"#!/bin/bash\nexec mise x opencode -- opencode \"$@\"\n").expect("write shim");
    assert!(is_wrapper_script(&shim), "a shebang shim must read as a wrapper");

    let cmd = tmp.path().join("opencode.cmd");
    std::fs::write(&cmd, b"@echo off\r\n").expect("write cmd");
    assert!(is_wrapper_script(&cmd), "a .cmd stub must read as a wrapper");

    let elf = tmp.path().join("opencode-real");
    std::fs::write(&elf, b"\x7fELF\x02\x01\x01\x00payload").expect("write elf");
    assert!(!is_wrapper_script(&elf), "an ELF payload must NOT read as a wrapper");

    let missing = tmp.path().join("absent");
    assert!(!is_wrapper_script(&missing), "an unreadable path is not a wrapper claim");
}

/// Reads the resolved opencode binary's own bytes as text. The installed
/// binary is a compiled, per-platform executable with no plain JS source to
/// read directly (oc-1's Blockers section) — but it is a Bun-bundled
/// standalone executable, so its own minified JS payload (tool
/// registrations included) is still present as plain, greppable ASCII text
/// INSIDE it, exactly the fact oc-3's "Static binary read" already
/// established and relied on for `apply_patch`. Lossy UTF-8 is fine: every
/// literal this derivation searches for is plain ASCII.
fn opencode_binary_text() -> Result<String, String> {
    let path = resolve_opencode_binary()?;
    let bytes = std::fs::read(&path).map_err(|e| format!("could not read {}: {e}", path.display()))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// The 14-element tool-id `Set` OpenCode's own icon-lookup helper builds
/// (`var Ea=new Set([...]);function Ia(U){return Ea.has(U)?U:"generic"}` in
/// `opencode-ai@1.18.16`, the version `.github/workflows/ci.yml` pins) — a
/// VALUES-based literal, so it survives the surrounding minified variable
/// name (`Ea`) changing on a rebuild. That is not hypothetical: the very
/// same rebuild drift hit `ADDITIONAL_TOOL_ANCHORS` below between 1.18.16
/// and 1.18.21, and this literal rode through it untouched — the case FOR
/// anchoring on values. This is the primary derivation anchor: every element
/// is a real, binary-confirmed OpenCode tool id.
const TOOL_SET_LITERAL: &str =
    "[\"bash\",\"glob\",\"read\",\"grep\",\"webfetch\",\"websearch\",\"write\",\"edit\",\"task\",\"apply_patch\",\"todowrite\",\"question\",\"skill\",\"execute\"]";

/// Three further registered tool ids the `Set` above omits, each confirmed
/// by ITS OWN independent literal anchor from that tool's registration body
/// in the same binary (`"<id>",s.gen(function*(){...` / `"<id>",s.
/// succeed(...`) — plus, for the ones `mapToolCall` does not already map,
/// whether that same body reveals a caller-supplied `filePath` parameter
/// (the exact shape `lsp` — this cell's finding — and `apply_patch` before
/// oc-3 closed it, both carry).
///
/// ANCHOR RULE, learned the hard way: anchor on the tool id and the real
/// registration VALUES that follow it, never on a minifier-chosen
/// identifier. These three anchors used to start with the registration
/// helper's minified name, `V(` — which is not stable across opencode
/// builds. It is `V` in the `opencode-ai@1.18.16` that
/// `.github/workflows/ci.yml` pins and `j` in 1.18.21; one character
/// changed, everything after it byte-identical. CI stayed green on the old
/// anchors while any developer on a newer build got a hard failure that
/// says nothing about opencode's actual tool registry. The shortened
/// anchors match BOTH versions and are each still unique (one hit apiece,
/// measured in the installed 1.18.21 bundle).
///
/// `s.succeed` and `s.gen` are themselves minified identifiers, kept
/// DELIBERATELY: `"plan_exit"` and `"lsp"` alone are far too weak to anchor
/// on — those strings appear all over the bundle. So this is a named trade,
/// not an oversight: enough minified text to stay unique, never the leading
/// identifier that the minifier renumbers. If a future rebuild renames `s`
/// too, this same assert fires and the fix is the same one line of
/// reasoning, diagnosed in seconds instead of re-derived from scratch.
///
/// `id` and `anchor` are independent binary evidence; `filepath_evidence`
/// records whether a scan of the text FOLLOWING `anchor` (bounded, since
/// tool bodies sit back-to-back in the bundle) found `filePath` — `Some(true
/// /false)` when the anchor was found, `None` when it was not (treated as
/// "unknown", never silently exempted).
struct AdditionalToolAnchor {
    id: &'static str,
    anchor: &'static str,
}

const ADDITIONAL_TOOL_ANCHORS: &[AdditionalToolAnchor] = &[
    AdditionalToolAnchor { id: "invalid", anchor: "\"invalid\",s.succeed({description:\"Do not use\"" },
    AdditionalToolAnchor { id: "plan_exit", anchor: "\"plan_exit\",s.gen" },
    AdditionalToolAnchor { id: "lsp", anchor: "\"lsp\",s.gen" },
];

/// For the tool ids inside `TOOL_SET_LITERAL` that `mapToolCall` does not
/// map, the anchor this derivation locates that id's OWN registration body
/// through (each tool uses its own real `permission:"<id>"` Effect
/// `.ask()` call as a stable, values-based anchor — confirmed present for
/// every one of these five in the installed binary).
const UNMAPPED_SET_TOOL_ANCHORS: &[(&str, &str)] = &[
    ("webfetch", "permission:\"webfetch\""),
    ("websearch", "permission:\"websearch\""),
    ("todowrite", "permission:\"todowrite\""),
    ("skill", "permission:\"skill\""),
    ("execute", "Script body executed by the confined interpreter"),
];

/// How far past an anchor this derivation looks for a `filePath` parameter
/// before giving up — wide enough to cover every tool body actually
/// observed in the installed binary (the longest, `lsp`'s, needed under 200
/// chars), narrow enough that it cannot walk into the NEXT tool's body and
/// misattribute its `filePath`.
const FILEPATH_SCAN_WINDOW: usize = 500;

/// True iff `anchor` is found in `text`, AND `filePath` appears within
/// `FILEPATH_SCAN_WINDOW` bytes after it. `None` when the anchor itself was
/// not found (an unlocatable body is "unknown", not "safe" — see the
/// exemption reasoning below).
fn anchor_body_contains_filepath(text: &str, anchor: &str) -> Option<bool> {
    let idx = text.find(anchor)?;
    let window_end = (idx + anchor.len() + FILEPATH_SCAN_WINDOW).min(text.len());
    Some(text[idx..window_end].contains("filePath"))
}

/// The tool-registry ground truth: every OpenCode tool id this cell could
/// mechanically confirm the installed binary registers, mapped to whether
/// this derivation found evidence it is write- or read-capable (reads/
/// writes an arbitrary caller-supplied file path) — `true`/`false` when
/// classified, `None` when the binary text gave no evidence either way
/// (treated as REQUIRING coverage below, never silently exempted — absence
/// of evidence is not evidence of safety).
///
/// `list` — the S3 judge's OTHER named finding alongside `lsp` — has NO
/// located static anchor in this binary: the judge's own evidence for it
/// was almost certainly a live `tool.definition` probe (oc-1/oc-3's OTHER,
/// non-static evidence source, exactly like oc-3's live-probe half), which
/// a `cargo test` cannot reproduce deterministically offline (no model
/// access, no network). It is recorded as a manually-confirmed gap in
/// discovery.md instead of a derived one here — named, never silently
/// assumed covered by this derivation's silence on it.
fn derive_opencode_tool_registry(text: &str) -> Result<BTreeMap<String, Option<bool>>, String> {
    if !text.contains(TOOL_SET_LITERAL) {
        return Err(format!(
            "the installed opencode binary's tool-id Set literal ({TOOL_SET_LITERAL}) was not \
             found — has the installed opencode-ai version changed shape? this derivation needs \
             updating rather than silently reporting zero tools"
        ));
    }
    let mut registry: BTreeMap<String, Option<bool>> = BTreeMap::new();
    for id in TOOL_SET_LITERAL.trim_start_matches('[').trim_end_matches(']').split(',') {
        let id = id.trim_matches('"');
        registry.insert(id.to_string(), None); // classified below only for unmapped ones
    }
    for (id, anchor) in UNMAPPED_SET_TOOL_ANCHORS {
        registry.insert((*id).to_string(), anchor_body_contains_filepath(text, anchor));
    }
    for extra in ADDITIONAL_TOOL_ANCHORS {
        let found = text.contains(extra.anchor);
        assert!(found, "opencode binary: additional tool anchor for \"{}\" not found — {}", extra.id, extra.anchor);
        registry.insert(extra.id.to_string(), anchor_body_contains_filepath(text, extra.anchor));
    }
    assert!(
        registry.len() >= 15,
        "derived opencode tool registry found only {} id(s) — expected at least 15 (14 from the \
         Set literal plus \"lsp\"); the derivation likely broke: {:?}",
        registry.len(),
        registry.keys().collect::<Vec<_>>()
    );
    Ok(registry)
}

/// F4: `mapToolCall`'s own `pairs.len() >= 9` floor and the three-belt
/// parity test's "at least one tool routes to this hook" both only check
/// mapToolCall against ITSELF. This test instead derives the REGISTERED
/// tool inventory from the installed opencode binary and requires every
/// write- or read-capable one to be EITHER mapped by `mapToolCall` OR
/// documented, on its own line, as a named gap in discovery.md (F5's fixed
/// `discovery_doc_names_as_a_gap`, reused here rather than a second
/// document-global check) — the exact discipline that would have caught
/// `apply_patch` before oc-3, and now catches the next such tool by name.
#[test]
fn every_registered_write_or_read_capable_opencode_tool_is_mapped_or_named_as_a_gap() {
    let text = match opencode_binary_text() {
        Ok(t) => t,
        Err(reason) => {
            // Windows is a NAMED platform exclusion, not a convenience skip:
            // resolution there lands on a `.cmd` shim with no bundled payload
            // to derive from (see `resolve_opencode_binary`). The registry
            // this gate derives is platform-independent, and the gate stays
            // hard-failing on every other platform — including the CI lane
            // that runs on every commit — so the question it asks is still
            // answered on every change. Teaching resolution about Windows
            // package-manager shims is filed as its own work.
            let allow = env_allows_skip() || cfg!(windows);
            eprintln!(
                "{} (env-limited: could not read the installed opencode binary: {reason}) — \
                 every_registered_write_or_read_capable_opencode_tool_is_mapped_or_named_as_a_gap",
                if allow { "SKIP" } else { "FAIL" }
            );
            if allow {
                return;
            }
            panic!(
                "a readable, installed `opencode` binary is required to derive its real tool \
                 registry ({reason}) — refusing to report this test green with zero registry \
                 coverage actually derived. Set {ALLOW_SKIP_ENV}=1 to explicitly accept a \
                 degraded, unproven run in an environment with no opencode install."
            );
        }
    };

    let registry = derive_opencode_tool_registry(&text)
        .unwrap_or_else(|reason| panic!("opencode tool-registry derivation broke: {reason}"));

    let mapped: BTreeSet<String> = opencode_tool_hook_pairs().into_iter().map(|(tool, _)| tool).collect();

    // Tool ids this derivation positively confirmed do NOT touch an
    // arbitrary caller-supplied file path (no `filePath` found within their
    // own registration body — `anchor_body_contains_filepath` returned
    // `Some(false)`) never need bee coverage; they are excluded from the
    // gap requirement below without needing a discovery.md mention.
    let mut gaps: Vec<String> = Vec::new();
    for (id, filepath_evidence) in &registry {
        if mapped.contains(id) {
            continue; // covered live by part 1's per-row fixtures above
        }
        if *filepath_evidence == Some(false) {
            continue; // confirmed non-file-capable by this derivation itself
        }
        // Either confirmed file-capable (`Some(true)`, e.g. `lsp`) or
        // unclassifiable (`None`) — both REQUIRE coverage: either a mapping
        // (absent, or this branch would not run) or a named gap.
        if !discovery_doc_names_as_a_gap(id, &["NAMED GAP", "NAMED EXCLUSION", "Deferred"]) {
            gaps.push(format!(
                "\"{id}\": registered by the installed opencode binary, not mapped by \
                 mapToolCall, and not documented on its own line as a named gap in \
                 docs/history/opencode-support/discovery.md (filepath_evidence={filepath_evidence:?})"
            ));
        }
    }

    assert!(
        gaps.is_empty(),
        "silent write/read-capable OpenCode tool-registry gap(s) — the exact defect class \
         apply_patch was before oc-3 closed it:\n{}",
        gaps.join("\n")
    );
}
