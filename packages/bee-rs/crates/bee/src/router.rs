// Routing table: which (group[, verb]) pairs are served natively.
//
// `bee rs-info` is the only native verb at R0 — a diagnostic that reports the
// runtime and the ported-verb list. It is deliberately outside the porcelain
// namespace so it can never collide with a Node-surface command.

use std::ffi::OsString;
use std::process::ExitCode;

pub enum Route {
    Native(NativeCmd),
    Delegate,
}

pub enum NativeCmd {
    RsInfo,
}

/// (group, verb) pairs served natively. A None verb means the whole group.
/// R3 flips entries into this table one at a time, each behind a green
/// diff-harness run.
pub const PORTED: &[(&str, Option<&str>)] = &[];

pub fn route(args: &[OsString]) -> Route {
    let first = args.first().and_then(|a| a.to_str());
    if first == Some("rs-info") {
        return Route::Native(NativeCmd::RsInfo);
    }

    let group = match first {
        Some(g) if !g.starts_with('-') => g,
        _ => return Route::Delegate,
    };
    let verb = args
        .get(1)
        .and_then(|a| a.to_str())
        .filter(|v| !v.starts_with('-'));

    let ported = PORTED.iter().any(|(g, v)| {
        *g == group && (v.is_none() || v.as_deref() == verb)
    });
    if ported {
        unreachable!("PORTED entry has no NativeCmd mapping yet");
    }
    Route::Delegate
}

pub fn run_native(cmd: NativeCmd, _args: &[OsString]) -> ExitCode {
    match cmd {
        NativeCmd::RsInfo => {
            let ported: Vec<String> = PORTED
                .iter()
                .map(|(g, v)| match v {
                    Some(v) => format!("{g} {v}"),
                    None => (*g).to_string(),
                })
                .collect();
            let info = serde_json::json!({
                "runtime": "rust",
                "version": env!("CARGO_PKG_VERSION"),
                "ported": ported,
                "fallback": "node bee.mjs",
            });
            println!("{}", serde_json::to_string_pretty(&info).unwrap());
            ExitCode::SUCCESS
        }
    }
}
