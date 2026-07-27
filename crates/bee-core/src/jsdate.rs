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


// ─── the WIDER `Date.parse`, for USER-SUPPLIED date arguments (rpl-5) ───────
//
// [`parse_iso_ms`] above deliberately accepts only the
// `"YYYY-MM-DDTHH:MM:SS(.sss)?Z"` shape, because every value it reads was
// written by bee's own `toISOString()` calls. `decisions archive --before`
// and `decisions search/active --since` are different: those strings come
// from a HUMAN, and mjs runs them through the real `Date.parse`. The
// registry's own documented example is `bee decisions archive --before
// 2099-01-01` (`command-registry.mjs:618`) — a DATE-ONLY string
// `parse_iso_ms` rejects and `Date.parse` accepts as UTC midnight. Reusing
// the narrow reader for those flags would have refused the one call the
// manifest advertises.
//
// DECLARED, NOT MASKED — and every declared form below is FAIL-CLOSED:
// this parser returns `None`, its call sites refuse with a non-zero exit,
// and the store is left byte-untouched. There is no form on which it
// silently produces a DIFFERENT instant from V8, which matters because
// `decisions archive` DELETES rows and has no inverse verb.
//
// 1. **A date-time with no timezone designator is LOCAL in ECMA-262**
//    (`2026-07-26T00:00:00`) and is REFUSED here.
//
//    This started life the other way round — read as UTC — and rpl-5's
//    goal-check reproduced what that cost. On a host at `TZ=Asia/Bangkok`,
//    `decisions archive --before 2026-07-26T00:00:00 --json` made mjs
//    archive one row and this port archive two, BOTH exiting 0 with no
//    warning, because V8 read the cutoff as `+07:00` (17:00Z the previous
//    day) and this read it as `00:00Z`. On a negative-offset host the same
//    error inverts into keeping a row mjs would have deleted.
//
//    Refusing is deliberate and is NOT a lesser fix. Reproducing ECMA-262
//    local time needs a local-offset source `std` does not have; guessing
//    one (an env `TZ` read, a fixed offset) would trade a divergence a
//    cross-runtime harness can SEE for one it cannot. A refusal is loud, is
//    asserted, and costs nothing on the documented surface — date-only
//    forms ARE UTC in ECMA-262, so `--before 2099-01-01` is unaffected.
//    Pinned by `date_parse_offsetless_datetime_is_refused` here and, end to
//    end through both real binaries, by
//    `queen-bee/tests/decisions_composed_oracle.rs`.
// 2. **V8's legacy fallback parser, plus the ES-grammar corners this
//    deliberately does not implement.** A string that fails the ES grammar
//    is handed to an implementation-specific parser ES leaves entirely
//    unconstrained. It is more reachable than it looks — every value here
//    was MEASURED against the frozen oracle, not assumed. All are finite in
//    V8 and `None` here:
//
//    - `2026/01/01`, `2026-1-1`, `Jan 1 2099` — legacy date spellings
//    - `2026-02-30` — legacy ROLL-OVER to March 2nd rather than a refusal
//    - `2026-07-26 00:00:00` — a SPACE where the format requires `T`
//    - `2026-07-26T00:00:00z` — a LOWERCASE designator
//    - `2026-07-26T00:00:00+0700` — an offset with no `:` separator
//    - `+002026-07-26`, `+002026-07-26T00:00:00Z` — EXPANDED (six-digit,
//      signed) years
//
//    The last four are the same fail-closed class as the first four and are
//    listed for that reason: not implementing them is a choice, and an
//    undeclared choice is indistinguishable from an oversight. Reproducing
//    an undocumented parser is not something a port can do faithfully, so
//    all of it is declared instead — both flags document their argument as
//    an "ISO date" — and the ES-grammar refusals both runtimes DO share
//    (`2026-13-01`, `2026-00-01`, `2026-01-00`, `T25:00:00Z`, `T00:60:00Z`,
//    `""`, `"true"`) are pinned below.
//
//    One consequence is recorded rather than removed: for a legacy form
//    like `2026/01/01` both runtimes exit 1 and leave the store untouched,
//    but with DIFFERENT wording — mjs accepts the cutoff and then finds
//    nothing qualifying, while this refuses the cutoff itself. That text
//    divergence is pinned as a declared artifact in
//    `decisions_composed_oracle.rs` rather than being chased into parity.

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Proleptic-Gregorian days since the epoch (Howard Hinnant's
/// `days_from_civil`), the same algorithm [`parse_iso_ms`] uses inline.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn all_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// `Date.parse(s)` over the ECMA-262 Date Time String Format:
/// `YYYY`, `YYYY-MM`, `YYYY-MM-DD`, each optionally followed by
/// `THH:mm`, `THH:mm:ss` or `THH:mm:ss.sss`, each optionally followed by
/// `Z` or `±HH:mm`. `None` is the `NaN` every mjs call site then rejects
/// with `Number.isFinite`.
///
/// Out-of-range components are `NaN` in the ISO grammar (`2026-02-30`,
/// `2026-13-01`), not a rolled-over date — so they are `None` here.
pub fn date_parse_ms(s: &str) -> Option<i64> {
    // The date half runs to the `T` (the only separator the format allows);
    // everything after it is time + optional offset.
    let (date_part, rest) = match s.find('T') {
        Some(i) => (&s[..i], Some(&s[i + 1..])),
        None => (s, None),
    };

    let mut fields = date_part.split('-');
    let year_raw = fields.next()?;
    if year_raw.len() != 4 || !all_digits(year_raw) {
        return None; // expanded ±YYYYYY years are not part of bee's surface
    }
    let year: i64 = year_raw.parse().ok()?;
    let month: i64 = match fields.next() {
        None => 1,
        Some(m) if m.len() == 2 && all_digits(m) => m.parse().ok()?,
        Some(_) => return None,
    };
    let day: i64 = match fields.next() {
        None => 1,
        Some(d) if d.len() == 2 && all_digits(d) => d.parse().ok()?,
        Some(_) => return None,
    };
    if fields.next().is_some() {
        return None;
    }
    if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) {
        return None;
    }

    let (mut hour, mut min, mut sec, mut millis) = (0i64, 0i64, 0i64, 0i64);
    let mut offset_minutes = 0i64;
    if let Some(rest) = rest {
        // Split the offset designator off the tail first: `Z`, or the LAST
        // `+`/`-` (a fractional-second run can never contain either).
        let (time, offset) = if let Some(stripped) = rest.strip_suffix('Z') {
            (stripped, Some(""))
        } else if let Some(i) = rest.rfind(['+', '-']) {
            (&rest[..i], Some(&rest[i..]))
        } else {
            (rest, None)
        };

        let mut tf = time.split(':');
        let h = tf.next()?;
        if h.len() != 2 || !all_digits(h) {
            return None;
        }
        hour = h.parse().ok()?;
        let m = tf.next()?; // `THH` alone is not a legal time
        if m.len() != 2 || !all_digits(m) {
            return None;
        }
        min = m.parse().ok()?;
        if let Some(secs) = tf.next() {
            let (whole, frac) = match secs.split_once('.') {
                Some((w, f)) => (w, Some(f)),
                None => (secs, None),
            };
            if whole.len() != 2 || !all_digits(whole) {
                return None;
            }
            sec = whole.parse().ok()?;
            if let Some(frac) = frac {
                if !all_digits(frac) {
                    return None;
                }
                // `Date.parse` reads exactly the first three fraction
                // digits and TRUNCATES the rest.
                let frac3 = format!("{frac:0<3}");
                millis = frac3[..3].parse().ok()?;
            }
        }
        if tf.next().is_some() {
            return None;
        }
        // Hour 24 is legal only as the exact instant 24:00:00.000.
        if hour > 24 || (hour == 24 && (min != 0 || sec != 0 || millis != 0)) || min > 59 || sec > 59 {
            return None;
        }

        match offset {
            // `Z` — the one designator that unambiguously means UTC.
            Some("") => {}
            // NO designator at all. ECMA-262 reads this as LOCAL time, and
            // there is no local-offset source in `std` — so this REFUSES
            // rather than silently picking a reading. See DECLARED
            // DIVERGENCE 1 above for why refusing is the only safe answer
            // here and why the date-only forms above are unaffected.
            None => return None,
            Some(raw) => {
                let sign = if raw.starts_with('-') { -1 } else { 1 };
                let body = &raw[1..];
                let (oh, om) = body.split_once(':')?;
                if oh.len() != 2 || om.len() != 2 || !all_digits(oh) || !all_digits(om) {
                    return None;
                }
                let oh: i64 = oh.parse().ok()?;
                let om: i64 = om.parse().ok()?;
                if oh > 23 || om > 59 {
                    return None;
                }
                offset_minutes = sign * (oh * 60 + om);
            }
        }
    }

    let days = days_from_civil(year, month, day);
    let secs = days * 86400 + hour * 3600 + min * 60 + sec;
    Some(secs * 1000 + millis - offset_minutes * 60_000)
}

#[cfg(test)]
mod date_parse_tests {
    use super::date_parse_ms;

    /// The value `parse_iso_ms` refuses and the registry's own example
    /// depends on: a bare date is UTC midnight.
    #[test]
    fn date_only_is_utc_midnight() {
        // Every value here was MEASURED through `node -e 'Date.parse(…)'`
        // against the frozen runtime, never derived by hand.
        assert_eq!(date_parse_ms("1970-01-01"), Some(0));
        assert_eq!(date_parse_ms("2026-07-26"), Some(1785024000000));
        assert_eq!(date_parse_ms("2099-01-01"), Some(4070908800000));
        assert_eq!(date_parse_ms("2026-07"), Some(1782864000000));
        assert_eq!(date_parse_ms("2026"), Some(1767225600000));
    }

    #[test]
    fn full_toisostring_shape_agrees_with_the_narrow_reader() {
        for s in ["2026-07-26T00:00:00.000Z", "1970-01-01T00:00:00.000Z", "2026-12-31T23:59:59.999Z"] {
            assert_eq!(date_parse_ms(s), super::parse_iso_ms(s), "disagreement on {s}");
        }
    }

    #[test]
    fn explicit_offsets_shift_the_instant() {
        assert_eq!(date_parse_ms("2026-07-26T07:00:00+07:00"), date_parse_ms("2026-07-26T00:00:00Z"));
        assert_eq!(date_parse_ms("2026-07-25T19:00:00-05:00"), date_parse_ms("2026-07-26T00:00:00Z"));
    }

    /// The refusals BOTH runtimes share: each of these is NaN in V8 too
    /// (measured), so agreeing on `None` here is real parity, not a
    /// coincidence.
    #[test]
    fn shared_nan_refusals() {
        for s in [
            "2026-13-01",
            "2026-00-01",
            "2026-01-00",
            "26-01-01",
            "",
            "true",
            "not a date",
            "2026-01-01T25:00:00Z",
            "2026-01-01T24:00:01Z",
            "2026-01-01T00:60:00Z",
            "2026-01-01T00",
        ] {
            assert_eq!(date_parse_ms(s), None, "expected NaN for {s:?}");
        }
        // 24:00:00.000 IS legal, and is the next midnight (V8: 1767312000000).
        assert_eq!(date_parse_ms("2026-01-01T24:00:00.000Z"), Some(1767312000000));
        assert_eq!(date_parse_ms("2026-01-01T24:00:00.000Z"), date_parse_ms("2026-01-02T00:00:00Z"));
        // Fraction digits past the third are TRUNCATED, not rounded
        // (V8: 1798761599999 for `.9999`).
        assert_eq!(date_parse_ms("2026-12-31T23:59:59.9999Z"), Some(1798761599999));
    }

    /// DECLARED DIVERGENCE 2 (see the module note above): every one of these
    /// is FINITE in V8 — `2026-02-30` even rolls over to March 2nd — and
    /// `None` here. The list is the parser's declared boundary, so it is
    /// spelled out in full rather than sampled: the rpl-5 goal-check found
    /// the previous version INCOMPLETE, with four measured same-class forms
    /// (lowercase designator, space separator, colon-less offset, expanded
    /// year) fail-closed in practice but undeclared on paper.
    #[test]
    fn date_parse_does_not_reproduce_the_v8_legacy_fallback() {
        for s in [
            // legacy date spellings
            "2026-02-30",
            "2026-1-1",
            "2026/01/01",
            "Jan 1 2099",
            // a SPACE where the ES format requires `T`
            "2026-07-26 00:00:00",
            // a LOWERCASE UTC designator
            "2026-07-26T00:00:00z",
            // an offset with no `:` separator
            "2026-07-26T00:00:00+0700",
            // EXPANDED (six-digit, signed) years, with and without a time
            "+002026-07-26",
            "+002026-07-26T00:00:00Z",
        ] {
            assert_eq!(date_parse_ms(s), None, "{s:?} is a declared divergence, not a supported form");
        }
    }

    /// DECLARED DIVERGENCE 1 (see the module note above): ECMA-262 reads an
    /// offset-less date-TIME as LOCAL, and there is no local-offset source in
    /// `std` — so this refuses instead of guessing a reading.
    ///
    /// The `assert_ne!` is the point of the test, not decoration: the
    /// previous behaviour was exactly `date_parse_ms(x) ==
    /// date_parse_ms(format!("{x}Z"))`, and that equality is what silently
    /// over-archived a row on a `+07:00` host.
    #[test]
    fn date_parse_offsetless_datetime_is_refused() {
        for s in [
            "2026-07-26T00:00",
            "2026-07-26T00:00:00",
            "2026-07-26T00:00:00.000",
            "2026-07-26T24:00:00.000",
        ] {
            assert_eq!(date_parse_ms(s), None, "{s:?} has no designator and must be refused, not assumed UTC");
            assert_ne!(
                date_parse_ms(s),
                date_parse_ms(&format!("{s}Z")),
                "{s:?} must NOT resolve to the same instant as its Z-suffixed form"
            );
        }
    }

    /// The other side of the same boundary: a DATE-ONLY string carries no
    /// time and IS UTC in ECMA-262, so refusing the offset-less date-TIME
    /// above must not touch it. This is the registry's own documented
    /// `bee decisions archive --before 2099-01-01`, so a regression here
    /// would break a manifest-as-tested contract.
    #[test]
    fn refusing_offsetless_datetimes_leaves_date_only_forms_intact() {
        assert_eq!(date_parse_ms("2099-01-01"), Some(4070908800000));
        assert_eq!(date_parse_ms("2099-01-01"), date_parse_ms("2099-01-01T00:00:00.000Z"));
        assert_eq!(date_parse_ms("2026-07"), Some(1782864000000));
        assert_eq!(date_parse_ms("2026"), Some(1767225600000));
    }
}

// Tests live in crates/bee-core/tests/guard_support.rs (this cell's single
// integration target — cargo test -p bee-core --test guard_support) rather
// than here, so every reader's round-trip/logic proof sits in one place
// per must-have.
