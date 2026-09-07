# install-fetch-reason — context

## The report

A user ran `scripts/install.sh`, expected the published binary, and watched a
full `cargo build --release` of the cloned tree in `/tmp/tmp.*/bee` instead.
Asked why, they could not answer: the log line the installer printed on the way
into that build names no reason.

## What is established

- The published binary path is not broken. Every one of the last 12 releases
  carries all three assets, and v2.33.3's assets were uploaded one second after
  the release was published, so no publish-window race explains it.
- The binary installed into the main checkout at 09:16 today IS the published
  one: `sha256sum .bee/bin/bee` = `add5eb6a…677e2`, byte-identical to
  `bee-x86_64-unknown-linux-gnu` on v2.33.3. The build the user saw came from a
  different run.
- `install.ps1` already carries the fix for exactly this: its fallback lines
  interpolate `$prebuiltErr` / `$_.Exception.Message`, under a comment that
  says a reasonless line made "a TLS failure, a proxy, a rate limit and a
  genuinely missing release ... indistinguishable in the log".
- `install.sh` never got it. `scripts/install.sh:227` prints
  `could not resolve a published release — building from source` and
  `scripts/install.sh:242` prints
  `no downloadable asset at $PREBUILT_TAG — building from source`. Neither
  carries the curl/wget error, because `fetch()` sends stderr nowhere.
- `installer_contracts.rs::install_ps1_names_why_the_published_binary_was_skipped`
  pins the Windows half of that contract and asserts nothing about the POSIX
  half. The test is the parity gap in writing.

## The gap this closes

A POSIX user cannot tell a blipped download from a proxy from a real missing
asset, and pays several minutes of compile either way. Naming the reason is
what turns "why did it rebuild" into a one-line answer, and a retry is what
stops a single blip from costing that build at all.

## Out of scope

Retry on `install.ps1`. `Invoke-WebRequest`'s `-MaximumRetryCount` does not
exist on Windows PowerShell 5.1, which the script explicitly still supports
(it raises the TLS floor for it), so retry parity there is its own question.
