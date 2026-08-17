// bee discovery — the wayfinding-flow map registry (D2, D4, D6.3;
// docs/history/wayfinding-flow/CONTEXT.md).
//
// A "map" is a fog-state effort's index at `docs/discovery/<effort>/MAP.md`
// plus one file per open question under `docs/discovery/<effort>/tickets/
// NNN-<slug>.md`. This module owns two things:
//
//   discovery list                       [--json]
//   discovery stub --effort <slug> --from <text> [--json]
//
// `list` scans every effort directory under `docs/discovery/` and reports
// its Destination line plus two ticket counts: `open` (status: open,
// regardless of blocking) and `frontier` (open, unclaimed, and every
// `blocked-by` target closed — D7's "what a session may take"). A MAP.md
// that cannot be read (missing, not valid UTF-8, any other io error) is
// never a crash: it surfaces as a visible `unreadable <path> — remedy: fix
// or delete` line, the same fail-open shape `verbs/triggers` already uses
// for a corrupt trigger record.
//
// `stub` creates a brand-new MAP.md for an effort whose destination is not
// yet known — the D6.3 landing spot for `bee route`'s park-for-vagueness
// verdict, so a parked, too-vague item gets a visible home instead of
// sinking silently. It is a typed, nothing-written refusal when the effort
// slug already has a directory: resuming an existing map is a different
// operation from charting a new one.
//
// Ticket frontmatter (D7: "convention-only in v1, no CLI guard") is read as
// plain `key: value` lines anywhere in the file — no YAML fencing required,
// so the skill's exact ticket template stays free to evolve without a
// parser rewrite. Four keys are recognized: `type` (read but not
// interpreted here — the skill enforces the four-type vocabulary),
// `status` (`open` when the key is absent — a freshly ticketed question is
// open by construction), `claimed-by` (present+non-empty means claimed),
// `blocked-by` (comma-separated ticket ids). A ticket file this module
// cannot read at all is skipped from both counts — the same "one bad file
// never sinks the read" contract as the MAP.md case, just without its own
// visible line, since a ticket file's exact shape is the skill's to define
// and this scan only ever consumes it.
//
// Root: WIDE (`resolve_store_root_any`) — `docs/discovery/` is plain
// git-tracked content meant to be visible the same way from every worktree
// and every session (D4: "open maps are visible from v1"), the same
// always-the-store-root choice `verbs/knowledge`'s `docs/knowledge` and
// `verbs/backlog`'s `.bee/backlog.jsonl` already make. A GRANTED linked
// worktree therefore reads and writes the SAME `docs/discovery/` as the
// main checkout, not a per-worktree copy — the intended behavior: an open
// map found in session 1 must resume in session 2 regardless of which
// worktree that later session happens to run from.

use super::feedback::{emit_error, emit_success, js_trim, parse_shape, ParsedArgs};
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_any as resolve_store_root, Roots};
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use serde_json::json;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── paths ──────────────────────────────────────────────────────────────

fn discovery_dir(root: &Path) -> PathBuf {
    root.join("docs").join("discovery")
}

fn map_path(root: &Path, slug: &str) -> PathBuf {
    discovery_dir(root).join(slug).join("MAP.md")
}

fn tickets_dir(root: &Path, slug: &str) -> PathBuf {
    discovery_dir(root).join(slug).join("tickets")
}

// ─── ticket frontmatter (D7 convention-only keys) ──────────────────────────

struct TicketRecord {
    status: String,
    claimed_by: Option<String>,
    blocked_by: Vec<String>,
}

/// Scans every line for the four known `key: value` prefixes — no fencing,
/// no ordering requirement, first occurrence of each key wins. A key never
/// seen takes its safe default: `status` open (a bare new ticket is open by
/// construction), `claimed_by` none, `blocked_by` empty.
fn parse_ticket(text: &str) -> TicketRecord {
    let mut status = "open".to_string();
    let mut claimed_by: Option<String> = None;
    let mut blocked_by: Vec<String> = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix("status:") {
            let v = js_trim(rest);
            if !v.is_empty() {
                status = v.to_string();
            }
        } else if let Some(rest) = line.strip_prefix("claimed-by:") {
            let v = js_trim(rest);
            if !v.is_empty() {
                claimed_by = Some(v.to_string());
            }
        } else if let Some(rest) = line.strip_prefix("blocked-by:") {
            let v = js_trim(rest);
            if !v.is_empty() {
                blocked_by = v.split(',').map(js_trim).filter(|s| !s.is_empty()).map(str::to_string).collect();
            }
        }
    }
    TicketRecord { status, claimed_by, blocked_by }
}

/// `(open, frontier)` — `open` counts every `status: open` ticket; `frontier`
/// narrows that to unclaimed AND every `blocked-by` target closed. A
/// `blocked-by` id this scan cannot resolve to a known ticket FAILS CLOSED
/// (still counts as blocking) rather than silently unblocking on a typo or a
/// ticket that raced away mid-scan.
fn scan_tickets(dir: &Path) -> (usize, usize) {
    let mut names: Vec<String> = match std::fs::read_dir(dir) {
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
    let mut records: HashMap<String, TicketRecord> = HashMap::new();
    for name in &names {
        if let Ok(text) = std::fs::read_to_string(dir.join(name)) {
            let id = name.trim_end_matches(".md").to_string();
            records.insert(id, parse_ticket(&text));
        }
        // An unreadable ticket file is skipped from both counts — its own
        // shape is the skill's to define, not this scan's to police.
    }
    let open = records.values().filter(|r| r.status == "open").count();
    let frontier = records
        .values()
        .filter(|r| {
            r.status == "open"
                && r.claimed_by.is_none()
                && r.blocked_by.iter().all(|b| records.get(b).map(|t| t.status == "closed").unwrap_or(false))
        })
        .count();
    (open, frontier)
}

// ─── MAP.md scan ────────────────────────────────────────────────────────

pub(crate) struct EffortEntry {
    pub(crate) name: String,
    pub(crate) destination: String,
    pub(crate) open: usize,
    pub(crate) frontier: usize,
}

/// First non-empty line under a `## Destination` heading, up to the next
/// heading. Empty string when the heading is missing or has no content
/// before the next `#` — never a parse failure; MAP.md's exact section
/// wording is the skill's discretion, this only ever reads the one section
/// `list` reports.
fn extract_destination(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let Some(idx) = lines.iter().position(|l| l.trim() == "## Destination") else {
        return String::new();
    };
    for line in &lines[idx + 1..] {
        let t = line.trim();
        if t.starts_with('#') {
            break;
        }
        if !t.is_empty() {
            return t.to_string();
        }
    }
    String::new()
}

/// Every effort directory under `docs/discovery/`, each either a scanned
/// `EffortEntry` or the path to a MAP.md this scan could not read at all
/// (missing, not valid UTF-8, any other io error). No `docs/discovery/`
/// directory at all is simply zero efforts, zero unreadable — never an
/// error of its own.
pub(crate) fn scan_discovery(root: &Path) -> (Vec<EffortEntry>, Vec<PathBuf>) {
    let dir = discovery_dir(root);
    let mut names: Vec<String> = match std::fs::read_dir(&dir) {
        Ok(entries) => entries
            .flatten()
            .filter_map(|e| {
                let ft = e.file_type().ok()?;
                ft.is_dir().then(|| e.file_name().to_string_lossy().into_owned())
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    names.sort();
    let mut efforts = Vec::new();
    let mut unreadable = Vec::new();
    for name in names {
        let mp = dir.join(&name).join("MAP.md");
        match std::fs::read_to_string(&mp) {
            Ok(text) => {
                let destination = extract_destination(&text);
                let (open, frontier) = scan_tickets(&tickets_dir(root, &name));
                efforts.push(EffortEntry { name, destination, open, frontier });
            }
            Err(_) => unreadable.push(mp),
        }
    }
    (efforts, unreadable)
}

fn unreadable_line(path: &Path) -> String {
    format!("unreadable {} — remedy: fix or delete", path.display())
}

// ─── stub content (D6, D2 section list) ────────────────────────────────────

fn stub_map_content(slug: &str, from: &str) -> String {
    format!(
        "# {slug}\n\n\
         ## Destination\n\n\
         (unknown — charting session needed)\n\n\
         ## Notes\n\n\
         {from}\n\n\
         ## Decisions so far\n\n\
         ## Not yet specified\n\n\
         ## Out of scope\n"
    )
}

/// Lowercase kebab-case only: ascii letters/digits, single dashes, no
/// leading/trailing dash. The slug becomes a directory name under
/// `docs/discovery/`, so this also closes off path traversal (`..`, `/`)
/// by construction — nothing outside this charset is accepted at all.
fn is_valid_effort_slug(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut prev_dash = true; // blocks a leading dash
    for c in s.chars() {
        if c == '-' {
            if prev_dash {
                return false;
            }
            prev_dash = true;
        } else if c.is_ascii_lowercase() || c.is_ascii_digit() {
            prev_dash = false;
        } else {
            return false;
        }
    }
    !prev_dash // blocks a trailing dash
}

// ─── argv plumbing ──────────────────────────────────────────────────────

struct Ctx {
    root: PathBuf,
    drift: crate::registry::Drift,
}

fn preamble(cmd: &str, pre_json: bool, t0: Instant) -> Result<Option<Ctx>, ExitCode> {
    let Ok(cwd) = std::env::current_dir() else { return Ok(None) };
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::Unsupported(why) => return Err(emit_unsupported_root(&cwd, cmd, pre_json, t0, &why)),
        Roots::None => return Err(emit_no_root_error(&cwd, cmd, pre_json, t0)),
    };
    let drift = check_manifest_drift(&root);
    Ok(Some(Ctx { root, drift }))
}

fn flag<'a>(parsed: &'a ParsedArgs, name: &str) -> Option<&'a str> {
    parsed.flags.get(name).map(|s| js_trim(s)).filter(|s| !s.is_empty())
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "discovery" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    let rest = &args[2..];
    match verb {
        "list" => run_list(parse_shape(rest, &[])?, t0),
        "stub" => run_stub(parse_shape(rest, &["effort", "from"])?, t0),
        _ => None,
    }
}

// ─── list ────────────────────────────────────────────────────────────────

fn run_list(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let cmd = "discovery list";
    let ctx = match preamble(cmd, parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let (efforts, unreadable) = scan_discovery(&ctx.root);
    let mut lines = Vec::new();
    let mut effort_values = Vec::new();
    for e in &efforts {
        let dest = if e.destination.is_empty() { "(no destination yet)" } else { e.destination.as_str() };
        lines.push(format!("- {}: {} (open {}, frontier {})", e.name, dest, e.open, e.frontier));
        effort_values.push(json!({
            "name": e.name,
            "destination": e.destination,
            "open": e.open,
            "frontier": e.frontier,
        }));
    }
    let mut unreadable_values = Vec::new();
    for p in &unreadable {
        lines.push(unreadable_line(p));
        unreadable_values.push(json!(p.display().to_string()));
    }
    let text = if lines.is_empty() { "No discovery maps.".to_string() } else { lines.join("\n") };
    let result = json!({ "efforts": effort_values, "unreadable": unreadable_values });
    Some(emit_success(&ctx.root, cmd, parsed.json, &ctx.drift, &result, &text, t0))
}

// ─── stub ────────────────────────────────────────────────────────────────

fn run_stub(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let cmd = "discovery stub";
    let ctx = match preamble(cmd, parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let Some(effort) = flag(&parsed, "effort") else {
        return Some(emit_error(&ctx.root, cmd, parsed.json, &format!("bee {cmd}: --effort is required."), t0));
    };
    let Some(from) = flag(&parsed, "from") else {
        return Some(emit_error(&ctx.root, cmd, parsed.json, &format!("bee {cmd}: --from is required."), t0));
    };
    if !is_valid_effort_slug(effort) {
        let msg = format!(
            "bee {cmd}: --effort must be a lowercase kebab-case slug (letters, digits, single dashes, no leading/trailing dash), got {effort:?}."
        );
        return Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0));
    }
    let dir = discovery_dir(&ctx.root).join(effort);
    if dir.exists() {
        let msg = format!(
            "bee {cmd}: effort \"{effort}\" already exists at {} — pick a different --effort or resume the existing map.",
            dir.display()
        );
        return Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0));
    }
    let mp = map_path(&ctx.root, effort);
    let content = stub_map_content(effort, from);
    if crate::fsutil::write_text_atomic(&mp, &content).is_err() {
        let msg = format!("bee {cmd}: could not write {}.", mp.display());
        return Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0));
    }
    let text = format!("Created discovery stub for \"{effort}\" at {}.", mp.display());
    let result = json!({ "effort": effort, "path": mp.display().to_string() });
    Some(emit_success(&ctx.root, cmd, parsed.json, &ctx.drift, &result, &text, t0))
}

// ─── tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, rel: &str, content: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }

    // ── extract_destination ─────────────────────────────────────────────

    #[test]
    fn extract_destination_reads_the_first_content_line_under_the_heading() {
        let text = "# eff\n\n## Destination\n\nA locked spec for X.\n\n## Notes\n\nsomething\n";
        assert_eq!(extract_destination(text), "A locked spec for X.");
    }

    #[test]
    fn extract_destination_is_empty_when_the_section_has_no_content_yet() {
        let text = "# eff\n\n## Destination\n\n## Notes\n\nsomething\n";
        assert_eq!(extract_destination(text), "");
    }

    #[test]
    fn extract_destination_is_empty_when_the_heading_is_missing() {
        assert_eq!(extract_destination("# eff\n\nsome prose\n"), "");
    }

    // ── ticket frontier calc (plan's test matrix: happy/edge rows) ─────────

    #[test]
    fn an_open_unclaimed_unblocked_ticket_is_frontier() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "tickets/001-a.md", "status: open\n\nWhat should X be?\n");
        let (open, frontier) = scan_tickets(&tmp.path().join("tickets"));
        assert_eq!((open, frontier), (1, 1));
    }

    #[test]
    fn a_claimed_ticket_is_open_but_not_frontier() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "tickets/001-a.md", "status: open\nclaimed-by: session-1\n");
        let (open, frontier) = scan_tickets(&tmp.path().join("tickets"));
        assert_eq!((open, frontier), (1, 0));
    }

    #[test]
    fn a_ticket_blocked_on_an_open_ticket_is_excluded_from_frontier() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "tickets/001-a.md", "status: open\n");
        write(tmp.path(), "tickets/002-b.md", "status: open\nblocked-by: 001-a\n");
        let (open, frontier) = scan_tickets(&tmp.path().join("tickets"));
        assert_eq!((open, frontier), (2, 1), "only 001-a is frontier");
    }

    #[test]
    fn a_ticket_blocked_on_a_closed_ticket_is_unblocked() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "tickets/001-a.md", "status: closed\n");
        write(tmp.path(), "tickets/002-b.md", "status: open\nblocked-by: 001-a\n");
        let (open, frontier) = scan_tickets(&tmp.path().join("tickets"));
        assert_eq!((open, frontier), (1, 1));
    }

    #[test]
    fn a_ticket_blocked_on_an_unknown_id_fails_closed_never_frontier() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "tickets/001-a.md", "status: open\nblocked-by: nope-999\n");
        let (open, frontier) = scan_tickets(&tmp.path().join("tickets"));
        assert_eq!((open, frontier), (1, 0));
    }

    #[test]
    fn a_closed_ticket_never_counts_as_open_or_frontier() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "tickets/001-a.md", "status: closed\n");
        let (open, frontier) = scan_tickets(&tmp.path().join("tickets"));
        assert_eq!((open, frontier), (0, 0));
    }

    #[test]
    fn a_ticket_with_no_status_line_defaults_open() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "tickets/001-a.md", "What should X be?\n");
        let (open, frontier) = scan_tickets(&tmp.path().join("tickets"));
        assert_eq!((open, frontier), (1, 1));
    }

    #[test]
    fn a_missing_tickets_dir_scans_as_zero_zero() {
        let tmp = tempfile::tempdir().unwrap();
        let (open, frontier) = scan_tickets(&tmp.path().join("tickets"));
        assert_eq!((open, frontier), (0, 0));
    }

    // ── scan_discovery: happy + malformed-map error row ─────────────────

    #[test]
    fn scan_discovery_reports_an_effort_with_its_destination_and_counts() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            tmp.path(),
            "docs/discovery/onboarding/MAP.md",
            "# onboarding\n\n## Destination\n\nA spec for the new flow.\n",
        );
        write(tmp.path(), "docs/discovery/onboarding/tickets/001-a.md", "status: open\n");
        let (efforts, unreadable) = scan_discovery(tmp.path());
        assert!(unreadable.is_empty(), "{unreadable:?}");
        assert_eq!(efforts.len(), 1);
        assert_eq!(efforts[0].name, "onboarding");
        assert_eq!(efforts[0].destination, "A spec for the new flow.");
        assert_eq!((efforts[0].open, efforts[0].frontier), (1, 1));
    }

    #[test]
    fn scan_discovery_is_empty_with_no_discovery_dir_at_all() {
        let tmp = tempfile::tempdir().unwrap();
        let (efforts, unreadable) = scan_discovery(tmp.path());
        assert!(efforts.is_empty());
        assert!(unreadable.is_empty());
    }

    #[test]
    fn an_unreadable_map_yields_the_remedy_line_never_a_crash() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("docs").join("discovery").join("broken");
        std::fs::create_dir_all(&dir).unwrap();
        // Invalid UTF-8 bytes: read_to_string must fail, never panic.
        std::fs::write(dir.join("MAP.md"), [0xFF, 0xFE, 0x00, 0xFF]).unwrap();
        let (efforts, unreadable) = scan_discovery(tmp.path());
        assert!(efforts.is_empty());
        assert_eq!(unreadable.len(), 1);
        let line = unreadable_line(&unreadable[0]);
        assert!(line.starts_with("unreadable "), "{line}");
        assert!(line.ends_with(" — remedy: fix or delete"), "{line}");
    }

    #[test]
    fn a_directory_with_no_map_md_is_reported_unreadable_not_dropped_silently() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("docs").join("discovery").join("empty-dir")).unwrap();
        let (efforts, unreadable) = scan_discovery(tmp.path());
        assert!(efforts.is_empty());
        assert_eq!(unreadable.len(), 1);
        assert!(unreadable[0].ends_with("MAP.md"), "{:?}", unreadable[0]);
    }

    // ── slug validation ──────────────────────────────────────────────────

    #[test]
    fn effort_slug_accepts_lowercase_kebab_case_only() {
        assert!(is_valid_effort_slug("onboarding-flow"));
        assert!(is_valid_effort_slug("a1"));
        assert!(!is_valid_effort_slug(""));
        assert!(!is_valid_effort_slug("-leading"));
        assert!(!is_valid_effort_slug("trailing-"));
        assert!(!is_valid_effort_slug("Has-Caps"));
        assert!(!is_valid_effort_slug("has space"));
        assert!(!is_valid_effort_slug("../escape"));
        assert!(!is_valid_effort_slug("a/b"));
    }

    // ── stub content ──────────────────────────────────────────────────────

    #[test]
    fn stub_content_carries_the_unknown_destination_and_the_from_text_under_notes() {
        let content = stub_map_content("onboarding-flow", "parked from route Qualify: too vague");
        assert!(content.contains("## Destination\n\n(unknown — charting session needed)"));
        assert!(content.contains("## Notes\n\nparked from route Qualify: too vague"));
        assert!(content.contains("## Decisions so far"));
        assert!(content.contains("## Not yet specified"));
        assert!(content.contains("## Out of scope"));
    }

    #[test]
    fn stub_write_then_scan_round_trips_the_destination_and_zero_frontier() {
        let tmp = tempfile::tempdir().unwrap();
        let content = stub_map_content("onboarding-flow", "an itch about the flow");
        crate::fsutil::write_text_atomic(&tmp.path().join("docs/discovery/onboarding-flow/MAP.md"), &content)
            .unwrap();
        let (efforts, unreadable) = scan_discovery(tmp.path());
        assert!(unreadable.is_empty(), "{unreadable:?}");
        assert_eq!(efforts.len(), 1);
        assert_eq!(efforts[0].name, "onboarding-flow");
        assert_eq!(efforts[0].destination, "(unknown — charting session needed)");
        assert_eq!((efforts[0].open, efforts[0].frontier), (0, 0));
    }
}
