# Product Backlog

<!--
GENERATED FILE — do not hand-edit.
Rendered by `bee backlog render` from event-sourced PBI records in .bee/backlog.jsonl (backlog-unification D1/D3).
Regenerate: `bee backlog render --write`. Check freshness: `bee backlog render --check`.
Deterministic: byte-identical for the same backlog.jsonl contents — status-grouped, id-sorted entries, LF endings,
never a generation timestamp or any other wall-clock value.
-->

| ID | Story | CoS | Status | Feature |
|----|-------|-----|--------|---------|
| p-50115b87 | Herd registry: name agents once (herding.agents), reference by name from models.*.<slot> ({kind:herd}) and herding.agent_command | Registry map name -> transport (cli one-shot command+promptVia OR herding pane argv); {kind:herd} arm in normalize_tier_value resolving at dispatch-prepare, unknown name = typed refusal naming the registry keys; herding.agent_command accepts a herd name; docs + samples. GRAY AREAS FOR SHAPING: (1) one registry or two — a herd name can mean a cli one-shot (gather/review/advisor) or a pane worker (cell execution via kind:herding, proven green in the 2.14.1 dogfood hee-1) — the resolver must know which transport a name carries; (2) key naming (herding.agents vs top-level agents); (3) whether a herd name on generation implies herding transport for cells and cli for gathers on the same slot. Relayed from the vnbptw-mapcompany session on the user's behalf, 2026-08-20; their gather-only constraint note predates 2.14.x. | proposed | herd-registry |
| p-74522649 | The fleet crate's unfocused-status hazard test is intermittently flaky, so one of the three recorded herdr hazards has unreliable proof | A judge running the full workspace suite observed crates/fleet/tests/herdr_backend.rs::status_does_not_depend_on_the_pane_having_been_focused FAIL once with left: Unverifiable, right: Finished, then pass 3/3 in isolation and on 3 subsequent full-workspace runs (2026-08-19, during cell ho-13's review). It is stub-backed with no real herdr, so the flake is in the test or its stub harness, not in a live server. Why it matters more than an ordinary flake: this test is the recorded proof for one of the three hazards the distill named on the herdr path — that a status read must not depend on the pane having been focused — and a hazard whose proof fires intermittently is a hazard that is not actually pinned. It also erodes the instrument: every cell in this feature was judged by mutating code and reading which tests go red, and a test that fails for its own reasons makes that instrument lie in both directions. CoS: the cause is identified (likely shared state or ordering between stub-backed tests running in parallel, since it passes in isolation), the test is deterministic under repeated full-suite runs, and if the cause is shared state the fix generalises to the other stub-backed tests in that file rather than being applied to this one alone. | proposed | — |
| p-814029a8 | The fleet crate stops shipping its fault-injecting fake backend inside the bee binary | packages/bee-rs/crates/fleet/src/backend.rs:12 declares pub mod fake; with no #[cfg(test)] and no feature gate, so FakeBackend and its whole fault-injection API compile into the shipped library that links into the bee binary (herding-orchestration D5). Found by the independent judge on cell ho-6, 2026-08-18; it is surface and dead weight, not a correctness bug, which is why it was not folded into that cell's rework. CoS: the fake is available to the crate's own unit tests and to its integration tests under tests/, and is absent from a release build of the bee binary. The awkward part is that integration tests under tests/ link the crate as an external consumer, so a bare #[cfg(test)] hides it from them — a named feature (for example test-support) enabled for the crate's own dev builds is the likely shape. Verify by inspecting the built artifact or by a compile-fail check that a non-test consumer cannot name FakeBackend. | proposed | — |
| p-9bab9fb0 | bee gate --lane writes a record bee worktree merge never reads, so a lane-run feature can never pass uat | GIVEN a feature started as a lane (bee state start-feature --as-lane), WHEN the owner approves its uat gate with bee gate --name uat --approved true --lane <feature>, THEN bee worktree merge sees the approval and lands the branch.  Today it does not. Hit for real on 2026-08-18 merging feature uat-stop-placement: the owner approved, the approval was written to .bee/lanes/uat-stop-placement.json as approved_gates.uat true, and bee worktree merge still refused WORKTREE_MERGE_UAT_PENDING. The merge was landed with the documented one-merge escape --skip-uat, recorded as a named deviation in the decisions log.  Cause: the merge-time precheck (uat_merge_precheck in packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs) reads the live workflow record gates.uat.approved first and the default .bee/state.json record approved_gates.uat as a same-feature fallback. It reads the LANE record at neither step. bee gate --lane writes only the lane record. The two never meet, so the gate is unreachable for any lane-run feature — and lanes are exactly what a session must use when the default pipeline is occupied.  Confirmed shape: .bee/lanes/uat-stop-placement.json carried approved_gates.uat true while the workflow record for the same feature still carried gates.uat {approved: false, state: pending}.  Fix direction: either have bee gate --lane also write the feature workflow record gates.<name>, or have the merge precheck consult the lane record as a third source. The first is likely correct — the workflow record is what every other reader consults, so the lane write should keep it in step rather than every reader learning a second place to look. Whichever is chosen, the same gap should be checked for the other four GATE_NAMES, not just uat. | proposed | uat-stop-placement |
| p-9fdc119a | Worker brief carries an expertise/context section: the knowledge-area docs and skill references the task needs, so the worker knows what capabilities to apply | 1) Brief gains an Expertise section written by the dispatcher like a leader briefing a worker: per entry, the file's PATH, its PURPOSE, and one line 'read this to do X correctly'. 2) Entries point at bee's own expertise — skill reference files under skills/ (e.g., style laws, contracts) and bee knowledge — not merely project docs. 3) The standalone-executor clause is rescoped: the worker stays OUTSIDE the bee workflow (never runs bee commands, no lifecycle/state writes), but is NOT denied bee expertise content — reading listed skill/knowledge files is explicitly allowed and encouraged. 4) Dispatcher (Main/orchestrator or herding dispatch) picks the entries per task, like a leader judging what skills the job needs; empty section renders nothing. 5) Same section shape reused in the bee-swarming Task-tool worker brief. 6) Proof: dry-run brief shows path+purpose entries; brief wording no longer tells the worker to ignore the listed files. | proposed | — |
| p-b94491a8 | The unattended cockpit stops stealing the owner's view every time it spawns a working agent | herdr 0.8.0's agent start has no --no-focus flag and moves the workspace's global tab focus (observed live in skills/bee-herding/references/spawn-proof.md, cell ho-2, 2026-08-18). pane split and tab create both honor --no-focus; agent start does not. The dispatch loop runs on a fixed interval, so every spawn pulls the owner away from whatever they were looking at — the exact failure --no-focus exists to prevent everywhere else in the cockpit. CoS: after a dispatch spawns a working agent, the owner's focused tab is the one they had before the spawn. Options to weigh, none chosen yet: restore focus explicitly after agent start returns (a focus command of our own, which the herdr skill notes also marks a tab seen and so perturbs idle/done); ask upstream herdr for --no-focus on agent start; or accept and document it. Blocked on nothing; it is a real defect in the unattended experience, not a blocker for the herding-orchestration feature. | proposed | — |
| p-cf66d519 | bee skills carry re-runnable eval suites that prove the skill changes behavior, not just that it exists | Every bee skill gets an evals/ suite in the format 'claude plugin eval' actually runs (evals/**/case.yaml, or prompt.md plus graders/*.md) — not a hand-rolled evals.json. Each suite carries: at least one positive case per major branch of the skill; at least one NEGATIVE-trigger case proving the skill does NOT fire on an adjacent request it should stay out of; and, for skills whose value is an ordering (bee-swarming, bee-herding), a case whose expected behavior names the ordering itself. Runs under --ablation with-without so the score is a delta against a no-skill baseline arm, with --max-cost-usd set. CI runs the suite for any skill whose SKILL.md changed in the diff. This complements, never replaces, the bee-writing-skills RED phase: RED is the hand-run baseline before a skill is written, evals are the re-runnable proof afterwards. Prior art and the case shapes worth copying: the herdr-agent-comms suite (7 cases, incl. a negative-trigger case and an ordering case) at luongnv89/skills 48730b3, digested in docs/history/research/herdr-orchestrator-distill.md. Today bee has zero evals folders across 12 skills. | proposed | — |
| p-e742f275 | bee state start-feature refuses on ANY active reservation, including another worktree's unrelated work | GIVEN a live sibling session holding reservations for its own feature in its own worktree, over paths the new feature never names, WHEN a second session runs bee state start-feature for an unrelated feature, THEN the start succeeds; only a genuine path overlap refuses.  Hit for real on 2026-08-18: starting feature uat-stop-placement was refused because session d24210c3 held packages/bee-rs/crates/fleet/* for cell ho-8 of feature herding-orchestration, in a different worktree, over a path uat-stop-placement never touches. The owner asked the obvious question: ho-8 is another worktree, why is it related. It is not.  Cause: packages/bee-rs/crates/bee/src/verbs/state_group/policy.rs:487-495 calls list_reservations(root, true, now_ms()) and refuses when the list is non-empty — repo-wide, unscoped by feature, worktree, session, or path. The correctly-scoped check already exists a few lines above for the lane path at policy.rs:365, which refuses only on declared-path overlap.  Workaround used: start the feature as a lane (--as-lane --paths ...), which takes the overlap-scoped check instead. That works but means the default pipeline is unusable whenever any sibling holds any reservation.  Fix direction: give the default-pipeline start the same overlap scoping the lane path already has — refuse only on reservations whose paths overlap what this feature declares, or held by this same session. Releasing a live sibling's reservations is never the remedy (critical pattern: never release another agent's reservations on a stall signal). | proposed | uat-stop-placement |
| p-ea551b87 | bee close's doc-deferral door must scan the lines a feature changed, not every line of a touched doc | GIVEN a feature whose cell edits one line in a large docs/ file that already contains deferral-shaped prose elsewhere, WHEN bee close runs, THEN the doc-deferral door reports only lines that feature actually changed, and pre-existing prose in the same file never blocks the close.  Evidence this is real: closing feature staging-optional on 2026-08-18 blocked on 7 lines in docs/handbook/register.md (288, 290, 291, 296, 314, 372, 481). Cell so-2 added exactly ONE config-table row to that file. All 7 predate the change (git show 668a60e1^:docs/handbook/register.md), and all 7 are false positives: they are the register section that DOCUMENTS .bee/deferred-queue.jsonl, plus table rows describing that bee close 'defers to merge when one exists'. The word 'defer' is the subject matter, not a promise to act later, so registering a trigger for them would be a lie and rewording would drop the name of the file being documented.  Cause: doc_deferral_scan_files at packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:1006-1030 takes each capped cell's files_changed, filters to docs/, and full-text scans the whole file. Only fenced code is exempt (close.rs:1075-1082) — headings and table rows are not.  Fix direction: diff-scope the scan against the merge base or the cell's recorded hunks; failing that, exempt headings and table rows the way fenced code already is. | proposed | staging-optional |
| p-ea779324 | Optimistic concurrency mode: reservations become advisory warnings, git merge resolves file overlap | User direction (2026-08-17): parallel sessions should exploit git's merge machinery instead of hard file-level reservation denies. Acceptance: (1) a config switch (e.g. reservations.mode=advisory\|strict) — advisory records the overlap, warns both sessions, and proceeds instead of refusing; (2) hard deny remains for high-risk lane and same-hunk/likely-conflict detection if cheaply detectable; (3) worktree merge surfaces any real git conflict as its own fix-first work item instead of silent resolution; (4) docs/skills teach: split cells by file boundary first, advisory overlap second; (5) strict stays default until user flips it. | proposed | — |
| p-f0de3f03 | worktree merge's pre-merge auto-commit sweeps all of docs/knowledge, not just the merging feature's subtree | GIVEN two sessions working different features in the same checkout, WHEN one runs bee worktree merge and its pre-merge bookkeeping auto-commit fires, THEN it commits only paths belonging to the merging feature, and a sibling session s in-flight docs/knowledge edits stay uncommitted and unattributed to the wrong feature.  Hit for real on 2026-08-18. Merging feature uat-stop-placement produced bookkeeping commit 7429dfda, subject "Auto-commit .bee, docs/decisions, docs/knowledge, and docs/history/uat-stop-placement bookkeeping before merging worktree beehive--wt--uat-stop-placement". It swallowed 21 insertions of a SIBLING session s spec sync to docs/knowledge/areas/workflow-state/gates.md, work belonging to feature start-feature-reservation-scope. No content was lost, but the authorship and the feature attribution are both wrong.  Note the asymmetry in the same prefix list: docs/history/<feature>/ IS scoped to the merging feature, while docs/knowledge/, docs/decisions/, and .bee/ are blanket. So the intent to scope is already there for one of the four.  This is the same shared-index sweep the concurrent-worker git guard exists to prevent — the guard refuses a bare git add while siblings are live, then the merge path does the equivalent itself.  Fix direction: scope the docs/knowledge/ (and docs/decisions/) sweep the way docs/history/ is already scoped, or narrow the auto-commit to paths the merging feature s own capped cells recorded in files_changed. Leaving a sibling s edits uncommitted is the correct outcome, not a failure — they belong to that sibling s own commit. | proposed | uat-stop-placement |
| p-ead9b2d4 | A bundle-only handover is proven: an area is rebuilt from its concepts alone, with no runtime and no source tree | TRIAGED 2026-08-16: parked — valid experiment (rebuild one area from docs/knowledge concepts alone on another stack) but it is a human-priced cross-stack session, not a code fix; needs the user to choose the area and spend the session. Evidence and scope unchanged from P69's last clause. | parked | knowledge-handover-proof |

## Done / Declined

- [p-06af049e] Bash write-guard extractor treats every non-flag token after cp/mv as a write target, so a READ operand can trigger a refusal — done
- [p-0a0fda78] 22 promote-proposals.md la kenh chet: bee close sinh proposal moi feature nhung 0/22 delivery draft duoc apply — duong promote thuc te la capture-queue — done
- [p-0a47ea8d] Per-check filter in suite harness (BEE_CHECK_ONLY): cell verify runs only relevant checks, full suite stays at slice tail + CI — done
- [p-0aa807b9] run_verify --impacted gains --level 1: direct-edge-only selection for the dev loop, transitive closure reserved for wave-close/merge — done
- [p-10caed3f] timings report verb: aggregate .bee/logs/timings.jsonl into slowest-command ranking — done
- [p-10e22a70] knowledge check bat con tro chet: path trong frontmatter sources/body Integration Points tro file khong ton tai — lop ri set lam ks-2 BLOCKED (bee-executing khai tu) — done
- [p-19b21cf2] Doc-link integrity test for docs/knowledge: every relative md link and [[wiki-link]] must resolve, enforced in cargo test — done
- [p-21583c96] Cua kiem luc close: solution co pham pattern da biet khong — check nhe truoc khi cap/close doi chieu diff voi critical patterns cua area cham toi — done
- [p-2eb26c53] Extend the unresolvable-shell-syntax family: brace/glob expansion and cd are not modelled by the Bash write-guard — done
- [p-3416fb38] multisession-native slice 2: workflow-first state (D1/D6/D7) — done
- [p-349ee32a] worktree merge deadlocks: it demands a clean main, but the worktree-first guard refuses the commit that would clean it — done
- [p-355d4740] Nhan 'critical' loang: 85/101 pattern (84%) gan critical — digest 4 dong chon tu tap gan-toan-bo, mat gia tri loc — done
- [p-3d6877c2] Skill token diet wave 2: migrate remaining skills (exploring, reviewing-remainder, briefing, herding, compounding, executing, qualifying...) to thin-body doctrine, remove grandfather exceptions — done
- [p-4072163a] Widen decisions * verbs from the narrow door to the wide door so a granted feature worktree can read/write its own workspace-local decision store — done
- [p-47d864b5] Do tai pham: KPI that cua knowledge la incident cung loai khong lap lai — dem va bao cao repeat-incident rate theo pattern — done
- [p-4ae119b0] Port worktree-concurrency-guard onto bee 1.18.2's new packages/bee layout and controlRoot architecture — done
- [p-4d735567] Add evidence-state ladder to knowledge patterns (Present/Wired/Exercised) so written-only patterns are visible risk — done
- [p-4f055a6f] multisession-native slice 5: integration queue + 15-invariant closure — done
- [p-50de38d7] Ship visibility: draft PR on first cap, walking-skeleton slice 1, evidence-based lane demotion, progress ticks (spec ak/plans/reports/spec-260727-1632-bee-ship-visibility.md) — done
- [p-50f3af4d] bee cells schedule should detect shared regen-obligation side-effects, not rely solely on declared cell files — done
- [p-62f0566d] bee --help --json does not list state gate's --actor / --bypass-level / --reason flags — done
- [p-6ec778c5] Step ticks: mandatory ak-style per-step progress lines (route/gates/dispatch/cap/verify/barrier/sync/close), fixed format, user language, bypass never silences — done
- [p-7037485e] Anchor miss lam digest roi ve recency fallback ngay ca khi feature dang bound (vd decisions-worktree-door) — pattern lien quan khong duoc uu tien — done
- [p-71d014e7] Reap orphaned workflow record on default feature swap — done
- [p-727e9529] Port bee core mjs to single compiled Rust binary queen-bee (CLI + hooks), jsonl storage unchanged, <5ms invocation — done
- [p-72af01ca] Status diet: status --brief fast path + workers stop paying full status at startup (dispatch embeds state line) — done
- [p-7484c2ad] One wave-batch dispatch call replaces per-cell prepare ceremony — done
- [p-7f564c5e] Workflow-record lifecycle: every startFeature path creates a record; close covers by feature; workflows list/close CLI verb — done
- [p-7fceeba1] Feature-close events: scribing+compounding run once at full feature completion; per-slice keeps only capture stubs — done
- [p-804bb35b] Three copies of scribing_debt exist; only one was reconciled with the deferred queue — done
- [p-808487c4] Session-close mid-phase warning omits the decision-0017 remedy: work finished inline mid-phase should stub to capture-queue + close, not only 'finish/cap or HANDOFF' — done
- [p-81a97109] Pure-binary installer: Node-free onboarding once queen-bee ships — done
- [p-8aae1301] Skill token diet: thin-body doctrine + byte budget fence for bee skills (spec ak/plans/reports/spec-260727-1619-bee-skill-token-diet.md) — done
- [p-8afb88a4] Test batching: slice test cell replaces per-cell red-first on feature work (spec ak/plans/reports/spec-260727-1626-bee-test-batching.md) — done
- [p-91ceee70] Foundation fixes: workflow close transition kills state-clobber zombies; Windows suite split kills 600s timeout — done
- [p-94ecc5a2] bee state gate does not reject an unknown gate name or a non-boolean --approved — done
- [p-9c48a67c] bee worktree new/register copies ALL .bee/cells files into a new worktree, including other features' stale uncapped claimed cells — done
- [p-9d7b36fc] repeat bash install rewrites managed files (timestamp churn) — done
- [p-a07607cb] Worktree-first enforcement: any code-touching action forks a feature worktree from the start instead of editing on main and hitting hold blocks — done
- [p-aa37ec95] Write-guard Bash refusals name a shell token instead of a path when the target cannot be resolved — done
- [p-b11732c2] bee worktree new must require --with-companion (or refuse) when a live concurrent session touches a shared companion checkout — done
- [p-b595a094] Main-verifies: feature-level verify by the orchestrator; workers implement+commit+report only; cap pending path + close-door gate — done
- [p-c15fb6f5] As a doctrine author, I want the pointer checks to read a citation naming more than one heading, so its target file and every heading it names are verified instead of the citation being skipped. — done
- [p-c7db35b1] Verb pull len tang always-loaded: preamble/AGENTS nêu bee knowledge search mot dong — session plain-turn/off-rail biet duong keo ma khong can load skill nao — done
- [p-c9e20303] One deterministic regen verb replaces the remembered three-step chain — done
- [p-cc2500e0] Explicit triage: machine-readable route record (class\|lane\|flags\|files) required at feature start, surfaced in status+preamble, updated by re-lane — done
- [p-ccc558a0] Installer verify fails: bee.mjs status reports drift=true post-apply when repo-copy sync leaves an orphaned extra lib file — done
- [p-d494b04b] Bootstrap bundle mot lenh cho host repo: goglbe khong co docs/knowledge nen khong digest/pattern/search — can duong dung bundle re tu specs hien co — done
- [p-d7c88155] Duong keo giua dong: bee knowledge search — tra pattern/area theo trieu chung (error string, hanh vi sai, ten co che) ngay giua luc solve, khong chi 2 diem bom co dinh — done
- [p-df2c0284] Advisory diff-vs-test check at cells finish: large diff with zero changed test files prints a non-blocking warning — done
- [p-e0213b88] Worker results carry a structured report schema, not prose — done
- [p-e0234de4] Judge is a mandatory close door for standard and high-risk lanes — done
- [p-e0efd4fe] Longitudinal validation for bee-evolving: a friction fix counts done only when the next comparable run stays clean — done
- [p-e20d82c9] multisession-native slice 3: sharded leases + handoff mailboxes (D4/D5) — done
- [p-e7b82571] A Bash command with any containment-failing literal target delegates the WHOLE command to fail-open, swallowing already-decided denials — done
- [p-e8a153e2] Validation speedup: delta validation, merged review wave, deferred presentation (spec ak/plans/reports/spec-260727-1610-bee-validation-speedup.md) — done
- [p-e8b98793] Registry drift test derives its flag list from set_gate.rs instead of a hardcoded copy — done
- [p-ed2de0d0] multisession-native slice 4: workspace isolation by default (D2/D3) — done
- [p-f5e682e7] bee-compounding rule: a pattern that recurs escalates to a durable owner (hook/guard/test), not another doc line — done
- [p-f82b037d] Write-guard refusal message hardening: bound token echo, literal-$ wording, mixed-command priority, literal message pins — done
- [p-f893dcba] Ap suat flush cho capture queue: qua nguong (vd 5 stub hoac 7 ngay) thi close/preamble chuyen tu nhac nhe sang ep manh, va flush chay duoc tu session thuong — done
- [p-fa847e3a] Parallel-by-default doctrine: cells in a slice run concurrently on disjoint ownership; serial names its conflict; wave-barrier regen — done
- [P1] A greenfield repo with no build gets an init lane on its first onboard — done
- [P10] Gate 4 walkthrough can quiz the approver — done
- [P11] SEE gray areas can be locked by reacting to a throwaway HTML mock — done
- [P12] The orchestrator measures worker results itself before accepting them — done
- [P13] Advisor mode is dogfooded end-to-end — done
- [P14] A worker tier can resolve to an external executor (multi-provider swarm) — done
- [P15] Settlements are recorded without blocking the flow — done
- [P16] Review work can pin its own model, separate from generation — done
- [P17] A tier can carry a reasoning-effort knob — done
- [P18] Bee learns from its own dogfood data and ships the improvement (evolving loop) — done
- [P19] Bee updates its own skill set by script, not by hand-copy — done
- [P2] Backlog rows can be ranked automatically instead of hand-ordered — done
- [P20] Socratic questions pass an explicit materiality test before being asked — done
- [P21] CONTEXT.md pins fuzzy domain terms in a glossary as they crystallize — done
- [P22] Every subagent dispatch is audit-logged with its resolved model/tier — done
- [P23] Bee runs one unified fan-out pattern: best model orchestrates every phase, mechanical work goes down-tier — done
- [P24] Codex receives the same bee skills and lifecycle guardrails as Claude Code — done
- [P26] Full independent review runs only when the user explicitly requests it — done
- [P27] A stuck worker consults a configured advisor before returning [BLOCKED] — done
- [P28] Multiple terminals work one project without stomping each other, and a finished session hands work to a fresh one — done
- [P3] Backlog status renders as README badges — done
- [P30] Tracked append-only .jsonl records merge without conflicts across branches — done
- [P31] `bee.mjs <group> <verb>` is the only CLI an agent ever sees or ships — done
- [P32] Bee knows when to ask a second opinion, and stops doing the work itself — done
- [P37] Slice 1d — SRC source-identity classifier — done
- [P39] Parallel cells stop politely waiting: the scheduler computes waves from declared files+deps — done
- [P4] Gates 2–3 are reviewed on a single human-readable implement plan — done
- [P42] Codex subagent waits do not spam empty completion panels — done
- [P43] Installer upgrades never split the release version across Codex, Claude, runtime, or project skills — done
- [P44] A single session fans work into independent-feature git worktrees, each with its own gates, merged back as event-sourced state — done
- [P45] Exploring asks fewer, batched rounds instead of one question per message — done
- [P46] Git/VCS commands are exempt from the intake gate — done
- [P47] A claim wedged by a hard crash frees itself — done
- [P48] Lane and session records stop being freely hand-writable gate inputs — done
- [P49] `--force-downgrade` names its blast radius before acting — done
- [P5] Capture-mode engages in-flight, not only when a human remembers — done
- [P52] Small work starts from an executable work packet, not a shrunken feature plan — done
- [P56] A crashed session's unsettled context is recovered from the harness transcript — done
- [P57] A reversed decision propagates to every artifact that cites it, a backlog row only flips done when its CoS is actually met, and decisions stay findable at scale — done
- [P59] CLI reads land under 100ms — lazy module loading first, a Rust write-guard hook binary second — done
- [P6] Model tiers are config-driven and runtime-keyed — done
- [P61] Rust write-guard hook as a deferring fast path — done
- [P63] The runtime is Rust, one process at a time, JS deleted as it goes — done
- [P64] Bee's knowledge lives in one validated OKF bundle instead of five scattered document kinds — done
- [P65] An agent loads only the knowledge its current task needs, within a token budget — done
- [P66] Every area spec becomes small typed concepts, and the legacy docs tree is retired — done
- [P67] Finishing work promotes knowledge instead of accumulating it — done
- [P68] Stale, dangling, and self-contradicting knowledge is found by machine, not by memory — done
- [P69] Every bee skill reads and writes the knowledge bundle instead of loose markdown — done
- [P7] Keep the strong model scarce, measurably — done
- [P70] A Codex session runs bee with the same experience as Claude Code — done
- [P71] A session that gets compacted lands back on the work it was doing — done
- [P73] Codex hook execution is a proven fact, not an open question — done
- [P75] A cell cannot be authored without the regen obligations its own file list implies — done
- [P77] The product backlog is per-item files with a generated index, so concurrent sessions never contend on one markdown table and a session reads only the rows it needs — done
- [P8] Advisor pattern (cheap main loop, ceiling on demand) is first-class — done
- [P9] Exploring teaches before asking when the user is in unfamiliar territory — done
- [p-0530164c] Add bee knowledge scribing-target + emit-frontmatter CLI verbs so scribing stops needing node -e import(knowledge.mjs) — declined
- [p-063d7b18] Test-prune đợt 2: test_conformance.mjs cắt về 4 scenario doctor độc quyền (~250 dòng); soi test_msn_invariants.mjs cùng tiêu chí — declined
- [p-179bdcae] AGENTS doctrine block is over its 20 KiB hard budget — declined
- [p-34c37c1f] okf_migrate --check-all: gộp 12 hàng --check <area> trong run_verify thành 1 boot (~7s full verify, verb --verify-pins đã có tiền lệ) — declined
- [p-722dfd0a] Tách god-module templates/lib/state.mjs thành module hẹp để giảm impacted fan-out — declined
- [p-726a4881] Port host-project verify registry (run_verify impacted doctrine) beyond bee mechanics — declined
- [p-759bd218] Test-prune đợt 2: test_okf_pins.mjs cắt nhóm case 4-11 test hạ tầng pin trỏ docs/specs đã đóng băng (~150 dòng) — declined
- [p-78448452] Quy ước import trong suite scripts/: 8 suite import .bee/bin/lib thay vì templates/lib — thống nhất theo test_compaction_module convention — declined
- [p-842955b5] verify_policy deferred cho host project — cap nhận targeted-run, full chain thuần CI — declined
- [p-86f2bfc1] compact-check mutates .bee/ across consecutive runs — declined
- [p-9fb64d0f] Reservation refusal must never write an unreleased cross-worktree hold mirror (self-poisoning deny loop) — declined
- [p-acc0e108] Test-prune đợt 2: race-harness scaffold chung cho 6 suite race (argVal/spawnRacer trùng byte-for-byte, ~250-350 dòng) — declined
- [p-c6e61dfb] P7 stage-instruction offload: self-contained pipeline stages (exploring/scribing/compounding) run in workers loading their own skill; orchestrator keeps hive+swarming+gates only — declined
- [p-cb0e94e3] Test-prune đợt 2: cắt scripts/test_worktree_store.mjs phần grants trùng test_worktree_cli, giữ replayLog; đổi tên tránh trùng basename với templates/tests/test_worktree_store.mjs — declined
- [p-d4a5dcff] Test-prune đợt 2: gộp test_bypass_matrix + test_gate_bypass_doctrine thành một doctrine gate table-driven (~150 dòng) — declined
- [p-dbe8b1a5] reservationStoreCorrupt still checks the legacy reservations.json file — declined
- [p-fb5764e6] run_verify D6: trần kích hoạt mà level-1 direct = 0 suite thì exit 0 im lặng — cân nhắc note to hơn hoặc delegate loud — declined
- [P25] Bee can select Codex custom agent roles when the collaboration runtime exposes reliable role selection — declined
- [P29] A headless outer loop runs validated cells through fresh sessions without a human at the keyboard — declined
- [P33] Turn-count discipline becomes an explicit rule, proven by the logger — declined
- [P34] Advisor thresholds are tuned from measured data, not guessed — declined
- [P35] A session's bill is attributable to lanes and cells — declined
- [P36] Decision records carry a machine-readable claim, so conflicts become detectable — declined
- [P38] Installer offers the status line as a one-flag opt-in — declined
- [P40] Each swarm worker gets its own git worktree — declined
- [P41] Write-time deny gains a wait/queue option — declined
- [P50] The two-terminal fresh-session handoff is proven by a real owner walkthrough — declined
- [P51] Worktree feature-parallelism reaches the host projects — declined
- [P53] The plan freeze is machine-enforced, not prose-ruled — declined
- [P54] Cells carry an explicit slice id for long multi-slice features — declined
- [P55] Lane graduation is automated when evidence crosses a threshold — declined
- [P60] The behaviour corpus is language-neutral, so any implementation can be validated by the same tests — declined
- [P62] Binaries ship reproducibly for five targets before the second port — declined
- [P72] The ~65% context handoff rule is measurable, then enforceable — declined
- [P74] Provider-native compaction transport stays out of bee — recorded, not re-litigated — declined
- [P76] Split `bee-exploring` into an automatic triage stage (`bee-qualifying`), a shared CONTEXT.md-writer (`bee-context-locking`), and a narrowed human `bee-exploring`, so a clear backlog item flows unattended to ready-for-pickup while an ambiguous one parks with a brief instead of blocking on a synchronous question — declined
