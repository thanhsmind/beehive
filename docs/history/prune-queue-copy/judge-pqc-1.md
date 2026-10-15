High-risk judge for pqc-1 (bee-review, opus, review stage), frozen head e40edc403.

```json pqc-1
{"schema": "judge-verdict/1", "verdict": "PASS", "checks": [
{"id": "head-and-scope-clean", "status": "PASS", "evidence": "HEAD e40edc403 equals the frozen head; porcelain empty; 3 cell files"},
{"id": "truth-copy-of-main-not-kept", "status": "PASS", "evidence": "prune.rs:155 subset return; tests a_capture_queue_copied_from_main_does_not_keep, a_crlf_copy_of_the_main_queue_does_not_keep"},
{"id": "truth-unique-pending-stub-kept-and-named", "status": "PASS", "evidence": "prune.rs:166-175; test a_pending_stub_main_never_names_keeps_and_names_it"},
{"id": "truth-every-doubt-keeps", "status": "PASS", "evidence": "prune.rs:143-151, 182-187, 193-197; tests a_missing_main_queue_with_a_pending_worktree_stub_keeps, an_unparseable_main_queue_line_keeps"},
{"id": "key-link-main-root-passed", "status": "PASS", "evidence": "prune.rs:461"},
{"id": "prohibition-handoff-unchanged", "status": "PASS", "evidence": "prune.rs:127-133 unchanged"},
{"id": "prohibition-no-test-weakened", "status": "PASS", "evidence": "no removed lines in tests.rs; both old queue tests ok"},
{"id": "prohibition-no-code-comments", "status": "PASS", "evidence": "doc comment rewritten in place, net comment count -1"},
{"id": "verify-command", "status": "PASS", "evidence": "worktree:: 213 passed, 0 failed (filtered run)"}
], "fixability": "automatic", "confidence": "high"}
```
