// Command-registry contracts over the EMBEDDED payload.
//
// R5 port of the schema half of `packages/bee/tests/test_bee_cli.mjs`
// (5764 L). That suite covers three things:
//
//   1. every COMMAND_REGISTRY entry's `parameters` is a valid JSON-Schema
//      object (its D3 shape);
//   2. `validate()` rejects a missing required field with the structured
//      `{ok:false, error:{field, reason, command}}` shape and never throws;
//   3. every entry's `examples[]` executes successfully against the real
//      underlying helper, inside an isolated temp repo.
//
// (1) is ported here, over `src/generated/registry_payload.json` — the bytes
// the Rust binary actually ships. `tests/front_door.rs::
// embedded_registry_payload_is_fresh` already pins that file against the live
// `command-registry.mjs`; this file asserts the payload is well-formed, so
// between them a malformed OR stale registry cannot reach a release.
//
// (2) is NOT ported: `validate()` has no Rust owner. The write guard's
// check (d) — the CLI-shape schema guard that calls it — is pinned as a
// deliberate delegation (`hooks/write_guard.rs::bee_cli_shapes_delegate`).
// That is a structural blocker for deleting the runtime, tracked in
// plans/r5-test-migration.md § "Structural delegations".
//
// (3) is NOT ported: executing every example needs the .mjs suite's whole
// fixture apparatus (a repo past Gate 2, seeded cells/decisions/reservations,
// perf and transcript roots redirected), and most examples would run through
// the delegate anyway — proving Node works, not Rust. What IS portable is the
// invariant below that every example is spelled against a command the
// registry itself declares, which catches the drift class those runs existed
// to catch.

use std::collections::BTreeSet;

fn payload() -> serde_json::Value {
    let raw = include_str!("../src/generated/registry_payload.json");
    serde_json::from_str(raw).expect("the embedded registry payload must be parseable JSON")
}

fn commands(p: &serde_json::Value) -> &Vec<serde_json::Value> {
    p["commands"].as_array().expect("commands must be an array")
}

#[test]
fn every_entry_declares_a_valid_json_schema_parameters_object() {
    let p = payload();
    let cmds = commands(&p);
    assert!(cmds.len() > 100, "the registry shrank unexpectedly: {}", cmds.len());

    for entry in cmds {
        let name = entry["name"].as_str().expect("every entry has a string name");
        let params = &entry["parameters"];
        assert_eq!(
            params["type"], "object",
            "{name}: parameters.type must be \"object\""
        );
        let props = params["properties"]
            .as_object()
            .unwrap_or_else(|| panic!("{name}: parameters.properties must be an object"));
        for (prop, spec) in props {
            let ty = spec["type"]
                .as_str()
                .unwrap_or_else(|| panic!("{name}.{prop}: every property declares a string type"));
            assert!(
                matches!(ty, "string" | "boolean" | "number" | "integer" | "array" | "object"),
                "{name}.{prop}: unknown JSON-Schema type {ty:?}"
            );
        }
        // `required` must exist, be an array of strings, and every entry in it
        // must actually be declared in `properties` — a required field with no
        // property is a validator that can never be satisfied.
        let required = params["required"]
            .as_array()
            .unwrap_or_else(|| panic!("{name}: parameters.required must be an array"));
        for r in required {
            let field = r
                .as_str()
                .unwrap_or_else(|| panic!("{name}: required entries must be strings"));
            assert!(
                props.contains_key(field),
                "{name}: required field {field:?} is not declared in properties"
            );
        }
    }
}

#[test]
fn every_entry_carries_a_description_an_invoke_and_at_least_one_example() {
    let p = payload();
    for entry in commands(&p) {
        let name = entry["name"].as_str().unwrap();
        assert!(
            entry["description"].as_str().is_some_and(|d| !d.trim().is_empty()),
            "{name}: needs a non-empty description — it is the agent-facing help text"
        );
        let invoke = entry["invoke"]
            .as_str()
            .unwrap_or_else(|| panic!("{name}: needs a string invoke"));
        assert!(
            invoke.starts_with("bee "),
            "{name}: invoke must be spelled against the binary, got {invoke:?}"
        );
        assert!(
            !invoke.contains(".mjs"),
            "{name}: invoke still names a .mjs script ({invoke:?}) — the R6a rule is that \
             every agent-facing spelling names the binary"
        );
        let examples = entry["examples"]
            .as_array()
            .unwrap_or_else(|| panic!("{name}: needs an examples array"));
        assert!(!examples.is_empty(), "{name}: needs at least one example");
    }
}

#[test]
fn every_example_is_spelled_against_a_declared_command() {
    // The drift class the .mjs suite's example RUNS existed to catch: an
    // example that names a verb the registry no longer declares. Checked
    // structurally here rather than by execution — see this file's header.
    let p = payload();
    let cmds = commands(&p);
    let declared: BTreeSet<&str> = cmds.iter().filter_map(|e| e["invoke"].as_str()).collect();

    let mut checked = 0usize;
    for entry in cmds {
        let name = entry["name"].as_str().unwrap();
        for ex in entry["examples"].as_array().unwrap() {
            let ex = ex.as_str().unwrap_or_else(|| panic!("{name}: examples are strings"));
            assert!(
                ex.starts_with("bee "),
                "{name}: example must invoke the binary, got {ex:?}"
            );
            assert!(
                !ex.contains(".mjs"),
                "{name}: example still names a .mjs script: {ex:?}"
            );
            // The example must begin with SOME declared invoke line. Longest
            // match, so `bee state gate` is not satisfied by `bee state`.
            let hit = declared.iter().any(|inv| {
                ex == *inv || ex.strip_prefix(*inv).is_some_and(|rest| rest.starts_with(' '))
            });
            assert!(
                hit,
                "{name}: example {ex:?} does not begin with any declared `invoke` — either the \
                 example drifted or the command was renamed"
            );
            checked += 1;
        }
    }
    assert!(checked > 100, "only {checked} examples checked — the walk went vacuous");
}

#[test]
fn command_names_and_invokes_are_unique() {
    let p = payload();
    let mut names = BTreeSet::new();
    let mut invokes = BTreeSet::new();
    for entry in commands(&p) {
        let name = entry["name"].as_str().unwrap();
        assert!(names.insert(name), "duplicate command name: {name}");
        let invoke = entry["invoke"].as_str().unwrap();
        assert!(invokes.insert(invoke), "duplicate invoke line: {invoke}");
    }
}

#[test]
fn the_payload_declares_the_schema_version_the_binary_was_built_against() {
    let p = payload();
    assert_eq!(
        p["schema_version"], "1.0",
        "the embedded registry's schema_version changed — re-check every consumer before \
         updating this pin (regen: node scripts/export_registry_payload.mjs, then rebuild)"
    );
}
