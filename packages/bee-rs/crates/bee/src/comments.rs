#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Lang {
    Rust,
    Shell,
    Python,
}

const CODE_ROOTS: [&str; 4] = [
    "packages/bee/hooks/",
    "packages/bee/lib/",
    "scripts/",
    ".bee/verify/",
];

const CRATE_ROOT: &str = "packages/bee-rs/crates/";

const LICENSE_MARKERS: [&str; 3] = ["Copyright", "SPDX-License-Identifier", "Licensed under"];

pub(crate) fn under_code_root(rel: &str) -> bool {
    if let Some(rest) = rel.strip_prefix(CRATE_ROOT) {
        if let Some((krate, tail)) = rest.split_once('/') {
            if !krate.is_empty() && tail.starts_with("src/") {
                return true;
            }
        }
    }
    CODE_ROOTS.iter().any(|root| rel.starts_with(root))
}

pub(crate) fn code_lang(rel: &str, first_line: &str) -> Option<Lang> {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => match ext {
            "rs" => Some(Lang::Rust),
            "sh" | "bash" => Some(Lang::Shell),
            "py" => Some(Lang::Python),
            _ => None,
        },
        _ => shebang_lang(first_line),
    }
}

fn shebang_lang(first_line: &str) -> Option<Lang> {
    let line = first_line.trim();
    if !line.starts_with("#!") {
        return None;
    }
    if line.contains("python") {
        return Some(Lang::Python);
    }
    if line.contains("bash") || line.contains("sh") {
        return Some(Lang::Shell);
    }
    None
}

pub(crate) fn is_comment_line(lang: Lang, line: &str, in_block: &mut bool) -> bool {
    let trimmed = line.trim();
    match lang {
        Lang::Rust => {
            if *in_block {
                if trimmed.contains("*/") {
                    *in_block = false;
                }
                return true;
            }
            if trimmed.starts_with("//") {
                return true;
            }
            if trimmed.starts_with("/*") {
                *in_block = !trimmed[2..].contains("*/");
                return true;
            }
            false
        }
        Lang::Shell | Lang::Python => trimmed.starts_with('#'),
    }
}

pub(crate) fn is_exception(lang: Lang, line: &str, in_leading_block: bool) -> bool {
    let trimmed = line.trim();
    if trimmed.starts_with("#!") {
        return true;
    }
    if in_leading_block {
        return true;
    }
    lang == Lang::Rust && comment_text(trimmed).starts_with("SAFETY:")
}

fn comment_text(trimmed: &str) -> &str {
    trimmed.trim_start_matches(['/', '*', '!', '#']).trim()
}

fn leading_license_lines(lang: Lang, text: &str) -> usize {
    let mut in_block = false;
    let mut started = false;
    let mut last = 0usize;
    let mut body = String::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        let comment = is_comment_line(lang, line, &mut in_block);
        if !started {
            if trimmed.is_empty() || trimmed.starts_with("#!") {
                continue;
            }
            if !comment {
                return 0;
            }
            started = true;
        } else if !comment {
            break;
        }
        body.push_str(trimmed);
        body.push('\n');
        last = index + 1;
    }
    if LICENSE_MARKERS.iter().any(|marker| body.contains(marker)) {
        last
    } else {
        0
    }
}

pub(crate) fn count_comment_lines(lang: Lang, text: &str) -> usize {
    comment_lines(lang, text).len()
}

pub(crate) fn comment_lines(lang: Lang, text: &str) -> Vec<(usize, String)> {
    let license_lines = leading_license_lines(lang, text);
    let mut in_block = false;
    let mut out = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        if !is_comment_line(lang, line, &mut in_block) {
            continue;
        }
        if is_exception(lang, line, number <= license_lines) {
            continue;
        }
        out.push((number, line.trim().to_string()));
    }
    out
}

pub(crate) fn added_comment_lines(lang: Lang, old: &str, new: &str) -> Vec<(usize, String)> {
    let mut held: HashMap<String, usize> = HashMap::new();
    for (_, text) in comment_lines(lang, old) {
        *held.entry(text).or_insert(0) += 1;
    }
    let mut out = Vec::new();
    for (number, text) in comment_lines(lang, new) {
        match held.get_mut(&text) {
            Some(count) if *count > 0 => *count -= 1,
            _ => out.push((number, text)),
        }
    }
    out
}

pub(crate) fn walk_code_files(root: &Path) -> Vec<(String, Lang)> {
    let mut paths = Vec::new();
    for start in CODE_ROOTS.iter().chain(std::iter::once(&CRATE_ROOT)) {
        walk(&root.join(start.trim_end_matches('/')), &mut paths);
    }
    let mut out: Vec<(String, Lang)> = Vec::new();
    for path in paths {
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if !under_code_root(&rel) {
            continue;
        }
        if let Some(lang) = file_lang(&rel, &path) {
            out.push((rel, lang));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out.dedup_by(|a, b| a.0 == b.0);
    out
}

fn file_lang(rel: &str, path: &Path) -> Option<Lang> {
    if let Some(lang) = code_lang(rel, "") {
        return Some(lang);
    }
    code_lang(rel, &first_line(path))
}

fn first_line(path: &Path) -> String {
    let Ok(bytes) = std::fs::read(path) else {
        return String::new();
    };
    let head = &bytes[..bytes.len().min(256)];
    String::from_utf8_lossy(head).lines().next().unwrap_or("").to_string()
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let Ok(kind) = entry.file_type() else { continue };
        if kind.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if matches!(name.as_ref(), "target" | "node_modules" | ".git") {
                continue;
            }
            walk(&path, out);
        } else if kind.is_file() {
            out.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_line_and_doc_comments_count() {
        let text = "// one\nlet x = 1;\n/// two\n//! three\n";
        assert_eq!(count_comment_lines(Lang::Rust, text), 3);
    }

    #[test]
    fn shell_and_python_hash_lines_count() {
        assert_eq!(count_comment_lines(Lang::Shell, "# a\necho hi\n  # b\n"), 2);
        assert_eq!(count_comment_lines(Lang::Python, "x = 1\n# note\n"), 1);
        assert_eq!(count_comment_lines(Lang::Shell, "echo '# not a comment'\n"), 0);
    }

    #[test]
    fn a_block_comment_counts_every_inner_line_and_then_closes() {
        let text = "/*\n a\n b\n*/\nlet x = 1;\n// after\n";
        assert_eq!(
            comment_lines(Lang::Rust, text),
            vec![
                (1, "/*".to_string()),
                (2, "a".to_string()),
                (3, "b".to_string()),
                (4, "*/".to_string()),
                (6, "// after".to_string()),
            ]
        );
    }

    #[test]
    fn a_one_line_block_comment_does_not_open_a_block() {
        let text = "/* one */\nlet x = 1;\nlet y = 2;\n";
        assert_eq!(count_comment_lines(Lang::Rust, text), 1);
    }

    #[test]
    fn a_shebang_never_counts_on_line_one_or_inside_a_body() {
        let text = "#!/usr/bin/env bash\necho hi\ncat <<'EOF' > f\n#!/usr/bin/env bash\nEOF\n# real\n";
        assert_eq!(comment_lines(Lang::Shell, text), vec![(6, "# real".to_string())]);
    }

    #[test]
    fn a_safety_line_never_counts() {
        let text = "// SAFETY: the pointer is valid\nunsafe { p.read() }\n// why not\n";
        assert_eq!(comment_lines(Lang::Rust, text), vec![(3, "// why not".to_string())]);
    }

    #[test]
    fn a_leading_license_block_never_counts_but_a_later_copyright_line_does() {
        let text = "// Copyright 2026 Someone\n// Licensed under Apache-2.0\n\nfn f() {}\n// Copyright, later\n";
        assert_eq!(comment_lines(Lang::Rust, text), vec![(5, "// Copyright, later".to_string())]);
    }

    #[test]
    fn a_leading_block_without_a_license_marker_counts() {
        let text = "// a header\n// of prose\nfn f() {}\n";
        assert_eq!(count_comment_lines(Lang::Rust, text), 2);
    }

    #[test]
    fn a_marker_at_line_start_inside_a_string_literal_counts_the_documented_limitation() {
        let text = "let s = \"\n// expand a project row\n\";\n";
        assert_eq!(
            comment_lines(Lang::Rust, text),
            vec![(2, "// expand a project row".to_string())]
        );
    }

    #[test]
    fn added_comment_lines_is_empty_for_a_move_and_for_a_delete() {
        let old = "// a\nfn f() {}\n// b\n";
        let moved = "fn f() {}\n// b\n// a\n";
        assert_eq!(added_comment_lines(Lang::Rust, old, moved), Vec::new());
        let deleted = "fn f() {}\n// a\n";
        assert_eq!(added_comment_lines(Lang::Rust, old, deleted), Vec::new());
    }

    #[test]
    fn added_comment_lines_names_the_line_and_the_text_of_an_add() {
        let old = "// a\nfn f() {}\n// b\n";
        let new = "// a\nfn f() {}\n// c\n// b\n";
        assert_eq!(added_comment_lines(Lang::Rust, old, new), vec![(3, "// c".to_string())]);
    }

    #[test]
    fn code_lang_reads_an_extension_less_shebang_file_and_rejects_markdown() {
        assert_eq!(code_lang("packages/bee-rs/crates/bee/src/x.rs", ""), Some(Lang::Rust));
        assert_eq!(code_lang("scripts/release.sh", ""), Some(Lang::Shell));
        assert_eq!(code_lang("scripts/x.bash", ""), Some(Lang::Shell));
        assert_eq!(code_lang("scripts/tool.py", ""), Some(Lang::Python));
        assert_eq!(
            code_lang(".bee/verify/verify-app/control-bee", "#!/usr/bin/env bash"),
            Some(Lang::Shell)
        );
        assert_eq!(code_lang("scripts/gen", "#!/usr/bin/env python3"), Some(Lang::Python));
        assert_eq!(code_lang("scripts/plain", "echo hi"), None);
        assert_eq!(code_lang("docs/notes.md", "#!/usr/bin/env bash"), None);
    }

    #[test]
    fn under_code_root_takes_any_crate_src_and_leaves_tests_and_docs_out() {
        assert!(under_code_root("packages/bee-rs/crates/fleet/src/lib.rs"));
        assert!(under_code_root("packages/bee-rs/crates/bee/src/hooks/mod.rs"));
        assert!(under_code_root("packages/bee/hooks/write-guard.mjs"));
        assert!(under_code_root("packages/bee/lib/paths.mjs"));
        assert!(under_code_root("scripts/release.sh"));
        assert!(under_code_root(".bee/verify/verify-app/control-bee"));
        assert!(!under_code_root("packages/bee-rs/crates/bee/tests/x.rs"));
        assert!(!under_code_root("packages/bee-rs/crates/bee/Cargo.toml"));
        assert!(!under_code_root("docs/x.rs"));
        assert!(!under_code_root(".bee/state.json"));
    }

    #[test]
    fn walk_code_files_returns_sorted_slash_keys_and_skips_build_trees() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let write = |rel: &str, body: &str| {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, body).unwrap();
        };
        write("scripts/a.sh", "# hi\n");
        write("scripts/sub/b.py", "x = 1\n");
        write("scripts/notes.md", "prose\n");
        write("scripts/target/skip.sh", "# skipped\n");
        write("scripts/node_modules/skip.sh", "# skipped\n");
        write("packages/bee-rs/crates/fleet/src/lib.rs", "fn f() {}\n");
        write("packages/bee-rs/crates/fleet/tests/t.rs", "fn t() {}\n");
        write(".bee/verify/control-bee", "#!/usr/bin/env bash\n");
        write("docs/x.rs", "fn d() {}\n");

        let got = walk_code_files(root);
        assert_eq!(
            got,
            vec![
                (".bee/verify/control-bee".to_string(), Lang::Shell),
                ("packages/bee-rs/crates/fleet/src/lib.rs".to_string(), Lang::Rust),
                ("scripts/a.sh".to_string(), Lang::Shell),
                ("scripts/sub/b.py".to_string(), Lang::Python),
            ]
        );
        assert!(got.iter().all(|(rel, _)| !rel.contains('\\')));
    }
}
