---
name: bee-verifying
description: >-
  Generate a project-local `verify-<app>` skill that drives the real product the way a user does — any language, framework, or platform — and prove it by running it end to end. Use ONCE per repo, when the repo has no scripted way to prove UI, CLI, TUI, service, desktop or API behavior; when onboarding reports no declared test command and offers verification; or when the user asks for a control skill, a drive harness, or "a way to actually run this app" for this repo. Not the periodic audit of a verification skill that already exists — that is bee-verify-upkeep.
disable-model-invocation: true
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: The generated skill is source under `.bee/verify/`; the vendored bee binary is what renders it into every runtime skill home. Absent, the generated skill still exists and still runs, but only the runtime whose home was written by hand can see it.
---

# Verifying — generate the skill that drives the real app

Every serious project needs a scripted way to drive the real app and prove
behavior: launch it, exercise a feature the way a user would, and capture
evidence. This skill generates that as a project-local skill, once per repo.
You write the generator's output for the next agent, not for a human: it will
be read cold, mid-task, by an agent that has never seen the app.

## Where the generated skill lives

The generated skill is **source** under `.bee/verify/verify-<app>/`. bee renders
it from there into every runtime skill home — `.claude/skills/`,
`.agents/skills/` and `.opencode/skills/` — the same way the `bee-*` skills are
rendered. One generation serves every runtime.

Two rules follow from that, and both hold for the rest of this skill's life:

- Write and edit only the source under `.bee/verify/verify-<app>/`. The rendered
  copies are bee's output; a hand edit there is lost at the next
  `bee onboard --apply`.
- After any edit to the source, run `bee onboard --apply` to re-render, so every
  runtime sees the same bytes.

## 1. Interview the repo, not the user

Answer these from the codebase and only ask the user what you cannot observe:

- **Surface:** what does a user actually touch? A web UI, a CLI/TUI, a desktop
  app, an API, a mobile app, a library? A repo can have several; pick the
  primary one and note the rest.
- **Run:** how does the app start locally? Prefer the repo's own documented dev
  command (package scripts, Makefile, README quickstart). Note ports, env vars,
  seed data, auth.
- **Drive:** how can an agent interact with it programmatically? Existing
  harnesses first — Playwright/Cypress specs, expect scripts, PTY helpers,
  curl-able endpoints, a debug port. Only then pick a generic recipe: browser/CDP
  for web and Electron, a tmux/PTY harness for CLI/TUI, plain HTTP for services.
- **Observe:** what evidence can be captured? Screenshots, terminal transcripts,
  response bodies, logs, exit codes, DB state.
- **Isolate:** can two instances run side by side (ports, data dirs, profiles)?
  If not, say so in the generated skill: refusing to double-drive a shared
  instance beats corrupting the user's session.

If the checkout doesn't build or start as-is, fix that first (or report it
precisely) before generating; a skill written against a broken base teaches wrong
steps. When an irrelevant missing asset blocks startup (a static dir the API
never serves, a sample config), the generated skill may create it, clearly marked
as verification scaffolding, and remove it in cleanup.

## 2. Generate the skill

Write `.bee/verify/verify-<app>/SKILL.md` with YAML frontmatter (`name:
verify-<app>` and a `description` that names the app, the surface, and when to
reach for it — without frontmatter the skill never registers) and these sections,
each grounded in what the interview actually found (no placeholders left):

- **Launch:** the exact command that starts the app for verification, and how to
  tell it's ready (a log line, a port answering, a prompt). Include teardown. For
  a short-lived CLI or TUI there is no server to keep alive: launch means build
  the binary (or install deps) once, then start each drive in its own isolated
  PTY or tmux session.
- **Doctor:** one read-only check that answers "is this instance worth driving?"
  — process up, right version/build, port owned by us, auth valid. An agent runs
  this first whenever anything looks off.
- **Drive:** the harness recipe with real selectors/commands from this repo, not
  examples. Prefer stable handles (ARIA labels, data attributes, prompt strings,
  route paths) over coordinates and tab order. The drive comes in two scopes — a
  fast, selectable one for the per-change loop and a full sweep for CI. Step 5
  states that contract; build both entry points here.
- **Evidence:** what to capture for a proof and where it goes. State the proof
  standards: exercise the real user path, not internal setters or test-only
  endpoints; capture the action and the resulting state, not just the final
  screen; verify side effects (files written, rows inserted, messages sent)
  alongside what's visible; mocks only where a production boundary already
  isolates the external system. When the safe path is a dry-run or test mode,
  verify what it actually skips by observing (files, network, git refs) rather
  than trusting its name: some dry-runs still touch the network or open a
  browser.
- **Cleanup:** how to tear down instances the run created. Never kill by process
  name; kill what you started. Cleanup removes instances and scratch state, never
  the evidence: proof artifacts survive the teardown, in a location the skill
  names.
- **Helpers:** any script the skill ships is executable in the source tree under
  `.bee/verify/verify-<app>/`, and its invocation is shown in the skill body. A
  helper the reader has to reverse-engineer is not a helper. bee's render strips
  the executable bit from the copies it writes into the runtime skill homes, so
  every invocation the skill body shows is written `bash <path> …` — a rendered
  copy cannot be executed directly.

## 3. Seed the feature map

Create `.bee/verify/verify-<app>/features/README.md` plus one file per
user-facing feature you can identify (aim for the top 3-5 to start, from routes,
commands, menus, or docs). Follow the shape in
[`references/feature-map-example/`](references/feature-map-example/), with a
README index and one file per feature. Each file answers, from the user's point
of view: what the feature is, how to reach it, how to drive it with the harness,
and what observable end state proves it works. The four H2s are `Sub-features`,
`How to get to it (user POV)`, `Driving it with <harness>`, and `Gotchas`. The
map is the repo's maintained verification source; a proof that drives one
convenient entry point is incomplete when the map lists others.

## 4. Prove the generated skill before handing it over

Run its own instructions end to end once: launch, doctor, drive ONE mapped
feature (one is enough; the map exists so later runs can cover the rest), capture
evidence, clean up. After cleanup, confirm the evidence still exists at the named
location — a cleanup that eats the proof fails this step. Fix what fails, and run
the generated cleanup after every failed iteration too, so broken attempts don't
strand processes and ports. A generated skill that was never executed is a draft,
not a deliverable.

Then run `bee onboard --apply` once, so the proven source reaches every runtime
skill home.

Time this run on a wall clock and keep the number. Step 5 asks a question that
is only honest with it.

## 5. Ask the second question, then compose the drive into the test command

Steps 1-4 cost the user nothing beyond the time they already agreed to spend.
Wiring the drive into the command this project declares as its test command is a
**second** agreement, with its own price, and it is asked separately.

### Two moments, never one yes

- **Moment one** — build the skill and prove it (steps 1-4). Cheap, reversible,
  and it touches no configuration. This is the yes the onboarding offer asked
  for, and so far it is the only yes you have.
- **Moment two** — asked *after* the step-4 proof run, never before, because
  that run is what measures the number the question turns on. Quote the measured
  figure back:

  > The check ran end to end in **2m 14s**. I can add it to the command this
  > project declares as its test command, so every commit proof and every CI
  > push drives the real product. That adds about 2m 14s to each of those runs.
  > Add it, or keep the check as something you run by hand?

Never bundle the two into one yes. A user who wanted proof their app works has
not thereby agreed to permanently slower commits: that is a different cost, paid
on every future run, and the choice belongs to whoever owns the repo. An
estimate is not the question either — quote what the proof run actually took.

On a **no**, the skill stays and stays manual: nothing is written, and the
generated skill loses nothing. Record that answer
(`bee decisions log --relation none`, or the repo's own record where bee is not
installed) so the question is not asked again unprompted at the next onboard.
Everything below runs only after a yes.

### Compose: append, never replace

The declared test command is `commands.test` in `.bee/config.json`.

- **The repo already declares one** — the new value is `<existing> && <drive>`.
  The existing command is never replaced, never rewritten and never reordered;
  the unit tests a project already trusts keep running, and the drive runs after
  them.
- **The repo declares none** — the drive alone becomes the value.
- **The value is a JSON array, not a string** — append ONE element to the array.
  Do not flatten the array into a string, and do not concatenate `&& <drive>`
  onto its last element.
- **Look for the drive before you write it.** Read the current value and search
  it for the drive path. Already there? Stop — the work is done. Appending twice
  makes every test run drive the app twice, and a re-run of this skill, a second
  generation, or a repeated yes all arrive at this same line.

### Always `bash <path>`, never a bare path

The composed command names the interpreter:

```
bash .bee/verify/verify-<app>/control-<app> drive --fast
```

The reason, stated here so a later reader does not "fix" it back to a bare path:
bee's render does not carry the executable bit into the copies it writes into the
runtime skill homes (source `0755`, every rendered copy `0644`), so a bare path
fails with `Permission denied` the first time a host onboards, and the `bash`
prefix is what makes one written line run from either copy.

### Where to write it

Edit `.bee/config.json` directly. bee state normally changes through the CLI
only, but `bee config set` is not built in this binary — it refuses, and its own
FIX line names the direct edit as the remedy:

```
bee: not built into this binary: `bee config set` is declared in the command
registry, the config verbs were never ported off Node. Nothing ran and nothing
changed. FIX: read and edit `.bee/config.json` directly — it is plain JSON
```

That refusal is why a hand edit is the sanctioned path here, and only here. Read
the file, change the one `commands.test` value, and leave every other byte as it
was. Then confirm two things: the file is still valid JSON, and the composed
command runs.

### Fast and full: the contract the control script must satisfy

The generated control script exposes TWO drive scopes. This is a contract on the
generated skill, not a suggestion:

- **Fast drive** — the per-change loop. It runs the launch, the doctor check and
  the mapped features a change touches, with a floor of one smoke path when
  nothing is named. It takes a feature selector, it needs no secret or network a
  fresh clone lacks, and it is short enough to sit in front of every commit.
- **Full drive** — every feature in the map, top to bottom. This one is CI's.

**The FAST drive is what composes into the declared test command.** The full
drive stays out of it and is invoked by CI on push. The reason is the one that
shaped the declared command in the first place: the dev loop runs the impacted
subset, and the full sweep belongs to CI. Composing the full drive would make
every commit proof pay for the whole feature map.

What "fast" scopes to is decided per repo, at generation time — a route group, a
command surface, one screen — but the two entry points and the selector are
fixed. A control script offering only one scope has not met this step.

## 6. Offer the maintenance loop

Point the user at `bee-verify-upkeep` for keeping the map honest as the app
changes. Suggest a cadence only if they ask.
