//! Fixture generator — the D5 host-real `.bee` store (CONTEXT.md D5,
//! rust-port-2 truths). Produces a self-sufficient store at pinned minimum
//! sizes (decisions.jsonl >= 700 KB, reservations.json >= 600 KB,
//! backlog.jsonl >= 250 KB, >= 250 cell files) that:
//!
//! - resolves as a bee root on its own (`.bee/onboarding.json` present, no
//!   `.git` anywhere above it — `resolveRootsCore`'s first branch matches
//!   immediately, per the dry-run proof this cell recorded against the real
//!   `bee.mjs`), and
//! - survives a real `node .bee/bin/bee.mjs status --json` read without
//!   crashing (every reader in the mjs estate is fail-open on missing/
//!   malformed substructure — proven empirically, not just read from source).
//!
//! `generate` REFUSES (returns `Err`, writes nothing) if any requested size
//! is below its pinned floor — the D5 acceptance instrument must never
//! silently benchmark against a smaller-than-host-real store.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Pinned minimums from CONTEXT.md D5 / plan.md cell rust-port-2.
pub const DECISIONS_FLOOR_BYTES: u64 = 700_000;
pub const RESERVATIONS_FLOOR_BYTES: u64 = 600_000;
pub const BACKLOG_FLOOR_BYTES: u64 = 250_000;
pub const CELLS_FLOOR_COUNT: usize = 250;

#[derive(Debug, Clone)]
pub struct FixtureRequest {
    pub out_dir: PathBuf,
    pub decisions_bytes: u64,
    pub reservations_bytes: u64,
    pub backlog_bytes: u64,
    pub cells_count: usize,
}

impl FixtureRequest {
    /// A request pinned to exactly the D5 floors — the happy-path shape.
    pub fn at_floors(out_dir: PathBuf) -> Self {
        Self {
            out_dir,
            decisions_bytes: DECISIONS_FLOOR_BYTES,
            reservations_bytes: RESERVATIONS_FLOOR_BYTES,
            backlog_bytes: BACKLOG_FLOOR_BYTES,
            cells_count: CELLS_FLOOR_COUNT,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FixtureReport {
    pub root: PathBuf,
    pub decisions_bytes: u64,
    pub reservations_bytes: u64,
    pub backlog_bytes: u64,
    pub cells_count: usize,
}

impl FixtureReport {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"root\":\"{}\",\"decisions_bytes\":{},\"reservations_bytes\":{},\"backlog_bytes\":{},\"cells_count\":{}}}",
            json_escape(&self.root.display().to_string()),
            self.decisions_bytes,
            self.reservations_bytes,
            self.backlog_bytes,
            self.cells_count,
        )
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Validate the four pins before touching the filesystem. Returns the first
/// violation found (decisions, then reservations, then backlog, then cells)
/// so a caller driving all-four-refuse coverage gets a distinct message per
/// case.
fn check_floors(req: &FixtureRequest) -> Result<(), String> {
    if req.decisions_bytes < DECISIONS_FLOOR_BYTES {
        return Err(format!(
            "decisions.jsonl requested {} byte(s) < pinned floor {} byte(s) (CONTEXT.md D5) — refusing to generate a sub-pinned-size fixture",
            req.decisions_bytes, DECISIONS_FLOOR_BYTES
        ));
    }
    if req.reservations_bytes < RESERVATIONS_FLOOR_BYTES {
        return Err(format!(
            "reservations.json requested {} byte(s) < pinned floor {} byte(s) (CONTEXT.md D5) — refusing to generate a sub-pinned-size fixture",
            req.reservations_bytes, RESERVATIONS_FLOOR_BYTES
        ));
    }
    if req.backlog_bytes < BACKLOG_FLOOR_BYTES {
        return Err(format!(
            "backlog.jsonl requested {} byte(s) < pinned floor {} byte(s) (CONTEXT.md D5) — refusing to generate a sub-pinned-size fixture",
            req.backlog_bytes, BACKLOG_FLOOR_BYTES
        ));
    }
    if req.cells_count < CELLS_FLOOR_COUNT {
        return Err(format!(
            "cells count requested {} < pinned floor {} (CONTEXT.md D5) — refusing to generate a sub-pinned-size fixture",
            req.cells_count, CELLS_FLOOR_COUNT
        ));
    }
    Ok(())
}

/// Generate the fixture at `req.out_dir`. Refuses (no writes at all) if any
/// pin is under its floor; otherwise writes a full `.bee/` store and returns
/// the byte/file counts actually written (measured, not just echoed back).
pub fn generate(req: &FixtureRequest) -> Result<FixtureReport, String> {
    check_floors(req)?;

    let bee_dir = req.out_dir.join(".bee");
    let cells_dir = bee_dir.join("cells");
    fs::create_dir_all(&cells_dir).map_err(|e| format!("mkdir {}: {e}", cells_dir.display()))?;

    write_onboarding(&bee_dir)?;
    write_state(&bee_dir)?;
    write_config(&bee_dir)?;

    let decisions_bytes = write_jsonl_padded(&bee_dir.join("decisions.jsonl"), req.decisions_bytes, decision_line)?;
    let backlog_bytes = write_jsonl_padded(&bee_dir.join("backlog.jsonl"), req.backlog_bytes, backlog_line)?;
    let reservations_bytes = write_reservations_padded(&bee_dir.join("reservations.json"), req.reservations_bytes)?;

    for i in 0..req.cells_count {
        let path = cells_dir.join(format!("fixture-{i:05}.json"));
        fs::write(&path, cell_json(i)).map_err(|e| format!("write {}: {e}", path.display()))?;
    }

    Ok(FixtureReport {
        root: req.out_dir.clone(),
        decisions_bytes,
        reservations_bytes,
        backlog_bytes,
        cells_count: req.cells_count,
    })
}

fn write_onboarding(bee_dir: &Path) -> Result<(), String> {
    // Minimal but real: bee_version present (installed:true), managed:{}
    // (empty — computeRuntimeDrift's checkGroup short-circuits on a
    // non-object/absent recorded map, so this fixture reports drift:false
    // without needing to carry the whole vendored .bee/bin tree).
    let path = bee_dir.join("onboarding.json");
    let body = "{\"schema_version\":\"1.0\",\"bee_version\":\"1.18.2\",\"managed\":{},\"created_at\":\"2026-07-26T00:00:00.000Z\",\"updated_at\":\"2026-07-26T00:00:00.000Z\"}";
    fs::write(&path, body).map_err(|e| format!("write {}: {e}", path.display()))
}

fn write_state(bee_dir: &Path) -> Result<(), String> {
    let path = bee_dir.join("state.json");
    let body = "{\"schema_version\":\"1.0\",\"phase\":\"idle\",\"feature\":null,\"mode\":null,\"approved_gates\":{\"context\":false,\"shape\":false,\"execution\":false,\"review\":false},\"workers\":[],\"summary\":\"\",\"next_action\":\"No active bee work.\"}";
    fs::write(&path, body).map_err(|e| format!("write {}: {e}", path.display()))
}

fn write_config(bee_dir: &Path) -> Result<(), String> {
    let path = bee_dir.join("config.json");
    let body = "{\"hooks\":{},\"lanes\":{},\"capabilities\":{},\"commands\":{},\"gate_bypass\":\"off\",\"models\":{}}";
    fs::write(&path, body).map_err(|e| format!("write {}: {e}", path.display()))
}

/// Deterministic filler text so each decisions.jsonl line lands in the same
/// ballpark (a few hundred bytes) as a real recorded decision, keeping the
/// padding loop's iteration count sane without pretending to be real content.
const FILLER_DECISION: &str = "Fixture decision line generated by queen-bench for the D5 host-real benchmark store. This text pads the record to a size representative of the live repo's decisions.jsonl so the bench never runs against a synthetic small-store fiction.";
const FILLER_RATIONALE: &str = "Rationale filler: this is bench fixture data, not a real recorded decision — generated solely to reach the pinned decisions.jsonl floor from CONTEXT.md D5 for spawn-inclusive p95 measurement.";

fn decision_line(i: usize) -> String {
    format!(
        "{{\"id\":\"fixture-decision-{i:06}\",\"type\":\"decide\",\"date\":\"2026-07-26T00:00:00.000Z\",\"decision\":\"{FILLER_DECISION}\",\"rationale\":\"{FILLER_RATIONALE}\",\"alternatives\":null,\"scope\":\"repo\",\"source\":\"fixture\",\"confidence\":0}}"
    )
}

const FILLER_DETAIL: &str = "Fixture backlog detail generated by queen-bench for the D5 host-real benchmark store — pads backlog.jsonl to the pinned floor for spawn-inclusive p95 measurement; not a real proposal.";
const FILLER_IMPACT: &str = "Fixture predicted_impact filler text, sized to keep each backlog.jsonl line representative of the live repo's line lengths.";

fn backlog_line(i: usize) -> String {
    format!(
        "{{\"ts\":\"2026-07-26T00:00:00.000Z\",\"type\":\"proposal\",\"title\":\"fixture backlog item {i:06}\",\"detail\":\"{FILLER_DETAIL}\",\"predicted_impact\":\"{FILLER_IMPACT}\",\"lane\":\"tiny\",\"source\":\"queen-bench fixture\"}}"
    )
}

fn reservation_entry(i: usize) -> String {
    format!(
        "{{\"agent\":\"fixture-agent-{i:06}\",\"cell\":\"fixture-cell-{i:06}\",\"path\":\"crates/fixture/path-{i:06}.rs\",\"ttl_seconds\":3600,\"reserved_at\":\"2026-07-26T00:00:00.000Z\",\"released_at\":\"2026-07-26T00:00:01.000Z\"}}"
    )
}

fn cell_json(i: usize) -> String {
    format!(
        "{{\"id\":\"fixture-cell-{i:05}\",\"feature\":\"queen-bench-fixture\",\"lane\":\"tiny\",\"behavior_change\":false,\"title\":\"fixture cell {i:05}\",\"action\":\"queen-bench D5 fixture filler cell — not real work\",\"must_haves\":{{\"truths\":[],\"prohibitions\":[]}},\"verify\":\"true\",\"read_first\":[],\"files\":[],\"deps\":[],\"status\":\"capped\",\"decisions\":[],\"trace\":{{\"worker\":null,\"outcome\":null,\"files_changed\":[],\"deviations\":[],\"friction\":null,\"capped_at\":null,\"behavior_change\":false,\"verification_evidence\":null,\"verify_output\":null,\"verify_passed\":null,\"claim_session\":null,\"claimed_at\":null}}}}"
    )
}

/// Append `make_line(i)` + `\n` until the file reaches `target_bytes`,
/// returning the exact byte count written.
fn write_jsonl_padded(
    path: &Path,
    target_bytes: u64,
    make_line: impl Fn(usize) -> String,
) -> Result<u64, String> {
    let mut file = fs::File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
    let mut written: u64 = 0;
    let mut i = 0usize;
    while written < target_bytes {
        let line = make_line(i);
        file.write_all(line.as_bytes())
            .and_then(|_| file.write_all(b"\n"))
            .map_err(|e| format!("write {}: {e}", path.display()))?;
        written += line.len() as u64 + 1;
        i += 1;
    }
    Ok(written)
}

/// Build `{"reservations":[...]}` incrementally until the serialized body
/// reaches `target_bytes`, then write it as one JSON document.
fn write_reservations_padded(path: &Path, target_bytes: u64) -> Result<u64, String> {
    let mut body = String::from("{\"reservations\":[");
    let mut first = true;
    let mut i = 0usize;
    const CLOSING_OVERHEAD: u64 = 2; // "]}"
    loop {
        let entry = reservation_entry(i);
        if !first {
            body.push(',');
        }
        body.push_str(&entry);
        first = false;
        i += 1;
        if body.len() as u64 + CLOSING_OVERHEAD >= target_bytes {
            break;
        }
    }
    body.push_str("]}");
    fs::write(path, &body).map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(body.len() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_each_pin_independently() {
        let base = FixtureRequest::at_floors(PathBuf::from("/unused-in-refusal-check"));
        let mut d = base.clone();
        d.decisions_bytes = DECISIONS_FLOOR_BYTES - 1;
        assert!(check_floors(&d).is_err());

        let mut r = base.clone();
        r.reservations_bytes = RESERVATIONS_FLOOR_BYTES - 1;
        assert!(check_floors(&r).is_err());

        let mut b = base.clone();
        b.backlog_bytes = BACKLOG_FLOOR_BYTES - 1;
        assert!(check_floors(&b).is_err());

        let mut c = base.clone();
        c.cells_count = CELLS_FLOOR_COUNT - 1;
        assert!(check_floors(&c).is_err());

        assert!(check_floors(&base).is_ok());
    }
}
