---
type: grilling
status: open
claimed-by: none
blocked-by: none
---

## Question

LAW 2 (D-588eecb5) says tests are written "enough, not overload". What
is the stop rule an agent can actually apply mid-task, and how does it
coexist with a coverage skill whose whole acceptance criterion is that
the coverage percentage went UP?

Candidate shapes to put to the user:

- (a) Reuse the existing test-economy D3 test-to-source ratio ceiling
  and add nothing new — one number, already enforced at cap.
- (b) A behavior-count rule: one test per distinct behavior, boundary,
  or error path; a second test asserting the same behavior is a
  duplicate to delete.
- (c) A budget the change's risk class sets — a tiny fix earns fewer
  tests than an auth change.

The two skills must not fight: one says stop, the other says add more.
Which wins, and on what signal?

## Answer

<open>
