# spike

1. Write the question as a YES/NO before you start, so that you know when it
   is answered.
2. Timebox it, and say the box out loud — a spike with no end turns into the
   build.
3. Keep the throwaway under `.bee/spikes/<feature>/` — the ANSWER is the
   deliverable, the code is not (the shape body is named in
   `bee-planning/references/planning-reference.md`, under "Shape
   bodies by mode").
4. Record the answer with `bee decisions log`, then DELETE the spike — the
   answer outlives the code, and a kept spike gets shipped.
5. When the question is which of several variants, build them all — side by
   side, the answer is seen, not argued. Put each variant under
   `.bee/spikes/<feature>/` behind one switcher, each variant labelled.
   Observe each variant on the real surface: drive it with the project's own
   verification skill when the project has one, otherwise log or print the
   thing you are deciding. The observation is the proof, not an assertion.
   The answer that step 4 records names the chosen variant. When the design
   space is open, add one variant the ask did not name.

A spike that becomes the implementation was never a spike.

Proof line: the decision id the answer was logged under, and the
observation of each variant.
