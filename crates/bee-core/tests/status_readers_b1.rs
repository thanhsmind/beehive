//! status_readers_b1 — the single integration target for rust-port-14's
//! review-derivation port (CONTEXT.md D5/D7): `bee_core::reviews`'s
//! `list_candidates`, `list_reviews`, `derive_candidate_status` and
//! `build_review_block`, with both git questions answered IN PROCESS by gix.
//! Run via `cargo test --manifest-path crates/Cargo.toml -p bee-core --test
//! status_readers_b1` (this cell's own `verify`).
//!
//! # Oracle discipline
//!
//! Nothing here is checked against this author's reading of the mjs source.
//! Two real oracles run as node children over every fixture:
//!
//! - **fine-grained**: `tests/support/status_readers_b1_oracle.mjs` imports
//!   the REAL, FROZEN `.bee/bin/lib/reviews.mjs` and returns the per-
//!   candidate derivation for the whole pass, so a disagreement names the
//!   candidate rather than only shifting a count;
//! - **coarse**: a real `node .bee/bin/bee.mjs status --json` run with cwd
//!   at the fixture, whose `review` key IS `buildReviewBlock` (bee.mjs:408)
//!   executing as-is. Neither oracle reimplements anything.
//!
//! # Fixtures
//!
//! Every git-dependent fixture starts from the GROWN generator of
//! rust-port-19 (`queen-bench --generate`), which produces a git-initialized
//! root with real ancestry (50 commits, real shas, two tags) and 60 review
//! candidates engineered to hit all four derived statuses. It is invoked as
//! a fixed prebuilt artifact, never rebuilt or edited from here. Divergence
//! classes are then produced by transforming a COPY of that root with the
//! git CLI — a test-time subprocess, which is unrelated to the derivation
//! path under test being subprocess-free.
//!
//! **Non-triviality is asserted, not assumed** (panel W5): the first test
//! below proves the node oracle produces a NON-degraded block with all four
//! statuses populated on the primary fixture, so every parity assertion that
//! follows is comparing real answers rather than two empty blocks agreeing.
//!
//! Every path used below is a fresh `tempfile::tempdir()` — never this
//! repo's own live `.bee/` store.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;

use bee_core::reviews::{self, Coverage, GitMemo};

// ─── locating the repo's frozen mjs layer and the prebuilt generator ───────

fn ancestor_containing(rel: &Path) -> Option<PathBuf> {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..12 {
        if dir.join(rel).exists() {
            return Some(dir.join(rel));
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn bee_mjs() -> PathBuf {
    ancestor_containing(Path::new(".bee/bin/bee.mjs"))
        .expect("status_readers_b1: could not locate .bee/bin/bee.mjs walking ancestors")
}

fn oracle_script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/support/status_readers_b1_oracle.mjs")
}

/// The prebuilt `queen-bench` binary, used as a fixed fixture-generation
/// artifact (rust-port-19 owns `queen-bench/*`; this cell never rebuilds or
/// edits it). Release is preferred, debug accepted.
fn queen_bench_bin() -> PathBuf {
    let exe = if cfg!(windows) { "queen-bench.exe" } else { "queen-bench" };
    for profile in ["release", "debug"] {
        if let Some(found) = ancestor_containing(&PathBuf::from("crates").join("target").join(profile).join(exe)) {
            return found;
        }
    }
    panic!(
        "status_readers_b1: could not locate a prebuilt queen-bench binary walking ancestors from {} \
         — build the workspace once (`cargo build --manifest-path crates/Cargo.toml -p queen-bench`) if it is genuinely missing.",
        env!("CARGO_MANIFEST_DIR")
    );
}

// ─── oracles ───────────────────────────────────────────────────────────────

/// Fine-grained oracle: the real `reviews.mjs`, via the tracked file driver.
fn run_oracle(op: &str, root: &Path) -> Value {
    let output = Command::new("node")
        .arg(oracle_script())
        .arg(op)
        .arg(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to spawn node status_readers_b1_oracle driver — is `node` on PATH?");
    assert!(
        output.status.success(),
        "oracle op {op} exited non-zero — stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("oracle op {op}: could not parse stdout {stdout:?}: {e}"))
}

/// Coarse oracle: the real `buildReviewBlock`, reached by running the real
/// `bee.mjs status --json` with cwd at the fixture root.
fn node_review_block(root: &Path) -> Value {
    let output = Command::new("node")
        .arg(bee_mjs())
        .arg("status")
        .arg("--json")
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to spawn node bee.mjs status --json");
    assert!(
        output.status.success(),
        "bee.mjs status --json exited non-zero at {} — stderr: {}",
        root.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("bee.mjs status --json did not parse ({e}); stdout: {stdout:?}"));
    parsed
        .get("review")
        .cloned()
        .unwrap_or_else(|| panic!("bee.mjs status --json output has no `review` key at {}", root.display()))
}

/// The Rust side of the fine-grained oracle: the same pass shape the driver
/// runs (sessions fetched once, one memo shared across the whole loop).
fn rust_derive_all(root: &Path) -> Value {
    let candidates = reviews::list_candidates(root);
    let sessions = reviews::list_reviews(root);
    let mut memo = GitMemo::new();
    let mut out = Vec::new();
    for candidate in &candidates {
        match reviews::derive_candidate_status(root, candidate, &sessions, &mut memo) {
            Ok(v) => out.push(v),
            Err(_) => panic!("rust derivation threw on candidate {candidate}"),
        }
    }
    Value::Array(out)
}

// ─── fixtures ──────────────────────────────────────────────────────────────

/// A generated fixture plus the scratch space its divergence-class variants
/// live in. `store` is the pristine generated root.
struct Fixture {
    dir: tempfile::TempDir,
    store: PathBuf,
}

impl Fixture {
    fn scratch(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }
}

/// Generate the grown D5 fixture: a git-initialized root with real ancestry
/// and 60 review candidates spanning all four derived statuses.
fn generate_fixture() -> Fixture {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = dir.path().join("store");
    let output = Command::new(queen_bench_bin())
        .arg("--generate")
        .arg("--out")
        .arg(&store)
        .output()
        .expect("failed to spawn queen-bench --generate");
    assert!(
        output.status.success(),
        "queen-bench --generate failed — stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(store.join(".git").exists(), "generated fixture has no .git — expected a git-initialized root");
    Fixture { dir, store }
}

fn git(cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn git {args:?} in {}: {e}", cwd.display()))
}

fn git_ok(cwd: &Path, args: &[&str]) {
    let out = git(cwd, args);
    assert!(
        out.status.success(),
        "git {args:?} failed in {} — stderr: {}",
        cwd.display(),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create_dir_all");
    for entry in fs::read_dir(from).expect("read_dir").flatten() {
        let src = entry.path();
        let dst = to.join(entry.file_name());
        let meta = entry.metadata().expect("metadata");
        if meta.is_dir() {
            copy_tree(&src, &dst);
        } else if meta.is_file() {
            fs::copy(&src, &dst).expect("copy file");
        }
        // Symlinks are not produced by the generator or by git in these
        // fixtures; skipping them keeps this helper dependency-free.
    }
}

/// A full copy of the generated root (store + git repository) at `name`.
fn clone_store(fx: &Fixture, name: &str) -> PathBuf {
    let dest = fx.scratch(name);
    copy_tree(&fx.store, &dest);
    dest
}

/// Copy just the `.bee` store into an already-existing git root.
fn graft_store_into(fx: &Fixture, dest: &Path) {
    copy_tree(&fx.store.join(".bee"), &dest.join(".bee"));
}

// ─── shared assertions ─────────────────────────────────────────────────────

/// The single parity assertion every divergence-class test delegates to:
/// both oracles, both granularities, one named class. Keeping it here is
/// what stops eight near-identical test bodies from being copy-pasted; the
/// per-class `#[test]` wrappers exist so the class name appears in cargo's
/// output, as the cell requires.
fn assert_class_matches_mjs(class: &str, root: &Path) {
    let node_block = node_review_block(root);
    let rust_block = reviews::build_review_block(root);
    assert_eq!(
        rust_block,
        node_block,
        "divergence class `{class}`: the gix review block disagrees with the real bee.mjs buildReviewBlock at {}",
        root.display()
    );
    let node_derived = run_oracle("derive-all", root);
    let rust_derived = rust_derive_all(root);
    assert_eq!(
        rust_derived,
        node_derived,
        "divergence class `{class}`: per-candidate derivation disagrees with the real reviews.mjs at {}",
        root.display()
    );
}

fn counts_of(block: &Value) -> (u64, u64, u64, u64, u64) {
    let c = block.get("candidates").cloned().unwrap_or(Value::Null);
    let get = |k: &str| c.get(k).and_then(Value::as_u64).unwrap_or(0);
    (get("total"), get("unreviewed"), get("in_review"), get("reviewed"), get("stale"))
}

// ─── primary fixture: non-triviality first, then parity ────────────────────

#[test]
fn primary_git_fixture_is_non_trivial_and_non_degraded() {
    let fx = generate_fixture();
    let node_block = node_review_block(&fx.store);
    assert!(
        node_block.get("degraded").is_none(),
        "NON-TRIVIALITY: the node oracle degraded on the primary fixture — every parity assertion in this \
         file would then be comparing two empty blocks. Block was {node_block}"
    );
    let (total, unreviewed, in_review, reviewed, stale) = counts_of(&node_block);
    assert!(total >= 60, "NON-TRIVIALITY: expected >=60 candidates from the grown fixture, got {total}");
    assert!(unreviewed > 0, "NON-TRIVIALITY: no `unreviewed` candidates — block {node_block}");
    assert!(in_review > 0, "NON-TRIVIALITY: no `in review` candidates — block {node_block}");
    assert!(reviewed > 0, "NON-TRIVIALITY: no `reviewed` candidates — block {node_block}");
    assert!(stale > 0, "NON-TRIVIALITY: no `review stale` candidates — block {node_block}");
    assert_eq!(
        unreviewed + in_review + reviewed + stale,
        total,
        "every candidate must land in exactly one of the four statuses"
    );
}

#[test]
fn review_block_matches_mjs_on_primary_git_fixture() {
    let fx = generate_fixture();
    assert_class_matches_mjs("primary (git-initialized, real ancestry)", &fx.store);
}

#[test]
fn per_candidate_derivation_matches_real_reviews_mjs() {
    let fx = generate_fixture();
    let node_derived = run_oracle("derive-all", &fx.store);
    let rust_derived = rust_derive_all(&fx.store);
    let node_arr = node_derived.as_array().expect("oracle derive-all returned a non-array");
    assert_eq!(node_arr.len(), 60, "grown fixture should carry 60 candidates, got {}", node_arr.len());
    // Element-wise so a disagreement names the candidate.
    let rust_arr = rust_derived.as_array().expect("rust derive-all built a non-array");
    for (i, (r, n)) in rust_arr.iter().zip(node_arr.iter()).enumerate() {
        assert_eq!(r, n, "candidate #{i}: rust derived {r}, real reviews.mjs derived {n}");
    }
    assert_eq!(rust_derived, node_derived);
}

#[test]
fn list_candidates_matches_real_reviews_mjs() {
    let fx = generate_fixture();
    let rust = Value::Array(reviews::list_candidates(&fx.store));
    assert_eq!(rust, run_oracle("list-candidates", &fx.store));
}

#[test]
fn list_reviews_matches_real_reviews_mjs() {
    let fx = generate_fixture();
    let rust = Value::Array(reviews::list_reviews(&fx.store));
    assert_eq!(rust, run_oracle("list-reviews", &fx.store));
}

// ─── divergence classes, one named test each (panel W5) ────────────────────

#[test]
fn divergence_shallow_clone() {
    let fx = generate_fixture();
    let dest = fx.scratch("shallow");
    // `file://` forces the git transport; a plain local path would hardlink
    // the whole object store and silently ignore --depth.
    let url = format!("file://{}", fx.store.display());
    let out = git(
        fx.dir.path(),
        &["clone", "--depth", "3", "--quiet", &url, dest.to_str().expect("utf8 path")],
    );
    assert!(
        out.status.success(),
        "shallow clone setup failed — stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        dest.join(".git/shallow").exists(),
        "shallow clone setup did not produce .git/shallow — the class would be vacuous"
    );
    graft_store_into(&fx, &dest);

    // The generated candidates all point at commits that were truncated
    // away, so every question about them fails at resolution and never
    // reaches an ancestry decision — parity would hold vacuously. Add the
    // pair that actually CROSSES the graft: a candidate at the tip under an
    // approved session pinned at the shallow root. Deciding "is the tip an
    // ancestor of the root?" requires walking past the root, whose parents
    // are absent — the exact case where the git CLI answers definitely
    // (exit 1, applying the graft) and gix's merge-base cannot.
    let tip = rev_parse(&dest, "HEAD");
    let mid = rev_parse(&dest, "HEAD~1");
    let shallow_root = rev_parse(&dest, "HEAD~2");
    write_session(&dest, "b1-shallow-approved", &["b1-shallow"], &shallow_root, "approved");
    append_candidate(&dest, "b1-shallow-tip", "b1-shallow", &tip, "standard");
    // ...and the pair that reaches the OTHER question: an ancestry hit, so
    // the derivation goes on to `rev-list <ref>..HEAD --count` inside a
    // grafted history. That is the count the CLI reports with exit 0 and a
    // truncated walk, and the one gix must not quietly disagree with.
    write_session(&dest, "b1-shallow-count", &["b1-shallow-count"], &mid, "approved");
    append_candidate(&dest, "b1-shallow-counted", "b1-shallow-count", &shallow_root, "standard");

    assert_class_matches_mjs("shallow clone", &dest);
}

fn rev_parse(root: &Path, spec: &str) -> String {
    let out = git(root, &["rev-parse", spec]);
    assert!(
        out.status.success(),
        "rev-parse {spec} failed in {} — stderr: {}",
        root.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Write a review session in the exact shape `reviews.mjs` `createReview`
/// produces (SPEC §8), covering whole features.
fn write_session(root: &Path, id: &str, features: &[&str], head: &str, status: &str) {
    let included: Vec<Value> = features
        .iter()
        .map(|f| serde_json::json!({"type": "feature", "id": f}))
        .collect();
    let session = serde_json::json!({
        "id": id,
        "requested_by": "status-readers-b1",
        "requested_at": "2026-07-26T00:00:00.000Z",
        "scope_description": "status_readers_b1 fixture session — not a real review",
        "included": included,
        "excluded": [],
        "baseline": "b1-baseline-unused",
        "head": head,
        "reviewer_manifest": [],
        "verification_preflight": {"checked_at": "2026-07-26T00:00:00.000Z", "cells_checked": [], "passed": true},
        "findings": [],
        "uat": [],
        "decision": {"status": status, "gate4": null},
        "created_at": "2026-07-26T00:00:00.000Z",
        "updated_at": "2026-07-26T00:00:00.000Z"
    });
    let dir = root.join(".bee/reviews");
    fs::create_dir_all(&dir).expect("mkdir reviews");
    fs::write(dir.join(format!("{id}.json")), serde_json::to_string(&session).expect("serialize"))
        .expect("write session");
}

/// Append a candidate ledger entry in `addCandidate`'s exact shape.
fn append_candidate(root: &Path, id: &str, feature: &str, head: &str, mode: &str) {
    let entry = serde_json::json!({
        "id": id,
        "type": "candidate",
        "date": "2026-07-26T00:00:00.000Z",
        "feature": feature,
        "head": head,
        "mode": mode,
        "baseline": Value::Null,
        "cells": []
    });
    let path = root.join(".bee/review-candidates.jsonl");
    let mut f = fs::OpenOptions::new().create(true).append(true).open(&path).expect("open ledger");
    writeln!(f, "{}", serde_json::to_string(&entry).expect("serialize")).expect("append ledger");
}

#[test]
fn divergence_packed_refs_only() {
    let fx = generate_fixture();
    let dest = clone_store(&fx, "packed");
    git_ok(&dest, &["pack-refs", "--all", "--prune"]);
    assert!(
        dest.join(".git/packed-refs").exists(),
        "pack-refs setup did not produce .git/packed-refs — the class would be vacuous"
    );
    assert_class_matches_mjs("packed-refs only", &dest);
}

#[test]
fn divergence_detached_head() {
    let fx = generate_fixture();
    let dest = clone_store(&fx, "detached");
    git_ok(&dest, &["checkout", "--quiet", "--detach", "HEAD~5"]);
    let head = fs::read_to_string(dest.join(".git/HEAD")).expect("read HEAD");
    assert!(
        !head.starts_with("ref:"),
        "detached-HEAD setup left HEAD symbolic ({head:?}) — the class would be vacuous"
    );
    assert_class_matches_mjs("detached HEAD", &dest);
}

#[test]
fn divergence_linked_worktree_head() {
    let fx = generate_fixture();
    let main = clone_store(&fx, "wt-main");
    let linked = fx.scratch("wt-linked");
    git_ok(
        &main,
        &["worktree", "add", "--quiet", "--detach", linked.to_str().expect("utf8 path"), "HEAD~2"],
    );
    assert!(
        linked.join(".git").is_file(),
        "linked-worktree setup did not produce a `.git` FILE — the class would be vacuous"
    );
    graft_store_into(&fx, &linked);
    assert_class_matches_mjs("linked worktree HEAD", &linked);
}

#[test]
fn divergence_objects_info_alternates() {
    let fx = generate_fixture();
    let src = clone_store(&fx, "alt-src");
    let dest = fx.scratch("alt-dest");
    git_ok(
        fx.dir.path(),
        &[
            "clone",
            "--quiet",
            "--shared",
            src.to_str().expect("utf8 path"),
            dest.to_str().expect("utf8 path"),
        ],
    );
    assert!(
        dest.join(".git/objects/info/alternates").exists(),
        "--shared clone did not produce objects/info/alternates — the class would be vacuous"
    );
    graft_store_into(&fx, &dest);
    assert_class_matches_mjs("objects/info/alternates", &dest);
}

#[test]
fn divergence_commit_graph_present() {
    let fx = generate_fixture();
    let dest = clone_store(&fx, "cgraph");
    git_ok(&dest, &["commit-graph", "write", "--reachable"]);
    let graph = dest.join(".git/objects/info/commit-graph");
    let graph_dir = dest.join(".git/objects/info/commit-graphs");
    assert!(
        graph.exists() || graph_dir.exists(),
        "commit-graph setup wrote no graph file — the class would be vacuous"
    );
    assert_class_matches_mjs("commit-graph present", &dest);
}

#[test]
fn divergence_refs_replace_graft() {
    let fx = generate_fixture();
    let dest = clone_store(&fx, "replace");
    // Re-graft a mid-history commit into a root commit: every ancestry
    // question crossing it now has a different answer under `refs/replace`.
    let target = git(&dest, &["rev-parse", "HEAD~20"]);
    assert!(target.status.success(), "rev-parse HEAD~20 failed");
    let sha = String::from_utf8_lossy(&target.stdout).trim().to_string();
    git_ok(&dest, &["replace", "--graft", &sha]);
    assert!(
        dest.join(".git/refs/replace").exists() || !git(&dest, &["replace", "-l"]).stdout.is_empty(),
        "replace setup created no replacement ref — the class would be vacuous"
    );
    assert_class_matches_mjs("refs/replace graft", &dest);
}

#[test]
fn divergence_unknown_object_is_ancestor() {
    let fx = generate_fixture();
    let dest = clone_store(&fx, "unknown-object");
    // The grown fixture already ships a bucket whose candidate head is a
    // well-formed but absent sha; add an explicitly named one covered by an
    // APPROVED session so the unresolved leg is exercised on a second shape.
    append_candidate(
        &dest,
        "b1-unknown-object",
        "queen-bench-fixture-unresolved",
        "0123456789abcdef0123456789abcdef01234567",
        "standard",
    );
    assert_class_matches_mjs("unknown-object is-ancestor", &dest);
}

// ─── tri-state fidelity, stated as its own contract ────────────────────────

#[test]
fn tri_state_unknown_object_is_unresolved_never_not_covered() {
    let fx = generate_fixture();
    let sessions = reviews::list_reviews(&fx.store);
    let candidates = reviews::list_candidates(&fx.store);

    let pick = |feature: &str| -> Value {
        candidates
            .iter()
            .find(|c| c.get("feature").and_then(Value::as_str) == Some(feature))
            .unwrap_or_else(|| panic!("grown fixture has no candidate for feature {feature}"))
            .clone()
    };

    // (a) UNKNOWN object: an absent-but-well-formed head under an approved
    // covering session. git exits 128 here, not 1 — the answer is
    // `unresolved`, and mjs turns that into `review stale` + a note. If the
    // port had collapsed unknown into `covered: false` the session would
    // simply have been skipped and this would read `unreviewed`.
    let mut memo = GitMemo::new();
    let unknown = pick("queen-bench-fixture-unresolved");
    let derived = reviews::derive_candidate_status(&fx.store, &unknown, &sessions, &mut memo).expect("derivation");
    assert_eq!(
        derived.get("status").and_then(Value::as_str),
        Some("review stale"),
        "unknown object must degrade to `review stale`, never be treated as a definite non-ancestor; got {derived}"
    );
    assert_eq!(
        derived.get("note").and_then(Value::as_str),
        Some("range unresolvable"),
        "the unresolved leg must carry mjs's `range unresolvable` note; got {derived}"
    );

    // (b) DEFINITE no: a real, resolvable head that is not an ancestor of
    // the covering session's head. Same code path, exit 1, and the session
    // is skipped — which for this candidate leaves `in review`/`reviewed`,
    // never a stale note. Proving (a) and (b) differ IS the tri-state.
    let known = pick("queen-bench-fixture-inreview");
    let known_derived =
        reviews::derive_candidate_status(&fx.store, &known, &sessions, &mut memo).expect("derivation");
    assert_ne!(
        known_derived.get("note").and_then(Value::as_str),
        Some("range unresolvable"),
        "a resolvable head must never produce the unresolved note; got {known_derived}"
    );

    // (c) The three-way answer is distinguishable at the type level too: a
    // definite non-ancestor and an unresolvable one are different variants,
    // not two spellings of `false`.
    assert_ne!(Coverage::NotCovered, Coverage::Unresolved);

    // And the whole block still matches the real mjs on this fixture.
    assert_class_matches_mjs("tri-state fidelity", &fx.store);
}

// ─── degraded parity ───────────────────────────────────────────────────────

#[test]
fn degraded_parity_null_candidate_line() {
    let fx = generate_fixture();
    let dest = clone_store(&fx, "degraded");
    // A literal `null` line survives readJsonl and then makes
    // deriveCandidateStatus throw a TypeError on property access, which
    // buildReviewBlock's catch turns into the zeroed `degraded` block.
    let ledger = dest.join(".bee/review-candidates.jsonl");
    let mut f = fs::OpenOptions::new().append(true).open(&ledger).expect("open ledger");
    writeln!(f, "null").expect("append ledger");
    drop(f);

    let node_block = node_review_block(&dest);
    assert_eq!(
        node_block.get("degraded").and_then(Value::as_bool),
        Some(true),
        "the degraded fixture did not actually degrade the node oracle — it would prove nothing; got {node_block}"
    );
    let rust_block = reviews::build_review_block(&dest);
    assert_eq!(rust_block, node_block, "degraded shape must match mjs exactly, zeroed counts included");
    assert_eq!(counts_of(&rust_block), (0, 0, 0, 0, 0));
}

#[test]
fn empty_store_matches_mjs() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    fs::create_dir_all(root.join(".bee/reviews")).expect("mkdir");
    // No ledger, no sessions, no git repository at all: every read falls
    // open and the block is empty but NOT degraded.
    let rust_block = reviews::build_review_block(root);
    assert_eq!(counts_of(&rust_block), (0, 0, 0, 0, 0));
    assert!(rust_block.get("degraded").is_none(), "an empty store is empty, not degraded: {rust_block}");
    assert_eq!(Value::Array(reviews::list_candidates(root)), run_oracle("list-candidates", root));
    assert_eq!(Value::Array(reviews::list_reviews(root)), run_oracle("list-reviews", root));
    assert_eq!(rust_derive_all(root), run_oracle("derive-all", root));
}

// ─── high_risk_unreviewed ──────────────────────────────────────────────────

#[test]
fn high_risk_unreviewed_matches_mjs() {
    let fx = generate_fixture();
    let dest = clone_store(&fx, "high-risk");
    // The generator writes every candidate with `mode: "standard"`, so the
    // high-risk warning leg would never fire on the pristine fixture. Retag
    // the two buckets that derive to unreviewed/stale — the exact pair the
    // mjs condition counts — so the leg is genuinely exercised.
    let ledger = dest.join(".bee/review-candidates.jsonl");
    let text = fs::read_to_string(&ledger).expect("read ledger");
    let mut rewritten = String::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let mut v: Value = serde_json::from_str(line).expect("ledger line");
        let feature = v.get("feature").and_then(Value::as_str).unwrap_or("").to_string();
        if feature == "queen-bench-fixture-unreviewed" || feature == "queen-bench-fixture-stale" {
            if let Some(map) = v.as_object_mut() {
                map.insert("mode".to_string(), Value::String("high-risk".to_string()));
            }
        }
        rewritten.push_str(&serde_json::to_string(&v).expect("reserialize"));
        rewritten.push('\n');
    }
    fs::write(&ledger, rewritten).expect("write ledger");

    let node_block = node_review_block(&dest);
    let high_risk = node_block.get("high_risk_unreviewed").and_then(Value::as_u64).unwrap_or(0);
    assert!(
        high_risk > 0,
        "the high-risk fixture did not produce any high_risk_unreviewed — the leg would be untested; got {node_block}"
    );
    let rust_block = reviews::build_review_block(&dest);
    assert_eq!(rust_block, node_block, "high_risk_unreviewed must match mjs exactly");
}

// ─── zero subprocess: the D5 headline, proven by removing git ──────────────

/// Re-execs this same test binary with `git` unreachable and asserts the
/// derivation still produces the answer the node oracle computed WITH git
/// available. mjs cannot do this — with no git binary, `spawnSync` fails and
/// every candidate degrades to `unresolved`. Producing the identical
/// non-degraded block without a `git` process is the D5 claim made falsifiable.
#[test]
fn no_git_on_path_still_derives_via_gix() {
    const CHILD_ENV: &str = "BEE_B1_NO_GIT_CHILD";

    if let Ok(payload) = std::env::var(CHILD_ENV) {
        // ---- child role: git is unreachable here ----
        let (root, expected_file) = payload.split_once('\u{1}').expect("child payload");
        assert!(
            Command::new("git").arg("--version").output().is_err(),
            "child role: `git` is still reachable, so this test would prove nothing"
        );
        let expected: Value = serde_json::from_str(&fs::read_to_string(expected_file).expect("read expected"))
            .expect("parse expected");
        let rust_block = reviews::build_review_block(Path::new(root));
        assert_eq!(
            rust_block, expected,
            "with no git on PATH the gix derivation must still produce the same review block"
        );
        assert!(rust_block.get("degraded").is_none(), "must not degrade merely because git is absent");
        return;
    }

    // ---- parent role ----
    let fx = generate_fixture();
    let expected = node_review_block(&fx.store);
    assert!(expected.get("degraded").is_none(), "parent oracle degraded; nothing to compare against");
    let expected_file = fx.scratch("expected-review-block.json");
    fs::write(&expected_file, serde_json::to_string(&expected).expect("serialize")).expect("write expected");

    let payload = format!("{}\u{1}{}", fx.store.display(), expected_file.display());
    let output = Command::new(std::env::current_exe().expect("current_exe"))
        .args(["--exact", "no_git_on_path_still_derives_via_gix", "--nocapture", "--test-threads", "1"])
        .env(CHILD_ENV, payload)
        // An empty PATH is what makes `git` unreachable. The child binary
        // itself is launched by absolute path, so it is unaffected.
        .env("PATH", "")
        .output()
        .expect("re-exec test binary");
    assert!(
        output.status.success(),
        "no-git child run failed.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

// ─── panic probe (panel B4) ────────────────────────────────────────────────

/// A truncated object store is untrusted input, not a bug: the review block
/// must degrade or resolve, and the PROCESS must survive. If the gix path
/// unwrapped anything, or a panic escaped `build_review_block`, this test
/// would abort the whole test binary rather than fail.
#[test]
fn panic_probe_truncated_object_store_degrades_never_aborts() {
    let fx = generate_fixture();
    let dest = clone_store(&fx, "panic-probe");

    // Truncate every loose object to 3 bytes: the zlib streams are now
    // garbage, so resolving or walking any of them fails mid-read.
    let objects = dest.join(".git/objects");
    let mut truncated = 0usize;
    for fanout in fs::read_dir(&objects).expect("read objects").flatten() {
        let name = fanout.file_name();
        let name = name.to_string_lossy().to_string();
        if name.len() != 2 || !name.chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }
        for obj in fs::read_dir(fanout.path()).expect("read fanout").flatten() {
            // Loose objects are written read-only (mode 444); drop that
            // first or the truncation silently fails to happen.
            let path = obj.path();
            let mut perms = fs::metadata(&path).expect("object metadata").permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            perms.set_readonly(false);
            fs::set_permissions(&path, perms).expect("make object writable");
            fs::write(&path, b"\x78\x01\x00").expect("truncate object");
            truncated += 1;
        }
    }
    assert!(truncated > 0, "panic probe truncated nothing — the fixture would be vacuous");

    // The assertion that matters: this call RETURNS.
    let rust_block = reviews::build_review_block(&dest);

    let degraded = rust_block.get("degraded").and_then(Value::as_bool) == Some(true);
    if !degraded {
        // Not degraded means every candidate still got a status — and each
        // must be one of the four legal ones, never a half-built answer.
        let (total, unreviewed, in_review, reviewed, stale) = counts_of(&rust_block);
        assert_eq!(
            unreviewed + in_review + reviewed + stale,
            total,
            "a corrupt object store must still classify every candidate: {rust_block}"
        );
        assert!(total > 0, "corrupt-store block lost its candidates entirely: {rust_block}");
        // ...and it must still agree with what the real mjs makes of the
        // same wreckage.
        assert_eq!(
            rust_block,
            node_review_block(&dest),
            "corrupt object store: rust and the real bee.mjs disagree on the degraded outcome"
        );
    }
}

// ─── the D5 budget condition (approach.md:21) ──────────────────────────────

/// `approach.md:21` makes the gix choice conditional: "Slice 2 bench must
/// show gix query set on this repo < 2 ms; if not, an mtime-keyed cache
/// layer fronts it". This test is that condition, kept executable rather
/// than measured once and written down — if it ever goes red, the escape
/// hatch (an mtime-keyed cache in front of the query set, still
/// zero-subprocess) is what the doc says to reach for, NOT a reversion to
/// spawning git.
///
/// The subject is a self-contained local clone of this repository: the real
/// ~1000-commit ancestry and the committed review ledger, entirely inside a
/// tempdir, so the live `.bee/` store is never read or written.
#[test]
fn gix_query_set_stays_under_the_d5_budget_on_this_repo() {
    let repo_root = ancestor_containing(Path::new(".bee/review-candidates.jsonl"))
        .and_then(|p| p.parent().and_then(Path::parent).map(Path::to_path_buf))
        .expect("locate this repository");
    let dir = tempfile::tempdir().expect("tempdir");
    let clone = dir.path().join("self");
    let out = git(
        dir.path(),
        &[
            "clone",
            "--local",
            "--quiet",
            repo_root.to_str().expect("utf8 path"),
            clone.to_str().expect("utf8 path"),
        ],
    );
    assert!(
        out.status.success(),
        "self-clone failed — stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let candidates = reviews::list_candidates(&clone);
    let sessions = reviews::list_reviews(&clone);
    assert!(
        candidates.len() >= 20 && !sessions.is_empty(),
        "self-clone carried {} candidates / {} sessions — too few for the budget check to mean anything",
        candidates.len(),
        sessions.len()
    );

    let raw_pass = || {
        // In-memory memo only: the bare gix query set, unfronted.
        let mut memo = GitMemo::new();
        for candidate in &candidates {
            reviews::derive_candidate_status(&clone, candidate, &sessions, &mut memo).expect("derivation");
        }
        memo.git_queries()
    };
    let cached_pass = || {
        let mut memo = GitMemo::with_disk_cache();
        let mut derived = Vec::new();
        for candidate in &candidates {
            derived.push(
                reviews::derive_candidate_status(&clone, candidate, &sessions, &mut memo).expect("derivation"),
            );
        }
        memo.flush(&clone);
        (memo.git_queries(), Value::Array(derived))
    };

    let median_of = |mut samples: Vec<u128>| -> u128 {
        samples.sort_unstable();
        samples[samples.len() / 2]
    };

    // ── unfronted: the measurement approach.md:21's condition is about ──
    let queries = raw_pass();
    assert!(queries > 0, "the budget check asked no git questions — it would be vacuous");
    let raw_samples: Vec<u128> = (0..5)
        .map(|_| {
            let start = std::time::Instant::now();
            raw_pass();
            start.elapsed().as_micros()
        })
        .collect();
    let raw_median = median_of(raw_samples.clone());

    // ── fronted by the mtime-keyed cache: the hot read ──
    let (cold_queries, cold_derived) = cached_pass();
    assert!(cold_queries > 0, "the first cached pass should still have asked the real questions");
    assert!(
        clone.join(".bee/runtime/review-git-cache.json").exists(),
        "the cache layer wrote no cache file — the hot path would never be fast"
    );
    let (warm_queries, warm_derived) = cached_pass();
    // Not zero, and deliberately so: property 2 of the cache contract keeps
    // `unresolved` answers out of the cache, so those questions are re-asked
    // on every pass. They are also the cheap ones — an unresolvable spec
    // fails at resolution and never reaches a graph walk.
    assert!(
        warm_queries < cold_queries,
        "a warm pass asked {warm_queries} of {cold_queries} questions — the cache is not fronting the query set"
    );
    assert_eq!(
        warm_derived, cold_derived,
        "the cache changed an answer; a fronted read must be indistinguishable from a cold one"
    );

    let warm_samples: Vec<u128> = (0..5)
        .map(|_| {
            let start = std::time::Instant::now();
            cached_pass();
            start.elapsed().as_micros()
        })
        .collect();
    let warm_median = median_of(warm_samples.clone());

    println!(
        "D5 budget on this repo ({} candidates / {} sessions / {} distinct git questions):\n  \
         unfronted gix query set: median {}.{:03} ms  samples {:?} us\n  \
         fronted by the mtime-keyed cache (approach.md:21): median {}.{:03} ms  samples {:?} us",
        candidates.len(),
        sessions.len(),
        queries,
        raw_median / 1000,
        raw_median % 1000,
        raw_samples,
        warm_median / 1000,
        warm_median % 1000,
        warm_samples
    );

    // What is asserted, and what is only reported.
    //
    // approach.md:21's "< 2 ms" is a property of the SHIPPED binary. Pinning
    // a wall-clock number to an unoptimized test build would be measuring
    // the wrong artifact — the same mistake rust-port-18 was filed to fix
    // ("a test that runs the wrong artifact proves nothing") — and pinning
    // it tightly on a loaded CI box would make a design gate flaky. So the
    // number is PRINTED (the release-profile measurement is recorded in this
    // cell's report), and what is ASSERTED here is the mechanism plus a
    // generous catastrophic-regression ceiling:
    //
    //   - the cache is actually written and actually consulted (above),
    //   - fronting buys at least a 2x win over the unfronted query set,
    //   - nothing has regressed by an order of magnitude.
    assert!(
        warm_median * 2 < raw_median,
        "fronting the query set bought less than 2x ({warm_median} us vs {raw_median} us unfronted) — \
         the cache layer approach.md:21 calls for is not doing its job. Fix the cache; do NOT revert to spawning git."
    );
    let ceiling = if cfg!(debug_assertions) { 40_000 } else { 8_000 };
    assert!(
        warm_median < ceiling,
        "the fronted read took {warm_median} us (ceiling {ceiling} us for this profile) — a large regression \
         against approach.md:21's under-2 ms condition, which is measured on the release profile."
    );
}

/// The cache is only trustworthy if it invalidates. Moving HEAD changes
/// every `<ref>..HEAD` count, and a cache that kept serving the old numbers
/// would be the silent-wrong-count failure this cell exists to prevent.
#[test]
fn git_cache_invalidates_when_head_moves() {
    let fx = generate_fixture();
    let dest = clone_store(&fx, "cache-invalidation");

    let before = reviews::build_review_block(&dest);
    let cache = dest.join(".bee/runtime/review-git-cache.json");
    assert!(cache.exists(), "no cache was written — invalidation would be untestable");
    let fingerprint_before = fs::read_to_string(&cache).expect("read cache");
    assert_eq!(before, node_review_block(&dest), "baseline must match mjs before anything moves");

    // Move HEAD. Every `reviewed` candidate pinned at the old tip now has
    // commits after its session head, i.e. it becomes `review stale`.
    git_ok(&dest, &["commit", "--allow-empty", "--quiet", "-m", "cache invalidation probe"]);

    let after = reviews::build_review_block(&dest);
    let fingerprint_after = fs::read_to_string(&cache).expect("read cache");
    assert_ne!(
        fingerprint_before, fingerprint_after,
        "the cache file did not change after HEAD moved — it is not keyed on HEAD"
    );
    assert_ne!(
        before, after,
        "the block did not change after a new commit — the cache served stale counts"
    );
    assert_eq!(
        after,
        node_review_block(&dest),
        "after invalidation the fronted read must agree with the real bee.mjs again"
    );
}

/// Property 2 of the cache contract: an `unresolved` answer is never
/// persisted, so an object that arrives later is always re-asked rather than
/// frozen at the wrong answer.
#[test]
fn git_cache_never_persists_unresolved_answers() {
    let fx = generate_fixture();
    let dest = clone_store(&fx, "cache-unresolved");
    let block = reviews::build_review_block(&dest);
    assert!(
        block.get("candidates").and_then(|c| c.get("stale")).and_then(Value::as_u64).unwrap_or(0) > 0,
        "fixture produced no stale/unresolved candidates — the property would be untested"
    );
    let cached: Value =
        serde_json::from_str(&fs::read_to_string(dest.join(".bee/runtime/review-git-cache.json")).expect("read cache"))
            .expect("parse cache");

    let covered = cached.get("covered").and_then(Value::as_object).expect("covered map");
    assert!(!covered.is_empty(), "cache stored no ancestry answers at all");
    for (key, value) in covered {
        assert_ne!(value.as_str(), Some("unresolved"), "cache persisted an unresolved answer at {key}");
        // Property 1: every cached key names two object ids, never a ref name.
        let specs: Vec<&str> = key.trim_start_matches("covered ").split(' ').collect();
        for spec in specs {
            assert!(
                spec.len() == 40 && spec.bytes().all(|b| b.is_ascii_hexdigit()),
                "cache stored a non-object-id spec {spec:?} — a ref name can silently move"
            );
        }
    }
    // The bogus-head bucket asks a question that can never resolve; it must
    // be absent from the cache, so it is re-asked on every pass.
    assert!(
        !covered.keys().any(|k| k.contains("deadbeef")),
        "the unresolvable question was persisted: {covered:?}"
    );
}

// ─── memo contract (D2, cli-performance CONTEXT) ───────────────────────────

#[test]
fn git_memo_asks_each_pair_once_per_pass() {
    let fx = generate_fixture();
    let candidates = reviews::list_candidates(&fx.store);
    let sessions = reviews::list_reviews(&fx.store);
    assert!(candidates.len() >= 60, "fixture too small to demonstrate memoization");

    let mut memo = GitMemo::new();
    for candidate in &candidates {
        reviews::derive_candidate_status(&fx.store, candidate, &sessions, &mut memo).expect("derivation");
    }
    let first_pass = memo.git_queries();
    assert!(
        first_pass > 0,
        "no git question was asked at all — the memo assertion would be vacuous"
    );
    assert!(
        first_pass < candidates.len(),
        "the memo did not collapse repeated (head,ref)/(ref) questions: {first_pass} queries for {} candidates",
        candidates.len()
    );

    // Same memo, same pass: every question is already answered.
    for candidate in &candidates {
        reviews::derive_candidate_status(&fx.store, candidate, &sessions, &mut memo).expect("derivation");
    }
    assert_eq!(memo.git_queries(), first_pass, "a repeated question escaped the memo");

    // A fresh memo is genuinely fresh — the cache is pass-local, never
    // persisted or shared between passes.
    let mut fresh = GitMemo::new();
    for candidate in &candidates {
        reviews::derive_candidate_status(&fx.store, candidate, &sessions, &mut fresh).expect("derivation");
    }
    assert_eq!(fresh.git_queries(), first_pass, "a fresh memo must re-ask, never inherit a previous pass");
}
