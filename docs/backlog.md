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
| p-b366b91c | Onboarding tells a project when its knowledge bundle is missing or due for upkeep | From knowledge-skill-pair (docs/history/knowledge-skill-pair/CONTEXT.md, Out Of Scope): an onboarding notice that offers bee-knowledge-setup or points at bee-knowledge-upkeep, as verify has; optionally an upkeep-due line in bee knowledge report and the D7 thresholds in config | in-flight | orient-and-onboard-notices |
| p-0d4da479 | Pin the doctor pi belt-contract branch with its own test and decide the dispatch prepare claim-probe race | A test in packages/bee-rs/crates/bee/src/doctor/tests.rs fails when the pi belt-contract branch in doctor.rs (near 1056) is disabled; the prepare.rs claim-probe race (probe verdict discarded before claim_and_reserve_for_dispatch, near 4311) is closed by a test or recorded with bee decisions log. Found by hrh-3, 2026-11-19: a_stale_pi_belt_row_names_bee_onboard_apply passes with the branch disabled. | proposed | — |
| p-2ce14340 | Test bee state plan-conflicts verdict --id a,b end to end so the comma split is guarded | A test in packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs runs the verdict verb with --id a,b and asserts two verdict rows; replacing the comma split in run_plan_conflicts_verdict with a single id turns it red. Found by pcx-t, 2026-11-19. | proposed | — |
| p-0b45c28d | Owner brief staleness: a brief records a digest of its sources and the briefs check flags it stale when they change | A brief may carry a sources digest; the briefs check reports a brief whose sources changed since it was written as stale; a brief with no digest stays valid. Moved out of owner-brief (D13): the spec marked it optional. | parked | owner-brief |
| p-165ef6d7 | Let one bee-reviewing lens run on a second model family | Source: docs/history/research/piggery-xia.md (rank 2), spec drop sup-20261016-a506; piggery's dual-lens template puts two lenses on two model families. bee-reviewing's severity rule counts corroboration only across different models or disjoint lenses (skills/bee-reviewing/SKILL.md:84), but every lens is dispatched --kind reviewer with no --role (SKILL.md:33-40), so all lenses share one review model. Done when: a team.<runtime> role (for example review-alt) can carry one lens to a second model through bee dispatch prepare, the skill names when to use it, absent config keeps today's behavior, and one measured review run compares a cross-model lens with a same-model lens. Not triaged. | parked | — |
| p-1e2aeda9 | Refuse a herding worker's dispatch of a role its own role may not dispatch | Split from piggery-borrow D3: per-role can_dispatch in team config, checked when BEE_HERDING_WORKER is set. Paseo workers already cannot run bee dispatch (hooks/worker_guard.rs); this covers herdr and tmux workers. Acceptance: a worker-env dispatch of a role outside its can_dispatch list refuses with permission.can_dispatch. | parked | — |
| p-3a756890 | Cut bee's own knowledge areas to map, boundaries, invariants, entry points and pointers without losing the reasons (knowledge-from-code D5, D10) | Every bee area concept follows the cut template; the pre-cut tree is frozen under docs/history/knowledge-archive/; every removed reason sentence cites a logged decision; bee knowledge check reports no area entry without a decision link | parked | — |
| p-4ec9c373 | Extend evidence labels to cell/worker claims at cap time (prove-the-whole-path layer) | Worker cap reports carry read/ran/guessed labels on their load-bearing claims, mirroring the R139 plan-gate discipline at the cell layer; scope shaped by its own feature. | parked | — |
| p-58bc5260 | herding run preflights the configured agent and carries the pane tail in a no-ack error instead of ten silent resend rounds | Before the pane split, herding run sends a trivial prompt to the configured agent with a short timeout; a provider rejection fails fast with the provider's own text (e.g. 'model … is not recognized'). When rounds return but no ack appears, the spawn_failed error includes a tail of the pane output. Regression test with a rejected model id. Evidence: docs/discovery/herding-orchestration/evidence/20260822-herding-transport-bypassed.md §2 | parked | herding-orchestration |
| p-5a383d13 | Give the pi belt contract tests a stub harness that runs on Windows | Eight tests in pi_worker_guard_contracts.rs and pi_paseo_heartbeat_contracts.rs are unix-only because their stub bee and paseo are POSIX sh scripts (decision in pi-tests-unix-stubs). Windows needs a stub the belt can spawn as bee.exe or through herding.paseo.command. Acceptance: the eight tests run and pass on Windows CI. | parked | — |
| p-74522649 | The fleet crate's unfocused-status hazard test is intermittently flaky, so one of the three recorded herdr hazards has unreliable proof | A judge running the full workspace suite observed crates/fleet/tests/herdr_backend.rs::status_does_not_depend_on_the_pane_having_been_focused FAIL once with left: Unverifiable, right: Finished, then pass 3/3 in isolation and on 3 subsequent full-workspace runs (2026-08-19, during cell ho-13's review). It is stub-backed with no real herdr, so the flake is in the test or its stub harness, not in a live server. Why it matters more than an ordinary flake: this test is the recorded proof for one of the three hazards the distill named on the herdr path — that a status read must not depend on the pane having been focused — and a hazard whose proof fires intermittently is a hazard that is not actually pinned. It also erodes the instrument: every cell in this feature was judged by mutating code and reading which tests go red, and a test that fails for its own reasons makes that instrument lie in both directions. CoS: the cause is identified (likely shared state or ordering between stub-backed tests running in parallel, since it passes in isolation), the test is deterministic under repeated full-suite runs, and if the cause is shared state the fix generalises to the other stub-backed tests in that file rather than being applied to this one alone. | parked | — |
| p-8b417003 | Request size rule: at most 3 paths and lane standard means one run | A request that touches at most 3 product paths at lane standard or below runs as one run with no split; deferred from path-rigor-floor D4 (8258ae2d), forgent-xia item 3. | parked | path-rigor-floor |
| p-bd1debcd | Supervisor notebook: enforce causal seven-field shape if terse notes prove insufficient | Demonthorn §4.2 wants Observation/Evidence/Suspected-mechanism/Impact/Question/Recommendation/Escalation on every supervisor record; bee supervisor record deliberately keeps a terser causal-lite shape (kind+signal+two-sentence note) per locked decision c80debd7. Revisit only if the terse shape loses causal context in practice - do not add the heavier structure speculatively. See decision 17e9c1f6 (sup-20260831-7f3a gap 4). | parked | — |
| p-cb3c2b41 | Rewrite existing knowledge concepts to the five writing rules |  | parked | knowledge-useful-rules |
| p-cf66d519 | bee skills carry re-runnable eval suites that prove the skill changes behavior, not just that it exists | Every bee skill gets an evals/ suite in the format 'claude plugin eval' actually runs (evals/**/case.yaml, or prompt.md plus graders/*.md) — not a hand-rolled evals.json. Each suite carries: at least one positive case per major branch of the skill; at least one NEGATIVE-trigger case proving the skill does NOT fire on an adjacent request it should stay out of; and, for skills whose value is an ordering (bee-swarming, bee-herding), a case whose expected behavior names the ordering itself. Runs under --ablation with-without so the score is a delta against a no-skill baseline arm, with --max-cost-usd set. CI runs the suite for any skill whose SKILL.md changed in the diff. This complements, never replaces, the bee-writing-skills RED phase: RED is the hand-run baseline before a skill is written, evals are the re-runnable proof afterwards. Prior art and the case shapes worth copying: the herdr-agent-comms suite (7 cases, incl. a negative-trigger case and an ordering case) at luongnv89/skills 48730b3, digested in docs/history/research/herdr-orchestrator-distill.md. Today bee has zero evals folders across 12 skills. | parked | — |
| p-e09a0b7e | A blind-lane citation can still invert a proposal's meaning by quoting one whole sentence that the sentence before it negates | Named limit, recorded by decision 79b5437b and pinned green by the probe a_whole_sentence_quote_framed_by_the_sentence_before_it_still_passes. A proposal reading 'Never do the following. Cache the token on the worker side.' cited as its second sentence alone resolves and passes, because the check's contract is within-sentence provenance and the governing meaning sits in the sentence before. Acceptance for anyone who takes this on: a citation whose immediately preceding sentence forward-references it (ends in a colon, or carries 'the following', 'as follows', 'these:', 'never do', 'do not') must either carry that sentence or refuse naming it; the change ships with red-first probes for at least the colon form and the 'the following' form; and it must NOT refuse ordinary citations - prove that by keeping every existing verbs::blind test green, the whitespace-only-difference probe included. Do not take this on by claiming the citation check proves faithfulness: decision 79b5437b holds that it proves provenance only. | parked | slp-blind-lanes |
| p-ea779324 | Optimistic concurrency mode: reservations become advisory warnings, git merge resolves file overlap | User direction (2026-08-17): parallel sessions should exploit git's merge machinery instead of hard file-level reservation denies. Acceptance: (1) a config switch (e.g. reservations.mode=advisory\|strict) — advisory records the overlap, warns both sessions, and proceeds instead of refusing; (2) hard deny remains for high-risk lane and same-hunk/likely-conflict detection if cheaply detectable; (3) worktree merge surfaces any real git conflict as its own fix-first work item instead of silent resolution; (4) docs/skills teach: split cells by file boundary first, advisory overlap second; (5) strict stays default until user flips it. | parked | — |
| p-ead9b2d4 | A bundle-only handover is proven: an area is rebuilt from its concepts alone, with no runtime and no source tree | TRIAGED 2026-08-16: parked — valid experiment (rebuild one area from docs/knowledge concepts alone on another stack) but it is a human-priced cross-stack session, not a code fix; needs the user to choose the area and spend the session. Evidence and scope unchanged from P69's last clause. | parked | knowledge-handover-proof |
| p-fb5b5d00 | Read-only multi-project bee dashboard (bee-hive web view) with feature timing stats — GitHub issue #69 | A separate read-only Rust binary opens a local port and shows, for every project using bee: backlog, cells by status, lanes, sessions, and feature start/end times with per-day and per-week ship counts, using a registry under ~/.config/bee. Needs discovery and shaping first. | parked | — |
| role-prompt-profiles | Role and hat instruction prompts ship as customizable files, expertise-style | Every role instruction a dispatch renders (worker cell, gather, reviewer, advisor, supervisor tick, herding roles, and the five hat briefs) has a shipped DEFAULT plus a per-repo override file the way .bee/expertise/* already works: the dispatch door reads the override first and falls back to the baked default. The immutable contract half (result form, status tokens, guard pins) stays fixed and test-pinned; only the method/personality half is overridable. Origin: user ask 2026-08-29 after the slp-advisor-nudge ship; the four cell templates currently bake into the binary (edit = rebuild) and hats have no prompt files yet. | parked | — |

## Done / Declined

- [p-00af3fb7] Feature playbook: write the caller's tutorial first for a shared API, and a recipe page of one-line steer prompts (pstack guide G4) — done
- [p-05d2a4f4] A herding-lane worker cannot record a dissent, because herding workers are told never to run a bee command — done
- [p-06af049e] Bash write-guard extractor treats every non-flag token after cp/mv as a write target, so a READ operand can trigger a refusal — done
- [p-0a0fda78] 22 promote-proposals.md la kenh chet: bee close sinh proposal moi feature nhung 0/22 delivery draft duoc apply — duong promote thuc te la capture-queue — done
- [p-0a3554a6] Class playbooks: give bee route --class a procedure — done
- [p-0a47ea8d] Per-check filter in suite harness (BEE_CHECK_ONLY): cell verify runs only relevant checks, full suite stays at slice tail + CI — done
- [p-0a5e6c44] One bee verb returns the whole models role table with descriptions, and every role-choosing door reminds the agent to read it before assigning — done
- [p-0aa807b9] run_verify --impacted gains --level 1: direct-edge-only selection for the dev loop, transitive closure reserved for wave-close/merge — done
- [p-0b2812f8] Check CoS checkability when a PBI is written, not only at dispatch — done
- [p-0dfb7aa9] Ship leader-owned PBI triage in bee doctrine so every host project triages its own proposals — done
- [p-10caed3f] timings report verb: aggregate .bee/logs/timings.jsonl into slowest-command ranking — done
- [p-10e22a70] knowledge check bat con tro chet: path trong frontmatter sources/body Integration Points tro file khong ton tai — lop ri set lam ks-2 BLOCKED (bee-executing khai tu) — done
- [p-1126e3ec] budget-overrun supervisor signal has no input — estimate_minutes is never written — done
- [p-1677892e] Guard: packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs prompt skew assertion was narrowed by the worker to make a stale vendored prompt pass — done
- [p-19b21cf2] Doc-link integrity test for docs/knowledge: every relative md link and [[wiki-link]] must resolve, enforced in cargo test — done
- [p-21583c96] Cua kiem luc close: solution co pham pattern da biet khong — check nhe truoc khi cap/close doi chieu diff voi critical patterns cua area cham toi — done
- [p-215ba2a9] Keep pi belt state on one globalThis slot and leader identity out of children — done
- [p-221e6d0e] A worker session bound to no lane is refused at the commit intake gate because the gate reads the control-root default record — done
- [p-2858afbb] herding run: two residual pointer-delivery suspects — pane_current focus-steal under parallel spawns, and start_with_retry duplicate keystrokes — done
- [p-28defd13] Read cli slots from both team and models in the outward guard — done
- [p-2eb26c53] Extend the unresolvable-shell-syntax family: brace/glob expansion and cd are not modelled by the Bash write-guard — done
- [p-3203f84b] A /how trace flow: fan out 2-4 explorers, return one runtime account — done
- [p-3416fb38] multisession-native slice 2: workflow-first state (D1/D6/D7) — done
- [p-349ee32a] worktree merge deadlocks: it demands a clean main, but the worktree-first guard refuses the commit that would clean it — done
- [p-34ce8a88] The plan-conflict derive cannot be both meaningful and satisfiable: it needs cells the execution gate forbids, and with cells it returns a quarter of the decision log — done
- [p-35008d7f] Section parsers keyed on a heading use first-occurrence find(), which prose can hijack — done
- [p-355d4740] Nhan 'critical' loang: 85/101 pattern (84%) gan critical — digest 4 dong chon tu tap gan-toan-bo, mat gia tri loc — done
- [p-3983800c] Resolve leader-check artifacts against the feature worktree — done
- [p-3d6877c2] Skill token diet wave 2: migrate remaining skills (exploring, reviewing-remainder, briefing, herding, compounding, executing, qualifying...) to thin-body doctrine, remove grandfather exceptions — done
- [p-3f3f67cf] dispatch prepare --kind cell resolves the payload from the cell's recorded tier; a ceiling cell reports channel session-model — done
- [p-4072163a] Widen decisions * verbs from the narrow door to the wide door so a granted feature worktree can read/write its own workspace-local decision store — done
- [p-41569e17] Existence-is-not-evidence: evidence labels on load-bearing plan claims, enforced at the plan gate, plus a mandatory cheap reality-touch before the gate — done
- [p-4251c625] bee blind check's second registry example exits 1 in a real repo, because a checked-in dossier cannot carry dispatch ids that exist in the runtime log — done
- [p-44a1faca] An autonomous run refuses a finish condition it cannot check — done
- [p-45481f13] Repair the opencode tool-registry derivation after the installed opencode-ai changed shape — done
- [p-47d864b5] Do tai pham: KPI that cua knowledge la incident cung loai khong lap lai — dem va bao cao repeat-incident rate theo pattern — done
- [p-49919909] Serialize the herding pane split so concurrent spawns stop all splitting right — done
- [p-4ae119b0] Port worktree-concurrency-guard onto bee 1.18.2's new packages/bee layout and controlRoot architecture — done
- [p-4d735567] Add evidence-state ladder to knowledge patterns (Present/Wired/Exercised) so written-only patterns are visible risk — done
- [p-4e99ddc4] Fix the four Windows-only path tests red on main — done
- [p-4f055a6f] multisession-native slice 5: integration queue + 15-invariant closure — done
- [p-50115b87] Herd registry: name agents once (herding.agents), reference by name from models.*.<slot> ({kind:herd}) and herding.agent_command — done
- [p-50de38d7] Ship visibility: draft PR on first cap, walking-skeleton slice 1, evidence-based lane demotion, progress ticks (spec ak/plans/reports/spec-260727-1632-bee-ship-visibility.md) — done
- [p-50f3af4d] bee cells schedule should detect shared regen-obligation side-effects, not rely solely on declared cell files — done
- [p-5614f047] Test the herding wiring sites that piggery-borrow left covered only through helpers — done
- [p-5771b088] Report each worker's context tokens and turns in bee herding status — done
- [p-596857ac] Consume a pi result-inbox claim only when the leader turn completes — done
- [p-59a26c99] Close the mistake loop end to end: honest intake, visible debt with a puller, guards proven red-then-green, recurrence caught — done
- [p-6061cd5f] Guard the instruction layer: refuse a shipped bee verb that no doctrine home names — done
- [p-607dee91] spawn_failed herding dispatch releases the cell claim and file reservations (or documents the one-liner) — done
- [p-624e2d7d] The opencode tool-registry derivation breaks on opencode-ai 1.18.21, so the declared suite is red on any machine newer than CI's pinned 1.18.16 — done
- [p-627c4c0f] Port the bee guard belt to the opencode 2.x plugin API — done
- [p-62f0566d] bee --help --json does not list state gate's --actor / --bypass-level / --reason flags — done
- [p-64f29f32] The LaneBrief shape rule refuses three kinds of genuinely neutral brief, and nobody had measured that cost until the guard shipped — done
- [p-69bee217] Make a research-class route actually refuse a source edit — done
- [p-6b237338] Page or spill bee herding status --json — done
- [p-6c3e4223] Decide one question-batching policy for shaping and wayfinding — done
- [p-6ec778c5] Step ticks: mandatory ak-style per-step progress lines (route/gates/dispatch/cap/verify/barrier/sync/close), fixed format, user language, bypass never silences — done
- [p-6f2623c9] The herding run worker path never caps its cell — done
- [p-7037485e] Anchor miss lam digest roi ve recency fallback ngay ca khi feature dang bound (vd decisions-worktree-door) — pattern lien quan khong duoc uu tien — done
- [p-71d014e7] Reap orphaned workflow record on default feature swap — done
- [p-7242d305] The dispatch-registration ledger retires its rows at cap, and a wave where nobody returned anything is flagged — done
- [p-727e9529] Port bee core mjs to single compiled Rust binary queen-bee (CLI + hooks), jsonl storage unchanged, <5ms invocation — done
- [p-72af01ca] Status diet: status --brief fast path + workers stop paying full status at startup (dispatch embeds state line) — done
- [p-7484c2ad] One wave-batch dispatch call replaces per-cell prepare ceremony — done
- [p-77bfcb18] Split trace-and-provenance.md into a trace file and a provenance file — done
- [p-7cc2ebe1] Give the herding worker cap one home: the prose cap and herding.limits.concurrency — done
- [p-7d42942d] A pause handoff has no CLI verb to clear it, so a stale record blocks every orient after its feature closes — done
- [p-7f564c5e] Workflow-record lifecycle: every startFeature path creates a record; close covers by feature; workflows list/close CLI verb — done
- [p-7fceeba1] Feature-close events: scribing+compounding run once at full feature completion; per-slice keeps only capture stubs — done
- [p-7fd7ef0e] Make bee herding cancel kill the worker's whole process tree — done
- [p-804bb35b] Three copies of scribing_debt exist; only one was reconciled with the deferred queue — done
- [p-806a3911] Cap the principle list bee orient prints per class — done
- [p-808487c4] Session-close mid-phase warning omits the decision-0017 remedy: work finished inline mid-phase should stub to capture-queue + close, not only 'finish/cap or HANDOFF' — done
- [p-814029a8] The fleet crate stops shipping its fault-injecting fake backend inside the bee binary — done
- [p-81a97109] Pure-binary installer: Node-free onboarding once queen-bee ships — done
- [p-8aae1301] Skill token diet: thin-body doctrine + byte budget fence for bee skills (spec ak/plans/reports/spec-260727-1619-bee-skill-token-diet.md) — done
- [p-8afb88a4] Test batching: slice test cell replaces per-cell red-first on feature work (spec ak/plans/reports/spec-260727-1626-bee-test-batching.md) — done
- [p-911f5ffe] The plan-freeze guard blocks git add, so an approved plan.md can never be committed — done
- [p-91ceee70] Foundation fixes: workflow close transition kills state-clobber zombies; Windows suite split kills 600s timeout — done
- [p-91e36c63] Check a contract version between the pi/opencode belts and the binary — done
- [p-94ecc5a2] bee state gate does not reject an unknown gate name or a non-boolean --approved — done
- [p-982de3cf] Make the bee-design core contract cover every ui command its references name — done
- [p-9b2e8ee4] Stop the dispatch authorize tests writing timings into packages/bee-rs/crates/bee/.bee — done
- [p-9bab9fb0] bee gate --lane writes a record bee worktree merge never reads, so a lane-run feature can never pass uat — done
- [p-9c48a67c] bee worktree new/register copies ALL .bee/cells files into a new worktree, including other features' stale uncapped claimed cells — done
- [p-9d7b36fc] repeat bash install rewrites managed files (timestamp churn) — done
- [p-9deeec40] proactive-leader-intake: hats-first intake — on receiving a human request the leader opens the hat wave itself (one open question per hat, silent brainstorm) and synthesizes the answers into the plan — done
- [p-9fdc119a] Worker brief carries an expertise/context section: the knowledge-area docs and skill references the task needs, so the worker knows what capabilities to apply — done
- [p-a0609bac] Refuse a dispatch over the team's declared limits at the door — done
- [p-a07607cb] Worktree-first enforcement: any code-touching action forks a feature worktree from the start instead of editing on main and hitting hold blocks — done
- [p-a1399c00] A models.<runtime> role slot may carry a description, and the session preamble surfaces it beside the role name so custom roles teach the planner what they are for — done
- [p-aa23f10e] A /why provenance flow: named evidence categories, empty searches reported — done
- [p-aa37ec95] Write-guard Bash refusals name a shell token instead of a path when the target cannot be resolved — done
- [p-abfd7136] Rule ids become spoken: the user invokes one by name and the reply names what changed — done
- [p-b11732c2] bee worktree new must require --with-companion (or refuse) when a live concurrent session touches a shared companion checkout — done
- [p-b1f11111] cargo clippy --all-targets fails on never_loop in the blind dossier parser, and no cell owns it — done
- [p-b4c2cb19] Make the pi belt contract tests pass on CI (node uncaught exception) — done
- [p-b595a094] Main-verifies: feature-level verify by the orchestrator; workers implement+commit+report only; cap pending path + close-door gate — done
- [p-b9700277] The worker consult's Advisor line is never rendered — P27's loop is live on paper, dead in wiring — done
- [p-b97e839a] Split .bee/expertise/review.md into reviewer-side and author-side homes — done
- [p-b9e2a334] Let the leader run its own live proof inside its feature worktree — done
- [p-babca9e6] Show the re-print command for each dispatch wave payload — done
- [p-c15fb6f5] As a doctrine author, I want the pointer checks to read a citation naming more than one heading, so its target file and every heading it names are verified instead of the citation being skipped. — done
- [p-c77e5226] bee backlog add refuses every call with a misleading unsupported_argument_shape error — done
- [p-c7db35b1] Verb pull len tang always-loaded: preamble/AGENTS nêu bee knowledge search mot dong — session plain-turn/off-rail biet duong keo ma khong can load skill nao — done
- [p-c9e20303] One deterministic regen verb replaces the remembered three-step chain — done
- [p-cbe95561] dispatch prepare reports herding transport reachability (transport_ready) so the fallback model is never guessed into — done
- [p-cc2500e0] Explicit triage: machine-readable route record (class\|lane\|flags\|files) required at feature start, surfaced in status+preamble, updated by re-lane — done
- [p-ccc558a0] Installer verify fails: bee.mjs status reports drift=true post-apply when repo-copy sync leaves an orphaned extra lib file — done
- [p-cf8f7437] Print one install line per runtime with a fix command in bee doctor — done
- [p-cff4f8e1] Make verbs::feedback issue_with_injection_title_drops_with_emptied_fields stable on Linux CI — done
- [p-d1f52f8b] bee <verb> --help prints each flag with its registry description and required marker; session rule: read a verb's help once before first use, never re-read — done
- [p-d494b04b] Bootstrap bundle mot lenh cho host repo: goglbe khong co docs/knowledge nen khong digest/pattern/search — can duong dung bundle re tu specs hien co — done
- [p-d7191e92] Skillstation in beehive: install check plus thin alias skills for user-invoked hub skills — done
- [p-d7c88155] Duong keo giua dong: bee knowledge search — tra pattern/area theo trieu chung (error string, hanh vi sai, ten co che) ngay giua luc solve, khong chi 2 diem bom co dinh — done
- [p-df2c0284] Advisory diff-vs-test check at cells finish: large diff with zero changed test files prints a non-blocking warning — done
- [p-e0213b88] Worker results carry a structured report schema, not prose — done
- [p-e0234de4] Judge is a mandatory close door for standard and high-risk lanes — done
- [p-e0efd4fe] Longitudinal validation for bee-evolving: a friction fix counts done only when the next comparable run stays clean — done
- [p-e20d82c9] multisession-native slice 3: sharded leases + handoff mailboxes (D4/D5) — done
- [p-e742f275] bee state start-feature refuses on ANY active reservation, including another worktree's unrelated work — done
- [p-e7b82571] A Bash command with any containment-failing literal target delegates the WHOLE command to fail-open, swallowing already-decided denials — done
- [p-e8a153e2] Validation speedup: delta validation, merged review wave, deferred presentation (spec ak/plans/reports/spec-260727-1610-bee-validation-speedup.md) — done
- [p-e8b98793] Registry drift test derives its flag list from set_gate.rs instead of a hardcoded copy — done
- [p-ea551b87] bee close's doc-deferral door must scan the lines a feature changed, not every line of a touched doc — done
- [p-eb824507] bee decisions search names the knowledge concept that cites a decision — done
- [p-ed2de0d0] multisession-native slice 4: workspace isolation by default (D2/D3) — done
- [p-f0de3f03] worktree merge's pre-merge auto-commit sweeps all of docs/knowledge, not just the merging feature's subtree — done
- [p-f574d0f5] bee-reviewing shows what the lead dismissed and why — done
- [p-f5e682e7] bee-compounding rule: a pattern that recurs escalates to a durable owner (hook/guard/test), not another doc line — done
- [p-f82b037d] Write-guard refusal message hardening: bound token echo, literal-$ wording, mixed-command priority, literal message pins — done
- [p-f893dcba] Ap suat flush cho capture queue: qua nguong (vd 5 stub hoac 7 ngay) thi close/preamble chuyen tu nhac nhe sang ep manh, va flush chay duoc tu session thuong — done
- [p-fa847e3a] Parallel-by-default doctrine: cells in a slice run concurrently on disjoint ownership; serial names its conflict; wave-barrier regen — done
- [p-fbfa8325] Trust checklist before an unattended loop in autonomous-run and herding bootstrap (pstack guide G5) — done
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
- [sup-20260830-7959] Beehive cleans up finished worktrees — done
- [sup-20260831-7f3a] Make bee supervisor actually run — bee-side governance — done
- [sup-20260924-fb8a] Bring pstack's playbook concept into bee, alongside expertise/ — done
- [sup-20261024-794d] Bring design-os (DESIGN:OS / ease-design) into bee — done
- [sup-20261030-76ac] Upgrade bee-design to ease-design 0.6.2 — done
- [sup-20261030-edb1] Bring design-os chart, diagram and motion checks into bee-design — done
- [sup-20261109-57a6] Knowledge as part of the feature flow — load at shaping/planning, write at cap, land with the merge — done
- [sup-20261112-1646] Waggledance CLI first, MCP tools as fallback — done
- [p-0530164c] Add bee knowledge scribing-target + emit-frontmatter CLI verbs so scribing stops needing node -e import(knowledge.mjs) — declined
- [p-063d7b18] Test-prune đợt 2: test_conformance.mjs cắt về 4 scenario doctor độc quyền (~250 dòng); soi test_msn_invariants.mjs cùng tiêu chí — declined
- [p-179bdcae] AGENTS doctrine block is over its 20 KiB hard budget — declined
- [p-2f3d12ff] Investigation route: a read-only class that answers without writing source — declined
- [p-34c37c1f] okf_migrate --check-all: gộp 12 hàng --check <area> trong run_verify thành 1 boot (~7s full verify, verb --verify-pins đã có tiền lệ) — declined
- [p-722dfd0a] Tách god-module templates/lib/state.mjs thành module hẹp để giảm impacted fan-out — declined
- [p-726a4881] Port host-project verify registry (run_verify impacted doctrine) beyond bee mechanics — declined
- [p-759bd218] Test-prune đợt 2: test_okf_pins.mjs cắt nhóm case 4-11 test hạ tầng pin trỏ docs/specs đã đóng băng (~150 dòng) — declined
- [p-78448452] Quy ước import trong suite scripts/: 8 suite import .bee/bin/lib thay vì templates/lib — thống nhất theo test_compaction_module convention — declined
- [p-842955b5] verify_policy deferred cho host project — cap nhận targeted-run, full chain thuần CI — declined
- [p-86f2bfc1] compact-check mutates .bee/ across consecutive runs — declined
- [p-96d73c27] Show context tokens and turns in the pi bee-workers widget — declined
- [p-9fb64d0f] Reservation refusal must never write an unreleased cross-worktree hold mirror (self-poisoning deny loop) — declined
- [p-acc0e108] Test-prune đợt 2: race-harness scaffold chung cho 6 suite race (argVal/spawnRacer trùng byte-for-byte, ~250-350 dòng) — declined
- [p-ad3436bc] Run bee-evolving over the week's feedback digest (user-approved 2026-08-30) — mine the ~620-commit week: frictions, findings, promote proposals, capture queue — declined
- [p-b7e85d77] plan-conflicts derive over-matches: 753 candidates for a standard 9-file feature, hard-blocking Gate 2 — declined
- [p-b94491a8] The unattended cockpit stops stealing the owner's view every time it spawns a working agent — declined
- [p-bea191e4] Decide whether uat_stop should stay "close" now that a foreign supervisor seat can open leads here — declined
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
