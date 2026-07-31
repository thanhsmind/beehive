// path_identity — Rust port of lib/path-identity.mjs (windows-path-identity
// wpi-1): ONE canonical "same directory?" comparison for path strings that
// may disagree on separators and case. The original's corrections carry over:
//
//   1. Case sensitivity is per-VOLUME, probed — never assumed from the OS.
//      Probe results cache per volume (device/volume-serial number when
//      statable, else the ancestor's path string).
//   2. Separator folding is the platform resolver's job (std::path::absolute
//      on the native platform), never a hand-rolled regex. On POSIX a
//      backslash stays ordinary filename data.
//   3. Filesystem identity (dev+ino / volume-serial+file-index) is a fast
//      path, never the only rule: a ZERO device or index (FAT/exFAT, SMB) is
//      UNUSABLE — fall back to normalized-string comparison, never treat it
//      as proof of identity. On Windows the identity numbers come from
//      GetFileInformationByHandle — the same call Node's libuv fills
//      st_dev/st_ino from, so the two runtimes agree on usability.
//   4. Case-folding requires BOTH sides' volumes to probe case-insensitive
//      (folding widens equality; disagreement must fail toward NOT-equal).
//   5. The probe prefers the read-only flip-own-basename check; the
//      write-based marker probe is the rare fallback.
//
// Divergence from the .mjs (documented, deliberate): the JS module's
// dependency injection (statFn/detectCaseFold/platformPath) existed for
// cross-platform unit tests; the Rust tests run on the real platform, so
// injection is limited to what the probe internals need.

#![allow(dead_code)] // consumers arrive with the guards/worktree ports (R2/R3)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::OnceLock;

fn absolute(p: &Path) -> PathBuf {
    std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf())
}

// ── filesystem identity ────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FileId {
    pub dev: u64,
    pub ino: u128,
}

#[cfg(windows)]
pub fn file_id(path: &Path) -> Option<FileId> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
        FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
        OPEN_EXISTING,
    };

    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    // Zero desired-access opens for metadata only; BACKUP_SEMANTICS is what
    // allows opening a directory handle at all.
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
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
    Some(FileId {
        dev: u64::from(info.dwVolumeSerialNumber),
        ino: (u128::from(info.nFileIndexHigh) << 32) | u128::from(info.nFileIndexLow),
    })
}

#[cfg(not(windows))]
pub fn file_id(path: &Path) -> Option<FileId> {
    use std::os::unix::fs::MetadataExt;
    let meta = std::fs::metadata(path).ok()?;
    Some(FileId { dev: meta.dev(), ino: u128::from(meta.ino()) })
}

/// Correction 3: Some(true/false) only when identity is usable on BOTH sides
/// (both exist, no zero device/index); None means "fall back to strings".
fn try_identity_equal(a: &Path, b: &Path) -> Option<bool> {
    let ia = file_id(a)?;
    let ib = file_id(b)?;
    if ia.dev == 0 || ia.ino == 0 || ib.dev == 0 || ib.ino == 0 {
        return None;
    }
    Some(ia == ib)
}

// ── volume case-behaviour probe, cached per volume ─────────────────────────

fn case_fold_cache() -> &'static Mutex<HashMap<String, bool>> {
    static CACHE: OnceLock<Mutex<HashMap<String, bool>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn nearest_existing_ancestor(start: &Path) -> PathBuf {
    let mut current = absolute(start);
    loop {
        if current.exists() {
            return current;
        }
        match current.parent() {
            Some(p) => current = p.to_path_buf(),
            None => return current,
        }
    }
}

/// Flips every ASCII-case-bearing char; None when nothing flipped.
fn flip_ascii_case(s: &str) -> Option<String> {
    let mut changed = false;
    let out: String = s
        .chars()
        .map(|ch| {
            if ch.is_lowercase() {
                changed = true;
                ch.to_uppercase().collect::<String>()
            } else if ch.is_uppercase() {
                changed = true;
                ch.to_lowercase().collect::<String>()
            } else {
                ch.to_string()
            }
        })
        .collect();
    changed.then_some(out)
}

/// Read-only probe (correction 5): stat the dir under a case-flipped spelling
/// of its own basename. None = no attempt possible; caller falls back to the
/// write-based probe.
fn read_only_case_probe(dir: &Path) -> Option<bool> {
    let base = dir.file_name()?.to_str()?;
    let flipped = flip_ascii_case(base)?;
    let flipped_path = dir.parent()?.join(flipped);
    let real = file_id(dir)?;
    match file_id(&flipped_path) {
        None => Some(false), // flipped spelling doesn't resolve -> case-sensitive
        Some(alt) => Some(real == alt),
    }
}

/// Write-based fallback probe: create lower-cased marker, look up upper-cased.
/// Any failure reads as case-SENSITIVE (the narrow, safe default).
fn write_based_case_probe(dir: &Path) -> bool {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let marker = format!(".bee-case-probe-{}-{}", std::process::id(), nanos);
    let lower_path = dir.join(marker.to_lowercase());
    let upper_path = dir.join(marker.to_uppercase());
    if std::fs::write(&lower_path, "").is_err() {
        return false;
    }
    let result = upper_path.exists();
    let _ = std::fs::remove_file(&lower_path);
    result
}

fn probe_case_insensitive(dir: &Path) -> bool {
    read_only_case_probe(dir).unwrap_or_else(|| write_based_case_probe(dir))
}

/// Cached per volume: key is the nearest existing ancestor's device number
/// when statable, else its path string.
pub fn detect_case_insensitive_volume(sample: &Path) -> bool {
    let root = nearest_existing_ancestor(sample);
    let key = match file_id(&root) {
        Some(id) if id.dev != 0 => format!("dev:{}", id.dev),
        _ => root.to_string_lossy().into_owned(),
    };
    if let Some(&cached) = case_fold_cache().lock().unwrap().get(&key) {
        return cached;
    }
    let result = probe_case_insensitive(&root);
    case_fold_cache().lock().unwrap().insert(key, result);
    result
}

// ── canonical comparison ───────────────────────────────────────────────────

/// canonicalPathsEqual(a, b): identity fast path when usable, else
/// normalized-string comparison, case-folded only when BOTH volumes probe
/// case-insensitive (correction 4).
pub fn canonical_paths_equal(a: &Path, b: &Path) -> bool {
    let ra = absolute(a);
    let rb = absolute(b);

    if let Some(identity) = try_identity_equal(&ra, &rb) {
        return identity;
    }

    let fold = detect_case_insensitive_volume(&ra) && detect_case_insensitive_volume(&rb);
    let sa = ra.to_string_lossy();
    let sb = rb.to_string_lossy();
    if fold {
        sa.to_lowercase() == sb.to_lowercase()
    } else {
        sa == sb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_dir_through_different_separator_spellings() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("sub");
        std::fs::create_dir_all(&dir).unwrap();
        let native = dir.clone();
        let forward = PathBuf::from(dir.to_string_lossy().replace('\\', "/"));
        assert!(canonical_paths_equal(&native, &forward));
    }

    #[test]
    fn distinct_dirs_never_equal() {
        let tmp = tempfile::tempdir().unwrap();
        let a = tmp.path().join("a");
        let b = tmp.path().join("b");
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        assert!(!canonical_paths_equal(&a, &b));
    }

    #[cfg(windows)]
    #[test]
    fn case_spellings_equal_on_case_insensitive_volume() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("CaseDir");
        std::fs::create_dir_all(&dir).unwrap();
        // Identity fast path resolves this regardless of the probe; also
        // exercise the probe itself on the temp volume.
        let lower = tmp.path().join("casedir");
        if detect_case_insensitive_volume(tmp.path()) {
            assert!(canonical_paths_equal(&dir, &lower));
        }
    }

    #[test]
    fn missing_paths_fall_back_to_string_compare() {
        let tmp = tempfile::tempdir().unwrap();
        let ghost_a = tmp.path().join("ghost");
        let ghost_b = tmp.path().join("ghost");
        assert!(canonical_paths_equal(&ghost_a, &ghost_b));
        let other = tmp.path().join("other-ghost");
        assert!(!canonical_paths_equal(&ghost_a, &other));
    }

    #[test]
    fn read_only_probe_settles_existing_case_bearing_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("Probe");
        std::fs::create_dir_all(&dir).unwrap();
        // Must be conclusive without the write probe on every mainstream FS.
        assert!(read_only_case_probe(&dir).is_some());
    }

    #[test]
    fn flip_ascii_case_reports_no_op() {
        assert_eq!(flip_ascii_case("abC"), Some("ABc".to_string()));
        assert_eq!(flip_ascii_case("123-_"), None);
    }
}
