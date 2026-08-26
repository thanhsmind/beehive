// verbs/mailbox — the human mailbox store (docs/history/human-mailbox/CONTEXT.md).
//
// After an unattended run, bee files ONE plain-language letter per run into
// `.bee/human-mailbox/`. This module owns the two things that letter is made
// of and nothing above them: the record shape, and the file-backed store that
// holds it — plus the ONE verb the feature owes its consumer, D6's read flip
// (`bee mailbox mark`, at the bottom of this file). hm-1 registered this as a
// library module with no probe because that command was still to come; hm-8
// added the probe, so `verbs/mod.rs` now dispatches `mailbox` like any other
// group. Modelled on `verbs/triggers/` (the closest working file-backed store)
// and `verbs/discovery.rs` (a store whose records are documents on disk rather
// than JSON blobs).
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

#![allow(dead_code)] // The store landed first (hm-1), the append at a clean
// stop second (hm-2), the composing pass third (hm-3), D6's read flip fourth
// (hm-8). What is still unused is the surface D12's recovery consumes.

use super::feedback::{emit_error, emit_success, js_trim, parse_shape, ParsedArgs};
use crate::fsutil::{append_jsonl, warn_corrupt_jsonl_line, write_text_atomic};
use crate::hooks::session_close::project_name;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_worktree, RootsWt};
use crate::verbs::cells::{read_session, sessions_dir, HEARTBEAT_STALE_SECONDS};
use crate::verbs::knowledge::deviation_text;
use crate::verbs::reservations::{date_parse_val, now_iso, now_ms};
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

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

/// D5's CLOSED kind set, in the human's own words. CONTEXT.md's "Specific
/// Ideas And References" says where the four came from: they read the
/// mailbox to see "where the agent decided something off-plan, and why" —
/// an unforeseen blocker, or a better route that only appeared during the
/// work. These four are that reading, written down.
///
/// Closed on purpose. An open kind field gives the same four situations
/// four vocabularies, and the human then has to read every line of the
/// letter's most-read section to learn which situation they are looking at.
/// A fifth kind is a DECISION (a new locked row), never a worker's choice
/// of words at 3am.
pub(crate) const DEPARTURE_KINDS: [&str; 4] = [
    "hit an unforeseen obstacle",
    "found a better route",
    "the plan was wrong about a fact",
    "something else had to be fixed first",
];

/// D5's three part names, in the order the decision says them. Used to tell
/// a departure ATTEMPT (an entry that reaches for these names and misses
/// one) from an entry that was never about a departure at all.
pub(crate) const DEPARTURE_PARTS: [&str; 3] = ["what", "why", "kind"];

/// D5's explicit no-departure statement: what a cell that followed its plan
/// SAYS, rather than staying quiet. CONTEXT.md's own words for why it must
/// be said out loud: "Silence and nothing-happened must not read alike." An
/// empty field cannot be told apart from a worker who never looked, and the
/// letter's most-read section is exactly the place that difference matters.
///
/// Matched as a PREFIX of the normalised line, so "Followed the plan." and
/// "followed the plan — nothing surprising came up" both count. The rule is
/// that the words were SAID, never that they were said in one exact
/// spelling.
pub(crate) const PLAN_FOLLOWED: &str = "followed the plan";

/// The separator between a departure's three parts: the same `" — "` the D8
/// proof string (`<command> — <result> — <scope reason>`) already asks a
/// worker to type, borrowed rather than re-declared so there is ONE
/// separator to learn and the two can never drift apart.
pub(crate) const DEPARTURE_SEPARATOR: &str = crate::verbs::cells::PROOF_SEPARATOR;

/// D5's three required parts — what was done differently, why, and which
/// kind, the kind from [`DEPARTURE_KINDS`].
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

    /// Crate-visible because the moment a departure can be READ is the
    /// moment of the stop itself (D8): the cap turns a worker's own
    /// structured `{what, why, kind}` report entry into this shape while it
    /// still has it in hand.
    ///
    /// PERMISSIVE on purpose, and the only reader that is: this is also how
    /// an ALREADY-FILED letter is read back (`LetterItem::from_value`), and
    /// a letter on disk is a record, never an input to validate. The kind
    /// set is closed at the door where a departure is WRITTEN
    /// ([`read_departure`]), so a letter filed by an older build keeps its
    /// departure instead of losing it on read.
    pub(crate) fn from_value(v: &Value) -> Option<Self> {
        let m = v.as_object()?;
        Some(Self {
            what: m.get("what")?.as_str()?.to_string(),
            why: m.get("why")?.as_str()?.to_string(),
            kind: m.get("kind")?.as_str()?.to_string(),
        })
    }

    /// Trimmed, inner whitespace collapsed, lowercased, trailing sentence
    /// punctuation dropped — how a human's typing is compared against a
    /// closed set without asking them to reproduce it byte for byte.
    fn normalise(s: &str) -> String {
        s.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
            .trim_end_matches(['.', '!'])
            .to_string()
    }

    /// The CANONICAL spelling of `kind`, when it is one of
    /// [`DEPARTURE_KINDS`]. Canonical rather than as-typed so every letter
    /// says the four kinds the same way: the human scans that column, and a
    /// column that changes its wording per entry stops being scannable.
    pub(crate) fn canonical_kind(kind: &str) -> Option<&'static str> {
        let normalised = Self::normalise(kind);
        DEPARTURE_KINDS.into_iter().find(|known| *known == normalised)
    }

    /// Does this line state, explicitly, that the plan was followed (D5)?
    pub(crate) fn plan_followed(line: &str) -> bool {
        Self::normalise(line).starts_with(PLAN_FOLLOWED)
    }

    /// D5's three parts out of ONE line: `<what> — <why> — <kind>`.
    ///
    /// Split at the FIRST separator for `what` and the LAST for `kind`, so
    /// everything between them is `why` and a why may carry the separator
    /// itself. That is the mirror image of the D8 proof string's own split
    /// (first two, reason last) and for the same reason: the segment that
    /// may contain anything is the one nobody has to match.
    ///
    /// `None` when a part is missing or empty, or when the kind is outside
    /// [`DEPARTURE_KINDS`] — a free-form line that happens to carry two
    /// dashes is a note, never a departure with an invented kind.
    pub(crate) fn parse_line(line: &str) -> Option<Self> {
        let (what, rest) = line.split_once(DEPARTURE_SEPARATOR)?;
        let (why, kind) = rest.rsplit_once(DEPARTURE_SEPARATOR)?;
        let (what, why) = (what.trim(), why.trim());
        if what.is_empty() || why.is_empty() {
            return None;
        }
        Some(Self {
            what: what.to_string(),
            why: why.to_string(),
            kind: Self::canonical_kind(kind)?.to_string(),
        })
    }

    /// A departure read out of ONE recorded deviation entry, in whichever
    /// of the two recorded shapes it arrived: the worker's structured
    /// `{what, why, kind}` object, or a one-line `<what> — <why> — <kind>`
    /// string. Anything else — a free-form note, a `{type, description}`
    /// mining entry, the plan-followed statement — reads as no departure.
    pub(crate) fn from_deviation(v: &Value) -> Option<Self> {
        match read_departure(v) {
            DeparturePart::Departure(d) => Some(d),
            _ => None,
        }
    }
}

/// What ONE recorded deviation entry says about D5's departure contract.
///
/// The door that enforces D5 lives at the cap
/// (`verbs/cells/handlers_close.rs`), because that is where a departure is
/// written; the READING of an entry lives here, with the shape, so the door
/// and the letter can never disagree about what counts as a departure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DeparturePart {
    /// Not about a departure at all: a free-form note, a `{type,
    /// description}` mining entry, a number. Recorded as it always was,
    /// never refused — D5 narrowed what a DEPARTURE is, not what may be
    /// written down.
    NotADeparture,
    /// The explicit "I followed my plan" statement (D5).
    PlanFollowed,
    /// All three parts present, kind from the closed set.
    Departure(Departure),
    /// A departure ATTEMPT that misses a part or names a kind outside the
    /// closed set. The string says what is wrong, in the words a refusal
    /// hands back to the worker who has to fix it.
    Malformed(String),
}

/// Read one recorded deviation entry (a report entry, a deviations-file
/// entry) against D5's contract.
pub(crate) fn read_departure(v: &Value) -> DeparturePart {
    match v {
        Value::String(s) => read_departure_line(s),
        Value::Object(m) => {
            // Not a departure attempt at all unless it reaches for at least
            // one of D5's part names. `{type, description}` — the shape the
            // knowledge miner and every deviations-file already use — names
            // none of them and passes straight through.
            if !DEPARTURE_PARTS.iter().any(|part| m.contains_key(*part)) {
                return DeparturePart::NotADeparture;
            }
            let missing: Vec<&str> = DEPARTURE_PARTS
                .into_iter()
                .filter(|part| {
                    m.get(*part).and_then(Value::as_str).map(str::trim).unwrap_or("").is_empty()
                })
                .collect();
            if !missing.is_empty() {
                return DeparturePart::Malformed(format!(
                    "it is missing D5's required part(s) {}",
                    missing.join(", ")
                ));
            }
            let kind = m.get("kind").and_then(Value::as_str).unwrap_or("");
            match Departure::canonical_kind(kind) {
                Some(canonical) => DeparturePart::Departure(Departure {
                    what: m.get("what").and_then(Value::as_str).unwrap_or("").trim().to_string(),
                    why: m.get("why").and_then(Value::as_str).unwrap_or("").trim().to_string(),
                    kind: canonical.to_string(),
                }),
                None => DeparturePart::Malformed(format!(
                    "its kind \"{kind}\" is not one of D5's four"
                )),
            }
        }
        _ => DeparturePart::NotADeparture,
    }
}

/// Read one line — a `--deviation` value, or a string deviation entry —
/// against D5's contract.
///
/// A line that is neither statement reads as [`DeparturePart::NotADeparture`]
/// rather than malformed: a recorded line is free to be an ordinary note.
/// The `--deviation` FLAG is the one place that is not enough, and its door
/// says so itself.
pub(crate) fn read_departure_line(line: &str) -> DeparturePart {
    if Departure::plan_followed(line) {
        return DeparturePart::PlanFollowed;
    }
    match Departure::parse_line(line) {
        Some(d) => DeparturePart::Departure(d),
        None => DeparturePart::NotADeparture,
    }
}

/// D5's closed kind set, rendered for a refusal or a prompt.
pub(crate) fn departure_kinds_line() -> String {
    DEPARTURE_KINDS.join(" / ")
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

/// The three clean stops D4 names.
///
/// Which code path reaches which is answered in
/// `verbs/cells/handlers_close.rs`, above `record_cap_in_mailbox` — one map
/// of all three, written beside the hook that exists. [`KIND_CAP`] was wired
/// by hm-2 and [`KIND_FEATURE_CLOSE`] by hm-10 (`verbs/drivers/close.rs`
/// `record_feature_close_in_mailbox`, on close's non-dry-run tail — a
/// `--dry-run` close stops nothing, so it appends nothing). [`KIND_BLOCKER`]
/// is wired by the cell that owns it.
pub(crate) const KIND_CAP: &str = "cap";
pub(crate) const KIND_FEATURE_CLOSE: &str = "feature-close";
pub(crate) const KIND_BLOCKER: &str = "blocker";
pub(crate) const ENTRY_KINDS: [&str; 3] = [KIND_CAP, KIND_FEATURE_CLOSE, KIND_BLOCKER];

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

#[cfg(test)]
thread_local! {
    /// Test-only instrument for D12's bounded-read promise: how many times
    /// [`read_entries`] opened a run's JSONL on THIS thread. It is a
    /// `thread_local!`, so each `#[test]` — which cargo runs on its own
    /// thread — sees only its own count and no two tests can interfere. It
    /// exists because "the detection opens no entry file" is a claim about a
    /// COST, and a cost claim that nothing measures rots the first time
    /// someone adds a read.
    pub(crate) static ENTRY_READS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Every entry this run appended, in append order. Fail-open: a missing file
/// is an empty run, and a line that will not parse (or parses to the wrong
/// shape) is skipped with the standard visible warning rather than sinking the
/// read — a torn last line from a run killed mid-write must never cost the
/// entries that landed before it.
pub(crate) fn read_entries(root: &Path, run: &str) -> Vec<Entry> {
    read_run(root, run).0
}

/// ONE read of this run's JSONL, folded into BOTH shapes a stored line
/// carries: the [`Entry`] every clean stop shares, and the [`CloseNote`] only
/// a feature-close stop adds beside it (D14 — see that section below).
///
/// One read rather than two, because the notes ride the very lines the
/// entries do: a second pass over the same file to answer the second half of
/// the same question would double the cost of every run end and give the two
/// answers a way to disagree. [`read_entries`] is the counted door, so
/// `ENTRY_READS` keeps measuring exactly what D12 promised — no entry file is
/// opened to decide which runs went silent.
///
/// Same fail-open posture the entry read always had: a missing file is an
/// empty run, and a line that will not parse is skipped with the standard
/// visible warning. A note is only ever read off a line that parsed as an
/// entry, so a torn line can never contribute half a letter section.
pub(crate) fn read_run(root: &Path, run: &str) -> (Vec<Entry>, Vec<CloseNote>) {
    #[cfg(test)]
    ENTRY_READS.with(|c| c.set(c.get() + 1));
    let path = entries_path(root, run);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return (Vec::new(), Vec::new());
    };
    let mut entries = Vec::new();
    let mut notes = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let parsed = serde_json::from_str::<Value>(line).ok();
        match parsed.as_ref().and_then(Entry::from_value) {
            Some(entry) => entries.push(entry),
            None => {
                warn_corrupt_jsonl_line(&path, index + 1);
                continue;
            }
        }
        if let Some(note) = parsed.as_ref().and_then(CloseNote::from_value) {
            notes.push(note);
        }
    }
    (entries, notes)
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

// ─── arming, and the run a stop belongs to (D4, D9, D11) ────────────────

/// D9: is the mailbox ARMED for this run — will a letter be composed and
/// filed when the run ends?
///
/// This NEVER gates an append. Every session appends its entries, attended
/// or not (D9): a session that starts attended and becomes an overnight run
/// must keep a complete record of its whole span, so the question below is
/// asked at the END of a run, by the composing pass, and never at the moment
/// of a stop.
///
/// TWO existing signals, both required, neither invented here:
///
///   1. `.bee/config.json`'s `herding` block (CONTEXT.md, Integration
///      Points) — this checkout is set up to run unattended at all. Read
///      through `state::read_config_raw`, so a `.bee/config.local.json`
///      overlay counts here exactly as it does everywhere else.
///   2. The owner's enable marker, `<main-root>/.bee/tmp/bee-herding.enable`,
///      read through `herding::enable_marker_state` — the one function that
///      already answers it. `docs/knowledge/areas/bee-herding/overview.md`
///      calls that marker "the switch that arms the loop", and only the
///      human ever sets it: it is this feature's own word "armed", already
///      implemented and already understood.
///
/// Why both. Signal 1 alone answers "this checkout CAN run unattended",
/// never "this run IS unattended" — any repo that configures herding would
/// file a letter for every attended session, the exact case D9 excludes.
/// Signal 2 alone would arm the mailbox in a checkout that never herds. No
/// new switch is added for the human to learn, and no new key is read.
pub(crate) fn armed(root: &Path) -> bool {
    herding_configured(root) && owner_armed_the_loop(root)
}

/// Signal 1: a non-empty `herding` block in the merged config.
fn herding_configured(root: &Path) -> bool {
    match crate::state::read_config_raw(root).get("herding") {
        Some(Value::Object(block)) => !block.is_empty(),
        _ => false,
    }
}

/// Signal 2: the owner's enable marker under `root`.
///
/// `root` is handed to `enable_marker_state` as the MAIN checkout root
/// explicitly, which is both correct and cheap on this path: every caller
/// today is a cap, and a cap resolves its store root to the main checkout
/// already (`cells cap` refuses inside a granted worktree; `cells finish`
/// resolves the cell ledger to `StoreRoots::main_root()`). Passing it
/// explicitly also keeps the answer free of the `git rev-parse
/// --git-common-dir` child process the `None` form spawns — a stop must not
/// pay for a subprocess to learn whether a letter will be written.
fn owner_armed_the_loop(root: &Path) -> bool {
    let Some(main_root) = root.to_str() else { return false };
    crate::herding::enable_marker_state(Some(main_root)).get("enabled").and_then(Value::as_bool)
        == Some(true)
}

/// The run of a stop that resolves no session of its own. A nameless run
/// still gets its entries (D4 — the record must survive everything), just in
/// one clearly-labelled bucket rather than silently dropped.
pub(crate) const UNATTRIBUTED_RUN: &str = "unattributed";

/// The run this stop belongs to (D11: one letter, one run).
///
/// A run is a SESSION's span. Both decisions that speak about a run's edges
/// say so in sessions: D9 ("every session appends its entries … its whole
/// span") and D12 ("gets its letter from the NEXT session that starts").
/// So the run id is the caller's session id, resolved by the caller through
/// the ordinary chain (`--session-id`, then `BEE_SESSION_ID`, then
/// `CLAUDE_CODE_SESSION_ID`, then the claim's own recorded session) and
/// handed in here — this function only normalises it, so nothing about
/// reading the environment hides inside the store.
///
/// Deliberately NOT the herding job id (`BEE_HERDING_JOB_ID`): one
/// unattended night dispatches many jobs, and a letter per job would shatter
/// the night into fragments D11 exists to keep whole.
pub(crate) fn run_id(session: Option<&str>) -> String {
    match session.map(str::trim) {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => UNATTRIBUTED_RUN.to_string(),
    }
}

/// D8's plain-language sentence for a capped piece of work, written HERE —
/// at the moment of the stop. The end-of-run pass may reorder, group and
/// drop, and may never state a fact no stored entry carries, so the sentence
/// cannot be deferred to it: an entry that stored only raw material would
/// force the composer to author, which D8 forbids.
///
/// Preference order is "the most human line already on hand": the cap's own
/// `--outcome` (a person wrote it as one line about what happened), then the
/// cell's title, then a last resort that says only what is certainly true.
/// The line is taken VERBATIM — never re-worded — for the same reason: this
/// is the one moment the words may be chosen, and they belong to whoever
/// wrote them.
///
/// The [`BEE_VOCABULARY`] floor is NOT applied here. That floor is D2's
/// validity rule for a letter's SUBJECT (`check_subject`), the row a human
/// reads in an inbox; an entry is raw material and keeps the human's own
/// words even when they say "cell". Choosing a readable subject out of these
/// sentences is the composing pass's own job.
pub(crate) fn cap_sentence(outcome: Option<&str>, title: Option<&str>) -> String {
    if let Some(line) = first_line(outcome) {
        return line;
    }
    match first_line(title) {
        Some(t) if t.ends_with('.') || t.ends_with('!') || t.ends_with('?') => {
            format!("Finished {t}")
        }
        Some(t) => format!("Finished {t}."),
        None => "Finished a piece of work.".to_string(),
    }
}

/// The first non-empty line of `s`, trimmed. A subject is one line (D2), and
/// a subject is later chosen out of these sentences.
fn first_line(s: Option<&str>) -> Option<String> {
    let s = s?;
    s.lines().map(str::trim).find(|l| !l.is_empty()).map(str::to_string)
}

/// Append at a clean stop, FAIL-OPEN.
///
/// The stop being recorded has ALREADY happened — the cap is on disk, the
/// blocker is recorded, the feature is closed. D10's promise that a run
/// which files no letter behaves exactly as it did before this feature is
/// unconditional, so no failure to record a mailbox entry may turn a landed
/// stop into a refusal. The failure is still SAID (a silent gap in a letter
/// is worse than a noisy one) and it names what the human will miss.
pub(crate) fn record_stop(root: &Path, run: &str, entry: &Entry) {
    warn_if_unrecorded(run, append_entry(root, run, entry));
}

/// The ONE thing said when a stop could not be written down, borrowed by
/// every stop kind rather than re-typed per kind — a second wording here is
/// a second promise about what the human will miss.
fn warn_if_unrecorded(run: &str, outcome: std::io::Result<()>) {
    if let Err(err) = outcome {
        eprintln!(
            "bee: could not record the human-mailbox entry for run \"{run}\" ({err}) — the work itself is recorded; this step will be missing from that run's letter."
        );
    }
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

// ─── D6: the read flip, and the ONE command bee owes its consumer ───────
//
// THE SEAM. Everything else in this module is bee talking to itself. This is
// the one place another program touches the mailbox, so it is written as a
// contract rather than a convenience:
//
//     bee mailbox mark --id <letter-file-name> --status read|unread [--json]
//
// WHAT IT IS CALLED, AND WHY — the question CONTEXT.md deferred to planning
// and plan.md left open ("what the read-flip command of D6 is called and
// where it sits in the verb tree"). Settled here, with its reasons, so no
// later reader has to re-derive them:
//
//   * The GROUP is `mailbox`: the store's own name, wired the way
//     `verbs/triggers/` is wired — one probe in `verbs/mod.rs`, one group,
//     its verbs beneath it. hm-1 registered this module as a LIBRARY module
//     with no probe precisely because the one command bee owes was this
//     cell's; this is that probe, and no new registration idiom is invented.
//   * The VERB is `mark`. It is what an inbox does to a letter, it is plain
//     language, and — the deciding reason — it says nothing about READING or
//     LISTING. D1 fixes this feature's ceiling: no rendering surface, no
//     listing UI, no viewer ships from bee. `mailbox read` would have read as
//     "show me the mailbox" at exactly the door where that must be
//     impossible.
//   * The VALUE rides `--status`, because `status` is the frontmatter field
//     bee owns (D3, D6) — the flag names the field it writes, and the closed
//     value set is [`KNOWN_STATUSES`] itself rather than a second copy. The
//     spelling is not new: `state worker update --status` and `work set
//     --status` already mean "the new status of this record". This CLI's flag
//     vocabulary is a ratchet (`catalog.rs`'s `PINNED_FLAG_COUNT`) — a second
//     word for one idea makes every caller who learned the first one wrong.
//   * The IDENTITY rides `--id`, the spelling `cells`, `decisions` and
//     `triggers` all use for "which record". A letter's id is its file name
//     in `.bee/human-mailbox/`, which is the only handle a consumer has: it
//     listed the directory, because by D3 the directory listing IS the index.
//
// IDEMPOTENT ON PURPOSE. Marking a letter that already carries that status is
// a no-op SUCCESS, never an error. The consumer is a UI on the far side of a
// process boundary; it will retry, and a retry punished with a failure teaches
// that caller to stop reading failures at all.
//
// THROUGH THE STORE, ALWAYS. The flip reads the record, changes the one field,
// re-renders and writes atomically. Nothing — not even this module's own verb
// — edits the markdown by hand (D6). [`render_letter`] is deterministic, so
// every other frontmatter field and the whole body come back byte for byte and
// the status line is the only line that moves.

/// What one flip did. `changed: false` is a success, not a refusal — see the
/// idempotence note above.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Marked {
    pub path: PathBuf,
    /// The status the letter carried before the flip.
    pub previous: String,
    /// The status it carries now.
    pub status: String,
    pub changed: bool,
}

/// Why a flip did not happen. Each variant names its own remedy — a refusal
/// that does not say what to fix is a refusal the caller works around.
#[derive(Debug)]
pub(crate) enum MarkError {
    /// A status outside the closed set bee owns (D6).
    UnknownStatus(String),
    /// The id is not a bare file name inside the mailbox directory.
    NotALetterId(String),
    /// No filed letter with that id. Names WHICH letter was asked for.
    NoSuchLetter(String),
    /// There is a file, and it is not a readable record.
    Unreadable(String),
    /// The record read back invalid — kept as its own arm so a bad record can
    /// never be reported as a disk problem.
    Invalid(LetterInvalid),
    Io(std::io::Error),
}

impl MarkError {
    pub(crate) fn message(&self) -> String {
        match self {
            // One wording for one rule: the closed-status refusal is the
            // letter's own, borrowed rather than re-typed.
            Self::UnknownStatus(status) => LetterInvalid::UnknownStatus(status.clone()).message(),
            Self::NotALetterId(id) => format!(
                "{id:?} is not a letter id — a letter id is its file name in .bee/human-mailbox, with no path separators"
            ),
            Self::NoSuchLetter(id) => format!(
                "no letter {id:?} in .bee/human-mailbox — remedy: list that directory and pass a name it holds"
            ),
            Self::Unreadable(why) => why.clone(),
            Self::Invalid(why) => why.message(),
            Self::Io(e) => format!("could not write the letter: {e}"),
        }
    }
}

/// A letter's id is its file name. The `.md` suffix is optional on the way in
/// and canonical on the way out, so a consumer that stored the stem and one
/// that stored the whole name reach the same letter — and every answer names
/// it the one way.
pub(crate) fn letter_id_to_filename(id: &str) -> String {
    if id.ends_with(".md") {
        id.to_string()
    } else {
        format!("{id}.md")
    }
}

/// A letter id addresses ONE file inside the mailbox directory and can never
/// reach out of it — the same plain-id rule `triggers resolve` puts in front
/// of its own store.
fn is_letter_id(id: &str) -> bool {
    !id.is_empty() && !id.contains('/') && !id.contains('\\') && !id.contains("..")
}

/// D6's flip, done BY THE STORE.
///
/// The path comes from the file that was read, never from a name recomputed
/// out of the frontmatter: a hand-renamed letter must be flipped where it
/// lies, not forked into a twin beside itself. That is why this does not go
/// through [`write_letter`] (which addresses by `filed_at` + `run`), and why
/// it re-runs the record's own validation before a byte moves — the door
/// `write_letter` puts in front of the disk is kept, not skipped.
pub(crate) fn mark_letter(root: &Path, id: &str, status: &str) -> Result<Marked, MarkError> {
    if !KNOWN_STATUSES.contains(&status) {
        return Err(MarkError::UnknownStatus(status.to_string()));
    }
    if !is_letter_id(id) {
        return Err(MarkError::NotALetterId(id.to_string()));
    }
    let path = mailbox_dir(root).join(letter_id_to_filename(id));
    let mut letter = match read_letter(&path) {
        Ok(letter) => letter,
        Err(LetterReadError::Missing) => return Err(MarkError::NoSuchLetter(id.to_string())),
        Err(other) => return Err(MarkError::Unreadable(other.message())),
    };
    let previous = letter.status.clone();
    if previous == status {
        // The retry a consumer makes after a dropped response, and the second
        // click on an already-read row. Nothing is written, so the file's
        // mtime does not move either.
        return Ok(Marked { path, previous, status: status.to_string(), changed: false });
    }
    letter.status = status.to_string();
    letter.validate().map_err(MarkError::Invalid)?;
    write_text_atomic(&path, &render_letter(&letter)).map_err(MarkError::Io)?;
    Ok(Marked { path, previous, status: status.to_string(), changed: true })
}

// ─── composing the letter at the end of a run (D4, D7, D8, D9, D11) ─────
//
// THE AUTHORSHIP BAN (D8) is the whole shape of this section. The composing
// pass is a RENDERER, never a summarizer: it may reorder, group and drop, and
// it may NEVER state a fact no stored entry carries. The plain-language
// sentences were already written at the moment of each event (`cap_sentence`,
// hm-2), so nothing below writes prose about what a run did — it sorts stored
// sentences into D7's sections, and every word it adds of its own is either a
// section heading D7 names or one fixed connective.
//
// That is also why there is no "and overall the night went well" line, no
// count, and no judgement anywhere here: each of those would be a fact the
// composer invented, and a letter is only worth reading if every sentence in
// it was true at the moment it was written down.

/// D7's five sections, in D7's own words and D7's own order. They are
/// constants because the headings ARE the decision — renaming one here renames
/// it in the decision, and that is not a formatting choice.
pub(crate) const SECTION_DONE: &str = "Done";
pub(crate) const SECTION_DEPARTED: &str = "Where I departed from the plan and why";
pub(crate) const SECTION_BROKEN: &str = "Broken or unfinished";
pub(crate) const SECTION_NEEDS_YOU: &str = "Needs your call";
pub(crate) const SECTION_NEXT: &str = "Next";
pub(crate) const SECTIONS: [&str; 5] =
    [SECTION_DONE, SECTION_DEPARTED, SECTION_BROKEN, SECTION_NEEDS_YOU, SECTION_NEXT];

/// The subject of a letter whose every stored sentence is unusable as one.
///
/// D2 makes the subject a VALIDITY rule, so a letter must have one; D8 forbids
/// authoring a fact no entry carries. Between those two, the last resort can
/// only say what is true by the letter's own existence: this run recorded
/// something, and it is here. It states nothing about WHAT the run did —
/// that is what the body and the frontmatter are for.
///
/// It fires rarely. Every stored sentence is a candidate first, and only a run
/// whose sentences are all multi-line, empty, or written in harness words
/// (`BEE_VOCABULARY`) reaches this line.
pub(crate) const FALLBACK_SUBJECT: &str = "The run left something for you to read.";

/// Whitespace normalised to single spaces on one line. A bullet is one line,
/// and collapsing runs of blanks changes no fact — it is the only edit this
/// pass makes to a stored sentence.
fn one_line(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn bullet(text: &str) -> String {
    format!("- {}", one_line(text))
}

/// A departure, rendered through `knowledge::deviation_text` — which
/// `cells/handlers_close.rs:13` names "the ONE rendering of a deviation entry",
/// already shared with the promote miner. A second idea of what a deviation
/// reads like is the defect this avoids, so D5's three parts are mapped onto
/// the `{type, description}` shape that function already understands and it
/// does the rendering.
///
/// The mapping joins two stored facts with a dash and invents neither (D8).
fn departure_line(d: &Departure) -> String {
    deviation_text(&json!({
        "type": d.kind.trim(),
        "description": format!("{} — {}", d.what.trim(), d.why.trim()),
    }))
}

/// D7: a section with nothing to report is DROPPED, never printed empty.
fn push_section(out: &mut String, heading: &str, lines: &[String]) {
    if lines.is_empty() {
        return;
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str("## ");
    out.push_str(heading);
    out.push_str("\n\n");
    for line in lines {
        out.push_str(line);
        out.push('\n');
    }
}

/// The human prose of a letter: D7's five sections over the stored entries.
///
/// Where each entry goes, and why nothing is invented on the way:
///
///   * `Done` — every entry that is not a blocker, by its own sentence. Not a
///     whitelist of kinds: an entry of some kind this build does not know is
///     still a thing the run did, and dropping it would lose a stored fact.
///   * `Where I departed from the plan and why` — every entry carrying D5's
///     three parts, through `departure_line`.
///   * `Broken or unfinished` — every `KIND_BLOCKER` entry, by its sentence.
///   * `Needs your call` — every stored `NeedsYou`, with its stable id and
///     what it blocks (D13). This feature ships no path to answer one; the id
///     is what keeps that surface reachable later.
///   * `Next` — DROPPED, always, today. No stored entry carries a next step:
///     an `Entry` has no such field, so the only way to print this section
///     would be to author it, which D8 forbids. D7 already says a section with
///     nothing to report is dropped, so the honest render of "no entry knows
///     what comes next" is silence. A later phase that gives an entry a next
///     step gives this section its source; until then the empty list below is
///     the decision, written out rather than left implicit.
pub(crate) fn compose_body(entries: &[Entry]) -> String {
    let mut done: Vec<String> = Vec::new();
    let mut departed: Vec<String> = Vec::new();
    let mut broken: Vec<String> = Vec::new();
    let mut needs_you: Vec<String> = Vec::new();

    for entry in entries {
        if entry.kind == KIND_BLOCKER {
            broken.push(bullet(&entry.what));
        } else {
            done.push(bullet(&entry.what));
        }
        if let Some(departure) = &entry.departure {
            departed.push(bullet(&departure_line(departure)));
        }
        for n in &entry.needs_you {
            // "blocks" is the ONE connective this pass adds of its own; D13
            // requires the item to name what it blocks, and the id leads so a
            // human can quote it back.
            needs_you.push(bullet(&format!(
                "[{}] {} — blocks: {}",
                n.id.trim(),
                one_line(&n.what),
                one_line(&n.blocks)
            )));
        }
    }
    let next: Vec<String> = Vec::new();

    let mut out = String::new();
    for (heading, lines) in [
        (SECTION_DONE, &done),
        (SECTION_DEPARTED, &departed),
        (SECTION_BROKEN, &broken),
        (SECTION_NEEDS_YOU, &needs_you),
        (SECTION_NEXT, &next),
    ] {
        push_section(&mut out, heading, lines);
    }
    out
}

/// D2's inbox row, CHOSEN out of the stored sentences rather than written.
///
/// The first stored sentence that is a valid subject on its own wins, taken
/// verbatim — never re-worded, because the words belong to whoever wrote them
/// at the moment of the stop (D8). Append order, so the choice is stable: a
/// run re-composed after two more stops keeps the subject it already had.
fn choose_subject(entries: &[Entry]) -> String {
    entries
        .iter()
        .map(|e| e.what.trim())
        .find(|s| check_subject(s).is_ok())
        .map(str::to_string)
        .unwrap_or_else(|| FALLBACK_SUBJECT.to_string())
}

/// Compose this run's stored entries into ONE letter (D4, D7, D8, D11).
///
/// `None` when the run stored nothing: a letter is composed FROM the entries,
/// so a run with no entry carries no fact to write one from, and a letter
/// about it could only be authored. A run that recorded nothing gets no
/// letter, and `runs_with_entries` never lists it — so D12's later recovery
/// pass does not see a hole either.
///
/// The status is always [`STATUS_UNREAD`]; a re-composed letter's caller puts
/// the human's own read state back (see [`file_run_letter`]).
pub(crate) fn compose_letter(
    project: &str,
    run: &str,
    filed_at: &str,
    entries: &[Entry],
) -> Option<Letter> {
    if entries.is_empty() {
        return None;
    }
    Some(Letter {
        subject: choose_subject(entries),
        run: run.to_string(),
        project: project.to_string(),
        filed_at: filed_at.to_string(),
        status: STATUS_UNREAD.to_string(),
        // D8 again, at the machine contract: an item is exactly what one entry
        // already held (`Entry::to_item`), reordered and grouped at most.
        items: entries.iter().map(Entry::to_item).collect(),
        needs_you: entries.iter().flat_map(|e| e.needs_you.iter().cloned()).collect(),
        body: compose_body(entries),
    })
}

/// What the end of a run did about its letter.
#[derive(Debug)]
pub(crate) enum RunEnd {
    /// D9: an attended run records its entries and files NO letter.
    NotArmed,
    /// The run stored nothing, so there is no fact to compose from.
    NoEntries,
    /// Written — or re-written in place, keeping D11's one letter per run.
    Filed(PathBuf),
    /// D11's refusal, not a failure: this run already has its ONE letter, so
    /// nothing was written and the letter it has is named. Only the D12
    /// recovery pass can reach it — an ordinary run end RE-COMPOSES its own
    /// letter in place instead (see [`file_run_letter`]).
    AlreadyFiled(PathBuf),
    /// Nothing was written, and the human will miss this run's letter. The
    /// string names what to fix.
    Failed(String),
}

/// The project a letter is about: the checkout's own name, through the same
/// `project_name` the session record already uses, so bee has one idea of
/// which project a run happened in.
fn project_of(root: &Path) -> String {
    project_name(&Value::String(root.to_string_lossy().into_owned()))
}

/// Compose and file this run's ONE letter — the end-of-run half of D4.
///
/// ARMING (D9) is asked HERE and nowhere earlier: every session appends its
/// entries, attended or not, and only an unattended run files a letter. A
/// session that started attended and became an overnight run therefore carries
/// its whole span into the letter it files.
///
/// D11 — one letter maps to one RUN, never one night, and never one job. A run
/// that reaches its end more than once (an agent that finishes an ask, works
/// on, and finishes another) RE-COMPOSES the letter it already has, in place:
/// the original `filed_at` keeps the filename stable, and the human's own read
/// state survives the rewrite. Filing a second file would be a second letter
/// for one run; dropping the later entries would lose facts the run recorded.
/// Both are refused, so the run's letter is always all of the run.
///
/// An existing letter that cannot be READ stops the write instead of routing
/// around it — writing beside it is how one run ends up with two letters.
pub(crate) fn file_run_letter(root: &Path, run: &str) -> RunEnd {
    if !armed(root) {
        return RunEnd::NotArmed;
    }
    // D14: the notes come off the very lines the entries do, so a run that
    // closed a feature carries its three extra sections into the ONE letter
    // it already gets — never into a second file beside it.
    let (entries, notes) = read_run(root, run);
    if entries.is_empty() {
        return RunEnd::NoEntries;
    }

    let mut filed_at = now_iso();
    let mut status = STATUS_UNREAD.to_string();
    if let Some(existing) = letter_files_for_run(root, run).into_iter().next() {
        match read_letter(&existing) {
            Ok(old) => {
                filed_at = old.filed_at;
                status = old.status;
            }
            Err(why) => {
                return RunEnd::Failed(format!(
                    "this run already has a letter at {} that cannot be read ({}) — remedy: fix or delete that file; writing a second letter for one run is refused (D11)",
                    existing.display(),
                    why.message()
                ))
            }
        }
    }

    let Some(mut letter) = compose_letter_with(&project_of(root), run, &filed_at, &entries, &notes)
    else {
        return RunEnd::NoEntries;
    };
    letter.status = status;
    match write_letter(root, &letter) {
        Ok(path) => RunEnd::Filed(path),
        Err(why) => RunEnd::Failed(why.message()),
    }
}

/// [`file_run_letter`], FAIL-OPEN — the shape [`record_stop`] already has.
///
/// The run has ended; nothing about a letter may turn that into a refusal
/// (D10). The failure is still SAID, because a silently missing letter is
/// worse than a noisy one: the human would read an empty mailbox as a quiet
/// night rather than as a broken store.
pub(crate) fn record_run_end(root: &Path, run: &str) -> RunEnd {
    let outcome = file_run_letter(root, run);
    if let RunEnd::Failed(why) = &outcome {
        eprintln!(
            "bee: could not file the human-mailbox letter for run \"{run}\" ({why}) — the work itself is recorded; that run has no letter to read."
        );
    }
    outcome
}

// ─── D12: the run that went silent ──────────────────────────────────────
//
// A run that dies WITHOUT reaching its own end gets its letter from the NEXT
// SESSION THAT STARTS. That letter is marked plainly as an unfinished run, it
// lists the entries up to the last one, and it names the moment the run went
// silent.
//
// NO BACKGROUND SCHEDULER, and the reason is the decision's own: a scheduler
// shares the failure mode it would exist to cover. The thing that kills a run
// at 3am — the machine sleeping, the laptop lid, the power cut — kills a timer
// in the same process and on the same box. Only a LATER session can be trusted
// to have survived, so the recovery is a few reads on a path a later session
// already walks (`bee work set`, in `verbs/work.rs`). Nothing polls.
//
// ── The bounded detection (plan.md's deferred question) ──────────────────
//
// CONTEXT.md deferred "how a later session recognises orphaned entries without
// scanning the whole store on every start". Settled here, with its costs:
//
//   0. `armed(root)` — one config read and one marker stat. An unarmed
//      checkout stops HERE and pays no directory read at all, so a repo that
//      never runs unattended never pays for this feature (D9).
//   1. `runs_with_entries` — ONE directory listing of `entries/`, names only.
//   2. `letter_run_slugs` — ONE directory listing of the mailbox, names only.
//   3. `session_ids` — ONE directory listing of `.bee/sessions/`, names only.
//   4. For each CANDIDATE only — a run that has entries, has no letter, and is
//      not this session — one session record read, and one entry-file read if
//      it is filed. Candidates are normally ZERO.
//
// Three directory listings and no file opened is the whole steady-state cost.
// It is O(runs + letters + sessions) in DIRECTORY ENTRIES and O(candidates) in
// FILE OPENS, and the second number is the one that matters: not one entry is
// read to answer "which runs went silent?". `ENTRY_READS` measures exactly
// that, so the promise cannot rot silently.
//
// Why the session ids drive the loop rather than the entry file names: an
// entry file is named `slug_capped(run, 64)` and a letter carries
// `short_run_slug(run)` — two different truncations of the same run id, and
// neither is invertible. The session id is the run id itself (`run_id`), so
// slugging it forwards is the one direction that is exact.
//
// ── Never sweep a LIVE run (the hard part) ──────────────────────────────
//
// From the outside a run still working looks EXACTLY like a dead one: entries
// on disk, no letter yet. Filing an "unfinished" letter for a run that is
// still going is worse than filing nothing — it tells the human a lie about
// work that is happening while they read it.
//
// The signal that separates them is the one bee already reclaims by: the
// session record under `.bee/sessions/<id>.json`, its `status`, and its
// `last_heartbeat`. A run IS a session's span (`run_id`), so the run id is the
// session id and the record is a direct lookup — no scan, no guess.
//
// FAIL CLOSED, the same posture the claim sweep and `worktree prune` take: a
// signal that is missing, unreadable or ambiguous KEEPS the run rather than
// reclaiming it. `run_went_silent` therefore says "silent" ONLY on positive
// evidence — the record says `closed`/`dead`, or its heartbeat parsed and is
// older than `HEARTBEAT_STALE_SECONDS`. No record at all, no heartbeat field,
// an unparseable stamp: not silent, no letter, try again next session. That is
// a deliberate divergence from `claims::heartbeat_stale`, which answers the
// opposite question (may I take this claim?) and so treats an absent record as
// stale. Here an absent record is an absent WITNESS.
//
// It is also why `UNATTRIBUTED_RUN` never gets an unfinished letter: nothing
// ever wrote a session record named "unattributed", so there is no witness
// that its run is over.
//
// ── The authorship ban still holds (D8) ─────────────────────────────────
//
// An unfinished letter invents NOTHING about why the run stopped. bee knows
// two things and says exactly those two: the run never reached its end, and
// the last moment it recorded anything. It does not say the run crashed, or
// failed, or was killed — it does not know that, and a letter is only worth
// reading if every sentence in it was true when it was written.
//
// The moment is the LAST ENTRY's `at`, not the session's last heartbeat: the
// last entry is a fact the run itself stored, so the sentence naming it stays
// inside D8's ban with nothing borrowed from outside the entry set.

/// The mark that makes an unfinished letter tell itself apart AT A GLANCE
/// (D12). It leads the SUBJECT, because the subject is the inbox row a human
/// reads first (D2) — a mark buried in the body is not "at a glance".
///
/// No frontmatter FIELD is added for it. D3 fixes the machine contract's field
/// list (`subject`, `run`, `project`, `filed_at`, `status`, `items[]`,
/// `needs_you[]`) and another project consumes it; extending that list is a
/// decision, not a worker's choice. The mark rides the field D3 already has.
pub(crate) const UNFINISHED_SUBJECT_MARK: &str = "Unfinished run:";

/// The body's own heading for the mark. It is NOT one of D7's five — those are
/// what a finished run's stops are sorted into, and this says something about
/// the run itself — so it leads the body, above them.
pub(crate) const SECTION_UNFINISHED: &str = "Unfinished run";

/// The two facts an unfinished letter is allowed to state, and all of them.
/// Constants because they are the decision's own words: what bee knows is
/// "it did not reach its end" and "this was the last thing it recorded".
pub(crate) const UNFINISHED_LINE: &str = "This run never reached its end.";
pub(crate) const SILENT_AFTER_PREFIX: &str = "Nothing more was recorded after";

/// The moment the run went silent: the `at` of its LAST stored entry — the
/// last thing the run said before it stopped saying anything.
pub(crate) fn went_silent_at(entries: &[Entry]) -> Option<String> {
    entries.last().map(|e| e.at.trim().to_string()).filter(|at| !at.is_empty())
}

/// The unfinished letter's prose: the mark first, then D7's five sections over
/// the entries, composed by the same [`compose_body`] a finished run uses.
///
/// Same renderer, one extra section — so an unfinished letter can never drift
/// away from what a finished one reads like, and the entries are listed up to
/// the last one exactly as D12 asks.
pub(crate) fn compose_unfinished_body(entries: &[Entry]) -> String {
    compose_unfinished_body_with(entries, &[])
}

/// [`compose_unfinished_body`] carrying D14's three extra sections, for a run
/// that closed a feature and then went silent.
pub(crate) fn compose_unfinished_body_with(entries: &[Entry], notes: &[CloseNote]) -> String {
    let moment = match went_silent_at(entries) {
        Some(at) => format!("- {UNFINISHED_LINE} {SILENT_AFTER_PREFIX} {at}."),
        // No entry carries a moment, so no moment is named. D8: silence beats
        // an invented timestamp.
        None => format!("- {UNFINISHED_LINE}"),
    };
    let lead = format!("## {SECTION_UNFINISHED}\n\n{moment}\n");
    let rest = compose_body_with(entries, notes);
    if rest.is_empty() {
        return lead;
    }
    format!("{lead}\n{rest}")
}

/// [`compose_letter`] for a run that went silent (D12).
///
/// Everything below the subject and the leading section is the ordinary
/// composition — same items, same sections, same authorship ban.
pub(crate) fn compose_unfinished_letter(
    project: &str,
    run: &str,
    filed_at: &str,
    entries: &[Entry],
) -> Option<Letter> {
    compose_unfinished_letter_with(project, run, filed_at, entries, &[])
}

/// [`compose_unfinished_letter`] carrying D14's three extra sections.
pub(crate) fn compose_unfinished_letter_with(
    project: &str,
    run: &str,
    filed_at: &str,
    entries: &[Entry],
    notes: &[CloseNote],
) -> Option<Letter> {
    let mut letter = compose_letter(project, run, filed_at, entries)?;
    letter.subject = format!("{UNFINISHED_SUBJECT_MARK} {}", letter.subject);
    letter.body = compose_unfinished_body_with(entries, notes);
    Some(letter)
}

/// Is there POSITIVE evidence that the session behind `run` is over?
///
/// Fail closed. Everything that is not evidence — no record, no heartbeat, a
/// stamp that will not parse — answers `false`, and the run keeps its silence
/// until a later session can see better. See this section's header for why
/// that is the opposite default from `claims::heartbeat_stale`.
fn run_went_silent(control: &Path, run: &str, now: f64) -> bool {
    let Ok(Some(record)) = read_session(control, run) else {
        return false; // no witness — never "it is dead", only "I cannot tell"
    };
    if matches!(record.get("status"), Some(Value::String(s)) if s == "closed" || s == "dead") {
        return true;
    }
    match date_parse_val(record.get("last_heartbeat")) {
        Ok(Some(ms)) => ms + HEARTBEAT_STALE_SECONDS * 1000.0 <= now,
        _ => false,
    }
}

/// The `<short-run-slug>` of every filed letter. ONE directory listing, names
/// only — the letter name is `<stamp>-<short-run-slug>.md` and the stamp
/// carries no `-` (see [`compact_utc_stamp`]), so the first `-` is the split.
fn letter_run_slugs(root: &Path) -> HashSet<String> {
    let mut out = HashSet::new();
    for path in list_letter_files(root) {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
        let Some(stem) = name.strip_suffix(".md") else { continue };
        if let Some((_, slug)) = stem.split_once('-') {
            out.insert(slug.to_string());
        }
    }
    out
}

/// Every session id this checkout has a record for, name-sorted. ONE directory
/// listing, no record opened — a record is read only for a candidate.
fn session_ids(control: &Path) -> Vec<String> {
    let dir = sessions_dir(control);
    let mut names: Vec<String> = match std::fs::read_dir(&dir) {
        Ok(entries) => entries
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                name.strip_suffix(".json").map(str::to_string)
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    names.sort();
    names
}

/// The runs that MIGHT have gone silent: they stored entries, they have no
/// letter, and they are not this session. Three directory listings and not one
/// file opened — see this section's header for the full cost.
///
/// `root` is the CONTROL root: entries, letters and session records all live
/// under it, and a linked worktree's cap already resolved its store there.
///
/// `current_run` is excluded belt-and-braces. A live sibling is already kept by
/// `run_went_silent`'s heartbeat, but THIS session's own heartbeat may not be
/// stamped yet at the moment it starts, and a session that filed an unfinished
/// letter for itself would be the worst version of this bug.
pub(crate) fn silent_run_candidates(root: &Path, current_run: &str) -> Vec<String> {
    let entry_slugs: HashSet<String> = runs_with_entries(root).into_iter().collect();
    if entry_slugs.is_empty() {
        return Vec::new();
    }
    let lettered = letter_run_slugs(root);
    let mut out = Vec::new();
    for id in session_ids(root) {
        if id == current_run {
            continue;
        }
        if !entry_slugs.contains(&slug_capped(&id, 64)) {
            continue;
        }
        if lettered.contains(&short_run_slug(&id)) {
            continue;
        }
        out.push(id);
    }
    out
}

/// File the ONE unfinished letter for a run that went silent (D12).
///
/// D11 is re-checked at the door to the disk even though the candidate filter
/// already applied it: a second letter for one run is the loss that rule
/// exists to prevent, and the last check before a write is the one that has to
/// hold. An existing letter is [`RunEnd::AlreadyFiled`] — a refusal, not a
/// failure, and nothing is rewritten. Unlike an ordinary run end, this pass
/// never re-composes a letter in place: the run is gone, so there is nothing
/// new to fold in, and the human's own reading of that letter stands.
pub(crate) fn file_unfinished_letter(root: &Path, run: &str) -> RunEnd {
    if !armed(root) {
        return RunEnd::NotArmed;
    }
    if let Some(existing) = letter_files_for_run(root, run).into_iter().next() {
        return RunEnd::AlreadyFiled(existing);
    }
    // D14: a run that closed a feature and THEN went silent keeps the three
    // extra sections its close recorded — the recovery pass reads the same
    // lines the ordinary run end does.
    let (entries, notes) = read_run(root, run);
    let Some(letter) =
        compose_unfinished_letter_with(&project_of(root), run, &now_iso(), &entries, &notes)
    else {
        return RunEnd::NoEntries;
    };
    match write_letter(root, &letter) {
        Ok(path) => RunEnd::Filed(path),
        Err(why) => RunEnd::Failed(why.message()),
    }
}

/// D12's whole pass: at the start of a session, file the unfinished letter of
/// every run that went silent. `root` is the CONTROL root; `current_run` is
/// this session's own run id.
///
/// Returns one `(run, outcome)` per run it decided about, so a caller can say
/// what happened; a run that is still live, or that cannot be judged, is not
/// in the list at all.
pub(crate) fn file_letters_for_silent_runs(root: &Path, current_run: &str) -> Vec<(String, RunEnd)> {
    // Step 0 of the bounded read: an unarmed checkout stops before it lists a
    // single directory (D9 — an attended checkout files no letter anyway).
    if !armed(root) {
        return Vec::new();
    }
    let now = now_ms();
    let mut out = Vec::new();
    for run in silent_run_candidates(root, current_run) {
        if !run_went_silent(root, &run, now) {
            continue;
        }
        out.push((run.clone(), file_unfinished_letter(root, &run)));
    }
    out
}

/// [`file_letters_for_silent_runs`], FAIL-OPEN — the shape [`record_run_end`]
/// and [`record_stop`] already have.
///
/// The caller's actual ask is its own; recovering some OTHER run's letter must
/// never turn that ask into a refusal. A failure is still said out loud, and
/// it names the run whose letter the human will not find.
pub(crate) fn record_silent_runs(root: &Path, current_run: &str) -> Vec<(String, RunEnd)> {
    let outcomes = file_letters_for_silent_runs(root, current_run);
    for (run, outcome) in &outcomes {
        if let RunEnd::Failed(why) = outcome {
            eprintln!(
                "bee: could not file the human-mailbox letter for run \"{run}\", which went silent without finishing ({why}) — remedy: that run has no letter to read until this is fixed."
            );
        }
    }
    outcomes
}

// ─── D14: the feature-close letter ──────────────────────────────────────
//
// D7 already promised this letter: "Architecture, behaviour and usage appear
// only in the feature-close letter." Until now no feature-close letter
// existed, so that clause described NOTHING. This section fills in the shape
// D7 already carved out — it does not invent a second letter format.
//
// ── The same record, the same file (D3, D11) ────────────────────────────
//
// A feature close is one of D4's three clean stops, and a stop happens INSIDE
// a run. So the feature-close letter is not a new artifact filed beside the
// run's letter — it IS the run's letter, for a run that closed a feature.
// Same frontmatter contract (nothing is added to D3's field list, for the
// same reason `UNFINISHED_SUBJECT_MARK` adds nothing: extending the machine
// contract another project consumes is a DECISION, not a worker's choice),
// same `<UTC-timestamp>-<short-run-slug>.md` name (D11), same one letter per
// run. Filing a second file would be a second letter for one run, which is
// precisely the loss D11 exists to prevent.
//
// ── "Only in the feature-close letter", held by construction ────────────
//
// The three extra sections are composed from [`CloseNote`]s, and only the
// feature-close stop ever writes one. A nightly run stores none, so its three
// lists are empty, so `push_section` — D7's own "a section with nothing to
// report is dropped, never printed empty" — drops all three. There is ONE
// dropping rule in this module and this reuses it rather than growing a
// second one. The promise holds because the material is absent, not because a
// second code path remembered to check a flag.
//
// ── The authorship ban is NOT relaxed here (D8) ─────────────────────────
//
// Architecture, behaviour and usage may state no fact no stored entry
// carries, exactly like the five sections above them. So nothing in this
// module writes a word of them: the feature close reads three lists of
// already-recorded facts out of the feature's own capped cells and STORES
// them at the moment of the stop (`verbs/drivers/close.rs`
// `record_feature_close_in_mailbox`), and the composing pass only sorts,
// dedupes and drops. If a feature recorded no material for a section, that
// section is absent — silence is the honest render, and authoring prose to
// fill a heading is the one thing D8 forbids outright.
//
// ── Where the three lists live ──────────────────────────────────────────
//
// On the feature-close stop's OWN JSONL line, beside the fields every stop
// shares. They are kind-specific payload — no other stop kind has an
// architecture — so they are modelled as their own record read alongside
// [`Entry`] rather than as three fields every cap would carry empty. Extra
// keys on an append-only JSONL line are forward-compatible by construction:
// every existing reader of that line still reads the entry it always read,
// and a letter filed by an older build simply has no notes to find.

/// D7's own three words, in D7's own order. Constants for the same reason
/// [`SECTIONS`] is: the headings ARE the decision.
pub(crate) const SECTION_ARCHITECTURE: &str = "Architecture";
pub(crate) const SECTION_BEHAVIOUR: &str = "Behaviour";
pub(crate) const SECTION_USAGE: &str = "Usage";
pub(crate) const CLOSE_SECTIONS: [&str; 3] =
    [SECTION_ARCHITECTURE, SECTION_BEHAVIOUR, SECTION_USAGE];

/// The keys the three lists ride under on a stored feature-close line, in the
/// same order as [`CLOSE_SECTIONS`].
const CLOSE_NOTE_KEYS: [&str; 3] = ["architecture", "behaviour", "usage"];

/// What a feature-close stop recorded for D14's three extra sections.
///
/// Every string in it is a fact some capped cell of the feature already
/// stored; see this section's header for why nothing here is written by the
/// composing pass.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CloseNote {
    pub architecture: Vec<String>,
    pub behaviour: Vec<String>,
    pub usage: Vec<String>,
}

impl CloseNote {
    pub(crate) fn is_empty(&self) -> bool {
        self.architecture.is_empty() && self.behaviour.is_empty() && self.usage.is_empty()
    }

    /// The three lists in [`CLOSE_SECTIONS`] order.
    fn lists(&self) -> [&Vec<String>; 3] {
        [&self.architecture, &self.behaviour, &self.usage]
    }

    /// `None` for a line that carries none of the three keys — which is every
    /// line every other stop kind ever wrote — and for one that carries them
    /// all empty, so an empty note can never keep a heading alive.
    fn from_value(v: &Value) -> Option<Self> {
        let m = v.as_object()?;
        if !CLOSE_NOTE_KEYS.iter().any(|key| m.contains_key(*key)) {
            return None;
        }
        let note = Self {
            architecture: string_list(m, CLOSE_NOTE_KEYS[0]),
            behaviour: string_list(m, CLOSE_NOTE_KEYS[1]),
            usage: string_list(m, CLOSE_NOTE_KEYS[2]),
        };
        (!note.is_empty()).then_some(note)
    }
}

/// Append the feature-close stop: ONE line carrying the [`Entry`] every stop
/// shares plus the three lists only this stop kind has. One O_APPEND write,
/// exactly like [`append_entry`] — a feature close that lands at 3am leaves
/// its record even if the run never reaches its own end (D4).
pub(crate) fn append_close_entry(
    root: &Path,
    run: &str,
    entry: &Entry,
    note: &CloseNote,
) -> std::io::Result<()> {
    let mut value = entry.to_value();
    if let Value::Object(map) = &mut value {
        for (key, list) in CLOSE_NOTE_KEYS.into_iter().zip(note.lists()) {
            map.insert(key.to_string(), json!(list));
        }
    }
    append_jsonl(&entries_path(root, run), &value)
}

/// [`record_stop`] for the feature-close stop — same FAIL-OPEN posture, same
/// one wording. The close has already happened; nothing about a letter may
/// turn it into a refusal (D10).
pub(crate) fn record_close_stop(root: &Path, run: &str, entry: &Entry, note: &CloseNote) {
    warn_if_unrecorded(run, append_close_entry(root, run, entry, note));
}

/// D8's plain-language sentence for a feature that just closed, written HERE
/// — at the moment of the stop, exactly like [`cap_sentence`], and for the
/// same reason: deferring it to composition would force the composer to
/// author, which D8 forbids.
///
/// It says only what the stop itself makes true — this feature's work is
/// finished — and names the feature the human asked for. It states nothing
/// about WHAT was built; that is what the sections below it are for.
pub(crate) fn close_sentence(feature: &str) -> String {
    match feature.trim() {
        "" => "Finished a piece of work.".to_string(),
        name => format!("Finished the work on {name}."),
    }
}

/// D14's three sections' bullets, in [`CLOSE_SECTIONS`] order.
///
/// Deduped in first-seen order across every feature-close stop the run
/// recorded (one run may close two features): dropping a repeat is DROPPING,
/// which D8 allows, and nothing is re-worded or added on the way. An entry
/// that is blank after `one_line` is dropped rather than rendered as an empty
/// bullet.
fn close_section_lines(notes: &[CloseNote]) -> [Vec<String>; 3] {
    let mut out: [Vec<String>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    for note in notes {
        for (slot, list) in out.iter_mut().zip(note.lists()) {
            for raw in list {
                if one_line(raw).is_empty() {
                    continue;
                }
                let line = bullet(raw);
                if !slot.contains(&line) {
                    slot.push(line);
                }
            }
        }
    }
    out
}

/// [`compose_body`] plus D14's three extra sections — the ONE body renderer a
/// feature-close letter uses.
///
/// The three come AFTER D7's five, which keeps those five contiguous and in
/// D7's own order: the news the human came for stays at the top of the file,
/// and the reference material about what the feature now is sits below it.
/// With no notes this is byte-identical to [`compose_body`], which is what
/// makes "a nightly letter never grows these sections" a property of the
/// material rather than of a second code path.
pub(crate) fn compose_body_with(entries: &[Entry], notes: &[CloseNote]) -> String {
    let mut out = compose_body(entries);
    for (heading, lines) in CLOSE_SECTIONS.into_iter().zip(close_section_lines(notes)) {
        push_section(&mut out, heading, &lines);
    }
    out
}

/// [`compose_letter`] for a run that closed a feature (D14).
///
/// The SAME record: only the body differs, and only by the sections the
/// stored notes carry material for. Subject, frontmatter, items and filename
/// are the nightly letter's, unchanged.
pub(crate) fn compose_letter_with(
    project: &str,
    run: &str,
    filed_at: &str,
    entries: &[Entry],
    notes: &[CloseNote],
) -> Option<Letter> {
    let mut letter = compose_letter(project, run, filed_at, entries)?;
    letter.body = compose_body_with(entries, notes);
    Some(letter)
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

// ─── the verb: `bee mailbox mark` (D6) ──────────────────────────────────
//
// Argv plumbing only. Every rule this command enforces lives in
// [`mark_letter`] above, where the store is — the door and the store can never
// disagree about what a flip is, because there is only one of them.
//
// Root topology: WORKTREE-NATIVE, and it stops there. The mailbox is a record
// of what a run did, a run happens in ONE checkout, and nothing coordinates
// across worktrees through it — so unlike `triggers`, this store never
// re-roots onto the control root.

/// The trimmed value of `--<name>`, or `None` when it is absent or empty.
fn mark_flag<'a>(parsed: &'a ParsedArgs, name: &str) -> Option<&'a str> {
    parsed.flags.get(name).map(|s| js_trim(s)).filter(|s| !s.is_empty())
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "mailbox" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    let rest = &args[2..];
    match verb {
        "mark" => run_mark(parse_shape(rest, &["id", "status"])?, t0),
        _ => None,
    }
}

fn run_mark(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let cmd = "mailbox mark";
    let Ok(cwd) = std::env::current_dir() else { return None };
    let root = match resolve_store_root_worktree(&cwd) {
        RootsWt::Go(r) => r.root,
        RootsWt::Unsupported(why) => {
            return Some(emit_unsupported_root(&cwd, cmd, parsed.pre_json, t0, &why))
        }
        RootsWt::None => return Some(emit_no_root_error(&cwd, cmd, parsed.pre_json, t0)),
    };
    let drift = check_manifest_drift(&root);
    let Some(id) = mark_flag(&parsed, "id") else {
        let msg = format!("bee {cmd}: --id is required — the letter's file name in .bee/human-mailbox.");
        return Some(emit_error(&root, cmd, parsed.json, &msg, t0));
    };
    let Some(status) = mark_flag(&parsed, "status") else {
        let msg = format!(
            "bee {cmd}: --status is required — {STATUS_UNREAD:?} or {STATUS_READ:?} (D6)."
        );
        return Some(emit_error(&root, cmd, parsed.json, &msg, t0));
    };
    match mark_letter(&root, id, status) {
        Err(why) => {
            let msg = format!("bee {cmd}: {}", why.message());
            Some(emit_error(&root, cmd, parsed.json, &msg, t0))
        }
        Ok(marked) => {
            // The answer always names the letter the ONE canonical way, so a
            // consumer that passed the bare stem still learns the file name.
            let name = marked
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .map(str::to_string)
                .unwrap_or_else(|| letter_id_to_filename(id));
            let result = json!({
                "letter": name,
                "path": marked.path.display().to_string(),
                "status": marked.status,
                "previous_status": marked.previous,
                "changed": marked.changed,
            });
            let text = if marked.changed {
                format!("Marked {name} {}.", marked.status)
            } else {
                format!("{name} was already {}. Nothing was written.", marked.status)
            };
            Some(emit_success(&root, cmd, parsed.json, &drift, &result, &text, t0))
        }
    }
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

    // ── D6: the read flip, the one command bee owes its consumer ────────

    /// The letter as bytes, minus the one line the flip is allowed to move.
    fn every_line_but_the_status(text: &str) -> Vec<String> {
        text.lines().filter(|l| !l.starts_with("status: ")).map(str::to_string).collect()
    }

    fn status_line(text: &str) -> String {
        text.lines().find(|l| l.starts_with("status: ")).expect("a letter carries a status").to_string()
    }

    #[test]
    fn a_flip_moves_the_status_line_and_nothing_else() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let path = write_letter(root, &sample_letter()).unwrap();
        let id = path.file_name().unwrap().to_str().unwrap().to_string();
        let before = std::fs::read_to_string(&path).unwrap();

        let marked = mark_letter(root, &id, STATUS_READ).unwrap();
        assert!(marked.changed);
        assert_eq!(marked.previous, STATUS_UNREAD);
        assert_eq!(marked.status, STATUS_READ);
        assert_eq!(marked.path, path, "the flip writes the file it read, never a twin");

        let after = std::fs::read_to_string(&path).unwrap();
        assert_eq!(status_line(&after), format!("status: {:?}", STATUS_READ));
        assert_eq!(
            every_line_but_the_status(&before),
            every_line_but_the_status(&after),
            "every other frontmatter field and the whole body survive byte for byte"
        );
        assert_eq!(list_letter_files(root).len(), 1, "one letter stays one letter");

        // Read back through the store: subject, items, needs_you and prose are
        // the record they were, and only the state bee owns has moved.
        let read_back = read_letter(&path).unwrap();
        let mut expected = sample_letter();
        expected.status = STATUS_READ.to_string();
        assert_eq!(read_back, expected);
    }

    #[test]
    fn flipping_back_restores_the_letter_byte_for_byte() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let path = write_letter(root, &sample_letter()).unwrap();
        let id = path.file_name().unwrap().to_str().unwrap().to_string();
        let original = std::fs::read_to_string(&path).unwrap();

        mark_letter(root, &id, STATUS_READ).unwrap();
        mark_letter(root, &id, STATUS_UNREAD).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    }

    #[test]
    fn flipping_an_already_flipped_letter_is_a_no_op_not_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let path = write_letter(root, &sample_letter()).unwrap();
        let id = path.file_name().unwrap().to_str().unwrap().to_string();

        mark_letter(root, &id, STATUS_READ).unwrap();
        let after_first = std::fs::read_to_string(&path).unwrap();

        // The consumer retries after a dropped response. It is not punished.
        let again = mark_letter(root, &id, STATUS_READ).expect("a retry is a success");
        assert!(!again.changed, "a second flip writes nothing");
        assert_eq!(again.previous, STATUS_READ);
        assert_eq!(again.status, STATUS_READ);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), after_first);
    }

    #[test]
    fn the_id_is_the_file_name_with_or_without_its_suffix() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let path = write_letter(root, &sample_letter()).unwrap();
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let stem = name.trim_end_matches(".md").to_string();

        let marked = mark_letter(root, &stem, STATUS_READ).unwrap();
        assert_eq!(marked.path, path, "the stem and the file name reach the same letter");
    }

    #[test]
    fn flipping_a_letter_that_is_not_there_refuses_and_names_it() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_letter(root, &sample_letter()).unwrap();

        let why = mark_letter(root, "20260101T000000Z-no-such-run.md", STATUS_READ).unwrap_err();
        assert!(matches!(why, MarkError::NoSuchLetter(_)));
        let message = why.message();
        assert!(message.contains("20260101T000000Z-no-such-run.md"), "{message}");
        assert!(message.contains("remedy"), "a refusal names what to fix: {message}");
    }

    #[test]
    fn a_status_outside_the_set_bee_owns_is_refused_before_the_disk() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let path = write_letter(root, &sample_letter()).unwrap();
        let id = path.file_name().unwrap().to_str().unwrap().to_string();
        let before = std::fs::read_to_string(&path).unwrap();

        let why = mark_letter(root, &id, "archived").unwrap_err();
        assert!(matches!(why, MarkError::UnknownStatus(_)));
        assert!(why.message().contains("archived"), "{}", why.message());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), before, "a refusal writes nothing");
    }

    #[test]
    fn a_letter_id_can_never_reach_out_of_the_mailbox() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        for id in ["../../etc/passwd", "sub/letter.md", "..", ""] {
            let why = mark_letter(root, id, STATUS_READ).unwrap_err();
            assert!(matches!(why, MarkError::NotALetterId(_)), "{id:?} must be refused");
        }
    }

    #[test]
    fn an_unreadable_file_is_reported_as_unreadable_not_as_a_missing_letter() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let path = write_letter(root, &sample_letter()).unwrap();
        let id = path.file_name().unwrap().to_str().unwrap().to_string();
        std::fs::write(&path, "someone deleted the fence\n").unwrap();

        let why = mark_letter(root, &id, STATUS_READ).unwrap_err();
        assert!(matches!(why, MarkError::Unreadable(_)));
        assert!(why.message().contains("remedy"), "{}", why.message());
    }

    // ── D9: arming, and D11's run identity ──────────────────────────────

    fn write_config(root: &Path, text: &str) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("config.json"), text).unwrap();
    }

    fn arm_the_loop(root: &Path) {
        std::fs::create_dir_all(root.join(".bee").join("tmp")).unwrap();
        std::fs::write(root.join(".bee").join("tmp").join("bee-herding.enable"), "").unwrap();
    }

    #[test]
    fn arming_needs_both_the_herding_block_and_the_owner_switch() {
        // Neither signal on its own arms the mailbox: a configured checkout
        // that nobody switched on is an ATTENDED session (D9 files no letter
        // for it), and an owner switch in a checkout with no herding block
        // has no unattended run to describe.
        for (config, switch, expected) in [
            (r#"{}"#, false, false),
            (r#"{"herding": {"agent_command": "claude-sonnet"}}"#, false, false),
            (r#"{}"#, true, false),
            (r#"{"herding": {}}"#, true, false),
            (r#"{"herding": {"agent_command": "claude-sonnet"}}"#, true, true),
        ] {
            let tmp = tempfile::tempdir().unwrap();
            let root = tmp.path();
            write_config(root, config);
            if switch {
                arm_the_loop(root);
            }
            assert_eq!(
                armed(root),
                expected,
                "config {config} with switch={switch} must read armed={expected}"
            );
        }
    }

    #[test]
    fn a_local_overlay_can_carry_the_herding_block() {
        // The merged config is what everything else in bee reads, so an
        // untracked `.bee/config.local.json` arms the mailbox exactly as the
        // tracked file does.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_config(root, r#"{}"#);
        std::fs::write(
            root.join(".bee").join("config.local.json"),
            r#"{"herding": {"agent_command": "claude-sonnet"}}"#,
        )
        .unwrap();
        arm_the_loop(root);
        assert!(armed(root), "the overlay's herding block counts");
    }

    #[test]
    fn arming_never_gates_the_append() {
        // D9: EVERY session appends its entries, attended or not — a session
        // that starts attended and becomes an overnight run must keep a
        // complete record of its whole span, so an unarmed checkout still
        // records everything.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_config(root, r#"{}"#);
        assert!(!armed(root));
        record_stop(root, "run-attended", &sample_entry("2026-08-25T01:00:00.000Z", "Did a thing"));
        let entries = read_entries(root, "run-attended");
        assert_eq!(entries.len(), 1, "an unarmed run records its entries too");
        assert_eq!(entries[0].what, "Did a thing");
    }

    #[test]
    fn a_failed_append_is_said_out_loud_and_never_thrown() {
        // The stop already happened, so the record can only ever warn (D10):
        // `entries` occupied by a plain file makes every append fail.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(mailbox_dir(root)).unwrap();
        std::fs::write(entries_dir(root), "not a directory").unwrap();
        record_stop(root, "run-blocked", &sample_entry("2026-08-25T01:00:00.000Z", "Did a thing"));
        assert!(read_entries(root, "run-blocked").is_empty());
    }

    #[test]
    fn a_run_with_no_session_still_has_a_name() {
        assert_eq!(run_id(Some("sess-9")), "sess-9");
        assert_eq!(run_id(Some("  sess-9  ")), "sess-9");
        assert_eq!(run_id(Some("   ")), UNATTRIBUTED_RUN);
        assert_eq!(run_id(None), UNATTRIBUTED_RUN);
    }

    // ── D8: the sentence is written at the moment of the event ──────────

    #[test]
    fn the_sentence_prefers_the_human_line_already_on_hand() {
        assert_eq!(
            cap_sentence(Some("Taught bee to write a note when it stops"), Some("t")),
            "Taught bee to write a note when it stops"
        );
        // Multi-line and padded outcomes still make ONE line (D2's shape).
        assert_eq!(cap_sentence(Some("\n  first line \nsecond"), None), "first line");
        // No outcome: the title, made into a sentence, punctuation kept.
        assert_eq!(
            cap_sentence(None, Some("the store that holds a letter")),
            "Finished the store that holds a letter."
        );
        assert_eq!(cap_sentence(Some("   "), Some("a job.")), "Finished a job.");
        // Nothing to go on: say only what is certainly true.
        assert_eq!(cap_sentence(None, None), "Finished a piece of work.");
    }

    // ── D7/D8/D9/D11: composing and filing the letter at run end ────────

    fn full_run() -> Vec<Entry> {
        let mut first = sample_entry("2026-08-25T01:00:00.000Z", "Taught the tool to leave you a note");
        first.departure = Some(Departure {
            what: "Kept one file per run instead of one file per event".to_string(),
            why: "A run that dies is findable by name that way".to_string(),
            kind: "found a better route".to_string(),
        });
        let second = sample_entry("2026-08-25T02:00:00.000Z", "Wired the note into every clean stop");
        let mut blocked = sample_entry("2026-08-25T03:00:00.000Z", "Stopped short of the rename");
        blocked.kind = KIND_BLOCKER.to_string();
        blocked.needs_you = vec![NeedsYou {
            id: "ny-7".to_string(),
            what: "Which name do you want for the folder?".to_string(),
            blocks: "the rename".to_string(),
        }];
        vec![first, second, blocked]
    }

    /// Lowercase word tokens — the unit "a fact this text states" is checked in.
    fn words(s: &str) -> Vec<String> {
        s.split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(str::to_lowercase)
            .collect()
    }

    #[test]
    fn composition_never_states_a_fact_no_entry_carries() {
        // D8's authorship ban, as a test that FAILS the moment the composing
        // pass writes prose of its own. Every word the body says must come
        // from a stored entry, from one of D7's five headings, or be the one
        // fixed connective the Needs-your-call bullet adds.
        let entries = full_run();
        let letter =
            compose_letter("beehive", "run-one", "2026-08-25T06:00:00.000Z", &entries).unwrap();

        let mut allowed: Vec<String> = Vec::new();
        for e in &entries {
            allowed.extend(words(&e.what));
            allowed.extend(words(&e.kind));
            allowed.extend(words(&e.at));
            for f in &e.files {
                allowed.extend(words(f));
            }
            if let Some(c) = &e.commit {
                allowed.extend(words(c));
            }
            if let Some(p) = &e.proof {
                allowed.extend(words(p));
            }
            if let Some(d) = &e.departure {
                allowed.extend(words(&d.what));
                allowed.extend(words(&d.why));
                allowed.extend(words(&d.kind));
            }
            for n in &e.needs_you {
                allowed.extend(words(&n.id));
                allowed.extend(words(&n.what));
                allowed.extend(words(&n.blocks));
            }
        }
        for heading in SECTIONS {
            allowed.extend(words(heading));
        }
        allowed.push("blocks".to_string());

        for word in words(&letter.body) {
            assert!(
                allowed.contains(&word),
                "the body says {word:?}, which no stored entry carries — that is authoring (D8)"
            );
        }
        // The subject is CHOSEN out of the stored sentences, never re-worded.
        assert!(
            entries.iter().any(|e| e.what == letter.subject),
            "the subject {:?} is not one of the stored sentences",
            letter.subject
        );
    }

    #[test]
    fn the_body_sorts_every_entry_into_one_of_the_sections_d7_names() {
        let entries = full_run();
        let body = compose_body(&entries);
        assert!(body.contains(&format!("## {SECTION_DONE}")));
        assert!(body.contains(&format!("## {SECTION_DEPARTED}")));
        assert!(body.contains(&format!("## {SECTION_BROKEN}")));
        assert!(body.contains(&format!("## {SECTION_NEEDS_YOU}")));
        // Done carries the two finished stops; the blocker is NOT among them.
        assert!(body.contains("- Taught the tool to leave you a note"));
        assert!(body.contains("- Wired the note into every clean stop"));
        // The blocker landed under "Broken or unfinished", not under Done.
        let broken_at = body.find(&format!("## {SECTION_BROKEN}")).unwrap();
        assert!(body.find("- Stopped short of the rename").unwrap() > broken_at);
        // D13: the stable id leads, and the item names what it blocks.
        assert!(body.contains("- [ny-7] Which name do you want for the folder? — blocks: the rename"));
    }

    #[test]
    fn a_departure_renders_through_the_one_deviation_renderer() {
        // knowledge::deviation_text is "the ONE rendering of a deviation
        // entry" — a second renderer here would be the defect. Its own
        // `{type, description}` arm reads "type: description".
        let entries = full_run();
        let body = compose_body(&entries);
        let d = entries[0].departure.clone().unwrap();
        let expected = deviation_text(&json!({
            "type": d.kind,
            "description": format!("{} — {}", d.what, d.why),
        }));
        assert!(body.contains(&format!("- {expected}")), "body was:\n{body}");
        assert!(expected.starts_with("found a better route: "));
    }

    #[test]
    fn a_section_with_nothing_to_report_is_absent_never_printed_empty() {
        // One plain cap: no departure, no blocker, no question, no next step.
        let body = compose_body(&[sample_entry("2026-08-25T01:00:00.000Z", "Did one thing")]);
        assert!(body.contains(&format!("## {SECTION_DONE}")));
        for heading in [SECTION_DEPARTED, SECTION_BROKEN, SECTION_NEEDS_YOU, SECTION_NEXT] {
            assert!(
                !body.contains(&format!("## {heading}")),
                "section {heading:?} has nothing to report and must be dropped (D7)\n{body}"
            );
        }
        assert!(!body.contains("\n\n\n"), "a dropped section leaves no hole");
    }

    #[test]
    fn the_next_section_is_dropped_rather_than_invented() {
        // No stored entry carries a next step, so printing this section would
        // mean authoring one — D8 forbids it, and D7 already says a section
        // with nothing to report is dropped.
        let body = compose_body(&full_run());
        assert!(!body.contains(&format!("## {SECTION_NEXT}")));
    }

    #[test]
    fn a_run_whose_sentences_cannot_be_a_subject_still_gets_a_valid_one() {
        // Every stored sentence here fails D2 — harness words, and a
        // multi-line one. The letter still needs a subject, and the last
        // resort says only what is true by the letter's own existence.
        let mut a = sample_entry("2026-08-25T01:00:00.000Z", "Capped cell hm-1 in the standard lane");
        a.needs_you = vec![];
        let b = sample_entry("2026-08-25T02:00:00.000Z", "two lines\nof outcome");
        let letter = compose_letter("beehive", "run-one", "2026-08-25T06:00:00.000Z", &[a, b]).unwrap();
        assert_eq!(letter.subject, FALLBACK_SUBJECT);
        assert!(check_subject(FALLBACK_SUBJECT).is_ok(), "the last resort must pass D2 itself");
        // The unusable sentences are still IN the letter — dropped as the
        // subject, never dropped as facts.
        assert!(letter.body.contains("Capped cell hm-1 in the standard lane"));
    }

    #[test]
    fn a_run_with_nothing_stored_composes_no_letter() {
        assert!(compose_letter("beehive", "run-one", "2026-08-25T06:00:00.000Z", &[]).is_none());
    }

    #[test]
    fn an_armed_run_files_exactly_one_letter_when_it_ends() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_config(root, r#"{"herding": {"agent_command": "claude-sonnet"}}"#);
        arm_the_loop(root);

        for entry in full_run() {
            append_entry(root, "run-one", &entry).unwrap();
        }
        let RunEnd::Filed(path) = file_run_letter(root, "run-one") else {
            panic!("an armed run that ends files its letter (D4, D9)");
        };

        // D11's name: timestamp-led, run-slugged, no subject in it.
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        assert!(name.ends_with("-run-one.md"), "{name}");
        assert_eq!(list_letter_files(root).len(), 1, "one run, one letter (D11)");

        // Every D3 field, with a non-empty subject.
        let letter = read_letter(&path).unwrap();
        assert!(!letter.subject.trim().is_empty());
        assert_eq!(letter.run, "run-one");
        assert!(!letter.project.trim().is_empty());
        assert!(!letter.filed_at.trim().is_empty());
        assert_eq!(letter.status, STATUS_UNREAD);
        assert_eq!(letter.items.len(), 3, "every stored entry is an item");
        assert_eq!(letter.needs_you.len(), 1);
        assert_eq!(letter.needs_you[0].id, "ny-7");
    }

    #[test]
    fn an_attended_run_records_its_entries_and_files_no_letter() {
        // D9: the herding block alone says the checkout CAN run unattended;
        // without the owner's switch this session is attended.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_config(root, r#"{"herding": {"agent_command": "claude-sonnet"}}"#);
        for entry in full_run() {
            append_entry(root, "run-one", &entry).unwrap();
        }
        assert!(matches!(file_run_letter(root, "run-one"), RunEnd::NotArmed));
        assert!(list_letter_files(root).is_empty());
        assert_eq!(read_entries(root, "run-one").len(), 3, "the entries are kept");
    }

    #[test]
    fn an_armed_run_that_stored_nothing_files_no_letter() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_config(root, r#"{"herding": {"agent_command": "claude-sonnet"}}"#);
        arm_the_loop(root);
        assert!(matches!(file_run_letter(root, "run-empty"), RunEnd::NoEntries));
        assert!(list_letter_files(root).is_empty());
    }

    #[test]
    fn two_runs_in_one_night_each_file_their_own_letter() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_config(root, r#"{"herding": {"agent_command": "claude-sonnet"}}"#);
        arm_the_loop(root);
        append_entry(root, "run-one", &sample_entry("2026-08-25T01:00:00.000Z", "The first run")).unwrap();
        append_entry(root, "run-two", &sample_entry("2026-08-25T05:00:00.000Z", "The second run")).unwrap();

        assert!(matches!(file_run_letter(root, "run-one"), RunEnd::Filed(_)));
        assert!(matches!(file_run_letter(root, "run-two"), RunEnd::Filed(_)));
        assert_eq!(list_letter_files(root).len(), 2, "one letter per run, never one per night (D11)");
        assert_eq!(letter_files_for_run(root, "run-one").len(), 1);
        assert_eq!(letter_files_for_run(root, "run-two").len(), 1);
    }

    #[test]
    fn a_run_that_ends_twice_keeps_one_letter_and_loses_no_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_config(root, r#"{"herding": {"agent_command": "claude-sonnet"}}"#);
        arm_the_loop(root);
        append_entry(root, "run-one", &sample_entry("2026-08-25T01:00:00.000Z", "The first thing")).unwrap();
        let RunEnd::Filed(first) = file_run_letter(root, "run-one") else { panic!("filed") };

        // The human reads it, then the same run works on and ends again.
        let mut read_back = read_letter(&first).unwrap();
        read_back.status = STATUS_READ.to_string();
        write_letter(root, &read_back).unwrap();
        append_entry(root, "run-one", &sample_entry("2026-08-25T04:00:00.000Z", "The second thing")).unwrap();

        let RunEnd::Filed(second) = file_run_letter(root, "run-one") else { panic!("re-filed") };
        assert_eq!(second, first, "one run keeps ONE letter, rewritten in place (D11)");
        assert_eq!(list_letter_files(root).len(), 1);
        let letter = read_letter(&second).unwrap();
        assert_eq!(letter.items.len(), 2, "the later entry is not lost");
        assert!(letter.body.contains("- The second thing"));
        assert_eq!(letter.status, STATUS_READ, "the human's read state survives the rewrite");
    }

    #[test]
    fn an_unreadable_existing_letter_stops_the_write_rather_than_doubling_it() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_config(root, r#"{"herding": {"agent_command": "claude-sonnet"}}"#);
        arm_the_loop(root);
        append_entry(root, "run-one", &sample_entry("2026-08-25T01:00:00.000Z", "The first thing")).unwrap();
        let RunEnd::Filed(path) = file_run_letter(root, "run-one") else { panic!("filed") };
        std::fs::write(&path, "someone deleted the fence\n").unwrap();

        let RunEnd::Failed(why) = record_run_end(root, "run-one") else {
            panic!("a second letter for one run is refused (D11)")
        };
        assert!(why.contains("remedy"), "a refusal names what to fix: {why}");
        assert_eq!(list_letter_files(root).len(), 1);
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
    // ── D5: the departure contract ──────────────────────────────────────

    #[test]
    fn a_departure_needs_all_three_parts_and_a_kind_from_the_closed_set() {
        let d = Departure::parse_line(
            "Used one file per run — a dead run's record stays findable by name — found a better route",
        )
        .expect("three parts, closed kind");
        assert_eq!(d.what, "Used one file per run");
        assert_eq!(d.why, "a dead run's record stays findable by name");
        assert_eq!(d.kind, "found a better route");

        // The WHY may carry the separator itself: what is everything before
        // the first, kind everything after the last.
        let d = Departure::parse_line("did x — because a — b — c — hit an unforeseen obstacle")
            .expect("the middle may hold separators");
        assert_eq!(d.what, "did x");
        assert_eq!(d.why, "because a — b — c");

        // The kind is compared loosely — case, spacing and a full stop are
        // typing, not meaning — and STORED canonically.
        assert_eq!(
            Departure::parse_line("did x — because y — The Plan Was  Wrong About A Fact.")
                .unwrap()
                .kind,
            "the plan was wrong about a fact"
        );

        for refused in [
            "just a free-form line",                       // no parts at all
            "did x — because y",                           // two parts only
            "did x — because y — because I felt like it",   // kind outside the four
            " — because y — found a better route",          // empty what
            "did x —  — found a better route",              // empty why
        ] {
            assert!(
                Departure::parse_line(refused).is_none(),
                "must not read as a departure: {refused}"
            );
        }

        // All four kinds are live, and nothing else is.
        for kind in DEPARTURE_KINDS {
            assert_eq!(Departure::canonical_kind(kind), Some(kind));
        }
        assert_eq!(Departure::canonical_kind("something else"), None);
    }

    #[test]
    fn a_cell_that_followed_its_plan_says_so_and_that_is_not_silence() {
        // D5: "Silence and nothing-happened must not read alike." The
        // statement is matched by its words, not by one exact spelling.
        for said in [
            "followed the plan",
            "Followed the plan.",
            "  Followed   the   plan  ",
            "followed the plan — nothing surprising came up",
        ] {
            assert!(Departure::plan_followed(said), "must count as said: {said}");
            assert_eq!(read_departure_line(said), DeparturePart::PlanFollowed);
        }
        assert!(!Departure::plan_followed(""), "silence is not a statement");
        assert!(!Departure::plan_followed("the plan was followed by nobody"));
    }

    #[test]
    fn a_recorded_entry_is_read_as_note_departure_or_malformed() {
        // A note stays a note — D5 narrowed what a DEPARTURE is, never what
        // may be written down.
        assert_eq!(read_departure(&json!("a free-form note")), DeparturePart::NotADeparture);
        // The shape the knowledge miner and every deviations-file use names
        // none of D5's parts, so it passes straight through.
        assert_eq!(
            read_departure(&json!({"type": "scope", "description": "why"})),
            DeparturePart::NotADeparture
        );
        assert_eq!(read_departure(&json!(7)), DeparturePart::NotADeparture);

        // Reaching for the parts and missing one is an ATTEMPT, and the
        // reading says which part is missing.
        let missing = read_departure(&json!({"what": "did x", "kind": "found a better route"}));
        match missing {
            DeparturePart::Malformed(problem) => assert!(problem.contains("why"), "{problem}"),
            other => panic!("expected a malformed reading, got {other:?}"),
        }
        let bad_kind =
            read_departure(&json!({"what": "did x", "why": "y", "kind": "felt like it"}));
        match bad_kind {
            DeparturePart::Malformed(problem) => {
                assert!(problem.contains("felt like it"), "{problem}")
            }
            other => panic!("expected a malformed reading, got {other:?}"),
        }

        // Both recorded shapes read as the same departure.
        let from_object = Departure::from_deviation(
            &json!({"what": "did x", "why": "y", "kind": "Found a better route"}),
        );
        let from_line = Departure::from_deviation(&json!("did x — y — found a better route"));
        assert_eq!(from_object, from_line);
        assert_eq!(from_object.unwrap().kind, "found a better route");

        // An already-filed letter is a record, never an input to validate:
        // the permissive reader keeps a departure an older build wrote.
        let filed = Departure::from_value(&json!({"what": "a", "why": "b", "kind": "an old kind"}))
            .expect("a filed letter keeps its departure");
        assert_eq!(filed.kind, "an old kind");
    }

    // ── D12: the run that went silent ───────────────────────────────────

    fn write_session(root: &Path, id: &str, status: &str, last_heartbeat: &str) {
        let dir = root.join(".bee").join("sessions");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{id}.json")),
            json!({ "id": id, "status": status, "last_heartbeat": last_heartbeat }).to_string(),
        )
        .unwrap();
    }

    /// A session record that says, positively, that the run is over.
    fn dead_session(root: &Path, id: &str) {
        write_session(root, id, "dead", "2026-08-25T03:00:00.000Z");
    }

    /// A session record that is beating right now — a run still working.
    fn live_session(root: &Path, id: &str) {
        write_session(root, id, "active", &now_iso());
    }

    fn armed_store(root: &Path) {
        write_config(root, r#"{"herding": {"agent_command": "claude-sonnet"}}"#);
        arm_the_loop(root);
    }

    #[test]
    fn a_run_that_went_silent_gets_a_marked_letter_naming_the_moment() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        armed_store(root);
        for entry in full_run() {
            append_entry(root, "sess-dead", &entry).unwrap();
        }
        dead_session(root, "sess-dead");
        live_session(root, "sess-now");

        let outcomes = file_letters_for_silent_runs(root, "sess-now");
        assert_eq!(outcomes.len(), 1, "one silent run, one decision");
        let RunEnd::Filed(path) = &outcomes[0].1 else {
            panic!("the silent run gets its letter from the next session (D12)");
        };
        let letter = read_letter(path).unwrap();

        // Marked plainly — the mark leads the inbox row, so a human tells this
        // letter from a clean one at a glance.
        assert!(
            letter.subject.starts_with(UNFINISHED_SUBJECT_MARK),
            "the subject {:?} does not mark the run unfinished",
            letter.subject
        );
        assert!(letter.body.contains(&format!("## {SECTION_UNFINISHED}")));
        // Lists the entries up to the last one.
        assert_eq!(letter.items.len(), full_run().len());
        // Names the moment it went silent — the last entry's own `at`.
        assert!(
            letter.body.contains("2026-08-25T03:00:00.000Z"),
            "the body never names the moment: {}",
            letter.body
        );
        assert_eq!(letter.status, STATUS_UNREAD);
        assert_eq!(letter.run, "sess-dead");
        // D7's sections still carry the run's work.
        assert!(letter.body.contains(&format!("## {SECTION_DONE}")));
    }

    #[test]
    fn an_unfinished_letter_states_only_facts_the_entries_carry() {
        // D8's authorship ban, over D12's own path. bee knows TWO things about
        // a silent run — it never reached its end, and when it last recorded
        // anything — and the body may say those two and nothing else.
        let entries = full_run();
        let body = compose_unfinished_body(&entries);

        let mut allowed: Vec<String> = Vec::new();
        for e in &entries {
            allowed.extend(words(&e.what));
            allowed.extend(words(&e.kind));
            allowed.extend(words(&e.at));
            for f in &e.files {
                allowed.extend(words(f));
            }
            if let Some(c) = &e.commit {
                allowed.extend(words(c));
            }
            if let Some(p) = &e.proof {
                allowed.extend(words(p));
            }
            if let Some(d) = &e.departure {
                allowed.extend(words(&d.what));
                allowed.extend(words(&d.why));
                allowed.extend(words(&d.kind));
            }
            for n in &e.needs_you {
                allowed.extend(words(&n.id));
                allowed.extend(words(&n.what));
                allowed.extend(words(&n.blocks));
            }
        }
        for heading in SECTIONS {
            allowed.extend(words(heading));
        }
        allowed.push("blocks".to_string());
        // The whole extra vocabulary D12 buys, named as constants so this test
        // measures the marking rather than being widened by it.
        allowed.extend(words(SECTION_UNFINISHED));
        allowed.extend(words(UNFINISHED_LINE));
        allowed.extend(words(SILENT_AFTER_PREFIX));

        for word in words(&body) {
            assert!(
                allowed.contains(&word),
                "the body says {word:?}, which no stored entry carries — that is authoring (D8)"
            );
        }
        // It never guesses WHY the run stopped.
        let lower = body.to_lowercase();
        for invented in ["crash", "killed", "failed", "died", "error"] {
            assert!(!lower.contains(invented), "the body guesses why: {invented:?}");
        }
    }

    #[test]
    fn a_run_still_working_is_never_swept() {
        // The hard case: a live run and a dead one look identical from the
        // outside — entries on disk, no letter. Only the heartbeat separates
        // them, and a live one must keep working undisturbed.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        armed_store(root);
        append_entry(root, "sess-live", &sample_entry("2026-08-25T01:00:00.000Z", "Still going"))
            .unwrap();
        live_session(root, "sess-live");
        live_session(root, "sess-now");

        assert!(file_letters_for_silent_runs(root, "sess-now").is_empty());
        assert!(list_letter_files(root).is_empty(), "a live run was given an unfinished letter");
    }

    #[test]
    fn an_ambiguous_or_missing_signal_files_nothing() {
        // FAIL CLOSED, the posture the claim sweep and `worktree prune` take:
        // no witness is "I cannot tell", never "it is dead". Each row here has
        // entries and no letter — the shape that WOULD be swept on evidence.
        let cases: Vec<(&str, Option<(&str, &str)>)> = vec![
            ("no session record at all", None),
            ("no heartbeat stamp", Some(("active", ""))),
            ("an unparseable heartbeat", Some(("active", "last tuesday"))),
            ("a fresh heartbeat", Some(("active", "NOW"))),
        ];
        for (why, record) in cases {
            let tmp = tempfile::tempdir().unwrap();
            let root = tmp.path();
            armed_store(root);
            append_entry(root, "sess-x", &sample_entry("2026-08-25T01:00:00.000Z", "Did a thing"))
                .unwrap();
            if let Some((status, heartbeat)) = record {
                let stamp = if heartbeat == "NOW" { now_iso() } else { heartbeat.to_string() };
                write_session(root, "sess-x", status, &stamp);
            }
            assert!(
                file_letters_for_silent_runs(root, "sess-now").is_empty(),
                "{why}: a run was swept on no evidence"
            );
            assert!(list_letter_files(root).is_empty(), "{why}: a letter was filed anyway");
        }
    }

    #[test]
    fn the_unattributed_bucket_is_never_swept() {
        // Nothing ever writes a session record named "unattributed", so there
        // is no witness that its run is over — and many runs share that bucket.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        armed_store(root);
        append_entry(root, UNATTRIBUTED_RUN, &sample_entry("2026-08-25T01:00:00.000Z", "A thing"))
            .unwrap();
        assert!(file_letters_for_silent_runs(root, "sess-now").is_empty());
    }

    #[test]
    fn this_session_never_files_an_unfinished_letter_for_itself() {
        // Belt-and-braces: a session whose own heartbeat is not stamped yet is
        // still not a silent run.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        armed_store(root);
        append_entry(root, "sess-now", &sample_entry("2026-08-25T01:00:00.000Z", "Working now"))
            .unwrap();
        write_session(root, "sess-now", "dead", "2020-01-01T00:00:00.000Z");
        assert!(file_letters_for_silent_runs(root, "sess-now").is_empty());
        assert!(list_letter_files(root).is_empty());
    }

    #[test]
    fn a_silent_run_that_already_has_its_letter_is_not_filed_twice() {
        // D11: one letter maps to one run.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        armed_store(root);
        for entry in full_run() {
            append_entry(root, "sess-dead", &entry).unwrap();
        }
        dead_session(root, "sess-dead");

        let first = file_letters_for_silent_runs(root, "sess-now");
        assert!(matches!(first[0].1, RunEnd::Filed(_)));
        assert_eq!(list_letter_files(root).len(), 1);

        // Second session start: the run has a letter, so it is not even a
        // candidate any more.
        assert!(silent_run_candidates(root, "sess-later").is_empty());
        assert!(file_letters_for_silent_runs(root, "sess-later").is_empty());
        assert_eq!(list_letter_files(root).len(), 1, "one run, one letter (D11)");
    }

    #[test]
    fn an_unarmed_checkout_files_nothing_and_lists_no_directory() {
        // D9, and step 0 of the bounded read: an attended checkout stops at the
        // config read.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_config(root, r#"{"herding": {"agent_command": "claude-sonnet"}}"#);
        append_entry(root, "sess-dead", &sample_entry("2026-08-25T01:00:00.000Z", "A thing"))
            .unwrap();
        dead_session(root, "sess-dead");
        assert!(file_letters_for_silent_runs(root, "sess-now").is_empty());
        assert!(list_letter_files(root).is_empty());
    }

    #[test]
    fn detection_opens_no_entry_file_however_many_runs_the_store_holds() {
        // The MEDIUM risk plan.md flags on this cell: this runs on a path that
        // has nothing to do with the mailbox, so the cost must not grow with
        // the store. The detection is three directory listings; the number that
        // matters is FILE OPENS, and it is zero.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        armed_store(root);
        for n in 0..60 {
            let run = format!("sess-{n:03}");
            for entry in full_run() {
                append_entry(root, &run, &entry).unwrap();
            }
            live_session(root, &run);
        }
        live_session(root, "sess-now");

        ENTRY_READS.with(|c| c.set(0));
        let candidates = silent_run_candidates(root, "sess-now");
        assert_eq!(candidates.len(), 60, "every run with entries and no letter is a candidate");
        assert_eq!(
            ENTRY_READS.with(|c| c.get()),
            0,
            "the detection opened an entry file — it must answer from directory names alone"
        );

        // And the whole pass over 60 live runs still opens nothing.
        ENTRY_READS.with(|c| c.set(0));
        assert!(file_letters_for_silent_runs(root, "sess-now").is_empty());
        assert_eq!(ENTRY_READS.with(|c| c.get()), 0, "a live run's entries were read");
    }

    #[test]
    fn only_the_run_that_is_actually_filed_has_its_entries_read() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        armed_store(root);
        for n in 0..40 {
            let run = format!("sess-{n:03}");
            append_entry(root, &run, &sample_entry("2026-08-25T01:00:00.000Z", "Still going"))
                .unwrap();
            live_session(root, &run);
        }
        for entry in full_run() {
            append_entry(root, "sess-dead", &entry).unwrap();
        }
        dead_session(root, "sess-dead");

        ENTRY_READS.with(|c| c.set(0));
        let outcomes = file_letters_for_silent_runs(root, "sess-now");
        assert_eq!(outcomes.len(), 1);
        assert_eq!(
            ENTRY_READS.with(|c| c.get()),
            1,
            "exactly one entry file is opened: the one letter that gets written"
        );
    }

    // ── D14: the feature-close letter ───────────────────────────────────

    fn close_note() -> CloseNote {
        CloseNote {
            architecture: vec![
                "packages/bee-rs/crates/bee/src/verbs/mailbox.rs".to_string(),
                "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs".to_string(),
            ],
            behaviour: vec![
                "A letter is filed once per run and never twice".to_string(),
                "A section with nothing to report is dropped".to_string(),
            ],
            usage: vec!["skills/bee-hive/SKILL.md".to_string()],
        }
    }

    fn close_entry() -> Entry {
        Entry {
            at: "2026-08-25T04:00:00.000Z".to_string(),
            kind: KIND_FEATURE_CLOSE.to_string(),
            what: close_sentence("human-mailbox"),
            files: vec![],
            commit: None,
            proof: None,
            departure: None,
            needs_you: vec![],
        }
    }

    #[test]
    fn the_feature_close_letter_carries_architecture_behaviour_and_usage() {
        // D7 promised these three appear in the feature-close letter. Until
        // this cell they appeared nowhere, so the promise described nothing.
        let mut entries = full_run();
        entries.push(close_entry());
        let letter = compose_letter_with(
            "beehive",
            "run-close",
            "2026-08-25T06:00:00.000Z",
            &entries,
            &[close_note()],
        )
        .unwrap();

        for heading in CLOSE_SECTIONS {
            assert!(
                letter.body.contains(&format!("## {heading}")),
                "the feature-close letter is missing the {heading:?} section:\n{}",
                letter.body
            );
        }
        // The stored material, verbatim.
        assert!(letter.body.contains("- packages/bee-rs/crates/bee/src/verbs/mailbox.rs"));
        assert!(letter.body.contains("- A letter is filed once per run and never twice"));
        assert!(letter.body.contains("- skills/bee-hive/SKILL.md"));

        // D7's five stay contiguous, in D7's order, ABOVE the three: the news
        // the human came for is still at the top of the file.
        let at = |heading: &str| letter.body.find(&format!("## {heading}")).unwrap();
        assert!(at(SECTION_DONE) < at(SECTION_DEPARTED));
        assert!(at(SECTION_NEEDS_YOU) < at(SECTION_ARCHITECTURE));
        assert!(at(SECTION_ARCHITECTURE) < at(SECTION_BEHAVIOUR));
        assert!(at(SECTION_BEHAVIOUR) < at(SECTION_USAGE));

        // SAME record shape: D3's frontmatter grew nothing, and D11's
        // filename rule is untouched.
        assert_eq!(letter.filename(), letter_filename(&letter.filed_at, &letter.run));
        assert_eq!(letter.items.len(), entries.len());
        assert_eq!(letter.status, STATUS_UNREAD);
    }

    #[test]
    fn a_nightly_letter_never_grows_the_extra_sections() {
        // The other direction of D7's promise: "only in the feature-close
        // letter". A nightly run stores no note, so there is no material, so
        // the three headings cannot appear.
        let entries = full_run();
        let letter =
            compose_letter("beehive", "run-night", "2026-08-25T06:00:00.000Z", &entries).unwrap();
        for heading in CLOSE_SECTIONS {
            assert!(
                !letter.body.contains(&format!("## {heading}")),
                "a nightly letter grew the {heading:?} section:\n{}",
                letter.body
            );
        }
        // With no notes the body is byte-identical to the five-section one —
        // the promise is a property of the material, not of a second path.
        assert_eq!(compose_body_with(&entries, &[]), compose_body(&entries));
    }

    #[test]
    fn an_extra_section_with_nothing_to_report_is_dropped_never_printed_empty() {
        // D7's own dropping rule, reused rather than re-implemented: a close
        // that recorded only architecture prints only Architecture.
        let entries = vec![close_entry()];
        let note = CloseNote { architecture: vec!["src/one.rs".to_string()], ..Default::default() };
        let body = compose_body_with(&entries, &[note]);
        assert!(body.contains(&format!("## {SECTION_ARCHITECTURE}")));
        assert!(!body.contains(&format!("## {SECTION_BEHAVIOUR}")));
        assert!(!body.contains(&format!("## {SECTION_USAGE}")));

        // An all-blank note keeps no heading alive at all.
        let blank = CloseNote {
            architecture: vec!["   ".to_string()],
            behaviour: vec![String::new()],
            usage: vec![],
        };
        let body = compose_body_with(&entries, &[blank]);
        for heading in CLOSE_SECTIONS {
            assert!(!body.contains(&format!("## {heading}")), "an empty section was printed");
        }
    }

    #[test]
    fn the_feature_close_letter_states_no_fact_no_entry_carries() {
        // D8's authorship ban, over D14's own path — the mirror of
        // `composition_never_states_a_fact_no_entry_carries` above. Every word
        // of the three extra sections must come from the STORED note; the
        // composing pass may sort, dedupe and drop, and never author.
        let entries = vec![close_entry()];
        let note = close_note();
        let body = compose_body_with(&entries, &[note.clone()]);

        let mut allowed: Vec<String> = Vec::new();
        for e in &entries {
            allowed.extend(words(&e.what));
            allowed.extend(words(&e.kind));
            allowed.extend(words(&e.at));
        }
        for list in note.lists() {
            for line in list {
                allowed.extend(words(line));
            }
        }
        for heading in SECTIONS.into_iter().chain(CLOSE_SECTIONS) {
            allowed.extend(words(heading));
        }

        for word in words(&body) {
            assert!(
                allowed.contains(&word),
                "the body says {word:?}, which no stored entry carries — that is authoring (D8)"
            );
        }
    }

    #[test]
    fn a_feature_close_stop_rides_one_line_that_every_older_reader_still_reads() {
        // The three lists are kind-specific payload on the stop's OWN line —
        // one append, one line, and `read_entries` reads the entry it always
        // read without a warning.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        append_entry(root, "run-a", &sample_entry("2026-08-25T01:00:00.000Z", "Did a thing"))
            .unwrap();
        append_close_entry(root, "run-a", &close_entry(), &close_note()).unwrap();

        let (entries, notes) = read_run(root, "run-a");
        assert_eq!(entries.len(), 2, "both stops read back as ordinary entries");
        assert_eq!(entries[1].kind, KIND_FEATURE_CLOSE);
        assert_eq!(notes, vec![close_note()], "the note rides the same line, verbatim");
        // The cap's own line carries no note, so it contributes no section.
        assert_eq!(read_entries(root, "run-a").len(), 2);
    }

    #[test]
    fn the_run_that_closed_a_feature_files_the_letter_that_carries_the_sections() {
        // End to end, and BOTH directions in one store: two runs, one that
        // closed a feature and one that did not. Same file shape, same name
        // rule, same one letter per run (D3, D11) — only the sections differ.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        armed_store(root);

        append_entry(root, "run-night", &sample_entry("2026-08-25T02:00:00.000Z", "Kept going"))
            .unwrap();
        append_entry(root, "run-close", &sample_entry("2026-08-25T03:00:00.000Z", "Kept going"))
            .unwrap();
        append_close_entry(root, "run-close", &close_entry(), &close_note()).unwrap();

        let RunEnd::Filed(closed) = file_run_letter(root, "run-close") else {
            panic!("the run that closed a feature files its letter");
        };
        let RunEnd::Filed(nightly) = file_run_letter(root, "run-night") else {
            panic!("the nightly run files its letter");
        };
        assert_eq!(letter_files_for_run(root, "run-close").len(), 1, "one letter, one run (D11)");

        let closed = read_letter(&closed).unwrap();
        let nightly = read_letter(&nightly).unwrap();
        for heading in CLOSE_SECTIONS {
            assert!(closed.body.contains(&format!("## {heading}")), "{}", closed.body);
            assert!(!nightly.body.contains(&format!("## {heading}")), "{}", nightly.body);
        }
        // Both are the same record, by the same rules: D3's frontmatter field
        // list is unchanged and D11 still names both files the same way.
        assert_eq!(closed.status, STATUS_UNREAD);
        assert_eq!(nightly.status, STATUS_UNREAD);
        assert!(closed.filename().ends_with(&format!("-{}.md", short_run_slug("run-close"))));
        assert!(nightly.filename().ends_with(&format!("-{}.md", short_run_slug("run-night"))));
        assert_eq!(list_letter_files(root).len(), 2, "one letter per run, no extra file");
    }
}
