# Approach: Worktree Concurrency Guard

## Recommended path

Add one shared structural-detection primitive, then wire it into the two surfaces named in D1: `bee-write-guard.mjs` (blocks an unverified write) and `bee.mjs handleWorktreeNew` (refuses at creation). The primitive answers one question: "does this target lie inside a nested/companion-shaped checkout that another concurrently-live session could also reach?" — combining `isConcurrentMode()` (`claims.mjs:283`, already built, zero callers in either surface today) with a new structural check, since neither surface consults concurrency at all right now (confirmed: zero hits grepping `isConcurrentMode` in `worktree-store.mjs` and `bee.mjs`).

Build in this order: (1) the shared detection primitive with tests proving its boundary cases, (2) write-guard wiring, (3) worktree-new wiring, (4) end-to-end tests extending the existing companion/write-guard suites. Steps 2 and 3 touch different files and can run as a parallel wave once step 1 lands; step 4 needs both.

## Rejected alternatives

- **A brand-new mid-session "declare this mount as trusted" CLI verb**, so a session already blocked by the write-guard could self-certify in place — rejected per D4: reopens exactly the self-discipline surface this feature exists to close, and adds an unhardened trust boundary for no benefit over the existing `bee worktree new --with-companion` paved road.
- **An override/`--force` escape hatch** on either surface — rejected per D3: no existing hook-level guard in this codebase (SESSION_REQUIRED, CLAIMED, reservation conflict, intake gate) offers one; each just gives an actionable FIX instead.
- **Gating the check on `gate_bypass`** so a `total`-bypass session skips it — rejected per D5: hook-level guards are architecturally outside the four Gates' bypass scope today, and staying consistent keeps this safety net meaningful even under bypass.

## Risk map

| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| Detection signal's exact trigger condition | **HIGH** | Two candidate shapes both matter and are NOT the same check: (a) a target that resolves outside the checkout's own realpath tree via symlink (the companion-mount shape, already partly recognized by `resolveCompanionMountedRelPath`, `bee-write-guard.mjs:384-414`) — today's containment check likely already blocks an *unrecognized* symlink escape regardless of concurrency, so the live gap there is specifically a **verified** companion mount getting zero concurrency check; (b) a target inside a **plain, non-symlinked nested `.git` boundary sitting inside a checkout multiple sessions can concurrently reach** (e.g., the MAIN checkout, or literally the same directory two sessions both `cd`'d into — STR65's actual incident shape) — this is NOT blocked by any existing check today, since it's just an ordinary subpath under root and bee's reservation/hold system has zero visibility into a nested `.git`'s own index either way. The backlog's own CoS wording ("...or a plain/unidentified session where isConcurrentMode() is true...") points at needing to cover (b), not just (a). | Validating-stage proof: write concrete fixtures for both shapes (symlinked companion mount, plain nested `.git` in a shared checkout, and a plain git submodule as the negative control) and confirm which existing checks already fire today, before finalizing the new check's exact trigger condition. This is the single most important thing bee-validating must nail down before any code lands — get it wrong and the guard either misses STR65's real shape or blocks legitimate submodule use. |
| Where to add the shared helper | LOW | `guards.mjs` already centralizes write-checking logic shared by the hook; `bee.mjs` already imports sibling libs for other checks. | Confirm during Epic 1 which file keeps the helper closest to its callers — a five-minute repo-reality check, not a design question. |
| Refusal message wording (paved-road framing) | LOW | Precedent already settled: `docs/history/learnings/20260717-guard-membership-escape-routes.md` — "a deny that only says no converts a guard into a lockout"; every existing typed refusal in this codebase already follows this. | None beyond following the existing pattern; cite it in the cell's `action`. |
| Backward compatibility (D6) | LOW | The check is additive and only fires when both `isConcurrentMode()` and the structural signal are true — a host with no nested/companion repo anywhere never trips the structural half. | A negative-control test proving a no-companion host sees zero new refusals. |

## Files and order

1. Shared detection helper + its own unit tests (new or extended `guards.mjs` export; exact home decided against the risk-map's HIGH item above).
2. `.bee/bin/hooks/bee-write-guard.mjs` — wire the helper + `isConcurrentMode()` into the existing dispatch around lines 791/804/818.
3. `.bee/bin/bee.mjs` `handleWorktreeNew` (~3845-3899) — pre-creation refusal reusing the same helper against the source checkout.
4. `scripts/test_worktree_companion.mjs` and `hooks/test_write_guard.mjs` — extend with the concurrency dimension; add the negative control for D6.

## Relevant learnings

- `docs/history/learnings/20260717-guard-membership-escape-routes.md` — every new guard deny must teach the legitimate escape route, or it becomes a lockout. Directly informs D3/D4's refusal wording.
- `docs/knowledge/areas/workflow-state/worktree-isolation.md` (R32-R35) — worktree isolation removes git-index contention only; reservations remain the ownership primitive; canonical physical containment always precedes authorization. Confirms this feature is additive to, not a replacement for, existing containment logic.
- `docs/knowledge/areas/worktree-parallelism/entering-creating-and-registering.md` — `worktree new` is the paved road (D7, GH #21); every refusal in that path is already typed and zero-mutation, the pattern this feature's worktree-new-time refusal should match.

## Validating findings (resolved)

- D2 was widened (superseded, decision `0ccc1cf3`) to also cover a plain nested repo physically inside the checkout's own tree, not just the symlink-escape shape — see CONTEXT.md D2 for the full text and rationale. User-approved after the configured advisor flagged the original narrower D2 text didn't cover STR65's actual incident shape.
- The advisor's independent review flagged two mechanical notes for whoever implements Epic 2's submodule-registration check: `git submodule add` produces a `.git` FILE (absorbed gitdir) at the submodule root, not a directory — the exclusion check must handle both shapes, not just directory-`.git`. A nested git *worktree* (a `.git` file whose gitdir resolves outside the tree, no symlink involved) is a related edge case not covered by the current 3-fixture spike — cheap to add when Epic 2 is planned.

## Questions for validating

- Does today's containment check (`canonicalRelPath` / `describeCrossWorktreeTarget`) already block an *unrecognized* symlink escape regardless of concurrency — meaning the write-guard's real gap is narrower (verified companion mounts only) than the backlog text's "plain/unidentified session" wording implies? Concrete fixture test needed, not inference.
- For the plain-nested-`.git`-in-a-shared-checkout shape: should the check fire whenever `isConcurrentMode()` is true and root is the ordinary (non-worktree) main checkout, or does it need a companion marker/config signal too — and if the latter, does that leave STR65's exact incident shape (no companion setup at all, just a nested `repo/` in a shared main checkout) uncovered?
- Which existing test file should host the new concurrency-dimension tests (extend `hooks/test_write_guard.mjs`'s existing companion-mount rows, vs. a new file) — a repo-convention question, not a product one.
