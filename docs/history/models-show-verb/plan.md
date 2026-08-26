# models-show-verb — plan

Three cells, one slice. Cites CONTEXT.md D1 (raw-table verb), D2
(read-before-assign reminders), D3 (seeded descriptions), D4 (status keeps
descriptions), D5 (resolution/guard/dispatch untouched).

## ms-1 — the verb (code)

`bee models show [--runtime claude|codex|opencode] --json`: read-only, prints
the RAW `models.<runtime>` table(s) from `.bee/config.json` — every slot
shape verbatim, `description` intact — plus the runtime's built-in defaults
for roles the config does not name, each marked by its source
(`configured` | `default`). Pattern: the `work show` verb end to end —
catalog entry (new `models` group), router wiring, handler module, help
text, tests. Whatever regenerates `generated/registry_payload.json` runs
inside the cell.

## ms-2 — status keeps descriptions (code, independent files)

`status_full/build.rs`: the `models` section merges each raw slot's
`description` back onto the normalized slot for display; internal resolution
keeps the stripped map. Test: a described slot shows its description in
`bee status --json`, an undescribed one is byte-identical to today.

## ms-3 — seeds + reminders (code, deps: ms-1, shares catalog.rs)

- `onboard/templates.rs` `default_config`: the four seeded claude roles
  become `{model, description}` objects; codex stays null. Existing seed
  tests updated.
- The missing-role refusal in cells validation and the `cells add` help
  sentence name the verb: run `bee models show` before assigning a role if
  the table has not been read this session.

## Verify

Per cell: the touched suite filtered (`onboard`, `status_full`, `cells`,
router/catalog tests), new tests named in the run output; `bee dev regen`
clean at the end.

## Cost if wrong

A read surface misprints or a help text misleads; no dispatch-path change.
