// bee hook write-guard — bee's most safety-critical hook. PreToolUse for
// Edit|Write|MultiEdit|Bash|Read|Glob|Grep|AskUserQuestion plus the Codex
// apply_patch path. Four checks in one guard, first hit wins.
//
// Every branch whose Rust equivalence is unproven for the input in front of
// it returns Outcome::Delegate — see hooks/mod.rs for what that resolves to
// now (fail open, loudly). Failing toward Delegate is always safe; a native
// decision that turns out wrong never is, so an unproven shape always
// delegates rather than guessing.
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
// CORRUPT JSON IS NATIVE: a read that finds present-but-unparseable JSON
// warns in bee's own words and takes the same fallback an absent file would
// — see read_json_g. The warning is queued and flushed with the rest of the
// buffered output, so the delegate contract below still holds byte-for-byte.
// A corrupt CONFIG file is native too, inside crate::state::read_config_raw
// — but that reader prints immediately, so a run that reads a bad config and
// THEN delegates for one of the reasons below can leak that one line
// (accepted: the remaining delegates are themselves being retired).
//
// DELEGATED BRANCHES (each justified at its site):
//   - node -e/--eval/-p inline-eval commands (internals-reach regex);
//   - companion-mount resolution when .bee/companion-session.json exists and
//     the target already failed containment;
//   - a declared guards.memory_root (non-empty string) when a target failed
//     containment;
//   - drive-relative (C:foo) / UNC (\\srv\...) target spellings on Windows;
//   - a small set of typed-refusal edges inside the shared-nested-checkout
//     primitive (non-ENOENT fs errors while walking the target's ancestry).
//     Its STRICT SESSION READ left this list at cell sfg-5: an unreadable
//     `.bee/sessions/<id>.json` is a native DENY now, in bee's own wording,
//     because a guard never falls open on data it merely read — see
//     `unreadable_session_refusal` (hook_local.rs) for the deliberate
//     departure from Node's V8-worded crash log;
//   - timestamp strings chrono cannot parse.
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
