// installer_contracts — the two installers' cross-platform invariants.
//
// WHY THIS FILE EXISTS. `scripts/install.sh` and `scripts/install.ps1` are the
// only two entry points a new host has, and after the R6 cutover NOTHING
// checked them: their suite (`packages/bee/scripts/test_onboard_bee.mjs` and
// friends) was deleted with the Node tree as unrunnable. Two invariants those
// tests had held then went quiet, and both had already drifted by the time
// anyone looked:
//
//   * install.ps1 says of itself "Keep this file pure ASCII: the onboard test
//     enforces it" — and the cutover commit put an em dash in a comment. On
//     Windows PowerShell 5.1 a non-ASCII byte in a BOM-less .ps1 is a
//     parse-time bomb (docs/knowledge/patterns/20260714-non-ascii-in-a-ps1-...).
//   * The R6 sweep removed the Node preflight from BOTH installers on the
//     grounds that the runtime no longer needs Node — but INSTALLING still
//     does, because plugin_distribution.mjs was never ported. A missing
//     preflight surfaces as `node: command not found` after a git clone and a
//     multi-minute cargo build.
//
// These are cheap invariants with expensive failure modes, which is exactly
// what a guard is for.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4).unwrap().to_path_buf()
}

fn read(rel: &str) -> String {
    let path = repo_root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Windows PowerShell 5.1 decodes a BOM-less script as the ANSI codepage, so a
/// single multi-byte character anywhere — comments included — can break parsing
/// on the exact platform this file exists to serve.
#[test]
fn install_ps1_is_pure_ascii() {
    let text = read("scripts/install.ps1");
    let offenders: Vec<(usize, char)> = text
        .lines()
        .enumerate()
        .flat_map(|(i, line)| {
            line.chars().filter(|c| !c.is_ascii()).map(move |c| (i + 1, c)).collect::<Vec<_>>()
        })
        .collect();
    assert!(
        offenders.is_empty(),
        "install.ps1 must stay pure ASCII (PowerShell 5.1 parses BOM-less files as ANSI); found {:?}",
        offenders
    );
}

/// Both installers build the binary with cargo and drive the unported
/// distribution helper with node. Whichever is missing, the user should learn
/// it in the first second — not after a clone and a release build.
#[test]
fn both_installers_preflight_every_tool_they_require() {
    for (rel, cargo_probe, node_probe) in [
        ("scripts/install.sh", "command -v cargo", "command -v node"),
        ("scripts/install.ps1", "Get-Command cargo", "Get-Command node"),
    ] {
        let text = read(rel);
        assert!(text.contains(cargo_probe), "{rel} builds with cargo but never preflights it");
        assert!(
            text.contains(node_probe),
            "{rel} invokes node (plugin_distribution.mjs is unported) but never preflights it — \
             the failure would land after a clone and a full cargo build"
        );
    }
}

/// The helper is the one surviving `.mjs` and the reason node is required at
/// all. If it is ever ported, both preflights and both error messages become
/// wrong together — this pins them to the same fact.
#[test]
fn the_node_requirement_names_the_file_that_causes_it() {
    let helper = repo_root().join("packages/bee/scripts/plugin_distribution.mjs");
    assert!(
        helper.is_file(),
        "plugin_distribution.mjs is gone — if it was ported, drop the node preflight from both \
         installers and this test with it"
    );
    for rel in ["scripts/install.sh", "scripts/install.ps1"] {
        assert!(
            read(rel).contains("plugin_distribution.mjs"),
            "{rel}'s node requirement must name the file that causes it, so the day it is ported \
             the requirement is findable"
        );
    }
}

/// The cutover deleted `onboard_bee.mjs`; both installers now drive
/// `bee onboard` through the native binary. Help text that still names it sends
/// a USER to a file that has not existed since 2026-08-02.
///
/// Comments are deliberately out of scope. The repository's convention is that
/// a retired name written in prose reads as history — `install.sh` carries a
/// load-bearing example ("this probe keyed on ... which is deleted"), and a
/// check that forced its removal would delete the record of why the probe
/// changed. What a user sees is the bar; what a maintainer reads is not.
#[test]
fn no_installer_help_text_names_the_deleted_node_onboarder() {
    for rel in ["scripts/install.sh", "scripts/install.ps1"] {
        let text = read(rel);
        let hits: Vec<usize> = text
            .lines()
            .enumerate()
            .filter(|(_, l)| {
                let t = l.trim_start();
                !t.starts_with('#') && l.contains("onboard_bee.mjs")
            })
            .map(|(i, _)| i + 1)
            .collect();
        assert!(
            hits.is_empty(),
            "{rel} shows the deleted onboard_bee.mjs to a user at line(s) {hits:?}"
        );
    }
}

/// cargo emits `bee` on unix and `bee.exe` on windows. An installer that knows
/// only one of the two works on only one of the two platforms.
#[test]
fn both_installers_handle_both_binary_names() {
    for rel in ["scripts/install.sh", "scripts/install.ps1"] {
        let text = read(rel);
        assert!(text.contains("bee.exe"), "{rel} never mentions bee.exe — Windows build unhandled");
        let unix_form = text.contains("release/bee\"")
            || text.contains("release/bee'")
            || text.contains("release/bee ")
            || text.contains("release\\bee'")
            || text.contains("target/release/bee[");
        assert!(unix_form, "{rel} never falls back to the extension-less unix binary name");
    }
}
