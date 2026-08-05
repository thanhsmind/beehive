# doc-viewer-links — plan

**Feature:** `doc-viewer-links` · **Lane:** standard · **Class:** feature
**Flags (2):** public-contracts, multi-domain · **Product files:** 6
**Decision:** `4205835b` (2026-08-05)
**Shape reviewed:** fresh-eyes pass, 2026-08-05 — 2 blockers and 3 medium
findings, all folded in below (host-project block, skill-tree re-render,
section placement, the two config readers, file count).

## The problem

When the agent points the user at a document it prints a bare repo-relative
path — `docs/history/bee-cockpit/plan.md`. The user runs a local document
viewer (mdview) that serves the same repo over HTTP, so the path they can
actually click is
`http://10.255.255.254:7700/p/beedashboard/docs/history/bee-cockpit/plan.md`.
Today the agent only produces that URL when it remembers to call the mdview
tool by hand. bee has no idea the viewer exists. `base_url`, `doc_url`, and
`doc_viewer` return nothing anywhere in the repo, and no code builds an
`http://` URL over a `docs/` path; the only `mdview` occurrences are prose
naming the user's own tool (`.bee/backlog.jsonl:384`,
`docs/history/worktree-feature-parallelism/CONTEXT.md:25`), not bee behavior.

## What gets built

One opt-in config key, read once, injected into the session so every doc
reference the agent writes is clickable.

```json
"doc_viewer": {
  "base_url": "http://10.255.255.254:7700",
  "project": "beedashboard"
}
```

bee joins these as `<base_url>/p/<project>/<repo-relative-path>` — mdview's own
URL layout. Two fields, no template for the user to get wrong.

### Locked decisions this shape honors

Every clause below is decision `4205835b`, cited, not reinterpreted:

| Locked | How the shape carries it |
|---|---|
| key is `doc_viewer` with `base_url` + `project` | read in `state.rs` beside `bypass_level` / `ship_visibility`, the two existing config readers |
| layout is `<base_url>/p/<project>/<path>` | one join helper; no template string anywhere |
| scope is agent prose ONLY | the value reaches the agent through the session preamble and the compaction capsule; no CLI verb prints it |
| CLI text and JSON output stay on raw paths | `orient`, `status`, `gate` are not touched — `orient.rs:204` keeps emitting the bare `docs/history/<f>/CONTEXT.md` |
| no `bee doc-url` helper | no new command, no registry entry |
| unset ⇒ today's behavior | reader returns `None`; every emitter is already conditional, so the sections simply do not render |

### Why the preamble, and why also the compaction capsule

The preamble is where a config-derived fact the agent needs all session already
lives — `### Standard commands (host project)` (`budget.rs:278`) is the exact
precedent: read from `.bee/config.json`, rendered only when recorded, two lines.

The compaction capsule (`compaction.rs`, item 10) re-injects that same commands
block after a compaction, because a fact the agent must not forget has to
survive one. A doc-viewer prefix is the same kind of fact. Without it, a long
session emits URLs until it compacts and then silently reverts to bare paths —
which is the failure the user asked to fix. One line, in the capsule's existing
ordered item list.

This is not a scope widening past "agent prose only": both surfaces feed the
agent's prose. Neither prints a URL to the user.

Those two are the whole set. `startup`, `resume`, and `clear` render the
preamble; `compact` renders the capsule instead (`session_init.rs:24-25`,
`:194`, `CAPSULE_SOURCE` at `:68`), so between them every session start is
covered and the prompt-context hook needs nothing.

**One thing that stays bare, on purpose.** Dispatched workers never see the
prefix — the SessionStart hook fires for the main session only
(`.claude/settings.json:10-20`), and no worker prompt carries config-derived
text. A path a worker returns and the orchestrator relays verbatim will be
bare. That is inside the locked scope: the user-facing prose is written by the
session agent, which has the prefix. Recorded here so it is a known limit
rather than a surprise.

### Half-set config is loud, not silent

`base_url` without `project` (or either one empty, or `doc_viewer` set to
something that is not an object) produces no URL AND one stderr line naming the
key — the same shape `ship_visibility` uses at `state.rs:214` for an
unrecognized value. A key that looks configured but quietly does nothing
is the trap worth spending four lines of code on. Fully unset stays silent: that
is the default, not a mistake.

## Cells — one slice

The slice is a walking skeleton: after cell 1 a real session on this repo, with
the key set, receives a real prefix and emits a real URL. Cell 2 is the contract
prose and the reference docs that tell agent and human the key exists.

### `dvl-1` — read the key and inject the prefix

- `packages/bee-rs/crates/bee/src/state.rs` — `pub fn doc_viewer_prefix(config: &Map<String, Value>) -> Option<String>`, beside `bypass_level` (`:197`) and `ship_visibility` (`:209`), which take the same argument. Trims one trailing `/` off `base_url`, strips surrounding `/` off `project`, returns `<base>/p/<project>`; `None` + one stderr warning on a half-set or wrong-typed key; `None` and silent when absent.
- `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs` — a `### Doc links` section **immediately after the Standard commands block** (`:276-303`), rendered only when the prefix resolves, naming the prefix and the rule (append the repo-relative path; link docs as URLs, never bare paths). Never appended at the end: `session_preamble/tests.rs:49-52` pins the preamble's closing trailer (`budget.rs:421`) with `ends_with`.
- `packages/bee-rs/crates/bee/src/hooks/compaction.rs` — the same one line as an item in the survival capsule, following item 10's own pattern at `:1485-1495`. No test asserts the capsule's item list or ordering (they are `contains`-only, `:1546`, `:1620`), and the capsule has no byte budget.
- **The two injectors read config through different functions** — the preamble through `hooks::session_preamble::state::read_config_raw_open` (`session_preamble/state.rs:50`, called at `budget.rs:207`), compaction through `hooks::compaction::read_config_failopen` (`compaction.rs:124`). Neither reaches `state.rs:157`. One shared `doc_viewer_prefix` in `state.rs`; each injector hands it its own map — exactly what `compaction.rs:180` already does with `bypass_level`.
- Tests: unit cases on the reader (both fields, trailing slash, half-set warns, empty string, non-object, `config.local.json` overlay wins) and render cases on both injectors (section present when configured, absent when not). The preamble budget test's fixture config (`session_preamble/tests.rs:258-262`) gains `"doc_viewer"`, so the new section sits inside the 5120-byte ceiling (`budget.rs:43`) rather than beside it. Print the rendered length once while wording the section — the test only reports size on failure, and current headroom is unmeasured.

### `dvl-2` — write the contract and the reference

- `AGENTS.md` — the Communication paragraph that says "link records instead of pasting them" (`:135-136`) gains the clause: when a doc viewer is configured, a doc reference is emitted as its URL.
- `packages/bee/AGENTS.block.md` (`:128-129`) — **the same edit, verbatim.** This is the block onboarding splices into every HOST project's AGENTS.md (`onboard/source.rs:49` → `onboard/merge.rs:23`); its managed body is byte-identical to the root file's. Editing only the root teaches the rule to bee's own repo and withholds it from every project that would actually set `doc_viewer`.
- `skills/bee-hive/references/routing-and-contracts.md:200` — the same rule in the Communication contract's craft paragraph, where "make a win runnable by naming the command or path" already lives.
- **Re-render the skill trees in the same cell.** `devtools/skill_trees.rs:996` (`render_matches_the_committed_trees`) byte-compares a fresh render of `skills/` against the committed plugin trees; a `skills/` edit without `bee dev render-skill-trees` turns this cell's own verify red. The two self-install copies (`.claude/skills/`, `.agents/skills/`, with their sha256 sidecars) are refreshed by the installer, not by that command — refresh them too, and commit all four trees.
- `docs/config-reference.md` — a `doc_viewer` row in the `## Other keys` table (`:154-166`) plus a short section: the two fields, the layout, what a half-set key does, and the one limit (a path containing spaces has to be percent-escaped by whoever writes the link — bee joins, it does not encode).
- `.bee/config-sample.json` — **both** shapes the sample carries per key: the annotation string and the live value (`ship_visibility` shows the pair at `:28` and `:80`).

## Smaller path check

**Asked:** is there a cheaper shape that still honors every locked decision?

Two cheaper shapes were considered and rejected on evidence:

- **Prose only, no Rust.** Put the base URL in `AGENTS.md` and let the agent read it. Fails the locked decision outright — the decision names a `.bee/config.json` key, and a hardcoded host in tracked prose is not configuration.
- **Preamble only, skip the compaction capsule.** Cheaper by one file. Rejected: `compaction.rs` item 10 exists precisely because the commands block must survive a compaction, and this fact has the same lifetime. Dropping it buys one file and loses the behavior in exactly the long sessions that need it most.

**Verdict: PASS** — two cells, six product files (`state.rs`, `budget.rs`,
`compaction.rs`, `session_preamble/tests.rs`, `packages/bee/AGENTS.block.md`,
`skills/bee-hive/references/routing-and-contracts.md`), plus four generated
skill trees and three docs. No new command, no new abstraction.

## Cost if the shape is wrong

Small and contained. Every emitter is behind `Option<String>`; an unset key
leaves all four surfaces byte-identical to today. If the URL layout turns out
wrong for some other viewer, the fix is one `format!` in `state.rs` — or a
follow-up that adds the template variant the user declined today, without
disturbing the reader or either injector.

## Verify

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml` — the one declared test command, run at each cap.
