// bee hook write-guard — bee's most safety-critical hook. PreToolUse for
// Edit|Write|MultiEdit|Bash|Read|Glob|Grep|AskUserQuestion plus the Codex
// apply_patch path. Four checks in one guard, first hit wins.
//
// Every branch whose Rust equivalence is unproven for the input in front of
// it returns Outcome::Delegate — see hooks/mod.rs for what that resolves to
// now (fail open, loudly). An unproven SHAPE delegates rather than guessing,
// because a native decision that turns out wrong is never safe.
//
// DELEGATING IS NOT ALWAYS SAFE, and the slp-followup-gaps cells (sfg-1,
// sfg-3, sfg-4, sfg-5, sfg-6) are the record of why this header used to say
// it was. A Delegate switches the WHOLE guard off for that call — every
// check, every path, every .bee mutation — so on data the guard merely READ,
// delegating is the fail-OPEN answer, not the cautious one. One malformed
// timestamp, one broken .git line, one truncated session record or one
// corrupt companion marker turned the guard off on ordinary work. The rule
// those five cells settled: A GUARD NEVER FALLS OPEN ON DATA IT MERELY READ.
// Every store and config reader reachable from resolve_write_record or
// check_write is now infallible by signature, or answers with a native
// refusal naming the file it could not read: NO reader turns unreadable data
// into a Delegate. What still delegates is listed below, and every entry is a
// SHAPE whose Rust equivalence is unproven — a declared guards.memory_root is
// read perfectly well and delegates on what it SAYS — never a record the
// guard could not read.
//
// The vendored-lib byte gate that used to prove the host's on-disk copy
// matched what this binary embeds is RETIRED — see
// hooks/write_guard/jspath.rs's header for why, and for what replaced its
// activation probe.
//
// CHECK (d) IS NATIVE: the CLI-shape schema guard lives in hooks/cli_shape.rs
// — registry resolution + validate-args semantics + the exact refusal bytes,
// resolved against the embedded REGISTRY_PAYLOAD (compiled in, pinned by
// tests/registry_contracts.rs). It also recognizes the BINARY spelling
// (`.bee/bin/bee <verb>` / `bee <verb>`) alongside the legacy
// `bee_<group>.mjs` shape; see cli_shape.rs's header for the one deliberate
// divergence that widening introduces.
//
// CORRUPT JSON IS NATIVE — in every reader this guard has, never delegated,
// in one of two answer shapes:
//   - where the absent-file fallback claims nothing the guard spends, the
//     reader warns in bee's own words and takes that fallback — see
//     read_json_g;
//   - where the absent-file fallback would be a POSITIVE claim the reader
//     cannot back, it is a native DENY naming the file and its remedy. Two
//     records are read that way: .bee/sessions/<id>.json on the strict
//     concurrency scan (sfg-5), where "absent" means "no live peer", and
//     .bee/companion-session.json on the companion-mount check (sfg-6),
//     where "absent" means "no verified mount". Both of those claims are
//     what lets a write through, so neither is granted off bytes the reader
//     could not parse — see unreadable_session_refusal and
//     unreadable_companion_marker_refusal (hook_local.rs).
//
// Those warnings are queued and flushed with the rest of the buffered output,
// so the delegate contract below still holds byte-for-byte.
// A corrupt CONFIG file is native too, inside crate::state::read_config_raw
// — but that reader prints immediately, so a run that reads a bad config and
// THEN delegates for one of the reasons below can leak that one line
// (accepted: the remaining delegates are themselves being retired).
//
// DELEGATED BRANCHES (each justified at its site):
//   - node -e/--eval/-p inline-eval commands (internals-reach regex);
//   - companion-mount resolution when .bee/companion-session.json exists and
//     the target already failed containment. The marker's mere PRESENCE
//     decides this branch — a perfect marker and a corrupt one delegate
//     identically — so no unreadable data decides it (pinned by
//     sfg6_the_containment_gated_delegate_never_turned_on_readability);
//   - a declared guards.memory_root (non-empty string) when a target failed
//     containment;
//   - drive-relative (C:foo) / UNC (\\srv\...) target spellings on Windows;
//   - a bash command whose tokenizer walk hit the nesting cap and truncated
//     (checks.rs `check_git_bash_command`, reached from main.rs) — the
//     delegate is on the COMMAND's shape, not on anything read from disk;
//   - a non-ASCII AskUserQuestion header (detectors.rs), likewise a shape of
//     the tool payload;
//   - a small set of typed-refusal edges inside the shared-nested-checkout
//     primitive: non-ENOENT realpath/stat errors on the target, on its
//     ancestry, on the checkout root, or on the two paths a READABLE
//     companion marker names; a non-ENOENT read error on .gitmodules (the
//     submodule-registration exclusion); and a process cwd the path
//     arithmetic cannot read. Both STORE RECORDS this primitive reads have
//     left the list: an unreadable .bee/sessions/<id>.json at cell sfg-5 and
//     an unreadable .bee/companion-session.json at cell sfg-6 are native
//     DENIES now, in bee's own wording, because a guard never falls open on
//     data it merely read — see `unreadable_session_refusal` and
//     `unreadable_companion_marker_refusal` (hook_local.rs) for the
//     deliberate departure from Node's V8-worded crash log.
//
// A "timestamp strings chrono cannot parse" bullet stood here until cell
// sfg-6 removed it. It was true when it was written and false by then: sfg-3,
// sfg-4 and sfg-5 rewrote every date_parse_ms caller in this guard — claims,
// heartbeats, leases, holds, reservations — to answer instead of propagate,
// so not one unparseable stamp reaches Delegate any more.
//
// Output is fully buffered: nothing is written before the native/delegate
// decision is final, so a Delegate outcome always carries zero output.








const HOOK_NAME: &str = "write-guard";

// ─── delegate refusal ───────────────────────────────────────────────────────

/// The branch's Rust equivalence is unproven — delegate.
///
/// pub(crate) since the wcg-3 port: `crate::nested_checkout` reuses this
/// module's shared-nested-checkout primitives (the guard is the ONE place
/// that verification lives — re-deriving it there would fork the guard, the
/// drift C5 exists to prevent), so it has to be able to name the error type
/// those primitives return. It maps `Nd` onto a native fail-closed refusal
/// rather than a delegation; see that module's header.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Nd;

pub(crate) type R<T> = Result<T, Nd>;

// ─── tests ─────────────────────────────────────────────────────────────────
// A tempfile-fixture decision table. `copy_lib` writes the onboarding marker
// that makes a fixture look like "a repo where bee is installed" (the
// vendored-lib byte gate it used to satisfy is retired).

#[cfg(test)]
mod tests;

mod jspath;
mod store;
mod guards;
mod paths;
mod checks;
mod hook_local;
mod detectors;
mod main;
pub(crate) use self::jspath::*;
pub(crate) use self::store::*;
pub(crate) use self::guards::*;
pub(crate) use self::paths::*;
pub(crate) use self::checks::*;
pub(crate) use self::hook_local::*;
pub(crate) use self::detectors::*;
pub(crate) use self::main::*;
