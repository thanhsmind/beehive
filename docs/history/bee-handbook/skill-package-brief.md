# Packaging brief — the `bee-handbook` skill

**For a bee session running in the worktree
`/home/thanhsmind/projects/goglbe/beegog--wt--bee-handbook` (branch `wt/bee-handbook`).**
This checkout is isolated from the live `i54-closeout` session in main. Do the work
here, then merge back from main later.

## Goal

Package a new bee skill `bee-handbook` that **regenerates the navigable handbook of
the bee harness** under `docs/handbook/` from live source, so running it later
refreshes the handbook to the latest state. The handbook itself was already designed
in main (overview + index + register + stages/<9> + using-as-planner); this skill
codifies its regeneration. In this worktree `docs/handbook/` does not exist yet —
the skill's own loop will generate it, which doubles as the acceptance test.

Route it as a **small-lane skill authoring** task (0 risk flags): bee-hive →
bee-writing-skills. Under `gate_bypass=total` (carried in the copied config) gates
auto-approve. Follow the four steps below, then verify, cap, and close.

## Files to create (verbatim)

### 1. `skills/bee-handbook/SKILL.md`

````markdown
---
name: bee-handbook
description: Regenerate the navigable handbook of the bee harness under docs/handbook/ from the live source of truth. Use when the user asks to build, refresh, rebuild, or update the bee handbook, or when harness changes (a chain skill, a .bee/ record shape, the CLI surface) have made the handbook stale.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: degraded
      reason: Reads bee records via the vendored .bee/bin helpers and runs the handbook self-check.
---

# bee-handbook — regenerate the harness handbook

`docs/handbook/` is a **derived layer** over the live bee harness, in the
[Harness Handbook](https://github.com/Ruhan-Wang/Harness_Handbook) convention:
the **chain is the set of stages**, the **`.bee/` runtime files are the state
registers**. This skill re-derives it from source so running it refreshes the
handbook to the latest state. It never invents — every fact traces to live source.

Announce one line ("refreshing the bee handbook"), then run the loop. This is a
**docs-lane** regeneration: it writes only under `docs/handbook/`, which is inside
the write-guard allowlist, so it needs no cell and no gate. Close by logging the
refresh (decision/capture stub) or stating "nothing drifted".

## The fixed output set (13 files — never more, never fewer)

```
docs/handbook/
  overview.md          system overview + architecture + chain diagram
  index.md             routing backbone: stage table + route-by-intent + lanes/gates
  register.md          every .bee/ file's schema + the CLI command surface
  using-as-planner.md  read-only planner guide (route -> read source -> EDIT plan)
  stages/<id>.md       one page per chain stage, id in:
    hive exploring planning validating swarming executing scribing compounding reviewing
```

If a chain skill is **added or removed**, add/remove its `stages/<id>.md` and update
the stage table in `index.md` and the chain diagram in `overview.md` — the file set
tracks the chain, not a frozen list.

## The regeneration loop

1. **Gather (delegate — Delegation contract).** Dispatch down-tier I/O workers; never
   read these inline (each is >3 files):
   - Worker A — read the nine chain skills at `skills/<name>/SKILL.md`
     (`bee-hive bee-exploring bee-planning bee-validating bee-swarming bee-executing
     bee-scribing bee-compounding bee-reviewing`). Per skill, return: **purpose ·
     when · inputs · outputs · gate (verbatim question) · state_touched · key_rules ·
     path**.
   - Worker B — read the state layer: `.bee/state.json`, `config.json`,
     `onboarding.json`, one `.bee/cells/*.json`, `reservations.json`, one line each
     of `decisions.jsonl` / `backlog.jsonl` / `capture-queue.jsonl`, `HANDOFF.json`
     shape, and the CLI groups from `.bee/bin/bee.mjs`. Return field lists + the
     group->verb map. **Never dump full data files — a representative shape only, and
     never a secret-shaped file.**

2. **Author the connective tissue yourself** (do NOT delegate synthesis):
   `overview.md`, `index.md`, `register.md`. These are the map's spine and must read
   as one system.

3. **Author `stages/<id>.md`** — one per chain skill, using the FIXED template below,
   filled only from Worker A's digest.

4. **Author `using-as-planner.md`** — the read-only planner discipline: route the
   handbook -> read the real source -> emit an EDIT completeness check -> hand the
   plan to bee's own chain (hive -> lane -> Gates 1-3). Localizing an edit is never
   approving it.

5. **Self-check (HARD-GATE — do not close red).** Run:
   `node skills/bee-handbook/scripts/check-handbook.mjs`
   It fails unless all 13 files exist and every internal `.md` link + heading anchor
   resolves. Quote its output in the close.

6. **Close (docs lane).** Log the refresh via `node .bee/bin/bee.mjs decisions log`
   (tags `docs,handbook`) noting any drift the regeneration corrected, or a capture
   stub, or an explicit "nothing drifted". A close with none of these is not a close.

## Fixed stage-page template

```
# Stage: <name> (`bee-<name>`)

**Purpose** — <one sentence>

**When it runs** — <trigger / chain position>

## Inputs
## Outputs
## Gate            (verbatim gate question, or "None")
## State touched   (link every .bee/ file to register.md#anchor)
## Key rules       (1-3 hard invariants unique to this stage)
## Source          `skills/bee-<name>/SKILL.md`
```

## Fidelity rules (YOU MUST)

- **NEVER invent.** Every stage/register fact comes from the live source read in
  step 1. A claim you cannot anchor to source becomes an explicit `> Open gap: …`
  note — never asserted as fact. This mirrors scribing's law.
- **Source wins over the old handbook.** When the previous handbook disagrees with
  source, the source is right and the old page was stale — overwrite it.
- **Tech-agnostic to the host.** The handbook documents *bee*; do not bake a host
  project's language/framework into it.
- **Overwrite in place.** The file set is fixed; regeneration replaces content, it
  does not fork new files or leave orphans.
- **Delegate the gather, keep the pen.** Step 1 is dispatched down-tier; steps 2–4
  (synthesis) stay on the orchestrator. A bare inline multi-file read here is the
  fan-out-rubric violation this skill exists to avoid.

## Headless

`mode:headless`: run the full loop and the self-check; never skip step 5. Defer any
ambiguous scope choice (e.g. an unfamiliar new chain skill with no clear stage
mapping) to an `Outstanding Questions` section of the terminal report — never guess a
page's content, emit an `Open gap` instead.

## Red Flags — STOP

- reading the nine skills or the `.bee/` layer **inline** instead of via I/O workers
- writing a stage-page claim not backed by the live `SKILL.md`
- adding a 14th file or leaving an orphan when the chain has 9 stages
- closing while `check-handbook.mjs` is red
- treating the old handbook as truth over the source

Violating the letter of the rules is violating the spirit of the rules.

## Handoff

Handbook regenerated and self-check green. Invoke bee-hive skill.
````

### 2. `skills/bee-handbook/agents/openai.yaml`

```yaml
interface:
  display_name: "Bee Handbook"
  short_description: "Regenerate the navigable handbook of the bee harness under docs/handbook/ from live source. Use when the user asks to build, refresh, rebuild, or update the bee handbook, or when harness changes have made it stale."
policy:
  allow_implicit_invocation: true
```

### 3. `skills/bee-handbook/scripts/check-handbook.mjs`

```js
#!/usr/bin/env node
// bee-handbook self-check. Verifies the fixed 13-file handbook set exists under
// docs/handbook/ and that every internal markdown link + heading anchor resolves.
// Exit 0 on success, 1 with a report on any failure. No dependencies.
import { readFileSync, existsSync, readdirSync, statSync } from 'node:fs';
import path from 'node:path';

const REPO = process.cwd();
const HB = path.join(REPO, 'docs', 'handbook');
const STAGES = ['hive', 'exploring', 'planning', 'validating', 'swarming',
  'executing', 'scribing', 'compounding', 'reviewing'];
const REQUIRED = ['overview.md', 'index.md', 'register.md', 'using-as-planner.md',
  ...STAGES.map((s) => path.join('stages', `${s}.md`))];

const errors = [];

for (const rel of REQUIRED) {
  if (!existsSync(path.join(HB, rel))) errors.push(`missing required file: docs/handbook/${rel}`);
}

function walk(dir) {
  const out = [];
  if (!existsSync(dir)) return out;
  for (const e of readdirSync(dir)) {
    const p = path.join(dir, e);
    if (statSync(p).isDirectory()) out.push(...walk(p));
    else if (e.endsWith('.md')) out.push(p);
  }
  return out;
}
const present = walk(HB).map((p) => path.relative(HB, p));
for (const rel of present) {
  if (!REQUIRED.includes(rel)) errors.push(`unexpected file (orphan): docs/handbook/${rel}`);
}

// GitHub-flavored heading slug, with -1/-2 disambiguation for duplicates.
function slugify(text) {
  return text.trim().toLowerCase().replace(/[^\w\s-]/g, '').replace(/\s+/g, '-');
}
function anchorsOf(file) {
  const counts = Object.create(null);
  const set = new Set();
  for (const line of readFileSync(file, 'utf8').split('\n')) {
    const m = /^#{1,6}\s+(.+?)\s*$/.exec(line);
    if (!m) continue;
    let a = slugify(m[1]);
    if (a in counts) { counts[a] += 1; a = `${a}-${counts[a]}`; } else counts[a] = 0;
    set.add(a);
  }
  return set;
}

const anchorCache = new Map();
function anchors(file) {
  if (!anchorCache.has(file)) anchorCache.set(file, existsSync(file) ? anchorsOf(file) : null);
  return anchorCache.get(file);
}

const linkRe = /\[[^\]]*\]\(([^)]+)\)/g;
for (const rel of present) {
  const file = path.join(HB, rel);
  const text = readFileSync(file, 'utf8');
  let m;
  while ((m = linkRe.exec(text))) {
    const raw = m[1].trim();
    if (/^(https?:|mailto:)/.test(raw)) continue;
    const [target, anchor] = raw.split('#');
    let targetFile = file;
    if (target) {
      targetFile = path.normalize(path.join(path.dirname(file), target));
      if (!existsSync(targetFile)) { errors.push(`${rel}: broken link target -> ${raw}`); continue; }
    }
    if (anchor) {
      const set = anchors(targetFile);
      if (set && !set.has(anchor)) errors.push(`${rel}: broken anchor -> ${raw}`);
    }
  }
}

if (errors.length) {
  console.error(`check-handbook: FAIL (${errors.length})`);
  for (const e of errors) console.error(`  - ${e}`);
  process.exit(1);
}
console.log(`check-handbook: OK — ${REQUIRED.length} files, all internal links + anchors resolve`);
```

### 4. `skills/bee-handbook/CREATION-LOG.md`

```markdown
# CREATION-LOG — bee-handbook

## Source material
- Reference concept: https://github.com/Ruhan-Wang/Harness_Handbook (overview →
  index → register → stages/<id>, plus a read-only planner "SKILL").
- Mapped onto bee: the chain is the set of stages; the `.bee/` runtime files are the
  state registers. Decision logged in main: tags `docs,handbook,harness`.

## Skill class
A **procedural / generator** skill, not a persuasion/compliance skill. The
meaningful test is not a pressure scenario but a deterministic output check: running
the regeneration loop must produce the fixed 13-file set with every internal link and
heading anchor resolving. `scripts/check-handbook.mjs` is that gate and is the
skill's own verify.

## RED → GREEN
- RED (baseline): an agent asked to "update the bee handbook" without this skill has
  no fixed file set, no stage template, and no fidelity rule — it produces an
  inconsistent or invented handbook, and no link/anchor gate catches drift.
- GREEN: with the skill, the file set is fixed to 9 chain stages + 4 connective
  files, every stage fact is source-derived (never invented), and
  `check-handbook.mjs` blocks a red close. Verified by generating docs/handbook/ in
  this worktree and running the check green.

## Fidelity guarantees
- gather is delegated down-tier (Delegation contract); synthesis stays on the
  orchestrator.
- "NEVER invent" mirrors scribing; unbacked claims become `> Open gap:` notes.
- overwrite-in-place, no orphans — enforced by the check's orphan detection.
```

## Propagate to the four skill mirrors (auto-discovery; no manual registry edit)

```
node scripts/render_plugin_skill_trees.mjs
node skills/bee-hive/scripts/onboard_bee.mjs --repo-root . --apply --repo-hooks
```

The first renders `.claude-plugin/skills/bee-handbook/*` and
`.codex-plugin/skills/bee-handbook/*` (+ both `.bee-render.json` sidecars); the second
mirrors into `.claude/skills/bee-handbook/*` and `.agents/skills/bee-handbook/*` and
refreshes `.bee/onboarding.json`. `plugin.json` needs no edit — it points at the
skills directory and the renderers auto-discover every `skills/bee-*` dir.

## Verify (all must pass before cap)

```
node --check skills/bee-handbook/scripts/check-handbook.mjs
node skills/bee-writing-skills/scripts/test_openai_metadata.mjs   # validates agents/openai.yaml shape
# then generate docs/handbook/ by running the skill's loop, and:
node skills/bee-handbook/scripts/check-handbook.mjs
test -f .claude/skills/bee-handbook/SKILL.md && \
test -f .agents/skills/bee-handbook/SKILL.md && \
test -f .claude-plugin/skills/bee-handbook/SKILL.md && \
test -f .codex-plugin/skills/bee-handbook/SKILL.md && echo "all four mirrors present"
```

## Close + merge back
1. Cap the cell with the verify output recorded; run scribing/compounding to close
   the feature in the worktree.
2. From **main** (not the worktree), once main is clean enough:
   `node .bee/bin/bee.mjs worktree merge --id beegog--wt--bee-handbook --cleanup`
   — this runs `git merge --no-ff wt/bee-handbook` then the configured verify against
   the merged tree as the semantic-conflict gate, and on green removes the worktree +
   branch + grant.

> Note: this brief lives in main at
> `docs/history/bee-handbook/skill-package-brief.md` (uncommitted). Read it by
> absolute path from the worktree session.
