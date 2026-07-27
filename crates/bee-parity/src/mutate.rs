//! Seeded-mutation support for the harness's own red-first self-check
//! (CONTEXT.md D7a advisor note 2 / cell must_have: "seeded-mutation run
//! reports a diff and the harness self-check treats missing detection as
//! failure"). Proves the differ's detection power the same way a red-first
//! test proves a check can fail: introduce a real, known divergence and
//! confirm it is caught before trusting a zero-diff result anywhere else.

use std::fs;
use std::path::Path;

/// The exact fixture content written by `queen-bench::fixture::write_state`
/// (crates/queen-bench/src/fixture.rs) — duplicated here as a literal
/// (not imported: bee-parity has no dependency on queen-bench, std-only
/// crate boundary, rust-port-1/2 precedent) so the mutation is a precise,
/// legible single-field flip rather than a blind text substitution.
const FIXTURE_STATE_BODY: &str = "{\"schema_version\":\"1.0\",\"phase\":\"idle\",\"feature\":null,\"mode\":null,\"approved_gates\":{\"context\":false,\"shape\":false,\"execution\":false,\"review\":false},\"workers\":[],\"summary\":\"\",\"next_action\":\"No active bee work.\"}";

const MUTATED_STATE_BODY: &str = "{\"schema_version\":\"1.0\",\"phase\":\"bee-parity-seeded-mutation\",\"feature\":null,\"mode\":null,\"approved_gates\":{\"context\":false,\"shape\":false,\"execution\":false,\"review\":false},\"workers\":[],\"summary\":\"\",\"next_action\":\"No active bee work.\"}";

/// Flip `root/.bee/state.json`'s `phase` from the fixture default `"idle"`
/// to a distinct, unmistakable marker value. Refuses (returns Err, writes
/// nothing) if the file's content does not match the exact fixture body
/// this crate expects — a silent mutation of unexpected content would not
/// prove anything about the differ's detection power.
pub fn seed_mutation(root: &Path) -> Result<(), String> {
    let state_path = root.join(".bee").join("state.json");
    let current = fs::read_to_string(&state_path).map_err(|e| format!("read {}: {e}", state_path.display()))?;
    if current != FIXTURE_STATE_BODY {
        return Err(format!(
            "seed_mutation: {} content does not match the expected fixture body — refusing to seed a mutation over unexpected content (this would prove nothing)",
            state_path.display()
        ));
    }
    fs::write(&state_path, MUTATED_STATE_BODY).map_err(|e| format!("write {}: {e}", state_path.display()))
}

// ─── rpl-1: per-scenario mutation targets ──────────────────────────────────
//
// The `state.json` phase flip above is the RIGHT control for `status`, and
// the WRONG control for everything else. `phase` appears in `status`'s output
// and in nothing else: no ledger verb — `intent show`, `capture count`,
// `decisions active`, `backlog counts`, `reviews list`, `feedback rank` —
// ever prints it. A generic `--cmd-check` arm that reused this flip would
// therefore hand every ledger scenario a negative control that CANNOT fire,
// and a control that cannot fire is worse than none: it converts "the differ
// caught nothing" from a red into a green.
//
// So the control becomes per-scenario. Each scenario declares the store IT
// actually reads and how to perturb it; a scenario with no declared target is
// refused at registration (`cmdcheck::ScenarioSet::register`), never silently
// skipped.

/// The store a scenario reads, and the precise perturbation that proves the
/// comparison can see a change in it.
#[derive(Clone, Copy)]
pub struct MutationTarget {
    /// The store-root-relative path this scenario actually reads. Recorded
    /// so the verify output names it and a reviewer can check the claim.
    pub store: &'static str,
    /// Applies the perturbation to a cloned root. MUST refuse (return `Err`)
    /// when the target content is not what it expected — a blind
    /// substitution proves nothing about detection power.
    pub apply: fn(&Path) -> Result<(), String>,
}

/// Replace `needle` with `replacement` in `rel_path` under `root`, refusing
/// unless `needle` occurs EXACTLY once.
///
/// This is the generalization of [`seed_mutation`]'s discipline: the flip is
/// pinned to content the caller has already asserted, so a fixture change
/// that moves the target surfaces as a loud refusal instead of a mutation
/// that silently did nothing (which would read as "the differ failed to
/// detect a divergence" — a red for the wrong reason).
pub fn replace_exactly_once(
    root: &Path,
    rel_path: &str,
    needle: &str,
    replacement: &str,
) -> Result<(), String> {
    let path = root.join(rel_path);
    let current = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let occurrences = current.matches(needle).count();
    if occurrences != 1 {
        return Err(format!(
            "replace_exactly_once: {needle:?} occurs {occurrences} time(s) in {} (expected exactly 1) — refusing to seed a mutation over unexpected content (this would prove nothing)",
            path.display()
        ));
    }
    let mutated = current.replacen(needle, replacement, 1);
    fs::write(&path, mutated).map_err(|e| format!("write {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_over_unexpected_content() {
        let base = std::env::temp_dir().join(format!("bee-parity-mutate-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join(".bee")).unwrap();
        fs::write(base.join(".bee").join("state.json"), b"{\"phase\":\"not-the-fixture-body\"}").unwrap();
        let err = seed_mutation(&base).unwrap_err();
        assert!(err.contains("does not match the expected fixture body"));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn flips_the_expected_fixture_body() {
        let base = std::env::temp_dir().join(format!("bee-parity-mutate-test-2-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join(".bee")).unwrap();
        fs::write(base.join(".bee").join("state.json"), FIXTURE_STATE_BODY).unwrap();
        seed_mutation(&base).unwrap();
        let after = fs::read_to_string(base.join(".bee").join("state.json")).unwrap();
        assert_eq!(after, MUTATED_STATE_BODY);
        let _ = fs::remove_dir_all(&base);
    }
}
