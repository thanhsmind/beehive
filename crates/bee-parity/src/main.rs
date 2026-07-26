//! bee-parity: the D7a CLI parity harness (CONTEXT.md D7a, plan.md Slice 0
//! cell 4 / rust-port-4) — same command, same fixture store, mjs vs rust
//! stdout/exit/side-effect files diffed.
//!
//! Slice 0 scope: no registry group has flipped yet (plan.md epic map:
//! Slice 0 is "proofs only, no flips"), so there is no queen-bee side to
//! diff against. `--self-check` proves the rig's OWN detection power
//! instead, per plan.md cell 4 / advisor notes 1+2+7:
//!
//! - self-parity: the SAME `bee.mjs status --json` command run against two
//!   independent clones of one generated fixture store must diff clean
//!   (after the declared volatile allowlist is applied) — proving the
//!   differ doesn't cry wolf on two runs that really did the same thing;
//! - seeded-mutation: a THIRD clone gets one deliberate, known mutation
//!   (`phase` flipped) before the same command runs against it — the
//!   differ MUST report a diff there, or the self-check itself fails
//!   (a rig that reports zero diff on a real divergence is a rig that
//!   cannot be trusted anywhere else in the port).
//!
//! Every root this binary touches is validated by `rootsafety` before its
//! result is trusted (CONTEXT.md D3 / validation decision B5): the harness
//! never reads or writes the repo's live `.bee/`.

mod clone;
mod differ;
mod mutate;
mod normalize;
mod rootsafety;
mod runner;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--self-check") => cmd_self_check(),
        _ => {
            eprintln!("usage: bee-parity --self-check");
            ExitCode::FAILURE
        }
    }
}

/// `crates/bee-parity` at compile time -> repo root two levels up. Same
/// technique as `queen-bench::repo_root` (rust-port-2 precedent) — robust
/// to the process's actual cwd at run time, since `main` below never
/// changes its own cwd (only the CHILD `node` invocations get a `cwd`).
fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or(manifest_dir)
}

fn queen_bench_bin() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let dir = exe
        .parent()
        .ok_or_else(|| "current_exe has no parent dir".to_string())?;
    let name = if cfg!(windows) { "queen-bench.exe" } else { "queen-bench" };
    let path = dir.join(name);
    if !path.exists() {
        return Err(format!(
            "queen-bench binary not found at {} — build the workspace first (cargo build --release --manifest-path crates/Cargo.toml)",
            path.display()
        ));
    }
    Ok(path)
}

fn now_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn fresh_temp_dir(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("bee-parity-{label}-{}-{}", std::process::id(), now_nanos()))
}

fn cmd_self_check() -> ExitCode {
    match run_self_check() {
        Ok(summary) => {
            println!("bee-parity --self-check: PASS — {summary}");
            ExitCode::SUCCESS
        }
        Err(msg) => {
            eprintln!("bee-parity --self-check: FAIL — {msg}");
            ExitCode::FAILURE
        }
    }
}

/// A set of temp dirs this run created, always cleaned up on the way out
/// (both success and failure paths) — never leave a stray fixture behind.
struct Workspace {
    dirs: Vec<PathBuf>,
}

impl Workspace {
    fn new() -> Self {
        Self { dirs: Vec::new() }
    }
    fn track(&mut self, dir: PathBuf) -> PathBuf {
        self.dirs.push(dir.clone());
        dir
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        for dir in &self.dirs {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

fn run_self_check() -> Result<String, String> {
    let repo_root = repo_root();
    let bee_mjs = repo_root.join(".bee").join("bin").join("bee.mjs");
    if !bee_mjs.exists() {
        return Err(format!("bee.mjs not found at {}", bee_mjs.display()));
    }
    let queen_bench = queen_bench_bin()?;

    let mut ws = Workspace::new();
    let golden = ws.track(fresh_temp_dir("golden"));
    let root_a = ws.track(fresh_temp_dir("leg-a"));
    let root_b = ws.track(fresh_temp_dir("leg-b"));
    let root_c = ws.track(fresh_temp_dir("leg-c-mutated"));

    // Guard against the OS temp dir itself somehow living inside the repo
    // tree (unlikely, but this is exactly the class of mistake B5 exists
    // to catch) BEFORE any clone touches disk.
    for dir in [&golden, &root_a, &root_b, &root_c] {
        let canon_dir = dir.parent().and_then(|p| p.canonicalize().ok());
        let canon_repo = repo_root.canonicalize().ok();
        if let (Some(d), Some(r)) = (canon_dir, canon_repo) {
            if d.starts_with(&r) {
                return Err(format!(
                    "refusing to run: temp root parent {} resolves inside the repo tree {}",
                    dir.display(),
                    repo_root.display()
                ));
            }
        }
    }

    // 1. Generate ONE golden fixture via the real queen-bench --generate
    //    (D7a: "Fixture stores come from queen-bench's generator").
    generate_fixture(&queen_bench, &golden)?;

    // 2. Clone it into three independent temp roots: two for self-parity,
    //    one for the seeded-mutation check.
    clone::copy_tree(&golden, &root_a)?;
    clone::copy_tree(&golden, &root_b)?;
    clone::copy_tree(&golden, &root_c)?;

    // 3. Seed a known, deliberate divergence into leg C only.
    mutate::seed_mutation(&root_c)?;

    // 4. Run the SAME command (`status --json`) through the SAME `bee.mjs`
    //    against each leg, asserting root-resolution safety around every
    //    single invocation (CONTEXT.md D3 / validation B5: "before every
    //    command").
    rootsafety::assert_structural_safety(&repo_root, &root_a)?;
    let run_a = runner::run_bee_status(&bee_mjs, &root_a)?;
    rootsafety::assert_resolves_to_fixture(&run_a)?;

    rootsafety::assert_structural_safety(&repo_root, &root_b)?;
    let run_b = runner::run_bee_status(&bee_mjs, &root_b)?;
    rootsafety::assert_resolves_to_fixture(&run_b)?;

    // Leg C is deliberately mutated, so it will NOT show the plain
    // fixture signature (`phase: idle`) — only the structural half of
    // root-safety applies here; the content-signature check is specific
    // to the un-mutated fixture body and would reject leg C by design.
    rootsafety::assert_structural_safety(&repo_root, &root_c)?;
    let run_c = runner::run_bee_status(&bee_mjs, &root_c)?;
    if run_c.exit_code != 0 {
        return Err(format!(
            "seeded-mutation leg exited {} unexpectedly (status --json should still succeed against a mutated-but-valid store): {}",
            run_c.exit_code, run_c.stdout
        ));
    }

    // 5a. SELF-PARITY: leg A vs leg B must diff clean, and both must have
    //     exited 0 (advisor notes 1+2: two identical crashes are not
    //     parity — exit-0 is checked independently of "diff empty").
    let self_parity = differ::diff_legs(&run_a, &run_b)?;
    if run_a.exit_code != 0 || run_b.exit_code != 0 {
        return Err(format!(
            "self-parity legs did not both exit 0 (A={}, B={}) — a zero diff between two crashes is not parity",
            run_a.exit_code, run_b.exit_code
        ));
    }
    if !self_parity.is_clean() {
        return Err(format!("self-parity (leg A vs leg B) reported a diff: {}", self_parity.describe()));
    }

    // 5b. SEEDED-MUTATION: leg A vs the deliberately mutated leg C MUST
    //     diff — missing detection here is itself a self-check failure
    //     (advisor note 2 / must_have #2).
    let mutation_diff = differ::diff_legs(&run_a, &run_c)?;
    if mutation_diff.is_clean() {
        return Err(
            "seeded-mutation check FAILED: leg A vs the deliberately mutated leg C reported ZERO diff — the differ cannot detect a real divergence, so its zero-diff results elsewhere cannot be trusted".to_string(),
        );
    }

    Ok(format!(
        "self-parity (A vs B, `status --json`) zero diff, both exit 0, both legs passed the fixture sanity read; seeded-mutation (A vs C) correctly detected: {}",
        mutation_diff.describe()
    ))
}

fn generate_fixture(queen_bench: &Path, out_dir: &Path) -> Result<(), String> {
    let output = Command::new(queen_bench)
        .arg("--generate")
        .arg("--out")
        .arg(out_dir)
        .output()
        .map_err(|e| format!("failed to spawn queen-bench --generate: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "queen-bench --generate exited {:?}; stderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}
