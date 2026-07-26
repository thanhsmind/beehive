//! writeguard_bash — D7b conformance corpus for the write-guard BASH path
//! (rust-port-11): proves `queen-bee hook write-guard` matches the REAL
//! `.bee/bin/hooks/bee-write-guard.mjs` for Bash-command analysis —
//! extract-targets (redirects, quoted-segment merges, sed -i, bare rm,
//! containment), git-bash incl. blanket staging (GIT SPAWN PARITY: fixtures
//! that resolve real git state run in GIT-INITIALIZED temp roots; plain
//! seeded roots have no .git and the git path never engages), the
//! internals-reach guard, and CLI-shape validation (valid + invalid) against
//! the rust-port-8 registry bridge. STALE-REGISTRY SEMANTICS are two
//! separate fixtures, never one symmetric diff (advisor note 2): the
//! rust-side stale-cache fixture is an ACCEPTED rust-only allow window
//! (coverage-gap line asserted, rust-side only); the node-side
//! import-failure fixture is node's own, separate skip proof.
//!
//! RIG DISCIPLINE (inherited from rust-port-7's hook_conformance rig, via
//! writeguard_core):
//! (i) SEEDING — every node-oracle run happens inside a fresh temp root
//!     seeded with `.bee/bin/lib/`, `.bee/bin/hooks/`, `.bee/onboarding.json`,
//!     an enabling `config.json`, and `.bee/cache/command-registry.json`;
//!     the oracle executes the SEEDED copy of `bee-write-guard.mjs`,
//!     sha256-verified against the repo source.
//! (ii) NON-TRIVIALITY BOTH WAYS — every deny fixture asserts the node
//!      oracle exited 2 BEFORE diffing, and is paired with an allow twin in
//!      the same shape with exactly one field flipped. Neither stale-
//!      registry fixture may be satisfied by a benign command on an intact
//!      root — both assert the deny precondition on an intact root first.
//! (iii) NEGATIVE CONTROL — an unseeded root must be DETECTED as invalid by
//!       the rig's own verifier.
//! (iv) Descriptive per-class fixture names, listed by cargo test output.
//!
//! Every temp root comes from `tempfile::tempdir()` — never the live `.bee/`.
//! The cell verify prefixes `node scripts/dump_command_registry.mjs` so
//! every temp root can seed a fresh registry cache.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Repo / binary resolution
// ---------------------------------------------------------------------------

fn repo_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..12 {
        if dir.join(".bee/bin/hooks/bee-write-guard.mjs").exists() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!(
        "could not locate .bee/bin/hooks/bee-write-guard.mjs walking ancestors from {}",
        env!("CARGO_MANIFEST_DIR")
    );
}

fn queen_bee_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_queen-bee"))
}

// ---------------------------------------------------------------------------
// Seeding (rig discipline i)
// ---------------------------------------------------------------------------

fn copy_dir_all(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap_or_else(|e| panic!("mkdir {dst:?}: {e}"));
    for entry in fs::read_dir(src).unwrap_or_else(|e| panic!("read_dir {src:?}: {e}")) {
        let entry = entry.expect("dir entry");
        let file_type = entry.file_type().expect("file type");
        let target = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target)
                .unwrap_or_else(|e| panic!("copy {:?} -> {target:?}: {e}", entry.path()));
        }
    }
}

fn sha256_hex(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    hasher.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// The oracle surface for the write-guard Bash path: every module the deny
/// decisions flow through, including the CLI-shape check's own imports
/// (validate-args.mjs, command-registry.mjs) and the registry cache the
/// rust side reads. A doctored seeded copy of any of these is a rig failure.
const SHA_CHECKED: &[&str] = &[
    "bin/hooks/adapter.mjs",
    "bin/hooks/bee-write-guard.mjs",
    "bin/hooks/tokenize-command.mjs",
    "bin/lib/guards.mjs",
    "bin/lib/state.mjs",
    "bin/lib/reservations.mjs",
    "bin/lib/lease-store.mjs",
    "bin/lib/worktree-holds.mjs",
    "bin/lib/workspace-store.mjs",
    "bin/lib/claims.mjs",
    "bin/lib/fsutil.mjs",
    "bin/lib/validate-args.mjs",
    "bin/lib/command-registry.mjs",
    "cache/command-registry.json",
];

struct SeededRoot {
    _dir: TempDir,
    root: PathBuf,
}

fn seed_root_with_config(config: Value) -> SeededRoot {
    let repo = repo_root();
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_path_buf();

    copy_dir_all(&repo.join(".bee/bin/lib"), &root.join(".bee/bin/lib"));
    copy_dir_all(&repo.join(".bee/bin/hooks"), &root.join(".bee/bin/hooks"));
    fs::copy(repo.join(".bee/onboarding.json"), root.join(".bee/onboarding.json")).expect("copy onboarding.json");
    fs::create_dir_all(root.join(".bee/cache")).expect("mkdir .bee/cache");
    fs::copy(
        repo.join(".bee/cache/command-registry.json"),
        root.join(".bee/cache/command-registry.json"),
    )
    .expect("copy command-registry.json — run `node scripts/dump_command_registry.mjs` first (the cell verify's own prefix)");
    fs::write(
        root.join(".bee/config.json"),
        serde_json::to_string_pretty(&config).expect("serialize config.json"),
    )
    .expect("write config.json");

    for rel in SHA_CHECKED {
        let src = repo.join(".bee").join(rel);
        let dst = root.join(".bee").join(rel);
        assert_eq!(
            sha256_hex(&src),
            sha256_hex(&dst),
            "seeded {rel} diverged from repo source — rig failure (doctored oracle copy)"
        );
    }

    SeededRoot { _dir: dir, root }
}

fn enabling_config() -> Value {
    json!({ "hooks": { "write-guard": true } })
}

/// Two independently seeded, identically configured + identically prepared
/// roots — node runs against one, the rust port against the other, so log
/// appends (and git index state) from either side can never leak across.
fn seed_pair_with(config: Value, setup: impl Fn(&Path)) -> (SeededRoot, SeededRoot) {
    let node = seed_root_with_config(config.clone());
    let rust = seed_root_with_config(config);
    setup(&node.root);
    setup(&rust.root);
    (node, rust)
}

fn write_json_file(root: &Path, rel: &str, value: &Value) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
}

fn write_state(root: &Path, phase: &str, execution_approved: bool) {
    write_json_file(
        root,
        ".bee/state.json",
        &json!({
            "schema_version": "1.0",
            "phase": phase,
            "feature": "fixture-feature",
            "mode": "standard",
            "approved_gates": { "context": true, "shape": true, "execution": execution_approved, "review": false },
        }),
    );
}

/// Rig discipline (iii): the verifier itself must DETECT an invalid/unseeded
/// root, not rubber-stamp it.
fn is_seeded_valid(root: &Path) -> bool {
    let repo = repo_root();
    for rel in SHA_CHECKED {
        let dst = root.join(".bee").join(rel);
        if !dst.exists() {
            return false;
        }
        let src = repo.join(".bee").join(rel);
        if sha256_hex(&src) != sha256_hex(&dst) {
            return false;
        }
    }
    root.join(".bee/onboarding.json").exists() && root.join(".bee/config.json").exists()
}

// ---------------------------------------------------------------------------
// Git fixture support (GIT SPAWN PARITY: git-bash fixtures that resolve real
// git state need GIT-INITIALIZED roots — a plain seeded root has no .git and
// the git resolution path never engages)
// ---------------------------------------------------------------------------

fn git_in(root: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn git {args:?}: {e}"));
    assert!(
        out.status.success(),
        "git {args:?} failed in {root:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn git_init_root(root: &Path) {
    git_in(root, &["init", "-q"]);
}

fn stage_file(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, content).unwrap();
    git_in(root, &["add", rel]);
}

// ---------------------------------------------------------------------------
// Running both runtimes
// ---------------------------------------------------------------------------

struct RunResult {
    status: i32,
    stdout: String,
    stderr: String,
}

fn run(mut cmd: Command, stdin: &str) -> RunResult {
    // The swarm environment exports BEE_AGENT_NAME for reservation
    // ownership; inferAgentName's env fallback would silently absorb it and
    // make fixtures environment-dependent — strip it from both runtimes.
    cmd.env_remove("BEE_AGENT_NAME");
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().unwrap_or_else(|e| panic!("spawn failed: {e}"));
    child
        .stdin
        .take()
        .expect("child stdin")
        .write_all(stdin.as_bytes())
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait for child");
    RunResult {
        status: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// Runs the SEEDED copy of bee-write-guard.mjs living inside `root` — the
/// real oracle per rig discipline (i).
fn run_node(root: &Path, stdin: &str) -> RunResult {
    let mut cmd = Command::new("node");
    cmd.arg(root.join(".bee/bin/hooks/bee-write-guard.mjs"));
    cmd.current_dir(root);
    run(cmd, stdin)
}

/// Runs the LIVE repo hook file directly — used only by the negative-control
/// fixture, which has nothing to seed by design.
fn run_live_node(cwd: &Path, stdin: &str) -> RunResult {
    let repo = repo_root();
    let mut cmd = Command::new("node");
    cmd.arg(repo.join(".bee/bin/hooks/bee-write-guard.mjs"));
    cmd.current_dir(cwd);
    run(cmd, stdin)
}

fn run_rust(root: &Path, stdin: &str) -> RunResult {
    let mut cmd = Command::new(queen_bee_bin());
    cmd.arg("hook").arg("write-guard");
    cmd.current_dir(root);
    run(cmd, stdin)
}

fn bash_payload(root: &Path, command: &str) -> String {
    json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": { "command": command },
        "cwd": root.to_string_lossy(),
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// Differ
// ---------------------------------------------------------------------------

/// Node-verdict-non-trivial (rig discipline ii): the node oracle must have
/// exited 2 on its own before any diffing, the deny reason must carry the
/// expected class marker, and the rust stderr must be byte-identical to the
/// node reason.
fn assert_deny_conformant(label: &str, node: &RunResult, rust: &RunResult, expected_marker: &str) {
    assert_eq!(node.status, 2, "{label}: node oracle did NOT deny (stderr={:?})", node.stderr);
    assert_eq!(rust.status, 2, "{label}: rust did not deny (stderr={:?})", rust.stderr);
    let node_reason = node.stderr.rsplit('\n').next().unwrap_or("");
    assert!(
        node_reason.contains(expected_marker),
        "{label}: node deny reason missing expected marker {expected_marker:?} — got {node_reason:?}"
    );
    assert_eq!(
        node_reason, rust.stderr,
        "{label}: deny reason diverged between node and rust"
    );
    assert!(node.stdout.trim().is_empty(), "{label}: a deny must not write stdout (node)");
    assert!(rust.stdout.trim().is_empty(), "{label}: a deny must not write stdout (rust)");
}

/// Allow-with-twin-deny pairing support: a genuine, silent allow on BOTH
/// runtimes (exit 0, matching stdout, empty rust stderr).
fn assert_allow_conformant(label: &str, node: &RunResult, rust: &RunResult) {
    assert_eq!(node.status, 0, "{label}: node oracle denied unexpectedly (stderr={:?})", node.stderr);
    assert_eq!(rust.status, 0, "{label}: rust denied unexpectedly (stderr={:?})", rust.stderr);
    assert_eq!(node.stdout.trim(), rust.stdout.trim(), "{label}: stdout diverged");
    assert!(rust.stderr.is_empty(), "{label}: rust allow must be silent on stderr — got {:?}", rust.stderr);
}

fn read_hooks_log(root: &Path) -> Vec<Value> {
    let Ok(text) = fs::read_to_string(root.join(".bee/logs/hooks.jsonl")) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("bad jsonl line {l:?}: {e}")))
        .collect()
}

// ---------------------------------------------------------------------------
// Fixture class: rig self-check + negative control
// ---------------------------------------------------------------------------

#[test]
fn rig_self_check_seeded_files_match_repo_sha256_and_registry_cache_is_fresh() {
    // The registry bridge itself must be fresh against the live source —
    // else every CLI-shape fixture would silently test yesterday's registry
    // (the exact class the stale fixture exists to keep visible).
    let repo = repo_root();
    let registry = bee_core::registry::load_registry(
        &repo.join(".bee/cache/command-registry.json"),
        &repo.join(".bee/bin/lib/command-registry.mjs"),
    )
    .expect("read command-registry.json — run `node scripts/dump_command_registry.mjs` first (the cell verify's own prefix)");
    assert!(
        registry.is_fresh(),
        "repo command-registry.json is STALE — run `node scripts/dump_command_registry.mjs` (the cell verify's own prefix)"
    );

    let seeded = seed_root_with_config(enabling_config());
    assert!(is_seeded_valid(&seeded.root), "a freshly seeded root must verify as valid");
}

#[test]
fn negative_control_unseeded_root_detected_as_invalid_and_both_runtimes_silent() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert!(!is_seeded_valid(dir.path()), "an unseeded root must never verify as a valid rig setup");

    let stdin = bash_payload(dir.path(), "echo hi > src/out.txt");
    let node = run_live_node(dir.path(), &stdin);
    let rust = run_rust(dir.path(), &stdin);
    assert_eq!(node.status, 0, "no discoverable root must be exit 0 (fail-open) — stderr={:?}", node.stderr);
    assert_eq!(rust.status, 0);
    assert!(node.stdout.trim().is_empty() && rust.stdout.trim().is_empty());
    assert!(!dir.path().join(".bee").exists(), "no root discoverable => no .bee dir may be created");
}

// ---------------------------------------------------------------------------
// Fixture class: extract-targets — redirects, fd-duplication, quoted-segment
// merge, sed -i, bare rm broad write, and Bash containment
// ---------------------------------------------------------------------------

#[test]
fn extract_targets_redirect_write_denies_at_idle_and_docs_redirect_twin_allows() {
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "idle", true));
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, "echo hi > src/out.txt"));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, "echo hi > src/out.txt"));
    assert_deny_conformant(
        "extract-targets/redirect(src)",
        &node,
        &rust,
        "bee intake gate: no bee work is active (phase: idle) — writing \"src/out.txt\"",
    );

    // Twin: SAME shape, target flipped to an allowlisted docs/ path.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "idle", true));
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, "echo hi > docs/out.md"));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, "echo hi > docs/out.md"));
    assert_allow_conformant("extract-targets/allow-twin(docs redirect)", &node, &rust);
}

#[test]
fn extract_targets_fd_duplication_is_not_a_write_and_stderr_file_redirect_twin_denies() {
    // `2>&1` duplicates a file descriptor — never a file target (decision
    // 0014); the SAME command redirecting stderr to a real source file is
    // the deny twin (one field flipped: `&1` -> `src/err.txt`).
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "idle", true));
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, "node scripts/x.mjs 2>&1"));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, "node scripts/x.mjs 2>&1"));
    assert_allow_conformant("extract-targets/fd-duplication(allow)", &node, &rust);

    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "idle", true));
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, "node scripts/x.mjs 2>src/err.txt"));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, "node scripts/x.mjs 2>src/err.txt"));
    assert_deny_conformant(
        "extract-targets/deny-twin(stderr to file)",
        &node,
        &rust,
        "bee intake gate: no bee work is active (phase: idle) — writing \"src/err.txt\"",
    );
}

#[test]
fn extract_targets_quoted_segment_merge_hits_direct_edit_deny_and_sibling_twin_allows() {
    // Adjacent quoted/unquoted segments merge into ONE token (bash word
    // splitting) — without the merge, DIRECT_EDIT_DENY could be bypassed by
    // concatenating quotes around a protected path.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "executing", true));
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, "touch '.bee/state'\".json\""));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, "touch '.bee/state'\".json\""));
    assert_deny_conformant(
        "extract-targets/quoted-merge(direct-edit)",
        &node,
        &rust,
        "bee direct-edit guard: \".bee/state.json\" is CLI-owned",
    );

    // Twin: SAME quoted-merge shape, target flipped to a non-CLI-owned sibling.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "executing", true));
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, "touch '.bee/notes'\".json\""));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, "touch '.bee/notes'\".json\""));
    assert_allow_conformant("extract-targets/allow-twin(quoted-merge sibling)", &node, &rust);
}

#[test]
fn extract_targets_sed_in_place_denies_at_idle_and_read_only_sed_twin_allows() {
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "idle", true));
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, "sed -i s/a/b/ src/app.js"));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, "sed -i s/a/b/ src/app.js"));
    assert_deny_conformant(
        "extract-targets/sed-in-place",
        &node,
        &rust,
        "bee intake gate: no bee work is active (phase: idle) — writing \"src/app.js\"",
    );

    // Twin: SAME command with the -i flag dropped — a read-only sed.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "idle", true));
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, "sed s/a/b/ src/app.js"));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, "sed s/a/b/ src/app.js"));
    assert_allow_conformant("extract-targets/allow-twin(sed without -i)", &node, &rust);
}

#[test]
fn extract_targets_bare_rm_counts_as_broad_write_and_scoped_docs_rm_twin_allows() {
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "idle", true));
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, "rm -rf"));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, "rm -rf"));
    assert_deny_conformant(
        "extract-targets/bare-rm(broad write)",
        &node,
        &rust,
        "bee intake gate: no bee work is active (phase: idle) — writing \"**\"",
    );

    // Twin: SAME rm shape, scoped to an allowlisted docs/ target.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "idle", true));
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, "rm docs/old.md"));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, "rm docs/old.md"));
    assert_allow_conformant("extract-targets/allow-twin(rm docs path)", &node, &rust);
}

#[test]
fn extract_targets_bash_containment_traversal_denies_and_in_tree_twin_allows() {
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "executing", true));
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, "touch ../outside.txt"));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, "touch ../outside.txt"));
    assert_deny_conformant(
        "extract-targets/bash-containment(traversal)",
        &node,
        &rust,
        "bee write guard denied Bash: one or more extracted targets could not be canonically contained",
    );

    // Twin: SAME touch shape, target flipped to a plain in-worktree path.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "executing", true));
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, "touch src/new.txt"));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, "touch src/new.txt"));
    assert_allow_conformant("extract-targets/allow-twin(in-tree touch)", &node, &rust);
}

// ---------------------------------------------------------------------------
// Fixture class: git-bash — staged-commit resolution from REAL git state
// (git-initialized roots), blanket staging as broad write, push never
// exempt, read-only vs unmodeled subcommands
// ---------------------------------------------------------------------------

#[test]
fn git_bash_commit_of_staged_source_denies_and_staged_docs_twin_allows() {
    // GIT SPAWN PARITY: this fixture is where the guard resolves the staged
    // index from real git state — the roots MUST be git-initialized.
    let deny_setup = |root: &Path| {
        write_state(root, "idle", true);
        git_init_root(root);
        stage_file(root, "src/app.js", "content\n");
    };
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), deny_setup);
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, "git commit -m checkpoint"));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, "git commit -m checkpoint"));
    assert_deny_conformant(
        "git-bash/commit(staged source)",
        &node,
        &rust,
        "running `git commit` — it would change \"src/app.js\"",
    );

    // Twin: SAME command, staged set flipped to a bookkeeping-only path.
    let allow_setup = |root: &Path| {
        write_state(root, "idle", true);
        git_init_root(root);
        stage_file(root, "docs/notes.md", "notes\n");
    };
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), allow_setup);
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, "git commit -m checkpoint"));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, "git commit -m checkpoint"));
    assert_allow_conformant("git-bash/allow-twin(staged docs only)", &node, &rust);
}

#[test]
fn git_bash_blanket_staging_flags_count_as_broad_writes_and_non_terminal_twin_allows() {
    // bsg-1: `git add -A` and `git commit -am` are blanket staging — the
    // extractor reports a broad write ("**") and the intake gate denies it
    // BEFORE any git spawn (the hot path stays spawn-free).
    for (name, command) in [("git add -A", "git add -A"), ("git commit -am", "git commit -am checkpoint")] {
        let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| {
            write_state(root, "idle", true);
            git_init_root(root);
        });
        let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, command));
        let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, command));
        assert_deny_conformant(
            &format!("git-bash/blanket-staging({name})"),
            &node,
            &rust,
            "bee intake gate: no bee work is active (phase: idle) — writing \"**\"",
        );
    }

    // Twin: SAME `git add -A` shape, only the phase flipped non-terminal.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| {
        write_state(root, "executing", true);
        git_init_root(root);
    });
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, "git add -A"));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, "git add -A"));
    assert_allow_conformant("git-bash/allow-twin(blanket staging, executing phase)", &node, &rust);
}

#[test]
fn git_bash_push_is_never_exempt_at_terminal_phase_and_non_terminal_twin_allows() {
    // `git push` classification is pure (no pathspec model) — no git spawn,
    // so a plain seeded root proves the spawn-free path too.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "idle", true));
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, "git push"));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, "git push"));
    assert_deny_conformant(
        "git-bash/push(never exempt)",
        &node,
        &rust,
        "git push is outward-facing and is never exempted from this gate",
    );

    // Twin: SAME command, only the phase flipped non-terminal.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "executing", true));
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, "git push"));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, "git push"));
    assert_allow_conformant("git-bash/allow-twin(push, executing phase)", &node, &rust);
}

#[test]
fn git_bash_read_only_allows_at_idle_and_unmodeled_mutation_twins_deny() {
    // Read-only allows: enumerated subcommand + flag-gated form.
    for command in ["git status", "git branch --list"] {
        let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "idle", true));
        let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, command));
        let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, command));
        assert_allow_conformant(&format!("git-bash/read-only({command})"), &node, &rust);
    }

    // Deny twin 1: `git stash` — recognized mutation with no pathspec model:
    // fail closed (unprovable), one field flipped from `git status`.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "idle", true));
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, "git stash"));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, "git stash"));
    assert_deny_conformant(
        "git-bash/deny-twin(stash unprovable)",
        &node,
        &rust,
        "running `git stash` (its changed paths could not be proved bookkeeping-only)",
    );

    // Deny twin 2: `git branch feature-x` — the SAME subcommand as the
    // flag-gated allow with only --list dropped: unrecognized, fail closed.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "idle", true));
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, "git branch feature-x"));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, "git branch feature-x"));
    assert_deny_conformant(
        "git-bash/deny-twin(bare branch unrecognized)",
        &node,
        &rust,
        "This git subcommand is not recognized as read-only or as a modeled bookkeeping-eligible mutation",
    );
}

// ---------------------------------------------------------------------------
// Fixture class: internals-reach — inline-eval lib imports deny; file-based
// runs and non-lib inline evals stay open
// ---------------------------------------------------------------------------

#[test]
fn internals_reach_inline_eval_lib_import_denies_and_file_based_and_non_lib_twins_allow() {
    let eval_cmd = r#"node -e "await import('./.bee/bin/lib/state.mjs')""#;
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "executing", true));
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, eval_cmd));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, eval_cmd));
    assert_deny_conformant(
        "internals-reach/inline-eval(bin/lib import)",
        &node,
        &rust,
        "bee internals-reach guard: this inline eval imports \"./.bee/bin/lib/state.mjs\"",
    );

    // Twin 1: SAME inline-eval shape, specifier flipped to a non-lib module.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "executing", true));
    let benign = r#"node -e "await import('node:fs')""#;
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, benign));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, benign));
    assert_allow_conformant("internals-reach/allow-twin(non-lib specifier)", &node, &rust);

    // Twin 2: SAME module path, reached as a FILE-BASED run — never blocked
    // (tests import lib modules legitimately that way).
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "executing", true));
    let file_based = "node .bee/bin/lib/state.mjs";
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, file_based));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, file_based));
    assert_allow_conformant("internals-reach/allow-twin(file-based run)", &node, &rust);
}

// ---------------------------------------------------------------------------
// Fixture class: CLI-shape — dispatcher + legacy-helper shapes validated
// against the registry (invalid denies, valid twin allows)
// ---------------------------------------------------------------------------

#[test]
fn cli_shape_dispatcher_missing_required_flag_denies_and_complete_twin_allows() {
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "executing", true));
    let bad = "node .bee/bin/bee.mjs cells show --json";
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, bad));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, bad));
    assert_deny_conformant(
        "cli-shape/invalid(dispatcher, missing --id)",
        &node,
        &rust,
        "bee CLI-shape guard: \"node .bee/bin/bee.mjs cells show --json\" does not match cells.show's schema — required, missing (--id) (field: id)",
    );

    // Twin: SAME invocation with the required flag supplied.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "executing", true));
    let good = "node .bee/bin/bee.mjs cells show --id demo-1 --json";
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, good));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, good));
    assert_allow_conformant("cli-shape/allow-twin(dispatcher, --id present)", &node, &rust);
}

#[test]
fn cli_shape_legacy_helper_shape_missing_required_flag_denies_and_complete_twin_allows() {
    // LEGACY_HELPER_RE transition guard (shim-retire, decision bbc6bcea):
    // old bee_<group>.mjs command SHAPES must keep resolving to the same
    // registry entries so the guard doesn't silently stop validating them.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "executing", true));
    let bad = "node .bee/bin/bee_cells.mjs show --json";
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, bad));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, bad));
    assert_deny_conformant(
        "cli-shape/invalid(legacy helper, missing --id)",
        &node,
        &rust,
        "does not match cells.show's schema — required, missing (--id) (field: id)",
    );

    // Twin: SAME legacy shape with the required flag supplied.
    let (node_seed, rust_seed) = seed_pair_with(enabling_config(), |root| write_state(root, "executing", true));
    let good = "node .bee/bin/bee_cells.mjs show --id demo-1 --json";
    let node = run_node(&node_seed.root, &bash_payload(&node_seed.root, good));
    let rust = run_rust(&rust_seed.root, &bash_payload(&rust_seed.root, good));
    assert_allow_conformant("cli-shape/allow-twin(legacy helper, --id present)", &node, &rust);
}

// ---------------------------------------------------------------------------
// Fixture class: stale-registry — TWO SEPARATE FIXTURES (advisor note 2),
// never one symmetric diff. Neither is satisfiable by a benign command on an
// intact root: both assert the deny precondition first.
// ---------------------------------------------------------------------------

#[test]
fn stale_cache_rust_side_skips_cli_shape_with_coverage_gap_line() {
    // RUST-SIDE ONLY: the registry cache is a rust-only artifact (node
    // imports the live mjs and cannot go stale) — this is the ACCEPTED
    // rust-only allow window, asserted rust-side only.
    let bad = "node .bee/bin/bee.mjs cells show --json";

    // Precondition (non-triviality): on an INTACT root the same command
    // DENIES — the stale skip below is a real behavioral window, not a
    // benign command passing through.
    let intact = seed_root_with_config(enabling_config());
    write_state(&intact.root, "executing", true);
    let fresh_run = run_rust(&intact.root, &bash_payload(&intact.root, bad));
    assert_eq!(
        fresh_run.status, 2,
        "precondition: the malformed CLI invocation must deny on an intact root (stderr={:?})",
        fresh_run.stderr
    );

    // Stale-cache root: SAME shape, only the dump's source_sha256 flipped so
    // it no longer matches the seeded command-registry.mjs.
    let stale = seed_root_with_config(enabling_config());
    write_state(&stale.root, "executing", true);
    let cache_path = stale.root.join(".bee/cache/command-registry.json");
    let mut dump: Value = serde_json::from_str(&fs::read_to_string(&cache_path).unwrap()).unwrap();
    dump["source_sha256"] = json!("0".repeat(64));
    fs::write(&cache_path, serde_json::to_string_pretty(&dump).unwrap()).unwrap();

    let stale_run = run_rust(&stale.root, &bash_payload(&stale.root, bad));
    assert_eq!(
        stale_run.status, 0,
        "stale cache must skip CLI-shape (accepted rust-only allow window) — stderr={:?}",
        stale_run.stderr
    );
    assert!(stale_run.stdout.trim().is_empty(), "the stale skip must not write stdout");
    assert!(stale_run.stderr.is_empty(), "the stale skip must be silent on stderr");
    let lines = read_hooks_log(&stale.root);
    assert!(
        lines.iter().any(|l| l.get("event").and_then(Value::as_str) == Some("coverage-gap")
            && l.get("gap").and_then(Value::as_str) == Some("cli-shape-registry-stale")),
        "expected a cli-shape-registry-stale coverage-gap line, got {lines:?}"
    );
}

#[test]
fn node_side_registry_import_failure_skips_cli_shape_with_its_own_log_line() {
    // NODE-SIDE ONLY: node's analogous skip is a dynamic-import failure of
    // command-registry.mjs — contained to check (d) by the hook's dedicated
    // try/catch (logCrash), a SEPARATE proof from the rust stale-cache
    // fixture above, never one diffed pair.
    let bad = "node .bee/bin/bee.mjs cells show --json";

    // Precondition (non-triviality): the same command DENIES on an intact root.
    let intact = seed_root_with_config(enabling_config());
    write_state(&intact.root, "executing", true);
    let fresh_run = run_node(&intact.root, &bash_payload(&intact.root, bad));
    assert_eq!(
        fresh_run.status, 2,
        "precondition: the malformed CLI invocation must deny on an intact root (stderr={:?})",
        fresh_run.stderr
    );

    // Import-failure root: SAME shape, only the seeded command-registry.mjs
    // removed — the import throws, check (d) fails open with a logged line.
    let broken = seed_root_with_config(enabling_config());
    write_state(&broken.root, "executing", true);
    fs::remove_file(broken.root.join(".bee/bin/lib/command-registry.mjs")).unwrap();

    let broken_run = run_node(&broken.root, &bash_payload(&broken.root, bad));
    assert_eq!(
        broken_run.status, 0,
        "a registry import failure must fail open for check (d) only — stderr={:?}",
        broken_run.stderr
    );
    assert!(broken_run.stdout.trim().is_empty(), "the import-failure skip must not write stdout");
    let lines = read_hooks_log(&broken.root);
    assert!(
        lines.iter().any(|l| l
            .get("error")
            .and_then(Value::as_str)
            .map(|s| s.contains("command-registry"))
            .unwrap_or(false)),
        "expected a logged line naming the failed command-registry import, got {lines:?}"
    );
}

// ---------------------------------------------------------------------------
// Fixture class: crash fail-open — an internal fault exits 0 (never a
// denial) plus a hooks.jsonl crash line, on the BASH path
// ---------------------------------------------------------------------------

#[test]
fn crash_fail_open_node_bash_path_exits_zero_and_logs_crash_instead_of_denying() {
    // The seeded root is shaped so a HEALTHY Bash run would DENY (idle
    // intake gate on a src redirect) — proving the exit 0 below is the
    // fail-open contract engaging, not a quietly-allowing fixture.
    let seeded = seed_root_with_config(enabling_config());
    write_state(&seeded.root, "idle", true);
    let stdin = bash_payload(&seeded.root, "echo hi > src/out.txt");
    let healthy = run_node(&seeded.root, &stdin);
    assert_eq!(healthy.status, 2, "precondition: the healthy oracle denies this shape");

    fs::write(
        seeded.root.join(".bee/bin/lib/guards.mjs"),
        b"throw new Error('rig-injected-fault: guards.mjs unavailable');\n",
    )
    .unwrap();
    for rel in ["bin/hooks/adapter.mjs", "bin/hooks/bee-write-guard.mjs", "bin/lib/state.mjs"] {
        assert_eq!(
            sha256_hex(&repo_root().join(".bee").join(rel)),
            sha256_hex(&seeded.root.join(".bee").join(rel)),
            "oracle surface {rel} must stay pristine in the crash fixture"
        );
    }

    let node = run_node(&seeded.root, &stdin);
    assert_eq!(node.status, 0, "fail-open contract: an internal crash must exit 0, never deny — stderr={:?}", node.stderr);
    assert!(node.stdout.trim().is_empty(), "a crash must never leak to stdout");
    let lines = read_hooks_log(&seeded.root);
    assert!(
        lines.iter().any(|l| l
            .get("error")
            .and_then(Value::as_str)
            .map(|s| s.contains("rig-injected-fault"))
            .unwrap_or(false)),
        "expected a crash line naming the injected fault, got {lines:?}"
    );
}

#[test]
fn crash_fail_open_rust_wrapper_catches_panic_exits_zero_and_writes_crash_line() {
    // The compiled port's crash class is a genuine panic inside the decision
    // region — the SAME fail-open boundary run() wraps the whole Bash path
    // in. Proven directly against the public wrapper with a real panic.
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let code = queen_bee::hooks::write_guard::run_fail_open(Some(root), Some("repo"), || {
        panic!("rig-injected-fault: deliberate panic in the bash decision region");
    });
    assert_eq!(code, 0, "fail-open contract: a panic must resolve to exit 0");
    let lines = read_hooks_log(root);
    assert!(
        lines.iter().any(|l| l
            .get("error")
            .and_then(Value::as_str)
            .map(|s| s.contains("rig-injected-fault"))
            .unwrap_or(false)),
        "expected a crash line naming the injected panic, got {lines:?}"
    );
}
