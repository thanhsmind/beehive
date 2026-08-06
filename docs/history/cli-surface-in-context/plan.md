# cli-surface-in-context — plan

## The problem, measured

Across one working session an agent invented four flag names that do not
exist: `decisions log --text` (the real pair is `--decision` / `--rationale`),
`cells claim` without `--worker`, `reservations list --active`, and
`reviews candidate add --ref` (the real one is `--head`). A second agent, in
another project, invented `state route --lane-value`.

Every one of those is a wrong **name**. Not one was a misunderstood meaning.
That distinction decides the whole design: an agent that knows a flag exists
can look up what it does; an agent that does not know it exists invents a
plausible spelling and is refused.

Why the names are guessed rather than read:

1. **The surface is not in context.** `bee --help` prints one line of prose per
   command and no flags at all. Flags live behind `bee <cmd> --help --json`,
   one tool call per command, across 142 commands.
2. **Until 2.2.4 the correct path was blocked.** The CLI-shape guard refused
   `bee <cmd> --help` whenever the subcommand had required parameters — the
   very commands whose flags most needed reading. The cheap correct move did
   not exist, so guessing was the only move. That is fixed; the habit it
   trained is not.
3. **The vocabulary does not generalize.** 143 distinct flag names across 142
   commands. The actor is `--worker` in `cells claim` but `--agent` in
   `reservations reserve`; a commit is `--head` in `reviews candidate add`;
   the body is `--decision` in `decisions log` but `--title` in `backlog add`.
   Knowledge of one verb transfers to no other.
4. **One of the two refusal shapes does not teach.**
   `missing_required_argument` names the missing flags, so a caller recovers in
   one round. `unsupported_argument_shape` says only that "an optional flag, a
   flag value, or a target" was refused — it holds both the registry entry and
   the argv and names neither.

## Size of the fix

Measured against the shipped registry:

| Variant | Content | Per session |
|---|---|---|
| V1 | flag names + required marker | ~1,560 tok |
| V2 | + declared type | ~1,950 tok |
| V3 | + one description line per flag | ~7,000 tok |
| V4 | the whole registry | ~44,200 tok |

**V2 is the chosen shape** (user decision, this session). It closes every
failure actually observed — all of them name errors — for about 2k tokens,
while V3 costs 3.6× that on every session of every project, including the
many that never touch the CLI. Meaning stays one `bee <cmd> --help` away,
which now works.

## Shape

Three cells, one per surface. They are independent: no shared file, no
ordering constraint.

### csc-1 — the preamble carries the flag index

`packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs`

A new `### Command surface` section, rendered from the embedded registry —
never from a checked-in copy, which would drift the moment a verb changes.
One line per command:

```
cells claim: --id*:str --worker*:str
reservations reserve: --agent*:str --cell*:str --path*:str --session:str --kind:str
```

`*` marks required. `--json` is omitted from every line and stated once in a
header note, since 130 of 142 commands accept it and repeating it costs ~1KB
for no information.

Placement: after `### Standard commands`, before `### Doc links`. The closing
trailer's bytes are pinned by an `ends_with` assertion in the preamble tests —
nothing may be appended at the end.

### csc-2 — the refusal names the flag it refused

`packages/bee-rs/crates/bee/src/router.rs`

At the `unsupported_argument_shape` branch, diff the argv's `--flags` against
the entry's schema properties. Name every unknown flag, and for each offer the
nearest declared spelling when one is close enough to be worth printing:

```
bee: unsupported argument shape for `bee state route`: unknown flag --lane-value.
Did you mean --lane? ...
```

When no flag is unknown — an out-of-enum value, a target that does not
exist — the message keeps today's wording, which is then accurate.

This honors cli-ergonomics D1 (8ef2bae6): every problem named in one message.

### csc-3 — a vocabulary ratchet

`packages/bee-rs/crates/bee/src/catalog.rs` (or the registry's own test
module)

A test pinning the count of distinct flag names across the registry. Adding a
new spelling for an existing concept then requires deliberately bumping a
number, which is a decision rather than an accident. It does not rename
anything today — renaming 143 flags is a breaking change this feature does not
take on — it stops the divergence from growing.

## What this does not do

- No flag is renamed or aliased. Existing callers keep working.
- No description text enters the preamble. That is V3, priced and declined.
- The CLI-shape guard is unchanged: it already refuses a malformed call before
  it runs, and remains the backstop.

## Verification

`commands.test` for every cell. Beyond it:

- csc-1: a preamble test asserting the section renders, that `--json` appears
  nowhere in it, that required flags carry `*`, and that the trailer bytes are
  unchanged. Plus a size assertion — the section must stay under a stated
  character budget, so a future verb explosion is caught rather than silently
  paid for.
- csc-2: a router test for the unknown-flag case (named + suggestion) and for
  the no-unknown-flag case (today's bytes).
- csc-3: the ratchet test is its own proof.
