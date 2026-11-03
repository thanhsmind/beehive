---
mode: tiny
---

# Plan: Windows path escaping in the herding ledger test

## Summary

Rewrite one test's hand-formatted JSON with `serde_json::json!` so a Windows
temp path no longer breaks the parse and the Windows CI job goes green.

Mode: `tiny` — 0 risk flags: none
Why this is the least workflow that protects the work: one test body, same
assertions; the Windows CI run after the push is the deterministic net.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The Windows job fails on exactly this test with a JSON escape error | ran | GitHub run 35716941630, `gh run view --log-failed` | `expected Result(blocked), got Malformed { error: "result-1.json is not valid JSON: invalid escape at line 1 column 92"`; `test result: FAILED. 3789 passed; 1 failed; 20 ignored` |
| 2 | The test formats the path into a raw JSON string | read | `packages/bee-rs/crates/bee/src/herding/run.rs:9645-9646` | `r#"{{"status":"blocked",…,"report_path":"{}"}}"#,` then `report_file.display()` |
| 3 | No other test in the file does the same | ran | `rg -n 'report_path":"\{\}' packages/bee-rs/crates/bee/src/herding/run.rs` | one hit, line 9645 |

## Cells (current slice)

```json
[
  {
    "id": "wjp-1",
    "feature": "windows-json-path",
    "title": "Escape the report path in the herding ledger test",
    "lane": "tiny",
    "role": "code",
    "deps": [],
    "decisions": ["63e42e22-0d65-4940-bb9a-f0072052b587"],
    "files": ["packages/bee-rs/crates/bee/src/herding/run.rs"],
    "read_first": ["packages/bee-rs/crates/bee/src/herding/run.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "In herding/run.rs test ledger_outcome_blocked_with_report_fills_evidence, replace the format! raw JSON string for result-1.json with serde_json::json!({status, summary, files_changed, proof, report_path: report_file.display().to_string()}).to_string() (per D1). Keep every assertion unchanged.",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee ledger_outcome_blocked_with_report_fills_evidence",
    "must_haves": {"truths": ["the test builds result-1.json with serde_json::json! and still passes on Linux"], "artifacts": [], "key_links": [], "prohibitions": ["Do not change the assertions", "Do not touch production code"]},
    "behavior_change": false
  }
]
```
