# Bee-Planning Provenance

Decision IDs and supersession history removed from live `bee-planning` text.

## Playbooks and Cell Rules
- **Playbook citation (`132551fb`) and file split (`playbook-file-split`, decision `bed1853c`):** Plans cite `skills/bee-planning/playbooks/<class>/<situation>.md` (or the class index for classes without situations) rather than copying steps; playbooks moved out of `planning-reference.md` into `skills/bee-planning/playbooks/`.
- **Contract-only cell actions (`agent-anti-patterns` D1):** Cell actions state outcomes and contracts, leaving internal structure to the worker so unproven details do not become constraints.
- **Predicted `affects_skills` and `affects_specs` (`D3`):** Required flat arrays on every cell in every lane.

## Plan Check — Hat Wave
- **Hat wave absorbs plan-reviewer (`proactive-leader-intake` D4, decision `b34fdea9`):** The plan-step hat wave replaced the old merged plan-reviewer subagent dispatch.
- **Advisor-kind hat seats (`proactive-leader-intake` D3, decision `423e1664`):** Dispatched via `bee dispatch prepare --kind advisor --role <hat-role>` (three default seats: `hat-facts-gaps`, `hat-alternatives`, `hat-user-impact`; all five on `high-risk`), replacing reviewer-tier subagents.
- **No wave on clear/tiny asks (`proactive-leader-intake` D2, decision `a52c854d`):** Preserves the fast path and avoids ceremony capture.

## Retired Keys
- **`commands.verify` retirement:** Retired in favor of `commands.test`; configs still carrying `commands.verify` trigger `RETIRED_VERIFY_KEY_WARNING` in `onboard/templates.rs`.
