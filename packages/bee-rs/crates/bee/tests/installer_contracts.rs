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
//   * The Node preflight, in BOTH directions. It was once removed while the
//     installers still needed Node (the failure then lands as `node: command
//     not found` after a clone and a multi-minute cargo build); it would now
//     be equally wrong to KEEP one, because neither installer runs node any
//     more — `bee dev plugin-distribution` and `bee dev install-support`
//     replaced every call. A preflight for a tool the script never invokes
//     refuses installs for no reason. The test below asserts the biconditional
//     rather than either half.
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

/// Every tool an installer RUNS, it must preflight — and every tool it no
/// longer runs, it must stop preflighting. Both directions matter: a missing
/// preflight surfaces as `node: command not found` after a clone and a full
/// cargo build, and a stale one refuses an install over a tool the script never
/// invokes.
///
/// The probe reads code, not prose. An earlier version of this test matched the
/// whole file and was satisfied by the phrase "`node -e`" inside a comment
/// explaining that the `node -e` steps were gone — a guard green on the very
/// sentence describing its own subject's removal.
#[test]
fn both_installers_preflight_every_tool_they_require() {
    for (rel, cargo_probe, node_probe) in [
        ("scripts/install.sh", "command -v cargo", "command -v node"),
        ("scripts/install.ps1", "Get-Command cargo", "Get-Command node"),
    ] {
        let text = read(rel);
        assert!(text.contains(cargo_probe), "{rel} builds with cargo but never preflights it");
        let code: String = text
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");
        let uses_node = code.contains("node -e") || code.contains("| node") || code.contains("node $");
        assert_eq!(
            uses_node,
            code.contains(node_probe),
            "{rel} must preflight node exactly when it still RUNS node. Runs node={uses_node}. \
             When the last `node -e` step is ported, drop the preflight in the same commit."
        );
    }
}

/// The helper is ported and its file is gone. This used to assert the opposite
/// — that the `.mjs` still existed — which is exactly the shape of guard that
/// has to flip when the work it was waiting for lands. It now pins the other
/// direction: no installer may reach for the deleted script, and both must
/// drive the verb that replaced it.
#[test]
fn the_distribution_helper_is_the_verb_not_the_deleted_script() {
    let helper = repo_root().join("packages/bee/scripts/plugin_distribution.mjs");
    assert!(
        !helper.exists(),
        "plugin_distribution.mjs is back — it was ported to `bee dev plugin-distribution`; two \
         implementations of the distribution transaction is the one outcome worth failing over"
    );
    for rel in ["scripts/install.sh", "scripts/install.ps1"] {
        let text = read(rel);
        assert!(
            !text.contains("plugin_distribution.mjs\""),
            "{rel} still invokes the deleted helper"
        );
        assert!(
            text.contains("dev plugin-distribution"),
            "{rel} must drive the distribution preflight through the native verb"
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

/// Cloning a TAG lands on a detached HEAD, and git then prints ~15 lines of
/// advice about branches and `git switch` — to stderr, where `--quiet` does not
/// reach it. In an installer that wall is the loudest output of an otherwise
/// successful run, about a throwaway checkout the user will never touch, and it
/// reads as a failure. A user hit exactly that on 2.1.1 and asked what broke.
#[test]
fn neither_installer_lets_git_lecture_the_user_about_detached_head() {
    for rel in ["scripts/install.sh", "scripts/install.ps1"] {
        let text = read(rel);
        for (i, line) in text.lines().enumerate() {
            let t = line.trim_start();
            // `--branch` is what makes it a clone-at-a-ref, and it is what
            // lands the checkout detached. Keying on the substring "clone"
            // instead matched `sparse-checkout`, which takes no ref at all.
            if t.starts_with('#') || !t.contains("git ") || !t.contains("--branch") {
                continue;
            }
            assert!(
                line.contains("advice.detachedHead=false"),
                "{rel}:{} clones without silencing git's detached-HEAD advice:
  {}",
                i + 1,
                t
            );
        }
    }
}

/// The fetch line must name the ref it ACTUALLY clones. It printed `$REF` —
/// which defaults to `main` — while cloning `${PREBUILT_TAG:-$REF}`, so a user
/// who downloaded a release binary was told they were fetching main and was
/// given the pinned tag. A log that names the wrong input is worse than no log:
/// it is the line someone reaches for when the two disagree.
#[test]
fn the_fetch_line_names_the_ref_that_is_cloned() {
    let text = read("scripts/install.sh");
    let clone_line = text
        .lines()
        .find(|l| !l.trim_start().starts_with('#') && l.contains("clone") && l.contains("--branch"))
        .expect("install.sh must clone somewhere");
    let logged = text
        .lines()
        .find(|l| !l.trim_start().starts_with('#') && l.contains("log \"fetch"))
        .expect("install.sh must log its fetch");
    // Whatever variable the clone pins to, the log must print the same one.
    let var = clone_line
        .split("--branch")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .map(|v| v.trim_matches('"').to_string())
        .expect("the clone names a ref");
    assert!(
        logged.contains(&var),
        "the fetch log prints a different ref from the one cloned.
  clones: {var}
  logs:   {}",
        logged.trim()
    );
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

/// Onboarding is invoked from the source checkout, and the binary is in place
/// before the apply — in BOTH installers.
///
/// THE DEFECT THIS EXISTS TO CATCH, twice. `bee onboard` vendors FROM a bee
/// checkout and locates one by walking up from the binary and from the cwd. On
/// the published path those are a temp dir and the target repo, so onboarding
/// refused every run and the install died. Separately, hook wiring is
/// FEATURE-DETECTED against `.bee/bin/bee[.exe]`: vendoring the binary after
/// the apply left the first apply wiring for a file that was not there, so a
/// fresh install never converged in one pass while a second run did.
///
/// install.sh was fixed first and install.ps1 shipped one release still
/// carrying both — which is the argument for a law that reads both files
/// rather than the one that happened to be under the debugger.
#[test]
fn both_installers_onboard_from_the_source_and_vendor_before_applying() {
    // install.sh — a subshell `cd "$BEE_SRC"` around each onboard call.
    let sh = read("scripts/install.sh");
    let sh_calls: Vec<&str> =
        sh.lines().filter(|l| l.contains("onboard --repo-root")).collect();
    assert!(sh_calls.len() >= 3, "install.sh onboard call sites vanished: {sh_calls:?}");
    for line in &sh_calls {
        assert!(
            line.contains(r#"cd "$BEE_SRC""#),
            "install.sh calls onboard without cd-ing to the source checkout: {line}"
        );
    }

    // install.ps1 — Push-Location $beeSrc around each one.
    let ps1 = read("scripts/install.ps1");
    let ps_lines: Vec<(usize, &str)> = ps1.lines().enumerate().collect();
    let ps_calls: Vec<usize> = ps_lines
        .iter()
        .filter(|(_, l)| l.contains("onboard --repo-root"))
        .map(|(i, _)| *i)
        .collect();
    assert!(ps_calls.len() >= 2, "install.ps1 onboard call sites vanished");
    for i in &ps_calls {
        let window = ps_lines[i.saturating_sub(3)..=*i]
            .iter()
            .map(|(_, l)| *l)
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            window.contains("Push-Location $beeSrc"),
            "install.ps1 calls onboard without Push-Location to the source: {}",
            ps_lines[*i].1
        );
    }

    // Ordering: the vendored copy precedes the apply, in both.
    // Anchor on the INVOCATIONS, not on any mention: `--apply` also appears in
    // the help text near the top of the file, and matching that would compare
    // prose against code and call the ordering wrong.
    let sh_copy = sh.find(r#"cp "$BEE_BIN""#)
        .expect("install.sh no longer copies the binary into the target");
    let sh_apply = sh
        .find(r#"onboard --repo-root "$TARGET_DIR" --apply"#)
        .expect("install.sh no longer applies onboarding");
    assert!(sh_copy < sh_apply, "install.sh vendors the binary AFTER the apply again");

    let ps_copy = ps1.find("Copy-Item $beeBin").expect("install.ps1 no longer copies a binary");
    let ps_apply = ps1.find("--apply @onboardFlags").expect("install.ps1 no longer applies");
    assert!(ps_copy < ps_apply, "install.ps1 vendors the binary AFTER the apply again");
}

/// Since rustc 1.95 an rlib holds only a metadata STUB; the full metadata lives
/// in a sibling .rmeta. Cargo's build pipelining hands a dependent that .rmeta
/// before the .rlib is finished, and when the two race the build dies with
/// "only metadata stub found for `rlib` dependency <crate>" trailed by a
/// cascade — cannot resolve a prelude import, cannot find attribute `derive`,
/// cannot find macro `write` — that reads as a corrupt toolchain and is not one
/// (rust-lang/cargo#16790). A user who hits it has no way to know that, and the
/// installer's own advice was "Fix the build", which is not advice. The
/// fallback build is a single cold compile that gains almost nothing from
/// pipelining, so it must not gamble on the race.
#[test]
fn both_installers_build_with_pipelining_disabled() {
    let sh = read("scripts/install.sh");
    let ps1 = read("scripts/install.ps1");

    // The contract is that the fallback build's OWN line carries the setting —
    // never that the assignment sits flush against `cargo`. Other env pins live
    // on that line too (`CARGO_TARGET_DIR`, added by ba1fe413), and an assertion
    // that pinned adjacency called a correct line wrong the moment one landed.
    // `--manifest-path` is what separates the INVOCATION from the log line
    // above it, which also spells "cargo build --release" as prose.
    let sh_build = sh
        .lines()
        .find(|l| l.contains("cargo build --release --manifest-path"))
        .expect("install.sh no longer runs a fallback cargo build");
    assert!(
        sh_build.contains("CARGO_BUILD_PIPELINING=false"),
        "install.sh runs the fallback cargo build with pipelining ON again"
    );
    assert!(
        ps1.contains("$env:CARGO_BUILD_PIPELINING = 'false'"),
        "install.ps1 runs the fallback cargo build with pipelining ON again"
    );

    // Set BEFORE the build, not after it — an assignment that trails the
    // invocation is a comment with syntax.
    let set = ps1
        .find("$env:CARGO_BUILD_PIPELINING = 'false'")
        .expect("checked above");
    let build = ps1
        .find("cargo build --release --manifest-path $cargoToml")
        .expect("install.ps1 no longer builds from source");
    assert!(set < build, "install.ps1 sets CARGO_BUILD_PIPELINING after the build it guards");
}

/// Windows PowerShell 5.1 takes its TLS floor from .NET's ServicePointManager,
/// which on plenty of hosts still resolves to Ssl3|Tls10 — and github.com has
/// refused everything below TLS 1.2 since 2018. Without this line
/// Invoke-WebRequest throws, the catch swallows it, and a Windows user with a
/// published bee-x86_64-pc-windows-msvc.exe waiting for them compiles the whole
/// crate graph instead. That is the failure this whole download path exists to
/// remove, reintroduced by omission.
#[test]
fn install_ps1_raises_the_tls_floor_before_it_downloads() {
    let ps1 = read("scripts/install.ps1");

    let floor = ps1
        .find("[Net.SecurityProtocolType]::Tls12")
        .expect("install.ps1 no longer raises the TLS floor; PS 5.1 hosts will fall back to a source build");
    // Anchor on the INVOCATION, not on any mention: the comment explaining this
    // very workaround names Invoke-WebRequest, and matching that would compare
    // prose against code and call the ordering wrong.
    let first_fetch = ps1
        .find("Invoke-WebRequest -UseBasicParsing")
        .expect("install.ps1 no longer downloads a published binary");
    assert!(floor < first_fetch, "install.ps1 raises the TLS floor AFTER its first download");

    // Widened, never narrowed, and only on 5.1: PowerShell 7 starts at
    // SystemDefault, where a bare assignment would FORBID the TLS 1.3 it would
    // otherwise negotiate.
    assert!(
        ps1.contains("-bor [Net.SecurityProtocolType]::Tls12"),
        "install.ps1 assigns the TLS floor instead of OR-ing it in — that narrows PS 7"
    );
    assert!(
        ps1.contains("$PSVersionTable.PSVersion.Major -lt 6"),
        "install.ps1 applies the 5.1 TLS workaround unconditionally"
    );
}

/// Every published-binary failure here is non-fatal by design: it logs and lets
/// the source build take over. That is right, and it was silent — the catch
/// discarded the exception, so a TLS failure, a proxy, a rate limit and a
/// genuinely absent release all printed the same reasonless line before minutes
/// of compiling. The one datum that tells a user what to fix must survive.
#[test]
fn install_ps1_names_why_the_published_binary_was_skipped() {
    let ps1 = read("scripts/install.ps1");

    assert!(
        ps1.contains("$prebuiltErr = $_.Exception.Message"),
        "install.ps1 discards the reason the release could not be resolved"
    );
    assert!(
        ps1.contains("could not resolve a published release$why"),
        "install.ps1 no longer reports WHY it could not resolve a release"
    );
    assert!(
        ps1.contains(r#"no downloadable asset at $prebuiltTag ($($_.Exception.Message))"#),
        "install.ps1 discards the reason the asset download failed"
    );
}
