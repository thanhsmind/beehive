# Hand verification

The feature documents were written from the code and the tests. This directory is the protocol for checking them against the built binary, one observable claim at a time.

## What is here

| File | Covers |
| --- | --- |
| [foundations.md](foundations.md) | `foundations/*` |
| [lifecycle.md](lifecycle.md) | `lifecycle/*` |
| [areas.md](areas.md) | `delegation/*`, `memory/*`, `discovery/*`, `coordination/*`, `observability/*`, `maintenance/*`, `reviews/*` |
| [cross-cutting.md](cross-cutting.md) | `cross-cutting/*` |

Each file has one table per document. Each row is an item with a stable ID (`CAPT-03`, `GATE-07`), a priority, what it needs, the claim with a link to the document section, the setup, numbered steps, the expected result, and a Result column for the tester. Items that cannot be checked by hand are listed under each document as "Not checkable by hand".

Priorities: **P1** is an established fact, a claim many documents depend on, or a suspected bug; **P2** is an ordinary claim; **P3** is a number or a wording detail.

## How to run a pass

1. Bring up the surface: build or use the vendored binary (`.bee/bin/bee` in beehive at the pinned commit). For store-writing items, create a scratch host repo — `mkdir -p host/.bee && echo '{}' > host/.bee/onboarding.json` — and run from inside it; delete the directory afterward. For hook items, a hooked session in a disposable clone of a host repo is required; the Needs column says which.
2. Confirm the commit. Every document says `Verified against beehive commit 6b0ae488`. If `git rev-parse --short HEAD` differs by more than this description set's own `docs:` commits, some failures will be drift, not defects.
3. Keep the documents open beside the terminal. Read the linked section before each item; the item is a summary, the section is the claim.
4. Work through P1 first across all files, then P2, then P3.
5. Record `pass`, `fail`, or `blocked` in the Result column, with a note for anything other than a clean pass.
6. File every fail in [`bug-triage.md`](../bug-triage.md): if the entry exists, add a Status line quoting the item ID; if not, add an entry with the item ID under "Raised by". A fail is not automatically a product bug; sometimes the document is wrong, and the fix is to the document. Say which in the Status line.
7. When every P1 and P2 item for a document has passed or been filed, change its row in the [coverage table](../README.md#coverage) from `drafted` to `verified`.

## Needs and conditions

- **scratch** — the scratch host repo above; no hooks fire there, so the binary's own behavior is isolated.
- **hooked** — a session under Claude Code with bee's hooks wired; the only way to see denies, repairs, the preamble, and the marks.
- **worktree** — a granted feature worktree (`bee worktree new`), plus main; needed for every control-plane split claim.
- **two sessions** — two live hooked sessions in one checkout, for write-policy, holds, and claim races.
- **piped** — stdout or stderr redirected (`2>/dev/null` and `>/dev/null`) to check the stream split.
- **codex** — a Codex runtime; every codex row is expected to be `blocked` until one is available.

## Driving the product from a script

Most items here are commands with expected output, streams, and exit codes, and can be run as a script; the Expected column is written to make that unambiguous. Use redirection to check streams and `echo $?` for exit codes. What a script cannot check: the hook-injected text an agent sees in conversation (preamble, capsule, nudges, repairs), which must be read in a live hooked session; and anything requiring a second live session's heartbeat timing. The store can always be *observed* directly (`cat` a store file to confirm a write) — observing is fine, writing by hand is exactly what the guards deny.

## Results so far

No formal pass has been run; every Result column is `—`. During drafting (2026-08-26, commit `6b0ae488`), the authors probed a subset of claims live — the capture group end to end, the front door's refusal wording, the stream split, the secret-guard Bash walk-around, `--check=true`, the unbuilt config/perf groups, `discovery list` truncation, and several deny texts in this repository's own hooked session. Those probes are recorded as "confirmed" notes in the documents and as Status lines in `bug-triage.md`; they are not checklist results, and no document is marked `verified` on their strength.
