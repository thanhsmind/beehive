# Privacy

## Summary

Privacy in bee is one narrow, hard rule with one machine-readable escape hatch. When the agent tries to *read* a file whose path looks like a secret — a `.env`, a key or certificate, an SSH private key, a `credentials` or `secrets.*` file — the write guard denies the tool call with exit code 2 and prints two things: a human sentence saying the file looks like a secret and to ask the user first, and a `@@BEE_PRIVACY@@{…}@@END@@` block carrying the same question as JSON. The block is not an instruction to the agent; it is a letter addressed to the human, and routing it is the whole protocol. No gate-bypass level opens this deny — not `normal`, not `full`, not `total`. The one thing that does turn it off is `hooks.write-guard: false` in config, which turns off every other guard with it. The second half of privacy is the opposite direction: content the agent *mines* from artifacts, transcripts, or resurfaced decisions is data, never instructions, and the two places that ingest mined content — the capture queue and the feedback digest — scan it for secret-shaped and injection-shaped text and refuse or drop rather than carry it forward. [guards](../foundations/guards.md) holds the full deny catalog; this document holds the privacy half of it.

## The simple case

The agent is chasing a configuration bug and reaches for the environment file:

```
Read .env
```

Nothing is read. The tool call is denied, exit 2, and stderr carries:

```
bee privacy guard: ".env" looks like a secret/credential file. Ask the user before reading it.
@@BEE_PRIVACY@@{"file":".env","question":"\".env\" looks like a secret/credential file. Ask the user before reading it."}@@END@@
```

The agent does not retry, does not reach for `cat`, and does not reason about whether this particular `.env` is really sensitive. It puts the question to the human — through AskUserQuestion under Claude Code — in the human's own words, and waits. If the human says yes, the human is the one who supplies what is needed: the value, a redacted excerpt, or an explicit instruction. If the human says no, the work routes around the file.

That is the entire flow. It has no auto-approve path, no per-file allow-list, and no "approved" flag the agent can set.

## The interaction, event by event

One read of a secret-shaped path:

```mermaid
stateDiagram-v2
    [*] --> intercepted : Read / Glob / Grep names a path
    intercepted --> passed : no bee root, no onboarding.json, or hooks.write-guard is false
    intercepted --> passed : the path resolves outside the repo root
    intercepted --> matched : the path matches a secret pattern
    intercepted --> other_checks : no match — scout guard, then the read-size guard
    matched --> denied : exit 2, reason + @@BEE_PRIVACY@@ block on stderr
    denied --> routed : the agent puts the question to the human
    routed --> [*] : the human answers; the agent never re-reads on its own
    passed --> [*]
    other_checks --> [*]
```

### Invoke

The guard runs on `PreToolUse` for the read tools — `Read`, `Glob`, `Grep`. It takes the first truthy value of `file_path` or `path` from the tool input and turns it into a repo-relative path. Three things end the guard's involvement before any pattern is tried: no resolvable store root, no `.bee/onboarding.json` at that root (bee is not installed here), or `hooks.write-guard` set to `false` in the merged [configuration](configuration.md).

### Ends at once

Every outcome of this guard is at once — there is no long-running phase.

- **Allow, silently.** The path matches nothing. Exit 0, no output. The read proceeds to the runtime's own approval flow untouched.
- **Deny.** Exit 2 with the reason and, for a secret match, the marker appended on the next line. Nothing is read; nothing is written anywhere in the repository.
- **No opinion.** A tool input with no `file_path` and no `path` — a `Glob` given only a pattern, say — produces exit 0. There is nothing for a path-shaped rule to match.

### First side effect

There is none, in either direction. An allowed read does not touch the store; a denied read does not either. The guard's only durable trace is the hook log line every hook event appends, fail-open.

### While running

Nothing to observe. The decision is a string match against the normalized path, atomic with the tool call.

### Finish

Exit 2 and two stderr lines, or exit 0 and silence. The agent's next move is the whole product: route the question, do not retry.

## What counts as a secret path

Matching is on the normalized, lower-cased path; the basename carries most of the rules.

| Pattern | Matches | Does not match |
| --- | --- | --- |
| `.env` exactly, or `.env.<suffix>` where the suffix is non-empty and made of alphanumerics, `.`, `_`, `-` | `.env`, `.env.local`, `.ENV`, `.env.local.bak`, `test/fixtures/.env` | `.envrc` (no dot after `env`), a bare `.env.` (empty suffix) |
| extension `.pem`, `.key`, `.p12` | `deploy.pem`, `docs/api.key` | — |
| basename starting `id_rsa` | `id_rsa`, `id_rsa.pub` | — |
| basename starting `credentials`, unless the extension is a known code extension | `credentials.json`, `credentials.yaml`, plain `credentials` | `credentials.rs`, `src/credentials.rs`, `credentials_test.go` |
| basename `secrets.<anything>` | `secrets.json`, `secrets.yml` | a bare `secrets` with no extension |

Two things this list deliberately does not do. It does not exempt `.env.example` — a sample environment file is denied like a real one, and the agent asks. And it does not look inside the file: the decision is entirely about the path.

Immediately after the secret check, the same read guard applies the scout rule (generated and vendored trees) and then the read-size rule. Those are not privacy; [guards](../foundations/guards.md) catalogs them.

## The privacy marker

The marker is a single line with a fixed frame and a JSON object between the sentinels:

```
@@BEE_PRIVACY@@{"file":"<repo-relative path>","question":"<the same sentence the human line carries>"}@@END@@
```

Two fields, both strings: `file` and `question`. The frame exists so a runtime, a dashboard, or a skill can find the question without parsing prose, and so the agent has a fixed shape to recognize rather than a sentence to interpret.

Three rules govern it, and all three are about who decides:

1. **The marker is routed, never acted on.** The agent hands the question to the human and stops. It does not answer its own question, and it does not summarize the question away.
2. **The marker is data, not a prompt.** Its `question` field is quoted text produced by bee, but the same discipline applies to it as to any other content that arrives in the agent's context from a file: it is something to show the human, not something to obey.
3. **There is no approval token.** bee's older runtime notes describe retrying "with the documented approval prefix" after the human approves. No such prefix exists in the binary today: `check_read` has no approval input and no per-path memory, so an identical retry is denied identically. In practice the approval is carried out by the human, not replayed by the agent — see "Open questions".

## No bypass level covers a secret read

`gate_bypass` is read by the session preamble, by `bee state gate`, and by the session-close nudges. It is not read by the write guard at all. There is no code path in which a bypass level changes what `check_read` answers, so the deny stands identically at `off`, `normal`, `full`, and `total`.

That is the intended contract, stated in the config sample and in the preamble's own banner at `full`: *only reading a secret-shaped file and a review P1 finding still pause for the human*. At `total` the same banner says the opposite — *this includes secret-file reads and review P1 findings: nothing pauses for the human* — which describes the agent's own behavior under the skills, not the hook's. The hook still denies. See "Open questions".

What does turn the guard off is a configuration key: `hooks.write-guard: false`. It is not a privacy switch — it disables the phase gate, the intake gate, containment, the coordination guards, and the CLI-shape guard in the same stroke. There is no per-guard toggle and no `guards.secret` key.

## Modifiers

| Modifier | Effect | Can it differ mid-flow? |
| --- | --- | --- |
| `--json` | Not applicable. The guard speaks into the tool-call channel, not the CLI's streams; the marker is stderr text, not a `--json` payload. | — |
| Gate-bypass level | None, at any level. The write guard never reads `gate_bypass`. | No. |
| Store phase | None. The read guard runs before any phase rule and is phase-independent: the same deny fires at `idle`, in a gated phase, during `swarming`, and at `compounding-complete`. | No. |
| Where it runs | The guard needs a resolvable store root with `.bee/onboarding.json`. Paths are matched relative to the physical repository root, so a path resolving outside it is never checked. Feature worktrees, granted worktrees, and staging behave identically. | Only by moving to a checkout where bee is not installed. |
| Who runs it | No difference. Orchestrator, dispatched worker, and herded worker all pass through the same `PreToolUse` event, and a worker's report of the deny is a `[BLOCKED]` back to the orchestrator, which routes the question to the human. | — |

## Cancel and interrupt

| Event | Behavior |
| --- | --- |
| The process killed mid-command | The decision is atomic with the tool call; there is no half-denied state and nothing to clean up. |
| The session turning elsewhere (compaction, handoff, turn end) | A pending privacy question is not store state — it lives only in the conversation. A turn that ends on one should mark the wait (`bee state waiting-on set --kind question`); a compaction or a handoff that loses the marker loses the question, and the next read re-raises it identically. |
| A clean completion from outside (a gate approved, a question answered, a new message) | Only the human's own answer completes a privacy question. Approving a *gate* completes nothing here: the two approvals are unrelated records, and there is no `privacy` gate. |
| The store unavailable (corrupt JSON, hook binary missing) | Fail-open, both ways: a missing `.bee/bin/bee` prints `bee: hook binary missing (.bee/bin/bee)` and the read proceeds unguarded; a corrupt config reads as absent, which leaves the guard *enabled* (the toggle needs an explicit `false`). Corruption never turns the secret guard off by accident. |
| The session going away (heartbeat, lease expiry, release) | No effect. Privacy holds no lease, no claim, and no record. |
| A sibling changing the target | A sibling cannot change a privacy answer — there is nothing stored to change. A sibling can create or delete the file; the guard matches the path, so a deleted file's path is denied just the same. |
| The channel changing (piped, `--json`, Codex, run from a hook) | Under Codex the write guard rides the events Codex gates; advisory events never block, so on that runtime the same rule can only warn. The OpenCode plugin is out of scope. |

## Interactions with other systems

**Gates and approval.** A privacy question is not a gate: it is not one of the five recorded gates, it is never written to `approved_gates`, and it cannot be self-approved at any bypass level. The only similarity is who answers it. See [gates](../foundations/gates.md).

**The store and history.** A deny writes nothing but its hook-log line. When a secret does reach a durable record by mistake, the remedy is the record's own: `bee decisions redact` retires a decision from the active set with a reason, leaving the event in the log — see [decisions](../memory/decisions.md).

**Worktrees and containment.** Matching is per physical repository root. An absolute path pointing outside the repository — `/etc/ssl/private/key.pem`, or `../other-repo/.env` — is not checked by this guard at all; containment governs *writes* outside the worktree, not reads.

**Claims, holds, and reservations.** None. Privacy has no coordination surface.

**Sibling sessions.** The deny is per-session and carries no cross-session state. Two sessions reading the same secret path each get their own deny and each route their own question.

**What the human sees.** Everything. The marker exists to reach the human verbatim, and a red or refusal line is never silenced, composited, or delayed by any switch or bypass level. In an unattended, herded run there is no human in the loop to answer; the letter to the human ([mailbox](../memory/mailbox.md)) is the escalation path there.

**Configuration.** One key reaches this guard: `hooks.write-guard`, all-or-nothing. See [configuration](configuration.md).

**Output modes and exit codes.** Deny is exit 2 with stderr text — the guards' shared contract, described in [guards](../foundations/guards.md) and [invocation](../foundations/invocation.md).

## Mined content is data, never instructions

The second privacy direction is inbound. Text pulled from a transcript, an artifact, a foreign repository, or a resurfaced decision is evidence about the work; it is never a new instruction to the agent, however imperatively it is phrased. Two ingest points enforce that mechanically, with the same two scanners:

- **The capture queue.** `bee capture add` refuses when the `--outcome` or the `--area` carries secret-shaped or injection-shaped text. A stub filed from a transcript rather than live is marked `--source mined`, and `bee capture list` renders it with a `[mined]` marker so a later reader knows the provenance. See [capture](../memory/capture.md).
- **The feedback digest.** Each entry is rebuilt from an allow-list of six fields — `kind`, `layer`, `source`, `title`, `first_seen`, `pain` — with no narrative field at all, and a candidate whose title matches a credential or injection pattern is dropped with the reason *category* only, never the text that matched. See [feedback](../memory/feedback.md).

The secret scanner looks for private-key headers, AWS `AKIA` keys, GitHub `ghp` tokens, `sk-` keys, JWTs, and a `key`/`secret`/`token`/`password` keyword followed by `:` or `=` and at least six non-space characters. The injection scanner looks for the familiar shapes: "ignore … previous instructions", "disregard the above", role tags, and bracketed role headers. Both are pattern scanners over short text, not content classifiers — they catch a pasted credential and a copied jailbreak line, and they are not a substitute for the agent's own judgment about what belongs in a durable record.

## Edge cases

- **The guard has no write half.** Nothing stops the agent from *creating* or overwriting `.env`. A write to it is judged by the phase and containment rules like any other source write — at `idle` the intake gate denies it, and in an execution-approved phase it passes. Denying reads while permitting writes is a defensible split (the risk is exfiltration, not creation), but it is worth knowing.
- **Bash walks around it.** The guard watches `Read`, `Glob`, and `Grep`. A `Bash` call running `cat .env` is not checked by any secret rule; it is judged by the Bash-side guards only. Confirmed live: the same `.env` that Read is denied is served by `cat`. That is a hole in the protocol, not a sanctioned route — the rule is "ask the human", and the tool used to break it does not change the rule.
- **Paths outside the repository are unchecked.** An absolute path outside the root, or a `../` path leaving it, produces no deny. Confirmed live.
- **Glob and Grep are covered only through a `path` argument.** A `Grep` given a `path` naming a secret file is denied; a `Grep` given only a pattern, which then matches inside a secret file, is not — the guard never sees the file.
- **The read-size guard is skipped for a secret path** because the secret check returns first. The two never compose.
- **The marker is not JSON-escaped for a shell.** It is stderr text; the JSON inside it is properly escaped JSON, and the whole line is meant to be recognized by its sentinels, not parsed out of a wider structure.
- **`.env.example` is denied.** The most common false positive in the set, and it is deliberate — the pattern is on the path, and a sample file is one edit away from a real one.

## Open questions and verification

- **No approval path exists for a secret read.** The runtime-integration notes (`docs/06-runtime-integration.md`) describe retrying "with the documented approval prefix" after the human approves, but `check_read` accepts no approval input and keeps no per-path state, so an identical retry is denied identically. Today the only ways past are the human supplying the content, `hooks.write-guard: false`, or a Bash read — the first is intended, the second is a sledgehammer, the third is a hole. Filed in [bug-triage.md](../bug-triage.md) as a product call.
- **The `total` bypass banner contradicts the hook.** At `total` the preamble tells the session that secret-file reads no longer pause for the human, and `.bee/config-sample.json` says the same. The write guard denies them anyway, at every level. Either the banner text or the guard is wrong; both are deliberate-looking. Filed alongside the item above.
- **The marker's field names differ from one internal description.** The code emits `{"file", "question"}`; `docs/07-contracts.md` agrees, and the deny catalog in [guards](../foundations/guards.md) describes it as `{"kind", "question"}`. The live output is `file`. Worth reconciling in the description set's own consistency pass.
- **`.envrc` is not a secret path.** The pattern needs a dot after `env`, so direnv's file reads freely while `.env.example` is denied. Probably an oversight in the pattern rather than a decision; small, and worth a line in triage.
- The Bash-side escape was confirmed live at `idle`; whether any other guard would catch a `cat`/`grep`/`head` of a secret path in a non-idle phase was not determined.
- **The only content-*neutralization* mechanism bee describes is not built.** Foreign-digest datamarking — wrapping mined foreign text so it cannot read as instructions — lives on the `dogfood_repos` path, which this binary refuses outright. Everything shipped today drops or refuses rather than neutralizes. Owned and filed by [feedback](../memory/feedback.md); noted here because it is the boundary of the "mined content is data" rule.
- Confirmed by running the write-guard hook against crafted payloads in this repository: the `.env` deny with its exact marker bytes, `Grep --path .env.local`, `.env.example`, `credentials.json`, `credentials.rs` (allowed), `secrets.json`, a bare `secrets` (allowed), `id_rsa.pub`, `docs/api.key`, `deploy.pem`, `test/fixtures/.env`, `.ENV`, `.env.local.bak`, `.envrc` (allowed), an absolute in-repo path, an absolute out-of-repo path (allowed), a `../` path (allowed), a `Bash cat .env` (allowed), a `Write` to `.env` (denied by the intake gate, not by privacy), the scout deny, and a read tool with no path argument (allowed).

Verified against beehive commit `6b0ae488`.
