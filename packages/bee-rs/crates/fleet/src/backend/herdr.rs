//! The herdr implementation of `WorkerBackend` (D16 — `64e8abe6`). Shells
//! out to the real `herdr` CLI with `std::process::Command`, argv-based,
//! stdin always null — the pattern
//! `packages/bee-rs/crates/bee/src/verbs/worktree/git.rs:86-91` already
//! established for `git`. No shell string is ever built.
//!
//! **Authorities, in order.** `skills/bee-herding/references/spawn-proof.md`
//! is a recording of a real herdr 0.8.0 round trip and is authoritative
//! over any description below it disagrees with. `herdr --skill` (not
//! available to this worker; not run) is the second authority. The
//! per-verb `--help` is the third. Where this module infers a JSON shape
//! spawn-proof.md does not show byte-for-byte (`agent list`'s per-agent
//! object, `agent read`'s response field), that inference is called out at
//! the parsing site — the parser is written to fail closed
//! (`WorkerStatus::Unverifiable` / an `Err`) on anything that does not
//! match, never to panic, so a wrong guess degrades safely instead of
//! silently.
//!
//! **Scope boundary — what `start` does NOT do.** `WorkerSpec` (the
//! generic core's own type, D2) carries only a `name` and a `task` — no
//! worktree cwd, no agent kind, nothing herdr's `pane split` /
//! `agent start --kind` sequence needs to create a FRESH pane from
//! nothing. Assembling that (splitting a pane into a worktree, picking a
//! kind from `herding.agent_command`) is the bee-side wave entry point's
//! job — herding-orchestration D17 (`8d413c12`), a new CLI verb, not yet
//! built and explicitly not this cell's scope. This backend's contract is
//! therefore: **`WorkerSpec::name`, when addressed through `HerdrBackend`,
//! is a herdr pane id** (herdr's own `<workspace>:p<N>` shape, e.g.
//! `"w4:pB"` — every example in spawn-proof.md has this shape) that the
//! caller has already split into place, OR a friendly agent name already
//! known to herdr (resolved via `agent list`) for referencing an existing
//! running worker. `start` on a pane id already running an agent is an
//! idempotent success; on a pane id with no agent yet, it calls
//! `agent start` for real; on anything else (a name `agent list` has never
//! seen, not pane-id-shaped) it refuses rather than fabricate a spawn it
//! has no cwd/kind to perform.
//!
//! **The status mapping is the hard part (D7).** herdr's five states map
//! onto the trait's five 1:1 by NAME (`idle`→`Ready`, `working`→`Working`,
//! `blocked`→`Blocked`, `done`→`Finished`, `unknown`→`Unverifiable` — the
//! binary's own skill states `unknown` "is not proof of completion",
//! which is exactly `Unverifiable`'s contract). Every failure of the
//! LOOKUP itself — a non-zero exit, an unparseable body, a missing
//! `agent_status` key, a null field, an off-enum string value, the target
//! simply absent from `agent list` — becomes `Unverifiable` too, never a
//! safe default (Ordering Invariant 4). See `map_status` and `status`
//! below.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde_json::Value;

use super::{WorkerBackend, WorkerStatus};
use crate::wave::WorkerSpec;

/// How many transcript lines `read_output` asks herdr for. Generous
/// enough that a normal reply is never truncated by this number alone —
/// the hazard this module actually defends against (the alternate-screen
/// read failure, see `read_output`) is a herdr-side loss `--lines` cannot
/// recover from at ANY size, which is why the spill-file fallback exists
/// rather than simply raising this constant further.
const READ_LINES: &str = "500";

/// The wall-clock ceiling passed to `agent start`'s own readiness gate.
/// Matches the value observed live in spawn-proof.md Step 3
/// (`--timeout 60000`).
const START_TIMEOUT_MS: &str = "60000";

/// A backend driving the real `herdr` CLI. Every method shells out with
/// `std::process::Command`, argv-based, stdin null — never a shell
/// string.
#[derive(Debug)]
pub struct HerdrBackend {
    /// Directories searched AHEAD of the inherited `PATH` for the `herdr`
    /// executable. Empty in production (`new`), so the real installed
    /// `herdr` resolves however the host's own `PATH` says it should —
    /// this field only exists for the PATH-prepended stub binary tests
    /// use (the precedent `packages/bee-rs/crates/bee/src/shell.rs:74-89`
    /// established for a Windows Git-Bash resolver, applied here on every
    /// platform: a stub substitution has none of that resolver's
    /// WSL-vs-Win32 ambiguity to resolve, so no OS gate is needed).
    path_prepend: Vec<PathBuf>,
    /// Where `send`/`read_output` spill a reply too long for the
    /// alternate-screen read window (see `read_output`). Defaults to the
    /// OS temp directory in production; tests point this at a scratch
    /// directory they control.
    spill_dir: PathBuf,
}

impl Default for HerdrBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl HerdrBackend {
    /// The production backend: `herdr` resolves off the host's own
    /// `PATH`, and long replies spill to the OS temp directory.
    pub fn new() -> Self {
        Self {
            path_prepend: Vec::new(),
            spill_dir: std::env::temp_dir(),
        }
    }

    /// A backend wired for testing: `dirs` is searched ahead of the
    /// inherited `PATH` (the PATH-prepended stub binary seam), and long
    /// replies spill into `spill_dir` instead of the shared OS temp
    /// directory, so parallel tests never collide. The crate's test suite
    /// runs with no real `herdr` on `PATH` at all (D7); every test using
    /// this constructor supplies its own stub.
    pub fn with_test_seams(dirs: Vec<PathBuf>, spill_dir: PathBuf) -> Self {
        Self {
            path_prepend: dirs,
            spill_dir,
        }
    }

    /// The child `PATH` a spawned `herdr` command should see: every
    /// `path_prepend` directory, in order, ahead of the inherited `PATH`.
    /// `None` when there is nothing to prepend, so production spawns
    /// leave `PATH` exactly as the process inherited it.
    fn child_path(&self) -> Option<OsString> {
        if self.path_prepend.is_empty() {
            return None;
        }
        let sep = if cfg!(windows) { ';' } else { ':' };
        let mut out = OsString::new();
        for (index, dir) in self.path_prepend.iter().enumerate() {
            if index > 0 {
                out.push(sep.to_string());
            }
            out.push(dir);
        }
        if let Some(inherited) = std::env::var_os("PATH") {
            out.push(sep.to_string());
            out.push(inherited);
        }
        Some(out)
    }

    /// Runs `herdr <args>`, argv-based (never a shell string), stdin
    /// always null (worktree/git.rs:86-91's precedent) so a herdr build
    /// that unexpectedly prompts can never block this call. Parses the
    /// exit code and stdout per herdr's own documented error contract
    /// (`docs/history/research/herdr-orchestrator-distill.md`, "Docs"):
    /// server errors are JSON on stderr with exit 1, syntax errors exit
    /// 2 — either way, a non-zero exit is `NonZeroExit` here, never
    /// silently swallowed.
    fn run_herdr(&self, args: &[&str]) -> Result<Value, HerdrCallError> {
        let mut cmd = Command::new("herdr");
        cmd.args(args);
        cmd.stdin(Stdio::null());
        if let Some(path) = self.child_path() {
            cmd.env("PATH", path);
        }
        let output = cmd.output().map_err(HerdrCallError::Spawn)?;
        if !output.status.success() {
            return Err(HerdrCallError::NonZeroExit {
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        serde_json::from_str(&stdout).map_err(|_| HerdrCallError::UnparseableBody { raw: stdout })
    }

    /// Where a reply too long for the alternate-screen read window is
    /// spilled for `worker` (see `read_output`). Deterministic per worker
    /// so `send`'s spill instruction and `read_output`'s recovery check
    /// agree on the same path without any side channel between them.
    fn spill_path(&self, worker: &str) -> PathBuf {
        let safe: String = worker
            .chars()
            .map(|c| if c == ':' || c == '/' || c == '\\' { '_' } else { c })
            .collect();
        self.spill_dir.join(format!("bee-fleet-herdr-{safe}.reply"))
    }
}

/// What can go wrong calling `herdr` itself — distinct from what `herdr`
/// reports about a WORKER, which is `WorkerStatus`. Every variant here is
/// exactly one of the lookup-failure shapes Ordering Invariant 4 (D7)
/// requires `status` to fold into `Unverifiable`, and that `start`/`send`/
/// `read_output` fold into a named `Err` rather than a panic.
#[derive(Debug)]
enum HerdrCallError {
    /// The process never launched at all (herdr missing from `PATH`, for
    /// example).
    Spawn(std::io::Error),
    /// herdr exited non-zero. Per its own error contract, `stderr` is JSON
    /// on a normal server-error exit (1) and plain text on a syntax-error
    /// exit (2); this variant carries the raw bytes either way rather than
    /// assuming which.
    NonZeroExit { code: Option<i32>, stderr: String },
    /// herdr exited 0 but stdout was not valid JSON.
    UnparseableBody { raw: String },
}

impl std::fmt::Display for HerdrCallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HerdrCallError::Spawn(err) => write!(f, "could not spawn herdr: {err}"),
            HerdrCallError::NonZeroExit { code, stderr } => {
                let code_disp = code.map_or_else(|| "unknown".to_string(), |c| c.to_string());
                write!(f, "herdr exited {code_disp}: {stderr}")
            }
            HerdrCallError::UnparseableBody { raw } => {
                write!(f, "herdr returned a body that was not valid JSON: {raw}")
            }
        }
    }
}

/// True for herdr's own pane-id shape — every example in spawn-proof.md
/// (`"w4:pA"`, `"w4:pB"`, `"w4:tA"`, `"w4:t1"`) is `<workspace>:<slot>`.
/// A caller-chosen agent name (`"ho2-spawn-rt"`, `"testslug"` in the same
/// document) never contains a colon, so this is a structural, not a
/// guessed, distinction.
fn is_pane_id_shaped(id: &str) -> bool {
    id.contains(':')
}

/// Finds the `agent list` entry addressed by `id` — matching either its
/// `name` field or its `pane_id` field, since by the time any of `start`/
/// `status` is called, `id` may be either (a fresh, never-deduped call, or
/// a canonical id already resolved by `canonical_id`). Returns `None` on
/// any shape mismatch (missing `result`/`agents`, not an array, no
/// matching entry) rather than panicking — the fail-closed default this
/// whole module holds to.
fn find_agent_entry<'a>(body: &'a Value, id: &str) -> Option<&'a Value> {
    body.get("result")?
        .get("agents")?
        .as_array()?
        .iter()
        .find(|entry| {
            entry.get("name").and_then(Value::as_str) == Some(id)
                || entry.get("pane_id").and_then(Value::as_str) == Some(id)
        })
}

/// The one place herdr's five status strings become `WorkerStatus`.
/// `unknown` maps to `Unverifiable`, not to some other "safe" reading —
/// herdr's own skill states `unknown` "is not proof of completion", which
/// is `Unverifiable`'s exact contract, not `Working`'s or `Blocked`'s. Any
/// string outside these five — an upstream herdr version adding a sixth
/// state, or a body this module's inferred schema misparsed — also maps
/// to `Unverifiable` (Ordering Invariant 4, D7): this function has no
/// panic path and no silent-safe path.
fn map_status(raw: &str) -> WorkerStatus {
    match raw {
        "idle" => WorkerStatus::Ready,
        "working" => WorkerStatus::Working,
        "blocked" => WorkerStatus::Blocked,
        "done" => WorkerStatus::Finished,
        "unknown" => WorkerStatus::Unverifiable,
        _ => WorkerStatus::Unverifiable,
    }
}

impl WorkerBackend for HerdrBackend {
    fn canonical_id(&self, name: &str) -> String {
        // Already herdr's own canonical addressing form — no lookup
        // needed, and no risk of a stale `agent list` read disagreeing
        // with the identifier's own shape.
        if is_pane_id_shaped(name) {
            return name.to_string();
        }
        // `name` is a friendly agent name; resolve it to the pane id that
        // addresses the exact same target, so a wave referencing one
        // worker by both its name and its pane id collapses to one
        // (herding-orchestration D15 — `fb8a8628`). Any failure of this
        // lookup — herdr unreachable, an unparseable body, the name
        // simply absent from the list — resolves `name` to itself: the
        // safe no-collapse default the trait documents, never a guess.
        match self.run_herdr(&["agent", "list"]) {
            Ok(body) => find_agent_entry(&body, name)
                .and_then(|entry| entry.get("pane_id").and_then(Value::as_str))
                .map(str::to_string)
                .unwrap_or_else(|| name.to_string()),
            Err(_) => name.to_string(),
        }
    }

    fn start(&self, worker: &WorkerSpec) -> anyhow::Result<()> {
        let body = self
            .run_herdr(&["agent", "list"])
            .map_err(|e| anyhow::anyhow!("herdr agent list failed while resolving {}: {e}", worker.name))?;
        if find_agent_entry(&body, &worker.name).is_some() {
            // Already running and addressable under this exact identity —
            // idempotent success, matching the trait's documented job
            // ("makes it addressable"; it already is). This is also what
            // makes `start` safe to call on a target this same process
            // already started earlier in a prior wave.
            return Ok(());
        }
        if !is_pane_id_shaped(&worker.name) {
            anyhow::bail!(
                "cannot start {:?}: no existing herdr agent answers to that name, and it is not \
                 a pane id this backend can start a fresh agent into — splitting a pane needs a \
                 worktree cwd this generic WorkerSpec does not carry; that wiring is the \
                 bee-side wave entry point's job (herding-orchestration D17, not yet built), not \
                 this backend's",
                worker.name
            );
        }
        // A pane id with no agent registered yet: split-then-start's
        // second half (D12). The slug passed as `agent start`'s own NAME
        // argument is derived from the pane id — herdr's own examples
        // (spawn-proof.md) use caller-chosen slugs with no fixed
        // relationship to the pane, so any stable, herdr-legal string
        // works; sanitizing the pane id's colon keeps it recognizable in
        // `herdr agent list` output without guessing at a naming scheme
        // this trait has no other input to derive one from.
        let slug = worker.name.replace(':', "-");
        // OBSERVED LIVE (spawn-proof.md, Step 3, Takeaway 2) — a thing no
        // documentation states: unlike `pane split` and `tab create`,
        // `agent start` has NO `--no-focus` flag. Starting an agent here
        // unconditionally moves the workspace's own active-tab focus to
        // the new agent's tab; this backend has no way to suppress that
        // because herdr offers none. A caller that cares about its own
        // tab's focus must restore it explicitly afterward (`herdr tab
        // focus <own-tab>`) — this module does not do that on the
        // caller's behalf, because it has no way to know which tab was
        // the caller's own.
        self.run_herdr(&[
            "agent",
            "start",
            &slug,
            "--kind",
            "claude",
            "--pane",
            &worker.name,
            "--timeout",
            START_TIMEOUT_MS,
        ])
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!("herdr agent start {slug} --pane {} failed: {e}", worker.name))
    }

    fn status(&self, worker: &str) -> WorkerStatus {
        // Deliberately the ONLY herdr call this method makes, and
        // deliberately never a focus-changing one (no `agent start`, no
        // `pane current`/`tab focus`): `agent list` is a plain read.
        // OBSERVED LIVE (spawn-proof.md Step 4) — a status read must not
        // depend on the target pane having been focused: the second turn
        // in that round trip read back `agent_status: "done"` with
        // `focused: false`, with no focus command issued in between, and
        // this method's own `agent_status`-only read reproduces that
        // exact reading regardless of any `focused` field's value.
        let body = match self.run_herdr(&["agent", "list"]) {
            Ok(body) => body,
            // A non-zero exit, a spawn failure, or an unparseable body —
            // the lookup itself failed. Fail-closed: `Unverifiable`, never
            // a safe-looking default (Ordering Invariant 4, D7).
            Err(_) => return WorkerStatus::Unverifiable,
        };
        let entry = match find_agent_entry(&body, worker) {
            Some(entry) => entry,
            // The target is simply absent from the list — indistinguishable
            // from "the lookup did not find it", which is exactly as
            // unsafe as any other lookup failure.
            None => return WorkerStatus::Unverifiable,
        };
        match entry.get("agent_status").and_then(Value::as_str) {
            // A missing key or a null field both fall out of
            // `and_then(Value::as_str)` returning `None` here — folded
            // into the same fail-closed arm as an off-enum value, never
            // distinguished from it, because neither is safe.
            Some(raw) => map_status(raw),
            None => WorkerStatus::Unverifiable,
        }
    }

    fn send(&self, worker: &str, task: &str) -> anyhow::Result<()> {
        let spill = self.spill_path(worker);
        let wrapped = format!(
            "{task}\n\n(If this reply would be longer than a typical terminal screen, instead \
             write your FULL reply to the file {spill} and reply here with EXACTLY that file \
             path on its own line and nothing else.)",
            spill = spill.display()
        );
        // Deliberately never `--wait`. OBSERVED / DOCUMENTED
        // (docs/history/research/herdr-orchestrator-distill.md, "Risks":
        // "`agent prompt --wait` does not track turns... if the agent is
        // already working, that active turn's completion may match"): a
        // naive `--wait` call issued right after this wave's own
        // dispatch-time re-check (Ordering Invariant 3) can still race an
        // in-flight turn started by something outside this wave's
        // control, and report ITS completion as this send's. Submitting
        // without `--wait` and leaving completion detection entirely to
        // the choreography's own baseline-before-dispatch, marker-after
        // poll (`crate::choreography::wait_for_target`) sidesteps the
        // race structurally: a wrong-turn reply can never contain THIS
        // send's marker, so `CompletionSignal::confirmed_against` can
        // never mistake it for confirmation, and the choreography keeps
        // polling until the real reply lands.
        self.run_herdr(&["agent", "prompt", worker, &wrapped])
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("herdr agent prompt {worker} failed: {e}"))
    }

    fn read_output(&self, worker: &str) -> anyhow::Result<String> {
        let body = self
            .run_herdr(&["agent", "read", worker, "--source", "recent-unwrapped", "--lines", READ_LINES])
            .map_err(|e| anyhow::anyhow!("herdr agent read {worker} failed: {e}"))?;
        // INFERRED beyond spawn-proof.md, which shows the observed
        // transcript text but not `agent read`'s raw JSON field name:
        // this module accepts either `result.text` (a string field) or
        // `result` itself being a bare string, and refuses — rather than
        // guessing further — if neither shape matches.
        let transcript = body
            .get("result")
            .and_then(|r| {
                r.get("text")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| r.as_str().map(str::to_string))
            })
            .ok_or_else(|| anyhow::anyhow!("herdr agent read {worker} returned a body with no recognizable text field"))?;

        // DOCUMENTED (herdr-orchestrator-distill.md, "the alternate-screen
        // read failure"): `--lines` cannot recover rows that have already
        // left the alternate screen — no `--lines` value fixes this, only
        // a different source can. herdr's own documented fallback is to
        // have the agent write its full reply to a file and reply with
        // just that path; `send` above asks for exactly that when a reply
        // might be long. Recognize the hand-back here: if the transcript's
        // last non-blank line CONTAINS this worker's spill path AND that
        // file exists, prefer its content over the (possibly truncated)
        // transcript read. A substring check, not equality: the CLI's own
        // rendering prefixes a reply line with a marker (spawn-proof.md's
        // transcript shows every reply as `"● <text>"`), so the agent's
        // own bare-path reply arrives as `"● <path>"`, never the path
        // alone.
        let spill = self.spill_path(worker);
        let spill_str = spill.to_string_lossy();
        let last_line = transcript
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("")
            .trim();
        if last_line.contains(spill_str.as_ref()) {
            if let Ok(spilled) = std::fs::read_to_string(&spill) {
                return Ok(spilled);
            }
        }
        Ok(transcript)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_status_covers_all_five_herdr_states() {
        assert_eq!(map_status("idle"), WorkerStatus::Ready);
        assert_eq!(map_status("working"), WorkerStatus::Working);
        assert_eq!(map_status("blocked"), WorkerStatus::Blocked);
        assert_eq!(map_status("done"), WorkerStatus::Finished);
        // `unknown` is herdr's own "not proof of completion" state — it
        // must land on Unverifiable, not on some other reading that would
        // let a caller treat it as safe.
        assert_eq!(map_status("unknown"), WorkerStatus::Unverifiable);
    }

    #[test]
    fn map_status_treats_any_off_enum_string_as_unverifiable() {
        assert_eq!(map_status("busy"), WorkerStatus::Unverifiable);
        assert_eq!(map_status(""), WorkerStatus::Unverifiable);
        assert_eq!(map_status("IDLE"), WorkerStatus::Unverifiable);
    }

    #[test]
    fn pane_id_shape_is_structural_not_a_guess() {
        assert!(is_pane_id_shaped("w4:pB"));
        assert!(is_pane_id_shaped("w4:t1"));
        assert!(!is_pane_id_shaped("ho2-spawn-rt"));
        assert!(!is_pane_id_shaped("testslug"));
    }

    #[test]
    fn find_agent_entry_matches_by_name_or_pane_id_and_none_on_shape_mismatch() {
        let body: Value = serde_json::json!({
            "type": "ok",
            "result": {
                "agents": [
                    {"name": "reviewer-1", "pane_id": "w4:pB", "agent_status": "idle"}
                ]
            }
        });
        assert!(find_agent_entry(&body, "reviewer-1").is_some());
        assert!(find_agent_entry(&body, "w4:pB").is_some());
        assert!(find_agent_entry(&body, "nobody").is_none());

        let malformed: Value = serde_json::json!({"type": "ok"});
        assert!(find_agent_entry(&malformed, "reviewer-1").is_none());
    }
}
