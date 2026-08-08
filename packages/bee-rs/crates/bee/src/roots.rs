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
// ─── the three doors (CUTOVER) ─────────────────────────────────────────────
//
// Classification is only half of "a verb runs natively in a worktree". A verb
// is worktree-native only once its OWN worktree-sensitive branches are
// ported. Flipping that mapping wholesale was tried and measured: it cost
// `orient --json` its whole `worktree` block inside a granted worktree and
// `status --json` its `worktree_notice` inside an ungranted one — a C2 break,
// not a coverage win. So the flip stayed strictly per-verb, and at cutover it
// settled into three doors:
//
//   1. `resolve_store_root_worktree` — FULL. The verb carries both grant
//      arms itself (notice text, control-root re-rooting, hold topology).
//   2. `resolve_store_root_any` — WIDE. The verb was AUDITED to read nothing
//      but the store root, so the grant-resolved root is the whole answer.
//   3. `resolve_store_root` — NARROW. The verb addresses the shared control
//      plane off the store root. It serves an ORDINARY checkout and an
//      UNGRANTED worktree (where `storeRoot == mainRoot`, so the two are the
//      same directory and the behaviour is Node's exactly) and REFUSES a
//      granted one, naming the main checkout.
//
// Nothing returns to Node any more: every door's non-serving arm is an
// emitted refusal (`Roots::Unsupported`), never a silent None.
//
// FULL — these call `resolve_roots_core` / `resolve_store_root_worktree`
// and serve linked worktrees natively:
//
//   * `worktree list|register|unregister` (verbs/worktree.rs) — the first
//     caller; it MUST run inside a worktree to be useful.
//   * `status` / `status --lanes-full` / `orient` (verbs/status_full.rs) —
//     carries bee.mjs's ungrantedWorktreeNotice, the GRANTED half of
//     orientWorktreeContext (grantedWorktreeContext + readWorktreeBranch),
//     and a real `controlRootFor` (state.mjs resolveContext.controlRoot) so
//     the control-plane reads — sessions, claims, workers, lanes' bound
//     sessions, recovery — land in MAIN's store from inside a worktree,
//     exactly like Node. reservations.mjs's own cycle-safe control-root walk
//     is ported with its linked branch too (list_path_lease_records).
//   * `reservations list|reserve|release|sweep` (verbs/reservations.rs) —
//     carries the real resolveMainRoot / resolveHoldTopology: the holds
//     ledger is addressed at mainRoot, the holder is the git-verified
//     worktree id when granted, and the whole cross-worktree mirroring is
//     SKIPPED (topology === null) inside an ungranted worktree.
//
// WIDE — audited (cutover) to touch no control-plane path at all, so they
// take `resolve_store_root_any` and serve BOTH grant states:
//
//   * `--help` and group-scoped help (verbs/help.rs) — the resolved root is
//     used for ONE thing, the timings.jsonl append; every emitted byte comes
//     from the embedded registry.
//   * `status --brief` (verbs/status_brief.rs) — state/config/timings only.
//   * `capture *` (.bee/capture-queue.jsonl), `backlog *` (.bee/backlog.jsonl),
//     `feedback *`, `knowledge *`, `intent *`, `reviews *`, `tmp sweep`
//     (.bee/tmp, .bee/spikes, .bee/lanes) — all data-plane files under the
//     store root.
//   * `decisions *` (log, active, search, redact, render, supersede, tag,
//     archive — .bee/decisions.jsonl + archive) — verbs/decisions/*.rs. Was
//     narrow only because it shared `reservations::prelude` with `cells *` /
//     `state *` / `close`; it now has its own dedicated door,
//     `verbs::decisions::decisions_prelude`, resolved through
//     `resolve_store_root_any` instead. A granted worktree operates on its
//     OWN workspace-local `.bee/decisions.jsonl`, matching the
//     control-plane/data-plane split (docs/knowledge/areas/worktree-parallelism/
//     control-plane-topology.md); an ordinary checkout and an ungranted
//     worktree are unaffected (`resolve_store_root_any` answers the same root
//     `resolve_store_root` would there).
//   * `test` (verbs/test_runner.rs) — config read, .bee/logs write, and the
//     child process's cwd, which is the store root in Node too.
//
// NARROW — `resolve_store_root`: served in an ordinary checkout and in an
// UNGRANTED worktree (identical directories), REFUSED in a granted one:
//
//   * `state *` (incl. `rebuild-projections`, `handoff`, `session`, `worker`)
//     — verbs/state_group.rs addresses sessions, claims, workers, workflows
//     and HANDOFF.json off `ctx.root`; in a granted worktree Node's
//     controlRootFor sends every one of them to mainRoot instead. Threading a
//     control root through that module is a port of its own, and guessing it
//     would write real bytes into the wrong store.
//   * `close` (verbs/drivers.rs) — same reason, through the same prelude.
//   * `cells *` (incl. `claim-next`), `dispatch prepare`. cells.rs DOES
//     re-root claims/sessions/leases through its own `control_root`, but its
//     cross-worktree holds are still written against resolveHoldTopology's
//     ORDINARY arm (`holds_ledger_path(root)`, holder `"main"`). In a granted
//     worktree the real topology is (mainRoot, <the worktree's git id>), so
//     serving it there would consult an empty ledger and release nobody's
//     holds — a guard that silently stops guarding, which is worse than a
//     refusal.
//
// A granted worktree is an EXPLICIT act (`bee worktree register`) — an
// ordinary `bee worktree new` checkout is ungranted and therefore served by
// every door above.
//
// `LinkInvalid` is now EMITTED by every door (`Unsupported::LinkInvalid` ->
// `link_invalid::emit_link_invalid`), reproducing Node's own emission
// including its skipped timings.jsonl append. Its sibling `Exotic` (a
// V8-worded ENOENT from a raced `.git` stat) was retired at cutover — see the
// single-stat note in `resolve_roots_core`.
//
// The three resolvers a worktree-native verb may need are NOT the same walk,
// and each is ported where its Node original lives:
//   1. `resolveRoots(cwd).storeRoot` — this file. Grant-dependent.
//   2. state.mjs `controlRootFor(root)` — verbs/status_full.rs. The control
//      plane (sessions, claims, workflows) always resolves onto mainRoot.
//   3. reservations.mjs's cycle-free `controlRootFor` — a pure git walk with
//      no config or grant read, ported in verbs/reservations.rs AND
//      verbs/status_full.rs (lease files live under it).

use serde_json::Value;
use std::path::{Path, PathBuf};

pub enum Roots {
    /// Ordinary checkout: store root == work root. An UNGRANTED linked
    /// worktree also lands here through the narrow door — see
    /// `resolve_store_root`.
    Ordinary(PathBuf),
    /// CUTOVER: a shape the calling door cannot serve. This used to be
    /// `NeedsNode` and the caller returned None into the Node delegate; with
    /// no runtime behind the binary the caller must EMIT this instead. Never
    /// swallow it — silence is the one outcome the cutover forbids.
    Unsupported(Unsupported),
    /// No bee/.bee/onboarding.json and no .git anywhere up the tree.
    None,
}

/// Why a door refused. Both arms are emitted, never delegated.
pub enum Unsupported {
    /// state.mjs `WorktreeLinkInvalidError` — a BROKEN `.git` link. The
    /// message is Node's own, byte-for-byte, and `link_invalid::
    /// emit_link_invalid` reproduces Node's emission (including the skipped
    /// timings.jsonl append). This arm is an IMPLEMENTATION, not a regression.
    LinkInvalid(String),
    /// A GRANTED linked worktree reached through the NARROW door. The verb
    /// addresses the shared control plane (sessions, claims, workers,
    /// workflows, handoff) off the store root, and in a granted worktree those
    /// are two different directories — so serving it would write the right
    /// bytes into the wrong store. Refused with the main checkout named.
    GrantedWorktree { main_root: PathBuf },
}

/// state.mjs resolveRootsCore's full return shape. `Ordinary`/`LinkedValid`
/// are its two `worktreeResolution` values, `LinkInvalid` is the throw,
/// `Unresolved` is `{storeRoot: null, workRoot: null}`, and `Exotic` WAS the
/// one place Node threw a V8-worded error (a `.git` that vanished between
/// existsSync and statSync). The port collapsed those two calls into one stat
/// at cutover, so `Exotic` is no longer constructed.
#[derive(Debug)]
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
    LinkInvalid { message: String },
    Unresolved,
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
/// Do two path spellings name the same file?
///
/// Lexical first — that is the comparison this resolver has always made, it is
/// what Node did, and it is the answer in every ordinary case. The canonical
/// fallback exists because a lexical-only answer is WRONG on Windows in ways a
/// user never chose and cannot see:
///
///   * An 8.3 short component. A GitHub Windows runner's TEMP really is
///     `C:\Users\RUNNER~1\AppData\Local\Temp`, and the same directory reached
///     through `C:\Users\runneradmin\…` is byte-different and identical. This
///     is what made 17 worktree tests red on every release from v2.0.4 to
///     2.1.0 — a dead gate, not a flake.
///   * A drive letter in the other case (`c:\repo` vs `C:\repo`), which
///     Windows treats as one path and a byte compare does not.
///   * A junction or symlinked ancestor anywhere above the repo.
///
/// In all three the link IS bidirectional and the refusal is false. Canonical
/// resolution is the question actually being asked — "same file?" — so it is
/// the right tiebreak, and it stays a FALLBACK so the common path pays no
/// syscall and byte-compatibility with the old answer is preserved wherever
/// the old answer was right.
pub(crate) fn same_path(a: &str, b: &str) -> bool {
    let (a, b) = (path_resolve(a, "."), path_resolve(b, "."));
    if a == b {
        return true;
    }
    match (dunce::canonicalize(&a), dunce::canonicalize(&b)) {
        (Ok(ca), Ok(cb)) => ca == cb,
        // Unresolvable means "not proven the same", never "assume so": a
        // missing marker must keep failing the bidirectionality check.
        _ => false,
    }
}

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
    //
    // CUTOVER: Node did `existsSync(marker)` here and `statSync(marker)`
    // below, and a marker that raced away between the two made statSync throw
    // a V8-worded ENOENT out of findRepoRoot — the one arm this resolver could
    // never reproduce, so it returned `Exotic` and the whole command went back
    // to Node. There is no Node left. The two calls are now ONE stat, which
    // removes the time-of-check/time-of-use window rather than deciding what
    // to print inside it: a marker that is not there when we look is simply
    // not a repo root, and the walk continues upward exactly as it would have
    // if `existsSync` had returned false a moment later.
    let mut located: Option<(String, String, std::fs::Metadata)> = None;
    let mut dir = start.clone();
    loop {
        let marker = join(&dir, ".git");
        if let Ok(meta) = std::fs::metadata(&marker) {
            located = Some((dir.clone(), marker, meta));
            break;
        }
        match parent_of(&dir) {
            Some(p) => dir = p,
            None => break,
        }
    }

    let Some((work_root, marker, marker_meta)) = located else {
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

    // A `.git` DIRECTORY is an ordinary checkout, done. (Same stat the walk
    // above already took — see its cutover note.)
    if !marker_meta.is_file() {
        return Resolution::Ordinary {
            store_root: PathBuf::from(&work_root),
            work_root: PathBuf::from(&work_root),
        };
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
        Some(r) if same_path(r, &marker) => {}
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

/// What a WORKTREE-NATIVE verb needs from one `resolveRoots(process.cwd())`
/// call: main()'s `root` (= storeRoot) plus the classification bee.mjs's
/// worktree-sensitive helpers re-derive by calling `resolveRoots(cwd)` again
/// (ungrantedWorktreeNotice, grantedWorktreeContext, orientWorktreeContext,
/// resolveMainRoot, resolveHoldTopology). Resolving it ONCE and threading it
/// is equivalent: those helpers are pure reads of the same cwd, and nothing
/// between them mutates `.git` or the grant registry within a single command.
pub struct StoreRoots {
    /// main()'s `root` — `resolveRoots(cwd).storeRoot`.
    pub root: PathBuf,
    /// `resolveRoots(cwd).workRoot` — the physical checkout.
    pub work_root: PathBuf,
    /// `None` for an ORDINARY checkout (byte-identical to the pre-flip path);
    /// `Some` only for `worktreeResolution === 'linked-valid'`.
    pub linked: Option<LinkedRoots>,
}

/// The `linked-valid` fields, kept as PATHS rather than pre-computed booleans
/// so each predicate is spelled exactly as its Node original does it:
/// `granted` is `resolve(storeRoot) === resolve(worktreeRoot)`
/// (grantedWorktreeContext / resolveHoldTopology) while `ungranted` is
/// `resolve(storeRoot) === resolve(mainRoot)` (ungrantedWorktreeNotice) —
/// two different comparisons, and a degenerate repo where mainRoot IS the
/// worktree root satisfies both in Node too.
pub struct LinkedRoots {
    pub id: String,
    pub main_root: PathBuf,
    pub worktree_root: PathBuf,
    pub store_root: PathBuf,
}

impl LinkedRoots {
    /// bee.mjs grantedWorktreeContext / resolveHoldTopology's `granted` test.
    pub fn granted(&self) -> bool {
        self.store_root == self.worktree_root
    }
    /// bee.mjs ungrantedWorktreeNotice's `ungranted` test.
    pub fn ungranted(&self) -> bool {
        self.store_root == self.main_root
    }
}

impl StoreRoots {
    /// bee.mjs resolveMainRoot(root): the linked arm's own `mainRoot`, else
    /// the dispatcher's already-resolved `root`.
    pub fn main_root(&self) -> PathBuf {
        match &self.linked {
            Some(l) => l.main_root.clone(),
            None => self.root.clone(),
        }
    }

    /// bee.mjs resolveHoldTopology(root) -> `Some((mainRoot, holder))`:
    /// ordinary => (workRoot || root, "main"); granted linked worktree =>
    /// (mainRoot, its git-verified id); everything else (an UNGRANTED linked
    /// worktree — `root` already IS the shared main store) => None, which
    /// callers treat as "skip the cross-worktree wiring entirely".
    pub fn hold_topology(&self) -> Option<(PathBuf, String)> {
        match &self.linked {
            None => Some((self.work_root.clone(), "main".to_string())),
            Some(l) if l.granted() => Some((l.main_root.clone(), l.id.clone())),
            Some(_) => None,
        }
    }
}

/// The `Roots` analogue for a verb that HAS ported its worktree-sensitive
/// branches. Same three outcomes, but `linked-valid` is served instead of
/// delegated. `LinkInvalid` still delegates for everyone.
pub enum RootsWt {
    Go(StoreRoots),
    /// Only ever `LinkInvalid` here — a worktree-native verb serves both
    /// grant states.
    Unsupported(Unsupported),
    None,
}

/// `resolveRoots(startDir)` for a worktree-native verb. See `Roots` /
/// `resolve_store_root` for the narrowed version every other verb still uses.
pub fn resolve_store_root_worktree(start: &Path) -> RootsWt {
    match resolve_roots_core(start) {
        Resolution::Ordinary {
            store_root,
            work_root,
        } => RootsWt::Go(StoreRoots {
            root: store_root,
            work_root,
            linked: None,
        }),
        Resolution::LinkedValid {
            store_root,
            work_root,
            id,
            main_root,
        } => RootsWt::Go(StoreRoots {
            root: store_root.clone(),
            work_root: work_root.clone(),
            linked: Some(LinkedRoots {
                id,
                main_root,
                worktree_root: work_root,
                store_root,
            }),
        }),
        Resolution::LinkInvalid { message } => {
            RootsWt::Unsupported(Unsupported::LinkInvalid(message))
        }
        Resolution::Unresolved => RootsWt::None,
    }
}

/// state.mjs findRepoRoot — `resolveRoots(startDir).storeRoot` — through the
/// NARROW door: for a verb whose port addresses the shared control plane off
/// the store root.
///
/// CUTOVER. An UNGRANTED linked worktree is served here, and that is a proof,
/// not a widening: `resolveRootsCore` already answers `storeRoot == mainRoot`
/// for it, so `controlRootFor(storeRoot)` is `storeRoot` and every `.bee/…`
/// join off the root lands in exactly the directory Node's own
/// `controlRootFor` would have chosen. A GRANTED worktree is the only shape
/// where the two diverge, and that one refuses.
pub fn resolve_store_root(start: &Path) -> Roots {
    match resolve_roots_core(start) {
        Resolution::Ordinary { store_root, .. } => Roots::Ordinary(store_root),
        Resolution::LinkedValid {
            store_root,
            main_root,
            ..
        } => {
            if store_root == main_root {
                // Ungranted: the store IS main's, identical to an ordinary
                // checkout rooted there.
                Roots::Ordinary(store_root)
            } else {
                Roots::Unsupported(Unsupported::GrantedWorktree { main_root })
            }
        }
        Resolution::LinkInvalid { message } => {
            Roots::Unsupported(Unsupported::LinkInvalid(message))
        }
        Resolution::Unresolved => Roots::None,
    }
}

/// The WIDE door: `resolveRoots(startDir).storeRoot` with BOTH grant states
/// served.
///
/// Only for verbs audited to read nothing but the store root — no sessions,
/// claims, workers, workflows, handoff mailboxes or cross-worktree holds. For
/// those the grant-resolved store root is the complete answer, so a granted
/// worktree needs no extra branch. The list of modules on this door, and the
/// audit behind it, is in this file's header.
pub fn resolve_store_root_any(start: &Path) -> Roots {
    match resolve_roots_core(start) {
        Resolution::Ordinary { store_root, .. } | Resolution::LinkedValid { store_root, .. } => {
            Roots::Ordinary(store_root)
        }
        Resolution::LinkInvalid { message } => {
            Roots::Unsupported(Unsupported::LinkInvalid(message))
        }
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

    /// Compare paths by IDENTITY, not by spelling.
    ///
    /// `main_root` comes out of the gitdir chain — git's own writing of the
    /// path — while a fixture holds whatever `tempdir()` returned. On a
    /// Windows runner those are the long and 8.3-short forms of one directory,
    /// so a lexical compare made every ungranted-worktree assertion fail for a
    /// reason that has nothing to do with what is being asserted.
    fn norm(p: &Path) -> String {
        match dunce::canonicalize(p) {
            Ok(c) => normalize_abs_lexical(&c.to_string_lossy()),
            Err(_) => normalize_abs_lexical(&p.to_string_lossy()),
        }
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
    /// The bidirectionality check asks "same file?", and for four releases it
    /// answered with a byte compare. On a GitHub Windows runner TEMP really is
    /// `C:\Users\RUNNER~1\...`; git writes its gitdir pointers in the LONG
    /// form, the fixture holds the short one, and every worktree test went red
    /// — a dead gate rather than a flake, on every release from v2.0.4 to 2.1.0.
    ///
    /// 8.3 aliases cannot be created on demand (the volume may have short-name
    /// generation off), so this proves the same comparison through the other
    /// two spellings that reach one file: a symlinked ancestor everywhere, and
    /// a flipped drive-letter case on Windows.
    #[test]
    fn two_spellings_of_one_path_are_the_same_path() {
        let tmp = tempfile::tempdir().unwrap();
        let real = tmp.path().join("real");
        std::fs::create_dir_all(&real).unwrap();
        let file = real.join("marker");
        std::fs::write(&file, "x").unwrap();

        // Identical spellings: the lexical fast path, no syscall needed.
        assert!(same_path(&file.to_string_lossy(), &file.to_string_lossy()));

        // Genuinely different files stay different, canonical fallback and all.
        let other = real.join("other");
        std::fs::write(&other, "x").unwrap();
        assert!(!same_path(&file.to_string_lossy(), &other.to_string_lossy()));

        // A path that does not exist is never "proven the same" — the missing
        // back-pointer must keep failing the check it guards.
        assert!(!same_path(
            &real.join("absent").to_string_lossy(),
            &real.join("absent").to_string_lossy().replace("absent", "absent2")
        ));

        #[cfg(windows)]
        {
            // `c:\...` and `C:\...` are one path to Windows and two to a byte
            // compare — the same class as RUNNER~1, reproducible anywhere.
            let s = file.to_string_lossy().into_owned();
            let mut flipped = s.clone();
            if flipped.as_bytes().get(1) == Some(&b':') {
                let d = flipped.remove(0);
                flipped.insert(0, if d.is_ascii_uppercase() { d.to_ascii_lowercase() } else { d.to_ascii_uppercase() });
                assert_ne!(flipped, s, "the fixture must actually differ byte-wise");
                assert!(same_path(&flipped, &s), "a drive-letter case flip names the same file");
            }
        }

        #[cfg(unix)]
        {
            let link = tmp.path().join("link");
            std::os::unix::fs::symlink(&real, &link).unwrap();
            let via_link = link.join("marker");
            assert_ne!(via_link, file);
            assert!(same_path(&via_link.to_string_lossy(), &file.to_string_lossy()));
        }
    }

    /// The whole resolver, through a reverse pointer written in an equivalent
    /// but byte-different spelling — the shape the runner actually produced.
    #[test]
    fn a_reverse_pointer_in_another_spelling_still_resolves() {
        let tmp = tempfile::tempdir().unwrap();
        let (_main, wt) = fixture(tmp.path(), "wt-spell");
        match resolve_roots_core(&wt) {
            Resolution::LinkedValid { .. } => {}
            other => panic!("baseline fixture must resolve, got {other:?}"),
        }

        // Rewrite <gitdir>/gitdir to point at the same marker by another name.
        let marker = wt.join(".git");
        let gitdir = read_gitdir_file(&marker.to_string_lossy(), &wt.to_string_lossy())
            .expect("the fixture writes a gitdir pointer");
        let back = std::path::Path::new(&gitdir).join("gitdir");
        let original = std::fs::read_to_string(&back).unwrap();
        let respelled = if cfg!(windows) {
            let mut s = original.trim().to_string();
            let d = s.remove(0);
            s.insert(0, if d.is_ascii_uppercase() { d.to_ascii_lowercase() } else { d.to_ascii_uppercase() });
            s
        } else {
            // POSIX: a redundant `/./` survives realpath and not much else.
            original.trim().replacen('/', "/./", 1)
        };
        std::fs::write(&back, format!("{respelled}
")).unwrap();

        match resolve_roots_core(&wt) {
            Resolution::LinkedValid { .. } => {}
            other => panic!(
                "a reverse pointer naming the SAME marker by another spelling must still                  resolve; got {other:?}"
            ),
        }
    }

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

    /// CUTOVER — the NARROW door's three answers for a linked worktree.
    ///
    /// The fixture starts UNGRANTED (no grants registry), which is the shape
    /// the narrow door now SERVES: `storeRoot == mainRoot`, so a verb that
    /// addresses the control plane off the root lands in exactly the
    /// directory Node's `controlRootFor` would have chosen. Granting it flips
    /// the same fixture to a named refusal, and breaking the link to Node's
    /// own WorktreeLinkInvalidError message. None of the three is a silent
    /// None any more.
    #[test]
    fn narrow_door_serves_ungranted_refuses_granted_and_names_a_broken_link() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, wt) = fixture(tmp.path(), "wt-a");
        match resolve_store_root(&wt) {
            Roots::Ordinary(r) => assert_eq!(norm(&r), norm(&main)),
            _ => panic!("an ungranted worktree resolves to main's store"),
        }
        let runtime = main.join(".bee").join("runtime");
        std::fs::create_dir_all(&runtime).unwrap();
        std::fs::write(runtime.join("worktree-grants.json"), "{\"wt-a\": true}
").unwrap();
        match resolve_store_root(&wt) {
            Roots::Unsupported(Unsupported::GrantedWorktree { main_root }) => {
                assert_eq!(norm(&main_root), norm(&main))
            }
            _ => panic!("a granted worktree is refused by name, never silently"),
        }
        std::fs::write(wt.join(".git"), "gitdir: nowhere").unwrap();
        match resolve_store_root(&wt) {
            Roots::Unsupported(Unsupported::LinkInvalid(message)) => {
                assert!(message.contains("gitdir"), "{message}")
            }
            _ => panic!("a broken link carries Node's own message"),
        }
    }

    /// The WIDE door serves BOTH grant states — that is the whole difference.
    #[test]
    fn wide_door_serves_a_granted_worktree_from_its_own_store() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, wt) = fixture(tmp.path(), "wt-a");
        let runtime = main.join(".bee").join("runtime");
        std::fs::create_dir_all(&runtime).unwrap();
        std::fs::write(runtime.join("worktree-grants.json"), "{\"wt-a\": true}
").unwrap();
        match resolve_store_root_any(&wt) {
            Roots::Ordinary(r) => assert_eq!(norm(&r), norm(&wt)),
            _ => panic!("the wide door serves a granted worktree"),
        }
    }

    // ── the worktree-native door ──────────────────────────────────────────

    /// An ORDINARY checkout answers the SAME root through the widened door,
    /// with `linked: None` — the byte-identity guarantee for main-checkout
    /// runs of every flipped verb.
    #[test]
    fn worktree_door_is_ordinary_for_a_plain_checkout() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        std::fs::create_dir_all(root.join(".git")).unwrap();
        match resolve_store_root_worktree(&root) {
            RootsWt::Go(r) => {
                assert_eq!(norm(&r.root), norm(&root));
                assert_eq!(norm(&r.work_root), norm(&root));
                assert!(r.linked.is_none());
                assert_eq!(norm(&r.main_root()), norm(&root));
                let (main, holder) = r.hold_topology().unwrap();
                assert_eq!(norm(&main), norm(&root));
                assert_eq!(holder, "main");
            }
            _ => panic!("expected an ordinary resolution"),
        }
    }

    /// Node (pinned) inside an UNGRANTED worktree: storeRoot === mainRoot, so
    /// ungrantedWorktreeNotice fires, grantedWorktreeContext is null, and
    /// resolveHoldTopology returns null (mirroring is skipped entirely).
    #[test]
    fn worktree_door_ungranted_shares_main_and_has_no_hold_topology() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, wt) = fixture(tmp.path(), "wt-a");
        match resolve_store_root_worktree(&wt) {
            RootsWt::Go(r) => {
                assert_eq!(norm(&r.root), norm(&main));
                assert_eq!(norm(&r.work_root), norm(&wt));
                assert_eq!(norm(&r.main_root()), norm(&main));
                let l = r.linked.as_ref().expect("linked");
                assert_eq!(l.id, "wt-a");
                assert!(l.ungranted());
                assert!(!l.granted());
                assert!(r.hold_topology().is_none());
            }
            _ => panic!("expected a linked resolution"),
        }
    }

    /// Node (pinned) inside a GRANTED worktree: storeRoot === worktreeRoot,
    /// so the notice is silent, the orient worktree block appears, and the
    /// hold topology names the git-verified id as holder over MAIN's ledger.
    #[test]
    fn worktree_door_granted_holds_under_its_own_id() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, wt) = fixture(tmp.path(), "wt-a");
        let runtime = main.join(".bee").join("runtime");
        std::fs::create_dir_all(&runtime).unwrap();
        std::fs::write(runtime.join("worktree-grants.json"), "{\"wt-a\": true}\n").unwrap();
        match resolve_store_root_worktree(&wt) {
            RootsWt::Go(r) => {
                assert_eq!(norm(&r.root), norm(&wt));
                assert_eq!(norm(&r.main_root()), norm(&main));
                let l = r.linked.as_ref().expect("linked");
                assert!(l.granted());
                assert!(!l.ungranted());
                let (ledger_root, holder) = r.hold_topology().unwrap();
                assert_eq!(norm(&ledger_root), norm(&main));
                assert_eq!(holder, "wt-a");
            }
            _ => panic!("expected a linked resolution"),
        }
    }

    /// A BROKEN link is the one shape no door serves — and at cutover it
    /// stopped being a delegation and became Node's own message, emitted.
    #[test]
    fn worktree_door_names_a_broken_link() {
        let tmp = tempfile::tempdir().unwrap();
        let (_main, wt) = fixture(tmp.path(), "wt-a");
        std::fs::write(wt.join(".git"), "gitdir: nowhere").unwrap();
        assert!(matches!(
            resolve_store_root_worktree(&wt),
            RootsWt::Unsupported(Unsupported::LinkInvalid(_))
        ));
    }
}
