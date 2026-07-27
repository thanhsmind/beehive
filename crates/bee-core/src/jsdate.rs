//! jsdate — a minimal `Date.parse`-equivalent for the ISO-8601/RFC3339
//! UTC timestamps bee's mjs stores write (`new Date().toISOString()`
//! everywhere: holds' `mirrored_at`, claims' `last_heartbeat`, leases'
//! `acquired_at`/`expires_at`, ...). Shared by [`crate::holds`] and
//! [`crate::claims`] rather than duplicated per module, since both need
//! the exact same "unparseable/absent timestamp -> None" fallback
//! semantics mjs's `Number.isFinite(Date.parse(...))` guard expresses.
//!
//! Deliberately narrow: only the `"YYYY-MM-DDTHH:MM:SS(.sss)?Z"` shape
//! `toISOString()` always produces is accepted — this is a reader for
//! bee's OWN store output, never a general-purpose date parser, and no
//! subprocess/date-library dependency is pulled in for it (D5: zero
//! subprocess spawns on the hot paths this crate serves).

/// Milliseconds since the Unix epoch for an ISO-8601 UTC timestamp of the
/// shape `toISOString()` produces, or `None` if the string doesn't match
/// (mirrors `Number.isFinite(Date.parse(s))` being false).
pub fn parse_iso_ms(s: &str) -> Option<i64> {
    let bytes = s.as_bytes();
    if bytes.len() < 20 || bytes[4] != b'-' || bytes[7] != b'-' || bytes[10] != b'T' {
        return None;
    }
    let year: i64 = s.get(0..4)?.parse().ok()?;
    let month: i64 = s.get(5..7)?.parse().ok()?;
    let day: i64 = s.get(8..10)?.parse().ok()?;
    let hour: i64 = s.get(11..13)?.parse().ok()?;
    let min: i64 = s.get(14..16)?.parse().ok()?;
    let sec: i64 = s.get(17..19)?.parse().ok()?;
    let mut millis: i64 = 0;
    let mut idx = 19;
    if s.as_bytes().get(idx) == Some(&b'.') {
        let start = idx + 1;
        let mut end = start;
        while s.as_bytes().get(end).is_some_and(|c| c.is_ascii_digit()) {
            end += 1;
        }
        let frac = &s[start..end];
        if frac.is_empty() {
            return None;
        }
        let frac3 = format!("{frac:0<3}");
        millis = frac3[..3].parse().ok()?;
        idx = end;
    }
    if s.as_bytes().get(idx) != Some(&b'Z') {
        return None; // only the always-UTC "Z" form bee's own writers produce
    }

    // Days-since-epoch via Howard Hinnant's civil_from_days algorithm
    // (proleptic Gregorian, matches JS Date's own calendar for this range).
    fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
        let y = if m <= 2 { y - 1 } else { y };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400; // [0, 399]
        let mp = (m + 9) % 12; // [0, 11]
        let doy = (153 * mp + 2) / 5 + d - 1; // [0, 365]
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
        era * 146097 + doe - 719468
    }
    let days = days_from_civil(year, month, day);
    let secs = days * 86400 + hour * 3600 + min * 60 + sec;
    Some(secs * 1000 + millis)
}


// Tests live in crates/bee-core/tests/guard_support.rs (this cell's single
// integration target — cargo test -p bee-core --test guard_support) rather
// than here, so every reader's round-trip/logic proof sits in one place
// per must-have.
