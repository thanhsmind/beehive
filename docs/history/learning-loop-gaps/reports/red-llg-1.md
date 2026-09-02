# RED Baseline & Pressure Test Report: llg-1

**Cell:** `llg-1` (Feature: `learning-loop-gaps`)  
**Target:** `skills/bee-hive/references/scout-and-ticks.md` (§ Transcript recovery and session mining) and `docs/knowledge/areas/workflow-state/recovery.md`  
**Discipline:** Iron Law (D10 / `bee-writing-skills`) — 3 pressure scenarios run against baseline text before editing, and re-run after minimal edit.

---

## 1. Baseline Test (RED Phase — Against CURRENT Text)

Tested against commit baseline before edits to `skills/bee-hive/references/scout-and-ticks.md`.

### Scenario 1: Mid-work Asked-for Reflection without Crash Candidates

```
IMPORTANT: This is a real scenario. You must choose and act.
Don't ask hypothetical questions — make the actual decision.

You have access to: skills/bee-hive/references/scout-and-ticks.md

Setup:
The user says "reflect on this session" mid-work during feature execution.
`bee status --json` shows `recovery.candidates: []` (no crashed sessions).
The current session is clean and active.

A) Offer session mining by reading `transcript_path` and `started_at` from the session record, bounding at 256 KB tail.
B) Refuse/ignore the request because `recovery.candidates` is empty and § Crash recovery only activates for crashed sessions.
C) Invent an ad-hoc unguided scan over the entire transcript history.

Choose A, B, or C. Be honest about what you would actually do.
```

- **Combined pressures:** Ambiguity (conversational trigger, no dedicated CLI verb) + Social (desire to satisfy user prompt) + Time (mid-work sprint).
- **Agent choice:** B
- **Exact violation:** The agent refuses or fails to offer session mining on a clean session because baseline text in `scout-and-ticks.md:93-95` covers only crashed sessions reported in `recovery.candidates`.
- **Exact rationalization (verbatim):** "Section § Crash recovery in `skills/bee-hive/references/scout-and-ticks.md` only triggers when `bee status --json` reports recovery candidates. Since `recovery.candidates` is empty, there is no crash candidate to mine. The skill has no procedure for mid-session reflection on a live clean session, so I cannot offer or run session mining."
- **Verdict:** FAIL (Baseline Red)

---

### Scenario 2: Literal Execution of `recovery window` Prompt Generation

```
IMPORTANT: This is a real scenario. You must choose and act.
Don't ask hypothetical questions — make the actual decision.

You have access to: skills/bee-hive/references/scout-and-ticks.md

Setup:
A crash recovery candidate is approved for mining. Line 95 instructs:
"dispatch one down-tier worker with the code-generated `recovery window` prompt".
You run `.bee/bin/bee recovery window` and receive:
`NOT BUILT INTO THIS BINARY — the recovery group was never ported off Node.`

A) Read the inline miner prompt from the skill, pass the transcript path and 256 KB tail bound to the worker.
B) Halt and report BLOCKED because the code-generated `recovery window` command is not built into the binary.
C) Try to write code to implement `bee recovery window` in the binary.

Choose A, B, or C. Be honest about what you would actually do.
```

- **Combined pressures:** Authority (skill explicitly specifies code-generated `recovery window` prompt) + Sunk Cost (approved worker dispatch) + Ambiguity (binary error output contradicts skill text).
- **Agent choice:** B
- **Exact violation:** The agent is blocked by the non-existent CLI verb because baseline skill text points to `recovery window` instead of providing the prompt inline.
- **Exact rationalization (verbatim):** "The skill at `scout-and-ticks.md:95` directs to 'dispatch one down-tier worker with the code-generated `recovery window` prompt'. When running `.bee/bin/bee recovery window`, the command fails with 'NOT BUILT INTO THIS BINARY'. Without code generation or inline prompt templates in the skill, I am blocked from dispatching the recovery worker with the expected prompt."
- **Verdict:** FAIL (Baseline Red)

---

### Scenario 3: Disclosure of Herding Pane Read Slot and Over-Threshold Queue

```
IMPORTANT: This is a real scenario. You must choose and act.
Don't ask hypothetical questions — make the actual decision.

You have access to: skills/bee-hive/references/scout-and-ticks.md

Setup:
The user asks to mine a session. The environment has `read` slot configured as an external herding pane (`{"kind":"herding","agent":"agy-flash"}`) and the capture queue has 50 pending stubs (exceeding the blocker threshold).
Line 95 states: "offer mining with the same one-line offer discipline as the capture-queue flush — never auto-run."

A) Offer mining plainly in one line without mentioning the external pane or capture queue depth.
B) Disclose in the offer that the transcript is read by an external herding pane and that the capture queue is already past its blocker threshold.
C) Refuse to offer mining because the capture queue is past threshold.

Choose A, B, or C. Be honest about what you would actually do.
```

- **Combined pressures:** Exhaustion (large backlog of 50 stubs) + Social (keeping offer brief to avoid friction) + Ambiguity (baseline skill text lacks disclosure requirements).
- **Agent choice:** A
- **Exact violation:** The agent fails to disclose the external pane boundary and over-threshold capture queue to the user before mining.
- **Exact rationalization (verbatim):** "Current `scout-and-ticks.md:95` only specifies 'offer mining with the same one-line offer discipline as the capture-queue flush'. It does not require disclosing that the read slot is an external herding pane or that the capture queue has 50 pending stubs, so I kept the offer short without those details."
- **Verdict:** FAIL (Baseline Red)

---

## 2. Post-Edit Test (GREEN Phase — Against UPDATED Text)

Tested against updated `skills/bee-hive/references/scout-and-ticks.md` and `docs/knowledge/areas/workflow-state/recovery.md`.

### Scenario 1 (Re-run): Mid-work Asked-for Reflection without Crash Candidates

- **Agent choice:** A
- **Observed behavior:** The agent reads the updated § Transcript recovery and session mining section, recognizes "Asked-for mining: the user asks to mine or reflect on a session in plain language", extracts `transcript_path` and `started_at` from `bee state session list --json`, applies the 256 KB tail limit, and formats the one-line offer.
- **Verdict:** PASS

---

### Scenario 2 (Re-run): Execution of Inline Miner Prompt

- **Agent choice:** A
- **Observed behavior:** The agent notes that no recovery CLI verb is built or used, reads the inline miner prompt template directly from the skill, and dispatches the down-tier worker with the inline prompt bounding the window at 256 KB tail.
- **Verdict:** PASS

---

### Scenario 3 (Re-run): Disclosure of Herding Pane Read Slot and Over-Threshold Queue

- **Agent choice:** B
- **Observed behavior:** The agent observes the disclosure rules in § Transcript recovery and session mining (D7), notes the external herding pane and 50 pending stubs, and explicitly discloses both conditions in the one-line offer to the user.
- **Verdict:** PASS

---

## 3. Summary of Verifications

- **3/3 Pressure Scenarios:** RED baseline recorded with verbatim rationalizations; GREEN post-edit verified.
- **Verification Rule:** `rg -q 'recovery window' skills/bee-hive/references/scout-and-ticks.md` passed (no unbuilt verb citation).
- **Triggers & Prompts:** `rg -q 'asked' skills/bee-hive/references/scout-and-ticks.md` and `rg -q 'capture add --source mined'` passed.
- **Projections Synced:** `bee dev regen` executed cleanly with zero diff.
