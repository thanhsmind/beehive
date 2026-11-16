# investigation

You own the answer. Plan, route, write. A request that asks "how does X work?",
"why was Y built this way?", "are we sure about Z?" or "should we do X or Y?"
produces a cited account or a recommendation, never a code change, so the
outcome is words with anchors.

1. Stay read-only: the outcome is an account, never a diff. An edit changes the
   thing you report on, so a reader could no longer trust the account to
   describe what they will find.
2. Route the question through the gathers, because a lead that reads the whole
   subsystem itself spends its context on the reading and has none left to
   judge. Run `bee-how` over the subsystem as a gather
   (`bee dispatch prepare --kind gather`): narrow questions take its Explain
   mode, and "are we sure?" takes its Critique mode, because that question
   asks for a challenge and not a tour. A question about motivation also runs
   `bee-why`, because the reason a thing is built a certain way lives in the
   recorded decisions and the code only shows the result.
3. Trace the runtime path, not just the file list, because the list shows what
   exists and not what runs. Both this step and the next have ONE home:
   `bee-researching/references/trace-and-provenance.md` — § "Trace" for this
   step, § "Provenance sweep" for the next.
4. Name every source searched that came up EMPTY, so that nobody searches it
   again or reads its silence as unchecked.
5. Write the `bee-how`-shaped account: Overview, Key Concepts, How It Works,
   Where Things Live, Gotchas, because a reader who opens several accounts
   finds the same parts in the same places. When the request is a decision
   between alternatives, end in a recommendation and a tradeoffs table: the
   reader came for a pick, and a survey that leaves the choice open sends the
   work back.
6. End with anchors a reader can open: `path:line`, or the command that ran,
   because an anchor nobody can open is a claim nobody can check.
7. Apply `bee-unslop` to the reply, because the reader meets the reply first
   and an account that reads as machine prose loses trust before its anchors
   are opened.

An investigation that precedes a code change stops at the account and hands
back to the user; the change itself re-routes to `bugfix` or `feature`, because
this class is read-only and the edit belongs to a class that proves it. Run
`bee-architect` only for that change, never inside the investigation, because
a design is written for a diff and this class writes no diff.

This is the investigation route (per D3, decision `f1ffa7bd`): the existing
`research` class, no new route and no new lane. Nothing yet ENFORCES step 1 —
read-only is craft here, not a guard (backlog `p-69bee217`).

Reply: the investigation output. For "are we sure?" answers, include your real
judgment with reasons, and push back when the premise is wrong, because
agreement that hides a false premise is the costliest answer.

Proof line: the account's anchors, each one opened or run.
