//! tokenize — shell word-splitter ported from BOTH
//! `.bee/bin/hooks/tokenize-command.mjs` (`tokenizeCommand`) and
//! `.bee/bin/lib/guards.mjs` (`tokenize`), rust-port-8 (CONTEXT.md D1).
//!
//! Those two mjs files are hand-synced duplicates of the exact same
//! algorithm — guards.mjs's `tokenize()` is the documented source of truth,
//! tokenize-command.mjs's `tokenizeCommand()` is extracted so
//! bee-write-guard.mjs's CLI-shape check can import it without triggering
//! guards.mjs's other top-level machinery. Both are FROZEN for the duration
//! of the rust-port feature (D1); this module is oracle-diffed against BOTH
//! copies by `crates/bee-core/tests/guard_support.rs` (file-based node
//! drivers in `tests/support/`), never edited to "improve" on either one —
//! drift BETWEEN the two mjs copies is exactly what the oracle test is
//! built to surface, not paper over.
//!
//! Algorithm (restated here for the Rust reader, mirrors both mjs copies
//! exactly):
//! - whitespace (space/tab/\n/\r) flushes the current word and is dropped;
//! - a backslash escapes the next character literally (consumed into the
//!   current word, the backslash itself dropped) — including inside what
//!   would otherwise look like whitespace or a separator;
//! - a `"` or `'` opens a quoted span that runs to the next occurrence of
//!   the SAME quote character (or end of string if unterminated); the
//!   quote characters themselves are dropped, their contents copied in
//!   verbatim (no nested escaping inside quotes — this differs from real
//!   shell quoting, matching the mjs source exactly);
//! - adjacent quoted/unquoted segments with no separating whitespace merge
//!   into ONE token (bash word-splitting semantics);
//! - `&&` and `||` are each flushed as their own two-character token;
//! - a lone `;`, `&`, or `|` is flushed as its own one-character token;
//! - everything else is copied into the current word.

/// Port of `tokenizeCommand`/`tokenize`: splits `command` into words and
/// chain-separator tokens (`;`, `&`, `|`, `&&`, `||`), exactly matching the
/// mjs algorithm described in the module doc comment above.
#[allow(unused_assignments)] // the `flush!()` at the very end sets has_current=false with nothing left to read it — a real no-op, not a bug
pub fn tokenize_command(command: &str) -> Vec<String> {
    let chars: Vec<char> = command.chars().collect();
    let len = chars.len();
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut has_current = false;

    macro_rules! flush {
        () => {
            if has_current {
                tokens.push(std::mem::take(&mut current));
                has_current = false;
            }
        };
    }

    let mut i = 0usize;
    while i < len {
        let ch = chars[i];
        if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
            flush!();
            i += 1;
            continue;
        }
        if ch == '\\' && i + 1 < len {
            current.push(chars[i + 1]);
            has_current = true;
            i += 2;
            continue;
        }
        if ch == '"' || ch == '\'' {
            // Find the next occurrence of the SAME quote char after i, or
            // end of string — matching `str.indexOf(ch, i + 1)` (-1 falls
            // back to `str.length`).
            let mut close = None;
            let mut j = i + 1;
            while j < len {
                if chars[j] == ch {
                    close = Some(j);
                    break;
                }
                j += 1;
            }
            let end = close.unwrap_or(len);
            for c in &chars[(i + 1)..end] {
                current.push(*c);
            }
            has_current = true;
            i = end + 1;
            continue;
        }
        if (ch == '&' && i + 1 < len && chars[i + 1] == '&')
            || (ch == '|' && i + 1 < len && chars[i + 1] == '|')
        {
            flush!();
            let mut pair = String::new();
            pair.push(ch);
            pair.push(ch);
            tokens.push(pair);
            i += 2;
            continue;
        }
        if ch == ';' || ch == '&' || ch == '|' {
            flush!();
            tokens.push(ch.to_string());
            i += 1;
            continue;
        }
        current.push(ch);
        has_current = true;
        i += 1;
    }
    flush!();
    tokens
}

// Tests live in crates/bee-core/tests/guard_support.rs (this cell's single
// integration target — cargo test -p bee-core --test guard_support) rather
// than here, so the oracle-diffed proof against BOTH mjs copies and the
// plain-Rust sanity cases sit in one place per must-have.
