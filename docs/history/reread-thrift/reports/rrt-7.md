# rrt-7 report: four contracts proven, four code cells judged

Verify (green, fresh run):

    cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- thin_worker_listings_off handoff_after_merge_door herding_result_job_digest cell_brief_search_rule_and_size
    test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 5591 filtered out

## Tests added (test modules only, no production code)

| Contract | Tests | File |
|---|---|---|
| thin_worker_listings_off | `thin_worker_listings_off_env_list_holds_the_slash_command_budget`, `..._claude_session_argv_follows_the_thin_switch`, `..._built_in_claude_pane_argv_follows_the_thin_switch` | herding/run.rs:13946-14012 |
| thin_worker_listings_off | `thin_worker_listings_off_spawn_argv_appends_the_flags_before_the_prompt` | herding/claude_session.rs:291 |
| handoff_after_merge_door | `..._refuses_a_second_feature_and_writes_nothing`, `..._same_session_with_a_reason_succeeds_and_is_audited`, `..._lets_a_first_start_and_a_sessionless_start_through` (each in a temp repo) | verbs/feature_group.rs:707-763 |
| handoff_after_merge_door | `handoff_after_merge_door_a_session_merge_records_the_merger_and_a_sessionless_one_does_not` (a real merge through `run_merge_with_caller`) | verbs/worktree/tests.rs:921-975 |
| herding_result_job_digest | `..._prints_every_field_in_text_and_json`, `..._cuts_a_long_result_and_names_the_count`, `..._refuses_an_unknown_job_and_reads_a_known_one`, `..._leaves_the_stdin_form_alone` | herding.rs (tests module) |
| cell_brief_search_rule_and_size | `..._keeps_the_fixed_text_under_the_cut`, `..._names_every_result_field_and_status` | herding/mailbox.rs tests |
| cell_brief_search_rule_and_size | `..._rendered_prompt_holds_the_search_rule_once` | verbs/drivers/tests.rs:178 |

What each truth rests on:

- Thin env: the const holds `SLASH_COMMAND_TOOL_CHAR_BUDGET=1`; `thin_peer_env` adds it when the switch is on and the launch is claude, and leaves it out when the switch is off or the launch is not claude. A thin claude-session spawn argv and a thin `claude-sonnet` pane argv hold `--disable-slash-commands --disallowedTools Agent`; with `thin_peer_boot=false` none of the three appear.
- Handoff door: a sessionless merge writes no `.bee/runtime/merged-by` directory; a merge in session `sess-merger-1` stores feature `alpha` for that session only. Feature start in a session that merged another feature exits 1, and writes no worktree, no lane file and no audit row. With `--same-session "<reason>"` it exits 0 and one `handoff-waived` audit row holds the reason (a blank reason is refused, nothing written). A first start and a sessionless start succeed.
- Digest: text and `--json` outputs carry status, summary, proof, files_changed, found, unknowns, skipped, report_path. A 6000-character summary gives at most 1500 characters, ends with ` [+N cut]` where N is the exact count cut, and keeps report_path. An unknown, empty or `../` job id and an unparseable result exit 1; a good job exits 0. `job_digest_args` is `None` for the stdin flag shapes, so that form still goes to `result_reader`.
- Brief: the cell template holds `Search rule:` and `` `rg -l` `` once. Fixed bytes, measured with `render_brief` on an empty task with short fixed paths (path bytes subtracted) plus `packages/bee/prompts/worker-cell.md`: HEAD 11350 (5885 brief with paths, 5645 template). The same fixture at the rrt-4 parent: 15164 (8111 + 7233), which matches the 15227 baseline within path noise, so the fixture is a fair ruler. The test asserts at most 11420, which is 75 percent of 15227. The brief names every MailboxResult and handback field (status, summary, files_changed, proof, report_path, knowledge_used, principles_used, sync_ack, knowledge_wrong, options, leaning, dissent, reaction, skipped, unknowns, found, stopped, question) and the statuses done, blocked, question, plus the four reaction words.

## Red proof per contract

Method: reverse-apply the code cell's production diff (`git show <commit> -- <files> | git apply -R`; for the worker-cell.md file `git show ba212afb5^:<path>`, because rrt-9 later edited the same file), run the tests, then re-apply and confirm `git diff --stat` is unchanged (7 files, 392 insertions) before the green run. Private target dir CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-rrt.

- rrt-1 (parent of 7359f63a0). Compile error, the new const and signature are gone:

      error[E0425]: cannot find value `THIN_CLAUDE_FLAGS` in this scope
      error[E0061]: this function takes 5 arguments but 6 arguments were supplied
      error: could not compile `bee` (bin "bee" test) due to 4 previous errors

- rrt-2 (parent of 1a2bf4de6, feature_group.rs and handlers.rs reversed; `integration_queue.rs` kept). Assertion failures:

      test ...feature_group::tests::handoff_after_merge_door_refuses_a_second_feature_and_writes_nothing ... FAILED
        feature_group.rs:681: left: Some(ExitCode(0)) right: Some(ExitCode(1))
      test ...feature_group::tests::handoff_after_merge_door_same_session_with_a_reason_succeeds_and_is_audited ... FAILED
        feature_group.rs:696: left: None right: Some(ExitCode(1))
      test ...worktree::tests::handoff_after_merge_door_a_session_merge_records_the_merger_and_a_sessionless_one_does_not ... FAILED
        worktree/tests.rs:970: left: None right: Some("alpha")
      test result: FAILED. 1 passed; 3 failed

  The one that passes at the parent is the first-start and sessionless case, which must pass both before and after.

- rrt-3 (parent of 9f3bbc9ba). Compile error, the digest functions are gone:

      error[E0425]: cannot find function `job_result_digest` in this scope
      error[E0425]: cannot find function `job_digest_args` in this scope
      error[E0425]: cannot find value `RESULT_DIGEST_LIMIT` in this scope
      error: could not compile `bee` (bin "bee" test) due to 16 previous errors

- rrt-4 (parent of ba212afb5; mailbox.rs and worker-cell.md at the parent). Assertion failures:

      fixed brief bytes 15164 exceed 75 percent of 15227
      field knowledge_used missing
      assertion `left == right` failed: Nickname (reservation identity): w   left: 0  right: 1   (Search rule count)
      test result: FAILED. 0 passed; 3 failed

For rrt-1 and rrt-3 the red is a compile error, because the contract adds items the tests name. A reviewer who wants an assertion-level red can stub the const and the function; I did not, since the brief allows a compile error.

## rrt-5 and rrt-6 parity

`wc -c packages/bee/LEADER.md` = 14915 (limit 15000, was 21685). `.bee/LEADER.md` and `.bee/playbooks/leader-detail.md` are byte-identical to the sources (`cmp`).

rrt-5 must_haves:

1. At most 15000 bytes: packages/bee/LEADER.md, 14915 bytes. Holds.
2. Each of the 14 rule markers kept with its boundary sentence: the markers are single-line `<!-- rule: <id> -->` lines at packages/bee/LEADER.md:33 (agents-review-user-invoked), :43 (gates-never-self-approved), :56 (worktree-first), :84 (name-applied-principles), :122 (knowledge-before-code), :133 (proof-at-cap), :146 (never-build-on-red), :152 (verify-before-agreeing), :191 (never-zero-execution-workers), :215 (capture-line-at-close), :226 (feedback-changes-framework), :235 (one-next-action), :265 (one-commit-per-cell), :271 (no-code-comments). The old file's 15th id string, agents-md-duplication-boundary, was a file path inside a sentence, not a marker, and it survives at leader-detail.md:61. Holds.
3. Every sentence that left is in full under packages/bee/playbooks/: I split `git show c9419a5d1^:packages/bee/LEADER.md` into 175 sentences, removed the marker comments, and looked each one up (whitespace folded) in the new LEADER.md plus leader-detail.md: 0 missing. LEADER.md:20 names `.bee/playbooks/leader-detail.md` with a read-when line. Holds.
4. No rule deleted or weakened, moved blocks listed: follows from 3; the moved blocks are the `Detail: leader-detail § ...` pointers at LEADER.md:31, 49, 55, 65, 82, 104, 112, 120, 144, 150, 167, 190, 224, 244, 256, 277, 284. Holds. The rrt-5 report was not in the tree for me to compare the list against.
5. Pinned strings still match: the leader added the LEADER-pinned unit targets (onboard, session_preamble) to the rrt-5 verify and ran them; I did not rerun them (outside this verify). Not checked by me.
6. `.bee/LEADER.md` and `.bee/playbooks/` refreshed: `cmp` equal for LEADER.md and leader-detail.md. Holds.

rrt-6 must_haves:

1. skills/bee-swarming/SKILL.md step 5: skills/bee-swarming/SKILL.md:91-93 ("read `bee herding result --job <id>` for the digest. Open the report only on a red, a question or a blocked status. Start a diff review with the stat view."). Rendered in all five trees (.agents, .claude-plugin, .claude, .codex-plugin, .opencode), one hit each. Holds.
2. cells.md path plus symbol: skills/bee-planning/references/cells.md:59-60. Holds.
3. handoff playbook: packages/bee/playbooks/handoff.md:34 and the same line in .bee/playbooks/handoff.md:34 (files identical). Holds.
4. Token-economy concept: docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md:175 (thin switches; 20739, 14803, 8887 all on the line), :176 handoff door D1, :177 digest D2, :178 search rule D4, :179 LEADER.md size D5, :180 brief size D6; frontmatter decisions list reread-thrift D1 to D6 at line 11 and anchors at line 14. The six lines carry no `deferred`, `until` or `later`. Holds.
5. Rendered trees and .bee/playbooks/handoff.md regenerated and committed: all five trees carry the new line; handoff.md is identical. Holds.

## Judge verdicts

```json rrt-1
{"schema":"judge-verdict/1","cell":"rrt-1","commit":"7359f63a0","verdict":"PASS","checks":[
{"id":"thin-env-budget","status":"PASS","evidence":"packages/bee-rs/crates/bee/src/herding/run.rs:2965 adds (SLASH_COMMAND_TOOL_CHAR_BUDGET,1) to THIN_PEER_CLAUDE_ENV; run.rs:13946 test shows it in thin_peer_env on and absent when off or not claude"},
{"id":"claude-session-argv","status":"PASS","evidence":"claude_session.rs:120 THIN_CLAUDE_FLAGS, :139 appended before the prompt when thin; run.rs:4052 passes thin_peer_switch_on(&main_cfg); test run.rs thin_worker_listings_off_claude_session_argv_follows_the_thin_switch"},
{"id":"pane-argv","status":"PASS","evidence":"run.rs:2768-2769 extends the agent_start args only for a claude kind with the switch on; test built_in_claude_pane_argv asserts args end with the three flags and are absent with the switch off"},
{"id":"non-thin-untouched","status":"PASS","evidence":"claude_session.rs:138 guards on thin; pane test with thin_peer_boot=false holds none of the three"},
{"id":"paseo-argv-unflagged","status":"PASS","evidence":"run.rs:2960 the const also reaches paseo env, but the cell names claude-session and pane launches only; no paseo argv flags added, recorded as unknown, not a fail"}
],"fixability":"automatic","confidence":"high"}
```

```json rrt-2
{"schema":"judge-verdict/1","cell":"rrt-2","commit":"1a2bf4de6","verdict":"PASS","checks":[
{"id":"merge-stores-merger","status":"PASS","evidence":"verbs/worktree/handlers.rs:1490-1497 records session and feature only when answer.ok, code is not ALREADY_UP_TO_DATE and the session is not the sessionless id; worktree/tests.rs:970 shows alpha stored, no merged-by directory for a sessionless merge"},
{"id":"start-refuses-handoff-due","status":"PASS","evidence":"verbs/feature_group.rs:194-203 refuses with [handoff_due] before the worktree, lane or audit write; test refuses_a_second_feature_and_writes_nothing asserts exit 1 and no worktree, lane file or audit row"},
{"id":"same-session-reason-audited","status":"PASS","evidence":"feature_group.rs:205-226 non-empty trimmed reason then audit::append kind handoff-waived; test asserts one row with the reason; blank reason refused with nothing written"},
{"id":"first-and-sessionless-pass","status":"PASS","evidence":"feature_group.rs:194-197 requires a session id and a stored feature different from the new one; test lets_a_first_start_and_a_sessionless_start_through passes at HEAD and at the parent"},
{"id":"rrt-9-fix-first","status":"PASS","evidence":"commits 372a81322 and b1e42e2b1 move the flag count pin to 259 (the --same-session flag added at registry_payload.json) and restore the proof-line example; verify of this cell is green on HEAD"}
],"fixability":"automatic","confidence":"high"}
```

```json rrt-3
{"schema":"judge-verdict/1","cell":"rrt-3","commit":"9f3bbc9ba","verdict":"PASS","checks":[
{"id":"digest-fields","status":"PASS","evidence":"herding.rs:1861-1887 digest_fields emits status, summary, proof, files_changed, found, unknowns, skipped, report_path; test prints_every_field_in_text_and_json"},
{"id":"cut-at-1500","status":"PASS","evidence":"herding.rs:1812 RESULT_DIGEST_LIMIT 1500 and the shrink loop in job_result_digest; test shows at most 1500 characters for text and json, suffix ` [+N cut]` equals the exact count cut, report_path kept"},
{"id":"typed-refusal","status":"PASS","evidence":"herding.rs:1924 and :1932 refuse an unknown job id with a named message and exit 1; test covers missing, empty, ../ and unparseable. The refusal is a plain stderr message, not a bracketed code, so `typed` is read as a distinct exit and named reason"},
{"id":"stdin-form-unchanged","status":"PASS","evidence":"herding.rs:1805-1810 envelope_result goes to result_reader whenever job_digest_args is None; test leaves_the_stdin_form_alone"}
],"fixability":"automatic","confidence":"medium"}
```

```json rrt-4
{"schema":"judge-verdict/1","cell":"rrt-4","commit":"ba212afb5","verdict":"PASS","checks":[
{"id":"search-rule-once","status":"PASS","evidence":"packages/bee/prompts/worker-cell.md:41 holds the Search rule line; test rendered_prompt_holds_the_search_rule_once counts one Search rule and one `rg -l`"},
{"id":"size-cut-25-percent","status":"PASS","evidence":"fixed bytes 15164 at the parent and 11350 at HEAD on the same fixture (-25.1%); the leader figures 15227 to 11413; the test asserts at most 11420"},
{"id":"every-field-and-status-named","status":"PASS","evidence":"herding/mailbox.rs render_brief names all MailboxResult and handback fields and done, blocked, question; knowledge_wrong appears in backticks, the rest in quotes; test names_every_result_field_and_status"},
{"id":"rrt-9-pins","status":"PASS","evidence":"372a81322 pins the flag vocabulary at 259 and restores the proof-line example; b1e42e2b1 shortens the wording to reach the cut; the cell verify is green at HEAD"},
{"id":"regen-wait","status":"PASS","evidence":"the one test that waited on the rrt-5 regen is covered by a6307cbb7 and c9419a5d1; no skipped test in the verify"}
],"fixability":"automatic","confidence":"high"}
```

## Found, unknowns, skipped

- found: `handoff-waived` is not in `KINDS` at packages/bee-rs/crates/bee/src/audit.rs:6, so `bee audit log --kind handoff-waived` would refuse the kind that feature start writes. Production code; out of this cell's scope.
- found: the audit row for `--same-session` is written before the lane and worktree are created (feature_group.rs:213-226); a start that fails later leaves a `handoff-waived` row behind. Not reproduced.
- found: `record_merger` ignores its write errors (integration_queue.rs:638-647), so a failed write silently turns the door off. This fails open, which the cell did not say either way.
- skipped: stdout capture of `bee herding result --job` (a gap in my tests, not a defect found in the code); a subprocess test is the way to do it.
- skipped: the LEADER.md pinned-string unit tests (onboard, session_preamble) and the full suite; the brief says to run only the cell verify.
- unknowns: whether thin paseo launches should also carry the argv flags; the cell names claude-session and pane launches only.
