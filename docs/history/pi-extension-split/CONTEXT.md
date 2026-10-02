# Context: pi-extension-split

## Ask

The Pi guard extension is one large file, `.pi/extensions/bee-guard.ts`
(3767 lines). The user wants it split into small files, the way other Pi
extensions are laid out (for example `pi-workflows/src/extension/*.ts`),
so it is easier to extend and to follow.

## Locked decisions

- **D1** (store `19afb54e`): The Pi guard ships as a directory extension
  `.pi/extensions/bee-guard/` whose entry is `index.ts`, split into one
  module per concern. Relative imports carry the `.ts` extension, so node
  and Pi load it with no build step. Behavior does not change, and
  `index.ts` re-exports every name the single file exported.
- **D2** (store `75e947e9`): Module-level mutable state lives in one
  `state.ts` object. No module assigns another module's `let` binding.
  Maps and Sets stay exported consts.
- **D3** (store `0c06dba7`): `bee onboard` vendors every file under
  `.pi/extensions/bee-guard/`, deletes a legacy `.pi/extensions/bee-guard.ts`,
  and prunes files inside `bee-guard/` that bee does not ship.
- **D4** (store `8822545c`): `bee doctor --runtime pi` compares every file
  of the embedded `bee-guard` directory byte for byte. `build.rs` generates
  the file list from the directory itself. Doctor fails when the legacy
  single file is present.
- **D5** (store `b8635d0e`): The Pi and OpenCode contract tests derive their
  source facts from the concatenated `bee-guard` module sources and run
  `index.ts` under node. Existing comments move verbatim with their code.

- **D6** (store `8cdd7725`, from the plan-step hat wave): the folder is
  flat, with no subfolders. Every `pi.on` handler stays in one `events.ts`
  in its original order. Every `register*` function takes the parameter
  named `pi`. The shipped set is every `.ts` file in the folder, sorted by
  name. Onboard plans `remove_pi_extension` for the legacy file before any
  copy, and only when the source folder ships files. Prune removes only
  `.ts` files bee does not ship and never follows symlinks. The remove arm
  accepts only `.pi/extensions/bee-guard/*.ts` or the exact legacy path.
  Doctor failure text names `bee onboard --apply`.

## Out of scope

- The OpenCode plugin `.opencode/plugins/bee-guard.ts` keeps its one-file
  shape.
- No guard behavior changes.
