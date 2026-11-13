---
id: refactoring-ui-craft
description: "Eight Refactoring UI craft rules bee-design lacks: de-emphasize competitors, drop data labels, actions by importance, non-proportional scaling, baseline alignment, tinted badges, native-size assets, grayscale first."
when: [hierarchy, buttons, badges, typography, responsive, icons, data-display, color, layout, screenshots]
---

# Refactoring UI craft

These rules sit beside [generation-craft-defaults.md](../references/generation-craft-defaults.md) and
[taste-rubric.md](../references/taste-rubric.md), never above them. The static ui gate checks none of
them; each one is checked by eye in the rendered page, and lands under `NOT_VERIFIED` in the hand-over
when it was not.

This file lives outside `references/` on purpose: `scripts/sync-design-os.sh` deletes and re-copies
`references/` on every sync, and these rules are bee's own.

## Rules

**de-emphasize-competitors** — when the primary element does not stand out, mute what competes with
it before adding emphasis to it. Lighten the colour and drop the weight of the surrounding elements
first; a bigger, bolder primary on a loud page only raises the noise.
Before: a bold dark nav, a bold dark sidebar and a bold dark page title all fight a bold blue
"Save". After: nav and sidebar items go to regular weight in a mid grey, and "Save" wins without
growing.
Check: squint at the render, or blur it. The primary element is the first thing that survives.

**drop-data-labels** — when a page displays data rather than collecting it, drop the `Label: value`
pair whose format or context already says what the value is, and lead with the identifying field.
Keep labels, a shade darker than the values, on spec sheets and dashboards that users scan by label.
Before: "Email: ada@example.com", "Phone: 555 0100", "Name: Ada Lovelace". After: the name as the
heading, then the address and the number on their own lines, each recognisable by its format.
Check: read each label aloud and ask whether the value alone would have told you the same. Where it
would, the label is noise.

**actions-by-importance** — style an action by its rank on the page: one solid primary, an outline or
low-contrast secondary, a plain link for the tertiary. A destructive action that is not the page's
main job stays secondary or tertiary until its confirm dialog, where it becomes the solid red
primary.
Before: "Save", "Cancel" and "Delete account" are three solid buttons, the last one red. After:
"Save" is solid, "Cancel" is an outline, "Delete account" is a plain link, and its dialog carries a
solid red "Delete account" beside a quiet "Keep account".
Check: count the solid buttons per view. More than one, or a red one outside a confirm dialog,
fails.

**non-proportional-scaling** — on narrow screens large elements shrink by a larger factor than small
ones. A heading at 2.5x body on desktop lands near 1.5x to 1.7x body on mobile. Never size headings
in `em` of the body size alone, because that keeps the desktop ratio at every width.
Before: `h1 { font-size: 2.5em }` renders 40px beside 16px body at 1440 and 375 px alike, and wraps
to four lines on a phone. After: the heading size is its own token per breakpoint, or a `clamp()`
whose floor sits near 1.6x body.
Check: open the page at 375, 768 and 1440 px. The ratio of the biggest heading to body text is
visibly smaller at 375.

**baseline-align-mixed-sizes** — text of different sizes on one line aligns on the text baseline
(`align-items: baseline`), never centred. A card title beside a "View all" link or a timestamp is the
usual case.
Before: a 24px title and a 14px link in a flex row with `align-items: center`; the link floats
above the title's baseline. After: the same row with `align-items: baseline`; the two share a
line of text.
Check: draw a line along the bottom of the letters in the render. Both texts sit on it.

**tinted-badges** — badges, pills and inline alerts take a light tint of the hue as background and a
dark shade of the same hue as text, instead of white text on a dark saturated fill. Contrast holds
and the badge stops stealing hierarchy from the primary action.
Before: a "Failed" pill in white on saturated red, louder than the page's own button. After: the
pill in pale red with dark red text, readable and quiet.
Check: measure the text against the tint, not against the page. Then check the badge is not the
loudest thing in its row.

**native-size-assets** — an icon drawn for 16-24px stays at its drawn size and fills a larger slot
with a tinted circle or rounded square behind it, never a 3x upscale. A product screenshot is never
shrunk whole into a small card; show a cropped region or a simplified mock instead.
Before: a 20px icon scaled to 64px, with thick, soft strokes; a full-window screenshot squeezed into
a 280px card until its text is unreadable. After: the 20px icon centred on a 56px tinted circle; the
card shows the one panel the caption talks about.
Check: look for strokes thicker than their neighbours' and for any text in an image too small to
read at the width it is shown.

**grayscale-first** — lay a new screen out in grayscale first, with hierarchy built only from
spacing, size, weight and contrast, then add colour to the primary action and the key states.
Before: a dashboard where the accent colour marks every heading, link and card edge, so nothing
leads. After: the same dashboard works in grey, and colour appears on the one button and on the
status chips.
Check: desaturate the render, for example with the `grayscale(1)` filter in the browser tools. The
hierarchy still reads.

## Not adopted

- **The accent border.** Refactoring UI uses a thick coloured border on one side of a card.
  [design-tells.md](../references/design-tells.md) bans it as the `side-tab` AI tell. A thin top bar
  on a card stays open; a thick one-side border does not.
- **Start with too much whitespace** as a bare rule. [generation-craft-defaults.md](../references/generation-craft-defaults.md)
  lists excessive whitespace as the `empty-boldness` failure mode. Extra space is kept only when it
  expresses grouping, priority or rhythm.

## Source

The ideas come from Refactoring UI by Adam Wathan and Steve Schoger, rewritten here in bee's words.
