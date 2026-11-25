# Config layers and split-team

bee reads one merged config from up to four files plus its built-in defaults. From lowest to
highest: the user-global file, tracked `.bee/config.json`, tracked `.bee/team.json`, and the
per-machine `.bee/config.local.json`; the highest file that sets a key wins. The global file is
`$BEEHIVE_CONFIG_DIR/config.json`, else `$XDG_CONFIG_HOME/beehive/config.json`, else
`~/.config/beehive/config.json`. bee reads it and never writes it, and it drops `gate_bypass`,
`hooks` and `guards` from it at read time. `.bee/team.json` holds the team table: `team`,
`herding.agents` and `herding.agent_command`. A first onboard seeds the team table into
`team.json`, minus any part the global file already sets. An older project still holding the
table in `config.json` gets a notice, and `bee config split-team` moves the table with the
merged view unchanged.

## Sub-features

- `layers-global-read` shows a global-only key in `config explain` with `source: "global"`.
- `layers-project-wins` keeps a project value over the global one for the same key, with the
  global value still listed.
- `layers-global-forbidden` reports `gate_bypass`, `hooks` and `guards` in the global file as
  `global-forbidden-key` problems and keeps all three out of the merged view.
- `layers-global-corrupt` names the global file when it is not JSON: config verbs refuse,
  every other verb warns on stderr and reads the layer as empty.
- `layers-onboard-global-team` skips seeding `team` when the global file sets it, and
  `team show` lists the global slots as `configured`.
- `split-team-notice` puts a notice in an onboard of a project whose `config.json` holds the
  table, and moves nothing.
- `split-team-move` moves the table into `team.json`. `team show --json`, the herding agent
  command and `validate` stay identical. A second run reports `moved: []`.
- `split-team-drop-global` drops every project team leaf equal to the global value. `team show`
  stays identical and a second run drops nothing.
- `layers-guard` refuses an agent write to the global file, a Bash command naming it or
  assigning `BEEHIVE_CONFIG_DIR` / `XDG_CONFIG_HOME`, and a governed change in `team.json`.

## How to get to it (user POV)

- Put a key in `~/.config/beehive/config.json` by hand (or through waggledance). Run
  `bee config explain --key <key>` in any onboarded project to see which layer the value
  comes from.
- Run `bee config validate` to list problems in all four files and the keys the global file
  supplies (`global_supplied`).
- Run `bee config get --key <key> --global` to read the global file alone.
- Run `bee onboard --apply` on a fresh repo to get `config.json` plus `team.json`. On an older
  project the run prints the split-team notice.
- Run `bee config split-team` to move the table, or `bee config split-team --drop-global` to
  move it and drop each leaf the global file already holds.
- Run `bee team show --json` to read the effective team table.

## Driving it with control-bee

Preconditions:

- `control-bee` isolates the user-global config by default (every bee process gets
  `BEEHIVE_CONFIG_DIR` set to an empty `<run dir>/global-config`). Exporting
  `BEEHIVE_CONFIG_DIR` to a scenario dir under `$VERIFY_HOME` BEFORE running
  `control-bee` overrides that default and passes it through to every bee process.
- Keep one global dir per scenario, e.g. `$VERIFY_HOME/g1/config.json`. Write each file as the
  harness (`control-bee put` only writes inside the sandbox). Point `BEEHIVE_CONFIG_DIR` at the
  scenario's dir for the calls that drive it.
- A launched sandbox, `control-bee doctor` fully `ok`. With an empty global dir, the launch
  leaves `.bee/team.json` holding `team`, `herding.agents` and `herding.agent_command`, and
  `.bee/config.json` holding none of them.

- **Global value, project wins.** Global file
  `{"from_global_only":"g","worktree_first":"off"}`. Run
  `control-bee cli -- config explain --key from_global_only --json`. Expect `source: "global"`
  and `effective: "g"`. Then run `control-bee cli -- config explain --key worktree_first --json`.
  Expect `source: "tracked"`, `effective: "on"`, `global: "off"`. Then
  `control-bee cli -- config validate --json` lists `from_global_only` under `global_supplied`.
- **Forbidden global keys.** Global file
  `{"gate_bypass":"total","hooks":{"write-guard":false},"guards":{"cross_project":false},"fine":1}`.
  `control-bee cli -- config validate --json` exits 1 with three `global-forbidden-key`
  problems, each with `file` set to the global path and `key` set to `guards`, `gate_bypass` or
  `hooks`. `config explain --key gate_bypass --json` shows `source: "tracked"` and
  `global: null`. `--key guards` and `--key hooks.brand-new-hook` show `source: "unset"`.
  `--key fine` shows `source: "global"`.
- **Corrupt global file.** Global file `{not json`.
  `control-bee cli -- config explain --key worktree_first --json` exits 1 with an `error`
  naming the global path. `config validate --json` exits 1 with `config-unparseable` and
  `file` set to the global path. `control-bee cli -- status --json` exits 0 and its `.err`
  holds `the global config file <path> is unreadable and was skipped`.
- **First onboard under a global team.** Global file
  `{"team":{"claude":{"code":{"model":"opus","description":"global code slot"}}}}`. Run
  `control-bee newrepo target`, then
  `control-bee host -- <binary> onboard --repo-root <target> --apply --json`. `applied` lists
  `.bee/team.json`. That file holds only `herding` (`agent_command`, `agents`), and
  `.bee/config.json` holds no `team`. Then
  `VERIFY_CWD=target control-bee cli -- team show --runtime claude --json` shows `code` with
  `model: "opus"`, `description: "global code slot"`, `source: "configured"`.
- **Legacy project and the notice.** A bee older than this feature kept the table in
  `config.json`, and the current binary cannot write that shape. Fold the sandbox's
  `team.json` into `config.json` with one script (keep `herding.control_command`), delete
  `team.json`, and commit the result. This is the one hand-written state in the recipe. Then
  `control-bee host -- <binary> onboard --repo-root <sandbox> --json` reports `up_to_date`. Its
  `notices` holds one line that names `bee config split-team`, says to commit
  `.bee/config.json` and `.bee/team.json` together, and says onboarding never moves the table.
  The same run with `--apply` leaves `config.json` holding `team` and creates no `team.json`.
- **Split and idempotence.** Save `control-bee cli -- team show --json` and
  `control-bee cli -- config explain --key herding.agent_command --json`. Run
  `control-bee cli -- config split-team --json`: `moved` is
  `["team","herding.agents","herding.agent_command"]` and `git status` shows `config.json`
  modified and `team.json` new. `team show --json` is byte-identical to the saved one, and so
  are the `effective` values of `herding.agent_command` and `herding.agents.<that name>`.
  Commit, then run `config split-team --json` again: `moved: []`, exit 0, clean `git status`.
- **Drop-global.** Copy the sandbox's `.bee/team.json` to a new global dir as
  `config.json` and point `BEEHIVE_CONFIG_DIR` at it. `team show --json` stays byte-identical.
  `control-bee cli -- config split-team --drop-global --json` lists every team leaf under
  `dropped`, leaves `.bee/team.json` as `{}` and `team show --json` byte-identical.
  `config explain --key herding.agent_command` now shows `source: "global"` with the same
  `effective`. A second `--drop-global` returns `moved: []`, `dropped: []`. A re-onboard
  with `--apply` re-seeds nothing.
- **Guard.** Feed `control-bee cli -- hook write-guard` one payload on stdin per probe, as in
  [`comment-guard.md`](./comment-guard.md), with `BEE_HOOK_NO_DELEGATE=1`. A `Write`, `Edit`
  or `apply_patch` on `$BEEHIVE_CONFIG_DIR/config.json` exits 2 with `bee config guard: the
  global config file ... is written only by waggledance`. So does a `Bash` command naming that
  path, or one holding `BEEHIVE_CONFIG_DIR=` or `XDG_CONFIG_HOME=`. A `Write` to `.bee/team.json`
  that adds `herding.agent_command` over a global value exits 2 with
  `".bee/team.json" changes governed config key herding.agent_command`. Control: a `Write` to
  `notes.md` in an idle sandbox exits 2 from the intake gate, which proves the hook ran.

## Gotchas

- **control-bee isolates the global file by default.** It sets `BEEHIVE_CONFIG_DIR`
  to an empty `<run dir>/global-config` unless the caller already exported
  `BEEHIVE_CONFIG_DIR`, in which case the exported dir is passed through.
- **The guard refuses the env assignment.** In a hooked session, a Bash command holding
  `BEEHIVE_CONFIG_DIR=` is refused. When the session itself carries `BEEHIVE_CONFIG_DIR`, a
  `Write` to that dir's `config.json` is refused too. Set the variable in the environment that
  starts the driving session, and write the global files from outside the guarded tool calls
  (a harness script, or a session without the hook). Never route around the refusal inside a
  guarded session.
- **A herded pane skips the guard.** With `BEE_HERDING_WORKER=1` in the environment (a
  `bee herding run` pane), `bee hook write-guard` runs only the concurrent-tree subset. Every
  probe in the Guard step then exits 0, the intake-gate control included. Unset
  `BEE_HERDING_WORKER` and `BEE_HERDING_JOB_ID` on the hook call to drive the full guard.
- **`config get` is not the merged view.** Without flags it reads `.bee/config.json` alone, so
  a global-only or team-only key reads `present: false`. Use `config explain` for the merged
  value and the layer it comes from.
- **`global: null` on a forbidden key is the drop.** `explain` prints the global layer after
  the drop, so `gate_bypass` reads `null` there even though the file sets it.
  `config get --key gate_bypass --global` reads `present: false` too. Only `config validate`
  shows that the file sets it.
- **The corrupt-file warning repeats.** A verb that reads config several times prints the two
  warning lines once per read. Assert on the path, not on a count.
- **`--drop-global` keeps the file.** A fully dropped `team.json` stays on disk as `{}`.
- **The commit-together line is text only.** The human `split-team` output says to commit both
  files together. The `--json` payload carries `moved`, `dropped` and `drop_global` only.
- **A wave's agent command has no read-only verb.** `bee herding wave` spawns panes. Prove the
  command a wave resolves through the same merged read: the `effective` values of
  `herding.agent_command` and `herding.agents.<name>`, and, for an array-valued
  `agent_command`, `control-bee cli -- herding command-template agent_command --main-root <sandbox>`,
  which prints one token per line.
- **The global team is withheld key by key.** With only `team` in the global file, a first
  onboard still seeds `herding.agents` and `herding.agent_command` into `team.json`.
