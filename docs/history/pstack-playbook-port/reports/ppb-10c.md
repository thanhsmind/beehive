# ppb-10c — the docs, release and content port checked against its sources

Cell ppb-10c checks the ppb-10 port (commit `ea91d0e27`) against its sources.
It covers D2 (`063a21e4`), D3 (`5a1fc656`) and decision `913115b3`.

Sources read:

- pstack `skills/poteto-mode/playbooks/authoring-a-skill.md` (refs checkout)
- the old bee class playbooks: `git show ea91d0e27^:skills/bee-planning/playbooks/{docs,release,content}.md`

Files under check:

- `skills/bee-planning/playbooks/docs.md` (class index)
- `skills/bee-planning/playbooks/docs/docs.md`
- `skills/bee-planning/playbooks/docs/authoring-a-skill.md`
- `skills/bee-planning/playbooks/release.md` (class index)
- `skills/bee-planning/playbooks/release/release.md`
- `skills/bee-planning/playbooks/content.md` (class index)
- `skills/bee-planning/playbooks/content/content.md`
- `docs/history/pstack-playbook-port/port-notes/docs.md`

## Verdict

The port is faithful. Every pstack step in authoring-a-skill is in the
situation file, three of them swapped for a bee counterpart and each swap
named in the port notes. Nothing was dropped. The old docs, release and
content texts moved into their situation files byte for byte below the
title line. Every index line parses under the contract grammar and names a
file that exists. Two small findings, neither blocking:

1. Port notes line 5–6 say "no file under `skills/` names one" (a
   pstack-only tool). That is false word for word: provenance credits such
   as `skills/bee-planning/references/design-sketch.md:11`,
   `skills/bee-unslop/SKILL.md:13` and `skills/bee-teach/SKILL.md:18` name
   `cursor/plugins`. None is a tool step and none is under
   `skills/bee-planning/playbooks/`, so the cell's verify is green. ppb-3c
   reported the same wording in `port-notes.md`. The claim should read "no
   file under `skills/bee-planning/playbooks/`".
2. `docs/authoring-a-skill.md` keeps pstack's "Tell the agent to do the thing
   and skip the reason", while its own steps 1–4 now carry a reason each
   (decision `61b52e96`). The port notes record the added reasons. The prose
   rule is about the skill being written, not this playbook, so this is a
   tension a reader may notice, not a defect.

## pstack authoring-a-skill — step by step

| pstack source line | In `docs/authoring-a-skill.md` | Status |
|---|---|---|
| "You own the skill's voice." | line 3 | present |
| Agent-facing prose has a higher bar; unhelpful sentences become instructions | lines 3–5 | present |
| 1. Use the create-skill skill (Cursor's built-in) | step 1, `bee-writing-skills` (exists under `skills/`) | present (swapped, noted) |
| 2. Validate: frontmatter has `name` and `description` | step 2 | present |
| 2. Referenced files exist | step 2 | present |
| 2. Cross-skill links resolve | step 2 | present |
| 3. Test cases if structural; skip if subjective | step 3 | present |
| 4. Run Opening a PR | step 4, bee close and `bee worktree merge` path, `uat` at `bee close` | present (swapped, noted) |
| When in doubt, delete; prose earns its keep by changing a decision | closing paragraph | present |
| Tell it to do the thing and skip the reason | closing paragraph | present |
| Explain only when the rule is confusing without one | closing paragraph | present |
| Match tone to scope | closing paragraph | present |
| Point at structural sources (types, READMEs, config); hardcoded details go stale | closing paragraph | present |
| the encode-lessons-in-structure principle skill | `bee-principle-encode-lessons-in-structure` (exists under `skills/`) | present (swapped, noted) |
| Delegate to other skills by path; don't restate | closing paragraph | present |
| A workflow you keep hitting but isn't captured → propose a new skill | closing paragraph | present |
| Reply: summary, key design decisions, validation notes | `Reply:` line | present |

Dropped: none. Missing: none. Added beyond pstack: a reason on each step, a
Proof line and the Proof floor line, all named in the port notes.

## Old bee playbooks — enforced lines

The body of each old class file (every line after `# <class>`) was diffed
against the body of the new situation file: identical for all three.

| Old file | Enforced line | New home | Status |
|---|---|---|---|
| `docs.md` | steps 1–6 | `docs/docs.md` steps 1–6 | present |
| `docs.md` | "A second document saying the same thing is not coverage." | `docs/docs.md` | present |
| `docs.md` | Proof line (parity or pointer check and its output) | `docs/docs.md` | present |
| `docs.md` | Proof floor line | `docs/docs.md` and `docs.md` index | present, byte-identical |
| `release.md` | steps 1–4 (`scripts/release.sh`, which exists) | `release/release.md` steps 1–4 | present |
| `release.md` | "A release commit without that `OK` line is not a release." | `release/release.md` | present |
| `release.md` | Proof line (final `OK` line verbatim) | `release/release.md` | present |
| `release.md` | Proof floor line | `release/release.md` and `release.md` index | present, byte-identical |
| `content.md` | steps 1–4 | `content/content.md` steps 1–4 | present |
| `content.md` | no-tool paragraph (D6, `8fd45bbb`) | `content/content.md` | present |
| `content.md` | Proof line (`<check> — <result> — <scope reason>`) | `content/content.md` | present |
| `content.md` | Proof floor line | `content/content.md` and `content.md` index | present, byte-identical |

Dropped: none. Missing: none.

## pstack-only tool names under `skills/bee-planning/playbooks/`

`rg -n -i 'cursor|graphite|bugbot|poteto-agent|poteto|create-skill|opening a pr' skills/bee-planning/playbooks/`
finds nothing (exit 1).

## Skills named

| Name | Where | Exists under `skills/` |
|---|---|---|
| `bee-writing-skills` | `docs/authoring-a-skill.md` step 1 | yes |
| `bee-principle-encode-lessons-in-structure` | `docs/authoring-a-skill.md` closing paragraph | yes (`SKILL.md` present) |

The three docs, release and content files name no other skill. Commands named
(`bee worktree merge`, `bee close`, `bee cells finish --help`,
`scripts/release.sh`) all have a home in the repo.

## Index lines against the contract grammar

Grammar (`913115b3`): under `## Situations`, one line per situation of the form
``- `<name>` — <triggers, English and Vietnamese> — delivers: <deliverable>``,
`<name>` matching `[a-z0-9-]+`.

| Index | Name | Grammar | Vietnamese triggers | File |
|---|---|---|---|---|
| `docs.md` | `docs` | match | yes | `docs/docs.md` exists |
| `docs.md` | `authoring-a-skill` | match | yes | `docs/authoring-a-skill.md` exists |
| `release.md` | `release` | match | yes | `release/release.md` exists |
| `content.md` | `content` | match | yes | `content/content.md` exists |

No situation file is unlisted. Each index has a title, one intro line, the
Situations section and the Proof floor line, the same shape as `bugfix.md`.

## Proof

- `rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-planning/playbooks/ ; test $? -eq 1` — exit 0 (green).
- `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test class_playbook_parity` — 9 passed, 0 failed.
