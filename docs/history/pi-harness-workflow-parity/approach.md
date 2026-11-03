# Approach: Pi harness workflow parity

## Recommended design

Keep lifecycle authority in the Rust CLI. Runtime adapters translate events and session ids into that contract.

Use six changes:

1. Add one session identity helper for `BEE_SESSION_ID`, `CLAUDE_CODE_SESSION_ID`, and `PI_SESSION_ID`.
2. Pass the active lane feature and the caller purpose into every non-cell dispatch prompt.
3. Parse exact cell packets from `plan.md` before shape approval. Record the preview hash and compare persisted cells with it.
4. Parse each cap proof once. Compare its command with the cell `verify` value and fill the structured trace fields.
5. Apply the main-checkout worktree refusal during all active code phases, not only `swarming`.
6. Add process and counter components to generated herding job ids.

Decision `d59befe3-73c6-415a-91f1-f43b485a1eeb` requires shared CLI and runtime enforcement. Decision `9daa8378-6c01-4635-a61e-7409338c28da` requires collision-safe allocation.

## Rejected alternatives

A Pi-only patch leaves CLI commands inconsistent. A skill-only gate preview can be skipped again. Free-text proof cannot support replay. Caller sleeps keep job-id correctness outside the allocator.

## Trust boundaries

The CLI treats runtime environment values, stored workflow state, plan bytes, worker reports, and shell targets as inputs. Each check validates the exact value that the next operation uses.

## Risks

| Component | Risk | Control | Proof |
|---|---|---|---|
| Session identity | High | One ordered helper and precedence tests | Pi-only and mixed-environment tests |
| Intent and dispatch | High | Feature-keyed lookup and prompt assertion | Cross-feature negative test and purpose byte test |
| Gate packet | High | Parsed JSON packet, preview hash, and add-time equality | Missing, stale, malformed, and matching packet tests |
| Proof trace | High | Exact command equality and one parser | Mismatch refusal and structured-field tests |
| Write guard | High | Active-phase main refusal with existing exemptions | Claude-shaped and Pi-shaped hook tests |
| Herding ids | Medium | Milliseconds, process id, and atomic counter | Concurrent uniqueness test |

## Rollback

Each cell is one commit. Revert the failing cell without changing prior cell behavior. Keep historical cap reads compatible throughout the change.
