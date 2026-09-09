# Hat wave — plan-step synthesis, leader-sees-team

Date: 2026-09-09. Three seats, two models, one draft (`plan.md` revision 1, commit `66a8963e`).
Seats: `hat-facts-gaps` (opus), `hat-alternatives` (opus), `hat-user-impact` (agy-flash pane).
No seat dropped. This wave is the feature's plan check (absorption, decision `b34fdea9`).

## Verdict

**Revision 2 is owed.** The claims table survived (14/14 verbatim, both `ran` rows reproduced),
but the shape did not: the helper was placed where the config is not in reach, eight cells of the
`declared` truth table were undefined, a second writer of the status vocabulary was missed, a
locked display law (B13) stood in the way of D1, and the preamble roster — as drafted — would have
been a 639-character wall with twelve roles hidden. Every accepted finding was re-verified by the
leader at its anchor before it changed the plan.

## Accepted blockers → what changed

| # | Seat | Finding (verified anchor) | Change in revision 2 |
|---|---|---|---|
| B1 | facts-gaps · alternatives | `derive_economics(channel, role, param_model, resolved, native_confirmed)` (`guard.rs:342-350`) has no config; `Resolved::Herding` carries a name, not an argv (`models.rs:349`). The plan's call site could not exist. | `declared` is computed in `prepare.rs` beside `param_model` (`:1972-1975`), where cfg/runtime/resolved are in hand, and passed as a sixth argument. |
| B2 | facts-gaps | `Herding { agent: None }` is a real shape (`prepare.rs:873`, `:1783`); its argv comes only from cfg (`wave.rs:356-368`, `:415`). Undefined. | Defined: the helper resolves `agent: None` through the same precedence `resolve_agent_command_for_runtime(cfg, None, rt)` uses, so the declared model is whatever that argv names. |
| B3 | facts-gaps | The model guard **recomputes** all six economics fields itself (`model_guard.rs:1213-1258`) and maps `Herding` → `None` (`:331-335`); two rows for one role would disagree. | One helper in the drivers module, called by both writers. The guard's audit line for a herding slot reports the same `declared` value prepare would. |
| B4 | facts-gaps | The "byte-identical null path" proof cited tests that never carry a herding agent (`guard.rs:667`, `tests.rs:775-782`); no config entry lacks `--model` any more. | A fixture registry entry with no model token and a test asserting `null` / `unverified` on it. |
| B5 | alternatives | B13 (`model-roles-and-escalation.md:158-175`) says `team show` "reads, never resolves"; B12 pins the door line's shape. D1 cannot be honoured under them. | **D4 logged**: `team show` may resolve for display and still writes/guards/dispatches nothing; the preamble roster goes one role per line, no descriptions, no six-role cap. |
| B6 | alternatives · facts-gaps | `swarming-reference.md:398` states `requested_model` is always null for cli-exec — D2 falsifies it; `advisor-protocol/slots-and-tiers.md:69` holds the vocabulary and lacks `declared`. | Both swept in the docs cell. |
| B7 | user-impact | The description-bearing single-line roster renders at 639 chars on this repo, and `DOOR_ROLES_SHOWN = 6` (`model_guard.rs:346`) hides `plan`, `docs` and ten more. | Multi-line roster, ≤ 60 chars per role line, every role, descriptions only in `team show` (D4). |

## Accepted warnings → what changed

- **Helper signature** — `declared_model_for(cfg, resolved, runtime) -> Option<String>` beside `resolve_agent_command_for_runtime`, handling `Herding` (both agent arms) and `Cli { command: String }` (`models.rs:324-326`), which is a string, not an argv.
- **Two cli-exec sites** (`prepare.rs:1755`, `:1781`). Rule: `declared` reads the **command that will run** — at `:1755` that is the `Native` slot's fallback command string, never its `model` field.
- **Tokenizer** — quote-aware split; accepts `--model X`, `--model=X`, `-m X`; when more than one is present the **first** wins (a named guess, recorded).
- **Herding `fallback`** (`prepare.rs:1870-1887`) is a model bee may re-dispatch on; the record describes the **primary** argv only.
- **`role_slot_display`** (`model_guard.rs:463-465`) sees only the `team` subtree; it gains the whole config so the roster can print a model.
- **The roster's real renderer** is `dispatch_door_lines` at `model_guard.rs:527-528`, not `render_role` alone — a claim row added.
- **Two display surfaces converge** on `Resolved` + the helper; `models_group.rs` gains the resolver import.
- **D3's worker home** is `packages/bee/prompts/worker-cell.md` — the file that reaches every worker, including a pane worker with no skill tree; `worker-details.md` keeps a pointer.
- **CONTEXT.md named the delegation contract's home wrongly** (`routing-and-contracts.md`); it is `gates-and-delegation.md:124`. Corrected in CONTEXT.
- **Claim 14 re-scoped** — 9 of this repo's 18 `claude` slots are herding and blind too (the seat said 17; the leader's re-count gives 18); the baseline is every herding slot on every runtime, not pi.
- **Refused dispatches emit no economics block** — out of scope, noted.
- **`declared` is a record word, not a display word** — the surfaces print the model name; the status token appears only in `dispatch.jsonl`.

## Rejected on evidence

- *Change `derive_economics`'s signature without moving the computation* — the function stays config-free and pure; the sixth argument is a value, not a config.
- *Keep the single-line roster and raise the cap* — 18 roles at ~30 chars is still a 500-character wall.
- *Put D3's worker sentence in `worker-details.md` only* — a herding pane worker never loads it.
