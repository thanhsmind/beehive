//! queen-bee: the single compiled binary that replaces bee.mjs + hooks in
//! host repos (CONTEXT.md D2).
//!
//! Slice 0 scope: workspace skeleton only (`ping`, `--version`). Slice 1
//! (rust-port-7) adds the `hook <name>` runtime: `queen-bee hook tools-logger`
//! and `queen-bee hook codex-subagent-audit` (the two trivial hooks); the
//! remaining 7 `bee-*.mjs` hooks and the full CLI surface (116 defs / 19
//! groups) land in later slices. See `queen_bee::hooks` for the dispatch and
//! `queen_bee::adapter` for the shared runtime port of
//! `.bee/bin/hooks/adapter.mjs`.

use std::env;
use std::io::Read;
use std::process::ExitCode;

fn main() -> ExitCode {
    let entered_at = std::time::Instant::now();
    let args: Vec<String> = env::args().skip(1).collect();

    match args.first().map(String::as_str) {
        Some("ping") => {
            println!("pong");
            ExitCode::SUCCESS
        }
        Some("--version") => {
            println!("queen-bee {}", bee_core::VERSION);
            ExitCode::SUCCESS
        }
        Some("hook") => run_hook_command(&args[1..]),
        Some("status") => run_status_command(&args[1..], entered_at),
        _ => {
            eprintln!("usage: queen-bee <ping|--version|status|hook NAME>");
            ExitCode::FAILURE
        }
    }
}

/// `bee.mjs status [--json] [--lanes-full]` (CONTEXT.md D3/D7a). Root
/// resolution mirrors `main()`'s `findRepoRoot(process.cwd())`, which is
/// `resolveRoots(cwd).storeRoot` (state.mjs:925) — NOT `workRoot`.
fn run_status_command(rest: &[String], entered_at: std::time::Instant) -> ExitCode {
    let use_json = rest.iter().any(|t| t == "--json" || t.starts_with("--json="));
    let lanes_full = rest.iter().any(|t| t == "--lanes-full");
    // `--profile` is a queen-bee-only diagnostic (D5's "report the
    // measured p95 plus a profile" escape). It writes to STDERR only, so
    // stdout stays byte-identical to the frozen oracle with or without it
    // and no parity leg can be perturbed by measuring.
    let profile_requested = rest.iter().any(|t| t == "--profile");

    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(err) => return status_error(&format!("could not resolve the current directory: {err}"), use_json),
    };
    let Some(root) = queen_bee::adapter::resolve_roots(&cwd).store_root else {
        return status_error(
            "No bee repo root found (no .bee/onboarding.json or .git up the tree). Run bee-hive onboarding.",
            use_json,
        );
    };

    let ctx = queen_bee::status::StatusContext::from_process(&root);
    let mut profile = queen_bee::status::Profile::default();
    let status = queen_bee::status::build_status_profiled(
        &ctx,
        queen_bee::status::StatusOptions { lanes_full },
        &mut profile,
    );
    // Serialization + the stdout write are timed too: they are real
    // per-invocation cost, and leaving them out would understate the
    // envelope the D5 escape report has to be honest about.
    let emit_started = std::time::Instant::now();
    if use_json {
        print!("{}", queen_bee::status::to_json_stdout(&status));
    } else {
        println!("{}", queen_bee::status::render_status_text(&status));
    }
    profile.record("serialize + stdout write", emit_started);

    // The profile is emitted LAST, and only to stderr — stdout is already
    // flushed byte-identical to the frozen oracle before a single profile
    // character exists.
    if profile_requested {
        profile.set_process_ms(entered_at);
        eprint!("{}", profile.render());
    }
    ExitCode::SUCCESS
}

/// `emitError(message, useJson)` — `--json` puts the error on STDOUT as
/// `{"error": ...}`; the text path writes to stderr. Exit 1 either way.
fn status_error(message: &str, use_json: bool) -> ExitCode {
    if use_json {
        println!("{}", serde_json::json!({ "error": message }));
    } else {
        eprintln!("{message}");
    }
    ExitCode::from(1)
}

fn run_hook_command(rest: &[String]) -> ExitCode {
    let Some(name) = rest.first() else {
        eprintln!("usage: queen-bee hook <name> [--source plugin|repo]");
        return ExitCode::FAILURE;
    };
    let argv = &rest[1..];

    let mut raw_stdin = String::new();
    // A stdin read failure is treated exactly like empty stdin (the
    // adapter's own tolerant-empty-payload path) — never a crash exit,
    // matching the fail-open contract every hook operates under.
    let _ = std::io::stdin().read_to_string(&mut raw_stdin);

    let code = queen_bee::hooks::run_hook(name, argv, &raw_stdin);
    u8::try_from(code).map(ExitCode::from).unwrap_or(ExitCode::FAILURE)
}
