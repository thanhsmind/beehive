# finding-recheck-trigger — plan

Route: class `feature` · lane `standard` · flags `public-contracts`,
`multi-domain` · product files 3.
Class playbook: `bee-planning/references/planning-reference.md`
("Class playbooks" → feature).

## Summary

A review finding that stays open must tell the leader when the code it is
about changes. Today it cannot: the leader recommended a finding (make
`bee cells cap` run the commit check) after a merged fix had already
closed it, because the finding lived only in chat and nothing watched its
files.

The shape reuses the trigger registry instead of building a new store.
`bee triggers` already records a watched condition and flips it to `due`
on read. This feature adds one predicate kind, `path-changed:<paths>`,
that goes `due` when a commit after the trigger's own anchor commit
touches one of those paths (D1, decision `95381603`). The anchor is the
HEAD of the tree where the finding is added (the reviewed worktree), so a
finding does not go due when its own feature merges. The per-prompt
reminder then prints `triggers due: N` so the signal reaches the leader
mid-session, not only at `bee orient`; it runs git only when main's HEAD
has moved (D2, `164e6903`). The reviewing doctrine tells the reviewer to
register open findings this way, and the leader to read the due list
before recommending (D3, `4c024330`).

## Discovery

One reality touch per novel surface, all taken before this plan:

- The trigger registry supports exactly two predicate kinds, both
  file-existence checks; nothing compares git history (claim 1).
- The evaluation loop flips `waiting` → `due` and persists it on read;
  its predicate call takes the spec string only, so a history predicate
  needs the record's anchor passed in (claim 2).
- `bee orient` already counts due triggers as a blocker (claim 3), so
  session start is covered with no change.
- The per-prompt reminder is built from the state record alone and is
  capped at three lines (claim 4); the planning function already holds
  the control root (claim 5), so the trigger store is reachable there.
- `triggers add` stores the decision id as its first eight characters
  and validates nothing else about it (claim 6).
- The reviewing skill sends P2/P3 findings to the backlog and nowhere
  else (claim 7) — the doctrine edit lands there.
- The registry payload describes `--predicate` with the two old kinds
  (claim 8); it has no regen chain, so it is hand-edited with the code.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | Triggers accept only path-exists and path-missing predicates | read | `packages/bee-rs/crates/bee/src/verbs/triggers/mod.rs:135` | `let after = spec.strip_prefix("path-exists:")` |
| 2 | The evaluation loop calls the predicate with the spec only and persists a due flip | read | `packages/bee-rs/crates/bee/src/verbs/triggers/mod.rs:257` | `if predicate_true(control, &spec) {` |
| 3 | bee orient already surfaces the due trigger count | read | `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:197` | `let (due, awaiting) = crate::verbs::triggers::due_and_manual_counts(control);` |
| 4 | The per-prompt reminder is capped at three lines | read | `packages/bee-rs/crates/bee/src/hooks/prompt_context.rs:939` | `lines.truncate(3);` |
| 5 | The prompt planning step already holds the control root | read | `packages/bee-rs/crates/bee/src/hooks/prompt_context.rs:257` | `let control_root = control_root_for_state(root)?;` |
| 6 | triggers add keeps only the first eight characters of --decision | read | `packages/bee-rs/crates/bee/src/verbs/triggers/mod.rs:422` | `let short8 = truncate_chars_head(decision, 8);` |
| 7 | bee-reviewing routes P2/P3 findings onward at its Finish step, with no watch on their files | read | `skills/bee-reviewing/SKILL.md:152` | `quote fresh output. P2/P3 go` |
| 8 | The registry payload names only the two old predicate kinds | read | `packages/bee-rs/crates/bee/src/generated/registry_payload.json:1` | `Optional machine-checkable predicate: path-exists:<path> or path-missing:<path>.` |

## Smaller path check

Cheaper shapes considered:

- **A new anchored-finding field on backlog rows** — rejected: backlog
  rows have no id and no update path, so clearing a re-check mark would
  need a new event type and a new reader. The trigger registry already
  has ids, `due`, `resolve`, and an orient surface.
- **Evaluate only at `bee worktree merge`** — rejected: a commit made on
  main directly (docs lane, release) would never flip the trigger.
  Evaluating on read matches the registry's own write-on-read law.
- **No prompt line, orient only** — rejected: the failure happened
  mid-session after a merge; orient runs at session start.

- **One `git log --name-only <oldest anchor>..HEAD` per read, matched in
  memory** — deferred: the HEAD-change cache (D2) already removes the
  per-prompt cost; revisit only if a read with many open findings is slow.

PASS — the three-file shape is the smallest that honours D1–D3.

## Hat wave

Seats run: `hat-facts-gaps`, `hat-alternatives`. Dropped:
`hat-user-impact` — the model guard refused the payload that dispatch
prepare returned for it (filed as a harness issue in the backlog).

What the wave changed:

- **Anchor root (BLOCKER, facts-gaps).** Anchoring on the control root
  HEAD made every finding go due at its own feature's merge. The anchor is
  now the HEAD of the tree where `triggers add` runs (D1 rev 2,
  `95381603`), with a merged-worktree test.
- **Claim 7 line (BLOCKER, facts-gaps).** The quote is at
  `SKILL.md:152`, and the Finish paragraph is `:152-156`. Fixed.
- **Per-prompt git cost (WARNING, both seats).** The prompt counter now
  re-evaluates only when the control HEAD moved (D2 rev 2, `164e6903`);
  the counter lives in the triggers module, so frt-2 depends on frt-1.
- **No decision id at Finish (WARNING, both seats).** Registration now
  follows `bee reviews record` and uses the review session id (D3 rev 2,
  `4c024330`).
- **Still-open after re-check (WARNING, alternatives).** Resolve and add
  again with a fresh anchor — in the bee-reviewing rule only.
- **Absolute paths and commas (NOTE, both).** Absolute paths and empty
  entries refuse at add; the comma limit is named in the help.

Dismissed: one `git log` per read across all triggers (alternatives,
NOTE) — deferred in the smaller-path check above, not needed after the
HEAD cache.

## Slices

One slice. Three cells: frt-1 (the predicate and the cached counter),
then frt-2 (the prompt line, which calls that counter) and frt-3 (the
doctrine) in parallel on disjoint files.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| frt-1 | Add a path-changed trigger predicate anchored at HEAD | `verbs/triggers/mod.rs`, `generated/registry_payload.json` | — | `bee triggers add --predicate path-changed:src/a.rs` records the HEAD sha, and `bee triggers list` shows it due after a commit touches `src/a.rs` | `cargo test … -p bee triggers` + registry contract tests green, red-first |
| frt-2 | Print the due trigger count in the per-prompt reminder | `hooks/prompt_context.rs` | frt-1 | every prompt shows `triggers due: 1 — bee triggers list --due` while a predicate trigger is due, and nothing when none is | `cargo test … -p bee prompt_context` green, red-first |
| frt-3 | Tell reviewers to register open findings as path-changed triggers | `skills/bee-reviewing/SKILL.md`, `skills/bee-hive/references/routing-and-contracts.md` + regen mirrors | frt-1 | the review Finish step says to register each open finding with its anchor files, and to read the due list before recommending | `bee dev regen` green + pointer check |

```json
[
  {
    "id": "frt-1",
    "feature": "finding-recheck-trigger",
    "title": "Add a path-changed trigger predicate anchored at HEAD",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "95381603-d480-411d-9c1f-fa8ed1be8279"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/triggers/mod.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/triggers/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per decision 95381603 (D1 rev 2). In verbs/triggers/mod.rs: (1) accept a third predicate kind `path-changed:<path>[,<path>...]` in is_valid_predicate: every comma-separated entry, trimmed, must be non-empty and repo-relative (an absolute path refuses); the add refusal text names all three kinds and says a path containing a comma cannot be watched. (2) Add an optional `anchored_at` field to TriggerRecord (from_value reads it when present, to_value writes it; records without it keep working). (3) In run_add, for a path-changed predicate, resolve HEAD of the tree where the command runs — ctx.root, NOT ctx.control — with crate::verbs::worktree::run_git(&ctx.root, [\"rev-parse\", \"HEAD\"]); a non-zero exit or empty output refuses with `bee triggers add: --predicate path-changed needs a git HEAD at <root> to anchor on.` and writes nothing; otherwise store the sha as anchored_at. Anchoring on the reviewed tree's HEAD is what keeps a finding from going due when its own feature branch merges into main. (4) Change predicate_true to also take the record's anchored_at. For path-changed: run `git log -1 --format=%H <anchored_at>..HEAD -- <paths...>` at the CONTROL root; non-empty stdout means true; a non-zero exit (unknown sha, no repo) or a missing anchored_at ALSO means true — a watched condition never sinks silently. path-exists and path-missing behave exactly as today. (5) Add `pub(crate) fn due_count_for_prompt(control: &Path) -> usize`: read the control root HEAD (run_git rev-parse HEAD); if it equals the sha stored in <control>/.bee/triggers/.last-eval-head, count predicate-tier `due` records WITHOUT evaluating (read_without_evaluating, no git log spawned); otherwise evaluate (read_and_evaluate), write the new sha to that file, and count. If HEAD cannot be read, evaluate. run_add deletes .last-eval-head after writing a trigger so the next prompt evaluates it. The file name does not end in .json, so the store reader never lists it. (6) Update the module header comment. In generated/registry_payload.json, update ONLY the triggers add `predicate` description: name path-changed, its repo-relative comma-separated paths, and that it is anchored at the HEAD of the tree where add runs. Tests (red first, in the existing tests module; real git repo fixture like leader_check.rs's diff_repo, plus `git worktree add` for the merge case): add rejects `path-changed:` with no path, with an empty entry, and with an absolute path; add in a repo records anchored_at = that tree's HEAD; add outside a git repo refuses and writes no file; no commit since anchor stays waiting; a commit touching an unrelated file stays waiting; a commit touching a watched path flips to due and persists; a bogus anchored_at reads as due; MERGE CASE — a trigger added inside a linked worktree on a feature branch whose own commits touched the watched path stays waiting after that branch is merged into main with a merge commit; due_count_for_prompt spawns no evaluation when HEAD is unchanged (observable: a trigger whose predicate became true after the cached sha was written is still counted as waiting until HEAD moves) and evaluates after HEAD moves.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee triggers && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts --test registry_dispatch",
    "must_haves": {
      "truths": [
        "a path-changed trigger records the HEAD sha of the tree where triggers add runs as anchored_at",
        "a path-changed trigger flips to due once a commit after anchored_at touches one of its paths, and stays waiting otherwise",
        "a trigger added in a feature worktree stays waiting after that feature branch merges into main",
        "a git error or missing anchor on a path-changed trigger reads as due, never as waiting",
        "due_count_for_prompt evaluates only when the control root HEAD moved since the last evaluation",
        "path-exists and path-missing triggers behave exactly as before"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/triggers/mod.rs",
          "substantive": "path-changed validation, anchored_at field, git-backed evaluation, due_count_for_prompt with the HEAD cache, and the new tests"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
          "substantive": "predicate description names path-changed"
        }
      ],
      "key_links": [
        "read_entries passes the record's anchored_at into predicate_true",
        "run_add deletes .last-eval-head after a write"
      ],
      "prohibitions": [
        "No change to trigger ids, file names, the resolve verb, or due_and_manual_counts' signature"
      ]
    }
  },
  {
    "id": "frt-2",
    "feature": "finding-recheck-trigger",
    "title": "Print the due trigger count in the per-prompt reminder",
    "lane": "standard",
    "role": "code",
    "deps": [
      "frt-1"
    ],
    "decisions": [
      "164e6903-1bda-4bed-8d76-15b4fd109a78"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs",
      "packages/bee-rs/crates/bee/src/verbs/triggers/mod.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per decision 164e6903 (D2 rev 2). In hooks/prompt_context.rs: where the plan calls build_prompt_reminder(&record) (control_root is already in hand at the top of that function, :257), read the due count with crate::verbs::triggers::due_count_for_prompt(&control_root) (added by frt-1; it re-evaluates only when the control HEAD moved). Pass it into build_prompt_reminder. When it is > 0, append the line `triggers due: <N> — bee triggers list --due` after the existing lines, raise the line cap from 3 to 4 so it is never cut, and add the count to the hashed fields (key `triggers_due`) so the reminder re-injects when N changes; when it is 0, the text AND the hash inputs stay byte-identical to today (do not add the key at 0). Keep every existing prompt_context test green unchanged. Tests (red first): a fixture with one predicate trigger file already `due` in <control>/.bee/triggers/ shows the new line; a fixture with only a manual waiting trigger shows no line; no triggers dir shows no line and the same hash as before.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee prompt_context",
    "must_haves": {
      "truths": [
        "the per-prompt reminder shows 'triggers due: N — bee triggers list --due' while N > 0 predicate triggers are due",
        "with zero due triggers the reminder text and hash are byte-identical to before"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs",
          "substantive": "due count read, new line, hash field, three new tests"
        }
      ],
      "key_links": [
        "the prompt plan calls crate::verbs::triggers::due_count_for_prompt with its control_root"
      ],
      "prohibitions": [
        "No manual-tier count in the prompt line",
        "No change to the existing reminder lines"
      ]
    }
  },
  {
    "id": "frt-3",
    "feature": "finding-recheck-trigger",
    "title": "Tell reviewers to register open findings as path-changed triggers",
    "lane": "standard",
    "role": "docs",
    "deps": [
      "frt-1"
    ],
    "decisions": [
      "4c024330-023c-4643-aed2-a03801e0a6cb"
    ],
    "files": [
      "skills/bee-reviewing/SKILL.md",
      "skills/bee-hive/references/routing-and-contracts.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "skills/bee-reviewing/SKILL.md",
      "skills/bee-hive/references/routing-and-contracts.md",
      "packages/bee-rs/crates/bee/src/verbs/triggers/mod.rs"
    ],
    "affects_skills": [
      "skills/bee-reviewing/SKILL.md",
      "skills/bee-hive/references/routing-and-contracts.md"
    ],
    "affects_specs": [],
    "action": "Per decision 4c024330 (D3 rev 2). (1) skills/bee-reviewing/SKILL.md, section Finish (the paragraph at :152-156 that sends P2/P3 to the backlog and closes with `bee reviews record --kind decision`): add, AFTER that record step, that every finding left open is ALSO registered as a watch on the files it is about — `bee triggers add --decision <the review session id> --condition \"re-check: <finding title>\" --predicate path-changed:<repo-relative anchor files, comma-separated>` — run from the reviewed tree, so the finding goes due when a later commit touches those files. Add the handling of a due finding in the same place: re-check it against the current code; fixed → `bee triggers resolve`; still open → resolve it and add it again, which takes a fresh anchor. (2) skills/bee-hive/references/routing-and-contracts.md: next to the leader completeness check section, add ONE pointer line: before recommending the next open finding, read `bee triggers list --due` and follow bee-reviewing's Finish rule for each due one. One fact, one home: the rule lives in bee-reviewing; routing-and-contracts only points; AGENTS.md is NOT edited. Write through the bee-technical-writing standard. Then run the regen chain `bee dev regen` and commit the rendered mirrors and docs/history/codex-harness-hardening/release-manifest.json with the edit.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pointer_integrity --test rule_index_parity",
    "must_haves": {
      "truths": [
        "bee-reviewing's Finish step tells the reviewer to register each open finding with a path-changed trigger on its anchor files, after bee reviews record, using the review session id",
        "bee-reviewing says a due finding that is still open is resolved and added again with a fresh anchor",
        "routing-and-contracts points the leader at bee triggers list --due before recommending an open finding"
      ],
      "artifacts": [
        {
          "path": "skills/bee-reviewing/SKILL.md",
          "substantive": "the registration sentence in Finish"
        },
        {
          "path": "skills/bee-hive/references/routing-and-contracts.md",
          "substantive": "one pointer line"
        }
      ],
      "key_links": [
        "the rendered skill mirrors carry both edits after bee dev regen"
      ],
      "prohibitions": [
        "No AGENTS.md edit",
        "No second copy of the rule"
      ]
    }
  }
]
```

## Test matrix

| Case | Cell | Expected |
|---|---|---|
| `path-changed:` with no path | frt-1 | add refuses |
| add in a git repo | frt-1 | record carries anchored_at = HEAD |
| add outside git | frt-1 | refuses, no file written |
| no commit since anchor | frt-1 | stays waiting |
| commit touches another file | frt-1 | stays waiting |
| commit touches a watched path | frt-1 | due, persisted |
| bogus anchored_at | frt-1 | due |
| absolute path or empty entry | frt-1 | add refuses |
| trigger added in a worktree, branch merged | frt-1 | stays waiting |
| prompt counter, HEAD unchanged | frt-1 | no evaluation |
| prompt counter, HEAD moved | frt-1 | evaluates, cache updated |
| one due predicate trigger | frt-2 | prompt line shown |
| only a manual waiting trigger | frt-2 | no line |
| no triggers dir | frt-2 | text and hash unchanged |

## Test scoping

Each cell runs its own filtered command. The full declared suite runs in
CI on push.

## Open questions

None.
