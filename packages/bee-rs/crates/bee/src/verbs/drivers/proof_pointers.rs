use std::path::Path;

use serde_json::{json, Value};

pub(crate) fn candidate_paths(proof: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in proof.split_whitespace() {
        let token = raw
            .trim_matches(|c: char| matches!(c, '"' | '\'' | '`' | ',' | '(' | ')'))
            .trim_end_matches(|c: char| c == ':' || c == '.');
        let token = strip_line_suffix(token);
        if token.is_empty()
            || !token.contains('/')
            || token.starts_with("http://")
            || token.starts_with("https://")
            || token.starts_with('-')
            || token.starts_with('/')
            || token.chars().any(|c| matches!(c, '$' | '*' | '{' | '='))
        {
            continue;
        }
        if !out.iter().any(|p| p == token) {
            out.push(token.to_string());
        }
    }
    out
}

fn strip_line_suffix(token: &str) -> &str {
    let Some((head, tail)) = token.rsplit_once(':') else {
        return token;
    };
    let numeric = !tail.is_empty()
        && tail.split('-').count() <= 2
        && tail.split('-').all(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
    if numeric {
        head
    } else {
        token
    }
}

pub(crate) fn missing_paths(root: &Path, proof: &str) -> Vec<String> {
    candidate_paths(proof)
        .into_iter()
        .filter(|p| !root.join(p).exists())
        .collect()
}

pub(crate) fn dead_proof_pointers(root: &Path, feature: &str) -> Vec<Value> {
    let Ok(cells) = crate::verbs::drivers::list_cells_including_archive(root, feature, Some("capped"))
    else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for cell in cells {
        let id = cell.get("id").and_then(Value::as_str).unwrap_or("");
        let proof = cell
            .get("trace")
            .and_then(|t| t.get("report"))
            .and_then(|r| r.get("tests"))
            .and_then(Value::as_str);
        let Some(proof) = proof else { continue };
        for path in missing_paths(root, proof) {
            out.push(json!({"cell": id, "path": path}));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dead_proof_tokenizer_keeps_real_paths_and_drops_urls_flags_globs() {
        let proof = "cargo test --manifest-path packages/bee-rs/Cargo.toml — green:unit — see https://example.com/a/b and `docs/x/y.md`, (src/lib.rs:12-20) glob src/*.rs $HOME/x/y --out=a/b -x/y /abs/path/z word";
        assert_eq!(
            candidate_paths(proof),
            vec!["packages/bee-rs/Cargo.toml", "docs/x/y.md", "src/lib.rs"]
        );
    }

    #[test]
    fn dead_proof_tokenizer_strips_trailing_colon_and_period() {
        assert_eq!(candidate_paths("see docs/a/b.md: and docs/c/d.md."), vec!["docs/a/b.md", "docs/c/d.md"]);
    }

    #[test]
    fn dead_proof_missing_paths_checks_under_the_root() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("docs")).unwrap();
        std::fs::write(tmp.path().join("docs/real.md"), "x").unwrap();
        let proof = "t — green:static — docs/real.md docs/missing/file.md";
        assert_eq!(missing_paths(tmp.path(), proof), vec!["docs/missing/file.md"]);
        assert!(missing_paths(tmp.path(), "cargo test — green:unit — docs/real.md").is_empty());
    }
}
