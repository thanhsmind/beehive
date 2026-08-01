// onboard::util — the small primitives onboard_bee.mjs opens with
// (l. 335–375, 415–421, 1663–1668) plus the Node `path`/`fs` semantics the
// rest of the port leans on.
//
// crate::fsutil already carries readJson/writeJsonAtomic for `.bee/` state;
// onboarding writes plain TEXT with its OWN `<file>.tmp` naming
// (writeFileAtomic, l. 349) and hashes UTF-8 STRING content (lib/fsutil.mjs
// hashFile), so those two live here rather than being bent into fsutil.

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

// ── hashing ────────────────────────────────────────────────────────────────

/// onboard_bee.mjs `sha256(text)` (l. 341) — hex sha256 of a STRING's UTF-8
/// bytes.
pub fn sha256_str(text: &str) -> String {
    let mut h = Sha256::new();
    h.update(text.as_bytes());
    format!("{:x}", h.finalize())
}

/// walkSkillTree hashes raw Buffers (l. 655), never the decoded string —
/// a separate hasher from sha256_str on purpose.
pub fn sha256_bytes(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

/// lib/fsutil.mjs hashFile: `sha256(readFileSync(file, 'utf8'))`. utf8 —
/// NOT the raw buffer — so an invalid byte sequence hashes as U+FFFD on both
/// runtimes (`String::from_utf8_lossy` is Node's utf8 decode).
pub fn hash_file(file: &Path) -> Option<String> {
    let bytes = std::fs::read(file).ok()?;
    Some(sha256_str(&String::from_utf8_lossy(&bytes)))
}

// ── text/json IO ───────────────────────────────────────────────────────────

/// onboard_bee.mjs readTextIfExists (l. 345). Divergence (documented): Node
/// THROWS on an unreadable-but-existing path (EISDIR/EACCES) and main()'s
/// catch turns that into `{"error": "<V8 message>"}`; reproducing a V8
/// message is a campaign non-goal, so the port reads such a path as "" —
/// see the module header's divergence list.
pub fn read_text_if_exists(file: &Path) -> String {
    match std::fs::read(file) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(_) => String::new(),
    }
}

/// onboard_bee.mjs readJsonIfExists (l. 356): whitespace-only or unparseable
/// content is `null`, never a throw.
pub fn read_json_if_exists(file: &Path) -> Option<serde_json::Value> {
    let text = read_text_if_exists(file);
    if text.trim().is_empty() {
        return None;
    }
    serde_json::from_str(&text).ok()
}

/// onboard_bee.mjs writeFileAtomic (l. 349): mkdir -p, write `<file>.tmp`,
/// rename. The tmp NAME is part of the observable filesystem behaviour on a
/// crash, so it is reproduced exactly rather than borrowed from
/// fsutil::write_json_atomic's pid-counter-random shape.
pub fn write_file_atomic(file: &Path, content: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut tmp = file.as_os_str().to_os_string();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, content)?;
    // Node's fs.renameSync overwrites an existing destination on both
    // platforms; std::fs::rename matches that (MOVEFILE_REPLACE_EXISTING).
    std::fs::rename(&tmp, file)
}

/// onboard_bee.mjs writeFileAtomicRandom (l. 1663): unpredictable tmp name
/// inside the managed skill namespace (F6 — a predictable `<file>.tmp` under
/// ~/.claude/skills would be a symlink-swap target). 8 random bytes, hex.
pub fn write_file_atomic_random(file: &Path, content: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut tmp = file.as_os_str().to_os_string();
    tmp.push(format!(".{}.tmp", random_hex_16()));
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, content)?;
    let res = std::fs::rename(&tmp, file);
    if res.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    res
}

/// 8 bytes of unpredictability as 16 hex chars — crypto.randomBytes(8)'s
/// role here is collision/guess resistance for a tmp name, not secrecy.
fn random_hex_16() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let mix = nanos
        ^ (u64::from(std::process::id()) << 32)
        ^ COUNTER.fetch_add(0x9E37_79B9_7F4A_7C15, Ordering::Relaxed);
    // One xorshift round so consecutive calls don't share a visible prefix.
    let mut x = mix | 1;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    format!("{x:016x}")
}

// ── lstat ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub struct Lstat {
    pub is_symlink: bool,
    pub is_dir: bool,
    pub is_file: bool,
}

/// onboard_bee.mjs lstatIfExists (l. 415) — never follows links, never
/// throws.
pub fn lstat_if_exists(p: &Path) -> Option<Lstat> {
    let meta = std::fs::symlink_metadata(p).ok()?;
    let ft = meta.file_type();
    Some(Lstat { is_symlink: ft.is_symlink(), is_dir: ft.is_dir(), is_file: ft.is_file() })
}

/// fs.existsSync — follows links (an existsSync on a dangling symlink is
/// false in Node too, which `Path::exists` matches).
pub fn exists(p: &Path) -> bool {
    p.exists()
}

// ── entryIdentity (win32 defect fix, see module header) ────────────────────

/// Physical filesystem identity of ONE path, never following links.
///
/// Ported from onboard_bee.mjs entryIdentity (l. 1008): `${st.dev}:${st.ino}`.
///
/// **BUG FIX (a), filed win32 defect.** Node builds that key from
/// `fs.Stats.dev`/`.ino`, which are JS **Numbers** (IEEE-754 doubles). On
/// win32 libuv fills st_ino from the 64-bit NTFS file index, so every index
/// above 2^53 loses its low bits — two DIFFERENT files can produce the same
/// `dev:ino` string, and detectAliasCollisions then blocks both skills as a
/// "case-insensitive alias" that does not exist (or, symmetrically, a real
/// alias slips through). Rust keeps the volume serial as u64 and the file
/// index as u128, so the identity is exact for every representable index.
///
/// Output shape is a comparable tuple rather than a string: the string was
/// only ever a Map key, never emitted, so this is not an observable
/// divergence.
pub type EntryId = (u64, u128);

#[cfg(windows)]
pub fn entry_identity(p: &Path) -> Option<EntryId> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_DELETE,
        FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    };

    // lstat semantics: OPEN_REPARSE_POINT makes the handle refer to the link
    // itself, BACKUP_SEMANTICS allows opening a directory at all. Zero
    // desired-access = metadata only (works without read permission).
    let wide: Vec<u16> = p.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
            std::ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return None;
    }
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    let ok = unsafe { GetFileInformationByHandle(handle, &mut info) };
    unsafe { CloseHandle(handle) };
    if ok == 0 {
        return None;
    }
    Some((
        u64::from(info.dwVolumeSerialNumber),
        (u128::from(info.nFileIndexHigh) << 32) | u128::from(info.nFileIndexLow),
    ))
}

#[cfg(not(windows))]
pub fn entry_identity(p: &Path) -> Option<EntryId> {
    use std::os::unix::fs::MetadataExt;
    let meta = std::fs::symlink_metadata(p).ok()?;
    Some((meta.dev(), u128::from(meta.ino())))
}

// ── directory listing ──────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Dirent {
    pub name: String,
    pub is_symlink: bool,
    pub is_dir: bool,
    pub is_file: bool,
}

/// `readdirSync(dir, {withFileTypes:true}).sort((a,b)=> a.name<b.name?-1: ...)`
/// — the exact ordering onboard_bee.mjs applies before every walk. Node's
/// Dirent types come from the same lstat-shaped scandir, so entries are typed
/// WITHOUT following links.
pub fn read_dir_sorted(dir: &Path) -> Vec<Dirent> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        out.push(Dirent {
            name,
            is_symlink: ft.is_symlink(),
            is_dir: ft.is_dir(),
            is_file: ft.is_file(),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// `readdirSync` that reports failure, for the two call sites that fail
/// CLOSED on an unreadable directory (computeSkillSyncTarget's
/// installedTreeExists probe).
pub fn read_dir_sorted_checked(dir: &Path) -> Result<Vec<Dirent>, ()> {
    if std::fs::read_dir(dir).is_err() {
        return Err(());
    }
    Ok(read_dir_sorted(dir))
}

// ── Node `path` semantics ──────────────────────────────────────────────────

/// Join a POSIX-ish relative path (`"a/b/c"`, this script's own convention)
/// onto a base, the way `path.join(base, ...rel.split("/"))` does.
pub fn join_rel(base: &Path, rel: &str) -> PathBuf {
    let mut out = base.to_path_buf();
    for part in rel.split('/') {
        if !part.is_empty() {
            out.push(part);
        }
    }
    out
}

fn cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Node `path.resolve(p)` — absolute + lexically normalized (`.`/`..`
/// collapsed, separators folded to the platform's), symlinks NOT resolved.
pub fn path_resolve(p: &str) -> PathBuf {
    let sep = std::path::MAIN_SEPARATOR;
    let cwd = cwd();
    let cwd_s = cwd.to_string_lossy().into_owned();
    let bytes = p.as_bytes();

    let has_drive = bytes.len() >= 2 && bytes[1] == b':';
    let rooted = p.starts_with('/') || p.starts_with('\\');

    let combined: String = if has_drive && bytes.len() >= 3 && (bytes[2] == b'/' || bytes[2] == b'\\')
    {
        p.to_string()
    } else if has_drive {
        // "C:foo" — drive-relative. Node resolves against cwd only when the
        // drive matches; otherwise against that drive's root. Treat it as
        // that drive's root (the conservative, deterministic reading).
        format!("{}{}{}", &p[..2], sep, &p[2..])
    } else if rooted {
        // Root-relative: keep the cwd's drive prefix on win32.
        let drive = if cwd_s.as_bytes().len() >= 2 && cwd_s.as_bytes()[1] == b':' {
            &cwd_s[..2]
        } else {
            ""
        };
        format!("{drive}{p}")
    } else if p.is_empty() {
        cwd_s.clone()
    } else {
        format!("{cwd_s}{sep}{p}")
    };

    PathBuf::from(normalize_lexical(&combined))
}

/// Collapse `.`/`..`, fold separators to the platform's, drop a trailing
/// separator (except at a root) — the tail of Node's path.resolve.
pub fn normalize_lexical(input: &str) -> String {
    let sep = std::path::MAIN_SEPARATOR;
    let bytes = input.as_bytes();
    let (prefix, rest) = if bytes.len() >= 2 && bytes[1] == b':' {
        (input[..2].to_string(), &input[2..])
    } else if input.starts_with("//") || input.starts_with("\\\\") {
        // UNC — keep the double-separator root verbatim, normalize the tail.
        (format!("{sep}{sep}"), &input[2..])
    } else {
        (String::new(), input)
    };
    let leading = rest.starts_with('/') || rest.starts_with('\\');

    let mut parts: Vec<&str> = Vec::new();
    for seg in rest.split(['/', '\\']) {
        match seg {
            "" | "." => {}
            ".." => {
                if parts.last().is_some_and(|p| *p != "..") {
                    parts.pop();
                } else if !leading {
                    parts.push("..");
                }
            }
            other => parts.push(other),
        }
    }
    let joined = parts.join(&sep.to_string());
    if leading {
        format!("{prefix}{sep}{joined}")
    } else if joined.is_empty() {
        prefix
    } else {
        format!("{prefix}{joined}")
    }
}

/// Node `path.basename` on an already-normalized platform path.
pub fn basename(p: &Path) -> String {
    p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

/// `fs.realpathSync` — dunce::canonicalize keeps the plain `C:\…` shape Node
/// returns instead of std's `\\?\C:\…` extended form.
pub fn realpath(p: &Path) -> Option<PathBuf> {
    dunce::canonicalize(p).ok()
}

/// Process-wide sandbox home for the in-file suite. onboard_bee.mjs's own
/// tests isolate by redirecting HOME/USERPROFILE for a SPAWNED process; the
/// Rust suite runs in-process and in parallel, so env mutation would race —
/// a write-once override gives every test the same sandbox instead. Two
/// machine-level surfaces depend on it (~/.claude/skills legacy refresh and
/// ~/.codex/config.toml), and neither may ever be touched by a test run.
#[cfg(test)]
pub static TEST_HOME: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

#[cfg(test)]
fn test_home() -> Option<PathBuf> {
    TEST_HOME.get().cloned()
}
#[cfg(not(test))]
fn test_home() -> Option<PathBuf> {
    None
}

/// os.homedir().
pub fn home_dir() -> PathBuf {
    if let Some(h) = test_home() {
        return h;
    }
    if cfg!(windows) {
        std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)
            .unwrap_or_default()
    } else {
        std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
    }
}

/// `a.startsWith(b + path.sep)` — the containment test this script uses for
/// every overlap guard.
pub fn is_under(child: &Path, ancestor: &Path) -> bool {
    let c = child.to_string_lossy().into_owned();
    let a = ancestor.to_string_lossy().into_owned();
    c.starts_with(&format!("{a}{}", std::path::MAIN_SEPARATOR))
}

// ── JS string helpers ──────────────────────────────────────────────────────

/// JS `str.replace(/\s*$/, "")` — strip the trailing whitespace run.
pub fn trim_trailing_ws(s: &str) -> &str {
    s.trim_end()
}

/// `text.split(/\r\n|\n/)`.
pub fn split_lines(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            let end = if i > 0 && bytes[i - 1] == b'\r' { i - 1 } else { i };
            out.push(&text[start..end]);
            start = i + 1;
        }
        i += 1;
    }
    out.push(&text[start..]);
    out
}

/// splitLinesPreserving (l. 770): [content, terminator] pairs whose
/// concatenation rebuilds the input byte-for-byte.
pub fn split_lines_preserving(text: &str) -> Vec<(&str, &str)> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            let (content_end, term_start) =
                if i > 0 && bytes[i - 1] == b'\r' { (i - 1, i - 1) } else { (i, i) };
            out.push((&text[start..content_end], &text[term_start..=i]));
            start = i + 1;
        }
        i += 1;
    }
    out.push((&text[start..], ""));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_lines_preserving_round_trips() {
        for input in ["", "a", "a\n", "a\r\nb", "a\r\nb\n", "\n\n", "x\ny\r\n"] {
            let joined: String =
                split_lines_preserving(input).iter().map(|(c, t)| format!("{c}{t}")).collect();
            assert_eq!(joined, input, "round trip for {input:?}");
        }
    }

    #[test]
    fn split_lines_matches_js_split() {
        assert_eq!(split_lines("a\r\nb\nc"), vec!["a", "b", "c"]);
        assert_eq!(split_lines(""), vec![""]);
        assert_eq!(split_lines("a\n"), vec!["a", ""]);
    }

    #[test]
    fn normalize_lexical_collapses_dots() {
        let sep = std::path::MAIN_SEPARATOR;
        if cfg!(windows) {
            assert_eq!(normalize_lexical("C:/a/./b/../c"), format!("C:{sep}a{sep}c"));
            assert_eq!(normalize_lexical("C:/a/"), format!("C:{sep}a"));
        } else {
            assert_eq!(normalize_lexical("/a/./b/../c"), "/a/c");
        }
    }

    #[test]
    fn hash_file_hashes_utf8_string_content() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("x.txt");
        std::fs::write(&f, "hello\n").unwrap();
        // Node: sha256(readFileSync(f,'utf8')) for "hello\n".
        assert_eq!(
            hash_file(&f).unwrap(),
            "5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"
        );
        assert_eq!(sha256_str("hello\n"), hash_file(&f).unwrap());
    }

    #[test]
    fn entry_identity_distinguishes_two_files() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        std::fs::write(&a, "1").unwrap();
        std::fs::write(&b, "2").unwrap();
        let ia = entry_identity(&a).unwrap();
        let ib = entry_identity(&b).unwrap();
        assert_ne!(ia, ib, "distinct files must not collide");
        assert_eq!(entry_identity(&a).unwrap(), ia, "identity is stable");
        assert!(entry_identity(&dir.path().join("nope")).is_none());
    }

    #[test]
    fn write_file_atomic_creates_parents_and_leaves_no_tmp() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("deep").join("out.txt");
        write_file_atomic(&f, b"body").unwrap();
        assert_eq!(std::fs::read_to_string(&f).unwrap(), "body");
        assert!(!dir.path().join("deep").join("out.txt.tmp").exists());
    }
}
