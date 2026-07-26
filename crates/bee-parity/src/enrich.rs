//! Branch-coverage enrichment for the `--status-check` parity proof
//! (rust-port-15).
//!
//! ## Why this exists
//!
//! The D5 fixture (`queen-bench::fixture`) is pinned for SIZE — it is what
//! makes the 5 ms bench honest. But its store SHAPE is deliberately quiet:
//! `phase: "idle"`, no feature, no mode, no lanes, no handoff, no
//! contention log, no configured commands or models, gates all pending. A
//! parity run over that store alone proves the parity of the code paths
//! that fire on a quiet store — and says NOTHING about every branch
//! `buildStatus`/`renderStatusText` only take when there is something to
//! report: the bypass banner, the Lanes line, the handoff branch of
//! `recommended_next`, the contention block, the models/tier-mix/PBI/
//! capture-queue/staleness lines, `formatSlot`'s cli and `model@effort`
//! shapes.
//!
//! So `--status-check` runs TWO scenarios over the same generated fixture:
//! `plain` (the fixture as generated) and `enriched` (the same store with
//! the additive records below written into it). Both scenarios diff the
//! frozen mjs oracle against `queen-bee` — mjs stays the single source of
//! truth in both, and nothing here is a normalization or an allowance.
//! Enrichment only ADDS input; it never touches the bench's own fixture,
//! and it never shrinks anything.
//!
//! ## Determinism
//!
//! Every value written here is a fixed literal, with two deliberate
//! exceptions that the declared volatile allowlist already covers: the
//! session heartbeat (a fresh ISO timestamp, so the session reads as live
//! in both legs — 15 minutes of staleness margin makes the two spawns'
//! millisecond difference irrelevant) and the deliberately ancient handoff
//! timestamp. Both normalize to `<TS>`.

use std::fs;
use std::path::Path;

/// The feature slug the enriched lane/state/cells agree on.
pub const ENRICHED_FEATURE: &str = "fixture-enriched";

/// The enriched `state.json` body, written as an exact literal so
/// [`seed_enriched_mutation`] can pin it the same way `mutate.rs` pins the
/// plain fixture body.
const ENRICHED_STATE_BODY: &str = concat!(
    "{\"schema_version\":\"1.0\",\"phase\":\"scribing\",\"feature\":\"fixture-enriched\",",
    "\"mode\":\"high-risk\",\"approved_gates\":{\"context\":true,\"shape\":true,\"execution\":true,\"review\":false},",
    "\"workers\":[],\"summary\":\"enriched parity scenario\",\"next_action\":\"Run bee-scribing capture.\"}"
);

const MUTATED_ENRICHED_STATE_BODY: &str = concat!(
    "{\"schema_version\":\"1.0\",\"phase\":\"bee-parity-seeded-mutation\",\"feature\":\"fixture-enriched\",",
    "\"mode\":\"high-risk\",\"approved_gates\":{\"context\":true,\"shape\":true,\"execution\":true,\"review\":false},",
    "\"workers\":[],\"summary\":\"enriched parity scenario\",\"next_action\":\"Run bee-scribing capture.\"}"
);

fn write(path: &Path, body: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create_dir_all {}: {e}", parent.display()))?;
    }
    fs::write(path, body).map_err(|e| format!("write {}: {e}", path.display()))
}

fn iso_now_millis() -> String {
    // A plain UTC ISO-8601 with millisecond precision — the shape
    // `heartbeatStale`'s `Date.parse` accepts and the allowlist's
    // timestamp matcher normalizes.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let millis = 0i64;
    let (y, mo, d, h, mi, s) = civil_from_unix(secs);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}.{millis:03}Z")
}

/// Days-from-civil inverse (Howard Hinnant's algorithm), std-only — no
/// chrono/time dependency (this crate has zero dependencies by design).
fn civil_from_unix(secs: i64) -> (i64, u32, u32, u32, u32, u32) {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d, (rem / 3600) as u32, ((rem % 3600) / 60) as u32, (rem % 60) as u32)
}

/// Write every additive record the enriched scenario needs into an
/// already-cloned fixture root. Idempotent per clone (each clone is used
/// exactly once).
pub fn apply(root: &Path) -> Result<(), String> {
    let bee = root.join(".bee");

    // 1. state.json — a real phase/mode/feature, gates partly approved, a
    //    non-empty next_action. Lights up: the Phase/Mode/Feature line's
    //    non-`none` branches, the Gates approved/pending mix, the §9
    //    post-execution review line (phase `scribing`), and
    //    `recommended_next`'s POST_EXECUTION_REVIEW_PHASES branch.
    write(&bee.join("state.json"), ENRICHED_STATE_BODY)?;

    // 2. A lane record + a LIVE session bound to it. Lights up:
    //    buildLaneSummary's active-lane resolution (which needs
    //    resolveSessionId's single-fresh-session inference to agree across
    //    both runtimes), the `Lanes:` text line, and `activeWorkers`.
    write(
        &bee.join("lanes").join(format!("{ENRICHED_FEATURE}.json")),
        &format!(
            "{{\"schema_version\":\"1.0\",\"phase\":\"scribing\",\"feature\":\"{ENRICHED_FEATURE}\",\"mode\":\"high-risk\",\"approved_gates\":{{\"context\":true,\"shape\":true,\"execution\":true,\"review\":false}},\"summary\":\"\",\"next_action\":\"\",\"created_at\":null}}"
        ),
    )?;
    let heartbeat = iso_now_millis();
    write(
        &bee.join("sessions").join("parity-live-session.json"),
        &format!(
            "{{\"id\":\"parity-live-session\",\"started_at\":\"{heartbeat}\",\"last_heartbeat\":\"{heartbeat}\",\"lane\":\"{ENRICHED_FEATURE}\"}}"
        ),
    )?;
    // An active claim for that session -> the worker row's `cell` field is
    // a real id rather than null.
    write(
        &bee.join("claims").join("fixture-00001.json"),
        &format!(
            "{{\"cell\":\"fixture-00001\",\"session\":\"parity-live-session\",\"claimed_at\":\"{heartbeat}\",\"ttl_seconds\":3600}}"
        ),
    )?;

    // 3. A deliberately ANCIENT handoff. Lights up: the `Handoff: PRESENT`
    //    line, `recommended_next`'s handoff branch, and the >7-day
    //    staleness warning (whose message interpolates the timestamp — a
    //    declared allowlist member).
    write(
        &bee.join("HANDOFF.json"),
        "{\"written_at\":\"2020-01-02T03:04:05.000Z\",\"kind\":\"pause\",\"summary\":\"parity enrichment handoff\"}",
    )?;

    // 4. A contention log with busy events. Lights up the whole
    //    `contention` block plus its text line — the additive
    //    omit-when-silent field. `.bee/logs` is excluded from the TREE
    //    diff, but it is an INPUT here and its effect lands in stdout,
    //    which is compared in full.
    write(
        &bee.join("logs").join("contention.jsonl"),
        concat!(
            "{\"ts\":\"2026-01-01T00:00:00.000Z\",\"result\":\"busy\",\"lock_name\":\"coordination\",\"holder_session\":\"s-holder\",\"caller_session\":\"s-caller\",\"lock_wait_ms\":1200}\n",
            "{\"ts\":\"2026-01-01T00:00:01.000Z\",\"result\":\"busy\",\"lock_name\":\"coordination\",\"holder_session\":\"s-holder\",\"caller_session\":\"s-caller2\",\"lock_wait_ms\":900}\n",
            "{\"ts\":\"2026-01-01T00:00:02.000Z\",\"result\":\"acquired\",\"lock_name\":\"cells\",\"lock_wait_ms\":4500}\n",
            "{\"ts\":\"2026-01-01T00:00:03.000Z\",\"result\":\"busy\",\"lock_name\":\"cells\",\"holder_session\":null,\"caller_session\":\"s-caller\",\"lock_wait_ms\":30}\n",
        ),
    )?;

    // 5. config.json — standard commands, a gate-bypass level, and a
    //    models map that exercises EVERY `formatSlot`/`normalizeTierValue`
    //    shape at once: bare string, {model,effort}, and a cli executor.
    //    Lights up: the bypass banner line, the `Standard commands:` line,
    //    the `Models (claude):` line, and `normalizeModels`' in-place
    //    override ordering.
    write(
        &bee.join("config.json"),
        concat!(
            "{\n",
            "  \"gate_bypass\": \"full\",\n",
            "  \"commands\": { \"verify\": \"  node scripts/run_verify.mjs  \", \"setup\": \"npm ci\", \"start\": \"\", \"bogus\": \"ignored\" },\n",
            "  \"models\": {\n",
            "    \"claude\": { \"generation\": { \"model\": \"sonnet\", \"effort\": \"medium\" }, \"review\": { \"kind\": \"cli\", \"command\": \"codex exec -m gpt-5.5 -s read-only\" }, \"extraction\": \"haiku\", \"advisor\": \"fable\" },\n",
            "    \"codex\": { \"generation\": { \"model\": \"gpt-5.5\", \"effort\": \"not-a-level\" } }\n",
            "  }\n",
            "}\n",
        ),
    )?;

    // 6. Pending capture stubs -> the `Capture queue:` line and the
    //    `capture_queue.ids` array (including the flush-cancels-a-stub
    //    rule, so the count is a derived number, not a line count).
    write(
        &bee.join("capture-queue.jsonl"),
        concat!(
            "{\"kind\":\"stub\",\"id\":\"stub-b\",\"at\":\"2026-01-02T00:00:00.000Z\",\"text\":\"second\"}\n",
            "{\"kind\":\"stub\",\"id\":\"stub-a\",\"at\":\"2026-01-01T00:00:00.000Z\",\"text\":\"first\"}\n",
            "{\"kind\":\"stub\",\"id\":\"stub-gone\",\"at\":\"2026-01-03T00:00:00.000Z\",\"text\":\"flushed\"}\n",
            "{\"kind\":\"flush\",\"id\":\"stub-gone\",\"at\":\"2026-01-04T00:00:00.000Z\"}\n",
        ),
    )?;

    // 7. The critical-patterns file -> `critical_patterns_present: true`.
    write(
        &root.join("docs").join("history").join("learnings").join("critical-patterns.md"),
        "# Critical patterns\n\n- parity enrichment marker\n",
    )?;

    Ok(())
}

/// The enriched scenario's own seeded mutation — same discipline as
/// `mutate::seed_mutation`: refuse unless the file is byte-for-byte the
/// body [`apply`] wrote, so the flip is a known single-field change and
/// not a blind substitution.
pub fn seed_enriched_mutation(root: &Path) -> Result<(), String> {
    let state_path = root.join(".bee").join("state.json");
    let current = fs::read_to_string(&state_path).map_err(|e| format!("read {}: {e}", state_path.display()))?;
    if current != ENRICHED_STATE_BODY {
        return Err(format!(
            "seed_enriched_mutation: {} does not match the enriched fixture body — refusing to seed a mutation over unexpected content (this would prove nothing)",
            state_path.display()
        ));
    }
    fs::write(&state_path, MUTATED_ENRICHED_STATE_BODY).map_err(|e| format!("write {}: {e}", state_path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enriched_bodies_differ_only_in_phase() {
        assert_eq!(
            ENRICHED_STATE_BODY.replace("\"phase\":\"scribing\"", "\"phase\":\"bee-parity-seeded-mutation\""),
            MUTATED_ENRICHED_STATE_BODY
        );
    }

    #[test]
    fn iso_now_is_shaped_like_an_iso_timestamp() {
        let ts = iso_now_millis();
        assert_eq!(ts.len(), 24, "{ts}");
        assert!(ts.ends_with('Z'));
        assert_eq!(&ts[4..5], "-");
        assert_eq!(&ts[10..11], "T");
    }

    #[test]
    fn civil_from_unix_matches_known_epochs() {
        assert_eq!(civil_from_unix(0), (1970, 1, 1, 0, 0, 0));
        // 2026-07-26T00:00:00Z
        assert_eq!(civil_from_unix(1_785_024_000), (2026, 7, 26, 0, 0, 0));
    }

    #[test]
    fn seed_refuses_over_unexpected_content() {
        let base = std::env::temp_dir().join(format!("bee-parity-enrich-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join(".bee")).unwrap();
        fs::write(base.join(".bee").join("state.json"), b"{}").unwrap();
        let err = seed_enriched_mutation(&base).unwrap_err();
        assert!(err.contains("does not match the enriched fixture body"));
        let _ = fs::remove_dir_all(&base);
    }
}
