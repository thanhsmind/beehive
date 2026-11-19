---
name: bee-blast-radius
description: >-
  Find what a change could break elsewhere before it ships, and prove the one fact it is safe because of by running real code. Use when the user types /bee-blast-radius or asks the blast radius of X or what a change could break. Not for how code runs (bee-how) or the review gate (bee-reviewing).
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: The knowledge search and the wide-change arena go out through the vendored bee binary and its dispatch door. Absent, the leader reads the sources inline and the account says so.
---

# Blast radius

Find what a change breaks somewhere else, before it ships. Use it for "blast radius of X", "what could this break", or reviewing a small diff you don't trust yet.

Companion to `bee-how` and `bee-why`. `bee-how` tells you what the code does. `bee-why` tells you why it's shaped that way. Blast radius tells you what it breaks somewhere else.

Listing the callers is not the job. The agent can grep those in a second. The job is the breakage grep won't show you.

## Don't trust your own writeup

A blast-radius writeup that sounds right is worthless. It reads as convincing whether or not it's true, and that is the trap you are walking into. So don't hand back the writeup. Find the one or two facts the whole thing depends on and prove them by running code. Words are where you start, not what you ship. This is `bee-principle-verify-before-reporting` applied to a risk list: reproduce or trace each suspected break before you file it, and record the suspicions that failed verification instead of dropping them.

### How sure are you

For each fact the change's safety depends on, get it as far down this list as is cheap, and say where it stopped.

1. You said so. Worthless on its own.
2. You pointed at the line. A real `file:line`, or the library's own source.
3. You showed the bad case can't happen. You walked the failure step by step and it doesn't reach.
4. You ran it. A script or test that calls the real code and fails loud if you're wrong.
5. You reproduced it in the running app.

Any safety fact you can't get to step 4, say so out loud. Don't write it up as settled. Step 4 is usually one small script that imports the same library the app ships and calls the exact function you're worried about.

## Steps

1. Read the change. The diff, the symbols it adds, changes, and deletes, and what it now does differently, including the part the diff doesn't spell out. Before any code hunt, read what the knowledge bundle already holds on the touched area (`bee knowledge search --text "<area or symbol>"`, then open the matching concepts): a recorded pitfall is a risk someone already paid for. Then run the Provenance sweep of `bee-why` (`bee-researching/references/provenance.md`) over the same change to pull the decision log, the commits and the feature history.
2. Find the one fact it's safe because of. Most changes that look scary are safe because of a single fact, like "this call only drops already-dead cache entries and does nothing else". Find that fact. If it holds, most of the scary cases die at once. Spend your time here, not on a long list of maybes.
3. Look where grep stops. Read the source of the library you call, and check its pinned version and any local patch. Work out when things run: microtasks, unmount and teardown, one runtime versus another. Follow what a symbol search misses: the JSON an API returns, a DB column, a wire format, another language reading the same bytes, a feature flag, code three hops downstream. For this repo, add the files bee copies into a host project and the records it persists: a changed skill, hook, gate or record shape reaches every project that vendored the old one.
4. Be honest about each risk. Give it a real chance of happening and a real cost if it does. Keep the risks you confirmed; list the ones you checked and cleared separately. Same rules as `bee-why`. Cite a real `file:line`, a search that finds nothing is still an answer, and never make up a caller or an API.
5. Prove the one fact. Write a script or test that runs the real code, run it, and paste what happened. If you can't prove it cheaply, mark it unproven. Don't round up.
6. For a big or wide change, run it through `bee-arena`. Ask several runners the same question and merge the answers. Each runner goes out through `bee dispatch prepare --runtime claude --kind advisor --role lane-1 --json`, then `--role lane-2` and `--role lane-3`, each payload used exactly as returned. Never pick a model or a subagent type by hand: which model a lane runs on is a setting of the bee team table. Different models catch different real bugs.

## What to hand back

- **What it does.** What changed, including the part that isn't obvious.
- **The one fact it's safe because of.** State it, say which step you got it to, and show the proof. If you couldn't prove it, write unproven.
- **Risks.** Only the real ones. Each names how it breaks, the `file:line`, how likely and how bad, and how to check. Paste the proof for the ones that matter.
- **Cleared.** What you checked and why it's fine.
- **Before you merge.** The cheapest test or repro that catches the real bug, including the script you wrote.

Write it through `bee-unslop`, cite real code, and strip anything private before it goes anywhere public.

## Reply

The writeup above, with the one safety fact either proven or marked unproven. Read-only: the script you wrote to prove the fact is the only file you add, and no bee state changes.

## Handoff

Writeup delivered. Invoke bee-hive skill.
