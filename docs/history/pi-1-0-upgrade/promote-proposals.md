promote proposal for work item "pi-1-0-upgrade" (docs/history/pi-1-0-upgrade/CONTEXT.md + docs/history/pi-1-0-upgrade/plan.md) — 14 capped cell(s): p1u-1, p1u-2, p1u-3, p1u-4, p1u-5, p1u-6, p1u-7, p1u-8, p1u-9, p1u-10, p1u-11, p1u-12, p1u-13, p1u-14
anchor: history — docs/history/pi-1-0-upgrade/CONTEXT.md, docs/history/pi-1-0-upgrade/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/pi-1-0-upgrade/delivery.md

---
type: bee.delivery
title: pi-1-0-upgrade — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-1-0-upgrade: 14 capped cell(s), 31 recorded deviation(s)."
timestamp: 2026-10-02
bee:
  id: pi-1-0-upgrade-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [docs/history/pi-1-0-upgrade/CONTEXT.md, docs/history/pi-1-0-upgrade/plan.md]
  sources: [docs/history/pi-1-0-upgrade/CONTEXT.md, docs/history/pi-1-0-upgrade/plan.md, .bee/cells/p1u-1.json, .bee/cells/p1u-2.json, .bee/cells/p1u-3.json, .bee/cells/p1u-4.json, .bee/cells/p1u-5.json, .bee/cells/p1u-6.json, .bee/cells/p1u-7.json, .bee/cells/p1u-8.json, .bee/cells/p1u-9.json, .bee/cells/p1u-10.json, .bee/cells/p1u-11.json, .bee/cells/p1u-12.json, .bee/cells/p1u-13.json, .bee/cells/p1u-14.json]
---

# pi-1-0-upgrade — Delivery

## What shipped

- **p1u-1** — codemode and tool_search skip the outer Pi guard and survive stage narrowing; nested writes, bash redirects and unknown tools stay denied; doctor pi READY byte-identical (4 file(s) changed)
- **p1u-2** — Drove bee on Pi 1.0.0: narrowing, codemode read passes and pre-gate write denied, 0 feed system messages per turn, /reload undoes narrowing until next turn, relocation into worktree, no-pane Pi worker done (1 file(s) changed)
- **p1u-3** — Pi 1.0.0 named as proven ceiling (run 20261002-135348-2965648) in belt header, audit page and contract test; 0.85.1 to 1.0.0 audit table added; D4 and D10 backlog rows filed (4 file(s) changed)
- **p1u-4** — write-guard refuses Pi leader source writes and hand-run bee herding run in approved swarming on small/standard/high-risk lanes (3 file(s) changed)
- **p1u-5** — gated phases answer read, bash and bee_advisor (GATED_TOOL_SET) with the harness workflow on; READ_ONLY_TOOLS unchanged; leader set has no verdict; workers keep the full set (2 file(s) changed)
- **p1u-6** — Pi belt passes the stage hook notices through verbatim and narrows only bee-known tools, re-adding them when a stage allows (5 file(s) changed)
- **p1u-7** — Live Pi 1.0 proof: a deepseek-flash leader on a small-lane cell called bee_dispatch, the worker wrote and capped the cell, the drain delivered the result; leader wrote no source (3 file(s) changed)
- **p1u-8** — session-close answers obligations_only with cap and advisor obligations once per key; a cell whose registered worker has no result yet is not owed; the leader's own claim does not hide a cap (3 file(s) changed)
- **p1u-9** — agent_before_settle asks session-close for obligations, shows each user_notice, appends one bee-obligation custom_message each and returns continue true once; the agent_settled nudge is suppressed after a forced turn; /bee-obligation-skip sends skip_key (5 file(s) changed)
- **p1u-10** — Live Pi 1.0 proof: one forced turn for a claimed uncapped cell (leader then dispatched and capped) and one for a gate-ready high-risk plan (leader called bee_advisor x5, did not approve the gate); no repeat on a second settle (2 file(s) changed)
- **p1u-11** — bee herding steer <job-id> --text writes steer-<n>.json through temp+rename in the job mailbox, refuses a missing job, a finished round or a capped cell, and prints the job and steer number (5 file(s) changed)
- **p1u-12** — Record typed scope changes in runtime file and owe one settle turn per change (3 file(s) changed)
- **p1u-13** — belt: async input handler records steer/follow-up text as scope input and passes it unchanged; worker steer drain from the main-checkout mailbox with the fixed prefix, 8 KB and shape limits; verdict refuses a missing job dir when BEE_HERDING_JOB_ID is set; undelivered steers named in the result header; bee_steer leader tool in the leader set (6 file(s) changed)
- **p1u-14** — Live Pi 1.0 proof: a steer reached a running worktree worker once (it declined a scope-changing steer by design), the worktree worker recorded its verdict, a late steer was refused, and a typed scope change got one forced turn while bee refused an unapproved cell add (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **p1u-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee stage_tools` — 88/88 pi_plugin_contracts plus 9 stage_tools unit tests, covering the belt and FULL_TOOL_SET this cell touched; full declared suite not run
- **p1u-2** — `bash .bee/verify/verify-app/control-bee doctor` — sandbox doctor healthy; evidence /home/thanhsmind/.local/state/bee-verify/evidence/20261002-135348-2965648/ (rpc-events.jsonl, drive-summary.txt, tui-*.txt, pi-worker.json); before-state on main: out…
- **p1u-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts` — cell verify; 88 passed; touched belt comments and the Pi contract test doc only, plus line-cite spot checks against the 1.0.0 CHANGELOG
- **p1u-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee write_guard` — 281 write_guard tests incl. 8 new, run as ~/.cargo/bin/cargo because the sandbox refused the PATH-expansion form; touched only write_guard; full suite not run
- **p1u-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee stage_tools` — leader re-ran the cell verify on 0d11eaf6e (bc6167b0c plus the two contract assertions updated to 82179c5d): test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 3890 filtered out; finished i…
- **p1u-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts` — the cell own verify, 93 passed; run without the PATH prefix because the worktree guard refuses the CARGO_HOME expansion and cargo was already on PATH; plus bee dev regen 3/3 green and .bee/bin/bee do…
- **p1u-7** — `bash .bee/verify/verify-app/control-bee doctor` — sandbox doctor healthy; evidence /home/thanhsmind/.local/state/bee-verify/evidence/20261002-153242-3268921/ (rpc-events.jsonl, drive-summary.txt); before-state: first run 20261002-151421-3214679 neve…
- **p1u-8** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee session_close` — leader re-ran the cell verify on 3b98c5e63 (advisor message states 5 runs and names the seats, amendment 562588f5): test result: ok. 61 passed; 0 failed; 0 ignored; 0 measured; 3845 filtered out; fin…
- **p1u-9** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts` — leader re-ran the cell verify on cf0471a14: test result: ok. 98 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.91s; worker also reported doctor --runtime pi READY; whole suit…
- **p1u-10** — `bash .bee/verify/verify-app/control-bee doctor` — sandbox doctor healthy; evidence /home/thanhsmind/.local/state/bee-verify/evidence/20261002-181622-3758076/ and /home/thanhsmind/.local/state/bee-verify/evidence/20261002-184508-3853826/ (drive-*.txt…
- **p1u-11** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding` — leader re-ran the cell verify plus --test registry_contracts on 78b3dc17e: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3918 filtered out; finished in 0.00s; test result: ok. 611 passe…
- **p1u-12** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee session_close` — session_close unit tests pass
- **p1u-13** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts` — leader re-ran the cell verify plus -p bee stage_tools on 5ff056b36: test result: ok. 104 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.01s; test result: ok. 16 passed; 0 fai…
- **p1u-14** — `bash .bee/verify/verify-app/control-bee doctor` — sandbox doctor healthy; evidence /home/thanhsmind/.local/state/bee-verify/evidence/20261002-201522-52484/ (steer-run.txt, drive-scope.txt, rpc-scope.jsonl); before-state: no-pane workers had no input…

## Deviations

- **p1u-1** — mapToolCall returns hook null for codemode/tool_search and the tool_call handler returns undefined on null, so MappedCall.hook widened to write-guard or null — the direct path to skip write-guard without a fake route — found a better route
- **p1u-1** — pi_tool_hook_pairs parser now reads hook: null as route none, and the coverage test pins the unguarded set to exactly codemode and tool_search — without it the parser would misattribute the default arm's write-guard to tool_search — something else had to be fixed first
- **p1u-1** — rebuilt and committed the tracked vendored .bee/bin/bee (reserved under p1u-1-coder) as the action asked; it is not in the cell files list — the plan was wrong about a fact
- **p1u-2** — the leader ran the cell instead of a worker — the worker-outward guard refuses any worker that starts pi — hit an unforeseen obstacle
- **p1u-2** — drove Pi over RPC plus one tmux TUI pass instead of the full verify-app hat-wave leader flow; ran one no-pane Pi worker, not the five-seat wave — /reload is TUI-only and the five-seat wave on 0.85.1 evidence stands; named in the map as not run on 1.0 — found a better route
- **p1u-3** — bee backlog add wrote the two rows into the worktree tracked .bee/backlog.jsonl, which is outside the cell files — the CLI picks the store, and the cell action requires the rows — something else had to be fixed first
- **p1u-3** — Also updated the Re-verified note above PI_BUILTIN_TOOLS in bee-guard.ts to 1.0.0 (same line count) — it named 0.85.1 as the ceiling in the same file — found a better route
- **p1u-3** — Audit table cites event names from the 1.0.0 pi binary because the 1.0.0 docs no longer list ui_prompt_start, ui_prompt_end or session_tree; recorded as a Known gap — the plan was wrong about a fact
- **p1u-4** — source write means a path outside the gated allow-list (.bee/, docs/history/, plans/, AGENTS.md) — the contract does not define source, and this is the list check_write already uses for pre-approval writes — the plan was wrong about a fact
- **p1u-5** — round 3 (bc6167b0c) adds GATED_TOOL_SET per amendment 82179c5d after the p1u-10 live run showed the planning leader could not reach bee_advisor — the plan was wrong about a fact
- **p1u-5** — the herding worker does not cap, so the leader capped after re-running the verify — hit an unforeseen obstacle
- **p1u-6** — the belt also re-activates bee tools a later stage allows (planning to swarming), not only removes them — without it a leader that came from planning never got bee_dispatch back, since the old code only called setActiveTools when something was removed — something else had to be fixed first
- **p1u-6** — the bee-known set is a belt constant BEE_STAGE_TOOLS built from PI_BUILTIN_TOOLS plus five names; parity with stage_tools.rs FULL_TOOL_SET is proven by the new test, which feeds the real hook full list plus two foreign tools and expects only the foreign ones kept — the hook carries no known-tools field and stage_tools.rs is outside this cell — found a better route
- **p1u-6** — left for the orchestrator: the test comment in packages/bee-rs/crates/bee/src/hooks/stage_tools.rs (stage_tools_keeps_the_verdict_keys_it_publishes) still says the belt builds its own sentences and the message keys have no reader; that is now false, and the file is outside this cell files — hit an unforeseen obstacle
- **p1u-7** — leader-run instead of a worker — the worker-outward guard refuses a worker that starts pi — hit an unforeseen obstacle
- **p1u-7** — the inline-write refusal is proven by the real hook probe in the sandbox, not by the model, which never attempted a write — something else had to be fixed first
- **p1u-8** — round 4 (3b98c5e63) adds the run count and seat names per amendment 562588f5 after the live run — the plan was wrong about a fact
- **p1u-8** — round 1 (5f97402a1) followed the plan; round 2 (a75fe2ba1) added the running-worker filter per amendment 90077144; round 3 (100fb7652) dropped the claim-heartbeat half per correction 295f1277 — the plan was wrong about a fact
- **p1u-8** — the herding worker wrote a malformed result-1.json and does not cap, so the leader capped after re-running the verify — hit an unforeseen obstacle
- **p1u-9** — the herding worker does not cap, so the leader capped after re-running the verify — hit an unforeseen obstacle
- **p1u-10** — leader-run instead of a worker — the worker-outward guard refuses a worker that starts pi — hit an unforeseen obstacle
- **p1u-10** — the first advisor run found two gaps (bee_advisor not in the planning set; no run count) and p1u-5 and p1u-8 were reworked before the final run — the plan was wrong about a fact
- **p1u-10** — the leader in the final advisor run stopped before recording the advisor_ref; filed as a backlog finding — something else had to be fixed first
- **p1u-11** — the herding worker does not cap, so the leader capped after re-running the verify and the registry contract tests — hit an unforeseen obstacle
- **p1u-11** — sync-ack: bee herding steer is reached on Pi through the bee_steer leader tool; the skills/bee-herding verb list update is filed as a backlog debt row
- **p1u-12** — followed the plan
- **p1u-12** — sync-ack: cell p1u-12 declared no skill changes
- **p1u-13** — the herding worker does not cap, so the leader capped after re-running the verify — hit an unforeseen obstacle
- **p1u-14** — leader-run instead of a worker — the worker-outward guard refuses a worker that starts pi — hit an unforeseen obstacle
- **p1u-14** — the steer test text asked for a scope change, which the worker correctly declined; delivery is proven by the worker report quoting it — found a better route
- **p1u-14** — in the scope run the small model answered the forced turn with an empty message; filed as a backlog finding — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work pi-1-0-upgrade` from 14 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-1-0-upgrade/CONTEXT.md`, `docs/history/pi-1-0-upgrade/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "pi-1-0-upgrade" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-10-02T14:49:37.185Z), the work item declares no bee.areas.

area hook-runtime:
  - [p1u-5] gated phases answer read, bash and bee_advisor (GATED_TOOL_SET) with the harness workflow on; READ_ONLY_TOOLS unchanged; leader set has no verdict; workers keep the full set — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/p1u-5.json)
  - [p1u-8] session-close answers obligations_only with cap and advisor obligations once per key; a cell whose registered worker has no result yet is not owed; the leader's own claim does not hide a cap — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/p1u-8.json)
  - [p1u-9] agent_before_settle asks session-close for obligations, shows each user_notice, appends one bee-obligation custom_message each and returns continue true once; the agent_settled nudge is suppressed after a forced turn; /bee-obligation-skip sends skip_key — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/p1u-9.json)
  - [p1u-12] Record typed scope changes in runtime file and owe one settle turn per change — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/p1u-12.json)
  - [p1u-13] belt: async input handler records steer/follow-up text as scope input and passes it unchanged; worker steer drain from the main-checkout mailbox with the fixed prefix, 8 KB and shape limits; verdict refuses a missing job dir when BEE_HERDING_JOB_ID is set; undelivered steers named in the result header; bee_steer leader tool in the leader set — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/p1u-13.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell p1u-1 — save as docs/knowledge/patterns/pi-1-0-upgrade-p1u-1-pitfall.md

---
type: bee.pattern
title: pi-1-0-upgrade cell p1u-1 — pitfall candidate
description: "Pitfall candidate mined from cell p1u-1's capped trace: mapToolCall returns hook null for codemode/tool_search and the tool_call handler returns undefined on null, so MappedCall.hook widened to write-guard or null —…"
timestamp: 2026-10-02
bee:
  id: pi-1-0-upgrade-p1u-1-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/p1u-1.json]
  polarity: pitfall
---

# pi-1-0-upgrade cell p1u-1 — pitfall candidate

## What the cell did

codemode and tool_search skip the outer Pi guard and survive stage narrowing; nested writes, bash redirects and unknown tools stay denied; doctor pi READY byte-identical

## Recorded evidence (verbatim from .bee/cells/p1u-1.json)

- **deviation** — mapToolCall returns hook null for codemode/tool_search and the tool_call handler returns undefined on null, so MappedCall.hook widened to write-guard or null — the direct path to skip write-guard without a fake route — found a better route
- **deviation** — pi_tool_hook_pairs parser now reads hook: null as route none, and the coverage test pins the unguarded set to exactly codemode and tool_search — without it the parser would misattribute the default arm's write-guard to tool_search — something else had to be fixed first
- **deviation** — rebuilt and committed the tracked vendored .bee/bin/bee (reserved under p1u-1-coder) as the action asked; it is not in the cell files list — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell p1u-2 — save as docs/knowledge/patterns/pi-1-0-upgrade-p1u-2-pitfall.md

---
type: bee.pattern
title: pi-1-0-upgrade cell p1u-2 — pitfall candidate
description: "Pitfall candidate mined from cell p1u-2's capped trace: the leader ran the cell instead of a worker — the worker-outward guard refuses any worker that starts pi — hit an unforeseen obstacle"
timestamp: 2026-10-02
bee:
  id: pi-1-0-upgrade-p1u-2-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/p1u-2.json]
  polarity: pitfall
---

# pi-1-0-upgrade cell p1u-2 — pitfall candidate

## What the cell did

Drove bee on Pi 1.0.0: narrowing, codemode read passes and pre-gate write denied, 0 feed system messages per turn, /reload undoes narrowing until next turn, relocation into worktree, no-pane Pi worker done

## Recorded evidence (verbatim from .bee/cells/p1u-2.json)

- **deviation** — the leader ran the cell instead of a worker — the worker-outward guard refuses any worker that starts pi — hit an unforeseen obstacle
- **deviation** — drove Pi over RPC plus one tmux TUI pass instead of the full verify-app hat-wave leader flow; ran one no-pane Pi worker, not the five-seat wave — /reload is TUI-only and the five-seat wave on 0.85.1 evidence stands; named in the map as not run on 1.0 — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell p1u-3 — save as docs/knowledge/patterns/pi-1-0-upgrade-p1u-3-pitfall.md

---
type: bee.pattern
title: pi-1-0-upgrade cell p1u-3 — pitfall candidate
description: "Pitfall candidate mined from cell p1u-3's capped trace: bee backlog add wrote the two rows into the worktree tracked .bee/backlog.jsonl, which is outside the cell files — the CLI picks the store, and the cell action…"
timestamp: 2026-10-02
bee:
  id: pi-1-0-upgrade-p1u-3-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/p1u-3.json]
  polarity: pitfall
---

# pi-1-0-upgrade cell p1u-3 — pitfall candidate

## What the cell did

Pi 1.0.0 named as proven ceiling (run 20261002-135348-2965648) in belt header, audit page and contract test; 0.85.1 to 1.0.0 audit table added; D4 and D10 backlog rows filed

## Recorded evidence (verbatim from .bee/cells/p1u-3.json)

- **deviation** — bee backlog add wrote the two rows into the worktree tracked .bee/backlog.jsonl, which is outside the cell files — the CLI picks the store, and the cell action requires the rows — something else had to be fixed first
- **deviation** — Also updated the Re-verified note above PI_BUILTIN_TOOLS in bee-guard.ts to 1.0.0 (same line count) — it named 0.85.1 as the ceiling in the same file — found a better route
- **deviation** — Audit table cites event names from the 1.0.0 pi binary because the 1.0.0 docs no longer list ui_prompt_start, ui_prompt_end or session_tree; recorded as a Known gap — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell p1u-4 — save as docs/knowledge/patterns/pi-1-0-upgrade-p1u-4-pitfall.md

---
type: bee.pattern
title: pi-1-0-upgrade cell p1u-4 — pitfall candidate
description: "Pitfall candidate mined from cell p1u-4's capped trace: source write means a path outside the gated allow-list (.bee/, docs/history/, plans/, AGENTS.md) — the contract does not define source, and this is the list ch…"
timestamp: 2026-10-02
bee:
  id: pi-1-0-upgrade-p1u-4-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/p1u-4.json]
  polarity: pitfall
---

# pi-1-0-upgrade cell p1u-4 — pitfall candidate

## What the cell did

write-guard refuses Pi leader source writes and hand-run bee herding run in approved swarming on small/standard/high-risk lanes

## Recorded evidence (verbatim from .bee/cells/p1u-4.json)

- **deviation** — source write means a path outside the gated allow-list (.bee/, docs/history/, plans/, AGENTS.md) — the contract does not define source, and this is the list check_write already uses for pre-approval writes — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell p1u-5 — save as docs/knowledge/patterns/pi-1-0-upgrade-p1u-5-pitfall.md

---
type: bee.pattern
title: pi-1-0-upgrade cell p1u-5 — pitfall candidate
description: "Pitfall candidate mined from cell p1u-5's capped trace: round 3 (bc6167b0c) adds GATED_TOOL_SET per amendment 82179c5d after the p1u-10 live run showed the planning leader could not reach bee_advisor — the plan was …"
timestamp: 2026-10-02
bee:
  id: pi-1-0-upgrade-p1u-5-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/p1u-5.json]
  polarity: pitfall
---

# pi-1-0-upgrade cell p1u-5 — pitfall candidate

## What the cell did

gated phases answer read, bash and bee_advisor (GATED_TOOL_SET) with the harness workflow on; READ_ONLY_TOOLS unchanged; leader set has no verdict; workers keep the full set

## Recorded evidence (verbatim from .bee/cells/p1u-5.json)

- **deviation** — round 3 (bc6167b0c) adds GATED_TOOL_SET per amendment 82179c5d after the p1u-10 live run showed the planning leader could not reach bee_advisor — the plan was wrong about a fact
- **deviation** — the herding worker does not cap, so the leader capped after re-running the verify — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell p1u-6 — save as docs/knowledge/patterns/pi-1-0-upgrade-p1u-6-pitfall.md

---
type: bee.pattern
title: pi-1-0-upgrade cell p1u-6 — pitfall candidate
description: "Pitfall candidate mined from cell p1u-6's capped trace: the belt also re-activates bee tools a later stage allows (planning to swarming), not only removes them — without it a leader that came from planning never got…"
timestamp: 2026-10-02
bee:
  id: pi-1-0-upgrade-p1u-6-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/p1u-6.json]
  polarity: pitfall
---

# pi-1-0-upgrade cell p1u-6 — pitfall candidate

## What the cell did

Pi belt passes the stage hook notices through verbatim and narrows only bee-known tools, re-adding them when a stage allows

## Recorded evidence (verbatim from .bee/cells/p1u-6.json)

- **deviation** — the belt also re-activates bee tools a later stage allows (planning to swarming), not only removes them — without it a leader that came from planning never got bee_dispatch back, since the old code only called setActiveTools when something was removed — something else had to be fixed first
- **deviation** — the bee-known set is a belt constant BEE_STAGE_TOOLS built from PI_BUILTIN_TOOLS plus five names; parity with stage_tools.rs FULL_TOOL_SET is proven by the new test, which feeds the real hook full list plus two foreign tools and expects only the foreign ones kept — the hook carries no known-tools field and stage_tools.rs is outside this cell — found a better route
- **deviation** — left for the orchestrator: the test comment in packages/bee-rs/crates/bee/src/hooks/stage_tools.rs (stage_tools_keeps_the_verdict_keys_it_publishes) still says the belt builds its own sentences and the message keys have no reader; that is now false, and the file is outside this cell files — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell p1u-7 — save as docs/knowledge/patterns/pi-1-0-upgrade-p1u-7-pitfall.md

---
type: bee.pattern
title: pi-1-0-upgrade cell p1u-7 — pitfall candidate
description: "Pitfall candidate mined from cell p1u-7's capped trace: leader-run instead of a worker — the worker-outward guard refuses a worker that starts pi — hit an unforeseen obstacle"
timestamp: 2026-10-02
bee:
  id: pi-1-0-upgrade-p1u-7-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/p1u-7.json]
  polarity: pitfall
---

# pi-1-0-upgrade cell p1u-7 — pitfall candidate

## What the cell did

Live Pi 1.0 proof: a deepseek-flash leader on a small-lane cell called bee_dispatch, the worker wrote and capped the cell, the drain delivered the result; leader wrote no source

## Recorded evidence (verbatim from .bee/cells/p1u-7.json)

- **deviation** — leader-run instead of a worker — the worker-outward guard refuses a worker that starts pi — hit an unforeseen obstacle
- **deviation** — the inline-write refusal is proven by the real hook probe in the sandbox, not by the model, which never attempted a write — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell p1u-8 — save as docs/knowledge/patterns/pi-1-0-upgrade-p1u-8-pitfall.md

---
type: bee.pattern
title: pi-1-0-upgrade cell p1u-8 — pitfall candidate
description: "Pitfall candidate mined from cell p1u-8's capped trace: round 4 (3b98c5e63) adds the run count and seat names per amendment 562588f5 after the live run — the plan was wrong about a fact"
timestamp: 2026-10-02
bee:
  id: pi-1-0-upgrade-p1u-8-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/p1u-8.json]
  polarity: pitfall
---

# pi-1-0-upgrade cell p1u-8 — pitfall candidate

## What the cell did

session-close answers obligations_only with cap and advisor obligations once per key; a cell whose registered worker has no result yet is not owed; the leader's own claim does not hide a cap

## Recorded evidence (verbatim from .bee/cells/p1u-8.json)

- **deviation** — round 4 (3b98c5e63) adds the run count and seat names per amendment 562588f5 after the live run — the plan was wrong about a fact
- **deviation** — round 1 (5f97402a1) followed the plan; round 2 (a75fe2ba1) added the running-worker filter per amendment 90077144; round 3 (100fb7652) dropped the claim-heartbeat half per correction 295f1277 — the plan was wrong about a fact
- **deviation** — the herding worker wrote a malformed result-1.json and does not cap, so the leader capped after re-running the verify — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell p1u-9 — save as docs/knowledge/patterns/pi-1-0-upgrade-p1u-9-pitfall.md

---
type: bee.pattern
title: pi-1-0-upgrade cell p1u-9 — pitfall candidate
description: "Pitfall candidate mined from cell p1u-9's capped trace: the herding worker does not cap, so the leader capped after re-running the verify — hit an unforeseen obstacle"
timestamp: 2026-10-02
bee:
  id: pi-1-0-upgrade-p1u-9-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/p1u-9.json]
  polarity: pitfall
---

# pi-1-0-upgrade cell p1u-9 — pitfall candidate

## What the cell did

agent_before_settle asks session-close for obligations, shows each user_notice, appends one bee-obligation custom_message each and returns continue true once; the agent_settled nudge is suppressed after a forced turn; /bee-obligation-skip sends skip_key

## Recorded evidence (verbatim from .bee/cells/p1u-9.json)

- **deviation** — the herding worker does not cap, so the leader capped after re-running the verify — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell p1u-10 — save as docs/knowledge/patterns/pi-1-0-upgrade-p1u-10-pitfall.md

---
type: bee.pattern
title: pi-1-0-upgrade cell p1u-10 — pitfall candidate
description: "Pitfall candidate mined from cell p1u-10's capped trace: leader-run instead of a worker — the worker-outward guard refuses a worker that starts pi — hit an unforeseen obstacle"
timestamp: 2026-10-02
bee:
  id: pi-1-0-upgrade-p1u-10-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/p1u-10.json]
  polarity: pitfall
---

# pi-1-0-upgrade cell p1u-10 — pitfall candidate

## What the cell did

Live Pi 1.0 proof: one forced turn for a claimed uncapped cell (leader then dispatched and capped) and one for a gate-ready high-risk plan (leader called bee_advisor x5, did not approve the gate); no repeat on a second settle

## Recorded evidence (verbatim from .bee/cells/p1u-10.json)

- **deviation** — leader-run instead of a worker — the worker-outward guard refuses a worker that starts pi — hit an unforeseen obstacle
- **deviation** — the first advisor run found two gaps (bee_advisor not in the planning set; no run count) and p1u-5 and p1u-8 were reworked before the final run — the plan was wrong about a fact
- **deviation** — the leader in the final advisor run stopped before recording the advisor_ref; filed as a backlog finding — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell p1u-11 — save as docs/knowledge/patterns/pi-1-0-upgrade-p1u-11-pitfall.md

---
type: bee.pattern
title: pi-1-0-upgrade cell p1u-11 — pitfall candidate
description: "Pitfall candidate mined from cell p1u-11's capped trace: the herding worker does not cap, so the leader capped after re-running the verify and the registry contract tests — hit an unforeseen obstacle"
timestamp: 2026-10-02
bee:
  id: pi-1-0-upgrade-p1u-11-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/p1u-11.json]
  polarity: pitfall
---

# pi-1-0-upgrade cell p1u-11 — pitfall candidate

## What the cell did

bee herding steer <job-id> --text writes steer-<n>.json through temp+rename in the job mailbox, refuses a missing job, a finished round or a capped cell, and prints the job and steer number

## Recorded evidence (verbatim from .bee/cells/p1u-11.json)

- **deviation** — the herding worker does not cap, so the leader capped after re-running the verify and the registry contract tests — hit an unforeseen obstacle
- **deviation** — sync-ack: bee herding steer is reached on Pi through the bee_steer leader tool; the skills/bee-herding verb list update is filed as a backlog debt row

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell p1u-12 — save as docs/knowledge/patterns/pi-1-0-upgrade-p1u-12-pitfall.md

---
type: bee.pattern
title: pi-1-0-upgrade cell p1u-12 — pitfall candidate
description: "Pitfall candidate mined from cell p1u-12's capped trace: followed the plan"
timestamp: 2026-10-02
bee:
  id: pi-1-0-upgrade-p1u-12-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/p1u-12.json]
  polarity: pitfall
---

# pi-1-0-upgrade cell p1u-12 — pitfall candidate

## What the cell did

Record typed scope changes in runtime file and owe one settle turn per change

## Recorded evidence (verbatim from .bee/cells/p1u-12.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: cell p1u-12 declared no skill changes

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell p1u-13 — save as docs/knowledge/patterns/pi-1-0-upgrade-p1u-13-pitfall.md

---
type: bee.pattern
title: pi-1-0-upgrade cell p1u-13 — pitfall candidate
description: "Pitfall candidate mined from cell p1u-13's capped trace: the herding worker does not cap, so the leader capped after re-running the verify — hit an unforeseen obstacle"
timestamp: 2026-10-02
bee:
  id: pi-1-0-upgrade-p1u-13-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/p1u-13.json]
  polarity: pitfall
---

# pi-1-0-upgrade cell p1u-13 — pitfall candidate

## What the cell did

belt: async input handler records steer/follow-up text as scope input and passes it unchanged; worker steer drain from the main-checkout mailbox with the fixed prefix, 8 KB and shape limits; verdict refuses a missing job dir when BEE_HERDING_JOB_ID is set; undelivered steers named in the result header; bee_steer leader tool in the leader set

## Recorded evidence (verbatim from .bee/cells/p1u-13.json)

- **deviation** — the herding worker does not cap, so the leader capped after re-running the verify — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell p1u-14 — save as docs/knowledge/patterns/pi-1-0-upgrade-p1u-14-pitfall.md

---
type: bee.pattern
title: pi-1-0-upgrade cell p1u-14 — pitfall candidate
description: "Pitfall candidate mined from cell p1u-14's capped trace: leader-run instead of a worker — the worker-outward guard refuses a worker that starts pi — hit an unforeseen obstacle"
timestamp: 2026-10-02
bee:
  id: pi-1-0-upgrade-p1u-14-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/p1u-14.json]
  polarity: pitfall
---

# pi-1-0-upgrade cell p1u-14 — pitfall candidate

## What the cell did

Live Pi 1.0 proof: a steer reached a running worktree worker once (it declined a scope-changing steer by design), the worktree worker recorded its verdict, a late steer was refused, and a typed scope change got one forced turn while bee refused an unapproved cell add

## Recorded evidence (verbatim from .bee/cells/p1u-14.json)

- **deviation** — leader-run instead of a worker — the worker-outward guard refuses a worker that starts pi — hit an unforeseen obstacle
- **deviation** — the steer test text asked for a scope change, which the worker correctly declined; delivery is proven by the worker report quoting it — found a better route
- **deviation** — in the scope run the small model answered the forced turn with an empty message; filed as a backlog finding — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 14 capped cell(s) mined, 1 delivery draft, 5 area bullet(s), 14 pattern candidate(s), 0 file(s) written.