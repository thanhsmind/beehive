# CLI provenance map

The `bee` CLI's help text (`bee --help [--json]`, rendered from
`packages/bee/lib/command-registry.mjs`) used to cite bee's own development
history inline: decision ids (D1, AO13, 0017…), feature/cell slugs
(lpsp-2, multisession-native-15…), CONTEXT.md references, and issue numbers.
Those tags were removed from the user-facing description strings (P1 —
Provenance exile); the behavioral statements they annotated were kept in
plain product language. This file preserves the tag→string mapping that used
to live inline, so the rules can still be traced back to their decisions.
One row per edited description string ("(desc)" = the command's own
description; "--flag" = that parameter's description). Code comments in the
same files still carry their citations and were deliberately left alone.

| Command / string | Provenance tags removed |
| --- | --- |
| status (desc) | lpsp-2, payload-size; D6, multisession-native-8 |
| status --brief | status-diet D1 |
| cells.add (desc) | ce-2 |
| cells.claim (desc) | D1 (msh-2); D3 |
| cells.verify --signature | D1 |
| cells.cap (desc) | main-verifies D1; D3 |
| cells.cap --feature-verify-pending | main-verifies D1 |
| cells.claim-next (desc) | fresh-session-handoff fsh-11, D2/D4; D3 (msh-2) |
| cells.reset-budget (desc) | D2; GH #27.4 (D-GHF-C) |
| cells.judge-record (desc) | D5 (self-correcting-loop); Δ6 |
| reservations.reserve (desc) | fresh-session-handoff D3; multisession-native-13, D4 |
| reservations.reserve --session | D3 |
| decisions.log (desc) | decision-propagation D7b |
| decisions.log --tags | decision-propagation D4a |
| decisions.supersede (desc) | decision-propagation D2; D6; decision-propagation D7b |
| decisions.supersede --tags | decision-propagation D6 |
| decisions.supersede --scope | decision-propagation D6 |
| decisions.active (desc) | decision-propagation D4a |
| decisions.active --all | decision-propagation D4c |
| decisions.active --untagged | dp-5; decision-propagation D7d |
| decisions.active --cell | state-query-surface sqs-b1, D 5ca69717 |
| decisions.active --feature | state-query-surface sqs-b1, D 5ca69717 |
| decisions.search (desc) | decision-propagation D4a, D8b |
| decisions.search --all | decision-propagation D4c |
| decisions.search --untagged | dp-5; decision-propagation D7d |
| decisions.search --cell | state-query-surface sqs-b1, D 5ca69717 |
| decisions.search --feature | state-query-surface sqs-b1, D 5ca69717 |
| decisions.archive (desc) | decision-propagation D4c |
| decisions.tag (desc) | decision-propagation D7c |
| decisions.render (desc) | decision-propagation D4b/D6; D7/D8; D6 |
| decisions.render --all | decision-propagation D4c |
| state.set (desc) | chain-integrity D1-REVISED; compounding-gate D2; i54-closeout D7 |
| state.set --waive-scribing-debt | chain-integrity D4 |
| state.set --waive-compounding | compounding-gate D2 |
| state.gate (desc) | validation-diet D2; i54-closeout D7; D14; AO3/AO13; D15 |
| state.gate --merge | Validation-diet D2 |
| state.gate --owner | packages-engine-move-3 |
| state.plan-rev.bump (desc) | multisession-native-9, CONTEXT.md D7, advisor consult slice 2 C2; C5, multisession-native-10; D2/D15; invariant 3; D7 |
| state.plan-rev.bump --no-lane | C5 |
| state.scribing-run (desc) | i54-closeout D7; sqs-b3 |
| state.compounding-run (desc) | compounding-gate D1; compounding-gate D2; i54-closeout D7 |
| state.route (desc) | explicit-triage CONTEXT.md D1; CONTEXT.md Outstanding Questions; D1; D3; "mode-gate" reworded to triage/lane classification |
| state.route --lane | "mode-gate" reworded to triage |
| state.route --flags | "mode-gate" reworded to triage |
| state.route --files | "mode-gate" reworded to triage |
| state.feature-verify.record (desc) | main-verifies D2; D5; D3 |
| state.feature-verify.show (desc) | main-verifies D2 |
| state.workflows.list (desc) | workflow-lifecycle wl-2 (rule-12 gap closed) |
| state.workflows.close (desc) | workflow-lifecycle wl-2 (rule-12 gap closed); rule 12 |
| state.start-feature (desc) | D6, multisession-native-8; C3; D2/D4 |
| state.start-feature --session-id | C3 |
| state.lanes (desc) | D2/D4 |
| state.rebuild-projections (desc) | multisession-native-7, D1; C5, multisession-native-10; multisession-native-16 |
| state.session.list (desc) | fresh-session-handoff fsh-1/fsh-3 |
| state.session.bind (desc) | D2/D4 |
| state.handoff.write (desc) | fresh-session-handoff fsh-9, D1; multisession-native-15, D5 |
| state.handoff.write --lane | multisession-native-15 |
| state.handoff.write --target-role | multisession-native-15 |
| state.handoff.write --session-id | multisession-native-15 |
| state.handoff.adopt (desc) | fresh-session-handoff fsh-9, D1; multisession-native-15, D5 |
| state.handoff.adopt --lane | multisession-native-15 |
| state.handoff.adopt --target-role | multisession-native-15 |
| state.handoff.show (desc) | D1; multisession-native-15 (D5) |
| state.handoff.show --lane | multisession-native-15 |
| state.handoff.show --target-role | multisession-native-15 |
| state.handoff.show --session-id | multisession-native-15 |
| state.advisor-ref.record (desc) | AO3/AO13; hive law 12; i54-closeout D7 |
| state.advisor-ref.show (desc) | AO13 |
| state.compact-log (desc) | compaction-hardening D3/D4/D5; D3's helper floor; D5; D4 |
| state.compact-check (desc) | D12/D13; D13; D10; D3's helper floor |
| state.compact-capsule (desc) | D6; D12; D9; D7; D19; D3's helper floor |
| backlog.rank (desc) | backlog-unification D3 |
| backlog.propose (desc) | backlog-unification D3 |
| backlog.pbi.status (desc) | exploring-D11a |
| capture.add (desc) | decision 0017 |
| intent.set (desc) | D2 |
| intent.show (desc) | D5 |
| intent.advance (desc) | D1 |
| reviews.create (desc) | R5; A10; A6 |
| reviews.record (desc) | R5 |
| reviews.candidate.add (desc) | GitHub #16 |
| reviews.status (desc) | R10; A7 |
| feedback.digest (desc) | P18 |
| feedback.collect (desc) | D2b |
| knowledge.check (desc) | D23; D4; D18; D13 |
| knowledge.check --strict | D4 |
| knowledge.check --json | D13 |
| knowledge.index (desc) | D21; D4 (OKF §9 kept — it cites the OKF spec, not bee history) |
| knowledge.index --check | D21/D4 stale-generated-index |
| knowledge.list (desc) | D15; D4 |
| knowledge.list --type | D18 vocabulary |
| knowledge.list --lifecycle | D19 |
| knowledge.context (desc) | D27; G5; G11 |
| knowledge.context --work | D32 |
| knowledge.context --budget | D27/D12 |
| knowledge.context --lane | i54-closeout D3 |
| knowledge.promote (desc) | D38/D2; D10 |
| knowledge.promote --work | D32 |
| worktree.new (desc) | GH #21; worktree-companion-hook |
| worktree.merge (desc) | GH #21, decision D8; unregister D8; multisession-native-22, D8 stage 5; "before this cell" |
| worktree.unregister (desc) | P40 default |
| herding.enable (desc) | D4 |
| herding.disable (desc) | D4 |
| config.set --local | D2, incident a7d2069 |
| config.unset --local | D2 |
| recovery.scan (desc) | D1; D2 |
| recovery.window (desc) | D3; D4 |
| dispatch.prepare (desc) | hardening-7 |
| dispatch.prepare --worker | hardening-7 |
| doctor (desc) | g22-3, D4; capability matrix row F1; D6 |
| doctor.attest (desc) | g22-3, D5-REVISED |

`packages/bee/bee.mjs` needed no help-string edits: its `--help` output is
rendered entirely from the registry above, and its per-group usage-fallback
strings carry no provenance tags. Runtime warning/refusal messages in
bee.mjs (e.g. the scribing-debt and feature-verify refusals) still cite
decisions; they are behavioral output that tests pin, not help text, and
were left out of scope.
