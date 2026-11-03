use std::ffi::OsString;
use std::process::ExitCode;

pub fn run<F>(args: &[OsString], verb: F) -> ExitCode
where
    F: FnOnce() -> ExitCode,
{
    #[cfg(unix)]
    {
        if let Some(target) = unix::engage(args) {
            return unix::spill(target, verb);
        }
    }
    #[cfg(not(unix))]
    let _ = args;
    verb()
}

#[cfg(unix)]
mod unix {
    use crate::roots::{resolve_roots_core, Resolution};
    use crate::state;
    use std::ffi::OsString;
    use std::io::{IsTerminal, Write};
    use std::os::fd::AsRawFd;
    use std::path::{Path, PathBuf};
    use std::process::ExitCode;

    pub const SPILL_VERBS: &[&[&str]] = &[
        &["status"],
        &["cells", "list"],
        &["decisions", "list"],
        &["backlog", "list"],
        &["knowledge", "context"],
        &["knowledge", "search"],
    ];

    const FULL_ENV: &str = "BEE_OUTPUT_FULL";
    const CONFIG_KEY: &str = "output_spill_bytes";

    pub struct Target {
        pub file: PathBuf,
        pub rel: String,
        pub threshold: u64,
    }

    fn allowlisted(strs: &[&str]) -> bool {
        if strs.iter().any(|t| *t == "--help") {
            return true;
        }
        SPILL_VERBS.iter().any(|pat| strs.starts_with(pat))
    }

    fn threshold(work_root: &Path) -> Option<u64> {
        state::read_config_raw(work_root)
            .get(CONFIG_KEY)?
            .as_u64()
            .filter(|n| *n > 0)
    }

    fn stem(strs: &[&str]) -> String {
        let mut parts: Vec<String> = strs
            .iter()
            .filter(|t| !t.starts_with('-'))
            .map(|t| {
                t.chars()
                    .map(|c| {
                        if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                            c
                        } else {
                            '_'
                        }
                    })
                    .collect::<String>()
            })
            .filter(|t| !t.is_empty())
            .collect();
        if parts.is_empty() {
            parts.push("help".to_string());
        }
        let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%S%3fZ").to_string();
        format!("{}-{stamp}", parts.join("-"))
    }

    pub fn engage(args: &[OsString]) -> Option<Target> {
        let strs: Vec<&str> = args.iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
        if !allowlisted(&strs) {
            return None;
        }
        if strs
            .iter()
            .any(|t| *t == "--json" || t.starts_with("--json="))
        {
            return None;
        }
        if std::env::var(FULL_ENV).as_deref() == Ok("1") {
            return None;
        }
        if std::io::stdout().is_terminal() {
            return None;
        }
        let cwd = std::env::current_dir().ok()?;
        let work_root = match resolve_roots_core(&cwd) {
            Resolution::Ordinary { work_root, .. } | Resolution::LinkedValid { work_root, .. } => {
                work_root
            }
            _ => return None,
        };
        let threshold = threshold(&work_root)?;
        let name = stem(&strs);
        Some(Target {
            file: work_root
                .join(".bee")
                .join("tmp")
                .join("output")
                .join(format!("{name}.txt")),
            rel: format!(".bee/tmp/output/{name}.txt"),
            threshold,
        })
    }

    struct Redirected(i32);

    impl Redirected {
        fn onto(fd: i32) -> std::io::Result<Self> {
            let saved = unsafe { libc::dup(libc::STDOUT_FILENO) };
            if saved < 0 {
                return Err(std::io::Error::last_os_error());
            }
            if unsafe { libc::dup2(fd, libc::STDOUT_FILENO) } < 0 {
                let err = std::io::Error::last_os_error();
                unsafe { libc::close(saved) };
                return Err(err);
            }
            Ok(Self(saved))
        }
    }

    impl Drop for Redirected {
        fn drop(&mut self) {
            let _ = std::io::stdout().flush();
            unsafe {
                libc::dup2(self.0, libc::STDOUT_FILENO);
                libc::close(self.0);
            }
        }
    }

    pub fn spill<F>(target: Target, verb: F) -> ExitCode
    where
        F: FnOnce() -> ExitCode,
    {
        let dir = target.file.parent().unwrap_or(Path::new(".")).to_path_buf();
        if crate::fsutil::ensure_dir(&dir).is_err() {
            return verb();
        }
        let Ok(file) = std::fs::File::create(&target.file) else {
            return verb();
        };
        let _ = std::io::stdout().flush();
        let Ok(redirected) = Redirected::onto(file.as_raw_fd()) else {
            return verb();
        };
        let code = verb();
        drop(redirected);
        drop(file);
        render(&target, code)
    }

    fn whole_lines(bytes: &[u8]) -> Vec<&[u8]> {
        let mut out = Vec::new();
        let mut start = 0usize;
        for (i, b) in bytes.iter().enumerate() {
            if *b == b'\n' {
                out.push(&bytes[start..=i]);
                start = i + 1;
            }
        }
        if start < bytes.len() {
            out.push(&bytes[start..]);
        }
        out
    }

    fn render(target: &Target, code: ExitCode) -> ExitCode {
        let Ok(bytes) = std::fs::read(&target.file) else {
            return code;
        };
        let total = bytes.len() as u64;
        if total <= target.threshold {
            let mut out = std::io::stdout().lock();
            let _ = out.write_all(&bytes);
            let _ = out.flush();
            drop(out);
            let _ = std::fs::remove_file(&target.file);
            return code;
        }

        let head_budget = target.threshold / 3 * 2;
        let tail_budget = target.threshold / 4;
        let lines = whole_lines(&bytes);

        let mut head_end = 0usize;
        let mut head_bytes = 0u64;
        while head_end < lines.len() {
            let n = lines[head_end].len() as u64;
            if head_bytes + n > head_budget {
                break;
            }
            head_bytes += n;
            head_end += 1;
        }

        let mut tail_start = lines.len();
        let mut tail_bytes = 0u64;
        while tail_start > head_end {
            let n = lines[tail_start - 1].len() as u64;
            if tail_bytes + n > tail_budget {
                break;
            }
            tail_bytes += n;
            tail_start -= 1;
        }

        let omitted = total - head_bytes - tail_bytes;
        let marker = format!(
            "[bee] output spilled: {omitted} of {total} bytes omitted — full output: {}\n",
            target.rel
        );

        let mut out = std::io::stdout().lock();
        for line in &lines[..head_end] {
            let _ = out.write_all(line);
        }
        let _ = out.write_all(marker.as_bytes());
        for line in &lines[tail_start..] {
            let _ = out.write_all(line);
        }
        let _ = out.flush();
        code
    }
}
