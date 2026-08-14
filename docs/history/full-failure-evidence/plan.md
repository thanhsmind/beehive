---
artifact_contract: bee-plan/v1
mode: standard
# approved_gate2: <unset until approval>
---

# Plan: Full Failure Evidence

Mode: `standard` — 3 risk flags: public-contracts, covered-contract-change,
multi-domain. No hard-gate flag.

Why this is the least workflow that protects the work: the first draft of this
plan was wrong in a way only a read of every consumer could show, and the
correction changes where the evidence goes. What earns a plan is that the
obvious place to put the log path is a string that gets hashed.

## Requirements (from CONTEXT.md)

- **D1** — the complete output of a failing declared command is written under
  `.bee/logs/`, and the excerpt names that path.
- **D2** — the excerpt stays bounded; the 500-character limit is not raised.
- **D3** — a green run writes no failure log and leaves no stale one behind.
- **D4** — the excerpt keeps its tail-of-output shape; no framework-specific
  parsing.

D1's "the excerpt names that path" is satisfied by the *refusal text a human
reads*, not by the `failure_excerpt` string itself — see Discovery 2. That is a
reading, not a change: D1 says the excerpt names the path, and the excerpt a
person sees is the rendered refusal.

## Discovery

1. **Three copies of the failure-recording logic, no fourth.** Verified:
   `finish_support.rs:119`, `close.rs:144`, `test_runner.rs:285`, each with its
   own 500 constant (`:29`, `:33`, `:66`). All three import
   `crate::textutil::truncate_chars_tail` (`finish_support.rs:13`,
   `close.rs:11`, `test_runner.rs:54`). No Node original survives.

2. **`failure_excerpt` is hashed, not merely displayed.** This refutes the
   first draft. `handlers_close.rs:218` passes it to
   `normalize_failure_signature`, which `trace.rs:409-427` reduces to
   `sha256[..12]` and stores on the cell trace. Appending anything to the
   string changes that signature, so two runs of the same failure would stop
   matching. There are seven readers in all, not two: `close.rs:237`,
   `finish_support.rs:169`, `handlers_close.rs:218`, `handlers_close.rs:229-241`,
   `handlers_write.rs:698` (a JSON consumer gating the red-base claim),
   `test_runner.rs:367-378` (a third `first_failure_line` copy), and
   `close.rs:1066-1072`.

3. **The record's key order is a weaker constraint than the string's
   content.** "Frozen key order" (`close.rs:190`) is a doc comment plus one
   test asserting the key vector (`test_runner.rs:478-481`);
   `handlers_write.rs:698` reads by name and is order-agnostic. Appending a key
   costs one test line. Changing the excerpt string costs a stored hash.

4. **`textutil` is text-only by locked decision.** `textutil.rs:34` imports
   only `std::cmp::Ordering` and performs no IO; its charter
   (`textutil.rs:1-15`) and `docs/history/js-parity-cleanup/CONTEXT.md:67-74`
   (D3) scope it to length, truncation and order primitives. A writer does not
   belong there. `crate::fsutil` already owns atomic text and JSON writes
   (`fsutil.rs:139-159`, which create parent directories).

5. **`.bee/logs/` is gitignored** (`.gitignore:5`), so a large log costs the
   repository nothing.

6. **The three runners are not serialized against each other.**
   `rg -n "acquire_named_lock|lock::"` finds nothing in `test_runner.rs` or
   `close.rs`; `handlers_close.rs:202` takes the cell lock only after the run.
   `finish_support.rs:362-367` makes `test_root == main root` for an ordinary
   checkout, so `bee test` and `bee cells finish` can run concurrently on one
   root.

Evidence commands: `rg -n "failure_excerpt" packages/bee-rs/crates/bee/src`,
`rg -n "FAILURE_EXCERPT_MAX" packages/bee-rs/crates/bee/src`.

## Approach

**The log path gets its own record key; the hashed string is left alone.**
`failure_excerpt` keeps its exact current value and bound, so the failure
signature is unchanged and every one of the seven readers keeps working. A new
`failure_log` key carries the path, appended last in the record so the frozen
key order grows rather than shifts. The refusal text a human reads gains a line
naming the log — that is the display layer, and it is the only place the path
appears to a person.

**Two cells, in this order**, because the review found one cell carrying a
helper, three rewrites, retention, and eight test sites:

- **`ffe-1` — consolidate, change nothing.** Extract the three identical
  excerpt blocks into one `pub(crate)` helper in `crate::fsutil` and call it
  from all three sites. Byte-identical behavior: same bound, same tail, same
  `(no output; exit N)` fallback, no log, no new key. **Proof:** every existing
  test passes with no *assertion* changed. Exactly one mechanical edit is
  allowed and expected — `test_runner.rs:537`, `:538` and `:544` name
  `FAILURE_EXCERPT_MAX` directly, so those three references retarget the
  surviving `pub(crate)` constant while asserting the same values. Any other
  test edit in this cell means the consolidation was not behavior-preserving
  and the cell is wrong.
  One difference to fold in rather than preserve: `finish_support.rs:118`
  assigns its trim back into `output` while the other two bind a local
  `trimmed`; `output` is dead afterwards (`:129` pushes only
  command/exit/duration/excerpt), so the local form is the one to keep.
  The exit type differs across the three — `Option<f64>`
  (`finish_support.rs:37`) against `Option<i64>` (`close.rs:82`,
  `test_runner.rs:218`) — and both render identically
  (`jsjson.rs:42` prints an integral f64 with no fraction, and
  `finish_support.rs:103` builds it from an integer code), so the helper takes
  whichever shape avoids a lossy conversion at any call site.
- **`ffe-2` — add the evidence.** On the now-single path, write the full output
  to the log, add the `failure_log` key, and add the refusal line. Tests to
  update are the ones this cell actually breaks, listed below.

**Retention (D3), stated once.** One log file per run, named for the runner and
the command index: `.bee/logs/test-failure-<runner>-<index>.log`, where
`<runner>` is `test`, `finish` or `close`. The runner segment answers Discovery
6 — three runners on one root never collide. Fixed names mean a later failure
overwrites its predecessor rather than accumulating. A run deletes the log for
every command index it ran that passed, and writes one for every index that
failed, so a mixed run leaves exactly the failing indices on disk. The delete
is best-effort and never changes a verdict.

**Bound (D2), stated as a number.** `failure_excerpt` remains ≤500 characters
of trimmed tail — unchanged, and now literally unchanged rather than
"unchanged plus a suffix". The log itself is unbounded; it is a file, not a
field.

**Error path, stated as a deliberate divergence.** Every adjacent write failure
in this area aborts the run (`finish_support.rs:132`, `close.rs:1025-1027`,
`test_runner.rs:125-130`). The log write does NOT: a failed log write leaves
`failure_log` null and the run's verdict untouched. The reason is that the log
is evidence about a failure, and losing evidence must never convert a red into
an error that hides which command failed. This is a divergence from the
surrounding posture and is stated so a worker does not read it as a bug.

**Rejected alternatives.**
- *Fold the path into `failure_excerpt`* — Discovery 2: it is hashed into the
  cell trace.
- *Raise the 500-character bound* — refused by D2 and by
  `docs/knowledge/patterns/20260723-clearing-a-red-by-widening-the-threshold-is-not-fixing-the-check.md`.
- *Put the helper in `textutil`* — Discovery 4: violates a locked charter.
- *Fix only `close.rs`, where the failure was seen* — the other two lose
  evidence identically.
- *Timestamped log files* — accumulate forever; fixed names plus per-run
  cleanup is the retention story.

**Risk map.**

| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| Consolidation (ffe-1) | MEDIUM | Three blocks that look identical may differ in a detail — `finish_support.rs:123` formats the exit through `jsjson::js_f64_to_string`, `close.rs:147` through `to_string` | Every existing test passes with no assertion changed; a diff review of the three blocks before extraction |
| New record key (ffe-2) | MEDIUM | `test_runner.rs:478-481` asserts the exact key vector | That assertion updated to expect the appended key, and `handlers_write.rs:698` still reads by name |
| Positional refusal text (ffe-2) | MEDIUM | `drivers/tests.rs:1904-1918` asserts close's red text by line index; a new line shifts `next:` from `lines[6]` to `lines[7]` | That test updated with the new index, still asserting the same content |
| Concurrent runners (D3) | LOW | Discovery 6 — two runs can share a root | Runner-segmented filenames; no test, the naming is the proof |
| Failure signature | LOW, load-bearing | The whole redraft exists to protect it | A test asserting `failure_excerpt` is byte-identical before and after ffe-2 for the same output |

## Shape

| Epic | Capability / Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| **E1** | One recorder instead of three | Three drifted copies guarantee the next fix lands twice | 1 | Every existing test green, unmodified |
| **E2** | Evidence that survives the run | An unreproduced red currently leaves nothing to read | 1 | Full output on disk, path in the record and the refusal, excerpt byte-identical |

**Slice queue.** One slice, two cells, `ffe-2` after `ffe-1`.

**Cell-facing specifics:**

- Helper home: `crate::fsutil`, beside `write_text_atomic`. Not `textutil`.
- Log path shape: `.bee/logs/test-failure-<runner>-<index>.log`, relative to
  the same root the run's `test-results.json` uses — `test_root` where the
  runner has one (`finish_support.rs:360-367`), the run root otherwise.
- Record key: `failure_log`, appended last, `null` when the command passed or
  the write failed.
- Refusal text: one line naming the log path, added after the excerpt block at
  `close.rs:1071` and the equivalent in `handlers_close.rs:229-241`.
- Tests `ffe-2` breaks, all of them: `cells/tests.rs:1697`, `:1702`, `:1704`,
  `:1710`; `test_runner.rs:478-481`, `:501`, `:528-530`; `drivers/tests.rs:1904-1918`.
- `docs/handbook/register.md:231-234` publishes the `test-results.json` shape
  and the `.bee/logs/` inventory; both gain the new file and key.

## Test matrix

Standard — the triad.

| Case | Cell | Probe |
|---|---|---|
| Happy path | ffe-2 | A failing command writes its complete output to the log; the record's `failure_log` names it; the log contains text the excerpt does not; `failure_excerpt` is byte-identical to what the same output produced before |
| Edge | ffe-2 | Empty-output failure keeps `(no output; exit N)` and still logs; a mixed run leaves logs for failing indices only; a green run removes a stale log |
| Error path | ffe-2 | An unwritable log directory leaves `failure_log` null, the excerpt intact, and the verdict unchanged |
| Consolidation | ffe-1 | The whole existing suite green with no assertion modified |

Declared suite at cap:
`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`.

## Out of scope

- Chasing the flake that prompted this.
- Framework-specific excerpt parsing (D4).
- Serializing the three runners against each other — Discovery 6 records the
  gap; runner-segmented names avoid it without a lock.
- The dispatcher message that reports a declined handler as an argument error.
