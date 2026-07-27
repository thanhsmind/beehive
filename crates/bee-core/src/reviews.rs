//! reviews — the review-derivation half of the status read spine
//! (rust-port-14, CONTEXT.md D5/D7), ported from the FROZEN
//! `.bee/bin/lib/reviews.mjs` (`listCandidates` :362, `listReviews` :117,
//! `deriveCandidateStatus` :483) plus the aggregation loop that turns those
//! into `status --json`'s `review` block (`.bee/bin/bin/bee.mjs`
//! `buildReviewBlock` :408).
//!
//! # Why this module exists: D5's headline
//!
//! The mjs derivation answers exactly two git questions, and answers both by
//! **spawning a subprocess** (`reviews.mjs:401` `spawnSync('git', …)`):
//!
//! | mjs call site | question | shape |
//! |---|---|---|
//! | `reviews.mjs:431` | is `head` an ancestor-or-equal of `ref`? | `git merge-base --is-ancestor <head> <ref>` |
//! | `reviews.mjs:448` | how many commits are on HEAD since `ref`? | `git rev-list <ref>..HEAD --count` |
//!
//! On a real store those spawns dominate the `status` cost (~97 ms measured,
//! `approach.md:18`). This module answers both **in process** with `gix`, so
//! the whole status path spawns nothing. `approach.md:19` records why `gix`
//! and not `libgit2`: a C dependency would complicate the musl static builds
//! D8 requires. That is also why this crate's `Cargo.toml` pins gix with
//! `default-features = false` — no network feature is compiled in at all.
//!
//! # Fidelity contract
//!
//! Everything observable is the mjs behavior, bug-for-bug:
//!
//! - **Tri-state ancestry is preserved.** `reviews.mjs:428-435` reads the
//!   `merge-base --is-ancestor` exit code three ways — `0` covered, `1` not
//!   covered, **anything else** (unknown object, missing binary, shallow
//!   cut) *unresolved*. An unknown object must degrade the candidate to
//!   `review stale` + `range unresolvable`, and must NEVER collapse into
//!   `covered: false`. [`Coverage`] is that three-way answer, not a `bool`.
//! - **Fail-open, never a silent wrong count.** Every gix failure maps to
//!   `Unresolved`, exactly where mjs's non-0/1 exit does; and the whole
//!   block is wrapped so that anything worse degrades it to
//!   `{degraded: true}` with zeroed counts, exactly as `buildReviewBlock`'s
//!   `try/catch` does.
//! - **Per-pass memo semantics.** `reviews.mjs:414-419` memoizes on the
//!   string keys `covered <head> <ref>` and `since <ref>` in a pass-local
//!   `Map`. [`GitMemo`] reproduces those key strings (including the JS
//!   stringification of a non-string value), so the same pair is asked once
//!   per pass and never cached across passes.
//! - **JS coercion, reproduced not approximated.** `===`, truthiness and
//!   template-literal stringification all have JS semantics here — see
//!   [`js_strict_eq`], [`js_truthy`], [`js_to_string`]. A missing key models
//!   `undefined` (`Option::None`); an explicit JSON `null` is a distinct
//!   value, because `undefined === null` is `false` in JS.
//!
//! # Panic discipline (cell must-have, panel B4)
//!
//! Nothing in this module unwraps, expects, or index-slices anything read
//! out of a repository or a store file. Repository data is untrusted input:
//! a truncated pack, a corrupt loose object or a garbage ref is a *normal*
//! input here, not a bug, and it degrades. [`build_review_block`]
//! additionally runs its body inside `catch_unwind`, which is the faithful
//! Rust reading of mjs's `try { … } catch { degraded }`: in JS *any* throw
//! from the block body degrades it, and an unwinding panic out of a
//! third-party crate is the only Rust construct in that same class. That
//! makes the guarantee structural rather than a promise about gix's
//! internals. (This depends on the release profile unwinding — see
//! `crates/Cargo.toml`, where rust-port-18 removed `panic = "abort"` for
//! precisely this reason.)

use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::fsutil::read_jsonl;

/// `reviews.mjs:377` — the four derived statuses, in mjs order. Status is
/// NEVER stored on a candidate; it is derived at read time (R6/R10).
pub const CANDIDATE_STATUSES: [&str; 4] = ["unreviewed", "in review", "reviewed", "review stale"];

/// `reviews.mjs:49` `reviewsDir`.
pub fn reviews_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("reviews")
}

/// `reviews.mjs:57` `candidatesPath`.
pub fn candidates_path(root: &Path) -> PathBuf {
    root.join(".bee").join("review-candidates.jsonl")
}

// ─── JS value semantics ─────────────────────────────────────────────────────
// The mjs source operates on `JSON.parse` output with no schema: a candidate
// or session field can legitimately be missing, null, or the wrong type, and
// the derivation's behavior in each of those cases is observable. These three
// helpers are the JS operators the port needs, with `Option<&Value>` modeling
// "property access": `None` is `undefined` (key absent), `Some(Value::Null)`
// is an explicit `null`. Keeping them distinct matters — `undefined === null`
// is `false` in JS.

/// JS `===`. Objects and arrays compare by *reference* in JS, and two values
/// parsed out of a store file are never the same reference, so structurally
/// equal objects correctly compare unequal here.
fn js_strict_eq(a: Option<&Value>, b: Option<&Value>) -> bool {
    match (a, b) {
        (None, None) => true, // undefined === undefined
        (None, _) | (_, None) => false,
        (Some(x), Some(y)) => match (x, y) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(p), Value::Bool(q)) => p == q,
            // JS has one numeric type: `1` and `1.0` are the same value.
            (Value::Number(p), Value::Number(q)) => match (p.as_f64(), q.as_f64()) {
                (Some(pf), Some(qf)) => pf == qf,
                _ => false,
            },
            (Value::String(p), Value::String(q)) => p == q,
            _ => false,
        },
    }
}

/// JS truthiness. `undefined`, `null`, `false`, `0`, `NaN` and `""` are
/// falsy; every object and array (including `{}` and `[]`) is truthy.
fn js_truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().map(|f| f != 0.0 && !f.is_nan()).unwrap_or(false),
        Some(Value::String(s)) => !s.is_empty(),
        Some(_) => true,
    }
}

/// JS `String(v)` / template-literal interpolation. This is not cosmetic:
/// mjs builds its memo keys with `` `covered ${head} ${ref}` ``, and passes
/// the same values straight into `spawnSync`'s argv, where a missing `head`
/// arrives at git as the literal text `undefined` (verified against the real
/// `bee.mjs status --json`, which reports such a candidate as `review stale`
/// rather than degrading the block). Reproducing the coercion is what keeps
/// both the key space and the resolution behavior identical.
fn js_to_string(v: Option<&Value>) -> String {
    match v {
        None => "undefined".to_string(),
        Some(Value::Null) => "null".to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Number(n)) => js_number_to_string(n),
        Some(Value::String(s)) => s.clone(),
        // `Array.prototype.toString` joins with "," and renders null/undefined
        // elements as the empty string.
        Some(Value::Array(a)) => a
            .iter()
            .map(|e| match e {
                Value::Null => String::new(),
                other => js_to_string(Some(other)),
            })
            .collect::<Vec<_>>()
            .join(","),
        Some(Value::Object(_)) => "[object Object]".to_string(),
    }
}

/// `String(n)` for a JS number: an integral value never grows a `.0` tail.
fn js_number_to_string(n: &serde_json::Number) -> String {
    if let Some(i) = n.as_i64() {
        return i.to_string();
    }
    if let Some(u) = n.as_u64() {
        return u.to_string();
    }
    match n.as_f64() {
        Some(f) if f.is_finite() && f.fract() == 0.0 => format!("{f:.0}"),
        Some(f) => f.to_string(),
        None => n.to_string(),
    }
}

/// `String(a).localeCompare(String(b), 'en', { numeric: true })`
/// (`reviews.mjs:135`), narrowed to the ASCII ids bee actually writes: digit
/// runs compare numerically (so `s-2` sorts before `s-10`), letter runs
/// compare case-insensitively first with a case-sensitive tiebreak — which
/// is what the `en` collation does for these ids, and what a plain byte
/// compare would get wrong on both counts.
fn locale_numeric_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let (ac, bc): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let (mut i, mut j) = (0usize, 0usize);
    let mut case_tiebreak = Ordering::Equal;
    while i < ac.len() && j < bc.len() {
        let (ca, cb) = (ac[i], bc[j]);
        if ca.is_ascii_digit() && cb.is_ascii_digit() {
            let start_a = i;
            let start_b = j;
            while i < ac.len() && ac[i].is_ascii_digit() {
                i += 1;
            }
            while j < bc.len() && bc[j].is_ascii_digit() {
                j += 1;
            }
            let na: String = ac[start_a..i].iter().collect();
            let nb: String = bc[start_b..j].iter().collect();
            let ta = na.trim_start_matches('0');
            let tb = nb.trim_start_matches('0');
            // Longer digit run (leading zeros stripped) is the larger number.
            let ord = ta.len().cmp(&tb.len()).then_with(|| ta.cmp(tb));
            if ord != Ordering::Equal {
                return ord;
            }
            if case_tiebreak == Ordering::Equal {
                case_tiebreak = na.len().cmp(&nb.len());
            }
            continue;
        }
        let (la, lb) = (ca.to_ascii_lowercase(), cb.to_ascii_lowercase());
        if la != lb {
            return la.cmp(&lb);
        }
        if case_tiebreak == Ordering::Equal && ca != cb {
            // Collation prefers lowercase before uppercase at equal letters.
            case_tiebreak = cb.cmp(&ca);
        }
        i += 1;
        j += 1;
    }
    (ac.len() - i).cmp(&(bc.len() - j)).then(case_tiebreak)
}

// ─── stores ─────────────────────────────────────────────────────────────────

/// `reviews.mjs:362` `listCandidates` — fail-open by construction: a missing
/// ledger reads as empty and a corrupt/truncated line is skipped, because
/// `readJsonl` already does both. Entries are returned untyped: a candidate
/// is whatever was written, and every consumer below tolerates any shape.
pub fn list_candidates(root: &Path) -> Vec<Value> {
    read_jsonl(&candidates_path(root))
}

/// `reviews.mjs:117` `listReviews` — one session per `.json` file, fail-open
/// per file (a corrupt or non-object session is skipped rather than breaking
/// the whole listing), id-sorted with [`locale_numeric_cmp`].
pub fn list_reviews(root: &Path) -> Vec<Value> {
    let dir = reviews_dir(root);
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut sessions: Vec<Value> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => continue, // readJson's fail-open null -> skipped below
        };
        match serde_json::from_str::<Value>(&text) {
            Ok(Value::Object(map)) => sessions.push(Value::Object(map)),
            // null / array / primitive / unparseable: skipped, matching the
            // mjs guard `!session || typeof !== 'object' || Array.isArray`.
            _ => continue,
        }
    }
    sessions.sort_by(|a, b| locale_numeric_cmp(&js_to_string(a.get("id")), &js_to_string(b.get("id"))));
    sessions
}

// ─── the two git questions, answered in process ─────────────────────────────

/// The tri-state `merge-base --is-ancestor` answer (`reviews.mjs:433-435`).
/// Deliberately not a `bool` or an `Option<bool>` at the call sites: the
/// whole point of the cell is that `Unresolved` never silently becomes
/// `NotCovered`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Coverage {
    /// git exit 0 — `head` is an ancestor of, or equal to, `ref`.
    Covered,
    /// git exit 1 — a definite "no".
    NotCovered,
    /// git exit anything-else — unknown object, unreadable repo, shallow
    /// cut, missing binary. Never a "no".
    Unresolved,
}

/// The `rev-list <ref>..HEAD --count` answer (`reviews.mjs:450-455`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Since {
    Count(u64),
    Unresolved,
}

/// The pass-local memo of `reviews.mjs:414-419`, plus the opened repository.
///
/// Born once per derivation pass in [`build_review_block`], never persisted
/// and never TTL'd — exactly the mjs contract. Two things live here:
///
/// - the two answer caches, keyed by the *same* strings mjs uses, so the
///   same `(head, ref)` / `(ref)` question is asked once per pass;
/// - the opened [`gix::Repository`], discovered lazily on the first question
///   and reused for the rest of the pass. mjs has no equivalent because each
///   of its questions is a fresh `git` process; holding one open repository
///   is the in-process analog and is not observable in the output.
///
/// [`GitMemo::git_queries`] counts the questions that actually reached gix,
/// which is how the memo contract is asserted from tests — the same role
/// `reviews.mjs`'s `runGit` injection seam plays for the mjs tests.
pub struct GitMemo {
    repo: Option<Option<gix::Repository>>,
    covered: HashMap<String, Coverage>,
    since: HashMap<String, Since>,
    git_queries: usize,
    /// `Some` when this pass is fronted by the on-disk cache; the inner
    /// fingerprint is filled in on the first git question.
    disk: Option<Option<String>>,
    disk_dirty: bool,
}

impl Default for GitMemo {
    fn default() -> Self {
        Self::new()
    }
}

impl GitMemo {
    /// An in-memory-only pass: exactly the mjs memo, nothing more. Used by
    /// callers that want the raw derivation (and by the parity tests, so
    /// they measure the derivation rather than the cache).
    pub fn new() -> Self {
        GitMemo {
            repo: None,
            covered: HashMap::new(),
            since: HashMap::new(),
            git_queries: 0,
            disk: None,
            disk_dirty: false,
        }
    }

    /// A pass fronted by the on-disk cache (`approach.md:21`'s escape
    /// hatch). Answers already known for the current repository state are
    /// served without touching the object database; the rest are asked and
    /// then written back by [`GitMemo::flush`].
    pub fn with_disk_cache() -> Self {
        GitMemo {
            disk: Some(None),
            ..GitMemo::new()
        }
    }

    /// Questions that reached gix (i.e. missed both the memo and the disk
    /// cache). The mjs analog is counting `runGit` invocations.
    pub fn git_queries(&self) -> usize {
        self.git_queries
    }

    /// Load the on-disk cache, once per pass, on the first git question —
    /// the fingerprint needs the repository open, and nothing should open it
    /// for a pass that never asks anything.
    fn ensure_disk_loaded(&mut self, root: &Path) {
        if !matches!(self.disk, Some(None)) {
            return;
        }
        let fingerprint = match self.repo(root) {
            Some(repo) => cache_fingerprint(repo),
            // No repository: nothing to key a cache on, and every answer
            // will be Unresolved anyway.
            None => {
                self.disk = None;
                return;
            }
        };
        let stored: Value = crate::fsutil::read_json(&cache_path(root), Value::Null);
        if stored.get("fingerprint").and_then(Value::as_str) == Some(fingerprint.as_str()) {
            if let Some(map) = stored.get("covered").and_then(Value::as_object) {
                for (key, value) in map {
                    match value.as_str() {
                        Some("covered") => {
                            self.covered.insert(key.clone(), Coverage::Covered);
                        }
                        Some("not-covered") => {
                            self.covered.insert(key.clone(), Coverage::NotCovered);
                        }
                        // Anything else (including a hand-edited
                        // "unresolved") is ignored rather than trusted.
                        _ => {}
                    }
                }
            }
            if let Some(map) = stored.get("since").and_then(Value::as_object) {
                for (key, value) in map {
                    if let Some(n) = value.as_u64() {
                        self.since.insert(key.clone(), Since::Count(n));
                    }
                }
            }
        }
        self.disk = Some(Some(fingerprint));
    }

    /// Write the pass's definite answers back, best-effort. A failure here
    /// costs a slower next pass and nothing else, so it is swallowed.
    pub fn flush(&self, root: &Path) {
        let fingerprint = match &self.disk {
            Some(Some(fp)) if self.disk_dirty => fp,
            _ => return,
        };
        let mut covered = Map::new();
        for (key, value) in &self.covered {
            // Property 2: `Unresolved` is never persisted.
            let encoded = match value {
                Coverage::Covered => "covered",
                Coverage::NotCovered => "not-covered",
                Coverage::Unresolved => continue,
            };
            if cacheable_covered_key(key) {
                covered.insert(key.clone(), Value::String(encoded.to_string()));
            }
        }
        let mut since = Map::new();
        for (key, value) in &self.since {
            if let Since::Count(n) = value {
                if cacheable_since_key(key) {
                    since.insert(key.clone(), Value::from(*n));
                }
            }
        }
        let mut out = Map::new();
        out.insert("fingerprint".to_string(), Value::String(fingerprint.clone()));
        out.insert("covered".to_string(), Value::Object(covered));
        out.insert("since".to_string(), Value::Object(since));
        let _ = crate::fsutil::write_json_atomic(&cache_path(root), &Value::Object(out));
    }

    /// Discover the repository from `root` upward, once per pass.
    ///
    /// Upward discovery — not a plain open of `root/.git` — is what matches
    /// mjs: it runs git with `cwd: root` (`reviews.mjs:401`), and git
    /// discovers the repository from there. Discovery also transparently
    /// handles a `.git` *file*, which is what a linked worktree has.
    fn repo(&mut self, root: &Path) -> Option<&gix::Repository> {
        if self.repo.is_none() {
            self.repo = Some(discover_repo(root));
        }
        self.repo.as_ref().and_then(|r| r.as_ref())
    }
}

/// Discover the repository from `root` upward with object replacement
/// (`refs/replace/*`) applied, which is what the git CLI does by default.
///
/// The `core.useReplaceRefs=false` override is NOT a preference — it
/// compensates for an inverted condition in gix 0.86.0
/// (`src/open/repository.rs:559-565`, `replacement_objects_refs_prefix`):
/// the value read from `core.useReplaceRefs` is bound to a variable named
/// `is_disabled` and then used as `if is_disabled { return Ok(None) }`, so
/// the *default* (`true`, replacements on in git) makes gix load NO
/// replacement table, and only an explicit `false` makes it load one. Left
/// alone, a repository carrying a `git replace --graft` would get ancestry
/// answers that silently disagree with `git merge-base --is-ancestor` — the
/// exact "silently disagree" failure this cell's divergence-class fixtures
/// exist to catch, and `divergence_refs_replace_graft` in
/// `tests/status_readers_b1.rs` is what proved it before this override
/// existed.
///
/// That test is also the tripwire: if a future gix corrects the inversion,
/// this override starts meaning what it says, replacements switch off, and
/// the test goes red rather than the counts going quietly wrong. Revisit
/// here when it does.
fn discover_repo(root: &Path) -> Option<gix::Repository> {
    let open_options = gix::open::Options::default().config_overrides(["core.useReplaceRefs=false"]);
    let mut repo = gix::discover_opts(root, gix::discover::upwards::Options::default(), open_options).ok()?;
    // A derivation pass asks ~20 ancestry questions over one history, so the
    // same commit objects are decompressed again and again. Without this
    // cache the pass on this repository measured ~99 ms — the zlib work, not
    // the graph work, was the cost. It is the single biggest lever on
    // `approach.md:21`'s under-2 ms condition.
    repo.object_cache_size_if_unset(OBJECT_CACHE_BYTES);
    Some(repo)
}

/// Object cache budget for one derivation pass. Sized to comfortably hold a
/// repository's worth of commit objects (they are small; trees and blobs are
/// never touched on this path) without being a memory surprise in a CLI that
/// exits milliseconds later.
const OBJECT_CACHE_BYTES: usize = 8 * 1024 * 1024;

// ─── the mtime-keyed cache (approach.md:21's escape hatch) ─────────────────
//
// approach.md:21 made the gix choice conditional: "Slice 2 bench must show
// gix query set on this repo < 2 ms; if not, an mtime-keyed cache layer
// fronts it (still zero-subprocess on the hot read)". Measured on this
// repository (971 commits, 62 candidates, 6 sessions, release profile) the
// query set costs ~11-13 ms — the condition is NOT met, so this is that
// cache. Note what the escape hatch is NOT: reverting to `spawnSync git`.
// A cache miss still answers in process.
//
// # Why this cache can be trusted
//
// A cache over git answers is only safe if a stale hit is IMPOSSIBLE, not
// merely unlikely — a silently wrong review count is precisely the failure
// this cell exists to prevent. Three properties make it sound:
//
// 1. **Only object-id questions are cached.** Both questions are cached only
//    when their revspecs are full object ids (`is_object_id_spec`). Ancestry
//    between two *fixed commits* is immutable: objects never change, so a
//    definite answer stays correct forever. A spec that is a ref NAME could
//    silently start pointing elsewhere, so it is never cached. (bee writes
//    shas — `addCandidate`/`createReview` both record commit shas — so this
//    excludes nothing in practice.)
// 2. **Only definite answers are cached.** `Unresolved` is never stored, so
//    an object that was missing and later arrives (a fetch, an unshallowing)
//    is always re-asked. A cache can therefore only ever *keep* a correct
//    answer, never *freeze* an incorrect one.
// 3. **The fingerprint covers everything else that can move an answer**:
//    the resolved HEAD (the `<ref>..HEAD` range depends on it), the shallow
//    boundary, replace refs (loose AND packed), and alternates. Any change
//    there drops the whole cache rather than trying to patch it.
//
// The cache lives at `.bee/runtime/review-git-cache.json` — additive, never
// read by the frozen mjs layer, and self-invalidating, so a stale or deleted
// file costs a slower pass and nothing else. Every read and write is
// best-effort: no failure here is ever allowed to change an answer.

fn cache_path(root: &Path) -> PathBuf {
    root.join(".bee").join("runtime").join("review-git-cache.json")
}

/// A full object id (sha1 or sha256), the only spec shape whose answers are
/// safe to cache — see property 1 above.
fn is_object_id_spec(spec: &str) -> bool {
    (spec.len() == 40 || spec.len() == 64) && spec.bytes().all(|b| b.is_ascii_hexdigit())
}

/// `covered <a> <b>` is cacheable only when BOTH specs are object ids.
fn cacheable_covered_key(key: &str) -> bool {
    let rest = match key.strip_prefix("covered ") {
        Some(rest) => rest,
        None => return false,
    };
    let mut parts = rest.split(' ');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(a), Some(b), None) => is_object_id_spec(a) && is_object_id_spec(b),
        _ => false,
    }
}

/// `since <ref>` is cacheable only when the ref is an object id; the HEAD
/// half of the range is pinned by the fingerprint.
fn cacheable_since_key(key: &str) -> bool {
    key.strip_prefix("since ").map(is_object_id_spec).unwrap_or(false)
}

/// `<mtime_nanos>:<len>` for a path, or `-` when it does not exist. Both
/// components matter: mtime alone can miss a same-second rewrite.
fn stat_stamp(path: &Path) -> String {
    match std::fs::symlink_metadata(path) {
        Ok(meta) => {
            let stamp = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            format!("{stamp}:{}", meta.len())
        }
        Err(_) => "-".to_string(),
    }
}

/// The cache key. Everything that can change an answer without changing the
/// object ids in the question itself.
fn cache_fingerprint(repo: &gix::Repository) -> String {
    // The common dir is where shallow/packed-refs/objects live even for a
    // LINKED WORKTREE, whose own git dir holds only its private HEAD.
    let common = repo.common_dir();
    let head = resolve_commit(repo, "HEAD")
        .map(|id| id.to_string())
        .unwrap_or_else(|| "-".to_string());
    format!(
        "v1 head={head} shallow={} packed-refs={} replace={} alternates={}",
        stat_stamp(&repo.shallow_file()),
        stat_stamp(&common.join("packed-refs")),
        stat_stamp(&common.join("refs").join("replace")),
        stat_stamp(&common.join("objects").join("info").join("alternates")),
    )
}

/// Resolve a revspec to the commit it names, peeling tags, or `None` if it
/// cannot be resolved to a commit at all — which is every case where git
/// itself would exit non-0/1 (`fatal: Not a valid object name`, a spec that
/// names a tree or blob, an object missing from a truncated store).
///
/// Every step is a `?` on a `Result`/`Option`: no unwrap, no expect, no
/// indexing. `spec` is untrusted store content.
fn resolve_commit(repo: &gix::Repository, spec: &str) -> Option<gix::ObjectId> {
    let id = repo.rev_parse_single(spec).ok()?;
    let object = id.object().ok()?;
    let commit = object.peel_to_kind(gix::objs::Kind::Commit).ok()?;
    Some(commit.id)
}

/// `git merge-base --is-ancestor <a> <b>`, in process.
///
/// `a` is an ancestor-or-equal of `b` exactly when it is one of their merge
/// bases. No merge base at all (disjoint histories) is a definite "no",
/// which is what the CLI's exit 1 means there — so the empty result maps to
/// `NotCovered`, not to `Unresolved`.
///
/// A failure to compute merge bases at all maps to `Unresolved`. That is the
/// honest reading: the answer is unknown, and the tri-state exists precisely
/// so an unknown never has to masquerade as a "no". It cannot become a
/// silently wrong count.
///
/// An earlier revision of this function kept a second implementation — an
/// ancestry walk from `b` via `rev_walk`, which honors `shallow_commits()`
/// — on the theory that gix's merge-base would error inside a shallow clone
/// where the CLI still answers by applying the graft. A seeded-mutation
/// probe disproved it: with the fallback deleted, every fixture in
/// `status_readers_b1` still passed, including `divergence_shallow_clone`
/// after it was strengthened to exercise BOTH questions across the graft
/// (an ancestry decision that must walk past the shallow root, and a
/// `<ref>..HEAD` count inside the grafted history). gix answers both
/// correctly, so the fallback was unreachable — dead code dressed as
/// defensiveness — and was removed. Restore it only against a fixture that
/// actually reaches it.
///
/// Object replacement (`refs/replace`) is applied by the repository's object
/// database — see [`discover_repo`].
fn is_ancestor(repo: &gix::Repository, a_spec: &str, b_spec: &str) -> Coverage {
    let a = match resolve_commit(repo, a_spec) {
        Some(id) => id,
        None => return Coverage::Unresolved,
    };
    let b = match resolve_commit(repo, b_spec) {
        Some(id) => id,
        None => return Coverage::Unresolved,
    };
    if a == b {
        return Coverage::Covered;
    }
    match repo.merge_bases_many(a, &[b]) {
        Ok(bases) => {
            if bases.iter().any(|base| base.detach() == a) {
                Coverage::Covered
            } else {
                // Includes the empty case: disjoint histories have no merge
                // base, and `--is-ancestor` exits 1 for them.
                Coverage::NotCovered
            }
        }
        Err(_) => Coverage::Unresolved,
    }
}

/// `git rev-list <ref>..HEAD --count`, in process. `with_hidden` is exactly
/// the `^<ref>` exclusion the range syntax means.
fn count_commits_since(repo: &gix::Repository, ref_spec: &str) -> Since {
    let head = match resolve_commit(repo, "HEAD") {
        Some(id) => id,
        None => return Since::Unresolved,
    };
    let reference = match resolve_commit(repo, ref_spec) {
        Some(id) => id,
        None => return Since::Unresolved,
    };
    let walk = match repo.rev_walk([head]).with_hidden([reference]).all() {
        Ok(w) => w,
        Err(_) => return Since::Unresolved,
    };
    let mut count: u64 = 0;
    for item in walk {
        match item {
            Ok(_) => count = count.saturating_add(1),
            Err(_) => return Since::Unresolved,
        }
    }
    Since::Count(count)
}

/// `reviews.mjs:427` `headCoveredBy`. The `head === ref` short-circuit runs
/// on the RAW values *before* any git question — including the mjs quirk
/// that two absent `head` fields are `undefined === undefined`, i.e. covered.
fn head_covered_by(root: &Path, head: Option<&Value>, reference: Option<&Value>, memo: &mut GitMemo) -> Coverage {
    if js_strict_eq(head, reference) {
        return Coverage::Covered;
    }
    let head_str = js_to_string(head);
    let ref_str = js_to_string(reference);
    let key = format!("covered {head_str} {ref_str}");
    memo.ensure_disk_loaded(root);
    if let Some(hit) = memo.covered.get(&key) {
        return *hit;
    }
    memo.git_queries += 1;
    let value = match memo.repo(root) {
        Some(repo) => is_ancestor(repo, &head_str, &ref_str),
        None => Coverage::Unresolved, // no repository at all: git would exit 128
    };
    if value != Coverage::Unresolved {
        memo.disk_dirty = true;
    }
    memo.covered.insert(key, value);
    value
}

/// `reviews.mjs:445` `commitsSince`.
fn commits_since(root: &Path, reference: Option<&Value>, memo: &mut GitMemo) -> Since {
    let ref_str = js_to_string(reference);
    let key = format!("since {ref_str}");
    memo.ensure_disk_loaded(root);
    if let Some(hit) = memo.since.get(&key) {
        return *hit;
    }
    memo.git_queries += 1;
    let value = match memo.repo(root) {
        Some(repo) => count_commits_since(repo, &ref_str),
        None => Since::Unresolved,
    };
    if value != Since::Unresolved {
        memo.disk_dirty = true;
    }
    memo.since.insert(key, value);
    value
}

// ─── derivation ─────────────────────────────────────────────────────────────

/// The one condition under which mjs's `deriveCandidateStatus` *throws*
/// rather than returning a status, which `buildReviewBlock`'s `catch` then
/// turns into `degraded: true`. Reading a property off `null` is a
/// `TypeError` in JS; every other candidate shape (a number, a string, an
/// array) yields `undefined` properties and derives normally.
#[derive(Debug, PartialEq, Eq)]
pub struct DerivationThrew;

/// `reviews.mjs:379` `sessionCoversCandidate`. Coverage attaches to content
/// identity only: a `feature`-typed scope entry naming the candidate's
/// feature, or a `cell`-typed scope covering *every* one of the candidate's
/// cells.
fn session_covers_candidate(session: &Value, candidate: &Value) -> bool {
    let included = match session.get("included") {
        Some(Value::Array(a)) => a,
        _ => return false,
    };
    let feature = candidate.get("feature");
    let feature_key = Value::String("feature".to_string());
    if included.iter().any(|e| {
        js_truthy(Some(e)) && js_strict_eq(e.get("type"), Some(&feature_key)) && js_strict_eq(e.get("id"), feature)
    }) {
        return true;
    }
    // `.filter(Boolean)` — falsy cell ids are dropped before the check.
    let cells: Vec<&Value> = match candidate.get("cells") {
        Some(Value::Array(a)) => a.iter().filter(|v| js_truthy(Some(v))).collect(),
        _ => Vec::new(),
    };
    if cells.is_empty() {
        return false;
    }
    let cell_key = Value::String("cell".to_string());
    let included_cell_ids: Vec<Option<&Value>> = included
        .iter()
        .filter(|e| js_truthy(Some(e)) && js_strict_eq(e.get("type"), Some(&cell_key)))
        .map(|e| e.get("id"))
        .collect();
    // `Set.prototype.has` uses SameValueZero, which differs from `===` only
    // for NaN — a value JSON cannot express.
    cells
        .iter()
        .all(|id| included_cell_ids.iter().any(|got| js_strict_eq(*got, Some(id))))
}

/// `reviews.mjs:395` `isSessionOpen` — anything short of an `approved`
/// decision counts as open, `blocked` included (SPEC §5 / R8).
fn is_session_open(session: &Value) -> bool {
    let decision = session.get("decision");
    if !js_truthy(decision) {
        return true;
    }
    let approved = Value::String("approved".to_string());
    !js_strict_eq(decision.and_then(|d| d.get("status")), Some(&approved))
}

/// Build the `{status, session?, note?}` result. `session.id` being absent
/// makes mjs produce `{session: undefined}`, which `JSON.stringify` OMITS —
/// so the key is omitted here too rather than emitted as `null`.
fn status_with_session(status: &str, session: &Value, note: Option<&str>) -> Value {
    let mut out = Map::new();
    out.insert("status".to_string(), Value::String(status.to_string()));
    if let Some(id) = session.get("id") {
        out.insert("session".to_string(), id.clone());
    }
    if let Some(n) = note {
        out.insert("note".to_string(), Value::String(n.to_string()));
    }
    Value::Object(out)
}

/// `reviews.mjs:483` `deriveCandidateStatus` — derived at read time, never
/// stored.
///
/// Priority, unchanged from mjs: any covering session still open outranks
/// everything (an active review beats a stale older approval); otherwise the
/// first covering approved session whose head is an ancestor-or-equal of the
/// candidate's head decides `reviewed` vs `review stale`; a covering session
/// whose ancestry cannot be resolved degrades to `review stale` +
/// `range unresolvable` rather than silently reporting `reviewed`; no
/// covering session at all is `unreviewed`.
pub fn derive_candidate_status(
    root: &Path,
    candidate: &Value,
    sessions: &[Value],
    memo: &mut GitMemo,
) -> Result<Value, DerivationThrew> {
    if candidate.is_null() {
        return Err(DerivationThrew);
    }
    let covering: Vec<&Value> = sessions
        .iter()
        .filter(|s| session_covers_candidate(s, candidate))
        .collect();

    // `open[open.length - 1]` — the LAST open covering session wins.
    if let Some(session) = covering.iter().rfind(|s| is_session_open(s)) {
        return Ok(status_with_session("in review", session, None));
    }

    let mut unresolved_session: Option<&Value> = None;
    for session in covering.iter().filter(|s| !is_session_open(s)) {
        let coverage = head_covered_by(root, candidate.get("head"), session.get("head"), memo);
        match coverage {
            Coverage::Unresolved => {
                if unresolved_session.is_none() {
                    unresolved_session = Some(session);
                }
                continue;
            }
            // The candidate's work postdates this session's frozen head —
            // not this session's coverage. Keep looking.
            Coverage::NotCovered => continue,
            Coverage::Covered => {}
        }
        return Ok(match commits_since(root, session.get("head"), memo) {
            Since::Unresolved => status_with_session("review stale", session, Some("range unresolvable")),
            Since::Count(n) if n > 0 => status_with_session("review stale", session, None),
            Since::Count(_) => status_with_session("reviewed", session, None),
        });
    }
    if let Some(session) = unresolved_session {
        return Ok(status_with_session("review stale", session, Some("range unresolvable")));
    }
    Ok(Value::Object(
        [("status".to_string(), Value::String("unreviewed".to_string()))]
            .into_iter()
            .collect(),
    ))
}

/// The zeroed block `buildReviewBlock` falls back to, with `degraded: true`.
fn degraded_block() -> Value {
    let mut counts = Map::new();
    for key in ["total", "unreviewed", "in_review", "reviewed", "stale"] {
        counts.insert(key.to_string(), Value::from(0));
    }
    let mut out = Map::new();
    out.insert("candidates".to_string(), Value::Object(counts));
    out.insert("open_sessions".to_string(), Value::Array(Vec::new()));
    out.insert("high_risk_unreviewed".to_string(), Value::from(0));
    out.insert("degraded".to_string(), Value::Bool(true));
    Value::Object(out)
}

/// `bee.mjs:408` `buildReviewBlock` — the `review` key of `status --json`.
///
/// Fail-open in the same two layers as mjs: every read underneath already
/// degrades rather than throwing, and the whole block is still wrapped so
/// that a future change to that contract can never break the scout. The
/// `catch_unwind` is that outer wrapper (see the module docs on panic
/// discipline); a `DerivationThrew` is the inner one.
pub fn build_review_block(root: &Path) -> Value {
    match std::panic::catch_unwind(AssertUnwindSafe(|| build_review_block_inner(root))) {
        Ok(Ok(block)) => block,
        Ok(Err(DerivationThrew)) | Err(_) => degraded_block(),
    }
}

fn build_review_block_inner(root: &Path) -> Result<Value, DerivationThrew> {
    let candidates = list_candidates(root);
    let sessions = list_reviews(root);

    let mut total: u64 = 0;
    let mut unreviewed: u64 = 0;
    let mut in_review: u64 = 0;
    let mut reviewed: u64 = 0;
    let mut stale: u64 = 0;
    let mut high_risk_unreviewed: u64 = 0;

    // One pass-local memo for the whole loop (D2, cli-performance CONTEXT):
    // candidates sharing a covering session's (head,ref)/(ref) pair answer
    // the underlying git question once instead of once per candidate.
    // Fronted by the on-disk cache: the status path is the hot read D5 is
    // about, and `approach.md:21`'s condition is not met without it.
    let mut memo = GitMemo::with_disk_cache();
    let high_risk = Value::String("high-risk".to_string());

    for candidate in &candidates {
        total = total.saturating_add(1);
        let derived = derive_candidate_status(root, candidate, &sessions, &mut memo)?;
        let status = derived.get("status").and_then(Value::as_str).unwrap_or("");
        match status {
            "unreviewed" => unreviewed = unreviewed.saturating_add(1),
            "in review" => in_review = in_review.saturating_add(1),
            "reviewed" => reviewed = reviewed.saturating_add(1),
            "review stale" => stale = stale.saturating_add(1),
            _ => {}
        }
        if js_truthy(Some(candidate))
            && js_strict_eq(candidate.get("mode"), Some(&high_risk))
            && (status == "unreviewed" || status == "review stale")
        {
            high_risk_unreviewed = high_risk_unreviewed.saturating_add(1);
        }
    }

    let mut counts = Map::new();
    counts.insert("total".to_string(), Value::from(total));
    counts.insert("unreviewed".to_string(), Value::from(unreviewed));
    counts.insert("in_review".to_string(), Value::from(in_review));
    counts.insert("reviewed".to_string(), Value::from(reviewed));
    counts.insert("stale".to_string(), Value::from(stale));

    // `.map(s => s.id)` puts `undefined` in the array for an id-less session,
    // and `JSON.stringify` renders that as `null`.
    let open_sessions: Vec<Value> = sessions
        .iter()
        .filter(|s| is_session_open(s))
        .map(|s| s.get("id").cloned().unwrap_or(Value::Null))
        .collect();

    // Write-back happens only on the success path: a pass that threw has no
    // business persisting a partial answer set.
    memo.flush(root);

    let mut out = Map::new();
    out.insert("candidates".to_string(), Value::Object(counts));
    out.insert("open_sessions".to_string(), Value::Array(open_sessions));
    out.insert("high_risk_unreviewed".to_string(), Value::from(high_risk_unreviewed));
    Ok(Value::Object(out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn js_strict_eq_keeps_undefined_and_null_distinct() {
        assert!(js_strict_eq(None, None));
        assert!(!js_strict_eq(None, Some(&Value::Null)));
        assert!(js_strict_eq(Some(&Value::Null), Some(&Value::Null)));
        assert!(js_strict_eq(Some(&json!(1)), Some(&json!(1.0))));
        // Objects compare by reference in JS: structurally equal is not equal.
        assert!(!js_strict_eq(Some(&json!({"a": 1})), Some(&json!({"a": 1}))));
    }

    #[test]
    fn js_to_string_matches_template_literal_coercion() {
        assert_eq!(js_to_string(None), "undefined");
        assert_eq!(js_to_string(Some(&Value::Null)), "null");
        assert_eq!(js_to_string(Some(&json!(12))), "12");
        assert_eq!(js_to_string(Some(&json!([1, null, "a"]))), "1,,a");
        assert_eq!(js_to_string(Some(&json!({}))), "[object Object]");
    }

    #[test]
    fn locale_numeric_cmp_orders_digit_runs_numerically() {
        assert_eq!(locale_numeric_cmp("s-2", "s-10"), std::cmp::Ordering::Less);
        assert_eq!(locale_numeric_cmp("a", "a"), std::cmp::Ordering::Equal);
        assert_eq!(locale_numeric_cmp("a", "B"), std::cmp::Ordering::Less);
    }
}
