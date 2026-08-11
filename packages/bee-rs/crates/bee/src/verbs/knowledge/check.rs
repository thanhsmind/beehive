// checkBundle and the subject skeleton
//
// Split out of the single 4.4k-line verbs/knowledge.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_any as resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{js_trim, keys_known, parse_flags, FlagV, Flags};
use serde_json::{json, Map, Number, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── G14 layer 1: the subject SKELETON (knowledge.mjs foldEncoding /
//     normalizeSubject) ────────────────────────────────────────────────────
//
// Encoding must never be able to buy a fork: a trailing period and a Cyrillic
// 'е' homoglyph both used to buy a NEW concept sitting beside the owner, so
// `bee.authoritative_for` identity is a skeleton, not a string:
//
//   NFKC            -> fullwidth, ligature and math-alphanumeric forms
//   lowercase       -> case is not identity
//   NFD + strip \p{M} -> diacritics are not identity
//   confusable fold -> cross-script look-alikes (NFKC does NOT do this: a
//                      Cyrillic 'е' U+0435 and a Latin 'e' stay distinct
//                      codepoints forever, which is exactly the defeat)
//   non-letter/digit runs -> a single space, ends trimmed
//
// This replaces the ASCII-only subset the port shipped with (a non-ASCII
// claim used to DELEGATE the whole `knowledge check` rather than guess the
// fold), so the anti-fork gate now answers natively for every claim.

/// knowledge.mjs CONFUSABLE_FOLD — the UTS #39 skeleton fold, bounded to the
/// look-alikes that collide with ASCII. Transcribed key-for-key from the .mjs
/// map (Cyrillic then Greek); order is irrelevant, membership is not.
pub(crate) const CONFUSABLE_FOLD: [(char, char); 42] = [
    // Cyrillic -> Latin
    ('а', 'a'), ('в', 'b'), ('е', 'e'), ('ё', 'e'), ('з', '3'), ('к', 'k'),
    ('м', 'm'), ('н', 'h'), ('о', 'o'), ('р', 'p'), ('с', 'c'), ('т', 't'),
    ('у', 'y'), ('х', 'x'), ('ѕ', 's'), ('і', 'i'), ('ї', 'i'), ('ј', 'j'),
    ('ԁ', 'd'), ('ԛ', 'q'), ('ԝ', 'w'), ('ѵ', 'v'), ('ӏ', 'l'), ('ѡ', 'w'),
    ('ғ', 'f'),
    // Greek -> Latin
    ('α', 'a'), ('β', 'b'), ('γ', 'y'), ('ε', 'e'), ('ζ', 'z'), ('η', 'n'),
    ('ι', 'i'), ('κ', 'k'), ('ν', 'v'), ('ο', 'o'), ('ρ', 'p'), ('τ', 't'),
    ('υ', 'u'), ('χ', 'x'), ('ϲ', 'c'), ('ϳ', 'j'), ('ϱ', 'p'),
];

pub(crate) fn confusable_fold(c: char) -> char {
    match CONFUSABLE_FOLD.iter().find(|(from, _)| *from == c) {
        Some((_, to)) => *to,
        None => c,
    }
}

/// knowledge.mjs foldEncoding — `NFKC -> toLowerCase -> NFD -> strip \p{M} ->
/// confusable map`. Keeps punctuation (normalizeSubject strips it).
pub(crate) fn fold_encoding(text: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    let bare: String = text
        .nfkc()
        .collect::<String>()
        // JS String.prototype.toLowerCase is the full Unicode Default Case
        // Conversion (locale-independent), which is exactly str::to_lowercase.
        .to_lowercase()
        .nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .collect();
    bare.chars().map(confusable_fold).collect()
}

/// knowledge.mjs normalizeSubject — the skeleton used for `authoritative_for`
/// ownership. `''` for a subject carrying no letters or digits at all (null,
/// '', '   ', '...'), which is the signal layer 2 refuses on.
///
/// `\p{L}|\p{N}` is spelled here as `is_alphabetic() || is_numeric()`. The two
/// sets differ only by Other_Alphabetic, which is made up of marks and of
/// enclosed/squared letter forms — the marks are already gone (NFD + \p{M}
/// strip above) and the enclosed forms are already folded to plain letters by
/// NFKC, so on the input this function actually sees the two spellings agree.
pub(crate) fn normalize_subject(subject: &str) -> String {
    let folded = fold_encoding(subject);
    let mut out = String::new();
    let mut in_run = false;
    for c in folded.chars() {
        if c.is_alphabetic() || c.is_numeric() {
            out.push(c);
            in_run = false;
        } else if !in_run {
            out.push(' ');
            in_run = true;
        }
    }
    // `.trim()` after the replacement: only ASCII spaces can remain at the ends.
    out.trim_matches(' ').to_string()
}

pub(crate) fn typeof_word(v: &Value) -> &'static str {
    match v {
        Value::Null => "object",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) | Value::Object(_) => "object",
    }
}

// ─── U2: dangling `bee.sources` paths (pointers rot when files move) ──────
//
// A sources entry is prose citation as often as it is a code pointer
// ("src/cli", "docs/specs/widgets.md#B5 (see the anchor table)", "See the
// gizmos spec"), so only the LEADING whitespace-delimited token of each
// entry is even a path candidate — everything after it is free text D4/U2
// never parses. Within that token: a URL or an ssh-style remote reads as a
// scheme (a ':' before the first '/' — `https://`, `git@host:org/repo`) and
// is never flagged; a rooted form (leading '/') is not repo-relative and is
// never flagged; a token carrying no '/' at all is prose. A trailing
// `#fragment` (a doc-anchor citation) is dropped before the existence check
// — the file is the pointer, the heading inside it is not.
pub(crate) fn dangling_source_candidate(entry: &str) -> Option<String> {
    let token = entry.split_whitespace().next()?;
    let token = token.split('#').next().unwrap_or("");
    if token.is_empty() || token.starts_with('/') {
        return None;
    }
    let slash = token.find('/')?;
    if let Some(colon) = token.find(':') {
        if colon < slash {
            return None; // scheme-shaped prefix — a URL or an ssh remote, never a repo path
        }
    }
    Some(token.to_string())
}

// ─── evidence-ladder: optional `bee.evidence` state (el-1) ─────────────────
//
// A pattern concept may carry `bee.evidence: present|wired|exercised` — a
// self-reported rung on the ladder from "written down" (present, the
// default) through "cited by a live enforcer" (wired) to "a passing test
// exercises it" (exercised). Optional, and NEVER guessed: absent reads as
// `present` and stays valid; anything else (a typo, a stray boolean, a list)
// is a profile WARNING naming the file and the value, never a bundle error —
// the same never-block-on-a-SHOULD posture every other profile warning
// takes (B2 above).

pub(crate) const EVIDENCE_STATES: [&str; 3] = ["present", "wired", "exercised"];

// ─── body link scanning (dangling_md_link / dangling_wiki_link) ───────────
//
// A concept body is free prose, not a spec grammar — these two extractors
// are lightweight bracket scans (matching dangling_source_candidate's
// token-first-then-filter style above), never a full CommonMark engine.

/// Every `[text](target)` link destination in a body, in appearance order.
/// A `<target>` angle-bracket wrapper is unwrapped; a space-separated
/// `"title"` suffix is dropped — only the destination itself is a candidate.
pub(crate) fn markdown_link_targets(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cursor = 0usize;
    while let Some(rel_start) = body[cursor..].find('[') {
        let bracket = cursor + rel_start;
        let after_bracket = bracket + 1;
        let Some(rel_close) = body[after_bracket..].find(']') else { break };
        let close = after_bracket + rel_close;
        let paren_start = close + 1;
        if body.as_bytes().get(paren_start) == Some(&b'(') {
            let after_paren = paren_start + 1;
            if let Some(rel_end) = body[after_paren..].find(')') {
                let end = after_paren + rel_end;
                let inner = body[after_paren..end].trim();
                let inner = inner.strip_prefix('<').and_then(|s| s.strip_suffix('>')).unwrap_or(inner);
                let target = inner.split_whitespace().next().unwrap_or("");
                if !target.is_empty() {
                    out.push(target.to_string());
                }
                cursor = end + 1;
                continue;
            }
        }
        cursor = after_bracket;
    }
    out
}

/// A link target is a dangling_md_link CANDIDATE only when it is relative
/// and names a `.md` file: http(s)/mailto schemes, absolute (`/…`) paths and
/// anchor-only (`#…`) targets are never a repo path and are never flagged. A
/// trailing `#fragment` is dropped before the `.md` check, matching
/// dangling_source's fragment handling.
pub(crate) fn md_link_candidate(target: &str) -> Option<String> {
    let target = target.trim();
    if target.is_empty()
        || target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("mailto:")
        || target.starts_with('/')
        || target.starts_with('#')
    {
        return None;
    }
    let path = target.split('#').next().unwrap_or("");
    if path.is_empty() || !path.ends_with(".md") {
        return None;
    }
    Some(path.to_string())
}

/// Every `[[target]]` wiki link target in a body, in appearance order. A
/// `[[target|display]]` or `[[target#anchor]]` form only offers the leading
/// token as the target candidate.
pub(crate) fn wiki_link_targets(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cursor = 0usize;
    while let Some(rel_start) = body[cursor..].find("[[") {
        let start = cursor + rel_start + 2;
        let Some(rel_end) = body[start..].find("]]") else { break };
        let end = start + rel_end;
        let target = body[start..end].trim().split(['|', '#']).next().unwrap_or("").trim();
        if !target.is_empty() {
            out.push(target.to_string());
        }
        cursor = end + 2;
    }
    out
}

/// Code regions are quotation, not linkage: a fenced block (``` / ~~~) or an
/// inline backtick span quoting `[[syntax]]` or `](file.md)` must never feed
/// the link extractors. Fenced lines are dropped whole; inline spans are
/// blanked to spaces so byte offsets elsewhere stay honest.
pub(crate) fn strip_code_regions(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut in_fence = false;
    for line in body.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            out.push('\n');
            continue;
        }
        if in_fence {
            out.push('\n');
            continue;
        }
        let mut in_span = false;
        for c in line.chars() {
            if c == '`' {
                in_span = !in_span;
                out.push(' ');
            } else if in_span {
                out.push(' ');
            } else {
                out.push(c);
            }
        }
        out.push('\n');
    }
    out
}

/// A wiki link resolves when `target`, or `target` minus an optional
/// `pattern-` prefix, matches the file stem of any `.md` in the bundle.
pub(crate) fn wiki_link_resolves(target: &str, stems: &[&str]) -> bool {
    if stems.contains(&target) {
        return true;
    }
    match target.strip_prefix("pattern-") {
        Some(stripped) => stems.contains(&stripped),
        None => false,
    }
}

pub(crate) fn check_bundle(dir: &Path, strict: bool) -> Option<CheckReport> {
    let mut errors: Vec<Value> = Vec::new();
    let mut warnings: Vec<Value> = Vec::new();
    let mut profile_errors: Vec<Value> = Vec::new();
    // bundle_dir always builds `dir` as `root/docs/knowledge` — the two
    // parents recover the repo root that `sources` paths are relative to
    // (unlike `required_context`, which is bundle-relative, D19).
    let repo_root = dir.parent().and_then(Path::parent);
    let files = list_bundle_markdown(dir)?;
    // Wiki-link resolution target set: every `.md` file's bare stem, reserved
    // files (index.md/log.md) included — "any .md in the bundle" (not just
    // concepts).
    let md_stems: Vec<&str> = files
        .iter()
        .map(|f| {
            let base = f.rsplit('/').next().unwrap_or(f);
            base.strip_suffix(".md").unwrap_or(base)
        })
        .collect();
    let mut parsed_concepts: Vec<Concept> = Vec::new();
    let mut concept_count = 0usize;

    for rel in &files {
        let base = rel.rsplit('/').next().unwrap_or(rel);
        let text = match read_file_lossy(&join_rel(dir, rel)) {
            Ok(t) => t,
            // checkBundle's own catch: push an `unreadable` finding and keep
            // walking. Node interpolated the libuv message; this carries the
            // Rust io message in the same sentence. CUTOVER: this arm used to
            // delegate purely because of those bytes.
            Err(e) => {
                errors.push(finding(rel, "unreadable", format!("could not read file: {e}")));
                continue;
            }
        };
        if is_reserved_basename(base) {
            if base == "index.md" {
                check_index_file(rel, &text, &mut errors)?;
            } else {
                check_log_file(rel, &text, &mut errors);
            }
            continue;
        }

        concept_count += 1;
        let parsed = parse_frontmatter(&text);
        let (data, block, body) = match parsed {
            Fm::Absent => {
                errors.push(finding(
                    rel,
                    "missing_frontmatter",
                    "a non-reserved .md inside the bundle is a concept and must carry frontmatter (D23; OKF §4)"
                        .to_string(),
                ));
                continue;
            }
            Fm::Failed { code, message, line } => {
                errors.push(finding(
                    rel,
                    "unparseable_frontmatter",
                    format!("frontmatter is unparseable — {code}: {message} (line {line})"),
                ));
                continue;
            }
            Fm::Parsed { data, block, body } => (data, block, body),
        };

        match data.get("type") {
            Some(Value::String(s)) if !js_trim(s).is_empty() => {
                if !CONCEPT_TYPES.contains(&s.as_str()) {
                    warnings.push(finding(
                        rel,
                        "unknown_type",
                        format!(
                            "type \"{s}\" is outside the profile's nine types (D18); OKF consumers tolerate it, bee flags it"
                        ),
                    ));
                }
            }
            _ => {
                errors.push(finding(
                    rel,
                    "empty_type",
                    "type is required and must be a non-empty string (OKF §4.1 MUST)".to_string(),
                ));
            }
        }

        for key_path in PROFILE_REQUIRED {
            // readPath: walk objects; anything non-object mid-walk => undefined.
            let mut value: Option<&Value> = None;
            let mut cursor: Option<&Map<String, Value>> = Some(&data);
            for (i, key) in key_path.iter().enumerate() {
                let Some(map) = cursor else {
                    value = None;
                    break;
                };
                let v = map.get(*key);
                if i + 1 == key_path.len() {
                    value = v;
                } else {
                    cursor = match v {
                        Some(Value::Object(m)) => Some(m),
                        _ => None, // arrays have no such string key; primitives stop the walk
                    };
                }
            }
            let present = matches!(value, Some(Value::String(s)) if !js_trim(s).is_empty());
            if !present {
                warnings.push(finding(
                    rel,
                    "missing_profile_field",
                    format!(
                        "profile-required field \"{}\" is missing or empty (D10: never invented — author it)",
                        key_path.join(".")
                    ),
                ));
            }
        }

        let re_emitted = emit_frontmatter(&data).ok();
        if re_emitted.as_deref() != Some(block.as_str()) {
            warnings.push(finding(
                rel,
                "not_canonical",
                "frontmatter parse→re-emit differs byte-wise from the file (hand-edited colon/#/CRLF/key-order outside the canonical emitted form) — normalize by re-emitting"
                    .to_string(),
            ));
        }

        // Optional bee.evidence: an absent field stays valid (reads as
        // `present`, D-el-1); anything present that is not one of the three
        // enum states warns by file and value, never errors the bundle.
        if let Some(value) = bee_of(&data).get("evidence") {
            let valid = matches!(value, Value::String(s) if EVIDENCE_STATES.contains(&s.as_str()));
            if !valid {
                warnings.push(finding(
                    rel,
                    "invalid_evidence_state",
                    format!(
                        "bee.evidence \"{}\" is not one of present|wired|exercised (absent reads as present)",
                        jsjson::js_to_string(value)
                    ),
                ));
            }
        }

        // ── D4-style body link checks (dangling_md_link / dangling_wiki_link) ──
        // Quoted syntax is not linkage: code fences and inline spans are
        // stripped before extraction (strip_code_regions).
        let scannable = strip_code_regions(&body);
        for target in markdown_link_targets(&scannable) {
            let Some(candidate) = md_link_candidate(&target) else { continue };
            // Resolved against the CONTAINING FILE's directory, not the
            // bundle root — a body link is authored relative to where it
            // lives, exactly like a filesystem link would be.
            let file_dir = dir_of(rel);
            let combined =
                if file_dir.is_empty() { candidate.clone() } else { format!("{file_dir}/{candidate}") };
            let resolved = match resolve_inside_bundle(dir, &combined) {
                Ok(r) => r,
                Err(()) => return None,
            };
            let exists = resolved.map(|p| p.exists()).unwrap_or(false);
            if !exists {
                warnings.push(finding(
                    rel,
                    "dangling_md_link",
                    format!(
                        "body link target \"{target}\" does not resolve to an existing file inside the bundle"
                    ),
                ));
            }
        }
        for target in wiki_link_targets(&scannable) {
            if !wiki_link_resolves(&target, &md_stems) {
                warnings.push(finding(
                    rel,
                    "dangling_wiki_link",
                    format!(
                        "wiki link target \"[[{target}]]\" matches no concept's file stem in the bundle"
                    ),
                ));
            }
        }

        parsed_concepts.push(Concept { path: rel.clone(), data });
    }

    // ── bundle-level profile checks (D31 uniqueness, D4 dangling targets) ──
    let mut by_id: Vec<(String, Vec<String>)> = Vec::new();
    let mut by_authority: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for concept in &parsed_concepts {
        let bee = bee_of(&concept.data);
        if let Some(id) = str_field(&bee, "id") {
            match by_id.iter_mut().find(|(k, _)| k == id) {
                Some((_, holders)) => holders.push(concept.path.clone()),
                None => by_id.push((id.to_string(), vec![concept.path.clone()])),
            }
        }
        if let Some(claim_v) = bee.get("authoritative_for") {
            let claim_ok = matches!(claim_v, Value::String(s) if !js_trim(s).is_empty());
            if !claim_ok {
                let got = match claim_v {
                    Value::Null => "null",
                    Value::Array(_) => "array",
                    other => typeof_word(other),
                };
                profile_errors.push(finding(
                    &concept.path,
                    "malformed_authoritative_for",
                    format!(
                        "bee.authoritative_for must be one non-empty string (got {got}) — a claim bee cannot read is an owner the anti-fork gate cannot see (D31)"
                    ),
                ));
            } else if let Value::String(claim) = claim_v {
                // The HARDENED skeleton, not the raw string: two claims that
                // differ only by punctuation, case or ENCODING are one
                // subject with two authorities.
                let key = normalize_subject(claim);
                match by_authority.iter_mut().find(|(k, _)| *k == key) {
                    Some((_, holders)) => holders.push((concept.path.clone(), claim.clone())),
                    None => by_authority.push((key, vec![(concept.path.clone(), claim.clone())])),
                }
            }
        }
    }
    for (id, holders) in &by_id {
        if holders.len() > 1 {
            warnings.push(finding(
                &holders[0],
                "duplicate_id",
                format!(
                    "bee.id \"{id}\" is claimed by {} concepts ({}) — ids are globally unique (D31)",
                    holders.len(),
                    holders.join(", ")
                ),
            ));
        }
    }
    for (_, holders) in &by_authority {
        if holders.len() > 1 {
            let mut subjects: Vec<String> = Vec::new();
            for (_, claim) in holders {
                if !subjects.contains(claim) {
                    subjects.push(claim.clone());
                }
            }
            let quoted: Vec<String> = subjects.iter().map(|s| format!("\"{s}\"")).collect();
            profile_errors.push(finding(
                &holders[0].0,
                "duplicate_authoritative_for",
                format!(
                    "bee.authoritative_for {} {} claimed by {} concepts ({}) — one subject, one authority (D31). Two authorities on one subject both parse and both index, and no reader can tell which is true.",
                    quoted.join(" / "),
                    if quoted.len() > 1 { "name one subject and are" } else { "is" },
                    holders.len(),
                    holders.iter().map(|(f, _)| f.as_str()).collect::<Vec<_>>().join(", ")
                ),
            ));
        }
    }
    for concept in &parsed_concepts {
        let bee = bee_of(&concept.data);
        if let Some(Value::Array(targets)) = bee.get("required_context") {
            for target in targets {
                let resolved = match target {
                    Value::String(s) => match resolve_inside_bundle(dir, s) {
                        Ok(r) => r,
                        Err(()) => return None,
                    },
                    _ => None,
                };
                let exists = resolved.map(|p| p.exists()).unwrap_or(false);
                if !exists {
                    warnings.push(finding(
                        &concept.path,
                        "dangling_required_context",
                        format!(
                            "required_context target \"{}\" does not resolve inside the bundle (D19: bundle-relative paths)",
                            jsjson::js_to_string(target)
                        ),
                    ));
                }
            }
        }
        if let Some(sup) = str_field(&bee, "supersedes") {
            if !by_id.iter().any(|(k, _)| k == sup) {
                warnings.push(finding(
                    &concept.path,
                    "dangling_supersedes",
                    format!("supersedes target id \"{sup}\" matches no concept's bee.id in the bundle"),
                ));
            }
        }
        if let (Some(root), Some(Value::Array(sources))) = (repo_root, bee.get("sources")) {
            for source in sources {
                let Value::String(s) = source else { continue };
                let Some(candidate) = dangling_source_candidate(s) else { continue };
                if !root.join(&candidate).exists() {
                    warnings.push(finding(
                        &concept.path,
                        "dangling_source",
                        format!(
                            "sources entry \"{s}\" names path \"{candidate}\" that does not exist on disk (U2: pointers rot when files move)"
                        ),
                    ));
                }
            }
        }
    }

    let ok = errors.is_empty() && profile_errors.is_empty() && (!strict || warnings.is_empty());
    Some(CheckReport {
        okf_errors: errors,
        profile_errors,
        warnings,
        files: files.len(),
        concepts: concept_count,
        ok,
    })
}
