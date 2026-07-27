//! decisions — `.bee/decisions.jsonl`, ported from `.bee/bin/lib/
//! decisions.mjs` (CONTEXT.md D3/D5).
//!
//! **Read side** (rust-port-13): `activeDecisions`.
//!
//! **Write side** (rpl-4): `logDecision`, `supersedeDecision`,
//! `redactDecision`, plus the machinery those three depend on — the bounded
//! synchronous `withDecisionsLockSync` retry over the D9 cross-process lock
//! ([`crate::lock`], never a second lock implementation), the dp-6 tag
//! classification gate, and the dp-2 `docs/**` citation sweep.
//! `archiveDecisions` and the `tag` write verb are deliberately NOT here:
//! this cell ports the three verbs `bee decisions log|supersede|redact`
//! drives, and inventing the rest would ship untested writers.
//!
//! `.bee/bin/lib/decisions.mjs` is FROZEN for the duration of the rust-port
//! feature (D1). This module mirrors `activeDecisions`'s full read-path
//! semantics: supersession/redaction exclusion, the tag overlay
//! (decision-propagation dp-5/D7c — latest `tag` event wins by date then
//! file order, REPLACING the whole `tags` array and `scope` only when the
//! winning event actually carries one), and the `all: true` archive-union
//! branch (dp-3/D4c — active-file events win ties by id, ordering by date
//! descending with an index tiebreak equivalent to `.reverse()` on an
//! unarchived store). Oracle-diffed against the real mjs module in
//! `tests/status_readers_a.rs`.
//!
//! # Why the appended line is byte-identical (rpl-4)
//!
//! Every event below is built as a [`serde_json::Map`] whose keys are
//! inserted in the mjs object literal's ORDER (`decisions.mjs:319`, `:468`,
//! `:495`), and `serde_json`'s `preserve_order` feature — declared
//! load-bearing in this crate's `Cargo.toml` — is what makes that order
//! survive serialization. `JSON.stringify`'s one reordering rule (integer-
//! like keys hoisted ascending, which `queen_bee::jsonout` exists to
//! reproduce on the stdout path) cannot apply here: every key an event or a
//! sweep hit can carry is a fixed non-numeric literal, so
//! [`crate::fsutil::append_jsonl`]'s plain `serde_json::to_string` is
//! already the JS spelling. That is a checked fact, not an assumption — see
//! the `event_keys_are_never_integer_like` test.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{Map, Value};

use crate::datamark::{assert_safe_decision_content, js_trim};
use crate::fsutil::{append_jsonl, ensure_dir, read_json, read_jsonl, write_json_atomic};
use crate::jsdate::parse_iso_ms;
use crate::lock::{acquire_store_lock_once, OnceOutcome, StoreLockGuard};

pub fn decisions_path(root: &Path) -> PathBuf {
    root.join(".bee").join("decisions.jsonl")
}

pub fn decisions_archive_path(root: &Path) -> PathBuf {
    root.join(".bee").join("decisions-archive.jsonl")
}

struct TagPatch {
    /// `Some(vec)` — REPLACE the whole `tags` array (even `Some(vec![])`);
    /// `None` — the winning tag event carried no array, leave `tags` alone.
    tags: Option<Vec<Value>>,
    /// `Some(scope)` — replace; `None` — leave `scope` alone.
    scope: Option<String>,
}

// A JS truthy-string check: `typeof v === 'string' && v` (empty string is
// falsy). Used everywhere the mjs source gates a field on "is this actually
// a non-empty string", not merely "is this present".
fn truthy_str(v: Option<&Value>) -> Option<&str> {
    v.and_then(Value::as_str).filter(|s| !s.is_empty())
}

fn event_date_ms(event: &Value) -> Option<i64> {
    event.get("date").and_then(Value::as_str).and_then(parse_iso_ms)
}

/// `buildTagOverlay(root)`: the active file's `tag` events only, latest
/// (by date, ties broken by later file position) wins per `target` id.
///
/// rust-port-23: takes the journal events the caller ALREADY read rather
/// than reading the file a second time. `buildTagOverlay` and
/// `activeDecisions` both read `.bee/decisions.jsonl` in the mjs source,
/// which is why one `active_decisions` call used to cost two journal
/// parses; they now share one read, at one instant.
///
/// The read-accounting counter for this store lives inside
/// `fsutil::read_jsonl` itself, keyed on the path (rust-port-22's rework)
/// — so the single surviving read below is still counted, and any future
/// caller that reads the journal from anywhere else in the crate is
/// counted too.
fn build_tag_overlay_from(events: &[Value]) -> HashMap<String, TagPatch> {
    let mut tag_events: Vec<(usize, Value)> = events
        .iter()
        .cloned()
        .enumerate()
        .filter(|(_, e)| {
            e.get("type").and_then(Value::as_str) == Some("tag") && e.get("target").and_then(Value::as_str).is_some()
        })
        .collect();
    tag_events.sort_by(|(ia, ea), (ib, eb)| {
        let ma = event_date_ms(ea);
        let mb = event_date_ms(eb);
        if let (Some(x), Some(y)) = (ma, mb) {
            if x != y {
                return x.cmp(&y); // ascending: earlier date first
            }
        }
        ia.cmp(ib) // ascending: earlier file position first
    });
    let mut overlay: HashMap<String, TagPatch> = HashMap::new();
    for (_, event) in tag_events {
        let Some(target) = event.get("target").and_then(Value::as_str) else { continue };
        let tags = event.get("tags").and_then(Value::as_array).cloned();
        let scope = truthy_str(event.get("scope")).map(str::to_string);
        overlay.insert(target.to_string(), TagPatch { tags, scope });
    }
    overlay
}

/// `applyTagOverlay(event, overlay)`: returns `event` unchanged (a clone,
/// since Rust values aren't shared references the way the mjs source's
/// "same object" identity check implies) when there is no patch for its id;
/// otherwise a copy with `tags`/`scope` overridden per [`TagPatch`].
fn apply_tag_overlay(event: &Value, overlay: &HashMap<String, TagPatch>) -> Value {
    let Some(id) = event.get("id").and_then(Value::as_str) else { return event.clone() };
    let Some(patch) = overlay.get(id) else { return event.clone() };
    let mut obj = event.as_object().cloned().unwrap_or_default();
    if let Some(tags) = &patch.tags {
        obj.insert("tags".to_string(), Value::Array(tags.clone()));
    }
    if let Some(scope) = &patch.scope {
        obj.insert("scope".to_string(), Value::String(scope.clone()));
    }
    Value::Object(obj)
}

fn is_decide_or_supersede(event: &Value) -> bool {
    matches!(event.get("type").and_then(Value::as_str), Some("decide") | Some("supersede"))
}

/// `activeDecisions(root, {recent, all})`: decide/supersede events not
/// themselves superseded or redacted, newest first (tag-overlay applied).
///
/// - `all: false` (default): reads ONLY the active store, ordered by a
///   plain positional reverse (byte-identical to the pre-dp-3 behavior).
/// - `all: true`: additionally unions in `.bee/decisions-archive.jsonl`
///   (missing/empty archive silently treated as "nothing extra"),
///   de-duplicated by id with the active copy winning, ordered by event
///   date descending with an original-insertion-index tiebreak — which is
///   mathematically identical to the `all: false` reverse whenever the
///   archive contributes nothing new.
pub fn active_decisions(root: &Path, recent: Option<usize>, all: bool) -> Vec<Value> {
    // rust-port-23: ONE journal read per call, shared by the tag overlay
    // and the event scan below (this call used to cost two). The archive
    // file is a DIFFERENT store (`decisions-archive.jsonl`), read only on
    // the `all` branch and deliberately excluded from the
    // `decisions_journal_parses` bucket.
    let events: Vec<Value> = read_jsonl(&decisions_path(root));
    let overlay = build_tag_overlay_from(&events);

    if !all {
        let mut superseded: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut redacted: std::collections::HashSet<String> = std::collections::HashSet::new();
        for event in &events {
            if event.get("type").and_then(Value::as_str) == Some("supersede") {
                if let Some(s) = truthy_str(event.get("supersedes")) {
                    superseded.insert(s.to_string());
                }
            }
            if event.get("type").and_then(Value::as_str) == Some("redact") {
                if let Some(r) = truthy_str(event.get("redacts")) {
                    redacted.insert(r.to_string());
                }
            }
        }
        let mut active: Vec<Value> = events
            .into_iter()
            .filter(|event| {
                let id = event.get("id").and_then(Value::as_str).unwrap_or("");
                is_decide_or_supersede(event) && !superseded.contains(id) && !redacted.contains(id)
            })
            .collect();
        active.reverse();
        let active: Vec<Value> = active.iter().map(|e| apply_tag_overlay(e, &overlay)).collect();
        return match recent {
            Some(n) => active.into_iter().take(n).collect(),
            None => active,
        };
    }

    // rust-port-23: the journal was already read once above; only the
    // archive file (a different store, uncounted by design) is read here.
    let active_events: Vec<Value> = events;
    let archived_events: Vec<Value> = read_jsonl(&decisions_archive_path(root));

    // Map insertion-order semantics (JS `Map.set` on an existing key keeps
    // its original position but updates the value): track first-seen order
    // separately from the value store.
    let mut order: Vec<String> = Vec::new();
    let mut by_id: HashMap<String, Value> = HashMap::new();
    for event in &active_events {
        if let Some(id) = event.get("id").and_then(Value::as_str) {
            if !by_id.contains_key(id) {
                order.push(id.to_string());
            }
            by_id.insert(id.to_string(), event.clone());
        }
    }
    for event in &archived_events {
        if let Some(id) = event.get("id").and_then(Value::as_str) {
            if !by_id.contains_key(id) {
                order.push(id.to_string());
                by_id.insert(id.to_string(), event.clone());
            }
        }
    }
    // ORDER-IRRELEVANT `remove` (rust-port-15 sweep): `by_id` is a
    // `std::collections::HashMap`, not a `serde_json::Map`, so the
    // `preserve_order`/`swap_remove` aliasing does not apply here at all —
    // and the output sequence comes from `order`, never from iterating
    // this map.
    let events: Vec<Value> = order.into_iter().map(|id| by_id.remove(&id).unwrap()).collect();

    let indexed: Vec<(usize, Value)> = events.into_iter().enumerate().collect();
    let mut superseded: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut redacted: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (_, event) in &indexed {
        if event.get("type").and_then(Value::as_str) == Some("supersede") {
            if let Some(s) = truthy_str(event.get("supersedes")) {
                superseded.insert(s.to_string());
            }
        }
        if event.get("type").and_then(Value::as_str) == Some("redact") {
            if let Some(r) = truthy_str(event.get("redacts")) {
                redacted.insert(r.to_string());
            }
        }
    }

    let mut active: Vec<(usize, Value)> = indexed
        .into_iter()
        .filter(|(_, event)| {
            let id = event.get("id").and_then(Value::as_str).unwrap_or("");
            is_decide_or_supersede(event) && !superseded.contains(id) && !redacted.contains(id)
        })
        .collect();

    active.sort_by(|(ia, ea), (ib, eb)| {
        let ma = event_date_ms(ea);
        let mb = event_date_ms(eb);
        if let (Some(x), Some(y)) = (ma, mb) {
            if x != y {
                return y.cmp(&x); // descending: later date first
            }
        }
        ib.cmp(ia) // descending: higher original index first
    });

    let result: Vec<Value> = active.iter().map(|(_, event)| apply_tag_overlay(event, &overlay)).collect();
    match recent {
        Some(n) => result.into_iter().take(n).collect(),
        None => result,
    }
}

// Read-side tests live in crates/bee-core/tests/status_readers_a.rs
// (rust-port-13's single integration target) rather than here.

// ═══════════════════════════════════════════════════════════════════════════
// THE WRITE SIDE (rpl-4)
// ═══════════════════════════════════════════════════════════════════════════

/// `decisions.mjs:48` — ONE unscoped lock name, not a per-id lock, because
/// `archiveDecisions` rewrites the WHOLE store and every appender must
/// serialize against that rewrite (dp-3).
pub const DECISIONS_LOCK_NAME: &str = "decisions";

/// `decisions.mjs:64-65`. ~300 ms worst-case wait, matching
/// `acquireGateWithRetry`'s budget — a bounded synchronous retry on top of
/// [`acquire_store_lock_once`], never a second lock implementation and never
/// a fall-through unlocked write.
const DECISIONS_LOCK_RETRY_ATTEMPTS: u32 = 15;
const DECISIONS_LOCK_RETRY_DELAY_MS: u64 = 20;

/// `decisions.mjs:72-83` `DecisionsLockBusyError`'s message, byte-exact.
/// `??` falls back only for null/undefined, and `String(x)` renders whatever
/// else the lock body carried — a JSON `null` session therefore reads
/// `unknown`, while a numeric one renders as a number.
fn decisions_lock_busy_message(holder: Option<&Value>) -> String {
    let who = match holder {
        Some(Value::Object(map)) => {
            let field = |k: &str| match map.get(k) {
                None | Some(Value::Null) => "unknown".to_string(),
                Some(v) => js_string_value(v),
            };
            format!(
                "pid={} session={} since {}",
                field("pid"),
                field("session"),
                field("ts")
            )
        }
        // `holder && typeof holder === 'object'` — null, absent, and every
        // primitive land here. An ARRAY is `typeof 'object'` in JS and would
        // take the first branch, but a lock body is never an array: the
        // protocol writes `{pid, session, ts, token}` and a body that fails
        // to parse is reported as no holder at all.
        _ => "unknown holder".to_string(),
    };
    format!("decisions store lock \"{DECISIONS_LOCK_NAME}\" busy: held by {who}")
}

/// `String(x)` for the shapes a parsed lock body can hold.
fn js_string_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "null".to_string(),
        Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Array(_) | Value::Object(_) => "[object Object]".to_string(),
    }
}

/// Releases the guard when this value drops — the Rust spelling of the mjs
/// `try { return fn(); } finally { lock.release(); }` at `decisions.mjs:100`.
/// Without it a panic inside the critical section would strand the lock file
/// until the stale-takeover window expired.
struct ReleaseOnDrop<'a>(&'a mut StoreLockGuard);

impl Drop for ReleaseOnDrop<'_> {
    fn drop(&mut self) {
        let _ = self.0.release();
    }
}

/// `decisions.mjs:89` `withDecisionsLockSync`.
///
/// Attempt budget is mjs's exactly: ONE initial acquire, then up to
/// [`DECISIONS_LOCK_RETRY_ATTEMPTS`] further attempts each preceded by a
/// [`DECISIONS_LOCK_RETRY_DELAY_MS`] sleep — 16 acquires and 15 sleeps
/// before the typed refusal. Every acquire goes through
/// [`acquire_store_lock_once`], so the D9 contention telemetry
/// (`.bee/logs/contention.jsonl`) is emitted by the SAME writer both
/// runtimes share, not by a parallel one here.
pub fn with_decisions_lock_sync<T>(
    root: &Path,
    f: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let mut outcome = acquire_store_lock_once(root, DECISIONS_LOCK_NAME)
        .map_err(|e| format!("decisions lock acquire failed: {e}"))?;
    let mut attempt: u32 = 0;
    let mut guard = loop {
        match outcome {
            OnceOutcome::Acquired(g) => break g,
            OnceOutcome::Busy { holder } => {
                if attempt >= DECISIONS_LOCK_RETRY_ATTEMPTS {
                    return Err(decisions_lock_busy_message(holder.as_ref()));
                }
                std::thread::sleep(Duration::from_millis(DECISIONS_LOCK_RETRY_DELAY_MS));
                attempt += 1;
                outcome = acquire_store_lock_once(root, DECISIONS_LOCK_NAME)
                    .map_err(|e| format!("decisions lock acquire failed: {e}"))?;
            }
        }
    };
    let _release = ReleaseOnDrop(&mut guard);
    f()
}

// ─── the tag taxonomy gate (dp-6, CONTEXT D7b) ─────────────────────────────

fn taxonomy_path(root: &Path) -> PathBuf {
    root.join("docs").join("decisions").join("taxonomy.json")
}

/// `decisions.mjs:118` `taxonomyFileExists`.
pub fn taxonomy_file_exists(root: &Path) -> bool {
    taxonomy_path(root).exists()
}

struct Taxonomy {
    schema_version: Value,
    tags: Vec<Value>,
    candidates: Vec<String>,
}

/// `decisions.mjs:126` `loadTaxonomy`. `readJson` already fails open on a
/// missing OR malformed file (warning to stderr on the latter — the artifact
/// `bee_parity::normalize::reconcile_parse_warnings` reconciles), so this
/// reuses it rather than adding a second parse-or-null path.
fn load_taxonomy(root: &Path) -> Option<Taxonomy> {
    let raw: Value = read_json(&taxonomy_path(root), Value::Null);
    let obj = raw.as_object()?;
    let tags = obj.get("tags").and_then(Value::as_array).cloned().unwrap_or_default();
    let candidates = obj
        .get("candidates")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default();
    // `raw.schema_version ?? 1` — null/absent become 1, anything else rides
    // through unchanged (including a string, which mjs would also preserve).
    let schema_version = match obj.get("schema_version") {
        None | Some(Value::Null) => Value::from(1),
        Some(v) => v.clone(),
    };
    Some(Taxonomy { schema_version, tags, candidates })
}

/// `decisions.mjs:145` `appendTaxonomyCandidatesSync` — a locked
/// read-modify-write under the SAME store lock, re-reading fresh under the
/// lock so two concurrent unknown-tag appends can never lose one.
fn append_taxonomy_candidates_sync(root: &Path, unknown_tags: &[String]) -> Result<(), String> {
    with_decisions_lock_sync(root, || {
        let Some(fresh) = load_taxonomy(root) else {
            return Ok(()); // vanished between the read and the write
        };
        let mut known: Vec<String> = fresh
            .tags
            .iter()
            .map(|t| match t.get("name") {
                Some(Value::String(s)) => s.clone(),
                // `t && t.name` on a non-object member yields undefined; the
                // Set then holds `undefined`, which no string tag equals.
                _ => "\u{0}<undefined>".to_string(),
            })
            .collect();
        known.extend(fresh.candidates.iter().cloned());
        let mut next_candidates = fresh.candidates.clone();
        for tag in unknown_tags {
            if !known.contains(tag) && !next_candidates.contains(tag) {
                next_candidates.push(tag.clone());
            }
        }
        if next_candidates.len() != fresh.candidates.len() {
            // Key order is the mjs object literal's: schema_version, tags,
            // candidates.
            let mut out = Map::new();
            out.insert("schema_version".to_string(), fresh.schema_version.clone());
            out.insert("tags".to_string(), Value::Array(fresh.tags.clone()));
            out.insert(
                "candidates".to_string(),
                Value::Array(next_candidates.into_iter().map(Value::String).collect()),
            );
            let path = taxonomy_path(root);
            if let Some(dir) = path.parent() {
                ensure_dir(dir).map_err(|e| format!("ensure_dir {}: {e}", dir.display()))?;
            }
            write_json_atomic(&path, &Value::Object(out))
                .map_err(|e| format!("write {}: {e}", path.display()))?;
        }
        Ok(())
    })
}

/// `decisions.mjs:192` `classifyDecisionTags`. Bootstrap-safe: with no
/// taxonomy file this never refuses and never writes.
fn classify_decision_tags(root: &Path, tags: &[String]) -> Result<(), String> {
    let Some(taxonomy) = load_taxonomy(root) else {
        return Ok(());
    };
    if tags.is_empty() {
        // `DecisionsUntaggedRefusedError` (`decisions.mjs:135`).
        return Err(
            "decisions: docs/decisions/taxonomy.json exists — this decision event needs at least one tag. Pass --tags (e.g. \"billing,recall\")."
                .to_string(),
        );
    }
    let mut known: Vec<String> = taxonomy
        .tags
        .iter()
        .filter_map(|t| t.get("name").and_then(Value::as_str).map(str::to_string))
        .collect();
    known.extend(taxonomy.candidates.iter().cloned());
    let unknown: Vec<String> = tags.iter().filter(|t| !known.contains(t)).cloned().collect();
    if !unknown.is_empty() {
        append_taxonomy_candidates_sync(root, &unknown)?;
    }
    Ok(())
}

// ─── tag normalization (dp-1) ──────────────────────────────────────────────

/// `decisions.mjs:255` `TAG_PATTERN` = `/^[a-z0-9][a-z0-9-]*$/`, as its JS
/// literal source (the refusal message interpolates the pattern itself).
const TAG_PATTERN_SOURCE: &str = "/^[a-z0-9][a-z0-9-]*$/";

fn tag_pattern_matches(tag: &str) -> bool {
    let mut chars = tag.chars();
    let Some(first) = chars.next() else { return false };
    if !(first.is_ascii_lowercase() || first.is_ascii_digit()) {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// `JSON.stringify(str)` for the refusal messages' `${JSON.stringify(tag)}`
/// slot. `serde_json` escapes the same set JS does for a plain string.
fn json_quote(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| format!("\"{s}\""))
}

/// `decisions.mjs:260` `normalizeTags`.
///
/// `None` (mjs `undefined`/`null`) yields `Ok(None)` — the event gains NO
/// `tags` key at all, which is the additive zero-migration shape the 400+
/// pre-dp-1 events rely on. An EMPTY resolved list is also `None`, so
/// `--tags ""` and no flag at all write the same bytes.
fn normalize_tags(tags: Option<&[String]>, caller: &str) -> Result<Option<Vec<String>>, String> {
    let Some(tags) = tags else { return Ok(None) };
    let cleaned: Vec<String> = tags.iter().map(|t| js_trim(t).to_string()).collect();
    for tag in &cleaned {
        if !tag_pattern_matches(tag) {
            return Err(format!(
                "{caller}: tag {} is not a valid lowercase slug (must match {TAG_PATTERN_SOURCE}).",
                json_quote(tag)
            ));
        }
    }
    Ok(if cleaned.is_empty() { None } else { Some(cleaned) })
}

// ─── the docs/** citation sweep (dp-2, CONTEXT D2) ─────────────────────────

const SWEEP_TEXT_EXTENSIONS: &[&str] = &[".md", ".json", ".yaml", ".yml", ".txt"];
const SWEEP_EXCERPT_MAX: usize = 160;

/// `path.extname(name).toLowerCase()`. Node returns `""` when the only dot
/// is the FIRST character (`.gitignore`), and takes the LAST dot otherwise.
fn js_extname_lower(name: &str) -> String {
    match name.rfind('.') {
        None | Some(0) => String::new(),
        Some(i) => name[i..].to_lowercase(),
    }
}

/// `collectSweepFiles` — depth-first recursion in `readdirSync` order, with
/// an unreadable directory silently skipped (mjs `catch { return; }`).
///
/// # The sort is the port, not a tidy-up
///
/// `fs.readdirSync` is NOT insertion- or inode-ordered: libuv implements it
/// with `scandir(3)` and sorts every directory with `strcmp` on the entry
/// name, so Node hands the mjs source a BYTE-SORTED list. `std::fs::read_dir`
/// does no such thing — it yields raw directory order, which on tmpfs is
/// creation order and on ext4 is hash order. Without this sort the two
/// runtimes produce the same SET of citation hits in a different SEQUENCE,
/// and since the sweep result is serialized into the supersede event and
/// persisted, that is a byte divergence in the store itself. Caught by
/// `tests/decisions_lock_conformance.rs`'s oracle comparison, which is
/// exactly the class of drift no output diff over a docs-less fixture could
/// have shown.
///
/// `strcmp` compares BYTES, not code points and not case-insensitively, so
/// `Beta.md` sorts before `alpha.md`; comparing `to_string_lossy()` or
/// lowercased names would reintroduce the divergence in a subtler form.
fn collect_sweep_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut sorted: Vec<std::fs::DirEntry> = entries.flatten().collect();
    sorted.sort_by(|a, b| a.file_name().as_encoded_bytes().cmp(b.file_name().as_encoded_bytes()));
    for entry in sorted {
        let full = entry.path();
        let Ok(file_type) = entry.file_type() else { continue };
        if file_type.is_dir() {
            collect_sweep_files(&full, out);
        } else if file_type.is_file() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if SWEEP_TEXT_EXTENSIONS.contains(&js_extname_lower(&name).as_str()) {
                out.push(full);
            }
        }
    }
}

/// JS `\w` — ASCII only. A non-unicode regex's `\b` is defined on exactly
/// `[A-Za-z0-9_]`, so every non-ASCII scalar is a NON-word character here,
/// which is what makes `docs/…/1178cfce.md` and `abc1178cfcedef` behave
/// differently the way `sweepDecisionCitations` intends.
fn is_js_word(c: Option<char>) -> bool {
    matches!(c, Some(ch) if ch.is_ascii_alphanumeric() || ch == '_')
}

fn is_boundary(before: Option<char>, at: Option<char>) -> bool {
    is_js_word(before) != is_js_word(at)
}

/// ECMA-262 `Canonicalize` for a NON-unicode, case-insensitive regex: take
/// `toUpperCase()`, keep the original when that is not exactly one scalar,
/// and keep the original when a non-ASCII input canonicalizes to ASCII (the
/// rule that stops U+212A KELVIN SIGN matching a literal `k`).
fn canonicalize(c: char) -> char {
    let mut upper = c.to_uppercase();
    let Some(u) = upper.next() else { return c };
    if upper.next().is_some() {
        return c;
    }
    if !c.is_ascii() && u.is_ascii() {
        return c;
    }
    u
}

/// `new RegExp(`\\b${escapeRegExp(needle)}\\b`, 'i').test(line)`, without a
/// regex engine — the needle is always a literal, so the only semantics that
/// matter are canonical case folding and the two `\b` assertions.
fn cites(line_chars: &[char], needle: &[char]) -> bool {
    let n = needle.len();
    if n == 0 || line_chars.len() < n {
        return false;
    }
    for i in 0..=(line_chars.len() - n) {
        if !(0..n).all(|k| canonicalize(line_chars[i + k]) == needle[k]) {
            continue;
        }
        let before = if i == 0 { None } else { Some(line_chars[i - 1]) };
        if !is_boundary(before, Some(line_chars[i])) {
            continue;
        }
        if !is_boundary(Some(line_chars[i + n - 1]), line_chars.get(i + n).copied()) {
            continue;
        }
        return true;
    }
    false
}

/// `trimmed.length > 160 ? trimmed.slice(0, 157) + '...' : trimmed`.
///
/// `.length` and `.slice` are UTF-16 CODE UNIT operations, not scalar or
/// byte ones, so the measurement and the cut both happen in UTF-16 space.
/// A cut that would split a surrogate pair leaves mjs with a lone surrogate
/// (which `JSON.stringify` escapes as `\udXXX`); `serde_json` cannot hold
/// one, so this yields U+FFFD there instead. That divergence is reachable
/// only by a docs line whose 158th UTF-16 unit is a high surrogate, and is
/// recorded here rather than papered over.
fn js_excerpt(trimmed: &str) -> String {
    let units: Vec<u16> = trimmed.encode_utf16().collect();
    if units.len() <= SWEEP_EXCERPT_MAX {
        return trimmed.to_string();
    }
    format!("{}...", String::from_utf16_lossy(&units[..SWEEP_EXCERPT_MAX - 3]))
}

/// `text.split(/\r?\n/)`.
fn split_js_lines(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let bytes = text.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if *b == b'\n' {
            let end = if i > 0 && bytes[i - 1] == b'\r' { i - 1 } else { i };
            out.push(&text[start..end]);
            start = i + 1;
        }
    }
    out.push(&text[start..]);
    out
}

/// `decisions.mjs:380` `sweepDecisionCitations`. Read-only: it never edits a
/// citing file. `scanned_at` is threaded in from the caller so the store
/// layer stays deterministic, the same discipline [`crate::capture`] uses.
///
/// Result key order is the mjs literal's — `scanned_at, hit_count, files`,
/// and per hit `file, line, excerpt`.
pub fn sweep_decision_citations(root: &Path, id: &str, short8: &str, scanned_at: &str) -> Value {
    let docs_root = root.join("docs");
    let mut candidate_files: Vec<PathBuf> = Vec::new();
    collect_sweep_files(&docs_root, &mut candidate_files);

    let id_needle: Vec<char> = id.chars().map(canonicalize).collect();
    let short_needle: Vec<char> = short8.chars().map(canonicalize).collect();

    let mut files: Vec<Value> = Vec::new();
    for file in &candidate_files {
        let Ok(text) = std::fs::read_to_string(file) else { continue };
        for (index, line) in split_js_lines(&text).into_iter().enumerate() {
            let line_chars: Vec<char> = line.chars().collect();
            if !cites(&line_chars, &id_needle) && !cites(&line_chars, &short_needle) {
                continue;
            }
            let trimmed = js_trim(line);
            let rel = file.strip_prefix(root).unwrap_or(file);
            let mut hit = Map::new();
            hit.insert(
                "file".to_string(),
                Value::String(rel.to_string_lossy().replace('\\', "/")),
            );
            hit.insert("line".to_string(), Value::from(index + 1));
            hit.insert("excerpt".to_string(), Value::String(js_excerpt(trimmed)));
            files.push(Value::Object(hit));
        }
    }

    let mut sweep = Map::new();
    sweep.insert("scanned_at".to_string(), Value::String(scanned_at.to_string()));
    sweep.insert("hit_count".to_string(), Value::from(files.len()));
    sweep.insert("files".to_string(), Value::Array(files));
    Value::Object(sweep)
}

// ─── the three write verbs ─────────────────────────────────────────────────

/// The inputs `handleDecisionsLog` (`bee.mjs:1932`) hands over, already
/// coerced by the CLI edge.
#[derive(Debug, Default, Clone)]
pub struct LogFields<'a> {
    pub decision: &'a str,
    pub rationale: &'a str,
    /// `flags.alternatives ? String(flags.alternatives) : null`.
    pub alternatives: Option<&'a str>,
    /// `flags.scope ? String(flags.scope) : 'repo'` — already defaulted.
    pub scope: &'a str,
    /// `flags.source ? String(flags.source) : 'user'` — already defaulted.
    pub source: &'a str,
    /// Parsed `--confidence`, or `None` for the mjs `null`.
    pub confidence: Option<i64>,
    /// `None` = flag absent (mjs `undefined`). `Some(list)` = `splitList`'s
    /// output, which may legitimately be empty.
    pub tags: Option<&'a [String]>,
}

fn opt_string(v: Option<&str>) -> Value {
    match v {
        Some(s) => Value::String(s.to_string()),
        None => Value::Null,
    }
}

/// `decisions.mjs:300` `logDecision`.
///
/// `id` and `date` are threaded in from the CLI edge rather than read here,
/// the discipline [`crate::capture::add_capture_stub`] established: the
/// store layer stays deterministic and unit-testable, and the wall clock is
/// stamped once, in [`crate::lock::iso8601_millis`]'s exact
/// `Date.prototype.toISOString()` shape.
pub fn log_decision(root: &Path, fields: &LogFields<'_>, id: &str, date: &str) -> Result<Value, String> {
    if js_trim(fields.decision).is_empty() {
        return Err("logDecision: decision text is required.".to_string());
    }
    if js_trim(fields.rationale).is_empty() {
        return Err("logDecision: rationale is required.".to_string());
    }
    // `assertSafe({decision, rationale, alternatives, scope, source})` —
    // `Object.entries` order, on the RAW (untrimmed) values.
    assert_safe_decision_content("decision", fields.decision)?;
    assert_safe_decision_content("rationale", fields.rationale)?;
    if let Some(alternatives) = fields.alternatives {
        assert_safe_decision_content("alternatives", alternatives)?;
    }
    assert_safe_decision_content("scope", fields.scope)?;
    assert_safe_decision_content("source", fields.source)?;

    let normalized_tags = normalize_tags(fields.tags, "logDecision")?;
    classify_decision_tags(root, normalized_tags.as_deref().unwrap_or(&[]))?;

    // Key order is the mjs object literal's (`decisions.mjs:319`): id, type,
    // date, decision, rationale, alternatives, scope, source, confidence —
    // then `tags` APPENDED only when present.
    let mut event = Map::new();
    event.insert("id".to_string(), Value::String(id.to_string()));
    event.insert("type".to_string(), Value::String("decide".to_string()));
    event.insert("date".to_string(), Value::String(date.to_string()));
    event.insert("decision".to_string(), Value::String(js_trim(fields.decision).to_string()));
    event.insert("rationale".to_string(), Value::String(js_trim(fields.rationale).to_string()));
    event.insert("alternatives".to_string(), opt_string(fields.alternatives));
    event.insert("scope".to_string(), Value::String(fields.scope.to_string()));
    event.insert("source".to_string(), Value::String(fields.source.to_string()));
    event.insert(
        "confidence".to_string(),
        fields.confidence.map(Value::from).unwrap_or(Value::Null),
    );
    if let Some(tags) = &normalized_tags {
        event.insert(
            "tags".to_string(),
            Value::Array(tags.iter().cloned().map(Value::String).collect()),
        );
    }
    let event = Value::Object(event);

    append_event(root, &event)?;
    Ok(event)
}

/// The inputs `handleDecisionsSupersede` (`bee.mjs:1966`) hands over.
#[derive(Debug, Default, Clone)]
pub struct SupersedeFields<'a> {
    pub supersedes: &'a str,
    pub decision: &'a str,
    pub rationale: &'a str,
    /// `None` = `--tags` absent (INHERIT from the target); `Some` = explicit.
    pub tags: Option<&'a [String]>,
    /// `None` = `--scope` absent (INHERIT); `Some` = explicit, even if blank.
    pub scope: Option<&'a str>,
}

/// `decisions.mjs:407` `supersedeDecision`.
///
/// `scanned_at` and `date` are two SEPARATE clock reads in the mjs source
/// (the sweep runs before the event literal is built), so they are threaded
/// separately rather than collapsed into one stamp.
pub fn supersede_decision(
    root: &Path,
    fields: &SupersedeFields<'_>,
    id: &str,
    scanned_at: &str,
    date: &str,
) -> Result<Value, String> {
    if js_trim(fields.supersedes).is_empty() {
        return Err("supersedeDecision: supersedes (decision id) is required.".to_string());
    }
    if js_trim(fields.decision).is_empty() {
        return Err("supersedeDecision: replacement decision text is required.".to_string());
    }
    if js_trim(fields.rationale).is_empty() {
        return Err("supersedeDecision: rationale is required.".to_string());
    }
    let target_id = js_trim(fields.supersedes).to_string();
    assert_safe_decision_content("decision", fields.decision)?;
    assert_safe_decision_content("rationale", fields.rationale)?;

    // dp-6 plan-check W3: inheritance consults the OVERLAY-APPLIED target,
    // so a legacy target classified only via a retro-`tag` event still reads
    // as tagged. `buildTagOverlay` re-reads the journal in mjs; this shares
    // the one read already taken, exactly as rust-port-23 did for
    // `active_decisions` — same events, same instant, one parse.
    let events: Vec<Value> = read_jsonl(&decisions_path(root));
    let overlay = build_tag_overlay_from(&events);
    let target = events
        .iter()
        .find(|e| e.get("id").and_then(Value::as_str) == Some(target_id.as_str()))
        .map(|raw| apply_tag_overlay(raw, &overlay));

    let resolved_scope = match fields.scope {
        Some(s) if !js_trim(s).is_empty() => js_trim(s).to_string(),
        _ => match target.as_ref().and_then(|t| truthy_str(t.get("scope"))) {
            Some(s) if !js_trim(s).is_empty() => js_trim(s).to_string(),
            _ => "repo".to_string(),
        },
    };
    assert_safe_decision_content("scope", &resolved_scope)?;

    // `normalizeTags` is shared, so its refusal text says `logDecision:`
    // even on this path — mjs hardcodes that prefix inside the helper.
    let resolved_tags = if fields.tags.is_some() {
        normalize_tags(fields.tags, "logDecision")?
    } else {
        // `target.tags` is arbitrary JSON in principle; mjs hands it
        // straight back to `normalizeTags`, which `String()`s each member
        // before validating. Only a NON-EMPTY array inherits — a target with
        // `tags: []` falls through to no tags key at all.
        let from_target: Vec<String> = target
            .as_ref()
            .and_then(|t| t.get("tags"))
            .and_then(Value::as_array)
            .map(|a| a.iter().map(js_string_value).collect())
            .unwrap_or_default();
        if from_target.is_empty() {
            None
        } else {
            normalize_tags(Some(&from_target), "logDecision")?
        }
    };

    classify_decision_tags(root, resolved_tags.as_deref().unwrap_or(&[]))?;

    // dp-2 lock doctrine: the sweep is computed BEFORE the append, so the
    // event is written to the store exactly once, already carrying its
    // result inline. Never a post-append rewrite of a written jsonl line.
    let short8: String = target_id.chars().take(8).collect();
    let sweep = sweep_decision_citations(root, &target_id, &short8, scanned_at);

    // Key order is the mjs object literal's (`decisions.mjs:468`): id, type,
    // date, supersedes, decision, rationale, scope, sweep [, tags].
    let mut event = Map::new();
    event.insert("id".to_string(), Value::String(id.to_string()));
    event.insert("type".to_string(), Value::String("supersede".to_string()));
    event.insert("date".to_string(), Value::String(date.to_string()));
    event.insert("supersedes".to_string(), Value::String(target_id));
    event.insert("decision".to_string(), Value::String(js_trim(fields.decision).to_string()));
    event.insert("rationale".to_string(), Value::String(js_trim(fields.rationale).to_string()));
    event.insert("scope".to_string(), Value::String(resolved_scope));
    event.insert("sweep".to_string(), sweep);
    if let Some(tags) = &resolved_tags {
        event.insert(
            "tags".to_string(),
            Value::Array(tags.iter().cloned().map(Value::String).collect()),
        );
    }
    let event = Value::Object(event);

    append_event(root, &event)?;
    Ok(event)
}

/// `decisions.mjs:488` `redactDecision`.
///
/// Note what is NOT here, faithfully: redact validates only that both flags
/// are non-blank. It never checks that the target exists, never runs the
/// content guard, and never refuses an already-redacted id — a second redact
/// of the same id is a second, perfectly ordinary event.
pub fn redact_decision(root: &Path, redacts: &str, reason: &str, id: &str, date: &str) -> Result<Value, String> {
    if js_trim(redacts).is_empty() {
        return Err("redactDecision: redacts (decision id) is required.".to_string());
    }
    if js_trim(reason).is_empty() {
        return Err("redactDecision: reason is required.".to_string());
    }
    // Key order is the mjs object literal's (`decisions.mjs:495`).
    let mut event = Map::new();
    event.insert("id".to_string(), Value::String(id.to_string()));
    event.insert("type".to_string(), Value::String("redact".to_string()));
    event.insert("date".to_string(), Value::String(date.to_string()));
    event.insert("redacts".to_string(), Value::String(js_trim(redacts).to_string()));
    event.insert("reason".to_string(), Value::String(js_trim(reason).to_string()));
    let event = Value::Object(event);

    append_event(root, &event)?;
    Ok(event)
}

/// The shared tail of all three verbs: ONE locked `appendJsonl`. Either the
/// append lands fully before `archiveDecisions` reads the file, or fully
/// after its rename — never mid-transaction.
fn append_event(root: &Path, event: &Value) -> Result<(), String> {
    with_decisions_lock_sync(root, || {
        let path = decisions_path(root);
        append_jsonl(&path, event).map_err(|e| format!("append {}: {e}", path.display()))
    })
}

#[cfg(test)]
mod write_tests {
    use super::*;

    /// The `jsonout` integer-key hoisting hazard cannot reach this store:
    /// every key any of the three events (or a sweep hit) can carry is a
    /// fixed literal, and none of them is integer-like. This is the check
    /// behind the module doc's claim — if a later cell adds a dynamic key,
    /// this test is where the claim stops being free.
    #[test]
    fn event_keys_are_never_integer_like() {
        let keys = [
            "id", "type", "date", "decision", "rationale", "alternatives", "scope", "source",
            "confidence", "tags", "supersedes", "sweep", "redacts", "reason", "scanned_at",
            "hit_count", "files", "file", "line", "excerpt",
        ];
        for key in keys {
            assert!(
                key.parse::<u32>().is_err(),
                "{key} is integer-like — JSON.stringify would hoist it and the appended line would drift"
            );
        }
    }

    #[test]
    fn lock_busy_message_matches_the_mjs_spelling() {
        let holder = serde_json::json!({"pid": 4321, "session": "sess-a", "ts": "2026-07-27T00:00:00.000Z"});
        assert_eq!(
            decisions_lock_busy_message(Some(&holder)),
            "decisions store lock \"decisions\" busy: held by pid=4321 session=sess-a since 2026-07-27T00:00:00.000Z"
        );
        // `??` catches null — a session-less lock body reads `unknown`, not
        // `null`.
        let nulled = serde_json::json!({"pid": 7, "session": Value::Null, "ts": Value::Null});
        assert_eq!(
            decisions_lock_busy_message(Some(&nulled)),
            "decisions store lock \"decisions\" busy: held by pid=7 session=unknown since unknown"
        );
        assert_eq!(
            decisions_lock_busy_message(None),
            "decisions store lock \"decisions\" busy: held by unknown holder"
        );
    }

    /// `\b` is ASCII-word-boundary, so a hyphen is itself a boundary: an id
    /// embedded in a longer ALNUM run never matches, but one followed by a
    /// hyphen does.
    #[test]
    fn citation_word_boundary_follows_js_semantics() {
        let needle: Vec<char> = "1178cfce".chars().map(canonicalize).collect();
        let hit = |s: &str| cites(&s.chars().collect::<Vec<_>>(), &needle);
        assert!(hit("see decision 1178cfce for details"));
        assert!(hit("1178CFCE")); // case-insensitive
        assert!(hit("(1178cfce)"));
        assert!(hit("1178cfce-abc")); // '-' is a non-word char: boundary holds
        assert!(!hit("abc1178cfcedef"));
        assert!(!hit("x1178cfce"));
        assert!(!hit("1178cfced"));
        // U+212A KELVIN SIGN must NOT canonicalize onto ASCII 'k'.
        let k: Vec<char> = "k".chars().map(canonicalize).collect();
        assert!(!cites(&"\u{212A}".chars().collect::<Vec<_>>(), &k));
    }

    #[test]
    fn excerpt_truncates_in_utf16_units_like_slice() {
        let short = "a short line";
        assert_eq!(js_excerpt(short), short);
        let long = "x".repeat(200);
        let cut = js_excerpt(&long);
        assert_eq!(cut.chars().count(), SWEEP_EXCERPT_MAX);
        assert!(cut.ends_with("..."));
        // A BMP-only 160-unit line is exactly at the boundary and is NOT cut.
        let boundary = "é".repeat(SWEEP_EXCERPT_MAX);
        assert_eq!(js_excerpt(&boundary), boundary);
        // 160 ASTRAL scalars are 320 UTF-16 units, so they ARE cut.
        let astral = "🐝".repeat(SWEEP_EXCERPT_MAX);
        assert!(js_excerpt(&astral).ends_with("..."));
    }

    #[test]
    fn extname_matches_node() {
        assert_eq!(js_extname_lower("a.MD"), ".md");
        assert_eq!(js_extname_lower("a.b.yaml"), ".yaml");
        assert_eq!(js_extname_lower(".gitignore"), "");
        assert_eq!(js_extname_lower("plain"), "");
    }

    #[test]
    fn split_lines_handles_crlf_and_a_trailing_newline() {
        assert_eq!(split_js_lines("a\r\nb\nc"), vec!["a", "b", "c"]);
        // A trailing newline yields a final EMPTY element, exactly as
        // `"a\n".split(/\r?\n/)` does — which is why line numbers stay
        // 1-based and correct.
        assert_eq!(split_js_lines("a\n"), vec!["a", ""]);
    }

    #[test]
    fn tag_pattern_refusal_names_the_js_literal() {
        let err = normalize_tags(Some(&["Bad Tag".to_string()]), "logDecision").unwrap_err();
        assert_eq!(
            err,
            "logDecision: tag \"Bad Tag\" is not a valid lowercase slug (must match /^[a-z0-9][a-z0-9-]*$/)."
        );
        assert_eq!(normalize_tags(None, "logDecision").unwrap(), None);
        // An empty resolved list writes no `tags` key at all.
        assert_eq!(normalize_tags(Some(&[]), "logDecision").unwrap(), None);
    }
}
