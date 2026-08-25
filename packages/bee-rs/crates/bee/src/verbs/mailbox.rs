// verbs/mailbox — the human mailbox store (docs/history/human-mailbox/CONTEXT.md).
//
// After an unattended run, bee files ONE plain-language letter per run into
// `.bee/human-mailbox/`. This module owns the two things that letter is made
// of and nothing above them: the record shape, and the file-backed store that
// holds it. It ships no verb of its own yet — the one command this feature
// owes (D6's read flip) lands in phase 3, so `verbs/mod.rs` registers this as
// a library module (the `workflow_store` / `workspace_store` shape), never a
// probed verb group. Modelled on `verbs/triggers/` (the closest working
// file-backed store) and `verbs/discovery.rs` (a store whose records are
// documents on disk rather than JSON blobs).
//
// ── Two layers (D4) ────────────────────────────────────────────────────────
//
//   .bee/human-mailbox/
//     entries/<run-slug>.jsonl            ← the entry layer, appended live
//     <UTC-timestamp>-<short-run-slug>.md ← the letter, composed at run end
//
// Every clean stop appends its raw entry the moment it happens; the letter is
// composed from those entries when the run ends. A run that dies at 3am still
// leaves everything up to the moment it died.
//
// ── The entry layer: ONE JSONL PER RUN (settled here, hm-1) ───────────────
//
// plan.md left this open between one JSONL per run and a directory of one file
// per entry. It is one JSONL per run, `entries/<run-slug>.jsonl`, append only,
// folded on read. The reasons, so no later reader has to re-derive them:
//
//   1. D11 makes one run the unit — one letter, one run. A per-run file makes
//      a run's entry set addressable BY NAME: composition opens exactly one
//      file instead of scanning a shared directory and filtering by a field.
//   2. D4 demands the append survive a run that dies mid-flight. `append_jsonl`
//      is a single O_APPEND write of one short line — no read-modify-write, no
//      new directory entry per stop, and two sessions appending to different
//      runs never touch the same file.
//   3. D12 asks a later session "which runs have entries but no letter?". With
//      per-run files that is one bounded directory listing of `entries/` diffed
//      against the filed letter names — no entry is read to answer it. A
//      directory-per-entry layout makes the same question a two-level scan.
//   4. It is the shape bee already uses for append-only stores with a folded
//      read (`.bee/backlog.jsonl`, `.bee/decisions.jsonl`) — named in
//      CONTEXT.md's Established Patterns as the shape D4's entry layer should
//      follow. No new storage pattern is invented.
//
// The fold is plain append order: an entry is an EVENT, never a mutation of an
// earlier one, so nothing is superseded on read.
//
// ── The letter: ONE artifact (D3) ────────────────────────────────────────
//
// One letter is ONE markdown file. Typed YAML frontmatter is the machine
// contract (`subject`, `run`, `project`, `filed_at`, `status`, `items[]` of
// `{what, files[], commit, proof, departure}`, `needs_you[]`); the body is the
// human prose. There is NO JSON twin and NO separate index stream — one
// artifact cannot drift against itself, and a directory listing is the index.
// The human opens the file with no tooling; the consumer parses frontmatter.
//
// `subject` is a VALIDITY rule, not a formatting preference (D2): a letter
// without a readable subject is refused before a byte is written, because the
// subject is the inbox row on the consuming side.
//
// Root topology: the mailbox is per-checkout, resolved from whatever root the
// caller hands in. It is a record of what a run did, and a run happens in one
// checkout — unlike a claim or a reservation, nothing coordinates across
// worktrees through it, so it never re-roots onto the control root.

#![allow(dead_code)] // The store lands first (hm-1); its callers arrive with
// hm-2 (append at a clean stop) and hm-3 (compose and file at run end).

use crate::fsutil::{append_jsonl, warn_corrupt_jsonl_line, write_text_atomic};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

// ─── store paths ────────────────────────────────────────────────────────

pub(crate) fn mailbox_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("human-mailbox")
}

pub(crate) fn entries_dir(root: &Path) -> PathBuf {
    mailbox_dir(root).join("entries")
}

/// One JSONL per run. The run id is slugged, so a run id carrying a path
/// separator can never escape the store directory.
pub(crate) fn entries_path(root: &Path, run: &str) -> PathBuf {
    entries_dir(root).join(format!("{}.jsonl", slug_capped(run, 64)))
}

// ─── slugs ──────────────────────────────────────────────────────────────

/// Lowercase kebab of `s`: runs of non-alphanumerics collapse to one `-`, no
/// leading or trailing dash. Empty when `s` carries no letters or digits.
fn kebab(s: &str) -> String {
    let mut out = String::new();
    let mut pending_dash = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.extend(c.to_lowercase());
        } else {
            pending_dash = true;
        }
    }
    out
}

/// Four hex digits of sha256(`s`) — the collision tail on a truncated slug.
fn short_digest(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(s.as_bytes());
    format!("{:02x}{:02x}", digest[0], digest[1])
}

/// `kebab(s)` capped at `max` chars, never empty. A slug that had to be cut
/// carries a digest tail of the FULL input, so two long run ids sharing a
/// prefix can never collapse onto one filename — a collision here would
/// silently merge two runs' entries, or overwrite one run's letter with
/// another's, which is exactly the loss D11's one-letter-per-run rule exists
/// to prevent.
fn slug_capped(s: &str, max: usize) -> String {
    let k = kebab(s);
    if k.is_empty() {
        return "run".to_string();
    }
    if k.chars().count() <= max {
        return k;
    }
    let head: String = k.chars().take(max.saturating_sub(5)).collect();
    let head = head.trim_end_matches('-');
    format!("{head}-{}", short_digest(s))
}

/// D11's `<short-run-slug>`: readable in a bare directory listing, still
/// unique per run.
pub(crate) fn short_run_slug(run: &str) -> String {
    slug_capped(run, 20)
}

// ─── the letter's filename (D11) ────────────────────────────────────────

/// `2026-08-25T03:15:00.123Z` -> `20260825T031500Z`. Compact because a
/// filename must never carry `:` (illegal on Windows), timestamp-led because
/// D11 wants a bare directory listing to sort by time on its own. Tolerant of
/// a fractional part and of an offset spelling; an unparseable input still
/// yields a sortable, filename-safe stamp rather than a panic.
pub(crate) fn compact_utc_stamp(iso: &str) -> String {
    let cut = iso.find(['.', '+']).unwrap_or(iso.len());
    let base = iso[..cut].trim_end_matches('Z');
    let mut out: String = base.chars().filter(char::is_ascii_alphanumeric).collect();
    if out.is_empty() {
        out.push_str("00000000T000000");
    }
    out.push('Z');
    out
}

/// D11: `<UTC-timestamp>-<short-run-slug>.md`. The subject stays in
/// frontmatter — it NEVER enters the filename.
pub(crate) fn letter_filename(filed_at: &str, run: &str) -> String {
    format!("{}-{}.md", compact_utc_stamp(filed_at), short_run_slug(run))
}

pub(crate) fn letter_path(root: &Path, filed_at: &str, run: &str) -> PathBuf {
    mailbox_dir(root).join(letter_filename(filed_at, run))
}

/// Every filed letter, name-sorted — which is time-sorted, by D11's own
/// construction. Names only: nothing is opened, so a caller asking "what is in
/// the mailbox" pays one directory read.
pub(crate) fn list_letter_files(root: &Path) -> Vec<PathBuf> {
    let dir = mailbox_dir(root);
    let mut names: Vec<String> = match std::fs::read_dir(&dir) {
        Ok(entries) => entries
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                name.ends_with(".md").then_some(name)
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    names.sort();
    names.into_iter().map(|n| dir.join(n)).collect()
}

/// The letters already filed for `run` — the bounded read D12 needs to ask
/// "did this run ever get its letter?" without opening a single file.
pub(crate) fn letter_files_for_run(root: &Path, run: &str) -> Vec<PathBuf> {
    let suffix = format!("-{}.md", short_run_slug(run));
    list_letter_files(root)
        .into_iter()
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with(&suffix)))
        .collect()
}

// ─── record parts ───────────────────────────────────────────────────────

/// D5's three required parts. The CLOSED KIND SET and the "a cell that
/// followed its plan says so explicitly" rule are the departure contract of
/// phase 2 (D5, D10) — this shape only has to carry them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Departure {
    pub what: String,
    pub why: String,
    pub kind: String,
}

impl Departure {
    fn to_value(&self) -> Value {
        json!({ "what": self.what, "why": self.why, "kind": self.kind })
    }

    fn from_value(v: &Value) -> Option<Self> {
        let m = v.as_object()?;
        Some(Self {
            what: m.get("what")?.as_str()?.to_string(),
            why: m.get("why")?.as_str()?.to_string(),
            kind: m.get("kind")?.as_str()?.to_string(),
        })
    }
}

/// D13: each Needs-your-call item carries a stable id and names what it
/// blocks. The id is what keeps a reply surface reachable later without
/// rewriting the record shape or any already-filed letter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NeedsYou {
    pub id: String,
    pub what: String,
    pub blocks: String,
}

impl NeedsYou {
    fn to_value(&self) -> Value {
        json!({ "id": self.id, "what": self.what, "blocks": self.blocks })
    }

    fn from_value(v: &Value) -> Option<Self> {
        let m = v.as_object()?;
        Some(Self {
            id: m.get("id")?.as_str()?.to_string(),
            what: m.get("what")?.as_str()?.to_string(),
            blocks: m.get("blocks")?.as_str()?.to_string(),
        })
    }
}

/// D3's `items[]` element: `{what, files[], commit, proof, departure}`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LetterItem {
    pub what: String,
    pub files: Vec<String>,
    pub commit: Option<String>,
    pub proof: Option<String>,
    pub departure: Option<Departure>,
}

impl LetterItem {
    fn to_value(&self) -> Value {
        json!({
            "what": self.what,
            "files": self.files,
            "commit": self.commit,
            "proof": self.proof,
            "departure": self.departure.as_ref().map(Departure::to_value),
        })
    }

    fn from_value(v: &Value) -> Option<Self> {
        let m = v.as_object()?;
        Some(Self {
            what: m.get("what")?.as_str()?.to_string(),
            files: string_list(m, "files"),
            commit: opt_string(m, "commit"),
            proof: opt_string(m, "proof"),
            departure: m.get("departure").and_then(Departure::from_value),
        })
    }
}

// ─── the entry (D4, D8) ─────────────────────────────────────────────────

/// The three clean stops D4 names. Which code path reaches which is hm-2's
/// question; the store only has to name them.
pub(crate) const ENTRY_KINDS: [&str; 3] = ["cap", "feature-close", "blocker"];

/// One raw append written at a clean stop, before any letter exists.
///
/// It mirrors [`LetterItem`] field for field, plus the moment it happened,
/// which stop it was, and any Needs-your-call items that stop raised. That
/// mirroring is deliberate and load-bearing: D8 makes the end-of-run pass a
/// RENDERER WITH AN AUTHORSHIP BAN — it may reorder, group and drop, and may
/// never state a fact no stored entry carries. If the letter could hold a
/// field the entry cannot, the composing pass would have to invent it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Entry {
    /// UTC ISO-8601: the moment the stop happened.
    pub at: String,
    /// One of [`ENTRY_KINDS`].
    pub kind: String,
    /// The plain-language sentence, written at the moment (D8).
    pub what: String,
    pub files: Vec<String>,
    pub commit: Option<String>,
    pub proof: Option<String>,
    pub departure: Option<Departure>,
    pub needs_you: Vec<NeedsYou>,
}

impl Entry {
    pub(crate) fn to_value(&self) -> Value {
        json!({
            "at": self.at,
            "kind": self.kind,
            "what": self.what,
            "files": self.files,
            "commit": self.commit,
            "proof": self.proof,
            "departure": self.departure.as_ref().map(Departure::to_value),
            "needs_you": self.needs_you.iter().map(NeedsYou::to_value).collect::<Vec<_>>(),
        })
    }

    /// `None` for anything JSON-shaped but missing a required field — treated
    /// by every caller exactly like a parse failure, so one bad line never
    /// sinks a run's whole entry set.
    pub(crate) fn from_value(v: &Value) -> Option<Self> {
        let m = v.as_object()?;
        Some(Self {
            at: m.get("at")?.as_str()?.to_string(),
            kind: m.get("kind")?.as_str()?.to_string(),
            what: m.get("what")?.as_str()?.to_string(),
            files: string_list(m, "files"),
            commit: opt_string(m, "commit"),
            proof: opt_string(m, "proof"),
            departure: m.get("departure").and_then(Departure::from_value),
            needs_you: m
                .get("needs_you")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(NeedsYou::from_value).collect())
                .unwrap_or_default(),
        })
    }

    /// The letter item this entry becomes. The composing pass groups and
    /// drops; it never authors (D8).
    pub(crate) fn to_item(&self) -> LetterItem {
        LetterItem {
            what: self.what.clone(),
            files: self.files.clone(),
            commit: self.commit.clone(),
            proof: self.proof.clone(),
            departure: self.departure.clone(),
        }
    }
}

/// Append one entry to this run's JSONL. One O_APPEND write, parents created
/// on the way — the whole point of the entry layer is that this survives a run
/// that never reaches its own end (D4).
pub(crate) fn append_entry(root: &Path, run: &str, entry: &Entry) -> std::io::Result<()> {
    append_jsonl(&entries_path(root, run), &entry.to_value())
}

/// Every entry this run appended, in append order. Fail-open: a missing file
/// is an empty run, and a line that will not parse (or parses to the wrong
/// shape) is skipped with the standard visible warning rather than sinking the
/// read — a torn last line from a run killed mid-write must never cost the
/// entries that landed before it.
pub(crate) fn read_entries(root: &Path, run: &str) -> Vec<Entry> {
    let path = entries_path(root, run);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(line).ok().as_ref().and_then(Entry::from_value) {
            Some(entry) => out.push(entry),
            None => warn_corrupt_jsonl_line(&path, index + 1),
        }
    }
    out
}

/// Every run that has appended at least one entry, name-sorted. One directory
/// read, no entry opened — D12's "which runs went silent?" starts here.
pub(crate) fn runs_with_entries(root: &Path) -> Vec<String> {
    let dir = entries_dir(root);
    let mut names: Vec<String> = match std::fs::read_dir(&dir) {
        Ok(entries) => entries
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                name.strip_suffix(".jsonl").map(str::to_string)
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    names.sort();
    names
}

// ─── the letter (D2, D3, D6) ────────────────────────────────────────────

/// D6: read/unread is a field bee owns inside the letter file. The consuming
/// inbox flips it by calling a bee command, never by writing the file.
pub(crate) const STATUS_UNREAD: &str = "unread";
pub(crate) const STATUS_READ: &str = "read";
pub(crate) const KNOWN_STATUSES: [&str; 2] = [STATUS_UNREAD, STATUS_READ];

/// One filed record: frontmatter is the machine contract, body is the prose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Letter {
    /// D2: one plain-language sentence answering "what happened" on its own.
    pub subject: String,
    pub run: String,
    pub project: String,
    /// UTC ISO-8601.
    pub filed_at: String,
    /// [`STATUS_UNREAD`] or [`STATUS_READ`].
    pub status: String,
    pub items: Vec<LetterItem>,
    pub needs_you: Vec<NeedsYou>,
    /// The human prose below the frontmatter. Its sections are hm-3's (D7).
    pub body: String,
}

/// Why a letter is not a valid record. Each variant names its own remedy in
/// [`LetterInvalid::message`] — a refusal that does not say what to fix is a
/// refusal the caller works around.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LetterInvalid {
    /// D2: no subject at all, or nothing but whitespace.
    MissingSubject,
    /// D2: a subject is ONE sentence — an inbox row, not a paragraph.
    MultiLineSubject,
    /// D2: "no bee vocabulary". See [`BEE_VOCABULARY`].
    BeeVocabulary(String),
    MissingField(&'static str),
    UnknownStatus(String),
}

impl LetterInvalid {
    pub(crate) fn message(&self) -> String {
        match self {
            Self::MissingSubject => {
                "a letter needs a subject: one plain sentence saying what happened (D2)".to_string()
            }
            Self::MultiLineSubject => {
                "the subject must be one sentence on one line — the rest belongs in the body (D2)"
                    .to_string()
            }
            Self::BeeVocabulary(word) => format!(
                "the subject says {word:?} — write it the way you would tell someone who does not use bee (D2)"
            ),
            Self::MissingField(field) => {
                format!("a letter needs {field}: the frontmatter is the machine contract (D3)")
            }
            Self::UnknownStatus(status) => format!(
                "unknown status {status:?} — a letter is {STATUS_UNREAD:?} or {STATUS_READ:?} (D6)"
            ),
        }
    }
}

/// The mechanical floor of D2's "no bee vocabulary" — words that mean nothing
/// outside this harness, matched whole and case-insensitively. It is a FLOOR,
/// not the whole rule: no word list can prove a sentence is plain language. It
/// catches the failure this decision was written against — a subject like
/// "Capped cell hm-1 and closed the slice" reaching a human inbox — and every
/// entry is here because it has no ordinary meaning in a sentence about what a
/// night's work did.
pub(crate) const BEE_VOCABULARY: [&str; 12] = [
    "bee",
    "cell",
    "cells",
    "worktree",
    "worktrees",
    "slice",
    "swarm",
    "swarming",
    "capped",
    "backlog",
    "orchestrator",
    "subagent",
];

/// D2 as a check. Returns the first reason the subject is not a readable inbox
/// row.
pub(crate) fn check_subject(subject: &str) -> Result<(), LetterInvalid> {
    if subject.trim().is_empty() {
        return Err(LetterInvalid::MissingSubject);
    }
    if subject.contains('\n') || subject.contains('\r') {
        return Err(LetterInvalid::MultiLineSubject);
    }
    for word in subject.split(|c: char| !c.is_ascii_alphanumeric()) {
        if word.is_empty() {
            continue;
        }
        let lower = word.to_lowercase();
        if BEE_VOCABULARY.contains(&lower.as_str()) {
            return Err(LetterInvalid::BeeVocabulary(word.to_string()));
        }
    }
    Ok(())
}

impl Letter {
    /// A letter that fails this is not a record. Checked BEFORE any byte is
    /// written and again on every read, so an invalid letter can neither be
    /// filed nor be handed to a consumer as if it were valid (D2, D3, D6).
    pub(crate) fn validate(&self) -> Result<(), LetterInvalid> {
        check_subject(&self.subject)?;
        for (name, value) in
            [("run", &self.run), ("project", &self.project), ("filed_at", &self.filed_at)]
        {
            if value.trim().is_empty() {
                return Err(LetterInvalid::MissingField(name));
            }
        }
        if !KNOWN_STATUSES.contains(&self.status.as_str()) {
            return Err(LetterInvalid::UnknownStatus(self.status.clone()));
        }
        Ok(())
    }

    fn from_frontmatter(v: &Value, body: &str) -> Option<Self> {
        let m = v.as_object()?;
        Some(Self {
            subject: m.get("subject")?.as_str()?.to_string(),
            run: m.get("run")?.as_str()?.to_string(),
            project: m.get("project")?.as_str()?.to_string(),
            filed_at: m.get("filed_at")?.as_str()?.to_string(),
            status: m.get("status")?.as_str()?.to_string(),
            items: m
                .get("items")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(LetterItem::from_value).collect())
                .unwrap_or_default(),
            needs_you: m
                .get("needs_you")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(NeedsYou::from_value).collect())
                .unwrap_or_default(),
            body: body.to_string(),
        })
    }

    /// The filename D11 gives this letter.
    pub(crate) fn filename(&self) -> String {
        letter_filename(&self.filed_at, &self.run)
    }
}

// ─── YAML frontmatter, emitted and read back ────────────────────────────

/// Every scalar is emitted as a JSON string. JSON is a subset of YAML 1.2, so
/// this is a valid double-quoted YAML scalar with exactly one escaping rule to
/// reason about — and a subject carrying a colon, a quote or a newline can
/// never break the block a consumer parses.
fn yq(s: &str) -> String {
    Value::String(s.to_string()).to_string()
}

fn emit_scalar(out: &mut String, indent: usize, key: &str, value: &str) {
    out.push_str(&" ".repeat(indent));
    out.push_str(key);
    out.push_str(": ");
    out.push_str(&yq(value));
    out.push('\n');
}

fn emit_opt(out: &mut String, indent: usize, key: &str, value: Option<&str>) {
    match value {
        Some(v) => emit_scalar(out, indent, key, v),
        None => {
            out.push_str(&" ".repeat(indent));
            out.push_str(key);
            out.push_str(": null\n");
        }
    }
}

fn emit_string_list(out: &mut String, indent: usize, key: &str, values: &[String]) {
    let pad = " ".repeat(indent);
    if values.is_empty() {
        out.push_str(&format!("{pad}{key}: []\n"));
        return;
    }
    out.push_str(&format!("{pad}{key}:\n"));
    for v in values {
        out.push_str(&format!("{pad}  - {}\n", yq(v)));
    }
}

/// The whole letter as the bytes that go on disk: `---`, frontmatter, `---`,
/// blank line, prose. Deterministic — the same letter renders byte-identically
/// every time, so a re-render shows up as a no-op diff.
pub(crate) fn render_letter(letter: &Letter) -> String {
    let mut out = String::from("---\n");
    emit_scalar(&mut out, 0, "subject", &letter.subject);
    emit_scalar(&mut out, 0, "run", &letter.run);
    emit_scalar(&mut out, 0, "project", &letter.project);
    emit_scalar(&mut out, 0, "filed_at", &letter.filed_at);
    emit_scalar(&mut out, 0, "status", &letter.status);

    if letter.items.is_empty() {
        out.push_str("items: []\n");
    } else {
        out.push_str("items:\n");
        for item in &letter.items {
            // The `- ` marker rides the item's first key; every sibling key of
            // the same item indents two further.
            emit_scalar(&mut out, 2, "- what", &item.what);
            emit_string_list(&mut out, 4, "files", &item.files);
            emit_opt(&mut out, 4, "commit", item.commit.as_deref());
            emit_opt(&mut out, 4, "proof", item.proof.as_deref());
            match &item.departure {
                None => out.push_str("    departure: null\n"),
                Some(d) => {
                    out.push_str("    departure:\n");
                    emit_scalar(&mut out, 6, "what", &d.what);
                    emit_scalar(&mut out, 6, "why", &d.why);
                    emit_scalar(&mut out, 6, "kind", &d.kind);
                }
            }
        }
    }

    if letter.needs_you.is_empty() {
        out.push_str("needs_you: []\n");
    } else {
        out.push_str("needs_you:\n");
        for n in &letter.needs_you {
            emit_scalar(&mut out, 2, "- id", &n.id);
            emit_scalar(&mut out, 4, "what", &n.what);
            emit_scalar(&mut out, 4, "blocks", &n.blocks);
        }
    }

    out.push_str("---\n");
    let body = letter.body.trim();
    if !body.is_empty() {
        out.push('\n');
        out.push_str(body);
        out.push('\n');
    }
    out
}

/// `(frontmatter, body)` for a document that opens with a `---` fence.
fn split_frontmatter(text: &str) -> Option<(&str, &str)> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let rest = text.strip_prefix("---\n").or_else(|| text.strip_prefix("---\r\n"))?;
    let mut offset = 0usize;
    for line in rest.split_inclusive('\n') {
        if line.trim_end_matches(['\n', '\r']) == "---" {
            return Some((&rest[..offset], &rest[offset + line.len()..]));
        }
        offset += line.len();
    }
    None
}

struct YLine {
    indent: usize,
    text: String,
}

fn y_lines(text: &str) -> Vec<YLine> {
    text.lines()
        .filter_map(|raw| {
            let line = raw.trim_end();
            let trimmed = line.trim_start();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            Some(YLine { indent: line.len() - trimmed.len(), text: trimmed.to_string() })
        })
        .collect()
}

fn y_scalar(s: &str) -> Value {
    let t = s.trim();
    match t {
        "" | "null" | "~" => Value::Null,
        "[]" => json!([]),
        "{}" => json!({}),
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => {
            if t.starts_with('"') {
                if let Ok(v) = serde_json::from_str::<Value>(t) {
                    return v;
                }
            }
            Value::String(t.to_string())
        }
    }
}

/// A sequence item is a nested mapping when its content opens a `key:` pair;
/// anything quoted, or carrying no colon at all, is a plain scalar item.
fn item_is_mapping(content: &str) -> bool {
    !content.starts_with('"') && content.contains(':')
}

/// A YAML-SUBSET reader: block mappings, block sequences, and the JSON-quoted
/// scalars [`render_letter`] emits. It reads what bee itself writes — the
/// EMITTER is what must be valid YAML for the consuming inbox's real parser,
/// and it is (JSON scalars inside plain block structure). Keeping the reader
/// small avoids pulling a YAML dependency into a crate that has none.
fn y_parse(lines: &mut Vec<YLine>, i: &mut usize, indent: usize) -> Value {
    if *i >= lines.len() || lines[*i].indent < indent {
        return Value::Null;
    }
    let is_item = |l: &YLine| l.text.starts_with("- ") || l.text == "-";
    if is_item(&lines[*i]) {
        let mut arr = Vec::new();
        while *i < lines.len() && lines[*i].indent == indent && is_item(&lines[*i]) {
            let content = lines[*i].text.strip_prefix("- ").unwrap_or("").to_string();
            if content.is_empty() {
                *i += 1;
                arr.push(y_parse(lines, i, indent + 2));
            } else if item_is_mapping(&content) {
                lines[*i] = YLine { indent: indent + 2, text: content };
                arr.push(y_parse(lines, i, indent + 2));
            } else {
                arr.push(y_scalar(&content));
                *i += 1;
            }
        }
        return Value::Array(arr);
    }
    let mut map = Map::new();
    while *i < lines.len() && lines[*i].indent == indent && !is_item(&lines[*i]) {
        let text = lines[*i].text.clone();
        let Some(colon) = text.find(':') else { break };
        let key = text[..colon].trim().to_string();
        let rest = text[colon + 1..].trim().to_string();
        *i += 1;
        let value = if rest.is_empty() {
            match lines.get(*i).map(|l| (l.indent, is_item(l))) {
                Some((child_indent, true)) if child_indent >= indent => {
                    y_parse(lines, i, child_indent)
                }
                Some((child_indent, false)) if child_indent > indent => {
                    y_parse(lines, i, child_indent)
                }
                _ => Value::Null,
            }
        } else {
            y_scalar(&rest)
        };
        map.insert(key, value);
    }
    Value::Object(map)
}

fn parse_frontmatter(text: &str) -> Value {
    let mut lines = y_lines(text);
    let mut i = 0usize;
    y_parse(&mut lines, &mut i, 0)
}

// ─── read / write ───────────────────────────────────────────────────────

#[derive(Debug)]
pub(crate) enum LetterWriteError {
    Invalid(LetterInvalid),
    Io(std::io::Error),
}

impl LetterWriteError {
    pub(crate) fn message(&self) -> String {
        match self {
            Self::Invalid(why) => why.message(),
            Self::Io(e) => format!("could not write the letter: {e}"),
        }
    }
}

/// Write one letter to `.bee/human-mailbox/<UTC-timestamp>-<short-run-slug>.md`
/// and return its path. Validation runs FIRST: an invalid record never reaches
/// the disk, so a refusal leaves the mailbox exactly as it was (D2). The write
/// is atomic (tmp + rename), so a reader never sees half a letter.
///
/// This is the store's write, not the composing pass — deciding WHAT goes in a
/// letter is hm-3's job (D7, D8).
pub(crate) fn write_letter(root: &Path, letter: &Letter) -> Result<PathBuf, LetterWriteError> {
    letter.validate().map_err(LetterWriteError::Invalid)?;
    let path = mailbox_dir(root).join(letter.filename());
    write_text_atomic(&path, &render_letter(letter)).map_err(LetterWriteError::Io)?;
    Ok(path)
}

#[derive(Debug)]
pub(crate) enum LetterReadError {
    Missing,
    /// Present but not a letter — no frontmatter fence, or a frontmatter block
    /// missing a field the contract names.
    Unreadable(PathBuf),
    /// Present, parseable, and not a valid record (D2's subject rule reaches
    /// the READ too — a consumer must never be handed a letter with no row).
    Invalid(LetterInvalid),
}

impl LetterReadError {
    pub(crate) fn message(&self) -> String {
        match self {
            Self::Missing => "no letter at that path".to_string(),
            Self::Unreadable(path) => {
                format!("unreadable letter {} — remedy: fix or delete the file", path.display())
            }
            Self::Invalid(why) => why.message(),
        }
    }
}

pub(crate) fn read_letter(path: &Path) -> Result<Letter, LetterReadError> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Err(if path.exists() {
            LetterReadError::Unreadable(path.to_path_buf())
        } else {
            LetterReadError::Missing
        });
    };
    let Some((frontmatter, body)) = split_frontmatter(&text) else {
        return Err(LetterReadError::Unreadable(path.to_path_buf()));
    };
    let value = parse_frontmatter(frontmatter);
    let Some(letter) = Letter::from_frontmatter(&value, body.trim()) else {
        return Err(LetterReadError::Unreadable(path.to_path_buf()));
    };
    letter.validate().map_err(LetterReadError::Invalid)?;
    Ok(letter)
}

// ─── small value helpers ────────────────────────────────────────────────

fn string_list(m: &Map<String, Value>, key: &str) -> Vec<String> {
    m.get(key)
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default()
}

fn opt_string(m: &Map<String, Value>, key: &str) -> Option<String> {
    m.get(key).and_then(Value::as_str).map(str::to_string)
}

// ─── tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_letter() -> Letter {
        Letter {
            subject: "Two pieces of work landed overnight and one needs your answer".to_string(),
            run: "run-2026-08-25-0315".to_string(),
            project: "beehive".to_string(),
            filed_at: "2026-08-25T03:15:00.123Z".to_string(),
            status: STATUS_UNREAD.to_string(),
            items: vec![
                LetterItem {
                    what: "Wrote the store that keeps these letters: a colon, a \"quote\" and all"
                        .to_string(),
                    files: vec!["a/one.rs".to_string(), "a/two.rs".to_string()],
                    commit: Some("abc1234".to_string()),
                    proof: Some("cargo test — green — the new store only".to_string()),
                    departure: Some(Departure {
                        what: "Used one file per run instead of one file per event".to_string(),
                        why: "It makes a dead run's record findable by name".to_string(),
                        kind: "found a better route".to_string(),
                    }),
                },
                LetterItem {
                    what: "Followed the plan exactly".to_string(),
                    files: vec![],
                    commit: None,
                    proof: None,
                    departure: None,
                },
            ],
            needs_you: vec![NeedsYou {
                id: "ny-1".to_string(),
                what: "Should the letters be kept forever?".to_string(),
                blocks: "the clean-up work".to_string(),
            }],
            body: "# Done\n\nTwo things.".to_string(),
        }
    }

    fn sample_entry(at: &str, what: &str) -> Entry {
        Entry {
            at: at.to_string(),
            kind: "cap".to_string(),
            what: what.to_string(),
            files: vec!["x.rs".to_string()],
            commit: Some("deadbee".to_string()),
            proof: Some("cargo test — green — one module".to_string()),
            departure: None,
            needs_you: vec![],
        }
    }

    // ── D2: the subject is a validity rule ──────────────────────────────

    #[test]
    fn a_letter_with_no_subject_is_refused_and_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let mut letter = sample_letter();
        letter.subject = "   ".to_string();
        let err = write_letter(root, &letter).unwrap_err();
        assert!(matches!(err, LetterWriteError::Invalid(LetterInvalid::MissingSubject)));
        // Refused BEFORE the disk: the mailbox is untouched, not half-written.
        assert!(!mailbox_dir(root).exists());
    }

    #[test]
    fn subject_validity_covers_empty_multiline_and_bee_vocabulary() {
        assert!(matches!(check_subject(""), Err(LetterInvalid::MissingSubject)));
        assert!(matches!(check_subject("\n\t "), Err(LetterInvalid::MissingSubject)));
        assert!(matches!(
            check_subject("Two things landed\nand one broke"),
            Err(LetterInvalid::MultiLineSubject)
        ));
        // The failure D2 was written against: harness words in the inbox row.
        assert!(matches!(
            check_subject("Capped cell hm-1 in the standard lane"),
            Err(LetterInvalid::BeeVocabulary(_))
        ));
        assert!(matches!(check_subject("Merged the worktree"), Err(LetterInvalid::BeeVocabulary(_))));
        // A plain sentence that says what happened passes.
        assert!(check_subject("The overnight run finished the mailbox store").is_ok());
    }

    #[test]
    fn an_invalid_subject_is_refused_on_the_read_too() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let letter = sample_letter();
        let path = write_letter(root, &letter).unwrap();
        // A human hand-edits the subject away: the consumer must not be handed
        // a letter with no inbox row.
        let text = std::fs::read_to_string(&path).unwrap();
        let broken = text.replacen(&format!("subject: {}", yq(&letter.subject)), "subject: \"\"", 1);
        assert_ne!(broken, text);
        std::fs::write(&path, broken).unwrap();
        assert!(matches!(
            read_letter(&path),
            Err(LetterReadError::Invalid(LetterInvalid::MissingSubject))
        ));
    }

    #[test]
    fn every_refusal_names_its_own_remedy() {
        for why in [
            LetterInvalid::MissingSubject,
            LetterInvalid::MultiLineSubject,
            LetterInvalid::BeeVocabulary("cell".to_string()),
            LetterInvalid::MissingField("run"),
            LetterInvalid::UnknownStatus("filed".to_string()),
        ] {
            assert!(!why.message().trim().is_empty());
        }
        assert!(LetterReadError::Missing.message().contains("no letter"));
    }

    // ── D3: one artifact, every field, no twin ──────────────────────────

    #[test]
    fn the_frontmatter_round_trips_every_field_the_contract_names() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let letter = sample_letter();
        let path = write_letter(root, &letter).unwrap();
        let read_back = read_letter(&path).unwrap();
        assert_eq!(read_back, letter);

        // Field by field, against CONTEXT.md's own D3 list — a round trip that
        // passes because both sides dropped the same field is not a proof.
        let text = std::fs::read_to_string(&path).unwrap();
        let (frontmatter, _) = split_frontmatter(&text).unwrap();
        let value = parse_frontmatter(frontmatter);
        for field in ["subject", "run", "project", "filed_at", "status", "items", "needs_you"] {
            assert!(value.get(field).is_some(), "frontmatter is missing {field}");
        }
        let first = &value["items"][0];
        for field in ["what", "files", "commit", "proof", "departure"] {
            assert!(first.get(field).is_some(), "item is missing {field}");
        }
        assert_eq!(first["files"][1], json!("a/two.rs"));
        assert_eq!(first["departure"]["kind"], json!("found a better route"));
        assert_eq!(value["needs_you"][0]["id"], json!("ny-1"));
        assert_eq!(value["needs_you"][0]["blocks"], json!("the clean-up work"));
    }

    #[test]
    fn a_filed_letter_has_no_json_twin_and_no_index_stream() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_letter(root, &sample_letter()).unwrap();
        let names: Vec<String> = std::fs::read_dir(mailbox_dir(root))
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        // Exactly one artifact: the letter. No sibling .json, no index stream.
        assert_eq!(names.len(), 1, "expected only the letter, found {names:?}");
        assert!(names[0].ends_with(".md"));
    }

    #[test]
    fn a_letter_with_no_items_still_round_trips() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let mut letter = sample_letter();
        letter.items = vec![];
        letter.needs_you = vec![];
        letter.body = String::new();
        let path = write_letter(root, &letter).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("items: []"));
        assert!(text.contains("needs_you: []"));
        assert_eq!(read_letter(&path).unwrap(), letter);
    }

    #[test]
    fn rendering_is_deterministic() {
        let letter = sample_letter();
        assert_eq!(render_letter(&letter), render_letter(&letter));
    }

    #[test]
    fn a_file_without_a_frontmatter_fence_is_unreadable_never_a_crash() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("20260825T031500Z-run.md");
        std::fs::write(&path, "just prose, no fence\n").unwrap();
        assert!(matches!(read_letter(&path), Err(LetterReadError::Unreadable(_))));
        let missing = tmp.path().join("nope.md");
        assert!(matches!(read_letter(&missing), Err(LetterReadError::Missing)));
    }

    // ── D11: the filename ───────────────────────────────────────────────

    #[test]
    fn the_filename_is_timestamp_led_and_never_carries_the_subject() {
        let letter = sample_letter();
        let name = letter.filename();
        assert_eq!(name, "20260825T031500Z-run-2026-08-25-0315.md");
        assert!(!name.contains("overnight"), "the subject stays in frontmatter (D11)");
        assert!(!name.contains(':'), "a filename with a colon is unwritable on Windows");
        assert_eq!(letter_path(Path::new("/r"), &letter.filed_at, &letter.run).file_name().unwrap(), name.as_str());
    }

    #[test]
    fn a_bare_directory_listing_sorts_by_time() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Filed out of order, and the later run sorts alphabetically first.
        let mut later = sample_letter();
        later.run = "aaa-later".to_string();
        later.filed_at = "2026-08-25T23:59:00.000Z".to_string();
        let mut earlier = sample_letter();
        earlier.run = "zzz-earlier".to_string();
        earlier.filed_at = "2026-08-25T01:00:00.000Z".to_string();
        write_letter(root, &later).unwrap();
        write_letter(root, &earlier).unwrap();
        let names: Vec<String> = list_letter_files(root)
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            names,
            vec![
                "20260825T010000Z-zzz-earlier.md".to_string(),
                "20260825T235900Z-aaa-later.md".to_string(),
            ]
        );
    }

    #[test]
    fn two_runs_in_one_night_make_two_letters() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let mut first = sample_letter();
        first.run = "run-one".to_string();
        first.filed_at = "2026-08-25T01:00:00.000Z".to_string();
        let mut second = sample_letter();
        second.run = "run-two".to_string();
        second.filed_at = "2026-08-25T05:00:00.000Z".to_string();
        write_letter(root, &first).unwrap();
        write_letter(root, &second).unwrap();
        assert_eq!(list_letter_files(root).len(), 2);
        assert_eq!(letter_files_for_run(root, "run-one").len(), 1);
        assert_eq!(letter_files_for_run(root, "run-two").len(), 1);
        assert_eq!(letter_files_for_run(root, "run-three").len(), 0);
    }

    #[test]
    fn two_long_run_ids_sharing_a_prefix_never_collapse_onto_one_name() {
        let a = "session-2026-08-25-overnight-alpha";
        let b = "session-2026-08-25-overnight-bravo";
        assert_ne!(short_run_slug(a), short_run_slug(b));
        assert_ne!(entries_path(Path::new("/r"), a), entries_path(Path::new("/r"), b));
        assert!(short_run_slug(a).chars().count() <= 20);
    }

    #[test]
    fn a_run_id_with_path_separators_cannot_escape_the_store() {
        let root = Path::new("/r");
        let path = entries_path(root, "../../etc/passwd");
        assert_eq!(path.parent().unwrap(), entries_dir(root));
        assert!(!letter_filename("2026-08-25T00:00:00Z", "../../etc").contains(".."));
    }

    #[test]
    fn the_stamp_survives_an_odd_timestamp() {
        assert_eq!(compact_utc_stamp("2026-08-25T03:15:00.123Z"), "20260825T031500Z");
        assert_eq!(compact_utc_stamp("2026-08-25T03:15:00Z"), "20260825T031500Z");
        assert_eq!(compact_utc_stamp("2026-08-25T03:15:00+00:00"), "20260825T031500Z");
        assert_eq!(compact_utc_stamp(""), "00000000T000000Z");
    }

    // ── D4: the entry layer, one JSONL per run ──────────────────────────

    #[test]
    fn entries_land_in_one_jsonl_per_run_and_read_back_in_order() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        append_entry(root, "run-one", &sample_entry("2026-08-25T01:00:00.000Z", "first")).unwrap();
        append_entry(root, "run-one", &sample_entry("2026-08-25T02:00:00.000Z", "second")).unwrap();
        append_entry(root, "run-two", &sample_entry("2026-08-25T03:00:00.000Z", "other run"))
            .unwrap();

        let one = read_entries(root, "run-one");
        assert_eq!(one.len(), 2);
        assert_eq!(one[0].what, "first");
        assert_eq!(one[1].what, "second");
        // D11: a second run's entries never mix into the first run's letter.
        let two = read_entries(root, "run-two");
        assert_eq!(two.len(), 1);
        assert_eq!(two[0].what, "other run");

        assert!(entries_path(root, "run-one").is_file());
        assert_eq!(runs_with_entries(root), vec!["run-one".to_string(), "run-two".to_string()]);
        // A run that never appended anything is simply empty — never an error.
        assert!(read_entries(root, "run-nine").is_empty());
    }

    #[test]
    fn an_entry_carries_every_field_a_letter_item_needs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let mut entry = sample_entry("2026-08-25T01:00:00.000Z", "did the thing");
        entry.departure = Some(Departure {
            what: "Stopped short of the second file".to_string(),
            why: "The plan was wrong about where the code lived".to_string(),
            kind: "the plan was wrong about a fact".to_string(),
        });
        entry.needs_you = vec![NeedsYou {
            id: "ny-7".to_string(),
            what: "Which name do you want?".to_string(),
            blocks: "the rename".to_string(),
        }];
        append_entry(root, "run-one", &entry).unwrap();
        let read_back = read_entries(root, "run-one");
        assert_eq!(read_back, vec![entry.clone()]);
        // D8: the composing pass renders, it never authors — so the item it
        // can build carries no field the entry did not already hold.
        let item = entry.to_item();
        assert_eq!(item.what, entry.what);
        assert_eq!(item.files, entry.files);
        assert_eq!(item.commit, entry.commit);
        assert_eq!(item.proof, entry.proof);
        assert_eq!(item.departure, entry.departure);
    }

    #[test]
    fn a_torn_last_line_never_costs_the_entries_before_it() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        append_entry(root, "run-one", &sample_entry("2026-08-25T01:00:00.000Z", "landed")).unwrap();
        // A run killed mid-write: a line of the wrong shape, then half a line.
        let path = entries_path(root, "run-one");
        let mut text = std::fs::read_to_string(&path).unwrap();
        text.push_str("{\"at\":\"2026-08-25T02:00:00.000Z\",\"kind\":\"cap\"}\n");
        text.push_str("{\"at\":\"2026-08-25T03:0");
        std::fs::write(&path, text).unwrap();

        let entries = read_entries(root, "run-one");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].what, "landed");
    }

    #[test]
    fn the_three_clean_stops_are_named() {
        assert!(ENTRY_KINDS.contains(&"cap"));
        assert!(ENTRY_KINDS.contains(&"feature-close"));
        assert!(ENTRY_KINDS.contains(&"blocker"));
    }

    // ── D6: bee owns the read state ─────────────────────────────────────

    #[test]
    fn status_is_a_closed_set_bee_owns() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let mut letter = sample_letter();
        assert_eq!(letter.status, STATUS_UNREAD);
        letter.status = STATUS_READ.to_string();
        let path = write_letter(root, &letter).unwrap();
        assert_eq!(read_letter(&path).unwrap().status, STATUS_READ);

        letter.status = "filed".to_string();
        assert!(matches!(
            write_letter(root, &letter).unwrap_err(),
            LetterWriteError::Invalid(LetterInvalid::UnknownStatus(_))
        ));
    }

    // ── the frontmatter reader ──────────────────────────────────────────

    #[test]
    fn the_reader_handles_the_shapes_the_emitter_produces() {
        let text = "\
key: \"a: colon, a \\\"quote\\\", a slash /\"
empty: []
nothing: null
list:
  - \"one\"
  - \"two\"
nested:
  - a: \"1\"
    b:
      c: \"2\"
";
        let v = parse_frontmatter(text);
        assert_eq!(v["key"], json!("a: colon, a \"quote\", a slash /"));
        assert_eq!(v["empty"], json!([]));
        assert_eq!(v["nothing"], Value::Null);
        assert_eq!(v["list"], json!(["one", "two"]));
        assert_eq!(v["nested"][0]["a"], json!("1"));
        assert_eq!(v["nested"][0]["b"]["c"], json!("2"));
    }
}
