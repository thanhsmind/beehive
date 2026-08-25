# doctor-probe-honesty — locked context

## The defect

`packages/bee-rs/crates/bee/src/doctor.rs`, the `binary_freshness_row` check.
It matches on `installed_binary_bee_version(&bin)`:

```rust
match installed_binary_bee_version(&bin) {
    ProbedBeeVersion::Missing => { /* ok: Some(false) */ }
    ProbedBeeVersion::Present(installed_version) => {
        if installed_version != source_version { /* ok: Some(false) */ }
    }
    ProbedBeeVersion::Failed => {}
}
```

`Failed` is an empty arm. When the probe cannot execute the binary at all, the
version comparison is skipped and control falls through to the mtime scan. If
no source input is newer, the function returns:

```rust
ok: Some(true),
detail: "installed binary matches source (version {source_version}), no source input newer than the binary"
```

That sentence is false. No installed version was ever read, so "matches source"
is a claim the code cannot support. A doctor run that could not probe the binary
reports the binary as fresh.

`Missing` and `Present`-but-different both correctly report `Some(false)`. Only
`Failed` degrades into a pass.

## How it surfaced

`doctor::tests::binary_freshness_catches_release_drift_when_package_versions_agree`
failed once inside a loaded full-suite run — `left: Some(true)`, `right:
Some(false)`, against its own message *"tautological package-version agreement
must not pass when release versions differ"*. It then passed on a rerun, passed
alone, passed on a second full run, and passes on main.

The test is hermetic — it builds its own tree under a `tempdir`. So the flake is
not the test being wrong: it is the probe transiently failing to exec the
just-written fake binary under parallel load (ETXTBSY is the usual cause). The
test was reporting the product defect truthfully. Treating it as noise and
re-running until green would have buried a real bug.

This matters concretely: a stale vendored `.bee/bin/bee` is exactly the condition
this repository has been sitting in, and it is the condition this row exists to
report.

## Locked decisions

1. **A probe that failed is never reported as fresh.** The `Failed` arm stops
   falling through into a row that asserts the binary matches source.
2. **A failed probe is `ok: None`, not `ok: Some(false)`.** Not being able to
   read the version is not evidence of staleness. `None` matches the convention
   this same function already uses for "cannot determine": an unreadable
   `.claude-plugin/plugin.json` and an absent installed binary both return
   `None`.
3. **The mtime scan still runs after a failed probe, and still wins when it
   finds something.** A source input newer than the binary is independent, real
   evidence of drift, and it stays `ok: Some(false)` with its existing detail —
   a more actionable answer than "unknown". Only the *fall-through* case
   changes.
4. **The resulting matrix, and nothing beyond it:**

   | probe | mtime scan | row |
   |---|---|---|
   | `Present`, versions equal | nothing newer | `Some(true)` — unchanged |
   | `Present`, versions differ | either | `Some(false)` — unchanged |
   | `Missing` | either | `Some(false)` — unchanged |
   | `Failed` | something newer | `Some(false)` — unchanged detail |
   | `Failed` | nothing newer | **`None`, naming that the probe failed** — the only new behavior |

5. **The detail line for the new case names the cause and the remedy** in the
   same voice as its neighbours, and never states a version it did not read.

## Hard constraints

- Only the `Failed`-and-nothing-newer cell of that matrix changes. Every other
  row keeps its current `ok` value and its current detail text.
- No change to `installed_binary_bee_version`, to `source_inputs`, or to the
  mtime comparison itself.
- The existing doctor tests stay green unedited; the new behavior earns its own
  test rather than being asserted by loosening an existing one.

## Acceptance

- A tree whose binary cannot be probed, with no source input newer than it,
  produces a row with `ok: None` whose detail names the failed probe — and does
  not contain the words "matches source".
- The same tree with a source input newer than the binary still produces
  `ok: Some(false)` with the existing newer-input detail.
- `Present`-equal, `Present`-differing, and `Missing` all keep their current
  rows, proven by the existing tests passing unedited.

## Out of scope

- Making the probe itself more robust (retry, backoff). The row must be honest
  about a failed probe regardless of how rare the failure is.
- Anything about how the vendored binary gets refreshed; that is
  `scripts/release.sh`'s job.
