use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::Value;

use super::job_verbs::JobVerbError;
use super::paseo::{self, PaseoCli, PaseoState};
use super::{mailbox, paseo_state, resolve_main_root, run};

pub(crate) enum Source<'a> {
    Job(&'a str),
    Agent(&'a str),
}

#[derive(Debug, PartialEq)]
pub(crate) struct Forked {
    pub agent_id: String,
    pub source_agent_id: String,
    pub session_file: PathBuf,
}

fn refuse(code: &'static str, message: String) -> JobVerbError {
    JobVerbError { code, message: format!("herding fork: {message}") }
}

fn source_agent_id(main_root: &Path, source: Source<'_>) -> Result<String, JobVerbError> {
    let job = match source {
        Source::Agent(id) => return Ok(id.to_string()),
        Source::Job(job) => job,
    };
    let path = mailbox::job_path(&main_root.join(".bee"), job);
    let crate::fsutil::ReadJson::Parsed(raw) = crate::fsutil::read_json(&path) else {
        return Err(refuse(
            "job_not_found",
            format!("job \"{job}\" has no job.json. FIX: check the job id with bee herding status"),
        ));
    };
    raw.get("paseo_agent_id")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            refuse(
                "paseo_agent_missing",
                format!("job \"{job}\" has no recorded paseo_agent_id. FIX: pass --agent <paseo id> instead"),
            )
        })
}

fn uuid_v7(now: DateTime<Utc>) -> String {
    let mut b = [0u8; 16];
    b[..6].copy_from_slice(&(now.timestamp_millis() as u64).to_be_bytes()[2..]);
    b[6..].copy_from_slice(&crate::verbs::feedback::random_bytes(10));
    b[6] = (b[6] & 0x0f) | 0x70;
    b[8] = (b[8] & 0x3f) | 0x80;
    let hex = crate::verbs::feedback::hex_lower(&b);
    format!("{}-{}-{}-{}-{}", &hex[..8], &hex[8..12], &hex[12..16], &hex[16..20], &hex[20..])
}

fn copy_session(source: &Path, now: DateTime<Utc>) -> Result<(String, PathBuf), JobVerbError> {
    let shown = source.display();
    let bytes = std::fs::read(source)
        .map_err(|e| refuse("session_unreadable", format!("cannot read {shown}: {e}. FIX: check the file")))?;
    let split = bytes.iter().position(|&c| c == b'\n').unwrap_or(bytes.len());
    let first = bytes[..split].strip_suffix(b"\r").unwrap_or(&bytes[..split]);
    let header = serde_json::from_slice::<Value>(first)
        .ok()
        .filter(|h| h.get("type").and_then(Value::as_str) == Some("session"));
    let Some(Value::Object(mut header)) = header else {
        return Err(refuse(
            "session_header_missing",
            format!("the first line of {shown} is not a pi session header. FIX: fork only a pi session file"),
        ));
    };
    let id = uuid_v7(now);
    let stamp = now.to_rfc3339_opts(SecondsFormat::Millis, true);
    header.insert("id".into(), Value::String(id.clone()));
    header.insert("timestamp".into(), Value::String(stamp.clone()));
    header.insert("parentSession".into(), Value::String(source.to_string_lossy().into_owned()));
    let dir = source.parent().unwrap_or(Path::new("."));
    let target = dir.join(format!("{}_{id}.jsonl", stamp.replace([':', '.'], "-")));
    let temp = PathBuf::from(format!("{}.tmp-{}", target.display(), std::process::id()));
    let mut out = serde_json::to_vec(&header).unwrap_or_default();
    out.extend_from_slice(&bytes[split..]);
    let written = std::fs::write(&temp, out).and_then(|()| std::fs::rename(&temp, &target));
    if let Err(e) = written {
        let _ = std::fs::remove_file(&temp);
        return Err(refuse(
            "session_write_failed",
            format!("cannot write {}: {e}. FIX: check the sessions directory", target.display()),
        ));
    }
    Ok((id, target))
}

pub(crate) fn import_argv(session_id: &str, cwd: &str, source_id: &str) -> Vec<String> {
    [
        "import",
        session_id,
        "--provider",
        "pi",
        "--cwd",
        cwd,
        "--label",
        &format!("bee_fork_of={source_id}"),
        "--json",
    ]
    .map(str::to_string)
    .to_vec()
}

pub(crate) fn fork_agent(
    main_root: &Path,
    source: Source<'_>,
    force: bool,
    paseo_home: Option<&Path>,
    sessions_root: Option<&Path>,
    cli: &dyn PaseoCli,
    now: DateTime<Utc>,
) -> Result<Forked, JobVerbError> {
    let id = source_agent_id(main_root, source)?;
    let home = paseo_home.ok_or_else(|| {
        refuse(
            "paseo_home_missing",
            "no Paseo home with an agents directory. FIX: set PASEO_HOME or --home in herding.paseo.command".into(),
        )
    })?;
    let record = paseo_state::find_agent(home, &id).ok_or_else(|| {
        refuse(
            "agent_not_found",
            format!("no readable agent file for \"{id}\" under {}. FIX: check the id with paseo ls -a", home.display()),
        )
    })?;
    if record.provider.as_deref() != Some("pi") {
        let provider = record.provider.as_deref().unwrap_or("unknown");
        return Err(refuse(
            "provider_not_pi",
            format!("agent \"{id}\" runs provider {provider}; fork copies pi sessions only. FIX: fork a pi agent"),
        ));
    }
    let handle = record.native_handle.filter(|h| !h.trim().is_empty()).ok_or_else(|| {
        refuse(
            "session_file_missing",
            format!("agent \"{id}\" has no persistence.nativeHandle. FIX: fork on the host where its session lives"),
        )
    })?;
    let session = std::fs::canonicalize(&handle).map_err(|_| {
        refuse(
            "session_file_missing",
            format!("session file {handle} does not exist. FIX: fork on the host where its session lives"),
        )
    })?;
    let root = sessions_root.and_then(|r| std::fs::canonicalize(r).ok());
    if !root.as_ref().is_some_and(|r| session.starts_with(r)) {
        return Err(refuse(
            "session_outside_root",
            format!("session file {handle} is not under the pi sessions root ~/.pi/agent/sessions/. FIX: fork only a default pi session"),
        ));
    }
    let cwd = record.cwd.filter(|c| !c.trim().is_empty()).ok_or_else(|| {
        refuse("cwd_missing", format!("agent \"{id}\" records no cwd. FIX: check the agent file"))
    })?;
    let live = cli.call(&paseo::inspect_argv(&id)).ok();
    if !force && live.as_deref().and_then(paseo::parse_inspect) == Some(PaseoState::Working) {
        return Err(refuse(
            "source_working",
            format!("agent \"{id}\" is Working. FIX: wait until it is idle, or pass --force"),
        ));
    }
    let (session_id, file) = copy_session(&session, now)?;
    let imported = cli
        .call(&import_argv(&session_id, &cwd, &id))
        .and_then(|out| paseo::parse_run_agent_id(&out));
    match imported {
        Ok(agent_id) => Ok(Forked { agent_id, source_agent_id: id, session_file: file }),
        Err(e) => {
            let _ = std::fs::remove_file(&file);
            Err(refuse(
                "import_failed",
                format!("paseo import failed: {e}; removed {}. FIX: check the Paseo daemon, then run the fork again", file.display()),
            ))
        }
    }
}

fn user_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .filter(|h| !h.is_empty())
        .map(PathBuf::from)
}

pub(super) fn fork(args: &[&str]) -> ExitCode {
    let (mut job, mut agent, mut main_root, mut force, mut json) = (None, None, None, false, false);
    let mut it = args.iter().copied();
    while let Some(arg) = it.next() {
        match arg {
            "--job" => job = it.next(),
            "--agent" => agent = it.next(),
            "--main-root" => main_root = it.next(),
            "--force" => force = true,
            "--json" => json = true,
            _ => {}
        }
    }
    let source = match (job, agent) {
        (Some(j), None) => Source::Job(j),
        (None, Some(a)) => Source::Agent(a),
        _ => {
            eprintln!("herding fork: pass exactly one of --job <id> or --agent <paseo id>. FIX: run bee herding fork --agent <id>");
            return ExitCode::FAILURE;
        }
    };
    let root = resolve_main_root(main_root);
    if root.is_none() && matches!(source, Source::Job(_)) {
        eprintln!("herding fork: could not resolve main checkout root. FIX: pass --main-root <path>");
        return ExitCode::FAILURE;
    }
    let root = root.unwrap_or_default();
    let cmd = paseo::paseo_command(&run::read_main_config(&root));
    let cli = paseo::RealPaseoCli::new(cmd.as_str()).with_timeout(Duration::from_secs(60));
    let home = paseo_state::resolve_home(&cmd);
    let sessions = user_home().map(|h| h.join(".pi").join("agent").join("sessions"));
    match fork_agent(&root, source, force, home.as_deref(), sessions.as_deref(), &cli, Utc::now()) {
        Ok(f) => {
            let file = f.session_file.display().to_string();
            let undo = format!("paseo archive {}; rm {file}", f.agent_id);
            let model = "provider default (not copied)";
            let next = format!("paseo send {} --prompt <text>", f.agent_id);
            if json {
                let out = serde_json::json!({
                    "agent_id": f.agent_id,
                    "source_agent_id": f.source_agent_id,
                    "session_file": file,
                    "undo": undo,
                    "model": model,
                    "next": next,
                });
                println!("{out}");
            } else {
                println!("herding: forked agent {} into {}", f.source_agent_id, f.agent_id);
                println!("session file: {file}");
                println!("undo: {undo}");
                println!("model: {model}");
                println!("next: {next}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"error": e.code, "message": e.message}));
            } else {
                eprintln!("{}", e.message);
            }
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct FakeCli {
        calls: Mutex<Vec<Vec<String>>>,
        inspect: &'static str,
        import: Result<String, String>,
    }

    impl PaseoCli for FakeCli {
        fn call(&self, args: &[String]) -> Result<String, String> {
            self.calls.lock().unwrap().push(args.to_vec());
            match args.first().map(String::as_str) {
                Some("inspect") => Ok(format!(r#"{{"Id":"src","Status":"{}"}}"#, self.inspect)),
                Some("import") => self.import.clone(),
                _ => Err("unexpected call".into()),
            }
        }
    }

    fn cli(inspect: &'static str, import: Result<&str, &str>) -> FakeCli {
        FakeCli {
            calls: Mutex::new(Vec::new()),
            inspect,
            import: import.map(str::to_string).map_err(str::to_string),
        }
    }

    const TAIL: &[u8] = b"\n{\"type\":\"message\",\"id\":\"a1\",\"parentId\":null}\r\n{\"type\":\"message\",\"id\":\"a2\"}  \n";

    struct Fixture {
        tmp: tempfile::TempDir,
        source: PathBuf,
    }

    impl Fixture {
        fn new(provider: &str, handle: Option<&str>) -> Self {
            let tmp = tempfile::tempdir().unwrap();
            let dir = tmp.path().join("home/.pi/agent/sessions/--repo--");
            std::fs::create_dir_all(&dir).unwrap();
            let source = dir.join("2026-10-12T07-06-31-387Z_src.jsonl");
            let mut body = br#"{"type":"session","version":3,"id":"old-id","timestamp":"2026-10-12T07:06:31.387Z","cwd":"/repo"}"#.to_vec();
            body.extend_from_slice(TAIL);
            std::fs::write(&source, body).unwrap();
            let src = source.to_string_lossy().into_owned();
            let handle = handle.map(|h| if h == "SOURCE" { src.clone() } else { h.to_string() });
            let agents = tmp.path().join("paseo/agents/proj");
            std::fs::create_dir_all(&agents).unwrap();
            let record = serde_json::json!({
                "id": "src", "provider": provider, "cwd": "/repo",
                "persistence": {"nativeHandle": handle},
            });
            std::fs::write(agents.join("src.json"), record.to_string()).unwrap();
            Self { tmp, source }
        }

        fn run(&self, source: Source<'_>, force: bool, cli: &FakeCli) -> Result<Forked, JobVerbError> {
            let now = DateTime::parse_from_rfc3339("2026-10-12T08:00:00.123Z").unwrap().with_timezone(&Utc);
            let root = self.tmp.path().join("home/.pi/agent/sessions");
            fork_agent(self.tmp.path(), source, force, Some(&self.tmp.path().join("paseo")), Some(&root), cli, now)
        }

        fn session_files(&self) -> Vec<String> {
            let mut names: Vec<String> = std::fs::read_dir(self.source.parent().unwrap())
                .unwrap()
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect();
            names.sort();
            names
        }
    }

    fn assert_refused(res: Result<Forked, JobVerbError>, code: &str) {
        let err = res.expect_err("fork must refuse");
        assert_eq!(err.code, code, "{}", err.message);
        assert!(err.message.contains("FIX:"), "{}", err.message);
    }

    #[test]
    fn fork_rewrites_the_header_copies_the_tail_byte_for_byte_and_imports() {
        let fx = Fixture::new("pi", Some("SOURCE"));
        let before = std::fs::read(&fx.source).unwrap();
        let cli = cli("idle", Ok(r#"{"agentId":"new-agent","status":"idle"}"#));
        let forked = fx.run(Source::Agent("src"), false, &cli).unwrap();
        assert_eq!(forked.agent_id, "new-agent");
        assert_eq!(forked.source_agent_id, "src");
        let bytes = std::fs::read(&forked.session_file).unwrap();
        let split = bytes.iter().position(|&c| c == b'\n').unwrap();
        assert_eq!(&bytes[split..], TAIL);
        let header: Value = serde_json::from_slice(&bytes[..split]).unwrap();
        let new_id = header["id"].as_str().unwrap();
        assert_ne!(new_id, "old-id");
        assert_eq!(&new_id[14..15], "7");
        assert_eq!(header["timestamp"], "2026-10-12T08:00:00.123Z");
        assert_eq!(header["parentSession"], fx.source.canonicalize().unwrap().to_string_lossy().as_ref());
        assert_eq!(header["version"], 3);
        assert_eq!(header["cwd"], "/repo");
        assert_eq!(
            forked.session_file.file_name().unwrap().to_string_lossy(),
            format!("2026-10-12T08-00-00-123Z_{new_id}.jsonl")
        );
        assert_eq!(std::fs::read(&fx.source).unwrap(), before);
        let calls = cli.calls.lock().unwrap().clone();
        assert_eq!(calls, vec![paseo::inspect_argv("src"), import_argv(new_id, "/repo", "src")]);
        assert_eq!(
            calls[1],
            ["import", new_id, "--provider", "pi", "--cwd", "/repo", "--label", "bee_fork_of=src", "--json"]
        );
        assert!(fx.session_files().iter().all(|n| !n.contains(".tmp-")));
    }

    #[test]
    fn fork_resolves_the_source_from_the_job_json() {
        let fx = Fixture::new("pi", Some("SOURCE"));
        let job = mailbox::job_path(&fx.tmp.path().join(".bee"), "job-1");
        std::fs::create_dir_all(job.parent().unwrap()).unwrap();
        std::fs::write(&job, r#"{"job_id":"job-1","paseo_agent_id":"src"}"#).unwrap();
        let cli = cli("idle", Ok(r#"{"agentId":"new-agent"}"#));
        assert_eq!(fx.run(Source::Job("job-1"), false, &cli).unwrap().source_agent_id, "src");
        assert_refused(fx.run(Source::Job("job-missing"), false, &cli), "job_not_found");
    }

    #[test]
    fn fork_refuses_a_non_pi_source() {
        let fx = Fixture::new("claude", Some("SOURCE"));
        assert_refused(fx.run(Source::Agent("src"), false, &cli("idle", Ok("{}"))), "provider_not_pi");
    }

    #[test]
    fn fork_refuses_a_missing_handle_or_a_missing_file() {
        let fx = Fixture::new("pi", None);
        assert_refused(fx.run(Source::Agent("src"), false, &cli("idle", Ok("{}"))), "session_file_missing");
        let fx = Fixture::new("pi", Some("/nowhere/s.jsonl"));
        assert_refused(fx.run(Source::Agent("src"), false, &cli("idle", Ok("{}"))), "session_file_missing");
    }

    #[test]
    fn fork_refuses_a_session_outside_the_pi_sessions_root() {
        let outside = tempfile::tempdir().unwrap();
        let file = outside.path().join("s.jsonl");
        std::fs::write(&file, "{\"type\":\"session\",\"id\":\"x\"}\n").unwrap();
        let fx = Fixture::new("pi", Some(file.to_str().unwrap()));
        assert_refused(fx.run(Source::Agent("src"), false, &cli("idle", Ok("{}"))), "session_outside_root");
    }

    #[test]
    fn fork_refuses_a_working_source_unless_forced() {
        let fx = Fixture::new("pi", Some("SOURCE"));
        let working = cli("running", Ok(r#"{"agentId":"new-agent"}"#));
        assert_refused(fx.run(Source::Agent("src"), false, &working), "source_working");
        assert_eq!(fx.session_files().len(), 1);
        assert_eq!(fx.run(Source::Agent("src"), true, &working).unwrap().agent_id, "new-agent");
    }

    #[test]
    fn fork_refuses_a_file_without_a_session_header() {
        let fx = Fixture::new("pi", Some("SOURCE"));
        std::fs::write(&fx.source, "{\"type\":\"message\"}\n").unwrap();
        assert_refused(fx.run(Source::Agent("src"), false, &cli("idle", Ok("{}"))), "session_header_missing");
    }

    #[test]
    fn fork_removes_the_copy_when_import_fails() {
        let fx = Fixture::new("pi", Some("SOURCE"));
        for import in [Err("daemon down"), Ok("{\"status\":\"idle\"}")] {
            let cli = cli("idle", import);
            assert_refused(fx.run(Source::Agent("src"), false, &cli), "import_failed");
            assert_eq!(fx.session_files(), vec!["2026-10-12T07-06-31-387Z_src.jsonl".to_string()]);
            let verbs: Vec<String> = cli.calls.lock().unwrap().iter().map(|c| c[0].clone()).collect();
            assert_eq!(verbs, vec!["inspect", "import"]);
        }
    }
}
