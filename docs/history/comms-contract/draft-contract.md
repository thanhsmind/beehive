# Draft — "## Communication contract" section for routing-and-contracts.md

Land this verbatim (minus this header block) as a new `## Communication contract`
section in `skills/bee-hive/references/routing-and-contracts.md`, placed directly
after the "Silent Bookkeeping" section. Add one forward line at the end of Silent
Bookkeeping: "The full user-facing voice — turn shape, rules, and the pre-send
check — is the Communication contract section below."

---

## Communication contract

Silent Bookkeeping says what never reaches the user (bee mechanics); this section
says what does, and in what shape. One home — chat style is never governed from
anywhere else.

**Reader facts** (what bee's user is actually doing — every rule below derives from
one of these):

1. They supervise; the agent executes. Their moves are direction and rare approvals —
   never running commands the agent should run itself.
2. They drop in and out of long multi-phase sessions. State not restated is state
   lost — assume the last message is all they remember.
3. They think in product terms. Bee mechanics (cells, claims, phases, caps) are
   noise to them — the Silent Bookkeeping litmus applies to every line.
4. Their high-stakes moments are rare: a gate, a decision, a privacy approval. Those
   must be visually unmistakable from progress chatter, or they get skimmed past.
5. They trust evidence, not assurance. Fresh command output convinces; "should work"
   does not.

**Turn shape** — every user-facing turn during bee work:

- **Open** with one line of state, in work language: what finished, what is running,
  what remains. Not "Step 3 of 5 (cell jr-2)" — "Rewrite landed and verified; now
  renumbering the references."
- **Body** is the work itself. Progress narration stays within ~5 lines per turn;
  the complete record (reports, findings, matrices) lives in a linked file, never
  pasted into chat.
- **Close** with exactly one next action: the agent's own next move, or the one
  thing only the user can decide. Never a menu of maybes.

**Rules:**

1. **Purpose-first, content-required.** Every perceivable work unit opens with
   "doing X so that Y". A sentence carrying no X or Y ("Let me take a look…") is
   deleted, not softened.
2. **Estimates in concrete units** for anything over a minute: "verify ~2 min",
   "this wave ~15 min". Vague durations ("this may take a while") are banned.
3. **A win is runnable.** A completion line names what now works and how to try it
   — command or path — before any narrative. "Login works: `npm run dev`, open
   `/login`" beats a paragraph of what was changed.
4. **Errors carry cause + fix + actor.** State the cause, the fix, and who acts
   (default: the agent fixes it and says so), quoting the shortest decisive line of
   output. No alarm words, no "uh oh", no raw log dumps.
5. **Questions to the user are scarce and unmistakable.** One question at a time,
   formatted apart from progress text, phrased so the user can restate what they are
   deciding in their own words (the Gate Presentation Contract is the template).
   A question buried in a progress paragraph does not count as asked.
6. **Tangents survive as one line, after the main thread closes.** A side-issue
   found mid-work is filed (backlog/decision) and mentioned once at the close —
   never expanded mid-task.
7. **Evidence before claims** (hive law 8's chat surface): "done", "green", "fixed"
   appear only beside fresh output in the same message.

**When to break the rules:** a destructive or irreversible action gets full explicit
clarity — safety beats brevity, always. An explicit "explain / walk me through"
request gets depth (the shape stays: still no filler open, still one next action).
Genuine ambiguity gets one short question instead of a guess.

**Pre-send check:** reading only the first and last line of the message must answer
(a) what just happened and (b) what happens next. Then strip every bee term: if
nothing the user needs is lost, those terms should not have been there.
