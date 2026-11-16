# prototype

You own the design decision, not the code. The prototype is a throwaway
instrument; the real build follows `playbooks/feature.md`. This is for
"prototype", "mock it up", "sketch this", "try this layout", or exploring a UI,
interaction or layout before committing. It is also for settling an empirical
fork (which behavior, which timing, which approach) by observing it run, when
you would otherwise ask the human a question a quick sketch could answer for
you.

This is the one playbook where the smallest-change rule (`bee-principle-laziness-protocol`) and the
verification bar invert. Speed over polish, code quality does not matter, no planning. The
rigor is in picking the right design cheaply. Be bold: propose variations the
user didn't ask for, throw an approach away and try another. The question and
its box come first and the answer outlives the code, because a spike with no
end turns into the build, and a spike that becomes the implementation was never
a spike.

1. Scope the decision the prototype exists to make: which layout, which
   interaction, which density, or for an empirical fork which behavior, timing
   or approach. Write it as a YES/NO before you start, so that you know when it
   is answered; a which-one question becomes the YES/NO on the leading variant.
   No decision means no prototype; route to `playbooks/feature.md`, because a
   sketch with no question to answer is the build started early.
2. Timebox it, and say the box out loud. A spike with no end turns into the
   build, and the box is what makes the question stay small.
3. Gather references when the design space is open. Search for prior art,
   summarize a moodboard of themes, palettes and layouts, and let the user pick
   directions before building. Skip when the direction is set. The user's pick
   costs a sentence; a wrong direction costs the whole sketch.
4. Build the throwaway under `.bee/spikes/<feature>/`, separate from
   production source, because the ANSWER is the deliverable, the code is not
   (the shape body is named in `bee-planning/references/plan-artifact.md`,
   under "Artifact: plan.md"). For a visual decision, vanilla
   HTML/CSS/JS or the lightest stack that renders the idea, CDN deps, a dev
   server with hot reload. For a behavioral or timing decision, the smallest
   script that exercises the question. No production framework, no tests, no
   abstractions, because each one is cost the answer never uses.
5. When comparing alternatives, build them all, side by side, behind one
   switcher (buttons or a keypress) under `.bee/spikes/<feature>/`, each variant labelled so the user can name it, because the answer is seen, not
   argued. This is `bee-principle-exhaust-the-design-space`, made cheap. When the design space is open, add one variant the ask did not name;
   the option nobody asked for is the one a narrow ask would never have
   surfaced.
6. Verify on the matching surface. For a visual decision, screenshot each
   variant and drive the interaction, through the project's `verify-app` skill
   when the project has one, otherwise through the real interface itself; the
   eye is the test. For a behavioral or timing decision, observe the thing you
   are deciding by logging the timing, printing the output or watching the
   render. The observation is the test here, not an assertion, because an
   assertion encodes a guess about the answer and an observation is the answer.
7. Present alternatives, tradeoffs and a recommendation. The output is the
   decision, not shippable code. Record the answer with `bee decisions log`,
   naming the chosen variant and the evidence, then DELETE the spike: the
   answer outlives the code, and a kept spike gets shipped. Hand the chosen
   direction to `playbooks/feature.md` for the real build, and to
   `bee-architect` for the shape, because the sketch settled which direction,
   not how the real code is cut.

Reply: the variants explored, the evidence (screenshots for a visual decision,
the observed output or timing for a behavioral one), tradeoffs, your
recommendation, and the decision id the answer was logged under. Say plainly
that the prototype was throwaway and is deleted.

A spike that becomes the implementation was never a spike.

Proof line: the decision id the answer was logged under, and the
observation of each variant.
Proof floor: the cap refuses a result weaker than this class's floor and names it; the table is the cap's own (`bee cells finish --help`).
