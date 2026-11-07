use assert_cmd::Command;
use std::path::{Path, PathBuf};

const THRESHOLD: u64 = 8000;

fn fixture(bytes: Option<u64>) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();
    std::fs::create_dir_all(root.join(".bee")).unwrap();
    std::fs::write(
        root.join(".bee").join("onboarding.json"),
        "{\"version\":1,\"completed\":true}\n",
    )
    .unwrap();
    let config = match bytes {
        Some(n) => format!("{{\"{}\":{n}}}\n", "output_spill_bytes"),
        None => "{}\n".to_string(),
    };
    std::fs::write(root.join(".bee").join("config.json"), config).unwrap();
    (tmp, root)
}

fn bee(root: &Path) -> Command {
    let mut cmd = Command::cargo_bin("bee").unwrap();
    cmd.current_dir(root);
    cmd.env_remove("BEE_OUTPUT_FULL");
    cmd
}

fn help_all(root: &Path) -> std::process::Output {
    bee(root).args(["--help", "--all"]).output().unwrap()
}

fn full_help_bytes(root: &Path) -> Vec<u8> {
    let out = bee(root)
        .args(["--help", "--all"])
        .env("BEE_OUTPUT_FULL", "1")
        .output()
        .unwrap();
    assert!(out.status.success(), "the unwrapped run must succeed");
    assert!(
        out.stdout.len() as u64 > THRESHOLD,
        "the fixture must actually produce more than the threshold, got {} bytes",
        out.stdout.len()
    );
    out.stdout
}

fn spill_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("tmp").join("output")
}

fn spill_files(root: &Path) -> Vec<PathBuf> {
    match std::fs::read_dir(spill_dir(root)) {
        Ok(rd) => rd.map(|e| e.unwrap().path()).collect(),
        Err(_) => Vec::new(),
    }
}

#[cfg(unix)]
#[test]
fn oversized_allowlisted_output_spills_to_a_file() {
    let tmp = tempfile::tempdir().unwrap();
    let expected = full_help_bytes(tmp.path());
    let (_held, root) = fixture(Some(THRESHOLD));

    let out = help_all(&root);
    assert!(out.status.success(), "a spilled run keeps the verb's exit code");

    let stdout = String::from_utf8(out.stdout).unwrap();
    let marker_count = stdout.matches("[bee] output spilled:").count();
    assert_eq!(marker_count, 1, "exactly one marker line: {stdout}");

    let marker = stdout
        .lines()
        .find(|l| l.starts_with("[bee] output spilled:"))
        .expect("a marker line");
    assert!(
        marker.contains("bytes omitted — full output: .bee/tmp/output/"),
        "the marker names the omitted count and a repo-relative path: {marker}"
    );

    let without_marker = stdout.len() - marker.len() - 1;
    assert!(
        without_marker as u64 <= THRESHOLD,
        "the kept body must fit the threshold, got {without_marker}"
    );
    assert!(
        stdout.len() as u64 <= THRESHOLD + marker.len() as u64 + 1,
        "stdout is the body plus one marker line, got {} bytes",
        stdout.len()
    );

    let files = spill_files(&root);
    assert_eq!(files.len(), 1, "the spill file stays behind: {files:?}");
    let spilled = std::fs::read(&files[0]).unwrap();
    assert_eq!(
        spilled, expected,
        "the spill file holds the verb's complete output, byte for byte"
    );

    let head = stdout.lines().next().unwrap();
    assert!(
        String::from_utf8_lossy(&spilled).starts_with(head),
        "the printed head is the leading whole lines of the spilled output"
    );
}

#[cfg(not(unix))]
#[test]
fn oversized_allowlisted_output_passes_through_unchanged_without_a_spill() {
    let tmp = tempfile::tempdir().unwrap();
    let expected = full_help_bytes(tmp.path());
    let (_held, root) = fixture(Some(THRESHOLD));

    let out = help_all(&root);
    assert!(
        out.status.success(),
        "a pass-through run keeps the verb's exit code"
    );
    assert_eq!(
        out.stdout, expected,
        "without the spill capture the bytes are the unwrapped bytes"
    );
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains("[bee] output spilled:"),
        "no marker line is printed"
    );
    assert!(spill_files(&root).is_empty(), "no spill file is written");
}

#[test]
fn output_under_the_threshold_prints_unchanged_and_keeps_no_file() {
    let (_held, root) = fixture(Some(200000));
    let expected = full_help_bytes(&root);
    assert!(expected.len() as u64 <= 200000);

    let out = help_all(&root);
    assert!(out.status.success());
    assert_eq!(
        out.stdout, expected,
        "under the threshold the bytes are the unwrapped bytes, marker and all"
    );
    assert!(
        spill_files(&root).is_empty(),
        "an under-threshold run leaves no spill file"
    );
}

#[test]
fn no_config_key_means_no_spill() {
    let (_held, root) = fixture(None);
    let expected = full_help_bytes(&root);

    let out = help_all(&root);
    assert!(out.status.success());
    assert_eq!(out.stdout, expected, "absent key prints the full output");
    assert!(spill_files(&root).is_empty(), "no spill file is written");
}

#[test]
fn zero_threshold_means_no_spill() {
    let (_held, root) = fixture(Some(0));
    let expected = full_help_bytes(&root);

    let out = help_all(&root);
    assert!(out.status.success());
    assert_eq!(out.stdout, expected, "a 0 threshold disables the wrap");
    assert!(spill_files(&root).is_empty(), "no spill file is written");
}

#[test]
fn bee_output_full_prints_the_full_output_for_one_run() {
    let (_held, root) = fixture(Some(THRESHOLD));
    let expected = full_help_bytes(&root);

    let out = bee(&root)
        .args(["--help", "--all"])
        .env("BEE_OUTPUT_FULL", "1")
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(out.stdout, expected, "BEE_OUTPUT_FULL=1 bypasses the wrap");
    assert!(spill_files(&root).is_empty(), "no spill file is written");
}

#[test]
fn a_json_run_never_spills() {
    let (_held, root) = fixture(Some(THRESHOLD));

    let plain = bee(&root)
        .args(["--help", "--all", "--json"])
        .env("BEE_OUTPUT_FULL", "1")
        .output()
        .unwrap();
    assert!(plain.status.success());
    assert!(serde_json::from_slice::<serde_json::Value>(&plain.stdout).is_ok());

    let out = bee(&root)
        .args(["--help", "--all", "--json"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        out.stdout, plain.stdout,
        "the JSON document is a machine contract and passes through whole"
    );
    assert!(spill_files(&root).is_empty(), "no spill file is written");
}

#[test]
fn an_argv_outside_the_allowlist_never_spills() {
    let (_held, root) = fixture(Some(1));

    let out = bee(&root).arg("definitely-not-a-bee-verb").output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(1),
        "the refusal keeps its own exit code"
    );
    assert!(
        out.stdout.is_empty(),
        "a text refusal belongs on stderr and stdout is never captured for it"
    );
    assert!(
        spill_files(&root).is_empty(),
        "a non-allowlisted argv writes no spill file"
    );
}

#[test]
fn the_exit_code_of_a_wrapped_run_equals_the_unwrapped_run() {
    let (_held, root) = fixture(Some(1));

    let wrapped = bee(&root).args(["--help", "--all"]).output().unwrap();
    let unwrapped = bee(&root)
        .args(["--help", "--all"])
        .env("BEE_OUTPUT_FULL", "1")
        .output()
        .unwrap();
    assert_eq!(
        wrapped.status.code(),
        unwrapped.status.code(),
        "wrapping must not change the verb's exit code"
    );

    let refused_wrapped = bee(&root).args(["--help", "--bogus"]).output().unwrap();
    let refused_unwrapped = bee(&root)
        .args(["--help", "--bogus"])
        .env("BEE_OUTPUT_FULL", "1")
        .output()
        .unwrap();
    assert_eq!(
        refused_wrapped.status.code(),
        refused_unwrapped.status.code(),
        "a wrapped refusal keeps its non-zero exit code"
    );
}
