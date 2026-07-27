//! Fixture generator — the D5 host-real `.bee` store (CONTEXT.md D5,
//! rust-port-2 truths, grown by rust-port-19 to also exercise the two
//! expensive `status` blocks the original pinned-size-only fixture missed).
//! Produces a self-sufficient store at pinned minimum sizes (decisions.jsonl
//! >= 700 KB, reservations.json >= 600 KB, backlog.jsonl >= 250 KB, >= 250
//! cell files, >= 60 review candidates, >= 50 git commits, a >= 300 KB crash-
//! candidate transcript) that:
//!
//! - resolves as a bee root on its own: `.bee/onboarding.json` is present,
//!   and (rust-port-19) a real git repo lives at the SAME directory as
//!   `.bee/` — `resolveRootsCore`'s first branch (no `.git` at that level)
//!   therefore does NOT match, but its git-root fallback branch does, and
//!   resolves `storeRoot === workRoot === this fixture root` either way
//!   (proven empirically against the real `bee.mjs`, see selftest.rs's
//!   sanity_read `root_resolution` assertion) — this is a DELIBERATE
//!   divergence from rust-port-2's original no-`.git` fixture: growing real
//!   ancestry into the store means a `.git` must exist somewhere, and the
//!   fixture root itself is the only place that keeps root resolution
//!   correct (per the validation decision this cell implements),
//! - carries genuine git ancestry (rust-port-19): a real git history so
//!   `deriveCandidateStatus`'s `merge-base --is-ancestor` / `rev-list
//!   --count` calls answer real questions instead of matching zero rows,
//! - carries a review-candidates ledger + review sessions (rust-port-19)
//!   engineered to hit all four `CANDIDATE_STATUSES` (unreviewed, in review,
//!   reviewed, review stale — the last one via both a real "commits since"
//!   count AND an unresolvable-ancestry candidate head), so
//!   `buildReviewBlock` does real derivation work instead of looping zero
//!   times,
//! - carries one heartbeat-stale, non-clean-end crash candidate session with
//!   an INJECTED transcript root (rust-port-19: `.bee/config.json`
//!   `recovery.transcript_roots`, never the host `$HOME/.claude/projects`)
//!   so `detectCrashCandidates`'s encoded-project-dir matching
//!   (`recovery.mjs`, mirroring `perf.mjs`'s `encodeProjectDir`) actually
//!   resolves a transcript and `readTranscriptTail` genuinely reads past its
//!   256 KB default window, and
//! - survives a real `node .bee/bin/bee.mjs status --json` read without
//!   crashing (every reader in the mjs estate is fail-open on missing/
//!   malformed substructure — proven empirically, not just read from source)
//!   AND reports non-trivial `review`/`recovery` blocks on that read
//!   (selftest.rs's `non_triviality` assertion — a fixture that still
//!   measures nothing is a failed generation, not a pass).
//!
//! `generate` REFUSES (returns `Err`, writes nothing) if any requested size
//! is below its pinned floor — the D5 acceptance instrument must never
//! silently benchmark against a smaller-than-host-real store.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Pinned minimums from CONTEXT.md D5 / plan.md cell rust-port-2.
pub const DECISIONS_FLOOR_BYTES: u64 = 700_000;
pub const RESERVATIONS_FLOOR_BYTES: u64 = 600_000;
pub const BACKLOG_FLOOR_BYTES: u64 = 250_000;
pub const CELLS_FLOOR_COUNT: usize = 250;

/// Pinned minimums added by rust-port-19 (validation decision 2026-07-26):
/// the three costs the original fixture measured as zero (git ancestry,
/// review candidates, transcript tail). REFUSED below, exactly like the
/// four floors above — never silently shrunk to make generation succeed.
pub const REVIEW_CANDIDATES_FLOOR_COUNT: usize = 60;
pub const GIT_COMMITS_FLOOR_COUNT: usize = 50;
pub const TRANSCRIPT_TAIL_FLOOR_BYTES: u64 = 300_000;

/// Pinned minimum added by rpl-2 (validation-slice4 matrix row 7): the
/// generator wrote NO intent store at all, so every `--cmd-check --group
/// intent` scenario would have diffed two EMPTY directories and reported a
/// meaningless clean. Same REFUSE-below discipline as the seven floors above
/// — never shrunk to make generation succeed, and additive to all of them.
///
/// Unlike those, this one is not request-driven: the seeded key set is fixed
/// (see [`INTENT_SEED_KEYS`]) because each key exists to exercise a specific
/// shape, not to reach a size. The floor is checked against what actually
/// landed on disk after the write, so a seeding regression is a refusal
/// rather than a quietly smaller store.
pub const INTENT_KEYS_FLOOR_COUNT: usize = 4;

#[derive(Debug, Clone)]
pub struct FixtureRequest {
    pub out_dir: PathBuf,
    pub decisions_bytes: u64,
    pub reservations_bytes: u64,
    pub backlog_bytes: u64,
    pub cells_count: usize,
    pub review_candidates_count: usize,
    pub git_commits_count: usize,
    pub transcript_tail_bytes: u64,
}

impl FixtureRequest {
    /// A request pinned to exactly the D5 floors — the happy-path shape.
    pub fn at_floors(out_dir: PathBuf) -> Self {
        Self {
            out_dir,
            decisions_bytes: DECISIONS_FLOOR_BYTES,
            reservations_bytes: RESERVATIONS_FLOOR_BYTES,
            backlog_bytes: BACKLOG_FLOOR_BYTES,
            cells_count: CELLS_FLOOR_COUNT,
            review_candidates_count: REVIEW_CANDIDATES_FLOOR_COUNT,
            git_commits_count: GIT_COMMITS_FLOOR_COUNT,
            transcript_tail_bytes: TRANSCRIPT_TAIL_FLOOR_BYTES,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FixtureReport {
    pub root: PathBuf,
    pub decisions_bytes: u64,
    pub reservations_bytes: u64,
    pub backlog_bytes: u64,
    pub cells_count: usize,
    pub git_commits_count: usize,
    pub review_candidates_count: usize,
    pub transcript_tail_bytes: u64,
    pub transcript_file: PathBuf,
    pub crash_session_id: String,
    /// The intent-anchor keys actually written under `.bee/intent/`
    /// (rpl-2). Reported rather than merely counted so a reader of
    /// `--generate`'s own output can SEE which shapes the store carries —
    /// the must_have is "proven by the generator's own output listing the
    /// seeded keys", and a bare count would not prove the unicode or the
    /// 120-character key is there.
    pub intent_keys: Vec<String>,
}

impl FixtureReport {
    pub fn to_json(&self) -> String {
        let intent_keys = self
            .intent_keys
            .iter()
            .map(|k| format!("\"{}\"", json_escape(k)))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"root\":\"{}\",\"decisions_bytes\":{},\"reservations_bytes\":{},\"backlog_bytes\":{},\"cells_count\":{},\"git_commits_count\":{},\"review_candidates_count\":{},\"transcript_tail_bytes\":{},\"transcript_file\":\"{}\",\"crash_session_id\":\"{}\",\"intent_keys_count\":{},\"intent_keys\":[{}]}}",
            json_escape(&self.root.display().to_string()),
            self.decisions_bytes,
            self.reservations_bytes,
            self.backlog_bytes,
            self.cells_count,
            self.git_commits_count,
            self.review_candidates_count,
            self.transcript_tail_bytes,
            json_escape(&self.transcript_file.display().to_string()),
            json_escape(&self.crash_session_id),
            self.intent_keys.len(),
            intent_keys,
        )
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Validate the four pins before touching the filesystem. Returns the first
/// violation found (decisions, then reservations, then backlog, then cells)
/// so a caller driving all-four-refuse coverage gets a distinct message per
/// case.
fn check_floors(req: &FixtureRequest) -> Result<(), String> {
    if req.decisions_bytes < DECISIONS_FLOOR_BYTES {
        return Err(format!(
            "decisions.jsonl requested {} byte(s) < pinned floor {} byte(s) (CONTEXT.md D5) — refusing to generate a sub-pinned-size fixture",
            req.decisions_bytes, DECISIONS_FLOOR_BYTES
        ));
    }
    if req.reservations_bytes < RESERVATIONS_FLOOR_BYTES {
        return Err(format!(
            "reservations.json requested {} byte(s) < pinned floor {} byte(s) (CONTEXT.md D5) — refusing to generate a sub-pinned-size fixture",
            req.reservations_bytes, RESERVATIONS_FLOOR_BYTES
        ));
    }
    if req.backlog_bytes < BACKLOG_FLOOR_BYTES {
        return Err(format!(
            "backlog.jsonl requested {} byte(s) < pinned floor {} byte(s) (CONTEXT.md D5) — refusing to generate a sub-pinned-size fixture",
            req.backlog_bytes, BACKLOG_FLOOR_BYTES
        ));
    }
    if req.cells_count < CELLS_FLOOR_COUNT {
        return Err(format!(
            "cells count requested {} < pinned floor {} (CONTEXT.md D5) — refusing to generate a sub-pinned-size fixture",
            req.cells_count, CELLS_FLOOR_COUNT
        ));
    }
    if req.review_candidates_count < REVIEW_CANDIDATES_FLOOR_COUNT {
        return Err(format!(
            "review candidates count requested {} < pinned floor {} (rust-port-19) — refusing to generate a fixture that would leave buildReviewBlock measuring near-zero rows",
            req.review_candidates_count, REVIEW_CANDIDATES_FLOOR_COUNT
        ));
    }
    if req.git_commits_count < GIT_COMMITS_FLOOR_COUNT {
        return Err(format!(
            "git commits count requested {} < pinned floor {} (rust-port-19) — refusing to generate a fixture with too little real ancestry for deriveCandidateStatus's git questions to be genuine",
            req.git_commits_count, GIT_COMMITS_FLOOR_COUNT
        ));
    }
    if req.transcript_tail_bytes < TRANSCRIPT_TAIL_FLOOR_BYTES {
        return Err(format!(
            "transcript tail bytes requested {} < pinned floor {} (rust-port-19) — refusing to generate a crash-candidate transcript smaller than readTranscriptTail's own default 256 KB window",
            req.transcript_tail_bytes, TRANSCRIPT_TAIL_FLOOR_BYTES
        ));
    }
    Ok(())
}

/// Generate the fixture at `req.out_dir`. Refuses (no writes at all) if any
/// pin is under its floor; otherwise writes a full `.bee/` store and returns
/// the byte/file counts actually written (measured, not just echoed back).
pub fn generate(req: &FixtureRequest) -> Result<FixtureReport, String> {
    check_floors(req)?;

    let bee_dir = req.out_dir.join(".bee");
    let cells_dir = bee_dir.join("cells");
    fs::create_dir_all(&cells_dir).map_err(|e| format!("mkdir {}: {e}", cells_dir.display()))?;

    // Canonicalized only AFTER the directory exists (canonicalize requires a
    // real path). This is the exact string `encodeProjectDir` mirrors below
    // must match against — Node's own `process.cwd()` when the sanity-read
    // subprocess's cwd is this same directory resolves the same realpath on
    // POSIX, so computing the encoded transcript subdir name here (fixture
    // generation time) rather than re-deriving it at read time keeps the two
    // sides from ever drifting apart.
    let canonical_root = fs::canonicalize(&req.out_dir)
        .map_err(|e| format!("canonicalize {}: {e}", req.out_dir.display()))?;
    let transcript_root = canonical_root.join("fixture-transcripts");

    write_onboarding(&bee_dir)?;
    write_state(&bee_dir)?;
    write_config(&bee_dir, &transcript_root)?;
    write_command_registry(&bee_dir)?;

    let decisions_bytes = write_jsonl_padded(&bee_dir.join("decisions.jsonl"), req.decisions_bytes, decision_line)?;
    let backlog_bytes = write_jsonl_padded(&bee_dir.join("backlog.jsonl"), req.backlog_bytes, backlog_line)?;
    let reservations_bytes = write_reservations_padded(&bee_dir.join("reservations.json"), req.reservations_bytes)?;

    for i in 0..req.cells_count {
        let path = cells_dir.join(format!("fixture-{i:05}.json"));
        fs::write(&path, cell_json(i)).map_err(|e| format!("write {}: {e}", path.display()))?;
    }

    // rust-port-19: real git ancestry co-located with `.bee/` (see the
    // module doc comment for why this still resolves the fixture as its own
    // store root), a review-candidates ledger + review sessions engineered
    // to hit all four derived statuses, and one injected-transcript crash
    // candidate — the three costs the pre-growth fixture measured as zero.
    let commits = init_git_repo(&canonical_root, req.git_commits_count)?;
    let review_candidates_count = write_review_state(&bee_dir, &commits, req.review_candidates_count)?;
    let (transcript_tail_bytes, transcript_file, crash_session_id) =
        write_crash_candidate(&canonical_root, &bee_dir, &transcript_root, req.transcript_tail_bytes)?;

    // rpl-2: the intent store. Before this, `.bee/intent/` did not exist at
    // all in a generated fixture (validation-slice4 matrix row 7), so an
    // intent parity scenario would have compared two absent directories.
    let intent_keys = write_intent_store(&bee_dir)?;

    Ok(FixtureReport {
        root: req.out_dir.clone(),
        decisions_bytes,
        reservations_bytes,
        backlog_bytes,
        cells_count: req.cells_count,
        git_commits_count: commits.len(),
        review_candidates_count,
        transcript_tail_bytes,
        transcript_file,
        crash_session_id,
        intent_keys,
    })
}

// ─── the intent store (rpl-2) ──────────────────────────────────────────────

/// The FIXED seeded key set. Each entry is here for a shape, not for a size:
///
/// * `default` — the shared last-resort key (`intent.mjs:43`), the one every
///   candidate walk ends at, so every no-flag read lands on a real record.
/// * `queen-bench-fixture` — feature-shaped, matching the `feature` slug the
///   generated cells already carry, so a state-driven key resolution has a
///   record to find.
/// * `etc-passwd` — the SANITIZED form of the traversal-shaped key
///   `../../etc/passwd`. Seeded under the sanitized name deliberately: that
///   is the only name either runtime can ever produce for that input, so a
///   traversal scenario reads a real anchor instead of an absent one.
/// * a 120-character key — exactly at `.slice(0, 120)`'s cap, so a store
///   already holding a key at the boundary is the baseline the long-key
///   scenarios run against.
/// * a genuinely NON-ASCII filename — `sanitizeIntentKey` can never produce
///   one, which is exactly why it belongs in the fixture: a real store can
///   hold hand-placed or foreign-tool files, and both runtimes must walk
///   past it without touching it, reading it, or failing on it.
///
/// Returned (not just counted) so `--generate`'s own JSON output lists them.
fn intent_seed_keys() -> Vec<String> {
    let long_prefix = "queen-bench-fixture-long-intent-key-";
    let long_key = format!("{long_prefix}{}", "x".repeat(120 - long_prefix.len()));
    vec![
        "default".to_string(),
        "queen-bench-fixture".to_string(),
        "etc-passwd".to_string(),
        long_key,
        // U+1F41D HONEY BEE plus Vietnamese diacritics — multi-byte in UTF-8
        // and, for the bee, a surrogate PAIR in UTF-16.
        "ý-định-\u{1f41d}-neo".to_string(),
    ]
}

/// One seeded anchor, in the EXACT bytes `writeJsonAtomic` produces:
/// `JSON.stringify(obj, null, 2)` plus a trailing newline
/// (`packages/bee/lib/fsutil.mjs:95`), with the key order of the
/// `writeIntent` object literal (`intent.mjs:203-215`). Written as bytes
/// rather than through a serializer so the fixture cannot drift from the
/// frozen writer's shape without this literal being edited.
///
/// `written_at` is a FIXED timestamp, never `now`: every clone of the golden
/// fixture must carry identical bytes, or the tree diff of a file neither
/// runtime touched would depend on when the fixture was generated.
fn intent_anchor_json(key: &str, index: usize) -> String {
    let request = format!(
        "Yêu cầu số {index} — the user's VERBATIM words, kept byte for byte: \"don't paraphrase me\" · 🐝 · tab\\there · ümlaut."
    );
    let acceptance = format!("Xong khi: fixture anchor {index} round-trips byte-identically through both runtimes.");
    format!(
        "{{\n  \"schema_version\": \"1.0\",\n  \"key\": \"{key}\",\n  \"written_at\": \"2026-07-26T00:00:00.000Z\",\n  \"request\": \"{request}\",\n  \"acceptance\": \"{acceptance}\",\n  \"next_action\": \"Bước {index}: keep going\",\n  \"feature\": \"queen-bench-fixture\",\n  \"lane\": \"tiny\",\n  \"cell\": \"fixture-cell-{index:05}\",\n  \"do_not_reverse\": [\n    \"đừng đảo ngược {index}\",\n    \"second rule\"\n  ],\n  \"stop_conditions\": [\n    \"dừng nếu {index}\"\n  ]\n}}\n",
        key = json_escape(key),
        request = json_escape(&request),
        acceptance = json_escape(&acceptance),
    )
}

/// Seed `.bee/intent/` and REFUSE if fewer than [`INTENT_KEYS_FLOOR_COUNT`]
/// files actually landed. The count is taken by reading the directory back,
/// not from the input list — a floor checked against the same list that
/// produced the writes would prove nothing about what is on disk.
fn write_intent_store(bee_dir: &Path) -> Result<Vec<String>, String> {
    let dir = bee_dir.join("intent");
    fs::create_dir_all(&dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
    let keys = intent_seed_keys();
    for (i, key) in keys.iter().enumerate() {
        let path = dir.join(format!("{key}.json"));
        fs::write(&path, intent_anchor_json(key, i + 1))
            .map_err(|e| format!("write {}: {e}", path.display()))?;
    }
    let written = fs::read_dir(&dir)
        .map_err(|e| format!("read_dir {}: {e}", dir.display()))?
        .filter(|e| e.as_ref().map(|e| e.path().is_file()).unwrap_or(false))
        .count();
    if written < INTENT_KEYS_FLOOR_COUNT {
        return Err(format!(
            "intent store wrote {written} anchor file(s) < pinned floor {INTENT_KEYS_FLOOR_COUNT} (rpl-2, validation-slice4 matrix row 7) — refusing to generate a fixture whose intent scenarios would diff two near-empty directories and report a meaningless clean"
        ));
    }
    Ok(keys)
}

fn write_onboarding(bee_dir: &Path) -> Result<(), String> {
    // Minimal but real: bee_version present (installed:true), managed:{}
    // (empty — computeRuntimeDrift's checkGroup short-circuits on a
    // non-object/absent recorded map, so this fixture reports drift:false
    // without needing to carry the whole vendored .bee/bin tree).
    let path = bee_dir.join("onboarding.json");
    let body = "{\"schema_version\":\"1.0\",\"bee_version\":\"1.18.2\",\"managed\":{},\"created_at\":\"2026-07-26T00:00:00.000Z\",\"updated_at\":\"2026-07-26T00:00:00.000Z\"}";
    fs::write(&path, body).map_err(|e| format!("write {}: {e}", path.display()))
}

fn write_state(bee_dir: &Path) -> Result<(), String> {
    let path = bee_dir.join("state.json");
    let body = "{\"schema_version\":\"1.0\",\"phase\":\"idle\",\"feature\":null,\"mode\":null,\"approved_gates\":{\"context\":false,\"shape\":false,\"execution\":false,\"review\":false},\"workers\":[],\"summary\":\"\",\"next_action\":\"No active bee work.\"}";
    fs::write(&path, body).map_err(|e| format!("write {}: {e}", path.display()))
}

/// `recovery.transcript_roots` (rust-port-19) points ONLY at the injected,
/// fixture-local `transcript_root` — never the host `$HOME/.claude/projects`
/// (normalizeTranscriptRootsConfig/scanTranscriptRoots, `recovery.mjs`). The
/// default Claude root is still scanned first by the real mjs code at read
/// time (that behavior is frozen, D1), but it never holds this fixture's
/// candidate, so resolution always falls through to this configured entry.
fn write_config(bee_dir: &Path, transcript_root: &Path) -> Result<(), String> {
    let path = bee_dir.join("config.json");
    let body = format!(
        "{{\"hooks\":{{}},\"lanes\":{{}},\"capabilities\":{{}},\"commands\":{{}},\"gate_bypass\":\"off\",\"models\":{{}},\"recovery\":{{\"transcript_roots\":[{{\"runtime\":\"fixture\",\"path\":\"{}\"}}]}}}}",
        json_escape(&transcript_root.display().to_string())
    );
    fs::write(&path, body).map_err(|e| format!("write {}: {e}", path.display()))
}

/// `crates/queen-bench` at compile time -> repo root two levels up. Same
/// technique as `queen-bench::repo_root` in `main.rs` and
/// `bee-parity::repo_root` (rust-port-2 precedent) — robust to the process's
/// actual cwd at run time. Resolved HERE rather than threaded through
/// [`FixtureRequest`] so the generator's public request shape (and every
/// caller of it) is unchanged by this addition.
fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or(manifest_dir)
}

/// Seed the COMMAND REGISTRY BRIDGE into the fixture (rpl-1 obligation (B)).
///
/// `bee-core::registry::load_registry` resolves two store-root-relative
/// paths — `.bee/cache/command-registry.json` (the dump) and
/// `.bee/bin/lib/command-registry.mjs` (the staleness anchor it re-hashes),
/// per `crates/queen-bee/src/hooks/write_guard.rs:1254-1255`. Before this,
/// `generate` wrote NEITHER, so every `queen-bee` invocation against a
/// generated fixture failed to resolve a registry at all — which makes the
/// registry-derived refusal texts (`bee.mjs:7186-7188`, built from the
/// entry's `parameters.properties`) unreachable, and with them every
/// `--cmd-check` scenario that asserts one.
///
/// Both files are copied VERBATIM from this repo, which is the point: the
/// mjs leg imports the repo's own `command-registry.mjs` directly (it is
/// spawned as `node <repo>/.bee/bin/bee.mjs` with only its cwd moved), so
/// copying the same bytes is what makes the two legs resolve the SAME
/// registry. This is safe for the parity TREE diff because `.bee/cache` is
/// already a whole-path exclusion (`crates/bee-parity/src/differ.rs:28`) and
/// because the `.mjs` copy is byte-identical in every clone — the goal is
/// agreement between the legs, never diffing the registry itself.
///
/// REFUSES loudly (writes nothing, returns `Err`) when either source file is
/// missing: a fixture that silently lacks the bridge is exactly the failure
/// this exists to end.
fn write_command_registry(bee_dir: &Path) -> Result<(), String> {
    let root = repo_root();
    let sources = [
        (
            root.join(".bee").join("cache").join("command-registry.json"),
            bee_dir.join("cache").join("command-registry.json"),
        ),
        (
            root.join(".bee").join("bin").join("lib").join("command-registry.mjs"),
            bee_dir.join("bin").join("lib").join("command-registry.mjs"),
        ),
    ];
    for (src, dest) in &sources {
        if !src.exists() {
            return Err(format!(
                "command registry bridge source {} is missing — refusing to generate a fixture without it (queen-bee cannot resolve a registry against such a store, so every registry-derived parity scenario would be unprovable); regenerate with `node scripts/dump_command_registry.mjs`",
                src.display()
            ));
        }
        let parent = dest.parent().ok_or_else(|| format!("no parent dir for {}", dest.display()))?;
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
        let bytes = fs::read(src).map_err(|e| format!("read {}: {e}", src.display()))?;
        fs::write(dest, &bytes).map_err(|e| format!("write {}: {e}", dest.display()))?;
    }
    Ok(())
}

/// Deterministic filler text so each decisions.jsonl line lands in the same
/// ballpark (a few hundred bytes) as a real recorded decision, keeping the
/// padding loop's iteration count sane without pretending to be real content.
const FILLER_DECISION: &str = "Fixture decision line generated by queen-bench for the D5 host-real benchmark store. This text pads the record to a size representative of the live repo's decisions.jsonl so the bench never runs against a synthetic small-store fiction.";
const FILLER_RATIONALE: &str = "Rationale filler: this is bench fixture data, not a real recorded decision — generated solely to reach the pinned decisions.jsonl floor from CONTEXT.md D5 for spawn-inclusive p95 measurement.";

fn decision_line(i: usize) -> String {
    format!(
        "{{\"id\":\"fixture-decision-{i:06}\",\"type\":\"decide\",\"date\":\"2026-07-26T00:00:00.000Z\",\"decision\":\"{FILLER_DECISION}\",\"rationale\":\"{FILLER_RATIONALE}\",\"alternatives\":null,\"scope\":\"repo\",\"source\":\"fixture\",\"confidence\":0}}"
    )
}

const FILLER_DETAIL: &str = "Fixture backlog detail generated by queen-bench for the D5 host-real benchmark store — pads backlog.jsonl to the pinned floor for spawn-inclusive p95 measurement; not a real proposal.";
const FILLER_IMPACT: &str = "Fixture predicted_impact filler text, sized to keep each backlog.jsonl line representative of the live repo's line lengths.";

fn backlog_line(i: usize) -> String {
    format!(
        "{{\"ts\":\"2026-07-26T00:00:00.000Z\",\"type\":\"proposal\",\"title\":\"fixture backlog item {i:06}\",\"detail\":\"{FILLER_DETAIL}\",\"predicted_impact\":\"{FILLER_IMPACT}\",\"lane\":\"tiny\",\"source\":\"queen-bench fixture\"}}"
    )
}

fn reservation_entry(i: usize) -> String {
    format!(
        "{{\"agent\":\"fixture-agent-{i:06}\",\"cell\":\"fixture-cell-{i:06}\",\"path\":\"crates/fixture/path-{i:06}.rs\",\"ttl_seconds\":3600,\"reserved_at\":\"2026-07-26T00:00:00.000Z\",\"released_at\":\"2026-07-26T00:00:01.000Z\"}}"
    )
}

fn cell_json(i: usize) -> String {
    format!(
        "{{\"id\":\"fixture-cell-{i:05}\",\"feature\":\"queen-bench-fixture\",\"lane\":\"tiny\",\"behavior_change\":false,\"title\":\"fixture cell {i:05}\",\"action\":\"queen-bench D5 fixture filler cell — not real work\",\"must_haves\":{{\"truths\":[],\"prohibitions\":[]}},\"verify\":\"true\",\"read_first\":[],\"files\":[],\"deps\":[],\"status\":\"capped\",\"decisions\":[],\"trace\":{{\"worker\":null,\"outcome\":null,\"files_changed\":[],\"deviations\":[],\"friction\":null,\"capped_at\":null,\"behavior_change\":false,\"verification_evidence\":null,\"verify_output\":null,\"verify_passed\":null,\"claim_session\":null,\"claimed_at\":null}}}}"
    )
}

/// Append `make_line(i)` + `\n` until the file reaches `target_bytes`,
/// returning the exact byte count written.
fn write_jsonl_padded(
    path: &Path,
    target_bytes: u64,
    make_line: impl Fn(usize) -> String,
) -> Result<u64, String> {
    let mut file = fs::File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
    let mut written: u64 = 0;
    let mut i = 0usize;
    while written < target_bytes {
        let line = make_line(i);
        file.write_all(line.as_bytes())
            .and_then(|_| file.write_all(b"\n"))
            .map_err(|e| format!("write {}: {e}", path.display()))?;
        written += line.len() as u64 + 1;
        i += 1;
    }
    Ok(written)
}

/// Build `{"reservations":[...]}` incrementally until the serialized body
/// reaches `target_bytes`, then write it as one JSON document.
fn write_reservations_padded(path: &Path, target_bytes: u64) -> Result<u64, String> {
    let mut body = String::from("{\"reservations\":[");
    let mut first = true;
    let mut i = 0usize;
    const CLOSING_OVERHEAD: u64 = 2; // "]}"
    loop {
        let entry = reservation_entry(i);
        if !first {
            body.push(',');
        }
        body.push_str(&entry);
        first = false;
        i += 1;
        if body.len() as u64 + CLOSING_OVERHEAD >= target_bytes {
            break;
        }
    }
    body.push_str("]}");
    fs::write(path, &body).map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(body.len() as u64)
}

// ─── rust-port-19: git ancestry + review candidates + injected transcript ──

fn run_git(root: &Path, args: &[&str]) -> Result<(), String> {
    run_git_capture(root, args).map(|_| ())
}

fn run_git_capture(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| format!("spawn `git {}`: {e}", args.join(" ")))?;
    if !output.status.success() {
        return Err(format!(
            "`git {}` failed (exit {:?}): {}",
            args.join(" "),
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Initialize a real git repo at `root` (co-located with `.bee/` — see the
/// module doc comment) with `commits_count` linear commits, tagging the
/// first-quarter and final commit as genuine refs. Returns every commit sha,
/// oldest first. `--allow-empty` keeps each commit a single fast `git
/// commit` spawn with no working-tree diffing; identity/signing are set
/// locally so this never depends on (or pollutes) the caller's global git
/// config.
fn init_git_repo(root: &Path, commits_count: usize) -> Result<Vec<String>, String> {
    run_git(root, &["init", "-q"])?;
    run_git(root, &["config", "user.email", "queen-bench@fixture.invalid"])?;
    run_git(root, &["config", "user.name", "queen-bench fixture"])?;
    run_git(root, &["config", "commit.gpgsign", "false"])?;
    for i in 0..commits_count {
        run_git(
            root,
            &["commit", "--allow-empty", "-q", "-m", &format!("fixture commit {i:05}")],
        )?;
    }
    let log = run_git_capture(root, &["log", "--format=%H"])?;
    let mut shas: Vec<String> = log
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if shas.len() < commits_count {
        return Err(format!(
            "git log reported {} commit(s) after generating {commits_count} — ancestry fixture is short",
            shas.len()
        ));
    }
    shas.reverse(); // oldest first

    // Real refs (not just bare shas) so a `deriveCandidateStatus` reader that
    // cares about ref shape sees genuine tags, not only anonymous commit ids.
    run_git(root, &["tag", "fixture-early", &shas[shas.len() / 4]])?;
    run_git(root, &["tag", "fixture-late", shas.last().unwrap()])?;

    Ok(shas)
}

/// Freeze one `.bee/reviews/<id>.json` session (SPEC §8 shape, reviews.mjs)
/// covering the given features at `head`, with `decision.status` set to
/// `status` ("approved" or "pending"). `baseline` is schema-required but
/// unused by `deriveCandidateStatus` — a plain placeholder is honest.
fn write_review_session(
    reviews_dir: &Path,
    id: &str,
    status: &str,
    head: &str,
    features: &[&str],
) -> Result<(), String> {
    let included: String = features
        .iter()
        .map(|f| format!("{{\"type\":\"feature\",\"id\":\"{}\"}}", json_escape(f)))
        .collect::<Vec<_>>()
        .join(",");
    let body = format!(
        "{{\"id\":\"{id}\",\"requested_by\":\"queen-bench-fixture\",\"requested_at\":\"2026-07-26T00:00:00.000Z\",\"scope_description\":\"queen-bench D5 fixture review session — not a real review\",\"included\":[{included}],\"excluded\":[],\"baseline\":\"fixture-baseline-unused\",\"head\":\"{head}\",\"reviewer_manifest\":[],\"verification_preflight\":{{\"checked_at\":\"2026-07-26T00:00:00.000Z\",\"cells_checked\":[],\"passed\":true}},\"findings\":[],\"uat\":[],\"decision\":{{\"status\":\"{status}\",\"gate4\":null}},\"created_at\":\"2026-07-26T00:00:00.000Z\",\"updated_at\":\"2026-07-26T00:00:00.000Z\"}}"
    );
    let path = reviews_dir.join(format!("{id}.json"));
    fs::write(&path, body).map_err(|e| format!("write {}: {e}", path.display()))
}

fn review_candidate_line(feature: &str, head: &str, idx: usize) -> String {
    format!(
        "{{\"id\":\"fixture-candidate-{feature}-{idx:04}\",\"type\":\"candidate\",\"date\":\"2026-07-26T00:00:00.000Z\",\"feature\":\"{feature}\",\"head\":\"{head}\",\"mode\":\"standard\",\"baseline\":null,\"cells\":[]}}"
    )
}

/// Build `.bee/reviews/` (3 sessions) + `.bee/review-candidates.jsonl`
/// (`candidates_count` entries) so `deriveCandidateStatus` hits all four
/// `CANDIDATE_STATUSES` on a real read:
///   - `queen-bench-fixture-stale`      — approved session, valid ancestor
///     head, but commits landed after the session's frozen head -> "review
///     stale" via a real `rev-list --count` > 0.
///   - `queen-bench-fixture-reviewed`   — approved session pinned at the
///     actual git tip, candidate head an ancestor of it -> "reviewed" (0
///     commits since).
///   - `queen-bench-fixture-inreview`   — session still `pending` -> "in
///     review" (open session always outranks git ancestry).
///   - `queen-bench-fixture-unresolved` — approved session, but the
///     candidate's OWN head is not a real object -> "review stale" via a
///     real unresolvable `merge-base --is-ancestor`.
///   - `queen-bench-fixture-unreviewed` — everything else: no covering
///     session at all -> "unreviewed". Takes the remainder up to
///     `candidates_count`, which `check_floors` has already proven is >=
///     `REVIEW_CANDIDATES_FLOOR_COUNT` before this function ever runs.
fn write_review_state(bee_dir: &Path, commits: &[String], candidates_count: usize) -> Result<usize, String> {
    if commits.len() < 6 {
        // Unreachable via the public API: check_floors refuses any
        // git_commits_count below GIT_COMMITS_FLOOR_COUNT (50) before
        // init_git_repo ever runs. Guarded explicitly anyway so a future
        // floor change that shrinks GIT_COMMITS_FLOOR_COUNT below 6 fails
        // loud here instead of silently panicking on out-of-range indexing.
        return Err(format!(
            "write_review_state: need >= 6 commits to build genuine ancestry fixtures, got {}",
            commits.len()
        ));
    }
    let tip = commits.last().unwrap().as_str();
    let ancestor_of_tip = commits[commits.len() - 3].as_str();
    let stale_head = commits[commits.len() - 6].as_str();
    let ancestor_of_stale = commits[0].as_str();
    let bogus_head = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef";

    let reviews_dir = bee_dir.join("reviews");
    fs::create_dir_all(&reviews_dir).map_err(|e| format!("mkdir {}: {e}", reviews_dir.display()))?;

    write_review_session(
        &reviews_dir,
        "fixture-review-approved-stale",
        "approved",
        stale_head,
        &["queen-bench-fixture-stale", "queen-bench-fixture-unresolved"],
    )?;
    write_review_session(
        &reviews_dir,
        "fixture-review-approved-tip",
        "approved",
        tip,
        &["queen-bench-fixture-reviewed"],
    )?;
    write_review_session(
        &reviews_dir,
        "fixture-review-open",
        "pending",
        tip,
        &["queen-bench-fixture-inreview"],
    )?;

    const BUCKET: usize = 5;
    let candidates_path = bee_dir.join("review-candidates.jsonl");
    let mut file =
        fs::File::create(&candidates_path).map_err(|e| format!("create {}: {e}", candidates_path.display()))?;
    let mut write_line = |feature: &str, head: &str, idx: usize| -> Result<(), String> {
        let line = review_candidate_line(feature, head, idx);
        file.write_all(line.as_bytes())
            .and_then(|_| file.write_all(b"\n"))
            .map_err(|e| format!("write {}: {e}", candidates_path.display()))
    };

    let mut written = 0usize;
    for i in 0..BUCKET {
        write_line("queen-bench-fixture-stale", ancestor_of_stale, i)?;
        written += 1;
    }
    for i in 0..BUCKET {
        write_line("queen-bench-fixture-reviewed", ancestor_of_tip, i)?;
        written += 1;
    }
    for i in 0..BUCKET {
        write_line("queen-bench-fixture-inreview", tip, i)?;
        written += 1;
    }
    for i in 0..BUCKET {
        write_line("queen-bench-fixture-unresolved", bogus_head, i)?;
        written += 1;
    }
    for i in 0..candidates_count.saturating_sub(written) {
        write_line("queen-bench-fixture-unreviewed", tip, i)?;
        written += 1;
    }

    Ok(written)
}

/// Mirrors `encodeProjectDir` (`.bee/bin/lib/perf.mjs`): the absolute
/// project path with '/', '\' and '.' each replaced by '-'. Must stay
/// byte-for-byte identical to the mjs original — this is the exact
/// computation `resolveTranscript` repeats at read time to find the
/// directory a session's transcript lives in.
fn encode_project_dir(path_str: &str) -> String {
    path_str
        .chars()
        .map(|c| if c == '/' || c == '\\' || c == '.' { '-' } else { c })
        .collect()
}

const FILLER_TRANSCRIPT_NOTE: &str = "Fixture transcript filler line generated by queen-bench for the D5 recovery-block benchmark store — pads the crash-candidate transcript past readTranscriptTail's default 256 KB window so the bench genuinely exercises the tail-read cost, not a synthetic empty file.";

fn transcript_filler_line(i: usize) -> String {
    format!(
        "{{\"type\":\"filler\",\"timestamp\":\"2026-07-26T00:05:00.000Z\",\"seq\":{i},\"note\":\"{FILLER_TRANSCRIPT_NOTE}\"}}"
    )
}

/// Write one heartbeat-stale, non-clean-end session (`.bee/sessions/<id>.json`)
/// plus its transcript file under the INJECTED `transcript_root` (never the
/// host `$HOME/.claude/projects` — `write_config` above wires
/// `recovery.transcript_roots` to this exact directory), laid out at
/// `<transcript_root>/<encodeProjectDir(canonical_root)>/<session_id>.jsonl`
/// — the same encoded-project-dir shape `detectCrashCandidates`
/// (`recovery.mjs`) resolves through `resolveTranscript`. `last_heartbeat`
/// is pinned to 2020 (unambiguously stale under the 900s window regardless
/// of when this fixture is later read); every transcript event carries a
/// fixed 2026-07-26 timestamp strictly after the fixture's decisions.jsonl
/// filler dates, so `lastDurableSettlement`'s comparison always finds
/// genuine `transcript_activity` — no `system/stop_hook_summary` event is
/// ever emitted, so `hasCleanEndTrio` never matches and the candidate is
/// never excluded as a clean stop.
fn write_crash_candidate(
    canonical_root: &Path,
    bee_dir: &Path,
    transcript_root: &Path,
    transcript_tail_bytes: u64,
) -> Result<(u64, PathBuf, String), String> {
    let sessions_dir = bee_dir.join("sessions");
    fs::create_dir_all(&sessions_dir).map_err(|e| format!("mkdir {}: {e}", sessions_dir.display()))?;

    let session_id = "fixture-crash-session-000001".to_string();
    let session_body = format!(
        "{{\"id\":\"{session_id}\",\"started_at\":\"2020-01-01T00:00:00.000Z\",\"last_heartbeat\":\"2020-01-01T00:00:00.000Z\"}}"
    );
    let session_path = sessions_dir.join(format!("{session_id}.json"));
    fs::write(&session_path, session_body).map_err(|e| format!("write {}: {e}", session_path.display()))?;

    let encoded_dir_name = encode_project_dir(&canonical_root.display().to_string());
    let encoded_dir = transcript_root.join(&encoded_dir_name);
    fs::create_dir_all(&encoded_dir).map_err(|e| format!("mkdir {}: {e}", encoded_dir.display()))?;

    let transcript_path = encoded_dir.join(format!("{session_id}.jsonl"));
    let bytes = write_jsonl_padded(&transcript_path, transcript_tail_bytes, transcript_filler_line)?;

    Ok((bytes, transcript_path, session_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_each_pin_independently() {
        let base = FixtureRequest::at_floors(PathBuf::from("/unused-in-refusal-check"));
        let mut d = base.clone();
        d.decisions_bytes = DECISIONS_FLOOR_BYTES - 1;
        assert!(check_floors(&d).is_err());

        let mut r = base.clone();
        r.reservations_bytes = RESERVATIONS_FLOOR_BYTES - 1;
        assert!(check_floors(&r).is_err());

        let mut b = base.clone();
        b.backlog_bytes = BACKLOG_FLOOR_BYTES - 1;
        assert!(check_floors(&b).is_err());

        let mut c = base.clone();
        c.cells_count = CELLS_FLOOR_COUNT - 1;
        assert!(check_floors(&c).is_err());

        let mut rc = base.clone();
        rc.review_candidates_count = REVIEW_CANDIDATES_FLOOR_COUNT - 1;
        assert!(check_floors(&rc).is_err());

        let mut gc = base.clone();
        gc.git_commits_count = GIT_COMMITS_FLOOR_COUNT - 1;
        assert!(check_floors(&gc).is_err());

        let mut tt = base.clone();
        tt.transcript_tail_bytes = TRANSCRIPT_TAIL_FLOOR_BYTES - 1;
        assert!(check_floors(&tt).is_err());

        assert!(check_floors(&base).is_ok());
    }

    #[test]
    fn intent_seed_covers_the_shapes_the_floor_exists_for() {
        let keys = intent_seed_keys();
        assert!(
            keys.len() >= INTENT_KEYS_FLOOR_COUNT,
            "the fixed seed set must satisfy its own floor: {keys:?}"
        );
        assert!(keys.iter().any(|k| k == "default"), "the shared default key must be seeded");
        assert!(
            keys.iter().any(|k| k == "etc-passwd"),
            "the sanitized form of the traversal key must be seeded"
        );
        assert!(
            keys.iter().any(|k| k.chars().count() == 120),
            "a key at the .slice(0, 120) cap must be seeded: {keys:?}"
        );
        assert!(
            keys.iter().any(|k| !k.is_ascii()),
            "a non-ASCII filename must be seeded — sanitizeIntentKey can never produce one, which is why the store has to carry one anyway"
        );
    }

    #[test]
    fn seeded_anchor_bytes_are_a_valid_normalizable_anchor() {
        // The seeded record must PASS normalizeAnchor (a `request` that is a
        // non-blank string), or every scenario reading it would render "(no
        // intent anchor)" and prove nothing about the anchor path.
        let body = intent_anchor_json("default", 1);
        let parsed: serde_json::Value = serde_json::from_str(&body).expect("seeded anchor must be valid JSON");
        assert!(parsed["request"].as_str().is_some_and(|s| !s.trim().is_empty()));
        assert_eq!(parsed["key"], "default");
        assert!(body.ends_with("}\n"), "writeJsonAtomic appends exactly one trailing newline");
        assert!(body.contains("\n  \"schema_version\""), "two-space indent, JSON.stringify(obj, null, 2)");
    }

    #[test]
    fn encode_project_dir_mirrors_mjs() {
        // Byte-for-byte parity with perf.mjs's encodeProjectDir: '/', '\',
        // '.' each become '-'; nothing else is touched.
        assert_eq!(encode_project_dir("/a/b/c"), "-a-b-c");
        assert_eq!(encode_project_dir("/tmp/queen-bench.fixture/1"), "-tmp-queen-bench-fixture-1");
        assert_eq!(encode_project_dir(r"C:\a\b"), "C:-a-b");
    }
}
