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

/// No shipped script carries a raw control byte.
///
/// THE DEFECT THIS EXISTS TO CATCH. `install.sh` resolved the newest release
/// with
///
///     sed -n 's|.*/releases/tag/\(v[0-9][^"]*\)".*|\1|p'
///
/// except that the two characters `\1` had been written to disk as the single
/// byte 0x01 — some tool in the authoring chain ate the backslash and left
/// SOH. sed then substituted the captured tag with a control character, and
/// one mangled byte took out BOTH install paths:
///
///   * `$RELEASES/download/<SOH>/bee-…` is not a URL — curl exits 3
///     ("Malformed input to a URL function"), so no published binary is ever
///     downloaded on any platform;
///   * `git clone --branch "${PREBUILT_TAG:-$REF}"` — SOH is NOT empty, so
///     the `:-` default never fires and git is asked for a branch named
///     "\x01". On Linux the clone dies there and the installer exits 1 having
///     installed nothing at all.
///
/// It is invisible in every ordinary view: a terminal prints SOH as nothing,
/// a diff shows the line as unchanged-looking, and `grep '||p'` does not match
/// because the replacement is not empty. Only `cat -A` or a byte dump shows
/// it. That is why this is a byte-level law and not a review checklist item.
#[test]
fn no_installer_carries_a_raw_control_byte() {
    let mut offenders: Vec<String> = Vec::new();
    let mut scanned = 0usize;
    for rel in ["scripts/install.sh", "scripts/install.ps1"] {
        let bytes = std::fs::read(repo_root().join(rel))
            .unwrap_or_else(|e| panic!("reading {rel}: {e}"));
        scanned += 1;
        for (i, b) in bytes.iter().enumerate() {
            // Tab, LF and CR are the only control bytes a shell script has any
            // business containing.
            if *b < 0x20 && !matches!(*b, b'\t' | b'\n' | b'\r') {
                let line = bytes[..i].iter().filter(|c| **c == b'\n').count() + 1;
                offenders.push(format!("{rel}:{line}: byte 0x{b:02x}"));
            }
        }
    }
    assert_eq!(scanned, 2, "the scan lost an installer — it can no longer go red");
    assert!(
        offenders.is_empty(),
        "a shipped installer contains a raw control byte. Almost certainly an escape sequence \
         that lost its backslash on the way to disk — check any sed/regex replacement nearby:\n  {}",
        offenders.join("\n  ")
    );
}

/// The capture is actually emitted. A `sed` substitution that captures a group
/// and then throws it away is the shape the bug above took, and it survives a
/// control-byte scan the moment the mangled byte happens to be printable.
#[test]
fn every_capturing_sed_substitution_in_install_sh_emits_its_capture() {
    let text = read("scripts/install.sh");
    let mut checked = 0usize;
    let mut offenders: Vec<String> = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let Some(start) = line.find("sed -n 's|") else { continue };
        let body = &line[start + "sed -n 's|".len()..];
        let Some(end) = body.find("|p'") else { continue };
        let expr = &body[..end];
        // pattern|replacement — split on the LAST unescaped bar.
        let Some(bar) = expr.rfind('|') else { continue };
        let (pattern, replacement) = (&expr[..bar], &expr[bar + 1..]);
        if !pattern.contains(r"\(") {
            continue; // no capture group, nothing to emit
        }
        checked += 1;
        if !replacement.contains(r"\1") {
            offenders.push(format!(
                "scripts/install.sh:{}: captures a group but the replacement is {replacement:?}",
                n + 1
            ));
        }
    }
    assert!(checked > 0, "no capturing sed substitution found — the scan went vacuous");
    assert!(offenders.is_empty(), "{}", offenders.join("\n"));
}

/// The vendored binary lands under the name everything else names.
///
/// THE DEFECT THIS EXISTS TO CATCH. Both installers copied the binary with
/// `basename` / `Split-Path -Leaf`. That yields `bee` only when the binary was
/// BUILT locally. On the published-binary path — the default, and the whole
/// point of shipping release assets — the source file is called
/// `bee-x86_64-unknown-linux-gnu`, so the host ended up with
/// `.bee/bin/bee-x86_64-unknown-linux-gnu` while every hook command, the
/// AGENTS.md block and every skill say `.bee/bin/bee`. Nothing errored: the
/// installer verified through that same wrong path and printed
/// "bee installed." The hooks were simply never going to fire.
#[test]
fn both_installers_vendor_the_binary_under_its_canonical_name() {
    // CODE ONLY. A law a comment can turn red is a law people satisfy by
    // rewording comments — and the comment right above the fix in install.sh
    // quotes the very expression this forbids, in order to explain it.
    let code = |rel: &str| -> String {
        read(rel)
            .lines()
            .map(str::trim_start)
            .filter(|l| !l.starts_with('#'))
            .collect::<Vec<_>>()
            .join("
")
    };
    let sh = code("scripts/install.sh");
    let ps1 = code("scripts/install.ps1");

    // What the wiring actually invokes — read from the shipped manifest, so
    // this law cannot drift away from the path onboarding writes.
    let manifest = read("packages/bee/hooks/claude-hooks.json");
    assert!(
        manifest.contains(".bee/bin/bee"),
        "the shipped hook manifest no longer names .bee/bin/bee — re-point this law"
    );

    assert!(
        !sh.contains(r#"basename "$BEE_BIN""#),
        "install.sh vendors under the source file name again — a downloaded asset          is not called bee"
    );
    assert!(
        sh.contains("HOST_BEE_NAME"),
        "install.sh must pick a canonical vendored name"
    );
    assert!(
        !ps1.contains("Split-Path $beeBin -Leaf)) -Force"),
        "install.ps1 vendors under the source file name again"
    );
    assert!(
        ps1.contains("hostBeeName"),
        "install.ps1 must pick a canonical vendored name"
    );
}
