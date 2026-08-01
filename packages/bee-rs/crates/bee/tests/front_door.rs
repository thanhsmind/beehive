// Black-box tests for the R0 front door: native rs-info, and verbatim
// delegation to node bee.mjs (skipped loudly if node is absent, matching the
// repo's env-limited-tests-skip-loudly convention).

use assert_cmd::Command;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    // crates/bee -> crates -> bee-rs -> packages -> beehive
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .unwrap()
        .to_path_buf()
}

fn js_entry() -> PathBuf {
    repo_root().join("packages").join("bee").join("bee.mjs")
}

fn node_available() -> bool {
    std::process::Command::new("node")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn rs_info_reports_rust_runtime() {
    let out = Command::cargo_bin("bee")
        .unwrap()
        .arg("rs-info")
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["runtime"], "rust");
    assert!(v["ported"].as_array().is_some());
}

#[test]
fn embedded_registry_payload_is_fresh() {
    if !node_available() {
        eprintln!("SKIP (env-limited: node not on PATH) embedded_registry_payload_is_fresh");
        return;
    }
    let root = repo_root();
    let embedded = std::fs::read_to_string(
        root.join("packages/bee-rs/crates/bee/src/generated/registry_payload.json"),
    )
    .expect("embedded payload file");
    let script = "import('./packages/bee/lib/command-registry.mjs').then(m => process.stdout.write(JSON.stringify({ schema_version: m.SCHEMA_VERSION, commands: m.COMMAND_REGISTRY })))";
    let out = std::process::Command::new("node")
        .args(["-e", script])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(out.status.success(), "node payload export failed: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        embedded,
        "registry payload is stale — re-run: node scripts/export_registry_payload.mjs, then rebuild bee-rs"
    );
}

// For unported argv shapes this exercises delegation; for `status --brief`
// it pits the NATIVE Rust implementation against Node — both must stay
// byte-identical (contract C2).
#[test]
fn output_is_byte_identical_to_node() {
    if !node_available() {
        eprintln!("SKIP (env-limited: node not on PATH) output_is_byte_identical_to_node");
        return;
    }
    let entry = js_entry();
    assert!(entry.is_file(), "bee.mjs not found at {}", entry.display());

    for args in [
        vec!["--help"],
        vec!["status", "--brief", "--json"],
        vec!["status", "--brief"],
    ] {
        let js = std::process::Command::new("node")
            .arg(&entry)
            .args(&args)
            .current_dir(repo_root())
            .output()
            .unwrap();
        let rs = Command::cargo_bin("bee")
            .unwrap()
            .args(&args)
            .env("BEE_JS_ENTRY", &entry)
            .current_dir(repo_root())
            .output()
            .unwrap();
        // bee's perf lines ("[bee] status 983ms") differ run to run even
        // Node-vs-Node — normalize duration tokens before comparing stderr.
        fn normalize_ms(bytes: &[u8]) -> String {
            let s = String::from_utf8_lossy(bytes).into_owned();
            let mut out = String::with_capacity(s.len());
            let mut chars = s.chars().peekable();
            let mut digits = String::new();
            while let Some(c) = chars.next() {
                if c.is_ascii_digit() {
                    digits.push(c);
                    continue;
                }
                if !digits.is_empty() && c == 'm' && chars.peek() == Some(&'s') {
                    chars.next();
                    out.push_str("Nms");
                } else {
                    out.push_str(&digits);
                    out.push(c);
                }
                digits.clear();
            }
            out.push_str(&digits);
            out
        }
        assert_eq!(js.status.code(), rs.status.code(), "exit code for {args:?}");
        assert_eq!(js.stdout, rs.stdout, "stdout for {args:?}");
        assert_eq!(
            normalize_ms(&js.stderr),
            normalize_ms(&rs.stderr),
            "stderr for {args:?}"
        );
    }
}

#[test]
fn missing_entry_fails_with_127_and_hint() {
    // `close` is still Node-delegated; `status` (the previous probe) is now
    // served natively end to end (status_full.rs), including the no-root
    // error path, so it never reaches the delegate.
    let tmp = tempfile::tempdir().unwrap();
    Command::cargo_bin("bee")
        .unwrap()
        .arg("close")
        .env("BEE_JS_ENTRY", tmp.path().join("nope.mjs"))
        .current_dir(tmp.path())
        .assert()
        .code(127)
        .stderr(predicates::str::contains("BEE_JS_ENTRY"));
}
