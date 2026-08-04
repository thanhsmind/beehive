# External CLI executors — the cell-execution protocol

The reserve/verify/cap/release contract for running a bee CELL on an external CLI
worker. It is not dispatched today: `resolveTier` refuses a cli-shaped tier for any
purpose except `{for:'gather'}`, so nothing in bee reaches this protocol, and no hook
or test checks a line of it. It lived in `skills/bee-swarming/references/swarming-reference.md`
until the doctrine diet moved it here — an orchestrator was loading a protocol it is
forbidden to run. Every word is preserved. Read it when the cli cell path is enabled;
the gather branch that IS live stays in the reference.

**Dispatch protocol** (`resolveTier(root, slot, runtime, {for:'gather'}).type === 'cli'`):

1. **Prompt file, never shell-quoted args:** write the standard worker prompt (Worker Prompt Template below, verbatim — same contract, same status tokens) **plus the cli-dispatch suffix from step 2** to `.bee/workers/<cell-id>.prompt.md`. The external worker starts with ZERO session context — the prompt carries goal, exact paths, constraints, non-goals, and the proof expected (the declared tests green at finish); spec quality decides success. The prompt file **is the contract**, at a stable path: it outlives the process, the worker re-reads it if it loses the thread, and rescue rounds reference it (`re-read .bee/workers/<cell-id>.prompt.md`) instead of re-pasting the spec. If dispatch ever runs in an isolated worktree, surface the same contract as a short block in that workspace's AGENTS.md — the one file external CLIs reliably read first.
2. **Finish contract — the cli-dispatch suffix**, appended verbatim to the template:

   ```text
   Cli dispatch extras:
   - This contract lives at .bee/workers/<CELL_ID>.prompt.md — re-read it if you lose the thread.
   - Your last FILE act, after capping and releasing but BEFORE returning the
     final status-token message: write .bee/workers/<CELL_ID>.result.json:
     { "cell_id": "<CELL_ID>", "outcome": "done|blocked|handoff|noop",
       "verify_command": "<the declared test command>", "verify_passed": true|false,
       "files_changed": ["<paths>"], "notes": "<one line>" }
   ```

   The outcome vocabulary is exactly the four status tokens — `result.json` is the cli **transport** of the same worker contract as the native markdown results, never a second contract. Exiting is not signaling; a worker that only exits has not finished.
3. **Spawn detached, output to files:** before launching — first dispatch or any resume round — delete any existing `.bee/workers/<cell-id>.result.json`; a stale result must never satisfy a later attempt. Run the configured command as a background process, prompt via stdin, final message to a dedicated file where the CLI supports it (codex: `-o .bee/workers/<cell-id>.result.md`), raw stream to a job log with stderr suppressed — thinking noise bloats the orchestrator's context; re-enable stderr only to debug a failing run. E.g. `<command> -o .bee/workers/<id>.result.md - < .bee/workers/<id>.prompt.md > .bee/workers/<id>.out.log 2>/dev/null`. Keep the launcher's job handle — its exit event is the "process ended" signal step 5 waits on. Record the worker (nickname, cell, `executor: cli`) in `.bee/state.json` as usual.
4. **Tend by artifact, not by chat:** the external worker runs the same `.bee/bin/bee` binary (reserve → finish) — one self-contained executable, no runtime to install — the cell status and reservations ARE the progress signal. Poll `.bee/bin/bee cells show --id <id>` and read `.bee/workers/<cell-id>.result.json` for the final outcome; never parse the raw JSONL stream. A quiet run is not a dead run — do not kill on silence alone.
5. **Accept by file, never by exit:** once the process ends, a cli run counts only if `result.json` exists, parses, and carries a valid outcome. Missing, unparseable, or invalid-outcome result = a failed run, routed to rescue (step 7) — never accepted, never silently waited on.
6. **Trust boundary — never on its word:** an external worker's `done` is never accepted on its word — the orchestrator ALWAYS re-runs the declared tests itself (`bee test`) and runs `bee cells judge --id <id>`. External executors never get the tiny/small spot-check relaxation; every external cell is goal-checked. The result file is a signal, never the evidence. On `standard`/`high-risk` `behavior_change` cells, the same semantic checklist judge from the tier table in `bee-hive/references/gates-and-delegation.md` ("Goal-check judge tier") applies here too — verification, not the on-demand review session.
7. **Rescue — resume before re-dispatch:** on a goal-check miss or a failed acceptance (step 5), prefer the CLI's session-resume (codex: `codex exec resume --last`, run from the repo dir; resume inherits the original session's sandbox/config — do not re-pass sandbox flags) with a short prompt carrying the diagnostic that applies — the failing verify output for a goal-check miss, or the acceptance failure (missing/unparseable/invalid `result.json`) for a step-5 reject — plus the contract path. It keeps the worker's context and costs far less than a fresh run. **After 2 failed resume rounds, stop ping-ponging:** mark `[BLOCKED]` and climb the normal rescue ladder (a stuck/garbled run is killed and re-dispatched; the tier rung may swap `cli` for a native model tier when the provider itself is failing).

Constraints: the external CLI must be able to edit the repo working tree and run node (the `.bee/bin` contract); grant write access scoped to the repo only (codex: `-s workspace-write`) — never a machine-wide bypass (`--yolo`-style flags) as the house default; the goal-check exists so bee does not have to *trust* the worker, not so it can hand over the machine. Secrets: the external process gets only its own provider's credentials from the user's environment — bee passes none.
