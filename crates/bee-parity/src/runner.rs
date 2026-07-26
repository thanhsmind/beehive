//! Spawns the real `bee.mjs status --json` against a given root and
//! captures a [`crate::differ::RunResult`]. One shared spawn path so the
//! root-safety sanity read and the actual diffed run never drift apart —
//! same command, same capture shape, every time (CONTEXT.md D7a: "run the
//! same command through node .bee/bin/bee.mjs ... in one and ... in the
//! other").

use std::path::Path;
use std::process::Command;

use crate::differ::RunResult;

pub fn run_bee_status(bee_mjs: &Path, root: &Path) -> Result<RunResult, String> {
    let output = Command::new("node")
        .arg(bee_mjs)
        .arg("status")
        .arg("--json")
        .current_dir(root)
        .output()
        .map_err(|e| format!("failed to spawn `node {} status --json` with cwd {}: {e}", bee_mjs.display(), root.display()))?;

    Ok(RunResult {
        root: root.to_path_buf(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}
