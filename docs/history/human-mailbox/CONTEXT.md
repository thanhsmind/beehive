# Human Mailbox — Context

**Feature slug:** human-mailbox
**Date:** 2026-08-25
**Shaping session:** complete
**Scope:** Deep
**Domain types:** RUN | READ | ORGANIZE

## Feature Boundary

After an unattended run, bee files one plain-language letter per run into
`.bee/human-mailbox/` — appended entry by entry as the run works, composed
when it ends — and offers the one command that flips a letter's read
state. It ends there: reading, listing and displaying letters is
waggledance's inbox, handed over separately (D17).

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted. Numbering matches `docs/discovery/human-mailbox/MAP.md`;
the short id in each row is the decision-log event.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 (303488de) | bee owns the mailbox data and the code that emits it, and nothing above it. The inbox UI is waggledance's. | Fixes this feature's ceiling: no rendering surface, no listing UI, no viewer ships from bee. |
| D2 (e3618e3b) | Every record carries a required subject — one sentence, plain language, no bee vocabulary, answering "what happened" on its own. A record without a readable subject is not valid. | The subject is the inbox row on the consuming side, so it is a validity rule, not a formatting preference. |
| D3 (1b079912) | One record is ONE markdown file per letter with typed YAML frontmatter: frontmatter is the machine contract (`subject`, `run`, `project`, `filed_at`, `status`, `items[]` of `{what, files[], commit, proof, departure}`, `needs_you[]`), body is the human prose. No JSON twin, no separate index stream. | One artifact cannot drift against itself; the human opens the file with no tooling and the consumer parses frontmatter. |
| D4 (1d56c1d2) | Two layers: every clean stop (cell capped, feature closed, blocker hit) appends its raw entry the moment it happens; the letter is composed from those entries when the run ends. | A run that dies at 3am must still leave everything up to the moment it died — end-only composition leaves nothing. |
| D5 (e9cb4c15) | A departure from the plan is recorded in three required parts — what was done differently, why, and which kind — kind from the closed set: hit an unforeseen obstacle / found a better route / the plan was wrong about a fact / something else had to be fixed first. A cell that followed its plan states that explicitly rather than leaving the field empty. | Narrows the free-form one-line `--deviation` value of dc6a2d26. Silence and nothing-happened must not read alike. |
| D6 (2009bc71) | Read/unread is a field bee owns inside the letter file. The waggledance inbox flips it by calling a bee command, never by writing the file. | bee stays the only writer of its own store; the consuming reader stays a reader. This feature must therefore ship that command. |
| D7 (b94381e5) | The nightly letter body carries five sections — Done / Where I departed from the plan and why / Broken or unfinished / Needs your call / Next. A section with nothing to report is dropped, never printed empty. Architecture, behaviour and usage appear only in the feature-close letter. | — |
| D8 (1c7a9d87) | The plain-language sentences are written at the moment of each event. The end-of-run pass may reorder, group and drop, and may never state a fact no stored entry carries. | The composing pass is a renderer with an authorship ban, not a summarizer. |
| D9 (d970d6fc) | Every session appends its entries, attended or not; only an unattended run composes and files a letter. | A session that starts attended and becomes an overnight run keeps a complete record of its whole span. |
| letter-digest D2 (aedb5be9) | Narrows D9: every `bee close` now files its close letter at the moment of close, attended sessions included; D9's rule (only an unattended run files the run-end letter) stays for run-end letters. | The feature-close letter no longer waits for run end or mailbox arming. |
| D10 (1fb69f4b) | The explicit no-departure statement D5 requires is enforced only while the mailbox is armed for the run. A cap in a run that files no letter keeps the byte-identical flagless behaviour dc6a2d26 promised. | Resolves the collision between D5 and dc6a2d26 without breaking existing callers. |
| D11 (349f25d8) | One letter maps to one run, never one night. Filename `<UTC-timestamp>-<short-run-slug>.md`. The subject stays in frontmatter. | A run is the unit the human reasons about; folding a night hides that one run died. Timestamp-led names sort correctly in a bare directory listing. |
| D12 (05b5f964) | A run that dies without reaching its own end gets its letter from the NEXT session that starts. That letter is marked plainly as an unfinished run, lists entries up to the last one, and names the moment the run went silent. | No background scheduler is added — a scheduler shares the failure mode it would exist to cover. |
| letter-digest D3 (dbbe0778) | Reuses D12's pattern for a new record: the daily/weekly digest is composed by the next session that starts after the period ended and finds the digest missing, from that period's close letters and usage records. | Same recover-on-next-session shape, no scheduler added for the digest either. |
| D13 (c3ece144) | Each item in the letter's Needs-your-call section carries a stable id and names what it blocks. This feature ships no path to answer from the inbox. | The id is what keeps a reply surface reachable without rewriting the record shape or any filed letter. |
| D14 (a6475e2c) | The feature-close letter is in scope: the same record shape carrying the extra architecture, behaviour and usage sections D7 names. A weekly digest is out of scope. | — |
| letter-digest D1 (b610a1dc) | Extends D14: the mailbox stays a directory of files (no email transport, no SMTP, no send command); the weekly digest D14 scoped out is a markdown file filed beside the letters in `.bee/human-mailbox/`. | The digest feature D14 deferred later shipped on the same record shape, unchanged. |
| D17 (1660158a) | bee is a development harness used to build waggledance — two separate projects, not two halves of one system. bee authors the record description in its own docs; the handover crosses by messaging the waggledance session, which owns what enters its own backlog. A bee session never writes a row into another project's backlog on its behalf. | Replaces D15 (4c05dde2) and D16 (e255fe3a). Fixes the delivery route, and bounds this feature to bee's own tree. |

### Agent's Discretion

Everything below the product surface: the verb group's naming and file
layout, how entries are stored before composition, how the append hook
reaches every clean stop, and how an unattended run is recognised at
runtime. Constraint: no new background process (D12), and no write into
another project's tree (D17).

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| letter | One filed record: a markdown file with frontmatter, covering exactly one run (D3, D11). |
| entry | One raw append written at a clean stop, before any letter exists (D4). |
| departure | A recorded difference between what the plan said and what was done, in three parts (D5). |
| filed | An entry set composed into a letter and written to `.bee/human-mailbox/` (D4). |
| armed | The mailbox is on for this run, so it will file a letter and D5's explicitness is enforced (D9, D10). |
| unfinished letter | A letter filed by a later session for a run that went silent (D12). |

## Specific Ideas And References

- The `ak:sumup` skill is the content model the human named — what shipped,
  what failed, what was worked around, what was decided, how to use it,
  what remains. D7 narrows it to five sections nightly and keeps the fuller
  shape for the feature-close letter.
- The human's own framing: they read the mailbox to see "where the agent
  decided something off-plan, and why" — an unforeseen blocker, or a better
  route that only appeared during the work. That is D5's closed kind set.

## Existing Code Context

From the quick scout only.

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/knowledge/` — `deviation_text`, described
  in `handlers_close.rs:13` as "the ONE rendering of a deviation entry",
  already shared with the miner. The letter must render through it rather
  than growing a second idea of what a deviation reads like.
- `packages/bee-rs/crates/bee/src/verbs/triggers/` — a file-backed verb group
  with add / list / resolve over a store directory: the closest working
  analogue to the mailbox's own verb group.
- `packages/bee-rs/crates/bee/src/verbs/discovery.rs` — a verb group whose
  store is documents on disk rather than a JSON blob.

### Established Patterns

- Result form validation at cap — `bee cells finish` validates the worker's
  `{outcome, commit, files, tests, deviations}` key-for-key onto the trace.
  D5 and D10 extend exactly this path, not a parallel one.
- Append-only JSONL stores under `.bee/` with a folded read (`backlog.jsonl`,
  `decisions.jsonl`) — the shape D4's entry layer should follow.

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs` — parses and
  validates `--deviation` at cap; where D5's three parts and D10's armed-only
  enforcement land.
- `packages/bee-rs/crates/bee/src/verbs/cells/trace.rs:32` — the trace's
  `deviations` array, seeded empty today.
- `packages/bee/prompts/worker-cell.md:45` — the Result form the worker is
  told to emit; D5 changes what a `deviations` line must contain, and D8
  adds the plain sentence written at the moment.
- `packages/bee-rs/crates/bee/src/verbs/work.rs` — session start and the
  herding surface: where D9 decides armed-or-not and where D12's next-session
  filing hooks in.
- `.bee/config.json`'s `herding` block — the existing signal for an
  unattended run.

## Canonical References

- `docs/discovery/human-mailbox/MAP.md` — the discovery map every locked
  decision above came from, with its closed tickets.
- Decision `dc6a2d26` — the `--deviation` flag D5 narrows and D10 reconciles.
- `docs/knowledge/work/dispatch-blocked-notify/delivery.md` (waggledance repo)
  — the existing alert path for a run that needs a human; adjacent, not part
  of this feature.

## Outstanding Questions

### Deferred To Planning

- [ ] How every clean stop reaches the append hook — cap, feature close and
      blocker are three different code paths — resolved by tracing them from
      `handlers_close.rs` outward.
- [ ] How a later session recognises orphaned entries for D12 without
      scanning the whole store on every start.
- [ ] What the read-flip command of D6 is called and where it sits in the
      verb tree.
- [ ] Whether the entry layer is one JSONL per run or a directory of entries,
      given D11's one-letter-per-run rule.

## Deferred Ideas

- Push the letter's subject line through waggledance's existing notification
  outbox — named as out of scope in the handover message to the waggledance
  session; it is that project's call, not bee's (D17).
- Answering a Needs-your-call item from the inbox — D13 keeps the ids so this
  stays reachable; the routing and permissions it needs are a separate shape.
- A weekly digest folding several nights — out of scope by D14; the record
  shape does not have to change to add one.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable and match the
discovery map. Planning reads the locked decisions, the code context, the
canonical references and the deferred-to-planning questions. The waggledance
inbox is NOT part of this feature's scope or its gate — it was handed to the
waggledance session under D17 and proceeds on its own.
