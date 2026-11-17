# Issue Sweep Nov 17 — Context

Asked: "xem github issure lấy về fix xong thì mark resolve" (2026-11-17):
take the open GitHub issues, fix them, then close them.

## Feature Boundary

In scope: the eleven open bee-report issues still live in current code —
#158, #159, #160, #161, #162, #163, #164, #165, #167, #168, #170, #171.
Already fixed in v2.74.0 and closed with evidence before this feature:
#154, #155, #156, #157; #169 closed on a green main CI run. Out of scope:
#69, the cross-project dashboard, parked as PBI p-fb5b5d00.

## Locked Decisions

- **D1 (719d1a38), issues #163 and #168.** git staging and commit of the onboarding-managed skill projection trees passes the idle intake gate and the worktree-first guard; Write and Edit into them stay gated.
- **D2 (681f1ed6), issue #165.** rm, mv, cp, mkdir, touch and tee are write commands only at the head of a shell segment.
- **D3 (6d88458e), issue #170.** A read-only herded worker may run date, echo, stat and pwd.
- **D4 (3ee2db95), issues #161 and #167.** A cap releases the cell's path leases and holds; the reservations list label reads the lease expiry.
- **D5 (8ed6f4ae), issue #160.** A verify that contains the proof separator can be capped.
- **D6 (e092dbb6), issue #171.** cells add and update refuse depends_on and dependencies and name deps.
- **D7 (c9826c53), issue #162.** A named reattribute dry run writes nothing.
- **D8 (93c9c272), issue #158.** The shared nested checkout scan skips .bee/tmp.
- **D9 (882c75e1), issue #159.** A herding job with a terminal result frees its concurrency slot.
- **D10 (2bcf3437), issue #164.** dispatch prepare reports an undeclared herding agent as not ready.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/patterns/20260818-a-green-count-is-not-evidence-that-your-new-test-ran.md | confirmed | testers quote filter counts and test names |
| docs/knowledge/areas/rust-runtime/prompt-files-and-learned-context.md | suspect | confirms #156 was fixed by the onboard binary copy |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs` | intake exemptions and shell write targets |
| `packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs` | worktree-first exemptions |
| `packages/bee-rs/crates/bee/src/hooks/worker_guard.rs` | read-only worker allowlist |
| `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs` | cap, finish and unclaim releases |
| `packages/bee-rs/crates/bee/src/verbs/reservations/emit.rs` | reservations list label |
| `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs` | proof parsing |
| `packages/bee-rs/crates/bee/src/verbs/cells/validate.rs` | new-cell normalization |
| `packages/bee-rs/crates/bee/src/verbs/decisions/verbs_write.rs` | reattribute |
| `packages/bee-rs/crates/bee/src/nested_checkout.rs` | nested checkout scan |
| `packages/bee-rs/crates/bee/src/herding/wave.rs` | concurrency live count |
| `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` | transport readiness |

## Outstanding Questions

None.
