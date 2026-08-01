// link_invalid — the native owner of `WorktreeLinkInvalidError` (a BROKEN
// linked-worktree link: a `.git` pointer whose bidirectional validation
// fails). roots.rs already builds all four of its message shapes byte-for-byte
// and pins them against Node; until this module existed nothing EMITTED them —
// every verb turned `Resolution::LinkInvalid` into a delegation.
//
// WHY IT COULD NOT BE NATIVE BEFORE, AND WHY IT CAN NOW. bee.mjs resolves the
// repo root inside main():
//
//     let root; try { root = findRepoRoot(process.cwd()); ... }
//     catch (error) { return emitError(error.message, jsonRequested); }
//
// so the THROW is caught and the message is emitted exactly like any other
// refusal: `{"error": "<message>"}` on stdout with --json, the bare message on
// stderr otherwise, exit 1. That half is fully deterministic.
//
// The half that forced a delegation is the direct-run TIMING wrapper. It calls
// findRepoRoot AGAIN, inside its own try:
//
//     try { const timingRoot = findRepoRoot(process.cwd()) || process.cwd();
//           fs.mkdirSync(logsDir, ...); fs.appendFileSync(timings.jsonl, ...); }
//     catch { /* fail-open */ }
//     process.stderr.write(`[bee] ${timingCmd} ${ms}ms\n`);
//
// A broken link makes that SECOND findRepoRoot throw too, so the timings.jsonl
// append is skipped while the `[bee] …` stderr line still prints. The shared
// Rust wrapper (verbs::record_timing) always appends, so reproducing this
// meant bypassing it — which is why the whole command went back to Node.
//
// With Node deleted there is nothing to delegate TO, so the constraint dies:
// `emit_link_invalid` below simply does what Node does — emit, then print the
// `[bee] <cmd> <ms>ms` line WITHOUT the timings.jsonl append. No divergence:
// the skipped append is Node's own behavior, reproduced deliberately rather
// than inherited from a shared wrapper.
//
// TIMING-LINE DIVERGENCE, NAMED. One byte-level difference remains and is
// accepted here rather than papered over: Node's `timingCmd` is resolved by
// `splitCommandTokens` + `resolveCommand` over the RAW argv, so an argv that
// resolves to no command logs `unknown`; this module is called from a verb
// probe that already knows its own command name, so it always prints the real
// one. The two agree for every argv a native verb serves (the probe only runs
// once its command resolved), and the line is stderr-only telemetry that no
// `--json` consumer and no test pins.
//
// RESIDUAL SCOPE, stated plainly: only the callers that route through here are
// native for a broken link. `verbs/worktree.rs` (all five worktree verbs) is
// wired as of this cell. Every other verb's prelude — and roots.rs's own
// `resolve_store_root`/`resolve_store_root_worktree` `NeedsNode` arms — still
// treats LinkInvalid as "not mine"; each becomes native by calling this one
// function, never by growing a second copy of the emission.

use crate::jsjson;
use serde_json::json;
use std::process::ExitCode;
use std::time::Instant;

/// bee.mjs main()'s `catch (error) { return emitError(error.message, ... ) }`
/// for a WorktreeLinkInvalidError, plus the direct-run wrapper's stderr timing
/// line with NO timings.jsonl append (see the module header for why that
/// absence is Node's behavior, not a shortcut).
pub(crate) fn emit_link_invalid(
    message: &str,
    cmd: &str,
    use_json: bool,
    t0: Instant,
) -> ExitCode {
    if use_json {
        println!("{}", jsjson::stringify(&json!({ "error": message })));
    } else {
        eprintln!("{message}");
    }
    eprintln!("[bee] {cmd} {}ms", t0.elapsed().as_millis() as u64);
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The one thing worth pinning here is the exit code and that the call is
    /// total — the message bytes themselves are roots.rs's, already pinned
    /// against Node over real `git worktree add` fixtures.
    #[test]
    fn a_broken_link_exits_one() {
        let code = emit_link_invalid("boom", "worktree list", false, Instant::now());
        assert_eq!(format!("{code:?}"), format!("{:?}", ExitCode::FAILURE));
    }
}
