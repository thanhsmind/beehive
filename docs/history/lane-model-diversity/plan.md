---
artifact_contract: bee-plan/v1
mode: standard
---

# Plan: Lane Model Diversity

Mode: `standard` — 2 risk flags: changes behavior an existing test asserts (T012a refusal arm), public contracts (dispatch door semantics)
Why this is the least workflow that protects the work: recorded law changes (role resolution + guard parity) need a plan and tests, but no hard-gate territory is touched.

Review wave: plan-checked by a reviewer dispatch (dispatch bf4fa852, 2026-08-29) — FAIL with 2 P1s; this revision folds in every named fix. P1-1 (description is display-only law), P1-2 (null seat slot escapes fall-through), P2-1 (phantom default), P2-2 (case folding), P3-1..3.

## Requirements (from CONTEXT.md)

- D1: Seat roles join `models.<runtime>`: `lane-1..lane-3`, `hat-facts-gaps`, `hat-risks`, `hat-value`, `hat-alternatives`, `hat-user-impact`. One config home.
- D2: Unconfigured seat falls through to `advisor` — for `--kind advisor` dispatches only; every other kind keeps T012a refusal. Advisor tail stays the one-name walk (`4faf1de9`).
- D3: A configured `hat-*` slot must carry a `description`; `lane-*` slots need none.
- D4: No dispatch-time model flag; `[bee-tier: …]` marker names the RESOLVED role.
- D5: `gates-and-delegation.md` names the seat roles in both procedure sections.

## Discovery

- `prepare.rs:1167-1184` — T012a canonical-role arm runs BEFORE every kind-specific arm (advisor `:1285`, escalation `:1300`, role walk `:1307`); `kind` is in scope. Rewriting the `None` branch to `Some("advisor")` lands on the existing advisor arm and its `advisor_not_configured` refusal for free (reviewer Q3).
- `models.rs:100-112` — `default_models` seeds NO advisor key. The danger of walking `[seat, advisor]` through `resolve_role_named` is the **Budget floor** (`models.rs:601-604`): an absent last entry returns `Resolved::Budget` = "no model param, inherit the session model" — exactly the outcome `4faf1de9` forbids for advisor (reviewer Q2; corrects this plan's earlier phantom-default claim).
- `model_guard.rs:356-362` + `.bee/config-sample.json:22` — recorded law: `description` is display-only; nothing that resolves, guards, or dispatches may read it. `normalize_models` drops the field before `dispatch prepare` ever sees it. D3's enforcement therefore CANNOT live at the dispatch door without superseding that law.
- `bee config validate` is NOT in the Rust binary ("config verbs were never ported off Node" — command output, this session).
- CONTEXT.md deferred question CLOSED: `resolve_brief_file` gates on `kind != "advisor"` only (`prepare.rs:130-138`) and never reads `role` — no `--role`/`--brief-file` interaction beyond the resolution walk (reviewer P3-1).

**Named deviations (both venue-only, intent intact):** D3 names `bee config validate` as the enforcement point — that command is not ported; and the dispatch door may not read `description` under the display-only law. Enforcement moves to `bee doctor`: an advisory finding, reading the RAW config (the `role_slot_description` pattern, `model_guard.rs:368-377`), flagging every configured `hat-*` slot whose description is missing — including string-shaped slots, which cannot carry one. Dispatch behavior never depends on the field, so the display-only law stands unsuperseded.

## Approach

Recommended (cites D1-D5): a closed `SEAT_ROLES` constant (8 names) beside `tier_role_list` in `models.rs` — the single home D5's docs cite; membership compared with `eq_ignore_ascii_case` (P2-2 — the same two-doors rule `prepare.rs:1152-1157` already enforces). In `prepare.rs`'s canonical-role arm: when `kind == "advisor"` and the declared role is a seat name whose slot **resolves nothing** — absent, null, or any value `resolve_configured` answers `None` for (P1-2) — rebind `canonical_role` to `"advisor"` instead of refusing; the existing advisor arm (`:1285`) then gives `resolve_advisor`'s one-name/no-floor semantics and the `advisor_not_configured` refusal when advisor is also off. The dispatch-log row records the asked-for seat beside the resolved role (`requested_role`, P3-3). A configured seat resolves normally; marker names the seat (D4). A non-seat unconfigured role keeps T012a refusal on every kind; a seat role on a non-advisor kind keeps its current path untouched (D2's scope) — including the `--kind cell` walk where a cell-declared seat heads `cell_role_list`.

Rejected alternatives:
- Walk `[seat, advisor]` through `resolve_role_named` — the tail's Budget floor (`models.rs:601-604`) hands an unconfigured advisor the session model, verbatim the `4faf1de9` defect; `resolve_advisor` must stay the tail.
- Fall-through for ANY unconfigured role on advisor kind — a typo'd seat name would silently run on the advisor model; refusal is the safer default outside the 8 names.
- Enforce D3 at the dispatch door — requires superseding the display-only description law for one field-read; doctor advisory honors D3's intent (self-documenting config) without touching resolution law.

Risk map: prepare.rs role arm / MEDIUM / refusal + fall-through + null-slot tests · models.rs constant / LOW / unit tests · model_guard parity / LOW / contract test · doctor advisory / LOW / one fixture · docs / LOW / pointer check.

## Shape

One slice, 3 cells:

1. `lmd-1` (role: code) — `models.rs`: `SEAT_ROLES` constant (case-folded membership helper); `prepare.rs`: advisor-kind seat fall-through on resolves-nothing (P1-2 shape), `requested_role` on the payload/log row; `doctor`: advisory finding for configured `hat-*` slots lacking a description (raw-config read). Unit tests in `verbs::drivers` covering the matrix. Cites D1, D2, D3 (deviation venue), D4.
2. `lmd-2` (role: test) — model-guard/parity contract tests: configured seat marker admits; unconfigured seat marker denies; fallen-through dispatch's marker reads `advisor`; `--kind cell` with a cell-declared seat role keeps today's `cell_role_list` path byte-for-byte. Cites D4; store `3c9d6262`.
3. `lmd-3` (role: docs) — `gates-and-delegation.md`: blind-lane fan-out names `--role lane-N` per lane; hat table gains its role names; one line per section pointing at `SEAT_ROLES` as the constant of record. `.bee/config-sample.json`: seat-role examples with hat descriptions. Cites D5, D3.

## Test matrix

Triad plus the reviewer's four holes:
- Happy: configured `lane-2` resolves its model, marker `[bee-tier: lane-2]`; configured `hat-risks` (object slot with description) resolves.
- Edge: absent `lane-3` on `--kind advisor` falls through to advisor, marker `advisor`, `requested_role: "lane-3"`; **null** `"lane-3": null` behaves identically (P1-2); mixed-case `--role Lane-2` resolves a configured `lane-2` and falls through when unconfigured (P2-2); string-shaped `"hat-risks": "opus"` resolves at dispatch and is flagged by doctor (display-only law intact); seat role on `--kind gather` refuses `role_not_configured` (T012a unchanged); typo `hat-risk` refuses `role_not_configured`.
- Error: fall-through with advisor also unconfigured/null refuses `advisor_not_configured` (P3-2: that code, not `role_not_configured`); doctor reports each configured hat slot missing a description by name.

## Out of scope

- `models.pi` runtime block (pi-support feature).
- Any change to blind-lane/hat-wave procedure mechanics, dossier format, `bee blind check`.
- Porting `bee config validate` off Node; superseding the display-only description law.
