# hrh-5t — Check the bounded herding status output (hrh-5, commit 43caa0c2)

**Verdict: GREEN on all three truths, with one coverage gap.** I did not read hrh-5's cap report. I read only the commit diff and ran the code. I edited no source. Every mutation below was reverted with `git checkout -- packages/bee-rs/crates/bee/src/herding.rs`, and `git status --short` was empty afterwards.

## Check 1: with many jobs, the default `status --json` stays under the cap and names the spill file. PASS

Unit test `herding::tests::status_json_default_is_bounded_and_names_the_spilled_full_form` passes in the cell verify (see the end of this report). It seeds 300 jobs, each with a 2000-byte message, and checks that the inline form is more than 160 000 bytes. It then checks that the default text is at most `STATUS_BOUND_BYTES` (16 000), that `full_output` starts with `.bee/tmp/output/`, that every row has `job_id` and no `last_message`, that rows plus `jobs_omitted` equal 300, and that the spilled file holds all 300 full jobs.

Live check with the real release binary (`cargo build --release --bin bee`), on a throwaway repo seeded with the same 300 jobs:

    $ bee herding status --json --main-root $S > live-2.json        (second run)
    exit=0
    1 15845 live-2.json                                             (wc -lc)
    keys: bounded, enabled, full_output, jobs, jobs_omitted, main_root, marker, reason, transport, untracked_paseo_agents_count
    jobs[0]: {'job_id': 'job-000', 'status': 'working', 'round': 1, 'mark': 'interrupted', 'mark_reason': 'process_restarted', 'pane_id': 'w4:p000', 'unread_messages': 1}
    bounded=True full_output=.bee/tmp/output/herding-status-20261119T154714897Z.json jobs_omitted=195 rows=105
    spilled file: 678 869 bytes, contains last_message

    $ bee herding status --json --full --main-root $S
    exit=0
    678870 bytes inline                                             (wc -c)

The JSON line is 15 845 bytes including the newline, which is under the 16 000-byte cap. The full form is 678 870 bytes, so the default output is 43× smaller.

## Check 2: with the bound removed the test fails, and with it restored the test passes. PASS

I ran each mutation with `cargo test --release --bin bee -- herding::tests::status_json`:

| Mutation | Edit | bounded test | full-flag test |
|---|---|---|---|
| A: no truncation | `if used > STATUS_BOUND_BYTES - 40 {` → `if false {` | **FAILED** at herding.rs:2280 `text.len() <= STATUS_BOUND_BYTES`, len `44273` | ok |
| B: never spill | `main_root.filter(\|_\| !full)` → `filter(\|_\| false)` | **FAILED** at herding.rs:2280, len `678381` | ok |
| C: wiring reverted | `println!("{}", status_json_text(...))` → `emit(obj.clone())` | ok | ok |
| D: `--full` ignored | `full = true;` → `full = false;` in the flag parse | ok | **FAILED** at herding.rs:2311 `!root.join(".bee/tmp/output").exists()` |

Result of restoring the source and running the cell verify again: both tests `ok`, 853 passed (output at the end).

Mutations A and B are the bound itself, and each turns the test red for the right reason, a byte count over the cap. Mutation A shows that trimming the rows alone (44 KB for 300 rows) is not enough. The truncation is what keeps the output under the cap.

**Gap (mutation C):** neither test covers the call site that prints. Both tests call `status_json_text` directly. If someone puts back the old `emit(obj.clone())` at herding.rs:1594, the default `--json` again prints the full 678 KB form, and both tests stay green. Mutation D goes red only by accident: the `println!` inside `status_with_panes_and_transport` spills a file, and that file trips the "no output dir" assertion. To cover the wiring, a test could capture what `status_with_panes_and_transport(&[.., "--json"])` prints, or return the text. I wrote no test, because this cell may not edit source.

## Check 3: the report quotes each check. PASS

This report quotes each command's output above.

## Found (outside this cell)

- **The bound covers only the JSON object, not all of stdout.** On the first live run, `wc -l` showed 301 lines and `wc -c` showed 33 845 bytes. Before the JSON, the sweep printed 300 `herding: marked job job-NNN interrupted (process_restarted)` lines to stdout. This also means `--json` stdout is not pure JSON on a run that marks orphans. This behaviour is older than hrh-5.
- **Spill files pile up.** Each default `--json` call writes a new `herding-status-<ts>.json` of about 680 KB under `.bee/tmp/output/`, and nothing prunes them. Two calls left two files.
- If `main_root` does not resolve, or the spill write fails, `status_json_text` falls back to the unbounded inline form. That is a quiet path back to a large output. It is fine when there is no root, but it should be named.

## Cell verify (fresh, source restored)

    $ PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- herding::
    exit=0
    test herding::tests::status_json_full_flag_prints_the_whole_form_inline ... ok
    test herding::tests::status_json_default_is_bounded_and_names_the_spilled_full_form ... ok
    test result: ok. 853 passed; 0 failed; 0 ignored; 0 measured; 4660 filtered out; finished in 34.92s
