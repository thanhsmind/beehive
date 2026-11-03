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
const PIN: &str = "ease-design@0.6.2";
const OLD_FAMILIES: [&str; 6] = ["a11y", "autofix", "content", "layout", "taste", "tell"];
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
    artifacts: Option<Map<String, Value>>,
}

impl Run {
    fn stop(code: u8, status: &'static str, line: String) -> Run {
        Run { code, status, lines: vec![line], pages: Map::new(), artifacts: None }
    }
}

fn output(run: &Run) -> Value {
    let mut out = json!({"status": run.status, "pages": run.pages, "lines": run.lines});
    if let Some(artifacts) = &run.artifacts {
        out["artifacts"] = Value::Object(artifacts.clone());
    }
    out
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
        println!("{}", jsjson::stringify_pretty(&output(&run)));
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

fn files(work: &Path) -> Vec<String> {
    let mut found = Vec::new();
    walk(work, "", &mut found);
    found.sort();
    found
}

fn pages(files: &[String], config_root: &Path) -> Vec<String> {
    let config = crate::state::read_config_raw(config_root);
    let globs: Vec<_> = match config.get("design").and_then(|d| d.get("pages")).and_then(Value::as_array) {
        Some(list) => list.iter().filter_map(Value::as_str).map(glob_tokens).collect(),
        None => vec![glob_tokens("**/*.html")],
    };
    files
        .iter()
        .filter(|page| {
            let chars: Vec<char> = page.chars().collect();
            globs.iter().any(|g| glob_match(g, &chars))
        })
        .cloned()
        .collect()
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

fn ui<T>(work: &Path, tools: &Tools, verb: &str, args: &[&str], subject: &str, read: impl FnOnce(&Value) -> Option<T>) -> Result<T, String> {
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
        .args(verb.split(' '))
        .args(args)
        .current_dir(work)
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("design gate: {subject} — ui {verb} did not start: {e}"))?;
    serde_json::from_slice::<Value>(&out.stdout)
        .ok()
        .as_ref()
        .and_then(|parsed| parsed.get("data"))
        .and_then(read)
        .ok_or_else(|| format!("design gate: {subject} — ui {verb} output was not the expected JSON"))
}

fn score(work: &Path, tools: &Tools, page: &str, counted: Option<&[String]>) -> Result<(Value, Vec<String>), String> {
    ui(work, tools, "gate", &[page, "--dir", ".", "--json"], page, |data| {
        let total = data.get("errorCount")?.as_u64()?;
        let warnings = data.get("warningCount")?.as_u64()?;
        let coverage = data.pointer("/tokenCoverage/overall/coverage").and_then(Value::as_f64).unwrap_or(0.0);
        let families = data.get("families").and_then(Value::as_object);
        let errors = match families {
            Some(families) => families
                .iter()
                .filter(|(name, _)| counted.is_none_or(|c| c.contains(name)))
                .map(|(_, family)| family.get("errorCount")?.as_u64())
                .sum::<Option<u64>>()?,
            None => total,
        };
        let names = families.map(|f| f.keys().cloned().collect()).unwrap_or_default();
        Some((json!({"errors": errors, "warnings": warnings, "coverage": (coverage * 10000.0).round() / 10000.0}), names))
    })
}

fn markers(work: &Path, page: &str) -> Vec<(&'static str, &'static str)> {
    let text = String::from_utf8_lossy(&std::fs::read(work.join(page)).unwrap_or_default()).into_owned();
    let scrub = text.match_indices("class=").any(|(at, m)| {
        let rest = &text[at + m.len()..];
        let Some(quote) = rest.chars().next().filter(|q| *q == '"' || *q == '\'') else { return false };
        text[..at].ends_with(char::is_whitespace) && rest[1..].split(quote).next().is_some_and(|v| v.split_whitespace().any(|t| t == "scrub"))
    });
    [
        (text.contains(r#"data-chart-owned="true""#), ("chart", "chart lint")),
        (text.contains(r#"data-diagram-owned="true""#), ("diagram", "diagram lint")),
        (scrub, ("motion", "tenant-lint")),
    ]
    .into_iter()
    .filter_map(|(hit, kind)| hit.then_some(kind))
    .collect()
}

fn lint(work: &Path, tools: &Tools, kind: &str, verb: &str, path: &str) -> Result<Value, String> {
    ui(work, tools, verb, &[path, "--json"], &format!("{kind} {path}"), |data| {
        let errors = data.get("errorCount")?.as_u64()?;
        let warnings = data.get("warningCount").map_or(Some(0), Value::as_u64)?;
        Some(json!({"errors": errors, "warnings": warnings}))
    })
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
    let base = match read_json(&work.join(BASELINE)) {
        ReadJson::Missing => Value::Null,
        ReadJson::Parsed(v) => v,
        ReadJson::Corrupt => return Run::stop(1, "ran", format!("design gate: {BASELINE} is not valid JSON")),
    };
    let known: Option<Vec<String>> = (!base.is_null()).then(|| match base["families"].as_array() {
        Some(list) => list.iter().filter_map(Value::as_str).map(str::to_string).collect(),
        None => OLD_FAMILIES.map(str::to_string).to_vec(),
    });
    let lenient = !base.is_null() && base.get("artifacts").is_none();
    let all = files(work);
    let found = pages(&all, config_root);
    let mut run = Run { code: 0, status: "ran", lines: Vec::new(), pages: Map::new(), artifacts: None };
    let mut families = std::collections::BTreeSet::new();
    for page in &found {
        match score(work, tools, page, known.as_deref().filter(|_| !write)) {
            Err(line) => run.lines.push(line),
            Ok((now, names)) => {
                families.extend(names);
                if !write {
                    run.lines.extend(regression(page, &now, base["pages"].get(page.as_str())));
                }
                run.pages.insert(page.clone(), now);
            }
        }
    }
    let flows = all.iter().filter(|f| f.rsplit('/').next().is_some_and(|n| n == "flow.json" || n.ends_with(".flow.json")));
    let targets: Vec<(&str, &str, &str)> = found
        .iter()
        .flat_map(|page| markers(work, page).into_iter().map(move |(kind, verb)| (kind, verb, page.as_str())))
        .chain(flows.map(|f| ("flow", "flow lint", f.as_str())))
        .collect();
    let (mut artifacts, mut uncounted) = (Map::new(), Vec::new());
    for &(kind, verb, path) in &targets {
        match lint(work, tools, kind, verb, path) {
            Err(line) => run.lines.push(line),
            Ok(now) => {
                let errors = now["errors"].as_u64().unwrap_or(0);
                if lenient && errors > 0 {
                    uncounted.push(format!("{kind} {path}"));
                } else if !lenient && !write {
                    run.lines.extend(match base["artifacts"][kind][path]["errors"].as_u64() {
                        Some(was) => (errors > was).then(|| format!("design gate: {kind} {path} — errors rose {was} -> {errors}")),
                        None => (errors > 0).then(|| format!("design gate: {kind} {path} — new {kind} has {errors} errors, must have 0")),
                    });
                }
                if let Value::Object(by_path) = artifacts.entry(kind).or_insert_with(|| json!({})) {
                    by_path.insert(path.to_string(), now);
                }
            }
        }
    }
    let new_families: Vec<&String> = families.iter().filter(|f| known.as_ref().is_some_and(|k| !k.contains(f))).collect();
    let advisory: Vec<String> = [
        (!new_families.is_empty()).then(|| format!("new families {}", new_families.iter().map(|f| f.as_str()).collect::<Vec<_>>().join(", "))),
        (!uncounted.is_empty()).then(|| format!("artifacts with errors {}", uncounted.join(", "))),
    ]
    .into_iter()
    .flatten()
    .collect();
    let (failed, total, linted) = (run.lines.len(), found.len(), targets.len());
    let written = if write && failed == 0 {
        record(&work.join(BASELINE), &json!({"version": PIN, "families": families, "pages": run.pages, "artifacts": artifacts}))
    } else {
        Ok(())
    };
    if linted > 0 {
        run.artifacts = Some(artifacts);
    }
    let tracked = record(&last_run, &json!({"status": "ran", "pages": total, "regressions": failed}));
    let s = |n: usize| if n == 1 { "" } else { "s" };
    let (scope, unit) = match linted {
        0 => (format!("{total} page{}", s(total)), "page"),
        _ => (format!("{total} page{} and {linted} artifact{}", s(total), s(linted)), "check"),
    };
    let summary = match (write, failed) {
        (true, 0) => written.is_ok().then(|| format!("design gate: baseline written for {scope} to {BASELINE}")),
        (true, _) => Some(format!("design gate: baseline not written — {failed} {unit}{} failed", s(failed))),
        (false, 0) => Some(format!("design gate: no regressions on {scope}")),
        (false, _) => None,
    };
    let broken: Vec<String> = [written.err(), tracked.err()].into_iter().flatten().collect();
    run.code = u8::from(failed > 0 || !broken.is_empty());
    if !advisory.is_empty() {
        run.lines.push(format!("design gate: {} — not counted until a person re-records with bee design gate --write", advisory.join("; ")));
    }
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
        put(
            &cli,
            &format!(
                "echo \"$@\" >> '{d}/calls'\ncase \"$1\" in gate|tenant-lint) k=\"$1/$2\";; *) k=\"$1-$2/$3\";; esac\ncat '{d}/canned/'\"$k\"\nexit 1\n",
                d = dir.path().display()
            ),
        );
        let tools = Tools { node: node.into_os_string(), cli: Some(cli.into_os_string()) };
        Fx { dir, root, tools }
    }

    impl Fx {
        fn page(&self, page: &str, output: &str) {
            put(&self.root.join(page), "<!doctype html>");
            put(&self.dir.path().join("canned/gate").join(page), output);
        }

        fn marked(&self, page: &str, html: &str, gate: &str) {
            self.page(page, gate);
            put(&self.root.join(page), html);
        }

        fn lint(&self, verb: &str, path: &str, output: &str) {
            put(&self.dir.path().join("canned").join(verb).join(path), output);
        }

        fn baseline_raw(&self, value: Value) {
            put(&self.baseline_path(), &value.to_string());
        }

        fn base(&self) -> Value {
            serde_json::from_str(&std::fs::read_to_string(self.baseline_path()).unwrap()).unwrap()
        }

        fn calls(&self) -> String {
            std::fs::read_to_string(self.dir.path().join("calls")).unwrap_or_default()
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

    fn gated(families: &[(&str, u64)]) -> String {
        let fams: Map<String, Value> = families
            .iter()
            .map(|(name, errors)| (name.to_string(), json!({"errorCount": errors, "warningCount": 0, "advisoryCount": 0, "findings": []})))
            .collect();
        let total: u64 = families.iter().map(|(_, e)| e).sum();
        json!({"ok": true, "data": {"errorCount": total, "warningCount": 0, "families": fams, "tokenCoverage": {"overall": {"coverage": 0.5}}}})
            .to_string()
    }

    fn linted(errors: u64) -> String {
        json!({"ok": true, "data": {"file": "x", "findings": [], "errorCount": errors, "warningCount": 2}}).to_string()
    }

    fn page_entry(errors: u64) -> Value {
        json!({"errors": errors, "warnings": 0, "coverage": 0.5})
    }

    fn lines(list: &[&str]) -> Vec<String> {
        list.iter().map(|l| l.to_string()).collect()
    }

    const CHART: &str = r#"<div data-chart-owned="true"><svg></svg></div>"#;
    const ADVICE: &str = "bee design gate --write";

    #[test]
    fn the_pin_is_ease_design_0_6_2() {
        assert_eq!(PIN, "ease-design@0.6.2");
    }

    #[test]
    fn an_old_baseline_counts_the_old_families_and_names_ds_usage_once() {
        let f = fx("v22.3.0");
        for page in ["a.html", "b.html"] {
            f.page(page, &gated(&[("a11y", 1), ("layout", 0), ("tell", 0), ("ds-usage", 330)]));
        }
        f.baseline(json!({"a.html": page_entry(1), "b.html": page_entry(1)}));
        let run = f.run(false);
        assert_eq!((run.code, run.lines.len()), (0, 2), "{:?}", run.lines);
        assert!(run.lines[0].contains("ds-usage") && run.lines[0].contains(ADVICE), "{:?}", run.lines);
        assert_eq!(run.lines[1], "design gate: no regressions on 2 pages");
        assert_eq!(f.last_run(), Some(json!({"status": "ran", "pages": 2, "regressions": 0})));
    }

    #[test]
    fn a_baseline_that_lists_ds_usage_fails_when_ds_usage_rose() {
        let f = fx("v22.3.0");
        f.page("a.html", &gated(&[("a11y", 0), ("ds-usage", 5)]));
        f.baseline_raw(json!({"version": PIN, "families": ["a11y", "ds-usage"], "pages": {"a.html": page_entry(2)}, "artifacts": {}}));
        let run = f.run(false);
        assert_eq!((run.code, run.lines), (1, lines(&["design gate: a.html — errors rose 2 -> 5"])));
    }

    #[test]
    fn write_records_the_pin_the_sorted_families_and_an_artifacts_map() {
        let f = fx("v22.3.0");
        f.page("a.html", &gated(&[("tell", 1), ("ds-usage", 2), ("a11y", 0)]));
        f.page("b.html", &gated(&[("layout", 0)]));
        f.baseline(json!({}));
        let run = f.run(true);
        assert_eq!((run.code, run.lines.len()), (0, 2), "{:?}", run.lines);
        assert!(run.lines[0].contains("ds-usage") && run.lines[0].contains(ADVICE), "{:?}", run.lines);
        assert_eq!(run.lines[1], "design gate: baseline written for 2 pages to .bee/design/gate-baseline.json");
        let base = f.base();
        assert_eq!(base["version"], "ease-design@0.6.2", "{base}");
        assert_eq!(base["families"], json!(["a11y", "ds-usage", "layout", "tell"]), "{base}");
        assert_eq!(base["pages"]["a.html"]["errors"], 3, "{base}");
        assert_eq!(base["artifacts"], json!({}), "{base}");
    }

    #[test]
    fn a_marked_chart_page_whose_lint_errors_rose_exits_one_naming_the_page_and_the_counts() {
        let f = fx("v22.3.0");
        f.marked("index.html", CHART, &gated(&[("a11y", 0)]));
        f.lint("chart-lint", "index.html", &linted(3));
        f.baseline_raw(json!({
            "version": PIN, "families": ["a11y"], "pages": {"index.html": page_entry(0)},
            "artifacts": {"chart": {"index.html": {"errors": 1, "warnings": 0}}}
        }));
        let run = f.run(false);
        assert_eq!((run.code, run.lines), (1, lines(&["design gate: chart index.html — errors rose 1 -> 3"])));
        assert_eq!(f.last_run(), Some(json!({"status": "ran", "pages": 1, "regressions": 1})));
    }

    #[test]
    fn a_flow_file_with_errors_and_no_baseline_entry_exits_one() {
        let f = fx("v22.3.0");
        put(&f.root.join(".bee/config.json"), r#"{"design": {"pages": ["site/*.html"]}}"#);
        f.scored("site/index.html", 0, 0.5);
        for flow in ["flow.json", "models/checkout.flow.json", "node_modules/x/flow.json", "models/flowchart.json"] {
            put(&f.root.join(flow), "{}");
        }
        f.lint("flow-lint", "flow.json", &linted(0));
        f.lint("flow-lint", "models/checkout.flow.json", &linted(2));
        let run = f.run(false);
        assert_eq!(
            (run.code, &run.lines),
            (1, &lines(&["design gate: flow models/checkout.flow.json — new flow has 2 errors, must have 0"]))
        );
        let flows = &run.artifacts.as_ref().expect("artifacts were found")["flow"];
        assert_eq!(flows.as_object().unwrap().keys().collect::<Vec<_>>(), vec!["flow.json", "models/checkout.flow.json"]);
    }

    #[test]
    fn a_project_with_no_artifacts_prints_the_same_lines_json_and_pages_as_before() {
        let f = fx("v22.3.0");
        f.scored("index.html", 2, 0.6);
        f.baseline(json!({"index.html": entry(2, 0.6)}));
        let run = f.run(false);
        let said = lines(&["design gate: no regressions on 1 page"]);
        assert_eq!((run.code, &run.lines), (0, &said));
        assert_eq!(output(&run), json!({"status": "ran", "pages": {"index.html": entry(2, 0.6)}, "lines": said}));
        let run = f.run(true);
        assert_eq!(run.lines, lines(&["design gate: baseline written for 1 page to .bee/design/gate-baseline.json"]));
        assert_eq!(f.base()["pages"], json!({"index.html": entry(2, 0.6)}));
        assert!(f.calls().lines().all(|call| call.starts_with("gate ")), "{}", f.calls());
    }

    #[test]
    fn the_advisory_never_counts_as_a_regression_and_never_blocks_write() {
        let f = fx("v22.3.0");
        f.marked("index.html", CHART, &gated(&[("a11y", 0), ("ds-usage", 7)]));
        f.lint("chart-lint", "index.html", &linted(4));
        f.baseline(json!({"index.html": page_entry(0)}));
        let run = f.run(false);
        assert_eq!((run.code, run.lines.len()), (0, 2), "{:?}", run.lines);
        let advice = run.lines[0].clone();
        assert!(advice.contains("ds-usage") && advice.contains("chart index.html") && advice.contains(ADVICE), "{advice}");
        assert_eq!(run.lines[1], "design gate: no regressions on 1 page and 1 artifact");
        assert_eq!(f.last_run(), Some(json!({"status": "ran", "pages": 1, "regressions": 0})));
        let run = f.run(true);
        assert_eq!(run.code, 0, "{:?}", run.lines);
        assert_eq!(
            run.lines,
            vec![advice, "design gate: baseline written for 1 page and 1 artifact to .bee/design/gate-baseline.json".to_string()]
        );
        assert_eq!(f.base()["artifacts"], json!({"chart": {"index.html": {"errors": 4, "warnings": 2}}}));
        assert_eq!(f.last_run(), Some(json!({"status": "ran", "pages": 1, "regressions": 0})));
    }

    #[test]
    fn a_baseline_with_no_artifacts_key_names_a_marked_chart_with_errors_and_exits_zero() {
        let f = fx("v22.3.0");
        f.marked("index.html", CHART, &gated(&[("a11y", 0)]));
        f.lint("chart-lint", "index.html", &linted(3));
        f.baseline_raw(json!({"version": PIN, "families": ["a11y"], "pages": {"index.html": page_entry(0)}}));
        let run = f.run(false);
        assert_eq!((run.code, run.lines.len()), (0, 2), "{:?}", run.lines);
        assert!(run.lines[0].contains("chart index.html") && run.lines[0].contains(ADVICE), "{:?}", run.lines);
        assert!(!run.lines[0].contains("families"), "{:?}", run.lines);
    }

    #[test]
    fn each_marker_runs_its_own_linter_and_scrub_lint_and_scroll_story_never_run() {
        let f = fx("v22.3.0");
        let zero = gated(&[("a11y", 0)]);
        f.marked("a.html", CHART, &zero);
        f.marked("b.html", r#"<div data-diagram-owned="true"></div>"#, &zero);
        f.marked("c.html", r#"<section class="hero scrub"><video></video></section>"#, &zero);
        f.marked("d.html", r#"<p data-class="scrub" class="scrubber">x</p>"#, &zero);
        f.marked("e.html", r#"<div data-chart-owned="false"></div>"#, &zero);
        f.lint("chart-lint", "a.html", &linted(0));
        f.lint("diagram-lint", "b.html", &linted(1));
        f.lint("tenant-lint", "c.html", &json!({"ok": true, "data": {"file": "c.html", "errorCount": 0, "findings": []}}).to_string());
        f.baseline_raw(json!({
            "version": PIN, "families": ["a11y"], "pages": {},
            "artifacts": {"diagram": {"b.html": {"errors": 1, "warnings": 0}}}
        }));
        let run = f.run(false);
        assert_eq!((run.code, &run.lines), (0, &lines(&["design gate: no regressions on 5 pages and 3 artifacts"])));
        assert_eq!(
            output(&run)["artifacts"],
            json!({
                "chart": {"a.html": {"errors": 0, "warnings": 2}},
                "diagram": {"b.html": {"errors": 1, "warnings": 2}},
                "motion": {"c.html": {"errors": 0, "warnings": 0}}
            })
        );
        let calls = f.calls();
        assert!(calls.contains("tenant-lint c.html --json"), "{calls}");
        assert!(!calls.contains("lint d.html") && !calls.contains("lint e.html"), "{calls}");
        assert!(!calls.contains("scrub-lint") && !calls.contains("scroll-story"), "{calls}");
    }

    #[test]
    fn a_linter_that_returns_other_json_is_a_failure_naming_the_kind_and_path() {
        let f = fx("v22.3.0");
        f.marked("index.html", CHART, &gated(&[("a11y", 0)]));
        f.lint("chart-lint", "index.html", "Error: the linter crashed");
        put(&f.root.join("flow.json"), "{}");
        f.lint("flow-lint", "flow.json", r#"{"data": {"warningCount": 0}}"#);
        let run = f.run(false);
        assert_eq!(
            (run.code, run.lines),
            (
                1,
                lines(&[
                    "design gate: chart index.html — ui chart lint output was not the expected JSON",
                    "design gate: flow flow.json — ui flow lint output was not the expected JSON",
                ])
            )
        );
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
        assert_eq!(pages(&files(&f.root), &f.root), vec!["docs/guide.html", "index.html"]);
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
