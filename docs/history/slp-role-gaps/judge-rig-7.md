Re-judge of rig-7 after its golden rework (bee-review, opus, review stage).

```json rig-7
{"schema": "judge-verdict/1", "verdict": "PASS", "checks": [
{"id": "truth-advisor-falsifier", "status": "PASS", "evidence": "packages/bee/prompts/advisor.md:17"},
{"id": "truth-blind-shuffled-scoring", "status": "PASS", "evidence": "gates-and-delegation.md Converge Base step and hat-wave synthesis"},
{"id": "truth-reviewer-head-check", "status": "PASS", "evidence": "packages/bee/prompts/reviewer.md:3-4 and skills/bee-reviewing/SKILL.md:51-55"},
{"id": "rework-goldens-bytes-only", "status": "PASS", "evidence": "bd933d3f0 changes only expected bytes in drivers/tests.rs; no assert changed"},
{"id": "rework-pinned-row-exact", "status": "PASS", "evidence": "cli_shape.rs KNOWN_HISTORICAL_EXCEPTIONS 14 to 15; row is an exact substring of plan.md:177; no_shipped_command_spelling 1 passed"},
{"id": "tests-green", "status": "PASS", "evidence": "drivers:: 441 passed; instruction_laws 9 passed"}
], "fixability": "automatic", "confidence": "high"}
```
