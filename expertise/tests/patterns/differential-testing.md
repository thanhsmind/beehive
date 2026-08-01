# Differential testing

## Context

When a second implementation must match an existing one: a port to another
language, a rewrite behind a flag, a cache beside its source of truth, a
fast path beside the slow path it replaces. Hand-written expectations are
the wrong tool here — you would be asserting what you *believe* the old
implementation does, and every belief you got wrong ships as a silent
behavior change.

## Mechanism

Run both implementations on identical input and compare their entire
observable output: stdout, stderr, exit code, and every file they wrote.
The old implementation is the oracle; you never write down what it does.

Three rules make this hold:

1. **Compare bytes, not shapes.** `assert_eq!` on the whole output, not a
   substring or a parsed subset. A subset comparison passes while the field
   you forgot to look at diverges.
2. **Normalize only what legitimately varies**, and normalize it on both
   sides — durations, generated ids, timestamps, pids. Prove the token
   really varies by running the OLD implementation twice: if two runs of the
   oracle differ, that token is noise; if they agree, a difference is a real
   defect and normalizing it would hide the bug you are hunting.
3. **Give each side its own world.** Run the two implementations against
   separate copies of the fixture, never the same directory in sequence.
   Any state one writes — a cache, a dedup stamp, a lock file — changes what
   the other sees, and the diff then reports your test's ordering rather
   than the implementations' behavior.

For mutating operations the output is not only what was printed: diff the
resulting store trees too, with the same normalization discipline.

## Example

```bash
# Each side gets its own copy of the same starting world.
cp -r fixture/ /tmp/twin-a/ && cp -r fixture/ /tmp/twin-b/

(cd /tmp/twin-a && node old-impl.mjs status --json) > /tmp/a.out 2>/tmp/a.err
(cd /tmp/twin-b && ./new-impl        status --json) > /tmp/b.out 2>/tmp/b.err

# stdout must match byte for byte; stderr only after masking the duration
# token, which two runs of the OLD implementation already disagree on.
diff /tmp/a.out /tmp/b.out
sed -E 's/[0-9]+ms/Nms/' /tmp/a.err | diff - <(sed -E 's/[0-9]+ms/Nms/' /tmp/b.err)
diff -r /tmp/twin-a /tmp/twin-b   # the stores they wrote
```

```rust
// Good — the whole payload is the assertion; the oracle defines the answer.
assert_eq!(old.stdout, new.stdout);
assert_eq!(old.status.code(), new.status.code());

// Bad — asserts the fields you thought to check. The one you forgot is
// exactly where the port drifts.
assert_eq!(parse(&new.stdout)["phase"], parse(&old.stdout)["phase"]);
```

## Notes

A differential harness earns more than parity: it is a bug detector for the
*old* implementation too. When the two disagree, read the difference before
assuming the new side is wrong — a sort that looked fine until two
implementations disagreed, an error message that interpolated a converted
value instead of the raw one, a spawn that resolved to a different binary
than intended. Each of those is a real defect the oracle was hiding, and the
diff is what made it visible.

Keep the harness running until the old implementation is deleted. It is the
only thing standing between "the new path works on the cases I imagined" and
"the new path answers identically on the cases that actually occur."
