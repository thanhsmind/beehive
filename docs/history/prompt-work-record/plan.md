# Prompt Work Record — Plan

**Feature:** prompt-work-record · **Lane:** standard · **Class:** feature
**Triage flags:** public-contracts, multi-domain · **Product files:** 2
**Context:** `docs/history/prompt-work-record/CONTEXT.md` (D1–D6)

> Route note: `bee state route --set` could not record this triage. It refuses
> inside a granted feature worktree (control-plane command), and from the main
> checkout it can only target the session-bound lane — this session has no
> session record in beehive's store, because its hooks write into waggledance's.
> The triage is therefore recorded here, in the plan, and the gap is filed to
> the backlog.

## Triage, counted

| Flag | Present | Why |
|---|---|---|
| auth / authorization | no | No identity or permission path is touched. |
| data-model | no | An additive key on an existing JSON record; every reader already tolerates unknown keys. |
| audit-security | no | The stored text is the user's own prompt, on the user's own machine, behind the board's existing access control. D5 **adds** redaction; no protection is weakened and no trust boundary moves. |
| external-systems | no | Nothing leaves the machine. |
| cross-platform | no | Pure Rust and JSON; no path, shell or process specifics. |
| public-contracts | **yes** | `.bee/sessions/<id>.json` is parsed by waggledance. |
| covered-contract-change | no | No existing assertion changes meaning. |
| proof-weakening | no | Tests are added only. |
| multi-domain | **yes** | The hook layer plus the sanitizer the decisions verbs already own. |

Two flags, story-sized behaviour → **standard**.

## Deferred-to-planning questions, answered

**Which hook writes it.** `hook activity`, no sibling. It is already wired to
`UserPromptSubmit` (`.claude/settings.json` wires `prompt-context` and
`activity` there), it is already the session record's only writer, and it
already owns the store lock, the atomic write, the capped log and the
always-exit-0 posture (`activity.rs:1-50`).

**Which secret guard.** `SECRET_PATTERNS` (`verbs/decisions/scanners.rs:354`) —
the six matchers behind `assert_safe_content`: private key, AKIA, ghp, sk-,
JWT, key=value secret. **Only the secret half.** `assert_safe_content` also
runs `INJECTION_PATTERNS` ("ignore previous instructions", "disregard…"), which
a user types legitimately every day; running those over a prompt would redact
ordinary work.

**Herded panes.** The same rule the state machine already follows: a pane bee
herding opened has no session record at all, only
`.bee/mailbox/<job-id>/activity.json`. The prompt record goes to that mailbox
file, through `write_activity_herded`, exactly as the state does.

## Shape

One slice is current. The whole change is a sanitizer plus one call site, both
inside the blast radius of a single test module.

### Slice 1 — the hook records the prompt (current)

`hook activity`, on `UserPromptSubmit`, puts a `work` object beside `activity`
on the session record (or on the herded mailbox record):

```
work: { title, text, status: "open", opened_at, updated_at, turns }
```

- `title` is the first prompt's text, and never changes while the record is
  open (D3).
- A follow-up prompt appends to `text` and bumps `turns`/`updated_at` rather
  than opening a second record (D3).
- `status` starts `open` and only the agent moves it (D4) — this slice writes
  no other value.
- `text` keeps the user's wording, with absolute paths scrubbed and a
  secret-shaped prompt stored as `[redacted]` (D5).
- The write happens **before** the state machine's suppression and
  no-transition early returns. A prompt arriving is a fact about the session
  regardless of what the state machine concludes about it — routing it through
  the transition logic would silently drop prompts whenever the prior state was
  sticky.
- D4's expiry needs no code: the record hangs off the session file, and every
  reader already computes liveness from `last_heartbeat` and drops a stale
  session. The cell proves that rather than building it.

### Slice 2 — the agent's upgrade surface (headline only)

A verb the agent calls to add the acceptance and move the status, which is what
promotes a stub to a board card (D1, D2). Not a cell yet.

### Slice 3 — waggledance renders it (headline only, other repo)

`BeeActivity`'s sibling: parse `work` and show the title on the session row;
show a card once an acceptance is present. Dependent on slice 1 shipping (D6).

## Smaller path

*Is there a cheaper shape that still honours every locked decision?*

Yes, and it is taken: slice 1 is **one** cell, not a sanitizer cell plus a
wiring cell. Evidence — `write_activity` (`activity.rs:447`) is the single
write point, and `SECRET_PATTERNS` (`scanners.rs:354`) already exists, so the
new code is one pure function and one call site in one file, sharing one test
module. Splitting it would buy two commits over the same tests. D4 shrank from
a cell to a proof for the same reason: the expiry is already inherent.

## Test scope

`packages/bee-rs` — the crate's own suite, run as the recorded
`commands.test`. Coverage judgment: `hooks/activity.rs`'s existing test module
covers the state machine and both sinks; the gap this cell authors is the
`work` object — first prompt, follow-up append, title immutability, path
scrub, each of the six secret shapes redacted, an injection-shaped prompt left
alone, the herded-mailbox sink, and a suppressed-state event still recording
the prompt.
