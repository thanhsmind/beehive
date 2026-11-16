# Cross-project work

Read when: work belongs to another project.

<!-- rule: agents-other-project-to-its-leader -->
- Work that belongs to another project goes to that project's leader
  through waggledance. Use the waggledance CLI first. Run
  `waggledance state <project-id>`. Then run
  `waggledance dispatch --project <project-id> --leader --request-id <id>`
  and give the task through `--task`, `--task-file` or stdin. Add
  `--follow-up-of <run>` when a `leaders[].runs` row of the state output
  shows that its leader already runs the same work. Add `--handoff` for
  a handoff. `<project-id>` is the `id` field of `waggledance list
  --json`. It is never a path. A task file stays inside this project.
  The output is JSON.
  The MCP tools are the fallback: `waggledance_ask_state`, then
  `waggledance_dispatch` with `leader: true` (`follow_up_of`,
  `handoff: true`) and the same `request_id`. Take the fallback when the
  `waggledance` binary is not on PATH, when it does not know the verb
  (`unrecognized subcommand`, exit 2), when a guard or permission
  refuses the shell call, or when the call fails with a tool error. The
  shared request id means a fallback never starts a second run. With
  `--task-file`, the fallback passes the file text as `task`.
  Never read, search or explore that project's code from this session:
  the write-guard refuses it and names the project root. For a research
  read of another repo, the user adds its path to
  `guards.cross_project_allow`.
<!-- /rule -->
