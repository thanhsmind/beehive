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
| P43 | Installer upgrades never split the release version across Codex, Claude, runtime, or project skills | Both top-level installers fail closed on a mixed release tuple and prove greenfield plus brownfield end to end before reporting success; 1.3.1 released 2026-07-16 with the Linux half proven (Bash E2E 15 cases as mandatory verify suite; read-only pre-confirm, rollback, status-gated refusals; plugin-first tuple coverage; managed-set cleanup fencing) and all 9 hosts onboarded to 1.3.1; Windows/PowerShell E2E remains the open half (owner request 2026-07-16; decisions 55ff17ef, 09b776b5, fc76ce41) | in-flight | installer-version-parity-1-3-1 |
| P69 | Every bee skill reads and writes the knowledge bundle instead of loose markdown | NARROWED (2026-08-11): three of five clauses shipped earlier; the compounding-promote clause is satisfied by the close-time auto-mined promote proposals converging into the capture queue (R80/R83) with bee-capturing as the reader; the ONE remaining clause is the bundle-only handover proof — rebuild an area from its concepts alone with no runtime — still unstarted. | in-flight | okf-switchover-f3 + okf-integration-close-f4 (3 of 5 clauses — see Delivered/Remaining) |
| p-10caed3f | timings report verb: aggregate .bee/logs/timings.jsonl into slowest-command ranking |  | proposed | — |
| p-9d7b36fc | repeat bash install rewrites managed files (timestamp churn) | RE-SCOPED (2026-08-11 triage): the named e2e suite is gone with the mjs retirement; the behavior claim (repeat bash install rewrites managed files) needs a Rust-side regression check against installer_contracts.rs before this can rank again — as written, unactionable. | proposed | verify-red-triage |
| p-c6e61dfb | P7 stage-instruction offload: self-contained pipeline stages (exploring/scribing/compounding) run in workers loading their own skill; orchestrator keeps hive+swarming+gates only | Cumulative orchestrator instruction load ~halved on top of P1-P6; dispatch topology decision spec'd and approved | proposed | — |
| P48 | Lane and session records stop being freely hand-writable gate inputs | Gate decisions read lane/session records that any process can hand-edit — a forged record can flip a gate or force compounding-complete. Decide the integrity mechanism (checksums, CLI-only writes like decisions.jsonl, or hook-guarded paths) and close the hole (source: v0.1.44 review finding, security + architecture, promoted from machine backlog 2026-07-17) | proposed | — |

## Done / Declined

- [p-0a0fda78] 22 promote-proposals.md la kenh chet: bee close sinh proposal moi feature nhung 0/22 delivery draft duoc apply — duong promote thuc te la capture-queue — done
- [p-0a47ea8d] Per-check filter in suite harness (BEE_CHECK_ONLY): cell verify runs only relevant checks, full suite stays at slice tail + CI — done
- [p-0aa807b9] run_verify --impacted gains --level 1: direct-edge-only selection for the dev loop, transitive closure reserved for wave-close/merge — done
- [p-10e22a70] knowledge check bat con tro chet: path trong frontmatter sources/body Integration Points tro file khong ton tai — lop ri set lam ks-2 BLOCKED (bee-executing khai tu) — done
- [p-21583c96] Cua kiem luc close: solution co pham pattern da biet khong — check nhe truoc khi cap/close doi chieu diff voi critical patterns cua area cham toi — done
- [p-3416fb38] multisession-native slice 2: workflow-first state (D1/D6/D7) — done
- [p-355d4740] Nhan 'critical' loang: 85/101 pattern (84%) gan critical — digest 4 dong chon tu tap gan-toan-bo, mat gia tri loc — done
- [p-3d6877c2] Skill token diet wave 2: migrate remaining skills (exploring, reviewing-remainder, briefing, herding, compounding, executing, qualifying...) to thin-body doctrine, remove grandfather exceptions — done
- [p-4072163a] Widen decisions * verbs from the narrow door to the wide door so a granted feature worktree can read/write its own workspace-local decision store — done
- [p-47d864b5] Do tai pham: KPI that cua knowledge la incident cung loai khong lap lai — dem va bao cao repeat-incident rate theo pattern — done
- [p-4ae119b0] Port worktree-concurrency-guard onto bee 1.18.2's new packages/bee layout and controlRoot architecture — done
- [p-4f055a6f] multisession-native slice 5: integration queue + 15-invariant closure — done
- [p-50de38d7] Ship visibility: draft PR on first cap, walking-skeleton slice 1, evidence-based lane demotion, progress ticks (spec ak/plans/reports/spec-260727-1632-bee-ship-visibility.md) — done
- [p-50f3af4d] bee cells schedule should detect shared regen-obligation side-effects, not rely solely on declared cell files — done
- [p-6ec778c5] Step ticks: mandatory ak-style per-step progress lines (route/gates/dispatch/cap/verify/barrier/sync/close), fixed format, user language, bypass never silences — done
- [p-7037485e] Anchor miss lam digest roi ve recency fallback ngay ca khi feature dang bound (vd decisions-worktree-door) — pattern lien quan khong duoc uu tien — done
- [p-71d014e7] Reap orphaned workflow record on default feature swap — done
- [p-727e9529] Port bee core mjs to single compiled Rust binary queen-bee (CLI + hooks), jsonl storage unchanged, <5ms invocation — done
- [p-72af01ca] Status diet: status --brief fast path + workers stop paying full status at startup (dispatch embeds state line) — done
- [p-7f564c5e] Workflow-record lifecycle: every startFeature path creates a record; close covers by feature; workflows list/close CLI verb — done
- [p-7fceeba1] Feature-close events: scribing+compounding run once at full feature completion; per-slice keeps only capture stubs — done
- [p-808487c4] Session-close mid-phase warning omits the decision-0017 remedy: work finished inline mid-phase should stub to capture-queue + close, not only 'finish/cap or HANDOFF' — done
- [p-81a97109] Pure-binary installer: Node-free onboarding once queen-bee ships — done
- [p-8aae1301] Skill token diet: thin-body doctrine + byte budget fence for bee skills (spec ak/plans/reports/spec-260727-1619-bee-skill-token-diet.md) — done
- [p-8afb88a4] Test batching: slice test cell replaces per-cell red-first on feature work (spec ak/plans/reports/spec-260727-1626-bee-test-batching.md) — done
- [p-91ceee70] Foundation fixes: workflow close transition kills state-clobber zombies; Windows suite split kills 600s timeout — done
- [p-94ecc5a2] bee state gate does not reject an unknown gate name or a non-boolean --approved — done
- [p-9c48a67c] bee worktree new/register copies ALL .bee/cells files into a new worktree, including other features' stale uncapped claimed cells — done
- [p-b11732c2] bee worktree new must require --with-companion (or refuse) when a live concurrent session touches a shared companion checkout — done
- [p-b595a094] Main-verifies: feature-level verify by the orchestrator; workers implement+commit+report only; cap pending path + close-door gate — done
- [p-c15fb6f5] As a doctrine author, I want the pointer checks to read a citation naming more than one heading, so its target file and every heading it names are verified instead of the citation being skipped. — done
- [p-c7db35b1] Verb pull len tang always-loaded: preamble/AGENTS nêu bee knowledge search mot dong — session plain-turn/off-rail biet duong keo ma khong can load skill nao — done
- [p-cc2500e0] Explicit triage: machine-readable route record (class\|lane\|flags\|files) required at feature start, surfaced in status+preamble, updated by re-lane — done
- [p-ccc558a0] Installer verify fails: bee.mjs status reports drift=true post-apply when repo-copy sync leaves an orphaned extra lib file — done
- [p-d494b04b] Bootstrap bundle mot lenh cho host repo: goglbe khong co docs/knowledge nen khong digest/pattern/search — can duong dung bundle re tu specs hien co — done
- [p-d7c88155] Duong keo giua dong: bee knowledge search — tra pattern/area theo trieu chung (error string, hanh vi sai, ten co che) ngay giua luc solve, khong chi 2 diem bom co dinh — done
- [p-e20d82c9] multisession-native slice 3: sharded leases + handoff mailboxes (D4/D5) — done
- [p-e8a153e2] Validation speedup: delta validation, merged review wave, deferred presentation (spec ak/plans/reports/spec-260727-1610-bee-validation-speedup.md) — done
- [p-ed2de0d0] multisession-native slice 4: workspace isolation by default (D2/D3) — done
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
- [P44] A single session fans work into independent-feature git worktrees, each with its own gates, merged back as event-sourced state — done
- [P45] Exploring asks fewer, batched rounds instead of one question per message — done
- [P46] Git/VCS commands are exempt from the intake gate — done
- [P47] A claim wedged by a hard crash frees itself — done
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
- [p-acc0e108] Test-prune đợt 2: race-harness scaffold chung cho 6 suite race (argVal/spawnRacer trùng byte-for-byte, ~250-350 dòng) — declined
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
