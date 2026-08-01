// roots — Rust port of state.mjs `resolveRootsCore`, BOTH arms.
//
// Provenance: packages/bee/lib/state.mjs resolveRootsCore / resolveRoots /
// findRepoRoot / locateGitRoot / readGitdirFile / WorktreeLinkInvalidError,
// plus lib/worktree-store.mjs readGrants (the grant registry the linked-
// worktree arm consults).
//
// Until this file carried the linked half, a `.git` FILE at the resolved root
// was classified `NeedsNode` and every native verb delegated inside a linked
// git worktree (plans/rust-port.md, "Coverage debts R6 must close" →
// "linked-worktree roots"). `resolve_roots_core` below now answers the whole
// question natively: gitdir pointer read, the `.git/worktrees/<id>` namespace
// shape check, the bidirectional back-pointer validation, the four
// WorktreeLinkInvalidError messages byte-for-byte, and the grant-registry
// lookup that decides between the worktree's own store and main's.
//
// CLI flavor vs hook flavor. hooks/adapter.rs carries a SECOND port of the
// same walk, deliberately: adapter.mjs realpaths every root, never throws,
// and treats a non-`worktrees` namespace as an ordinary checkout
// (`git init --separate-git-dir`). The CLI flavor ported here does none of
// that — no realpath (Node uses plain `path.resolve`), and every failed
// validation THROWS WorktreeLinkInvalidError, which bee.mjs's main() turns
// into `emitError(error.message)`. The two flavors must not be merged.
//
// ─── why `resolve_store_root` still returns NeedsNode for a linked worktree ──
//
// Classification is only half of "a verb runs natively in a worktree". Every
// verb ported so far was ported against the invariant "the native path only
// ever holds an ORDINARY classification", and several of them encode it:
//
//   * verbs/status_full.rs hardcodes `worktree_notice: Option<String> = None`
//     (bee.mjs ungrantedWorktreeNotice), and its orient port implements only
//     the ordinary-checkout half of orientWorktreeContext. Inside an
//     UNGRANTED worktree Node emits a `worktree_notice`; inside a GRANTED one
//     Node emits an orient `worktree` block. Both are structurally missing
//     here.
//   * verbs/reservations.rs states the invariant in its own header and treats
//     resolveMainRoot(root)/resolveHoldTopology(root) as the constants
//     `root` / `{mainRoot: root, holder: 'main'}`. In a worktree Node answers
//     differently (holder = the git-verified id when granted; the whole
//     cross-worktree mirroring is SKIPPED when ungranted).
//   * state.mjs `controlRootFor` re-roots claims/sessions/workflows onto
//     mainRoot for a linked worktree; the ported verbs assume
//     controlRoot === root (status_full.rs control_root_for says so).
//
// So flipping this mapping to `Ordinary(store_root)` would make those verbs
// serve WRONG bytes in a worktree — a C2 break, not a coverage win. The flip
// is per-verb work: a verb becomes worktree-native once its own
// worktree-sensitive branches are ported, and it opts in by calling
// `resolve_roots_core` instead of `resolve_store_root`. `verbs/worktree.rs`
// is the first such caller (it MUST run inside a worktree to be useful).
// Everything else keeps today's delegation, unchanged and byte-identical.

use serde_json::Value;
use std::path::{Path, PathBuf};

pub enum Roots {
    /// Ordinary checkout: store root == work root.
    Ordinary(PathBuf),
    /// A shape the calling verb has not proven it can serve: a linked
    /// worktree (valid or broken), or a stat that raced.
    NeedsNode,
    /// No .bee/onboarding.json and no .git anywhere up the tree.
    None,
}

/// state.mjs resolveRootsCore's full return shape. `Ordinary`/`LinkedValid`
/// are its two `worktreeResolution` values, `LinkInvalid` is the throw,
/// `Unresolved` is `{storeRoot: null, workRoot: null}`, and `Exotic` is the
/// one place Node throws a V8-worded error (a `.git` that vanished between
/// existsSync and statSync) — never reproduced natively, always delegated.
pub enum Resolution {
    Ordinary {
        store_root: PathBuf,
        work_root: PathBuf,
    },
    LinkedValid {
        store_root: PathBuf,
        work_root: PathBuf,
        id: String,
        main_root: PathBuf,
    },
    /// WorktreeLinkInvalidError — `.message` byte-for-byte, `.code` is
    /// always "WORKTREE_LINK_INVALID" and is not modeled separately because
    /// bee.mjs only ever surfaces `.message`. The message is built and
    /// pinned against Node in this file's tests; no verb EMITS it yet (every
    /// caller delegates this arm, see the header), hence the allow.
    LinkInvalid {
        #[allow(dead_code)]
        message: String,
    },
    Unresolved,
    Exotic,
}

// ─── path primitives (Node path.resolve/basename/dirname, win32 flavor) ────
// Twins of the same three helpers in verbs/status_full.rs; kept local because
// roots.rs sits below verbs/ in the dependency order.

fn js_trim(s: &str) -> &str {
    s.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
}

/// Node's lexical normalization: separators unified, `.`/empty dropped, `..`
/// popped. No filesystem access, exactly like path.resolve.
fn normalize_abs_lexical(p: &str) -> String {
    let sep = std::path::MAIN_SEPARATOR;
    let unified: String = p.replace(['/', '\\'], &sep.to_string());
    let mut prefix = String::new();
    let mut rest = unified.as_str();
    if cfg!(windows) {
        let bytes = rest.as_bytes();
        if bytes.len() >= 2 && bytes[1] == b':' {
            prefix = rest[..2].to_string();
            rest = &rest[2..];
        }
    }
    let absolute = rest.starts_with(sep);
    let mut parts: Vec<&str> = Vec::new();
    for seg in rest.split(sep) {
        match seg {
            "" | "." => {}
            ".." => {
                if !parts.is_empty() {
                    parts.pop();
                }
            }
            s => parts.push(s),
        }
    }
    let mut out = prefix;
    if absolute {
        out.push(sep);
    }
    out.push_str(&parts.join(&sep.to_string()));
    out
}

/// Node path.resolve(base, p): an absolute `p` normalizes alone, a relative
/// one joins onto `base`.
fn path_resolve(base: &str, p: &str) -> String {
    let b = p.as_bytes();
    let is_abs = p.starts_with('/') || p.starts_with('\\') || (b.len() >= 2 && b[1] == b':');
    if is_abs {
        normalize_abs_lexical(p)
    } else {
        normalize_abs_lexical(&format!("{base}{}{p}", std::path::MAIN_SEPARATOR))
    }
}

fn path_basename(p: &str) -> &str {
    match p.rfind(std::path::MAIN_SEPARATOR) {
        Some(idx) => &p[idx + 1..],
        None => p,
    }
}

fn path_dirname(p: &str) -> String {
    // path.resolve(x, '..') is what resolveRootsCore actually uses for the
    // two hops above `gitdir`; dirname is only used for `mainRoot`, whose
    // input is always a normalized absolute path.
    path_resolve(p, "..")
}

/// `path.resolve(startDir || process.cwd())` — the one entry normalization
/// both walks start from.
fn resolve_start(start: &Path) -> String {
    let abs = std::path::absolute(start).unwrap_or_else(|_| start.to_path_buf());
    normalize_abs_lexical(&abs.to_string_lossy())
}

/// One walk-up step: `path.dirname(dir)`, stopping when it stops changing.
fn parent_of(dir: &str) -> Option<String> {
    let parent = path_dirname(dir);
    if parent == dir {
        None
    } else {
        Some(parent)
    }
}

fn exists(p: &str) -> bool {
    Path::new(p).exists()
}

fn join(base: &str, seg: &str) -> String {
    format!("{base}{}{seg}", std::path::MAIN_SEPARATOR)
}

// ─── state.mjs readGitdirFile ─────────────────────────────────────────────

/// `fs.readFileSync(file,'utf8').trim()`, an optional `gitdir:` prefix
/// stripped and re-trimmed, backslashes rewritten to the platform separator,
/// then `path.resolve(base, raw)`. Any read failure is `null`.
///
/// Note the ordering Node uses and this port keeps: emptiness is tested
/// BEFORE the prefix strip, so a bare `gitdir:` resolves to `base` itself
/// (and then fails the namespace check) instead of returning null.
fn read_gitdir_file(file: &str, base: &str) -> Option<String> {
    // Node decodes utf8 lossily (U+FFFD for invalid bytes) rather than
    // throwing, so a non-UTF-8 pointer file must not read as "missing".
    let bytes = std::fs::read(file).ok()?;
    let text = String::from_utf8_lossy(&bytes);
    let raw = js_trim(&text);
    if raw.is_empty() {
        return None;
    }
    let raw = match raw.strip_prefix("gitdir:") {
        Some(rest) => js_trim(rest),
        None => raw,
    };
    let sep_fixed = raw.replace('\\', &std::path::MAIN_SEPARATOR.to_string());
    Some(path_resolve(base, &sep_fixed))
}

// ─── worktree-store.mjs readGrants ────────────────────────────────────────

/// `<mainStoreRoot>/runtime/worktree-grants.json`, `{}` on any missing /
/// unreadable / malformed file — this never fails, because a throw here
/// would propagate into a fail-open hook and become a silent allow.
///
/// `main_store_root` is always the MAIN checkout's `.bee` directory, never a
/// worktree's own: that asymmetry is the security property (a worktree can
/// write any self-claiming marker it likes into its own `.bee/runtime/` and
/// it is never read).
pub fn read_grants(main_store_root: &Path) -> Value {
    let file = main_store_root.join("runtime").join("worktree-grants.json");
    match std::fs::read(&file)
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
    {
        // `parsed && typeof parsed === 'object'` — arrays pass that test in
        // JS too, so an array registry is kept as-is and indexed below.
        Some(v @ Value::Object(_)) | Some(v @ Value::Array(_)) => v,
        _ => Value::Object(serde_json::Map::new()),
    }
}

/// `grants[id] === true` with JS property-access semantics (an array
/// registry is indexed numerically, everything else misses).
pub fn grant_is_true(grants: &Value, id: &str) -> bool {
    match grants {
        Value::Object(m) => m.get(id) == Some(&Value::Bool(true)),
        Value::Array(a) => {
            // JS array index access: only a canonical integer key hits.
            let canonical = id.parse::<usize>().ok().filter(|n| n.to_string() == id);
            canonical.and_then(|n| a.get(n)) == Some(&Value::Bool(true))
        }
        _ => false,
    }
}

// ─── resolveRootsCore ─────────────────────────────────────────────────────

/// The whole classification, both arms. Never panics; the arms map 1:1 onto
/// what Node returns or throws.
pub fn resolve_roots_core(start: &Path) -> Resolution {
    let start = resolve_start(start);

    // Pass 1: a fixture/project may live below an unrelated ancestor holding
    // a `.git` (e.g. /tmp/.git). The nearest onboarding marker WITHOUT a
    // `.git` of its own wins outright; linked worktrees keep their own `.git`
    // FILE and so continue into the validation below.
    let mut dir = start.clone();
    loop {
        if exists(&join(&join(&dir, ".bee"), "onboarding.json")) && !exists(&join(&dir, ".git")) {
            return Resolution::Ordinary {
                store_root: PathBuf::from(&dir),
                work_root: PathBuf::from(&dir),
            };
        }
        match parent_of(&dir) {
            Some(p) => dir = p,
            None => break,
        }
    }

    // locateGitRoot(startDir): nearest ancestor holding a `.git` marker.
    let mut located: Option<(String, String)> = None;
    let mut dir = start.clone();
    loop {
        let marker = join(&dir, ".git");
        if exists(&marker) {
            located = Some((dir.clone(), marker));
            break;
        }
        match parent_of(&dir) {
            Some(p) => dir = p,
            None => break,
        }
    }

    let Some((work_root, marker)) = located else {
        // No git anywhere: the nearest onboarding marker, else nothing.
        let mut dir = start.clone();
        loop {
            if exists(&join(&join(&dir, ".bee"), "onboarding.json")) {
                return Resolution::Ordinary {
                    store_root: PathBuf::from(&dir),
                    work_root: PathBuf::from(&dir),
                };
            }
            match parent_of(&dir) {
                Some(p) => dir = p,
                None => break,
            }
        }
        return Resolution::Unresolved;
    };

    match std::fs::metadata(&marker) {
        // A `.git` DIRECTORY is an ordinary checkout, done.
        Ok(m) if !m.is_file() => {
            return Resolution::Ordinary {
                store_root: PathBuf::from(&work_root),
                work_root: PathBuf::from(&work_root),
            }
        }
        // statSync threw (the marker raced away between existsSync and
        // statSync): Node surfaces the V8 ENOENT message. Never reproduced.
        Err(_) => return Resolution::Exotic,
        Ok(_) => {}
    }

    // ── linked worktree: `.git` is a FILE ─────────────────────────────────
    let invalid = |reason: &str| Resolution::LinkInvalid {
        message: format!("{reason} ({marker})"),
    };

    let Some(gitdir) = read_gitdir_file(&marker, &work_root) else {
        return invalid("linked worktree gitdir is missing or malformed");
    };
    let worktrees_root = path_resolve(&gitdir, "..");
    let common_git_dir = path_resolve(&worktrees_root, "..");
    if path_basename(&common_git_dir) != ".git" || path_basename(&worktrees_root) != "worktrees" {
        return invalid("linked worktree gitdir is outside the expected .git/worktrees namespace");
    }
    let id = path_basename(&gitdir);
    if id.is_empty() || id == "." || id == ".." {
        return invalid("linked worktree id is empty");
    }
    // The back-pointer: <gitdir>/gitdir must resolve to the SAME `.git`
    // marker we started from. This is the half that makes the link
    // bidirectional — a worktree cannot claim a gitdir that does not claim it
    // back.
    let reverse = read_gitdir_file(&join(&gitdir, "gitdir"), &gitdir);
    match &reverse {
        Some(r) if path_resolve(r, ".") == path_resolve(&marker, ".") => {}
        _ => return invalid("linked worktree reverse gitdir pointer is missing or mismatched"),
    }
    let main_root = path_dirname(&common_git_dir);
    // Opt-in per-worktree store: a worktree whose git-verified id is
    // registered in the MAIN store's grant registry resolves to its own local
    // store; an unregistered id resolves to main (the P40 default). Grants
    // are read only from the main store, never from anything the worktree
    // claims about itself.
    let granted = grant_is_true(&read_grants(Path::new(&join(&main_root, ".bee"))), id);
    Resolution::LinkedValid {
        store_root: PathBuf::from(if granted { &work_root } else { &main_root }),
        work_root: PathBuf::from(&work_root),
        id: id.to_string(),
        main_root: PathBuf::from(&main_root),
    }
}

/// state.mjs findRepoRoot — `resolveRoots(startDir).storeRoot` — narrowed to
/// what a verb ported for ORDINARY checkouts can serve. See this file's
/// header for why every linked-worktree arm still routes to Node.
pub fn resolve_store_root(start: &Path) -> Roots {
    match resolve_roots_core(start) {
        Resolution::Ordinary { store_root, .. } => Roots::Ordinary(store_root),
        // Classified natively, but the CALLER's port assumes an ordinary
        // checkout — delegate rather than serve wrong bytes.
        Resolution::LinkedValid { .. } => Roots::NeedsNode,
        // Node throws WorktreeLinkInvalidError out of main()'s findRepoRoot,
        // which also SKIPS the timings.jsonl append (the throw escapes the
        // logging block too). Reproducing that means bypassing the shared
        // timing wrapper, so the whole command delegates instead.
        Resolution::LinkInvalid { .. } => Roots::NeedsNode,
        Resolution::Exotic => Roots::NeedsNode,
        Resolution::Unresolved => Roots::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    // Every expectation below was pinned against Node before it was written
    // here, with scripts equivalent to:
    //   node -e "import('.../lib/state.mjs').then(m=>{try{console.log(
    //     JSON.stringify(m.resolveRoots(process.argv[1])))}catch(e){
    //     console.log(e.name, e.message)}})" <dir>
    // run over the SAME fixture shapes these tests build (a real
    // `git worktree add`, plus the four hand-broken links).

    fn git(cwd: &Path, args: &[&str]) {
        let out = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("git must be on PATH for the worktree fixtures");
        assert!(
            out.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// A real main checkout with one real linked worktree (`git worktree
    /// add`). Returns (main_root, worktree_root) as Node would resolve them.
    fn fixture(tmp: &Path, wt_name: &str) -> (PathBuf, PathBuf) {
        let main = tmp.join("main");
        std::fs::create_dir_all(main.join(".bee")).unwrap();
        std::fs::write(main.join(".bee").join("onboarding.json"), "{}").unwrap();
        std::fs::write(main.join("f.txt"), "x").unwrap();
        git(&main, &["init", "-q", "."]);
        git(&main, &["config", "user.email", "a@b.c"]);
        git(&main, &["config", "user.name", "t"]);
        git(&main, &["add", "-A"]);
        git(&main, &["commit", "-qm", "init"]);
        let wt = tmp.join(wt_name);
        git(
            &main,
            &["worktree", "add", "-q", wt.to_str().unwrap(), "-b", &format!("wt/{wt_name}")],
        );
        (main, wt)
    }

    fn norm(p: &Path) -> String {
        normalize_abs_lexical(&p.to_string_lossy())
    }

    #[test]
    fn onboarding_marker_without_git_wins_over_ancestor_git() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        let nested = root.join("fixture");
        std::fs::create_dir_all(nested.join(".bee")).unwrap();
        std::fs::write(nested.join(".bee").join("onboarding.json"), "{}").unwrap();
        match resolve_store_root(&nested) {
            Roots::Ordinary(r) => assert_eq!(norm(&r), norm(&nested)),
            _ => panic!("expected ordinary root at the fixture"),
        }
    }

    #[test]
    fn ordinary_git_dir_resolves_to_work_root() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        std::fs::create_dir_all(root.join(".git")).unwrap();
        let deep = root.join("a").join("b");
        std::fs::create_dir_all(&deep).unwrap();
        match resolve_store_root(&deep) {
            Roots::Ordinary(r) => assert_eq!(norm(&r), norm(&root)),
            _ => panic!("expected ordinary root"),
        }
    }

    #[test]
    fn nothing_found_is_none() {
        let tmp = tempfile::tempdir().unwrap();
        // A bare temp dir tree with neither marker. (If the OS temp root ever
        // sits under a git checkout this would misclassify — acceptable in
        // practice for CI/dev machines.)
        let deep = tmp.path().join("x").join("y");
        std::fs::create_dir_all(&deep).unwrap();
        assert!(matches!(
            resolve_store_root(&deep),
            Roots::None | Roots::Ordinary(_)
        ));
    }

    // ── the linked half ───────────────────────────────────────────────────

    /// Node (pinned): worktreeResolution 'linked-valid', storeRoot = mainRoot
    /// while the id is UNREGISTERED, workRoot/worktreeRoot = the worktree,
    /// id = the directory basename.
    #[test]
    fn real_linked_worktree_ungranted_resolves_to_main_store() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, wt) = fixture(tmp.path(), "wt-a");
        match resolve_roots_core(&wt) {
            Resolution::LinkedValid {
                store_root,
                work_root,
                id,
                main_root,
            } => {
                assert_eq!(norm(&store_root), norm(&main));
                assert_eq!(norm(&work_root), norm(&wt));
                assert_eq!(norm(&main_root), norm(&main));
                assert_eq!(id, "wt-a");
            }
            _ => panic!("expected linked-valid"),
        }
        // Deep inside the worktree resolves identically (walk-up).
        let deep = wt.join("a").join("b");
        std::fs::create_dir_all(&deep).unwrap();
        assert!(matches!(
            resolve_roots_core(&deep),
            Resolution::LinkedValid { .. }
        ));
    }

    /// Node (pinned): the SAME fixture with `{"wt-a": true}` in the MAIN
    /// store's registry flips storeRoot to the worktree's own root.
    #[test]
    fn granted_linked_worktree_resolves_to_its_own_store() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, wt) = fixture(tmp.path(), "wt-a");
        let runtime = main.join(".bee").join("runtime");
        std::fs::create_dir_all(&runtime).unwrap();
        std::fs::write(
            runtime.join("worktree-grants.json"),
            "{\"wt-a\": true}\n",
        )
        .unwrap();
        match resolve_roots_core(&wt) {
            Resolution::LinkedValid { store_root, id, .. } => {
                assert_eq!(norm(&store_root), norm(&wt));
                assert_eq!(id, "wt-a");
            }
            _ => panic!("expected linked-valid"),
        }
        // A grant for a DIFFERENT id, and a non-`true` value, both miss.
        std::fs::write(
            runtime.join("worktree-grants.json"),
            "{\"wt-a\": \"true\", \"other\": true}\n",
        )
        .unwrap();
        match resolve_roots_core(&wt) {
            Resolution::LinkedValid { store_root, .. } => assert_eq!(norm(&store_root), norm(&main)),
            _ => panic!("expected linked-valid"),
        }
        // A corrupt registry reads as {} (readGrants never throws).
        std::fs::write(runtime.join("worktree-grants.json"), "{not json").unwrap();
        match resolve_roots_core(&wt) {
            Resolution::LinkedValid { store_root, .. } => assert_eq!(norm(&store_root), norm(&main)),
            _ => panic!("expected linked-valid"),
        }
    }

    /// A worktree whose grant registry lives in the WORKTREE's own store is
    /// ignored — grants are read from main only (the spike-4 security case).
    #[test]
    fn self_written_grant_marker_is_ignored() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, wt) = fixture(tmp.path(), "wt-a");
        let own = wt.join(".bee").join("runtime");
        std::fs::create_dir_all(&own).unwrap();
        std::fs::write(own.join("worktree-grants.json"), "{\"wt-a\": true}\n").unwrap();
        match resolve_roots_core(&wt) {
            Resolution::LinkedValid { store_root, .. } => assert_eq!(norm(&store_root), norm(&main)),
            _ => panic!("expected linked-valid"),
        }
    }

    /// Node (pinned): "linked worktree reverse gitdir pointer is missing or
    /// mismatched (<marker>)" — for a deleted back-pointer, a back-pointer
    /// aimed elsewhere, and a gitdir that does not exist at all.
    #[test]
    fn broken_back_pointer_is_link_invalid() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, wt) = fixture(tmp.path(), "wt-b");
        let back = main.join(".git").join("worktrees").join("wt-b").join("gitdir");
        let expected = format!(
            "linked worktree reverse gitdir pointer is missing or mismatched ({})",
            normalize_abs_lexical(&wt.join(".git").to_string_lossy())
        );

        std::fs::remove_file(&back).unwrap();
        match resolve_roots_core(&wt) {
            Resolution::LinkInvalid { message } => assert_eq!(message, expected),
            _ => panic!("expected link-invalid (missing back-pointer)"),
        }

        std::fs::write(&back, tmp.path().join("elsewhere").join(".git").to_string_lossy().as_ref())
            .unwrap();
        match resolve_roots_core(&wt) {
            Resolution::LinkInvalid { message } => assert_eq!(message, expected),
            _ => panic!("expected link-invalid (mismatched back-pointer)"),
        }

        // Dangling gitdir: correct namespace shape, no such directory.
        let wt_g = tmp.path().join("wt-g");
        std::fs::create_dir_all(&wt_g).unwrap();
        std::fs::write(
            wt_g.join(".git"),
            format!(
                "gitdir: {}",
                main.join(".git").join("worktrees").join("ghost").to_string_lossy()
            ),
        )
        .unwrap();
        match resolve_roots_core(&wt_g) {
            Resolution::LinkInvalid { message } => assert_eq!(
                message,
                format!(
                    "linked worktree reverse gitdir pointer is missing or mismatched ({})",
                    normalize_abs_lexical(&wt_g.join(".git").to_string_lossy())
                )
            ),
            _ => panic!("expected link-invalid (dangling gitdir)"),
        }
    }

    /// Node (pinned): a gitdir outside `<something>/.git/worktrees/<id>` is
    /// "outside the expected .git/worktrees namespace" — including the bare
    /// `gitdir:` case, which resolves to the worktree root itself. (The hook
    /// flavor calls this shape ORDINARY; the CLI flavor throws. Both are
    /// intentional — see this file's header.)
    #[test]
    fn wrong_namespace_is_link_invalid() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("wt-d");
        std::fs::create_dir_all(tmp.path().join("ns").join("notworktrees").join("x")).unwrap();
        std::fs::create_dir_all(&wt).unwrap();
        let expected = format!(
            "linked worktree gitdir is outside the expected .git/worktrees namespace ({})",
            normalize_abs_lexical(&wt.join(".git").to_string_lossy())
        );
        std::fs::write(
            wt.join(".git"),
            format!(
                "gitdir: {}",
                tmp.path().join("ns").join("notworktrees").join("x").to_string_lossy()
            ),
        )
        .unwrap();
        match resolve_roots_core(&wt) {
            Resolution::LinkInvalid { message } => assert_eq!(message, expected),
            _ => panic!("expected link-invalid (namespace)"),
        }

        // `gitdir:` with no payload -> path.resolve(base, '') === base.
        std::fs::write(wt.join(".git"), "gitdir:").unwrap();
        match resolve_roots_core(&wt) {
            Resolution::LinkInvalid { message } => assert_eq!(message, expected),
            _ => panic!("expected link-invalid (empty payload)"),
        }
    }

    /// Node (pinned): an empty `.git` file is "gitdir is missing or
    /// malformed".
    #[test]
    fn empty_git_file_is_link_invalid() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("wt-e");
        std::fs::create_dir_all(&wt).unwrap();
        std::fs::write(wt.join(".git"), "   \n").unwrap();
        match resolve_roots_core(&wt) {
            Resolution::LinkInvalid { message } => assert_eq!(
                message,
                format!(
                    "linked worktree gitdir is missing or malformed ({})",
                    normalize_abs_lexical(&wt.join(".git").to_string_lossy())
                )
            ),
            _ => panic!("expected link-invalid (empty .git file)"),
        }
    }

    /// A RELATIVE gitdir pointer (and a back-pointer written without the
    /// `gitdir:` prefix, which is how git itself writes it) still validates.
    #[test]
    fn relative_gitdir_pointer_validates() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, wt) = fixture(tmp.path(), "wt-h");
        std::fs::write(wt.join(".git"), "gitdir: ../main/.git/worktrees/wt-h").unwrap();
        match resolve_roots_core(&wt) {
            Resolution::LinkedValid { id, main_root, .. } => {
                assert_eq!(id, "wt-h");
                assert_eq!(norm(&main_root), norm(&main));
            }
            _ => panic!("expected linked-valid for a relative pointer"),
        }
    }

    /// Every existing caller keeps today's behavior: a linked worktree —
    /// valid or broken — still routes to Node through `resolve_store_root`.
    #[test]
    fn resolve_store_root_still_delegates_for_linked_worktrees() {
        let tmp = tempfile::tempdir().unwrap();
        let (_main, wt) = fixture(tmp.path(), "wt-a");
        assert!(matches!(resolve_store_root(&wt), Roots::NeedsNode));
        std::fs::write(wt.join(".git"), "gitdir: nowhere").unwrap();
        assert!(matches!(resolve_store_root(&wt), Roots::NeedsNode));
    }
}
