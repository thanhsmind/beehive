// bee dev impact-registry — Rust port of scripts/impact_registry.mjs (cov-1,
// ci-owned-verify D3): the committed file -> suite relatedness registry that
// `run_verify.mjs --impacted` resolves through.
//
// PROVENANCE. Every function names its .mjs source. Two nearby ports exist
// and are deliberately NOT shared:
//   * src/verbs/cells.rs already carries a level-1 `queryRegistry` slice for
//     the porcelain side; it is module-private and cells.rs is not a file
//     this port may edit, so `query_registry` below is re-derived from the
//     same .mjs with the SAME semantics — including the one that makes the
//     level-1 slice subtle: a file is judged "known to the registry" by its
//     `all` list REGARDLESS of level, so a transitively-reachable file
//     contributes zero suites at level 1 without being UNMAPPED.
//   * the suite list comes from run_verify.mjs's own `SUITES` export, which
//     the .mjs imports live. A binary cannot `import()` a JS module, so
//     `discover_suites` re-derives it by parsing the four declarations that
//     produce it (DISCOVERY_ROOTS, EXCLUDE, ARGS_OVERRIDE, EXTRA_SUITES) and
//     re-running the same glob. Every anchor is verified; a run_verify.mjs
//     whose declarations no longer parse returns None and delegates rather
//     than computing a registry from a partial suite set.
//
// WHAT IT SCANS. Four regex-level edge types (no AST): static relative ESM
// imports/re-exports, `import(pathToFileURL(<expr>).href)` and literal
// dynamic imports, spawn/exec argv literals, and `runModuleWorker(<expr>)`
// first args — every `<expr>` resolved through the same tiny `const NAME =
// path.join(...)` tracker. The documented blind spots (call-indirection,
// env-pointed paths, readFileSync fixtures) are inherited exactly: an edge
// this port cannot resolve is ABSENT, never guessed.
//
// STRANGLER ROUTING. `--write`, `--check`, `--query <file...> [--level 1]`.
// A failure whose text is a V8 message (an unreadable/corrupt registry, an
// unreadable source) returns None before any output; every deterministic
// usage/refusal string is reproduced byte for byte.

use super::jspath;
use crate::jsjson;
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// provenance: impact_registry.mjs REGISTRY_PATH_REL.
const REGISTRY_PATH_REL: &str = "scripts/impact-registry.json";
/// provenance: impact_registry.mjs EDGE_SCAN_EXTENSIONS.
const EDGE_SCAN_EXTENSIONS: [&str; 3] = [".mjs", ".js", ".cjs"];
/// provenance: impact_registry.mjs RESOLVE_EXTS.
const RESOLVE_EXTS: [&str; 4] = ["", ".mjs", ".js", ".cjs"];

fn registry_path(root: &Path) -> PathBuf {
    root.join("scripts").join("impact-registry.json")
}

// ═══ path helpers (impact_registry.mjs, "path helpers") ════════════════════

struct Ctx {
    root: String,
    edge_cache: HashMap<String, Vec<String>>,
}

impl Ctx {
    fn new(root: &Path) -> Self {
        Ctx {
            root: root.to_string_lossy().into_owned(),
            edge_cache: HashMap::new(),
        }
    }

    /// provenance: toRepoRelative.
    fn to_repo_relative(&self, abs: &str) -> String {
        jspath::relative(&self.root, abs)
            .split(jspath::SEP)
            .collect::<Vec<_>>()
            .join("/")
    }

    /// provenance: toAbs.
    fn to_abs(&self, repo_rel: &str) -> String {
        let mut parts: Vec<&str> = vec![self.root.as_str()];
        parts.extend(repo_rel.split('/'));
        jspath::join(&parts)
    }
}

/// provenance: existsAsFile — `statSync(p).isFile()`, false on any throw.
fn exists_as_file(abs: &str) -> bool {
    std::fs::metadata(abs).map(|m| m.is_file()).unwrap_or(false)
}

/// provenance: resolveModuleFile — try the bare path, then each extension,
/// then index.mjs/index.js; best-effort otherwise (filtered downstream).
fn resolve_module_file(abs_no_ext: &str) -> String {
    for ext in RESOLVE_EXTS {
        let candidate = format!("{abs_no_ext}{ext}");
        if exists_as_file(&candidate) {
            return candidate;
        }
    }
    for idx in ["index.mjs", "index.js"] {
        let candidate = jspath::join(&[abs_no_ext, idx]);
        if exists_as_file(&candidate) {
            return candidate;
        }
    }
    abs_no_ext.to_string()
}

// ═══ the tiny expression resolver ══════════════════════════════════════════

fn is_quote(c: char) -> bool {
    c == '"' || c == '\'' || c == '`'
}

/// provenance: impact_registry.mjs splitTopLevelArgs — one linear
/// quote/bracket-depth scan; a `,` at depth 0 outside a quote splits.
fn split_top_level_args(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut args: Vec<String> = Vec::new();
    let mut depth: i64 = 0;
    let mut quote: Option<char> = None;
    let mut cur = String::new();
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if let Some(q) = quote {
            cur.push(c);
            if c == '\\' {
                i += 1;
                if i < chars.len() {
                    cur.push(chars[i]);
                }
                i += 1;
                continue;
            }
            if c == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        if is_quote(c) {
            quote = Some(c);
            cur.push(c);
            i += 1;
            continue;
        }
        if c == '(' || c == '[' || c == '{' {
            depth += 1;
            cur.push(c);
            i += 1;
            continue;
        }
        if c == ')' || c == ']' || c == '}' {
            depth -= 1;
            cur.push(c);
            i += 1;
            continue;
        }
        if c == ',' && depth == 0 {
            args.push(std::mem::take(&mut cur));
            i += 1;
            continue;
        }
        cur.push(c);
        i += 1;
    }
    if !cur.trim().is_empty() {
        args.push(cur);
    }
    args
}

/// provenance: /^["'`]([^"'`]*)["'`]$/ — note the two ends are INDEPENDENT
/// character classes in the .mjs, so a mismatched pair still matches.
fn quoted_literal(expr: &str) -> Option<&str> {
    let chars: Vec<char> = expr.chars().collect();
    if chars.len() < 2 || !is_quote(chars[0]) || !is_quote(chars[chars.len() - 1]) {
        return None;
    }
    let start = chars[0].len_utf8();
    let end = expr.len() - chars[chars.len() - 1].len_utf8();
    if start > end {
        return None;
    }
    let inner = &expr[start..end];
    if inner.chars().any(is_quote) {
        return None;
    }
    Some(inner)
}

/// `expr.starts_with(head)` and `expr.ends_with(")")` — the greedy
/// `([\s\S]*)\)$` capture is everything between.
fn call_inner<'a>(expr: &'a str, head: &str) -> Option<&'a str> {
    let rest = expr.strip_prefix(head)?;
    rest.strip_suffix(')')
}

/// provenance: impact_registry.mjs resolveJoinArg.
fn resolve_join_arg(
    arg_raw: &str,
    vars: &HashMap<String, String>,
    file_abs: &str,
    file_dir: &str,
) -> Option<String> {
    let arg = arg_raw.trim();
    if let Some(lit) = quoted_literal(arg) {
        return Some(lit.to_string()); // a literal path.join segment, used as-is
    }
    resolve_expr_to_abs(arg, vars, file_abs, file_dir)
}

/// provenance: impact_registry.mjs resolveExprToAbs — the whole shape list,
/// in the .mjs's own order. Anything else is `undefined`, a documented blind
/// spot, never a guess.
fn resolve_expr_to_abs(
    expr_raw: &str,
    vars: &HashMap<String, String>,
    file_abs: &str,
    file_dir: &str,
) -> Option<String> {
    let expr = expr_raw.trim();
    if expr.is_empty() {
        return None;
    }
    if expr == "__dirname" {
        return Some(file_dir.to_string());
    }
    if expr == "__filename" {
        return Some(file_abs.to_string());
    }
    if let Some(v) = vars.get(expr) {
        return Some(v.clone());
    }
    if let Some(inner) = call_inner(expr, "fileURLToPath(") {
        if inner.trim() == "import.meta.url" {
            return Some(file_abs.to_string());
        }
    }
    if let Some(inner) = call_inner(expr, "path.dirname(") {
        // /^path\.dirname\(\s*fileURLToPath\(\s*import\.meta\.url\s*\)\s*\)$/
        if let Some(deep) = call_inner(inner.trim(), "fileURLToPath(") {
            if deep.trim() == "import.meta.url" {
                return Some(file_dir.to_string());
            }
        }
        // /^path\.dirname\(([\s\S]*)\)$/
        let resolved = resolve_expr_to_abs(inner, vars, file_abs, file_dir)?;
        return Some(jspath::dirname(&resolved));
    }
    for (head, kind) in [("path.join(", "join"), ("path.resolve(", "resolve")] {
        let Some(inner) = call_inner(expr, head) else { continue };
        let arg_list: Vec<String> = split_top_level_args(inner)
            .into_iter()
            .map(|a| a.trim().to_string())
            .filter(|a| !a.is_empty())
            .collect();
        let mut parts: Vec<String> = Vec::new();
        for a in &arg_list {
            if a.starts_with("...") {
                return None; // spread — unresolvable
            }
            parts.push(resolve_join_arg(a, vars, file_abs, file_dir)?);
        }
        if parts.is_empty() {
            return None;
        }
        let refs: Vec<&str> = parts.iter().map(String::as_str).collect();
        return Some(if kind == "join" { jspath::join(&refs) } else { jspath::resolve(&refs) });
    }
    if let Some(lit) = quoted_literal(expr) {
        if lit.starts_with('.') || lit.starts_with('/') {
            return Some(jspath::resolve(&[file_dir, lit]));
        }
        return None;
    }
    None
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// `\b` before position `i` in `chars`.
fn word_boundary_before(chars: &[char], i: usize) -> bool {
    i == 0 || !is_word_char(chars[i - 1])
}

/// provenance: impact_registry.mjs extractVars with
/// CONST_ASSIGN_RE = /\b(?:const|let)\s+(\w+)\s*=\s*([^;]+);/g — scanned in
/// source order so a later assignment can reference an earlier one.
fn extract_vars(source: &str, file_abs: &str, file_dir: &str) -> HashMap<String, String> {
    let chars: Vec<char> = source.chars().collect();
    let mut vars: HashMap<String, String> = HashMap::new();
    let mut i = 0usize;
    'outer: while i < chars.len() {
        for kw in ["const", "let"] {
            let k: Vec<char> = kw.chars().collect();
            if i + k.len() > chars.len() || chars[i..i + k.len()] != k[..] {
                continue;
            }
            if !word_boundary_before(&chars, i) {
                continue;
            }
            let mut j = i + k.len();
            let ws_start = j;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j == ws_start {
                continue; // `\s+`
            }
            let name_start = j;
            while j < chars.len() && is_word_char(chars[j]) {
                j += 1;
            }
            if j == name_start {
                continue; // `(\w+)`
            }
            let name: String = chars[name_start..j].iter().collect();
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j >= chars.len() || chars[j] != '=' {
                continue;
            }
            j += 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            let expr_start = j;
            while j < chars.len() && chars[j] != ';' {
                j += 1;
            }
            if j >= chars.len() || j == expr_start {
                continue; // `([^;]+);` needs at least one char and a `;`
            }
            let expr: String = chars[expr_start..j].iter().collect();
            if let Some(resolved) = resolve_expr_to_abs(&expr, &vars, file_abs, file_dir) {
                vars.insert(name, resolved);
            }
            i = j + 1; // regex lastIndex lands after the `;`
            continue 'outer;
        }
        i += 1;
    }
    vars
}

// ═══ balanced call-argument extraction ═════════════════════════════════════

/// provenance: impact_registry.mjs findMatchingParen.
fn find_matching_paren(chars: &[char], open_idx: usize) -> Option<usize> {
    let mut depth: i64 = 0;
    let mut quote: Option<char> = None;
    let mut i = open_idx;
    while i < chars.len() {
        let c = chars[i];
        if let Some(q) = quote {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        if is_quote(c) {
            quote = Some(c);
            i += 1;
            continue;
        }
        if c == '(' {
            depth += 1;
        } else if c == ')' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// provenance: impact_registry.mjs extractCallArgsList — one `<callee>(`
/// anchor per call site, then a balanced scan. A callee whose parens never
/// balance advances the cursor by one, exactly as the regex's lastIndex does.
fn extract_call_args_list(chars: &[char], callees: &[&str]) -> Vec<String> {
    let mut results = Vec::new();
    let mut i = 0usize;
    'outer: while i < chars.len() {
        for callee in callees {
            let k: Vec<char> = callee.chars().collect();
            if i + k.len() >= chars.len() || chars[i..i + k.len()] != k[..] {
                continue;
            }
            if chars[i + k.len()] != '(' || !word_boundary_before(chars, i) {
                continue;
            }
            let open_idx = i + k.len();
            match find_matching_paren(chars, open_idx) {
                Some(close_idx) => {
                    results.push(chars[open_idx + 1..close_idx].iter().collect());
                    i = close_idx + 1;
                }
                None => i = open_idx + 1,
            }
            continue 'outer;
        }
        i += 1;
    }
    results
}

// ═══ the four edge types ═══════════════════════════════════════════════════

/// Extract statements matched by `/\b(import|export)\s[^;]*?;/g`.
fn statements(chars: &[char], keyword: &str) -> Vec<String> {
    let k: Vec<char> = keyword.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        if i + k.len() < chars.len()
            && chars[i..i + k.len()] == k[..]
            && word_boundary_before(chars, i)
            && chars[i + k.len()].is_whitespace()
        {
            if let Some(rel) = chars[i + k.len()..].iter().position(|c| *c == ';') {
                let end = i + k.len() + rel;
                out.push(chars[i..=end].iter().collect());
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// `stmt.match(/\bfrom\s+["']([^"']+)["']/)` — the FIRST occurrence.
fn from_specifier(stmt: &str) -> Option<String> {
    let chars: Vec<char> = stmt.chars().collect();
    let k = ['f', 'r', 'o', 'm'];
    let mut i = 0usize;
    while i + 4 < chars.len() {
        if chars[i..i + 4] == k && word_boundary_before(&chars, i) && chars[i + 4].is_whitespace() {
            let mut j = i + 4;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if let Some(spec) = single_or_double_quoted(&chars, j) {
                return Some(spec);
            }
        }
        i += 1;
    }
    None
}

/// `["']([^"']+)["']` starting exactly at `j`.
fn single_or_double_quoted(chars: &[char], j: usize) -> Option<String> {
    if j >= chars.len() || (chars[j] != '"' && chars[j] != '\'') {
        return None;
    }
    let mut k = j + 1;
    let mut body = String::new();
    while k < chars.len() && chars[k] != '"' && chars[k] != '\'' {
        body.push(chars[k]);
        k += 1;
    }
    if k >= chars.len() || body.is_empty() {
        return None; // `[^"']+` needs at least one char, and a closing quote
    }
    Some(body)
}

/// provenance: impact_registry.mjs staticImportEdges.
fn static_import_edges(chars: &[char], file_dir: &str, edges: &mut Vec<String>) {
    for keyword in ["import", "export"] {
        for stmt in statements(chars, keyword) {
            let specifier = from_specifier(&stmt).or_else(|| {
                // `stmt.match(/^import\s+["']([^"']+)["']/)`
                if keyword != "import" {
                    return None;
                }
                let sc: Vec<char> = stmt.chars().collect();
                let mut j = 6usize; // after "import"
                let ws = j;
                while j < sc.len() && sc[j].is_whitespace() {
                    j += 1;
                }
                if j == ws {
                    return None;
                }
                single_or_double_quoted(&sc, j)
            });
            let Some(specifier) = specifier else { continue };
            if !(specifier.starts_with('.') || specifier.starts_with('/')) {
                continue; // bare / node: specifiers
            }
            push_unique(edges, resolve_module_file(&jspath::resolve(&[file_dir, &specifier])));
        }
    }
}

/// provenance: impact_registry.mjs dynamicImportEdges.
fn dynamic_import_edges(
    chars: &[char],
    vars: &HashMap<String, String>,
    file_abs: &str,
    file_dir: &str,
    edges: &mut Vec<String>,
) {
    for args_str in extract_call_args_list(chars, &["import"]) {
        let trimmed = args_str.trim();
        // /^pathToFileURL\(([\s\S]*)\)\.href$/
        if let Some(rest) = trimmed.strip_suffix(".href") {
            if let Some(inner) = call_inner(rest, "pathToFileURL(") {
                if let Some(resolved) = resolve_expr_to_abs(inner, vars, file_abs, file_dir) {
                    if !resolved.is_empty() {
                        push_unique(edges, resolve_module_file(&resolved));
                    }
                }
                continue;
            }
        }
        if let Some(specifier) = quoted_literal(trimmed) {
            if specifier.starts_with('.') || specifier.starts_with('/') {
                push_unique(edges, resolve_module_file(&jspath::resolve(&[file_dir, specifier])));
            }
            continue;
        }
        if let Some(resolved) = resolve_expr_to_abs(trimmed, vars, file_abs, file_dir) {
            if !resolved.is_empty() {
                push_unique(edges, resolve_module_file(&resolved));
            }
        }
    }
}

/// provenance: impact_registry.mjs spawnArgvEdges — any argv token that
/// resolves to a real repo file becomes an edge, which is how a suite that
/// spawns `.bee/bin/bee.mjs` inherits bee.mjs's whole closure.
fn spawn_argv_edges(
    chars: &[char],
    vars: &HashMap<String, String>,
    file_abs: &str,
    file_dir: &str,
    edges: &mut Vec<String>,
) {
    for args_str in
        extract_call_args_list(chars, &["spawn", "spawnSync", "execFile", "execFileSync"])
    {
        let parts: Vec<String> = split_top_level_args(&args_str)
            .into_iter()
            .map(|p| p.trim().to_string())
            .collect();
        let Some(arr_part) = parts.iter().find(|p| p.starts_with('[')) else { continue };
        // `.replace(/^\[/, "").replace(/\]$/, "")`
        let inner = arr_part.strip_prefix('[').unwrap_or(arr_part);
        let inner = inner.strip_suffix(']').unwrap_or(inner);
        for tok_raw in split_top_level_args(inner) {
            let tok = tok_raw.trim();
            if tok.is_empty() || tok.starts_with("...") {
                continue;
            }
            if let Some(resolved) = resolve_expr_to_abs(tok, vars, file_abs, file_dir) {
                if !resolved.is_empty() {
                    push_unique(edges, resolve_module_file(&resolved));
                }
            }
        }
    }
}

/// provenance: impact_registry.mjs runModuleWorkerEdges.
fn run_module_worker_edges(
    chars: &[char],
    vars: &HashMap<String, String>,
    file_abs: &str,
    file_dir: &str,
    edges: &mut Vec<String>,
) {
    for args_str in extract_call_args_list(chars, &["runModuleWorker"]) {
        let parts = split_top_level_args(&args_str);
        let Some(first) = parts.first() else { continue };
        if let Some(resolved) = resolve_expr_to_abs(first, vars, file_abs, file_dir) {
            if !resolved.is_empty() {
                push_unique(edges, resolve_module_file(&resolved));
            }
        }
    }
}

/// `Set#add` semantics over an insertion-ordered Vec.
fn push_unique(v: &mut Vec<String>, item: String) {
    if !v.contains(&item) {
        v.push(item);
    }
}

impl Ctx {
    /// provenance: impact_registry.mjs getEdges (with its per-run cache).
    fn get_edges(&mut self, repo_rel: &str) -> Vec<String> {
        if let Some(cached) = self.edge_cache.get(repo_rel) {
            return cached.clone();
        }
        let abs = self.to_abs(repo_rel);
        let mut raw: Vec<String> = Vec::new();
        let ext = jspath::extname(repo_rel);
        if EDGE_SCAN_EXTENSIONS.contains(&ext.as_str()) && exists_as_file(&abs) {
            let source = std::fs::read(&abs)
                .map(|b| String::from_utf8_lossy(&b).into_owned())
                .unwrap_or_default();
            let chars: Vec<char> = source.chars().collect();
            let file_dir = jspath::dirname(&abs);
            let vars = extract_vars(&source, &abs, &file_dir);
            static_import_edges(&chars, &file_dir, &mut raw);
            dynamic_import_edges(&chars, &vars, &abs, &file_dir, &mut raw);
            spawn_argv_edges(&chars, &vars, &abs, &file_dir, &mut raw);
            run_module_worker_edges(&chars, &vars, &abs, &file_dir, &mut raw);
        }
        let mut rel_edges: Vec<String> = Vec::new();
        for abs_edge in raw {
            if !exists_as_file(&abs_edge) {
                continue;
            }
            let rel = self.to_repo_relative(&abs_edge);
            if rel.is_empty() || rel.starts_with("..") {
                continue;
            }
            push_unique(&mut rel_edges, rel);
        }
        self.edge_cache.insert(repo_rel.to_string(), rel_edges.clone());
        rel_edges
    }

    /// provenance: impact_registry.mjs closureFor — BFS over all four edge
    /// types, the visited set being the closure.
    fn closure_for(&mut self, entry: &str) -> Vec<String> {
        let mut visited: Vec<String> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        let mut queue: std::collections::VecDeque<String> =
            std::collections::VecDeque::from([entry.to_string()]);
        while let Some(cur) = queue.pop_front() {
            if !seen.insert(cur.clone()) {
                continue;
            }
            visited.push(cur.clone());
            for child in self.get_edges(&cur) {
                if !seen.contains(&child) {
                    queue.push_back(child);
                }
            }
        }
        visited
    }
}

// ═══ suite discovery (run_verify.mjs SUITES, re-derived) ═══════════════════

/// Strip `//` line comments from one line, honouring `'`/`"` string state
/// within that line. The regions parsed here hold no multi-line strings, so
/// per-line state is exact — and a comment's own apostrophes and backticks
/// can never matter because everything after the `//` is dropped.
fn strip_line_comment(line: &str) -> &str {
    let chars: Vec<char> = line.chars().collect();
    let mut quote: Option<char> = None;
    let mut byte = 0usize;
    for i in 0..chars.len() {
        let c = chars[i];
        match quote {
            Some(q) => {
                if c == '\\' {
                    // skip the escaped char
                } else if c == q {
                    quote = None;
                }
            }
            None => {
                if c == '"' || c == '\'' {
                    quote = Some(c);
                } else if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
                    return &line[..byte];
                }
            }
        }
        byte += c.len_utf8();
    }
    line
}

/// Collect the lines of a `const NAME = [` / `new Set([` declaration, from
/// the anchor to the line that closes it, comments already stripped.
fn declaration_body(source: &str, anchor: &str, closers: &[&str]) -> Option<String> {
    let mut lines = source.lines();
    let start = lines.by_ref().find(|l| l.trim_start().starts_with(anchor))?;
    // Single-line form: `const X = new Set([...]);`
    let head = strip_line_comment(start).trim_end();
    if closers.iter().any(|c| head.ends_with(c)) {
        let after = head.find(anchor)? + anchor.len();
        let body = &head[after..];
        let end = closers.iter().filter_map(|c| body.rfind(c)).max()?;
        return Some(body[..end].to_string());
    }
    let mut body = String::new();
    for line in lines {
        let stripped = strip_line_comment(line).trim_end();
        if closers.iter().any(|c| stripped.trim() == *c) {
            return Some(body);
        }
        body.push_str(stripped);
        body.push('\n');
    }
    None
}

/// Every `"…"`/`'…'` string literal in `body`, in order.
fn string_literals(body: &str) -> Vec<String> {
    let chars: Vec<char> = body.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '"' || chars[i] == '\'' {
            let q = chars[i];
            let mut j = i + 1;
            let mut s = String::new();
            while j < chars.len() && chars[j] != q {
                if chars[j] == '\\' && j + 1 < chars.len() {
                    j += 1;
                }
                s.push(chars[j]);
                j += 1;
            }
            if j >= chars.len() {
                return out; // unterminated — stop rather than guess
            }
            out.push(s);
            i = j + 1;
            continue;
        }
        i += 1;
    }
    out
}

/// Parse `[["a","b"], ["c"]]`-shaped bodies into suite entries.
fn string_array_arrays(body: &str) -> Option<Vec<Vec<String>>> {
    let chars: Vec<char> = body.chars().collect();
    let mut out: Vec<Vec<String>> = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        match chars[i] {
            '[' => {
                let mut depth = 1i64;
                let mut j = i + 1;
                while j < chars.len() && depth > 0 {
                    match chars[j] {
                        '[' => depth += 1,
                        ']' => depth -= 1,
                        '"' | '\'' => {
                            let q = chars[j];
                            j += 1;
                            while j < chars.len() && chars[j] != q {
                                if chars[j] == '\\' {
                                    j += 1;
                                }
                                j += 1;
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }
                if depth != 0 {
                    return None;
                }
                let inner: String = chars[i + 1..j - 1].iter().collect();
                if inner.contains('[') {
                    return None; // deeper nesting is not this shape
                }
                let items = string_literals(&inner);
                if items.is_empty() {
                    return None;
                }
                out.push(items);
                i = j;
            }
            c if c.is_whitespace() || c == ',' => i += 1,
            _ => return None, // an unexpected token — refuse rather than guess
        }
    }
    Some(out)
}

/// provenance: run_verify.mjs discoverSuites + the four declarations feeding
/// it. Returns None when any anchor is missing or unparsable.
fn discover_suites(root: &Path) -> Option<Vec<Vec<String>>> {
    let source = std::fs::read_to_string(root.join("scripts").join("run_verify.mjs")).ok()?;
    let discovery_roots =
        string_literals(&declaration_body(&source, "const DISCOVERY_ROOTS = [", &["];"])?);
    if discovery_roots.is_empty() {
        return None;
    }
    let exclude: Vec<String> =
        string_literals(&declaration_body(&source, "const EXCLUDE = new Set([", &["]);"])?);
    let args_override: Vec<String> = string_literals(&declaration_body(
        &source,
        "const ARGS_OVERRIDE = new Set([",
        &["]);"],
    )?);
    let extra_suites =
        string_array_arrays(&declaration_body(&source, "const EXTRA_SUITES = [", &["];"])?)?;
    if extra_suites.is_empty() {
        return None;
    }

    let mut found: Vec<Vec<String>> = Vec::new();
    for disc in &discovery_roots {
        let dir = {
            let mut parts: Vec<&str> = vec![root.to_str()?];
            parts.extend(disc.split('/'));
            jspath::join(&parts)
        };
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let Ok(ft) = entry.file_type() else { continue };
            if !ft.is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with("test_") || !name.ends_with(".mjs") {
                continue;
            }
            let rel = format!("{disc}/{name}");
            if exclude.contains(&rel) || args_override.contains(&rel) {
                continue;
            }
            found.push(vec![rel]);
        }
    }
    // `found.sort((a, b) => a[0].localeCompare(b[0]))`
    if !super::sort_by_locale(&mut found, |e| e[0].as_str()) {
        return None;
    }
    found.extend(extra_suites);
    Some(found)
}

// ═══ registry build / (de)serialize / query ════════════════════════════════

/// provenance: impact_registry.mjs suiteLabel.
fn suite_label(entry: &[String]) -> String {
    entry.join(" ")
}

/// provenance: impact_registry.mjs buildRegistry — `{version: 2, files:
/// {<rel>: {direct: [...], all: [...]}}}`, keys and suite lists in bare
/// `.sort()` (code-unit) order.
fn build_registry(root: &Path) -> Option<Value> {
    let suites = discover_suites(root)?;
    let mut ctx = Ctx::new(root);
    let mut file_to_all: HashMap<String, Vec<String>> = HashMap::new();
    let mut file_to_direct: HashMap<String, Vec<String>> = HashMap::new();
    for entry in &suites {
        let label = suite_label(entry);
        for f in ctx.closure_for(&entry[0]) {
            let slot = file_to_all.entry(f).or_default();
            if !slot.contains(&label) {
                slot.push(label.clone());
            }
        }
        let mut direct: Vec<String> = vec![entry[0].clone()];
        for e in ctx.get_edges(&entry[0]) {
            push_unique(&mut direct, e);
        }
        for f in direct {
            let slot = file_to_direct.entry(f).or_default();
            if !slot.contains(&label) {
                slot.push(label.clone());
            }
        }
    }
    let mut all_keys: Vec<String> = file_to_all
        .keys()
        .chain(file_to_direct.keys())
        .cloned()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    super::js_default_sort(&mut all_keys);

    let mut files = Map::new();
    for key in all_keys {
        let mut direct = file_to_direct.get(&key).cloned().unwrap_or_default();
        let mut all = file_to_all.get(&key).cloned().unwrap_or_default();
        super::js_default_sort(&mut direct);
        super::js_default_sort(&mut all);
        let mut o = Map::new();
        o.insert(
            "direct".into(),
            Value::Array(direct.into_iter().map(Value::String).collect()),
        );
        o.insert("all".into(), Value::Array(all.into_iter().map(Value::String).collect()));
        files.insert(key, Value::Object(o));
    }
    let mut registry = Map::new();
    registry.insert("version".into(), Value::Number(2.into()));
    registry.insert("files".into(), Value::Object(files));
    Some(Value::Object(registry))
}

/// provenance: impact_registry.mjs serializeRegistry.
fn serialize_registry(registry: &Value) -> String {
    format!("{}\n", jsjson::stringify_pretty(registry))
}

/// provenance: impact_registry.mjs normalizeQueryPath.
fn normalize_query_path(ctx: &Ctx, input: &str) -> String {
    let abs = if jspath::is_absolute(input) {
        input.to_string()
    } else {
        let cwd = std::env::current_dir()
            .map(|c| c.to_string_lossy().into_owned())
            .unwrap_or_default();
        jspath::resolve(&[&cwd, input])
    };
    ctx.to_repo_relative(&abs)
}

/// provenance: impact_registry.mjs queryRegistry. `level == Some(1)` narrows
/// to DIRECT edges; "known to the registry at all" is judged by `all`
/// REGARDLESS of level, so a transitively-reachable file contributes zero
/// suites at level 1 without being reported UNMAPPED — the same semantics
/// verbs/cells.rs's private level-1 slice encodes.
fn query_registry(
    ctx: &Ctx,
    registry: &Value,
    files: &[&str],
    level: Option<u32>,
) -> (Vec<String>, Vec<String>) {
    let mut mapped: Vec<String> = Vec::new();
    let mut unmapped: Vec<String> = Vec::new();
    for f in files {
        let rel = normalize_query_path(ctx, f);
        let entry = registry.get("files").and_then(|v| v.get(&rel));
        let all = entry.and_then(|e| e.get("all")).and_then(Value::as_array);
        let Some(all) = all.filter(|a| !a.is_empty()) else {
            unmapped.push(rel);
            continue;
        };
        let suites = if level == Some(1) {
            entry
                .and_then(|e| e.get("direct"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
        } else {
            all.clone()
        };
        for s in suites {
            if let Some(s) = s.as_str() {
                push_unique(&mut mapped, s.to_string());
            }
        }
    }
    super::js_default_sort(&mut mapped);
    (mapped, unmapped)
}

// ═══ CLI ═══════════════════════════════════════════════════════════════════

pub(super) fn run(args: &[&str]) -> Option<ExitCode> {
    let root = super::bee_source_root()?;
    let mode = args.first().copied();

    if mode == Some("--write") {
        let registry = build_registry(&root)?;
        let json = serialize_registry(&registry);
        let count = registry["files"].as_object().map(Map::len).unwrap_or(0);
        if std::fs::write(registry_path(&root), &json).is_err() {
            return None;
        }
        println!("impact_registry --write: wrote {REGISTRY_PATH_REL} ({count} files)");
        return Some(ExitCode::SUCCESS);
    }

    if mode == Some("--check") {
        let registry = build_registry(&root)?;
        let expected = serialize_registry(&registry);
        let actual = std::fs::read_to_string(registry_path(&root)).ok();
        if actual.as_deref() == Some(expected.as_str()) {
            println!("impact_registry --check: {REGISTRY_PATH_REL} is up to date");
            return Some(ExitCode::SUCCESS);
        }
        eprintln!(
            "{}",
            if actual.is_none() {
                format!("impact_registry --check: {REGISTRY_PATH_REL} is missing.")
            } else {
                format!("impact_registry --check: {REGISTRY_PATH_REL} is STALE (drift detected).")
            }
        );
        eprintln!("FIX: node scripts/impact_registry.mjs --write");
        return Some(ExitCode::FAILURE);
    }

    if mode == Some("--query") {
        let rest = &args[1..];
        let mut level: Option<u32> = None;
        let mut query_files: Vec<&str> = Vec::new();
        let mut i = 0usize;
        while i < rest.len() {
            if rest[i] == "--level" {
                let value = rest.get(i + 1);
                if value != Some(&"1") {
                    // `JSON.stringify(value)` — `undefined` when absent.
                    let shown = match value {
                        Some(v) => jsjson::stringify(&Value::String((*v).to_string())),
                        None => "undefined".to_string(),
                    };
                    eprintln!(
                        "usage: node scripts/impact_registry.mjs --query <file...> [--level 1] (got --level {shown})"
                    );
                    return Some(ExitCode::FAILURE);
                }
                level = Some(1);
                i += 2;
                continue;
            }
            query_files.push(rest[i]);
            i += 1;
        }
        if query_files.is_empty() {
            eprintln!("usage: node scripts/impact_registry.mjs --query <file...> [--level 1]");
            return Some(ExitCode::FAILURE);
        }
        // A read/parse failure prints the V8 message — delegate instead.
        let text = std::fs::read_to_string(registry_path(&root)).ok()?;
        let registry: Value = serde_json::from_str(&text).ok()?;
        let ctx = Ctx::new(&root);
        let (mapped, unmapped) = query_registry(&ctx, &registry, &query_files, level);
        for u in &unmapped {
            eprintln!("UNMAPPED: {u} (no known suite relates to this file — full verify still covers it)");
        }
        for s in &mapped {
            println!("{s}");
        }
        return Some(ExitCode::SUCCESS);
    }

    eprintln!(
        "usage: node scripts/impact_registry.mjs --write | --check | --query <file...> [--level 1]"
    );
    Some(ExitCode::from(2))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("..").join("..")
    }

    #[test]
    fn split_top_level_args_respects_quotes_and_brackets() {
        assert_eq!(split_top_level_args("a, b"), ["a", " b"]);
        assert_eq!(split_top_level_args("f(a, b), c"), ["f(a, b)", " c"]);
        assert_eq!(split_top_level_args("\"x,y\", z"), ["\"x,y\"", " z"]);
        assert_eq!(split_top_level_args("[a, b], c"), ["[a, b]", " c"]);
        assert_eq!(split_top_level_args("'a\\'b', c"), ["'a\\'b'", " c"]);
        // a trailing all-whitespace arg is dropped (`cur.trim() !== ""`)
        assert_eq!(split_top_level_args("a,  "), ["a"]);
    }

    #[test]
    fn quoted_literal_matches_the_mjs_char_classes() {
        assert_eq!(quoted_literal("\"abc\""), Some("abc"));
        assert_eq!(quoted_literal("'abc'"), Some("abc"));
        assert_eq!(quoted_literal("`abc`"), Some("abc"));
        assert_eq!(quoted_literal("\"abc'"), Some("abc")); // ends are independent classes
        assert_eq!(quoted_literal("\"a\"b\""), None); // inner quote
        assert_eq!(quoted_literal("abc"), None);
        assert_eq!(quoted_literal("\"\""), Some(""));
    }

    #[test]
    fn expression_resolver_handles_every_shape_it_claims() {
        let file_abs = jspath::join(&["/repo", "scripts", "x.mjs"]);
        let file_dir = jspath::dirname(&file_abs);
        let mut vars = HashMap::new();
        vars.insert("REPO_ROOT".to_string(), jspath::join(&["/repo"]));
        let r = |e: &str| resolve_expr_to_abs(e, &vars, &file_abs, &file_dir);

        assert_eq!(r("__dirname").as_deref(), Some(file_dir.as_str()));
        assert_eq!(r("__filename").as_deref(), Some(file_abs.as_str()));
        assert_eq!(r("fileURLToPath(import.meta.url)").as_deref(), Some(file_abs.as_str()));
        assert_eq!(
            r("path.dirname(fileURLToPath( import.meta.url ))").as_deref(),
            Some(file_dir.as_str())
        );
        assert_eq!(r("REPO_ROOT"), Some(jspath::join(&["/repo"])));
        assert_eq!(
            r("path.join(REPO_ROOT, \".bee\", \"bin\", \"bee.mjs\")"),
            Some(jspath::join(&["/repo", ".bee", "bin", "bee.mjs"]))
        );
        assert_eq!(r("path.dirname(__filename)").as_deref(), Some(file_dir.as_str()));
        assert_eq!(r("\"./sibling.mjs\""), Some(jspath::join(&["/repo/scripts", "sibling.mjs"])));
        // blind spots stay blind
        assert_eq!(r("beeModulePath()"), None);
        assert_eq!(r("path.join(...parts)"), None);
        assert_eq!(r("\"bare-specifier\""), None);
        assert_eq!(r(""), None);
    }

    #[test]
    fn extract_vars_chains_assignments_in_source_order() {
        let file_abs = jspath::join(&["/repo", "scripts", "x.mjs"]);
        let file_dir = jspath::dirname(&file_abs);
        let source = r#"
const __dirname2 = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname2, "..");
const BEE = path.join(REPO_ROOT, ".bee", "bin", "bee.mjs");
const NOPE = someCall();
let LATER = path.join(REPO_ROOT, "lib");
"#;
        let vars = extract_vars(source, &file_abs, &file_dir);
        assert_eq!(vars.get("REPO_ROOT").map(String::as_str), Some(jspath::join(&["/repo"]).as_str()));
        assert_eq!(vars.get("BEE").cloned(), Some(jspath::join(&["/repo", ".bee/bin/bee.mjs"])));
        assert_eq!(vars.get("LATER").cloned(), Some(jspath::join(&["/repo", "lib"])));
        assert!(!vars.contains_key("NOPE"));
    }

    #[test]
    fn call_arg_extraction_is_paren_balanced() {
        let src: Vec<char> = "spawnSync(node, [A, path.join(B, \"c)\")], {cwd})".chars().collect();
        let args = extract_call_args_list(&src, &["spawn", "spawnSync", "execFile", "execFileSync"]);
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], "node, [A, path.join(B, \"c)\")], {cwd}");
        // `\b` holds after a `.`
        let m: Vec<char> = "child_process.spawn(a, [b])".chars().collect();
        assert_eq!(extract_call_args_list(&m, &["spawn"]).len(), 1);
        // …and not mid-identifier
        let n: Vec<char> = "respawn(a, [b])".chars().collect();
        assert_eq!(extract_call_args_list(&n, &["spawn"]).len(), 0);
    }

    #[test]
    fn statement_scanner_takes_the_shortest_run_to_a_semicolon() {
        let src: Vec<char> = "import x from \"./a.mjs\";\nimport \"./b.mjs\";\nexport { y } from \"./c.mjs\";".chars().collect();
        let imports = statements(&src, "import");
        assert_eq!(imports.len(), 2);
        assert_eq!(from_specifier(&imports[0]).as_deref(), Some("./a.mjs"));
        assert_eq!(from_specifier(&imports[1]), None);
        let exports = statements(&src, "export");
        assert_eq!(from_specifier(&exports[0]).as_deref(), Some("./c.mjs"));
    }

    // ── edges over a fixture repo ─────────────────────────────────────────

    #[test]
    fn edges_cover_all_four_types_over_a_fixture() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("scripts").join("tests")).unwrap();
        std::fs::create_dir_all(root.join("lib")).unwrap();
        std::fs::write(root.join("lib").join("dep.mjs"), b"export const a = 1;\n").unwrap();
        std::fs::write(root.join("lib").join("deeper.mjs"), b"export const b = 2;\n").unwrap();
        std::fs::write(root.join("cli.mjs"), b"import './lib/deeper.mjs';\n").unwrap();
        std::fs::write(root.join("worker.mjs"), b"export const w = 3;\n").unwrap();
        std::fs::write(
            root.join("scripts").join("tests").join("test_a.mjs"),
            br#"
import { a } from "../../lib/dep.mjs";
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..", "..");
const CLI = path.join(REPO_ROOT, "cli.mjs");
spawnSync("node", [CLI, "--json"], { cwd: REPO_ROOT });
await import(pathToFileURL(path.join(REPO_ROOT, "worker.mjs")).href);
runModuleWorker(path.join(REPO_ROOT, "lib", "deeper.mjs"), {});
"#,
        )
        .unwrap();

        let mut ctx = Ctx::new(root);
        let mut edges = ctx.get_edges("scripts/tests/test_a.mjs");
        edges.sort();
        assert_eq!(edges, ["cli.mjs", "lib/deeper.mjs", "lib/dep.mjs", "worker.mjs"]);

        // BFS inherits cli.mjs's own static import.
        let mut closure = ctx.closure_for("scripts/tests/test_a.mjs");
        closure.sort();
        assert_eq!(
            closure,
            ["cli.mjs", "lib/deeper.mjs", "lib/dep.mjs", "scripts/tests/test_a.mjs", "worker.mjs"]
        );
    }

    // ── suite-list parsing ────────────────────────────────────────────────

    #[test]
    fn declaration_parsing_survives_comments_and_apostrophes() {
        let source = r#"
const DISCOVERY_ROOTS = [
  "scripts/tests", // the suite's home
  "packages/bee/tests",
];
const EXCLUDE = new Set([]);
const EXTRA_SUITES = [
  ["scripts/release_manifest.mjs", "--selftest"],
  // a comment with a `backtick`, an apostrophe's worth of prose, and [brackets]
  ["scripts/ledger_parity.mjs", "--check"],
];
const ARGS_OVERRIDE = new Set(["scripts/tests/test_installers_e2e.mjs"]);
"#;
        assert_eq!(
            string_literals(&declaration_body(source, "const DISCOVERY_ROOTS = [", &["];"]).unwrap()),
            ["scripts/tests", "packages/bee/tests"]
        );
        assert!(string_literals(
            &declaration_body(source, "const EXCLUDE = new Set([", &["]);"]).unwrap()
        )
        .is_empty());
        assert_eq!(
            string_literals(
                &declaration_body(source, "const ARGS_OVERRIDE = new Set([", &["]);"]).unwrap()
            ),
            ["scripts/tests/test_installers_e2e.mjs"]
        );
        let extra = string_array_arrays(
            &declaration_body(source, "const EXTRA_SUITES = [", &["];"]).unwrap(),
        )
        .unwrap();
        assert_eq!(
            extra,
            [
                vec!["scripts/release_manifest.mjs".to_string(), "--selftest".into()],
                vec!["scripts/ledger_parity.mjs".to_string(), "--check".into()],
            ]
        );
    }

    #[test]
    fn strip_line_comment_ignores_slashes_inside_strings() {
        assert_eq!(strip_line_comment("  \"a/b.mjs\", // note"), "  \"a/b.mjs\", ");
        assert_eq!(strip_line_comment("  \"http://x\","), "  \"http://x\",");
        assert_eq!(strip_line_comment("plain"), "plain");
    }

    /// The live run_verify.mjs must still parse — this is what makes a
    /// declaration rename a loud failure instead of a silently short suite
    /// list.
    #[test]
    fn live_run_verify_suites_parse() {
        let root = repo_root();
        if !root.join("scripts").join("run_verify.mjs").is_file() {
            return;
        }
        let suites = discover_suites(&root).expect("run_verify.mjs SUITES must parse");
        assert!(suites.len() > 50, "suspiciously few suites: {}", suites.len());
        assert!(suites.iter().all(|s| !s.is_empty()));
        // The discovered half is localeCompare-sorted and comes first; the
        // EXTRA_SUITES half is appended verbatim.
        assert!(suites.iter().any(|s| s[0] == "scripts/release_manifest.mjs"));
        assert!(suites.iter().any(|s| s[0].starts_with("scripts/tests/test_")));
    }

    // ── THE PIN: the committed registry ───────────────────────────────────

    /// Rebuild the registry from the real tree and byte-compare it against
    /// the committed scripts/impact-registry.json. This is `--write` and
    /// `--check` proven together.
    #[test]
    fn rebuild_reproduces_the_committed_registry_byte_for_byte() {
        let root = repo_root();
        if !registry_path(&root).is_file() {
            return;
        }
        let registry = build_registry(&root).expect("registry builds");
        let rebuilt = serialize_registry(&registry);
        let committed = std::fs::read_to_string(registry_path(&root)).unwrap();
        if rebuilt != committed {
            // Name the drift instead of dumping two 100 kB blobs.
            let a: Value = serde_json::from_str(&rebuilt).unwrap();
            let b: Value = serde_json::from_str(&committed).unwrap();
            let (af, bf) = (a["files"].as_object().unwrap(), b["files"].as_object().unwrap());
            let only_rebuilt: Vec<&String> = af.keys().filter(|k| !bf.contains_key(*k)).collect();
            let only_committed: Vec<&String> = bf.keys().filter(|k| !af.contains_key(*k)).collect();
            let changed: Vec<&String> = af
                .keys()
                .filter(|k| bf.contains_key(*k) && af[*k] != bf[*k])
                .collect();
            panic!(
                "registry drift — only-rebuilt: {only_rebuilt:?}\nonly-committed: {only_committed:?}\nchanged: {changed:?}"
            );
        }
    }

    // ── query ─────────────────────────────────────────────────────────────

    #[test]
    fn query_levels_and_unmapped_semantics() {
        let root = repo_root();
        let ctx = Ctx::new(&root);
        let registry: Value = serde_json::from_str(
            r#"{"version":2,"files":{
              "a.mjs":{"direct":["s1","s2"],"all":["s1","s2","s3"]},
              "t.mjs":{"direct":[],"all":["s9"]}
            }}"#,
        )
        .unwrap();
        let abs_a = ctx.to_abs("a.mjs");
        let abs_t = ctx.to_abs("t.mjs");
        let abs_x = ctx.to_abs("unknown.mjs");

        let (mapped, unmapped) = query_registry(&ctx, &registry, &[&abs_a], None);
        assert_eq!(mapped, ["s1", "s2", "s3"]);
        assert!(unmapped.is_empty());

        let (mapped, unmapped) = query_registry(&ctx, &registry, &[&abs_a], Some(1));
        assert_eq!(mapped, ["s1", "s2"]);
        assert!(unmapped.is_empty());

        // transitive-only: zero suites at level 1, but NOT unmapped
        let (mapped, unmapped) = query_registry(&ctx, &registry, &[&abs_t], Some(1));
        assert!(mapped.is_empty());
        assert!(unmapped.is_empty(), "a transitively-reachable file is never UNMAPPED");

        let (mapped, unmapped) = query_registry(&ctx, &registry, &[&abs_x], None);
        assert!(mapped.is_empty());
        assert_eq!(unmapped, ["unknown.mjs"]);

        // the union across several inputs is deduped and sorted
        let (mapped, _) = query_registry(&ctx, &registry, &[&abs_a, &abs_t], None);
        assert_eq!(mapped, ["s1", "s2", "s3", "s9"]);
    }

    #[test]
    fn serialize_shape_matches_node() {
        let v: Value = serde_json::from_str(r#"{"version":2,"files":{"a":{"direct":[],"all":["x"]}}}"#)
            .unwrap();
        assert_eq!(
            serialize_registry(&v),
            "{\n  \"version\": 2,\n  \"files\": {\n    \"a\": {\n      \"direct\": [],\n      \"all\": [\n        \"x\"\n      ]\n    }\n  }\n}\n"
        );
    }
}
