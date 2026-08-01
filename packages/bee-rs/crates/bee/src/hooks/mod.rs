// hooks — `bee hook <name>` subcommands: the Rust replacements for the
// `node .bee/hooks/bee-<name>.mjs` wrapper scripts. claude-hooks.json is the
// only wiring change (contract C3); each hook's stdin/stdout/exit contract
// is byte-identical to its .mjs original.
//
// stdin is read ONCE here and handed to the hook impl; a native hook that
// meets a genuinely-rare edge it cannot replicate byte-for-byte returns
// `Outcome::Delegate` and the ORIGINAL .mjs wrapper re-runs with the same
// bytes — the strangler bail, at hook granularity.

pub mod adapter;
pub mod chain_nudge;
pub mod cli_shape;
pub mod codex_subagent_audit;
pub mod model_guard;
pub mod prompt_context;
pub mod session_close;
pub mod session_init;
pub mod state_sync;
pub mod tools_logger;
pub mod write_guard;

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

pub enum Outcome {
    Done(ExitCode),
    /// Re-run the Node wrapper with the same stdin. Native code must have
    /// produced NO output before returning this.
    Delegate,
}

fn read_stdin_once() -> Vec<u8> {
    use std::io::Read;
    let mut buf = Vec::new();
    let _ = std::io::stdin().read_to_end(&mut buf);
    buf
}

fn hook_js_path(name: &str) -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("BEE_HOOK_JS_DIR") {
        let p = PathBuf::from(dir).join(format!("bee-{name}.mjs"));
        return p.is_file().then_some(p);
    }
    if let Some(root) = std::env::var_os("CLAUDE_PLUGIN_ROOT") {
        let p = PathBuf::from(root)
            .join("packages")
            .join("bee")
            .join("hooks")
            .join(format!("bee-{name}.mjs"));
        if p.is_file() {
            return Some(p);
        }
    }
    // Repo-checkout fallback: walk up from the exe, then cwd.
    let mut starts: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            starts.push(dir.to_path_buf());
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        starts.push(cwd);
    }
    for start in starts {
        let mut dir: Option<&std::path::Path> = Some(&start);
        while let Some(d) = dir {
            // Vendored host layout first (deployment-true), then the bee
            // repo checkout layout.
            let vendored = d
                .join(".bee")
                .join("bin")
                .join("hooks")
                .join(format!("bee-{name}.mjs"));
            if vendored.is_file() {
                return Some(vendored);
            }
            let repo = d
                .join("packages")
                .join("bee")
                .join("hooks")
                .join(format!("bee-{name}.mjs"));
            if repo.is_file() {
                return Some(repo);
            }
            dir = d.parent();
        }
    }
    None
}

fn delegate_to_node(name: &str, argv_rest: &[String], stdin: &[u8]) -> ExitCode {
    // Test tripwire: prove a hook ran NATIVE (a delegation under this env is
    // loud instead of silently byte-identical-by-construction).
    if std::env::var_os("BEE_HOOK_NO_DELEGATE").is_some() {
        eprintln!("bee(rs): hook {name} DELEGATED (BEE_HOOK_NO_DELEGATE tripwire)");
        return ExitCode::from(42);
    }
    let Some(script) = hook_js_path(name) else {
        // A hook must NEVER fail closed over infrastructure: exit 0 silently,
        // matching the wrappers' own fail-open posture.
        return ExitCode::SUCCESS;
    };
    use std::io::Write;
    use std::process::{Command, Stdio};
    let child = Command::new("node")
        .arg(&script)
        .args(argv_rest)
        .stdin(Stdio::piped())
        .spawn();
    let Ok(mut child) = child else { return ExitCode::SUCCESS };
    if let Some(mut pipe) = child.stdin.take() {
        let _ = pipe.write_all(stdin);
    }
    match child.wait() {
        Ok(s) => ExitCode::from(s.code().unwrap_or(0).clamp(0, 255) as u8),
        Err(_) => ExitCode::SUCCESS,
    }
}

/// Dispatch `bee hook <name> [args...]`. Returns None when argv is not a
/// hook invocation at all.
pub fn try_native(args: &[OsString]) -> Option<ExitCode> {
    if args.first()?.to_str()? != "hook" {
        return None;
    }
    let name = args.get(1).and_then(|a| a.to_str()).unwrap_or("").to_string();
    let rest: Vec<String> = args[2..]
        .iter()
        .filter_map(|a| a.to_str().map(str::to_string))
        .collect();
    let stdin = read_stdin_once();
    let stdin_str = String::from_utf8_lossy(&stdin).into_owned();
    let outcome = match name.as_str() {
        "tools-logger" => tools_logger::run(&rest, &stdin_str),
        "codex-subagent-audit" => codex_subagent_audit::run(&rest, &stdin_str),
        "chain-nudge" => chain_nudge::run(&rest, &stdin_str),
        "state-sync" => state_sync::run(&rest, &stdin_str),
        "prompt-context" => prompt_context::run(&rest, &stdin_str),
        "session-init" => session_init::run(&rest, &stdin_str),
        "session-close" => session_close::run(&rest, &stdin_str),
        "model-guard" => model_guard::run(&rest, &stdin_str),
        "write-guard" => write_guard::run(&rest, &stdin_str),
        _ => {
            eprintln!("bee hook: unknown hook \"{name}\"");
            return Some(ExitCode::FAILURE);
        }
    };
    Some(match outcome {
        Outcome::Done(code) => code,
        Outcome::Delegate => delegate_to_node(&name, &rest, &stdin),
    })
}
