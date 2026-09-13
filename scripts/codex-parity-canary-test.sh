#!/usr/bin/env bash
# Hermetic behavioral tests for scripts/codex-parity-canary.sh isolation.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CANARY_SCRIPT="$REPO_ROOT/scripts/codex-parity-canary.sh"

TEST_TMP="$(mktemp -d "${TMPDIR:-/var/tmp}/canary-isolation-test-XXXXXX")"
trap 'rm -rf "$TEST_TMP"' EXIT

FAILURES=0

record_pass() {
  printf 'PASS: %s\n' "$1"
}

record_fail() {
  printf 'FAIL: %s — %s\n' "$1" "$2" >&2
  FAILURES=$((FAILURES + 1))
}

# Ensure a mock release bee binary exists if none is built
MOCK_BEE="$TEST_TMP/bin/bee"
mkdir -p "$TEST_TMP/bin"
cat << EOF > "$MOCK_BEE"
#!/usr/bin/env bash
if [[ "\${1:-}" == "onboard" ]]; then
  target=""
  while [[ \$# -gt 0 ]]; do
    if [[ "\$1" == "--repo-root" ]]; then
      target="\$2"
      shift 2
    else
      shift
    fi
  done
  if [[ -n "\$target" ]]; then
    mkdir -p "\$target/.bee/bin" "\$target/.codex" "\$target/docs/history/canary"
    if [[ -f "$REPO_ROOT/.codex/hooks.json" ]]; then
      cp "$REPO_ROOT/.codex/hooks.json" "\$target/.codex/hooks.json"
    fi
    touch "\$target/.bee/backlog.jsonl"
    touch "\$target/.bee/state.json"
    echo '{"status": "applied"}'
    exit 0
  fi
fi
echo '{"status": "applied"}'
exit 0
EOF
chmod +x "$MOCK_BEE"

# -----------------------------------------------------------------------------
# Test 1: Missing CODEX_BIN is refused without invoking mise or PATH discovery
# -----------------------------------------------------------------------------
test_missing_codex_bin() {
  local name="Missing CODEX_BIN is refused without mise discovery"
  local fake_mise_dir="$TEST_TMP/fake_mise_bin"
  mkdir -p "$fake_mise_dir"
  local fake_mise_sentinel="$TEST_TMP/fake_mise_invoked.txt"
  cat << EOF > "$fake_mise_dir/mise"
#!/bin/sh
echo "MISE_WAS_INVOKED" > "$fake_mise_sentinel"
exit 1
EOF
  chmod +x "$fake_mise_dir/mise"

  local safe_home="$TEST_TMP/test1_home"
  mkdir -p "$safe_home"

  local output
  set +e
  output=$(PATH="$fake_mise_dir:$PATH" \
           HOME="$safe_home" \
           CANARY_CODEX_HOME="$safe_home/codex" \
           BEE_BIN="$MOCK_BEE" \
           env -u CODEX_BIN bash "$CANARY_SCRIPT" 2>&1)
  local status=$?
  set -e

  if [[ $status -eq 0 ]]; then
    record_fail "$name" "Canary exited 0 when CODEX_BIN was missing."
    return
  fi

  if [[ -f "$fake_mise_sentinel" ]]; then
    record_fail "$name" "Canary invoked mise when CODEX_BIN was unset."
    return
  fi

  if [[ "$output" =~ (explicit|direct|required|CODEX_BIN) ]]; then
    record_pass "$name"
  else
    record_fail "$name" "Error message did not mention CODEX_BIN or explicit executable requirement: $output"
  fi
}

# -----------------------------------------------------------------------------
# Test 2: Wrapper script or bare PATH command in CODEX_BIN is refused
# -----------------------------------------------------------------------------
test_wrapper_codex_bin() {
  local name="Wrapper script and bare PATH command in CODEX_BIN are refused"
  local safe_home="$TEST_TMP/test2_home"
  mkdir -p "$safe_home"

  # Case 2a: bare command name without path separator
  local out_bare
  set +e
  out_bare=$(HOME="$safe_home" CANARY_CODEX_HOME="$safe_home/codex" \
             BEE_BIN="$MOCK_BEE" CODEX_BIN="codex" bash "$CANARY_SCRIPT" 2>&1)
  local status_bare=$?
  set -e

  if [[ $status_bare -eq 0 ]]; then
    record_fail "$name" "Canary accepted bare command name 'codex' in CODEX_BIN."
    return
  fi

  if [[ ! "$out_bare" =~ (path|explicit|wrapper|direct|CODEX_BIN) ]]; then
    record_fail "$name" "Error message for bare CODEX_BIN was not specific: $out_bare"
    return
  fi

  # Case 2b: wrapper script invoking mise
  local wrapper_sentinel="$TEST_TMP/wrapper_executed.txt"
  local wrapper_file="$TEST_TMP/mise_wrapper.sh"
  cat << EOF > "$wrapper_file"
#!/usr/bin/env bash
echo "WRAPPER_WAS_EXECUTED" > "$wrapper_sentinel"
mise use -g codex
exec codex "\$@"
EOF
  chmod +x "$wrapper_file"

  local out_wrapper
  set +e
  out_wrapper=$(HOME="$safe_home" CANARY_CODEX_HOME="$safe_home/codex" \
                BEE_BIN="$MOCK_BEE" CODEX_BIN="$wrapper_file" bash "$CANARY_SCRIPT" 2>&1)
  local status_wrapper=$?
  set -e

  if [[ -f "$wrapper_sentinel" ]]; then
    record_fail "$name" "Mise wrapper script was executed by canary instead of refused."
    return
  fi

  if [[ $status_wrapper -eq 0 ]]; then
    record_fail "$name" "Canary accepted mise wrapper script in CODEX_BIN."
    return
  fi

  if [[ ! "$out_wrapper" =~ (mise|wrapper|direct) ]]; then
    record_fail "$name" "Error message for wrapper script did not name mise or wrapper: $out_wrapper"
    return
  fi

  record_pass "$name"
}

# -----------------------------------------------------------------------------
# Test 3: Canary cannot resolve user's Codex wrapper through inherited PATH
# -----------------------------------------------------------------------------
test_inherited_path_wrapper() {
  local name="Canary cannot resolve user's Codex wrapper through inherited PATH"
  local wrapper_dir="$TEST_TMP/host_wrapper_dir"
  local wrapper_sentinel="$TEST_TMP/host_wrapper_invoked.txt"
  mkdir -p "$wrapper_dir"
  cat << EOF > "$wrapper_dir/codex"
#!/bin/sh
echo "HOST_WRAPPER_INVOKED" > "$wrapper_sentinel"
exit 1
EOF
  chmod +x "$wrapper_dir/codex"

  local safe_home="$TEST_TMP/test3_home"
  mkdir -p "$safe_home"

  local wrapper_on_path_marker="$TEST_TMP/wrapper_on_path_observed.txt"
  # Direct probe executable that checks what 'command -v codex' resolves to
  local direct_bin="$TEST_TMP/direct_codex_test3"
  cat << EOF > "$direct_bin"
#!/usr/bin/env bash
resolved=\$(command -v codex || true)
if [[ "\$resolved" == *host_wrapper_dir* ]]; then
  echo "\$resolved" > "$wrapper_on_path_marker"
fi
if [[ "\${1:-}" == "--version" ]]; then
  echo "codex 0.154.0 (direct)"
  exit 0
fi
echo "codex-test-ok"
exit 0
EOF
  chmod +x "$direct_bin"

  set +e
  PATH="$wrapper_dir:$PATH" \
  HOME="$safe_home" \
  CANARY_CODEX_HOME="$safe_home/codex" \
  BEE_BIN="$MOCK_BEE" \
  CODEX_BIN="$direct_bin" \
  bash "$CANARY_SCRIPT" >/dev/null 2>&1
  set -e

  if [[ -f "$wrapper_sentinel" ]]; then
    record_fail "$name" "Host Codex wrapper was invoked via inherited PATH."
    return
  fi

  if [[ -f "$wrapper_on_path_marker" ]]; then
    local bad_path
    bad_path=$(cat "$wrapper_on_path_marker")
    record_fail "$name" "Canary subprocess resolved host wrapper on inherited PATH: $bad_path."
    return
  fi

  record_pass "$name"
}

# -----------------------------------------------------------------------------
# Test 4: Unsafe homes and symlinks into real settings are rejected before probe
# -----------------------------------------------------------------------------
test_unsafe_homes() {
  local name="Unsafe homes and symlinks into real settings are rejected before probe launch"

  local fake_host="$TEST_TMP/fake_host_home"
  mkdir -p "$fake_host/.codex" "$fake_host/.config"
  echo "HOST_SENTINEL_UNTOUCHED" > "$fake_host/sentinel.txt"
  echo "HOST_AUTH_UNTOUCHED" > "$fake_host/.codex/auth.json"

  local probe_executed_marker="$TEST_TMP/probe_executed_marker.txt"
  local dummy_direct="$TEST_TMP/dummy_direct"
  cat << EOF > "$dummy_direct"
#!/usr/bin/env bash
echo "PROBE_WAS_EXECUTED" > "$probe_executed_marker"
echo "codex 0.154.0"
exit 0
EOF
  chmod +x "$dummy_direct"

  # 4a: CANARY_CODEX_HOME is the real home itself
  rm -f "$probe_executed_marker"
  local out_real_home
  set +e
  out_real_home=$(HOME="$fake_host" CANARY_CODEX_HOME="$fake_host" \
                  BEE_BIN="$MOCK_BEE" CODEX_BIN="$dummy_direct" bash "$CANARY_SCRIPT" 2>&1)
  local status_real_home=$?
  set -e
  if [[ $status_real_home -eq 0 ]]; then
    record_fail "$name" "Canary accepted real home as CANARY_CODEX_HOME."
    return
  fi
  if [[ -f "$probe_executed_marker" ]]; then
    record_fail "$name" "Probe executed before refusing real home."
    return
  fi

  # 4b: CANARY_CODEX_HOME is real .codex
  rm -f "$probe_executed_marker"
  local out_real_codex
  set +e
  out_real_codex=$(HOME="$fake_host" CANARY_CODEX_HOME="$fake_host/.codex" \
                   BEE_BIN="$MOCK_BEE" CODEX_BIN="$dummy_direct" bash "$CANARY_SCRIPT" 2>&1)
  local status_real_codex=$?
  set -e
  if [[ $status_real_codex -eq 0 ]]; then
    record_fail "$name" "Canary accepted real .codex as CANARY_CODEX_HOME."
    return
  fi
  if [[ -f "$probe_executed_marker" ]]; then
    record_fail "$name" "Probe executed before refusing real .codex."
    return
  fi

  # 4c: CANARY_CODEX_HOME is a symlink pointing to real home
  rm -f "$probe_executed_marker"
  local symlink_home="$TEST_TMP/symlink_to_real_home"
  ln -s "$fake_host" "$symlink_home"
  local out_symlink_home
  set +e
  out_symlink_home=$(HOME="$fake_host" CANARY_CODEX_HOME="$symlink_home" \
                     BEE_BIN="$MOCK_BEE" CODEX_BIN="$dummy_direct" bash "$CANARY_SCRIPT" 2>&1)
  local status_symlink_home=$?
  set -e
  if [[ $status_symlink_home -eq 0 ]]; then
    record_fail "$name" "Canary accepted symlink to real home as CANARY_CODEX_HOME."
    return
  fi
  if [[ -f "$probe_executed_marker" ]]; then
    record_fail "$name" "Probe executed before refusing symlink to real home."
    return
  fi

  # 4d: CANARY_CODEX_HOME contains a symlink pointing to real auth.json
  rm -f "$probe_executed_marker"
  local dir_with_link="$TEST_TMP/dir_with_link"
  mkdir -p "$dir_with_link"
  ln -s "$fake_host/.codex/auth.json" "$dir_with_link/auth.json"
  local out_link_inside
  set +e
  out_link_inside=$(HOME="$fake_host" CANARY_CODEX_HOME="$dir_with_link" \
                    BEE_BIN="$MOCK_BEE" CODEX_BIN="$dummy_direct" bash "$CANARY_SCRIPT" 2>&1)
  local status_link_inside=$?
  set -e
  if [[ $status_link_inside -eq 0 ]]; then
    record_fail "$name" "Canary accepted CANARY_CODEX_HOME containing symlink into real auth.json."
    return
  fi
  if [[ -f "$probe_executed_marker" ]]; then
    record_fail "$name" "Probe executed before refusing symlink inside CANARY_CODEX_HOME."
    return
  fi

  # 4e: CANARY_CODEX_HOME is root /
  rm -f "$probe_executed_marker"
  local out_root
  set +e
  out_root=$(HOME="$fake_host" CANARY_CODEX_HOME="/" \
             BEE_BIN="$MOCK_BEE" CODEX_BIN="$dummy_direct" bash "$CANARY_SCRIPT" 2>&1)
  local status_root=$?
  set -e
  if [[ $status_root -eq 0 ]]; then
    record_fail "$name" "Canary accepted root directory as CANARY_CODEX_HOME."
    return
  fi
  if [[ -f "$probe_executed_marker" ]]; then
    record_fail "$name" "Probe executed before refusing root directory."
    return
  fi

  # Check that host sentinels were not touched
  if [[ "$(cat "$fake_host/sentinel.txt")" != "HOST_SENTINEL_UNTOUCHED" || \
        "$(cat "$fake_host/.codex/auth.json")" != "HOST_AUTH_UNTOUCHED" ]]; then
    record_fail "$name" "Host sentinels were modified during rejection checks."
    return
  fi

  record_pass "$name"
}

# -----------------------------------------------------------------------------
# Test 5: Hostile fake subprocess writes only isolated HOME and XDG paths
# -----------------------------------------------------------------------------
test_hostile_subprocess_isolation() {
  local name="Hostile fake subprocess writes only isolated HOME and XDG paths"

  local host_home="$TEST_TMP/host_home_test5"
  mkdir -p "$host_home/.config" "$host_home/.local/share" "$host_home/.cache"
  echo "HOST_HOME_SENTINEL" > "$host_home/sentinel.txt"
  echo "HOST_CONFIG_SENTINEL" > "$host_home/.config/sentinel.txt"
  echo "HOST_DATA_SENTINEL" > "$host_home/.local/share/sentinel.txt"
  echo "HOST_CACHE_SENTINEL" > "$host_home/.cache/sentinel.txt"

  local safe_codex_home="$TEST_TMP/safe_codex_test5"
  mkdir -p "$safe_codex_home"

  local hostile_bin="$TEST_TMP/hostile_codex_bin"
  cat << 'EOF' > "$hostile_bin"
#!/usr/bin/env bash
if [[ "${1:-}" == "--version" ]]; then
  echo "codex 0.154.0-hostile"
  # Attempt to vandalize HOME and XDG paths
  echo "CORRUPTED" >> "$HOME/sentinel.txt" 2>/dev/null || true
  echo "CORRUPTED" >> "${XDG_CONFIG_HOME:-$HOME/.config}/sentinel.txt" 2>/dev/null || true
  echo "CORRUPTED" >> "${XDG_DATA_HOME:-$HOME/.local/share}/sentinel.txt" 2>/dev/null || true
  echo "CORRUPTED" >> "${XDG_CACHE_HOME:-$HOME/.cache}/sentinel.txt" 2>/dev/null || true
  exit 0
fi
exit 0
EOF
  chmod +x "$hostile_bin"

  # Run canary through version check with the hostile binary
  set +e
  HOME="$host_home" \
  XDG_CONFIG_HOME="$host_home/.config" \
  XDG_DATA_HOME="$host_home/.local/share" \
  XDG_CACHE_HOME="$host_home/.cache" \
  CANARY_CODEX_HOME="$safe_codex_home" \
  BEE_BIN="$MOCK_BEE" \
  CODEX_BIN="$hostile_bin" \
  bash "$CANARY_SCRIPT" >/dev/null 2>&1
  set -e

  # Verify host sentinels remain 100% UNCHANGED
  if [[ "$(cat "$host_home/sentinel.txt")" != "HOST_HOME_SENTINEL" ]]; then
    record_fail "$name" "Host HOME sentinel was corrupted by hostile subprocess."
    return
  fi
  if [[ "$(cat "$host_home/.config/sentinel.txt")" != "HOST_CONFIG_SENTINEL" ]]; then
    record_fail "$name" "Host XDG_CONFIG sentinel was corrupted by hostile subprocess."
    return
  fi
  if [[ "$(cat "$host_home/.local/share/sentinel.txt")" != "HOST_DATA_SENTINEL" ]]; then
    record_fail "$name" "Host XDG_DATA sentinel was corrupted by hostile subprocess."
    return
  fi
  if [[ "$(cat "$host_home/.cache/sentinel.txt")" != "HOST_CACHE_SENTINEL" ]]; then
    record_fail "$name" "Host XDG_CACHE sentinel was corrupted by hostile subprocess."
    return
  fi

  record_pass "$name"
}

# -----------------------------------------------------------------------------
# Test 6: Bash syntax of scripts/codex-parity-canary.sh
# -----------------------------------------------------------------------------
test_syntax() {
  local name="Syntax check bash -n scripts/codex-parity-canary.sh"
  if bash -n "$CANARY_SCRIPT"; then
    record_pass "$name"
  else
    record_fail "$name" "bash -n reported syntax errors in $CANARY_SCRIPT"
  fi
}

echo "Running hermetic canary isolation tests..."
test_missing_codex_bin
test_wrapper_codex_bin
test_inherited_path_wrapper
test_unsafe_homes
test_hostile_subprocess_isolation
test_syntax

echo "--------------------------------------------------"
if [[ $FAILURES -eq 0 ]]; then
  echo "ALL TESTS PASSED."
  exit 0
else
  echo "TEST FAILURES: $FAILURES"
  exit 1
fi
