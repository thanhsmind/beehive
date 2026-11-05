# Plan: worker-status-digest

## Summary

A leader learns a worker's progress from one status line. Workers write
short progress notes to `.bee/mailbox/<job>/progress.jsonl`; a new mode of
`bee herding status --job <id>` turns those notes into one line, through
the cheap supervisor role when there are more than three notes, and
verbatim otherwise. It never reads a log, a pane or a transcript. Two
disjoint cells, one wave: wsdg-1 builds the Rust (notify, status, the
herded brief); wsdg-2 writes the prose (native worker prompt, bee-swarming
tending, AGENTS block, bee-herding knowledge). Playbook:
`skills/bee-planning/playbooks/feature.md` — step 6 (statechart) does not
apply: no event-order flow changes. Lane `standard`: flags public
contracts (a CLI verb mode and the notify contract), multi-domain (Rust,
prompts, doctrine), external systems (the waggledance PBI).

SMALLER PATH check: PASS. The note writer is the existing
`bee herding notify` (no new verb), and the reader is a mode of the
existing `bee herding status` verb. The model call reuses the dispatch
door's own payload, so there is no second model-selection path.

Decisions: `d9007aca`, `d981106a`, `28b87b5b` (supersedes `657bbe71`),
`b4fd9daa`, `610b73c9` — see `CONTEXT.md`.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | `bee herding status` already exists and lists jobs | read | packages/bee-rs/crates/bee/src/herding.rs:139 | "status" => Some(status(rest)), |
| 2 | notify appends every kind to messages.jsonl today | read | packages/bee-rs/crates/bee/src/herding/notify.rs:68 | .open(messages_path(&job_dir)) |
| 3 | notify refuses without BEE_HERDING_JOB_ID today | read | packages/bee-rs/crates/bee/src/herding/notify.rs:31 | "BEE_HERDING_JOB_ID is not set; notify runs only inside a herded worker. |
| 4 | The herded brief already allows `bee herding notify` | read | packages/bee-rs/crates/bee/src/herding/mailbox.rs:391 | `bee herding notify --kind <question\|blocked\|dependency\|progress\|reopen> --text <text>`. |
| 5 | bee's own stall alert is a progress-kind message line | read | packages/bee-rs/crates/bee/src/herding.rs:791 | notify::append_message(main_root, Some(job_id), "progress", &text, None, ms_to_utc(now_ms)); |
| 6 | The supervisor role resolves to a Bash herding-run payload here | ran | .bee/bin/bee dispatch prepare --runtime claude --kind gather --role supervisor --json | "command": ".bee/bin/bee herding run --task-file - --json --agent \"paseo-pi-agy-flash\" --read-only --ceiling 1800" |
| 7 | The native worker prompt has a native-only contract block | read | packages/bee/prompts/worker-cell.md:29 | {{#if native_bookkeeping}} |
| 8 | bee-swarming's tend step is where the leader watches workers | read | skills/bee-swarming/SKILL.md:77 | 4. Tend: read each worker's Result form |
| 9 | The AGENTS block has a dispatch section to carry the rule | read | packages/bee/AGENTS.block.md:145 | ## Work in parallel, coordinate through the store |

## Discovery

- Chesterton's fence on `notify --kind progress` → `messages.jsonl`: the
  Pi drain delivers every new `messages.jsonl` line to the leader (decision
  `657bbe71`). That delivery is exactly the leader cost D1 removes, so
  progress moves out; the other four kinds stay because the leader must act
  on them. bee's stall alert (claim 5) stays in `messages.jsonl`: it is an
  alert to the leader, not a worker note.
- The `status` verb already has a fleet mode (no `--job`). The new mode
  is selected only by `--job`; the fleet output does not change.
- The supervisor run is a subprocess of the same binary with the dispatch
  door's payload (claim 6). Tests must not reach paseo: the summarizer is
  injected behind a seam the tests replace.

## Approach

`--job` selects the digest mode. It reads the latest `result-N.json` and
`progress.jsonl`, nothing else in the mailbox, and prints one line. More
than 3 notes and a cache miss call the supervisor through the dispatch
door's Bash payload with a task that holds only the notes; the answer is
cut to one line of at most 120 characters and cached.

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 (this slice) | notify progress home, status digest mode, briefs, doctrine | user order 2026-11-05 | `bee herding status --job X` prints `no progress notes yet`, then the last note after a notify | release after 2.75.0 |

## Cells — current slice (preview)

```json
[
  {
    "id": "wsdg-1",
    "feature": "worker-status-digest",
    "title": "Add the one-line worker status digest and move progress notes to progress.jsonl",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["d9007aca-6c61-4f07-a0e0-cf6bb15daa88", "d981106a-71ea-4d4f-8fd3-c424a0f00e25", "28b87b5b-426e-4fa9-aee1-6447b7754441", "b4fd9daa-7a1d-4f9b-aa88-b5ebf7ac4ae6"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding.rs",
      "packages/bee-rs/crates/bee/src/herding/notify.rs",
      "packages/bee-rs/crates/bee/src/herding/mailbox.rs",
      "packages/bee-rs/crates/bee/src/herding/status_digest.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json"
    ],
    "read_first": [
      "docs/history/worker-status-digest/CONTEXT.md",
      "docs/history/worker-status-digest/plan.md",
      "packages/bee-rs/crates/bee/src/herding/notify.rs",
      "packages/bee-rs/crates/bee/src/herding/mailbox.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md"],
    "regen_obligation_ack": "wave-barrier",
    "action": "Write the red test FIRST and watch it fail: `bee herding status --job J` on a mailbox with job.json and a log.txt holding a sentinel string but no progress.jsonl and no result file prints exactly `no progress notes yet` (JSON: {job, line, source: none, notes: 0}), exits 0, and the sentinel never appears in the output. Then build. (1) notify.rs: `--kind progress` appends one line {ts, step, note} to `.bee/mailbox/<job>/progress.jsonl` (add `progress_path` in mailbox.rs beside messages/result paths), never to messages.jsonl; `--step` optional; a note over 120 characters is refused with a FIX; `--job <id>` overrides BEE_HERDING_JOB_ID and, for progress only, needs no job.json (it creates the job dir); the job-id path checks stay. Every other kind keeps today's messages.jsonl behaviour byte for byte, and the internal stall alert (herding.rs, search `no_diff_progress for`) keeps writing to messages.jsonl. (2) New module herding/status_digest.rs, wired from the existing `status` verb when `--job` is present (no `--job` keeps today's fleet output unchanged). It reads ONLY progress.jsonl and the highest result-N.json (reuse mailbox's latest_result_round); a torn last line is skipped. Order: result file → `<status>: <summary>` (source result); no notes → `no progress notes yet` (source none); notes ≤ 3 → last note verbatim as `<step>: <note>` or `<note>` (source verbatim); notes > 3 → if status-digest.json holds the same note count, its line (source cached); else ask the supervisor and cache {notes, line, at} (source supervisor); any failure → last note verbatim (source verbatim, plus `fallback` reason in JSON). The supervisor call runs this same binary's `dispatch prepare --runtime <rt> --kind gather --role supervisor --json` (rt: the runtime the dispatch door detects, claude when unknown) and runs the returned payload only when tool is Bash, passing on stdin a task that holds the notes and asks for one status line; take the first non-empty line of the result summary, cut to 120 chars. A non-Bash payload means no supervisor. Put the call behind a trait or fn seam so tests inject a fake summarizer and never reach paseo. (3) mailbox.rs render_brief: add one short paragraph telling a herded worker to append one JSON line {\"ts\",\"step\",\"note\"} (note ≤120 chars) to the absolute progress.jsonl path at each meaningful step, either by hand or with notify --kind progress; keep the notify line. (4) generated/registry_payload.json: if it describes `herding status` or `herding notify`, add --job / --step there (hand-maintained). Tests (behaviour, not structure): the red test above; ≤3 notes → last note verbatim and the fake summarizer is never called; result-2.json present → result line wins; 4 notes + fake summarizer → source supervisor and status-digest.json written, a second call → source cached and the fake not called again; 4 notes + failing summarizer → verbatim fallback; notify --kind progress writes progress.jsonl and leaves messages.jsonl absent; a 121-char note refused; notify --kind progress --job X with no job.json succeeds; notify --kind blocked still writes messages.jsonl. Update existing tests that assert progress lands in messages.jsonl (notify.rs tests near `half done`, herding.rs near line 2226 and 3222) to the new contract, citing decision 28b87b5b. No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml herding",
    "must_haves": {
      "truths": [
        "bee herding status --job J with no progress.jsonl and no result prints `no progress notes yet` and exits 0, and never prints log.txt content",
        "With 3 or fewer notes the line is the last note verbatim and no model is called",
        "A result-N.json outranks notes",
        "With more than 3 notes the supervisor line is used and cached by note count; failure falls back to the last note verbatim",
        "bee herding notify --kind progress writes progress.jsonl only, refuses a note over 120 characters, and accepts --job with no job.json",
        "Other notify kinds and the stall alert still write messages.jsonl",
        "bee herding status with no --job is unchanged",
        "The herded brief tells the worker to append progress notes to the absolute progress.jsonl path"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/status_digest.rs", "substantive": "the digest mode with its supervisor seam and tests"}
      ],
      "key_links": [
        "herding.rs status routes --job to status_digest",
        "status_digest reaches the supervisor only through dispatch prepare's payload"
      ],
      "prohibitions": [
        "status_digest never opens log.txt, activity.json, a pane capture or a transcript",
        "No test reaches paseo or a real model",
        "No new top-level verb",
        "No bee dev regen inside the cell"
      ]
    }
  },
  {
    "id": "wsdg-2",
    "feature": "worker-status-digest",
    "title": "Teach workers to write progress notes and leaders to read only the status line",
    "lane": "standard",
    "role": "docs",
    "deps": [],
    "decisions": ["d9007aca-6c61-4f07-a0e0-cf6bb15daa88", "d981106a-71ea-4d4f-8fd3-c424a0f00e25", "b4fd9daa-7a1d-4f9b-aa88-b5ebf7ac4ae6"],
    "files": [
      "packages/bee/prompts/worker-cell.md",
      "skills/bee-swarming/SKILL.md",
      "skills/bee-swarming/references/swarming-reference.md",
      "packages/bee/AGENTS.block.md",
      "docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md",
      "docs/knowledge/areas/bee-herding/the-mailbox-broker.md"
    ],
    "read_first": [
      "docs/history/worker-status-digest/CONTEXT.md",
      "docs/history/worker-status-digest/plan.md",
      "skills/bee-swarming/SKILL.md",
      "packages/bee/AGENTS.block.md"
    ],
    "affects_skills": ["skills/bee-swarming/SKILL.md", "skills/bee-swarming/references/swarming-reference.md"],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md", "docs/knowledge/areas/bee-herding/the-mailbox-broker.md"],
    "regen_obligation_ack": "wave-barrier",
    "action": "(1) packages/bee/prompts/worker-cell.md, inside the native_bookkeeping contract block: an obligation line — at each meaningful step (read done, red test seen, green, committed) record one progress note with `.bee/bin/bee herding notify --kind progress --job {{cell_id}} --step <word> --text \"<note, at most 120 characters>\"`; a note never ends the turn. (2) skills/bee-swarming/SKILL.md step 4 (Tend): the leader learns a running worker's progress only from `bee herding status --job <id>` — one line — and never reads a worker's transcript, pane log, log.txt or activity stream for progress; cite decision d9007aca. Mirror one sentence in swarming-reference.md where it describes tending or stuck workers (search `Operating Contract in full`), pointing to SKILL.md as the home. (3) packages/bee/AGENTS.block.md § Work in parallel: one bullet — a leader watches a worker through `bee herding status --job <id>` (one line), never by reading its transcript, pane log or activity stream; workers write the notes. Keep it to two lines; it is the doctrine home, the skill text carries the how. (4) docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md: a section `Status digest` with the D4 order (result, none, verbatim ≤3, supervisor >3 with cache, fallback), the sources list, what it never reads, and Pointers to herding/status_digest.rs and herding.rs; add the decisions to the concept's decisions list. the-mailbox-broker.md: progress notes live in progress.jsonl, are never delivered to the leader, and messages.jsonl keeps question/blocked/dependency/reopen and the stall alert (decision 28b87b5b). Do not edit AGENTS.md at the repo root or any rendered skill copy under .claude/, .agents/ or .opencode/ — the leader runs bee dev regen at wave close. Write through the bee-technical-writing skill.",
    "verify": "rg -n 'herding status --job' packages/bee/prompts/worker-cell.md skills/bee-swarming/SKILL.md packages/bee/AGENTS.block.md docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md — expected: a hit in SKILL.md, AGENTS.block.md and the knowledge concept; rg -n 'notify --kind progress' packages/bee/prompts/worker-cell.md — expected: one hit",
    "must_haves": {
      "truths": [
        "worker-cell.md obliges a native worker to record progress notes with notify --kind progress --job <cell id>",
        "bee-swarming Tend names bee herding status --job as the only progress read and forbids transcript, pane log and activity reads",
        "AGENTS.block.md carries the rule in the dispatch section",
        "The bee-herding knowledge describes the digest order and the progress.jsonl home"
      ],
      "artifacts": [
        {"path": "docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md", "substantive": "a Status digest section with Pointers"}
      ],
      "key_links": ["AGENTS.block.md bullet and SKILL.md Tend step say the same rule; SKILL.md is the how"],
      "prohibitions": ["No edit to root AGENTS.md or rendered skill copies", "No Rust edits"]
    }
  }
]
```

## Test matrix

| Case | Proof |
|---|---|
| no notes, no result → `no progress notes yet`, log never read | status_digest tests, red first (wsdg-1) |
| ≤3 notes → verbatim, no model | status_digest tests (wsdg-1) |
| result outranks notes | status_digest tests (wsdg-1) |
| >3 notes → supervisor, cached; failure → verbatim | status_digest tests with a fake summarizer (wsdg-1) |
| notify progress home, 120-char cap, --job without job.json | notify tests (wsdg-1) |
| other kinds and stall alert unchanged | existing notify and herding tests (wsdg-1) |
| prose carries the rule | rg verify line (wsdg-2) |
| user-visible path | after build: notify a note into a scratch job, then `bee herding status --job <id>` (`green:live` at close) |

## Measure

At close the leader records, in the delivery record, the leader-side
bytes and token estimate for one sample wave: before = the logs and
activity reads a leader made to learn progress; after = the status lines.

## Open Questions

(none)

## Out of scope

- `waggledance_await` summary-only mode: filed as a PBI in the waggledance
  project through its leader; no edit to that repo from beehive.
- Changing the fleet output of `bee herding status` without `--job`.
