# wcg-2 — wire the shared-nested-checkout guard into bee-write-guard.mjs

**Status:** [DONE]

**Outcome:** Wired `guards.isSharedNestedCheckoutTarget` into `bee-write-guard.mjs`'s Edit/Write and Bash dispatch (both hook copies) as a hard fail-closed refusal that runs BEFORE `checkWrite` (D1b/D3/D5). A live write whose target resolves inside a genuinely shared nested checkout another session can also reach — and that no verified companion marker covers — is now denied while another session is live, with a typed refusal directing to a FRESH `bee worktree new --with-companion` (D4, never an in-place conversion). Verified companion mounts reached through their sanctioned symlink stay allowed, and a solo session is a pure no-op (the acting session is excluded from the concurrency check, D6). The check never consults `gate_bypass`. Verify green.

**Files changed:**
- `hooks/bee-write-guard.mjs` + `.bee/bin/hooks/bee-write-guard.mjs` (byte-identical copies) — two helpers (`lexicalAbsTarget`, `sharedNestedCheckoutRefusal`) and the pre-`checkWrite` guard loop; the Edit/Write and Bash branches now track which targets are physically-contained (canonicalRelPath-resolved) so only those — never companion-marker-resolved targets — are candidates.
- `hooks/test_write_guard.mjs` — 5 new rows (78-82): concurrent plain-nested Edit denied, refusal-message wording, solo/own-session allowed, concurrent Bash denied, verified-companion-mount stays allowed.
- `.bee/onboarding.json` — `repo_hooks` managed-hash regen for the changed hook (onboard `--apply --repo-hooks`).
- `docs/history/codex-harness-hardening/release-manifest.json` — the 2 touched-file content-hash entries only (`hooks/bee-write-guard.mjs`, `hooks/test_write_guard.mjs`); mode-drift noise and 2 unrelated pre-existing `.bee-render.json` entries excluded per the cell's `regen_obligation_ack`.

**Verify:** `node --test hooks/test_write_guard.mjs && node scripts/ledger_parity.mjs --check` → exit 0 (tests 1 / pass 1 / fail 0, ALL PASS across 82 rows; ledger matches). Red-first proven: before wiring, rows 78/81 returned status 0 (the plain-nested concurrent write succeeded — STR65's exact unguarded shape) and row 79 found no refusal message ("4 FAILURE(S)"); rows 80/82 already passed, scoping the change to the concurrent+unverified-shared case.

**Design note:** the write-guard exemption for a verified companion mount (D1b "no verified companion marker covers it") is enforced by the hook wiring, not the primitive — only `canonicalRelPath`-resolved (physically-in-root) targets are fed to `isSharedNestedCheckoutTarget`, and a companion mount always resolves via `resolveCompanionMountedRelPath` instead, so shape (a) never fires in the hook. `apply_patch` targets were left unguarded per the cell's explicit scope (Bash line 791, Edit/Write line 804 only) — a same-threat gap worth a follow-up if apply_patch write coverage is desired.

Full trace, deviations, and behavior-change evidence: `.bee/cells/wcg-2.json`.
