---
id: refactoring-ui-craft
description: "Twenty-five Refactoring UI craft rules bee-design lacks, for hierarchy, layout and type, colour, depth and images, plus a symptom lookup, overcorrections and not-a-finding list for reviewing a rendered page."
when: [hierarchy, buttons, badges, typography, responsive, icons, data-display, color, layout, screenshots, review, symptoms, depth, shadows, images]
---

# Refactoring UI craft

These rules sit beside [generation-craft-defaults.md](../references/generation-craft-defaults.md) and
[taste-rubric.md](../references/taste-rubric.md), never above them. The static ui gate checks none of
them; each one is checked by eye in the rendered page, and lands under `NOT_VERIFIED` in the hand-over
when it was not.

This file lives outside `references/` on purpose: `scripts/sync-design-os.sh` deletes and re-copies
`references/` on every sync, and these rules are bee's own.

## Rules

### Hierarchy

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

**actions-by-importance** — style an action by its rank on the page: one solid primary, an outline
or low-contrast secondary, a plain link for the tertiary. A destructive action that is not the
page's main job stays secondary or tertiary until its confirm dialog, where it becomes the solid red
primary.
Before: "Save", "Cancel" and "Delete account" are three solid buttons, the last one red. After:
"Save" is solid, "Cancel" is an outline, "Delete account" is a plain link, and its dialog carries a
solid red "Delete account" beside a quiet "Keep account".
Check: count the solid buttons per view. More than one, or a red one outside a confirm dialog,
fails.

**labels-are-secondary** — a label that has to stay is smaller and lighter than the value it names,
or it is folded into the value. The value is what the reader came for.
Before: "Bedrooms: 3" with label and value in the same size and weight. After: "3 bedrooms", or a
small grey "Bedrooms" above a large "3".
Check: look at each label and value pair. The value is the first thing read, and the label is never
the louder of the two.

**three-text-colors-two-weights** — text uses three colours, primary, secondary and tertiary, and
two weight bands: 400-500 for body and 600-700 for emphasis. Nothing under 400 carries UI text,
because thin strokes break up on small sizes and low-density screens.
Before: headings in 300, captions in 200 and four different greys across one card. After: headings
in 600, body in 400, and only three text colours, each with one job: the main reading, the
supporting line, the faint hint.
Check: list the distinct text colours and font weights in the render. More than three colours, more
than two weight bands, or any weight under 400 fails.

**color-never-alone** — status, change and error states carry an icon, a shape or a word besides
their colour. Colour only reinforces what the mark already says, so a colour-blind user and a
greyscale print lose nothing.
Before: a table where overdue rows differ from paid rows only by a red or green number. After: the
overdue number gains a warning icon and the word "Overdue"; the colour stays as a second cue.
Check: desaturate the render. Every state is still told apart.

### Layout and type

**non-proportional-scaling** — on narrow screens large elements shrink by a larger factor than small
ones. A heading at 2.5x body on desktop lands near 1.5x to 1.7x body on mobile. Never size headings
in `em` of the body size alone, because that keeps the desktop ratio at every width.
Before: `h1 { font-size: 2.5em }` renders 40px beside 16px body at 1440 and 375 px alike, and wraps
to four lines on a phone. After: the heading size is its own token per breakpoint, or a `clamp()`
whose floor sits near 1.6x body.
Check: open the page at 375, 768 and 1440 px. The ratio of the biggest heading to body text is
visibly smaller at 375.

**baseline-align-mixed-sizes** — text of different sizes on one line aligns on the text baseline
(`align-items: baseline`), never centred. A card title beside a "View all" link or a timestamp is
the usual case.
Before: a 24px title and a 14px link in a flex row with `align-items: center`; the link floats above
the title's baseline. After: the same row with `align-items: baseline`; the two share a line of
text.
Check: draw a line along the bottom of the letters in the render. Both texts sit on it.

**columns-before-width** — content that would pass its comfortable width splits into columns or
gains a sidebar rather than stretching across the canvas. Dense tables are the exception and keep
the room their columns need.
Before: a settings form stretched to 1400px with 900px-wide inputs. After: the form at 640px beside
a sidebar of help text; the data table below it still spans the full width.
Check: resize to a wide viewport. No text block, input or card is wider than it needs to be, and no
table is squeezed.

**measure-inside-wide-containers** — a paragraph keeps 45-75 characters per line even when its card
or column is wider. Items stacked in one column may therefore have different widths.
Before: a paragraph filling a 960px card, 130 characters per line. After: the card stays 960px, the
paragraph sits at `max-width: 65ch`, and the image above it still spans the card.
Check: count characters on a long line of a paragraph. Outside 45-75 fails, whatever its container.

**leading-follows-measure** — line-height rises toward 1.8-2 as lines get longer and falls toward
1.0-1.2 as type grows. One universal 1.5 is wrong at both ends.
Before: a 70-character paragraph and a 48px headline both at 1.5, the headline's lines drifting
apart. After: the paragraph at 1.7, the headline at 1.1.
Check: compare line-height with line length and size. Wide text with tight leading, or a large
heading with body leading, fails.

**center-only-short-text** — centred text stops at two or three lines. A longer block aligns to the
reading direction, because a ragged left edge on both sides makes the eye hunt for each line start.
Before: a four-paragraph feature description centred under its heading. After: the heading stays
centred and the paragraphs align start inside a narrower block.
Check: count the lines of every centred block. More than three fails.

**numbers-align-right** — numeric columns in a table align right and use tabular figures, so digits
line up by place and totals compare at a glance.
Before: prices left-aligned in proportional digits, "9.99" above "1,249.00". After: both
right-aligned with `font-variant-numeric: tabular-nums` and the same decimals.
Check: draw a vertical line through the decimal points of a numeric column. They line up.

### Color

**tinted-badges** — badges, pills and inline alerts take a light tint of the hue as background and a
dark shade of the same hue as text, instead of white text on a dark saturated fill. Contrast holds
and the badge stops stealing hierarchy from the primary action.
Before: a "Failed" pill in white on saturated red, louder than the page's own button. After: the
pill in pale red with dark red text, readable and quiet.
Check: measure the text against the tint, not against the page. Then check the badge is not the
loudest thing in its row.

**grayscale-first** — lay a new screen out in grayscale first, with hierarchy built only from
spacing, size, weight and contrast, then add colour to the primary action and the key states.
Before: a dashboard where the accent colour marks every heading, link and card edge, so nothing
leads. After: the same dashboard works in grey, and colour appears on the one button and on the
status chips.
Check: desaturate the render, for example with the `grayscale(1)` filter in the browser tools. The
hierarchy still reads.

**no-faded-text** — secondary text takes a chosen lighter colour, never a lower opacity. Opacity
lets the background bleed through and the text turns muddy; on a coloured surface the chosen colour
is a lighter or darker shade of that surface's hue, never grey.
Before: a caption at `opacity: .5` on a white card, and grey text on a blue banner. After: the
caption in a fixed mid-grey, and the banner text in a pale blue picked from the same ramp.
Check: search the styles for `opacity` and grey text values on text elements. Text that is faint by
opacity, or grey on a colour, fails.

**thicken-not-darken** — when a border is too faint to separate two areas, make it wider and keep
its soft colour. A darker hairline turns a page into a grid of dark lines.
Before: a 1px light grey card edge that vanishes, replaced by a 1px near-black one that outlines
everything. After: a 2px border in the same light grey.
Check: scan the render for dark hairlines, then ask whether a thicker light border would do the same
job.

**colored-text-on-color** — text on a coloured surface uses a lighter or darker shade of that
surface's hue. When contrast falls short, shift the hue toward a brighter neighbour rather than
reaching for white at reduced opacity.
Before: white text at 60% opacity on a teal panel, which reads washed out. After: a pale teal chosen
from the same ramp, with the hue nudged slightly toward green until it clears contrast.
Check: measure text against its own background. Then confirm the text colour is a shade of the
surface, not grey or translucent white.

**no-new-shades** — once a colour ramp exists, adjust its shades instead of adding new ones at the
point of use. No inline `lighten()` or `darken()`, no one-off hex beside the ramp.
Before: a hover state written as `darken($blue, 7%)` and a card tint written as `#e8f0fd` beside a
ramp that already has both near neighbours. After: hover uses blue-700 and the tint uses blue-100;
if neither is right, the ramp entry itself is tuned.
Check: search the styles for colour functions and for hex values missing from the tokens.

### Depth

**light-from-above** — one light source sits above the page, so shadows fall down with zero
horizontal offset. A raised element may carry a slightly lighter top edge; an inset element takes
its shadow at the top.
Before: a card with `box-shadow: 4px 2px 8px` and an input with a drop shadow below it. After: the
card with `0 4px 8px`, a faint light top edge, and the input with `inset 0 2px 4px`.
Check: find the shadows in the render. Each has a downward offset and no sideways one, and wells
read as sunk.

**two-part-shadows** — an elevated element combines a large soft offset shadow with a tight dark
ambient shadow. The ambient part fades as the element rises, because contact darkens less the
farther the object floats.
Before: a dialog with one shadow, `0 20px 40px`, that looks pasted on. After: the dialog with `0
20px 40px` plus a faint `0 2px 4px`, and a low card with a strong ambient and a small soft shadow.
Check: compare a low and a high element. The higher one has the softer, fainter ambient shadow.

**color-carries-depth** — in a flat design with no shadows, a surface lighter than its background
reads raised and a darker one reads inset. Depth comes from the colour step instead.
Before: a flat panel and the page in the same grey, so the panel looks painted on. After: the panel
a step lighter than the page, and its search field a step darker than the panel.
Check: look at the flat render with shadows off. Raised and sunken areas are still told apart.

### Images

**native-size-assets** — an icon drawn for 16-24px stays at its drawn size and fills a larger slot
with a tinted circle or rounded square behind it, never a 3x upscale. A product screenshot is never
shrunk whole into a small card; show a cropped region or a simplified mock instead.
Before: a 20px icon scaled to 64px, with thick, soft strokes; a full-window screenshot squeezed into
a 280px card until its text is unreadable. After: the 20px icon centred on a 56px tinted circle; the
card shows the one panel the caption talks about.
Check: look for strokes thicker than their neighbours' and for any text in an image too small to
read at the width it is shown.

**image-text-overlay** — text over an image gets a dark overlay under light text and a light overlay
under dark text. Other routes: lower the image contrast and recover its brightness, or desaturate it
and tint it with a multiply fill. A soft zero-offset text glow is the last resort.
Before: white headline over a bright sky photo, readable on the left and lost on the right. After: a
50% black overlay under the headline, or the photo desaturated with a multiply fill of the brand
colour.
Check: measure text contrast at the brightest and the darkest spot behind it.

**contain-user-uploads** — user images sit in fixed-ratio containers with `object-fit: cover`. They
carry an inner shadow or a faint ring, so a light upload does not bleed into a light background.
Before: avatars at their own aspect ratios in a grid of uneven heights, one white-background logo
dissolving into the card. After: each in a 1:1 box with `object-cover` and a 1px ring at 10% black.
Check: upload a tall image, a wide image and a white-background image. The grid holds, and each
upload has a visible edge.

## Symptom lookup

Start from what the owner says, then check the named rules first. A symptom names a place to look, not a
verdict.

| The owner says | Check first |
| --- | --- |
| It looks busy, noisy or cluttered | **de-emphasize-competitors**, **three-text-colors-two-weights**, **thicken-not-darken**; [design-tells.md](../references/design-tells.md) for decoration that adds noise |
| It is confusing and I cannot say why | **labels-are-secondary**, **baseline-align-mixed-sizes**, **light-from-above**; spacing groups in [taste-rubric.md](../references/taste-rubric.md) Axis 3 |
| It looks amateurish or unfinished | **no-new-shades**, **tinted-badges**, **grayscale-first**; [taste-rubric.md](../references/taste-rubric.md) Axis 7 for consistency |
| The important thing does not stand out | **de-emphasize-competitors**, **actions-by-importance**, **three-text-colors-two-weights** |
| It is hard to read | **measure-inside-wide-containers**, **leading-follows-measure**, **colored-text-on-color**, **no-faded-text** |
| The colours look off | **no-new-shades**, **colored-text-on-color**, **color-never-alone**; [color-science.md](../references/color-science.md) for the ramp |
| It is wrong at some widths | **non-proportional-scaling**, **columns-before-width**, **measure-inside-wide-containers**; [generation-craft-defaults.md](../references/generation-craft-defaults.md) responsive contract |
| The depth looks wrong | **light-from-above**, **two-part-shadows**, **color-carries-depth** |
| The images look bad | **image-text-overlay**, **native-size-assets**, **contain-user-uploads** |

## Overcorrections

A correct rule applied too hard produces its own defect. When a fix lands, look for these.

| Rule applied too hard | What goes wrong | Correction |
| --- | --- | --- |
| **columns-before-width** | A narrow column squeezes a data table until its cells wrap | Tables keep the room their columns need; only prose and forms are narrowed |
| **thicken-not-darken** or fewer borders | A page covered in dark hairlines, or with no borders at all so nothing separates | Fewer borders, not none; a wider border keeps its soft colour |
| Headings sized by rank | Every heading shrinks to label size | Titles are often small, not always; rank sets the size |
| Generous white space | A dense tool turns airy and shows one row per screen | Density is right when it is deliberate and grouped |
| **labels-are-secondary**, **no-faded-text** | Secondary text becomes too faint to read | Keep it one clear step below primary and above the contrast floor |
| **no-new-shades** | Shades are frozen even when the ramp is wrong | Tune the existing shade; the rule forbids adding, not adjusting |
| **light-from-above** | Bevels and glossy gradients chase photo-realism | One quiet light source; stop at a readable raised or sunk cue |

## Not a finding

A reviewer must not report these. Each one is a correct use of a rule or a legitimate choice.

| Looks like a defect | Why it is not |
| --- | --- |
| Deliberate dashboard density | Dense, grouped and aligned data is the job of the page |
| `max-width: 70ch` | A `ch` unit states the measure rule directly |
| A serif face | A personality choice, not an error |
| Flat design with no shadows | Valid when **color-carries-depth** supplies the depth |
| A centred short hero headline | **center-only-short-text** allows up to three lines |
| A visually hidden heading | The heading stays for structure and assistive tech |
| A large red button inside a confirm dialog | **actions-by-importance** makes the destructive action primary there |
| Unequal margins above and below a heading | A heading belongs to the text under it, so more space above is right |
| Two background colours and no border | A colour step can separate areas as well as a line |
| Items of different widths in one column | **measure-inside-wide-containers** allows it |

## Not adopted

- **The accent border.** Refactoring UI uses a thick coloured border on one side of a card.
  [design-tells.md](../references/design-tells.md) bans it as the `side-tab` AI tell. A thin top bar
  on a card stays open; a thick one-side border does not.
- **Start with too much whitespace** as a bare rule. [generation-craft-defaults.md](../references/generation-craft-defaults.md)
  lists excessive whitespace as the `empty-boldness` failure mode. Extra space is kept only when it
  expresses grouping, priority or rhythm.
- **HSL authoring and hue rotation across a ramp.** [color-science.md](../references/color-science.md) reasons in OKLCH with a constant hue.
- **Saturation rising at both ends of a ramp.** [color-science.md](../references/color-science.md) tapers chroma at both ends.
- **A hand-picked type scale.** bee derives one modular ratio; see [taste-rubric.md](../references/taste-rubric.md) Axis 2.
- **The system font stack as the default.** [design-tells.md](../references/design-tells.md) flags platform-default faces.
- **Justified text, even with hyphenation.** [design-tells.md](../references/design-tells.md) flags justified body copy.
- **Arbitrary values as a failure.** [mode-constraints.md](../references/mode-constraints.md) allows arbitrary-value utilities.
- **Repeating background patterns and shape illustrations.** [design-tells.md](../references/design-tells.md) lists them as tells.

## Source

The ideas come from Refactoring UI by Adam Wathan and Steve Schoger, rewritten here in bee's words.
Some rules are restated from edisonmbli/refactoring-ui-skill at commit 4459662, MIT License,
Copyright (c) 2026 Edison Li, itself an unofficial restatement of Refactoring UI.
