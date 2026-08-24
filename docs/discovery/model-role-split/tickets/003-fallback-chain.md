---
type: grilling
status: closed
claimed-by: wayfinder (resolved)
blocked-by: (none)
---

## Question

Does a role entry gain an **ordered fallback chain** (model A, then B,
then C on failure), or do bee's two existing single-step mechanisms stay
as they are?

What exists today:

- Explicit-only composite `{primary, fallback_policy: "explicit-only",
  fallback: {kind: "cli", …}}` — `models.rs:134-166`, decision 3ceba8f5
  D2. One fallback, and by that decision it never fires silently.
- Herding slot `fallback: "default"` — `models.rs:112-133`, decision
  267192c1. A flag, not a model; absent, a failure stays loud.

Both were deliberately built to fail loudly rather than degrade
quietly. A chain is the opposite posture: keep going down the list. So
this is not an additive feature — it reopens a settled stance.

The real question is therefore: **which failures should a chain absorb?**
A quota refusal and a rate limit are transient and worth retrying
elsewhere. A tool-contract failure or a bad result is not — falling
through to a weaker model there hides the defect.

Related evidence that the loud posture has teeth: decision 4faf1de9 —
an advisor consult was recorded as NOT OBTAINED when the configured
advisor hit its quota, and no substitute was run, because the advisor
has no fallback by design.

## Upstream answer to this exact question (xia, 2026-08-24)

`~/Projects/refs/oh-my-pi` @ `2b66ee69` (docs/history/research/oh-my-pi-model-roles-distill.md)
runs a chain and keeps a loud posture, by splitting the two layers this
ticket conflates:

- **Resolution layer** — an unset or unavailable role falls through to
  the next name. No failure involved; nothing is hidden.
- **Runtime layer** — `retry.fallbackChains`, a `Record<string,
  string[]>` keyed by role, by exact model, or by `provider/*`
  (`docs/settings.md:439-463`), walked only on an **error-class gate**.

What advances their chain: `UsageLimit` (429/quota), `AccountPolicy`,
`MalformedFunctionCall` (replay-safe only), `EmptyResponse`, stream
stall / HTTP2 reset, 5xx. What does **not**: tool errors, bad or
unwanted output, and `ThinkingLoop` — explicitly excluded, it stays on
the same model (`turn-recovery.ts:2048-2061`, `:1101-1106`).

That is precisely this ticket's stated question — "which failures should
a chain absorb?" — answered as: transient and infrastructural, never
semantic. Under that gate a chain does not contradict bee's loud
posture (decisions `3ceba8f5` D2, `267192c1`, `4faf1de9`), because no
*result* failure is ever absorbed. Reverting is configurable too:
`fallbackRevertPolicy` defaults to `cooldown-expiry`
(`docs/settings.md:476-478`).

Still the user's decision; this ticket stays open.

## Answer

**Yes, with the two layers held apart** — decision `50808d48`. bee gains an
ordered **runtime** fallback chain, and it does not reopen the
loud-failure stance, because the stance was never about this layer.

- **Resolution layer** (`06e49368`): an unset or unresolvable role
  yields to the next name. No failure is involved.
- **Runtime layer** (this ticket): a configured chain may carry a
  dispatch to another model after the first fails.

The chain is **explicit-only** — no built-in default chain for any
role — so absent configuration nothing changes, and decisions
`3ceba8f5` D2, `267192c1` and `4faf1de9` stay intact rather than being
reopened. The advisor keeps its no-fallback behavior unless the owner
configures a chain for it deliberately.

The gate answers this ticket's own stated question, "which failures
should a chain absorb?":

| Fires a chain step | Never fires one |
|---|---|
| quota, rate limit | a tool error |
| provider auth or policy rejection | a wrong or unwanted result |
| empty response | a failed proof, a red test |
| malformed tool call, replay-safe only | anything semantic |
| stream stall, connection reset, 5xx | |

Transient and infrastructural only. No **result** failure is ever
absorbed, so falling to a weaker model can never hide a defect. Every
chain step is recorded on the dispatch, because a quiet degrade is the
failure mode being guarded against.

Owner-delegated, 2026-08-25; the agent's call, overturnable.
