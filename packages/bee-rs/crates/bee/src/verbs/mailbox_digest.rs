// verbs/mailbox_digest — the daily and weekly digest
// (docs/history/letter-digest/CONTEXT.md).
//
// The mailbox is a directory of files a human reads in place (D1). A digest is
// one more file in that same directory: it folds ONE finished period — one UTC
// day, or one ISO week — out of that period's close letters and the
// `.bee/usage/<feature>.json` records closed inside it.
//
//   .bee/human-mailbox/
//     <UTC-timestamp>-<short-run-slug>.md   ← a letter, one per run (D11)
//     digest-2026-08-25.md                  ← one finished UTC day
//     digest-2026-W35.md                    ← one finished ISO week
//
// ── WHO COMPOSES IT, AND WHEN (D3) ──────────────────────────────────────
//
// Nobody schedules this. The next session that starts after a period has ended
// finds the digest missing and composes it — the same recover-on-next-session
// pattern D12 already uses for the run that died at 3am. That is why this
// module owns no verb, no CLI surface and no viewer (D1): it is library code
// with one door, [`compose_due_digests`], which a session-start path calls
// fail-open.
//
// ── DETECTION IS BY NAME, NOTHING IS OPENED TO DECIDE (D3, D12) ─────────
//
// A letter's filename is UTC-stamp-led, so the day a letter belongs to is
// readable off the directory listing. A period is DUE when three things are
// true of the names alone:
//
//   1. it has fully ended in UTC (its last day is before today);
//   2. at least one letter carries a stamp inside it;
//   3. no `digest-<period>.md` file exists yet.
//
// Rule 3 is the whole idempotence story, and it is why this pass NEVER rewrites
// or reopens a digest: the file's existence IS the "already composed" marker
// (CONTEXT.md's Agent's Discretion, constrained to directory/file evidence).
// Once a period is closed it stays closed even if a letter for it arrives late
// — a D12-recovered letter digests under its own `filed_at` day, and if that
// day already has a digest, the letter is simply not in it. A human reading a
// digest must be able to trust the bytes they read yesterday.
//
// ── A RENDERER, NEVER A SUMMARIZER (human-mailbox D8) ───────────────────
//
// The digest may reorder, group, and DROP. It may never state a fact no letter
// and no stored usage field carries. Concretely, and on purpose:
//
//   * letter subjects and their Done / Broken or unfinished / Needs your call
//     bullets are transcribed VERBATIM, exactly as the letter filed them;
//   * usage numbers are the stored fields of the record, printed under the
//     field names the record itself uses;
//   * there are NO counts, NO sums, NO "a busy week" lines. "3 letters" is a
//     number bee computed here, not a number any letter carries — and a reader
//     cannot tell the two apart once they are on the same page.
//
// The only words this pass adds of its own are structural: the two section
// headings and the bullet labels that say which letter section a line came
// from.

#![allow(dead_code)] // The composer lands first (ld-2); the session-start hook
// that calls it — and the weekly lesson mining that reads its fold — is ld-3.

use super::mailbox::{
    emit_scalar, emit_string_list, list_letter_files, mailbox_dir, read_letter,
    Letter, DIGEST_PREFIX, SECTION_BROKEN, SECTION_DONE, SECTION_NEEDS_YOU,
};
use crate::fsutil::write_text_atomic;
use chrono::{Datelike, Days, NaiveDate, Weekday};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// ─── the period ─────────────────────────────────────────────────────────

/// The two periods a digest folds. There is no third: CONTEXT.md's D3 names
/// the daily and the weekly fold, and a period bee cannot name in a filename
/// is a period it cannot detect as already-composed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PeriodKind {
    Day,
    Week,
}

impl PeriodKind {
    /// Daily before weekly when two periods END on the same date, so the
    /// composing order is total and stable rather than map-order.
    fn rank(self) -> u8 {
        match self {
            Self::Day => 0,
            Self::Week => 1,
        }
    }
}

/// One finished stretch of UTC time, with the two dates that bound it.
///
/// `end` is INCLUSIVE — the last day inside the period — because that is the
/// date a human means by "the week ended on Sunday", and an exclusive bound
/// invites the off-by-one that would fold Monday's letters into last week.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Period {
    pub kind: PeriodKind,
    /// `2026-08-25` for a day, `2026-W35` for an ISO week. It is the filename
    /// tail AND the human-readable name of the period — one spelling, so a
    /// person reading the directory and a program checking existence agree.
    pub id: String,
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl Period {
    pub(crate) fn day(date: NaiveDate) -> Self {
        Self {
            kind: PeriodKind::Day,
            id: format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day()),
            start: date,
            end: date,
        }
    }

    /// The ISO week that contains `date` — Monday through Sunday, numbered by
    /// the ISO calendar, so a week that straddles New Year keeps ONE number
    /// and one digest instead of splitting into two half weeks.
    pub(crate) fn week(date: NaiveDate) -> Self {
        let iso = date.iso_week();
        let start =
            NaiveDate::from_isoywd_opt(iso.year(), iso.week(), Weekday::Mon).unwrap_or(date);
        let end = start.checked_add_days(Days::new(6)).unwrap_or(start);
        Self { kind: PeriodKind::Week, id: format!("{}-W{:02}", iso.year(), iso.week()), start, end }
    }

    /// `digest-<id>.md`, in the mailbox directory beside the letters (D1).
    pub(crate) fn filename(&self) -> String {
        format!("{DIGEST_PREFIX}{}.md", self.id)
    }

    fn contains(&self, date: NaiveDate) -> bool {
        self.start <= date && date <= self.end
    }

    /// Has the whole period gone by, in UTC? A period is folded only once its
    /// last day is BEHIND today — a digest of a day still running would be a
    /// half record that nothing ever completes (rule 3 forbids reopening it).
    fn has_ended(&self, today: NaiveDate) -> bool {
        self.end < today
    }

    /// D2's inbox row for a digest: one plain sentence naming the period, in
    /// words a person who never heard of this harness can read. It states
    /// nothing about WHAT happened — that would be authoring (D8); it names
    /// the stretch of time whose letters are below it.
    fn subject(&self) -> String {
        match self.kind {
            PeriodKind::Day => format!("What happened on {}.", self.id),
            PeriodKind::Week => {
                format!("What happened in the week of {} through {}.", self.start, self.end)
            }
        }
    }
}

// ─── reading the period off a filename ──────────────────────────────────

/// The UTC day a letter's filename stamps, from the NAME alone.
///
/// The name is `<YYYYMMDD>T<HHMMSS>Z-<short-run-slug>.md`, so the first eight
/// characters are the day. `None` for anything else — a digest, a stray
/// markdown file, or a stamp that will not parse as a date. Nothing is opened:
/// this is what makes the whole detection pass one directory read.
pub(crate) fn letter_day_from_name(name: &str) -> Option<NaiveDate> {
    let stem = name.strip_suffix(".md")?;
    if stem.starts_with(DIGEST_PREFIX) {
        return None;
    }
    let stamp = stem.split('-').next()?;
    if stamp.len() < 8 || !stamp[..8].bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let year: i32 = stamp[..4].parse().ok()?;
    let month: u32 = stamp[4..6].parse().ok()?;
    let day: u32 = stamp[6..8].parse().ok()?;
    NaiveDate::from_ymd_opt(year, month, day)
}

/// The UTC date an ISO-8601 stamp falls on: its first ten characters.
/// Tolerant of a fractional part, an offset spelling, or a bare date.
pub(crate) fn utc_date(iso: &str) -> Option<NaiveDate> {
    let s = iso.trim();
    if s.len() < 10 {
        return None;
    }
    NaiveDate::parse_from_str(&s[..10], "%Y-%m-%d").ok()
}

// ─── the usage record (D3's second input) ───────────────────────────────

fn usage_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("usage")
}

/// One `.bee/usage/<feature>.json` record, dated into a period by its stored
/// `closed_at`.
#[derive(Clone, Debug)]
pub(crate) struct UsageRecord {
    pub feature: String,
    pub closed_on: NaiveDate,
    pub value: Value,
}

/// Every readable usage record under `root`, feature-name sorted.
///
/// Fail-open, exactly like the entry read: a record that will not parse, or
/// one with no usable `closed_at`, is skipped with a visible warning rather
/// than sinking the digest. A missing directory is simply no records.
pub(crate) fn read_usage_records(root: &Path) -> Vec<UsageRecord> {
    let dir = usage_dir(root);
    let mut names: Vec<String> = match std::fs::read_dir(&dir) {
        Ok(entries) => entries
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                name.ends_with(".json").then_some(name)
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    names.sort();
    let mut out = Vec::new();
    for name in names {
        let path = dir.join(&name);
        let Ok(text) = std::fs::read_to_string(&path) else {
            eprintln!("bee: could not read {} — that record is not in the digest.", path.display());
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&text) else {
            eprintln!(
                "bee: could not parse {} — remedy: fix or delete that file; it is not in the digest.",
                path.display()
            );
            continue;
        };
        let Some(closed_on) = value.get("closed_at").and_then(Value::as_str).and_then(utc_date)
        else {
            eprintln!(
                "bee: {} carries no readable closed_at — it cannot be dated into a digest.",
                path.display()
            );
            continue;
        };
        let feature = value
            .get("feature")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| name.trim_end_matches(".json").to_string());
        out.push(UsageRecord { feature, closed_on, value });
    }
    out
}

/// The stored top-level fields a digest transcribes, in this order.
///
/// `sessions` is DROPPED — a per-session dump is longer than the letters it
/// sits under, and dropping is exactly what D8 leaves a renderer free to do.
/// `schema` is dropped for the same reason: it is a machine contract marker,
/// not something that happened. Everything printed is a stored field under the
/// record's OWN field name, so a reader can go find it.
const USAGE_FIELDS: [&str; 2] = ["closed_at", "totals"];

/// One scalar, as the record spells it. `None` for anything not a scalar, so a
/// nested object is walked rather than printed as JSON at a human.
fn scalar_text(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Transcribe every scalar leaf under `value` as `- <field path>: <value>`.
/// The path is built out of the record's own key names, so nothing here is a
/// number this pass computed.
fn transcribe(field: &str, value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, inner) in map {
                transcribe(&format!("{field}.{key}"), inner, out);
            }
        }
        other => {
            if let Some(text) = scalar_text(other) {
                out.push(format!("- {field}: {text}"));
            }
        }
    }
}

// ─── reading a letter's own sections back ───────────────────────────────

/// A letter body's `## <heading>` blocks, in file order, each with its lines
/// kept VERBATIM (blank lines dropped).
///
/// The letter is composed by `mailbox::compose_body`, which writes exactly
/// this shape; reading it back this way means the digest transcribes the very
/// bytes the human would have read in the letter, with no second opinion about
/// what a bullet means.
pub(crate) fn body_sections(body: &str) -> Vec<(String, Vec<String>)> {
    let mut out: Vec<(String, Vec<String>)> = Vec::new();
    for line in body.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            out.push((heading.trim().to_string(), Vec::new()));
            continue;
        }
        let text = line.trim();
        if text.is_empty() {
            continue;
        }
        if let Some(last) = out.last_mut() {
            last.1.push(text.to_string());
        }
    }
    out
}

/// The lines a letter filed under one heading, or empty when it has none.
pub(crate) fn section_lines(body: &str, heading: &str) -> Vec<String> {
    body_sections(body)
        .into_iter()
        .find(|(h, _)| h == heading)
        .map(|(_, lines)| lines)
        .unwrap_or_default()
}

/// The three letter sections a digest carries forward, in the letter's own
/// order (human-mailbox D7). `Where I departed from the plan and why` and
/// `Next` are dropped: the first is a per-run detail the letter itself keeps,
/// the second is never written today.
const FOLDED_SECTIONS: [&str; 3] = [SECTION_DONE, SECTION_BROKEN, SECTION_NEEDS_YOU];

// ─── composing one digest ───────────────────────────────────────────────

/// What one composed digest folded — the path it was written to, the period it
/// covers, and the letters that went into it.
///
/// The letter paths ride along because the weekly fold's next reader (the
/// lesson mining of D4) needs exactly this set, and re-deriving it would give
/// two passes two chances to disagree about which letters a week held.
#[derive(Clone, Debug)]
pub(crate) struct DigestWritten {
    pub path: PathBuf,
    pub period: Period,
    pub letters: Vec<PathBuf>,
}

/// The bytes of one digest: `---`, frontmatter, `---`, blank line, prose.
/// Deterministic, and spelled with the letter's own YAML emitters so the two
/// files in this directory can never drift into two dialects.
pub(crate) fn render_digest(
    period: &Period,
    filed_at: &str,
    letters: &[(PathBuf, Letter)],
    unreadable: &[String],
    usage: &[&UsageRecord],
) -> String {
    let mut out = String::from("---\n");
    emit_scalar(&mut out, 0, "subject", &period.subject());
    // The one field that tells a consumer this file is not a letter. It is
    // here rather than implied by the filename because the filename is a
    // convention and the frontmatter is the contract (human-mailbox D3).
    emit_scalar(&mut out, 0, "type", "digest");
    emit_scalar(
        &mut out,
        0,
        "period",
        match period.kind {
            PeriodKind::Day => "day",
            PeriodKind::Week => "week",
        },
    );
    emit_scalar(&mut out, 0, "period_id", &period.id);
    emit_scalar(&mut out, 0, "filed_at", filed_at);
    // Which files this digest was folded from — names, so a reader can open
    // any of them, and so a later pass citing a letter cites the same name.
    let names: Vec<String> = letters
        .iter()
        .map(|(p, _)| p.file_name().unwrap_or_default().to_string_lossy().into_owned())
        .collect();
    emit_string_list(&mut out, 0, "letters", &names);
    // The letters this period held that could NOT be read. A missing file and
    // an unreadable one must never read alike: this list says "we looked and
    // could not read these", which is a fact about the period, while silence
    // would let a torn letter vanish from the record with only a stderr line
    // nobody kept. It is the same honesty `write_usage_record` follows with
    // its own `skipped` field.
    emit_string_list(&mut out, 0, "unreadable", unreadable);
    out.push_str("---\n");

    let body = digest_body(letters, usage);
    if !body.trim().is_empty() {
        out.push('\n');
        out.push_str(body.trim());
        out.push('\n');
    }
    out
}

/// The prose: the period's letters grouped by project, then the usage records
/// closed inside it. A section with nothing to report is dropped, the way a
/// letter drops one (D7).
fn digest_body(letters: &[(PathBuf, Letter)], usage: &[&UsageRecord]) -> String {
    let mut out = String::new();

    // Grouped by the letter's own `project` field — grouping is reordering,
    // which is what a renderer is allowed to do. The map is ordered, so two
    // runs of this pass over the same letters render the same bytes.
    let mut by_project: BTreeMap<String, Vec<&Letter>> = BTreeMap::new();
    for (_, letter) in letters {
        by_project.entry(letter.project.clone()).or_default().push(letter);
    }
    if !by_project.is_empty() {
        out.push_str("## Letters\n");
        for (project, group) in &by_project {
            out.push_str(&format!("\n### {project}\n"));
            for letter in group {
                out.push_str(&format!("\n#### {}\n", letter.subject.trim()));
                for heading in FOLDED_SECTIONS {
                    let lines = section_lines(&letter.body, heading);
                    if lines.is_empty() {
                        continue;
                    }
                    out.push_str(&format!("\n**{heading}**\n\n"));
                    for line in lines {
                        out.push_str(&line);
                        out.push('\n');
                    }
                }
            }
        }
    }

    let mut usage_out = String::new();
    for record in usage {
        let mut lines: Vec<String> = Vec::new();
        for field in USAGE_FIELDS {
            if let Some(value) = record.value.get(field) {
                transcribe(field, value, &mut lines);
            }
        }
        if lines.is_empty() {
            continue;
        }
        usage_out.push_str(&format!("\n### {}\n\n", record.feature));
        for line in lines {
            usage_out.push_str(&line);
            usage_out.push('\n');
        }
    }
    if !usage_out.is_empty() {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str("## Token usage\n");
        out.push_str(&usage_out);
    }

    out
}

// ─── the one door (D3) ──────────────────────────────────────────────────

/// Compose every digest that is due at `now`, oldest period first, and answer
/// what was written.
///
/// `now` is an ISO-8601 stamp rather than a clock read inside, so the caller —
/// and a test — pins the one input that decides which periods have ended.
///
/// Fail-open throughout: an unreadable letter is skipped with a warning and
/// the digest still composes from the rest; a write that fails names itself
/// and the pass carries on to the next period. This runs beside somebody
/// else's work (a session start), and it must never turn their ask into a
/// refusal.
///
/// Idempotent by file existence: a period whose digest is already on disk is
/// skipped without opening it. Nothing here ever rewrites a digest.
pub(crate) fn compose_due_digests(root: &Path, now: &str) -> Vec<DigestWritten> {
    let Some(today) = utc_date(now) else {
        eprintln!(
            "bee: could not read the current date from {now:?} — no digest was composed this time."
        );
        return Vec::new();
    };

    // ONE directory listing, names only (D12's bounded-read pattern).
    let mut dated: Vec<(NaiveDate, PathBuf)> = Vec::new();
    for path in list_letter_files(root) {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
        if let Some(day) = letter_day_from_name(name) {
            dated.push((day, path));
        }
    }
    if dated.is_empty() {
        return Vec::new();
    }

    // Every period some letter falls in, that has fully ended. Keyed so the
    // order is total: OLDEST FIRST means the period that finished first is
    // folded first, so the key leads with the end date — a week starts before
    // the days inside it but finishes after them, and a reader watching the
    // directory should see Tuesday's digest appear before the week's.
    let mut periods: BTreeMap<(NaiveDate, u8, String), Period> = BTreeMap::new();
    for (day, _) in &dated {
        for period in [Period::day(*day), Period::week(*day)] {
            if period.has_ended(today) {
                periods.insert((period.end, period.kind.rank(), period.id.clone()), period);
            }
        }
    }
    if periods.is_empty() {
        return Vec::new();
    }

    let usage = read_usage_records(root);
    let dir = mailbox_dir(root);
    let mut written = Vec::new();
    for period in periods.into_values() {
        let path = dir.join(period.filename());
        if path.exists() {
            // Already composed. Never reopened, never rewritten — the file IS
            // the marker (see this module's header).
            continue;
        }
        let paths: Vec<PathBuf> =
            dated.iter().filter(|(d, _)| period.contains(*d)).map(|(_, p)| p.clone()).collect();
        let mut letters: Vec<(PathBuf, Letter)> = Vec::new();
        let mut unreadable: Vec<String> = Vec::new();
        for p in &paths {
            match read_letter(p) {
                Ok(letter) => letters.push((p.clone(), letter)),
                Err(why) => {
                    eprintln!(
                        "bee: skipping {} while folding the {} digest ({}) — the rest of the period is still in it.",
                        p.display(),
                        period.id,
                        why.message()
                    );
                    unreadable
                        .push(p.file_name().unwrap_or_default().to_string_lossy().into_owned());
                }
            }
        }
        let in_period: Vec<&UsageRecord> =
            usage.iter().filter(|r| period.contains(r.closed_on)).collect();
        let text = render_digest(&period, now, &letters, &unreadable, &in_period);
        match write_text_atomic(&path, &text) {
            Ok(()) => written.push(DigestWritten {
                path,
                period,
                letters: letters.into_iter().map(|(p, _)| p).collect(),
            }),
            Err(e) => eprintln!(
                "bee: could not write the {} digest to {} ({e}) — that period has no digest to read.",
                period.id,
                path.display()
            ),
        }
    }
    written
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verbs::mailbox::{check_subject, write_letter, LetterItem, STATUS_UNREAD};
    use serde_json::json;

    /// A letter filed at `stamp` for `run`, with the three sections a digest
    /// folds. Built through `write_letter`, so a test can never file a letter
    /// the store itself would refuse.
    fn file_letter(root: &Path, run: &str, stamp: &str, subject: &str, done: &str) -> PathBuf {
        let letter = Letter {
            subject: subject.to_string(),
            run: run.to_string(),
            project: "beehive".to_string(),
            filed_at: stamp.to_string(),
            status: STATUS_UNREAD.to_string(),
            items: vec![LetterItem {
                what: done.to_string(),
                files: vec!["src/lib.rs".to_string()],
                commit: None,
                proof: None,
                departure: None,
            }],
            needs_you: Vec::new(),
            body: format!(
                "## {SECTION_DONE}\n\n- {done}\n\n## {SECTION_BROKEN}\n\n- the windows path test still fails\n"
            ),
        };
        write_letter(root, &letter).unwrap()
    }

    fn read(path: &Path) -> String {
        std::fs::read_to_string(path).unwrap()
    }

    fn names(root: &Path) -> Vec<String> {
        let mut out: Vec<String> = std::fs::read_dir(mailbox_dir(root))
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with(DIGEST_PREFIX))
            .collect();
        out.sort();
        out
    }

    // ── happy ───────────────────────────────────────────────────────────

    #[test]
    fn a_finished_day_with_two_letters_gets_one_digest_folding_both() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        file_letter(root, "run-a", "2026-08-25T03:15:00.000Z", "The hook now refuses an unknown name.", "refused an unknown hook name before the read");
        file_letter(root, "run-b", "2026-08-25T21:40:00.000Z", "The release script waits for the build.", "made the release wait for its build");

        let written = compose_due_digests(root, "2026-09-01T09:00:00.000Z");

        assert_eq!(names(root), vec!["digest-2026-08-25.md", "digest-2026-W35.md"]);
        assert_eq!(written.len(), 2, "one daily and one weekly digest: {written:?}");
        assert_eq!(written[0].period.kind, PeriodKind::Day, "oldest period first, daily leading");
        assert_eq!(written[0].period.id, "2026-08-25");
        assert_eq!(written[1].period.id, "2026-W35");
        assert_eq!(written[0].letters.len(), 2);

        let daily = read(&mailbox_dir(root).join("digest-2026-08-25.md"));
        assert!(daily.contains("type: \"digest\""), "{daily}");
        assert!(daily.contains("subject: \"What happened on 2026-08-25.\""), "{daily}");
        assert!(daily.contains("period_id: \"2026-08-25\""), "{daily}");
        assert!(daily.contains("- \"20260825T031500Z-run-a.md\""), "{daily}");
        assert!(daily.contains("### beehive"), "letters group by project: {daily}");
        assert!(daily.contains("#### The hook now refuses an unknown name."), "{daily}");
        assert!(daily.contains("#### The release script waits for the build."), "{daily}");
        assert!(daily.contains("- refused an unknown hook name before the read"), "{daily}");
        assert!(daily.contains("- made the release wait for its build"), "{daily}");
        assert!(daily.contains(&format!("**{SECTION_BROKEN}**")), "{daily}");

        let weekly = read(&mailbox_dir(root).join("digest-2026-W35.md"));
        assert!(weekly.contains("period: \"week\""), "{weekly}");
        assert!(
            weekly.contains("What happened in the week of 2026-08-24 through 2026-08-30."),
            "{weekly}"
        );
    }

    #[test]
    fn the_digest_subject_is_a_readable_inbox_row() {
        // D2 reaches the digest too: the subject is the row a human scans.
        for period in [
            Period::day(NaiveDate::from_ymd_opt(2026, 8, 25).unwrap()),
            Period::week(NaiveDate::from_ymd_opt(2026, 8, 25).unwrap()),
        ] {
            let subject = period.subject();
            assert!(check_subject(&subject).is_ok(), "{subject:?}: {:?}", check_subject(&subject));
        }
    }

    #[test]
    fn a_usage_record_closed_in_the_period_is_transcribed_field_by_field() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        file_letter(root, "run-a", "2026-08-25T03:15:00.000Z", "The mailbox now files at close.", "filed the letter at close");
        let record = json!({
            "schema": "bee-usage/v1",
            "feature": "letter-digest",
            "closed_at": "2026-08-25T22:00:00.000Z",
            "sessions": [{ "session_id": "s-1", "subagent_count": 4 }],
            "skipped": 1,
            "totals": { "main": { "new": 1200.0, "cached": 300.0, "total": 1500.0 } },
        });
        let dir = root.join(".bee").join("usage");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("letter-digest.json"), record.to_string()).unwrap();

        compose_due_digests(root, "2026-09-01T09:00:00.000Z");
        let daily = read(&mailbox_dir(root).join("digest-2026-08-25.md"));

        assert!(daily.contains("## Token usage"), "{daily}");
        assert!(daily.contains("### letter-digest"), "{daily}");
        assert!(daily.contains("- closed_at: 2026-08-25T22:00:00.000Z"), "{daily}");
        assert!(daily.contains("- totals.main.total: 1500.0"), "{daily}");
        assert!(daily.contains("- totals.main.cached: 300.0"), "{daily}");
        assert!(!daily.contains("s-1"), "the per-session dump is dropped, not summarised: {daily}");
    }

    #[test]
    fn a_usage_record_closed_outside_the_period_stays_out() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        file_letter(root, "run-a", "2026-08-25T03:15:00.000Z", "The mailbox now files at close.", "filed the letter at close");
        let dir = root.join(".bee").join("usage");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("other.json"),
            json!({
                "feature": "other",
                "closed_at": "2026-08-31T10:00:00.000Z",
                "totals": { "main": { "total": 99.0 } },
            })
            .to_string(),
        )
        .unwrap();

        compose_due_digests(root, "2026-09-01T09:00:00.000Z");
        let daily = read(&mailbox_dir(root).join("digest-2026-08-25.md"));
        assert!(!daily.contains("### other"), "a record from another day was folded in: {daily}");
        assert!(!daily.contains("99"), "{daily}");
    }

    // ── edge ────────────────────────────────────────────────────────────

    #[test]
    fn nothing_is_written_for_a_period_with_no_letters_or_one_still_running() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Empty mailbox: no period exists to fold.
        assert!(compose_due_digests(root, "2026-09-01T09:00:00.000Z").is_empty());

        // A letter filed TODAY: neither its day nor its week has ended.
        file_letter(root, "run-live", "2026-09-01T03:15:00.000Z", "The work is still running.", "did a thing");
        assert!(
            compose_due_digests(root, "2026-09-01T09:00:00.000Z").is_empty(),
            "a day still running was digested"
        );
        assert!(names(root).is_empty(), "{:?}", names(root));
    }

    #[test]
    fn an_existing_digest_is_never_rewritten() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        file_letter(root, "run-a", "2026-08-25T03:15:00.000Z", "The first run finished.", "did the first thing");
        let first = compose_due_digests(root, "2026-09-01T09:00:00.000Z");
        assert_eq!(first.len(), 2);
        let daily_path = mailbox_dir(root).join("digest-2026-08-25.md");
        let before = read(&daily_path);

        // A letter that arrives late for a period already closed (D12 recovery
        // digests under its own filed_at day) must not reopen the digest.
        file_letter(root, "run-late", "2026-08-25T23:00:00.000Z", "The late run finished too.", "did the late thing");
        let second = compose_due_digests(root, "2026-09-01T09:00:00.000Z");

        assert!(second.is_empty(), "a second pass wrote something: {second:?}");
        assert_eq!(read(&daily_path), before, "the digest was rewritten under the human");
        assert!(!read(&daily_path).contains("The late run finished too."));
        assert_eq!(names(root).len(), 2, "no extra digest file: {:?}", names(root));
    }

    #[test]
    fn the_body_states_nothing_the_letters_do_not_carry() {
        // human-mailbox D8: a renderer, never a summarizer. Every body line is
        // either structural (a heading this pass owns) or a byte-for-byte
        // transcription of something a letter filed.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let a = file_letter(root, "run-a", "2026-08-25T03:15:00.000Z", "The hook now refuses an unknown name.", "refused an unknown hook name before the read");
        let b = file_letter(root, "run-b", "2026-08-25T21:40:00.000Z", "The release script waits for the build.", "made the release wait for its build");

        compose_due_digests(root, "2026-09-01T09:00:00.000Z");
        let digest = read(&mailbox_dir(root).join("digest-2026-08-25.md"));
        let body = digest.split("---\n").nth(2).unwrap().to_string();

        let mut allowed: Vec<String> = vec!["## Letters".to_string(), "### beehive".to_string()];
        for heading in FOLDED_SECTIONS {
            allowed.push(format!("**{heading}**"));
        }
        for path in [&a, &b] {
            let letter = read_letter(path).unwrap();
            allowed.push(format!("#### {}", letter.subject));
            for (_, lines) in body_sections(&letter.body) {
                allowed.extend(lines);
            }
        }
        for line in body.lines().filter(|l| !l.trim().is_empty()) {
            assert!(
                allowed.iter().any(|a| a == line),
                "the digest authored a line no letter carries: {line:?}\n{digest}"
            );
        }
        assert!(!body.contains("2 letters"), "a count bee computed: {digest}");
    }

    // ── error ───────────────────────────────────────────────────────────

    #[test]
    fn a_torn_letter_is_skipped_and_the_rest_of_the_period_still_folds() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        file_letter(root, "run-good", "2026-08-25T03:15:00.000Z", "The good run finished.", "did the good thing");
        // Half a file, the way a run killed mid-write leaves one.
        std::fs::write(mailbox_dir(root).join("20260825T210000Z-run-torn.md"), "---\nsubject: \"tor")
            .unwrap();

        let written = compose_due_digests(root, "2026-09-01T09:00:00.000Z");

        assert_eq!(written.len(), 2, "the torn letter sank the whole digest: {written:?}");
        assert_eq!(written[0].letters.len(), 1, "the torn letter was folded in anyway");
        let daily = read(&mailbox_dir(root).join("digest-2026-08-25.md"));
        assert!(daily.contains("#### The good run finished."), "{daily}");
        assert!(
            daily.contains("letters:\n  - \"20260825T031500Z-run-good.md\""),
            "the folded list names only what is in the body: {daily}"
        );
        assert!(
            daily.contains("unreadable:\n  - \"20260825T210000Z-run-torn.md\""),
            "a torn letter vanished from the record instead of being named: {daily}"
        );
    }

    // ── period arithmetic ───────────────────────────────────────────────

    #[test]
    fn a_period_is_read_off_the_filename_alone() {
        assert_eq!(
            letter_day_from_name("20260825T031500Z-run-a.md"),
            NaiveDate::from_ymd_opt(2026, 8, 25)
        );
        assert_eq!(letter_day_from_name("digest-2026-08-25.md"), None, "a digest is not a letter");
        assert_eq!(letter_day_from_name("README.md"), None);
        assert_eq!(letter_day_from_name("00000000T000000Z-run-a.md"), None, "an unparseable stamp");
        assert_eq!(letter_day_from_name("20261325T000000Z-run-a.md"), None, "month 13");
    }

    #[test]
    fn an_iso_week_runs_monday_to_sunday_and_names_itself_once() {
        let tuesday = NaiveDate::from_ymd_opt(2026, 8, 25).unwrap();
        let week = Period::week(tuesday);
        assert_eq!(week.id, "2026-W35");
        assert_eq!(week.filename(), "digest-2026-W35.md");
        assert_eq!(week.start, NaiveDate::from_ymd_opt(2026, 8, 24).unwrap());
        assert_eq!(week.end, NaiveDate::from_ymd_opt(2026, 8, 30).unwrap());
        // Every day of that week names the same period — one week, one digest.
        for day in 24..=30 {
            let d = NaiveDate::from_ymd_opt(2026, 8, day).unwrap();
            assert_eq!(Period::week(d).id, "2026-W35");
            assert!(week.contains(d));
        }
        assert!(!week.contains(NaiveDate::from_ymd_opt(2026, 8, 31).unwrap()));
        assert!(week.has_ended(NaiveDate::from_ymd_opt(2026, 8, 31).unwrap()));
        assert!(!week.has_ended(NaiveDate::from_ymd_opt(2026, 8, 30).unwrap()));
        assert_eq!(Period::day(tuesday).filename(), "digest-2026-08-25.md");
    }
}
