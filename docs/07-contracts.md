# 07 — Implementation Contracts (v0.1)

Normative contracts every v0.1 component codes against. When this doc and another doc disagree, this doc wins for implementation details.

## Ground rules

- Node 18+, ESM (`.mjs`), **zero npm dependencies**. Windows-safe paths (use `node:path`).
- Atomic writes everywhere: write `<file>.tmp`, then `renameSync`.
- All scripts fail safe: helpers exit non-zero with a one-line JSON error on `--json`; hooks NEVER break a session (wrap everything, log to `.bee/logs/hooks.jsonl`, exit 0 unless deliberately blocking).
- Source of truth for vendored code: `packages/bee/` in the plugin. Onboarding copies it to `<repo>/.bee/bin/` (helpers) and `<repo>/.bee/bin/lib/` (modules).
- Hooks live in the plugin (`packages/bee/hooks/`). At runtime they resolve the target repo root from CWD and dynamic-import lib from `<root>/.bee/bin/lib/`. If root or lib is missing → exit 0 silently (self-arm rule).

## Refusal messages: ERROR / WHY / FIX (docs/09 item 5)

bee refuses a lot by design — the message decides whether a refusal teaches in one
correction or causes retry thrash. Every user-facing refusal from `bin/lib/` or a hook
carries three elements:

1. **ERROR** — the rule that fired, named (`capCell: cell "x-1" has no passing verify result`).
2. **WHY** — the reason in the same sentence (`— an assertion is not evidence`).
3. **FIX** — the next command or action, concrete (`run … and record it: bee cells verify --id x-1 --command CMD --passed true`).

A refusal that ends at "not allowed" with no stated next step violates this contract.
Guard denials follow the same shape (`reason` names the gate/conflict, why, and the route:
surface Gate 2, reserve first, or return `[BLOCKED]`). Tests assert the FIX element (the
stated next action) for the three highest-traffic paths: cap-refusal, gate-block,
reservation-conflict.

## Runtime files

```
.bee/onboarding.json   { schema_version, bee_version, managed: {agents_block, helpers, lib}, created_at, updated_at }
.bee/state.json        see below
.bee/config.json       { hooks: {"session-init":true,"prompt-context":true,"write-guard":true,"state-sync":true,"chain-nudge":true,"session-close":true}, lanes:{}, capabilities:{} }
.bee/HANDOFF.json      { phase, feature, mode, cells_in_flight:[], done:[], remaining:[], next_action, written_at }
.bee/reservations.json { reservations: [ {agent, cell, path, ttl_seconds, reserved_at, released_at|null} ] }
.bee/decisions.jsonl   append-only decision events
.bee/backlog.jsonl     friction/grooming items
.bee/cells/<id>.json   one cell per file
.bee/logs/hooks.jsonl  hook crash/audit log
.bee/.inject-cache.json injection dedup state
.bee/manifest-hash.json { hash, checked_at } — bee's persisted sha256 of {schema_version, COMMAND_REGISTRY}, for cross-invocation drift detection; gitignored, rewritten on every bee invocation
```

`state.json` default:

```json
{
  "schema_version": "1.0",
  "phase": "idle",
  "feature": null,
  "mode": null,
  "approved_gates": { "context": false, "shape": false, "execution": false, "review": false },
  "workers": [],
  "summary": "",
  "next_action": "Invoke bee-hive."
}
```

Cell schema: as in [02-architecture.md](02-architecture.md) (id, feature, title, lane, status, deps, decisions, files, read_first, action, must_haves, verify, trace{worker, outcome, files_changed, deviations, friction, capped_at, behavior_change, verification_evidence, verify_output}).

## lib API (`packages/bee/lib/`)

All functions are sync unless noted. `root` = absolute repo root path.

### `fsutil.mjs`
- `readJson(file, fallback=null)`, `writeJsonAtomic(file, obj)`, `appendJsonl(file, obj)`, `readJsonl(file)` → array, `ensureDir(dir)`.

### `state.mjs`
- `findRepoRoot(startDir)` → path|null — walk up looking for `.bee/onboarding.json`, else first `.git`, else null.
- `defaultState()`, `readState(root)` (returns default when missing), `writeState(root, state)`.
- `gateApproved(state, gateName)` → boolean.
- `readHandoff(root)` → object|null, `readOnboarding(root)` → object|null, `readConfig(root)` → object (default hooks all true).
- `hookEnabled(root, name)` → boolean.

### `cells.mjs`
- `cellsDir(root)`, `listCells(root, {feature=null, status=null})` → array sorted by id.
- `readCell(root, id)` → cell|null, `writeCell(root, cell)`.
- `addCell(root, cell)` — validates: id, feature, title, lane ∈ {tiny,small,standard,high-risk,spike}, action, verify. `standard|high-risk` also require non-empty `must_haves.truths`. Throws `Error` with clear message.
- `readyCells(root, feature=null)` → open cells whose deps are all capped.
- `claimCell(root, id, worker)` — throws if: gate `execution` not approved, cell missing, status ≠ open, deps not all capped. Sets status=claimed, trace.worker.
- `recordVerify(root, id, {command, output, passed})` — stores on cell trace (`verify_output`, `verify_passed`).
- `capCell(root, id, {files_changed=[], deviations=[], friction=null, behavior_change=false, verification_evidence=null, outcome})` — throws unless `trace.verify_passed === true`; throws if `behavior_change` true and `verification_evidence` missing; for lane high-risk, requires files_changed non-empty and outcome. Sets status=capped, capped_at ISO.
- `blockCell(root, id, reason)`, `dropCell(root, id, reason)`.

### `reservations.mjs`
- `listReservations(root, {activeOnly=false})` — active = released_at null and not TTL-expired.
- `findConflicts(root, agent, paths)` → array of conflicting reservations held by *other* agents covering any path (path match: exact, prefix directory, or trivial `*` glob suffix).
- `reserve(root, {agent, cell, path, ttl=3600})` → `{ok:true}` or `{ok:false, conflicts}`.
- `release(root, {agent, cell=null})` — marks released_at.
- `sweepExpired(root)` → count released.

### `guards.mjs`
- `SECRET_PATTERNS`: regexes for `.env`(+suffixes), `*.pem`, `*.key`, `id_rsa*`, `*.p12`, `credentials*`, `secrets.*`.
- `SCOUT_DIRS`: `node_modules/`, `dist/`, `build/`, `.git/objects`, `vendor/`, `coverage/`, `.next/`, `__pycache__/`.
- `GATE_ALLOWED_PREFIXES`: `.bee/`, `docs/` (covers `docs/history/`), `.spikes/`, `plans/`, `AGENTS.md`.
- `checkWrite(root, state, relPath, agentName=null)` → `{allow:true}` or `{allow:false, kind:'intake'|'gate'|'reservation', reason}`.
  - intake (v0.1.1, repository-harness lesson): when `state.phase` is `idle` AND path not under GATE_ALLOWED_PREFIXES → deny with `kind:'intake'` pointing at bee-hive routing. Default-on; disable per repo via `config.guards.idle_gate: false`. This closes the "first ad-hoc edit slips through before any workflow starts" hole.
  - gate: block only when `state.phase` ∈ {`exploring`,`planning`} (`GATED_PHASES` — validation-diet D3 retired the standalone `validating` phase; the standalone execution gate folded into Gate 2, both carried by `planning`) AND path not under GATE_ALLOWED_PREFIXES AND `approved_gates.execution` is false. During `swarming`: reservation check via `findConflicts` when `agentName`/`BEE_AGENT_NAME` provided; unreserved-but-conflicting → deny.
- `checkRead(relPath)` → `{allow:true}` or `{allow:false, kind:'privacy'|'scout', reason, marker}` where privacy marker = `@@BEE_PRIVACY@@{json}@@END@@` containing `{file, question}`.
- `extractBashTargets(command)` → `{paths:[], broadWrite:boolean}` (khuym patterns: `sed -i`, `tee`, `rm`, `mv`, `cp`, `mkdir`, `touch`, `git add|mv|rm`, redirection `>`).

### Session preamble (`hooks/session_preamble.rs`, ported from `inject.mjs`)
- `buildSessionPreamble(root)` → markdown string: bee version + onboarding health; phase/mode/feature; gate states; HANDOFF block ("present it and WAIT — never auto-resume") when present; up to 10-line digest of `docs/history/learnings/critical-patterns.md`; last 3 active decisions (datamarked); when `docs/specs/` exists, one state-layer line ("Area specs + reading map at `docs/specs/` — read the touched area's spec before its code"); "Run `.bee/bin/bee status --json` for detail. Route via bee-hive." This is the **startup-shaped** orientation, and it is what `startup`/`clear`/`resume` render, byte-identical. **R6 note:** this line used to read `node .bee/bin/bee status --json` — the last agent-facing Node spelling, pinned by the byte-parity contract for as long as the Node engine existed. The cutover renders the preamble natively (`hooks/session_preamble.rs`) and the divergence is deliberate: `no_mjs_spelling_survives_anywhere_in_the_preamble` fails the build if any `.mjs` spelling comes back.
- `buildCompactCapsule(root, {sessionId, handoffOutcome})` → markdown string: the **compact-scoped** orientation that replaces the preamble body when `SessionStart` fires with `source=compact`. Narrower by design — the state mismatch line, onboarding-MISSING, the HANDOFF block (including its `- Adoption not applied: <reason>` line, which is why `handoffOutcome` is a required argument at the call site), the gate-bypass banner, phase/mode/feature/lane, the claimed cell with its `verify` command and dependency status, the first open gate, `next_action`, the recorded standard commands, the compaction survival count and its warning, and a one-line pointer to where the critical patterns live. The intent anchor is **not** rendered here: the hook prefixes it, exactly as on `resume`.
- `buildPromptReminder(root)` → `{text, hash}` — 1–3 lines: phase / mode / next_action / first open gate. `hash` = stable hash of those fields.
- `shouldInject(root, key, hash)` / `markInjected(root, key, hash)` — via `.bee/.inject-cache.json`; inject when hash differs from last or >30 min elapsed.

### `decisions.mjs`
- `logDecision(root, {decision, rationale, alternatives=null, scope='repo', source='user', confidence=null})` → event with uuid + ISO date; **rejects** (throws) content matching secret patterns or instruction-injection heuristics (e.g., `ignore previous`, role tags `<system>`).
- `supersedeDecision(root, {supersedes, decision, rationale})`, `redactDecision(root, {redacts, reason})`.
- `activeDecisions(root, {recent=null})` — decide events not superseded/redacted, newest first.
- `datamark(text)` — strip/neutralize backticks fences, role tags, control chars; wrap in `«…»`.

### `command-registry.mjs` (harness-integration-adopt, decision 30606de4; sole registry since D1/D5, shim-retire)
- `SCHEMA_VERSION` — manifest schema version string (currently `'1.0'`).
- `COMMAND_REGISTRY` — array of entries, one per subcommand across all 9 command groups (status, cells — including `cells.update` —, reservations, decisions, state, backlog, capture, reviews, feedback): `{name, invoke, description, parameters, examples, deprecated}`. `parameters` is JSON-Schema in the exact shape Claude Code's own tool definitions use (`{type:"object", properties, required}`). The registry's old `helper` field (which of the legacy per-group scripts implemented a command) was removed together with its strip code and test assertion (D5) — every entry now dispatches straight to `bee`. `examples[]` are literal, tested argument strings (`tests/test_bee_cli.mjs` runs every one against the real dispatcher).

### `validate-args.mjs` (harness-integration-adopt, decision 30606de4)
- `isValidParameterSchema(schema)` → boolean — structural check that a `parameters` value is well-formed JSON-Schema in the registry's shape (object type, every `required` name present in `properties`, every property carrying a `type`).
- `validate(commandEntry, parsedArgs={})` → `{ok:true}` or `{ok:false, error:{field, reason, command}}` — never throws. Checks every schema-required field is present, then that every present field's CLI-string value type-matches its schema `type` (CLI flags arrive as strings; a schema `type:"boolean"` accepts `"true"`/`"false"` as well as a native boolean). Shared by `bee` (dispatch-time validation) and `packages/bee/hooks/bee-write-guard.mjs`'s CLI-shape check (pre-execution validation of Bash calls) — one validator, two call sites, never duplicated.

## CLI surface (`the bee binary`)

`bee <group> <verb> [--flags]` is the sole shipped CLI (D1, shim-retire, decision bbc6bcea): one argv wrapper over `lib/`, every group supports `--json`, non-zero exit + `{error}` JSON on failure. It began (harness-integration-adopt Phase 1, decision 30606de4) as an *additive* dispatcher living alongside one legacy script per group; those per-group scripts are now deleted from `packages/bee/` and, on `--apply`, from every host's `.bee/bin/` too (D2's `RETIRED_HELPERS` removal pass) — `bee` imports the same `lib/*.mjs` functions they used to.

```
bee status [--json]
  → { onboarding, phase, mode, feature, gates, handoff, cells:{open,claimed,capped,blocked}, pbi:{proposed,in_flight,done}|null, active_reservations, critical_patterns_present, recent_decisions, staleness_warnings, recommended_next }
    (pbi added additively, harness10 D10: counts of docs/backlog.md Status column, null when the file is absent)

bee cells list [--feature F] [--status S] | ready [--feature F] | show --id ID
             | add --stdin                     (one cell object or a whole-slice JSON array; --file cell.json also accepted)
             | claim --id ID --worker NAME
             | verify --id ID --command CMD --passed true|false [--output TEXT | --output-file F]
             | cap --id ID [--outcome TEXT] [--files a,b] [--behavior-change] [--evidence-file F] [--deviations-file F] [--friction TEXT]
               (cap refuses for small/standard/high-risk lanes when the recorded verify has no output and no evidence, or when --files is empty — decision 0004)
             | block --id ID --reason R | drop --id ID --reason R

bee reservations reserve --agent A --cell C --path P [--ttl N]
                    | release --agent A [--cell C]
                    | list [--active-only] | sweep

bee decisions log --decision D --rationale R --relation supersedes:<id>|touches:<id>|none [--trigger T] [--alternatives A] [--scope S] [--confidence N]
                 | supersede --id UUID --decision D --rationale R
                 | redact --id UUID --reason R
                 | active [--recent N] | search --text T

bee state set --owner <selected pre-mutation phase> | gate | worker add|update|remove|clear|prune | scribing-run | start-feature | handoff show|write|adopt

bee backlog add | counts | rank | badges

bee capture add | list | flush | count

bee reviews create | list | show | record | candidate add | candidates | status

bee feedback digest [--out PATH] [--json]     (P18, evolving loop; decision 8cd4c84e / D2)
                 | count [--json]
                 | collect [--json]
                 | rank [--json]
  digest   → builds and writes the LOCAL allowlist digest (buildDigest) to .bee/feedback-digest.json
             (or --out PATH); schema_version + counts + dropped[] + entries[] (allowlist fields only,
             see "evolving contract" below)
  count    → same digest, prints counts only (no write)
  collect  → the MERGED view: buildDigest(root) folded with every configured dogfood_repos digest via
             mergeDigests (revalidates + datamarks every foreign field — D2b); absent/unreachable
             dogfood repos are skipped with a warning, never thrown
  rank     → clusterEntries + rankClusters over the SAME merged view collect returns; no business
             logic in the CLI — see lib API below

--help [--json]
    .bee/bin/bee --help --json emits {schema_version, commands:[{name, invoke, description,
    parameters, examples, deprecated}]} — the same JSON-Schema tool-definition shape Claude Code's
    own tool/subagent surface uses; the registry carries no per-group dispatch field any more (D5
    removed it together with the legacy scripts).
    Manifest drift: a sha256 of {schema_version, COMMAND_REGISTRY} is persisted to
    .bee/manifest-hash.json (gitignored — rewritten on every invocation); when it differs from the
    prior persisted hash, a manifest_changed:true hint is written to stderr only — stdout's shape
    never changes.
    Unknown command → a Levenshtein nearest-match suggestion, never a bare not-found.
    Group/command-scoped help (GH #23): `bee <group> [verb] --help [--json]` emits the manifest
    filtered to that command or group prefix (exact name or `<name>.*`), exit 0 — `state --help`
    lists the 17 state.* entries instead of the old Unknown-command error. A token matching no
    entry and no group prefix keeps the fallback behavior above unchanged.
    Deferred: an MCP server wrapper and a mandatory every-session --help --json discovery call —
    foundation-add without demonstrated need.
```

## Evolving contract (P18, self-improvement loop — [handbook/evolving.md](handbook/evolving.md); decisions D1–D5, `8cd4c84e`, `ff26725d`)

Enforced invariants only — this section states no promise the code above does not already make:

- **The digest is an allowlist, not a redaction filter (`8cd4c84e`, supersedes `20784de8`).**
  `buildDigest` writes only `{kind, layer, source, title, first_seen, pain}` per entry plus
  `dropped[]`; there is no `detail`/`text`/`outcome`/`deviations` field to leak because none is ever
  read into the digest object.
- **The consumer revalidates every foreign field (D2b).** `mergeDigests` re-runs the secret and
  injection pattern scans against each configured `dogfood_repos` digest and wraps every surviving
  foreign `title` in `datamark()` before it is returned — a hand-edited or hostile foreign digest is
  never trusted as-is. `bee feedback rank`/`collect` only ever consume `mergeDigests`'s output,
  never a foreign digest file directly.
- **Bee-repo-only (D3).** [handbook/evolving.md](handbook/evolving.md) step 0 is a hard guard
  (`test -f packages/bee/lib/feedback.mjs && test -f docs/handbook/writing-skills.md`)
  that refuses to proceed anywhere the bee-repo-only files are absent; pressure-tested RED-first
  under the full Iron Law (decision `ff26725d`) — see
  `docs/history/evolving-loop/reports/evolving-10-pressure.md`.
- **Two human gates (D5).** The skill stops for an explicit human choice before any implementation
  (Gate A) and an explicit human approval of the complete diff before any push (Gate B); push is a
  named manual step, never automatic. Both gates are pressure-tested RED-first (same report).

## Hook contracts (`packages/bee/hooks/`)

`packages/bee/hooks/hooks.json` (Claude Code plugin hook config) wires:

| Script | Event / matcher | Behavior |
|---|---|---|
| `bee-session-init.mjs` | SessionStart `startup\|resume\|clear\|compact` | prefix the intent-anchor lead block on `compact`/`resume`, then print the orientation body to stdout — `buildSessionPreamble` on `startup\|clear\|resume`, `buildCompactCapsule` on `compact`; exit 0 |
| `bee-prompt-context.mjs` | UserPromptSubmit | `buildPromptReminder`; print only when `shouldInject(root,'prompt',hash)`; mark; exit 0 |
| `bee-write-guard.mjs` | PreToolUse `Edit\|Write\|MultiEdit\|Bash\|Read\|Glob\|Grep` | parse stdin payload (`tool_name`, `tool_input`); for reads → `checkRead`; for writes/Bash → `checkWrite` (+`extractBashTargets`); for Bash, an additive 4th check (harness-integration-adopt, decision 30606de4) recognizes a call shaped like a `bee`/`bee_*.mjs` invocation and validates it against `command-registry.mjs`'s schema via `validate-args.mjs` before the shell executes it — malformed calls deny with a structured correction, unrecognized shapes fail open, and this check can only ever *assign* a denial (never overwrite one the other three checks already set). Deny = **exit 2 with reason on stderr** (include marker text for privacy). Allow = exit 0 silent. |
| `bee-state-sync.mjs` | PostToolUse `TaskCreate\|TaskUpdate\|TodoWrite` + SubagentStop + Stop | refresh cell counts + last_activity into state.json; exit 0 |
| `bee-chain-nudge.mjs` | SubagentStop | if state.workers lists this agent or phase=swarming → print nudge ("collect [STATUS], update cell, check reservations; when wave clean → next step"); phase=reviewing → reviewer-synthesis nudge; else silent |
| `bee-session-close.mjs` | Stop | if phase not idle/compounding-complete and no HANDOFF → print warning listing claimed-uncapped cells + active reservations. If phase IS idle: decision-review nudge (v0.1.1) — when `git status` shows changed source files and no decision was logged in the last 6h, print a deduped (`shouldInject`) reminder to ask the user about recording a decision/learning. Never blocks; exit 0 |

Common prologue for every hook: read stdin fully (may be empty), `findRepoRoot(cwd)`, require `.bee/onboarding.json` + `hookEnabled(root, '<name>')`, dynamic-import lib from `<root>/.bee/bin/lib/` with try/catch → exit 0 on any miss; crash-log to `.bee/logs/hooks.jsonl`.

## Onboarding (`packages/bee/scripts/bee onboard`)

```
.bee/bin/bee onboard --repo-root <path> [--apply] [--json] [--repo-hooks] [--claude-md]
```

1. Verify Node ≥18. 2. Compute plan: AGENTS.md BEE block (insert or update between `<!-- BEE:START -->` / `<!-- BEE:END -->`, content from `packages/bee/AGENTS.block.md` — do NOT touch anything outside markers); create `.bee/` runtime files if missing (never overwrite existing state/decisions/cells); copy `packages/bee/*.mjs` + `packages/bee/lib/*` → `.bee/bin/`; create `docs/history/learnings/critical-patterns.md` stub if missing. 3. Without `--apply` → report `{status: 'up_to_date'|'changes_needed', plan:[...]}`. With `--apply` → apply + write `.bee/onboarding.json` with managed versions. `--repo-hooks` additionally merges hook entries into `<repo>/.claude/settings.json` (backup first). `--claude-md` writes/extends `CLAUDE.md` with a bare `@AGENTS.md` import (harness pattern: auto-loads the BEE block on Claude Code when plugin hooks are unavailable); never duplicates the import, never rewrites existing user content.

`BEE_VERSION = '0.1.0'` exported from `lib/state.mjs`; onboarding compares for drift.

## Skill conventions (v0.1)

- Frontmatter: `name` (bare hyphen-case = dir name), `description` (purpose clause for the slash menu + "Use when …" trigger conditions; never workflow steps), `metadata.version: '0.1'`, `metadata.ecosystem: bee`, `metadata.dependencies` mapping or `[]`.
- Body < 200 lines; one `references/` level; end with handoff sentence `[Outcome]. Invoke bee-<next> skill.`
- Every skill documents `mode:headless` behavior in one short section.
- Commands quoted in skills MUST match the CLI surface above verbatim.
- Each skill keeps a creation log in `docs/decisions/skills/<skill>-creation-log.md`: provenance (which upstream skill it adapts), what changed, and an honest `Pressure testing: PENDING (scheduled per Iron Law before 1.0)` note with the 3 scenarios from 04-skills-spec listed as the planned RED set.

### OpenAI skill metadata projection

`skills/bee-*/SKILL.md` frontmatter is the canonical identity and trigger-description source for both runtimes. `scripts/render_openai_metadata.mjs` deterministically projects each live bee skill to `skills/bee-*/agents/openai.yaml`: `interface.display_name` comes from the hyphenated `name`, `interface.short_description` comes from the folded `description`, and `policy.allow_implicit_invocation` is always `true`. Independent descriptions, default prompts, and workflow prose in the projection are prohibited.

Run the renderer without arguments to regenerate every projection and with `--check` to verify the tree. Check mode exits non-zero and names the first class of drift as `MISSING`, `STALE`, `ORPHAN`, or `MALFORMED`; the canonical library suite invokes this check so absent and stale projections cannot pass repository verification.

No runtime-specific installer is added. The plugin already packages the shared `skills/` tree, and onboarding's existing whole-skill manifest comparison emits `sync_skill` whenever any nested byte differs; its deep-mirror apply therefore distributes `agents/openai.yaml`, removes stale nested files, and verifies byte parity through the same path as every other skill file.
