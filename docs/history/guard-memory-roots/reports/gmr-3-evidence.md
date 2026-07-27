# gmr-3 — evidence

Cell **gmr-3** (`guard-memory-roots`, lane small, GH #71): a declared
`guards.memory_root`, gated on a live `.bee-write-root` marker, lets the agent
write its own memory. Worker `exec-gmr3`.

**No new tests were written and no existing test was edited** (user's explicit
instruction). Verification is (a) a live probe matrix against the vendored hook
and (b) the whole existing suite staying green unmodified.

## What changed

- `packages/bee/hooks/bee-write-guard.mjs` — new `resolveDeclaredMemoryRoot` /
  `isDeclaredMemoryRootTarget` / `expandConfigHomePrefix` /
  `rootTouchesRepoControlDir` / `isBroadTargetSpelling`, plus wiring in the
  Bash branch and the Write/Edit/MultiEdit branch. The `apply_patch` branch is
  untouched (D11).
- `.bee/bin/hooks/bee-write-guard.mjs` — vendored copy.
- `docs/history/codex-harness-hardening/release-manifest.json` — regenerated.
- `docs/config-reference.md` — the two operator steps and what the grant means.

Containment reuses the existing `realpathOrNull` / `resolveTargetRealpath` /
`isUnderRoot` helpers — the same discipline the worktree wall itself uses. The
whole check is wrapped in its own try/catch returning "no match", so a throw
can never reach the hook's outer catch (which returns exit 0 and would fail the
ENTIRE hook open).

## Probe matrix — 30/30

Driven against the **vendored** hook `.bee/bin/hooks/bee-write-guard.mjs` with
real JSON payloads on stdin, exit codes captured. `2` = deny, `0` = allow.
Every allow row below was taken at **phase `idle`** — the intake gate shut —
which is precisely the moment D6 exists for.

The fixture is an isolated onboarded non-git directory (adapter `resolveRoots`
first branch ⇒ `root === storeRoot === fixture`), with its own
`config.local.json` and `state.json` and a symlinked `.bee/bin`. This repo's
own config and state were never touched, and no declared root was left behind.

| # | Case | Expect | Exit |
|---|---|---|---|
| 1 | no root declared → Write into memory dir | deny | 2 |
| 2 | no root declared → Bash redirect into memory dir | deny | 2 |
| 3 | declared + marker → Write into root | allow | 0 |
| 4 | declared + marker → Edit into root | allow | 0 |
| 5 | declared + marker → Bash redirect into root | allow | 0 |
| 6 | declared + marker → Bash `mkdir -p` nested in root | allow | 0 |
| 7 | declared as `~/…` (config-value expansion) → Write | allow | 0 |
| 8 | declared, NO marker → Write | deny | 2 |
| 9 | declared, NO marker → Bash redirect | deny | 2 |
| 10 | traversal out of root → Write | deny | 2 |
| 11 | traversal out of root → Bash redirect | deny | 2 |
| 12 | symlink inside root resolving outside → Write | deny | 2 |
| 13 | refused shape: root contains the worktree | deny | 2 |
| 14 | refused shape: bare home directory | deny | 2 |
| 15 | refused shape: root contains a `.git` dir | deny | 2 |
| 16 | refused shape: root has a `.bee` path segment | deny | 2 |
| 17 | refused shape: filesystem root | deny | 2 |
| 18 | refused shape: root is not an existing directory | deny | 2 |
| 19 | malformed config: number | deny | 2 |
| 20 | malformed config: relative path | deny | 2 |
| 21 | malformed config: empty string | deny | 2 |
| 22 | tilde-spelled target naming the declared root → Bash | deny | 2 |
| 23 | `$HOME`-spelled target naming the declared root → Bash | deny | 2 |
| 24 | ordinary in-repo `docs/` path at idle | allow | 0 |
| 25 | ordinary in-repo SOURCE path at idle → still gate-denied | deny | 2 |
| 26 | in-repo source path, no root declared → still gate-denied | deny | 2 |
| 27 | mixed Bash: memory root + in-repo source → still denied | deny | 2 |
| 28 | broad spelling inside root (`rm -rf <root>/*`) | allow | 0 |
| 29 | memory write + pathless broad (`git add --all`) → still checked | deny | 2 |
| 30 | `apply_patch` naming a memory-root path → still denied (D11) | deny | 2 |

Rows 13-16 each had a **valid marker present**, so only the shape refusal can
be producing the denial. Row 17 (`/`) could not be given a marker, so it is
denied by both the filesystem-root refusal (which runs first in code) and the
missing marker. Rows 25-27 prove the short-circuit did not disable the intake
gate for ordinary paths, and row 29 proves a blanket `git add --all` riding
along with a memory write still reaches `checkWrite` as `**`.

## Verify

```
node scripts/ledger_parity.mjs --check && node scripts/release_manifest.mjs --check && node scripts/run_verify.mjs
```

`PASS run_verify: 108 suite(s), concurrency=5, wall=78249ms` — every existing
suite unedited, including the blanket out-of-worktree denial invariants
(`escapeRows` in `test_write_guard.mjs`) that D2 requires to keep passing
unchanged.

Vendor parity (`md5sum`), after
`render_plugin_skill_trees.mjs` → `onboard_bee.mjs --apply` →
`release_manifest.mjs --write`:

```
a92610a9aba277caea1e5f7d02b70ac3  .bee/bin/hooks/bee-write-guard.mjs
a92610a9aba277caea1e5f7d02b70ac3  packages/bee/hooks/bee-write-guard.mjs
```

## Deviations

1. **Probe roots live in a system temp dir, not `.bee/tmp/gmr-3/`.** They
   cannot live there: a path with a `.bee` segment is itself a refused root
   shape (row 16), so a probe root under `.bee/` would be refused and prove
   nothing. The disposable harness itself was written to `.bee/tmp/gmr-3/` as
   instructed and deleted after the run.
2. **`.git`/`.bee` refusal is read strictly**: refused when any path SEGMENT of
   the realpathed root is `.git`/`.bee` (not just the basename), plus a
   bounded depth-1 child check for a `.git`/`.bee` directory inside it. A
   deeper recursive scan on every hook invocation would be real cost for a
   sanity refusal, and a human still has to place the marker at the root
   itself. Erring toward refusal is the fail-closed direction (D4).
3. **The broad-write fallthrough is narrowed precisely, not bluntly.** A
   command whose extracted targets are all memory-root hits and whose broad
   signal comes from a memory-root target's own spelling (`rm -rf <root>/*`)
   does not fall through to `relPaths = ["**"]` (row 28). When `broadWrite`
   instead came from a PATHLESS trigger (`git add --all`, `git commit -a`, a
   bare `rm`), `"**"` still applies and the blanket write is still fully
   checked (row 29).
