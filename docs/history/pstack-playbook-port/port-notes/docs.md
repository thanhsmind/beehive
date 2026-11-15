# Port notes — docs class

Source: pstack poteto-mode at commit `b9ddc83`, MIT, copyright 2026 Lauren Tan
(notice in `skills/bee-planning/playbooks/LICENSE.pstack`). Cell ppb-10.
This file is the only place a pstack-only tool is named; no ported file names one.

## docs

No pstack source: pstack has no docs playbook, so the situation is the bee
`docs.md` text. The six steps, the closing line and the Proof line moved
unchanged into `skills/bee-planning/playbooks/docs/docs.md`. `docs.md` is now
the class index.

Tool swaps: none.

Dropped steps: none.

## authoring-a-skill

Source `playbooks/authoring-a-skill.md`, written to
`skills/bee-planning/playbooks/docs/authoring-a-skill.md`.

Tool swaps:

- The create-skill skill (Cursor's built-in for SKILL.md files) became
  `bee-writing-skills`.
- Run Opening a PR became the bee close and worktree merge path: capped work,
  `bee worktree merge`, and the owner's `uat` acceptance at `bee close`.
- The encode-lessons-in-structure principle skill became
  `bee-principle-encode-lessons-in-structure`.

Dropped steps: none.

Added to the pstack text: a reason on steps 1 to 4 (decision `61b52e96`
requires one of class playbooks; pstack gave none), a Proof line and the Proof
floor line. The paragraph of prose rules and the Reply line are kept.

## release

No pstack source. The four steps, the closing line and the Proof line moved
unchanged into `skills/bee-planning/playbooks/release/release.md`; `release.md`
is now the class index with that one situation.

Tool swaps: none.

Dropped steps: none.

## content

No pstack source. The four steps, the no-tool paragraph and the Proof line
moved unchanged into `skills/bee-planning/playbooks/content/content.md`;
`content.md` is now the class index with that one situation.

Tool swaps: none.

Dropped steps: none.
