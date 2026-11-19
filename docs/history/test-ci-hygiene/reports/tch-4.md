# tch-4 — fake-backend-gated contract check

Checked on commit 3ed579166 (tch-3). No source edited.

## Run 1 — release bee binary holds no FakeBackend

```
cargo build --release --manifest-path packages/bee-rs/Cargo.toml -p bee
    Finished `release` profile [optimized] target(s) in 1m 22s
grep -a -c FakeBackend ~/.cache/cargo-target/release/bee
0
```

## Run 2 — normal bee build does not enable test-support

```
cargo tree --manifest-path packages/bee-rs/Cargo.toml -p bee -e features -e normal | grep -E 'fleet|test-support'
├── fleet feature "default"
│   └── fleet v0.1.0 (.../packages/bee-rs/crates/fleet)
cargo tree ... -p bee -e features -e normal | grep -c test-support
0
```

Gate in source, `packages/bee-rs/crates/fleet/src/backend.rs:12-13`:

```
#[cfg(any(test, feature = "test-support"))]
pub mod fake;
```

So a build of fleet without the feature does not declare `fleet::backend::fake`.

## Run 3 — choreography and wave tests pass

```
cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p fleet -p bee -- choreography wave
bee unittests:        test result: ok. 116 passed; 0 failed; 1 ignored; 0 measured; 5387 filtered out
fleet unittests:      test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 67 filtered out
tests/choreography.rs: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out
cargo test --release ... -p fleet --test choreography
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Cell verify

```
cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p fleet
exit 0; 117 passed, 0 failed across all fleet test binaries
```
