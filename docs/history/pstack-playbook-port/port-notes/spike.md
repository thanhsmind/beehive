# Port notes — spike class

Source: pstack poteto-mode at commit `b9ddc83`, MIT, copyright 2026 Lauren Tan
(notice in `skills/bee-planning/playbooks/LICENSE.pstack`). Cell ppb-8. This
file is the only place a pstack-only tool is named; no file under `skills/`
names one. `skills/bee-planning/playbooks/spike.md` is now the class index with
one situation.

## prototype

Source `playbooks/prototype.md`, merged with the bee `spike` class playbook
(D3) into `skills/bee-planning/playbooks/spike/prototype.md`.

Tool swaps:

- The control skill (Verify, screenshot each variant) became the project's
  `verify-app` skill, or the real interface when the project has none.
- The scratch directory "separate from production source" became
  `.bee/spikes/<feature>/`, the bee spike home; the bee line wins on where the
  throwaway lives and on deleting it after the answer is logged.
- The throwaway artifact and scratch path in the Reply became the decision id
  the answer was logged under, because bee deletes the spike (D3, the bee
  line "record the answer with `bee decisions log`, then DELETE the spike").
- The route to Feature became `playbooks/feature.md`, in step 1 and step 7.
- The "exhaust-the-design-space" principle skill citation in the switcher step
  became the bee spike line itself: build all variants side by side behind one
  switcher.

Dropped steps:

- The `architect` handoff for the shape in the last step: the skill does not
  exist in bee yet (D5, slice 3). The step hands the chosen direction to
  `playbooks/feature.md` alone. Restored (ppb-24): step 7 hands the direction
  to `bee-architect` as well.
- The `exhaust-the-design-space` principle skill citation: no bee principle of
  that name exists yet (D5, slice 3). The step keeps its rule in plain words.
  Still waiting (ppb-24): no `bee-principle-exhaust-the-design-space` exists, so the method stays a plain sentence.
  Restored (ppb-37): step 5 names `bee-principle-exhaust-the-design-space`.
- The `Laziness Protocol` citation in the intro ("smallest change and the
  verification bar invert"): no bee principle of that name exists yet (D5,
  slice 3). The sentence keeps the inversion without the name.
  Still waiting (ppb-24): no `bee-principle-laziness-protocol` exists, so the inversion stays a plain sentence.
  Restored (ppb-37): the intro names `bee-principle-laziness-protocol`.

Kept from the bee `spike` playbook (D3): the YES/NO question before you start,
the timebox said out loud, the throwaway under `.bee/spikes/<feature>/` with
the answer as the deliverable and the pointer to `planning-reference.md`
"Shape bodies by mode", the answer recorded with `bee decisions log` and the
spike deleted, the build-all-variants-behind-one-switcher step with each
variant labelled, observed on the real surface and the chosen variant named in
the recorded answer, the added variant the ask did not name, the closing line
"A spike that becomes the implementation was never a spike.", the Proof line
and the Proof floor line.

Added to the pstack text: a reason on every step where pstack gave none
(decision `61b52e96`), and the timebox step from the bee playbook as its own
numbered step.

## Steps re-added in ppb-24

Restored: `architect` (the shape handoff, now `bee-architect`).

Still waiting, because no skill of that name exists in `skills/`:
`exhaust-the-design-space` and `laziness-protocol`. Each keeps its method as a
plain sentence.
