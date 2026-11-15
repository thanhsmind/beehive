---
name: bee-principle-encode-lessons-in-structure
description: "Apply when you are writing the same instruction a second time or giving the same correction again. Turn the lesson into structure — a code change, then a check, then prose last."
---

# Encode Lessons in Structure

A lesson seen twice has cost twice. Writing it as one more sentence asks
every future reader to remember it, and they will not. Climb the ladder and
stop at the highest rung that fits: change the code or data so the mistake
cannot be written; failing that, add a test, hook, guard or doctor row that
catches it; only when the lesson needs judgment, write prose.

Pick the strongest rung the situation allows. Within the structural rungs the
order is: a state that cannot be represented, then a lint or banned API that
fails the build, then one canonical helper, then a runtime check. Workers copy
whatever the surrounding code already does, so a weaker guard becomes the
next template.

Prose then has a job: make it prominent and show the failure it prevents.
When the structural fix exists, delete the prose that taught the lesson. The
instruction is the symptom, and a stale sentence beside a working guard
suggests there is still something to remember. When the fix is structural, use
only the structural fix.

Close the loop on every correction. When the human steps in or a check fails,
decide whether it is a one-off or a pattern. A one-off is a record
(`bee-capturing`); a recurring fix is a skill, a test or a guard; a systemic
issue is a principle. Then apply it now or open a backlog item. Three
failures to avoid: acknowledging without recording ("I'll keep that in mind"
persists nowhere), recording without routing (a note about a guard that should
exist is wasted until the guard does), and fixing one instance while leaving
the pattern intact.

**Check:** which rung of the ladder can enforce this lesson, and is it the
highest one available?

**Not when:** the lesson has shown up once. One occurrence is a note, not yet
a pattern worth building structure for.

**Why:** a rule held in memory fails on the day someone is tired or new; a
rule held in structure does not need anyone to remember it.

**Depth:** `.bee/expertise/knowledge.md` § Encode lessons in structure.
