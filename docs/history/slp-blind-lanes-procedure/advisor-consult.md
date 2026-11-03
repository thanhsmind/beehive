# Advisor consult — slp-blind-lanes-procedure shape gate

- Date: 2026-08-28 · Tier: advisor (fable) · Repo HEAD at consult: `099cfef3`
- Question: is the plan safe to approve at the shape gate?
- Verdict: **SAFE WITH NAMED CHANGES**

## 1. Phase 2's bet is unsound as written

A blanket fence-skip is a one-keystroke bypass of the leaning guard, in every
round — not only round two. After the change as drafted, a round-ONE brief can
carry a fenced "I recommend option A" and pass the guard while every lane reads
the verdict. That is the list-shrink the shipped rule forbids, done by scope
instead of by deletion, to make the round-2 corpus pass.

Three further defects in the bet:

- An unclosed fence marks everything after it as fenced, so a fence opened after
  the last required heading hides arbitrary trailing prose from every scan with
  no refusal at all.
- The fence rule must reach all three scans — the stem scan, the heading scan,
  and the Question-enumeration scan. A quoted rival proposal carries its own
  `##` headings and its own bullet lists, so the other two arms fire on it
  today. The drafted proof named the stem arm only.
- The 8 KB brief cap collides with D2(c)'s word "verbatim": four required
  sections plus a full rival proposal will exceed it, and the cap's own remedy
  text ("move the bulk into the read diet") contradicts the decision.

**The bound that makes it sound:** skip TAGGED fences only — the opening info
string equals one designated token. Untagged fenced text scans exactly as
today, and an unclosed fence is a typed refusal. A tagged fence is then an
explicit recorded claim, forgeable but a named lie in a recorded brief — the
same trust posture D4 already takes.

## 2. Phase 1: render, but refuse the combination

Rendering `--expertise` for gather and reviewer is right; refusing outright
would break callers that pass it today. But rendering it into a blind lane's
payload beside a brief opens two holes: the leaning guard reads brief bytes
only, so the reading list is an unlinted leaning channel into a lane; and the
digest proves the BRIEFS were equal while the payloads diverge, which the
door's own comment already claims cannot happen. The plan made this same
argument itself when it dropped the read-diet carrier.

The drafted proof is also insufficient: an `{{#if}}` over an undefined variable
is silently falsy, so a template twin that misses the update swallows expertise
again with every test green — the exact defect the phase exists to kill.

## 3. Ordering

Phase 2 does not depend on Phase 1 — disjoint surfaces, and an unnamed serial
dependency is the concurrency law's defect. 3b genuinely does not depend on 3a.
If the combination refusal lands, Phase 4's prose then depends on Phase 1 too.
Factual slip: there are three existing letter producers that hard-code an empty
list, not two.

## 4. Scope integrity

All three dropped items verified true at HEAD; the drops are legitimate and no
locked decision is quietly shrunk. The one real shrink risk is D2(c)'s
"verbatim" against the byte cap.

## 5. Missing entirely: round-2 chain of custody

The dossier's cross-critique section is unchecked free prose. Round-2 briefs
differ per lane by construction, so they sit outside the digest chain, the
recorded-brief re-lint and the citation check. No phase named that. Minimum fix
is prose: require round-2 dispatch ids in the section and record the uncovered
status as a named limit.

## Named changes (all nine folded into plan.md)

1. Bound the skip to tagged fences.
2. An unclosed fence refuses.
3. One shared fence implementation, not a second copy.
4. Red-first per scan, not per guard.
5. Refuse `--expertise` combined with `--brief-file`.
6. Positive-case proof per kind, plus a disk-match probe per edited template.
7. Name the recorded fallback for a rival proposal over the byte cap.
8. Round-2 chain of custody named as a limit in the prose.
9. Shape-table corrections: drop 2→1, add 4→1, three producers not two.
