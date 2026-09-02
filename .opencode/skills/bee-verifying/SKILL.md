---
name: bee-verifying
description: >-
  Generate a project-local `verify-app` skill that drives the real product the way a user does — any language, framework, or platform — and prove it by running it end to end. Use ONCE per repo, when the repo has no scripted way to prove UI, CLI, TUI, service, desktop or API behavior; when onboarding reports no declared test command and offers verification; or when the user asks for a control skill, a drive harness, or "a way to actually run this app" for this repo. Not the periodic audit of a verification skill that already exists — that is bee-verify-upkeep.
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

The generated skill is **source** under `.bee/verify/verify-app/`. That path is
a constant. The name is `verify-app` in every repo — never a per-project name,
and never `bee-` prefixed, because bee prunes any `bee-*` skill it does not own
itself. Content differs per project; the name never does, and the app's identity
lives in the skill's `description`, which is what routing reads. bee renders the
source into every runtime skill home — `.claude/skills/`, `.agents/skills/` and
`.opencode/skills/` — the same way the `bee-*` skills are rendered. One
generation serves every runtime.

Two rules follow from that, and both hold for the rest of this skill's life:

- Write and edit only the source under `.bee/verify/verify-app/`. The rendered
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

Write `.bee/verify/verify-app/SKILL.md` with YAML frontmatter (`name:
verify-app`, the constant, plus a `description` that names the app, the surface,
and when to reach for it — the description carries the identity the name no
longer does, and without frontmatter the skill never registers) and these
sections, each grounded in what the interview actually found (no placeholders
left):

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
  fast, selectable one for the per-change loop and a full sweep. Step 5 states
  that contract; build both entry points here.
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
  `.bee/verify/verify-app/`, and its invocation is shown in the skill body as
  `bash <path> …`, never a bare path. A helper the reader has to
  reverse-engineer is not a helper. The reason for the `bash` prefix, stated
  here so a later reader does not "fix" it back: bee's render does not carry the
  executable bit into the copies it writes into the runtime skill homes (source
  `0755`, every rendered copy `0644`), so a bare path fails with
  `Permission denied` the first time a host onboards. The prefix is what makes
  one written line run from either copy — the drive line a cap proof invokes
  included.

## 3. Seed the feature map

Create `.bee/verify/verify-app/features/README.md` plus one file per
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

## 5. Fast and full: the contract the control script must satisfy

The generated control script exposes TWO drive scopes. This is a contract on the
generated skill, not a suggestion:

- **Fast drive** — the per-change loop. It runs the launch, the doctor check and
  the mapped features a change touches, with a floor of one smoke path when
  nothing is named. It takes a feature selector, it needs no secret or network a
  fresh clone lacks, and it is short enough to sit in front of every commit.
- **Full drive** — every feature in the map, top to bottom.

**The FAST drive is what proves a user-facing change.** A worker runs it for
that change's cap proof:

```
bash .bee/verify/verify-app/control-<app> drive --fast <feature>
```

and records the result as `green:live` — the real product was driven and its
result inspected. The FULL drive is the periodic sweep over the whole map, run
on a release or an audit; a per-change loop that ran it would pay for every
feature to prove one.

Neither scope is written into the project's declared test command, and this
skill never edits `commands.test`. A test asks whether the code stayed correct;
a drive asks whether the product works for a user. One command for both makes a
red result ambiguous — a broken function and an app that failed to launch report
identically — and forces CI to carry a live app harness.

What "fast" scopes to is decided per repo, at generation time — a route group, a
command surface, one screen — but the two entry points and the selector are
fixed. A control script offering only one scope has not met this step.

## 6. Offer the maintenance loop

Point the user at `bee-verify-upkeep` for keeping the map honest as the app
changes. Suggest a cadence only if they ask.
