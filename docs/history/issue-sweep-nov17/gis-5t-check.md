# gis-5t: independent check of gis-5 (issue 171)

Cell under test: gis-5, commit 6a563b0ae. Parent commit: d785a3dfc. Decision: e092dbb6 (CONTEXT D6).

## Truths

| Truth | Result | Fix site | Tests |
| --- | --- | --- | --- |
| cells add refuses a cell carrying depends_on and names deps | PASS | `packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:293` | `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs:4067` (gis-5), `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs:4097` (gis-5t edge) |
| cells add refuses a cell carrying dependencies | PASS | `packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:293` | `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs:4067` (gis-5), `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs:4097` (gis-5t edge) |
| a cell with deps is added unchanged | PASS | no change; normalize keeps deps | `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs:4067` (gis-5), `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs:4097` (gis-5t edge) |

gis-5 also refuses both keys in `cells update`, at `packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs:520`, tested at `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs:4089`.

The gis-5t edge test `add_batch_refuses_a_foreign_dependency_key_even_beside_deps_or_empty` goes through the batch report that `cells add` and `--dry-run` use. It checks three cases:

- A cell with `"deps": []` and `"depends_on": []` is refused.
- A cell with `"deps": []` and `"dependencies": null` is refused.
- A cell with `"deps": ["d-9"]` passes, and its deps come out unchanged.

So the refusal fires on the key alone. A real `deps` beside the foreign key does not excuse it, and neither does an empty or null value.

## Red on the parent commit

Run on `git archive d785a3dfc` in a scratch directory, with this commit's `tests.rs` copied in. `.pi/extensions/bee-guard` was copied from the worktree too, because build.rs needs it and git archive does not include it. gis-5 does not change `.pi`.

`cargo test --release --bin bee -- add_refuses_a_cell update_names_deps add_batch_refuses` on the parent gave `0 passed; 3 failed`:

- `add_refuses_a_cell_that_names_its_dependencies_by_a_foreign_key` FAILED: `expected a thrown refusal`. The parent accepts `depends_on`, which is the reported symptom.
- `update_names_deps_for_a_foreign_dependency_key` FAILED: `a foreign dependency key is refused with a hint`. The parent has no hint for the key.
- `add_batch_refuses_a_foreign_dependency_key_even_beside_deps_or_empty` FAILED at `assertion failed: !ok`. The parent batch accepts the cell.

Truth 3 is a no-regression truth. The parent already added a `deps` cell unchanged, so no test of it alone can be red on the parent. Its assertions sit after the refusal assertions in both tests, so on the parent they never run.

## Symptom checked with the real binary

Scratch git repo, fixed release binary built from the worktree HEAD:

```
$ bee cells add --file c.json --dry-run --json      (c.json has "depends_on": ["x-1"])
"problems": ["addCell: \"depends_on\" is not a cell field — a cell names what it waits on with \"deps\". FIX: rename \"depends_on\" to \"deps\"."]
exit=1
$ bee cells add --stdin                              ("dependencies": ["x-1"])
addCell: "dependencies" is not a cell field — a cell names what it waits on with "deps". FIX: rename "dependencies" to "deps".
exit=1
$ bee cells add --stdin                              ("deps": ["x-1"])
Added d-1 [open] (tiny) t
$ bee cells show --id d-1 --json                     "deps": ["x-1"]
$ echo '{"depends_on":["x-1"]}' | bee cells update --id d-1 --stdin
updateCell: field "depends_on" is frozen — a cell names what it waits on with "deps", not this key — use "deps". The whole patch is refused; the cell is untouched.
exit=1
```

No parent binary was built for the CLI run. The parent's failing unit tests show that the parent accepts these keys.

## Verify

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- verbs::cells`

`test result: ok. 533 passed; 0 failed; 10 ignored; 0 measured; 4875 filtered out`

The new tests in that run:

- `verbs::cells::tests::add_refuses_a_cell_that_names_its_dependencies_by_a_foreign_key ... ok`
- `verbs::cells::tests::update_names_deps_for_a_foreign_dependency_key ... ok`
- `verbs::cells::tests::add_batch_refuses_a_foreign_dependency_key_even_beside_deps_or_empty ... ok`

## Notes

- The update refusal calls `depends_on` a "frozen" field. The hint text is correct, but "frozen" suggests a real cell field that cannot change. That wording comes from the shared frozen-key message, which is out of scope here.
- If you build the parent tree into the same `CARGO_TARGET_DIR`, it overwrites the test binary under the same hash. The next worktree `cargo test` then runs the parent binary until the sources are touched. The first verify run here failed for exactly that reason. It went green after touching the sources and rebuilding.
