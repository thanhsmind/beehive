// bee hook write-guard — Rust port of hooks/bee-write-guard.mjs (+ its
// tokenize-command.mjs helper), bee's most safety-critical hook. PreToolUse
// for Edit|Write|MultiEdit|Bash|Read|Glob|Grep|AskUserQuestion plus the Codex
// apply_patch path. Four checks in one guard, first hit wins — see the .mjs
// header for the (a)-(d) map; every branch below carries a provenance comment
// naming its .mjs source.
//
// GUARD PORTING CONTRACT (rust-port.md C2/C5): a decision (allow vs deny)
// must NEVER flip relative to the .mjs. Every branch whose Rust equivalence
// cannot be PROVEN returns Outcome::Delegate (the dispatcher re-runs the
// original Node wrapper with identical stdin). Failing toward Node is always
// safe; failing open differently than the .mjs never is.
//
// THE VENDORED-LIB BYTE GATE: the .mjs dynamically imports vendored lib
// modules from <storeRoot>/.bee/bin/lib/*.mjs. This port replicates the
// CURRENT packages/bee/lib implementations; if the vendored files differ in
// any byte (mid-upgrade host, tampered fixture, a test whose guards.mjs
// deliberately throws on import), the native semantics are unproven and the
// whole run delegates. The import closure of state.mjs + guards.mjs +
// validate-args.mjs + command-registry.mjs is embedded at compile time and
// byte-compared at runtime before any native decision is made.
//
// CHECK (d) IS NATIVE (R6 blocker closed): the CLI-shape schema guard lives in
// hooks/cli_shape.rs — registry resolution + validate-args semantics + the
// exact refusal bytes, resolved against the embedded REGISTRY_PAYLOAD that the
// byte gate above proves matches the host's own command-registry.mjs. It also
// recognizes the R6a BINARY spelling (`.bee/bin/bee <verb>` / `bee <verb>`)
// that no `.mjs` regex could see; that is the one deliberate divergence from
// the .mjs and it only ever ADDS a denial for a call Node left unguarded —
// see cli_shape.rs's header.
//
// CORRUPT JSON IS NATIVE (cutover 2026-08-01): readJson()-level corruption
// used to delegate because Node's warn quoted V8's parse message. It now warns
// in bee's own words and takes readJson's `null` fallback — see read_json_g.
// The warning is queued and flushed with the rest of the buffered output, so
// the delegate contract below still holds byte-for-byte. A corrupt CONFIG file
// is native too, inside crate::state::read_config_raw — but that reader prints
// immediately, so a run that reads a bad config and THEN delegates for one of
// the reasons below can leak that one line (accepted: the remaining delegates
// are themselves being retired).
//
// DELEGATED BRANCHES (each justified at its site):
//   - node -e/--eval/-p inline-eval commands (internals-reach regex);
//   - companion-mount resolution when .bee/companion-session.json exists and
//     the target already failed containment;
//   - a declared guards.memory_root (non-empty string) when a target failed
//     containment;
//   - drive-relative (C:foo) / UNC (\\srv\...) target spellings on Windows;
//   - JS-throw equivalents inside the shared-nested-checkout primitive
//     (strict-mode session reads, non-ENOENT fs errors) — Node turns those
//     into a typed deny plus a V8-worded crash log line;
//   - timestamp strings chrono cannot parse where JS Date.parse might.
//
// Output is fully buffered: nothing is written before the native/delegate
// decision is final, so Delegate always re-runs Node with zero output emitted.








const HOOK_NAME: &str = "write-guard";

// ─── strangler bail ────────────────────────────────────────────────────────

/// "Needs Node": the branch's Rust equivalence is unproven — delegate.
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
// Mirrors packages/bee/hooks/test_write_guard.mjs's decision table with
// tempfile fixtures. `copy_lib` used to vendor the embedded `.mjs` closure so
// the byte gate would pass; with the gate retired it writes the onboarding
// marker instead — the fixture's job is still "make this look like a repo
// where bee is installed", only the marker changed.

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
