// Split out of the single 4.9k-line verbs/drivers.rs. Code unchanged; only module placement and item visibility moved.
//
// Moved verbatim out of the parent file's inline module, indentation
// and all: a moved inline module is the same child of the same parent,
// so no path changes, and the fixtures inside are raw strings whose
// leading whitespace is content.

    #![allow(dead_code, clippy::all)]

    use crate::jsjson;
    use crate::state::read_config_raw;
    use crate::verbs::reservations::js_trim;
    use serde_json::{Map, Number, Value};
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};

    /// provenance: verbs/knowledge.rs:167 bundle_dir (lib/knowledge.mjs
    /// bundleDir + the delegating slice of resolveProductRoot).
    pub(super) fn bundle_dir(root: &Path) -> Option<PathBuf> {
        let config = read_config_raw(root);
        match config.get("product_root") {
            None | Some(Value::Null) => {}
            Some(Value::String(s)) if s.is_empty() => {}
            Some(_) => return None,
        }
        Some(root.join("docs").join("knowledge"))
    }

    pub(super) fn key_re_ok(key: &str) -> bool {
    // /^[A-Za-z_][A-Za-z0-9_.-]*$/
    let mut chars = key.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

    pub(super) fn is_reserved_basename(base: &str) -> bool {
    base == "index.md" || base == "log.md"
}

/// JS `\s` (same set String.prototype.trim strips) — via reservations.
    pub(super) fn js_is_space(c: char) -> bool {
    crate::verbs::reservations::js_is_ws(c)
}

    pub(super) fn js_quote_str(s: &str) -> String {
    jsjson::stringify(&Value::String(s.to_string()))
}


// ─── parser (accepts exactly the emitted subset; loud typed failure) ───────

    pub(super) enum Fm {
    Absent,
    Parsed {
        data: Map<String, Value>,
        block: String,
        body: String,
    },
    Failed {
        code: &'static str,
        message: String,
        line: usize,
    },
    // CUTOVER: a `NeedsNode` variant lived here for the one shape only V8
    // could decide — an unpaired surrogate escape in a quoted scalar. It (and
    // the `has_surrogate_escape` sniff that produced it) is retired: such a
    // scalar is a `Failed { code: "bad_quoted_string" }` like any other
    // undecodable one.
}

    pub(super) fn fm_fail(code: &'static str, message: String, line: usize) -> Result<Value, Fm> {
    Err(Fm::Failed { code, message, line })
}

    pub(super) fn parse_scalar_token(raw: &str, line_no: usize) -> Result<Value, Fm> {
    if raw == "true" {
        return Ok(Value::Bool(true));
    }
    if raw == "false" {
        return Ok(Value::Bool(false));
    }
    if raw.starts_with('"') {
        match serde_json::from_str::<Value>(raw) {
            Ok(Value::String(s)) => return Ok(Value::String(s)),
            Ok(_) => {
                return fm_fail("bad_quoted_string", "quoted value did not decode to a string".to_string(), line_no)
            }
            // CUTOVER: an unpaired surrogate escape used to answer
            // `Fm::NeedsNode` — V8's JSON.parse decoded it where serde does
            // not, so the whole command went to Node. Nothing in this process
            // can hold such a string and there is no Node left to ask, so it
            // is exactly what every other undecodable quoted scalar is: the
            // typed bad_quoted_string failure, same code, same line, same
            // exit path. `has_surrogate_escape` is retired with it.
            Err(_) => {
                return fm_fail(
                    "bad_quoted_string",
                    format!("quoted value {} is not one complete JSON string", js_quote_str(raw)),
                    line_no,
                );
            }
        }
    }
    if raw.starts_with('\'') {
        return fm_fail(
            "single_quoted_string",
            "single-quoted scalars are outside the emitted subset — use double quotes".to_string(),
            line_no,
        );
    }
    // /^[&*!|>%@`{}]/
    if matches!(raw.chars().next(), Some('&' | '*' | '!' | '|' | '>' | '%' | '@' | '`' | '{' | '}')) {
        return fm_fail(
            "unsupported_scalar",
            format!(
                "value starting with \"{}\" (anchor/alias/block/flow-map indicator) is outside the emitted subset",
                raw.chars().next().unwrap()
            ),
            line_no,
        );
    }
    Ok(Value::String(raw.to_string()))
}

    pub(super) fn parse_flow_list(raw: &str, line_no: usize) -> Result<Value, Fm> {
    if !raw.ends_with(']') {
        return fm_fail(
            "bad_flow_list",
            format!("flow list {} does not close with \"]\"", js_quote_str(raw)),
            line_no,
        );
    }
    let inner = js_trim(&raw[1..raw.len() - 1]);
    if inner.is_empty() {
        return Ok(Value::Array(Vec::new()));
    }
    let mut segments: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut escaped = false;
    for ch in inner.chars() {
        if in_quote {
            current.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_quote = false;
            }
        } else if ch == '"' {
            current.push(ch);
            in_quote = true;
        } else if ch == ',' {
            segments.push(std::mem::take(&mut current));
        } else {
            current.push(ch);
        }
    }
    if in_quote {
        return fm_fail("bad_flow_list", "unterminated quoted item inside flow list".to_string(), line_no);
    }
    segments.push(current);
    let mut value = Vec::new();
    for segment in &segments {
        let token = js_trim(segment);
        if token.is_empty() {
            return fm_fail("bad_flow_list", "empty item inside flow list".to_string(), line_no);
        }
        value.push(parse_scalar_token(token, line_no)?);
    }
    Ok(Value::Array(value))
}

    pub(super) fn parse_key_value_line(line: &str, target: &mut Map<String, Value>, line_no: usize, prefix: &str) -> Result<(), Fm> {
    let Some(sep) = line.find(": ") else {
        return fm_fail(
            "unrecognized_line",
            format!(
                "line {} is not \"key: value\", a \"bee:\" map header, or a closing \"---\"",
                js_quote_str(line)
            ),
            line_no,
        )
        .map(|_| ());
    };
    let key = &line[..sep];
    if !key_re_ok(key) {
        return fm_fail(
            "bad_key",
            format!("{} is not a legal frontmatter key", js_quote_str(key)),
            line_no,
        )
        .map(|_| ());
    }
    if target.contains_key(key) {
        return fm_fail("duplicate_key", format!("duplicate key \"{prefix}{key}\""), line_no).map(|_| ());
    }
    let raw = &line[sep + 2..];
    if raw.is_empty() {
        return fm_fail("empty_value", format!("key \"{prefix}{key}\" has no value after \": \""), line_no)
            .map(|_| ());
    }
    let parsed = if raw.starts_with('[') {
        parse_flow_list(raw, line_no)?
    } else {
        parse_scalar_token(raw, line_no)?
    };
    target.insert(key.to_string(), parsed);
    Ok(())
}

/// parseFrontmatter(text) — see lib/knowledge.mjs for the full contract.
    pub(super) fn parse_frontmatter(text: &str) -> Fm {
    let open_len = if text.starts_with("---\r\n") {
        5
    } else if text.starts_with("---\n") {
        4
    } else {
        return Fm::Absent;
    };

    let mut cursor = open_len;
    let mut block_end: Option<usize> = None;
    let mut inner_end = 0usize;
    while cursor <= text.len() {
        let nl = text[cursor..].find('\n').map(|p| p + cursor);
        let line_end = nl.unwrap_or(text.len());
        let mut line = &text[cursor..line_end];
        if let Some(stripped) = line.strip_suffix('\r') {
            line = stripped;
        }
        if line == "---" {
            inner_end = cursor;
            block_end = Some(nl.map(|p| p + 1).unwrap_or(text.len()));
            break;
        }
        let Some(nl) = nl else { break };
        cursor = nl + 1;
    }
    let Some(block_end) = block_end else {
        return Fm::Failed {
            code: "unclosed_frontmatter",
            message: "frontmatter opened with \"---\" but never closed".to_string(),
            line: 1,
        };
    };

    let block = text[..block_end].to_string();
    let body = text[block_end..].to_string();
    let inner_raw = &text[open_len..inner_end];
    let inner_lines: Vec<&str> = if inner_raw.is_empty() {
        Vec::new()
    } else {
        let mut v: Vec<&str> = inner_raw.split('\n').collect();
        v.pop();
        v
    };

    let mut data: Map<String, Value> = Map::new();
    let mut in_bee_map = false;
    let mut line_no = 1usize;
    for raw_line in inner_lines {
        line_no += 1;
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if line.is_empty() {
            return Fm::Failed {
                code: "blank_line",
                message: "blank line inside frontmatter is outside the emitted subset".to_string(),
                line: line_no,
            };
        }
        if line.contains('\t') {
            return Fm::Failed {
                code: "tab_in_frontmatter",
                message: "tab character inside frontmatter is outside the emitted subset".to_string(),
                line: line_no,
            };
        }
        if let Some(inner) = line.strip_prefix("  ") {
            if !in_bee_map {
                return Fm::Failed {
                    code: "unexpected_indent",
                    message: "indented line outside the \"bee:\" map".to_string(),
                    line: line_no,
                };
            }
            if inner.starts_with(' ') {
                return Fm::Failed {
                    code: "bad_indent",
                    message: "bee: map entries are indented exactly two spaces".to_string(),
                    line: line_no,
                };
            }
            let bee = data
                .get_mut("bee")
                .and_then(Value::as_object_mut)
                .expect("bee map exists while in_bee_map");
            match parse_key_value_line(inner, bee, line_no, "bee.") {
                Ok(()) => continue,
                Err(f) => return f,
            }
        }
        if line.starts_with(' ') {
            return Fm::Failed {
                code: "bad_indent",
                message: "root-level lines must not be indented".to_string(),
                line: line_no,
            };
        }
        in_bee_map = false;
        // /^([^:\s]+):$/ — a map header line.
        let header_key = line.strip_suffix(':').filter(|key| {
            !key.is_empty() && key.chars().all(|c| c != ':' && !js_is_space(c))
        });
        if let Some(key) = header_key {
            if !key_re_ok(key) {
                return Fm::Failed {
                    code: "bad_key",
                    message: format!("{} is not a legal frontmatter key", js_quote_str(key)),
                    line: line_no,
                };
            }
            if key != "bee" {
                return Fm::Failed {
                    code: "unsupported_map",
                    message: format!(
                        "nested map \"{key}:\" is outside the emitted subset (the only nested map is \"bee:\")"
                    ),
                    line: line_no,
                };
            }
            if data.contains_key("bee") {
                return Fm::Failed {
                    code: "duplicate_key",
                    message: "duplicate key \"bee\"".to_string(),
                    line: line_no,
                };
            }
            data.insert("bee".to_string(), Value::Object(Map::new()));
            in_bee_map = true;
            continue;
        }
        if let Err(f) = parse_key_value_line(line, &mut data, line_no, "") {
            return f;
        }
    }

    Fm::Parsed { data, block, body }
}

// ─── bundle walk (listBundleMarkdown — never leaves docs/knowledge/, D23) ──

/// lstat-level symlink test matching Node's dirent.isSymbolicLink(): on
/// Windows any reparse point (symlink OR junction) counts, like libuv.
    pub(super) fn is_symlinkish(path: &Path) -> bool {
    let Ok(md) = std::fs::symlink_metadata(path) else { return false };
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        (md.file_attributes() & 0x400) != 0 // FILE_ATTRIBUTE_REPARSE_POINT
    }
    #[cfg(not(windows))]
    {
        md.file_type().is_symlink()
    }
}

/// None => delegate (non-UTF-16-sortable names or unrepresentable OsStrings).
    pub(super) fn list_bundle_markdown(dir: &Path) -> Option<Vec<String>> {
    fn walk(abs: &Path, rel: &str, out: &mut Vec<String>) -> Option<()> {
        let entries = match std::fs::read_dir(abs) {
            Ok(rd) => rd,
            Err(_) => return Some(()),
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_str()?.to_string();
            let child_abs = entry.path();
            if is_symlinkish(&child_abs) {
                continue; // a symlink could escape the bundle — never follow (D23)
            }
            let Ok(ft) = entry.file_type() else { continue };
            let child_rel = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
            if ft.is_dir() {
                walk(&child_abs, &child_rel, out)?;
            } else if ft.is_file() && name.ends_with(".md") {
                out.push(child_rel);
            }
        }
        Some(())
    }
    let mut out = Vec::new();
    if dir.exists() {
        walk(dir, "", &mut out)?;
    }
    // JS Array#sort compares UTF-16 code units; UTF-8 byte order agrees below
    // U+E000 (supplementary chars sort before U+E000..U+FFFF under UTF-16).
    if out.iter().any(|rel| rel.chars().any(|c| c >= '\u{e000}')) {
        return None;
    }
    out.sort();
    Some(out)
}

    pub(super) fn read_file_lossy(path: &Path) -> std::io::Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

// ─── path resolution inside the bundle (resolveInsideBundle subset) ────────

/// resolveInsideBundle + normalizeBundleTarget: lexically resolve `target`
/// against the ABSOLUTE bundle `dir` exactly like path.resolve (pops through
/// '..' and re-entry, clamps at the filesystem root, case-sensitive prefix
/// compare), and return the bundle-relative path with '/' separators when the
/// result is a strict descendant of `dir`; None when it escapes (never
/// followed, D23). Err(()) => delegate (drive-letter / rooted shapes whose
/// win32 path.resolve semantics are not fully modeled here).
    pub(super) fn normalize_bundle_target(dir: &Path, target: &str) -> Result<Option<String>, ()> {
    if target.is_empty() {
        return Ok(None);
    }
    if target.contains(':') || target.starts_with('/') || target.starts_with('\\') {
        return Err(()); // drive-relative / rooted forms — Node decides
    }
    // The bundle dir's own normal components are the containment prefix
    // (path.resolve(dir) — dir is already absolute and '..'-free here).
    let base: Vec<String> = dir
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(os) => os.to_str().map(str::to_string),
            _ => None,
        })
        .collect();
    let mut stack: Vec<String> = base.clone();
    for seg in target.split(['/', '\\']) {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            stack.pop(); // at the root, path.resolve clamps — pop of empty is a no-op
        } else {
            stack.push(seg.to_string());
        }
    }
    if stack.len() <= base.len() || stack[..base.len()] != base[..] {
        return Ok(None); // not a strict descendant of the bundle dir
    }
    Ok(Some(stack[base.len()..].join("/")))
}

/// resolveInsideBundle for existence checks: absolute path when contained.
    pub(super) fn resolve_inside_bundle(dir: &Path, target: &str) -> Result<Option<PathBuf>, ()> {
    Ok(normalize_bundle_target(dir, target)?.map(|rel| join_rel(dir, &rel)))
}

// ─── concept inventory (collectConcepts) ───────────────────────────────────

    pub(super) struct Concept {
        pub(super) path: String,
        pub(super) data: Map<String, Value>,
}

/// None => delegate (walk/name issues, V8-only frontmatter).
    pub(super) fn collect_concepts(dir: &Path) -> Option<Vec<Concept>> {
    let mut concepts = Vec::new();
    for rel in list_bundle_markdown(dir)? {
        let base = rel.rsplit('/').next().unwrap_or(&rel);
        if is_reserved_basename(base) {
            continue;
        }
        let data = match read_file_lossy(&join_rel(dir, &rel)) {
            Err(_) => Map::new(), // unreadable: keep the row with empty data
            Ok(text) => match parse_frontmatter(&text) {
                Fm::Parsed { data, .. } => data,
                _ => Map::new(),
            },
        };
        concepts.push(Concept { path: rel, data });
    }
    Some(concepts)
}

    pub(super) fn join_rel(dir: &Path, rel: &str) -> PathBuf {
    let mut p = dir.to_path_buf();
    for seg in rel.split('/') {
        p.push(seg);
    }
    p
}

/// beeOf(data) — the bee map when it is a plain object, else empty.
    pub(super) fn bee_of(data: &Map<String, Value>) -> Map<String, Value> {
    match data.get("bee") {
        Some(Value::Object(m)) => m.clone(),
        _ => Map::new(),
    }
}

    pub(super) fn dir_of(rel: &str) -> &str {
    match rel.rfind('/') {
        Some(p) => &rel[..p],
        None => "",
    }
}

    pub(super) fn str_field<'a>(map: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    match map.get(key) {
        Some(Value::String(s)) if !s.is_empty() => Some(s),
        _ => None,
    }
}

// ─── checkBundle (D4/D13 + G14 layer 3) ────────────────────────────────────

// ─── context (buildContextManifest + relevance ranking) ────────────────────

    pub(super) const CONTEXT_ESTIMATOR: &str = "bytes/4";
    pub(super) const KEEP: usize = 20;
    pub(super) const FLOOR: usize = 3;
    pub(super) const META_WEIGHT: f64 = 0.25;
    pub(super) const BODY_WEIGHT: f64 = 1.0;
    pub(super) const TAG_WEIGHT: f64 = 0.05;
    pub(super) const AREA_WEIGHT: f64 = 0.05;
    pub(super) const ZERO_SIGNAL_MIN_POPULATION: usize = 10;
    pub(super) const ZERO_SIGNAL_MAX_RATIO: f64 = 0.5;

    pub(super) const RELEVANCE_STOPWORDS: &str = "a an the and or but if then else for of to in on at by is are was were be been being it its this that these those with without from as not no never always every each any all some one two three you your we our they their he she i me my do does did done can could should would may might must will shall have has had so than which who whom what when where why how more most less least very just only also into out up down over under again further once here there both few other own same too s t don now";

    pub(super) fn stopwords() -> HashSet<&'static str> {
    RELEVANCE_STOPWORDS.split(' ').collect()
}

/// relevanceTokens(text) — lowercase, [a-z0-9]+ runs, >2 chars, stopped,
/// crude singularization.
    pub(super) fn relevance_tokens(text: &str, stops: &HashSet<&'static str>) -> Vec<String> {
    let lower: String = text.to_lowercase();
    let mut out = Vec::new();
    for raw in lower.split(|c: char| !c.is_ascii_lowercase() && !c.is_ascii_digit()) {
        if raw.len() <= 2 || stops.contains(raw) {
            continue;
        }
        let token = if raw.len() > 4 && raw.ends_with('s') && !raw.ends_with("ss") {
            &raw[..raw.len() - 1]
        } else {
            raw
        };
        out.push(token.to_string());
    }
    out
}

/// Insertion-ordered unique token list (JS Set semantics for f64-sum order).
    pub(super) fn uniq(tokens: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for t in tokens {
        if seen.insert(t.clone()) {
            out.push(t);
        }
    }
    out
}

    pub(super) fn concept_body(dir: &Path, rel: &str) -> Option<String> {
    let raw = match read_file_lossy(&join_rel(dir, rel)) {
        Ok(t) => t,
        Err(_) => return Some(String::new()),
    };
    match parse_frontmatter(&raw) {
        Fm::Parsed { body, .. } => Some(body),
        _ => Some(raw),
    }
}

    pub(super) fn meta_text_of(concept: &Concept) -> String {
    let t = match concept.data.get("title") {
        Some(v) if crate::verbs::reservations::truthy(v) => jsjson::js_to_string(v),
        _ => String::new(),
    };
    let d = match concept.data.get("description") {
        Some(v) if crate::verbs::reservations::truthy(v) => jsjson::js_to_string(v),
        _ => String::new(),
    };
    let tags = match concept.data.get("tags") {
        Some(Value::Array(items)) => items
            .iter()
            .map(|v| match v {
                Value::Null => String::new(),
                other => jsjson::js_to_string(other),
            })
            .collect::<Vec<_>>()
            .join(" "),
        _ => String::new(),
    };
    format!("{t} {d} {tags}")
}

/// Number(score.toFixed(6)) — display-precision rounding (divergence note in
/// the header covers the tie-rounding difference).
    pub(super) fn to_fixed6(x: f64) -> f64 {
    format!("{x:.6}").parse().unwrap_or(x)
}

    pub(super) fn score_critical_relevance(
    dir: &Path,
    criticals: &[&Concept],
    work: &Concept,
) -> Option<Vec<(String, f64)>> {
    let stops = stopwords();
    let mut fields: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    let mut df: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for concept in criticals {
        let meta = uniq(relevance_tokens(&meta_text_of(concept), &stops));
        let body = uniq(relevance_tokens(&concept_body(dir, &concept.path)?, &stops));
        let mut union_seen: HashSet<&String> = HashSet::new();
        for token in meta.iter().chain(body.iter()) {
            if union_seen.insert(token) {
                *df.entry(token.clone()).or_insert(0) += 1;
            }
        }
        fields.push((meta, body));
    }
    let population = criticals.len();
    let idf = |token: &str| ((population as f64 + 1.0) / (*df.get(token).unwrap_or(&0) as f64 + 1.0)).ln() + 1.0;

    let query: HashSet<String> = relevance_tokens(
        &format!("{} {}", meta_text_of(work), concept_body(dir, &work.path)?),
        &stops,
    )
    .into_iter()
    .collect();
    let work_bee = bee_of(&work.data);
    let work_tags: HashSet<String> = match work.data.get("tags") {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_lowercase())
            .collect(),
        _ => HashSet::new(),
    };
    let work_areas: HashSet<&str> = match work_bee.get("areas") {
        Some(Value::Array(items)) => items.iter().filter_map(|v| v.as_str()).collect(),
        _ => HashSet::new(),
    };

    let coverage = |set: &[String]| -> f64 {
        let mut hit = 0.0f64;
        let mut total = 0.0f64;
        for token in set {
            let weight = idf(token);
            total += weight;
            if query.contains(token) {
                hit += weight;
            }
        }
        if total == 0.0 {
            0.0
        } else {
            hit / total
        }
    };

    let mut scores = Vec::new();
    for (i, concept) in criticals.iter().enumerate() {
        let bee = bee_of(&concept.data);
        let tags = match concept.data.get("tags") {
            Some(Value::Array(items)) => items
                .iter()
                .filter(|v| matches!(v, Value::String(s) if work_tags.contains(&s.to_lowercase())))
                .count(),
            _ => 0,
        };
        let areas = match bee.get("areas") {
            Some(Value::Array(items)) => items
                .iter()
                .filter(|v| matches!(v, Value::String(s) if work_areas.contains(s.as_str())))
                .count(),
            _ => 0,
        };
        let (meta, body) = &fields[i];
        let score = TAG_WEIGHT * tags as f64
            + AREA_WEIGHT * areas as f64
            + META_WEIGHT * coverage(meta)
            + BODY_WEIGHT * coverage(body);
        scores.push((concept.path.clone(), to_fixed6(score)));
    }
    Some(scores)
}

    pub(super) fn num(v: f64) -> Value {
    Number::from_f64(v).map(Value::Number).unwrap_or(Value::Null)
}

    pub(super) enum ManifestOut {
    Built(Value),
    Thrown(String),
    NeedsNode,
}

    pub(super) fn build_context_manifest(dir: &Path, work: &str, budget: f64, budget_raw: &Value) -> ManifestOut {
    let work_id = js_trim(work);
    if work_id.is_empty() {
        return ManifestOut::Thrown("knowledge context: missing_work — --work <id> is required (D27).".to_string());
    }
    if !budget.is_finite() || budget < 0.0 {
        // Node's message JSON.stringify's the RAW flags.budget — the CLI's
        // string (quoted) or the lane-preset number, never the conversion.
        return ManifestOut::Thrown(format!(
            "knowledge context: bad_budget — --budget must be a non-negative token count, got {} (D27).",
            jsjson::stringify(budget_raw)
        ));
    }

    let Some(concepts) = collect_concepts(dir) else { return ManifestOut::NeedsNode };

    let work_concept = concepts.iter().find(|c| {
        matches!(c.data.get("type"), Some(Value::String(t)) if t == "bee.work-item")
            && matches!(bee_of(&c.data).get("id"), Some(Value::String(id)) if id == work_id)
    });
    let Some(work_concept) = work_concept else {
        return ManifestOut::Thrown(format!(
            "knowledge context: unknown_work — no bee.work-item concept in docs/knowledge/ carries bee.id \"{work_id}\" (D27)."
        ));
    };

    let mut ranked: Vec<(String, String)> = Vec::new(); // (rel, reason)
    let mut selected: HashSet<String> = HashSet::new();
    let by_path: std::collections::HashMap<&str, &Concept> =
        concepts.iter().map(|c| (c.path.as_str(), c)).collect();
    let select = |rel: &str, reason: String, ranked: &mut Vec<(String, String)>, selected: &mut HashSet<String>| {
        if selected.contains(rel) || !by_path.contains_key(rel) {
            return false;
        }
        selected.insert(rel.to_string());
        ranked.push((rel.to_string(), reason));
        true
    };

    // (1) the work item
    select(&work_concept.path, "work item".to_string(), &mut ranked, &mut selected);

    // (2) the plan sibling in the same work/<id>/ directory
    let work_dir = dir_of(&work_concept.path).to_string();
    let plan = concepts.iter().find(|c| {
        matches!(c.data.get("type"), Some(Value::String(t)) if t == "bee.plan") && dir_of(&c.path) == work_dir
    });
    if let Some(plan) = plan {
        select(&plan.path, format!("plan sibling in {work_dir}/"), &mut ranked, &mut selected);
    }

    // (3) required_context, transitive, BFS depth order, cycles deduped silently
    let mut queue: std::collections::VecDeque<(String, usize)> =
        ranked.iter().map(|(rel, _)| (rel.clone(), 0usize)).collect();
    while let Some((node_rel, depth)) = queue.pop_front() {
        let targets = match by_path.get(node_rel.as_str()).map(|c| bee_of(&c.data)) {
            Some(bee) => match bee.get("required_context") {
                Some(Value::Array(items)) => items.clone(),
                _ => continue,
            },
            None => continue,
        };
        for target in &targets {
            let Value::String(target) = target else { continue };
            let rel = match normalize_bundle_target(dir, target) {
                Ok(Some(rel)) => rel,
                Ok(None) => continue,
                Err(()) => return ManifestOut::NeedsNode,
            };
            if !by_path.contains_key(rel.as_str()) || selected.contains(&rel) {
                continue;
            }
            select(&rel, format!("required_context depth {} via {node_rel}", depth + 1), &mut ranked, &mut selected);
            queue.push_back((rel, depth + 1));
        }
    }

    // (4) the critical concepts, ranked by relevance and cut (G5/G11)
    let criticals: Vec<&Concept> = concepts
        .iter()
        .filter(|c| matches!(bee_of(&c.data).get("critical"), Some(Value::Bool(true))))
        .collect();
    let Some(relevance) = score_critical_relevance(dir, &criticals, work_concept) else {
        return ManifestOut::NeedsNode;
    };
    let score_of = |path: &str| -> f64 {
        relevance.iter().find(|(p, _)| p == path).map(|(_, s)| *s).unwrap_or(0.0)
    };
    let zero_signal_count = criticals.iter().filter(|c| score_of(&c.path) == 0.0).count();
    if criticals.len() >= ZERO_SIGNAL_MIN_POPULATION
        && (zero_signal_count as f64) > criticals.len() as f64 * ZERO_SIGNAL_MAX_RATIO
    {
        return ManifestOut::Thrown(format!(
            "knowledge context: zero_signal — {zero_signal_count} of {} bee.critical concepts score 0 against work item \"{work_id}\", above the pinned {} ratio. A ranking where most items tie at zero is a path sort wearing a relevance label — widen the work item's description/body, or fix the ranking, but do not ship this order (G11).",
            criticals.len(),
            jsjson::js_f64_to_string(ZERO_SIGNAL_MAX_RATIO)
        ));
    }
    let mut ranked_criticals: Vec<&&Concept> = criticals.iter().collect();
    ranked_criticals.sort_by(|a, b| {
        score_of(&b.path)
            .partial_cmp(&score_of(&a.path))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
    });
    let mut excluded: Vec<Value> = Vec::new();
    let mut floor_paths: Vec<String> = Vec::new();
    let mut kept = 0usize;
    for (index, concept) in ranked_criticals.iter().enumerate() {
        let rank = index + 1;
        let score = score_of(&concept.path);
        if selected.contains(&concept.path) {
            continue; // already in via required_context — never re-cut
        }
        if kept >= KEEP {
            let mut m = Map::new();
            m.insert("path".into(), Value::String(format!("docs/knowledge/{}", concept.path)));
            m.insert("score".into(), num(score));
            m.insert(
                "reason".into(),
                Value::String(format!(
                    "below the relevance cut — rank {rank} of {}, keep {KEEP} (G5)",
                    ranked_criticals.len()
                )),
            );
            excluded.push(Value::Object(m));
            continue;
        }
        let is_floor = kept < FLOOR;
        if is_floor {
            floor_paths.push(format!("docs/knowledge/{}", concept.path));
        }
        select(
            &concept.path,
            format!(
                "critical pattern (relevance {}, rank {rank} of {}{})",
                jsjson::js_f64_to_string(score),
                ranked_criticals.len(),
                if is_floor { ", floor" } else { "" }
            ),
            &mut ranked,
            &mut selected,
        );
        kept += 1;
    }

    // (5) decisions whose areas overlap the work item's areas
    let work_bee_map = bee_of(&work_concept.data);
    let work_areas: Vec<&str> = match work_bee_map.get("areas") {
        Some(Value::Array(items)) => items.iter().filter_map(|v| v.as_str()).collect(),
        _ => Vec::new(),
    };
    for concept in &concepts {
        if !matches!(concept.data.get("type"), Some(Value::String(t)) if t == "bee.decision") {
            continue;
        }
        let areas = match bee_of(&concept.data).get("areas") {
            Some(Value::Array(items)) => items.clone(),
            _ => Vec::new(),
        };
        let overlap: Vec<String> = areas
            .iter()
            .filter(|a| matches!(a, Value::String(s) if work_areas.contains(&s.as_str())))
            .map(jsjson::js_to_string)
            .collect();
        if overlap.is_empty() {
            continue;
        }
        select(&concept.path, format!("decision for area {}", overlap.join(", ")), &mut ranked, &mut selected);
    }

    struct Sized {
        repo_rel: String,
        reason: String,
        bytes: u64,
        est: f64,
        floor: bool,
    }
    let sized: Vec<Sized> = ranked
        .iter()
        .map(|(rel, reason)| {
            let repo_rel = format!("docs/knowledge/{rel}");
            let bytes = std::fs::metadata(join_rel(dir, rel)).map(|m| m.len()).unwrap_or(0);
            let est = (bytes as f64 / 4.0).ceil();
            let floor = floor_paths.contains(&repo_rel);
            Sized { repo_rel, reason: reason.clone(), bytes, est, floor }
        })
        .collect();

    let floor_cost: f64 = sized.iter().filter(|s| s.floor).map(|s| s.est).sum();
    let rank_one_cost = sized.first().map(|s| s.est).unwrap_or(0.0);
    let mut reserve = (budget - rank_one_cost).min(floor_cost).max(0.0);
    let mut available = budget - reserve;

    let mut entries: Vec<Value> = Vec::new();
    let mut truncated: Vec<String> = Vec::new();
    let mut total_est = 0.0f64;
    let mut cutting = false;
    for item in &sized {
        if item.floor {
            if item.est > reserve {
                truncated.push(item.repo_rel.clone());
                continue;
            }
            reserve -= item.est;
            total_est += item.est;
        } else {
            if cutting || item.est > available {
                cutting = true;
                truncated.push(item.repo_rel.clone());
                continue;
            }
            available -= item.est;
            total_est += item.est;
        }
        let mut m = Map::new();
        m.insert("path".into(), Value::String(item.repo_rel.clone()));
        m.insert("bytes".into(), Value::Number(Number::from(item.bytes)));
        m.insert("est_tokens".into(), num(item.est));
        m.insert("reason".into(), Value::String(item.reason.clone()));
        entries.push(Value::Object(m));
    }

    let decisions: Vec<Value> = match bee_of(&work_concept.data).get("decisions") {
        Some(Value::Array(items)) => items.iter().filter(|v| v.is_string()).cloned().collect(),
        _ => Vec::new(),
    };

    // CONSERVATION (G11)
    let accounted: HashSet<String> = entries
        .iter()
        .filter_map(|e| e.get("path").and_then(Value::as_str).map(str::to_string))
        .chain(truncated.iter().cloned())
        .chain(excluded.iter().filter_map(|e| e.get("path").and_then(Value::as_str).map(str::to_string)))
        .collect();
    let lost: Vec<String> = criticals
        .iter()
        .map(|c| format!("docs/knowledge/{}", c.path))
        .filter(|repo_rel| !accounted.contains(repo_rel))
        .collect();
    if !lost.is_empty() {
        return ManifestOut::Thrown(format!(
            "knowledge context: conservation — {} bee.critical concept(s) were neither included, truncated nor excluded: {} (G11). This is a bug in the ranking, not a condition of the bundle.",
            lost.len(),
            lost.join(", ")
        ));
    }

    let mut manifest = Map::new();
    manifest.insert("work".into(), Value::String(work_id.to_string()));
    manifest.insert("decisions".into(), Value::Array(decisions));
    manifest.insert("budget".into(), num(budget));
    manifest.insert("estimator".into(), Value::String(CONTEXT_ESTIMATOR.to_string()));
    manifest.insert("total_est".into(), num(total_est));
    manifest.insert("entries".into(), Value::Array(entries));
    manifest.insert("truncated".into(), Value::Array(truncated.into_iter().map(Value::String).collect()));
    manifest.insert("excluded".into(), Value::Array(excluded));
    manifest.insert("floor".into(), Value::Array(floor_paths.into_iter().map(Value::String).collect()));
    manifest.insert("critical_total".into(), Value::Number(Number::from(criticals.len())));
    manifest.insert("zero_signal_count".into(), Value::Number(Number::from(zero_signal_count)));
    ManifestOut::Built(Value::Object(manifest))
}

