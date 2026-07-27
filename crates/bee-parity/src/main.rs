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
mod cmdcheck;
mod differ;
mod enrich;
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
        Some("--status-check") => cmd_status_check(),
        Some("--cmd-check") => cmd_cmd_check(&args[1..]),
        _ => {
            eprintln!("usage: bee-parity <--self-check|--status-check|--cmd-check (--all|--group NAME)>");
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

/// Resolve a sibling workspace binary next to this one. BINARY FRESHNESS
/// (this cell's action note): neither `bee-parity` nor `queen-bench`
/// depends on `queen-bee` as a cargo dependency, so `cargo run -p
/// bee-parity` alone will happily find and measure a STALE
/// `target/release/queen-bee`. The verify command therefore builds the
/// whole workspace first; this resolver only checks the binary EXISTS, it
/// cannot tell fresh from stale.
fn sibling_bin(stem: &str) -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let dir = exe
        .parent()
        .ok_or_else(|| "current_exe has no parent dir".to_string())?;
    let name = if cfg!(windows) { format!("{stem}.exe") } else { stem.to_string() };
    let path = dir.join(name);
    if !path.exists() {
        return Err(format!(
            "{stem} binary not found at {} — build the whole workspace first (cargo build --release --manifest-path crates/Cargo.toml)",
            path.display()
        ));
    }
    Ok(path)
}

fn queen_bench_bin() -> Result<PathBuf, String> {
    sibling_bin("queen-bench")
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

// ─── --status-check (rust-port-15, D7a: the FIRST real mjs↔rust diff) ────
//
// `--self-check` above proves the RIG. This arm proves the PORT: the same
// `status` command, over clones of the same host-real fixture store, run
// once through the frozen `node .bee/bin/bee.mjs` oracle and once through
// the compiled `queen-bee`, diffed byte-for-byte after the ONE declared
// volatile allowlist (`normalize.rs`) — stdout, exit code, and the post-run
// store tree (minus the fixed whole-path exclusion set in `differ.rs`).
//
// Both output legs are checked, not just `--json`: D3 byte-compatibility
// covers the human text renderer too, and a text-only normalization leak
// would otherwise be invisible. Each leg carries its OWN seeded-mutation
// negative control, asserted on the STDOUT diff specifically (a tree-only
// detection would prove nothing about that leg's output comparison).

fn cmd_status_check() -> ExitCode {
    match run_status_check() {
        Ok(summary) => {
            println!("bee-parity --status-check: PASS — {summary}");
            ExitCode::SUCCESS
        }
        Err(msg) => {
            eprintln!("bee-parity --status-check: FAIL — {msg}");
            ExitCode::FAILURE
        }
    }
}

fn run_status_check() -> Result<String, String> {
    let repo_root = repo_root();
    let bee_mjs = repo_root.join(".bee").join("bin").join("bee.mjs");
    if !bee_mjs.exists() {
        return Err(format!("bee.mjs not found at {}", bee_mjs.display()));
    }
    let queen_bench = queen_bench_bin()?;
    let bins = runner::Binaries { bee_mjs, queen_bee: sibling_bin("queen-bee")? };

    let mut ws = Workspace::new();
    let golden = ws.track(fresh_temp_dir("status-golden"));
    assert_temp_outside_repo(&repo_root, &golden)?;
    generate_fixture(&queen_bench, &golden)?;

    let mut leg_reports = Vec::new();
    for scenario in [Scenario::Plain, Scenario::Enriched] {
        for leg in [runner::Leg::Json, runner::Leg::Text, runner::Leg::JsonLanesFull] {
            leg_reports.push(check_one_leg(&repo_root, &bins, &golden, scenario, leg, &mut ws)?);
        }
    }

    Ok(leg_reports.join("; "))
}

/// Which store SHAPE the legs run over.
///
/// `Plain` is the D5 fixture exactly as generated — the pinned-size store
/// the bench measures. `Enriched` is that same store plus the additive
/// records in [`enrich`], which are the ONLY way most of `buildStatus`'s
/// and `renderStatusText`'s conditional branches ever execute (the fixture
/// is deliberately quiet: idle phase, no feature, no lanes, no handoff, no
/// contention, no configured commands or models). A parity proof that runs
/// only over the quiet store proves parity only for the quiet paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scenario {
    Plain,
    Enriched,
}

impl Scenario {
    fn label(self) -> &'static str {
        match self {
            Scenario::Plain => "plain",
            Scenario::Enriched => "enriched",
        }
    }

    fn prepare(self, root: &Path) -> Result<(), String> {
        match self {
            Scenario::Plain => Ok(()),
            Scenario::Enriched => enrich::apply(root),
        }
    }

    /// Each scenario pins its OWN expected `state.json` body before
    /// flipping `phase`, so a mutation is never seeded blindly.
    fn seed_mutation(self, root: &Path) -> Result<(), String> {
        match self {
            Scenario::Plain => mutate::seed_mutation(root),
            Scenario::Enriched => enrich::seed_enriched_mutation(root),
        }
    }

    /// The session id both runtimes resolve for this scenario. The
    /// enriched scenario pins the live session it writes, so
    /// `buildLaneSummary` resolves an ACTIVE lane and serializes that
    /// lane record IN FULL — the only way the default (non-`--lanes-full`)
    /// payload ever exercises `build_lane_rows`'s key ordering. The plain
    /// fixture has no session records, so it pins nothing and clears the
    /// ambient environment instead.
    fn session_id(self) -> Option<&'static str> {
        match self {
            Scenario::Plain => None,
            Scenario::Enriched => Some(enrich::LIVE_SESSION_ID),
        }
    }
}

/// One scenario x one output leg, end to end: parity (mjs vs rust, zero
/// diff, both exit 0) plus that combination's own seeded-mutation negative
/// control.
fn check_one_leg(
    repo_root: &Path,
    bins: &runner::Binaries,
    golden: &Path,
    scenario: Scenario,
    leg: runner::Leg,
    ws: &mut Workspace,
) -> Result<String, String> {
    let label = format!("{}/{}", scenario.label(), leg.label());
    let root_mjs = ws.track(fresh_temp_dir(&format!("status-{}-{}-mjs", scenario.label(), leg.label())));
    let root_rust = ws.track(fresh_temp_dir(&format!("status-{}-{}-rust", scenario.label(), leg.label())));
    let root_mutated = ws.track(fresh_temp_dir(&format!("status-{}-{}-mutated", scenario.label(), leg.label())));
    for dir in [&root_mjs, &root_rust, &root_mutated] {
        assert_temp_outside_repo(repo_root, dir)?;
        clone::copy_tree(golden, dir)?;
        scenario.prepare(dir)?;
    }
    scenario.seed_mutation(&root_mutated)?;

    // Root-safety before EVERY command (validation decision B5).
    rootsafety::assert_structural_safety(repo_root, &root_mjs)?;
    let run_mjs = runner::run_status(bins, runner::Runtime::Mjs, leg, &root_mjs, scenario.session_id())?;
    rootsafety::assert_structural_safety(repo_root, &root_rust)?;
    let run_rust = runner::run_status(bins, runner::Runtime::QueenBee, leg, &root_rust, scenario.session_id())?;

    // Positive content assertions, per leg and per runtime — a zero diff
    // between two EMPTY outputs would otherwise read as parity. The JSON
    // leg reuses the existing fixture-signature read; the text leg asserts
    // the renderer's own first line.
    if leg.is_json() {
        if scenario == Scenario::Plain {
            // The fixture-signature read pins the PLAIN fixture's own body
            // (`phase: idle`, `feature: null`, `workers: []`), so it
            // applies to the plain scenario only — the enriched scenario
            // deliberately rewrites exactly those fields.
            rootsafety::assert_resolves_to_fixture(&run_mjs)?;
            rootsafety::assert_resolves_to_fixture(&run_rust)?;
        } else {
            for (who, run) in [("mjs", &run_mjs), ("queen-bee", &run_rust)] {
                assert_enriched_signature(&label, who, run, leg)?;
            }
        }
    } else {
        for (who, run) in [("mjs", &run_mjs), ("queen-bee", &run_rust)] {
            if !run.stdout.contains("bee status (plugin v") {
                return Err(format!(
                    "[{label}] {who} produced no rendered status text (missing the `bee status (plugin v…)` header) — a zero diff against it would prove nothing: {}",
                    truncate(&run.stdout)
                ));
            }
        }
    }

    // Advisor notes 1+2: exit-0 is checked INDEPENDENTLY of "diff empty" —
    // two identical failures are not parity.
    if run_mjs.exit_code != 0 || run_rust.exit_code != 0 {
        return Err(format!(
            "[{label}] legs did not both exit 0 (mjs={}, queen-bee={}) — a zero diff between two failures is not parity.\nmjs stdout:\n{}\nqueen-bee stdout:\n{}",
            run_mjs.exit_code,
            run_rust.exit_code,
            truncate(&run_mjs.stdout),
            truncate(&run_rust.stdout)
        ));
    }

    let parity = differ::diff_legs(&run_mjs, &run_rust)?;
    if !parity.is_clean() {
        return Err(format!("[{label}] mjs vs queen-bee reported a diff: {}", parity.describe()));
    }

    // PER-LEG NEGATIVE CONTROL: the same comparison, against a store whose
    // `phase` was deliberately flipped, MUST still report a STDOUT diff.
    // Asserting on `stdout_diff` (not merely `!is_clean()`) is the point —
    // the mutated state.json alone would make the TREE differ, which would
    // mask a leg whose stdout comparison had been normalized into silence.
    rootsafety::assert_structural_safety(repo_root, &root_mutated)?;
    let run_mutated = runner::run_status(bins, runner::Runtime::QueenBee, leg, &root_mutated, scenario.session_id())?;
    if run_mutated.exit_code != 0 {
        return Err(format!(
            "[{label}] seeded-mutation leg exited {} unexpectedly (status should still succeed against a mutated-but-valid store): {}",
            run_mutated.exit_code,
            truncate(&run_mutated.stdout)
        ));
    }
    let mutation_diff = differ::diff_legs(&run_mjs, &run_mutated)?;
    if mutation_diff.stdout_diff.is_none() {
        return Err(format!(
            "[{label}] seeded-mutation check FAILED: the deliberately flipped `phase` produced ZERO STDOUT diff on this leg — its output comparison cannot detect a real divergence, so its zero-diff parity result above cannot be trusted"
        ));
    }

    Ok(format!(
        "[{label}] mjs vs queen-bee zero diff over {} bytes of stdout (+ exit + store tree), both exit 0; seeded-mutation detected in stdout",
        run_mjs.stdout.len()
    ))
}

/// The enriched scenario's positive-content read: proof that the store
/// enrichment actually reached the payload on BOTH runtimes, so a zero diff
/// there is a diff over the branches enrichment exists to light up — not
/// two runtimes agreeing that they ignored the extra records.
fn assert_enriched_signature(
    label: &str,
    who: &str,
    run: &differ::RunResult,
    leg: runner::Leg,
) -> Result<(), String> {
    let compact: String = run.stdout.chars().filter(|c| !c.is_whitespace()).collect();
    // A FULL lane record must actually appear in this payload — that is
    // the object whose key order `build_lane_rows` decides, and without it
    // a zero diff says nothing about the `shift_remove`/`swap_remove`
    // hazard (rust-port-15). The default payload reaches it only via the
    // ACTIVE lane of the summary; `--lanes-full` reaches it via the array.
    let lane_row_marker = match leg {
        runner::Leg::JsonLanesFull => "\"lanes\":[{\"schema_version\"",
        _ => "\"active\":{\"schema_version\"",
    };
    if !compact.contains(lane_row_marker) {
        return Err(format!(
            "[{label}] {who}: no FULL lane record in the payload (expected `{lane_row_marker}`) — the lane-row key ordering this scenario exists to pin was never serialized, so a zero diff would not cover it.\n{}",
            truncate(&run.stdout)
        ));
    }
    if !compact.contains("\"bound_sessions\":[\"parity-live-session\"]") {
        return Err(format!(
            "[{label}] {who}: the lane record carries no bound session — `resolveSessionId` did not resolve the pinned live session, so the active-lane path was not exercised.\n{}",
            truncate(&run.stdout)
        ));
    }
    let markers = [
        ("\"phase\":\"scribing\"", "the enriched phase"),
        ("\"feature\":\"fixture-enriched\"", "the enriched feature"),
        ("\"gate_bypass_level\":\"full\"", "the configured gate-bypass level"),
        ("\"contention\":{", "the contention block"),
        ("\"worst_wait_ms\":4500", "the contention worst-wait scan across non-busy events"),
        ("\"kind\":\"cli\"", "the cli-shaped model slot"),
        ("\"session_id\":\"parity-live-session\"", "the live worker row"),
        ("\"cell\":\"fixture-00001\"", "the active claim on that worker row"),
        ("\"ids\":[\"stub-a\",\"stub-b\"]", "the capture-queue stub ids (flushed stub excluded, sorted by `at`)"),
        ("\"critical_patterns_present\":true", "the critical-patterns marker"),
    ];
    for (marker, what) in markers {
        if !compact.contains(marker) {
            return Err(format!(
                "[{label}] {who}: enrichment did not reach the payload — {what} is missing (`{marker}`). A zero diff over an unenriched payload would not prove the branches this scenario exists to cover.\n{}",
                truncate(&run.stdout)
            ));
        }
    }
    Ok(())
}

// ─── --cmd-check (rpl-1, D7a): the GENERIC command-parity arm ─────────────
//
// `--status-check` above proves ONE command. This arm proves any command, and
// is the shared verify surface every ledger cell in this slice hangs its own
// group's scenarios on. See `cmdcheck.rs` for the four properties it has to
// have and why each exists.

fn cmd_cmd_check(args: &[String]) -> ExitCode {
    match run_cmd_check(args) {
        Ok(summary) => {
            println!("bee-parity --cmd-check: PASS\n{summary}");
            ExitCode::SUCCESS
        }
        Err(msg) => {
            eprintln!("bee-parity --cmd-check: FAIL — {msg}");
            ExitCode::FAILURE
        }
    }
}

fn run_cmd_check(args: &[String]) -> Result<String, String> {
    let selector = cmdcheck::parse_selector(args)?;
    let repo_root = repo_root();
    let bee_mjs = repo_root.join(".bee").join("bin").join("bee.mjs");
    if !bee_mjs.exists() {
        return Err(format!("bee.mjs not found at {}", bee_mjs.display()));
    }
    let queen_bench = queen_bench_bin()?;
    let bins = runner::Binaries { bee_mjs, queen_bee: sibling_bin("queen-bee")? };

    let set = cmdcheck::all_scenarios()?;
    if set.is_empty() {
        return Err("no scenarios are registered at all — `--cmd-check` cannot pass on an empty table".to_string());
    }
    // The registered count per group is printed FIRST and unconditionally —
    // the number a verify actually exercised has to be visible in its own
    // output, not inferred from the absence of failures.
    let counts = set.render_counts();

    let selected = set.select(selector.group.as_deref());
    if let Some(group) = &selector.group {
        if selected.is_empty() {
            return Err(format!(
                "{counts}\ngroup `{group}` has ZERO registered scenarios — a verify that exercises nothing is not a verify. Register this group's scenarios in cmdcheck::all_scenarios() before capping it."
            ));
        }
    }
    if selected.is_empty() {
        return Err(format!("{counts}\nno scenarios selected"));
    }

    let mut ws = Workspace::new();
    let golden = ws.track(fresh_temp_dir("cmd-golden"));
    assert_temp_outside_repo(&repo_root, &golden)?;
    generate_fixture(&queen_bench, &golden)?;

    let mut lines = vec![
        counts,
        format!("running {} of {} registered scenario(s):", selected.len(), set.len()),
    ];
    for scenario in selected {
        let mut track = |dir: PathBuf| ws.track(dir);
        lines.push(format!("  {}", cmdcheck::check_one_scenario(&repo_root, &bins, &golden, scenario, &mut track)?));
    }
    Ok(lines.join("\n"))
}

fn truncate(s: &str) -> String {
    let limit = 2000usize;
    if s.len() <= limit {
        return s.to_string();
    }
    let mut end = limit;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}… (truncated)", &s[..end])
}

/// Guard against the OS temp dir itself somehow living inside the repo tree
/// — exactly the class of mistake B5 exists to catch — BEFORE any clone
/// touches disk.
fn assert_temp_outside_repo(repo_root: &Path, dir: &Path) -> Result<(), String> {
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
    Ok(())
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
