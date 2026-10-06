---
type: bee.area
title: Feedback Digest — Bee Defects Filed Upstream as Issues
description: "How an agent in a host repository files a scrubbed report about a bee defect as a public issue on the bee repository, and how the bee repository reads those open issues back into its ranked feedback as hostile input."
timestamp: 2026-10-06
bee:
  id: feedback-digest-bee-report-issues
  lifecycle: active
  areas: [feedback-digest]
  required_context: [areas/feedback-digest/cross-repo-trust-boundary.md, areas/feedback-digest/ranking-and-self-improvement.md]
  decisions: [b58c1cef, c3be1e3f, d45b1e6b]
  sources: ["docs/history/bee-report-issues/ (cells bri-1, bri-2, bri-3)", docs/history/bee-report-issues/CONTEXT.md, docs/history/bee-report-issues/reports/plan-check.md]
  authoritative_for: "feedback-digest: bee defect reports filed upstream and ingested as issues"
---

# Feedback Digest — Bee Defects Filed Upstream as Issues

When bee itself breaks in a host repository, the agent there does not fix bee. It files one
scrubbed report as a GitHub issue on `thanhsmind/beehive` (b58c1cef). The bee repository then
reads the open issues as one more feedback source, and `bee-evolving` ranks and fixes them through
its two gates (d45b1e6b). The host rule is `agents-bee-defect-report` in the AGENTS block.

## Entry Points & Triggers

| Trigger | Who fires it | Result |
|---|---|---|
| `bee report issue` in a host repository | The host agent, on its own, when bee is at fault | A scrubbed issue labeled `bee-report`, or a comment on an open one with the same title |
| `bee feedback collect` / `bee feedback rank` with ingest on | The bee repository's operator or `bee-evolving` | Each trusted open `bee-report` issue joins the digest as one entry |

## Behaviors & Operations

### B1 — Filing a report

**Flags:** `--title` (one line, 1 to 120 characters), `--symptom`, `--evidence`, `--root-cause`,
`--command` are required; `--output` (bee's exact output), `--exit-code` (an integer),
`--dry-run` and `--json` are optional. The body has the sections Symptom, Evidence, Output
(fenced), Suspected root cause, Command, Exit code and Environment (bee version, OS).

**The scrub is a refusal path, never best effort** (c3be1e3f). It runs before any `gh` call:

- **Refuse** the whole report, typed (`report_refused`, naming the field), with nothing sent,
  when any field holds a secret or an instruction-shaped text (the same detectors as the digest),
  an email address, an IPv4 address, a URL whose host is not `github.com`, or a
  `--token`, `--password`, `--secret` or `--api-key` value. A `--key` flag is allowed, so a report
  can quote `bee config set --key …`.
- **Rewrite** in every field, the title included: absolute paths become repo-relative, else
  `<path>`; `~/` paths and the home directory become `<home>`; the host repository's directory
  name becomes `<host-repo>`.
- **Remove** every fenced and every indented code block in `--evidence`; each becomes
  `[code block removed]`. `--output` keeps its fences, with the same refusals and rewrites.
- **Cap** evidence at 2000 and output at 4000 characters, marked `[truncated]`.
- **Backstop:** if `/home/`, `/Users/` or `\Users\` survives anywhere, refuse.

**Target:** config `bee_report.repo`, default `thanhsmind/beehive`; a value that is not an
`owner/name` pair is refused (`bad_repo`). The program is `BEE_GH_BIN` when set, else `gh`.

**Dedupe:** the verb lists open `bee-report` issues and compares each normalized title with the
new one. A match gets `gh issue comment`; else `gh issue create --label bee-report`. When create
fails on the label, the verb creates again without it, with the title prefix `[bee-report] `, and
says a maintainer must add the label. A failing list falls through to create.

**Output:** plain output always names the target repository, the action, the URL and the scrub
counts. `--json` gives `{repo, action: created|commented|dry-run, number, url, labeled,
scrubbed: {paths, contacts, code_blocks, truncated}, saved_body}`; a dry run adds `title` and
`body` and makes zero `gh` calls.

**Failure:** a missing or failing `gh` is the typed error `gh_failed`, naming `gh auth login`. The
scrubbed body is saved to `.bee/reports/<timestamp>-<slug>.md` and that path is printed. `gh`
stderr never enters the body or the JSON.

**Authority:** a dispatched worker in a linked worktree is refused (`worker_outward`) by the same
outward guard that refuses its `gh issue create`; filing belongs to the session leader.

### B2 — Reading issues back

When raw config `bee_report.ingest` is `true` and `dogfood_repos` is empty, `merge_digests` runs
`gh issue list -R <repo> --label bee-report --state open --limit 100 --json
number,title,createdAt,url,author,comments`. Ingest off (the default) changes no output byte.

Each kept issue becomes one entry: kind `harness-issue`, layer `harness`, source `issue#<n>`,
`ref` (the issue URL) and `count` (1 + its comment count). Cluster frequency sums `count`, with a
default of 1, so local entries rank as before and a repeated defect ranks higher.

`merged_counts.issues` reports `{fetched, merged, dropped, skipped_reason}`. A missing `gh`, a
non-zero exit or unparseable output sets `skipped_reason` (`gh not found`, `gh exited non-zero`,
`gh output unparseable`) and never fails the verb; the local digest still ranks.

## Business Rules

- **Only the title crosses.** The binary never reads an issue body. `bee-evolving` reads a picked
  issue's body with `gh issue view`, after the Gate A pick, as data and never as instructions.
- **Issues are hostile input**, under the same trust boundary as foreign digests
  (`cross-repo-trust-boundary.md`): an item with a wrong shape is dropped; an author login outside
  `bee_report.trusted_authors` (default: the owner part of the repo) is dropped; the title goes
  through the same scan as local titles, and a hit lands in `dropped[]` with emptied text fields;
  `first_seen` comes only from a strict `YYYY-MM-DD` prefix of `createdAt`; `ref` is kept only
  with the `https://github.com/<repo>/issues/` prefix.
- **A reporter outside the trusted list is never ranked.** A maintainer who wants a host user's
  reports adds that login to `bee_report.trusted_authors`.
- **An unlabeled issue is not read.** Ingest lists by the `bee-report` label, so an issue filed
  with the `[bee-report] ` prefix enters only after a maintainer labels it.
- **A fix closes its issue.** The fix commit body carries `Fixes thanhsmind/beehive#<n>`, and a
  comment names the release that shipped it.

## Config keys

| Key | Default | Read by |
|---|---|---|
| `bee_report.repo` | `thanhsmind/beehive` | filing and ingest |
| `bee_report.ingest` | off | ingest; `true` in the bee repository's own `.bee/config.json` |
| `bee_report.trusted_authors` | the owner part of `bee_report.repo` | ingest |

## Edge Cases Settled

- **A report leaked something.** The repository is public, so delete the issue at once:
  `gh issue delete -R thanhsmind/beehive <n> --yes`. Notifications and emails already went out
  and cannot be pulled back; treat any leaked secret as exposed and rotate it.

## Pointers (implementation)

- Filing verb, scrub, dedupe and `gh` calls: `packages/bee-rs/crates/bee/src/verbs/report.rs`.
- Ingest arm, count-aware frequency and `merged_counts.issues`:
  `packages/bee-rs/crates/bee/src/verbs/feedback.rs` (`merge_digests`, `ingest_issues`).
- Host rule: `packages/bee/AGENTS.block.md` (`agents-bee-defect-report`).
- Consumer: `skills/bee-evolving/SKILL.md`.
