// merge_ready — the stored "this feature is finished in its worktree and
// waits for the human to merge" fact (docs/history/merge-ready-fact).
//
// D1: the FEATURE record (its lane record when `.bee/lanes/<f>.json` exists,
// else the default `.bee/state.json` record when that record names the same
// feature) carries an optional
//   merge_ready: {since, branch, worktree_id, uat: pending|approved, blocked_by: []}
// written by the cap that leaves zero open/claimed/blocked cells for a
// feature that has a granted worktree. A feature with zero cells never gets
// it (`capped >= 1` is part of the condition).
//
// D2: `bee close` rewrites `blocked_by`, `bee gate --name uat` flips `uat`,
// and every reopen of one of the feature's cells removes the whole key (the
// next last-cap re-sets it, with a fresh `since`).
//
// D3: ADDITIVE PROJECTION ONLY. Nothing in bee reads `merge_ready` to decide
// anything — not a gate, not the merge door, not a refusal. It is written
// here and surfaced verbatim; an external board is the only reader.
//
// FAIL-OPEN, ALWAYS. Every function here is a side effect hung off a verb
// that has already done its real work. A missing record, a corrupt lane
// file, a lock that will not open, a projection rebuild that throws — each
// answers "nothing written" and NEVER changes the calling verb's own result
// or exit code. That is why nothing below returns a Result: there is no
// error for a caller to handle, by design.
//
// All four writers go through the ONE mutation seam every other record
// mutation uses (state_group/ledger.rs: resolve the lock scope, take the
// locks, read the record strictly, write through the projection) so a
// merge_ready write rebuilds the same projections a `bee state set` does and
// takes the same lock order — never a hand-rolled JSON patch.
#![allow(unused_imports)]

use super::*;
use crate::verbs::cells::list_cells;
use crate::verbs::reservations::js_trim;
use crate::verbs::state_group::{
    acquire_mutation_locks, read_state_peek, resolve_mutation_lock_scope, resolve_mutation_target,
    write_through_projection, Scope,
};
use crate::verbs::status_full::find_granted_worktree_for_feature;
use crate::verbs::worktree::current_branch;
use serde_json::{json, Map, Value};
use std::path::Path;

/// The one spelling of the key, so no caller re-types it.
pub(crate) const KEY: &str = "merge_ready";

/// Which record carries this feature's `merge_ready`.
enum Home {
    /// `.bee/lanes/<feature>.json` exists.
    Lane,
    /// No lane file, and the default `.bee/state.json` record names this
    /// same feature.
    Default,
    /// Neither — this feature has no record to hang the fact on.
    None,
}

fn home(root: &Path, feature: &str) -> Home {
    if feature.is_empty() {
        return Home::None;
    }
    // `lane_path` refuses a path-shaped feature; that refusal is "no lane".
    if let Ok(path) = lane_path(root, feature) {
        if path.exists() {
            return Home::Lane;
        }
    }
    // The PEEK read (never the strict one): a corrupt default record is
    // "no home", not a throw — fail-open is the whole contract here.
    let Ok(state) = read_state_peek(root) else { return Home::None };
    match state.get("feature") {
        Some(Value::String(f)) if f == feature => Home::Default,
        _ => Home::None,
    }
}

/// Resolve the feature's record over the ledger mutation seam, hand it to
/// `mutate`, and write it back through the projection when `mutate` reports
/// it changed something. `false` on every no-op and every failure alike —
/// the caller cannot tell them apart, and by design does not care.
fn with_record<F>(root: &Path, feature: &str, mutate: F) -> bool
where
    F: FnOnce(&mut Map<String, Value>) -> bool,
{
    let (lane_feature, no_lane, scope) = match home(root, feature) {
        Home::Lane => {
            let Ok(scope) = resolve_mutation_lock_scope(root, Some(feature), false) else {
                return false;
            };
            (Some(feature), false, scope)
        }
        // `resolve_mutation_lock_scope(root, None, false)` would re-derive
        // this same `{feature, lane: false}` from the peeked `state.feature`
        // that `home` just matched — EXCEPT that a session lane binding
        // would route it onto a lane instead. `home` already decided which
        // record owns the fact, so the scope names that record directly and
        // the target is resolved with `no_lane` to match it.
        Home::Default => (None, true, Scope { feature: Some(feature.to_string()), lane: false }),
        Home::None => return false,
    };
    let Ok(workflows) = list_workflows(root) else { return false };
    // Held until the write below completes — the same RAII window every
    // other mutation through this seam takes.
    let Ok(_locks) = acquire_mutation_locks(root, &scope, &workflows) else { return false };
    let Ok(mut target) = resolve_mutation_target(root, lane_feature, KEY, no_lane) else {
        // A corrupt or missing record refuses LOUDLY there on purpose (it is
        // the strict read). Swallowing it is what keeps the calling verb's
        // own result intact.
        return false;
    };
    if !mutate(target.record_mut()) {
        return false;
    }
    let record = target.record().clone();
    write_through_projection(root, &target, &record, &[]).is_ok()
}

/// `approved_gates.uat === true` on the record the fact lands on.
fn uat_approved(record: &Map<String, Value>) -> bool {
    matches!(
        record.get("approved_gates").and_then(|g| g.get("uat")),
        Some(Value::Bool(true))
    )
}

/// D1's writer, called from the cap door with the cell's own feature.
///
/// Returns the value it wrote, or `None` when the cap was not the last one,
/// the feature has no granted worktree, or the feature has no record. The
/// cap records that answer under `merge_ready` in its own result (null when
/// nothing was written) — the ONE result-shape change this fact makes.
///
/// Re-setting over an existing fact KEEPS `since`, `uat` and `blocked_by`
/// (the human-facing "waiting since" must not restart because a later cell
/// capped) and refreshes only `branch`/`worktree_id`, which can genuinely
/// move under a feature.
pub(crate) fn set_after_cap(root: &Path, feature: &str, now_iso: &str) -> Option<Value> {
    if feature.is_empty() {
        return None;
    }
    // Every count comes from the SAME `list_cells` the status verbs read, so
    // "nothing outstanding" here means exactly what a reader would compute.
    // NOTE: `list_cells` applies the island scope of a granted worktree — the
    // cap door always runs at MAIN's store root (`cells finish`'s
    // `finish_topology`), where no island scope applies.
    let count = |status: &str| list_cells(root, Some(feature), Some(status)).ok().map(|c| c.len());
    let open = count("open")?;
    let claimed = count("claimed")?;
    let blocked = count("blocked")?;
    let capped = count("capped")?;
    if open != 0 || claimed != 0 || blocked != 0 {
        return None;
    }
    if capped == 0 {
        return None; // D1: a zero-cell feature never becomes merge-ready.
    }
    let (worktree_id, worktree_root) = find_granted_worktree_for_feature(root, feature)?;
    // The worktree's real branch; the conventional name only when git cannot
    // answer (a detached HEAD, or no git at all).
    let branch = current_branch(Path::new(&worktree_root))
        .filter(|b| !js_trim(b).is_empty())
        .unwrap_or_else(|| format!("wt/{feature}"));

    let mut written: Option<Value> = None;
    let wrote = with_record(root, feature, |record| {
        let prev = match record.get(KEY) {
            Some(Value::Object(m)) => Some(m.clone()),
            _ => None,
        };
        let carry = |key: &str, fallback: Value| -> Value {
            match prev.as_ref().and_then(|p| p.get(key)) {
                Some(v) if !v.is_null() => v.clone(),
                _ => fallback,
            }
        };
        let uat = carry(
            "uat",
            json!(if uat_approved(record) { "approved" } else { "pending" }),
        );
        let since = carry("since", json!(now_iso));
        let blocked_by = match prev.as_ref().and_then(|p| p.get("blocked_by")) {
            Some(Value::Array(a)) => Value::Array(a.clone()),
            _ => json!([]),
        };
        let mut fact = Map::new();
        fact.insert("since".into(), since);
        fact.insert("branch".into(), json!(branch));
        fact.insert("worktree_id".into(), json!(worktree_id));
        fact.insert("uat".into(), uat);
        fact.insert("blocked_by".into(), blocked_by);
        let value = Value::Object(fact);
        record.insert(KEY.into(), value.clone());
        written = Some(value);
        true
    });
    if wrote {
        written
    } else {
        None
    }
}

/// D2: remove the fact entirely. Every reopen path calls this — the feature
/// is no longer finished, and the next last-cap re-sets it with a fresh
/// `since`. `false` when there was nothing to remove.
pub(crate) fn clear(root: &Path, feature: &str) -> bool {
    if feature.is_empty() {
        return false;
    }
    with_record(root, feature, |record| record.remove(KEY).is_some())
}

/// D2: `bee gate --name uat` flips the stored `uat` field. Never CREATES the
/// fact — a feature that is not merge-ready has no `uat` to flip.
// Its one caller is the `gate` verb, wired in the next cell of this feature
// (mrf-2); the tests below are its only use until then. Drop the allow with
// that wiring.
#[allow(dead_code)]
pub(crate) fn set_uat(root: &Path, feature: &str, approved: bool) -> bool {
    if feature.is_empty() {
        return false;
    }
    with_record(root, feature, |record| {
        let Some(Value::Object(fact)) = record.get_mut(KEY) else { return false };
        let next = json!(if approved { "approved" } else { "pending" });
        if fact.get("uat") == Some(&next) {
            return false; // already so — no write, no projection churn
        }
        fact.insert("uat".into(), next);
        true
    })
}

/// D2: `bee close` rewrites `blocked_by` with the doors it found still
/// standing (empty on a green close). Never CREATES the fact.
// Its one caller is `bee close`, wired in mrf-2 — see `set_uat` above.
#[allow(dead_code)]
pub(crate) fn set_blocked_by(root: &Path, feature: &str, doors: &[&str]) -> bool {
    if feature.is_empty() {
        return false;
    }
    with_record(root, feature, |record| {
        let Some(Value::Object(fact)) = record.get_mut(KEY) else { return false };
        let next = Value::Array(doors.iter().map(|d| json!(d)).collect());
        if fact.get("blocked_by") == Some(&next) {
            return false;
        }
        fact.insert("blocked_by".into(), next);
        true
    })
}

// ─── tests ─────────────────────────────────────────────────────────────────
//
// The cap/reopen WIRING is proved in verbs/cells/tests.rs, against real
// cells and a real granted worktree. What is proved here is the helper's own
// contract: which record it lands on, what it refuses to create, and that a
// broken record is silence rather than a throw.
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn write_json(path: &Path, body: &Value) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, serde_json::to_string_pretty(body).unwrap()).unwrap();
    }

    fn lane(root: &Path, feature: &str, extra: Value) {
        let mut body = json!({"schema_version": "1.0", "feature": feature, "phase": "idle"});
        if let (Value::Object(b), Value::Object(e)) = (&mut body, &extra) {
            for (k, v) in e {
                b.insert(k.clone(), v.clone());
            }
        }
        write_json(&root.join(".bee").join("lanes").join(format!("{feature}.json")), &body);
    }

    fn read_lane_raw(root: &Path, feature: &str) -> Value {
        let raw = std::fs::read_to_string(
            root.join(".bee").join("lanes").join(format!("{feature}.json")),
        )
        .unwrap();
        serde_json::from_str(&raw).unwrap()
    }

    /// A fact planted directly on the record, so the flip/clear helpers have
    /// something to act on without a whole cap fixture.
    fn plant(root: &Path, feature: &str) {
        lane(
            root,
            feature,
            json!({"merge_ready": {
                "since": "2026-01-01T00:00:00.000Z",
                "branch": "wt/demo",
                "worktree_id": "wt-demo",
                "uat": "pending",
                "blocked_by": []
            }}),
        );
    }

    #[test]
    fn merge_ready_set_uat_flips_only_an_existing_fact() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        plant(root, "demo");

        assert!(set_uat(root, "demo", true));
        assert_eq!(read_lane_raw(root, "demo")["merge_ready"]["uat"], json!("approved"));
        // Idempotent: the same value again writes nothing.
        assert!(!set_uat(root, "demo", true));
        assert!(set_uat(root, "demo", false));
        assert_eq!(read_lane_raw(root, "demo")["merge_ready"]["uat"], json!("pending"));

        // A record with no fact is never given one by a uat flip.
        lane(root, "bare", json!({}));
        assert!(!set_uat(root, "bare", true));
        assert!(read_lane_raw(root, "bare").get("merge_ready").is_none());
    }

    #[test]
    fn merge_ready_set_blocked_by_rewrites_the_door_list() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        plant(root, "demo");

        assert!(set_blocked_by(root, "demo", &["scribing", "judge-debt"]));
        assert_eq!(
            read_lane_raw(root, "demo")["merge_ready"]["blocked_by"],
            json!(["scribing", "judge-debt"])
        );
        // A green close writes the empty list back.
        assert!(set_blocked_by(root, "demo", &[]));
        assert_eq!(read_lane_raw(root, "demo")["merge_ready"]["blocked_by"], json!([]));
        assert!(!set_blocked_by(root, "demo", &[]), "no change, no write");
    }

    #[test]
    fn merge_ready_clear_removes_the_key_and_is_a_no_op_without_it() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        plant(root, "demo");

        assert!(clear(root, "demo"));
        let after = read_lane_raw(root, "demo");
        assert!(after.get("merge_ready").is_none(), "the whole key is gone: {after}");
        assert!(after.get("feature").is_some(), "the rest of the record survives");
        assert!(!clear(root, "demo"), "clearing twice writes nothing");
    }

    /// The record the fact lands on: the lane when one exists, the default
    /// state record when it names the same feature, nothing otherwise.
    #[test]
    fn merge_ready_lands_on_the_lane_then_the_default_record_then_nowhere() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // No lane, no matching default record -> no home at all.
        write_json(
            &root.join(".bee").join("state.json"),
            &json!({"schema_version": "1.0", "feature": "other"}),
        );
        assert!(!clear(root, "demo"));

        // The default record NAMES the feature -> that is the home.
        write_json(
            &root.join(".bee").join("state.json"),
            &json!({"schema_version": "1.0", "feature": "demo", "phase": "idle",
                    "merge_ready": {"uat": "pending"}}),
        );
        assert!(clear(root, "demo"));
        let state: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(".bee").join("state.json")).unwrap(),
        )
        .unwrap();
        assert!(state.get("merge_ready").is_none());

        // A lane file wins over the default record.
        plant(root, "demo");
        assert!(set_uat(root, "demo", true));
        assert_eq!(read_lane_raw(root, "demo")["merge_ready"]["uat"], json!("approved"));
    }

    /// The fail-open contract: a corrupt lane record makes the helper answer
    /// "nothing written" instead of throwing into the calling verb.
    #[test]
    fn merge_ready_is_silent_over_a_corrupt_record() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let file = root.join(".bee").join("lanes").join("demo.json");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "{not json").unwrap();

        assert!(!clear(root, "demo"));
        assert!(!set_uat(root, "demo", true));
        assert!(!set_blocked_by(root, "demo", &["x"]));
        assert!(set_after_cap(root, "demo", "2026-01-01T00:00:00.000Z").is_none());
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            "{not json",
            "a corrupt record is left exactly as found"
        );
    }

    /// An empty feature name reaches nothing — the cap door passes whatever
    /// the cell carries, and a cell can carry no feature at all.
    #[test]
    fn merge_ready_ignores_an_empty_feature() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        assert!(!clear(root, ""));
        assert!(!set_uat(root, "", true));
        assert!(!set_blocked_by(root, "", &[]));
        assert!(set_after_cap(root, "", "2026-01-01T00:00:00.000Z").is_none());
    }

    /// D1's zero-cell arm: a feature with no capped cell is never
    /// merge-ready, even with a lane record present.
    #[test]
    fn merge_ready_set_after_cap_refuses_a_feature_with_no_cells() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        lane(root, "demo", json!({}));
        assert!(set_after_cap(root, "demo", "2026-01-01T00:00:00.000Z").is_none());
        assert!(read_lane_raw(root, "demo").get("merge_ready").is_none());
    }
}
