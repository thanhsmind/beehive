pstack is a Cursor plugin for rigorous agent work. The same SKILL.md files also load in Claude Code, Codex, and other coding agents.

\~~~

AI coding agents can write a lot of code.

That is useful, but it is not the same as engineering.

The hard part is understanding the existing system, choosing a good design, verifying the result, reviewing the diff, and shipping it without breaking something else.

[pstack](https://github.com/cursor/plugins/tree/main/pstack) is a Cursor plugin built around that problem.

It was created by Lauren Tan, also known as [@poteto](https://x.com/poteto) on X. The stated goal is not more code. It is less code, higher quality, and enough verification that several agents can work in parallel without turning the repository into a mess.

pstack is much bigger than a collection of prompts.

It currently contains 23 workflow skills, 21 engineering principles, 22 task playbooks, 2 specialized subagents, helper programs, and an optional automation pack.

This is how the plugin is organized in the Cursor repository:

![The pstack directory in Cursor's plugins repository, with its agents, Benny automations, guide, and skills](https://flaviocopes.com/images/pstack/repository.webp)

But you do not need to memorize any of that.

Most of the time, you use one command:

```text
/poteto-mode
```

You describe the result you want. `poteto-mode` chooses a playbook, creates a task list, calls the other skills when needed, delegates work to suitable models, and demands evidence before it reports success.

This is the main idea behind pstack.

It turns a short request into a complete engineering workflow.

## Why pstack exists

Lauren explained the thinking behind pstack in her article [How I Use Cursor](https://x.com/poteto/status/2058975157503570132).

Before joining Cursor, she used Claude Code and started building her own orchestration layer around it. A simple CLI was easy to extend, but it also left the human coordinating every agent.

Cursor changed that model for her. She could switch models during a task, give subagents different models, compact context quickly, and use purpose-built interfaces such as the browser and Design Mode.

But the main lesson was not “run more agents.”

Lauren compares working with agents to managing an engineering team. New engineers need to learn the codebase and the way the team investigates, designs, tests, and communicates. Agents need the same guidance, except they keep forgetting it.

Rules, skills, tools, and memory provide that guidance.

This is why pstack goes deep before it goes broad. It turns repeated agent failures into explicit playbooks. The goal is to make one agent trustworthy on a complete problem before multiplying it across many tasks.

At launch, Lauren showed that her skills had been used 9,000 times inside Cursor in one week. pstack packages those internal habits into a public plugin.

## Does pstack work outside Cursor?

Yes. The official package is a Cursor plugin. The skills themselves are `SKILL.md` files.

That is the same format [Claude Code](https://docs.anthropic.com/en/docs/claude-code/skills), Codex, and other coding agents already load.

You can copy the skill folders into that tool’s skills directory.

If you want a ready-made Claude Code port, use [pstack-claude](https://github.com/michael-denyer/pstack-claude). It is not the official package. It translates Cursor-specific pieces to Claude Code equivalents, and it also ships a Codex plugin.

You lose the Cursor-only pieces:

- `/add-plugin`
- `/setup-pstack` writing `~/.cursor/rules/pstack-models.mdc`
- assigning a different model on each subagent
- `/loop`

The playbooks, principles, `/how`, `/why`, and `/interrogate` still make sense. They are instructions, not Cursor APIs.

Cursor remains the best fit. pstack wants different models for different jobs, and Cursor can assign those models on one task.

The rest of this article follows the official Cursor install. That is where pstack is maintained.

## Install pstack

pstack ships through the Cursor plugin system.

Open a Cursor chat and run:

```text
/add-plugin pstack
```

Cursor opens a plugin picker. Choose pstack:

![The Cursor chat plugin picker after typing /add-plugin pstack, with pstack highlighted](https://flaviocopes.com/images/pstack/add-plugin.webp)

Then configure the models pstack can use:

```text
/setup-pstack
```

The setup skill detects the models available in your Cursor account. It assigns them to roles such as implementation, investigation, judgment, and review.

The configuration is saved in:

```text
~/.cursor/rules/pstack-models.mdc
```

This is an override file. When a role is missing, pstack uses its built-in default.

You can also set a role to `auto` or `inherit-parent`. Both values tell pstack to use the model from the parent chat.

Panel roles accept several models. The number of models in the list becomes the number of reviewers or candidates pstack starts.

The bundled defaults split work by model strength. Precisely specified code goes to Sol. Fast mechanical work goes to Grok. Judgment and prose go to Fable. Review panels mix those models with Opus.

You do not have to keep those choices. That is why `/setup-pstack` exists.

After setup, start a new chat so the rule is loaded.

Then run your first task:

```text
/poteto-mode add a --json flag to this command. Keep the text output unchanged. Verify both forms against the sample project.
```

That prompt contains a goal and a way to check it.

This is exactly what pstack wants.

## The pstack architecture

`poteto-mode` is a router.

It does not contain every instruction needed for every task. It selects smaller pieces and runs them in the right order.

The complete flow looks like this:

<svg id="mermaid-1788252504180" width="100%" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" style="max-width: 276px;" viewBox="0 0 276 986.78125" role="graphics-document document" aria-roledescription="flowchart-v2"><g><marker id="mermaid-1788252504180_flowchart-v2-pointEnd" viewBox="0 0 10 10" refX="5" refY="5" markerUnits="userSpaceOnUse" markerWidth="8" markerHeight="8" orient="auto"><path d="M 0 0 L 10 5 L 0 10 z" style="stroke-width: 1; stroke-dasharray: 1, 0;"></path></marker><marker id="mermaid-1788252504180_flowchart-v2-pointStart" viewBox="0 0 10 10" refX="4.5" refY="5" markerUnits="userSpaceOnUse" markerWidth="8" markerHeight="8" orient="auto"><path d="M 0 5 L 10 10 L 10 0 z" style="stroke-width: 1; stroke-dasharray: 1, 0;"></path></marker><marker id="mermaid-1788252504180_flowchart-v2-pointEnd-margin" viewBox="0 0 11.5 14" refX="11.5" refY="7" markerUnits="userSpaceOnUse" markerWidth="10.5" markerHeight="14" orient="auto"><path d="M 0 0 L 11.5 7 L 0 14 z" style="stroke-width: 0; stroke-dasharray: 1, 0;"></path></marker><marker id="mermaid-1788252504180_flowchart-v2-pointStart-margin" viewBox="0 0 11.5 14" refX="1" refY="7" markerUnits="userSpaceOnUse" markerWidth="11.5" markerHeight="14" orient="auto"><polygon points="0,7 11.5,14 11.5,0" style="stroke-width: 0; stroke-dasharray: 1, 0;"></polygon></marker><marker id="mermaid-1788252504180_flowchart-v2-circleEnd" viewBox="0 0 10 10" refX="11" refY="5" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto"><circle cx="5" cy="5" r="5" style="stroke-width: 1; stroke-dasharray: 1, 0;"></circle></marker><marker id="mermaid-1788252504180_flowchart-v2-circleStart" viewBox="0 0 10 10" refX="-1" refY="5" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto"><circle cx="5" cy="5" r="5" style="stroke-width: 1; stroke-dasharray: 1, 0;"></circle></marker><marker id="mermaid-1788252504180_flowchart-v2-circleEnd-margin" viewBox="0 0 10 10" refY="5" refX="12.25" markerUnits="userSpaceOnUse" markerWidth="14" markerHeight="14" orient="auto"><circle cx="5" cy="5" r="5" style="stroke-width: 0; stroke-dasharray: 1, 0;"></circle></marker><marker id="mermaid-1788252504180_flowchart-v2-circleStart-margin" viewBox="0 0 10 10" refX="-2" refY="5" markerUnits="userSpaceOnUse" markerWidth="14" markerHeight="14" orient="auto"><circle cx="5" cy="5" r="5" style="stroke-width: 0; stroke-dasharray: 1, 0;"></circle></marker><marker id="mermaid-1788252504180_flowchart-v2-crossEnd" viewBox="0 0 11 11" refX="12" refY="5.2" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto"><path d="M 1,1 l 9,9 M 10,1 l -9,9" style="stroke-width: 2; stroke-dasharray: 1, 0;"></path></marker><marker id="mermaid-1788252504180_flowchart-v2-crossStart" viewBox="0 0 11 11" refX="-1" refY="5.2" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto"><path d="M 1,1 l 9,9 M 10,1 l -9,9" style="stroke-width: 2; stroke-dasharray: 1, 0;"></path></marker><marker id="mermaid-1788252504180_flowchart-v2-crossEnd-margin" viewBox="0 0 15 15" refX="17.7" refY="7.5" markerUnits="userSpaceOnUse" markerWidth="12" markerHeight="12" orient="auto"><path d="M 1,1 L 14,14 M 1,14 L 14,1" style="stroke-width: 2.5;"></path></marker><marker id="mermaid-1788252504180_flowchart-v2-crossStart-margin" viewBox="0 0 15 15" refX="-3.5" refY="7.5" markerUnits="userSpaceOnUse" markerWidth="12" markerHeight="12" orient="auto"><path d="M 1,1 L 14,14 M 1,14 L 14,1" style="stroke-width: 2.5; stroke-dasharray: 1, 0;"></path></marker><g><g></g><g><path d="M138,61.797L138,65.964C138,70.13,138,78.464,138,86.13C138,93.797,138,100.797,138,104.297L138,107.797" id="mermaid-1788252504180-L_A_B_0" style=";" data-edge="true" data-et="edge" data-id="L_A_B_0" data-points="W3sieCI6MTM4LCJ5Ijo2MS43OTY4NzV9LHsieCI6MTM4LCJ5Ijo4Ni43OTY4NzV9LHsieCI6MTM4LCJ5IjoxMTEuNzk2ODc1fV0=" data-look="classic" marker-end="url(#mermaid-1788252504180_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M138,165.594L138,169.76C138,173.927,138,182.26,138,189.927C138,197.594,138,204.594,138,208.094L138,211.594" id="mermaid-1788252504180-L_B_C_0" style=";" data-edge="true" data-et="edge" data-id="L_B_C_0" data-points="W3sieCI6MTM4LCJ5IjoxNjUuNTkzNzV9LHsieCI6MTM4LCJ5IjoxOTAuNTkzNzV9LHsieCI6MTM4LCJ5IjoyMTUuNTkzNzV9XQ==" data-look="classic" marker-end="url(#mermaid-1788252504180_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M138,293.188L138,297.354C138,301.521,138,309.854,138,317.521C138,325.188,138,332.188,138,335.688L138,339.188" id="mermaid-1788252504180-L_C_D_0" style=";" data-edge="true" data-et="edge" data-id="L_C_D_0" data-points="W3sieCI6MTM4LCJ5IjoyOTMuMTg3NX0seyJ4IjoxMzgsInkiOjMxOC4xODc1fSx7IngiOjEzOCwieSI6MzQzLjE4NzV9XQ==" data-look="classic" marker-end="url(#mermaid-1788252504180_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M138,539.797L138,543.964C138,548.13,138,556.464,138,564.13C138,571.797,138,578.797,138,582.297L138,585.797" id="mermaid-1788252504180-L_D_E_0" style=";" data-edge="true" data-et="edge" data-id="L_D_E_0" data-points="W3sieCI6MTM4LCJ5Ijo1MzkuNzk2ODc1fSx7IngiOjEzOCwieSI6NTY0Ljc5Njg3NX0seyJ4IjoxMzgsInkiOjU4OS43OTY4NzV9XQ==" data-look="classic" marker-end="url(#mermaid-1788252504180_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M138,643.594L138,647.76C138,651.927,138,660.26,138,667.927C138,675.594,138,682.594,138,686.094L138,689.594" id="mermaid-1788252504180-L_E_F_0" style=";" data-edge="true" data-et="edge" data-id="L_E_F_0" data-points="W3sieCI6MTM4LCJ5Ijo2NDMuNTkzNzV9LHsieCI6MTM4LCJ5Ijo2NjguNTkzNzV9LHsieCI6MTM4LCJ5Ijo2OTMuNTkzNzV9XQ==" data-look="classic" marker-end="url(#mermaid-1788252504180_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M138,747.391L138,751.557C138,755.724,138,764.057,138,771.724C138,779.391,138,786.391,138,789.891L138,793.391" id="mermaid-1788252504180-L_F_G_0" style=";" data-edge="true" data-et="edge" data-id="L_F_G_0" data-points="W3sieCI6MTM4LCJ5Ijo3NDcuMzkwNjI1fSx7IngiOjEzOCwieSI6NzcyLjM5MDYyNX0seyJ4IjoxMzgsInkiOjc5Ny4zOTA2MjV9XQ==" data-look="classic" marker-end="url(#mermaid-1788252504180_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M138,874.984L138,879.151C138,883.318,138,891.651,138,899.318C138,906.984,138,913.984,138,917.484L138,920.984" id="mermaid-1788252504180-L_G_H_0" style=";" data-edge="true" data-et="edge" data-id="L_G_H_0" data-points="W3sieCI6MTM4LCJ5Ijo4NzQuOTg0Mzc1fSx7IngiOjEzOCwieSI6ODk5Ljk4NDM3NX0seyJ4IjoxMzgsInkiOjkyNC45ODQzNzV9XQ==" data-look="classic" marker-end="url(#mermaid-1788252504180_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path></g><g><g><g data-id="L_A_B_0" transform="translate(0, 0)"></g></g><g><g data-id="L_B_C_0" transform="translate(0, 0)"></g></g><g><g data-id="L_C_D_0" transform="translate(0, 0)"></g></g><g><g data-id="L_D_E_0" transform="translate(0, 0)"></g></g><g><g data-id="L_E_F_0" transform="translate(0, 0)"></g></g><g><g data-id="L_F_G_0" transform="translate(0, 0)"></g></g><g><g data-id="L_G_H_0" transform="translate(0, 0)"></g></g></g><g><g id="mermaid-1788252504180-flowchart-A-0" data-look="classic" transform="translate(138, 34.8984375)"><rect style="" x="-80.40625" y="-26.8984375" width="160.8125" height="53.796875" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-50.40625, -11.8984375)"><rect></rect><foreignObject width="100.8125" height="23.796875"><p>Your request</p></foreignObject></g></g><g id="mermaid-1788252504180-flowchart-B-1" data-look="classic" transform="translate(138, 138.6953125)"><rect style="" x="-76.203125" y="-26.8984375" width="152.40625" height="53.796875" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-46.203125, -11.8984375)"><rect></rect><foreignObject width="92.40625" height="23.796875"><p>poteto-mode</p></foreignObject></g></g><g id="mermaid-1788252504180-flowchart-C-3" data-look="classic" transform="translate(138, 254.390625)"><rect style="" x="-130" y="-38.796875" width="260" height="77.59375" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-100, -23.796875)"><rect></rect><foreignObject width="200" height="47.59375"><p>Read the principles index</p></foreignObject></g></g><g id="mermaid-1788252504180-flowchart-D-5" data-look="classic" transform="translate(138, 441.4921875)"><polygon points="98.3046875,0 196.609375,-98.3046875 98.3046875,-196.609375 0,-98.3046875" transform="translate(-97.8046875, 98.3046875)" fill="none" stroke="currentColor"></polygon><g style="" transform="translate(-71.40625, -11.8984375)"><rect></rect><foreignObject width="142.8125" height="23.796875"><p>Choose a playbook</p></foreignObject></g></g><g id="mermaid-1788252504180-flowchart-E-7" data-look="classic" transform="translate(138, 616.6953125)"><rect style="" x="-122.40625" y="-26.8984375" width="244.8125" height="53.796875" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-92.40625, -11.8984375)"><rect></rect><foreignObject width="184.8125" height="23.796875"><p>Call specialist skills</p></foreignObject></g></g><g id="mermaid-1788252504180-flowchart-F-9" data-look="classic" transform="translate(138, 720.4921875)"><rect style="" x="-122.40625" y="-26.8984375" width="244.8125" height="53.796875" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-92.40625, -11.8984375)"><rect></rect><foreignObject width="184.8125" height="23.796875"><p>Delegate by model role</p></foreignObject></g></g><g id="mermaid-1788252504180-flowchart-G-11" data-look="classic" transform="translate(138, 836.1875)"><rect style="" x="-130" y="-38.796875" width="260" height="77.59375" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-100, -23.796875)"><rect></rect><foreignObject width="200" height="47.59375"><p>Inspect and verify the result</p></foreignObject></g></g><g id="mermaid-1788252504180-flowchart-H-13" data-look="classic" transform="translate(138, 951.8828125)"><rect style="" x="-126.6015625" y="-26.8984375" width="253.203125" height="53.796875" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-96.6015625, -11.8984375)"><rect></rect><foreignObject width="193.203125" height="23.796875"><p>Clean, review, and ship</p></foreignObject></g></g></g></g></g><defs></defs><defs></defs><linearGradient id="mermaid-1788252504180-gradient" gradientUnits="objectBoundingBox" x1="0%" y1="0%" x2="100%" y2="0%"><stop offset="0%" stop-color="rgb(80 80 80)" stop-opacity="1"></stop><stop offset="100%" stop-color="hsl(0, 0%, 1.7647058824%)" stop-opacity="1"></stop></linearGradient></svg>

The first task-list item is always reading the principles index inside the [`poteto-mode` skill](https://github.com/cursor/plugins/tree/main/pstack/skills/poteto-mode).

Next, it matches the request to a playbook.

A bug goes to the Bug fix playbook. New behavior goes to Feature. A structural change goes to Refactoring. A question goes to Investigation. A measured slowdown goes to Perf issue.

The matched playbook is copied into the task list verbatim.

This detail matters.

The model does not read a playbook and then improvise a shorter plan that quietly drops half the checks. Every named step remains visible. When pstack skips something, the task list keeps the step and records the reason.

`poteto-mode` is also sticky.

After you enter the mode, normal follow-up messages stay inside it. You can type `continue`, `do it`, or `keep going until done`. The current playbook and conversation provide the missing context.

When you change subjects, say `new task` so pstack matches a fresh playbook:

```text
/poteto-mode new task. Find out why the cache entry survives logout. Do not change code yet.
```

The last sentence keeps the work read-only.

## What the playbooks do

pstack has 22 user-facing playbooks. There is also an internal Opening a PR playbook used by the others.

The easiest way to understand them is to group them by job.

### Understand before changing

The Investigation playbook handles read-only questions.

It routes a question through `/how`, and through `/why` when the question involves history or intent.

```text
/poteto-mode how does this notification retry work? Are we doing one subscriber query for every notification?
```

The result is an architectural explanation, not a diff.

For unfamiliar systems, this is the right starting point. An agent that edits the first plausible function often fixes a symptom. An agent that traces the runtime path has a chance to find the real boundary.

### Build and change code

The main code playbooks are Bug fix, Feature, Refactoring, Perf issue, Hillclimb, Prototype, and Visual parity.

They do not share one generic checklist.

A bug must be reproduced before the fix. The agent forms competing causes, rules them out with runtime evidence, and verifies the original reproduction on the same interface afterward.

A feature starts with `/how`, runs `/architect` when the design deserves it, delegates implementation, reviews the diff, and verifies the result on the real interface.

A refactoring first records existing behavior. It might use a characterization test, snapshot, or equivalence script. Then it changes structure in small steps while keeping that behavior check green.

A performance task starts with a trace. It compares a baseline with the result after the change. “It feels faster” does not count.

Hillclimb is for improving one metric over several attempts. Each attempt states a hypothesis, measures the result, keeps the win, and discards the loss.

Prototype builds the smallest throwaway artifact needed to make a decision. Visual parity starts with screenshots and treats a nonzero pixel difference as a failure.

### Diagnose without fixing

Runtime forensics and Trace forensics stop at the diagnosis.

Runtime forensics captures a live signal. That might be a CPU profile, heap snapshot, or browser trace.

Trace forensics starts with an artifact someone already captured. It turns large trace data into something queryable, narrows it to the costly frame or retention path, and maps the finding back to source code.

The playbook does not quietly turn a diagnosis request into a fix. Once the cause is known, you can start a new Bug fix or Perf issue task.

### Keep long work moving

The long-running playbooks include Autonomous run, Multi-phase plan, Orchestrate, Autopilot-full, and Autopilot-stack.

They operate at different scales.

Autonomous run drives one task until a checkable condition passes.

Autopilot-full runs a queue of independent pull requests through verification and merge. Autopilot-stack creates one reviewed Graphite stack but leaves the final landing to the human.

Orchestrate is the heavy option. It is for a project that lasts several days, creates many stacked pull requests, and needs a standing coordinator plus a fleet of agents.

pstack is careful about this distinction. A long task is not automatically a program. If one agent can finish the work in a session, Orchestrate is too much.

### Pick work back up safely

Session pickup reconstructs a previous agent’s state from its transcript, branch, and decision log. It identifies what is done, what remains, and where the next agent should resume.

Pause safely does the opposite. It stops at an atomic boundary, makes the current work durable, and writes a resume note.

These playbooks prevent an expensive failure mode. A new agent should not redo three hours of completed work because it did not know where the last one stopped.

### Maintain the delivery pipeline

Babysit drives a pull request or stack to merge-ready. It checks conflicts first, reports any required rebase, then handles review threads and CI.

Shipping is separate. It verifies each pull request with a fresh agent, checks that old verdicts still describe the current commit, and lands only the contiguous verified part of a stack.

This separation is deliberate.

A green pull request is ready for a merge decision. It is not permission to merge.

## /how, /why, /teach, and /recall

The most useful pstack skills might be the ones that do not write code.

### /how traces the current system

Use `/how` when you need to understand runtime behavior, ownership, or architecture:

```text
/how how does the rate limiter work?
```

For a narrow question, one explainer reads the code and answers.

For a larger subsystem, pstack splits the exploration into two to four parts. One agent might trace the data model. Another follows the request path. A third reads configuration and metrics.

An explainer then combines the findings into one account.

The output focuses on the concepts, runtime flow, relevant files, and sharp edges. It is a mental model, not annotated source code.

`/how` also has a critique mode. It explains the system first, then several models review the architecture.

That ordering avoids generic architecture advice. The critics receive a traced system, not a filename and a guess.

### /why looks for historical evidence

Code can tell us what a function does.

It rarely tells us why the team chose that shape.

`/why` starts from Git history and pull requests. It then discovers the external sources available through Cursor MCP connections.

It can search seven evidence categories:

- source control
- issues and tickets
- long-form documents
- team chat
- infrastructure monitoring
- error tracking
- product analytics

One investigator owns each available category. A final model combines the evidence and keeps direct facts separate from inference.

Empty searches are reported too. If no ticket or design document explains a choice, the reader should know that.

This is slower than reading the code and inventing a plausible reason.

It is also much more honest.

### /teach combines mechanics and history

`/teach` sits above `/how` and `/why`.

It is for the moment when a list of files and functions is not enough:

```text
/teach me how this pull request changes retries. Convince me it fixes the cause instead of the symptom.
```

The skill builds a plain explanation of what the system is, how it works, and why it has that shape.

### /recall rebuilds your own context

`/recall` searches recent Cursor transcripts from the current workspace. It combines that history with current Git and pull-request state.

Use it when you return to a topic after a few days:

```text
/recall catch me up on the export work from last week
```

The result is a short brief with completed work, active threads, recurring problems, and the next useful move.

## Design with /architect and /arena

AI agents tend to start implementing too soon.

pstack tries to settle the shape first.

### /architect starts from the caller

`/architect` has five phases:

1. ground the problem
2. sketch several shapes
3. agree when you requested a checkpoint
4. implement against the sketch
5. scrap the sketch when repeated friction proves it wrong

Grounding runs `/how` over the surrounding system. If the change moves ownership or crosses layers, it may also run `/why`.

The sketch starts with caller usage. Types, function signatures, and module boundaries follow from that usage.

This is a good constraint. A design can look elegant inside its own file while being awkward for every caller.

By default, `/architect` continues into implementation. Add `with checkpoint` when you want to approve the design first:

```text
/architect with checkpoint. Design the import pipeline before writing code.
```

The last phase is just as important as the first.

If implementation keeps adding casts, optional fields that are always present, repeated exceptions, or parameters the sketch never anticipated, pstack treats that friction as evidence. It throws the shape away and designs again.

### /arena runs competing attempts

`/arena` gives the same task to several models.

Each candidate writes to its own worktree or temporary directory. Each also explains the alternatives it considered and rejected.

The coordinator creates a private rubric. A separate model judges every candidate against it. Meanwhile, the coordinator reads every result from start to finish.

Then it picks one candidate as the base and folds in the strongest ideas from the others.

<svg id="mermaid-1788252504241" width="100%" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" style="max-width: 1247.640625px;" viewBox="0 0 1247.640625 277.390625" role="graphics-document document" aria-roledescription="flowchart-v2"><g><marker id="mermaid-1788252504241_flowchart-v2-pointEnd" viewBox="0 0 10 10" refX="5" refY="5" markerUnits="userSpaceOnUse" markerWidth="8" markerHeight="8" orient="auto"><path d="M 0 0 L 10 5 L 0 10 z" style="stroke-width: 1; stroke-dasharray: 1, 0;"></path></marker><marker id="mermaid-1788252504241_flowchart-v2-pointStart" viewBox="0 0 10 10" refX="4.5" refY="5" markerUnits="userSpaceOnUse" markerWidth="8" markerHeight="8" orient="auto"><path d="M 0 5 L 10 10 L 10 0 z" style="stroke-width: 1; stroke-dasharray: 1, 0;"></path></marker><marker id="mermaid-1788252504241_flowchart-v2-pointEnd-margin" viewBox="0 0 11.5 14" refX="11.5" refY="7" markerUnits="userSpaceOnUse" markerWidth="10.5" markerHeight="14" orient="auto"><path d="M 0 0 L 11.5 7 L 0 14 z" style="stroke-width: 0; stroke-dasharray: 1, 0;"></path></marker><marker id="mermaid-1788252504241_flowchart-v2-pointStart-margin" viewBox="0 0 11.5 14" refX="1" refY="7" markerUnits="userSpaceOnUse" markerWidth="11.5" markerHeight="14" orient="auto"><polygon points="0,7 11.5,14 11.5,0" style="stroke-width: 0; stroke-dasharray: 1, 0;"></polygon></marker><marker id="mermaid-1788252504241_flowchart-v2-circleEnd" viewBox="0 0 10 10" refX="11" refY="5" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto"><circle cx="5" cy="5" r="5" style="stroke-width: 1; stroke-dasharray: 1, 0;"></circle></marker><marker id="mermaid-1788252504241_flowchart-v2-circleStart" viewBox="0 0 10 10" refX="-1" refY="5" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto"><circle cx="5" cy="5" r="5" style="stroke-width: 1; stroke-dasharray: 1, 0;"></circle></marker><marker id="mermaid-1788252504241_flowchart-v2-circleEnd-margin" viewBox="0 0 10 10" refY="5" refX="12.25" markerUnits="userSpaceOnUse" markerWidth="14" markerHeight="14" orient="auto"><circle cx="5" cy="5" r="5" style="stroke-width: 0; stroke-dasharray: 1, 0;"></circle></marker><marker id="mermaid-1788252504241_flowchart-v2-circleStart-margin" viewBox="0 0 10 10" refX="-2" refY="5" markerUnits="userSpaceOnUse" markerWidth="14" markerHeight="14" orient="auto"><circle cx="5" cy="5" r="5" style="stroke-width: 0; stroke-dasharray: 1, 0;"></circle></marker><marker id="mermaid-1788252504241_flowchart-v2-crossEnd" viewBox="0 0 11 11" refX="12" refY="5.2" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto"><path d="M 1,1 l 9,9 M 10,1 l -9,9" style="stroke-width: 2; stroke-dasharray: 1, 0;"></path></marker><marker id="mermaid-1788252504241_flowchart-v2-crossStart" viewBox="0 0 11 11" refX="-1" refY="5.2" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto"><path d="M 1,1 l 9,9 M 10,1 l -9,9" style="stroke-width: 2; stroke-dasharray: 1, 0;"></path></marker><marker id="mermaid-1788252504241_flowchart-v2-crossEnd-margin" viewBox="0 0 15 15" refX="17.7" refY="7.5" markerUnits="userSpaceOnUse" markerWidth="12" markerHeight="12" orient="auto"><path d="M 1,1 L 14,14 M 1,14 L 14,1" style="stroke-width: 2.5;"></path></marker><marker id="mermaid-1788252504241_flowchart-v2-crossStart-margin" viewBox="0 0 15 15" refX="-3.5" refY="7.5" markerUnits="userSpaceOnUse" markerWidth="12" markerHeight="12" orient="auto"><path d="M 1,1 L 14,14 M 1,14 L 14,1" style="stroke-width: 2.5; stroke-dasharray: 1, 0;"></path></marker><g><g></g><g><path d="M99.855,111.797L111.314,98.98C122.773,86.164,145.691,60.531,160.65,47.715C175.609,34.898,182.609,34.898,186.109,34.898L189.609,34.898" id="mermaid-1788252504241-L_A_B_0" style=";" data-edge="true" data-et="edge" data-id="L_A_B_0" data-points="W3sieCI6OTkuODU0NTU0OTU5MTY3NTQsInkiOjExMS43OTY4NzV9LHsieCI6MTY4LjYwOTM3NSwieSI6MzQuODk4NDM3NX0seyJ4IjoxOTMuNjA5Mzc1LCJ5IjozNC44OTg0Mzc1fV0=" data-look="classic" marker-end="url(#mermaid-1788252504241_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M143.609,138.695L147.776,138.695C151.943,138.695,160.276,138.695,167.943,138.695C175.609,138.695,182.609,138.695,186.109,138.695L189.609,138.695" id="mermaid-1788252504241-L_A_C_0" style=";" data-edge="true" data-et="edge" data-id="L_A_C_0" data-points="W3sieCI6MTQzLjYwOTM3NSwieSI6MTM4LjY5NTMxMjV9LHsieCI6MTY4LjYwOTM3NSwieSI6MTM4LjY5NTMxMjV9LHsieCI6MTkzLjYwOTM3NSwieSI6MTM4LjY5NTMxMjV9XQ==" data-look="classic" marker-end="url(#mermaid-1788252504241_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M99.855,165.594L111.314,178.41C122.773,191.227,145.691,216.859,160.65,229.676C175.609,242.492,182.609,242.492,186.109,242.492L189.609,242.492" id="mermaid-1788252504241-L_A_D_0" style=";" data-edge="true" data-et="edge" data-id="L_A_D_0" data-points="W3sieCI6OTkuODU0NTU0OTU5MTY3NTQsInkiOjE2NS41OTM3NX0seyJ4IjoxNjguNjA5Mzc1LCJ5IjoyNDIuNDkyMTg3NX0seyJ4IjoxOTMuNjA5Mzc1LCJ5IjoyNDIuNDkyMTg3NX1d" data-look="classic" marker-end="url(#mermaid-1788252504241_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M346.016,34.898L350.182,34.898C354.349,34.898,362.682,34.898,381.942,47.292C401.201,59.685,431.387,84.472,446.48,96.865L461.573,109.258" id="mermaid-1788252504241-L_B_E_0" style=";" data-edge="true" data-et="edge" data-id="L_B_E_0" data-points="W3sieCI6MzQ2LjAxNTYyNSwieSI6MzQuODk4NDM3NX0seyJ4IjozNzEuMDE1NjI1LCJ5IjozNC44OTg0Mzc1fSx7IngiOjQ2NC42NjQzMzE4MTU0NDQ4LCJ5IjoxMTEuNzk2ODc1fV0=" data-look="classic" marker-end="url(#mermaid-1788252504241_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M346.016,138.695L350.182,138.695C354.349,138.695,362.682,138.695,370.349,138.695C378.016,138.695,385.016,138.695,388.516,138.695L392.016,138.695" id="mermaid-1788252504241-L_C_E_0" style=";" data-edge="true" data-et="edge" data-id="L_C_E_0" data-points="W3sieCI6MzQ2LjAxNTYyNSwieSI6MTM4LjY5NTMxMjV9LHsieCI6MzcxLjAxNTYyNSwieSI6MTM4LjY5NTMxMjV9LHsieCI6Mzk2LjAxNTYyNSwieSI6MTM4LjY5NTMxMjV9XQ==" data-look="classic" marker-end="url(#mermaid-1788252504241_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M346.016,242.492L350.182,242.492C354.349,242.492,362.682,242.492,381.942,230.099C401.201,217.706,431.387,192.919,446.48,180.526L461.573,168.132" id="mermaid-1788252504241-L_D_E_0" style=";" data-edge="true" data-et="edge" data-id="L_D_E_0" data-points="W3sieCI6MzQ2LjAxNTYyNSwieSI6MjQyLjQ5MjE4NzV9LHsieCI6MzcxLjAxNTYyNSwieSI6MjQyLjQ5MjE4NzV9LHsieCI6NDY0LjY2NDMzMTgxNTQ0NDgsInkiOjE2NS41OTM3NX1d" data-look="classic" marker-end="url(#mermaid-1788252504241_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M598.828,138.695L602.995,138.695C607.161,138.695,615.495,138.695,623.161,138.695C630.828,138.695,637.828,138.695,641.328,138.695L644.828,138.695" id="mermaid-1788252504241-L_E_F_0" style=";" data-edge="true" data-et="edge" data-id="L_E_F_0" data-points="W3sieCI6NTk4LjgyODEyNSwieSI6MTM4LjY5NTMxMjV9LHsieCI6NjIzLjgyODEyNSwieSI6MTM4LjY5NTMxMjV9LHsieCI6NjQ4LjgyODEyNSwieSI6MTM4LjY5NTMxMjV9XQ==" data-look="classic" marker-end="url(#mermaid-1788252504241_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M801.234,138.695L805.401,138.695C809.568,138.695,817.901,138.695,825.568,138.695C833.234,138.695,840.234,138.695,843.734,138.695L847.234,138.695" id="mermaid-1788252504241-L_F_G_0" style=";" data-edge="true" data-et="edge" data-id="L_F_G_0" data-points="W3sieCI6ODAxLjIzNDM3NSwieSI6MTM4LjY5NTMxMjV9LHsieCI6ODI2LjIzNDM3NSwieSI6MTM4LjY5NTMxMjV9LHsieCI6ODUxLjIzNDM3NSwieSI6MTM4LjY5NTMxMjV9XQ==" data-look="classic" marker-end="url(#mermaid-1788252504241_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M1079.234,138.695L1083.401,138.695C1087.568,138.695,1095.901,138.695,1103.568,138.695C1111.234,138.695,1118.234,138.695,1121.734,138.695L1125.234,138.695" id="mermaid-1788252504241-L_G_H_0" style=";" data-edge="true" data-et="edge" data-id="L_G_H_0" data-points="W3sieCI6MTA3OS4yMzQzNzUsInkiOjEzOC42OTUzMTI1fSx7IngiOjExMDQuMjM0Mzc1LCJ5IjoxMzguNjk1MzEyNX0seyJ4IjoxMTI5LjIzNDM3NSwieSI6MTM4LjY5NTMxMjV9XQ==" data-look="classic" marker-end="url(#mermaid-1788252504241_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path></g><g><g><g data-id="L_A_B_0" transform="translate(0, 0)"></g></g><g><g data-id="L_A_C_0" transform="translate(0, 0)"></g></g><g><g data-id="L_A_D_0" transform="translate(0, 0)"></g></g><g><g data-id="L_B_E_0" transform="translate(0, 0)"></g></g><g><g data-id="L_C_E_0" transform="translate(0, 0)"></g></g><g><g data-id="L_D_E_0" transform="translate(0, 0)"></g></g><g><g data-id="L_E_F_0" transform="translate(0, 0)"></g></g><g><g data-id="L_F_G_0" transform="translate(0, 0)"></g></g><g><g data-id="L_G_H_0" transform="translate(0, 0)"></g></g></g><g><g id="mermaid-1788252504241-flowchart-A-0" data-look="classic" transform="translate(75.8046875, 138.6953125)"><rect style="" x="-67.8046875" y="-26.8984375" width="135.609375" height="53.796875" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-37.8046875, -11.8984375)"><rect></rect><foreignObject width="75.609375" height="23.796875"><p>One brief</p></foreignObject></g></g><g id="mermaid-1788252504241-flowchart-B-1" data-look="classic" transform="translate(269.8125, 34.8984375)"><rect style="" x="-76.203125" y="-26.8984375" width="152.40625" height="53.796875" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-46.203125, -11.8984375)"><rect></rect><foreignObject width="92.40625" height="23.796875"><p>Candidate A</p></foreignObject></g></g><g id="mermaid-1788252504241-flowchart-C-3" data-look="classic" transform="translate(269.8125, 138.6953125)"><rect style="" x="-76.203125" y="-26.8984375" width="152.40625" height="53.796875" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-46.203125, -11.8984375)"><rect></rect><foreignObject width="92.40625" height="23.796875"><p>Candidate B</p></foreignObject></g></g><g id="mermaid-1788252504241-flowchart-D-5" data-look="classic" transform="translate(269.8125, 242.4921875)"><rect style="" x="-76.203125" y="-26.8984375" width="152.40625" height="53.796875" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-46.203125, -11.8984375)"><rect></rect><foreignObject width="92.40625" height="23.796875"><p>Candidate C</p></foreignObject></g></g><g id="mermaid-1788252504241-flowchart-E-7" data-look="classic" transform="translate(497.421875, 138.6953125)"><rect style="" x="-101.40625" y="-26.8984375" width="202.8125" height="53.796875" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-71.40625, -11.8984375)"><rect></rect><foreignObject width="142.8125" height="23.796875"><p>Cross-model judge</p></foreignObject></g></g><g id="mermaid-1788252504241-flowchart-F-13" data-look="classic" transform="translate(725.03125, 138.6953125)"><rect style="" x="-76.203125" y="-26.8984375" width="152.40625" height="53.796875" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-46.203125, -11.8984375)"><rect></rect><foreignObject width="92.40625" height="23.796875"><p>Pick a base</p></foreignObject></g></g><g id="mermaid-1788252504241-flowchart-G-15" data-look="classic" transform="translate(965.234375, 138.6953125)"><rect style="" x="-114" y="-26.8984375" width="228" height="53.796875" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-84, -11.8984375)"><rect></rect><foreignObject width="168" height="23.796875"><p>Fold in useful ideas</p></foreignObject></g></g><g id="mermaid-1788252504241-flowchart-H-17" data-look="classic" transform="translate(1184.4375, 138.6953125)"><rect style="" x="-55.203125" y="-26.8984375" width="110.40625" height="53.796875" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-25.203125, -11.8984375)"><rect></rect><foreignObject width="50.40625" height="23.796875"><p>Verify</p></foreignObject></g></g></g></g></g><defs></defs><defs></defs><linearGradient id="mermaid-1788252504241-gradient" gradientUnits="objectBoundingBox" x1="0%" y1="0%" x2="100%" y2="0%"><stop offset="0%" stop-color="rgb(80 80 80)" stop-opacity="1"></stop><stop offset="100%" stop-color="hsl(0, 0%, 1.7647058824%)" stop-opacity="1"></stop></linearGradient></svg>

This is not a vote.

One candidate can win while another contributes a better error model or a smaller interface.

If every candidate converges on the same shape, that agreement is useful evidence. If they diverge wildly, pstack treats the prompt as underspecified and runs the arena again with a clearer brief.

## /swarm is different from /arena

Both skills start several agents. They solve different problems.

`/arena` repeats the same task. It compares the results and produces one synthesized artifact.

`/swarm` splits a task into independent slices or declared race arms. It waits for every worker and returns one report.

For example:

```text
/swarm check every package under packages/ against its check.sh. One worker per package. One report.
```

Each worker returns `PASS`, `ISSUES`, or `BLOCKED` with evidence.

Use Arena when you want competing designs.

Use Swarm when you want coverage.

## Multi-model review with /interrogate

`/interrogate` sends the same diff, intent, and review rules to several models.

It does not assign theatrical personas. Model diversity provides the different perspectives.

The lead reviewer merges duplicate findings, notes where models agree, and places every point into one of four groups:

- act on
- consider
- noted
- dismissed

The dismissed section is part of the result.

Review agents produce noise. Showing what the lead rejected, and why, lets you override the judgment instead of receiving a mysterious filtered list.

The skill never applies changes automatically.

Run it when a diff is ready to attack:

```text
/interrogate review the whole branch. No nitpicks unless they reveal a bug or regression.
```

## The 21 principles

pstack includes 21 small principle skills.

`poteto-mode` keeps a short index of them in its own file. It reads that index at the start of multi-step work. When a task triggers a principle, it can open the complete skill and apply it.

Some principles reduce code:

- Laziness Protocol prefers deletion and the smallest complete change.
- Subtract Before You Add removes dead paths before introducing a new design.
- Minimize Reader Load reduces layers and hidden state.

Some shape architecture:

- Model the Domain replaces scattered conditions with one explicit structure.
- Boundary Discipline validates external data at the edge and keeps internal logic clean.
- Type System Discipline makes invalid states hard to represent.
- Make Operations Idempotent makes retries converge on the same result.

Some define proof:

- Prove It Works checks the real artifact.
- Fix Root Causes reproduces the symptom and follows it to the mechanism.
- Sequence Work into Verifiable Units ends each small step with a check.

The delegation rules are practical too.

Guard the Context Window sends bulk reading to subagents and keeps summaries in the main chat. Separate Before Serializing Shared State gives parallel writers separate worktrees instead of adding locks around one shared directory.

You do not invoke these principles as commands.

You use their names when you need to steer the current run:

```text
Apply prove it works. Run the real import flow and inspect the records it writes.
```

The reply must name the decision the principle changed. Merely repeating the principle name does not count.

## Verification is a first-class part of the workflow

pstack rejects “the build passed” as complete evidence.

The verification should match the thing that changed:

- a command-line change runs the real command
- a UI change walks the changed flow
- a migration replays real input
- a performance change compares traces
- a storage change reads the value back

When a repository has no reliable way to do that, pstack can create one:

```text
/create-verification-skill
```

The skill inspects the repository and writes a project-local `verify-<app>` skill.

The generated skill has exact instructions for five jobs:

1. launch the application
2. check that the instance is healthy
3. drive the user-facing behavior
4. capture evidence
5. clean up only what the verification started

It also creates a feature map. Each feature records how a user reaches it, how an agent drives it, and what observable state proves it works.

Before handing the skill over, pstack runs it once from start to finish.

There is a maintenance skill too:

```text
/maintain-verification-skill
```

It compares every mapped feature with the current source, then runs one live pass. It can update the verification skill, but it cannot hide a product bug by editing the documentation.

I like this part of pstack a lot.

“Verify it” becomes a repository capability instead of a new conversation every time.

## Run pstack while you sleep

Long autonomous work needs a finish condition.

“Work on this for four hours” measures motion.

“Stop when there are zero old callers and every parser fixture passes” measures a result.

A complete overnight request can look like this:

```text
/poteto-mode I am going to bed. Migrate every caller to the new parser in a fresh worktree.
Done means zero old callers, every parser fixture passes, and the old API is deleted.
Keep a decision log. Do not ask before committing.
/loop until done. If you reach a real dead end, stop and explain it.
```

`/loop` is a Cursor command, not a pstack skill. It wakes the task on an event or a timed heartbeat.

Each iteration follows the same pattern:

<svg id="mermaid-1788252504294" width="100%" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" style="max-width: 441.1090087890625px;" viewBox="0 0 441.1090087890625 786.171875" role="graphics-document document" aria-roledescription="flowchart-v2"><g><marker id="mermaid-1788252504294_flowchart-v2-pointEnd" viewBox="0 0 10 10" refX="5" refY="5" markerUnits="userSpaceOnUse" markerWidth="8" markerHeight="8" orient="auto"><path d="M 0 0 L 10 5 L 0 10 z" style="stroke-width: 1; stroke-dasharray: 1, 0;"></path></marker><marker id="mermaid-1788252504294_flowchart-v2-pointStart" viewBox="0 0 10 10" refX="4.5" refY="5" markerUnits="userSpaceOnUse" markerWidth="8" markerHeight="8" orient="auto"><path d="M 0 5 L 10 10 L 10 0 z" style="stroke-width: 1; stroke-dasharray: 1, 0;"></path></marker><marker id="mermaid-1788252504294_flowchart-v2-pointEnd-margin" viewBox="0 0 11.5 14" refX="11.5" refY="7" markerUnits="userSpaceOnUse" markerWidth="10.5" markerHeight="14" orient="auto"><path d="M 0 0 L 11.5 7 L 0 14 z" style="stroke-width: 0; stroke-dasharray: 1, 0;"></path></marker><marker id="mermaid-1788252504294_flowchart-v2-pointStart-margin" viewBox="0 0 11.5 14" refX="1" refY="7" markerUnits="userSpaceOnUse" markerWidth="11.5" markerHeight="14" orient="auto"><polygon points="0,7 11.5,14 11.5,0" style="stroke-width: 0; stroke-dasharray: 1, 0;"></polygon></marker><marker id="mermaid-1788252504294_flowchart-v2-circleEnd" viewBox="0 0 10 10" refX="11" refY="5" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto"><circle cx="5" cy="5" r="5" style="stroke-width: 1; stroke-dasharray: 1, 0;"></circle></marker><marker id="mermaid-1788252504294_flowchart-v2-circleStart" viewBox="0 0 10 10" refX="-1" refY="5" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto"><circle cx="5" cy="5" r="5" style="stroke-width: 1; stroke-dasharray: 1, 0;"></circle></marker><marker id="mermaid-1788252504294_flowchart-v2-circleEnd-margin" viewBox="0 0 10 10" refY="5" refX="12.25" markerUnits="userSpaceOnUse" markerWidth="14" markerHeight="14" orient="auto"><circle cx="5" cy="5" r="5" style="stroke-width: 0; stroke-dasharray: 1, 0;"></circle></marker><marker id="mermaid-1788252504294_flowchart-v2-circleStart-margin" viewBox="0 0 10 10" refX="-2" refY="5" markerUnits="userSpaceOnUse" markerWidth="14" markerHeight="14" orient="auto"><circle cx="5" cy="5" r="5" style="stroke-width: 0; stroke-dasharray: 1, 0;"></circle></marker><marker id="mermaid-1788252504294_flowchart-v2-crossEnd" viewBox="0 0 11 11" refX="12" refY="5.2" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto"><path d="M 1,1 l 9,9 M 10,1 l -9,9" style="stroke-width: 2; stroke-dasharray: 1, 0;"></path></marker><marker id="mermaid-1788252504294_flowchart-v2-crossStart" viewBox="0 0 11 11" refX="-1" refY="5.2" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto"><path d="M 1,1 l 9,9 M 10,1 l -9,9" style="stroke-width: 2; stroke-dasharray: 1, 0;"></path></marker><marker id="mermaid-1788252504294_flowchart-v2-crossEnd-margin" viewBox="0 0 15 15" refX="17.7" refY="7.5" markerUnits="userSpaceOnUse" markerWidth="12" markerHeight="12" orient="auto"><path d="M 1,1 L 14,14 M 1,14 L 14,1" style="stroke-width: 2.5;"></path></marker><marker id="mermaid-1788252504294_flowchart-v2-crossStart-margin" viewBox="0 0 15 15" refX="-3.5" refY="7.5" markerUnits="userSpaceOnUse" markerWidth="12" markerHeight="12" orient="auto"><path d="M 1,1 L 14,14 M 1,14 L 14,1" style="stroke-width: 2.5; stroke-dasharray: 1, 0;"></path></marker><g><g></g><g><path d="M234.532,85.594L229.144,89.76C223.756,93.927,212.98,102.26,207.591,109.927C202.203,117.594,202.203,124.594,202.203,128.094L202.203,131.594" id="mermaid-1788252504294-L_A_B_0" style=";" data-edge="true" data-et="edge" data-id="L_A_B_0" data-points="W3sieCI6MjM0LjUzMjI5NDcyODE0MTA3LCJ5Ijo4NS41OTM3NX0seyJ4IjoyMDIuMjAzMTI1LCJ5IjoxMTAuNTkzNzV9LHsieCI6MjAyLjIwMzEyNSwieSI6MTM1LjU5Mzc1fV0=" data-look="classic" marker-end="url(#mermaid-1788252504294_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M202.203,213.188L202.203,217.354C202.203,221.521,202.203,229.854,202.203,237.521C202.203,245.188,202.203,252.188,202.203,255.688L202.203,259.188" id="mermaid-1788252504294-L_B_C_0" style=";" data-edge="true" data-et="edge" data-id="L_B_C_0" data-points="W3sieCI6MjAyLjIwMzEyNSwieSI6MjEzLjE4NzV9LHsieCI6MjAyLjIwMzEyNSwieSI6MjM4LjE4NzV9LHsieCI6MjAyLjIwMzEyNSwieSI6MjYzLjE4NzV9XQ==" data-look="classic" marker-end="url(#mermaid-1788252504294_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M202.203,316.984L202.203,321.151C202.203,325.318,202.203,333.651,202.203,341.318C202.203,348.984,202.203,355.984,202.203,359.484L202.203,362.984" id="mermaid-1788252504294-L_C_D_0" style=";" data-edge="true" data-et="edge" data-id="L_C_D_0" data-points="W3sieCI6MjAyLjIwMzEyNSwieSI6MzE2Ljk4NDM3NX0seyJ4IjoyMDIuMjAzMTI1LCJ5IjozNDEuOTg0Mzc1fSx7IngiOjIwMi4yMDMxMjUsInkiOjM2Ni45ODQzNzV9XQ==" data-look="classic" marker-end="url(#mermaid-1788252504294_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M162.3,506.878L152.083,519.678C141.866,532.478,121.433,558.079,111.217,576.362C101,594.646,101,605.612,101,611.095L101,616.578" id="mermaid-1788252504294-L_D_E_0" style=";" data-edge="true" data-et="edge" data-id="L_D_E_0" data-points="W3sieCI6MTYyLjI5OTYwMzkxMzk1OTcxLCJ5Ijo1MDYuODc3NzI4OTEzOTU5N30seyJ4IjoxMDEsInkiOjU4My42Nzk2ODc1fSx7IngiOjEwMSwieSI6NjIwLjU3ODEyNX1d" data-look="classic" marker-end="url(#mermaid-1788252504294_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M242.107,506.878L252.323,519.678C262.54,532.478,282.973,558.079,293.19,576.362C303.406,594.646,303.406,605.612,303.406,611.095L303.406,616.578" id="mermaid-1788252504294-L_D_F_0" style=";" data-edge="true" data-et="edge" data-id="L_D_F_0" data-points="W3sieCI6MjQyLjEwNjY0NjA4NjA0MDI5LCJ5Ijo1MDYuODc3NzI4OTEzOTU5N30seyJ4IjozMDMuNDA2MjUsInkiOjU4My42Nzk2ODc1fSx7IngiOjMwMy40MDYyNSwieSI6NjIwLjU3ODEyNX1d" data-look="classic" marker-end="url(#mermaid-1788252504294_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M101,674.375L101,678.542C101,682.708,101,691.042,116.604,699.209C132.209,707.377,163.418,715.379,179.022,719.38L194.626,723.382" id="mermaid-1788252504294-L_E_G_0" style=";" data-edge="true" data-et="edge" data-id="L_E_G_0" data-points="W3sieCI6MTAxLCJ5Ijo2NzQuMzc1fSx7IngiOjEwMSwieSI6Njk5LjM3NX0seyJ4IjoxOTguNTAxMTI5MDA3OTc4MywieSI6NzI0LjM3NX1d" data-look="classic" marker-end="url(#mermaid-1788252504294_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M303.406,674.375L303.406,678.542C303.406,682.708,303.406,691.042,303.406,698.708C303.406,706.375,303.406,713.375,303.406,716.875L303.406,720.375" id="mermaid-1788252504294-L_F_G_0" style=";" data-edge="true" data-et="edge" data-id="L_F_G_0" data-points="W3sieCI6MzAzLjQwNjI1LCJ5Ijo2NzQuMzc1fSx7IngiOjMwMy40MDYyNSwieSI6Njk5LjM3NX0seyJ4IjozMDMuNDA2MjUsInkiOjcyNC4zNzV9XQ==" data-look="classic" marker-end="url(#mermaid-1788252504294_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path><path d="M370.63,724.375L381.043,720.208C391.457,716.042,412.283,707.708,422.696,694.892C433.109,682.076,433.109,664.776,433.109,645.493C433.109,626.211,433.109,604.945,433.109,573.18C433.109,541.414,433.109,499.148,433.109,458.866C433.109,418.583,433.109,380.284,433.109,352.484C433.109,324.685,433.109,307.385,433.109,290.086C433.109,272.786,433.109,255.487,433.109,236.204C433.109,216.922,433.109,195.656,433.109,174.391C433.109,153.125,433.109,131.859,424.029,117.323C414.949,102.787,396.789,94.98,387.709,91.077L378.628,87.173" id="mermaid-1788252504294-L_G_A_0" style=";" data-edge="true" data-et="edge" data-id="L_G_A_0" data-points="W3sieCI6MzcwLjYzMDA3MzQ3OTYwMjYsInkiOjcyNC4zNzV9LHsieCI6NDMzLjEwOTM3NSwieSI6Njk5LjM3NX0seyJ4Ijo0MzMuMTA5Mzc1LCJ5Ijo2NDcuNDc2NTYyNX0seyJ4Ijo0MzMuMTA5Mzc1LCJ5Ijo1ODMuNjc5Njg3NX0seyJ4Ijo0MzMuMTA5Mzc1LCJ5Ijo0NTYuODgyODEyNX0seyJ4Ijo0MzMuMTA5Mzc1LCJ5IjozNDEuOTg0Mzc1fSx7IngiOjQzMy4xMDkzNzUsInkiOjI5MC4wODU5Mzc1fSx7IngiOjQzMy4xMDkzNzUsInkiOjIzOC4xODc1fSx7IngiOjQzMy4xMDkzNzUsInkiOjE3NC4zOTA2MjV9LHsieCI6NDMzLjEwOTM3NSwieSI6MTEwLjU5Mzc1fSx7IngiOjM3NC45NTM2MDcxODIyMTg5NSwieSI6ODUuNTkzNzV9XQ==" data-look="classic" marker-end="url(#mermaid-1788252504294_flowchart-v2-pointEnd)" fill="none" stroke="currentColor"></path></g><g><g><g data-id="L_A_B_0" transform="translate(0, 0)"></g></g><g><g data-id="L_B_C_0" transform="translate(0, 0)"></g></g><g><g data-id="L_C_D_0" transform="translate(0, 0)"></g></g><g transform="translate(101, 583.6796875)"><g data-id="L_D_E_0" transform="translate(-12.6015625, -11.8984375)"><foreignObject width="25.203125" height="23.796875"><p>Yes</p></foreignObject></g></g><g transform="translate(303.40625, 583.6796875)"><g data-id="L_D_F_0" transform="translate(-8.40625, -11.8984375)"><foreignObject width="16.8125" height="23.796875"><p>No</p></foreignObject></g></g><g><g data-id="L_E_G_0" transform="translate(0, 0)"></g></g><g><g data-id="L_F_G_0" transform="translate(0, 0)"></g></g><g><g data-id="L_G_A_0" transform="translate(0, 0)"></g></g></g><g><g id="mermaid-1788252504294-flowchart-A-0" data-look="classic" transform="translate(284.703125, 46.796875)"><rect style="" x="-130" y="-38.796875" width="260" height="77.59375" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-100, -23.796875)"><rect></rect><foreignObject width="200" height="47.59375"><p>Check the finish condition</p></foreignObject></g></g><g id="mermaid-1788252504294-flowchart-B-1" data-look="classic" transform="translate(202.203125, 174.390625)"><rect style="" x="-130" y="-38.796875" width="260" height="77.59375" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-100, -23.796875)"><rect></rect><foreignObject width="200" height="47.59375"><p>Make one justified change</p></foreignObject></g></g><g id="mermaid-1788252504294-flowchart-C-3" data-look="classic" transform="translate(202.203125, 290.0859375)"><rect style="" x="-122.40625" y="-26.8984375" width="244.8125" height="53.796875" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-92.40625, -11.8984375)"><rect></rect><foreignObject width="184.8125" height="23.796875"><p>Verify the real result</p></foreignObject></g></g><g id="mermaid-1788252504294-flowchart-D-5" data-look="classic" transform="translate(202.203125, 456.8828125)"><polygon points="89.8984375,0 179.796875,-89.8984375 89.8984375,-179.796875 0,-89.8984375" transform="translate(-89.3984375, 89.8984375)" fill="none" stroke="currentColor"></polygon><g style="" transform="translate(-63, -11.8984375)"><rect></rect><foreignObject width="126" height="23.796875"><p>Did it improve?</p></foreignObject></g></g><g id="mermaid-1788252504294-flowchart-E-7" data-look="classic" transform="translate(101, 647.4765625)"><rect style="" x="-93" y="-26.8984375" width="186" height="53.796875" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-63, -11.8984375)"><rect></rect><foreignObject width="126" height="23.796875"><p>Keep and commit</p></foreignObject></g></g><g id="mermaid-1788252504294-flowchart-F-9" data-look="classic" transform="translate(303.40625, 647.4765625)"><rect style="" x="-59.40625" y="-26.8984375" width="118.8125" height="53.796875" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-29.40625, -11.8984375)"><rect></rect><foreignObject width="58.8125" height="23.796875"><p>Discard</p></foreignObject></g></g><g id="mermaid-1788252504294-flowchart-G-11" data-look="classic" transform="translate(303.40625, 751.2734375)"><rect style="" x="-122.40625" y="-26.8984375" width="244.8125" height="53.796875" fill="none" stroke="currentColor"></rect><g style="" transform="translate(-92.40625, -11.8984375)"><rect></rect><foreignObject width="184.8125" height="23.796875"><p>Write one decision row</p></foreignObject></g></g></g></g></g><defs></defs><defs></defs><linearGradient id="mermaid-1788252504294-gradient" gradientUnits="objectBoundingBox" x1="0%" y1="0%" x2="100%" y2="0%"><stop offset="0%" stop-color="rgb(80 80 80)" stop-opacity="1"></stop><stop offset="100%" stop-color="hsl(0, 0%, 1.7647058824%)" stop-opacity="1"></stop></linearGradient></svg>

The decision log is a TSV file.

Each row records the time, phase, decision, reason, evidence, and result. It is local by default. A large project can commit it when reviewers need the trail to trust the work.

If Git branches and worktrees are new to you, my [free Git course](https://flaviocopes.com/courses/git/) explains the model behind them.

## What pstack does not include

Some advanced pstack workflows refer to tools from other places.

`/deslop`, `control-cli`, and `control-ui` come from the separate `cursor-team-kit` plugin.

`/create-skill`, `/babysit`, and `/loop` are Cursor built-ins. Inside `poteto-mode`, the pstack Babysit playbook replaces the built-in babysitting flow for pull-request status work.

The advanced shipping playbooks also assume GitHub and Graphite.

This does not affect the basic `/poteto-mode` workflow. It matters when you expect every cleanup, UI-control, stacked-pull-request, and overnight feature to work from pstack alone.

The repository also contains a dormant automation pack named Benny.

Benny can triage Slack issue reports, reproduce confirmed problems, and fix them with UI evidence. The automation files are not registered as slash skills. Setup copies them into a target repository when you explicitly enable the pack.

Lauren’s article explains what the complete Benny pipeline is meant to do.

One Benny automation starts with triage. It reads a bug report and its image or video attachments, inspects the relevant code, and asks the reporter for clearer reproduction steps when needed.

It then checks Git history, Slack discussions, and Notion decisions. That extra context helps it distinguish a regression from behavior that was designed intentionally.

After it creates a ticket, another Benny automation picks it up through `/orchestrate`. It tries to reproduce the problem with computer use before changing code. For performance problems, it can capture before-and-after CPU traces and heap snapshots.

Fresh workers verify the fix against the ticket. Other workers record before-and-after videos and open a pull request with the evidence.

This is still a work in progress, but it shows the larger idea behind pstack. A software factory starts with trust in one complete loop: understand, reproduce, fix, verify, and show the result.

## Make pstack yours

`poteto-mode` encodes Lauren’s engineering style.

pstack does not pretend that style is universal.

Run this to create a personal mode:

```text
/automate-me
```

The skill reads recent Cursor transcripts from the current workspace. It looks for repeated preferences in delegation, verification, code, prose, and process.

Then it asks which patterns are really yours and creates:

```text
.cursor/skills/<your-name>-mode/SKILL.md
```

Use `/reflect` after a difficult task when you want to improve an existing skill.

`/reflect` sends the transcript to several reviewers. A synthesizer sorts their proposals into accepted, rejected, and backlog. Nothing changes until you approve it.

That last guard matters.

One strange task should not become a permanent rule for every future task.

## How I would use pstack

I would use pstack for work where a plausible diff is not enough.

A good example is the purchase webhook on this site.

One purchase can generate more than one Paddle delivery. The important question is not whether an `if` statement compiles. I need to trace both payloads, prove which path sends the email, reproduce the duplicate behavior, and verify one welcome email is sent after the fix.

I would start with this:

```text
/poteto-mode the purchase webhook can send two welcome emails for one purchase.
Reproduce both Paddle deliveries first. Trace the cause, fix it, and verify one email is sent.
Do not change the fulfillment behavior for a valid purchase.
```

That prompt gives pstack a symptom, a required reproduction, a finish condition, and behavior to preserve.

I would also use pstack for a large migration, a hard performance problem, or an overnight run with a result I can test in the morning.

I would not use the complete workflow for every change.

Moving one publication date, correcting one sentence, or changing a small configuration value does not need several models, an architecture arena, a decision log, and a verification skill.

The machinery has a cost.

pstack can start several agents for one task. If they all use frontier models, the tokens add up fast.

I would use Composer 2.5 for routine work and keep frontier models for the difficult parts. `/setup-pstack` lets me choose a model for each role.

Long playbooks also require me to trust the routing rules.

For small work, I prefer the shorter loop in [fstack](https://flaviocopes.com/fstack/). It keeps the human close to each decision and aims for the smallest process that works.

pstack is for the other end of the spectrum. It is designed for deeper investigation, adversarial review, real runtime proof, and autonomous work that must remain auditable.

## A practical first workflow

Do not start by memorizing all 44 skills.

Install pstack, run setup, and choose one real task.

Most of the time, `/poteto-mode` is enough. I can learn the direct commands when I need them.

Use this shape:

```text
/poteto-mode <what you observed or what you want>
Done means <something the agent can run or inspect>.
Keep <existing behavior that must not change>.
```

Then watch the task list.

You should see the principles read first, a matching playbook copied into place, and specialist skills called when their step arrives.

After the first run, try the direct skills that solve a question you have:

```text
/how how does this subsystem work?
```
```text
/why why was this limit chosen?
```
```text
/interrogate review this diff
```

And when the final explanation is accurate but unreadable:

```text
/bro
```

That skill rewrites the last reply in plain language.

## Common mistakes

The [official pstack guide](https://github.com/cursor/plugins/tree/main/pstack/docs/guide) calls out several mistakes worth avoiding.

Do not list every skill you want pstack to run. State the goal and constraints. The playbook owns the sequence.

Do not give an autonomous run a vague finish condition. The loop needs a result it can check.

Do not let parallel writers share one working directory. Give each one an isolated worktree or output path.

Do not use `/arena` when you need coverage. Use `/swarm`. Arena compares several answers to one brief. Swarm divides the work.

Do not treat `auto` as a model name. It tells the subagent to inherit the parent model.

Do not accept a green build as proof of behavior. Run the command, drive the interface, inspect the record, or compare the trace.

Do not accept every review finding. Read what the lead dismissed and decide whether the reason holds.

And do not reach for Orchestrate because a task sounds large. Use it when the work is genuinely a multi-day program with many independent units.

## My take

pstack is ambitious.

It tries to turn Cursor into a small engineering organization. There is a coordinator, specialist investigators, implementation agents, competing architects, skeptical reviewers, verification workers, and a shipping process.

That can look like too much ceremony.

Sometimes it is.

But the best ideas in pstack do not depend on using the whole system.

Reproduce before fixing. Start from caller usage. Compare designs when the decision is expensive. Use different models for review. Keep parallel writers isolated. Verify the real artifact. Give autonomous work a finish condition. Preserve a decision trail when you will review the work later.

Those are solid engineering habits with or without AI.

The clever part is that pstack turns them into executable workflows.

You can still ignore every individual skill and type `/poteto-mode`.

That is the right place to start.

Tagged: [AI](https://flaviocopes.com/tags/ai/) · [All topics](https://flaviocopes.com/blog/#topics)

<iframe frameborder="0" title="Add Preferred Source" src="https://news.google.com/swg/ui/v1/addpreferredsourcebuttoniframe?_=1788252504117&amp;origin=https%3A%2F%2Fflaviocopes.com&amp;source=https%3A%2F%2Fflaviocopes.com%2Fpstack%2F&amp;theme=light&amp;hl=en&amp;publicationId=publication-id-free"></iframe>

Want me to talk about your product? You can [sponsor this site](https://flaviocopes.com/sponsor/).

\~~~

Related posts about ai:
