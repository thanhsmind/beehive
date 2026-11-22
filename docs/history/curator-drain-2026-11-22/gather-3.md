# Stale promote-proposal audit — round 1 (read-only)

Read: the 8 docs/history/<feature>/promote-proposals.md files (framework-feedback-rule, claude-sub-session-dispatch, report-usage-herding, thin-boot-default, thin-worker-boot, herding-job-feature, paseo-visible-sessions, paseo-worktree-workspace). Searched: docs/knowledge/areas/** and docs/knowledge/patterns/** with rg -n -i. Section (a) ignored as instructed.

### framework-feedback-rule
b: none
c: ffr-1 | ROUTINE | ran inline on leader, herding concurrency full | n/a
c: ffr-1 | ROUTINE | own feature after context-mark feature closed | n/a
c: ffr-1 | ROUTINE | regen rewrote rendered copies and onboarding.json | n/a
c: ffr-1 | ROUTINE | sync-ack: AGENTS.md adds one rule | n/a

### claude-sub-session-dispatch
b: bee-herding | cssd-1 | PRESENT docs/knowledge/areas/bee-herding/claude-session-transport.md:24-25 | claude-session herding transport via --session-model
b: bee-herding | cssd-2 | PRESENT docs/knowledge/areas/bee-herding/claude-session-transport.md:30 | native Claude dispatches route to sub-sessions
c: cssd-1 | LESSON | BEE_HERDING_* env must reach supervisor-started session | UNCOVERED (no pattern on --settings env propagation)
c: cssd-1 | ROUTINE | herding/tests.rs claude_session None in four literals | n/a
c: cssd-1 | ROUTINE | sync-ack: cockpit skill unchanged | n/a
c: cssd-2 | LESSON | test-only thread-local keeps tests hermetic under claude PATH | COVERED docs/knowledge/patterns/20260726-a-harness-that-inherits-ambient-env-shrinks-its-proof.md
c: cssd-2 | LESSON | leader fixed fixture outside cell file list | COVERED docs/knowledge/patterns/20260908-a-cell-s-files-list-is-a-floor-not-a-fence.md
c: cssd-2 | ROUTINE | claude_session module made pub(crate) | n/a
c: cssd-2 | ROUTINE | sync-ack: cockpit skill untouched | n/a

### report-usage-herding
b: bee-herding | ruh-2 | PRESENT docs/knowledge/areas/bee-herding/claude-session-transport.md:96 | herded sessions merged into usage rows
b: workflow-state | ruh-2 | PRESENT docs/knowledge/areas/workflow-state/close-usage-record.md:60-69 | herded sessions merged, counted once, printed
c: none

### thin-boot-default
b: bee-herding | tbd-1 | PRESENT docs/knowledge/areas/bee-herding/thin-peer-boot.md:64 | thin boot on unless explicitly false
b: bee-herding | tbd-2 | PRESENT docs/knowledge/areas/bee-herding/thin-peer-boot.md:88 | hook reads switch from merged config
c: tbd-1 | ROUTINE | sync-ack: skill covers cockpit roles only | n/a
c: tbd-2 | ROUTINE | sync-ack: same as tbd-1 | n/a

### thin-worker-boot
b: bee-herding | twb-1a | PRESENT docs/knowledge/areas/bee-herding/thin-peer-boot.md:97 | herding readers use merged layered config
b: workflow-state | twb-1a | ABSENT | only bee-herding/thin-peer-boot.md:97 states it
b: bee-herding | twb-2a | PRESENT docs/knowledge/areas/bee-herding/claude-session-transport.md:96 | herded transcripts join usage via mailbox
b: workflow-state | twb-2a | PRESENT docs/knowledge/areas/workflow-state/close-usage-record.md:60-69 | usage record gains herding block, counted once
b: bee-herding | twb-3a | PRESENT docs/knowledge/areas/bee-herding/thin-peer-boot.md:51-59 | session-init prints only peer-boot.md
b: workflow-state | twb-3a | ABSENT | no thin-peer text in workflow-state
b: bee-herding | twb-3b | PRESENT docs/knowledge/areas/bee-herding/thin-peer-boot.md:32-43 | four trimming env vars for Claude panes
b: workflow-state | twb-3b | ABSENT | no env-default text in workflow-state
b: bee-herding | twb-5a | ABSENT | no 1276-char thin block or LEADER split in bee-herding (split is in doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md:209)
b: workflow-state | twb-5a | ABSENT | no thin block or LEADER split in workflow-state
b: bee-herding | twb-5b | ABSENT | no leader-doctrine preamble line text anywhere in areas
b: workflow-state | twb-5b | ABSENT | no leader-doctrine preamble line text anywhere in areas
c: twb-1a | LESSON | dispatched fix blocked spawns; leader ran inline | COVERED docs/knowledge/patterns/20261027-a-dispatched-stage-that-fails-preflight-gets-run-inline-by-the-leader-and-the-preflight-never-gets-fixed.md (shape match)
c: twb-2a | ROUTINE | sync-ack: usage record gains herding block | n/a
c: twb-3a | ROUTINE | leader reworked hook scope to session-init only | n/a
c: twb-3a | ROUTINE | sync-ack: default-off hook output | n/a
c: twb-3b | ROUTINE | sync-ack: env defaults behind default-off switch | n/a
c: twb-5a | ROUTINE | sync-ack: doctrine moves verbatim | n/a

### herding-job-feature
b: bee-herding | hjf-1 | ABSENT | job feature stamp not in bee-herding; only workflow-state/close-usage-record.md:74-80, :113
b: workflow-state | hjf-1 | PRESENT docs/knowledge/areas/workflow-state/close-usage-record.md:74 | herded job feature stamped and read first
c: hjf-1 | ROUTINE | sync-ack: herding skill covers cockpit roles only | n/a

### paseo-visible-sessions
b: none (no bee.areas declared)
c: pvs-1 | ROUTINE | sync-ack: herding skill names no transport detail | n/a
c: pvs-2 | ROUTINE | escalated dispatch lands on session-model channel | n/a (documented at claude-session-transport.md:109)

### paseo-worktree-workspace
b: none (no bee.areas declared)
c: pw-1 | LESSON | worker idle 900s with no write; leader ran inline | UNCOVERED (nearest 20261027 pattern; trigger differs: idle timeout, not preflight)
c: pw-1 | ROUTINE | sync-ack: internal transport helpers | n/a
c: pw-2 | ROUTINE | leader executed on session model after escalation | n/a
c: pw-2 | ROUTINE | read_main_config private; paseo_ws reads inline | n/a
c: pw-2 | ROUTINE | sync-ack: worktree verbs internal | n/a
c: pw-3 | ROUTINE | leader executed on session model | n/a
c: pw-3 | ROUTINE | wait source and probe scrubs moved to pw-1 | n/a
c: pw-3 | ROUTINE | sync-ack: herding transport internal | n/a
c: pw-4 | LESSON | plan wrong: D5 made parent-cascade text false | UNCOVERED (nearest 20260723-an-append-only-learning-artifact-transmits-obsolete-advice.md; different trigger)

## Notes
- No CONFLICT found: bundle values for thin switch (default on unless false), escalation channel session-model, claude_sessions switch, and herded-usage join all agree with the proposals.
- Not searched: nothing skipped in the 8 features. Guard refused two rg/cut/for-loop combinations; re-ran as plain rg calls only.
