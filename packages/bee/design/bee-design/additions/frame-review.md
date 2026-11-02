---
id: frame-review
description: "The brief for the fresh-context critic in the Check step: judge the frames beside the closest gold example, quote a frame for every finding, rank by how much of the screen it spoils."
when: [critique, review, critic, frames, screenshots, gold, taste, check]
---

# Frame review

Give this file to the critic in the Check step. It is how the critic looks; what the critic scores is
in [taste-rubric.md](../references/taste-rubric.md), and the critic's duties are rule 14 of
[rules.md](../references/rules.md). Review of a design change on a pull request is
[design-review.md](../references/design-review.md), not this file.

This file lives outside `references/` on purpose: `scripts/sync-design-os.sh` deletes and re-copies
`references/` on every sync, and this brief is bee's own.

## What the critic gets

- the frames or screenshots of the page, at each width that was shot;
- the page URL, to shoot a frame that is missing;
- [taste-rubric.md](../references/taste-rubric.md);
- the closest gold example, with its still beside the page's.

The gold example is the shipped product or benchmark nearest to this brief: the one the brief names,
or the nearest one from the reference duel in [taste-rubric.md](../references/taste-rubric.md).

The critic never gets the builder's plan, its reasons, or the build conversation. The critic judges
the picture, not what the builder meant.

## First question

Before any rubric item: does this screen answer this product and this page, or could it be any product
in its category? A layout the brief did not shape is a finding, ranked with framing, even when every
check passes.

Then hold the screen to the brief's own words. When the brief names a page color, a size or a look,
check that first.

## Beside the gold

Put the page beside the closest gold example and name the one difference that matters most. Say which
of the two is sparser, flatter or smaller, and why.

## The bar for a finding

- A finding needs a frame. Quote the frame file or the screenshot that shows it.
- A predicted failure is not a finding, and a predicted pass is not a pass. When the frame you need
  does not exist, shoot it from the URL, then answer.
- A choice the builder calls intentional is not settled. Look at the frame and judge it again.
- Do not fix anything. Do not soften a finding because the page is nearly done.

## Quick tells

Each of these five shapes is a finding when a frame shows it, even when no rubric line names it:

- **Too much** — every area full, no rest, the eye cannot find where to start.
- **Too little** — empty panes, rows with more air between them than they are tall, a frame mostly
  background.
- **Boxed in a card** — the product sits in a small card or window with even margins, like a
  screenshot pasted on a page.
- **Flat** — no focal point; every element has the same weight, and the eye lands on gray bars.
- **Bland** — nothing on the screen says which brand this is; change the logo and it is any product.

## The list you return

Rank findings by how much of the screen they spoil, not by how easy they are to fix:

1. framing
2. hierarchy and density
3. motion
4. polish

Each finding gives three things: the frame that shows it, what is wrong, and the fix in one line.

Then say what you could not check, and why. When there is nothing to find, say "nothing" and list the
frames you looked at.

## Source

Adapted from Stageshow by Benji Taylor (MIT, https://github.com/benjitaylor/skills), its
`references/review.md`, rewritten in bee's words for rendered UI in general. The scene kit and the
device rules of that skill were not taken; its frame scripts live in the motion pack, at
`bee-design-motion/additions/frame-rehearsal.md`.
