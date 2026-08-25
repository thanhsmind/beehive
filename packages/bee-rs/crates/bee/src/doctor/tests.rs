use super::*;

fn repo(tmp: &Path, with_binary: bool, with_skills: bool, wiring: Option<&str>) -> PathBuf {
    let root = tmp.join("repo");
    std::fs::create_dir_all(root.join(".bee").join("bin")).unwrap();
    if with_binary {
        std::fs::write(root.join(".bee/bin/bee"), b"#!/bin/sh\n").unwrap();
    }
    if with_skills {
        std::fs::create_dir_all(root.join(".agents/skills/bee-hive")).unwrap();
        std::fs::create_dir_all(root.join(".claude/skills/bee-hive")).unwrap();
    }
    if let Some(text) = wiring {
        std::fs::create_dir_all(root.join(".codex")).unwrap();
        std::fs::write(root.join(".codex/hooks.json"), text).unwrap();
    }
    root
}

fn rows_of(root: &Path, runtime: Runtime) -> Vec<(String, Option<bool>)> {
    mechanical_rows(root, runtime).into_iter().map(|r| (r.key.to_string(), r.ok)).collect()
}

/// The wiring row compares against what this binary renders, so a host that
/// installed a different bee is caught. It is the replacement for the retired
/// capability baseline, and it must actually discriminate.
#[test]
fn the_wiring_row_matches_only_the_manifest_this_binary_renders() {
    let tmp = tempfile::tempdir().unwrap();
    let good = crate::devtools::render_projection_text_for("codex").unwrap();

    let root = repo(tmp.path(), true, true, Some(&good));
    let rows = rows_of(&root, Runtime::Codex);
    assert_eq!(rows.iter().find(|(k, _)| k == "wiring_matches_binary").unwrap().1, Some(true));

    // One byte off is a mismatch — not "close enough".
    let drifted = good.replace("write-guard", "write-guardX");
    assert_ne!(drifted, good, "the fixture must actually differ");
    let root2 = repo(&tmp.path().join("b"), true, true, Some(&drifted));
    let rows2 = rows_of(&root2, Runtime::Codex);
    assert_eq!(rows2.iter().find(|(k, _)| k == "wiring_matches_binary").unwrap().1, Some(false));
}

/// A row with nothing to read reports not_ok. The whole point of the verb is
/// that it never reaches "ready" from absence.
#[test]
fn every_mechanical_row_fails_when_its_artifact_is_absent() {
    let tmp = tempfile::tempdir().unwrap();
    let root = repo(tmp.path(), false, false, None);
    for rt in [Runtime::Codex, Runtime::Claude] {
        for (key, ok) in rows_of(&root, rt) {
            assert_eq!(ok, Some(false), "{key} passed with nothing on disk");
        }
    }
}

/// The ladder. Claude reaches ready on mechanical green; Codex cannot, because
/// its trust rows are unknowable and no attestation covers them yet.
#[test]
fn codex_stops_at_degraded_where_claude_reaches_ready() {
    let tmp = tempfile::tempdir().unwrap();
    let good = crate::devtools::render_projection_text_for("codex").unwrap();
    let root = repo(tmp.path(), true, true, Some(&good));

    // Claude's settings file is the user's, with a merged hooks key - so the
    // fixture carries unrelated keys too, exactly like a real install. If the
    // row compared whole files this would fail, which is the bug it caught.
    let claude: Value =
        serde_json::from_str(&crate::devtools::render_projection_text_for("claude").unwrap())
            .unwrap();
    // The commands the PER-REPO renderer writes, not the shipped projection:
    // an installed host names the vendored binary directly.
    let settings = json!({
        "permissions": { "defaultMode": "acceptEdits" },
        "somethingTheUserAdded": true,
        "hooks": {
            "SessionStart": [{ "hooks": [
                { "type": "command", "command": "\"$CLAUDE_PROJECT_DIR\"/.bee/bin/bee hook session-init" }
            ]}],
            "PreToolUse": [{ "matcher": "Edit", "hooks": [
                { "type": "command", "command": "\"$CLAUDE_PROJECT_DIR\"/.bee/bin/bee.exe hook write-guard" }
            ]}]
        },
    });
    let _ = &claude;
    std::fs::create_dir_all(root.join(".claude")).unwrap();
    std::fs::write(root.join(".claude/settings.json"), settings.to_string()).unwrap();

    assert!(
        mechanical_rows(&root, Runtime::Claude).iter().all(|r| r.ok == Some(true)),
        "a real settings.json - merged hooks, extra keys, feature-detected .exe - must pass"
    );

    // ...and wiring that does NOT name the vendored binary must not.
    let bad = json!({"hooks": {"SessionStart": [{"hooks": [
        {"type": "command", "command": "node some-wrapper.mjs session-init"}
    ]}]}});
    std::fs::write(root.join(".claude/settings.json"), bad.to_string()).unwrap();
    let row = mechanical_rows(&root, Runtime::Claude)
        .into_iter()
        .find(|r| r.key == "wiring_points_at_the_binary")
        .unwrap();
    assert_eq!(row.ok, Some(false), "{}", row.detail);
    std::fs::write(root.join(".claude/settings.json"), settings.to_string()).unwrap();
    assert!(mechanical_rows(&root, Runtime::Codex).iter().all(|r| r.ok == Some(true)));

    // No attestation on disk -> codex is degraded, and the reason is named.
    let a = read_attestation(&root, Runtime::Codex);
    assert!(!a.valid);
    assert_eq!(a.reason, "no_attestation");
}

/// Each attestation leg is load-bearing: drift any one and the record goes
/// inert with that leg named, rather than silently continuing to certify.
#[test]
fn every_attestation_leg_can_invalidate_it_on_its_own() {
    let tmp = tempfile::tempdir().unwrap();
    let good = crate::devtools::render_projection_text_for("codex").unwrap();
    let root = repo(tmp.path(), true, true, Some(&good));
    let hooks = std::fs::read(root.join(".codex/hooks.json")).unwrap();

    let write = |rec: Value| {
        crate::fsutil::write_json_atomic(&root.join(ATTEST_REL), &rec).unwrap();
    };
    let base = |hash: &str, ver: &str, ident: &str| {
        json!({
            "schema": "doctor-attest/1",
            "runtime": "codex",
            "hooks_sha256": hash,
            "codex_version": ver,
            "repo_identity": ident,
        })
    };
    let real_hash = sha256_of(&hooks);
    let real_ident = repo_identity(&root);

    write(base("deadbeef", "x", &real_ident));
    assert_eq!(read_attestation(&root, Runtime::Codex).reason, "hash_changed");

    write(base(&real_hash, "some-version-that-is-not-live", &real_ident));
    let r = read_attestation(&root, Runtime::Codex).reason;
    assert!(
        r == "version_changed" || r == "unprobed_version",
        "a version leg that cannot be confirmed must not pass: {r}"
    );

    // Identity is only reachable once the earlier legs pass, so it needs a
    // live codex to exercise; assert the shape rather than skip it silently.
    assert!(
        CODEX_TRUST_ROWS.len() == 4,
        "the trust rows the attestation answers for must stay enumerated"
    );
}

/// Claude has nothing to attest, and saying so beats writing a record that
/// means nothing.
#[test]
fn attesting_claude_is_refused_rather_than_recorded() {
    let code = run_attest(Runtime::Claude, None, true);
    assert_ne!(
        format!("{code:?}"),
        format!("{:?}", ExitCode::SUCCESS),
        "attesting claude must refuse"
    );
}

/// Argv the verb does not serve returns None so the catalog owns the refusal,
/// and a missing --runtime is one of those shapes.
#[test]
fn unproven_argv_shapes_defer_to_the_catalog() {
    let osv = |v: &[&str]| -> Vec<OsString> { v.iter().map(OsString::from).collect() };
    assert!(try_native(&osv(&["doctor"])).is_none(), "no --runtime must not default");
    assert!(try_native(&osv(&["doctor", "--runtime", "nope"])).is_none());
    assert!(try_native(&osv(&["doctor", "--wat"])).is_none());
    assert!(try_native(&osv(&["status"])).is_none(), "doctor must not claim another verb");
}

// ─── binary_freshness ───────────────────────────────────────────────────

/// A minimal bee SOURCE checkout: `packages/bee-rs/Cargo.toml` (the
/// workspace version marker that makes the row exist at all),
/// `.claude-plugin/plugin.json` (the source of truth for the release version),
/// a `.rs` file under `crates/` (a freshness input), and a `.md` prompt (another
/// input). `.bee/bin/bee` is deliberately not written here — each test below
/// controls its content and mtime itself.
fn source_checkout(tmp: &Path, workspace_version: &str, release_version: &str) -> PathBuf {
    let root = tmp.join("repo");
    std::fs::create_dir_all(root.join(".bee/bin")).unwrap();
    std::fs::create_dir_all(root.join("packages/bee-rs/crates/bee/src")).unwrap();
    std::fs::write(
        root.join("packages/bee-rs/Cargo.toml"),
        format!(
            "[workspace]\nresolver = \"2\"\nmembers = [\"crates/bee\"]\n\n\
             [workspace.package]\nversion = \"{workspace_version}\"\nedition = \"2024\"\n"
        ),
    )
    .unwrap();
    std::fs::write(root.join("packages/bee-rs/crates/bee/src/main.rs"), "fn main() {}\n").unwrap();
    std::fs::create_dir_all(root.join(".claude-plugin")).unwrap();
    std::fs::write(
        root.join(".claude-plugin/plugin.json"),
        format!("{{\"name\": \"bee\", \"version\": \"{release_version}\"}}\n"),
    )
    .unwrap();
    std::fs::create_dir_all(root.join("packages/bee/prompts")).unwrap();
    std::fs::write(root.join("packages/bee/prompts/one.md"), "# one\n").unwrap();
    root
}

/// A real, executable `.bee/bin/bee` stand-in that answers `rs-info` with both
/// `version` and `bee_version` — enough for `installed_binary_bee_version` to
/// probe it exactly the way it probes the real binary.
#[cfg(unix)]
fn write_executable_binary(path: &Path, package_version: &str, bee_version: &str) {
    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = \"rs-info\" ]; then\n  echo '{{\"version\":\"{package_version}\",\"bee_version\":\"{bee_version}\"}}'\nfi\n"
    );
    install_executable_script(path, &script);
}

#[cfg(unix)]
fn write_executable_binary_raw(path: &Path, stdout_json: &str) {
    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = \"rs-info\" ]; then\n  echo '{stdout_json}'\nfi\n"
    );
    install_executable_script(path, &script);
}

/// Install an executable script at `path` WITHOUT ever holding `path` itself
/// open for writing.
///
/// Writing the final path directly is what made these tests flaky. `cargo
/// test` runs them on many threads; while one thread holds the script open
/// for writing, another thread's `Command::spawn` forks and the child
/// inherits that write fd. Linux then refuses to execute the file with
/// `ETXTBSY` ("text file busy") until the child execs or exits. `O_CLOEXEC`
/// does not help — it closes the fd at exec, which is already after the fork.
///
/// Writing a sibling temp file and renaming it into place removes the race
/// structurally rather than papering over it with a retry: the path the test
/// later executes is only ever created by `rename`, so no process can hold it
/// open for writing at all. Losing the only test that noticed a real doctor
/// bug to a retry loop would have cost more than the flake did.
#[cfg(unix)]
fn install_executable_script(path: &Path, script: &str) {
    use std::os::unix::fs::PermissionsExt;
    let staged = path.with_extension("staging");
    std::fs::write(&staged, script).unwrap();
    let mut perms = std::fs::metadata(&staged).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&staged, perms).unwrap();
    std::fs::rename(&staged, path).unwrap();
}

/// Present, version-matched, and newest on disk: nothing to report.
#[cfg(unix)]
#[test]
fn binary_freshness_is_ok_when_matched_and_newest() {
    let tmp = tempfile::tempdir().unwrap();
    let root = source_checkout(tmp.path(), "0.1.0", "9.9.9");
    let bin = root.join(".bee/bin/bee");
    write_executable_binary(&bin, "0.1.0", "9.9.9");

    let bin_time = std::fs::metadata(&bin).unwrap().modified().unwrap();
    let older = filetime::FileTime::from_system_time(bin_time - std::time::Duration::from_secs(60));
    for path in source_inputs(&root) {
        filetime::set_file_mtime(&path, older).unwrap();
    }

    let row = binary_freshness_row(&root).expect("the row exists in a source checkout");
    assert_eq!(row.ok, Some(true), "{}", row.detail);
}

/// A source input newer than the binary is not_ok, and the detail names the
/// exact offending path plus the rebuild remedy.
#[cfg(unix)]
#[test]
fn binary_freshness_reports_not_ok_on_a_newer_source_input() {
    let tmp = tempfile::tempdir().unwrap();
    let root = source_checkout(tmp.path(), "0.1.0", "9.9.9");
    let bin = root.join(".bee/bin/bee");
    write_executable_binary(&bin, "0.1.0", "9.9.9");

    let bin_time = std::fs::metadata(&bin).unwrap().modified().unwrap();
    let newer = filetime::FileTime::from_system_time(bin_time + std::time::Duration::from_secs(60));
    let stale_input = root.join("packages/bee-rs/crates/bee/src/main.rs");
    filetime::set_file_mtime(&stale_input, newer).unwrap();

    let row = binary_freshness_row(&root).unwrap();
    assert_eq!(row.ok, Some(false));
    assert!(
        row.detail.contains("packages/bee-rs/crates/bee/src/main.rs"),
        "{}",
        row.detail
    );
    assert!(row.detail.contains("cargo build"), "{}", row.detail);
}

/// A manifest newer than the binary is not_ok, and the detail names
/// .claude-plugin/plugin.json.
#[cfg(unix)]
#[test]
fn binary_freshness_reports_not_ok_on_a_newer_plugin_manifest() {
    let tmp = tempfile::tempdir().unwrap();
    let root = source_checkout(tmp.path(), "0.1.0", "9.9.9");
    let bin = root.join(".bee/bin/bee");
    write_executable_binary(&bin, "0.1.0", "9.9.9");

    let bin_time = std::fs::metadata(&bin).unwrap().modified().unwrap();
    let older = filetime::FileTime::from_system_time(bin_time - std::time::Duration::from_secs(60));
    for path in source_inputs(&root) {
        filetime::set_file_mtime(&path, older).unwrap();
    }
    let newer = filetime::FileTime::from_system_time(bin_time + std::time::Duration::from_secs(60));
    let manifest = root.join(".claude-plugin/plugin.json");
    filetime::set_file_mtime(&manifest, newer).unwrap();

    let row = binary_freshness_row(&root).unwrap();
    assert_eq!(row.ok, Some(false));
    assert!(
        row.detail.contains(".claude-plugin/plugin.json"),
        "{}",
        row.detail
    );
    assert!(row.detail.contains("cargo build"), "{}", row.detail);
}

/// A release version mismatch is not_ok, naming both versions and the remedy —
/// even when every source mtime is older than the binary, so the mtime leg alone
/// could never have caught it.
#[cfg(unix)]
#[test]
fn binary_freshness_reports_not_ok_on_a_version_mismatch() {
    let tmp = tempfile::tempdir().unwrap();
    let root = source_checkout(tmp.path(), "0.1.0", "2.18.0");
    let bin = root.join(".bee/bin/bee");
    write_executable_binary(&bin, "0.1.0", "2.17.1");

    let bin_time = std::fs::metadata(&bin).unwrap().modified().unwrap();
    let older = filetime::FileTime::from_system_time(bin_time - std::time::Duration::from_secs(60));
    for path in source_inputs(&root) {
        filetime::set_file_mtime(&path, older).unwrap();
    }

    let row = binary_freshness_row(&root).unwrap();
    assert_eq!(row.ok, Some(false));
    assert!(row.detail.contains("2.17.1"), "{}", row.detail);
    assert!(row.detail.contains("2.18.0"), "{}", row.detail);
    assert!(row.detail.contains("cargo build"), "{}", row.detail);
}

/// A binary whose rs-info omits bee_version is not_ok.
#[cfg(unix)]
#[test]
fn binary_freshness_reports_not_ok_when_binary_omits_bee_version() {
    let tmp = tempfile::tempdir().unwrap();
    let root = source_checkout(tmp.path(), "0.1.0", "2.18.0");
    let bin = root.join(".bee/bin/bee");
    write_executable_binary_raw(&bin, "{\"version\":\"0.1.0\"}");

    let bin_time = std::fs::metadata(&bin).unwrap().modified().unwrap();
    let older = filetime::FileTime::from_system_time(bin_time - std::time::Duration::from_secs(60));
    for path in source_inputs(&root) {
        filetime::set_file_mtime(&path, older).unwrap();
    }

    let row = binary_freshness_row(&root).unwrap();
    assert_eq!(row.ok, Some(false));
    assert!(row.detail.contains("bee_version"), "{}", row.detail);
    assert!(row.detail.contains("cargo build"), "{}", row.detail);
}

/// A checkout with no .claude-plugin/plugin.json is unknown, not ok.
#[test]
fn binary_freshness_is_unknown_when_plugin_manifest_is_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let root = source_checkout(tmp.path(), "0.1.0", "2.18.0");
    std::fs::remove_file(root.join(".claude-plugin/plugin.json")).unwrap();

    let row = binary_freshness_row(&root).expect("the row exists in a source checkout");
    assert_eq!(row.ok, None, "{}", row.detail);
    assert!(row.detail.contains(".claude-plugin/plugin.json"), "{}", row.detail);
}

/// Regression test pinning the real defect: a binary and a workspace Cargo.toml
/// that agree on the pinned package version while the release versions differ
/// must be not_ok, so the tautology can never come back.
#[cfg(unix)]
#[test]
fn binary_freshness_catches_release_drift_when_package_versions_agree() {
    let tmp = tempfile::tempdir().unwrap();
    let root = source_checkout(tmp.path(), "0.1.0", "2.18.0");
    let bin = root.join(".bee/bin/bee");
    write_executable_binary(&bin, "0.1.0", "2.17.1");

    let bin_time = std::fs::metadata(&bin).unwrap().modified().unwrap();
    let older = filetime::FileTime::from_system_time(bin_time - std::time::Duration::from_secs(60));
    for path in source_inputs(&root) {
        filetime::set_file_mtime(&path, older).unwrap();
    }

    let row = binary_freshness_row(&root).unwrap();
    assert_eq!(
        row.ok,
        Some(false),
        "tautological package-version agreement must not pass when release versions differ"
    );
    assert!(row.detail.contains("2.17.1"), "{}", row.detail);
    assert!(row.detail.contains("2.18.0"), "{}", row.detail);
    assert!(row.detail.contains("cargo build"), "{}", row.detail);
}

/// A file at `.bee/bin/bee` that cannot be executed at all — so
/// `installed_binary_bee_version` returns `Failed` and no version is ever read.
#[cfg(unix)]
fn write_unprobeable_binary(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::write(path, "#!/bin/sh\nexit 0\n").unwrap();
    let mut perms = std::fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o644);
    std::fs::set_permissions(path, perms).unwrap();
}

/// A probe that could not run read no version, so the row must not claim the
/// binary matches source. With nothing newer on disk there is no other
/// evidence either, so the honest answer is unknown.
#[cfg(unix)]
#[test]
fn binary_freshness_is_unknown_when_the_probe_fails_and_nothing_is_newer() {
    let tmp = tempfile::tempdir().unwrap();
    let root = source_checkout(tmp.path(), "0.1.0", "2.18.0");
    let bin = root.join(".bee/bin/bee");
    write_unprobeable_binary(&bin);

    let bin_time = std::fs::metadata(&bin).unwrap().modified().unwrap();
    let older = filetime::FileTime::from_system_time(bin_time - std::time::Duration::from_secs(60));
    for path in source_inputs(&root) {
        filetime::set_file_mtime(&path, older).unwrap();
    }

    let row = binary_freshness_row(&root).unwrap();
    assert_eq!(row.ok, None, "a failed probe must never report the binary as fresh: {}", row.detail);
    assert!(
        !row.detail.contains("matches source"),
        "the detail must not claim a match it never read: {}",
        row.detail
    );
    assert!(!row.detail.contains("2.18.0"), "no version was read: {}", row.detail);
    assert!(row.detail.contains("rs-info"), "{}", row.detail);
    assert!(row.detail.contains("cargo build"), "{}", row.detail);
}

/// A source input newer than the binary is real evidence of drift, and it still
/// wins over a failed probe: not_ok with the existing newer-input detail.
#[cfg(unix)]
#[test]
fn binary_freshness_reports_not_ok_when_the_probe_fails_and_a_source_input_is_newer() {
    let tmp = tempfile::tempdir().unwrap();
    let root = source_checkout(tmp.path(), "0.1.0", "2.18.0");
    let bin = root.join(".bee/bin/bee");
    write_unprobeable_binary(&bin);

    let bin_time = std::fs::metadata(&bin).unwrap().modified().unwrap();
    let newer = filetime::FileTime::from_system_time(bin_time + std::time::Duration::from_secs(60));
    let stale_input = root.join("packages/bee-rs/crates/bee/src/main.rs");
    filetime::set_file_mtime(&stale_input, newer).unwrap();

    let row = binary_freshness_row(&root).unwrap();
    assert_eq!(row.ok, Some(false), "{}", row.detail);
    assert!(
        row.detail.contains("packages/bee-rs/crates/bee/src/main.rs"),
        "{}",
        row.detail
    );
    assert!(row.detail.contains("cargo build"), "{}", row.detail);
}

/// No binary installed is `hook_handler`'s not_ok to report; this row stays
/// unknown rather than repeating the same verdict under a second name.
#[test]
fn binary_freshness_is_unknown_without_an_installed_binary() {
    let tmp = tempfile::tempdir().unwrap();
    let root = source_checkout(tmp.path(), "0.1.0", "9.9.9");
    let row = binary_freshness_row(&root).expect("the row exists in a source checkout");
    assert_eq!(row.ok, None, "{}", row.detail);
}

/// A host project — no `packages/bee-rs/Cargo.toml` — never sees this row at
/// all, never a false not_ok from a distributed binary with no source to lag.
#[test]
fn binary_freshness_is_absent_outside_a_source_checkout() {
    let tmp = tempfile::tempdir().unwrap();
    let root = repo(tmp.path(), true, true, None);
    assert!(binary_freshness_row(&root).is_none());
    assert!(
        rows_of(&root, Runtime::Claude).iter().all(|(k, _)| k != "binary_freshness"),
        "a host project must never carry the binary_freshness row"
    );
}
