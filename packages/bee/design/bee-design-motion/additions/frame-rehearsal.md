---
id: frame-rehearsal-additions
description: "Three browser scripts that check a moving page frame by frame: a contact sheet, a jump and drift scan, and a box-fit snapshot. What each one checks, how to run it, its flags, exit codes and limits."
when: [motion, animation, rehearsal, frames, screenshots, contact-sheet, jump, drift, layout, overflow, verification, playwright]
---

# Frame rehearsal

Three scripts read a page the way a viewer meets it: as frames in time. They
live in `additions/rehearse/`, outside `references/` on purpose — the design-os
sync deletes and re-copies `references/`, and these are bee's own.

They are a port of the Stageshow scripts (MIT, see
[rehearse/LICENSE-stageshow.md](rehearse/LICENSE-stageshow.md)).

| Script | Question it answers |
| --- | --- |
| `sheet.mjs` | What does the page look like across time, side by side, at this width? |
| `jumps.mjs` | Does anything step instead of glide while the page animates? |
| `layout.mjs` | Does a control's content fit its box, and does a padded container hold its bottom padding? |

## Setup

The scripts need `playwright-core` in the host project, once:

```sh
npm i -D playwright-core
```

No browser download is needed. The scripts start the Chrome already on the
machine (`channel: 'chrome'`); set `CHROME_PATH` to a Chrome or Chromium
binary to pick another. A missing `playwright-core` or a Chrome that will not
start prints how to fix it and exits 2.

## The select flag

Every script works on any page. `--select` is a CSS selector for the element
the check reads, and it defaults to `body`, so the whole page is the subject
unless you narrow it.

```sh
node additions/rehearse/sheet.mjs "http://localhost:3000/#/demo" --select "#demo" --out shots/demo.png
```

The selector takes the first match in `layout.mjs` and `sheet.mjs` (every
matching visible root is checked by `layout.mjs`). `jumps.mjs` follows the
element and any `.ss-stage` inside it.

## sheet.mjs — the contact sheet

Opens the URL at `--width`, waits for the `--select` element, screenshots that
element `--count` times every `--every` milliseconds at device scale 2, and
lays the frames out in one grid PNG inside the same browser. Each frame is
labelled with its time from the first frame. The grid is drawn at 2x and each
frame is drawn at its natural pixel size, so nothing is scaled up or down.

```sh
node additions/rehearse/sheet.mjs <url> [--select body] [--width 1280] [--every 1000] [--count 12] [--cols auto] [--out sheet.png]
```

| Flag | Default | Meaning |
| --- | --- | --- |
| `--select` | `body` | The element to shoot. |
| `--width` | `1280` | Viewport width in CSS px. |
| `--every` | `1000` | Milliseconds between frames. |
| `--count` | `12` | Frames in the sheet. |
| `--cols` | `auto` | Grid columns. Auto fits about 1800px. |
| `--out` | `sheet.png` | Output PNG, relative to the current directory. |

The absolute path written is printed. Exit 0 means the sheet was written; exit
2 means the run was incomplete (no matching element, no frames, a page error)
and nothing was written.

## jumps.mjs — the jump and drift scan

Records the box of every visible element inside `--select` on every animation
frame for `--ms` milliseconds, then reports three kinds of line:

- `jump:` an element that moves or resizes more than 2px in one frame while it
  was still the frame before and after. A glide is not a jump, and an element
  that appears or leaves is not a jump either — its entrance handles that.
- `drift:` two neighbors in flow that keep their size but not the gap between
  them, over four frames or more. They run on different curves.
- `anchor drift:` an open popover or menu whose top edge moves while its height
  changes. An open surface holds the edge it opened from.

A fourth line, `same-box content change:`, is not a finding: an element whose
text or drawing changes in place needs a human eye, because geometry cannot
judge whether the change is right.

```sh
node additions/rehearse/jumps.mjs <url> [--ms 20000] [--width 1280] [--select body] [--report jumps.json] [--strict]
```

- Frames around a late frame (a gap over 1.6x the typical interval) are not
  judged: a stalled page carries several frames of gliding at once and cannot
  tell a step from a glide. Coverage reports that gap.
- `--report` writes the JSON result: `status`, `exitCode`, each stage's
  `coverage`, `findings` and `manual`.
- Exit 0 means no candidate was found, 1 means candidates with `--strict`,
  2 means the run was incomplete — too few frames, a gap, a sparse interval, no
  sampled element, a resource error or a page error.

## layout.mjs — the box-fit snapshot

Waits for `--select`, waits `--delay` ms for fonts and transitions, then checks
one snapshot of the page. Two rules:

- `control-block-fit`: a button or chip whose content, padding and borders need
  more height than its box has, by more than 1 CSS px (font metrics and
  fractional rounding earn that 1px).
- `control-edge-inset`: near-zero padding on one side with content against that
  edge and a real 5px or more inset on the opposite side. Ordinary optical 1 to
  2px adjustments, icon-only buttons and asymmetric dropdown affordances pass.
- `container-block-inset`: a container with 6px or more of bottom padding whose
  last child leaves less room below it than that padding, measured against the
  nearest real frame.
- `nested-composition-crop`: a window cut by an inner clip while an outer
  deliberate crop continues further. One crop explains itself; a second cut
  inside it does not.

```sh
node additions/rehearse/layout.mjs <url> [--select body] [--width 1280] [--delay 300] [--report layout.json] [--screenshot layout.png] [--strict]
```

Moving geometry is reported as `incomplete:`, never as a pass — re-run once the
motion settles. A visible iframe is listed as `manual:`; check its direct URL
separately. `--screenshot` captures the 1000px-high viewport beside the report.

Exit codes are the same contract as `jumps.mjs`: 0 clean, 1 findings with
`--strict`, 2 incomplete (an error, no supported element checked, or deferred
geometry).

## What the scripts cannot see

- **Taste.** A clean report is not approval. Watch the animation at speed,
  slowed, and a second time after the final edit.
- **A sheet is not a film.** A contact sheet misses a one-frame flash. Sampling
  at 1000ms cannot show a 200ms transition; use `jumps.mjs` for that, or a much
  shorter `--every` on the sheet.
- **Text and glyph rendering, focus styling, and optical centering.** `layout.mjs`
  measures CSS boxes, not ink; a rotated glyph's contour or a morph's keyframes
  are visual review.
- **Different curves on purpose.** Drift is a candidate: a deliberate stagger
  must be named and looked at.
- **Anything behind an iframe, a closed shadow root, or a canvas.** The scripts
  read the DOM they can reach in the top document.
- **Reduced motion, hover, press and focus states** unless the page reaches
  them on its own during the recorded window.

## Attribution

Scripts ported from Stageshow: https://github.com/benjitaylor/skills, folder
`stageshow/scripts`, MIT, Copyright (c) 2026 Benji Taylor. The full license text
is in [rehearse/LICENSE-stageshow.md](rehearse/LICENSE-stageshow.md). `sheet.mjs`
is new here; `film.mjs`, `film-checks.mjs` and `motion-watch.mjs` stay upstream
because they are written against the Stageshow kit, which bee does not ship.

Code in these scripts carries no comments, by bee's rule. The reasons the
upstream comments carried are the ones written above: why 2px and 3x the calm
motion, why a late frame is skipped, why a moved popover is drift but a growing
one is not, why the 1px tolerance, and why an appearing element is not a jump.
