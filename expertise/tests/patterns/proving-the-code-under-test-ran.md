# Proving the code under test ran

## Context

When the system has a way to produce the right answer *without* the code you
are testing: a delegate to an older implementation, a retry, a cache, a
default, an environment fallback. The test asserts an outcome, the outcome
is correct, and the test passes — while the path you meant to exercise never
executed. The suite reports coverage it does not have, and the day the
fallback is removed, every one of those tests turns red at once.

## Mechanism

Make the fallback impossible, loudly, for the duration of the test — then
assert the outcome as usual. If the code under test did not run, the test
must fail with a distinctive signal rather than quietly succeeding.

Two shapes, depending on what you control:

- **Sabotage the fallback.** Point it at something that cannot work — an
  unreachable path, a missing binary, a closed port. A run that reaches the
  fallback dies with a recognizable error instead of answering correctly.
- **Trip a wire in the fallback.** When the fallback is your own code, have
  it exit with a reserved code and a named message under a test-only
  environment flag. This is the better shape: the failure says *which*
  fallback fired, not merely that something went wrong.

Assert on the tripwire's absence, not only on the payload. A green outcome
plus a silent tripwire is the proof; a green outcome alone is not.

## Example

```rust
// The fallback exits 42 and names itself when the tripwire is armed.
fn delegate(name: &str) -> ExitCode {
    if std::env::var_os("NO_DELEGATE").is_some() {
        eprintln!("DELEGATED to {name} (tripwire)");
        return ExitCode::from(42);
    }
    run_old_implementation(name)
}
```

```bash
# Good — a delegation now fails loudly instead of answering correctly.
NO_DELEGATE=1 ./bin status --json > out.txt 2>&1
test $? -ne 42 || { echo "FAIL: fell back to the old path"; exit 1; }

# Bad — passes identically whether the native path ran or the old one did.
./bin status --json | grep -q '"phase"'
```

## Notes

The same blindness has a milder form worth checking for: a test that would
pass even if the feature were deleted. When an assertion is weak enough to
hold on an empty result, a default value, or an unrelated code path, it is
measuring nothing. Break the production code on purpose once and watch the
test go red — if it stays green, the test never tested it.

Keep the tripwire out of the shipped default path. It is armed by an
environment variable the product never sets, and it must never be able to
change behavior for a real user.
