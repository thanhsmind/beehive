# install.ps1 - install bee (https://github.com/thanhsmind/beehive) into a project.
#
# Two layers:
#   1. Runtime layer (opt-in, -GlobalSkills): copy the bee skills into your
#      agent's global skills directory (~/.claude/skills and/or ~/.codex/skills).
#      Off by default - the per-project sync in layer 2 is the default layout.
#   2. Repo layer: run `bee onboard` against the target project - installs the
#      AGENTS.md BEE block, .bee/ runtime files, vendored helpers, and (by
#      default) syncs the bee skills per-project into <repo>/.claude/skills and
#      <repo>/.agents/skills.
#
# Greenfield (empty dir / no git) and brownfield (existing repo) are both
# supported: onboarding merges via BEE:START/END markers, never touches content
# outside them, never overwrites existing state, and is idempotent.
#
# Flags:
#   -GlobalSkills   Also copy bee skills into the legacy global runtime
#                   directories (~/.claude/skills, ~/.codex/skills) and pass
#                   --global-skills through to onboarding. Off by default -
#                   onboarding's per-project sync (layer 2) is the default.
#   -NoClaudeMd     Skip writing/extending CLAUDE.md with the bare @AGENTS.md
#                   import. By default onboarding writes it.
#   -ClaudeMd       Accepted for compatibility; a no-op alias of the default
#                   (CLAUDE.md is written unless -NoClaudeMd is passed).
#
# Examples:
#   .\scripts\install.ps1                                   # this checkout -> current dir
#   .\scripts\install.ps1 -Directory C:\proj -Yes           # non-interactive
#   .\scripts\install.ps1 -DryRun                           # plan only
#   .\scripts\install.ps1 -GlobalSkills -Yes                # also install skills globally
#   iwr -useb https://raw.githubusercontent.com/thanhsmind/beehive/main/scripts/install.ps1 -OutFile install-bee.ps1
#   .\install-bee.ps1 -Directory C:\proj -Yes

[CmdletBinding()]
param(
  [string]$Directory = (Get-Location).Path,
  [ValidateSet('claude', 'codex', 'both')]
  [string]$Runtime = 'both',
  [ValidateSet('plugin-first', 'repo-copy')]
  [string]$Distribution = 'repo-copy',
  [string]$PluginStateFile = '',
  [string]$OwnershipLedger = '',
  [string]$Source = '',
  [string]$Ref = 'main',
  [switch]$BuildFromSource,
  [switch]$NoHooks,
  [switch]$GlobalSkills,
  [switch]$NoClaudeMd,
  [switch]$ClaudeMd,
  [switch]$NoGitInit,
  [switch]$Yes,
  [switch]$DryRun
)

$ErrorActionPreference = 'Stop'
$RepoUrl = 'https://github.com/thanhsmind/beehive.git'

function Fail([string]$Message) { Write-Error $Message; exit 1 }

function Confirm-Step([string]$Question) {
  if ($Yes) { return $true }
  if (-not [Environment]::UserInteractive) {
    Fail "$Question - no interactive console. Re-run with -Yes to accept."
  }
  $answer = Read-Host "$Question [y/N]"
  return $answer -match '^(y|yes)$'
}

# Runs `<cli> plugin list --json` without ever letting a broken CLI's raw
# stderr stream straight to the terminal (field report: a codex npm shim that
# crashes on invocation prints a multi-line red error block before the
# installer's own one-line warning, which reads as a hard failure even though
# the install continues fine). This is the sanctioned exception to the
# no-bare-2>-redirection rule above: the redirect happens INSIDE a scriptblock
# that sets $ErrorActionPreference = 'Continue' for the native call, so a
# nonzero exit becomes ordinary stderr output instead of a terminating
# NativeCommandError. Returns ExitCode, StdOut (plain string lines) and StdErr
# (stringified error-stream lines) so callers decide how much to surface.
function Invoke-PluginListProbe([string]$CliName) {
  & {
    $ErrorActionPreference = 'Continue'
    $out = @()
    $err = @()
    & $CliName plugin list --json 2>&1 | ForEach-Object {
      if ($_ -is [System.Management.Automation.ErrorRecord]) { $err += $_.ToString() }
      else { $out += [string]$_ }
    }
    [PSCustomObject]@{ ExitCode = $LASTEXITCODE; StdOut = $out; StdErr = $err }
  }
}

# D8: pre-confirmation is READ-ONLY. Builds a combined { claude, codex }
# plugin-state json file from `plugin list` status probes ONLY - never a
# mutating verb (marketplace add / plugin add / install / remove /
# uninstall). Mirrors install.sh's probe_plugin_state, including its
# --plugin-state-file early return (a caller-supplied fixture file is used
# as-is, no real CLI probing, so fixture-driven runs never touch a real
# runtime plugin).
function New-PluginStateFile([string]$DestDir) {
  if ($PluginStateFile) { return (Resolve-Path $PluginStateFile).ProviderPath }

  $claudeStatePath = Join-Path $DestDir 'claude.json'
  $codexStatePath = Join-Path $DestDir 'codex.json'
  Set-Content -Encoding ASCII -Path $claudeStatePath -Value '[]'
  Set-Content -Encoding ASCII -Path $codexStatePath -Value '[]'

  if ($Runtime -in @('codex', 'both')) {
    $codex = Get-Command codex -ErrorAction SilentlyContinue
    if ($codex) {
      $codexStatusProbe = Invoke-PluginListProbe 'codex'
      if ($codexStatusProbe.ExitCode -ne 0) {
        # CLI is on PATH but not runnable (field report: a codex npm shim that
        # crashes with "Missing optional dependency @openai/codex-linux-x64"
        # on Windows+WSL). plugin-first genuinely needs the CLI, so it still
        # refuses, but now with a named, actionable message. repo-copy never
        # calls the CLI for anything but this read-only probe, so it warns
        # and keeps the pre-seeded '[]' state content instead of failing.
        $codexFirstErr = if ($codexStatusProbe.StdErr.Count -gt 0) { $codexStatusProbe.StdErr[0] } else { '' }
        if ($Distribution -eq 'plugin-first') {
          foreach ($line in $codexStatusProbe.StdErr) { Write-Host $line }
          Fail "codex CLI is on PATH but not runnable ('codex plugin list --json' failed). Fix options: repair or reinstall the codex CLI, re-run with -Distribution repo-copy (does not require a runtime CLI), or re-run with -Runtime claude to exclude codex."
        } else {
          Write-Warning "codex CLI found on PATH but not runnable ('codex plugin list --json' failed: $codexFirstErr); repo-copy does not require it, continuing without it."
        }
      } else {
        Set-Content -Encoding UTF8 -Path $codexStatePath -Value ($codexStatusProbe.StdOut -join "`n")
      }
    } elseif ($Distribution -eq 'plugin-first') { Fail 'Codex CLI is required for plugin-first' }
  }

  if ($Runtime -in @('claude', 'both')) {
    $claude = Get-Command claude -ErrorAction SilentlyContinue
    if ($claude) {
      $claudeStatusProbe = Invoke-PluginListProbe 'claude'
      if ($claudeStatusProbe.ExitCode -ne 0) {
        # Same broken-but-present-CLI policy as the codex probe above.
        $claudeFirstErr = if ($claudeStatusProbe.StdErr.Count -gt 0) { $claudeStatusProbe.StdErr[0] } else { '' }
        if ($Distribution -eq 'plugin-first') {
          foreach ($line in $claudeStatusProbe.StdErr) { Write-Host $line }
          Fail "claude CLI is on PATH but not runnable ('claude plugin list --json' failed). Fix options: repair or reinstall the claude CLI, re-run with -Distribution repo-copy (does not require a runtime CLI), or re-run with -Runtime codex to exclude claude."
        } else {
          Write-Warning "claude CLI found on PATH but not runnable ('claude plugin list --json' failed: $claudeFirstErr); repo-copy does not require it, continuing without it."
        }
      } else {
        Set-Content -Encoding UTF8 -Path $claudeStatePath -Value ($claudeStatusProbe.StdOut -join "`n")
      }
    } elseif ($Distribution -eq 'plugin-first') { Fail 'Claude CLI is required for plugin-first' }
  }

  $combined = @{ claude = (Get-Content $claudeStatePath -Raw | ConvertFrom-Json); codex = (Get-Content $codexStatePath -Raw | ConvertFrom-Json) }
  $result = Join-Path $DestDir 'state.json'
  Set-Content -Encoding UTF8 -Path $result -Value ($combined | ConvertTo-Json -Depth 100)
  return $result
}

# Whether the bee plugin was recorded as installed for <RuntimeName> in a
# { claude, codex } state json blob. Tolerant of a bare array, a wrapped
# {plugins|items|data} shape, or a single object; unparseable/missing data
# reads as "not installed" (fail-safe for rollback comparisons). Mirrors
# install.sh's plugin_was_installed.
function Test-PluginWasInstalled([string]$RuntimeName, [string]$StateJsonPath) {
  if (-not (Test-Path $StateJsonPath)) { return $false }
  try {
    $state = Get-Content $StateJsonPath -Raw | ConvertFrom-Json
    $list = $state.$RuntimeName
    if (-not $list) { return $false }
    $items = @($list)
    if ($items.Count -eq 1 -and $items[0].PSObject.Properties.Name -contains 'plugins') { $items = @($items[0].plugins) }
    elseif ($items.Count -eq 1 -and $items[0].PSObject.Properties.Name -contains 'items') { $items = @($items[0].items) }
    elseif ($items.Count -eq 1 -and $items[0].PSObject.Properties.Name -contains 'data') { $items = @($items[0].data) }
    foreach ($item in $items) {
      $name = $null
      if ($item.name) { $name = $item.name }
      elseif ($item.id) { $name = $item.id }
      elseif ($item.plugin -and $item.plugin.name) { $name = $item.plugin.name }
      if ($name -eq 'bee' -or ([string]$name).StartsWith('bee@')) {
        $statusRaw = $item.status
        if (-not $statusRaw) { $statusRaw = $item.state }
        if (-not $statusRaw) { $statusRaw = '' }
        $statusText = ([string]$statusRaw).ToLowerInvariant()
        if ($item.installed -eq $true) { return $true }
        return -not (@('removed', 'not_installed') -contains $statusText)
      }
    }
  } catch { return $false }
  return $false
}

# D8 point 3: mutate the plugin ONLY after the confirmation gate has been
# passed (called post-confirm by the caller below). plugin-first installs
# the plugin package; repo-copy removes it (best-effort - a pre-run absence
# means nothing to remove). Fixture runs (-PluginStateFile) skip every real
# CLI transition entirely, mirroring install.sh's plugin-state-file early
# return in transition_plugin.
function Invoke-PluginTransition {
  if ($PluginStateFile) { return $true }
  foreach ($rt in @('codex', 'claude')) {
    if ($Runtime -notin @($rt, 'both')) { continue }
    $cmd = Get-Command $rt -ErrorAction SilentlyContinue
    if (-not $cmd) {
      if ($Distribution -eq 'plugin-first') { Fail "$rt CLI is required for plugin-first" }
      continue
    }
    if ($Distribution -eq 'plugin-first') {
      # Mutation verbs take NO --json (only `plugin list` does); the real
      # CLI rejects `--json` here with `error: unknown option '--json'`.
      & $rt plugin marketplace add $beeSrc | Out-Null
      if ($LASTEXITCODE -ne 0) { return $false }
      if ($rt -eq 'codex') { & codex plugin add 'bee@bee' | Out-Null } else { & claude plugin install 'bee@bee' | Out-Null }
      if ($LASTEXITCODE -ne 0) { return $false }
    } else {
      $preProbe = Invoke-PluginListProbe $rt
      if ($preProbe.ExitCode -eq 0 -and ($preProbe.StdOut -join '') -match 'bee@bee') {
        if ($rt -eq 'codex') { & codex plugin remove 'bee@bee' | Out-Null } else { & claude plugin uninstall 'bee@bee' | Out-Null }
      }
    }
  }
  return $true
}

# D8 point 4: restore every active runtime to its exact pre-run installed
# state. Honest rollback: re-probes the CURRENT state and only acts where it
# genuinely differs from the snapshot taken before the transition ran - a
# transition that died before mutating anything (e.g. at `marketplace add`)
# leaves current == pre-run, so rollback is correctly a no-op success, never
# removing a never-installed plugin. Mirrors install.sh's rollback_plugin.
function Invoke-PluginRollback {
  if ($PluginStateFile) { return $true }
  $ok = $true
  $rollbackDir = Join-Path $stateTempDir ("rollback-" + [Guid]::NewGuid().ToString('N'))
  New-Item -ItemType Directory -Force -Path $rollbackDir | Out-Null
  $nowStateFile = New-PluginStateFile $rollbackDir
  foreach ($rt in @('codex', 'claude')) {
    if ($Runtime -notin @($rt, 'both')) { continue }
    $cmd = Get-Command $rt -ErrorAction SilentlyContinue
    if (-not $cmd) { continue }
    $was = Test-PluginWasInstalled $rt $preStateFile
    $now = Test-PluginWasInstalled $rt $nowStateFile
    if ($was -eq $now) { continue }
    if ($was) {
      # pre-run had the plugin, the transition removed it: re-install.
      & $rt plugin marketplace add $beeSrc | Out-Null
      if ($LASTEXITCODE -ne 0) { $ok = $false }
      if ($rt -eq 'codex') { & codex plugin add 'bee@bee' | Out-Null } else { & claude plugin install 'bee@bee' | Out-Null }
      if ($LASTEXITCODE -ne 0) { $ok = $false }
    } else {
      # pre-run lacked the plugin, the transition installed it: remove it.
      if ($rt -eq 'codex') { & codex plugin remove 'bee@bee' | Out-Null } else { & claude plugin uninstall 'bee@bee' | Out-Null }
      if ($LASTEXITCODE -ne 0) { $ok = $false }
    }
  }
  return $ok
}

# A post-transition (or post-preflight, or post-apply) failure: roll the
# plugin state back to the pre-run snapshot, leave the target untouched,
# report BOTH the primary failure and any rollback failure, and exit
# nonzero - never convert a failed install into success. Uses
# $host.UI.WriteErrorLine (not Write-Error) so the message prints to stderr
# WITHOUT tripping $ErrorActionPreference = 'Stop' into a terminating error
# that would skip the rollback call below. Mirrors install.sh's
# handle_transition_failure.
function Invoke-PluginTransitionFailure([string]$Message) {
  $host.UI.WriteErrorLine("Error: $Message")
  if (Invoke-PluginRollback) {
    $host.UI.WriteErrorLine('rollback: pre-run plugin state restored; target left unchanged')
  } else {
    $host.UI.WriteErrorLine('Error: rollback failed to fully restore the pre-run plugin state')
  }
  exit 1
}

# ---------- prerequisites ----------

# bee is a single native binary. It used to be compiled per machine from the
# source checkout (decision 1f4262ca); the release workflow now publishes one
# per platform, so a Rust toolchain is needed only when no published binary
# fits this host - or when the caller asks for a build outright. The check
# therefore lives next to the build it guards, not up here.

# ...but INSTALLING bee still needs node, and that is a different question from
# running it. The R6 sweep removed the Node preflight along with the runtime;
# plugin_distribution.mjs was never ported, and this script invokes it below.
# Without this check the failure lands as a bare "node is not recognized" AFTER
# a clone and a multi-minute cargo build.
$nodeCmd = Get-Command node -ErrorAction SilentlyContinue
if (-not $nodeCmd) { Fail 'Node.js is required to INSTALL bee (the distribution helper packages/bee/scripts/plugin_distribution.mjs is not ported yet). bee itself runs as a native binary and needs no Node at runtime. Install Node 18+: https://nodejs.org' }

# ---------- published binary (preferred) ----------
#
# Non-fatal by design: any failure logs its reason, leaves $beeBin unset, and
# the source build takes over. An installer that dies because a download
# blipped would be worse than the build it replaces.
$releasesBase = 'https://github.com/thanhsmind/beehive/releases'
$prebuiltAsset = 'bee-x86_64-pc-windows-msvc.exe'
$beeBin = $null
$prebuiltTag = $null
$prebuiltDir = $null
$scriptDirEarly = Split-Path -Parent $PSCommandPath

if ($BuildFromSource) {
  Write-Host 'binary   -BuildFromSource given; skipping the published binary'
} elseif ($Source) {
  Write-Host 'binary   -Source given; building that checkout rather than downloading'
} elseif ($scriptDirEarly -and (Test-Path (Join-Path $scriptDirEarly '..\packages\bee-rs\Cargo.toml'))) {
  # Running from inside a checkout. Pairing a downloaded binary with THIS
  # tree's skills is the version skew the design exists to avoid.
  Write-Host 'binary   running inside a bee checkout; building it rather than downloading'
} else {
  try {
    # A tag ref installs THAT release; anything else takes the latest. The
    # source tree is pinned to the same tag below, so the binary and the
    # instruction layer it vendors always come from one commit.
    if ($Ref -match '^v[0-9]') {
      $prebuiltTag = $Ref
    } else {
      $resp = Invoke-WebRequest -UseBasicParsing -Uri "$releasesBase/latest" -MaximumRedirection 5
      $final = $null
      if ($resp.BaseResponse.ResponseUri) { $final = $resp.BaseResponse.ResponseUri.AbsoluteUri }
      elseif ($resp.BaseResponse.RequestMessage) { $final = $resp.BaseResponse.RequestMessage.RequestUri.AbsoluteUri }
      if ($final -and $final -match '/releases/tag/(v[^/]+)$') { $prebuiltTag = $Matches[1] }
    }
  } catch { $prebuiltTag = $null }

  if (-not $prebuiltTag) {
    Write-Host 'binary   could not resolve a published release - building from source'
  } else {
    try {
      $prebuiltDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName())
      New-Item -ItemType Directory -Force -Path $prebuiltDir | Out-Null
      $binPath = Join-Path $prebuiltDir $prebuiltAsset
      $sumPath = Join-Path $prebuiltDir 'SHA256SUMS'
      Invoke-WebRequest -UseBasicParsing -Uri "$releasesBase/download/$prebuiltTag/$prebuiltAsset" -OutFile $binPath
      Invoke-WebRequest -UseBasicParsing -Uri "$releasesBase/download/$prebuiltTag/SHA256SUMS" -OutFile $sumPath
      # Verified, never trusted: this binary is copied into the target repo and
      # then executed by every hook.
      $wantLine = Get-Content $sumPath | Where-Object { $_.EndsWith($prebuiltAsset) } | Select-Object -First 1
      $want = $null
      if ($wantLine) { $want = ($wantLine -split '\s+')[0] }
      $got = (Get-FileHash -Algorithm SHA256 $binPath).Hash.ToLower()
      if ($want -and $got -eq $want.ToLower()) {
        $beeBin = $binPath
        Write-Host "binary   $prebuiltTag $prebuiltAsset (checksum verified) - no build needed"
      } else {
        Write-Host "binary   CHECKSUM MISMATCH for $prebuiltAsset at $prebuiltTag - refusing it, building from source"
        Remove-Item -Recurse -Force $prebuiltDir -ErrorAction SilentlyContinue
      }
    } catch {
      Write-Host "binary   no downloadable asset at $prebuiltTag - building from source"
      if ($prebuiltDir) { Remove-Item -Recurse -Force $prebuiltDir -ErrorAction SilentlyContinue }
    }
  }
}

# ---------- resolve bee source (local checkout or clone) ----------

$cleanupDir = $null
$stateTempDir = $null
try {
  if ($Source) {
    # .ProviderPath, not .Path: on UNC paths PS 5.1 returns a provider-qualified
    # string (Microsoft.PowerShell.Core\FileSystem::\\...) that node cannot open.
    $beeSrc = (Resolve-Path $Source).ProviderPath
  # R6 CUTOVER: this probe keyed on packages\bee\scripts\onboard_bee.mjs, which
  # is deleted. Left alone it answers "no" for every local checkout and
  # RE-CLONES from GitHub instead of installing the tree in front of you - the
  # quietest possible wrong answer. The marker is now the Rust crate manifest:
  # it is what the build step below compiles, so a yes here means a build can
  # actually run.
  } elseif ($PSScriptRoot -and (Test-Path (Join-Path $PSScriptRoot '..\packages\bee-rs\Cargo.toml'))) {
    $beeSrc = (Resolve-Path (Join-Path $PSScriptRoot '..')).ProviderPath
  } else {
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
      Fail 'git is required to fetch bee (or pass -Source <local-checkout>).'
    }
    $cleanupDir = Join-Path ([IO.Path]::GetTempPath()) ("bee-install-" + [Guid]::NewGuid().ToString('N').Substring(0, 8))
    Write-Host "fetch    $RepoUrl (ref: $Ref)"
    $clonePath = Join-Path $cleanupDir 'bee'

    # Check out only the trees the installer reads. Windows rejects : * ? " < > | in
    # filenames, so a full checkout of any ref carrying such a path aborts the working
    # tree ("invalid path") and leaves an empty clone that fails much later. A sparse
    # checkout keeps the install working on every ref, historical tags included.
    # (Keep this file pure ASCII: the onboard test enforces it.)
    # No 2> redirection anywhere below: under Windows PowerShell 5.1 with
    # $ErrorActionPreference = 'Stop', redirecting a native command's stderr turns its
    # warnings into terminating NativeCommandErrors. Exit codes and the probe decide.
    # Pinned to the binary's own tag when one was downloaded.
    $cloneRef = if ($prebuiltTag) { $prebuiltTag } else { $Ref }
    git clone --quiet --depth 1 --branch $cloneRef --no-checkout $RepoUrl $clonePath
    if ($LASTEXITCODE -ne 0) { Fail 'Clone failed. Check network access to github.com/thanhsmind/beehive.' }

    # sparse-checkout needs git 2.25+; on older git it exits non-zero and the checkout
    # below is simply a full one (which may still trip over an invalid path; the probe
    # is what turns that into an honest error instead of a silent empty source).
    git -C $clonePath sparse-checkout set skills packages .claude-plugin .codex-plugin docs/history/codex-harness-hardening
    git -C $clonePath checkout --quiet HEAD

    $beeSrc = $clonePath
    # R6 CUTOVER: same marker move as the probe above. Keyed on the deleted
    # onboard_bee.mjs this became a HARD Fail on every clone-path install, with
    # a message blaming the user's git version for a file that no longer exists.
    if (-not (Test-Path (Join-Path $beeSrc 'packages\bee-rs\Cargo.toml'))) {
      Fail "Checkout of $RepoUrl (ref: $Ref) produced no packages/ tree. Update git to 2.25+ (sparse checkout), or pass -Source <local-checkout>."
    }
  }

  # Build the binary from the resolved source checkout. This IS the install:
  # the repo ships no binary, so every host compiles its own once.
  $cargoToml = Join-Path $beeSrc 'packages\bee-rs\Cargo.toml'
  if (-not (Test-Path $cargoToml)) { Fail "Not a bee checkout (missing packages/bee-rs/Cargo.toml): $beeSrc" }
  if (-not $beeBin) {
    $cargoCmd = Get-Command cargo -ErrorAction SilentlyContinue
    if (-not $cargoCmd) { Fail 'No published binary was usable for this host and cargo is not on PATH. Install rustup (https://rustup.rs), or re-run where a release asset exists.' }
    Write-Host 'build    cargo build --release (packages/bee-rs) - first build takes a few minutes'
    cargo build --release --manifest-path $cargoToml
    if ($LASTEXITCODE -ne 0) { Fail 'cargo build --release failed. Fix the build, then re-run the installer.' }
    $beeBin = Join-Path $beeSrc 'packages\bee-rs\target\release\bee.exe'
    if (-not (Test-Path $beeBin)) { $beeBin = Join-Path $beeSrc 'packages\bee-rs\target\release\bee' }
    if (-not (Test-Path $beeBin)) { Fail 'cargo build produced no binary at packages/bee-rs/target/release/bee[.exe]' }
  }
  $distributionHelper = Join-Path $beeSrc 'packages\bee\scripts\plugin_distribution.mjs'
  $releaseManifest = Join-Path $beeSrc 'docs\history\codex-harness-hardening\release-manifest.json'
  if (-not (Test-Path $distributionHelper)) { Fail "Not a bee release (missing plugin_distribution.mjs): $beeSrc" }
  if (-not (Test-Path $releaseManifest)) { Fail "Not a bee release (missing release manifest): $beeSrc" }
  $beeVersion = try {
    (Get-Content (Join-Path $beeSrc '.claude-plugin\plugin.json') -Raw | ConvertFrom-Json).version
  } catch { 'unknown' }
  Write-Host "source   $beeSrc (bee $beeVersion)"

  # Basename-only replacement of user/global roots is forbidden. The shared
  # planner consumes an exact ledger before any such cleanup.
  if ($GlobalSkills -and -not $OwnershipLedger) {
    Fail '-GlobalSkills requires -OwnershipLedger; basename-only global replacement is refused'
  }

  # ---------- layer 2: target repo (greenfield / brownfield) ----------

  if (-not (Test-Path $Directory)) {
    if ($DryRun) {
      Write-Host "would create  $Directory (greenfield)"
    } else {
      if (-not (Confirm-Step "Target $Directory does not exist. Create it (greenfield)?")) { Fail 'Aborted.' }
      New-Item -ItemType Directory -Force -Path $Directory | Out-Null
    }
  }
  if (Test-Path $Directory) { $Directory = (Resolve-Path $Directory).ProviderPath }

  $mode = 'brownfield'
  if (-not (Test-Path (Join-Path $Directory '.git'))) {
    $mode = 'greenfield'
    if (-not $NoGitInit) {
      if ($DryRun) {
        Write-Host "would run     git init ($Directory is not a git repo)"
      } elseif ((Get-Command git -ErrorAction SilentlyContinue) -and (Confirm-Step "No git repo at $Directory. Run git init?")) {
        git -C $Directory init --quiet
      }
    }
  } elseif (Test-Path (Join-Path $Directory '.bee\onboarding.json')) {
    $mode = 'brownfield (bee already onboarded - refresh)'
  } elseif ((Test-Path (Join-Path $Directory 'AGENTS.md')) -or (Test-Path (Join-Path $Directory 'CLAUDE.md'))) {
    $mode = 'brownfield (existing agent docs - BEE block will be merged, nothing outside markers touched)'
  }
  Write-Host "target   $Directory [$mode]"

  $onboardFlags = @()
  # Thread -Runtime through to onboard_bee.mjs on both branches: it's the flag
  # that gates the codex-hybrid write (pluginSource && runtimeCoversCodex(runtime)
  # in onboard_bee.mjs computePlan/applyPlan) so plugin-first + -Runtime codex/both
  # actually reaches the hook-write path this installer's --codex-hybrid cleanup
  # scoping ($distributionArgs above) assumes is active. repo-copy passes it too
  # for symmetry - onboard_bee.mjs's skill-sync targets are runtime-independent
  # today, so this is currently a no-op there, but it keeps both branches honest
  # about which runtime the installer was asked for instead of always defaulting
  # to onboard_bee.mjs's own "both".
  $onboardFlags += @('--runtime', $Runtime)
  if ($Distribution -eq 'plugin-first') { $onboardFlags += '--plugin-source' }
  elseif (-not $NoHooks) { $onboardFlags += '--repo-hooks' }
  if ($NoClaudeMd) { $onboardFlags += '--no-claude-md' }
  if ($ClaudeMd) { $onboardFlags += '--claude-md' }
  if ($GlobalSkills) { $onboardFlags += '--global-skills' }

  if ($PluginStateFile -and -not (Test-Path $PluginStateFile)) { Fail "-PluginStateFile not found: $PluginStateFile" }
  $stateTempDir = Join-Path ([IO.Path]::GetTempPath()) ("bee-plugin-state-" + [Guid]::NewGuid().ToString('N'))
  New-Item -ItemType Directory -Force -Path $stateTempDir | Out-Null

  # 1. read-only probe of the CURRENT plugin state (pre-confirmation, no
  #    mutation) - install.sh D8 order, mirrored here.
  $stateFile = New-PluginStateFile $stateTempDir
  $preStateFile = Join-Path $stateTempDir 'pre-state.json'
  Copy-Item $stateFile $preStateFile -Force

  Write-Host "plan     bee onboard $($onboardFlags -join ' ') (dry-run first)"
  # CWD IS LOAD-BEARING. `bee onboard` vendors FROM a bee source checkout and
  # finds one by walking up from the binary and from the cwd. On the published
  # path those are a temp dir and the TARGET repo, neither of which is a
  # checkout, so onboarding refused every run. Running it from $beeSrc is what
  # puts the clone in view.
  Push-Location $beeSrc
  try { & $beeBin onboard --repo-root $Directory @onboardFlags } finally { Pop-Location }
  if ($LASTEXITCODE -ne 0) { Fail 'Onboarding plan failed.' }

  if ($DryRun) {
    Write-Host 'dry-run  nothing written. Re-run without -DryRun to apply.'
    exit 0
  }

  # 2. single confirmation gate covers BOTH the plugin transition and the
  #    onboarding apply below - nothing above this point mutates a plugin,
  #    target, or home.
  if (-not (Confirm-Step "Apply this onboarding plan to $Directory?")) { Fail 'Aborted - nothing applied.' }

  # VENDOR THE BINARY BEFORE ONBOARDING APPLIES, not after.
  #
  # Hook wiring is FEATURE-DETECTED: onboarding writes binary-shaped hook
  # commands when it finds .bee/bin/bee[.exe] in the target. Copying it in
  # after the apply meant the first apply always wired for a binary that was
  # not there yet, so a fresh install never converged in one pass while a
  # second run did. Ordering, not logic.
  #
  # The vendored NAME is a contract too: hooks, AGENTS.md and the skills all
  # name .bee/bin/bee.exe. A downloaded release asset is called
  # bee-x86_64-pc-windows-msvc.exe, so copying it under its own leaf name
  # would leave every hook pointing at a file that is not there - with the
  # installer still reporting success, because it verifies through that same
  # wrong path.
  $hostBinDir = Join-Path $Directory '.bee\bin'
  if (-not (Test-Path $hostBinDir)) { New-Item -ItemType Directory -Force $hostBinDir | Out-Null }
  $hostBeeName = if ($beeBin -like '*.exe') { 'bee.exe' } else { 'bee' }
  Copy-Item $beeBin (Join-Path $hostBinDir $hostBeeName) -Force
  $hostBee = Join-Path $hostBinDir $hostBeeName

  # 3. mutate the plugin ONLY now, after confirmation - the pre-run snapshot
  #    recorded above ($preStateFile) is what a later failure rolls back to.
  if (-not (Invoke-PluginTransition)) { Invoke-PluginTransitionFailure 'Plugin transition failed' }

  # Re-probe post-transition: the distribution helper below must see the
  # plugin's ACTUAL current state (installed for plugin-first, removed for
  # repo-copy) to prove/clean correctly - matches install.sh's second
  # probe_plugin_state call immediately before its own $DIST_HELPER call.
  $stateFile = New-PluginStateFile $stateTempDir

  $distributionArgs = @('--mode', $Distribution, '--runtime', $Runtime, '--repo-root', $Directory, '--release-manifest', $releaseManifest, '--plugin-state-file', $stateFile)
  # GH #22 P0-1 (cph-1 self-erasure fix): a plugin-first install whose runtime
  # scope covers codex gets the codex-hybrid .codex/hooks.json + .bee/bin/hooks/
  # write from onboard_bee.mjs, gated by its own --runtime (now threaded
  # through via $onboardFlags above, so onboard_bee.mjs's codexHybrid
  # computation sees the SAME $Runtime this installer resolved -Runtime
  # codex/both to). Without --codex-hybrid here, the distribution helper's
  # cleanup pass below would immediately strip the very hook entries onboarding
  # just wrote, right back to zero mechanical enforcement for Codex sessions.
  if ($Distribution -eq 'plugin-first' -and $Runtime -in @('codex', 'both')) {
    $distributionArgs += @('--codex-hybrid')
  }
  if ($OwnershipLedger) { $distributionArgs += @('--ledger', $OwnershipLedger) }
  if ($GlobalSkills) {
    $claudeHome = if ($env:CLAUDE_HOME) { $env:CLAUDE_HOME } else { Join-Path $HOME '.claude' }
    $codexHome = if ($env:CODEX_HOME) { $env:CODEX_HOME } else { Join-Path $HOME '.codex' }
    if ($Runtime -in @('claude', 'both')) { $distributionArgs += @('--user-skill-root', (Join-Path $claudeHome 'skills')) }
    if ($Runtime -in @('codex', 'both')) { $distributionArgs += @('--user-skill-root', (Join-Path $codexHome 'skills')) }
  }

  node $distributionHelper @distributionArgs
  if ($LASTEXITCODE -ne 0) { Invoke-PluginTransitionFailure 'Distribution preflight refused after transition' }

  # 4. apply onboarding. A refused/blocked apply (e.g. the codex-hybrid hook
  #    write preflight in onboard_bee.mjs applyPlan refusing because
  #    .codex/hooks.json or .bee/bin/hooks/ can't be written) names the
  #    concrete way out below, then rolls the plugin transition back.
  Push-Location $beeSrc
  try { $applyOutput = & $beeBin onboard --repo-root $Directory --apply @onboardFlags } finally { Pop-Location }
  if ($LASTEXITCODE -ne 0) {
    Write-Host ($applyOutput -join "`n")
    Write-Host '  fix options:'
    Write-Host '    - re-run with -Distribution repo-copy (no codex-hybrid hook write required)'
    Write-Host '    - clear the obstacle blocking the write (see reason above) and re-run -Distribution plugin-first'
    Invoke-PluginTransitionFailure 'Onboarding apply failed'
  }

  if ($Distribution -eq 'plugin-first') {
    node $distributionHelper @distributionArgs --apply
    if ($LASTEXITCODE -ne 0) { Invoke-PluginTransitionFailure 'Plugin-first cleanup refused; repository fallbacks were preserved' }
  }

  # ---------- verify ----------

  Push-Location $Directory
  try {
    $statusJson = & $hostBee status --json
    if ($LASTEXITCODE -ne 0) { Fail 'Verification failed: bee status did not run.' }
    # Join first: PS 5.1 pipes a multi-line array into ConvertFrom-Json line by line.
    $status = ($statusJson -join "`n") | ConvertFrom-Json
    if (-not $status.onboarding -or $status.onboarding.installed -ne $true) {
      Fail 'Verification failed: bee status reports not installed.'
    }
    $expectedVersion = (Get-Content (Join-Path $beeSrc '.claude-plugin\plugin.json') -Raw | ConvertFrom-Json).version
    if ($status.onboarding.bee_version -ne $expectedVersion -or
        $status.onboarding.plugin_version -ne $expectedVersion -or
        $status.onboarding.drift -ne $false) {
      Fail "Verification failed: version parity mismatch (expected $expectedVersion; bee $($status.onboarding.bee_version), plugin $($status.onboarding.plugin_version), drift $($status.onboarding.drift))."
    }
    Write-Host "verify   onboarding ok (bee $($status.onboarding.bee_version)), phase: $($status.phase)"
  } finally {
    Pop-Location
  }

  Write-Host ''
  Write-Host 'bee installed.'
  Write-Host "  next: open an agent session in $Directory"
  Write-Host '  - Claude Code: the session preamble appears via hooks; or say "Route this through bee: <task>"'
  Write-Host '  - Codex: the AGENTS.md BEE block bootstraps; first step is bee status'
  Write-Host '  - scout any time: .bee/bin/bee status --json'
} finally {
  if ($cleanupDir -and (Test-Path $cleanupDir)) {
    Remove-Item -Recurse -Force $cleanupDir -ErrorAction SilentlyContinue
  }
  if ($stateTempDir -and (Test-Path $stateTempDir)) {
    Remove-Item -Recurse -Force $stateTempDir -ErrorAction SilentlyContinue
  }
}
