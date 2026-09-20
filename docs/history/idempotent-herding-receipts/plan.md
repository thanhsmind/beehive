---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: Idempotent Herding Receipts — Slice 2

## Summary

Run the same dispatch twice and bee starts a second worker. It has no memory
that the first one already answered. This slice gives it that memory: if the
same job name and round arrives carrying the same work, bee hands back the
answer it already has and starts nothing. If the same name arrives carrying
*different* work, bee stops and says so instead of quietly doing both.

The check happens early — in the parent process, before bee launches anything
in the background. That is what makes the refusal visible on the terminal you
are actually watching.

Slice 1 shipped at `aa918c8`. The cleanup command that was once slice 3 is
**dropped** (D8): 14 MB of gitignored disk did not justify a delete with no undo.

Mode: `high-risk` — 4 flags: data-model, public-contracts,
covered-contract-change, multi-domain. Earned this time: the pre-flight sits
above **every** dispatch on **every** runtime, so a wrong answer in either
direction is felt everywhere.

Why this is the least workflow that protects the work: one new predicate, one
new stored value, one refusal. The five decisions below were settled by a
second shaping pass precisely so this plan has nothing left to guess.

## Requirements (from CONTEXT.md)

- **D1** (`2eb61b7b`) — Receipts cover every `bee herding run`. Same `job_id` +
  round + same digest returns the stored result and spawns nothing. Delivery
  stays at-least-once; no exactly-once claim anywhere.
- **D3** (`42a59eea`) — Same key, different digest → error loudly, name the
  stored receipt path and both digests, spawn nothing.
- **D4 receipt-home half** (restated live as `11b68a1b`) — the receipt lives in
  the existing `.bee/mailbox/<job-id>/`. No new store.
- **D5** (`5d0692b5`) — A replay returns the ordinary run JSON envelope carrying
  the stored result, exits 0, and **still writes the result-inbox marker**.
- **D6** (`7b3a93d4`, restated `11b68a1b`) — The digest covers the **task and
  files**, never the rendered `brief-N.txt`.
- **D7** (`10e20c92`) — The pre-flight runs in the **parent**, above the
  `spawn_detached_runner` branch, not inside `fn execute`.
- **D8** (`11b68a1b`) — No prune verb. Nothing is deleted.

Plus the CONTEXT implementation rule: the pre-flight resolves the round as the
callee does — 1 for a fresh run, `latest_result_round + 1` for `--continue`.

## Load-bearing claims

Labels: `read` = the file was opened at the named line; `ran` = the named
command was executed and its output kept; `guessed` = neither, and no such row
may survive the gate.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | `run()` parses options first, with nothing done yet — the placement D7 names | read | `packages/bee-rs/crates/bee/src/herding/run.rs:3918-3924` | `pub(super) fn run(flags: &[&str]) -> ExitCode {` · `let opts = match parse_options(flags) {` |
| 2 | The detach branch returns a SUCCESS envelope and exits before any child work — so a refusal below it is invisible | read | `packages/bee-rs/crates/bee/src/herding/run.rs:3944-3952` | `return match spawn_detached_runner(flags, &opts) {` · `println!("{}", detached_envelope(&opts.job_id, session));` · `ExitCode::SUCCESS` |
| 3 | `run()` branches on `no_pane` itself, so `execute()` is NOT above every dispatch | read | `packages/bee-rs/crates/bee/src/herding/run.rs:3959-3963` | `let result = if opts.no_pane {` · `execute_no_pane(&opts)` · `} else {` · `execute(&opts, transport.as_ref().unwrap().as_ref())` |
| 4 | Both transports write the inbox marker inside their own callee, so D5's "still write the marker" needs its own call on the replay path | read | `packages/bee-rs/crates/bee/src/herding/run.rs:2892-2896`, `:2296-2302` | `pub(super) fn execute_no_pane(opts: &Options) -> ExecResult {` · `write_inbox_marker(&bee_dir, opts);` |
| 5 | A caller can supply an explicit `job_id`, and bee's own detached launcher does — so D3's conflict case is reachable in bee's own code, not only by a user | read | `packages/bee-rs/crates/bee/src/herding/run.rs:325`, `:3878` | `"--job-id" => {` · `args.extend(["--job-id".to_string(), job_id.to_string()]);` |
| 6 | `--continue` reuses the job_id and computes round = prior + 1 | read | `packages/bee-rs/crates/bee/src/herding/run.rs:3323-3327` | `let prior_round = match mailbox::latest_result_round(&entries) {` · `let next_round = prior_round + 1;` |
| 7 | A fresh run is round 1 by construction, which is why the round rule is forced rather than chosen | read | `packages/bee-rs/crates/bee/src/herding/run.rs:2635` | `let brief_file = mailbox::brief_path(&bee_dir, &opts.job_id, 1);` |
| 8 | `render_brief` embeds the absolute worktree root and the configured proof command — the evidence behind D6 | read | `packages/bee-rs/crates/bee/src/herding/mailbox.rs:277`, `:294`, `:312` | `files_block.push_str(&format!("  - {}\n", spec.worktree_root.join(f).display()));` · `spec.worktree_root.join(p)` · `let proof_command = read_proof_command(spec.worktree_root);` |
| 9 | The mailbox path helpers are round-keyed, so a digest path is one more sibling — no new store (D4 receipt-home half) | read | `packages/bee-rs/crates/bee/src/herding/mailbox.rs:76`, `:92` | `pub(crate) fn brief_path(bee_dir: &Path, job_id: &str, round: u32) -> PathBuf {` · `pub(crate) fn result_path(bee_dir: &Path, job_id: &str, round: u32) -> PathBuf {` |
| 10 | Two filename parser families already claim `brief-N.txt` and `result-N.json`, so the digest filename must sit outside both | read | `packages/bee-rs/crates/bee/src/herding/run.rs:3127-3133`; `.pi/extensions/bee-guard.ts:729` | `parse_brief_filename` strips `brief-`/`.txt` · `/^result-(\d+)\.json$/` |
| 11 | Slice 1 shipped and is green in main — this slice builds on a green base | ran | `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` at `aa918c8` | 36 suites, 0 failed; `pi_plugin_contracts` 79 passed |
| 12 | 344 legacy mailboxes carry no digest, which is why a missing digest must NOT refuse | ran | `ls .bee/mailbox \| wc -l` in `/home/thanhsmind/Projects/goglbe/beehive` | `344` |

## Discovery

The placement in claim 1 is the whole slice. An earlier draft put this check at
the top of `fn execute`; the hat wave showed `execute()` is reached only by
direct callers, because `run()` branches on `no_pane` (claim 3) and the
detached path returns earlier still (claim 2). A check there would have passed
its own unit tests while still spawning both a second native worker and a
second detached runner.

Claim 4 is the consequence D5 exists for: the marker write lives inside each
callee, so a replay that returns from `run()` skips it unless the replay path
writes it deliberately. Without that, a detached Pi session gets a success
envelope and waits forever.

## Approach

**Recommended path.** One cell, two moves in one file plus tests.

1. **The digest.** A new `digest_path(bee_dir, job_id, round)` joins the mailbox
   helper family (claim 9), named outside both parser families (claim 10). Its
   content is a hash of the dispatch's **task and files** (D6), written
   atomically beside `result-N.json` when a job is dispatched.
2. **The pre-flight.** A new predicate runs in `run()` immediately after
   `parse_options` (claim 1), before the transport choice and before the detach
   branch (D7). It resolves the round as the callee does, then:
   - no `result-N.json` → **proceed** (this is a fresh or crashed-early job);
   - result present, digest **absent** → **return the receipt** (the 344 legacy
     mailboxes, claim 12);
   - result present, digest **matches** → **return the receipt** (D1);
   - result present, digest **differs** → **refuse**, naming the stored receipt
     path and both digests (D3);
   - digest present but **unreadable** → **proceed**, with a note on stderr.

   Returning the receipt means printing the ordinary run JSON envelope carrying
   the stored result, exiting 0, **and writing the result-inbox marker** (D5,
   claim 4).

**Rejected alternatives.**

- The check inside `fn execute` — claims 2 and 3 falsify it.
- Digesting the rendered brief — claim 8; D6 settled it.
- A separate receipt store — the mailbox dir already is the receipt (claim 9).
- Gating the pre-flight on "explicit `--job-id` only" to skip a `stat` on the
  common path — saves one filesystem call before splitting a tmux pane, and
  weakens D1's "every dispatch" from literal to by-construction. Not worth it.
- A prune verb — dropped by D8.

**SMALLER PATH check.** Is there a cheaper shape honoring D1, D3, D5, D6, D7?
The digest and the pre-flight are one behavior: a pre-flight with nothing stored
to compare cannot tell a replay from a conflict. Neither half stands alone.
PASS, no redraft.

**Risk map.**

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Pre-flight placement in `run()` | HIGH — above every dispatch on every runtime | ihr-2 | Both transports driven, plus the detached path; a false mismatch would refuse everything |
| False mismatch on `--continue` | HIGH — a round resolved wrong refuses every continue | ihr-2 | A `--continue` at round 2 with the same task proceeds, not refuses |
| Legacy mailboxes with no digest | MEDIUM — 344 of them (claim 12) | ihr-2 | A mailbox with a result and no digest returns the receipt, never refuses |
| Marker on the replay path | MEDIUM — silent loss on detached Pi if skipped | ihr-2 | A replayed `--inbox-session` run leaves a marker the drain finds |
| Digest filename collision | LOW — claim 10 names both families to avoid | ihr-2 | `latest_result_round` and the drain's readdir both ignore the digest file |

Waves: one cell, no parallelism.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "plan-hat-wave", "classification": "not-applicable", "role": "advisor", "reason": "The hat wave is once per feature and already ran at the first plan step; its synthesis is what produced D5-D8 and this slice. Re-stamped against the new plan bytes, not re-run."},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "See plan-hat-wave."},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "See plan-hat-wave; its findings are D5, D7 and the fail-open rules in CONTEXT."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "See plan-hat-wave; its materiality FAIL on prune became D8."},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "See plan-hat-wave; its mechanism findings became D6 and D7."},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "See plan-hat-wave."},
    {"stage": "research", "classification": "not-applicable", "role": "read", "reason": "Every load-bearing claim was read directly in this repo."},
    {"stage": "extraction", "classification": "conditional", "role": "extraction", "condition": "A narrow fact is needed from a known file during execution.", "reason": "Cheap narrow lookups."},
    {"stage": "gather", "classification": "conditional", "role": "read", "condition": "A multi-file hunt is needed that the leader does not hold.", "reason": "Mechanical multi-file reads delegate down-tier."},
    {"stage": "cell-execution", "classification": "required", "role": "code", "reason": "The cell writes Rust and its tests."},
    {"stage": "test-authoring", "classification": "required", "role": "test", "reason": "Red-first: the refusal and the replay-return are both driven before they exist."},
    {"stage": "docs", "classification": "conditional", "role": "docs", "condition": "The refusal changes a documented CLI contract.", "reason": "config-reference and the herding knowledge area."},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "The user invokes an independent review.", "reason": "Independent review is user-invoked only."},
    {"stage": "supervisor", "classification": "not-applicable", "role": "supervisor", "reason": "Attended session."},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "No release in this feature."},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "Every writing stage resolves to a named role above."},
    {"stage": "plan", "classification": "not-applicable", "role": "plan", "reason": "This is the last slice; no further slice is drafted after it."},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "No open design shape — D5-D7 fix the mechanism."},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "See blind-lane-1."},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "See blind-lane-1."}
  ]
}
```

## Shape

**Feature outcome.** A dispatch that already ran is safe to meet twice.

**Repo-reality basis.** The mailbox already keys every artifact by `job_id` +
round (claim 9), and `run()` already is the one place both transports and the
detached path pass through (claims 1–3).

| Epic | Capability / risk area | Why it exists | Slices | Proof needed |
|---|---|---|---|---|
| A result is never mistaken for a replay | Delivery correctness | Slice 1 | **Shipped** `aa918c8` | Done: 36 suites green |
| A dispatch that already ran does not run twice | Duplicate work and conflicting reuse | D1, D3, D5, D6, D7 | Slice 2 (this plan) | Both transports and the detached path driven |
| The mailbox does not grow forever | Retention | **Dropped** by D8 | — | — |

**Slice queue.** One slice: the receipt and its refusal. It is the last.

**Current slice to prepare: slice 2.**

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| ihr-2 | Return the stored receipt on a replay and refuse a reused job id carrying different work | `packages/bee-rs/crates/bee/src/herding/run.rs`, `packages/bee-rs/crates/bee/src/herding/mailbox.rs`, tests | — | Re-running a finished dispatch prints the answer it already has and starts nothing; reusing that job id for different work stops with both digests named | `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — red first on the replay and refusal cases, then green |

```json
[
  {
    "id": "ihr-2",
    "feature": "idempotent-herding-receipts",
    "title": "Return the stored receipt on a replay and refuse a reused job id carrying different work",
    "lane": "high-risk",
    "role": "code",
    "deps": [],
    "decisions": ["2eb61b7b", "42a59eea", "5d0692b5", "7b3a93d4", "10e20c92", "11b68a1b", "035a2ee5-28e7-48b3-a119-d96f7e0d81a1"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding/mailbox.rs"
    ],
    "read_first": [
      "docs/history/idempotent-herding-receipts/plan.md",
      "docs/history/idempotent-herding-receipts/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding/mailbox.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "RED FIRST, inside this cell: write the failing tests before the code exists and watch each fail for its reported reason. Cover, at minimum, the replay-returns-receipt case and the differing-digest-refuses case.\n\n(1) DIGEST HELPER, in mailbox.rs. Add `digest_path(bee_dir, job_id, round)` beside `brief_path` (:76) and `result_path` (:92), following that family exactly. The filename MUST fall outside both existing parser families — `parse_brief_filename` strips `brief-`/`.txt` (run.rs:3127-3133) and the Pi drain matches /^result-(\\d+)\\.json$/ (.pi/extensions/bee-guard.ts:729) — so use a name like `digest-N.sha256`. Add a `dispatch_digest(task, files)` function whose input is the dispatch's TASK TEXT and FILE LIST and NOTHING ELSE (D6): never the rendered brief, because render_brief embeds the absolute worktree root (mailbox.rs:277, :294) and the configured proof command (:312), so the same work in a moved worktree would hash differently. Write the digest atomically (crate::fsutil::write_json_atomic or the same temp+rename discipline) wherever the brief for that round is written — run.rs:2635 for a fresh run, :3374 for a --continue round.\n\nWhile you are in mailbox.rs, fix the wrong doc comment at :75: it describes `brief_path` as \"`.bee/mailbox/<job-id>/result-N.json` for a given round\". It returns brief-N.txt.\n\n(2) PRE-FLIGHT, in run.rs. Add it inside `fn run` (:3918) IMMEDIATELY after `parse_options` returns (:3924) — before the transport choice and before the `spawn_detached_runner` branch (:3944). D7 fixes this placement and claims 2 and 3 are why: `run()` branches on `no_pane` itself (:3959) and the detached path returns a SUCCESS envelope and exits (:3948-3951), so a check inside `fn execute` is reached only by direct callers and would leave a detached refusal invisible. Do NOT put it in `execute` or `execute_no_pane`.\n\nResolve the round exactly as the callee does — 1 for a fresh run (brief-1 is written at :2635), `mailbox::latest_result_round(...) + 1` for `--continue` (:3323-3327). A pre-flight that assumes round 1 for a --continue would refuse EVERY continue on every runtime.\n\nThen apply these five outcomes, in this order, and FAIL OPEN on I/O:\n  - no `result-N.json` for the resolved round -> PROCEED (fresh job, or a worker that crashed after its brief and before its result; it must stay re-runnable under its own id).\n  - result present, digest file ABSENT -> RETURN THE RECEIPT. Do not refuse: 344 legacy mailboxes carry no digest (claim 12).\n  - result present, digest MATCHES -> RETURN THE RECEIPT (D1).\n  - result present, digest DIFFERS -> REFUSE. The message names the stored receipt path and BOTH digests, and says the caller must pass a fresh --job-id. Follow the house refusal shape: `bee herding run: <what> FIX: <remedy>`.\n  - digest present but UNREADABLE -> PROCEED, with one note on stderr. An ENOSPC or permission fault must never refuse every dispatch.\n\nRETURNING THE RECEIPT means all three of: print the ordinary run JSON envelope carrying the STORED result (the same shape a real run prints), exit 0, AND call `write_inbox_marker` (D5). The marker is not optional — both transports write it inside their own callee (:2296-2302, :2892-2896), so a replay returning early from `run()` skips it, and a detached Pi session would then see a success envelope and wait forever for an injection that never comes.\n\nDo NOT add any prune verb or delete anything (D8). Do NOT weaken the at-least-once guarantee or add an exactly-once claim anywhere (D1). Do NOT touch .pi/extensions/bee-guard.ts — slice 1 is shipped and this cell must not re-open it.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml",
    "must_haves": {
      "truths": [
        "each new test fails first, for its own reported reason, before the code exists",
        "a second run with the same job_id, same round and same task+files prints the stored result, exits 0, and spawns nothing",
        "that replay ALSO writes the result-inbox marker, so a detached Pi session still receives an injection",
        "a second run with the same job_id and round but a different task refuses, names the stored receipt path and both digests, and spawns nothing",
        "a mailbox holding a result but NO digest file returns the receipt and never refuses",
        "an unreadable digest proceeds to spawn and notes it on stderr",
        "a job with a brief but no result for the round proceeds, so a worker that crashed early stays re-runnable",
        "a --continue at round 2 carrying the same task proceeds and is never refused as a round-1 conflict",
        "the digest input is the task and files only: the same task dispatched from a different worktree path produces the same digest",
        "the digest filename is ignored by latest_result_round and by the Pi drain's result regex",
        "the pre-flight lives in fn run after parse_options, not in execute or execute_no_pane",
        "the brief_path doc comment at mailbox.rs:75 no longer says result-N.json"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/mailbox.rs", "substantive": "digest_path joining the brief_path/result_path family, dispatch_digest over task+files, atomic write, and the corrected brief_path doc comment"},
        {"path": "packages/bee-rs/crates/bee/src/herding/run.rs", "substantive": "the pre-flight in fn run after parse_options with all five outcomes, the receipt return including write_inbox_marker, and the digest write beside each brief write"}
      ],
      "key_links": [
        "the pre-flight sits above the spawn_detached_runner branch, so a refusal prints on the caller's own terminal",
        "the replay path calls write_inbox_marker explicitly, because both callees that normally write it are skipped",
        "dispatch_digest takes task and files, never a rendered brief"
      ],
      "prohibitions": [
        "Do not place the check in fn execute or fn execute_no_pane",
        "Do not digest the rendered brief-N.txt",
        "Do not refuse when the digest file is absent or unreadable",
        "Do not add a prune verb or delete any mailbox directory",
        "Do not touch .pi/extensions/bee-guard.ts or the release manifest",
        "Do not add an exactly-once claim anywhere in code, tests or docs"
      ]
    },
    "behavior_change": true
  }
]
```

## Verify

`cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`
— the repo's declared `commands.test`. Scope reason: the pre-flight sits above
every dispatch, so the herding suites, the dispatch-door suites and the runtime
belt contract suites all exercise it, and they all live in that one manifest.
No release-manifest check this time: this cell touches no hashed root (slice 1
did, and is shipped).

## Test matrix

High-risk lane: the twelve dimensions, probes where the dimension applies.

| # | Dimension | Probe | Pass when |
|---|---|---|---|
| 1 | User types | A human re-running a command from shell history; bee's own detached launcher re-execing with `--job-id` (claim 5) | Both reach the same pre-flight and the same outcome |
| 2 | Input extremes | Empty task; a files list of 0 and of many; a job_id with path-unsafe characters | The digest is stable and the path stays inside the mailbox dir |
| 3 | Timing | A replay arriving while the first worker is mid-round (brief written, no result) | PROCEED — no result means not a replay |
| 4 | Scale | 344 legacy mailboxes with no digest | None refuse |
| 5 | State transitions | Fresh run → `--continue` round 2 → replay of round 2 | Round 2 replay returns round 2's receipt, not round 1's |
| 6 | Environment | A worktree moved to a new absolute path, same task | Same digest, no refusal (D6) |
| 7 | Error cascades | Digest file unreadable (permissions); mailbox dir unreadable | PROCEED with a stderr note; never a refusal |
| 8 | Authorization | n/a — no trust boundary crossed | — |
| 9 | Data integrity | The digest file must not be parsed as a brief or a result | `latest_result_round` and the drain's regex both skip it |
| 10 | Integration | A replayed `--inbox-session` dispatch | A marker is written and the drain injects the stored result |
| 11 | Compliance | n/a | — |
| 12 | Business logic | Same id, same round, different task | Refuses, naming both digests and the receipt path |

Behavior-change row: run a plain re-dispatch of a finished job on `main` and on
`head`. Pass when main spawns a second worker and head prints the stored result
and spawns none.

## Open Questions

(none)

The one question the first plan carried — whether a non-Pi caller reuses a
`job_id` — is answered by claim 5: bee's own detached launcher does, at
`run.rs:3878`. It meets the pre-flight like any other caller, and because a
detached re-exec of a fresh job has no stored result, it PROCEEDS.

## Out of scope

- `bee herding prune` and any mailbox collection — dropped by D8, filed as a
  backlog row.
- Exactly-once injection — would supersede `a05636f8`.
- Any change to `.pi/extensions/bee-guard.ts` — slice 1 shipped it.
- Rules 1, 2, 4 and 5 of `docs/history/research/pi-workflows-xia.md` — one
  backlog row, each needing its own shaping.
