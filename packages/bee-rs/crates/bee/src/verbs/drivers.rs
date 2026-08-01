// bee drivers — native port of the two porcelain DRIVER verbs: `bee dispatch
// prepare` (the worker-prompt / payload assembler) and `bee close` (the
// feature-close driver). Both are "drivers" in the porcelain sense: they own
// no store of their own, they compose other subsystems into one gesture.
//
// ─── Argv shapes served NATIVELY ───────────────────────────────────────────
//
//   close --feature <F> [--json]
//   close --feature <F> --dry-run [--json]
//   dispatch prepare --runtime <claude|codex> --kind <cell|gather|reviewer|advisor>
//                    [--cell <C>] [--worker <W>] [--force-ownership] [--json]
//
// Everything else returns None BEFORE any output, any lock, and any write, and
// the whole command re-runs under Node (campaign rule 1 — conservative argv
// routing; the dispatcher's validate()/nearest-match machinery is never
// reproduced here).
//
// ─── Shapes DELEGATED, and why ─────────────────────────────────────────────
//
//   * `dispatch prepare --claim` (the claim+reserve+prompt gesture).
//     bee.mjs's claimAndReserveForDispatch composes TWO mutating doors that
//     are already ported natively but live PRIVATE inside their own modules:
//     claimCellFromFlags (verbs/cells.rs `run_claim` — write-policy
//     shared-disjoint lease pre-check, lane-record gate resolution, the
//     O_EXCL claims-store protocol with fence epochs and session adoption,
//     per-cell store lock, budget checks) and reservePathAtomic /
//     releaseReservationsForAgent (verbs/reservations.rs — control-root
//     resolution, cross-worktree foreign-hold scan, TTL leases). Re-deriving
//     ~1.5k lines of proven MUTATING store code into a third file would fork
//     the store-mutation logic in two places, which is exactly the drift
//     contract C1 exists to prevent; and a pre-flight-then-mutate shape
//     cannot honour campaign rule 2 ("a refusal reached AFTER a lock attempt
//     must be native") without re-deriving every refusal anyway. So --claim
//     returns None before touching anything. NOTE: the PRODUCT of --claim —
//     the assembled worker prompt, including the machine-assembled
//     Learned-context and Prior-rounds blocks — is byte-for-byte the same
//     string this file builds for the non-claim `--kind cell` shape (bee.mjs
//     sequences the claim BEFORE the payload build and never feeds it back
//     into prepareDispatch), so the prompt itself is covered natively and
//     byte-diffed. R6 debt: re-export cells.rs's claim door and
//     reservations.rs's reserve door as pub(crate) and finish this branch.
//
//   * `dispatch prepare --runtime codex` on a host that HAS a
//     `.bee/native-transport-probe.json` whose `schema` matches. Beyond that
//     point readNativeTransportClassification shells out to `codex --version`
//     and `codex features list` and hashes ~/.codex config scope — process
//     probes with no native port. An ABSENT / unparseable / schema-mismatched
//     probe record short-circuits to `native_budget_only` with no subprocess
//     at all, which IS ported (the overwhelmingly common host shape).
//
//   * `dispatch prepare --stdin`-family and every unknown/missing/invalid
//     flag, `--help` anywhere, non-UTF-8 argv, stray positionals: Node's
//     validate()/emitError machinery owns those bytes.
//
//   * `close --feature <F>` when `.bee/lanes/<F>.json` exists — the
//     blueprint's lane coverage debt. That record's own `last_scribing_run`
//     joins scribingDebt's threshold and a corrupt one prints a console.warn;
//     both are unported. Scoped to the ONE named feature, because readLane
//     touches exactly that one file: a lane-using repo still closes every
//     other feature natively. (Workflow records are NOT a guard — nothing on
//     close's read path consults `.bee/runtime/workflows/`.)
//
//   * Linked-worktree roots (crate::roots -> NeedsNode). This also makes
//     bee.mjs's `grantedWorktreeContext()` provably `null` on every shape this
//     file serves, so close's merge-back line is never rendered natively —
//     the granted-worktree branch is Node's.
//
//   * Corrupt JSON anywhere on a read path (Node's readJson warns with the V8
//     parse message), `dogfood_repos` entries (normalizeDogfoodRepos
//     console.warns per dead repo), a configured non-empty `product_root`
//     (repo-divorce topology), and every V8-only shape the lifted knowledge
//     helpers already flag (see their provenance banner below).
//
//   * `close` without a POSIX shell on win32 (Node falls back to cmd.exe).
//
// ─── Provenance (every ported lib function) ────────────────────────────────
//
//   bee.mjs            handleDispatchPrepare (~7495), handleClose (~7643),
//                      renderTestCommandLines (~7601), buildCloseReportDoors,
//                      renderCloseDoorLines, CLOSE_TESTS_UNDECLARED_DETAIL,
//                      requireFlag, grantedWorktreeContext,
//                      readNativeTransportClassification (delegating slice),
//                      nativeTransportProbePath, doctorSafeReadJson,
//                      emit/emitError/main's dispatch frame.
//   lib/dispatch-prepare.mjs
//                      DISPATCH_RUNTIMES/DISPATCH_KINDS, slotForKind,
//                      purposeForKind, checkCellClaimOwnership, oneLine,
//                      PRIOR_ROUNDS_MAX_EVENT_LINES, priorRoundEventLines,
//                      LEARNED_CONTEXT_MAX_LINES, bundleLearnedLines,
//                      learnedContextLines, cellPromptBody, promptBodyFor,
//                      appendPrepareRecord, prepareDispatch.
//   lib/prompt-renderer.mjs
//                      loadPrompt, render (the C4 byte-identity pin).
//   lib/dispatch-guard.mjs
//                      PINNED_AGENT_TYPE, deriveEconomics,
//                      NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE,
//                      NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY.
//   lib/state.mjs      DEFAULT_MODELS, EFFORT_LEVELS, RUNTIMES,
//                      CONFIGURABLE_SLOTS, MODEL_NORMALIZE_SLOTS,
//                      normalizeTierValue, normalizeModels, nativeResolved,
//                      resolveTier, resolveAdvisor, readState (the slice
//                      scribingDebt consumes), readConfig (via
//                      crate::state::read_config_raw).
//   lib/test-runner.mjs
//                      declaredTestCommands, runDeclaredTests,
//                      spawnDeclaredCommand, posixShell, firstFailureLine,
//                      TEST_RESULTS_RELATIVE, FAILURE_EXCERPT_MAX_CHARS.
//   lib/cells.mjs      readCell, ID_PATTERN, ARCHIVE_DIR_NAME, listCells,
//                      scribingDebt, bestScribingStampMs,
//                      scribingRunStampMs, readScribingLedger.
//   lib/capture.mjs    captureQueuePath, pendingCaptureStubs, captureQueue.
//   lib/knowledge.mjs  bundleDir, bundleMode, collectConcepts,
//                      buildContextManifest (+ its whole ranking closure),
//                      KNOWLEDGE_CONTEXT_LANE_BUDGETS/DEFAULT_BUDGET.
//
// ─── Re-derived Rust (the "may not edit that file" rule) ───────────────────
//
// The knowledge-context builder is already ported in verbs/knowledge.rs but
// every function it needs is PRIVATE to that module. The `kctx` module below
// is a verbatim lift of that port (see its own banner for the exact line
// ranges) — not a re-implementation — so the two cannot semantically drift;
// `learned_context_agrees_with_the_knowledge_verb_port` pins them to the same
// answer on a fixture. Same for the test runner (verbs/test_runner.rs) and
// readCell/listCells (verbs/cells.rs), each re-derived here with a per-
// function provenance line naming BOTH the .mjs source and the Rust port.
//
// ─── Documented divergences ────────────────────────────────────────────────
//
//   * `dispatch_id` is crypto.randomUUID() in Node; here it is
//     crate::verbs::reservations::pseudo_uuid_v4 (the same v4 SHAPE, a
//     different entropy source — already the campaign's convention, and the
//     twin-diff harness masks ids).
//   * A test-record write failure after close's run reports Rust's io message
//     where Node reports V8's (the .bee/logs dir is pre-flighted, so this is
//     a hard-race-only path). Same for a dispatch.jsonl append failure —
//     except that one is fail-open on BOTH sides, so it is unobservable.
//   * Node's spawnSync 64 MiB maxBuffer kill and its 10s shell probe timeout
//     are not replicated (inherited from verbs/test_runner.rs).
//   * The prompt templates are COMPILED IN (include_str!) rather than read off
//     disk. A runtime skew guard byte-compares the embedded template against
//     the on-disk one whenever the repo ships prompts (canonical
//     `packages/bee/prompts/` or vendored `.bee/bin/prompts/`) and delegates
//     on any mismatch — the same "lib skew ⇒ delegate" discipline R2's
//     write-guard uses, so a prompt can never render from stale bytes.

use crate::fsutil::{ensure_dir, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::roots::{resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::verbs::reservations::{
    finish, js_is_ws, parse_flags, prelude, pseudo_uuid_v4, truthy, FlagV, Flags, Out, Pre, R2,
};
use crate::verbs::emit_no_root_error;
use serde_json::{Map, Number, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

/// This argv/repo/store shape belongs to the Node runtime.
#[derive(Debug)]
struct Delegate;
type D<T> = Result<T, Delegate>;

impl From<Delegate> for crate::verbs::reservations::Err2 {
    fn from(_: Delegate) -> Self {
        crate::verbs::reservations::Err2::Ex
    }
}
impl From<crate::state::Bail> for Delegate {
    fn from(_: crate::state::Bail) -> Self {
        Delegate
    }
}

// ═══ JS string primitives ══════════════════════════════════════════════════

/// provenance: JS String.prototype.trim — same set verbs/test_runner.rs's
/// js_trim uses (ECMA WhiteSpace ∪ LineTerminator, incl. U+FEFF/NBSP).
fn js_trim(s: &str) -> &str {
    s.trim_matches(js_is_ws)
}

/// JS template-literal coercion for a possibly-absent field.
fn tpl(v: Option<&Value>) -> String {
    match v {
        None => "undefined".to_string(),
        Some(v) => jsjson::js_to_string(v),
    }
}

/// JS `typeof v === 'string' && v` — the truthy-string idiom.
fn truthy_str(v: Option<&Value>) -> Option<&str> {
    match v {
        Some(Value::String(s)) if !s.is_empty() => Some(s),
        _ => None,
    }
}

/// JS `a === b` over JSON primitives (`None` models `undefined`).
fn strict_eq(a: Option<&Value>, b: Option<&Value>) -> bool {
    match (a, b) {
        (None, None) => true,
        (None, _) | (_, None) => false,
        (Some(x), Some(y)) => match (x, y) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(p), Value::Bool(q)) => p == q,
            (Value::Number(p), Value::Number(q)) => p.as_f64() == q.as_f64(),
            (Value::String(p), Value::String(q)) => p == q,
            _ => false,
        },
    }
}

fn vget<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    v.as_object().and_then(|m| m.get(key))
}

/// JS `.slice(0, n)` over UTF-16 code units. A boundary that would split a
/// surrogate pair drops the pair rather than emitting a lone high surrogate
/// (no Rust String can hold one) — the mirror of test_runner.rs's utf16_tail
/// divergence note, unreachable for ASCII/BMP prose.
fn utf16_head(s: &str, n: usize) -> String {
    let units: Vec<u16> = s.encode_utf16().collect();
    if units.len() <= n {
        return s.to_string();
    }
    let mut end = n;
    if (0xD800..=0xDBFF).contains(&units[end - 1]) {
        end -= 1;
    }
    String::from_utf16_lossy(&units[..end])
}

fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

/// provenance: dispatch-prepare.mjs oneLine(text, max = 140) —
/// `String(text ?? '').replace(/\s+/g, ' ').trim()`, ellipsised at `max`
/// UTF-16 units with a 3-unit "..." tail.
fn one_line(text: Option<&Value>, max: usize) -> String {
    let raw = match text {
        None | Some(Value::Null) => String::new(),
        Some(v) => jsjson::js_to_string(v),
    };
    // /\s+/g -> ' '
    let mut collapsed = String::with_capacity(raw.len());
    let mut in_ws = false;
    for c in raw.chars() {
        if js_is_ws(c) {
            in_ws = true;
        } else {
            if in_ws {
                collapsed.push(' ');
                in_ws = false;
            }
            collapsed.push(c);
        }
    }
    if in_ws {
        collapsed.push(' ');
    }
    let flat = js_trim(&collapsed).to_string();
    if utf16_len(&flat) > max {
        format!("{}...", utf16_head(&flat, max - 3))
    } else {
        flat
    }
}

// ═══ prompt renderer (lib/prompt-renderer.mjs — contract C4) ═══════════════
//
// The templates are embedded at BUILD time from the canonical checkout, the
// same way crate::registry embeds the command-manifest payload: both runtimes
// then hash/render identical bytes. loadPrompt's normalization (CRLF -> LF,
// ONE trailing newline stripped) is applied here at load, exactly as Node
// applies it after readFileSync.

const PROMPT_WORKER_CELL: &str = include_str!("../../../../../bee/prompts/worker-cell.md");
const PROMPT_GATHER: &str = include_str!("../../../../../bee/prompts/gather.md");
const PROMPT_REVIEWER: &str = include_str!("../../../../../bee/prompts/reviewer.md");
const PROMPT_ADVISOR: &str = include_str!("../../../../../bee/prompts/advisor.md");

fn embedded_prompt(name: &str) -> Option<&'static str> {
    match name {
        "worker-cell" => Some(PROMPT_WORKER_CELL),
        "gather" => Some(PROMPT_GATHER),
        "reviewer" => Some(PROMPT_REVIEWER),
        "advisor" => Some(PROMPT_ADVISOR),
        _ => None,
    }
}

/// provenance: prompt-renderer.mjs loadPrompt — CRLF normalized to LF and ONE
/// trailing newline stripped.
fn normalize_template(raw: &str) -> String {
    let lf = raw.replace("\r\n", "\n");
    match lf.strip_suffix('\n') {
        Some(s) => s.to_string(),
        None => lf,
    }
}

fn load_prompt(name: &str) -> Option<String> {
    embedded_prompt(name).map(normalize_template)
}

/// Runtime skew guard (R2 write-guard discipline). The Node renderer resolves
/// prompts RELATIVE TO ITS OWN MODULE — `packages/bee/prompts/` for a
/// canonical checkout, `.bee/bin/prompts/` for a vendored engine. Whenever a
/// repo ships either directory, its bytes must equal the compiled-in bytes or
/// this port would render a stale template: byte-compare and delegate on any
/// mismatch. A repo shipping neither (a pure-binary install) trusts the
/// embedded copy, which is the only copy that exists there.
fn prompts_match_disk(root: &Path, name: &str) -> bool {
    let embedded = match embedded_prompt(name) {
        Some(t) => t,
        None => return false,
    };
    let candidates = [
        root.join("packages").join("bee").join("prompts").join(format!("{name}.md")),
        root.join(".bee").join("bin").join("prompts").join(format!("{name}.md")),
    ];
    for file in candidates {
        let Ok(bytes) = std::fs::read(&file) else { continue };
        let disk = String::from_utf8_lossy(&bytes);
        if normalize_template(&disk) != normalize_template(embedded) {
            return false;
        }
    }
    true
}

/// provenance: prompt-renderer.mjs render(template, vars) — the whole minimal
/// grammar, byte-faithful:
///   * `\n{{#if NAME}}<inner>\n{{/if}}` blocks are consumed WITH the newline
///     that precedes the opening marker; a truthy var splices `<inner>` in
///     verbatim (substitution still runs over it afterwards), a falsy var
///     leaves zero residue bytes. Non-greedy `[\s\S]*?` == first following
///     `\n{{/if}}`.
///   * a `{{#if ` inside an inner block is a loud refusal (nesting), as is any
///     surviving `{{#if `/`{{/if}}` after the block pass.
///   * `{{NAME}}` placeholders substitute String(vars[NAME]); an
///     undefined/null value is a loud refusal.
/// Names are `[A-Za-z0-9_]+` in both markers.
fn render(template: &str, vars: &[(&str, &str)]) -> Result<String, String> {
    let lookup = |name: &str| -> Option<&str> {
        vars.iter().find(|(k, _)| *k == name).map(|(_, v)| *v)
    };

    // ── pass 1: conditional blocks ────────────────────────────────────────
    let mut out = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // Match /\n\{\{#if ([A-Za-z0-9_]+)\}\}/ at i.
        let Some(rest) = template.get(i..) else {
            out.push_str(&template[i..]);
            break;
        };
        if !rest.starts_with("\n{{#if ") {
            let ch = template[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }
        let name_start = i + "\n{{#if ".len();
        let name_end = match template[name_start..].find("}}") {
            Some(p) => name_start + p,
            None => {
                let ch = template[i..].chars().next().unwrap();
                out.push(ch);
                i += ch.len_utf8();
                continue;
            }
        };
        let name = &template[name_start..name_end];
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            let ch = template[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }
        let inner_start = name_end + 2;
        let Some(close_rel) = template[inner_start..].find("\n{{/if}}") else {
            let ch = template[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
            continue;
        };
        let inner = &template[inner_start..inner_start + close_rel];
        if inner.contains("{{#if ") {
            return Err(format!(
                "prompt-renderer: nested {{{{#if}}}} inside block \"{name}\" — nesting is not supported."
            ));
        }
        // JS truthiness of the substituted var: only a non-empty string here
        // (dispatch-prepare passes joined line lists), undefined -> falsy.
        if lookup(name).map(|v| !v.is_empty()).unwrap_or(false) {
            out.push_str(inner);
        }
        i = inner_start + close_rel + "\n{{/if}}".len();
    }
    if out.contains("{{#if ") || out.contains("{{/if}}") {
        return Err("prompt-renderer: unmatched or malformed {{#if}}/{{/if}} marker in template.".to_string());
    }

    // ── pass 2: {{name}} substitution (a substituted value is never
    //    re-scanned — the scan walks the PRE-substitution text) ────────────
    let mut result = String::with_capacity(out.len());
    let mut i = 0usize;
    while i < out.len() {
        let rest = &out[i..];
        if let Some(after) = rest.strip_prefix("{{") {
            if let Some(close) = after.find("}}") {
                let name = &after[..close];
                if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    match lookup(name) {
                        Some(v) => result.push_str(v),
                        None => {
                            return Err(format!(
                                "prompt-renderer: no value supplied for placeholder {{{{{name}}}}}."
                            ))
                        }
                    }
                    i += 2 + close + 2;
                    continue;
                }
            }
        }
        let ch = rest.chars().next().unwrap();
        result.push(ch);
        i += ch.len_utf8();
    }
    Ok(result)
}

// ═══ models config (lib/state.mjs) ═════════════════════════════════════════

const EFFORT_LEVELS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];
const RUNTIMES: [&str; 2] = ["claude", "codex"];
/// CONFIGURABLE_SLOTS = [...CONFIGURABLE_TIERS, 'review'].
const CONFIGURABLE_SLOTS: [&str; 3] = ["extraction", "generation", "review"];
/// MODEL_NORMALIZE_SLOTS = [...CONFIGURABLE_SLOTS, 'advisor'].
const MODEL_NORMALIZE_SLOTS: [&str; 4] = ["extraction", "generation", "review", "advisor"];

/// provenance: state.mjs DEFAULT_MODELS.
fn default_models(runtime: &str) -> Map<String, Value> {
    let mut m = Map::new();
    if runtime == "claude" {
        m.insert("extraction".into(), Value::String("haiku".into()));
        m.insert("generation".into(), Value::String("sonnet".into()));
        m.insert("review".into(), Value::String("opus".into()));
    } else {
        m.insert("extraction".into(), Value::Null);
        m.insert("generation".into(), Value::Null);
        m.insert("review".into(), Value::Null);
    }
    m
}

fn is_plain_object(v: &Value) -> bool {
    matches!(v, Value::Object(_))
}

/// provenance: state.mjs normalizeTierValue. `None` == JS `undefined` (the
/// slot keeps its default); `Some(Value::Null)` == an explicit null slot.
fn normalize_tier_value(value: Option<&Value>) -> Option<Value> {
    let value = value?;
    match value {
        Value::String(s) if !js_trim(s).is_empty() => {
            return Some(Value::String(js_trim(s).to_string()))
        }
        Value::String(_) => return None,
        Value::Null => return Some(Value::Null),
        v if !is_plain_object(v) => return None,
        _ => {}
    }
    let obj = value.as_object().unwrap();
    // { kind: 'cli', command }
    if matches!(obj.get("kind"), Some(Value::String(k)) if k == "cli") {
        if let Some(Value::String(cmd)) = obj.get("command") {
            if !js_trim(cmd).is_empty() {
                let mut out = Map::new();
                out.insert("kind".into(), Value::String("cli".into()));
                out.insert("command".into(), Value::String(js_trim(cmd).to_string()));
                return Some(Value::Object(out));
            }
        }
    }
    // { kind: 'native', model, effort?, fork_turns?, agent_type? }
    if matches!(obj.get("kind"), Some(Value::String(k)) if k == "native") {
        if let Some(Value::String(model)) = obj.get("model") {
            if !js_trim(model).is_empty() {
                let mut out = Map::new();
                out.insert("kind".into(), Value::String("native".into()));
                out.insert("model".into(), Value::String(js_trim(model).to_string()));
                if let Some(Value::String(e)) = obj.get("effort") {
                    if EFFORT_LEVELS.contains(&js_trim(e)) {
                        out.insert("effort".into(), Value::String(js_trim(e).to_string()));
                    }
                }
                if let Some(Value::String(f)) = obj.get("fork_turns") {
                    if js_trim(f) == "none" {
                        out.insert("fork_turns".into(), Value::String("none".into()));
                    }
                }
                if let Some(Value::String(a)) = obj.get("agent_type") {
                    if !js_trim(a).is_empty() {
                        out.insert("agent_type".into(), Value::String(js_trim(a).to_string()));
                    }
                }
                return Some(Value::Object(out));
            }
        }
    }
    // Explicit-fallback composite: { primary: {kind:'native', model}, ... }
    if let Some(primary) = obj.get("primary") {
        if is_plain_object(primary) {
            let p = primary.as_object().unwrap();
            let native_primary = matches!(p.get("kind"), Some(Value::String(k)) if k == "native")
                && matches!(p.get("model"), Some(Value::String(m)) if !js_trim(m).is_empty());
            if native_primary {
                let mut out = Map::new();
                out.insert("primary".into(), normalize_tier_value(Some(primary))?);
                if matches!(obj.get("fallback_policy"), Some(Value::String(s)) if s == "explicit-only") {
                    out.insert("fallback_policy".into(), Value::String("explicit-only".into()));
                    if let Some(fb) = obj.get("fallback") {
                        if is_plain_object(fb) {
                            let f = fb.as_object().unwrap();
                            let cli = matches!(f.get("kind"), Some(Value::String(k)) if k == "cli");
                            if let (true, Some(Value::String(cmd))) = (cli, f.get("command")) {
                                if !js_trim(cmd).is_empty() {
                                    let mut fbo = Map::new();
                                    fbo.insert("kind".into(), Value::String("cli".into()));
                                    fbo.insert(
                                        "command".into(),
                                        Value::String(js_trim(cmd).to_string()),
                                    );
                                    out.insert("fallback".into(), Value::Object(fbo));
                                }
                            }
                        }
                    }
                }
                return Some(Value::Object(out));
            }
        }
    }
    // { model, effort? } — only when `kind` is absent.
    if obj.get("kind").is_none() {
        if let Some(Value::String(model)) = obj.get("model") {
            if !js_trim(model).is_empty() {
                let mut out = Map::new();
                out.insert("model".into(), Value::String(js_trim(model).to_string()));
                if let Some(Value::String(e)) = obj.get("effort") {
                    if EFFORT_LEVELS.contains(&js_trim(e)) {
                        out.insert("effort".into(), Value::String(js_trim(e).to_string()));
                    }
                }
                return Some(Value::Object(out));
            }
        }
    }
    None
}

/// provenance: state.mjs normalizeModels — defaults per runtime, overlaid by
/// the normalized value of each MODEL_NORMALIZE_SLOTS entry.
fn normalize_models(raw: Option<&Value>) -> Map<String, Value> {
    let mut out = Map::new();
    for rt in RUNTIMES {
        out.insert(rt.to_string(), Value::Object(default_models(rt)));
    }
    if let Some(raw) = raw {
        if is_plain_object(raw) {
            for rt in RUNTIMES {
                let Some(src) = raw.get(rt) else { continue };
                if !is_plain_object(src) {
                    continue;
                }
                for slot in MODEL_NORMALIZE_SLOTS {
                    if let Some(value) = normalize_tier_value(src.get(slot)) {
                        out.get_mut(rt)
                            .and_then(Value::as_object_mut)
                            .unwrap()
                            .insert(slot.to_string(), value);
                    }
                }
            }
        }
    }
    out
}

/// The `models` slice of readConfig(root). Delegates on the two readConfig
/// side effects this port does not reproduce: a corrupt config file (Node's
/// readJson V8 warning) and normalizeDogfoodRepos' per-dead-repo console.warn.
fn read_models(root: &Path) -> D<Map<String, Value>> {
    let config = read_config_raw(root)?;
    if let Some(Value::Array(items)) = config.get("dogfood_repos") {
        if !items.is_empty() {
            return Err(Delegate); // normalizeDogfoodRepos may warn to stderr
        }
    }
    Ok(normalize_models(config.get("models")))
}

/// provenance: state.mjs resolveTier / resolveAdvisor return shapes.
#[derive(Clone, Debug, PartialEq)]
enum Resolved {
    Inherit,
    Model {
        model: String,
        effort: Option<String>,
    },
    Budget,
    Cli {
        command: String,
    },
    Native {
        model: String,
        effort: Option<String>,
        fork_turns: String,
        agent_type: String,
        fallback: Option<String>,
    },
    Refused {
        slot: String,
    },
}

const CLI_REFUSAL_FIX: &str = "declare {for:\"gather\"} for a read-only gather; cli cell execution stays refused until a cell-execution dogfood is green (plan 2A/W9)";

/// provenance: state.mjs nativeResolved — normalize already trimmed/validated
/// the leaf; this only applies the resolved defaults.
fn native_resolved(value: &Map<String, Value>, fallback: Option<String>) -> Resolved {
    Resolved::Native {
        model: match value.get("model") {
            Some(Value::String(s)) => s.clone(),
            other => tpl(other),
        },
        effort: match value.get("effort") {
            None | Some(Value::Null) => None,
            Some(v) => Some(jsjson::js_to_string(v)),
        },
        fork_turns: match value.get("fork_turns") {
            None | Some(Value::Null) => "none".to_string(),
            Some(v) => jsjson::js_to_string(v),
        },
        agent_type: match value.get("agent_type") {
            None | Some(Value::Null) => "worker".to_string(),
            Some(v) => jsjson::js_to_string(v),
        },
        fallback,
    }
}

/// The composite `{primary, fallback_policy:'explicit-only', fallback}` arm
/// shared by resolveTier and resolveAdvisor.
fn composite_resolved(obj: &Map<String, Value>) -> Option<Resolved> {
    let primary = obj.get("primary")?;
    if !is_plain_object(primary) {
        return None;
    }
    let mut fallback = None;
    if matches!(obj.get("fallback_policy"), Some(Value::String(s)) if s == "explicit-only") {
        if let Some(fb) = obj.get("fallback") {
            if matches!(fb.get("kind"), Some(Value::String(k)) if k == "cli") {
                if let Some(Value::String(cmd)) = fb.get("command") {
                    fallback = Some(cmd.clone());
                }
            }
        }
    }
    Some(native_resolved(primary.as_object().unwrap(), fallback))
}

/// provenance: state.mjs resolveTier(root, slot, runtime, purpose). `slot`
/// here is always a CONFIGURABLE_SLOTS member or 'advisor' (coerced to
/// 'generation' exactly like Node); `for_gather` is purposeForKind's verdict.
fn resolve_tier(
    models: &Map<String, Value>,
    slot: &str,
    runtime: &str,
    for_gather: bool,
) -> Resolved {
    if slot == "ceiling" {
        return Resolved::Inherit;
    }
    let rt = if RUNTIMES.contains(&runtime) { runtime } else { "claude" };
    let s = if CONFIGURABLE_SLOTS.contains(&slot) { slot } else { "generation" };
    let table = models.get(rt);
    let mut value = table.and_then(|t| t.get(s)).cloned();
    if matches!(value, None | Some(Value::Null)) && s == "review" {
        value = table.and_then(|t| t.get("generation")).cloned();
    }
    let Some(value) = value.filter(|v| !v.is_null()) else {
        return Resolved::Budget;
    };
    if let Value::String(model) = &value {
        return Resolved::Model { model: model.clone(), effort: None };
    }
    let Some(obj) = value.as_object() else { return Resolved::Budget };
    if matches!(obj.get("kind"), Some(Value::String(k)) if k == "cli") {
        if !for_gather {
            return Resolved::Refused { slot: s.to_string() };
        }
        return Resolved::Cli {
            command: truthy_str(obj.get("command")).unwrap_or_default().to_string(),
        };
    }
    if matches!(obj.get("kind"), Some(Value::String(k)) if k == "native") {
        return native_resolved(obj, None);
    }
    if let Some(r) = composite_resolved(obj) {
        return r;
    }
    if let Some(Value::String(model)) = obj.get("model") {
        return Resolved::Model {
            model: model.clone(),
            effort: match obj.get("effort") {
                Some(v) if truthy(v) => Some(jsjson::js_to_string(v)),
                _ => None,
            },
        };
    }
    Resolved::Budget
}

/// provenance: state.mjs resolveAdvisor — NEVER budget, NEVER a tier fallback;
/// `None` unambiguously means "no advisor".
fn resolve_advisor(models: &Map<String, Value>, runtime: &str) -> Option<Resolved> {
    let rt = if RUNTIMES.contains(&runtime) { runtime } else { "claude" };
    let value = models.get(rt).and_then(|t| t.get("advisor"))?;
    if value.is_null() {
        return None;
    }
    if let Value::String(model) = value {
        return Some(Resolved::Model { model: model.clone(), effort: None });
    }
    let obj = value.as_object()?;
    if matches!(obj.get("kind"), Some(Value::String(k)) if k == "cli") {
        return Some(Resolved::Cli {
            command: match obj.get("command") {
                Some(Value::String(c)) => c.clone(),
                _ => return None, // `{type:'cli', command: undefined}` never reaches here post-normalize
            },
        });
    }
    if matches!(obj.get("kind"), Some(Value::String(k)) if k == "native") {
        return Some(native_resolved(obj, None));
    }
    if let Some(r) = composite_resolved(obj) {
        return Some(r);
    }
    if let Some(Value::String(model)) = obj.get("model") {
        return Some(Resolved::Model {
            model: model.clone(),
            effort: match obj.get("effort") {
                Some(v) if truthy(v) => Some(jsjson::js_to_string(v)),
                _ => None,
            },
        });
    }
    None
}

// ═══ dispatch-guard.mjs (the enforcement vocabulary) ═══════════════════════

const NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE: &str = "native_model_override";
const NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY: &str = "native_budget_only";

/// provenance: dispatch-guard.mjs PINNED_AGENT_TYPE (W3 pinned-type rule).
fn pinned_agent_type(tier: &str) -> &'static str {
    match tier {
        "generation" => "bee-gather",
        "extraction" => "bee-extract",
        "review" => "bee-review",
        _ => "general-purpose", // `PINNED_AGENT_TYPE[tier] || 'general-purpose'`
    }
}

/// provenance: dispatch-guard.mjs deriveEconomics — the ONE honest
/// pinned/unverified/inherited-or-unknown/native-requested split. Key order is
/// frozen: {logical_tier, requested_model, effective_model,
/// effective_model_status, channel, enforcement}.
fn derive_economics(
    channel: &str,
    tier: &str,
    param_model: Option<&str>,
    resolved: &Resolved,
    native_confirmed: bool,
) -> Map<String, Value> {
    let is_native_confirmed =
        channel == "codex-native" && matches!(resolved, Resolved::Native { .. }) && native_confirmed;
    let resolved_model: Option<String> = match resolved {
        Resolved::Model { model, .. } | Resolved::Native { model, .. } => Some(model.clone()),
        _ => None,
    };

    let enforcement = if channel == "cli-exec" {
        "cli-command"
    } else if is_native_confirmed {
        "native-model-param"
    } else if channel == "codex-native" {
        "prompt-budget"
    } else if param_model.is_some() {
        "model-param"
    } else {
        "prompt-budget"
    };

    let mut effective_model = Value::Null;
    let effective_model_status = if is_native_confirmed {
        "native-requested"
    } else if channel == "codex-native" {
        "inherited-or-unknown"
    } else if channel == "cli-exec" {
        "unverified"
    } else if let Some(pm) = param_model {
        effective_model = Value::String(pm.to_string());
        "pinned"
    } else {
        "unverified"
    };

    let requested_model = if channel == "cli-exec" {
        Value::Null
    } else {
        match param_model.map(str::to_string).or(resolved_model) {
            Some(m) => Value::String(m),
            None => Value::Null,
        }
    };

    let mut out = Map::new();
    out.insert("logical_tier".into(), Value::String(tier.to_string()));
    out.insert("requested_model".into(), requested_model);
    out.insert("effective_model".into(), effective_model);
    out.insert(
        "effective_model_status".into(),
        Value::String(effective_model_status.to_string()),
    );
    out.insert("channel".into(), Value::String(channel.to_string()));
    out.insert("enforcement".into(), Value::String(enforcement.to_string()));
    out
}

// ═══ cells (lib/cells.mjs; Rust port: verbs/cells.rs) ══════════════════════

fn cells_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("cells")
}
/// provenance: cells.mjs ARCHIVE_DIR_NAME (verbs/cells.rs:330).
const ARCHIVE_DIR_NAME: &str = "archive";

/// provenance: cells.mjs ID_PATTERN /^[A-Za-z0-9][A-Za-z0-9._-]*$/
/// (verbs/cells.rs:333 id_pattern_ok).
fn id_pattern_ok(id: &str) -> bool {
    let mut chars = id.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

/// provenance: fsutil.mjs readJson(file, null) (verbs/cells.rs:347
/// read_cell_json) — corrupt is Node's V8-warning path, so it delegates.
fn rj(file: &Path) -> D<Option<Value>> {
    match read_json(file) {
        ReadJson::Missing => Ok(None),
        ReadJson::Corrupt => Err(Delegate),
        ReadJson::Parsed(Value::Null) => Ok(None),
        ReadJson::Parsed(v) => Ok(Some(v)),
    }
}

/// provenance: cells.mjs readCell (verbs/cells.rs:419 read_cell) — the active
/// file wins, then every `.bee/cells/archive/<feature>/` dir in readdir order.
fn read_cell(root: &Path, id: &str) -> D<Option<Value>> {
    if id.is_empty() || !id_pattern_ok(id) {
        return Ok(None);
    }
    if let Some(v) = rj(&cells_dir(root).join(format!("{id}.json")))? {
        return Ok(Some(v));
    }
    let Ok(entries) = std::fs::read_dir(cells_dir(root).join(ARCHIVE_DIR_NAME)) else {
        return Ok(None);
    };
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        if let Some(v) = rj(&entry.path().join(format!("{id}.json")))? {
            return Ok(Some(v));
        }
    }
    Ok(None)
}

/// provenance: cells.mjs listCells(root, {feature, status}) — the active scan
/// only (verbs/status_full.rs:1571 list_cells). The sort is LOAD-BEARING here:
/// scribingDebt maps the result to ids and close joins them into the
/// scribing-debt door detail, so the order reaches an emitted byte (caught by
/// a live diff against the beehive repo itself, where a plain byte sort put
/// "rust-port-5" after "rust-port-23").
fn list_cells(root: &Path, feature: &str, status: &str) -> D<Vec<Value>> {
    let mut cells: Vec<Value> = Vec::new();
    let Ok(entries) = std::fs::read_dir(cells_dir(root)) else {
        return Ok(cells);
    };
    for entry in entries.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !name.ends_with(".json") {
            continue;
        }
        let Some(cell) = rj(&entry.path())? else { continue };
        if !matches!(cell, Value::Object(_) | Value::Array(_)) {
            continue; // `typeof cell !== 'object'`
        }
        if !matches!(vget(&cell, "feature"), Some(Value::String(f)) if f == feature) {
            continue;
        }
        if !matches!(vget(&cell, "status"), Some(Value::String(s)) if s == status) {
            continue;
        }
        cells.push(cell);
    }
    cells.sort_by(|a, b| locale_cmp(&tpl(vget(a, "id")), &tpl(vget(b, "id")), true));
    Ok(cells)
}

// ─── String.prototype.localeCompare('en', {numeric:true}) ──────────────────
//
// VERBATIM LIFT of verbs/status_full.rs:429-503 (char_class_key + locale_cmp),
// whose own provenance is the measured V8/ICU behavior on the id/feature
// alphabet ([A-Za-z0-9._-] plus ISO timestamps):
//   primary:  class order _ < - < . < (other punct) < digits < letters
//             (letters case-folded; numeric mode compares digit runs BY VALUE,
//              so "01" == "1" with no length tiebreak, matching ICU)
//   tertiary: first case difference, lowercase before uppercase.
// R6 debt: promote to a shared module alongside the kctx lift.

fn char_class_key(c: char) -> (u8, u32) {
    if c.is_whitespace() {
        return (0, c as u32);
    }
    match c {
        '_' => (1, 0),
        '-' => (1, 1),
        ',' => (1, 2),
        ';' => (1, 3),
        ':' => (1, 4),
        '!' => (1, 5),
        '?' => (1, 6),
        '.' => (1, 7),
        _ if c.is_ascii_digit() => (2, c as u32 - '0' as u32),
        _ if c.is_alphabetic() => (3, c.to_lowercase().next().unwrap_or(c) as u32),
        _ => (1, 100 + c as u32),
    }
}

fn locale_cmp(a: &str, b: &str, numeric: bool) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let av: Vec<char> = a.chars().collect();
    let bv: Vec<char> = b.chars().collect();
    let (mut i, mut j) = (0usize, 0usize);
    while i < av.len() && j < bv.len() {
        let (ca, cb) = (av[i], bv[j]);
        if numeric && ca.is_ascii_digit() && cb.is_ascii_digit() {
            let si = i;
            while i < av.len() && av[i].is_ascii_digit() {
                i += 1;
            }
            let sj = j;
            while j < bv.len() && bv[j].is_ascii_digit() {
                j += 1;
            }
            let ra: String = av[si..i].iter().collect();
            let rb: String = bv[sj..j].iter().collect();
            let ta = ra.trim_start_matches('0');
            let tb = rb.trim_start_matches('0');
            let ord = ta.len().cmp(&tb.len()).then_with(|| ta.cmp(tb));
            if ord != Ordering::Equal {
                return ord;
            }
            continue;
        }
        let ord = char_class_key(ca).cmp(&char_class_key(cb));
        if ord != Ordering::Equal {
            return ord;
        }
        i += 1;
        j += 1;
    }
    let ord = (av.len() - i).cmp(&(bv.len() - j));
    if ord != Ordering::Equal {
        return ord;
    }
    // Tertiary (case) pass — only when primary-equal.
    let (mut i, mut j) = (0usize, 0usize);
    while i < av.len() && j < bv.len() {
        let (ca, cb) = (av[i], bv[j]);
        if numeric && ca.is_ascii_digit() && cb.is_ascii_digit() {
            while i < av.len() && av[i].is_ascii_digit() {
                i += 1;
            }
            while j < bv.len() && bv[j].is_ascii_digit() {
                j += 1;
            }
            continue;
        }
        if ca != cb && ca.is_alphabetic() && cb.is_alphabetic() {
            let (la, lb) = (ca.is_lowercase(), cb.is_lowercase());
            if la != lb {
                return if la { Ordering::Less } else { Ordering::Greater };
            }
        }
        i += 1;
        j += 1;
    }
    Ordering::Equal
}

// ═══ dispatch prepare ══════════════════════════════════════════════════════

const DISPATCH_RUNTIMES: [&str; 2] = ["codex", "claude"];
const DISPATCH_KINDS: [&str; 4] = ["cell", "gather", "reviewer", "advisor"];

/// provenance: dispatch-prepare.mjs slotForKind (the PURPOSE MAP, advisor A1).
fn slot_for_kind(kind: &str) -> &'static str {
    match kind {
        "cell" | "gather" => "generation",
        "reviewer" => "review",
        _ => "advisor",
    }
}

/// provenance: dispatch-prepare.mjs purposeForKind — only 'cell' is
/// cell-execution; everything else is an explicit read-only gather.
fn purpose_is_gather(kind: &str) -> bool {
    kind != "cell"
}

struct Ownership {
    ok: bool,
    code: Option<&'static str>,
    status: Value,
    owner: Value,
    reason: String,
}

/// provenance: dispatch-prepare.mjs checkCellClaimOwnership (hardening-7) —
/// the CELL RECORD's own status/trace.worker, never the claims store.
fn check_cell_claim_ownership(cell: &Value, worker: &str) -> Ownership {
    let status = vget(cell, "status").cloned().unwrap_or(Value::Null);
    let status_str = tpl(vget(cell, "status"));
    let id = tpl(vget(cell, "id"));
    if !matches!(vget(cell, "status"), Some(Value::String(s)) if s == "claimed") {
        return Ownership {
            ok: false,
            code: Some("not_claimed"),
            status,
            owner: Value::Null,
            reason: format!(
                "cell \"{id}\" is \"{status_str}\", not \"claimed\" — dispatch prepare requires a claimed cell (run bee.mjs cells claim or cells claim-next first). Pass --force-ownership to override (audited)."
            ),
        };
    }
    let owner: Value = match vget(cell, "trace").and_then(|t| vget(t, "worker")) {
        Some(Value::String(w)) => Value::String(w.clone()),
        _ => Value::Null,
    };
    let owner_matches = matches!(&owner, Value::String(w) if w == worker);
    if !owner_matches {
        let shown = match &owner {
            Value::String(w) if !w.is_empty() => w.clone(),
            _ => "(unknown)".to_string(), // `owner || '(unknown)'`
        };
        return Ownership {
            ok: false,
            code: Some("not_owner"),
            status,
            owner,
            reason: format!(
                "cell \"{id}\" is claimed by worker \"{shown}\" — \"{worker}\" does not own this claim. Pass --force-ownership to override (audited)."
            ),
        };
    }
    Ownership { ok: true, code: None, status, owner, reason: String::new() }
}

/// provenance: dispatch-prepare.mjs PRIOR_ROUNDS_MAX_EVENT_LINES.
const PRIOR_ROUNDS_MAX_EVENT_LINES: usize = 12;
/// provenance: dispatch-prepare.mjs LEARNED_CONTEXT_MAX_LINES.
const LEARNED_CONTEXT_MAX_LINES: usize = 8;

/// provenance: dispatch-prepare.mjs priorRoundEventLines — the machine-
/// assembled digest of the cell record's own trace history, chronological
/// (ISO strings compare lexicographically; timeless events sink to the end in
/// insertion order, the sort being stable in both runtimes), capped at 12 with
/// one count line replacing the elided oldest.
fn prior_round_event_lines(cell: &Value) -> Vec<String> {
    let trace = match vget(cell, "trace") {
        Some(v) if is_plain_object(v) => v.clone(),
        _ => Value::Object(Map::new()),
    };
    let arr = |key: &str| -> Vec<Value> {
        match vget(&trace, key) {
            Some(Value::Array(items)) => items.clone(),
            _ => Vec::new(),
        }
    };
    // (at, line) — `at` is `null` for a timeless event.
    let mut events: Vec<(Option<String>, String)> = Vec::new();
    let at_of = |v: &Value, key: &str| -> Option<String> {
        // `attempt.at || null` — a falsy `at` becomes null.
        match vget(v, key) {
            Some(x) if truthy(x) => Some(jsjson::js_to_string(x)),
            _ => None,
        }
    };

    for attempt in arr("attempts") {
        if !is_plain_object(&attempt) {
            continue;
        }
        let worker = truthy_str(vget(&attempt, "worker"))
            .map(str::to_string)
            .unwrap_or_else(|| "(unknown worker)".to_string());
        let verdict = vget(&attempt, "verdict");
        let sig = || match vget(&attempt, "failure_signature") {
            Some(v) if truthy(v) => jsjson::js_to_string(v),
            _ => "(none recorded)".to_string(),
        };
        if matches!(verdict, Some(Value::String(s)) if s == "blocked") {
            let note = one_line(vget(&attempt, "note"), 140);
            let reason = if note.is_empty() {
                format!("failure signature {}", sig())
            } else {
                note
            };
            events.push((at_of(&attempt, "at"), format!("- {worker} blocked: {reason}")));
        } else if matches!(verdict, Some(Value::String(s)) if s == "tests-red") {
            let note = one_line(vget(&attempt, "note"), 140);
            let note = if note.is_empty() { "(no excerpt recorded)".to_string() } else { note };
            events.push((at_of(&attempt, "at"), format!("- {worker} tests red: {note}")));
        } else if matches!(verdict, Some(Value::String(s)) if s == "fail") {
            events.push((
                at_of(&attempt, "at"),
                format!("- {worker} failed verify: failure signature {}", sig()),
            ));
        }
    }

    let capped_at = match vget(&trace, "capped_at") {
        Some(v) if truthy(v) => Some(jsjson::js_to_string(v)),
        _ => None,
    };
    for deviation in arr("deviations") {
        let Value::String(text) = &deviation else { continue };
        if js_trim(text).is_empty() {
            continue;
        }
        events.push((
            capped_at.clone(),
            format!("- (prior worker) deviation: {}", one_line(Some(&deviation), 140)),
        ));
    }

    for consult in arr("semantic_judge") {
        if !is_plain_object(&consult) {
            continue;
        }
        let judge = truthy_str(vget(&consult, "judge_model"))
            .map(str::to_string)
            .unwrap_or_else(|| "(judge)".to_string());
        let pointer = match vget(&consult, "failure_signature") {
            Some(v) if truthy(v) => {
                format!(" (failure signature {})", one_line(Some(v), 40))
            }
            _ => String::new(),
        };
        events.push((
            at_of(&consult, "recorded_at"),
            format!("- {judge} consult: {}{pointer}", tpl(vget(&consult, "verdict"))),
        ));
    }

    if let Some(Value::String(reason)) = vget(&trace, "reopened_reason") {
        if !js_trim(reason).is_empty() {
            events.push((
                at_of(&trace, "reopened_at"),
                format!(
                    "- (orchestrator) reopened: {}",
                    one_line(vget(&trace, "reopened_reason"), 140)
                ),
            ));
        }
    }
    if let Some(rework) = vget(&trace, "reopened_for_rework") {
        if truthy(rework) && is_plain_object(rework) {
            let reason = one_line(vget(rework, "reason"), 140);
            let reason = if reason.is_empty() {
                "NEEDS_REVISION verdict after cap".to_string()
            } else {
                reason
            };
            events.push((
                at_of(rework, "at"),
                format!("- (judge) reopened for rework: {reason}"),
            ));
        }
    }

    // Stable sort with Node's own comparator.
    events.sort_by(|a, b| match (&a.0, &b.0) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, _) => std::cmp::Ordering::Greater,
        (_, None) => std::cmp::Ordering::Less,
        (Some(x), Some(y)) => x.cmp(y),
    });
    let mut lines: Vec<String> = events.into_iter().map(|(_, line)| line).collect();
    if lines.len() > PRIOR_ROUNDS_MAX_EVENT_LINES {
        let kept = PRIOR_ROUNDS_MAX_EVENT_LINES - 1;
        let elided = lines.len() - kept;
        let tail = lines.split_off(lines.len() - kept);
        lines = std::iter::once(format!(
            "- ({elided} earlier event(s) elided — the cell record holds the rest)"
        ))
        .chain(tail)
        .collect();
    }
    lines
}

/// provenance: knowledge.mjs KNOWLEDGE_CONTEXT_LANE_BUDGETS /
/// KNOWLEDGE_CONTEXT_DEFAULT_BUDGET, read through `budgets[cell.lane] ?? default`.
fn lane_budget(lane: Option<&Value>) -> f64 {
    match lane {
        Some(Value::String(l)) => match l.as_str() {
            "tiny" => 8000.0,
            "small" => 12000.0,
            "standard" => 20000.0,
            "high-risk" => 30000.0,
            _ => 20000.0,
        },
        _ => 20000.0,
    }
}

/// provenance: dispatch-prepare.mjs bundleLearnedLines — the work-item
/// manifest first (every failure inside its try/catch falls through), then the
/// bundle index pointer. `Err(Delegate)` is NOT a JS-visible failure: it means
/// the lifted knowledge port cannot decide this bundle, so the whole command
/// re-runs under Node.
fn bundle_learned_lines(
    root: &Path,
    cell: &Value,
    read_first: &HashSet<String>,
) -> D<Vec<String>> {
    let Some(dir) = kctx::bundle_dir(root) else { return Err(Delegate) };
    let budget = lane_budget(vget(cell, "lane"));
    let work = match vget(cell, "feature") {
        Some(Value::String(s)) => s.clone(),
        // A non-string `work` makes buildContextManifest throw missing_work
        // (`typeof work === 'string' ? work.trim() : ''`) -> the catch arm.
        _ => String::new(),
    };
    let manifest = if work.is_empty() {
        None
    } else {
        match kctx::build_context_manifest(&dir, &work, budget, &kctx::num(budget)) {
            kctx::ManifestOut::Built(m) => Some(m),
            kctx::ManifestOut::Thrown(_) => None, // caught by dispatch-prepare's try
            kctx::ManifestOut::NeedsNode => return Err(Delegate),
        }
    };
    if let Some(manifest) = manifest {
        let Some(concepts) = kctx::collect_concepts(&dir) else { return Err(Delegate) };
        // `new Map(...)`: last write wins per key (never hit for a real bundle,
        // where paths are unique).
        let mut titles: Vec<(String, Option<String>)> = Vec::new();
        for concept in &concepts {
            let key = format!("docs/knowledge/{}", concept.path);
            let title = match concept.data.get("title") {
                Some(Value::String(t)) if !t.is_empty() => Some(t.clone()),
                _ => None,
            };
            if let Some(slot) = titles.iter_mut().find(|(k, _)| *k == key) {
                slot.1 = title;
            } else {
                titles.push((key, title));
            }
        }
        let entries = match manifest.get("entries") {
            Some(Value::Array(items)) => items.clone(),
            _ => Vec::new(),
        };
        let mut lines = Vec::new();
        for entry in &entries {
            let path = tpl(vget(entry, "path"));
            if read_first.contains(&path) {
                continue; // read_first stays authoritative — never duplicated
            }
            // `titles.get(entry.path) || entry.path.slice(lastIndexOf('/') + 1)`
            let title = titles
                .iter()
                .find(|(k, _)| *k == path)
                .and_then(|(_, t)| t.clone())
                .unwrap_or_else(|| match path.rfind('/') {
                    Some(p) => path[p + 1..].to_string(),
                    None => path.clone(),
                });
            lines.push(format!(
                "- {path} — {}",
                one_line(Some(&Value::String(title)), 140)
            ));
        }
        if !lines.is_empty() {
            return Ok(lines);
        }
    }
    if dir.join("index.md").exists() && !read_first.contains("docs/knowledge/index.md") {
        return Ok(vec![
            "- docs/knowledge/index.md — Knowledge bundle index (see \"Critical patterns\")"
                .to_string(),
        ]);
    }
    Ok(Vec::new())
}

/// provenance: knowledge.mjs bundleMode — a DIRECTORY is not a bundle: at
/// least one non-reserved markdown file must parse as a strict OKF concept
/// carrying a non-empty string `type`.
fn bundle_mode(root: &Path) -> D<bool> {
    let Some(dir) = kctx::bundle_dir(root) else { return Err(Delegate) };
    match std::fs::metadata(&dir) {
        Ok(m) if m.is_dir() => {}
        _ => return Ok(false),
    }
    let Some(rels) = kctx::list_bundle_markdown(&dir) else { return Err(Delegate) };
    for rel in rels {
        let base = rel.rsplit('/').next().unwrap_or(&rel);
        if kctx::is_reserved_basename(base) {
            continue;
        }
        let Ok(text) = kctx::read_file_lossy(&kctx::join_rel(&dir, &rel)) else { continue };
        match kctx::parse_frontmatter(&text) {
            kctx::Fm::Parsed { data, .. } => {
                if matches!(data.get("type"), Some(Value::String(t)) if !t.is_empty()) {
                    return Ok(true);
                }
            }
            kctx::Fm::NeedsNode => return Err(Delegate),
            _ => {}
        }
    }
    Ok(false)
}

/// provenance: dispatch-prepare.mjs learnedContextLines — source resolution,
/// first hit wins, capped at LEARNED_CONTEXT_MAX_LINES.
fn learned_context_lines(root: &Path, cell: &Value) -> D<Vec<String>> {
    let mut read_first: HashSet<String> = HashSet::new();
    if let Some(Value::Array(items)) = vget(cell, "read_first") {
        for entry in items {
            if let Value::String(s) = entry {
                let normalized = s.replace('\\', "/");
                let normalized = normalized.strip_prefix("./").unwrap_or(&normalized);
                read_first.insert(normalized.to_string());
            }
        }
    }
    let mut lines = if bundle_mode(root)? {
        bundle_learned_lines(root, cell, &read_first)?
    } else if root
        .join("docs")
        .join("history")
        .join("learnings")
        .join("critical-patterns.md")
        .exists()
        && !read_first.contains("docs/history/learnings/critical-patterns.md")
    {
        vec![
            "- docs/history/learnings/critical-patterns.md — Critical patterns (hard-won learnings)"
                .to_string(),
        ]
    } else {
        Vec::new()
    };
    lines.truncate(LEARNED_CONTEXT_MAX_LINES);
    Ok(lines)
}

/// provenance: dispatch-prepare.mjs cellPromptBody / promptBodyFor.
fn prompt_body_for(
    root: &Path,
    kind: &str,
    cell: Option<&Value>,
    worker: Option<&str>,
) -> D<Result<String, String>> {
    if kind != "cell" {
        let Some(template) = load_prompt(kind) else { return Err(Delegate) };
        return Ok(render(&template, &[]));
    }
    let cell = cell.expect("kind cell always carries a loaded cell");
    let Some(template) = load_prompt("worker-cell") else { return Err(Delegate) };
    let learned = learned_context_lines(root, cell)?.join("\n");
    let prior = prior_round_event_lines(cell).join("\n");
    let cell_json = jsjson::stringify_pretty(cell);
    let feature = tpl(vget(cell, "feature"));
    let cell_id = tpl(vget(cell, "id"));
    Ok(render(
        &template,
        &[
            ("worker", worker.unwrap_or("undefined")),
            ("cell_id", &cell_id),
            ("feature", &feature),
            ("cell_json", &cell_json),
            ("learned_context", &learned),
            ("prior_rounds", &prior),
        ],
    ))
}

/// provenance: dispatch-prepare.mjs appendPrepareRecord — fail-open, exactly
/// like Node's try/catch: a log failure never blocks the payload.
fn append_prepare_record(root: &Path, record: &Map<String, Value>) {
    let mut line = Map::new();
    line.insert(
        "ts".into(),
        Value::String(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
    );
    line.insert("source".into(), Value::String("prepare".into()));
    for (k, v) in record {
        line.insert(k.clone(), v.clone());
    }
    let _ = crate::fsutil::append_jsonl(
        &root.join(".bee").join("logs").join("dispatch.jsonl"),
        &Value::Object(line),
    );
}

/// A prepareDispatch outcome: a returned value (envelope OR typed refusal), or
/// a thrown Error (malformed CALL).
enum Prepared {
    Value(Value),
    Thrown(String),
}

/// provenance: dispatch-prepare.mjs prepareDispatch(root, {...}). Throws only
/// on a malformed CALL; every legitimate cli-shaped / unconfigured-advisor /
/// native-unavailable / claim-ownership resolution is a typed {ok:false}
/// RETURN, not an exception.
#[allow(clippy::too_many_arguments)]
fn prepare_dispatch(
    root: &Path,
    runtime: &str,
    kind: &str,
    cell_id: Option<&str>,
    worker: Option<&str>,
    force_ownership: bool,
    classification: Option<&str>,
    record_it: bool,
) -> D<Prepared> {
    // The runtime/kind gates already fired in the probe (validate() owns those
    // bytes), so both are known-good here.
    debug_assert!(DISPATCH_RUNTIMES.contains(&runtime) && DISPATCH_KINDS.contains(&kind));

    let mut cell: Option<Value> = None;
    let mut ownership_override: Option<Value> = None;
    let mut resolved_worker: Option<String> = None;

    if kind == "cell" {
        let Some(cell_id) = cell_id else {
            return Ok(Prepared::Thrown(
                "dispatch prepare: --cell is required when --kind cell.".to_string(),
            ));
        };
        let Some(loaded) = read_cell(root, cell_id)? else {
            return Ok(Prepared::Thrown(format!(
                "dispatch prepare: cell \"{cell_id}\" not found."
            )));
        };
        let Some(worker) = worker.filter(|w| !js_trim(w).is_empty()) else {
            return Ok(Prepared::Thrown(
                "dispatch prepare: --worker is required when --kind cell.".to_string(),
            ));
        };
        let trimmed = js_trim(worker).to_string();
        let ownership = check_cell_claim_ownership(&loaded, &trimmed);
        if !ownership.ok && !force_ownership {
            let mut refusal = Map::new();
            refusal.insert("ok".into(), Value::Bool(false));
            refusal.insert("type".into(), Value::String("refused".into()));
            refusal.insert("reason".into(), Value::String("claim_ownership".into()));
            refusal.insert(
                "code".into(),
                ownership.code.map(|c| Value::String(c.into())).unwrap_or(Value::Null),
            );
            refusal.insert("status".into(), ownership.status);
            refusal.insert("owner".into(), ownership.owner);
            refusal.insert("fix".into(), Value::String(ownership.reason));
            return Ok(Prepared::Value(Value::Object(refusal)));
        }
        if force_ownership {
            let mut ov = Map::new();
            ov.insert("forced_by".into(), Value::String(trimmed.clone()));
            ov.insert("bypassed".into(), Value::Bool(!ownership.ok));
            ov.insert(
                "code".into(),
                if ownership.ok {
                    Value::Null
                } else {
                    ownership.code.map(|c| Value::String(c.into())).unwrap_or(Value::Null)
                },
            );
            ov.insert(
                "owner_bypassed".into(),
                if ownership.ok { Value::Null } else { ownership.owner.clone() },
            );
            ov.insert(
                "status_bypassed".into(),
                if ownership.ok { Value::Null } else { ownership.status.clone() },
            );
            ov.insert("transferred".into(), Value::Bool(false));
            ov.insert("note".into(), Value::String("advisory bypass only — cell.trace.worker (the actual claim owner) was NOT transferred; no correct transfer primitive exists on this ownership axis (see comment above).".into()));
            ownership_override = Some(Value::Object(ov));
        }
        resolved_worker = Some(trimmed);
        cell = Some(loaded);
    }

    let tier_token = slot_for_kind(kind);
    let models = read_models(root)?;
    let resolved = if kind == "advisor" {
        match resolve_advisor(&models, runtime) {
            Some(r) => r,
            None => {
                let mut refusal = Map::new();
                refusal.insert("ok".into(), Value::Bool(false));
                refusal.insert("reason".into(), Value::String("advisor_not_configured".into()));
                refusal.insert("fix".into(), Value::String(format!(
                    "set models.{runtime}.advisor in .bee/config.json to enable an advisor consult (resolveAdvisor never falls back to another tier)."
                )));
                return Ok(Prepared::Value(Value::Object(refusal)));
            }
        }
    } else {
        let r = resolve_tier(&models, tier_token, runtime, purpose_is_gather(kind));
        if let Resolved::Refused { slot } = &r {
            let mut refusal = Map::new();
            refusal.insert("ok".into(), Value::Bool(false));
            refusal.insert("type".into(), Value::String("refused".into()));
            refusal.insert("reason".into(), Value::String("cli_tier_gather_only".into()));
            refusal.insert("slot".into(), Value::String(slot.clone()));
            refusal.insert("fix".into(), Value::String(CLI_REFUSAL_FIX.into()));
            return Ok(Prepared::Value(Value::Object(refusal)));
        }
        r
    };

    let prompt_body = match prompt_body_for(root, kind, cell.as_ref(), resolved_worker.as_deref())? {
        Ok(body) => body,
        Err(msg) => return Ok(Prepared::Thrown(msg)),
    };
    let requested_model = match &resolved {
        Resolved::Model { model, .. } => Some(model.clone()),
        _ => None,
    };
    let pinned_type = pinned_agent_type(tier_token);

    let mut tool = String::new();
    let mut payload = Map::new();
    let mut channel = String::new();
    let mut refusal: Option<Value> = None;
    let mut native_confirmed = false;
    // envelopeExtra, kept as its two possible keys so the spread order below
    // stays byte-identical.
    let mut extra_transport: Option<&str> = None;
    let mut extra_fallback_reason: Option<&str> = None;

    match &resolved {
        Resolved::Native { model, effort, fallback, agent_type, .. } => {
            native_confirmed = classification == Some(NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE);
            if native_confirmed {
                tool = "spawn_agent".into();
                payload.insert(
                    "agent_type".into(),
                    Value::String(if agent_type.is_empty() {
                        "worker".to_string()
                    } else {
                        agent_type.clone()
                    }),
                );
                payload.insert(
                    "message".into(),
                    Value::String(format!("[bee-tier: {tier_token}]\n{prompt_body}")),
                );
                payload.insert("model".into(), Value::String(model.clone()));
                payload.insert("fork_turns".into(), Value::String("none".into()));
                if let Some(effort) = effort {
                    payload.insert("reasoning_effort".into(), Value::String(effort.clone()));
                }
                channel = "codex-native".into();
                extra_transport = Some("native-override");
            } else if let Some(command) = fallback.as_ref().filter(|c| !c.is_empty()) {
                tool = "Bash".into();
                payload.insert("command".into(), Value::String(command.clone()));
                payload.insert("stdin".into(), Value::String(prompt_body.clone()));
                channel = "cli-exec".into();
                extra_fallback_reason = Some("native_unavailable");
            } else {
                let mut r = Map::new();
                r.insert("ok".into(), Value::Bool(false));
                r.insert("type".into(), Value::String("refused".into()));
                r.insert("reason".into(), Value::String("native_unavailable".into()));
                r.insert(
                    "detail".into(),
                    Value::String(
                        classification
                            .filter(|c| !c.is_empty())
                            .unwrap_or(NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY)
                            .to_string(),
                    ),
                );
                refusal = Some(Value::Object(r));
            }
        }
        Resolved::Cli { command } => {
            tool = "Bash".into();
            payload.insert("command".into(), Value::String(command.clone()));
            payload.insert("stdin".into(), Value::String(prompt_body.clone()));
            channel = "cli-exec".into();
        }
        _ if runtime == "codex" => {
            tool = "spawn_agent".into();
            payload.insert(
                "task_name".into(),
                Value::String(match &cell {
                    Some(c) => tpl(vget(c, "id")),
                    None => format!("bee-{kind}"),
                }),
            );
            payload.insert(
                "message".into(),
                Value::String(format!("[bee-tier: {tier_token}]\n{prompt_body}")),
            );
            payload.insert("fork_turns".into(), Value::String("none".into()));
            channel = "codex-native".into();
        }
        _ => {
            tool = "Agent".into();
            payload.insert("subagent_type".into(), Value::String(pinned_type.into()));
            payload.insert(
                "prompt".into(),
                Value::String(format!("[bee-tier: {tier_token}]\n{prompt_body}")),
            );
            payload.insert(
                "description".into(),
                Value::String(format!(
                    "{kind} ({})",
                    // `requestedModel || tierToken`
                    requested_model.clone().filter(|m| !m.is_empty()).unwrap_or_else(|| tier_token.to_string())
                )),
            );
            if let Resolved::Model { model, .. } = &resolved {
                payload.insert("model".into(), Value::String(model.clone()));
            }
            channel = "claude-agent".into();
        }
    }

    if let Some(refusal) = refusal {
        return Ok(Prepared::Value(refusal));
    }

    let param_model = match (&channel[..], &resolved) {
        ("claude-agent", Resolved::Model { model, .. }) => Some(model.clone()),
        _ => None,
    };
    let economics = derive_economics(
        &channel,
        tier_token,
        param_model.as_deref(),
        &resolved,
        native_confirmed,
    );

    let dispatch_id = pseudo_uuid_v4();

    let mut record = Map::new();
    record.insert("dispatch_id".into(), Value::String(dispatch_id.clone()));
    record.insert("kind".into(), Value::String(kind.to_string()));
    record.insert(
        "cell".into(),
        match &cell {
            Some(c) => vget(c, "id").cloned().unwrap_or(Value::Null),
            None => Value::Null,
        },
    );
    record.insert("runtime".into(), Value::String(runtime.to_string()));
    let classification_value = match classification {
        Some(c) if !c.is_empty() => Value::String(c.to_string()),
        _ => Value::Null,
    };
    if let Some(reason) = extra_fallback_reason {
        record.insert("native_fallback_reason".into(), Value::String(reason.into()));
        record.insert("native_classification".into(), classification_value.clone());
    }
    if extra_transport.is_some() {
        record.insert("native_classification".into(), classification_value);
    }
    if let Some(ov) = &ownership_override {
        record.insert("ownership_override".into(), ov.clone());
    }
    for (k, v) in &economics {
        record.insert(k.clone(), v.clone());
    }
    // `record_it` is false on the PROBE pass: run() builds the whole envelope
    // once to discover delegate-shaped inputs before a byte is produced, then
    // rebuilds it for real. Gating the append here means a command that ends
    // up delegating never leaves a prepare line behind, and one that is served
    // leaves exactly one — Node's count.
    if record_it {
        append_prepare_record(root, &record);
    }

    let mut envelope = Map::new();
    envelope.insert("tool".into(), Value::String(tool));
    envelope.insert("payload".into(), Value::Object(payload));
    envelope.insert("dispatch_id".into(), Value::String(dispatch_id));
    envelope.insert("economics".into(), Value::Object(economics));
    if let Some(t) = extra_transport {
        envelope.insert("transport".into(), Value::String(t.into()));
    }
    if let Some(r) = extra_fallback_reason {
        envelope.insert("fallback_reason".into(), Value::String(r.into()));
    }
    if let Some(ov) = ownership_override {
        envelope.insert("ownership_override".into(), ov);
    }
    Ok(Prepared::Value(Value::Object(envelope)))
}

/// provenance: bee.mjs readNativeTransportClassification — the delegating
/// slice. An absent / unreadable / unparseable probe record and a
/// schema-mismatched one both short-circuit to native_budget_only with NO
/// subprocess; anything past that point shells out to codex-cli, so it
/// delegates.
const NATIVE_TRANSPORT_PROBE_SCHEMA: &str = "native-transport-probe/1";

fn native_transport_classification(root: &Path) -> D<&'static str> {
    let file = root.join(".bee").join("native-transport-probe.json");
    // doctorSafeReadJson: unreadable OR unparseable both yield null.
    let record = match std::fs::read(&file) {
        Err(_) => None,
        Ok(bytes) => serde_json::from_str::<Value>(&String::from_utf8_lossy(&bytes)).ok(),
    };
    match record {
        None => Ok(NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY),
        Some(r) if !matches!(vget(&r, "schema"), Some(Value::String(s)) if s == NATIVE_TRANSPORT_PROBE_SCHEMA) => {
            Ok(NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY)
        }
        // A live probe record: doctorRepoIdentity + `codex --version` +
        // `codex features list` + the config-scope hash all have to run.
        Some(_) => Err(Delegate),
    }
}

fn run_dispatch_prepare(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(
        &flags,
        &["runtime", "kind", "cell", "worker", "force-ownership", "claim", "session-id"],
    ) {
        return None;
    }
    // --claim: the claim+reserve doors are Node's (see the file header).
    if flags.get("claim").is_some() {
        return None;
    }
    // `--session-id` is documented as ignored without --claim; a caller that
    // passes it anyway is an unproven shape here.
    if flags.get("session-id").is_some() {
        return None;
    }
    // validate(): boolean-typed --force-ownership given as =value.
    match flags.get("force-ownership") {
        None | Some(FlagV::Present) => {}
        Some(FlagV::S(s)) if s == "true" || s == "false" => {}
        Some(FlagV::S(_)) => return None,
    }
    // validate(): runtime/kind required + enum-checked.
    let runtime = flags.req_str("runtime")?.to_string();
    let kind = flags.req_str("kind")?.to_string();
    if !DISPATCH_RUNTIMES.contains(&runtime.as_str()) || !DISPATCH_KINDS.contains(&kind.as_str()) {
        return None; // validate()'s enum message
    }
    // `typeof flags.cell === 'string' && flags.cell ? flags.cell : null`
    let cell_id = flags.truthy_str("cell").map(str::to_string);
    let worker = flags.truthy_str("worker").map(str::to_string);
    let force_ownership = matches!(flags.get("force-ownership"), Some(FlagV::Present));

    // ── everything that can still delegate happens BEFORE prelude: its
    //    drift-cache write would otherwise swallow the Node re-run's
    //    manifest_changed line. ─────────────────────────────────────────────
    let cwd = std::env::current_dir().ok()?;
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::NeedsNode => return None,
        Roots::None => {
            return Some(emit_no_root_error(&cwd, "dispatch prepare", use_json, t0));
        }
    };
    let prompt_name = if kind == "cell" { "worker-cell" } else { kind.as_str() };
    if !prompts_match_disk(&root, prompt_name) {
        return None; // prompt skew ⇒ delegate (C4)
    }
    let classification = if runtime == "codex" {
        Some(native_transport_classification(&root).ok()?)
    } else {
        None
    };
    // Dry-run the whole build to surface every delegate-shaped input before a
    // single byte (or the prepare-time log line) is produced. The build is
    // free of side effects apart from appendPrepareRecord, which is applied on
    // the SECOND pass only.
    let prepared = prepare_dispatch(
        &root,
        &runtime,
        &kind,
        cell_id.as_deref(),
        worker.as_deref(),
        force_ownership,
        classification,
        false,
    )
    .ok()?;

    let ctx = match prelude("dispatch prepare", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    let out: R2<Out> = match prepared {
        Prepared::Thrown(msg) => Ok(Out::Thrown(msg)),
        Prepared::Value(_) => {
            // Re-run for real so the prepare-time record is appended exactly
            // once, with a freshly minted dispatch_id/ts like Node's.
            match prepare_dispatch(
                &ctx.root,
                &runtime,
                &kind,
                cell_id.as_deref(),
                worker.as_deref(),
                force_ownership,
                classification,
                true,
            ) {
                Ok(Prepared::Value(result)) => {
                    let text = jsjson::stringify_pretty(&result);
                    Ok(Out::Emit(result, text, 0))
                }
                Ok(Prepared::Thrown(msg)) => Ok(Out::Thrown(msg)),
                Err(_) => Err(crate::verbs::reservations::Err2::Ex),
            }
        }
    };
    finish(&ctx, out)
}

// ═══ close ═════════════════════════════════════════════════════════════════

/// provenance: test-runner.mjs TEST_RESULTS_RELATIVE (verbs/test_runner.rs:60).
const TEST_RESULTS_RELATIVE: &str = ".bee/logs/test-results.json";
/// provenance: test-runner.mjs FAILURE_EXCERPT_MAX_CHARS (verbs/test_runner.rs:63).
const FAILURE_EXCERPT_MAX: usize = 500;
/// provenance: bee.mjs CLOSE_TESTS_UNDECLARED_DETAIL.
const CLOSE_TESTS_UNDECLARED_DETAIL: &str = "no commands.test declared — close has no test door here; declare commands.test in .bee/config.json (string or array) to give it one";

/// provenance: test-runner.mjs declaredTestCommands + state.mjs
/// normalizeCommands (verbs/test_runner.rs:184 declared_test_commands).
/// `None` == JS `null` (undeclared).
fn declared_test_commands(root: &Path) -> D<Option<Vec<String>>> {
    let config = read_config_raw(root)?;
    if let Some(Value::Array(items)) = config.get("dogfood_repos") {
        if !items.is_empty() {
            return Err(Delegate); // normalizeDogfoodRepos may warn to stderr
        }
    }
    let raw_test = config
        .get("commands")
        .and_then(Value::as_object)
        .and_then(|c| c.get("test"));
    let normalized: Vec<String> = match raw_test {
        Some(Value::String(s)) => {
            let t = js_trim(s);
            if t.is_empty() { Vec::new() } else { vec![t.to_string()] }
        }
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(js_trim)
            .filter(|t| !t.is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    };
    let cleaned: Vec<String> = normalized.into_iter().filter(|c| c != "none").collect();
    Ok(if cleaned.is_empty() { None } else { Some(cleaned) })
}

struct CommandResult {
    command: String,
    exit: Option<i64>,
    duration_ms: u64,
    failure_excerpt: Option<String>,
}

struct TestRun {
    ran_at: String,
    green: bool,
    undeclared: bool,
    commands: Vec<CommandResult>,
    write_error: Option<String>,
}

/// provenance: test-runner.mjs spawnDeclaredCommand + posixShell
/// (verbs/test_runner.rs:235 shell_command) — on Windows the child's PATH is
/// set explicitly so Rust resolves the bare `bash` PATH-FIRST like libuv,
/// finding Git Bash instead of CreateProcess's System32-first WSL bash.
fn shell_command(shell: &str) -> Command {
    let mut cmd = Command::new(shell);
    if cfg!(windows) {
        if let Some(path) = std::env::var_os("PATH") {
            cmd.env("PATH", path);
        }
    }
    cmd
}

/// provenance: test-runner.mjs posixShell (verbs/test_runner.rs:249).
fn posix_shell() -> Option<&'static str> {
    let shell = if cfg!(windows) { "bash" } else { "/bin/sh" };
    let probe = shell_command(shell)
        .args(["-c", "exit 0"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    match probe {
        Ok(s) if s.success() => Some(shell),
        _ => None,
    }
}

/// provenance: test-runner.mjs runDeclaredTests (verbs/test_runner.rs:263).
fn run_declared_tests(root: &Path, commands: &[String], shell: &str) -> TestRun {
    let ran_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let mut results: Vec<CommandResult> = Vec::new();
    let mut green = true;
    for command in commands {
        let started = Instant::now();
        let spawned = shell_command(shell)
            .arg("-c")
            .arg(command)
            .current_dir(root)
            .stdin(Stdio::null())
            .output();
        let duration_ms = started.elapsed().as_millis() as u64;
        let (mut output, exit, spawn_err) = match &spawned {
            Ok(out) => (
                format!(
                    "{}{}",
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                ),
                out.status.code().map(i64::from),
                None,
            ),
            Err(e) => (String::new(), None, Some(e.to_string())),
        };
        if let Some(msg) = spawn_err {
            output.push_str(&format!("\n[bee test] spawn error: {msg}"));
        }
        let passed = spawned.is_ok() && exit == Some(0);
        if !passed {
            green = false;
        }
        let failure_excerpt = if passed {
            None
        } else {
            let trimmed = js_trim(&output).to_string();
            let tail = utf16_tail(&trimmed, FAILURE_EXCERPT_MAX);
            Some(if tail.is_empty() {
                format!(
                    "(no output; exit {})",
                    exit.map(|e| e.to_string()).unwrap_or_else(|| "null".to_string())
                )
            } else {
                tail
            })
        };
        results.push(CommandResult { command: command.clone(), exit, duration_ms, failure_excerpt });
    }
    let mut record = Map::new();
    record.insert("ran_at".into(), Value::String(ran_at.clone()));
    record.insert("green".into(), Value::Bool(green));
    record.insert(
        "commands".into(),
        Value::Array(results.iter().map(command_result_value).collect()),
    );
    let write_error = write_json_atomic(
        &root.join(".bee").join("logs").join("test-results.json"),
        &Value::Object(record),
    )
    .err()
    .map(|e| e.to_string());
    TestRun { ran_at, green, undeclared: false, commands: results, write_error }
}

/// JS `.slice(-n)` over UTF-16 units (verbs/test_runner.rs:420 utf16_tail).
fn utf16_tail(s: &str, n: usize) -> String {
    let units: Vec<u16> = s.encode_utf16().collect();
    if units.len() <= n {
        return s.to_string();
    }
    let mut start = units.len() - n;
    if (0xDC00..=0xDFFF).contains(&units[start]) {
        start -= 1;
    }
    String::from_utf16_lossy(&units[start..])
}

/// {command, exit, duration_ms, failure_excerpt} — frozen key order.
fn command_result_value(c: &CommandResult) -> Value {
    let mut m = Map::new();
    m.insert("command".into(), Value::String(c.command.clone()));
    m.insert(
        "exit".into(),
        match c.exit {
            Some(code) => Value::Number(Number::from(code)),
            None => Value::Null,
        },
    );
    m.insert("duration_ms".into(), Value::Number(Number::from(c.duration_ms)));
    m.insert(
        "failure_excerpt".into(),
        match &c.failure_excerpt {
            Some(s) => Value::String(s.clone()),
            None => Value::Null,
        },
    );
    Value::Object(m)
}

/// provenance: bee.mjs renderTestCommandLines (~7601) — shared by `bee test`
/// and close, so the two surfaces can never render the same run differently.
fn render_test_command_lines(run: &TestRun) -> Vec<String> {
    run.commands
        .iter()
        .map(|c| {
            let secs = format!("{:.1}s", c.duration_ms as f64 / 1000.0);
            match &c.failure_excerpt {
                None => format!("✓ {} ({})", c.command, secs),
                Some(_) => format!(
                    "✗ {} ({}, exit {})",
                    c.command,
                    secs,
                    c.exit.map(|e| e.to_string()).unwrap_or_else(|| "spawn-failed".to_string())
                ),
            }
        })
        .collect()
}

/// provenance: test-runner.mjs firstFailureLine (verbs/test_runner.rs:381).
fn first_failure_line(run: &TestRun) -> Option<String> {
    let failing = run
        .commands
        .iter()
        .find(|c| c.failure_excerpt.as_deref().is_some_and(|s| !s.is_empty()))?;
    failing
        .failure_excerpt
        .as_deref()?
        .split('\n')
        .map(js_trim)
        .find(|l| !l.is_empty())
        .map(str::to_string)
}

// ── scribing debt + capture queue (the report-only doors) ──────────────────

/// provenance: cells.mjs scribingRunStampMs (verbs/status_full.rs:1700).
fn scribing_run_stamp_ms(run: Option<&Value>) -> Option<f64> {
    let run = run?;
    if !truthy(run) {
        return None;
    }
    let at = vget(run, "at").filter(|v| truthy(v));
    let chosen = at.or_else(|| vget(run, "date"));
    let parsed = date_parse(chosen);
    if parsed.is_finite() { Some(parsed) } else { None }
}

/// provenance: reservations.rs js_date_parse, wrapped: an exotic date shape
/// (which V8 may parse and this port may not) yields NaN here, which is the
/// same control-flow branch Node takes for an unparseable date.
fn date_parse(v: Option<&Value>) -> f64 {
    match v {
        Some(Value::String(s)) => match crate::verbs::reservations::js_date_parse(s) {
            Ok(Some(ms)) => ms,
            _ => f64::NAN,
        },
        _ => f64::NAN,
    }
}

/// provenance: cells.mjs bestScribingStampMs (verbs/status_full.rs:1718),
/// scoped to `feature`. LANE records are excluded by routing (a repo carrying
/// `.bee/lanes/*.json` delegates before this runs), so the lane arm is a
/// provable no-op here.
fn best_scribing_stamp_ms(root: &Path, feature: &str, state: &Map<String, Value>) -> Option<f64> {
    let feature_value = Value::String(feature.to_string());
    let mut best: Option<f64> = None;
    for entry in read_jsonl(&root.join(".bee").join("logs").join("scribing-runs.jsonl")) {
        if !truthy(&entry) || !strict_eq(vget(&entry, "feature"), Some(&feature_value)) {
            continue;
        }
        let parsed = date_parse(vget(&entry, "ts"));
        if parsed.is_finite() && best.map(|b| parsed > b).unwrap_or(true) {
            best = Some(parsed);
        }
    }
    if let Some(lsr) = state.get("last_scribing_run") {
        if truthy(lsr) && strict_eq(vget(lsr, "feature"), Some(&feature_value)) {
            if let Some(stamp) = scribing_run_stamp_ms(Some(lsr)) {
                if best.map(|b| stamp > b).unwrap_or(true) {
                    best = Some(stamp);
                }
            }
        }
    }
    best
}

/// provenance: fsutil.mjs readJsonl (verbs/status_full.rs:526) — unparseable
/// lines are silently skipped.
fn read_jsonl(file: &Path) -> Vec<Value> {
    let Ok(bytes) = std::fs::read(file) else { return Vec::new() };
    let text = String::from_utf8_lossy(&bytes);
    let mut events = Vec::new();
    for line in text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        let trimmed = js_trim(line);
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
            events.push(v);
        }
    }
    events
}

/// provenance: state.mjs readState — the ONE field scribingDebt reads through
/// it here is `last_scribing_run` (the feature comes from --feature, never
/// from the record), and defaultState() carries no such key, so the raw file
/// object IS the merged value for it.
fn read_state(root: &Path) -> D<Map<String, Value>> {
    match rj(&root.join(".bee").join("state.json"))? {
        Some(Value::Object(m)) => Ok(m),
        _ => Ok(Map::new()),
    }
}

struct DebtSummary {
    count: usize,
    ids: Vec<Value>,
}

/// provenance: cells.mjs scribingDebt(root, {feature}) — the feature-scoped
/// overrides arm (scribing-integrity si-1), which is the one close uses.
fn scribing_debt(root: &Path, feature: &str) -> D<DebtSummary> {
    let state = read_state(root)?;
    let threshold = best_scribing_stamp_ms(root, feature, &state).unwrap_or(0.0);
    let mut ids = Vec::new();
    for cell in list_cells(root, feature, "capped")? {
        let trace = vget(&cell, "trace").cloned().unwrap_or(Value::Object(Map::new()));
        if !matches!(vget(&trace, "behavior_change"), Some(Value::Bool(true))) {
            continue;
        }
        let capped_at = date_parse(vget(&trace, "capped_at"));
        if capped_at.is_finite() && capped_at > threshold {
            ids.push(vget(&cell, "id").cloned().unwrap_or(Value::Null));
        }
    }
    Ok(DebtSummary { count: ids.len(), ids })
}

/// provenance: capture.mjs pendingCaptureStubs + captureQueue
/// (verbs/status_full.rs:2382) — only the COUNT reaches close's door text, so
/// pendingCaptureStubs' localeCompare sort cannot affect an emitted byte.
fn capture_queue_count(root: &Path) -> usize {
    let events = read_jsonl(&root.join(".bee").join("capture-queue.jsonl"));
    let mut flushed: Vec<Value> = Vec::new();
    let mut stubs: Vec<&Value> = Vec::new();
    for event in &events {
        if !matches!(event, Value::Object(_)) {
            continue;
        }
        let id = vget(event, "id");
        if matches!(vget(event, "kind"), Some(Value::String(k)) if k == "flush")
            && id.map(truthy).unwrap_or(false)
        {
            flushed.push(id.unwrap().clone());
        } else if matches!(vget(event, "kind"), Some(Value::String(k)) if k == "stub")
            && id.map(truthy).unwrap_or(false)
        {
            stubs.push(event);
        }
    }
    stubs
        .into_iter()
        .filter(|s| !flushed.iter().any(|f| strict_eq(Some(f), vget(s, "id"))))
        .count()
}

struct Door {
    door: &'static str,
    blocking: bool,
    detail: String,
    command: Option<&'static str>,
}

impl Door {
    fn value(&self) -> Value {
        let mut m = Map::new();
        m.insert("door".into(), Value::String(self.door.into()));
        m.insert("blocking".into(), Value::Bool(self.blocking));
        m.insert("detail".into(), Value::String(self.detail.clone()));
        m.insert(
            "command".into(),
            match self.command {
                Some(c) => Value::String(c.into()),
                None => Value::Null,
            },
        );
        Value::Object(m)
    }
}

/// provenance: bee.mjs buildCloseReportDoors — capture is DEFERRED (decision
/// c8e25271): both doors are report-only reminders, never a due-now step.
fn build_close_report_doors(root: &Path, feature: &str) -> D<Vec<Door>> {
    let scribing = scribing_debt(root, feature)?;
    let mut doors = Vec::new();
    doors.push(Door {
        door: "scribing-debt",
        blocking: false,
        detail: if scribing.count > 0 {
            format!(
                "pending — {} behavior_change cell(s) uncaptured ({}); settle later via bee-capturing",
                scribing.count,
                js_join(&scribing.ids, ", ")
            )
        } else {
            "clear".to_string()
        },
        command: None,
    });
    let queue = capture_queue_count(root);
    doors.push(Door {
        door: "capture-queue",
        blocking: false,
        detail: if queue > 0 {
            format!("pending — {queue} capture stub(s) awaiting flush; settle later via bee-capturing")
        } else {
            "clear".to_string()
        },
        command: None,
    });
    Ok(doors)
}

/// JS Array.prototype.join (null/undefined render empty).
fn js_join(items: &[Value], sep: &str) -> String {
    items
        .iter()
        .map(|v| match v {
            Value::Null => String::new(),
            other => jsjson::js_to_string(other),
        })
        .collect::<Vec<_>>()
        .join(sep)
}

/// provenance: bee.mjs renderCloseDoorLines.
fn render_close_door_lines(doors: &[Door]) -> Vec<String> {
    doors
        .iter()
        .map(|d| {
            if !d.blocking && d.detail == "clear" {
                return format!("door {}: clear", d.door);
            }
            format!(
                "door {}: {} — {}{}",
                d.door,
                if d.blocking { "BLOCKING" } else { "open" },
                d.detail,
                match d.command {
                    Some(c) => format!(" | settle: {c}"),
                    None => String::new(),
                }
            )
        })
        .collect()
}

/// provenance: bee.mjs handleClose (~7643). `worktree` is provably null here
/// (see the file header), so the merge-back line never renders natively.
fn close_handler(
    root: &Path,
    feature: &str,
    dry_run: bool,
    declared: Option<Vec<String>>,
    shell: Option<&'static str>,
) -> D<Out> {
    if dry_run {
        let mut doors = vec![Door {
            door: "tests",
            blocking: false,
            detail: match &declared {
                Some(cmds) => format!(
                    "commands.test declared ({} command(s)) — close runs the full declared suite fresh; a stale test-results record is never trusted",
                    cmds.len()
                ),
                None => CLOSE_TESTS_UNDECLARED_DETAIL.to_string(),
            },
            command: if declared.is_some() { Some("bee test") } else { None },
        }];
        doors.extend(build_close_report_doors(root, feature)?);
        let next_line = match &declared {
            Some(_) => format!("next: bee close --feature {feature} — runs the declared tests and reports"),
            None => format!(
                "next: feature \"{feature}\" has no test door — close proceeds; capture stays pending for bee-capturing"
            ),
        };
        let mut result = Map::new();
        result.insert("feature".into(), Value::String(feature.to_string()));
        result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
        let mut lines = render_close_door_lines(&doors);
        lines.push(next_line);
        return Ok(Out::Emit(Value::Object(result), lines.join("\n"), 0));
    }

    // The real run: the tests door is the full declared run, fresh.
    let run = match (&declared, shell) {
        (Some(commands), Some(shell)) => run_declared_tests(root, commands, shell),
        _ => TestRun {
            ran_at: String::new(),
            green: false,
            undeclared: true,
            commands: Vec::new(),
            write_error: None,
        },
    };
    if let Some(msg) = &run.write_error {
        // Node: writeJsonAtomic throws -> main's catch -> emitError.
        return Ok(Out::Thrown(msg.clone()));
    }
    let report_doors = build_close_report_doors(root, feature)?;

    if !run.undeclared && !run.green {
        let failing: Vec<&CommandResult> =
            run.commands.iter().filter(|c| c.failure_excerpt.is_some()).collect();
        let first_line = first_failure_line(&run);
        let mut doors = vec![Door {
            door: "tests",
            blocking: true,
            detail: format!(
                "the declared test run is RED ({} of {} command(s) failed; record: {TEST_RESULTS_RELATIVE})",
                failing.len(),
                run.commands.len()
            ),
            command: Some("bee test"),
        }];
        doors.extend(report_doors);
        let mut result = Map::new();
        result.insert("feature".into(), Value::String(feature.to_string()));
        result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
        result.insert("ran_tests".into(), Value::Bool(true));
        let mut tests = Map::new();
        tests.insert("ran_at".into(), Value::String(run.ran_at.clone()));
        tests.insert("green".into(), Value::Bool(false));
        tests.insert(
            "commands".into(),
            Value::Array(run.commands.iter().map(command_result_value).collect()),
        );
        tests.insert("results".into(), Value::String(TEST_RESULTS_RELATIVE.into()));
        result.insert("tests".into(), Value::Object(tests));

        let mut lines = vec![format!(
            "Tests RED for \"{feature}\" — close stops at the tests door (record: {TEST_RESULTS_RELATIVE}):"
        )];
        lines.extend(render_test_command_lines(&run));
        for c in &failing {
            lines.push(format!(
                "--- {} (exit {}) ---\n{}",
                c.command,
                c.exit.map(|e| e.to_string()).unwrap_or_else(|| "spawn-failed".to_string()),
                c.failure_excerpt.clone().unwrap_or_default()
            ));
        }
        lines.push(format!(
            "next: the red is the work — fix it ({}), then re-run bee close --feature {feature}",
            first_line.unwrap_or_else(|| "see the excerpt above".to_string())
        ));
        return Ok(Out::Emit(Value::Object(result), lines.join("\n"), 1));
    }

    // Green (or no declared test path): what remains is the capture checklist.
    let tests_door = if run.undeclared {
        Door {
            door: "tests",
            blocking: false,
            detail: CLOSE_TESTS_UNDECLARED_DETAIL.to_string(),
            command: None,
        }
    } else {
        Door {
            door: "tests",
            blocking: false,
            detail: format!(
                "GREEN — {} command(s) passed (record: {TEST_RESULTS_RELATIVE})",
                run.commands.len()
            ),
            command: None,
        }
    };
    let scribing_detail = report_doors
        .iter()
        .find(|d| d.door == "scribing-debt")
        .map(|d| d.detail.clone())
        .unwrap_or_default();
    let queue_detail = report_doors
        .iter()
        .find(|d| d.door == "capture-queue")
        .map(|d| d.detail.clone())
        .unwrap_or_default();
    let mut doors = vec![tests_door];
    doors.extend(report_doors);

    let headline = if run.undeclared {
        format!(
            "No commands.test declared for \"{feature}\" — nothing gated close; declare commands.test in .bee/config.json to give close a test door."
        )
    } else {
        format!(
            "Tests GREEN for \"{feature}\" — {} command(s) passed (record: {TEST_RESULTS_RELATIVE}).",
            run.commands.len()
        )
    };
    let mut result = Map::new();
    result.insert("feature".into(), Value::String(feature.to_string()));
    result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
    result.insert("ran_tests".into(), Value::Bool(!run.undeclared));
    result.insert(
        "tests".into(),
        if run.undeclared {
            Value::Null
        } else {
            let mut tests = Map::new();
            tests.insert("ran_at".into(), Value::String(run.ran_at.clone()));
            tests.insert("green".into(), Value::Bool(true));
            tests.insert(
                "commands".into(),
                Value::Array(run.commands.iter().map(command_result_value).collect()),
            );
            tests.insert("results".into(), Value::String(TEST_RESULTS_RELATIVE.into()));
            Value::Object(tests)
        },
    );

    let mut lines = vec![headline];
    if !run.undeclared {
        lines.extend(render_test_command_lines(&run));
    }
    lines.push(format!(
        "Capture (deferred, decision c8e25271): scribing {scribing_detail}; capture queue {queue_detail}."
    ));
    lines.push(
        "next: done — capture is recorded as pending (run bee-capturing whenever; orient keeps the reminder)."
            .to_string(),
    );
    Ok(Out::Emit(Value::Object(result), lines.join("\n"), 0))
}

/// provenance: state.mjs readLane / lanePath / requireLaneFeature — the
/// DELEGATION slice. The only lane read on close's path is
/// bestScribingStampMs' `readLane(root, feature)`, which touches exactly ONE
/// file: `.bee/lanes/<feature>.json`. Absent (or a malformed feature name,
/// which lanePath throws on and readLane catches as "no lane") it is a
/// provable no-op, so a lane-using repo still runs close natively for every
/// feature that has no lane record of its own. When the file IS there, the
/// lane record's own `last_scribing_run` joins the threshold AND a corrupt
/// record prints a console.warn with a path.relative-derived string — both
/// unported (the blueprint's lane/workflow coverage debt), so that ONE
/// feature delegates.
///
/// Workflows are deliberately NOT part of this guard: nothing close reads
/// (readState / listCells / captureQueue / readConfig) consults
/// `.bee/runtime/workflows/`, so their presence changes no byte here.
fn feature_has_lane_record(root: &Path, feature: &str) -> bool {
    let trimmed = js_trim(feature);
    if trimmed.is_empty()
        || trimmed.contains('\\')
        || trimmed.contains('/')
        || trimmed.contains("..")
    {
        return false; // lanePath throws -> readLane returns null (fail-open)
    }
    root.join(".bee").join("lanes").join(format!("{trimmed}.json")).exists()
}

fn run_close(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(&flags, &["feature", "dry-run"]) {
        return None;
    }
    // validate(): a boolean-typed flag given as =value must be true/false.
    match flags.get("dry-run") {
        None | Some(FlagV::Present) => {}
        Some(FlagV::S(s)) if s == "true" || s == "false" => {}
        Some(FlagV::S(_)) => return None,
    }
    // validate(): --feature required; requireFlag also rejects ''/true.
    let feature = flags.req_str("feature")?.to_string();
    // `flags['dry-run'] === true`: only the flag-alone form is JS `true`.
    let dry_run = matches!(flags.get("dry-run"), Some(FlagV::Present));

    // ── everything that can still delegate happens BEFORE prelude, whose
    //    drift-cache write would swallow the Node re-run's drift line. ──────
    let cwd = std::env::current_dir().ok()?;
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::NeedsNode => return None,
        Roots::None => return Some(emit_no_root_error(&cwd, "close", use_json, t0)),
    };
    if feature_has_lane_record(&root, &feature) {
        return None;
    }
    let declared = declared_test_commands(&root).ok()?;
    let shell = if !dry_run && declared.is_some() {
        let s = posix_shell()?; // no POSIX sh — Node's cmd.exe fallback owns it
        ensure_dir(&root.join(".bee").join("logs")).ok()?;
        Some(s)
    } else {
        None
    };
    // Delegation pre-flight for the report doors: they are pure reads, so
    // computing them here (and again, for real, after the suite runs) can
    // only cost two cheap directory scans — but it means a corrupt store can
    // still hand the whole command to Node BEFORE a test suite is spent.
    build_close_report_doors(&root, &feature).ok()?;

    let ctx = match prelude("close", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    let out: R2<Out> = close_handler(&ctx.root, &feature, dry_run, declared, shell)
        .map_err(crate::verbs::reservations::Err2::from);
    finish(&ctx, out)
}

// ═══ routing ═══════════════════════════════════════════════════════════════

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    match args.first()?.to_str()? {
        "close" => {
            let toks: Vec<&str> =
                args[1..].iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
            if toks.iter().any(|t| *t == "--help") {
                return None; // Node renders command-scoped help
            }
            let (flags, use_json) = parse_flags(&toks)?;
            run_close(flags, use_json, t0)
        }
        "dispatch" => {
            if args.get(1)?.to_str()? != "prepare" {
                return None;
            }
            let toks: Vec<&str> =
                args[2..].iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
            if toks.iter().any(|t| *t == "--help") {
                return None;
            }
            let (flags, use_json) = parse_flags(&toks)?;
            run_dispatch_prepare(flags, use_json, t0)
        }
        _ => None,
    }
}

// ═══ kctx — VERBATIM LIFT of the knowledge-context port ════════════════════
//
// `bee knowledge context` is already ported in verbs/knowledge.rs, but every
// function the Learned-context block needs (bundleDir, listBundleMarkdown,
// parseFrontmatter, collectConcepts, normalizeBundleTarget,
// scoreCriticalRelevance, buildContextManifest) is PRIVATE to that module and
// that file may not be edited to widen them. Rather than re-implement the
// ranking — which would be a second, independently-drifting answer to the same
// question — the code below is a byte-for-byte lift of
// verbs/knowledge.rs lines 212-233, 316-627, 629-686, 846-951 and 1325-1772
// (commit-current at the time of this port). Its ultimate provenance is
// lib/knowledge.mjs: bundleDir / KEY_RE / RESERVED_BASENAMES /
// parseFrontmatter / listBundleMarkdown / normalizeBundleTarget /
// resolveInsideBundle / collectConcepts / beeOf / dirOf / CONTEXT_ESTIMATOR /
// CRITICAL_RELEVANCE / RELEVANCE_STOPWORDS / relevanceTokens / conceptBody /
// metaTextOf / scoreCriticalRelevance / buildContextManifest.
//
// `learned_context_agrees_with_the_knowledge_verb_port` (below) pins the lift
// to the shipped `bee knowledge context` verb on a fixture bundle, so a future
// edit to either copy that changes the answer fails the suite.
//
// R6 debt: promote these to a shared `crate::knowledge_context` module and
// delete this copy.
mod kctx {
    #![allow(dead_code, clippy::all)]

    use crate::jsjson;
    use crate::state::read_config_raw;
    use crate::verbs::reservations::js_trim;
    use serde_json::{Map, Number, Value};
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};

    /// provenance: verbs/knowledge.rs:167 bundle_dir (lib/knowledge.mjs
    /// bundleDir + the delegating slice of resolveProductRoot).
    pub(super) fn bundle_dir(root: &Path) -> Option<PathBuf> {
        let config = read_config_raw(root).ok()?;
        match config.get("product_root") {
            None | Some(Value::Null) => {}
            Some(Value::String(s)) if s.is_empty() => {}
            Some(_) => return None,
        }
        Some(root.join("docs").join("knowledge"))
    }

    pub(super) fn key_re_ok(key: &str) -> bool {
    // /^[A-Za-z_][A-Za-z0-9_.-]*$/
    let mut chars = key.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

    pub(super) fn is_reserved_basename(base: &str) -> bool {
    base == "index.md" || base == "log.md"
}

/// JS `\s` (same set String.prototype.trim strips) — via reservations.
    pub(super) fn js_is_space(c: char) -> bool {
    crate::verbs::reservations::js_is_ws(c)
}

    pub(super) fn js_quote_str(s: &str) -> String {
    jsjson::stringify(&Value::String(s.to_string()))
}


// ─── parser (accepts exactly the emitted subset; loud typed failure) ───────

    pub(super) enum Fm {
    Absent,
    Parsed {
        data: Map<String, Value>,
        block: String,
        body: String,
    },
    Failed {
        code: &'static str,
        message: String,
        line: usize,
    },
    /// A shape only V8 could decide (lone-surrogate escapes in a quoted
    /// scalar) — the whole command must delegate.
    NeedsNode,
}

    pub(super) fn fm_fail(code: &'static str, message: String, line: usize) -> Result<Value, Fm> {
    Err(Fm::Failed { code, message, line })
}

/// Lone-surrogate escape sniff (\uD800–\uDFFF): JSON.parse accepts them,
/// serde rejects — same heuristic feedback.rs uses for jsonl rows.
    pub(super) fn has_surrogate_escape(s: &str) -> bool {
    let b = s.as_bytes();
    for i in 0..b.len().saturating_sub(3) {
        if b[i] == b'\\'
            && (b[i + 1] == b'u' || b[i + 1] == b'U')
            && (b[i + 2] == b'd' || b[i + 2] == b'D')
            && matches!(b[i + 3], b'8' | b'9' | b'a'..=b'f' | b'A'..=b'F')
        {
            return true;
        }
    }
    false
}

    pub(super) fn parse_scalar_token(raw: &str, line_no: usize) -> Result<Value, Fm> {
    if raw == "true" {
        return Ok(Value::Bool(true));
    }
    if raw == "false" {
        return Ok(Value::Bool(false));
    }
    if raw.starts_with('"') {
        match serde_json::from_str::<Value>(raw) {
            Ok(Value::String(s)) => return Ok(Value::String(s)),
            Ok(_) => {
                return fm_fail("bad_quoted_string", "quoted value did not decode to a string".to_string(), line_no)
            }
            Err(_) => {
                if has_surrogate_escape(raw) {
                    return Err(Fm::NeedsNode);
                }
                return fm_fail(
                    "bad_quoted_string",
                    format!("quoted value {} is not one complete JSON string", js_quote_str(raw)),
                    line_no,
                );
            }
        }
    }
    if raw.starts_with('\'') {
        return fm_fail(
            "single_quoted_string",
            "single-quoted scalars are outside the emitted subset — use double quotes".to_string(),
            line_no,
        );
    }
    // /^[&*!|>%@`{}]/
    if matches!(raw.chars().next(), Some('&' | '*' | '!' | '|' | '>' | '%' | '@' | '`' | '{' | '}')) {
        return fm_fail(
            "unsupported_scalar",
            format!(
                "value starting with \"{}\" (anchor/alias/block/flow-map indicator) is outside the emitted subset",
                raw.chars().next().unwrap()
            ),
            line_no,
        );
    }
    Ok(Value::String(raw.to_string()))
}

    pub(super) fn parse_flow_list(raw: &str, line_no: usize) -> Result<Value, Fm> {
    if !raw.ends_with(']') {
        return fm_fail(
            "bad_flow_list",
            format!("flow list {} does not close with \"]\"", js_quote_str(raw)),
            line_no,
        );
    }
    let inner = js_trim(&raw[1..raw.len() - 1]);
    if inner.is_empty() {
        return Ok(Value::Array(Vec::new()));
    }
    let mut segments: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut escaped = false;
    for ch in inner.chars() {
        if in_quote {
            current.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_quote = false;
            }
        } else if ch == '"' {
            current.push(ch);
            in_quote = true;
        } else if ch == ',' {
            segments.push(std::mem::take(&mut current));
        } else {
            current.push(ch);
        }
    }
    if in_quote {
        return fm_fail("bad_flow_list", "unterminated quoted item inside flow list".to_string(), line_no);
    }
    segments.push(current);
    let mut value = Vec::new();
    for segment in &segments {
        let token = js_trim(segment);
        if token.is_empty() {
            return fm_fail("bad_flow_list", "empty item inside flow list".to_string(), line_no);
        }
        value.push(parse_scalar_token(token, line_no)?);
    }
    Ok(Value::Array(value))
}

    pub(super) fn parse_key_value_line(line: &str, target: &mut Map<String, Value>, line_no: usize, prefix: &str) -> Result<(), Fm> {
    let Some(sep) = line.find(": ") else {
        return fm_fail(
            "unrecognized_line",
            format!(
                "line {} is not \"key: value\", a \"bee:\" map header, or a closing \"---\"",
                js_quote_str(line)
            ),
            line_no,
        )
        .map(|_| ());
    };
    let key = &line[..sep];
    if !key_re_ok(key) {
        return fm_fail(
            "bad_key",
            format!("{} is not a legal frontmatter key", js_quote_str(key)),
            line_no,
        )
        .map(|_| ());
    }
    if target.contains_key(key) {
        return fm_fail("duplicate_key", format!("duplicate key \"{prefix}{key}\""), line_no).map(|_| ());
    }
    let raw = &line[sep + 2..];
    if raw.is_empty() {
        return fm_fail("empty_value", format!("key \"{prefix}{key}\" has no value after \": \""), line_no)
            .map(|_| ());
    }
    let parsed = if raw.starts_with('[') {
        parse_flow_list(raw, line_no)?
    } else {
        parse_scalar_token(raw, line_no)?
    };
    target.insert(key.to_string(), parsed);
    Ok(())
}

/// parseFrontmatter(text) — see lib/knowledge.mjs for the full contract.
    pub(super) fn parse_frontmatter(text: &str) -> Fm {
    let open_len = if text.starts_with("---\r\n") {
        5
    } else if text.starts_with("---\n") {
        4
    } else {
        return Fm::Absent;
    };

    let mut cursor = open_len;
    let mut block_end: Option<usize> = None;
    let mut inner_end = 0usize;
    while cursor <= text.len() {
        let nl = text[cursor..].find('\n').map(|p| p + cursor);
        let line_end = nl.unwrap_or(text.len());
        let mut line = &text[cursor..line_end];
        if let Some(stripped) = line.strip_suffix('\r') {
            line = stripped;
        }
        if line == "---" {
            inner_end = cursor;
            block_end = Some(nl.map(|p| p + 1).unwrap_or(text.len()));
            break;
        }
        let Some(nl) = nl else { break };
        cursor = nl + 1;
    }
    let Some(block_end) = block_end else {
        return Fm::Failed {
            code: "unclosed_frontmatter",
            message: "frontmatter opened with \"---\" but never closed".to_string(),
            line: 1,
        };
    };

    let block = text[..block_end].to_string();
    let body = text[block_end..].to_string();
    let inner_raw = &text[open_len..inner_end];
    let inner_lines: Vec<&str> = if inner_raw.is_empty() {
        Vec::new()
    } else {
        let mut v: Vec<&str> = inner_raw.split('\n').collect();
        v.pop();
        v
    };

    let mut data: Map<String, Value> = Map::new();
    let mut in_bee_map = false;
    let mut line_no = 1usize;
    for raw_line in inner_lines {
        line_no += 1;
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if line.is_empty() {
            return Fm::Failed {
                code: "blank_line",
                message: "blank line inside frontmatter is outside the emitted subset".to_string(),
                line: line_no,
            };
        }
        if line.contains('\t') {
            return Fm::Failed {
                code: "tab_in_frontmatter",
                message: "tab character inside frontmatter is outside the emitted subset".to_string(),
                line: line_no,
            };
        }
        if let Some(inner) = line.strip_prefix("  ") {
            if !in_bee_map {
                return Fm::Failed {
                    code: "unexpected_indent",
                    message: "indented line outside the \"bee:\" map".to_string(),
                    line: line_no,
                };
            }
            if inner.starts_with(' ') {
                return Fm::Failed {
                    code: "bad_indent",
                    message: "bee: map entries are indented exactly two spaces".to_string(),
                    line: line_no,
                };
            }
            let bee = data
                .get_mut("bee")
                .and_then(Value::as_object_mut)
                .expect("bee map exists while in_bee_map");
            match parse_key_value_line(inner, bee, line_no, "bee.") {
                Ok(()) => continue,
                Err(f) => return f,
            }
        }
        if line.starts_with(' ') {
            return Fm::Failed {
                code: "bad_indent",
                message: "root-level lines must not be indented".to_string(),
                line: line_no,
            };
        }
        in_bee_map = false;
        // /^([^:\s]+):$/ — a map header line.
        let header_key = line.strip_suffix(':').filter(|key| {
            !key.is_empty() && key.chars().all(|c| c != ':' && !js_is_space(c))
        });
        if let Some(key) = header_key {
            if !key_re_ok(key) {
                return Fm::Failed {
                    code: "bad_key",
                    message: format!("{} is not a legal frontmatter key", js_quote_str(key)),
                    line: line_no,
                };
            }
            if key != "bee" {
                return Fm::Failed {
                    code: "unsupported_map",
                    message: format!(
                        "nested map \"{key}:\" is outside the emitted subset (the only nested map is \"bee:\")"
                    ),
                    line: line_no,
                };
            }
            if data.contains_key("bee") {
                return Fm::Failed {
                    code: "duplicate_key",
                    message: "duplicate key \"bee\"".to_string(),
                    line: line_no,
                };
            }
            data.insert("bee".to_string(), Value::Object(Map::new()));
            in_bee_map = true;
            continue;
        }
        if let Err(f) = parse_key_value_line(line, &mut data, line_no, "") {
            return f;
        }
    }

    Fm::Parsed { data, block, body }
}

// ─── bundle walk (listBundleMarkdown — never leaves docs/knowledge/, D23) ──

/// lstat-level symlink test matching Node's dirent.isSymbolicLink(): on
/// Windows any reparse point (symlink OR junction) counts, like libuv.
    pub(super) fn is_symlinkish(path: &Path) -> bool {
    let Ok(md) = std::fs::symlink_metadata(path) else { return false };
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        (md.file_attributes() & 0x400) != 0 // FILE_ATTRIBUTE_REPARSE_POINT
    }
    #[cfg(not(windows))]
    {
        md.file_type().is_symlink()
    }
}

/// None => delegate (non-UTF-16-sortable names or unrepresentable OsStrings).
    pub(super) fn list_bundle_markdown(dir: &Path) -> Option<Vec<String>> {
    fn walk(abs: &Path, rel: &str, out: &mut Vec<String>) -> Option<()> {
        let entries = match std::fs::read_dir(abs) {
            Ok(rd) => rd,
            Err(_) => return Some(()),
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_str()?.to_string();
            let child_abs = entry.path();
            if is_symlinkish(&child_abs) {
                continue; // a symlink could escape the bundle — never follow (D23)
            }
            let Ok(ft) = entry.file_type() else { continue };
            let child_rel = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
            if ft.is_dir() {
                walk(&child_abs, &child_rel, out)?;
            } else if ft.is_file() && name.ends_with(".md") {
                out.push(child_rel);
            }
        }
        Some(())
    }
    let mut out = Vec::new();
    if dir.exists() {
        walk(dir, "", &mut out)?;
    }
    // JS Array#sort compares UTF-16 code units; UTF-8 byte order agrees below
    // U+E000 (supplementary chars sort before U+E000..U+FFFF under UTF-16).
    if out.iter().any(|rel| rel.chars().any(|c| c >= '\u{e000}')) {
        return None;
    }
    out.sort();
    Some(out)
}

    pub(super) fn read_file_lossy(path: &Path) -> std::io::Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

// ─── path resolution inside the bundle (resolveInsideBundle subset) ────────

/// resolveInsideBundle + normalizeBundleTarget: lexically resolve `target`
/// against the ABSOLUTE bundle `dir` exactly like path.resolve (pops through
/// '..' and re-entry, clamps at the filesystem root, case-sensitive prefix
/// compare), and return the bundle-relative path with '/' separators when the
/// result is a strict descendant of `dir`; None when it escapes (never
/// followed, D23). Err(()) => delegate (drive-letter / rooted shapes whose
/// win32 path.resolve semantics are not fully modeled here).
    pub(super) fn normalize_bundle_target(dir: &Path, target: &str) -> Result<Option<String>, ()> {
    if target.is_empty() {
        return Ok(None);
    }
    if target.contains(':') || target.starts_with('/') || target.starts_with('\\') {
        return Err(()); // drive-relative / rooted forms — Node decides
    }
    // The bundle dir's own normal components are the containment prefix
    // (path.resolve(dir) — dir is already absolute and '..'-free here).
    let base: Vec<String> = dir
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(os) => os.to_str().map(str::to_string),
            _ => None,
        })
        .collect();
    let mut stack: Vec<String> = base.clone();
    for seg in target.split(['/', '\\']) {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            stack.pop(); // at the root, path.resolve clamps — pop of empty is a no-op
        } else {
            stack.push(seg.to_string());
        }
    }
    if stack.len() <= base.len() || stack[..base.len()] != base[..] {
        return Ok(None); // not a strict descendant of the bundle dir
    }
    Ok(Some(stack[base.len()..].join("/")))
}

/// resolveInsideBundle for existence checks: absolute path when contained.
    pub(super) fn resolve_inside_bundle(dir: &Path, target: &str) -> Result<Option<PathBuf>, ()> {
    Ok(normalize_bundle_target(dir, target)?.map(|rel| join_rel(dir, &rel)))
}

// ─── concept inventory (collectConcepts) ───────────────────────────────────

    pub(super) struct Concept {
        pub(super) path: String,
        pub(super) data: Map<String, Value>,
}

/// None => delegate (walk/name issues, V8-only frontmatter).
    pub(super) fn collect_concepts(dir: &Path) -> Option<Vec<Concept>> {
    let mut concepts = Vec::new();
    for rel in list_bundle_markdown(dir)? {
        let base = rel.rsplit('/').next().unwrap_or(&rel);
        if is_reserved_basename(base) {
            continue;
        }
        let data = match read_file_lossy(&join_rel(dir, &rel)) {
            Err(_) => Map::new(), // unreadable: keep the row with empty data
            Ok(text) => match parse_frontmatter(&text) {
                Fm::Parsed { data, .. } => data,
                Fm::NeedsNode => return None,
                _ => Map::new(),
            },
        };
        concepts.push(Concept { path: rel, data });
    }
    Some(concepts)
}

    pub(super) fn join_rel(dir: &Path, rel: &str) -> PathBuf {
    let mut p = dir.to_path_buf();
    for seg in rel.split('/') {
        p.push(seg);
    }
    p
}

/// beeOf(data) — the bee map when it is a plain object, else empty.
    pub(super) fn bee_of(data: &Map<String, Value>) -> Map<String, Value> {
    match data.get("bee") {
        Some(Value::Object(m)) => m.clone(),
        _ => Map::new(),
    }
}

    pub(super) fn dir_of(rel: &str) -> &str {
    match rel.rfind('/') {
        Some(p) => &rel[..p],
        None => "",
    }
}

    pub(super) fn str_field<'a>(map: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    match map.get(key) {
        Some(Value::String(s)) if !s.is_empty() => Some(s),
        _ => None,
    }
}

// ─── checkBundle (D4/D13 + G14 layer 3) ────────────────────────────────────

// ─── context (buildContextManifest + relevance ranking) ────────────────────

    pub(super) const CONTEXT_ESTIMATOR: &str = "bytes/4";
    pub(super) const KEEP: usize = 20;
    pub(super) const FLOOR: usize = 3;
    pub(super) const META_WEIGHT: f64 = 0.25;
    pub(super) const BODY_WEIGHT: f64 = 1.0;
    pub(super) const TAG_WEIGHT: f64 = 0.05;
    pub(super) const AREA_WEIGHT: f64 = 0.05;
    pub(super) const ZERO_SIGNAL_MIN_POPULATION: usize = 10;
    pub(super) const ZERO_SIGNAL_MAX_RATIO: f64 = 0.5;

    pub(super) const RELEVANCE_STOPWORDS: &str = "a an the and or but if then else for of to in on at by is are was were be been being it its this that these those with without from as not no never always every each any all some one two three you your we our they their he she i me my do does did done can could should would may might must will shall have has had so than which who whom what when where why how more most less least very just only also into out up down over under again further once here there both few other own same too s t don now";

    pub(super) fn stopwords() -> HashSet<&'static str> {
    RELEVANCE_STOPWORDS.split(' ').collect()
}

/// relevanceTokens(text) — lowercase, [a-z0-9]+ runs, >2 chars, stopped,
/// crude singularization.
    pub(super) fn relevance_tokens(text: &str, stops: &HashSet<&'static str>) -> Vec<String> {
    let lower: String = text.to_lowercase();
    let mut out = Vec::new();
    for raw in lower.split(|c: char| !c.is_ascii_lowercase() && !c.is_ascii_digit()) {
        if raw.len() <= 2 || stops.contains(raw) {
            continue;
        }
        let token = if raw.len() > 4 && raw.ends_with('s') && !raw.ends_with("ss") {
            &raw[..raw.len() - 1]
        } else {
            raw
        };
        out.push(token.to_string());
    }
    out
}

/// Insertion-ordered unique token list (JS Set semantics for f64-sum order).
    pub(super) fn uniq(tokens: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for t in tokens {
        if seen.insert(t.clone()) {
            out.push(t);
        }
    }
    out
}

    pub(super) fn concept_body(dir: &Path, rel: &str) -> Option<String> {
    let raw = match read_file_lossy(&join_rel(dir, rel)) {
        Ok(t) => t,
        Err(_) => return Some(String::new()),
    };
    match parse_frontmatter(&raw) {
        Fm::Parsed { body, .. } => Some(body),
        Fm::NeedsNode => None,
        _ => Some(raw),
    }
}

    pub(super) fn meta_text_of(concept: &Concept) -> String {
    let t = match concept.data.get("title") {
        Some(v) if crate::verbs::reservations::truthy(v) => jsjson::js_to_string(v),
        _ => String::new(),
    };
    let d = match concept.data.get("description") {
        Some(v) if crate::verbs::reservations::truthy(v) => jsjson::js_to_string(v),
        _ => String::new(),
    };
    let tags = match concept.data.get("tags") {
        Some(Value::Array(items)) => items
            .iter()
            .map(|v| match v {
                Value::Null => String::new(),
                other => jsjson::js_to_string(other),
            })
            .collect::<Vec<_>>()
            .join(" "),
        _ => String::new(),
    };
    format!("{t} {d} {tags}")
}

/// Number(score.toFixed(6)) — display-precision rounding (divergence note in
/// the header covers the tie-rounding difference).
    pub(super) fn to_fixed6(x: f64) -> f64 {
    format!("{x:.6}").parse().unwrap_or(x)
}

    pub(super) fn score_critical_relevance(
    dir: &Path,
    criticals: &[&Concept],
    work: &Concept,
) -> Option<Vec<(String, f64)>> {
    let stops = stopwords();
    let mut fields: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    let mut df: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for concept in criticals {
        let meta = uniq(relevance_tokens(&meta_text_of(concept), &stops));
        let body = uniq(relevance_tokens(&concept_body(dir, &concept.path)?, &stops));
        let mut union_seen: HashSet<&String> = HashSet::new();
        for token in meta.iter().chain(body.iter()) {
            if union_seen.insert(token) {
                *df.entry(token.clone()).or_insert(0) += 1;
            }
        }
        fields.push((meta, body));
    }
    let population = criticals.len();
    let idf = |token: &str| ((population as f64 + 1.0) / (*df.get(token).unwrap_or(&0) as f64 + 1.0)).ln() + 1.0;

    let query: HashSet<String> = relevance_tokens(
        &format!("{} {}", meta_text_of(work), concept_body(dir, &work.path)?),
        &stops,
    )
    .into_iter()
    .collect();
    let work_bee = bee_of(&work.data);
    let work_tags: HashSet<String> = match work.data.get("tags") {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_lowercase())
            .collect(),
        _ => HashSet::new(),
    };
    let work_areas: HashSet<&str> = match work_bee.get("areas") {
        Some(Value::Array(items)) => items.iter().filter_map(|v| v.as_str()).collect(),
        _ => HashSet::new(),
    };

    let coverage = |set: &[String]| -> f64 {
        let mut hit = 0.0f64;
        let mut total = 0.0f64;
        for token in set {
            let weight = idf(token);
            total += weight;
            if query.contains(token) {
                hit += weight;
            }
        }
        if total == 0.0 {
            0.0
        } else {
            hit / total
        }
    };

    let mut scores = Vec::new();
    for (i, concept) in criticals.iter().enumerate() {
        let bee = bee_of(&concept.data);
        let tags = match concept.data.get("tags") {
            Some(Value::Array(items)) => items
                .iter()
                .filter(|v| matches!(v, Value::String(s) if work_tags.contains(&s.to_lowercase())))
                .count(),
            _ => 0,
        };
        let areas = match bee.get("areas") {
            Some(Value::Array(items)) => items
                .iter()
                .filter(|v| matches!(v, Value::String(s) if work_areas.contains(s.as_str())))
                .count(),
            _ => 0,
        };
        let (meta, body) = &fields[i];
        let score = TAG_WEIGHT * tags as f64
            + AREA_WEIGHT * areas as f64
            + META_WEIGHT * coverage(meta)
            + BODY_WEIGHT * coverage(body);
        scores.push((concept.path.clone(), to_fixed6(score)));
    }
    Some(scores)
}

    pub(super) fn num(v: f64) -> Value {
    Number::from_f64(v).map(Value::Number).unwrap_or(Value::Null)
}

    pub(super) enum ManifestOut {
    Built(Value),
    Thrown(String),
    NeedsNode,
}

    pub(super) fn build_context_manifest(dir: &Path, work: &str, budget: f64, budget_raw: &Value) -> ManifestOut {
    let work_id = js_trim(work);
    if work_id.is_empty() {
        return ManifestOut::Thrown("knowledge context: missing_work — --work <id> is required (D27).".to_string());
    }
    if !budget.is_finite() || budget < 0.0 {
        // Node's message JSON.stringify's the RAW flags.budget — the CLI's
        // string (quoted) or the lane-preset number, never the conversion.
        return ManifestOut::Thrown(format!(
            "knowledge context: bad_budget — --budget must be a non-negative token count, got {} (D27).",
            jsjson::stringify(budget_raw)
        ));
    }

    let Some(concepts) = collect_concepts(dir) else { return ManifestOut::NeedsNode };

    let work_concept = concepts.iter().find(|c| {
        matches!(c.data.get("type"), Some(Value::String(t)) if t == "bee.work-item")
            && matches!(bee_of(&c.data).get("id"), Some(Value::String(id)) if id == work_id)
    });
    let Some(work_concept) = work_concept else {
        return ManifestOut::Thrown(format!(
            "knowledge context: unknown_work — no bee.work-item concept in docs/knowledge/ carries bee.id \"{work_id}\" (D27)."
        ));
    };

    let mut ranked: Vec<(String, String)> = Vec::new(); // (rel, reason)
    let mut selected: HashSet<String> = HashSet::new();
    let by_path: std::collections::HashMap<&str, &Concept> =
        concepts.iter().map(|c| (c.path.as_str(), c)).collect();
    let select = |rel: &str, reason: String, ranked: &mut Vec<(String, String)>, selected: &mut HashSet<String>| {
        if selected.contains(rel) || !by_path.contains_key(rel) {
            return false;
        }
        selected.insert(rel.to_string());
        ranked.push((rel.to_string(), reason));
        true
    };

    // (1) the work item
    select(&work_concept.path, "work item".to_string(), &mut ranked, &mut selected);

    // (2) the plan sibling in the same work/<id>/ directory
    let work_dir = dir_of(&work_concept.path).to_string();
    let plan = concepts.iter().find(|c| {
        matches!(c.data.get("type"), Some(Value::String(t)) if t == "bee.plan") && dir_of(&c.path) == work_dir
    });
    if let Some(plan) = plan {
        select(&plan.path, format!("plan sibling in {work_dir}/"), &mut ranked, &mut selected);
    }

    // (3) required_context, transitive, BFS depth order, cycles deduped silently
    let mut queue: std::collections::VecDeque<(String, usize)> =
        ranked.iter().map(|(rel, _)| (rel.clone(), 0usize)).collect();
    while let Some((node_rel, depth)) = queue.pop_front() {
        let targets = match by_path.get(node_rel.as_str()).map(|c| bee_of(&c.data)) {
            Some(bee) => match bee.get("required_context") {
                Some(Value::Array(items)) => items.clone(),
                _ => continue,
            },
            None => continue,
        };
        for target in &targets {
            let Value::String(target) = target else { continue };
            let rel = match normalize_bundle_target(dir, target) {
                Ok(Some(rel)) => rel,
                Ok(None) => continue,
                Err(()) => return ManifestOut::NeedsNode,
            };
            if !by_path.contains_key(rel.as_str()) || selected.contains(&rel) {
                continue;
            }
            select(&rel, format!("required_context depth {} via {node_rel}", depth + 1), &mut ranked, &mut selected);
            queue.push_back((rel, depth + 1));
        }
    }

    // (4) the critical concepts, ranked by relevance and cut (G5/G11)
    let criticals: Vec<&Concept> = concepts
        .iter()
        .filter(|c| matches!(bee_of(&c.data).get("critical"), Some(Value::Bool(true))))
        .collect();
    let Some(relevance) = score_critical_relevance(dir, &criticals, work_concept) else {
        return ManifestOut::NeedsNode;
    };
    let score_of = |path: &str| -> f64 {
        relevance.iter().find(|(p, _)| p == path).map(|(_, s)| *s).unwrap_or(0.0)
    };
    let zero_signal_count = criticals.iter().filter(|c| score_of(&c.path) == 0.0).count();
    if criticals.len() >= ZERO_SIGNAL_MIN_POPULATION
        && (zero_signal_count as f64) > criticals.len() as f64 * ZERO_SIGNAL_MAX_RATIO
    {
        return ManifestOut::Thrown(format!(
            "knowledge context: zero_signal — {zero_signal_count} of {} bee.critical concepts score 0 against work item \"{work_id}\", above the pinned {} ratio. A ranking where most items tie at zero is a path sort wearing a relevance label — widen the work item's description/body, or fix the ranking, but do not ship this order (G11).",
            criticals.len(),
            jsjson::js_f64_to_string(ZERO_SIGNAL_MAX_RATIO)
        ));
    }
    let mut ranked_criticals: Vec<&&Concept> = criticals.iter().collect();
    ranked_criticals.sort_by(|a, b| {
        score_of(&b.path)
            .partial_cmp(&score_of(&a.path))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
    });
    let mut excluded: Vec<Value> = Vec::new();
    let mut floor_paths: Vec<String> = Vec::new();
    let mut kept = 0usize;
    for (index, concept) in ranked_criticals.iter().enumerate() {
        let rank = index + 1;
        let score = score_of(&concept.path);
        if selected.contains(&concept.path) {
            continue; // already in via required_context — never re-cut
        }
        if kept >= KEEP {
            let mut m = Map::new();
            m.insert("path".into(), Value::String(format!("docs/knowledge/{}", concept.path)));
            m.insert("score".into(), num(score));
            m.insert(
                "reason".into(),
                Value::String(format!(
                    "below the relevance cut — rank {rank} of {}, keep {KEEP} (G5)",
                    ranked_criticals.len()
                )),
            );
            excluded.push(Value::Object(m));
            continue;
        }
        let is_floor = kept < FLOOR;
        if is_floor {
            floor_paths.push(format!("docs/knowledge/{}", concept.path));
        }
        select(
            &concept.path,
            format!(
                "critical pattern (relevance {}, rank {rank} of {}{})",
                jsjson::js_f64_to_string(score),
                ranked_criticals.len(),
                if is_floor { ", floor" } else { "" }
            ),
            &mut ranked,
            &mut selected,
        );
        kept += 1;
    }

    // (5) decisions whose areas overlap the work item's areas
    let work_bee_map = bee_of(&work_concept.data);
    let work_areas: Vec<&str> = match work_bee_map.get("areas") {
        Some(Value::Array(items)) => items.iter().filter_map(|v| v.as_str()).collect(),
        _ => Vec::new(),
    };
    for concept in &concepts {
        if !matches!(concept.data.get("type"), Some(Value::String(t)) if t == "bee.decision") {
            continue;
        }
        let areas = match bee_of(&concept.data).get("areas") {
            Some(Value::Array(items)) => items.clone(),
            _ => Vec::new(),
        };
        let overlap: Vec<String> = areas
            .iter()
            .filter(|a| matches!(a, Value::String(s) if work_areas.contains(&s.as_str())))
            .map(jsjson::js_to_string)
            .collect();
        if overlap.is_empty() {
            continue;
        }
        select(&concept.path, format!("decision for area {}", overlap.join(", ")), &mut ranked, &mut selected);
    }

    struct Sized {
        repo_rel: String,
        reason: String,
        bytes: u64,
        est: f64,
        floor: bool,
    }
    let sized: Vec<Sized> = ranked
        .iter()
        .map(|(rel, reason)| {
            let repo_rel = format!("docs/knowledge/{rel}");
            let bytes = std::fs::metadata(join_rel(dir, rel)).map(|m| m.len()).unwrap_or(0);
            let est = (bytes as f64 / 4.0).ceil();
            let floor = floor_paths.contains(&repo_rel);
            Sized { repo_rel, reason: reason.clone(), bytes, est, floor }
        })
        .collect();

    let floor_cost: f64 = sized.iter().filter(|s| s.floor).map(|s| s.est).sum();
    let rank_one_cost = sized.first().map(|s| s.est).unwrap_or(0.0);
    let mut reserve = (budget - rank_one_cost).min(floor_cost).max(0.0);
    let mut available = budget - reserve;

    let mut entries: Vec<Value> = Vec::new();
    let mut truncated: Vec<String> = Vec::new();
    let mut total_est = 0.0f64;
    let mut cutting = false;
    for item in &sized {
        if item.floor {
            if item.est > reserve {
                truncated.push(item.repo_rel.clone());
                continue;
            }
            reserve -= item.est;
            total_est += item.est;
        } else {
            if cutting || item.est > available {
                cutting = true;
                truncated.push(item.repo_rel.clone());
                continue;
            }
            available -= item.est;
            total_est += item.est;
        }
        let mut m = Map::new();
        m.insert("path".into(), Value::String(item.repo_rel.clone()));
        m.insert("bytes".into(), Value::Number(Number::from(item.bytes)));
        m.insert("est_tokens".into(), num(item.est));
        m.insert("reason".into(), Value::String(item.reason.clone()));
        entries.push(Value::Object(m));
    }

    let decisions: Vec<Value> = match bee_of(&work_concept.data).get("decisions") {
        Some(Value::Array(items)) => items.iter().filter(|v| v.is_string()).cloned().collect(),
        _ => Vec::new(),
    };

    // CONSERVATION (G11)
    let accounted: HashSet<String> = entries
        .iter()
        .filter_map(|e| e.get("path").and_then(Value::as_str).map(str::to_string))
        .chain(truncated.iter().cloned())
        .chain(excluded.iter().filter_map(|e| e.get("path").and_then(Value::as_str).map(str::to_string)))
        .collect();
    let lost: Vec<String> = criticals
        .iter()
        .map(|c| format!("docs/knowledge/{}", c.path))
        .filter(|repo_rel| !accounted.contains(repo_rel))
        .collect();
    if !lost.is_empty() {
        return ManifestOut::Thrown(format!(
            "knowledge context: conservation — {} bee.critical concept(s) were neither included, truncated nor excluded: {} (G11). This is a bug in the ranking, not a condition of the bundle.",
            lost.len(),
            lost.join(", ")
        ));
    }

    let mut manifest = Map::new();
    manifest.insert("work".into(), Value::String(work_id.to_string()));
    manifest.insert("decisions".into(), Value::Array(decisions));
    manifest.insert("budget".into(), num(budget));
    manifest.insert("estimator".into(), Value::String(CONTEXT_ESTIMATOR.to_string()));
    manifest.insert("total_est".into(), num(total_est));
    manifest.insert("entries".into(), Value::Array(entries));
    manifest.insert("truncated".into(), Value::Array(truncated.into_iter().map(Value::String).collect()));
    manifest.insert("excluded".into(), Value::Array(excluded));
    manifest.insert("floor".into(), Value::Array(floor_paths.into_iter().map(Value::String).collect()));
    manifest.insert("critical_total".into(), Value::Number(Number::from(criticals.len())));
    manifest.insert("zero_signal_count".into(), Value::Number(Number::from(zero_signal_count)));
    ManifestOut::Built(Value::Object(manifest))
}

}


// ═══ tests ═════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── fixtures ───────────────────────────────────────────────────────────

    fn w(root: &Path, rel: &str, body: &str) {
        let file = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, body).unwrap();
    }

    fn repo(tmp: &tempfile::TempDir, config: &str) -> PathBuf {
        let root = tmp.path().to_path_buf();
        w(&root, ".bee/onboarding.json", "{\"version\":1}");
        w(&root, ".bee/config.json", config);
        root
    }

    // ── C4: the prompt byte-identity pin ───────────────────────────────────

    /// The compiled-in templates MUST be the checked-out prompts/*.md bytes.
    /// This is contract C4 restated for the Rust runtime: edit a prompt file
    /// and the binary must be rebuilt; break the include_str! paths and this
    /// fails.
    #[test]
    fn embedded_prompts_are_the_checked_out_files_byte_for_byte() {
        let prompts = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
            .join("bee")
            .join("prompts");
        for (name, embedded) in [
            ("worker-cell", PROMPT_WORKER_CELL),
            ("gather", PROMPT_GATHER),
            ("reviewer", PROMPT_REVIEWER),
            ("advisor", PROMPT_ADVISOR),
        ] {
            let disk = std::fs::read(prompts.join(format!("{name}.md")))
                .unwrap_or_else(|e| panic!("prompts/{name}.md unreadable: {e}"));
            assert_eq!(
                embedded.as_bytes(),
                disk.as_slice(),
                "prompts/{name}.md drifted from the compiled-in copy"
            );
        }
    }

    /// loadPrompt's normalization: CRLF -> LF, exactly ONE trailing newline
    /// stripped (a template ending in two newlines keeps one).
    #[test]
    fn load_prompt_normalizes_line_endings_like_node() {
        assert_eq!(normalize_template("a\r\nb\r\n"), "a\nb");
        assert_eq!(normalize_template("a\nb\n\n"), "a\nb\n");
        assert_eq!(normalize_template("a\nb"), "a\nb");
        // The real worker-cell template ends with `{{/if}}\n` -> `{{/if}}`.
        assert!(load_prompt("worker-cell").unwrap().ends_with("{{/if}}"));
        assert!(!load_prompt("gather").unwrap().ends_with('\n'));
    }

    /// The `{{#if}}` block grammar: a dropped block leaves ZERO residue bytes
    /// (the preceding newline goes with it), a kept block splices its inner
    /// lines in exactly.
    #[test]
    fn render_conditional_blocks_leave_no_residue() {
        let t = "head\n{{#if x}}\nkept {{v}}\n{{/if}}\ntail";
        assert_eq!(render(t, &[("x", ""), ("v", "V")]).unwrap(), "head\ntail");
        assert_eq!(
            render(t, &[("x", "yes"), ("v", "V")]).unwrap(),
            "head\nkept V\ntail"
        );
    }

    #[test]
    fn render_refuses_nesting_and_missing_placeholders() {
        let nested = "a\n{{#if x}}\n{{#if y}}\nz\n{{/if}}\n{{/if}}\nb";
        let err = render(nested, &[("x", "1"), ("y", "1")]).unwrap_err();
        assert!(err.contains("nested"), "{err}");
        let err = render("a {{who}} b", &[]).unwrap_err();
        assert_eq!(
            err,
            "prompt-renderer: no value supplied for placeholder {{who}}."
        );
    }

    /// A first-dispatch cell with neither machine block renders byte-identically
    /// to the unconditional template — the invariant the C4 pin protects.
    #[test]
    fn worker_cell_without_machine_blocks_matches_the_plain_template() {
        let template = load_prompt("worker-cell").unwrap();
        let rendered = render(
            &template,
            &[
                ("worker", "w"),
                ("cell_id", "c-1"),
                ("feature", "f"),
                ("cell_json", "{}"),
                ("learned_context", ""),
                ("prior_rounds", ""),
            ],
        )
        .unwrap();
        assert!(!rendered.contains("Learned context"));
        assert!(!rendered.contains("Prior rounds"));
        assert!(rendered.starts_with("Nickname (reservation identity): w\n"));
        // Zero residue: both block markers and their newlines are gone.
        assert!(!rendered.contains("{{"));
        assert!(rendered.contains("- docs/history/f/plan.md (when present)\n\nContract:\n"));
    }

    // ── oneLine ────────────────────────────────────────────────────────────

    #[test]
    fn one_line_collapses_whitespace_and_ellipsises() {
        assert_eq!(one_line(Some(&json!("  a \n b  ")), 140), "a b");
        assert_eq!(one_line(None, 140), "");
        assert_eq!(one_line(Some(&Value::Null), 140), "");
        assert_eq!(one_line(Some(&json!(42)), 140), "42");
        let long = "x".repeat(60);
        assert_eq!(
            one_line(Some(&json!(long)), 40),
            format!("{}...", "x".repeat(37))
        );
    }

    // ── prior rounds ───────────────────────────────────────────────────────

    #[test]
    fn prior_rounds_orders_chronologically_and_skips_passes() {
        let cell = json!({
            "id": "c-1",
            "trace": {
                "capped_at": "2026-01-05T00:00:00.000Z",
                "attempts": [
                    {"at": "2026-01-03T00:00:00.000Z", "worker": "w2", "verdict": "tests-red", "note": "AssertionError: 1 != 2"},
                    {"at": "2026-01-01T00:00:00.000Z", "worker": "w0", "verdict": "fail", "failure_signature": "sig"},
                    {"at": "2026-01-04T00:00:00.000Z", "worker": "w3", "verdict": "pass"},
                    {"at": "2026-01-02T00:00:00.000Z", "verdict": "blocked"}
                ],
                "deviations": ["renamed the helper", "  "],
                "semantic_judge": [{"recorded_at": "2026-01-06T00:00:00.000Z", "verdict": "NEEDS_REVISION"}]
            }
        });
        assert_eq!(
            prior_round_event_lines(&cell),
            vec![
                "- w0 failed verify: failure signature sig",
                "- (unknown worker) blocked: failure signature (none recorded)",
                "- w2 tests red: AssertionError: 1 != 2",
                "- (prior worker) deviation: renamed the helper",
                "- (judge) consult: NEEDS_REVISION",
            ]
        );
        // A cell with no history produces NO lines (first-dispatch parity).
        assert!(prior_round_event_lines(&json!({"id": "c-2"})).is_empty());
    }

    #[test]
    fn prior_rounds_elides_the_oldest_past_twelve() {
        let attempts: Vec<Value> = (1..=15)
            .map(|i| json!({"at": format!("2026-01-{i:02}T00:00:00.000Z"), "worker": format!("w{i}"), "verdict": "blocked", "note": "n"}))
            .collect();
        let lines = prior_round_event_lines(&json!({"trace": {"attempts": attempts}}));
        assert_eq!(lines.len(), PRIOR_ROUNDS_MAX_EVENT_LINES);
        assert_eq!(
            lines[0],
            "- (4 earlier event(s) elided — the cell record holds the rest)"
        );
        assert_eq!(lines[1], "- w5 blocked: n");
        assert_eq!(lines[11], "- w15 blocked: n");
    }

    #[test]
    fn timeless_events_sink_to_the_end_in_insertion_order() {
        let cell = json!({"trace": {"attempts": [
            {"worker": "no-ts", "verdict": "blocked", "note": "a"},
            {"at": "2026-01-01T00:00:00.000Z", "worker": "dated", "verdict": "blocked", "note": "b"}
        ]}});
        assert_eq!(
            prior_round_event_lines(&cell),
            vec!["- dated blocked: b", "- no-ts blocked: a"]
        );
    }

    // ── claim-ownership guard ──────────────────────────────────────────────

    #[test]
    fn claim_ownership_refusals_are_byte_faithful() {
        let open = json!({"id": "c-1", "status": "open"});
        let o = check_cell_claim_ownership(&open, "w");
        assert!(!o.ok);
        assert_eq!(o.code, Some("not_claimed"));
        assert_eq!(o.reason, "cell \"c-1\" is \"open\", not \"claimed\" — dispatch prepare requires a claimed cell (run bee.mjs cells claim or cells claim-next first). Pass --force-ownership to override (audited).");

        let foreign = json!({"id": "c-1", "status": "claimed", "trace": {"worker": "other"}});
        let o = check_cell_claim_ownership(&foreign, "w");
        assert_eq!(o.code, Some("not_owner"));
        assert_eq!(o.owner, json!("other"));
        assert_eq!(o.reason, "cell \"c-1\" is claimed by worker \"other\" — \"w\" does not own this claim. Pass --force-ownership to override (audited).");

        // A claimed cell with no trace.worker reads "(unknown)".
        let orphan = json!({"id": "c-1", "status": "claimed"});
        assert!(check_cell_claim_ownership(&orphan, "w")
            .reason
            .contains("worker \"(unknown)\""));

        let mine = json!({"id": "c-1", "status": "claimed", "trace": {"worker": "w"}});
        assert!(check_cell_claim_ownership(&mine, "w").ok);
    }

    // ── tier resolution ────────────────────────────────────────────────────

    fn models_from(raw: &str) -> Map<String, Value> {
        normalize_models(Some(&serde_json::from_str::<Value>(raw).unwrap()))
    }

    #[test]
    fn resolve_tier_covers_every_documented_slot_shape() {
        let m = models_from("{}");
        // Defaults (DEFAULT_MODELS).
        assert_eq!(
            resolve_tier(&m, "generation", "claude", false),
            Resolved::Model { model: "sonnet".into(), effort: None }
        );
        assert_eq!(
            resolve_tier(&m, "review", "claude", true),
            Resolved::Model { model: "opus".into(), effort: None }
        );
        // codex defaults are null -> budget; review falls back to generation
        // (also null) -> budget.
        assert_eq!(resolve_tier(&m, "generation", "codex", false), Resolved::Budget);
        assert_eq!(resolve_tier(&m, "review", "codex", true), Resolved::Budget);
        // ceiling is never configured.
        assert_eq!(resolve_tier(&m, "ceiling", "claude", false), Resolved::Inherit);
        // An unknown slot ('advisor') coerces to generation — the trap
        // resolveAdvisor exists to avoid.
        assert_eq!(
            resolve_tier(&m, "advisor", "claude", true),
            Resolved::Model { model: "sonnet".into(), effort: None }
        );

        // review: null falls back to the generation tier BEFORE the cli check.
        let m = models_from(
            r#"{"claude":{"generation":{"kind":"cli","command":"glm run"},"review":null}}"#,
        );
        assert_eq!(
            resolve_tier(&m, "review", "claude", true),
            Resolved::Cli { command: "glm run".into() }
        );
        // cli + cell purpose -> typed refusal naming the RESOLVED slot.
        assert_eq!(
            resolve_tier(&m, "review", "claude", false),
            Resolved::Refused { slot: "review".into() }
        );

        // {model, effort}
        let m = models_from(r#"{"claude":{"generation":{"model":"opus","effort":"high"}}}"#);
        assert_eq!(
            resolve_tier(&m, "generation", "claude", false),
            Resolved::Model { model: "opus".into(), effort: Some("high".into()) }
        );
        // An invalid effort is dropped by normalize.
        let m = models_from(r#"{"claude":{"generation":{"model":"opus","effort":"turbo"}}}"#);
        assert_eq!(
            resolve_tier(&m, "generation", "claude", false),
            Resolved::Model { model: "opus".into(), effort: None }
        );

        // native leaf + explicit-only cli fallback composite
        let m = models_from(
            r#"{"codex":{"generation":{"primary":{"kind":"native","model":"gpt-5","effort":"high"},"fallback_policy":"explicit-only","fallback":{"kind":"cli","command":"codex exec"}}}}"#,
        );
        assert_eq!(
            resolve_tier(&m, "generation", "codex", false),
            Resolved::Native {
                model: "gpt-5".into(),
                effort: Some("high".into()),
                fork_turns: "none".into(),
                agent_type: "worker".into(),
                fallback: Some("codex exec".into()),
            }
        );
        // Without the policy string the fallback is stripped (no silent
        // native->cli switching).
        let m = models_from(
            r#"{"codex":{"generation":{"primary":{"kind":"native","model":"gpt-5"},"fallback":{"kind":"cli","command":"codex exec"}}}}"#,
        );
        match resolve_tier(&m, "generation", "codex", false) {
            Resolved::Native { fallback, .. } => assert_eq!(fallback, None),
            other => panic!("expected native, got {other:?}"),
        }
    }

    #[test]
    fn resolve_advisor_never_falls_back() {
        // Unset -> None (not budget, not generation).
        assert_eq!(resolve_advisor(&models_from("{}"), "claude"), None);
        assert_eq!(
            resolve_advisor(&models_from(r#"{"claude":{"advisor":"opus"}}"#), "claude"),
            Some(Resolved::Model { model: "opus".into(), effort: None })
        );
        // An explicit null is still "no advisor".
        assert_eq!(
            resolve_advisor(&models_from(r#"{"claude":{"advisor":null}}"#), "claude"),
            None
        );
        // An unknown runtime coerces to claude.
        assert_eq!(
            resolve_advisor(&models_from(r#"{"claude":{"advisor":"opus"}}"#), "banana"),
            Some(Resolved::Model { model: "opus".into(), effort: None })
        );
    }

    // ── economics ──────────────────────────────────────────────────────────

    #[test]
    fn derive_economics_matches_the_honest_mapping() {
        let model = Resolved::Model { model: "sonnet".into(), effort: None };
        let e = derive_economics("claude-agent", "generation", Some("sonnet"), &model, false);
        assert_eq!(
            jsjson::stringify(&Value::Object(e)),
            r#"{"logical_tier":"generation","requested_model":"sonnet","effective_model":"sonnet","effective_model_status":"pinned","channel":"claude-agent","enforcement":"model-param"}"#
        );
        // codex-native without a confirmed override is ALWAYS
        // inherited-or-unknown, whatever the tier resolves to.
        let e = derive_economics("codex-native", "generation", None, &Resolved::Budget, false);
        assert_eq!(
            jsjson::stringify(&Value::Object(e)),
            r#"{"logical_tier":"generation","requested_model":null,"effective_model":null,"effective_model_status":"inherited-or-unknown","channel":"codex-native","enforcement":"prompt-budget"}"#
        );
        // A confirmed native override: native-requested, effective_model still
        // null (catalog-accepted is not runtime-confirmed, D7).
        let native = Resolved::Native {
            model: "gpt-5".into(),
            effort: None,
            fork_turns: "none".into(),
            agent_type: "worker".into(),
            fallback: None,
        };
        let e = derive_economics("codex-native", "generation", None, &native, true);
        assert_eq!(
            jsjson::stringify(&Value::Object(e)),
            r#"{"logical_tier":"generation","requested_model":"gpt-5","effective_model":null,"effective_model_status":"native-requested","channel":"codex-native","enforcement":"native-model-param"}"#
        );
        // cli-exec never reports a requested_model.
        let cli = Resolved::Cli { command: "glm run".into() };
        let e = derive_economics("cli-exec", "generation", None, &cli, false);
        assert_eq!(
            jsjson::stringify(&Value::Object(e)),
            r#"{"logical_tier":"generation","requested_model":null,"effective_model":null,"effective_model_status":"unverified","channel":"cli-exec","enforcement":"cli-command"}"#
        );
    }

    #[test]
    fn pinned_types_match_the_rendered_bee_agents() {
        assert_eq!(pinned_agent_type("generation"), "bee-gather");
        assert_eq!(pinned_agent_type("extraction"), "bee-extract");
        assert_eq!(pinned_agent_type("review"), "bee-review");
        // 'advisor' has no rendered agent — `|| 'general-purpose'`.
        assert_eq!(pinned_agent_type("advisor"), "general-purpose");
    }

    // ── prepareDispatch envelopes ──────────────────────────────────────────

    #[test]
    fn gather_envelope_is_the_claude_agent_shape() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        let out =
            prepare_dispatch(&root, "claude", "gather", None, None, false, None, false).unwrap();
        let Prepared::Value(v) = out else { panic!("expected an envelope") };
        assert_eq!(v.get("tool"), Some(&json!("Agent")));
        let p = v.get("payload").unwrap();
        assert_eq!(p.get("subagent_type"), Some(&json!("bee-gather")));
        assert_eq!(p.get("model"), Some(&json!("sonnet")));
        assert_eq!(p.get("description"), Some(&json!("gather (sonnet)")));
        // The marker anchors at the very start of the prompt.
        let prompt = p.get("prompt").unwrap().as_str().unwrap();
        assert!(prompt.starts_with("[bee-tier: generation]\nGather: locate and digest"));
        // The prepare-time record is NOT written on a non-recording pass.
        assert!(!root.join(".bee/logs/dispatch.jsonl").exists());
    }

    #[test]
    fn recording_pass_appends_exactly_one_prepare_line() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        prepare_dispatch(&root, "claude", "gather", None, None, false, None, true).unwrap();
        let log = std::fs::read_to_string(root.join(".bee/logs/dispatch.jsonl")).unwrap();
        assert_eq!(log.lines().count(), 1);
        let line: Value = serde_json::from_str(log.lines().next().unwrap()).unwrap();
        assert_eq!(line.get("source"), Some(&json!("prepare")));
        assert_eq!(line.get("kind"), Some(&json!("gather")));
        assert_eq!(line.get("cell"), Some(&Value::Null));
        assert_eq!(line.get("channel"), Some(&json!("claude-agent")));
        assert_eq!(line.get("enforcement"), Some(&json!("model-param")));
    }

    #[test]
    fn advisor_not_configured_is_a_typed_refusal_not_a_throw() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "advisor", None, None, false, None, false).unwrap()
        else {
            panic!("expected a refusal value")
        };
        assert_eq!(
            jsjson::stringify(&v),
            r#"{"ok":false,"reason":"advisor_not_configured","fix":"set models.claude.advisor in .bee/config.json to enable an advisor consult (resolveAdvisor never falls back to another tier)."}"#
        );
    }

    #[test]
    fn cli_shaped_generation_refuses_for_cell_and_serves_gather() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"models":{"claude":{"generation":{"kind":"cli","command":"glm run"}}}}"#,
        );
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","status":"claimed","trace":{"worker":"w"}}"#,
        );

        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, false)
                .unwrap()
        else {
            panic!()
        };
        assert_eq!(v.get("reason"), Some(&json!("cli_tier_gather_only")));
        assert_eq!(v.get("slot"), Some(&json!("generation")));
        assert_eq!(v.get("fix"), Some(&json!(CLI_REFUSAL_FIX)));

        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "gather", None, None, false, None, false).unwrap()
        else {
            panic!()
        };
        assert_eq!(v.get("tool"), Some(&json!("Bash")));
        assert_eq!(v.get("payload").unwrap().get("command"), Some(&json!("glm run")));
        assert!(v
            .get("payload")
            .unwrap()
            .get("stdin")
            .unwrap()
            .as_str()
            .unwrap()
            .starts_with("Gather:"));
    }

    #[test]
    fn malformed_calls_throw_with_node_wording() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        let thrown = |o: Prepared| match o {
            Prepared::Thrown(m) => m,
            _ => panic!("expected a throw"),
        };
        assert_eq!(
            thrown(
                prepare_dispatch(&root, "claude", "cell", None, Some("w"), false, None, false)
                    .unwrap()
            ),
            "dispatch prepare: --cell is required when --kind cell."
        );
        assert_eq!(
            thrown(
                prepare_dispatch(
                    &root, "claude", "cell", Some("ghost"), Some("w"), false, None, false
                )
                .unwrap()
            ),
            "dispatch prepare: cell \"ghost\" not found."
        );
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","status":"claimed","trace":{"worker":"w"}}"#,
        );
        assert_eq!(
            thrown(
                prepare_dispatch(
                    &root, "claude", "cell", Some("c-1"), Some("   "), false, None, false
                )
                .unwrap()
            ),
            "dispatch prepare: --worker is required when --kind cell."
        );
    }

    #[test]
    fn force_ownership_always_leaves_an_audit_line() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","status":"claimed","trace":{"worker":"owner"}}"#,
        );
        // Forced past a real conflict.
        let Prepared::Value(v) = prepare_dispatch(
            &root, "claude", "cell", Some("c-1"), Some("thief"), true, None, true,
        )
        .unwrap() else {
            panic!()
        };
        let ov = v.get("ownership_override").unwrap();
        assert_eq!(ov.get("bypassed"), Some(&json!(true)));
        assert_eq!(ov.get("code"), Some(&json!("not_owner")));
        assert_eq!(ov.get("owner_bypassed"), Some(&json!("owner")));
        assert_eq!(ov.get("transferred"), Some(&json!(false)));
        // Forced with NO conflict still audits (msh-4 mirror).
        let Prepared::Value(v) = prepare_dispatch(
            &root, "claude", "cell", Some("c-1"), Some("owner"), true, None, false,
        )
        .unwrap() else {
            panic!()
        };
        let ov = v.get("ownership_override").unwrap();
        assert_eq!(ov.get("bypassed"), Some(&json!(false)));
        assert_eq!(ov.get("code"), Some(&Value::Null));
        // The audited override rides the dispatch record too.
        let log = std::fs::read_to_string(root.join(".bee/logs/dispatch.jsonl")).unwrap();
        assert!(log.contains("\"ownership_override\""));
    }

    #[test]
    fn native_unavailable_refuses_rather_than_downgrading() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"models":{"codex":{"generation":{"kind":"native","model":"gpt-5"}}}}"#,
        );
        // No confirmed override, no configured fallback -> typed refusal.
        let Prepared::Value(v) = prepare_dispatch(
            &root,
            "codex",
            "gather",
            None,
            None,
            false,
            Some(NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY),
            false,
        )
        .unwrap() else {
            panic!()
        };
        assert_eq!(
            jsjson::stringify(&v),
            r#"{"ok":false,"type":"refused","reason":"native_unavailable","detail":"native_budget_only"}"#
        );
        // A confirmed override emits the spawn_agent model-override payload.
        let Prepared::Value(v) = prepare_dispatch(
            &root,
            "codex",
            "gather",
            None,
            None,
            false,
            Some(NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE),
            false,
        )
        .unwrap() else {
            panic!()
        };
        assert_eq!(v.get("transport"), Some(&json!("native-override")));
        let p = v.get("payload").unwrap();
        assert_eq!(p.get("agent_type"), Some(&json!("worker")));
        assert_eq!(p.get("model"), Some(&json!("gpt-5")));
        assert_eq!(p.get("fork_turns"), Some(&json!("none")));
    }

    #[test]
    fn absent_probe_record_classifies_budget_only_without_a_subprocess() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        assert_eq!(
            native_transport_classification(&root).unwrap(),
            NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY
        );
        // Corrupt / schema-mismatched records short-circuit the same way.
        w(&root, ".bee/native-transport-probe.json", "{not json");
        assert_eq!(
            native_transport_classification(&root).unwrap(),
            NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY
        );
        w(&root, ".bee/native-transport-probe.json", r#"{"schema":"other/9"}"#);
        assert_eq!(
            native_transport_classification(&root).unwrap(),
            NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY
        );
        // A LIVE record needs codex-cli probes — delegate.
        w(
            &root,
            ".bee/native-transport-probe.json",
            r#"{"schema":"native-transport-probe/1"}"#,
        );
        assert!(native_transport_classification(&root).is_err());
    }

    // ── learned context ────────────────────────────────────────────────────

    /// The knowledge-bundle fixture, shared by the manifest cross-check and
    /// the learned-context tests. Byte-stable on purpose: the golden manifest
    /// below quotes its exact byte sizes.
    fn bundle_fixture(root: &Path) {
        w(root, "docs/knowledge/index.md", "# Knowledge\n\n## Critical patterns\n\n- none yet\n");
        w(root, "docs/knowledge/work/demo/work-item.md",
          "---\ntype: bee.work-item\ntitle: Demo work item\ndescription: port the dispatch driver to rust\nbee:\n  id: demo\n  lifecycle: active\n  areas: [dispatch]\n  decisions: [d-1]\n---\n\nThe demo work item body mentions dispatch prompts and rust.\n");
        w(root, "docs/knowledge/work/demo/plan.md",
          "---\ntype: bee.plan\ntitle: Demo plan\ndescription: the plan for demo\nbee:\n  id: demo-plan\n  lifecycle: active\n---\n\nPlan body.\n");
        w(root, "docs/knowledge/patterns/dispatch-prompt.md",
          "---\ntype: bee.pattern\ntitle: Dispatch prompt assembly\ndescription: how dispatch prompts are assembled\nbee:\n  id: p-dispatch\n  lifecycle: active\n  critical: true\n  areas: [dispatch]\n---\n\nDispatch prompts are assembled from templates and machine blocks.\n");
        w(root, "docs/knowledge/patterns/unrelated.md",
          "---\ntype: bee.pattern\ntitle: Unrelated pattern\ndescription: about billing invoices\nbee:\n  id: p-billing\n  lifecycle: active\n  critical: true\n  areas: [billing]\n---\n\nBilling invoices and refunds.\n");
        w(root, "docs/knowledge/areas/dispatch.md",
          "---\ntype: bee.decision\ntitle: Dispatch decision\ndescription: a decision about dispatch\nbee:\n  id: d-1\n  lifecycle: active\n  areas: [dispatch]\n---\n\nDecided.\n");
    }

    /// THE CROSS-CHECK the port rules require: the `kctx` lift must produce
    /// the SAME manifest as the shipped `bee knowledge context` verb (whose own
    /// copy lives in verbs/knowledge.rs) and as Node's lib/knowledge.mjs
    /// buildContextManifest. The golden below was captured from BOTH runtimes
    /// on this exact fixture — `node bee.mjs knowledge context --work demo
    /// --budget 20000 --json` and `bee.exe knowledge context --work demo
    /// --budget 20000 --json` printed it byte-for-byte — so a drift in either
    /// Rust copy, or from the .mjs, fails here.
    #[test]
    fn learned_context_agrees_with_the_knowledge_verb_port() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        bundle_fixture(&root);
        let dir = kctx::bundle_dir(&root).unwrap();
        let kctx::ManifestOut::Built(manifest) =
            kctx::build_context_manifest(&dir, "demo", 20000.0, &kctx::num(20000.0))
        else {
            panic!("expected a built manifest")
        };
        const GOLDEN: &str = concat!(
            r#"{"work":"demo","decisions":["d-1"],"budget":20000,"estimator":"bytes/4","#,
            r#""total_est":240,"entries":["#,
            r#"{"path":"docs/knowledge/work/demo/work-item.md","bytes":232,"est_tokens":58,"reason":"work item"},"#,
            r#"{"path":"docs/knowledge/work/demo/plan.md","bytes":124,"est_tokens":31,"reason":"plan sibling in work/demo/"},"#,
            r#"{"path":"docs/knowledge/patterns/dispatch-prompt.md","bytes":252,"est_tokens":63,"reason":"critical pattern (relevance 0.508333, rank 1 of 2, floor)"},"#,
            r#"{"path":"docs/knowledge/patterns/unrelated.md","bytes":195,"est_tokens":49,"reason":"critical pattern (relevance 0, rank 2 of 2, floor)"},"#,
            r#"{"path":"docs/knowledge/areas/dispatch.md","bytes":156,"est_tokens":39,"reason":"decision for area dispatch"}"#,
            r#"],"truncated":[],"excluded":[],"#,
            r#""floor":["docs/knowledge/patterns/dispatch-prompt.md","docs/knowledge/patterns/unrelated.md"],"#,
            r#""critical_total":2,"zero_signal_count":1}"#,
        );
        assert_eq!(jsjson::stringify(&manifest), GOLDEN);
    }

    #[test]
    fn learned_context_uses_the_manifest_and_honours_read_first() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        bundle_fixture(&root);
        assert!(bundle_mode(&root).unwrap());
        let cell = json!({"id": "c-1", "feature": "demo", "lane": "small"});
        assert_eq!(
            learned_context_lines(&root, &cell).unwrap(),
            vec![
                "- docs/knowledge/work/demo/work-item.md — Demo work item",
                "- docs/knowledge/work/demo/plan.md — Demo plan",
                "- docs/knowledge/patterns/dispatch-prompt.md — Dispatch prompt assembly",
                "- docs/knowledge/patterns/unrelated.md — Unrelated pattern",
                "- docs/knowledge/areas/dispatch.md — Dispatch decision",
            ]
        );
        // read_first stays authoritative: its entries are never duplicated,
        // and backslashes / "./" prefixes normalize first.
        let cell = json!({
            "id": "c-1", "feature": "demo",
            "read_first": ["docs\\knowledge\\work\\demo\\work-item.md", "./docs/knowledge/areas/dispatch.md"]
        });
        let lines = learned_context_lines(&root, &cell).unwrap();
        assert!(!lines.iter().any(|l| l.contains("work-item.md")));
        assert!(!lines.iter().any(|l| l.contains("areas/dispatch.md")));
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn learned_context_falls_back_to_the_index_pointer_then_to_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        bundle_fixture(&root);
        // A cell whose feature names no work item: the manifest throws
        // unknown_work (caught) and the index pointer answers instead.
        let cell = json!({"id": "c-1", "feature": "not-a-work-item"});
        assert_eq!(
            learned_context_lines(&root, &cell).unwrap(),
            vec!["- docs/knowledge/index.md — Knowledge bundle index (see \"Critical patterns\")"]
        );
        // …unless read_first already names it.
        let cell =
            json!({"id": "c-1", "feature": "nope", "read_first": ["docs/knowledge/index.md"]});
        assert!(learned_context_lines(&root, &cell).unwrap().is_empty());
    }

    #[test]
    fn no_bundle_falls_back_to_the_legacy_critical_patterns_file() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        let cell = json!({"id": "c-1", "feature": "demo"});
        // Nothing at all -> the block is omitted (byte-identical to a repo
        // with no knowledge layer).
        assert!(learned_context_lines(&root, &cell).unwrap().is_empty());
        w(&root, "docs/history/learnings/critical-patterns.md", "# Critical patterns\n");
        assert_eq!(
            learned_context_lines(&root, &cell).unwrap(),
            vec!["- docs/history/learnings/critical-patterns.md — Critical patterns (hard-won learnings)"]
        );
        // A directory full of markdown that parses as NO concept is not a
        // bundle (advisor-digest-f3 finding 1).
        w(&root, "docs/knowledge/stray.md", "just prose, no frontmatter\n");
        assert!(!bundle_mode(&root).unwrap());
    }

    #[test]
    fn learned_context_is_capped_at_eight_pointer_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        bundle_fixture(&root);
        for i in 0..12 {
            w(
                &root,
                &format!("docs/knowledge/patterns/extra-{i}.md"),
                &format!("---\ntype: bee.pattern\ntitle: Extra {i}\ndescription: dispatch prompts rust {i}\nbee:\n  id: p-extra-{i}\n  lifecycle: active\n  critical: true\n  areas: [dispatch]\n---\n\nDispatch prompts extra {i}.\n"),
            );
        }
        let cell = json!({"id": "c-1", "feature": "demo", "lane": "high-risk"});
        let lines = learned_context_lines(&root, &cell).unwrap();
        assert_eq!(lines.len(), LEARNED_CONTEXT_MAX_LINES);
    }

    #[test]
    fn lane_budget_scales_and_defaults() {
        assert_eq!(lane_budget(Some(&json!("tiny"))), 8000.0);
        assert_eq!(lane_budget(Some(&json!("small"))), 12000.0);
        assert_eq!(lane_budget(Some(&json!("standard"))), 20000.0);
        assert_eq!(lane_budget(Some(&json!("high-risk"))), 30000.0);
        assert_eq!(lane_budget(None), 20000.0);
        assert_eq!(lane_budget(Some(&json!("banana"))), 20000.0);
    }

    // ── close ──────────────────────────────────────────────────────────────

    #[test]
    fn declaration_normalizes_strings_arrays_and_the_none_sentinel() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"commands":{}}"#);
        assert!(declared_test_commands(&root).unwrap().is_none());
        w(&root, ".bee/config.json", r#"{"commands":{"test":"  npm test  "}}"#);
        assert_eq!(
            declared_test_commands(&root).unwrap(),
            Some(vec!["npm test".to_string()])
        );
        w(&root, ".bee/config.json", r#"{"commands":{"test":[" a ",1,""," b "]}}"#);
        assert_eq!(
            declared_test_commands(&root).unwrap(),
            Some(vec!["a".to_string(), "b".to_string()])
        );
        w(&root, ".bee/config.json", r#"{"commands":{"test":["none"," none "]}}"#);
        assert!(declared_test_commands(&root).unwrap().is_none());
        // A dogfood_repos list makes readConfig warn per dead repo -> delegate.
        w(
            &root,
            ".bee/config.json",
            r#"{"commands":{"test":"x"},"dogfood_repos":["Z:/gone"]}"#,
        );
        assert!(declared_test_commands(&root).is_err());
        // Corrupt config bails to Node.
        w(&root, ".bee/config.json", "{broken");
        assert!(declared_test_commands(&root).is_err());
    }

    #[test]
    fn close_dry_run_reports_the_doors_and_runs_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"commands":{"test":["a","b"]}}"#);
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(result, text, code) =
            close_handler(&root, "demo", true, declared, None).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 0);
        assert_eq!(
            text,
            concat!(
                "door tests: open — commands.test declared (2 command(s)) — close runs the full declared suite fresh; a stale test-results record is never trusted | settle: bee test\n",
                "door scribing-debt: clear\n",
                "door capture-queue: clear\n",
                "next: bee close --feature demo — runs the declared tests and reports"
            )
        );
        assert_eq!(result.get("feature"), Some(&json!("demo")));
        // Nothing ran: no record file.
        assert!(!root.join(".bee/logs/test-results.json").exists());

        // Undeclared repo: the teaching detail + a different next line.
        w(&root, ".bee/config.json", "{}");
        let Out::Emit(_, text, _) = close_handler(&root, "demo", true, None, None).unwrap() else {
            panic!()
        };
        assert!(text.starts_with(&format!("door tests: open — {CLOSE_TESTS_UNDECLARED_DETAIL}\n")));
        assert!(text.ends_with(
            "next: feature \"demo\" has no test door — close proceeds; capture stays pending for bee-capturing"
        ));
    }

    #[test]
    fn close_green_reports_the_capture_checklist() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"commands":{"test":"echo suite-green"}}"#);
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(result, text, code) =
            close_handler(&root, "demo", false, declared, Some(shell)).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 0);
        let lines: Vec<&str> = text.split('\n').collect();
        assert_eq!(
            lines[0],
            "Tests GREEN for \"demo\" — 1 command(s) passed (record: .bee/logs/test-results.json)."
        );
        assert!(lines[1].starts_with("✓ echo suite-green ("));
        assert_eq!(
            lines[2],
            "Capture (deferred, decision c8e25271): scribing clear; capture queue clear."
        );
        assert_eq!(
            lines[3],
            "next: done — capture is recorded as pending (run bee-capturing whenever; orient keeps the reminder)."
        );
        assert_eq!(result.get("ran_tests"), Some(&json!(true)));
        assert_eq!(
            result.get("tests").unwrap().get("results"),
            Some(&json!(".bee/logs/test-results.json"))
        );
        // The run is FRESH: the record exists and is green.
        let record: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(".bee/logs/test-results.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(record.get("green"), Some(&json!(true)));
    }

    #[test]
    fn close_red_stops_at_the_tests_door_and_exits_one() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"commands":{"test":["echo boom-line; echo more 1>&2; exit 3","echo second-ok"]}}"#,
        );
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(result, text, code) =
            close_handler(&root, "demo", false, declared, Some(shell)).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 1);
        let lines: Vec<&str> = text.split('\n').collect();
        assert_eq!(
            lines[0],
            "Tests RED for \"demo\" — close stops at the tests door (record: .bee/logs/test-results.json):"
        );
        assert!(lines[1].starts_with("✗ echo boom-line; echo more 1>&2; exit 3 ("));
        assert!(lines[1].ends_with(", exit 3)"));
        assert!(lines[2].starts_with("✓ echo second-ok ("));
        assert_eq!(lines[3], "--- echo boom-line; echo more 1>&2; exit 3 (exit 3) ---");
        assert_eq!(lines[4], "boom-line");
        assert_eq!(lines[5], "more");
        assert_eq!(
            lines[6],
            "next: the red is the work — fix it (boom-line), then re-run bee close --feature demo"
        );
        // The record is STILL written on a red (a red is a normal result).
        assert!(root.join(".bee/logs/test-results.json").exists());
        let doors = result.get("doors").unwrap().as_array().unwrap();
        assert_eq!(doors[0].get("blocking"), Some(&json!(true)));
        assert_eq!(
            doors[0].get("detail"),
            Some(&json!("the declared test run is RED (1 of 2 command(s) failed; record: .bee/logs/test-results.json)"))
        );
        // The report-only doors are never blocking, even beside a red.
        assert_eq!(doors[1].get("blocking"), Some(&json!(false)));
        assert_eq!(doors[2].get("blocking"), Some(&json!(false)));
    }

    #[test]
    fn close_surfaces_pending_capture_reminders() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        w(&root, ".bee/state.json", r#"{"feature":"demo"}"#);
        w(&root, ".bee/cells/demo-4.json", r#"{"id":"demo-4","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        w(&root, ".bee/cells/demo-5.json", r#"{"id":"demo-5","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-02T00:00:00.000Z"}}"#);
        // A capped cell of ANOTHER feature never counts.
        w(&root, ".bee/cells/other.json", r#"{"id":"other","feature":"x","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-02T00:00:00.000Z"}}"#);
        w(
            &root,
            ".bee/capture-queue.jsonl",
            "{\"kind\":\"stub\",\"id\":\"s1\"}\n{\"kind\":\"stub\",\"id\":\"s2\"}\n{\"kind\":\"flush\",\"id\":\"s1\"}\n",
        );

        let doors = build_close_report_doors(&root, "demo").unwrap();
        assert_eq!(
            doors[0].detail,
            "pending — 2 behavior_change cell(s) uncaptured (demo-4, demo-5); settle later via bee-capturing"
        );
        assert_eq!(
            doors[1].detail,
            "pending — 1 capture stub(s) awaiting flush; settle later via bee-capturing"
        );
        assert_eq!(
            render_close_door_lines(&doors),
            vec![
                "door scribing-debt: open — pending — 2 behavior_change cell(s) uncaptured (demo-4, demo-5); settle later via bee-capturing",
                "door capture-queue: open — pending — 1 capture stub(s) awaiting flush; settle later via bee-capturing",
            ]
        );

        // A scribing run after the caps clears the debt (threshold is >, not >=).
        w(
            &root,
            ".bee/logs/scribing-runs.jsonl",
            "{\"feature\":\"demo\",\"ts\":\"2026-07-03T00:00:00.000Z\"}\n",
        );
        assert_eq!(build_close_report_doors(&root, "demo").unwrap()[0].detail, "clear");
        // A ledger row for ANOTHER feature never moves this feature's threshold.
        w(
            &root,
            ".bee/logs/scribing-runs.jsonl",
            "{\"feature\":\"elsewhere\",\"ts\":\"2026-07-03T00:00:00.000Z\"}\n",
        );
        assert_eq!(scribing_debt(&root, "demo").unwrap().count, 2);
    }

    /// Regression: the scribing-debt door JOINS the cell ids, so listCells'
    /// numeric-aware localeCompare order reaches an emitted byte. A plain byte
    /// sort put "rust-port-5" after "rust-port-23" — caught by a live diff
    /// against the beehive repo itself.
    #[test]
    fn scribing_debt_ids_keep_numeric_aware_collation_order() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        for n in ["5", "7", "11", "23"] {
            w(&root, &format!(".bee/cells/f-{n}.json"), &format!(
                r#"{{"id":"f-{n}","feature":"demo","status":"capped","trace":{{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}}}"#
            ));
        }
        let debt = scribing_debt(&root, "demo").unwrap();
        assert_eq!(
            js_join(&debt.ids, ", "),
            "f-5, f-7, f-11, f-23",
            "numeric runs compare by value, not byte order"
        );
        // "01" and "1" are fully equal at every ICU level.
        assert_eq!(locale_cmp("a01", "a1", true), std::cmp::Ordering::Equal);
        assert_eq!(locale_cmp("_a", "-a", true), std::cmp::Ordering::Less);
    }

    #[test]
    fn state_last_scribing_run_raises_the_threshold_only_for_its_own_feature() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        w(&root, ".bee/cells/c.json", r#"{"id":"c","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        w(
            &root,
            ".bee/state.json",
            r#"{"last_scribing_run":{"feature":"other","at":"2026-07-09T00:00:00.000Z"}}"#,
        );
        assert_eq!(scribing_debt(&root, "demo").unwrap().count, 1);
        w(
            &root,
            ".bee/state.json",
            r#"{"last_scribing_run":{"feature":"demo","at":"2026-07-09T00:00:00.000Z"}}"#,
        );
        assert_eq!(scribing_debt(&root, "demo").unwrap().count, 0);
    }

    // ── routing (every delegating shape returns None before any output) ─────

    #[test]
    fn routing_serves_only_the_proven_shapes() {
        let os = |v: &[&str]| -> Vec<OsString> { v.iter().map(OsString::from).collect() };
        let t0 = Instant::now();
        // Not our verbs.
        assert!(try_native(&os(&["status"]), t0).is_none());
        assert!(try_native(&os(&["dispatch"]), t0).is_none());
        assert!(try_native(&os(&["dispatch", "guard"]), t0).is_none());
        // --help anywhere -> Node renders command-scoped help.
        assert!(try_native(&os(&["close", "--help"]), t0).is_none());
        assert!(try_native(&os(&["dispatch", "prepare", "--help"]), t0).is_none());
        // Stray positionals, unknown flags, missing/empty required flags.
        assert!(try_native(&os(&["close", "extra", "--feature", "f"]), t0).is_none());
        assert!(try_native(&os(&["close", "--feature", "f", "--wat", "x"]), t0).is_none());
        assert!(try_native(&os(&["close"]), t0).is_none());
        assert!(try_native(&os(&["close", "--feature="]), t0).is_none());
        assert!(try_native(&os(&["close", "--feature", "f", "--dry-run=maybe"]), t0).is_none());
        // dispatch: --claim and --session-id are Node's (the claim+reserve
        // doors); bad enums and missing requireds are validate()'s.
        assert!(try_native(
            &os(&["dispatch", "prepare", "--runtime", "claude", "--kind", "cell", "--cell", "c", "--worker", "w", "--claim"]),
            t0
        )
        .is_none());
        assert!(try_native(
            &os(&["dispatch", "prepare", "--runtime", "claude", "--kind", "cell", "--session-id", "s"]),
            t0
        )
        .is_none());
        assert!(try_native(
            &os(&["dispatch", "prepare", "--runtime", "banana", "--kind", "gather"]),
            t0
        )
        .is_none());
        assert!(try_native(
            &os(&["dispatch", "prepare", "--runtime", "claude", "--kind", "banana"]),
            t0
        )
        .is_none());
        assert!(try_native(&os(&["dispatch", "prepare", "--kind", "gather"]), t0).is_none());
        assert!(try_native(&os(&["dispatch", "prepare", "--runtime", "claude"]), t0).is_none());
    }

    #[test]
    fn only_a_feature_with_its_own_lane_record_delegates() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        assert!(!feature_has_lane_record(&root, "demo"));
        // Another feature's lane record never makes THIS feature delegate.
        w(&root, ".bee/lanes/other.json", "{}");
        assert!(!feature_has_lane_record(&root, "demo"));
        w(&root, ".bee/lanes/demo.json", "{}");
        assert!(feature_has_lane_record(&root, "demo"));
        assert!(feature_has_lane_record(&root, "  demo  ")); // lanePath trims
        // A malformed feature name makes readLane fail-open to "no lane".
        w(&root, ".bee/lanes/x.json", "{}");
        assert!(!feature_has_lane_record(&root, "a/b"));
        assert!(!feature_has_lane_record(&root, ".."));
        assert!(!feature_has_lane_record(&root, "   "));
        // Workflow records are irrelevant to close.
        w(&root, ".bee/runtime/workflows/wf-1.json", "{}");
        assert!(!feature_has_lane_record(&root, "nolane"));
    }

    #[test]
    fn prompt_skew_delegates() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        // No on-disk prompts: the embedded copy is the only one.
        assert!(prompts_match_disk(&root, "gather"));
        // A vendored copy that MATCHES is fine (CRLF normalization included).
        w(&root, ".bee/bin/prompts/gather.md", &PROMPT_GATHER.replace('\n', "\r\n"));
        assert!(prompts_match_disk(&root, "gather"));
        // A skewed vendored copy delegates.
        w(&root, ".bee/bin/prompts/gather.md", "Gather: something else\n");
        assert!(!prompts_match_disk(&root, "gather"));
    }

    #[test]
    fn utf16_slicing_matches_js() {
        assert_eq!(utf16_head("abcdef", 3), "abc");
        assert_eq!(utf16_head("abc", 10), "abc");
        assert_eq!(utf16_tail(&"x".repeat(650), 500).len(), 500);
        // Astral chars count 2 UTF-16 units each, like JS.
        let astral = "🐝".repeat(4); // 8 units
        assert_eq!(utf16_head(&astral, 5).encode_utf16().count(), 4);
    }
}
