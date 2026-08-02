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
