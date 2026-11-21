---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: cut the tokens re-read on every call

## Summary

Six cuts from CONTEXT.md D1 to D6: a handoff door at feature start, a result
digest for the leader, a thin worker launch without the skill and agent
listings, a search rule in the cell prompt, a shorter leader doctrine file,
and a smaller fixed brief.

Mode: `standard` — four Rust cells, two docs cells with a regen, one test
cell that also judges, one writer cell.

## Requirements (from CONTEXT.md)

- D1: feature start refuses a second feature in one session unless `--same-session` carries a reason.
- D2: `bee herding result --job <id>` prints a digest of at most 1500 characters; the leader reads it in place of the report.
- D3: thin claude launches get `SLASH_COMMAND_TOOL_CHAR_BUDGET=1`, and the two flags where bee builds the argv.
- D4: a search rule in the cell prompt; a cell action names path plus symbol.
- D5: `packages/bee/LEADER.md` at most 15000 bytes with every rule kept.
- D6: fixed brief text at least 25 percent smaller.
- C1, C2, C3, C4: contracts thin-worker-listings-off, handoff-after-merge-door, herding-result-job-digest, cell-brief-search-rule-and-size.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | One const holds the thin worker env | read | packages/bee-rs/crates/bee/src/herding/run.rs:2956 | `pub(crate) const THIN_PEER_CLAUDE_ENV: [(&str, &str); 4] = [` |
| 2 | One function builds the claude-session argv | read | packages/bee-rs/crates/bee/src/herding/claude_session.rs:120 | `pub(crate) fn spawn_argv(name: &str, model: &str, effort: Option<&str>, settings: &str, prompt: &str) -> Vec<String> {` |
| 3 | The merge already prints the handoff directive | read | packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs:5651 | `"then write the handoff and continue in a fresh leader session (one leader session per feature)."` |
| 4 | Feature start has one entry | read | packages/bee-rs/crates/bee/src/verbs/feature_group.rs:117 | `fn run_start(tokens: &[OsString], t0: Instant) -> Option<ExitCode> {` |
| 5 | The herding result verb has one entry | read | packages/bee-rs/crates/bee/src/herding.rs:1805 | `pub(crate) fn envelope_result(flags: &[&str]) -> ExitCode {` |
| 6 | One function renders the brief | read | packages/bee-rs/crates/bee/src/herding/mailbox.rs:287 | `pub(crate) fn render_brief(spec: &BriefSpec) -> String {` |
| 7 | The cell prompt text is one template | read | packages/bee/prompts/worker-cell.md:25 | `The cell names five parts apart:` |
| 8 | The env switch and flags cut the worker's first-call context and hooks still fire | ran | docs/history/reread-thrift/CONTEXT.md:61 | `claude -p` in this repo, two runs each: 20739, 14803, 13041, 8868, 8887 tokens; hook_response rows for SessionStart, PreToolUse, Stop |

## Approach

Situation: `skills/bee-planning/playbooks/perf/perf-issue.md`. architect
skipped: each change extends one existing function; the one new stored
shape is the merger session record of D1.

The playbook says to change one thing per measurement. D3 is the one cut
with a direct before and after number, taken with one command. D1, D2, D4,
D5 and D6 change bytes that a later real run re-reads; their proof is the
contract test plus a byte count, and the next feature run gives the spend
number.

Throughput checkpoint:
- Blocking first steps: none.
- Independent workstreams: rrt-1, rrt-2 and rrt-4 touch disjoint files and run at once.
- Shared mutable state: rrt-3 follows rrt-2 because both add a flag to the command registry; rrt-5 and rrt-6 regenerate rendered files alone, after the code cells.
- Smallest safe decomposition: one tester that also judges and checks the docs cells.

SMALLER PATH: doctrine text alone would cover D1, D2 and D4 with no code.
Rejected: token-spend-cuts D1 already prints the directive at the merge and at the context mark; a printed line is advice, and the measured session ran three features in one context.

Rejected: a paseo argument channel (paseo passes only env to the agent);
`CLAUDE_CODE_MAX_SUBAGENTS_PER_SESSION=0` (measured 20758, no cut).

Hat wave not opened: the ask is clear and each choice has a measurement.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "cd477eb57b66a0cef7310533c504a43a20e8b19e131431e73112bda9a9f114c8",
  "stages": [
    {
      "stage": "implementation",
      "classification": "required",
      "role": "code",
      "reason": "rrt-1 to rrt-4 change Rust code and the cell prompt."
    },
    {
      "stage": "test-and-live-proof",
      "classification": "required",
      "role": "test",
      "reason": "rrt-7 proves the contracts, judges the code cells and checks the docs cells."
    },
    {
      "stage": "docs",
      "classification": "required",
      "role": "docs",
      "reason": "rrt-5 and rrt-6 change the leader doctrine, two skills, a playbook and a concept."
    },
    {
      "stage": "writing",
      "classification": "required",
      "role": "writer",
      "reason": "rrt-8 consolidates knowledge and writes delivery.md."
    },
    {
      "stage": "read-only-gather",
      "classification": "required",
      "role": "read",
      "reason": "One code scout ran before the plan."
    },
    {
      "stage": "slice-judge",
      "classification": "not-applicable",
      "role": "review",
      "reason": "token-spend-cuts D5: the tester returns the judge blocks."
    },
    {
      "stage": "hat-facts-gaps",
      "classification": "not-applicable",
      "role": "hat-facts-gaps",
      "reason": "Clear ask with measured evidence; the hat wave threshold is not met."
    },
    {
      "stage": "hat-alternatives",
      "classification": "not-applicable",
      "role": "hat-alternatives",
      "reason": "The launch alternatives were measured side by side."
    },
    {
      "stage": "hat-user-impact",
      "classification": "not-applicable",
      "role": "hat-user-impact",
      "reason": "CLI text and prompts; no UI."
    },
    {
      "stage": "hat-risks",
      "classification": "not-applicable",
      "role": "hat-risks",
      "reason": "Each risk has a guard in the risk map."
    },
    {
      "stage": "hat-value",
      "classification": "not-applicable",
      "role": "hat-value",
      "reason": "Owner-requested with spend evidence."
    },
    {
      "stage": "deployment",
      "classification": "not-applicable",
      "role": "deploy",
      "reason": "No release in this feature."
    },
    {
      "stage": "planning",
      "classification": "not-applicable",
      "role": "plan",
      "reason": "The leader wrote the plan."
    },
    {
      "stage": "fact-extraction",
      "classification": "not-applicable",
      "role": "extraction",
      "reason": "No extraction stage."
    },
    {
      "stage": "generation-fallback",
      "classification": "not-applicable",
      "role": "generation",
      "reason": "Every cell has a configured role."
    },
    {
      "stage": "generic-advisor",
      "classification": "not-applicable",
      "role": "advisor",
      "reason": "No advisor consult."
    },
    {
      "stage": "supervision",
      "classification": "not-applicable",
      "role": "supervisor",
      "reason": "Attended session."
    },
    {
      "stage": "blind-lane-1",
      "classification": "not-applicable",
      "role": "lane-1",
      "reason": "No blind lanes."
    },
    {
      "stage": "blind-lane-2",
      "classification": "not-applicable",
      "role": "lane-2",
      "reason": "No blind lanes."
    },
    {
      "stage": "blind-lane-3",
      "classification": "not-applicable",
      "role": "lane-3",
      "reason": "No blind lanes."
    },
    {
      "stage": "design-ui",
      "classification": "not-applicable",
      "role": "design",
      "reason": "No rendered UI."
    }
  ]
}
```

## Risk map

| Risk | Guard |
|---|---|
| A thin worker needs a skill it can no longer list | Workers load principles through the skillstation CLI; the switches apply only where the thin env applies; C1 pins that other launches are unchanged |
| The handoff door blocks a wanted second feature | `--same-session` with a reason; a sessionless call passes |
| A leader rule is lost in the LEADER.md cut | All 14 markers stay; moved text lands in full in a named playbook; rule parity tests |
| A cut brief drops a result field | C4 pins every field and status |
| The digest hides a red | The digest prints status and proof first; the leader opens the report on a red, a question or a blocked status |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| rrt-1 | Start a thin claude worker without the skill and agent listings | 3 files | — | worker starts lighter | herding launch tests |
| rrt-2 | Refuse a second feature in the session that merged the first | 5 files | — | handoff line, refusal | worktree and feature tests |
| rrt-3 | Print a short digest of a job's result | 2 files | rrt-2 | short digest | herding tests |
| rrt-4 | Give the cell prompt a search rule and cut the fixed brief text | 3 files | — | smaller brief | mailbox and drivers tests |
| rrt-5 | Shrink the leader doctrine file and keep every rule | 7 files | rrt-1, rrt-2, rrt-3, rrt-4 | shorter leader file | rule parity tests |
| rrt-6 | Put the new rules in the doctrine and the knowledge | 6 files | rrt-5 | doctrine matches | manifest and knowledge check |
| rrt-7 | Prove the four contracts and judge the code cells | 9 files | rrt-1, rrt-2, rrt-3, rrt-4, rrt-5, rrt-6 | contracts pass | contract filters |
| rrt-8 | Consolidate the feature's knowledge and write the delivery note | 2 files | rrt-1, rrt-2, rrt-3, rrt-4, rrt-5, rrt-6, rrt-7 | delivery note | knowledge check |

```json
[
{"id": "rrt-1", "title": "Start a thin claude worker without the skill and agent listings", "role": "code", "deps": [], "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- herding::run herding::claude_session herding::wave", "must_haves": {"truths": ["THIN_PEER_CLAUDE_ENV in herding/run.rs holds the pair SLASH_COMMAND_TOOL_CHAR_BUDGET=1, so every launch that receives that env list receives it", "a claude-session spawn for a thin worker (the same condition that applies the thin peer env) has --disable-slash-commands and --disallowedTools Agent in its argv before the prompt; a spawn that is not thin has neither", "the built-in claude herding pane argv for a thin worker has the same two flags; a pane that is not thin, and every non-claude agent, is unchanged", "an operator env override of SLASH_COMMAND_TOOL_CHAR_BUDGET wins the same way the existing overrides of the thin env pairs win", "existing herding::run, herding::claude_session and herding::wave tests pass, updated only where a pinned argv or env count breaks"], "why": "A thin worker's first-call context is 20739 tokens; with the env switch 14803, with the flags 8887 (D3).", "out_of_scope": ["paseo argv", "the worker guard", "new tests"]}, "decisions": ["6d55d3dd-1d8d-46d9-92a9-3645f14c11c2", "1dd8b534-956a-48e5-9b11-9ba3a90a9d45"], "files": ["packages/bee-rs/crates/bee/src/herding/run.rs", "packages/bee-rs/crates/bee/src/herding/claude_session.rs", "packages/bee-rs/crates/bee/src/herding/wave.rs"], "action": "Implement CONTEXT.md D3. In packages/bee-rs/crates/bee/src/herding/run.rs add the env pair to THIN_PEER_CLAUDE_ENV (line 2956; thin_peer_env line 2967 decides when it applies). Thread a thin flag to packages/bee-rs/crates/bee/src/herding/claude_session.rs spawn_argv (line 120; caller in run.rs near 4041) and add the two flags there when thin. For the herding pane, find where run.rs starts the pane agent (herdr.agent_start near 2768, args from packages/bee-rs/crates/bee/src/herding/wave.rs resolve_agent_command line 397, built-in claude argv near 195) and append the two flags when the command is claude (is_claude_command) and the thin env applies. Update only existing assertions that break; add no new tests (rrt-7 owns them). Locate with rg -l, then rg -n on one named file, and read ranges only. Write no code comment. Run only the cell verify, never the full suite. One commit with the cell trailer, files staged by explicit path. If a file outside the list must change, stop and say so in found. In your result put principles_used as an array of full slugs (bee-principle-<name> — <the decision it changed>), [] when none, and sync_ack with a reason if the cap sync door asks.", "feature": "reread-thrift", "lane": "standard", "read_first": ["docs/history/reread-thrift/CONTEXT.md"], "affects_specs": [], "affects_skills": []},
{"id": "rrt-2", "title": "Refuse a second feature in the session that merged the first", "role": "code", "deps": [], "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- verbs::worktree verbs::feature_group", "must_haves": {"truths": ["the merge text is unchanged: it already ends with the handoff directive (verbs/worktree/phases.rs near 879)", "a successful merge stores the merging session id and the merged feature where feature start can read it; a sessionless merge stores nothing", "bee feature start, before it creates anything, refuses with the typed reason handoff_due when the calling session id is stored as the merger of a different feature; the refusal names --same-session and the handoff playbook, and no worktree, lane or route is written", "bee feature start with --same-session and a non-empty reason proceeds and writes an audit row of kind handoff-waived that holds the reason; an empty reason is refused", "a first feature start in a session, and a call with no resolvable session id, behave as before", "the --same-session flag is declared wherever the dispatcher and the CLI-shape guard read verb flags, so the guard accepts it", "existing worktree and feature_group tests pass unchanged"], "why": "A context reset at each merge would have cut 35M of 76M leader tokens; the rule was prose only (D1).", "out_of_scope": ["writing the handoff itself", "context size detection", "new tests"]}, "decisions": ["b56a554a-4d23-45fd-a6cd-636360d06e55", "cd3be8ac-5e4d-4629-9bca-995d94ed7775"], "files": ["packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs", "packages/bee-rs/crates/bee/src/verbs/feature_group.rs", "packages/bee-rs/crates/bee/src/verbs/worktree/mod.rs", "packages/bee-rs/crates/bee/src/integration_queue.rs", "packages/bee-rs/crates/bee/src/generated/registry_payload.json"], "action": "Implement CONTEXT.md D1. The merge text stays as it is. The merge already resolves a session id (near 1397-1400) and the integration queue request stores session_id (packages/bee-rs/crates/bee/src/integration_queue.rs near 392); reuse that record if it outlives the merge, else write one small record under .bee through an existing store helper. Feature start: packages/bee-rs/crates/bee/src/verbs/feature_group.rs run_start (line 117; resolve_session_id near 192) checks the record before any write. Declare the flag where the other feature start flags are declared (rg -l with-companion shows the flag homes; regenerate packages/bee-rs/crates/bee/src/generated/registry_payload.json with the project's own generator if it is generated). Update only existing assertions that break; add no new tests (rrt-7 owns them). Locate with rg -l, then rg -n on one named file, and read ranges only. Write no code comment. Run only the cell verify, never the full suite. One commit with the cell trailer, files staged by explicit path. If a file outside the list must change, stop and say so in found. In your result put principles_used as an array of full slugs (bee-principle-<name> — <the decision it changed>), [] when none, and sync_ack with a reason if the cap sync door asks.", "feature": "reread-thrift", "lane": "standard", "read_first": ["docs/history/reread-thrift/CONTEXT.md"], "affects_specs": [], "affects_skills": []},
{"id": "rrt-3", "title": "Print a short digest of a job's result", "role": "code", "deps": ["rrt-2"], "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- herding", "must_haves": {"truths": ["bee herding result --job <id> reads that job's latest result file from the job mailbox under the control root and parses it with parse_result_text (herding/mailbox.rs)", "it prints, one field per line, status, summary, proof, files_changed, then found, unknowns and skipped from the handback when present, then report_path", "the whole output is at most 1500 characters; a field that is cut ends with the count of characters cut; the report body is never read or printed", "an unknown job id or a job with no result file is a typed refusal with a non-zero exit", "with the json flag it prints the same fields as one object", "bee herding result without --job reads the stdin envelope exactly as before", "the --job flag is declared wherever the dispatcher and the CLI-shape guard read verb flags", "existing herding tests pass unchanged"], "why": "The leader read whole reports with cat: raw reads were 25 percent of its re-read weight (D2).", "out_of_scope": ["herding status", "the result file format", "new tests"]}, "decisions": ["5c368ef5-0a5d-463b-9188-19f19daa4752", "7e81e0f2-59bb-462b-a9d4-9ed7f8df07ce"], "files": ["packages/bee-rs/crates/bee/src/herding.rs", "packages/bee-rs/crates/bee/src/generated/registry_payload.json"], "action": "Implement CONTEXT.md D2 in packages/bee-rs/crates/bee/src/herding.rs: envelope_result (line 1805) and result_reader (line 1824) are the stdin form; the verb is routed near line 148. Add the --job form beside them. Find how herding status locates a job mailbox (rg -n job.json in packages/bee-rs/crates/bee/src/herding.rs) and reuse it. Parse with crate::herding::mailbox::parse_result_text (mailbox.rs line 786; MailboxResult near 577). Update only existing assertions that break; add no new tests (rrt-7 owns them). Locate with rg -l, then rg -n on one named file, and read ranges only. Write no code comment. Run only the cell verify, never the full suite. One commit with the cell trailer, files staged by explicit path. If a file outside the list must change, stop and say so in found. In your result put principles_used as an array of full slugs (bee-principle-<name> — <the decision it changed>), [] when none, and sync_ack with a reason if the cap sync door asks.", "feature": "reread-thrift", "lane": "standard", "read_first": ["docs/history/reread-thrift/CONTEXT.md"], "affects_specs": [], "affects_skills": []},
{"id": "rrt-4", "title": "Give the cell prompt a search rule and cut the fixed brief text", "role": "code", "deps": [], "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- herding::mailbox verbs::drivers", "must_haves": {"truths": ["packages/bee/prompts/worker-cell.md holds one short Search rule block: locate with rg -l or one rg -n on a named path, bound every search output (a max count or a named file), read a range with offset and limit, never print a whole file", "the fixed text of a non-read-only cell brief is at least 25 percent smaller in bytes: measure render_brief for one fixed BriefSpec with an empty task plus the bytes of worker-cell.md, before and after, and write both numbers in the result summary", "the brief still names every MailboxResult field (round, status, summary, files_changed, proof, report_path, the handback fields, options, leaning, dissent, knowledge_used, principles_used, sync_ack, knowledge_wrong, question) and every status the parser accepts", "the ack step, the report-first rule and the temp-file-then-rename rule are still stated", "a read-only brief keeps its Shell rules block", "the template placeholders of worker-cell.md are unchanged and the template test every_template_carries_the_original_request_block_and_matches_disk passes", "existing herding::mailbox and verbs::drivers tests pass, with pinned wording updated to the new text"], "why": "rg and fd output was 41 percent of worker tool output; 25 briefs shared 10.6k characters of fixed text (D4, D6).", "out_of_scope": ["the result parser", "the read-only guard", "new tests"]}, "decisions": ["b4521fc9-3c8d-4646-946d-dc7bfd66e7f6", "acb2c2d8-1e44-4837-a250-106dee765cce", "5737bdba-2382-4086-88ca-68db5e094f2d"], "files": ["packages/bee/prompts/worker-cell.md", "packages/bee-rs/crates/bee/src/herding/mailbox.rs", "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"], "action": "Implement CONTEXT.md D4 and D6. Prompt text: packages/bee/prompts/worker-cell.md (five-parts block lines 25-33, learned context 43-47, principles 48-52). Brief: packages/bee-rs/crates/bee/src/herding/mailbox.rs render_brief (line 287; ack 396-414, executor 415-426, report 440-456, result contract 457-509, how to write 511-519). Cut by saying each rule once in short sentences, never by dropping a rule or a field. Pinned wording lives in mailbox.rs tests (near 1284-1775, the result-form block near 1496) and packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs (near 10981, 11085, 13281); update those pins to the new text. Add no new tests (rrt-7 owns them). Locate with rg -l, then rg -n on one named file, and read ranges only. Write no code comment. Run only the cell verify, never the full suite. One commit with the cell trailer, files staged by explicit path. If a file outside the list must change, stop and say so in found. In your result put principles_used as an array of full slugs (bee-principle-<name> — <the decision it changed>), [] when none, and sync_ack with a reason if the cap sync door asks.", "feature": "reread-thrift", "lane": "standard", "read_first": ["docs/history/reread-thrift/CONTEXT.md"], "affects_specs": [], "affects_skills": []},
{"id": "rrt-5", "title": "Shrink the leader doctrine file and keep every rule", "role": "docs", "deps": ["rrt-1", "rrt-2", "rrt-3", "rrt-4"], "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity --test pointer_integrity --test knowledge_first_nudge --test verification_contract_parity", "must_haves": {"truths": ["packages/bee/LEADER.md is at most 15000 bytes (wc -c), down from 21685", "each of the 14 rule markers stays in it, opening and closing tag, with the sentence that states the boundary: agents-review-user-invoked, agents-gates-never-self-approved, agents-worktree-first, agents-name-applied-principles, agents-knowledge-before-code, agents-proof-at-cap, agents-never-build-on-red, agents-verify-before-agreeing, agents-never-zero-execution-workers, agents-capture-line-at-close, agents-feedback-changes-framework, agents-one-next-action, agents-one-commit-per-cell, agents-no-code-comments", "every sentence that leaves the file is present, in full, in a file under packages/bee/playbooks/ (new or existing), and LEADER.md's Playbooks list names that file with a read-when line", "no rule is deleted or weakened; the report lists each moved block with its new home", "strings that tests pin (rg -n LEADER in packages/bee-rs/crates/bee/tests and in src/onboard/tests.rs and src/hooks/session_preamble/tests.rs) still match, or the pin is updated in the same commit", "the repo's own .bee/LEADER.md and .bee/playbooks/ are refreshed from the source by the worktree-built binary"], "why": "The leader re-reads this file on every call: about 5.6k tokens times 334 calls (D5).", "out_of_scope": ["AGENTS.md", "the root LEADER.md", "skill bodies"]}, "decisions": ["4c6e84ea-d386-4ac0-b989-590c55928468"], "files": ["packages/bee/LEADER.md", "packages/bee/playbooks/leader-detail.md", ".bee/LEADER.md", ".bee/playbooks/leader-detail.md", "packages/bee-rs/crates/bee/src/onboard/tests.rs", "packages/bee-rs/crates/bee/src/onboard/source.rs", "packages/bee-rs/crates/bee/tests/rule_index_parity.rs"], "action": "Implement CONTEXT.md D5. First find where playbooks are sourced and copied (rg -n playbooks packages/bee-rs/crates/bee/src/onboard/source.rs) so a new playbook file ships and is copied. Keep in LEADER.md the boundary sentence of each rule; move procedure detail, examples, full forms and long lists to packages/bee/playbooks/leader-detail.md under one heading per rule id, with a read-when line in the Playbooks list. Good first candidates: the route-each-task procedure, the triage of proposed PBIs, the long forms inside agents-verify-before-agreeing, agents-one-next-action, agents-no-code-comments and the mistake-recording bullet. Build with CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-rrt cargo build --release --manifest-path packages/bee-rs/Cargo.toml and run that binary's dev regen (/home/thanhsmind/.cache/cargo-target-rrt/release/bee dev regen); never ./.bee/bin/bee for regen. Restore .claude/skills changes you did not cause. Then run the verify. Locate with rg -l, then rg -n on one named file, and read ranges only. Write no code comment. Run only the cell verify, never the full suite. One commit with the cell trailer, files staged by explicit path. If a file outside the list must change, stop and say so in found. In your result put principles_used as an array of full slugs (bee-principle-<name> — <the decision it changed>), [] when none, and sync_ack with a reason if the cap sync door asks.", "feature": "reread-thrift", "lane": "standard", "read_first": ["docs/history/reread-thrift/CONTEXT.md"], "affects_specs": [], "affects_skills": []},
{"id": "rrt-6", "title": "Put the new rules in the doctrine and the knowledge", "role": "docs", "deps": ["rrt-5"], "verify": "./.bee/bin/bee dev release-manifest --check && ./.bee/bin/bee knowledge check", "must_haves": {"truths": ["skills/bee-swarming/SKILL.md Orchestrate step 5 says: read bee herding result --job <id> for the digest; open a report only on a red, a question or a blocked status; start a diff review with the stat view", "skills/bee-planning/references/cells.md Cell quality rules says a cell action names each function it changes as path plus symbol", "packages/bee/playbooks/handoff.md says feature start refuses a second feature in the same session unless --same-session carries a reason", "docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md states the thin worker switches with the measured numbers 20739, 14803 and 8887, the handoff door, the result digest, the search rule, the LEADER.md size limit and the brief size cut, citing reread-thrift D1 to reread-thrift D6, each D-number whole on one line", "the rendered skill trees and the repo's .bee/playbooks/handoff.md are regenerated with the binary built from this worktree and committed"], "why": "Doctrine and docs/knowledge are what every project receives and reads.", "out_of_scope": ["Rust code"]}, "decisions": ["b56a554a-4d23-45fd-a6cd-636360d06e55", "5c368ef5-0a5d-463b-9188-19f19daa4752", "6d55d3dd-1d8d-46d9-92a9-3645f14c11c2", "b4521fc9-3c8d-4646-946d-dc7bfd66e7f6", "4c6e84ea-d386-4ac0-b989-590c55928468", "acb2c2d8-1e44-4837-a250-106dee765cce"], "files": ["skills/bee-swarming/SKILL.md", "skills/bee-planning/references/cells.md", "packages/bee/playbooks/handoff.md", ".bee/playbooks/handoff.md", "docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md", "docs/history/codex-harness-hardening/release-manifest.json"], "action": "Edit the four files in short plain sentences. In the concept, find the paragraph with rg -n, cite reread-thrift D-numbers (never raw decision ids), add them to frontmatter decisions and the code files to anchors; avoid the words deferred, until and later. Build with CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-rrt cargo build --release --manifest-path packages/bee-rs/Cargo.toml and run that binary's dev regen (/home/thanhsmind/.cache/cargo-target-rrt/release/bee dev regen); never ./.bee/bin/bee for regen. Restore .claude/skills changes you did not cause. Then run the verify. Locate with rg -l, then rg -n on one named file, and read ranges only. Write no code comment. Run only the cell verify, never the full suite. One commit with the cell trailer, files staged by explicit path. If a file outside the list must change, stop and say so in found. In your result put principles_used as an array of full slugs (bee-principle-<name> — <the decision it changed>), [] when none, and sync_ack with a reason if the cap sync door asks.", "feature": "reread-thrift", "lane": "standard", "read_first": ["docs/history/reread-thrift/CONTEXT.md"], "affects_specs": [], "affects_skills": ["skills/bee-swarming/SKILL.md", "skills/bee-planning/references/cells.md"]},
{"id": "rrt-7", "title": "Prove the four contracts and judge the code cells", "role": "test", "deps": ["rrt-1", "rrt-2", "rrt-3", "rrt-4", "rrt-5", "rrt-6"], "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- thin_worker_listings_off handoff_after_merge_door herding_result_job_digest cell_brief_search_rule_and_size", "must_haves": {"truths": ["tests named with thin_worker_listings_off show: the thin env list holds SLASH_COMMAND_TOOL_CHAR_BUDGET=1; a thin claude-session argv and a thin built-in claude pane argv hold --disable-slash-commands and --disallowedTools followed by Agent; a launch that is not thin holds none of the three", "tests named with handoff_after_merge_door show: a sessionless merge stores no merger record; feature start from a session that merged another feature refuses with handoff_due and writes nothing; with --same-session and a reason it succeeds and an audit row holds the reason; a first start and a sessionless start succeed", "tests named with herding_result_job_digest show: the --job form prints status, summary, proof, files_changed, found, unknowns, skipped and report_path; a long result is cut to at most 1500 characters and names the count cut; an unknown job is a typed refusal; the stdin form is unchanged", "tests named with cell_brief_search_rule_and_size show: the rendered cell prompt holds the search rule once; the fixed brief bytes are at most 75 percent of the baseline number rrt-4 recorded; the brief names every MailboxResult field and status", "each contract has a test that failed at the parent of its code cell commit, quoted in the report", "the report lists each rrt-5 and rrt-6 must_have with the file:line that shows it, or names the gap, and gives wc -c of packages/bee/LEADER.md", "the report holds one fenced block per cell rrt-1, rrt-2, rrt-3, rrt-4 with info string json <cell-id> holding a judge-verdict/1 object: schema, verdict PASS or NEEDS_REVISION, checks each with id, status PASS or FAIL, evidence (file:line and what it shows) and failure_signature when FAIL, fixability automatic or authority, confidence low, medium or high"], "why": "A code cell never certifies itself; the tester also carries the judge.", "out_of_scope": ["production code"]}, "decisions": ["1dd8b534-956a-48e5-9b11-9ba3a90a9d45", "cd3be8ac-5e4d-4629-9bca-995d94ed7775", "7e81e0f2-59bb-462b-a9d4-9ed7f8df07ce", "5737bdba-2382-4086-88ca-68db5e094f2d", "b56a554a-4d23-45fd-a6cd-636360d06e55", "5c368ef5-0a5d-463b-9188-19f19daa4752", "6d55d3dd-1d8d-46d9-92a9-3645f14c11c2", "b4521fc9-3c8d-4646-946d-dc7bfd66e7f6", "acb2c2d8-1e44-4837-a250-106dee765cce"], "files": ["packages/bee-rs/crates/bee/src/herding/run.rs", "packages/bee-rs/crates/bee/src/herding/claude_session.rs", "packages/bee-rs/crates/bee/src/herding/wave.rs", "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs", "packages/bee-rs/crates/bee/src/verbs/feature_group.rs", "packages/bee-rs/crates/bee/src/herding.rs", "packages/bee-rs/crates/bee/src/herding/mailbox.rs", "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs", "docs/history/reread-thrift/reports/rrt-7.md"], "action": "Check contracts thin-worker-listings-off (1dd8b534), handoff-after-merge-door (cd3be8ac), herding-result-job-digest (7e81e0f2) and cell-brief-search-rule-and-size (5737bdba) on the rrt-1 to rrt-4 commits. Add tests only inside existing test modules of the listed files. Every new test name contains one of the four contract names with underscores. Red proof per contract: put the production file at its code cell's parent with git show <parent>:<path>, run the verify, record the failure or compile error, restore to HEAD. Then run the verify green. Edit no production code. Check rrt-5 and rrt-6 parity and judge rrt-1 to rrt-4 from their diffs. Write everything to docs/history/reread-thrift/reports/rrt-7.md. Locate with rg -l, then rg -n on one named file, and read ranges only. Write no code comment. Run only the cell verify, never the full suite. One commit with the cell trailer, files staged by explicit path. If a file outside the list must change, stop and say so in found. In your result put principles_used as an array of full slugs (bee-principle-<name> — <the decision it changed>), [] when none, and sync_ack with a reason if the cap sync door asks.", "feature": "reread-thrift", "lane": "standard", "read_first": ["docs/history/reread-thrift/CONTEXT.md"], "affects_specs": [], "affects_skills": []},
{"id": "rrt-8", "title": "Consolidate the feature's knowledge and write the delivery note", "role": "writer", "deps": ["rrt-1", "rrt-2", "rrt-3", "rrt-4", "rrt-5", "rrt-6", "rrt-7"], "verify": "./.bee/bin/bee knowledge check", "must_haves": {"truths": ["bee knowledge consolidate --feature reread-thrift ran and its deduped output is merged into docs/knowledge with no near-duplicate concept", "docs/history/reread-thrift/delivery.md states what shipped per D-number, the measured numbers before and after (the D3 launch numbers, LEADER.md bytes, the brief fixed bytes) and what was left out of scope"], "why": "Every code-touching standard feature ends with one consolidation pass.", "out_of_scope": ["code", "skills"]}, "decisions": ["b56a554a-4d23-45fd-a6cd-636360d06e55", "5c368ef5-0a5d-463b-9188-19f19daa4752", "6d55d3dd-1d8d-46d9-92a9-3645f14c11c2", "b4521fc9-3c8d-4646-946d-dc7bfd66e7f6", "4c6e84ea-d386-4ac0-b989-590c55928468", "acb2c2d8-1e44-4837-a250-106dee765cce"], "files": ["docs/history/reread-thrift/delivery.md", "docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md"], "action": "Run ./.bee/bin/bee knowledge consolidate --feature reread-thrift and follow its output. Write docs/history/reread-thrift/delivery.md from CONTEXT.md, the cell results and docs/history/reread-thrift/reports/rrt-7.md, in short plain sentences; avoid the words deferred, until and later. Then run the verify. Locate with rg -l, then rg -n on one named file, and read ranges only. Write no code comment. Run only the cell verify, never the full suite. One commit with the cell trailer, files staged by explicit path. If a file outside the list must change, stop and say so in found. In your result put principles_used as an array of full slugs (bee-principle-<name> — <the decision it changed>), [] when none, and sync_ack with a reason if the cap sync door asks.", "feature": "reread-thrift", "lane": "standard", "read_first": ["docs/history/reread-thrift/CONTEXT.md"], "affects_specs": [], "affects_skills": []}
]
```
