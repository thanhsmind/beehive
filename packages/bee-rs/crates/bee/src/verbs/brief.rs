use crate::state::read_config_raw;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

const VI_LABELS: [&str; 6] = [
    "**Là gì:**",
    "**Vì sao:**",
    "**Nguồn gốc:**",
    "**Chi phí và rủi ro:**",
    "**Anh cần làm:**",
    "**Chuẩn xong:**",
];
const EN_LABELS: [&str; 6] = [
    "**What:**",
    "**Why:**",
    "**Origin:**",
    "**Cost and risk:**",
    "**Your call:**",
    "**Done when:**",
];
const AUDIENCES: [&str; 2] = ["owner", "internal"];
const SIZES: [&str; 3] = ["S", "M", "L"];
const ASKS: [&str; 5] = ["build", "park", "drop", "answer", "test"];
const FILE_EXTENSIONS: [&str; 11] =
    [".rs", ".md", ".json", ".js", ".mjs", ".ts", ".py", ".sh", ".toml", ".yml", ".yaml"];
const MAX_TITLE: usize = 80;
const MIN_WORDS: usize = 120;
const MAX_WORDS: usize = 220;

pub(crate) fn brief_rel(id: &str) -> String {
    format!("docs/briefs/{id}.md")
}

pub(crate) fn brief_language(root: &Path) -> (String, Option<String>) {
    let config = read_config_raw(root);
    match config.get("brief").and_then(|b| b.get("language")) {
        None | Some(Value::Null) => ("vi".into(), None),
        Some(Value::String(s)) if s == "vi" || s == "en" => (s.clone(), None),
        Some(Value::String(s)) => ("vi".into(), Some(s.clone())),
        Some(other) => ("vi".into(), Some(other.to_string())),
    }
}

fn split_frontmatter(text: &str) -> Result<(Vec<(String, String)>, Vec<&str>, Vec<String>), String> {
    let lines: Vec<&str> = text.split('\n').collect();
    if lines.first() != Some(&"---") {
        return Err("the brief does not start with a --- frontmatter line".into());
    }
    let Some(close) = lines.iter().skip(1).position(|l| *l == "---").map(|p| p + 1) else {
        return Err("the frontmatter has no closing --- line".into());
    };
    let mut pairs = Vec::new();
    let mut defects = Vec::new();
    for line in &lines[1..close] {
        if line.trim().is_empty() {
            continue;
        }
        if line.starts_with(char::is_whitespace) {
            defects.push(format!("the frontmatter line \"{line}\" is indented; nested keys are not allowed"));
            continue;
        }
        match line.split_once(':') {
            Some((key, value)) if !key.trim().is_empty() => {
                pairs.push((key.trim().to_string(), value.trim().to_string()))
            }
            _ => defects.push(format!("the frontmatter line \"{line}\" is not a key: value pair")),
        }
    }
    Ok((pairs, lines[close + 1..].to_vec(), defects))
}

fn frontmatter_title(text: &str) -> Option<String> {
    let text = text.replace("\r\n", "\n");
    let (pairs, _, _) = split_frontmatter(&text).ok()?;
    pairs.into_iter().find(|(k, _)| k == "title").map(|(_, v)| v).filter(|v| !v.is_empty())
}

fn check_enum(pairs: &[(String, String)], key: &str, allowed: &[&str], defects: &mut Vec<String>) {
    match pairs.iter().find(|(k, _)| k == key) {
        None => defects.push(format!("the frontmatter has no {key} key")),
        Some((_, v)) if !allowed.contains(&v.as_str()) => {
            defects.push(format!("{key} is \"{v}\"; it must be one of {}", allowed.join("|")))
        }
        _ => {}
    }
}

fn forbidden_tokens(body: &str, defects: &mut Vec<String>) {
    for token in body.split_whitespace() {
        if token.contains('`') {
            defects.push(format!("the body holds a backtick (code): {token}"));
        }
        let bytes = token.as_bytes();
        if bytes.iter().enumerate().any(|(i, b)| {
            *b == b'(' && i > 0 && (bytes[i - 1].is_ascii_alphabetic() || bytes[i - 1] == b'_')
        }) {
            defects.push(format!("the body holds a function call shape: {token}"));
        }
        let stripped = token.trim_matches(|c: char| "()[]{},.;:!?\"'".contains(c));
        if has_pbi_id(stripped) {
            defects.push(format!("the body names a backlog id: {stripped}"));
        }
        if stripped.contains('/') {
            defects.push(format!("the body holds a file path: {stripped}"));
        }
        let lower = stripped.to_ascii_lowercase();
        if FILE_EXTENSIONS.iter().any(|ext| lower.ends_with(ext)) {
            defects.push(format!("the body names a file: {stripped}"));
        }
    }
}

fn has_pbi_id(token: &str) -> bool {
    let b = token.as_bytes();
    (0..b.len()).any(|i| {
        b[i..].starts_with(b"p-")
            && (i == 0 || !b[i - 1].is_ascii_alphanumeric())
            && b.len() >= i + 10
            && b[i + 2..i + 10].iter().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(c))
    })
}

pub(crate) fn validate_brief(text: &str, language: &str) -> Vec<String> {
    let text = text.replace("\r\n", "\n");
    let (pairs, body, mut defects) = match split_frontmatter(&text) {
        Ok(parts) => parts,
        Err(defect) => return vec![defect],
    };
    match pairs.iter().find(|(k, _)| k == "title") {
        None => defects.push("the frontmatter has no title key".into()),
        Some((_, t)) if t.is_empty() => defects.push("title is empty".into()),
        Some((_, t)) if t.chars().count() > MAX_TITLE => defects.push(format!(
            "title is {} characters, over the {MAX_TITLE}-character limit",
            t.chars().count()
        )),
        _ => {}
    }
    check_enum(&pairs, "audience", &AUDIENCES, &mut defects);
    check_enum(&pairs, "size", &SIZES, &mut defects);
    check_enum(&pairs, "ask", &ASKS, &mut defects);

    let labels = if language == "en" { EN_LABELS } else { VI_LABELS };
    let mut found: Vec<(usize, usize)> = Vec::new();
    for (n, line) in body.iter().enumerate() {
        if !line.starts_with("**") {
            continue;
        }
        match labels.iter().position(|l| line.starts_with(l)) {
            Some(k) => found.push((n, k)),
            None => defects.push(format!("the line \"{line}\" starts with ** but is not one of the six labels")),
        }
    }
    let mut labels_sound = true;
    for (k, label) in labels.iter().enumerate() {
        match found.iter().filter(|(_, f)| *f == k).count() {
            0 => {
                defects.push(format!("the label {label} is missing"));
                labels_sound = false;
            }
            1 => {}
            c => {
                defects.push(format!("the label {label} starts {c} lines; it must start exactly one"));
                labels_sound = false;
            }
        }
    }
    if labels_sound && found.windows(2).any(|w| w[0].1 > w[1].1) {
        defects.push(format!("the labels are out of order; the order is {}", labels.join(" ")));
        labels_sound = false;
    }
    match body.iter().find(|l| !l.trim().is_empty()) {
        Some(first) if first.starts_with(labels[0]) => {}
        _ => defects.push(format!("the body does not open with the label {}", labels[0])),
    }
    if labels_sound {
        for (i, (line, k)) in found.iter().enumerate() {
            let end = found.get(i + 1).map_or(body.len(), |(n, _)| *n);
            let rest = &body[line + 1..end];
            let head = &body[*line][labels[*k].len()..];
            if head.trim().is_empty() && rest.iter().all(|l| l.trim().is_empty()) {
                defects.push(format!("the {} section is empty", labels[*k]));
            }
            if *k == labels.len() - 1 {
                let bullets = rest.iter().filter(|l| l.starts_with("- ")).count();
                if !(2..=4).contains(&bullets) {
                    defects.push(format!(
                        "the {} section holds {bullets} bullet line(s); it needs 2 to 4",
                        labels[*k]
                    ));
                }
            }
        }
    }

    let body_text = body.join("\n");
    let words =
        body_text.split_whitespace().filter(|t| t.chars().any(char::is_alphanumeric)).count();
    if !(MIN_WORDS..=MAX_WORDS).contains(&words) {
        defects.push(format!("the body has {words} words; it needs {MIN_WORDS} to {MAX_WORDS}"));
    }
    forbidden_tokens(&body_text, &mut defects);

    let mut seen = std::collections::HashSet::new();
    defects.retain(|d| seen.insert(d.clone()));
    defects
}

pub(crate) enum BriefStatus {
    Ok,
    Missing,
    Malformed(Vec<String>),
}

impl BriefStatus {
    pub(crate) fn word(&self) -> &'static str {
        match self {
            BriefStatus::Ok => "ok",
            BriefStatus::Missing => "missing",
            BriefStatus::Malformed(_) => "malformed",
        }
    }

    pub(crate) fn mark(&self) -> &'static str {
        match self {
            BriefStatus::Ok => "ok",
            BriefStatus::Missing => "MISSING",
            BriefStatus::Malformed(_) => "MALFORMED",
        }
    }

    pub(crate) fn defects(&self) -> &[String] {
        match self {
            BriefStatus::Malformed(d) => d,
            _ => &[],
        }
    }
}

pub(crate) fn brief_status(root: &Path, id: &str) -> BriefStatus {
    let Ok(bytes) = std::fs::read(root.join(brief_rel(id))) else {
        return BriefStatus::Missing;
    };
    let Ok(text) = String::from_utf8(bytes) else {
        return BriefStatus::Malformed(vec!["the brief is not UTF-8 text".into()]);
    };
    let defects = validate_brief(&text, &brief_language(root).0);
    if defects.is_empty() {
        BriefStatus::Ok
    } else {
        BriefStatus::Malformed(defects)
    }
}

fn read_record(path: &Path) -> Option<(String, String)> {
    let value: Value = serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    let feature = value.get("feature")?.as_str()?.to_string();
    let phase = value.get("phase").and_then(Value::as_str).unwrap_or("").to_string();
    (!feature.is_empty()).then_some((feature, phase))
}

fn active_features(root: &Path) -> BTreeMap<String, String> {
    let bee = root.join(".bee");
    let mut records: Vec<(String, String)> = read_record(&bee.join("state.json")).into_iter().collect();
    if let Ok(entries) = std::fs::read_dir(bee.join("lanes")) {
        let mut paths: Vec<_> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .collect();
        paths.sort();
        records.extend(paths.iter().filter_map(|p| read_record(p)));
    }
    let mut active = BTreeMap::new();
    for (feature, phase) in records {
        if phase != "idle" && phase != "compounding-complete" {
            active.entry(feature).or_insert(phase);
        }
    }
    active
}

fn brief_value(id: &str, status: &BriefStatus) -> Value {
    json!({"status": status.word(), "path": brief_rel(id), "defects": status.defects()})
}

pub(crate) fn owner_view(root: &Path) -> Value {
    let (language, unknown) = brief_language(root);
    let features = active_features(root);
    let pbis = super::backlog::fold_pbi_records(root).unwrap_or_default();
    let field = |p: &Value, k: &str| p.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let folds_into = |p: &Value| {
        field(p, "status") == "in-flight" && features.contains_key(&field(p, "feature"))
    };
    let mut items = Vec::new();
    let mut problems = 0usize;
    for (slug, phase) in &features {
        let status = brief_status(root, slug);
        let title = match status {
            BriefStatus::Ok => std::fs::read_to_string(root.join(brief_rel(slug)))
                .ok()
                .and_then(|t| frontmatter_title(&t))
                .unwrap_or_else(|| slug.clone()),
            _ => slug.clone(),
        };
        let folded: Vec<Value> = pbis
            .iter()
            .filter(|p| folds_into(p) && field(p, "feature") == *slug)
            .map(|p| json!({"id": field(p, "id"), "title": field(p, "title")}))
            .collect();
        problems += usize::from(!matches!(status, BriefStatus::Ok));
        items.push(json!({
            "kind": "feature",
            "id": slug,
            "title": title,
            "status": phase,
            "brief": brief_value(slug, &status),
            "folded": folded,
        }));
    }
    for p in &pbis {
        let status_word = field(p, "status");
        if !(status_word == "proposed" || status_word == "in-flight") || folds_into(p) {
            continue;
        }
        let id = field(p, "id");
        let status = brief_status(root, &id);
        problems += usize::from(!matches!(status, BriefStatus::Ok));
        items.push(json!({
            "kind": "pbi",
            "id": id,
            "title": field(p, "title"),
            "status": status_word,
            "brief": brief_value(&id, &status),
        }));
    }
    json!({"language": language, "unknown_language": unknown, "items": items, "problems": problems})
}

#[cfg(test)]
mod tests {
    use super::*;

    const VI_DRAFT: &str = r#"---
title: Mỗi việc và mỗi tính năng có bản tóm tắt đủ để anh duyệt
audience: owner
size: M
ask: build
---

**Là gì:** Hiện nay một việc chỉ có tên và một câu ngắn. Sau thay đổi này, mỗi việc mới và mỗi tính năng mới có một bản tóm tắt sáu đoạn bằng tiếng Việt, viết ngay lúc tạo. Công cụ báo khi bản tóm tắt thiếu hoặc sai mẫu.

**Vì sao:** Anh không hiểu một câu quá ngắn, nên không thể duyệt hay chọn hướng. Lý do và cái giá của việc nằm trong tài liệu nội bộ.

**Nguồn gốc:** Yêu cầu từ waggledance ngày 28 tháng 10.

**Chi phí và rủi ro:** Cỡ vừa. Rủi ro: người viết bỏ qua bước tóm tắt, nên công cụ báo việc thiếu. Đổi mẫu có thể làm hỏng bảng của agent-works, nên mẫu giữ nguyên. Phụ thuộc mẫu đã thống nhất với waggledance.

**Anh cần làm:** Chọn duyệt để xây, hoãn, hoặc bỏ. Tôi đề nghị duyệt, vì các dự án khác chờ bản này để bổ sung tóm tắt.

**Chuẩn xong:**
- Việc mới có tóm tắt sai mẫu bị từ chối.
- Việc mới không có tóm tắt vẫn vào danh sách nhưng bị gắn cờ.
- Bước kiểm tra liệt kê mọi việc đang mở còn thiếu tóm tắt.
- Việc đang làm của một tính năng hiện trong tính năng đó.
"#;

    fn en_brief() -> String {
        let filler = "The owner reads this short note and decides quickly what happens next with the work. ";
        format!(
            "---\ntitle: Every item carries a brief the owner can approve\naudience: owner\nsize: M\nask: build\n---\n\n**What:** {a}\n\n**Why:** {a}\n\n**Origin:** A request from the owner.\n\n**Cost and risk:** {a}\n\n**Your call:** Approve to build.\n\n**Done when:**\n- A malformed brief is refused.\n- A missing brief is flagged.\n",
            a = filler.repeat(3)
        )
    }

    fn defects_of(text: &str) -> String {
        validate_brief(text, "vi").join("\n")
    }

    fn with_body_line(extra: &str) -> String {
        VI_DRAFT.replacen("**Vì sao:**", &format!("**Vì sao:** {extra}"), 1)
    }

    #[test]
    fn the_vi_draft_validates_clean() {
        assert_eq!(validate_brief(VI_DRAFT, "vi"), Vec::<String>::new());
        assert_eq!(validate_brief(&VI_DRAFT.replace('\n', "\r\n"), "vi"), Vec::<String>::new());
    }

    #[test]
    fn each_defect_class_refuses_by_name() {
        let body_only = VI_DRAFT.splitn(3, "---\n").nth(2).unwrap();
        assert!(defects_of(body_only).contains("does not start with a --- frontmatter line"));
        assert!(defects_of(&VI_DRAFT.replacen("size: M", "size: M\n  nested: x", 1)).contains("nested keys"));
        let long_title = format!("title: {}", "a".repeat(81));
        let t = VI_DRAFT.replacen(
            "title: Mỗi việc và mỗi tính năng có bản tóm tắt đủ để anh duyệt",
            &long_title,
            1,
        );
        assert!(defects_of(&t).contains("title is 81 characters"));
        assert!(defects_of(&VI_DRAFT.replacen("audience: owner", "audience: boss", 1)).contains("audience is \"boss\""));
        assert!(defects_of(&VI_DRAFT.replacen("size: M", "size: XL", 1)).contains("size is \"XL\""));
        assert!(defects_of(&VI_DRAFT.replacen("ask: build", "ask: maybe", 1)).contains("ask is \"maybe\""));
        assert!(defects_of(&VI_DRAFT.replacen("**Nguồn gốc:**", "Nguồn gốc:", 1)).contains("**Nguồn gốc:** is missing"));
        let swapped = VI_DRAFT
            .replacen("**Vì sao:**", "@@", 1)
            .replacen("**Nguồn gốc:**", "**Vì sao:**", 1)
            .replacen("@@", "**Nguồn gốc:**", 1);
        assert!(defects_of(&swapped).contains("out of order"));
        let more = with_body_line(&"chữ ".repeat(11));
        assert!(defects_of(&more).contains("the body has 221 words; it needs 120 to 220"));
        let sized = |filler: usize| {
            format!(
                "---\ntitle: Ngắn\naudience: owner\nsize: S\nask: park\n---\n**Là gì:** {}\n**Vì sao:** Một.\n**Nguồn gốc:** Hai.\n**Chi phí và rủi ro:** Ba.\n**Anh cần làm:** Bốn.\n**Chuẩn xong:**\n- Năm.\n- Sáu.\n",
                "chữ ".repeat(filler)
            )
        };
        let floor = defects_of(&sized(0));
        let fixed: usize = floor.split("the body has ").nth(1).unwrap().split(' ').next().unwrap().parse().unwrap();
        assert!(defects_of(&sized(119 - fixed)).contains("the body has 119 words; it needs 120 to 220"));
        assert_eq!(validate_brief(&sized(120 - fixed), "vi"), Vec::<String>::new());
        let five = VI_DRAFT.trim_end().to_string() + "\n- Một dòng thứ năm.\n";
        assert!(defects_of(&five).contains("holds 5 bullet line(s); it needs 2 to 4"));
        assert!(defects_of(&with_body_line("p-1234abcd")).contains("backlog id: p-1234abcd"));
        assert!(defects_of(&with_body_line("docs/briefs")).contains("file path: docs/briefs"));
        assert!(defects_of(&with_body_line("brief.rs")).contains("names a file: brief.rs"));
        assert!(defects_of(&with_body_line("`x`")).contains("backtick"));
        assert!(defects_of(&with_body_line("foo(")).contains("function call shape: foo("));
    }

    #[test]
    fn an_en_brief_validates_under_en_and_fails_under_vi() {
        let en = en_brief();
        assert_eq!(validate_brief(&en, "en"), Vec::<String>::new());
        let vi = validate_brief(&en, "vi").join("\n");
        assert!(vi.contains("**Là gì:** is missing"), "{vi}");
    }

    fn root_with_config(config: Option<&str>) -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        if let Some(c) = config {
            std::fs::write(tmp.path().join(".bee/config.json"), c).unwrap();
        }
        tmp
    }

    #[test]
    fn brief_language_reads_absent_vi_en_and_unknown() {
        assert_eq!(brief_language(root_with_config(None).path()), ("vi".into(), None));
        let vi = root_with_config(Some(r#"{"brief":{"language":"vi"}}"#));
        assert_eq!(brief_language(vi.path()), ("vi".into(), None));
        let en = root_with_config(Some(r#"{"brief":{"language":"en"}}"#));
        assert_eq!(brief_language(en.path()), ("en".into(), None));
        let fr = root_with_config(Some(r#"{"brief":{"language":"fr"}}"#));
        assert_eq!(brief_language(fr.path()), ("vi".into(), Some("fr".into())));
    }

    #[test]
    fn owner_view_folds_in_flight_work_and_skips_closed_items() {
        let tmp = root_with_config(None);
        let root = tmp.path();
        std::fs::write(root.join(".bee/state.json"), r#"{"feature":"quiet","phase":"idle"}"#).unwrap();
        std::fs::create_dir_all(root.join(".bee/lanes")).unwrap();
        std::fs::write(root.join(".bee/lanes/alpha.json"), r#"{"feature":"alpha","phase":"executing"}"#).unwrap();
        std::fs::write(
            root.join(".bee/lanes/old.json"),
            r#"{"feature":"old","phase":"compounding-complete"}"#,
        )
        .unwrap();
        let rows = [
            r#"{"kind":"pbi","event":"add","id":"p-00000001","title":"folded","status":"in-flight","feature":"alpha"}"#,
            r#"{"kind":"pbi","event":"add","id":"p-00000002","title":"proposed","status":"proposed","feature":"alpha"}"#,
            r#"{"kind":"pbi","event":"add","id":"p-00000003","title":"parked","status":"parked"}"#,
            r#"{"kind":"pbi","event":"add","id":"p-00000004","title":"done","status":"done"}"#,
            r#"{"kind":"pbi","event":"add","id":"p-00000005","title":"declined","status":"declined"}"#,
            r#"{"kind":"pbi","event":"add","id":"p-00000006","title":"loose","status":"in-flight","feature":"old"}"#,
        ];
        std::fs::write(root.join(".bee/backlog.jsonl"), rows.join("\n") + "\n").unwrap();
        std::fs::create_dir_all(root.join("docs/briefs")).unwrap();
        std::fs::write(root.join("docs/briefs/alpha.md"), VI_DRAFT).unwrap();

        let view = owner_view(root);
        let items = view["items"].as_array().unwrap();
        let ids: Vec<&str> = items.iter().map(|i| i["id"].as_str().unwrap()).collect();
        assert_eq!(ids, vec!["alpha", "p-00000002", "p-00000006"]);
        assert_eq!(items[0]["kind"], "feature");
        assert_eq!(items[0]["status"], "executing");
        assert_eq!(items[0]["brief"]["status"], "ok");
        assert_eq!(items[0]["title"], "Mỗi việc và mỗi tính năng có bản tóm tắt đủ để anh duyệt");
        assert_eq!(items[0]["folded"], json!([{"id": "p-00000001", "title": "folded"}]));
        assert_eq!(items[1]["kind"], "pbi");
        assert_eq!(items[1]["brief"]["status"], "missing");
        assert_eq!(items[1]["brief"]["path"], "docs/briefs/p-00000002.md");
        assert_eq!(view["problems"], 2);
        assert_eq!(view["language"], "vi");
        assert_eq!(view["unknown_language"], Value::Null);
    }
}
