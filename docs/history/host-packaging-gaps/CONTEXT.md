# host-packaging-gaps — context

## What this is

A fresh-host install of bee 2.43.0 (network installer, empty git repo,
2026-09-22) showed four gaps that stop bee from working fully on a
project other than the bee repo. The user asked for all four to be
fixed in one pass ("làm luôn cả 4 vấn đề đi").

Evidence run: `scripts/install.sh -d <fresh repo> -y` from
`raw.githubusercontent.com/.../main`, then `bee doctor --runtime
claude|codex|pi` and `bee dispatch prepare --runtime pi` in that repo.

## Locked decisions

**D1 — Pi doctor freshness judges a host against its own install record.**
In a host repo there is no `.claude-plugin/plugin.json`, so the Pi
`binary_freshness` row reports unknown and the whole Pi verdict is
BLOCKED on every host. The row stays mandatory and fail-closed for Pi
(it was put there on purpose — `5c61c85ed`). In a host, the expected
version comes from `.bee/onboarding.json` `bee_version`, the version the
onboarder installed. A match is ok, a mismatch is not_ok, and a host
with neither file stays unknown. The source-checkout path is unchanged.

**D2 — A freshly onboarded host can dispatch Pi workers.**
Onboarding writes a `team.pi` role table and a Pi herding agent, so
`bee dispatch prepare --runtime pi` resolves instead of refusing
`pi_requires_herding`. The default Pi agent runs the plain `pi` binary
with the user's own Pi default model: bee does not pick a provider or a
model for the user. An existing host config gains the Pi keys only
where they are absent; no existing value changes.

**D3 — `pi` is a first-class `--runtime` value.**
`bee onboard`, `scripts/install.sh` and `scripts/install.ps1` accept
`pi` beside `claude`, `codex` and `both`, and the docs name it.

**D4 — The release ships macOS and ARM Linux binaries.**
`release-binaries.yml` adds `aarch64-apple-darwin`,
`x86_64-apple-darwin` and `aarch64-unknown-linux-gnu`, each built and
version-checked on a native runner. `scripts/install.sh` maps
`Darwin/arm64`, `Darwin/x86_64` and `Linux/aarch64|arm64` to them, and
its checksum step works on stock macOS (`shasum -a 256` when
`sha256sum` is absent). `scripts/release.sh` expects the new count.

**D5 — Pi doctor passes in a plain terminal when every Pi role runs no pane.**
Added on the user's word ("sửa luôn cả lỗi herdr đó đi", 2026-09-22)
after the plan-step user-impact seat showed that a Pi user outside
herdr or tmux still sees BLOCKED on `herding_transport`. A Pi agent on
runtime pi dispatches with `--no-pane`, which needs no multiplexer. The
row is ok when every `team.pi` slot is such an agent; any pane-transport
Pi slot, or no `team.pi`, keeps today's HERDR/TMUX check.

## Out of scope

- `bee doctor --runtime opencode` (a fifth gap seen in the same run).
- A Windows ARM64 asset.
- A doctor row that checks Pi has a default model or auth.
