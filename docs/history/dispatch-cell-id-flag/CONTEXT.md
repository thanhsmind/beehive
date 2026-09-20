# dispatch-cell-id-flag

## What was asked

Patch the P1 filed against `herding-cap-check`: the cap-refusal it
added is INERT, because the dispatch door never passes the cell id the
refusal keys on.

## What was found

- `bee dispatch prepare --kind cell` composes
  `.bee/bin/bee herding run --task-file - --json --cwd "…" --agent "…"
  --ceiling 1800` — `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2480-2531`.
  It carries neither `--cell` nor `--cell-id`. (ran: `bee dispatch prepare`)
- `herding run` parses `--cell-id` only — `herding/run.rs:369-372`. So
  `opts.cell_id` is always `None` on a door-prepared cell dispatch.
- `postflight_cap_check` returns `Proceed` on `cell_id == None` —
  `herding/run.rs:4065-4067`. The refusal at `:4191-4197` can never fire.
- `bee dispatch wave` builds each cell's payload through the same
  `prepare_dispatch`, so ONE fix covers both doors — `prepare.rs:3862`.
- A byte-exact assertion covers the herding command for `kind: "cell"` —
  `prepare.rs:5859`. It must be updated with the change.

## Decisions

- **D1 — the flag rides the herding arm only.** `Resolved::Cli`
  composes a user-configured binary's command line, not a bee verb, so
  `--cell-id` there would be an unknown flag on someone else's program.
  The native/escalated arms spawn no herding process, so no cap check
  runs on them.
- **D2 — `--cell-id "<id>"`, quoted, only when `kind == "cell"`.**
  Quoted like its `--agent`/`--seat` neighbours. Non-cell kinds keep
  their command byte-identical.
- **D3 — placed after the `--agent` block, before `--no-pane`.** It is
  cell identity, not runtime tuning, and it must not depend on runtime:
  the cap check fires on every runtime.

## What will be done

Append `--cell-id "<id>"` to the herding command for cell dispatches,
update the byte-exact cell row of
`pi_native_flag_is_never_added_on_claude_or_codex_runtimes`, and add a
test proving the flag rides the cell command and no other kind.
