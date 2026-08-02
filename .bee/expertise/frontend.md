# Surfaces people use

A user-facing surface is judged by what happens when things go wrong, not
by how it looks when everything is loaded and the network is fast. The
happy path is the easy half and it is the half that gets built; the empty
list, the failed request, the second click, the keyboard user, and the
slow connection are where a competent-looking interface turns out to be
unfinished.

Everything here is about behavior, not about a particular framework or
toolkit. The rules hold for a web page, a native app, a terminal
interface, or a desktop window, because they come from how people use
things and how networks fail, not from any library's model.

## Where to look

| Situation / goal | Entry |
|---|---|
| Building any view that fetches something | Four states, always |
| The same value is tracked in two places | State has exactly one owner |
| A view's contents should survive a refresh or a shared link | The address is state |
| Building any control, input, or interactive element | Use the platform's control |
| Deciding whether accessibility is in scope | Accessibility is correctness |
| Building or changing a form | Forms lose work, or they don't |
| Showing a result before the server confirmed it | Optimistic needs a rollback |
| The list has nothing in it yet | The empty state is a real screen |
| A request failed and the user is looking at it | Errors tell the user what to do |
| The interface feels slow | Perceived speed is designed |
| Content must work on a small screen or a touch device | Design for the input, not the device |
| A long list or a heavy view stutters | Rendering cost lives in the list |
| Writing the words in the interface | The words are the interface |
| Deciding what the client may be trusted with | The client is not a trust boundary |
| Testing a user-facing surface | Test what the user does |

## Four states, always

When building any view that depends on something asynchronous → design
all four states before writing the successful one. Every remote read has
them, and skipping any one produces a specific, familiar bug:

- **Loading** — something is happening. Missing, and the interface looks
  broken or invites a second click.
- **Empty** — the request succeeded and there is nothing. Missing, and
  users see a blank region and assume failure.
- **Error** — the request failed. Missing, and the interface either shows
  an eternal loading state or renders as if the data were empty, which is
  a lie.
- **Loaded** — the data is here.

Two refinements matter in practice. *Empty is not one state*: "no results
for this filter" and "you have not created anything yet" call for
completely different screens, one offering a way to widen the search and
the other explaining how to start. And *partial* is a fifth state in any
view assembled from several sources — decide deliberately whether one
failed section blocks the page or degrades inside its own box. Blocking a
whole dashboard because one widget's request failed is a choice, and
usually the wrong one.

## State has exactly one owner

When the same value exists in two places → decide which one is the truth
and derive the other, because two copies will disagree and the disagreement
will be intermittent. Most confusing interface bugs are a synchronization
failure between duplicated state, not a rendering failure.

Sort state by where it belongs, and keep the categories from bleeding
into each other. **Server state** is data you fetched; it is a cache of
something you do not own, and it needs the cache thinking from
`performance.md` ("A cache is a copy with an owner") — staleness,
invalidation, and what happens on refetch. **Client state** is genuinely
local: what is expanded, what is selected, what is being dragged.
**Address state** is what should survive a reload or a paste into a chat
— see the next entry. **Form state** is in-progress input that has not
been committed anywhere yet, which is why losing it is so expensive.

Copying server state into local state to edit it is the most common
version of the problem: the copy is now stale the moment anything else
changes the original, and there is no rule saying which wins. Either edit
in place, or make the copy explicitly a draft with a defined moment when
it merges back.

## The address is state

When a view has a meaningful configuration — a filter, a tab, a page, a
selected item, a search query → put it in the address, not only in
memory. This is what makes a view refreshable, shareable, bookmarkable,
and navigable with the back gesture, and those are not niceties: users
reload, they share links, and they press back constantly.

The rule that follows: the back gesture must do what the user expects,
which is to undo their last navigation, not to leave the application from
the middle of a task. And a link someone pastes to a colleague should
open the same thing they were looking at. When a state is worth
navigating to, it needs an address; when it is not — which panel is
expanded, whether a menu is open — it does not, and putting it there
creates noise in the history.

## Use the platform's control

When building a control — a button, a checkbox, a select, a dialog, a
link → use the platform's own element before building your own. The
native control arrives with keyboard behavior, focus handling,
accessibility semantics, input-method support, high-contrast and
zoom behavior, and platform conventions that users already know. A
custom replacement starts with none of it and acquires it one bug report
at a time.

The specific failure to watch for is an element that *looks* interactive
without *being* interactive: a clickable div is unreachable by keyboard,
announces nothing to assistive technology, does not respond to the enter
key, and cannot be opened in a new window when it is really a link. The
distinction that resolves most of these: something that navigates is a
link, something that acts is a button, and they are not interchangeable
even when they are styled identically.

When a custom control is genuinely necessary, the cost is the full
behavior contract — keyboard interaction, focus management, state
announcement — and that cost should be a deliberate decision, not a
discovery.

## Accessibility is correctness

When deciding whether accessibility is in scope → it is. An interface
that cannot be operated by keyboard, cannot be read aloud, or cannot be
seen at the contrast it ships with is not a styled interface with a
missing feature; it is an interface that does not work for a real portion
of the people using it. In many contexts it is also a legal obligation,
but the engineering argument stands alone.

The checks that catch most of it, in order of return:

- **Semantics first.** Use the element that means what you mean —
  heading, list, button, label, table. Assistive technology navigates by
  structure, and a page built entirely from generic containers is a wall
  of undifferentiated text.
- **Everything works from the keyboard.** Tab through the whole surface:
  can you reach every control, is the order sensible, can you see where
  you are, can you escape from anything you can enter, and does focus
  land somewhere useful when content appears or disappears.
- **Every input has a real label**, associated with it, not a placeholder
  standing in for one — placeholders vanish exactly when the user needs
  to check what they are filling in.
- **Never encode meaning in color alone.** A red border must be
  accompanied by text; a status shown only as a colored dot is invisible
  to a large number of people.
- **Images and icons carry text alternatives** when they mean something,
  and are hidden from assistive technology when they are decoration.
- **Contrast and motion.** Text must be readable at the contrast you
  ship, and animation that moves large regions should respect a stated
  preference for reduced motion.

Test it the way it is used: navigate with only a keyboard, then listen to
the page. Automated checks catch a minority of real problems and none of
the ones about whether the experience makes sense.

## Forms lose work, or they don't

When building a form → the governing rule is that the user's typing is
precious and the system is responsible for it. Every other rule below
follows from that.

Validate at the right moment: not on every keystroke while a field is
still being filled — which shouts "invalid email" at someone who has
typed two characters — but when a field is left, and again on submit. Put
the error next to the field it belongs to, in words that say what to do,
and never clear what was typed because part of it was wrong.

On submit, three things must hold. The control disables or otherwise
prevents a second submission while the first is in flight, because double
submissions create duplicates. The request is safe to repeat anyway
(`apis.md`, "Every unsafe operation needs a retry story"), because the
control is not a guarantee. And a failure returns the user to their
filled-in form with an explanation, never to an empty one — losing ten
minutes of typing to a server error is the single most infuriating
failure in this category.

For anything long, preserve progress as it is entered so a reload,
navigation, or crash is survivable. And validate on the trusted side
regardless of what the form checks (`security.md`, "Validate at the
boundary, allow-list first"); client-side validation is a courtesy to the
user, not a control.

## Optimistic needs a rollback

When showing a result before the server has confirmed it → the interface
is now asserting something that may turn out to be false, and the design
is not finished until you have said what happens when it is. Optimistic
updates are worth it — they make an interface feel immediate — but only
with three pieces in place: the previous state is kept so it can be
restored, the failure path visibly restores it and explains why, and the
operation is idempotent so a retry does not double it.

Be selective about where it applies. Reversing a toggle after a failure
is fine. Silently un-sending something the user believes they sent, or
un-deleting something they watched disappear, breaks trust more than the
half-second of latency ever cost. Use optimism where the failure is rare
and the reversal is comprehensible.

## The empty state is a real screen

When a list, a table, or a dashboard has nothing in it → design that
screen with the same care as the full one, because it is the first thing
every new user sees. A blank area reads as a broken page; it is also the
one moment when the user is guaranteed to be looking for what to do next.

A good empty state says what belongs here, why it is empty, and offers
the one action that fills it. Distinguish the varieties, which need
different words: nothing yet, nothing matching this filter, nothing you
have permission to see, and nothing because loading failed — the last of
which is an error, not an empty, and must never be shown as one.

## Errors tell the user what to do

When a request fails and the user is looking at the result → the message
answers three things in their language: what did not happen, why, and
what they can do about it. What it must not do is show them a code, a
stack trace, an internal identifier alone, or a sentence describing your
architecture.

Match the presentation to the severity and the scope: a failure in one
widget belongs in that widget, not in a modal that interrupts everything;
a failure that lost the user's work needs to be loud and needs to have
preserved the work. Offer the retry when retrying is meaningful, and do
not offer it when the request will never succeed — a retry button on a
permission error just makes the person click twice to learn the same
thing.

Keep a correlation identifier available for support, quietly, so a user
who reports the problem can be matched to the log entry (`operations.md`,
"Logs are evidence, not narration").

## Perceived speed is designed

When an interface feels slow → what people experience is the time until
something useful appears and whether the wait is legible, not the total
duration. The techniques that buy this are design decisions, not
optimizations:

- **Respond to input immediately**, even before the work is done — a
  pressed state, a disabled control, a spinner in the right place.
  Silence after a click reads as a failure and produces a second click.
- **Show what is ready** rather than holding everything until the slowest
  part arrives.
- **Reserve the space** that loading content will occupy, so nothing
  jumps when it arrives. Content that shifts under a moving cursor causes
  misclicks and is experienced as sloppiness.
- **Prefer a shape that resembles the coming content** over an
  indeterminate spinner, and give any wait over a few seconds a real
  sense of progress or an explanation.
- **Start likely work early** — prefetch the next page, warm the request
  on intent rather than on click — where the cost of being wrong is low.

The corresponding measurement discipline is in `performance.md`
("Perceived speed is real speed"): instrument the moment the user can
act, not the moment the last byte lands.

## Design for the input, not the device

When content must work across screen sizes → let the content decide where
the layout changes rather than targeting a list of device dimensions.
Device categories churn and overlap; the point at which a line of text
becomes uncomfortable or a table stops fitting does not.

The input method matters more than the width. Touch needs targets large
enough to hit and spacing that forgives; anything that only appears on
hover is invisible on a touch device and must have a non-hover path;
precise dragging and right-click are unavailable. Assume in both
directions: a small screen may have a keyboard, a large screen may be a
touch display.

Beyond size, the things that break real layouts are text that is longer
than the mockup's, text that the user has enlarged, content in a language
that reads the other way, and a very long unbroken string. Try all four
before calling a layout done.

## Rendering cost lives in the list

When a view stutters → the cause is almost always quantity rather than
complexity: a long list, a large table, a view that re-renders everything
in response to a change that touched one row. Interfaces are fast until
they meet real data volumes, and then they are slow in a way that feels
like the whole application is broken.

Three moves cover most of it, in order. Render only what is visible when
the collection is genuinely large. Make sure a change to one item does
not cause the rest to be rebuilt — which usually means stable identity
per item and state kept as close to where it is used as possible. And
move expensive work off the path that must stay responsive: a computation
that blocks input for 200 ms is felt directly, whatever it is computing.

Measure before rebuilding, with the same discipline as anywhere else
(`performance.md`, "No number, no optimization"). Interface performance
intuition is especially unreliable because development machines are fast
and development datasets are small.

## The words are the interface

When writing the text in a surface → treat it as part of the engineering,
because it is what users actually read and it is where most confusion
originates. Labels, button text, error messages, empty states, and
confirmations do more for usability than most layout decisions.

Prefer the user's vocabulary over the system's — a message naming an
internal component tells the reader nothing they can act on. Say what a
button will do rather than "OK", especially when the action is
destructive and the dialog is asking for confirmation. State errors in
terms of the situation rather than the failure — the person does not care
that a request returned an error; they care that their change was not
saved and whether they can try again. And keep terminology identical
across the surface: two words for one concept makes users think there are
two concepts.

## The client is not a trust boundary

When deciding what the client may be trusted with → nothing that matters
for correctness or access. Everything that runs on the user's device can
be read, modified, replayed, or replaced, including code you shipped and
values you sent for it to send back. A hidden field, a disabled control,
a client-side role check, and a validation rule in the form are all
usability features (`security.md`, "Draw the boundary first").

Two practical consequences that come up constantly. Anything embedded in
the client is public — a key, an endpoint, a rule, a comment — so a
secret in client code is a published secret. And the interface's
authoritative view of what the user may do comes from the server on every
request, not from a permission list fetched at login and cached
indefinitely, which goes stale exactly when access is revoked.

## Test what the user does

When testing a user-facing surface → assert on what a person can perceive
and do, not on internal structure. A test that finds a control the way a
user finds it — by its visible text, its label, its role — survives a
restyle and fails when the interface actually breaks. A test bound to
internal identifiers, class names, or component internals fails on every
refactor and passes while the button is invisible, which is the opposite
of what a test is for.

That framing has a useful side effect: a surface that is hard to test
this way is usually hard to *use* this way. If a test cannot find the
submit control by its accessible name, neither can a screen reader.

Cover the four states rather than only the loaded one, and prefer a small
number of tests that walk a real task end to end over many that assert
individual renders — the bugs in this layer live in the transitions
between states, not inside them (`tests.md`, "Pick the cheapest level
that can fail").
