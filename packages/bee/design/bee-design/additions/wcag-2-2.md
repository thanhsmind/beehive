---
id: wcag-2-2-additions
description: "Six WCAG 2.2 rules the design-os accessibility reference does not carry: focus not obscured, drag alternative, accessible authentication, auto-rotation pause, scrim contrast, error summary."
when: [accessibility, a11y, wcag, focus, drag, login, carousel, modal, scrim, form, error]
---

# WCAG 2.2 additions

These rules sit beside [accessibility.md](../references/accessibility.md), never above it. Its
honesty rule holds here too: a page that follows them is not "compliant". The static gate checks
none of them; each one is checked by eye, by keyboard, or in a rendered browser, and lands under
`NOT_VERIFIED` in the hand-over when it was not.

This file lives outside `references/` on purpose: `scripts/sync-design-os.sh` deletes and
re-copies `references/` on every sync, and these rules are bee's own.

## Focus not obscured

**focus-not-obscured** (2.4.11, AA) — a sticky header, a cookie banner, a chat bubble or any other
fixed layer never covers the control that holds keyboard focus. Tab through the page with every
fixed layer open; when focus lands under one, add `scroll-padding-top` or `scroll-margin` equal to
the layer's height, or move the layer. The AAA form (2.4.12) keeps the whole focused component in
view, not just part of it.

## Drag has a single-pointer alternative

**dragging-alternative** (2.5.7, AA) — every drag the page defines — a sortable list, a slider
thumb, a kanban card, a map pan — also works with single taps or clicks and with the keyboard:
move-up and move-down buttons, a select for the target column, arrow keys on the slider. Drags the
browser or OS provides, such as scrolling, are out of scope.

## Accessible authentication

**accessible-authentication** (3.3.8, AA) — a login or verification step never asks the user to
remember, transcribe or solve something unless an alternative exists. Password fields allow paste
and carry `autocomplete="current-password"` or `new-password` so a password manager can fill them;
one-time codes allow paste and carry `autocomplete="one-time-code"`. A puzzle CAPTCHA needs a
non-cognitive path, such as a passkey, an email link or an object-recognition alternative.

## Moving content can stop

**auto-rotation-pause** (2.2.2, A) — a carousel, ticker or auto-advancing slideshow that moves for
more than five seconds has a visible pause or stop control. It also stops while any of its content
holds keyboard focus or the pointer, and does not start at all under
`prefers-reduced-motion: reduce`.

## Scrim contrast is measured on the real background

**scrim-contrast-measured** (1.4.3, 1.4.11) — text and controls on a scrim, a dimmed overlay or a
gradient over an image are measured against the composed colour behind them, at the lightest point
of the image, in both themes. The scrim token alone proves nothing: its alpha mixes with whatever is
underneath. When no single value clears 4.5:1 for body text, darken the scrim or put the text on a
solid surface.

## Error summary takes focus

**error-summary-focus** (3.3.1, 3.3.3) — this extends the one line in
[content-design.md](../references/content-design.md) that asks for an error summary. After a failed
submit, the summary appears at the top of the form, receives focus, names the count of errors, and
lists each error as a link that moves focus to its field. Each field keeps its own inline message
with `aria-describedby` and `aria-invalid="true"`; the summary never replaces them.

## Source

Ported from ui-ux-pro-max-skill (https://github.com/nextlevelbuilder/ui-ux-pro-max-skill, commit
28b3b8d, MIT, Copyright (c) 2024 Next Level Builder), its quick reference and pre-delivery
checklist, rewritten in bee's words. The rest of that skill was not taken; the reasons are in
`docs/history/research/ui-ux-pro-max-into-bee-design-port.md`.
