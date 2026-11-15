# ohp-6 check: the Principles close sentence and its skill pointers

Verdict: **pass**

Scope: commit `2e70aa164` (ohp-3), decision D3, and the Communication contract
in `skills/bee-hive/references/routing-and-contracts.md`. Nothing outside this
record was edited.

## What was checked

1. **The sentence is present and says what D3 locks.** The Turn shape
   `**Close**` bullet now reads: "A multi-step routed task's closing turn also
   carries one Principles line: each applied principle slug and the decision it
   changed, or a plain statement that none changed anything (rule:
   agents-name-applied-principles)." This matches D3 in
   `docs/history/orient-host-playbook/CONTEXT.md`: slug and decision, or none,
   scoped to a multi-step routed task.
2. **The five numbered rules are unchanged.** `git diff 2e70aa164~1 2e70aa164`
   on the source file shows one removed and one added line, both the Close
   bullet. Rules 1 to 5 sit below that bullet, and no later commit touches the
   file.
3. **No other section repeats the sentence.** `rg "applied principle slug"`
   finds one line in the source and one in each of the five rendered trees
   (`.claude`, `.agents`, `.claude-plugin`, `.codex-plugin`, `.opencode`). The
   cited rule id `agents-name-applied-principles` already exists in
   `.bee/LEADER.md` and `packages/bee/LEADER.md`, so the citation resolves.

## Command output summary

The cell's verify command:

```
cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- skill_reference pointer render_skill_trees
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 5208 filtered out
.bee/bin/bee dev release-manifest --check
release_manifest --check: 950 file(s) match stored manifest, 1 unhashed artifact(s) present
```

Running the same filters with `--list` showed that `skill_reference` and
`render_skill_trees` match no test in the `bee` binary. All 27 tests come from
the `pointer` filter, and they are unrelated unit tests whose names contain
"pointer". The verify passes, but it does not run the pointer integrity or
render parity checks it names. To cover those checks, these also ran:

```
cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity --test rule_index_parity --test principle_index_parity
pointer_integrity: ok. 10 passed; 0 failed
principle_index_parity: ok. 5 passed; 0 failed
rule_index_parity: ok. 5 passed; 0 failed
cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- devtools::skill_trees
test result: ok. 14 passed; 0 failed
```

`devtools::skill_trees` holds the render parity tests for the rendered skill
trees. Together with the release manifest match, it confirms that the five
rendered copies match the source.

## Finding for the plan

The verify filters `skill_reference` and `render_skill_trees` match nothing.
A future cell that copies this verify gets a green result without running the
skill pointer or render parity tests. The working filters are
`--test pointer_integrity` and `--bin bee -- devtools::skill_trees`.
