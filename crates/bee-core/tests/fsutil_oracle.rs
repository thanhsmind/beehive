//! fsutil_oracle — cross-runtime differ proving `bee_core::fsutil` matches
//! the REAL `.bee/bin/lib/fsutil.mjs` (rust-port-5, advisor note 5): every
//! "matches the mjs reader" claim here is proven by running the actual mjs
//! file in a node child (`tests/support/mjs_oracle.mjs`) and diffing its
//! output against the Rust functions — never by this file's author's
//! reading of fsutil.mjs alone.
//!
//! Every temp path used below comes from `tempfile::tempdir()` (OS temp
//! dir, e.g. under `/tmp`), never the repo's live `.bee/` store — the
//! oracle driver receives that path as an explicit argv, so nothing here
//! can accidentally touch the real store.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn oracle_script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/support/mjs_oracle.mjs")
}

struct OracleOutput {
    stdout: String,
    stderr: String,
    status: i32,
}

fn run_oracle(op: &str, target: &Path, stdin_payload: Option<&str>) -> OracleOutput {
    let mut cmd = Command::new("node");
    cmd.arg(oracle_script()).arg(op).arg(target);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    let mut child = cmd
        .spawn()
        .expect("failed to spawn node mjs_oracle driver — is `node` on PATH?");
    {
        let mut stdin = child.stdin.take().expect("child stdin");
        stdin
            .write_all(stdin_payload.unwrap_or("").as_bytes())
            .expect("write oracle stdin");
    }
    let output = child.wait_with_output().expect("wait for mjs_oracle driver");
    OracleOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        status: output.status.code().unwrap_or(-1),
    }
}

/// Truth: "truncated/corrupt jsonl fixtures produce identical results from
/// the rust reader and the real mjs reader run on the same bytes." Every
/// fixture is written to disk once and read by BOTH sides — table-driven
/// over 7 distinct corruption shapes (test-economy D3: repeated behavior
/// stays one table, not near-duplicate test fns).
#[test]
fn fsutil_oracle_readjsonl_matches_mjs_reader_on_corrupt_fixtures() {
    let cases: Vec<(&str, &[u8])> = vec![
        ("two_valid_lines", b"{\"n\":1}\n{\"n\":2}\n"),
        ("truncated_last_line", b"{\"n\":1}\n{\"n\":2, \"bro"),
        ("no_trailing_newline", b"{\"n\":1}\n{\"n\":2}"),
        (
            "blank_lines_interspersed",
            b"{\"n\":1}\n\n   \n{\"n\":2}\n",
        ),
        (
            "invalid_json_in_middle",
            b"{\"n\":1}\nnot json at all\n{\"n\":2}\n",
        ),
        ("crlf_line_endings", b"{\"n\":1}\r\n{\"n\":2}\r\n"),
        (
            "leading_bom_first_line_corrupt",
            b"\xEF\xBB\xBF{\"n\":1}\n{\"n\":2}\n",
        ),
        ("empty_file", b""),
    ];

    for (name, bytes) in cases {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("fixture.jsonl");
        std::fs::write(&file, bytes).expect("write fixture");

        let rust_result: Vec<Value> = bee_core::fsutil::read_jsonl(&file);

        let oracle = run_oracle("read-jsonl", &file, None);
        assert_eq!(
            oracle.status, 0,
            "case {name}: oracle driver exited non-zero — stderr: {}",
            oracle.stderr
        );
        let mjs_result: Vec<Value> =
            serde_json::from_str(oracle.stdout.trim()).unwrap_or_else(|e| {
                panic!("case {name}: could not parse oracle stdout {:?}: {e}", oracle.stdout)
            });

        assert_eq!(
            rust_result, mjs_result,
            "case {name}: rust readJsonl diverged from the real mjs readJsonl on identical bytes"
        );
    }
}

/// Truth: "unknown JSON fields survive read-modify-write round-trips" —
/// proven with a record the REAL mjs writer produced (never a Rust-authored
/// fixture standing in for it), then a Rust read-modify-write, then a
/// value-level check (never byte-for-byte, since re-serializing through a
/// typed struct is free to reorder keys/whitespace) that every unknown
/// field survived unchanged and is still visible to mjs afterward — proving
/// interleaving safety (D3), not just Rust-side self-consistency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ExampleRecord {
    id: String,
    count: u64,
    #[serde(flatten)]
    extra: serde_json::Map<String, Value>,
}

#[test]
fn fsutil_oracle_unknown_fields_survive_mjs_write_then_rust_read_modify_write() {
    let cases: Vec<Value> = vec![
        json!({
            "id": "rec-1", "count": 1,
            "future_field": "kept-as-is",
            "nested": {"a": 1, "b": [1, 2, 3]}
        }),
        json!({
            "id": "rec-2", "count": 7,
            "tags": ["x", "y", "z"],
            "flag": true, "maybe": null
        }),
        json!({
            "id": "rec-3", "count": 42,
            "score": 3.5, "meta": {"deep": {"deeper": "still here"}}
        }),
    ];

    for original in cases {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("record.json");

        // 1. The REAL mjs writer produces the fixture on disk.
        let write = run_oracle("write-json", &file, Some(&original.to_string()));
        assert_eq!(
            write.status, 0,
            "mjs oracle write-json failed for {original}: {}",
            write.stderr
        );

        // 2. Rust reads it through the typed flatten struct.
        let mut record: ExampleRecord = bee_core::fsutil::read_json(
            &file,
            ExampleRecord {
                id: String::new(),
                count: 0,
                extra: serde_json::Map::new(),
            },
        );
        let original_obj = original.as_object().unwrap();
        for (k, v) in original_obj {
            if k == "id" || k == "count" {
                continue;
            }
            assert_eq!(
                record.extra.get(k),
                Some(v),
                "unknown field {k:?} lost or changed on rust read of mjs-written record {original}"
            );
        }

        // 3. Rust modifies a known field and writes back with the Rust writer.
        record.count += 1;
        bee_core::fsutil::write_json_atomic(&file, &record).expect("rust write_json_atomic");

        // 4. Re-read with the Rust reader: known field updated, unknown fields intact.
        let reread: Value = bee_core::fsutil::read_json(&file, Value::Null);
        assert_eq!(reread.get("count"), Some(&json!(original["count"].as_u64().unwrap() + 1)));
        for (k, v) in original_obj {
            if k == "id" || k == "count" {
                continue;
            }
            assert_eq!(
                reread.get(k),
                Some(v),
                "unknown field {k:?} lost after rust read-modify-write of {original}"
            );
        }

        // 5. The REAL mjs reader must see the same thing (cross-runtime
        //    interleaving safety, D3) — not just Rust reading its own write.
        let mjs_reread = run_oracle("read-json", &file, None);
        assert_eq!(mjs_reread.status, 0, "mjs oracle read-json failed: {}", mjs_reread.stderr);
        let mjs_value: Value = serde_json::from_str(mjs_reread.stdout.trim()).expect("parse mjs reread");
        assert_eq!(
            mjs_value, reread,
            "mjs reader disagreed with rust reader after a rust read-modify-write of an mjs-written record"
        );
    }
}

/// Sanity: the oracle driver itself agrees with mjs's own fallback-on-missing
/// semantics (readJson(file, null) on a nonexistent path), so the other
/// tests' use of the driver as ground truth is itself trustworthy.
#[test]
fn fsutil_oracle_read_json_missing_file_matches_mjs_fallback() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("does-not-exist.json");

    let rust_result: Value = bee_core::fsutil::read_json(&file, Value::Null);
    let oracle = run_oracle("read-json", &file, None);
    assert_eq!(oracle.status, 0, "oracle read-json failed: {}", oracle.stderr);
    let mjs_result: Value = serde_json::from_str(oracle.stdout.trim()).expect("parse oracle stdout");

    assert_eq!(rust_result, Value::Null);
    assert_eq!(mjs_result, Value::Null);
    assert_eq!(rust_result, mjs_result);
}
