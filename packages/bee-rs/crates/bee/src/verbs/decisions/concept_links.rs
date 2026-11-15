use serde_json::Value;
use std::path::Path;

const MAX_LINKS: usize = 3;

pub(crate) struct ConceptFile {
    path: String,
    lines: Vec<String>,
}

pub(crate) fn read_concepts(root: &Path) -> Vec<ConceptFile> {
    let mut paths = Vec::new();
    collect_md(&root.join("docs").join("knowledge"), &mut paths);
    let mut files: Vec<ConceptFile> = paths
        .into_iter()
        .filter_map(|p| {
            let body = std::fs::read_to_string(&p).ok()?;
            let rel = p.strip_prefix(root).ok()?.to_string_lossy().replace('\\', "/");
            Some(ConceptFile { path: rel, lines: body.lines().map(str::to_string).collect() })
        })
        .collect();
    files.sort_by(|a, b| a.path.cmp(&b.path));
    files
}

fn collect_md(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_md(&p, out);
        } else if p.extension().is_some_and(|e| e == "md") {
            out.push(p);
        }
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}

fn slug_ends(line: &str, slug: &str) -> Vec<usize> {
    let mut ends = Vec::new();
    for (start, _) in line.match_indices(slug) {
        let end = start + slug.len();
        let before_ok = line[..start].chars().next_back().map_or(true, |c| !is_word_char(c));
        let after_ok = line[end..].chars().next().map_or(true, |c| !is_word_char(c));
        if before_ok && after_ok {
            ends.push(end);
        }
    }
    ends
}

fn d_tokens(tail: &str) -> Vec<(u64, u64)> {
    let chars: Vec<char> = tail.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let boundary = i == 0 || !chars[i - 1].is_alphanumeric();
        if chars[i] == 'D' && boundary && chars.get(i + 1).is_some_and(|c| c.is_ascii_digit()) {
            let (a, next) = read_number(&chars, i + 1);
            let mut hi = a;
            let mut end = next;
            if chars.get(next) == Some(&'-') {
                let mut j = next + 1;
                if chars.get(j) == Some(&'D') {
                    j += 1;
                }
                if chars.get(j).is_some_and(|c| c.is_ascii_digit()) {
                    let (b, after) = read_number(&chars, j);
                    hi = b;
                    end = after;
                }
            }
            tokens.push((a, hi.max(a)));
            i = end;
        } else {
            i += 1;
        }
    }
    tokens
}

fn read_number(chars: &[char], from: usize) -> (u64, usize) {
    let mut n: u64 = 0;
    let mut i = from;
    while let Some(d) = chars.get(i).and_then(|c| c.to_digit(10)) {
        n = n.saturating_mul(10).saturating_add(d as u64);
        i += 1;
    }
    (n, i)
}

fn line_cites_feature(line: &str, slug: &str, n: Option<u64>) -> bool {
    slug_ends(line, slug).into_iter().any(|end| {
        let tokens = d_tokens(&line[end..]);
        match n {
            Some(n) => tokens.iter().any(|(lo, hi)| *lo <= n && n <= *hi),
            None => !tokens.is_empty(),
        }
    })
}

fn text_prefix(text: &str) -> (Option<String>, Option<u64>) {
    let mut words = text.split_whitespace();
    let slug = words.next().map(str::to_string);
    let n = words.next().and_then(|w| {
        let rest = w.strip_prefix('D')?;
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        digits.parse().ok()
    });
    (slug, n)
}

fn feature_and_number(event: &Value) -> (Option<String>, Option<u64>) {
    let text = event.get("decision").and_then(Value::as_str).unwrap_or("");
    let (prefix_slug, n) = text_prefix(text);
    let field = event.get("feature").and_then(Value::as_str).filter(|s| !s.is_empty());
    match field {
        Some(f) => {
            let n = if prefix_slug.as_deref() == Some(f) { n } else { None };
            (Some(f.to_string()), n)
        }
        None => match n {
            Some(_) => (prefix_slug, n),
            None => (None, None),
        },
    }
}

pub(crate) fn concept_links(files: &[ConceptFile], event: &Value) -> Vec<String> {
    let mut links: Vec<String> = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    let id = event.get("id").and_then(Value::as_str).unwrap_or("");
    let short: String = id.chars().take(8).collect();
    if short.chars().count() == 8 {
        for file in files {
            if links.len() >= MAX_LINKS {
                break;
            }
            if let Some(pos) = file.lines.iter().position(|l| l.contains(&short)) {
                links.push(format!("{}:{}", file.path, pos + 1));
                seen.push(&file.path);
            }
        }
    }
    if let (Some(slug), n) = feature_and_number(event) {
        for file in files {
            if links.len() >= MAX_LINKS {
                break;
            }
            if seen.contains(&file.path.as_str()) {
                continue;
            }
            if let Some(pos) = file.lines.iter().position(|l| line_cites_feature(l, &slug, n)) {
                links.push(format!("{}:{}", file.path, pos + 1));
            }
        }
    }
    links
}
