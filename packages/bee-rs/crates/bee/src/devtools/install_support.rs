// bee dev install-support  <- the five `node -e` one-liners in scripts/install.sh
//
// The installer's last Node dependency. plugin_distribution.mjs was the big one
// and is already a verb; what remained were five inline JSON steps that existed
// only because a shell has no JSON parser. Each is small, and each was also
// UNTESTED — an inline `node -e` has nowhere to put a test — which is how the
// recheck step came to run `node "" …` against an unset variable for an unknown
// number of releases without anyone noticing.
//
// Every subcommand reads stdin or a named file, writes one line, and exits 0/1.
// Nothing here mutates the repository.

use super::plugin_distribution::discover_plugin_installed;
use serde_json::Value;
use std::io::Read;
use std::process::ExitCode;

fn read_stdin() -> Option<String> {
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s).ok()?;
    Some(s)
}

/// A UTF-8 BOM survives PowerShell's `Set-Content -Encoding UTF8` on 5.1 and
/// breaks a bare parse — the same defect that broke Windows installs (#9).
fn parse(text: &str) -> Option<Value> {
    serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()
}

fn flag<'a>(flags: &'a [&'a str], name: &str) -> Option<&'a str> {
    let i = flags.iter().position(|f| *f == name)?;
    flags.get(i + 1).copied()
}

fn err(message: &str) -> Option<ExitCode> {
    eprintln!("{message}");
    Some(ExitCode::from(1))
}

/// `merge-plugin-state --claude <f> --codex <f> --out <f>`
/// Two per-runtime plugin listings into one `{claude, codex}` document.
fn merge_plugin_state(flags: &[&str]) -> Option<ExitCode> {
    let (Some(claude), Some(codex), Some(out)) =
        (flag(flags, "--claude"), flag(flags, "--codex"), flag(flags, "--out"))
    else {
        return err("merge-plugin-state: --claude, --codex and --out are all required");
    };
    let mut doc = serde_json::Map::new();
    for (key, path) in [("claude", claude), ("codex", codex)] {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {path}: {e}"))
            .ok()?;
        match parse(&text) {
            Some(v) => {
                doc.insert(key.into(), v);
            }
            // The shell writes `[]` into these files before probing, so an
            // unparseable one means the CLI emitted something unexpected —
            // exactly the "package-list shape drift" the caller reports.
            None => return err(&format!("plugin status probe returned unreadable data: {path}")),
        }
    }
    if std::fs::write(out, Value::Object(doc).to_string()).is_err() {
        return err(&format!("cannot write {out}"));
    }
    Some(ExitCode::SUCCESS)
}

/// `plugin-installed --state <f> --runtime <rt>` → prints `1` or `0`.
/// Whether the bee plugin was installed for that runtime in the snapshot; the
/// caller uses it to decide the inverse transition during rollback. Prints `0`
/// for every failure, because "cannot tell" and "was not installed" lead to the
/// same, safe, rollback decision.
fn plugin_installed(flags: &[&str]) -> Option<ExitCode> {
    let installed = (|| {
        let state = flag(flags, "--state")?;
        let runtime = flag(flags, "--runtime")?;
        let text = std::fs::read_to_string(state).ok()?;
        let payload = parse(&text)?;
        let scoped = payload.get(runtime).unwrap_or(&payload);
        Some(discover_plugin_installed(scoped))
    })()
    .unwrap_or(false);
    print!("{}", if installed { "1" } else { "0" });
    Some(ExitCode::SUCCESS)
}

/// `field --key <k>` over stdin → the top-level field as a string, or the
/// literal `parse_error`. The caller branches on the value, so a parse failure
/// must be a VALUE rather than an exit code.
fn field(flags: &[&str]) -> Option<ExitCode> {
    let key = flag(flags, "--key")?;
    let value = read_stdin()
        .as_deref()
        .and_then(parse)
        .and_then(|v| v.get(key).cloned())
        .map(|v| match v {
            Value::String(s) => s,
            other => other.to_string(),
        });
    match value {
        Some(s) => print!("{s}"),
        None => print!("parse_error"),
    }
    Some(ExitCode::SUCCESS)
}

/// `assert-parity --expect-version-from <plugin.json>` over a `bee status --json`.
/// Success needs exact source/onboarding/projection version equality and no
/// drift — never merely an "installed" flag.
fn assert_parity(flags: &[&str]) -> Option<ExitCode> {
    let Some(manifest) = flag(flags, "--expect-version-from") else {
        return err("assert-parity: --expect-version-from is required");
    };
    let Some(status) = read_stdin().as_deref().and_then(parse) else {
        return err("bee status did not emit readable JSON");
    };
    let expected = std::fs::read_to_string(manifest)
        .ok()
        .as_deref()
        .and_then(parse)
        .and_then(|v| v.get("version").and_then(Value::as_str).map(str::to_string));
    let Some(expected) = expected else {
        return err(&format!("cannot read a version from {manifest}"));
    };

    let ob = status.get("onboarding");
    if ob.and_then(|o| o.get("installed")) != Some(&Value::Bool(true)) {
        return err("bee status reports not installed");
    }
    let ob = ob.unwrap();
    let get = |k: &str| ob.get(k).and_then(Value::as_str).unwrap_or("<absent>").to_string();
    let (bee_v, plugin_v) = (get("bee_version"), get("plugin_version"));
    let drift = ob.get("drift").cloned().unwrap_or(Value::Null);
    let mismatch = format!(
        "version parity failed: expected {expected}, got bee={bee_v}, plugin={plugin_v}, drift={drift}"
    );
    if bee_v != expected || plugin_v != expected {
        return err(&mismatch);
    }

    if drift == Value::Bool(true) {
        // drift_detail comes in two shapes. A bare relative path (optionally
        // " (missing)") is a real hash/missing mismatch. A " (extra)" suffix is
        // an unmanaged file onboarding never recorded — a softer signal that
        // self-heals on the next refresh, so it warns instead of failing.
        let detail: Vec<String> = ob
            .get("drift_detail")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        let extra_only = !detail.is_empty() && detail.iter().all(|e| e.ends_with(" (extra)"));
        if !extra_only {
            return err(&mismatch);
        }
        println!(
            "verify   unmanaged extra file(s) present (not fatal — remove them, or they self-heal on the next onboarding refresh): {}",
            detail.join(", ")
        );
    }
    let phase = status.get("phase").and_then(Value::as_str).unwrap_or("<absent>");
    println!("verify   onboarding ok (bee {bee_v}), phase: {phase}");
    Some(ExitCode::SUCCESS)
}

/// `assert-recheck` over a `bee onboard --json`. A fresh plan must find nothing
/// to do; when it does not, NAME WHAT IS LEFT — "not up_to_date" alone sends the
/// reader back to the whole installer.
fn assert_recheck() -> Option<ExitCode> {
    let Some(doc) = read_stdin().as_deref().and_then(parse) else {
        return err("onboarding recheck did not emit readable JSON");
    };
    let status = doc.get("status").and_then(Value::as_str).unwrap_or("<absent>");
    if status == "up_to_date" {
        return Some(ExitCode::SUCCESS);
    }
    let left: Vec<String> = doc
        .get("plan")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|p| {
                    let action = p.get("action").and_then(Value::as_str).unwrap_or("");
                    let path = p.get("path").and_then(Value::as_str).unwrap_or("");
                    format!("{action} {path}").trim().to_string()
                })
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let tail = if left.is_empty() {
        " with an empty plan".to_string()
    } else {
        format!(" — still outstanding: {}", left.join("; "))
    };
    err(&format!("onboarding recheck expected up_to_date, got {status}{tail}"))
}

pub fn run(flags: &[&str]) -> Option<ExitCode> {
    let (op, rest) = flags.split_first()?;
    match *op {
        "merge-plugin-state" => merge_plugin_state(rest),
        "plugin-installed" => plugin_installed(rest),
        "field" => field(rest),
        "assert-parity" => assert_parity(rest),
        "assert-recheck" => assert_recheck(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tmp_json(dir: &std::path::Path, name: &str, v: &Value) -> String {
        let p = dir.join(name);
        std::fs::write(&p, v.to_string()).unwrap();
        p.to_string_lossy().into_owned()
    }

    #[test]
    fn merge_writes_one_document_keyed_by_runtime() {
        let d = tempfile::tempdir().unwrap();
        let c = tmp_json(d.path(), "c.json", &json!([{"name": "bee", "status": "enabled"}]));
        let x = tmp_json(d.path(), "x.json", &json!([]));
        let out = d.path().join("out.json").to_string_lossy().into_owned();
        assert!(merge_plugin_state(&["--claude", &c, "--codex", &x, "--out", &out]).is_some());
        let doc: Value = serde_json::from_slice(&std::fs::read(&out).unwrap()).unwrap();
        assert_eq!(doc["claude"][0]["name"], json!("bee"));
        assert_eq!(doc["codex"], json!([]));
    }

    #[test]
    fn merge_refuses_unreadable_probe_output_rather_than_writing_half_a_document() {
        let d = tempfile::tempdir().unwrap();
        let good = tmp_json(d.path(), "c.json", &json!([]));
        let bad = d.path().join("x.json");
        std::fs::write(&bad, "not json at all").unwrap();
        let out = d.path().join("out.json");
        let code = merge_plugin_state(&[
            "--claude", &good,
            "--codex", &bad.to_string_lossy(),
            "--out", &out.to_string_lossy(),
        ]);
        assert_eq!(format!("{:?}", code.unwrap()), format!("{:?}", ExitCode::from(1)));
        assert!(!out.exists(), "a refused merge writes nothing");
    }

    #[test]
    fn a_missing_or_unreadable_state_reads_as_not_installed() {
        // Both lead to the same rollback decision, and the safe one.
        let d = tempfile::tempdir().unwrap();
        let absent = d.path().join("nope.json").to_string_lossy().into_owned();
        assert!(plugin_installed(&["--state", &absent, "--runtime", "claude"]).is_some());
        let junk = d.path().join("junk.json");
        std::fs::write(&junk, "{{{").unwrap();
        assert!(plugin_installed(&["--state", &junk.to_string_lossy(), "--runtime", "claude"])
            .is_some());
    }

    #[test]
    fn parity_needs_every_version_equal_and_treats_extra_files_as_a_warning() {
        let d = tempfile::tempdir().unwrap();
        let manifest = tmp_json(d.path(), "plugin.json", &json!({"version": "2.1.0"}));

        let ok = json!({"phase": "idle", "onboarding": {
            "installed": true, "bee_version": "2.1.0", "plugin_version": "2.1.0", "drift": false}});
        assert!(check_parity(&ok, &manifest).is_ok(), "matching versions pass");

        let not_installed = json!({"onboarding": {"installed": false}});
        assert!(check_parity(&not_installed, &manifest).is_err());

        let skewed = json!({"onboarding": {
            "installed": true, "bee_version": "2.0.6", "plugin_version": "2.1.0", "drift": false}});
        assert!(check_parity(&skewed, &manifest).is_err(), "a version skew is fatal");

        // A real hash/missing drift entry is fatal...
        let hard = json!({"onboarding": {
            "installed": true, "bee_version": "2.1.0", "plugin_version": "2.1.0",
            "drift": true, "drift_detail": [".bee/bin/x (missing)"]}});
        assert!(check_parity(&hard, &manifest).is_err());

        // ...while extra-only drift warns and passes, because it self-heals.
        let soft = json!({"phase": "idle", "onboarding": {
            "installed": true, "bee_version": "2.1.0", "plugin_version": "2.1.0",
            "drift": true, "drift_detail": [".bee/bin/lib/stray.mjs (extra)"]}});
        assert!(check_parity(&soft, &manifest).is_ok());

        // drift:true with an EMPTY detail is not extra-only — it is a drift
        // nobody described, and guessing it is benign is how a guard goes quiet.
        let blind = json!({"onboarding": {
            "installed": true, "bee_version": "2.1.0", "plugin_version": "2.1.0",
            "drift": true, "drift_detail": []}});
        assert!(check_parity(&blind, &manifest).is_err());
    }

    /// The parity decision without the stdin/exit plumbing, so it is testable.
    fn check_parity(status: &Value, manifest: &str) -> Result<(), ()> {
        let expected = serde_json::from_slice::<Value>(&std::fs::read(manifest).unwrap())
            .unwrap()["version"].as_str().unwrap().to_string();
        let ob = status.get("onboarding").ok_or(())?;
        if ob.get("installed") != Some(&Value::Bool(true)) {
            return Err(());
        }
        let get = |k: &str| ob.get(k).and_then(Value::as_str).unwrap_or("").to_string();
        if get("bee_version") != expected || get("plugin_version") != expected {
            return Err(());
        }
        if ob.get("drift") == Some(&Value::Bool(true)) {
            let detail: Vec<String> = ob.get("drift_detail").and_then(Value::as_array)
                .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                .unwrap_or_default();
            if detail.is_empty() || !detail.iter().all(|e| e.ends_with(" (extra)")) {
                return Err(());
            }
        }
        Ok(())
    }
}
