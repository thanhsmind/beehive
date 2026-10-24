use crate::fsutil::{read_json, ReadJson};
use crate::hooks::write_guard::{glob_match, glob_tokens};
use crate::jsjson;
use crate::roots::{resolve_store_root_worktree, RootsWt};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Value};
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

const CMD: &str = "design gate";
const PIN: &str = "ease-design@0.6.1";
const NOT_RUN: &str = "design gate: not run — Node 20 or newer is missing";
const BASELINE: &str = ".bee/design/gate-baseline.json";
const SKIP_DIRS: [&str; 3] = ["node_modules", "target", "dist"];

struct Tools {
    node: OsString,
    cli: Option<OsString>,
}

struct Run {
    code: u8,
    status: &'static str,
    lines: Vec<String>,
    pages: Map<String, Value>,
}

impl Run {
    fn stop(code: u8, status: &'static str, line: String) -> Run {
        Run { code, status, lines: vec![line], pages: Map::new() }
    }
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "design" || args.get(1)?.to_str()? != "gate" {
        return None;
    }
    let (mut write, mut use_json) = (false, false);
    for arg in &args[2..] {
        match arg.to_str()? {
            "--write" => write = true,
            "--json" => use_json = true,
            _ => return None,
        }
    }
    let cwd = std::env::current_dir().ok()?;
    let roots = match resolve_store_root_worktree(&cwd) {
        RootsWt::Go(roots) => roots,
        RootsWt::Unsupported(why) => return Some(emit_unsupported_root(&cwd, CMD, use_json, t0, &why)),
        RootsWt::None => return Some(emit_no_root_error(&cwd, CMD, use_json, t0)),
    };
    let tools = Tools {
        node: std::env::var_os("BEE_DESIGN_NODE_BIN").unwrap_or_else(|| OsString::from("node")),
        cli: std::env::var_os("BEE_DESIGN_OS_CLI"),
    };
    let run = gate(&roots.work_root, &roots.root, &roots.main_root(), write, &tools);
    if use_json {
        let out = json!({"status": run.status, "pages": run.pages, "lines": run.lines});
        println!("{}", jsjson::stringify_pretty(&out));
    } else if run.code == 0 {
        println!("{}", run.lines.join("\n"));
    } else {
        eprintln!("{}", run.lines.join("\n"));
    }
    record_timing(&roots.root, CMD, t0, run.code == 0);
    Some(ExitCode::from(run.code))
}

fn node_ok(node: &OsStr) -> bool {
    let Ok(out) = Command::new(node).arg("--version").stdin(Stdio::null()).stderr(Stdio::null()).output() else {
        return false;
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let major = text.trim().trim_start_matches('v').split('.').next().and_then(|m| m.parse::<u32>().ok());
    out.status.success() && major.is_some_and(|m| m >= 20)
}

fn pages(work: &Path, config_root: &Path) -> Vec<String> {
    let config = crate::state::read_config_raw(config_root);
    let globs: Vec<_> = match config.get("design").and_then(|d| d.get("pages")).and_then(Value::as_array) {
        Some(list) => list.iter().filter_map(Value::as_str).map(glob_tokens).collect(),
        None => vec![glob_tokens("**/*.html")],
    };
    let mut found = Vec::new();
    walk(work, "", &mut found);
    found.retain(|page| {
        let chars: Vec<char> = page.chars().collect();
        globs.iter().any(|g| glob_match(g, &chars))
    });
    found.sort();
    found
}

fn walk(dir: &Path, prefix: &str, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let (Ok(kind), Some(name)) = (entry.file_type(), entry.file_name().to_str().map(str::to_string)) else {
            continue;
        };
        let rel = if prefix.is_empty() { name.clone() } else { format!("{prefix}/{name}") };
        if kind.is_dir() && !name.starts_with('.') && !SKIP_DIRS.contains(&name.as_str()) {
            walk(&entry.path(), &rel, out);
        } else if kind.is_file() {
            out.push(rel);
        }
    }
}

fn score(work: &Path, tools: &Tools, page: &str) -> Result<Value, String> {
    let mut cmd = match &tools.cli {
        Some(cli) => {
            let mut c = Command::new(&tools.node);
            c.arg(cli);
            c
        }
        None => {
            let mut c = Command::new("npx");
            c.args(["--yes", PIN]);
            c
        }
    };
    let out = cmd
        .args(["gate", page, "--dir", ".", "--json"])
        .current_dir(work)
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("design gate: {page} — ui gate did not start: {e}"))?;
    let malformed = || format!("design gate: {page} — ui gate output was not the expected JSON");
    let parsed: Value = serde_json::from_slice(&out.stdout).map_err(|_| malformed())?;
    let data = parsed.get("data").ok_or_else(malformed)?;
    let errors = data.get("errorCount").and_then(Value::as_u64).ok_or_else(malformed)?;
    let warnings = data.get("warningCount").and_then(Value::as_u64).ok_or_else(malformed)?;
    let coverage = data.pointer("/tokenCoverage/overall/coverage").and_then(Value::as_f64).unwrap_or(0.0);
    Ok(json!({"errors": errors, "warnings": warnings, "coverage": (coverage * 10000.0).round() / 10000.0}))
}

fn regression(page: &str, now: &Value, base: Option<&Value>) -> Option<String> {
    let errors = now["errors"].as_u64().unwrap_or(0);
    let Some((was, base)) = base.and_then(|b| Some((b["errors"].as_u64()?, b))) else {
        return (errors > 0).then(|| format!("design gate: {page} — new page has {errors} errors, must have 0"));
    };
    let coverage = now["coverage"].as_f64().unwrap_or(0.0);
    let was_coverage = base["coverage"].as_f64().unwrap_or(0.0);
    let mut found = Vec::new();
    if errors > was {
        found.push(format!("errors rose {was} -> {errors}"));
    }
    if coverage < was_coverage {
        found.push(format!("token coverage fell {was_coverage} -> {coverage}"));
    }
    (!found.is_empty()).then(|| format!("design gate: {page} — {}", found.join("; ")))
}

fn claimed(cells_root: &Path) -> Result<Vec<String>, String> {
    let cells = crate::verbs::cells::list_cells(cells_root, None, Some("claimed"))
        .map_err(|_| "design gate: --write refused, nothing written — the cell store could not be read".to_string())?;
    Ok(cells.iter().map(|c| jsjson::js_to_string(&c["id"])).collect())
}

fn record(path: &Path, value: &Value) -> Result<(), String> {
    let text = format!("{}\n", jsjson::stringify_pretty(value));
    std::fs::create_dir_all(path.parent().unwrap_or(path))
        .and_then(|_| std::fs::write(path, text))
        .map_err(|e| format!("design gate: could not write {}: {e}", path.display()))
}

fn gate(work: &Path, config_root: &Path, cells_root: &Path, write: bool, tools: &Tools) -> Run {
    if write {
        match claimed(cells_root) {
            Err(line) => return Run::stop(1, "refused", line),
            Ok(ids) if !ids.is_empty() => {
                let line = format!(
                    "design gate: --write refused, nothing written — claimed cells: {}. FIX: the baseline is a person's act; record it when no cell is claimed.",
                    ids.join(", ")
                );
                return Run::stop(1, "refused", line);
            }
            Ok(_) => {}
        }
    }
    let last_run = work.join(".bee/design/last-run.json");
    if !node_ok(&tools.node) {
        return match record(&last_run, &json!({"status": "not-run", "reason": "node-missing"})) {
            Ok(()) => Run::stop(0, "not-run", NOT_RUN.to_string()),
            Err(line) => Run::stop(1, "not-run", line),
        };
    }
    let baseline = match read_json(&work.join(BASELINE)) {
        ReadJson::Missing => Value::Null,
        ReadJson::Parsed(v) => v["pages"].clone(),
        ReadJson::Corrupt => return Run::stop(1, "ran", format!("design gate: {BASELINE} is not valid JSON")),
    };
    let found = pages(work, config_root);
    let mut run = Run { code: 0, status: "ran", lines: Vec::new(), pages: Map::new() };
    for page in &found {
        match score(work, tools, page) {
            Err(line) => run.lines.push(line),
            Ok(now) => {
                if !write {
                    run.lines.extend(regression(page, &now, baseline.get(page.as_str())));
                }
                run.pages.insert(page.clone(), now);
            }
        }
    }
    let (failed, total) = (run.lines.len(), found.len());
    let written = if write && failed == 0 {
        record(&work.join(BASELINE), &json!({"version": PIN, "pages": run.pages}))
    } else {
        Ok(())
    };
    let tracked = record(&last_run, &json!({"status": "ran", "pages": total, "regressions": failed}));
    let s = |n: usize| if n == 1 { "" } else { "s" };
    let summary = match (write, failed) {
        (true, 0) => written.is_ok().then(|| format!("design gate: baseline written for {total} page{} to {BASELINE}", s(total))),
        (true, _) => Some(format!("design gate: baseline not written — {failed} page{} failed", s(failed))),
        (false, 0) => Some(format!("design gate: no regressions on {total} page{}", s(total))),
        (false, _) => None,
    };
    let broken: Vec<String> = [written.err(), tracked.err()].into_iter().flatten().collect();
    run.code = u8::from(failed > 0 || !broken.is_empty());
    run.lines.extend(summary);
    run.lines.extend(broken);
    run
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use serde_json::json;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::process::Command;

    struct Fx {
        dir: tempfile::TempDir,
        root: PathBuf,
        tools: Tools,
    }

    fn put(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    fn fx(node_version: &str) -> Fx {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("repo");
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        let node = dir.path().join("fake-node");
        put(
            &node,
            &format!("#!/bin/sh\nif [ \"$1\" = --version ]; then echo {node_version}; exit 0; fi\nexec /bin/sh \"$@\"\n"),
        );
        std::fs::set_permissions(&node, std::fs::Permissions::from_mode(0o755)).unwrap();
        for _ in 0..100 {
            match Command::new(&node).arg("--version").output() {
                Err(e) if e.raw_os_error() == Some(libc::ETXTBSY) => std::thread::sleep(std::time::Duration::from_millis(10)),
                _ => break,
            }
        }
        let cli = dir.path().join("fake-cli");
        put(&cli, &format!("cat '{}/canned/'\"$2\"\nexit 1\n", dir.path().display()));
        let tools = Tools { node: node.into_os_string(), cli: Some(cli.into_os_string()) };
        Fx { dir, root, tools }
    }

    impl Fx {
        fn page(&self, page: &str, output: &str) {
            put(&self.root.join(page), "<!doctype html>");
            put(&self.dir.path().join("canned").join(page), output);
        }

        fn scored(&self, page: &str, errors: u64, coverage: f64) {
            let out = json!({"data": {"errorCount": errors, "warningCount": 1, "tokenCoverage": {"overall": {"coverage": coverage}}}});
            self.page(page, &out.to_string());
        }

        fn baseline(&self, pages: Value) {
            put(&self.baseline_path(), &json!({"version": "0.6.1", "pages": pages}).to_string());
        }

        fn baseline_path(&self) -> PathBuf {
            self.root.join(".bee/design/gate-baseline.json")
        }

        fn cell(&self, id: &str, status: &str) {
            put(&self.root.join(format!(".bee/cells/{id}.json")), &json!({"id": id, "status": status}).to_string());
        }

        fn last_run(&self) -> Option<Value> {
            let text = std::fs::read_to_string(self.root.join(".bee/design/last-run.json")).ok()?;
            Some(serde_json::from_str(&text).unwrap())
        }

        fn run(&self, write: bool) -> Run {
            gate(&self.root, &self.root, &self.root, write, &self.tools)
        }
    }

    fn entry(errors: u64, coverage: f64) -> Value {
        json!({"errors": errors, "warnings": 1, "coverage": coverage})
    }

    #[test]
    fn a_clean_run_exits_zero_with_one_summary_line() {
        let f = fx("v22.3.0");
        f.scored("index.html", 2, 0.61);
        f.scored("about.html", 0, 0.5);
        f.baseline(json!({"index.html": entry(2, 0.6), "about.html": entry(0, 0.5)}));
        let run = f.run(false);
        assert_eq!((run.code, run.lines.len()), (0, 1), "{:?}", run.lines);
        assert!(run.lines[0].contains("no regressions on 2 pages"), "{:?}", run.lines);
    }

    #[test]
    fn an_error_rise_exits_one_naming_the_page_and_the_counts() {
        let f = fx("v22.3.0");
        f.scored("index.html", 3, 0.6);
        f.scored("about.html", 0, 0.5);
        f.baseline(json!({"index.html": entry(2, 0.6), "about.html": entry(0, 0.5)}));
        let run = f.run(false);
        assert_eq!((run.code, run.lines.len()), (1, 1), "{:?}", run.lines);
        assert!(run.lines[0].contains("index.html") && run.lines[0].contains("errors rose 2 -> 3"), "{:?}", run.lines);
    }

    #[test]
    fn a_coverage_fall_exits_one_naming_the_page() {
        let f = fx("v22.3.0");
        f.scored("index.html", 2, 0.55);
        f.baseline(json!({"index.html": entry(2, 0.6)}));
        let run = f.run(false);
        assert_eq!(run.code, 1, "{:?}", run.lines);
        assert!(run.lines[0].contains("index.html") && run.lines[0].contains("coverage fell 0.6 -> 0.55"), "{:?}", run.lines);
    }

    #[test]
    fn a_new_page_with_errors_exits_one() {
        let f = fx("v22.3.0");
        f.scored("new.html", 3, 0.0);
        let run = f.run(false);
        assert_eq!(run.code, 1, "{:?}", run.lines);
        assert!(run.lines[0].contains("new.html") && run.lines[0].contains("new page has 3 errors"), "{:?}", run.lines);
    }

    #[test]
    fn a_new_page_with_zero_errors_exits_zero() {
        let f = fx("v22.3.0");
        f.scored("new.html", 0, 0.0);
        let run = f.run(false);
        assert_eq!((run.code, run.lines.len()), (0, 1), "{:?}", run.lines);
        assert!(run.lines[0].contains("no regressions on 1 page"), "{:?}", run.lines);
    }

    #[test]
    fn malformed_tool_output_exits_one_naming_the_page() {
        let f = fx("v22.3.0");
        f.page("bad.html", "Error: the linter crashed");
        f.page("odd.html", r#"{"data": {"warningCount": 1}}"#);
        let run = f.run(false);
        assert_eq!((run.code, run.lines.len()), (1, 2), "{:?}", run.lines);
        assert!(run.lines[0].contains("bad.html") && run.lines[0].contains("not the expected JSON"), "{:?}", run.lines);
        assert!(run.lines[1].contains("odd.html") && run.lines[1].contains("not the expected JSON"), "{:?}", run.lines);
    }

    #[test]
    fn write_with_no_claimed_cell_records_every_page() {
        let f = fx("v22.3.0");
        f.scored("index.html", 4, 0.123456);
        f.scored("about.html", 0, 1.0);
        f.cell("x-1", "open");
        f.cell("x-2", "capped");
        let run = f.run(true);
        assert_eq!(run.code, 0, "{:?}", run.lines);
        let text = std::fs::read_to_string(f.baseline_path()).expect("the baseline was written");
        let base: Value = serde_json::from_str(&text).unwrap();
        assert!(base["version"].is_string(), "{base}");
        assert_eq!(base["pages"].as_object().unwrap().len(), 2, "{base}");
        assert_eq!(base["pages"]["index.html"]["errors"], 4, "{base}");
        assert_eq!(base["pages"]["index.html"]["warnings"], 1, "{base}");
        assert_eq!(base["pages"]["index.html"]["coverage"].as_f64(), Some(0.1235), "{base}");
        assert_eq!(base["pages"]["about.html"]["coverage"].as_f64(), Some(1.0), "{base}");
    }

    #[test]
    fn write_refuses_and_writes_nothing_while_a_cell_is_claimed() {
        let f = fx("v22.3.0");
        f.scored("index.html", 9, 0.1);
        f.baseline(json!({"index.html": entry(2, 0.6)}));
        let before = std::fs::read(f.baseline_path()).unwrap();
        f.cell("dob-5", "claimed");
        f.cell("dob-6", "claimed");
        f.cell("dob-7", "open");
        let run = f.run(true);
        assert_eq!(run.code, 1, "{:?}", run.lines);
        let said = run.lines.join("\n");
        assert!(said.contains("dob-5") && said.contains("dob-6") && !said.contains("dob-7"), "{said}");
        assert_eq!(std::fs::read(f.baseline_path()).unwrap(), before);
        assert!(f.last_run().is_none());
    }

    #[test]
    fn node_missing_or_below_twenty_prints_the_exact_not_run_line_and_records_it() {
        let not_run = vec!["design gate: not run — Node 20 or newer is missing".to_string()];
        for version in ["v18.19.0", "banana"] {
            let f = fx(version);
            f.scored("index.html", 9, 0.0);
            let run = f.run(false);
            assert_eq!((run.code, &run.lines), (0, &not_run), "{version}");
            assert_eq!(f.last_run(), Some(json!({"status": "not-run", "reason": "node-missing"})), "{version}");
        }
        let mut f = fx("v22.3.0");
        f.tools.node = f.dir.path().join("no-such-node").into_os_string();
        f.scored("index.html", 9, 0.0);
        let run = f.run(false);
        assert_eq!((run.code, &run.lines), (0, &not_run));
        assert_eq!(f.last_run(), Some(json!({"status": "not-run", "reason": "node-missing"})));
    }

    #[test]
    fn a_run_that_ran_records_ran_with_its_counts() {
        let f = fx("v20.0.0");
        f.scored("a.html", 0, 0.0);
        f.scored("b.html", 2, 0.0);
        let run = f.run(false);
        assert_eq!(run.code, 1, "{:?}", run.lines);
        assert_eq!(f.last_run(), Some(json!({"status": "ran", "pages": 2, "regressions": 1})));
    }

    #[test]
    fn design_pages_globs_limit_the_pages() {
        let f = fx("v22.3.0");
        f.scored("site/index.html", 0, 0.0);
        f.scored("other.html", 5, 0.0);
        put(&f.root.join(".bee/config.json"), r#"{"design": {"pages": ["site/*.html"]}}"#);
        let run = f.run(false);
        assert_eq!(run.code, 0, "{:?}", run.lines);
        assert_eq!(run.pages.keys().collect::<Vec<_>>(), vec!["site/index.html"]);
    }

    #[test]
    fn the_default_pages_skip_hidden_dirs_dependencies_and_build_output() {
        let f = fx("v22.3.0");
        for page in ["index.html", "docs/guide.html", ".hidden/x.html", "node_modules/p/x.html", "target/x.html", "dist/x.html", "notes.txt"] {
            put(&f.root.join(page), "x");
        }
        assert_eq!(pages(&f.root, &f.root), vec!["docs/guide.html", "index.html"]);
    }

    #[test]
    fn bee_design_gate_routes_and_reads_its_env_overrides_end_to_end() {
        let mut exe = std::env::current_exe().unwrap();
        exe.pop();
        if exe.ends_with("deps") {
            exe.pop();
        }
        let bin = exe.join("bee");
        assert!(bin.is_file(), "built bee binary not found at {}", bin.display());
        let f = fx("v20.1.0");
        put(&f.root.join(".bee/onboarding.json"), "{}");
        f.scored("index.html", 0, 0.9);
        let out = Command::new(&bin)
            .args(["design", "gate", "--json"])
            .env("BEE_DESIGN_NODE_BIN", &f.tools.node)
            .env("BEE_DESIGN_OS_CLI", f.tools.cli.as_ref().unwrap())
            .current_dir(&f.root)
            .output()
            .unwrap();
        let said = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        assert!(out.status.success(), "{said}");
        let v: Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| panic!("{e}: {said}"));
        assert_eq!((v["status"].as_str(), v["pages"]["index.html"]["errors"].as_u64()), (Some("ran"), Some(0)), "{v}");
    }
}
