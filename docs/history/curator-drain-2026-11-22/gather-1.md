# Stale promote-proposal audit (read-only), 8 features

Paths read: the eight docs/history/<feature>/promote-proposals.md files (agent-efficiency-skills, agent-efficiency, lean-context, finish-phase, device-aware-cap, agent-shell-rename, worker-status-digest, machine-cap-half); docs/knowledge/areas/{workflow-state,bee-herding,onboarding,hook-runtime,doctrine-layer,worktree-parallelism,rust-runtime}/ (targeted rg); docs/knowledge/patterns/ (targeted rg). Nothing written.

Notes: no area dir named workflow-engine exists, so finish-phase searched the whole bundle. Onboarding was searched with about 8 terms only, so its ABSENT lines are weak. Failure-signature ids have no text in the proposals and are marked UNCLASSIFIED.

### agent-efficiency-skills
b: workflow-state | asf-1 | ABSENT (partial: dissent-and-the-verdict-duty.md:273 says a gap leader-check reopens a cell; no herding-run trailer-commit cap statement) | skills name one-verb feature start/finish; herding run caps
b: bee-herding | asf-1 | ABSENT (same partial as above; herding-run cap text not found) | skills name one-verb feature start/finish; herding run caps
b: workflow-state | asf-2 | PRESENT gates.md:61 | feature start leaves lane mode null; small routes skip plan.md
b: bee-herding | asf-2 | ABSENT (only plan.md mentions, no as-lane rule) | feature start leaves lane mode null; small routes skip plan.md
c: asf-1 | LESSON | routing prose said gap verdict records without reopen; code reopens | COVERED docs/knowledge/patterns/20260831-prose-that-recommends-a-shape-drifts-from-the-resolver-that-computes-it.md
c: asf-1 | ROUTINE | left regen churn and sibling file out of commit | path-scoped commit
c: asf-1 | ROUTINE | sync-ack: herding text lives in references, not SKILL.md | sync-ack

### agent-efficiency
b: workflow-state | aef-1 | PRESENT gates.md:62 | feature finish refuses outside main checkout
b: bee-herding | aef-1 | ABSENT in area (whole bundle: gates.md:62, gates.md:227) | feature finish refuses outside main checkout
b: onboarding | aef-1 | ABSENT (whole bundle: gates.md:62) | feature finish refuses outside main checkout
b: workflow-state | aef-2 | CONFLICT bee-herding/the-paseo-channel.md:248-249 (leader caps the cell by hand after the run) | herding run auto-caps finished cell with green proof
b: bee-herding | aef-2 | CONFLICT bee-herding/the-paseo-channel.md:248-249 (leader caps the cell by hand) | herding run auto-caps finished cell with green proof
b: onboarding | aef-2 | ABSENT (whole bundle: CONFLICT the-paseo-channel.md:248-249) | herding run auto-caps finished cell with green proof
b: workflow-state | aef-3 | CONFLICT doctrine-layer/lane-and-working-discipline.md:322-325 (editing prompts is a binary change; skew makes dispatch prepare refuse every worker) | dispatch prepare works after prompt change, no rebuild
b: bee-herding | aef-3 | CONFLICT doctrine-layer/lane-and-working-discipline.md:322-325 (whole-bundle hit) | dispatch prepare works after prompt change, no rebuild
b: onboarding | aef-3 | ABSENT (whole bundle: CONFLICT lane-and-working-discipline.md:322-325) | dispatch prepare works after prompt change, no rebuild
b: workflow-state | aef-4 | ABSENT (no goal-check miss wording in bundle; nearest dissent-and-the-verdict-duty.md:273) | goal-check miss recorded as gap verdict, reopens capped cell
b: bee-herding | aef-4 | ABSENT (no goal-check wording in bundle) | goal-check miss recorded as gap verdict, reopens capped cell
b: onboarding | aef-4 | ABSENT | goal-check miss recorded as gap verdict, reopens capped cell
b: workflow-state | aef-5 | ABSENT (dispatch.md:410 names a Prior rounds block; no full-reason rule) | prior rounds digest carries block and reopen reasons in full
b: bee-herding | aef-5 | ABSENT (same) | prior rounds digest carries block and reopen reasons in full
b: onboarding | aef-5 | ABSENT | prior rounds digest carries block and reopen reasons in full
b: workflow-state | aef-6 | ABSENT (no statement of the prompt trim rule; worker prompt facts at dispatch.md:162-177 do not cover it) | worker cell prompt trimmed to what skill carries
b: bee-herding | aef-6 | ABSENT | worker cell prompt trimmed to what skill carries
b: onboarding | aef-6 | ABSENT | worker cell prompt trimmed to what skill carries
b: workflow-state | aef-7 | ABSENT (capture-queue-and-the-blocker-threshold.md:27,37,42 mention drain offers only; no promote-settlement rule) | capture queue drained, pre-cell promote proposals settled
b: bee-herding | aef-7 | ABSENT | capture queue drained, pre-cell promote proposals settled
b: onboarding | aef-7 | ABSENT | capture queue drained, pre-cell promote proposals settled
c: aef-1 | LESSON | round-2 worker hit 30-min ceiling, deadlocked on nested CwdGuard; plan wrong about a fact | UNCOVERED (nearest 20260901-a-stated-ancestry-direction-is-a-coin-flip-until-it-is-measured.md)
c: aef-1 | ROUTINE | sync-ack: feature-finish skill text landed in asf-1 | sync-ack
c: aef-1 | LESSON | feature finish from non-main checkout stops after merge, exits 0, no stamp or close (feature_group.rs:315-318, failure signature) | UNCOVERED (nearest 20260714-a-fail-open-host-swallows-fail-closed-throws.md, different trap)
c: aef-2 | ROUTINE | sync-ack: bee-herding skill text lands in cell aef-8 | sync-ack
c: aef-3 | ROUTINE | sync-ack: internal render fallback, no skill change | sync-ack
c: aef-5 | ROUTINE | sync-ack: internal digest length change | sync-ack
c: aef-6 | ROUTINE | sync-ack: prompt trim moves no rule | sync-ack
c: aef-7 | LESSON | leader ran promote completes from main; worktree host guard refuses completing verb | COVERED docs/knowledge/patterns/20260813-a-refusal-from-one-verb-of-a-family-does-not-speak-for-the-verb-the-contract-rests-on.md
c: aef-7 | LESSON | slp-role-gaps promote proposal predates the cell and had no complete event (failure signature) | UNCOVERED
c: aef-7 | (same line as above) judge found slp-role-gaps scoped out by date | included in the LESSON above; not separately classified

### lean-context
b: hook-runtime | lnc-1 | CONFLICT hook-runtime/token-economy-of-injected-context.md:126-127 (default is 120000, lowered from 200000; cell says 150000) | prompt-context hook adds one handoff line at threshold
b: doctrine-layer | lnc-1 | CONFLICT (no doctrine-layer hit; whole-bundle hit same line token-economy-of-injected-context.md:126-127) | prompt-context hook adds one handoff line at threshold
c: lnc-1 | ROUTINE | statusline.rs needed no change | no-change note
c: lnc-1 | ROUTINE | execute split into execute_to for test capture | implementation detail
c: lnc-1 | ROUTINE | sync-ack: SKILL.md names 150k mark via lnc-2 | sync-ack
c: lnc-2 | ROUTINE | bee dev regen rewrote skill trees and onboarding.json | regen output
c: lnc-2 | ROUTINE | path-scoped commit because guard refused staging with 2 live workers | path-scoped commit
c: lnc-2 | LESSON | team.claude.docs changed by user; config guard reserves role models for the user | UNCOVERED (no patterns/ hit for user-reserved config)
c: lnc-2 | ROUTINE | sync-ack: AGENTS.md handoff line only | sync-ack

### finish-phase
b: workflow-engine (dir absent; whole bundle) | fph-1 | PRESENT workflow-state/gates.md:229 (advances to swarming after merge, finish-phase D1) | feature finish advances lane phase to swarming
c: none

### device-aware-cap
b: none (no bee.areas). Bundle also holds the behavior: bee-herding/waves-and-occupancy.md:100,108 (CPU/2 default; memory floor), and worktree-parallelism/pruning-dead-worktrees.md:140-144 (green close runs prune sweep, dac-2) — not in the (b) list, reported for reference.
c: dac-1 | ROUTINE | sync-ack: cockpit four-slot cap outside check_limits | sync-ack

### agent-shell-rename
b: none (no capped behavior_change cell)
c: asr-1 | ROUTINE | sync-ack: workflow-state skills moved by sibling asr-2 | sync-ack
c: asr-2 | UNCLASSIFIED | failure_signature a9c98468e397 (id only, no text in proposal) | n/a

### worker-status-digest
b: none (no bee.areas)
c: wsdg-1 | ROUTINE | sync-ack: status digest text ships in wsdg-2 | sync-ack
c: wsdg-2 | ROUTINE | proof line names rg commands without expected clauses (proof separator) | proof-line format
c: wsdg-2 | UNCLASSIFIED | failure_signature f56c05fd7e35 (id only, no text in proposal) | n/a

### machine-cap-half
b: none (no bee.areas)
c: none (0 capped cells)

Summary: 8 features, 0 files written. CONFLICTs: aef-2 (the-paseo-channel.md:248-249), aef-3 (lane-and-working-discipline.md:322-325), lnc-1/lnc-2 (token-economy-of-injected-context.md:126-127). LESSON UNCOVERED: aef-1 x2, aef-7 failure, lnc-2 deviation. LESSON COVERED: asf-1 (prose drift), aef-7 deviation.
