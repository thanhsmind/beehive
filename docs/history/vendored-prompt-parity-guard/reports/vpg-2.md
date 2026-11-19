# vpg-2 — vendored prompt parity test, red then green

Contract: vendored-prompt-parity (decision a9aedf60), checked on vpg-1 commit 289bb6f4c.
No source edited.

Verify command (all runs):

    PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- vendored_prompts

## Run 1 — one line appended to .bee/bin/prompts/peer-boot.md

Setup: `echo "extra line" >> .bee/bin/prompts/peer-boot.md`. Exit 101.

    vendored /mnt/data/Projects/goglbe/beehive--wt--vendored-prompt-parity-guard/packages/bee-rs/crates/bee/../../../../.bee/bin/prompts/peer-boot.md differs from its source — run `bee dev regen`
    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5503 filtered out; finished in 0.00s

Restored with `git checkout HEAD -- .bee/bin/prompts/peer-boot.md`.

## Run 2 — extra file .bee/bin/prompts/stray.md

Setup: `echo x > .bee/bin/prompts/stray.md`. Exit 101.

    vendored /mnt/data/Projects/goglbe/beehive--wt--vendored-prompt-parity-guard/packages/bee-rs/crates/bee/../../../../.bee/bin/prompts/stray.md has no source twin in packages/bee/prompts — run `bee dev regen`
    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5503 filtered out; finished in 0.00s

Removed with `rm .bee/bin/prompts/stray.md`.

## Run 3 — clean tree

Exit 0.

    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5503 filtered out; finished in 0.00s

`git status --short .bee/bin/prompts` printed nothing: the directory is clean.
