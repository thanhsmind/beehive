# guard-hardening — plan

Route: class=feature lane=high-risk flags=[audit-security,proof-weakening,
multi-domain] files=6. CONTEXT.md E1–E6 govern.

## Shape

One slice, three cells. gh-1 and gh-2 touch disjoint Rust regions but
the same test files may grow in both — gh-2 runs after gh-1 (named
reason: shared test-module edit surface, cheap to serialize, high-risk
lane favors clean bisection). gh-3 is config+docs, parallel-safe with
gh-2, runs last anyway as the integration cap.

- **gh-1 (E1)** — Containment allowlist in write-guard: after canonical
  resolution, a write target under `<home>/.claude/projects/` or
  `<system-temp>/claude/` is exempt from the outside-root containment
  deny (both the Edit/Write path and extracted Bash targets). Resolution
  first, allowlist second — a symlink or `..` path that escapes INTO
  the allowlist roots from a repo-relative spelling is still judged by
  its resolved location; a resolved location outside both roots and
  outside the worktree stays denied. Tests: memory-dir write allowed,
  scratchpad write allowed, sibling-worktree write still denied,
  traversal spelling of an allowed root allowed (resolved), unrelated
  out-of-root path still denied. Home and temp resolve from the same
  sources the existing code uses (no new env contract without need).
- **gh-2 (E2)** — Extend `direct_edit_verb`'s CLI-owned set with
  `.bee/cells/*.json`, `.bee/lanes/*.json`, `.bee/onboarding.json`
  (glob or prefix+extension match consistent with the existing five
  entries' style). `.bee/config.json` and `.bee/decisions.jsonl` stay
  writable — add a regression test asserting BOTH the new denies and
  the two preserved allowances.
- **gh-3 (E3+E5)** — `.claude/settings.json`: add
  `"permissions": {"defaultMode": "bypassPermissions", "deny":
  ["Bash(grep:*)", "Bash(find:*)"]}` preserving the existing object.
  Sync knowledge: unenforced-obedience.md (or the fitting doctrine doc)
  gains the E5 record — which rules stay markdown-only and why; the
  prompt-writing-standard's deterministic-backstop section gains a
  pointer to the now-real deny example. Learnings file for the feature.

## Constraints (bind every cell)

- **Fail-closed** — every new branch defaults to deny on resolution
  failure; the allowlist never bypasses reservation/hold checks or the
  gate boundary, only the containment deny (E1 scope).
- **No behavior change outside the three named deltas** — existing
  write-guard tests must pass unmodified; a test that must change is a
  STOP-and-report, not an edit.
- **Meaning-preserving docs** — knowledge sync cites CONTEXT.md E-ids.
- **Proof** — `commands.test` full suite green at every cap (E6);
  gh-1/gh-2 also run the write-guard-focused tests explicitly and quote
  them.

## Verify

`commands.test`: `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test
--release --manifest-path packages/bee-rs/Cargo.toml`.

## Edge dimensions considered (high-risk triad+)

Path canonicalization (symlink, `..`, UNC/drive-letter case on win32),
worktree-vs-main resolution, missing HOME/TEMP env (fail-closed),
concurrent-session holds unaffected by the exemption, settings.json
merge round-trip under `bee onboard --apply`.

## Smaller-path check

Cheaper shapes considered: settings-only fix (cannot reach containment
— that deny lives in the hook binary, so Rust is unavoidable for E1);
dropping containment per the user's literal ask (rejected by E1 — it
removes a live safety boundary; the allowlist meets the actual need).
Three cells is the smallest shape honoring E1–E6. PASS.
