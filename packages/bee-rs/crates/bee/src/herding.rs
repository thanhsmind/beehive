// herding — the bee-herding cockpit's EXECUTABLE helpers, ported off Node
// (R6a of plans/rust-port.md), plus the D17 wave/occupancy verbs below.
//
//   bee herding classify-lane <PBI-ID>   <- skills/bee-herding/scripts/classify-lane.mjs
//   bee herding interlock [--main-root P] <- skills/bee-herding/scripts/dispatch-interlock.mjs
//   bee herding command-template <key>   <- control-loop.sh's `node -e` config reader
//   bee herding herdr-result <path>      <- bootstrap-cockpit.sh's json_result
//   bee herding herdr-pane-id --label L  <- bootstrap-cockpit.sh's find_dispatch_pane
//
// Two more verbs (herding-orchestration D17) live beside these, in wave.rs:
//
//   bee herding wave      <- the bee-side entry point: turns herding.agent_command
//                             (D14 split) into a running fleet::Wave through a real
//                             HerdrBackend, then appends one row to the wave ledger.
//   bee herding occupancy <- the CLI bridge to the wave ledger's read side (D10),
//                             reachable from a markdown role for the first time.
//   bee herding record-worker <- (herding-orchestration D18) the recording verb
//                             role-dispatch.md §8 calls right after a successful
//                             spawn, so the row occupancy reads next iteration
//                             actually exists. See wave.rs.
//
//   bee herding control-loop <- (herding-orchestration D8) the Rust replacement
//                             for skills/bee-herding/scripts/control-loop.sh.
//                             See control_loop.rs — control-loop.sh was deleted
//                             by ho-14, which rewired bootstrap-cockpit.sh onto
//                             this verb; ho-15 moved the references it left.
//
//   bee herding run       <- (herding-executor D1/D2/D5/D6/D9) start one
//                             bee-ignorant external agent in a pane, wait on
//                             a file mailbox with native health-check
//                             liveness, return one structured result. See
//                             run.rs.
//
// The next three (herdr-result, herdr-pane-id, command-template) are the cockpit
// shell scripts' inline `node -e` snippets. They
// are not bee state at all — they parse a config file and herdr's own JSON
// envelopes — but they were the only remaining reason `control-loop.sh` and
// `bootstrap-cockpit.sh` needed a Node runtime on PATH, so they move with the
// rest rather than silently breaking when the engine is deleted.
//
// These are HOST-REPO tools (the control loop runs them in whatever repo it is
// herding), so they cannot live under `bee dev …`: that namespace gates on
// `bee_source_root()` and refuses outside a bee source checkout. They are not
// bee.mjs porcelain verbs either — no Node command has ever spelled
// `bee herding …` — so, like `onboard` and `dev`, this namespace probes before
// the verb tree and can never collide with a delegated Node verb.
//
// CONTRACTS PRESERVED FROM THE .mjs (both are safety interlocks; their output
// shape and exit codes are what role-dispatch.md and control-loop.sh read):
//   classify-lane → one JSON object {pbi, lane, hard_gate_flags, lane_safe,
//     reason} on stdout, always exit 0. Fail-CLOSED: anything unclassifiable
//     (no id, unreachable fold, empty title+cos, out-of-enum status) comes back
//     lane "high-risk" / lane_safe:false with a reason naming why.
//   interlock → one JSON object {enabled, marker, main_root, reason};
//     exit 0 enabled · 3 disabled · 1 cannot-decide.
//
// The one deliberate behavioral change: classify-lane reads the PBI fold
// IN-PROCESS (verbs::backlog::fold_pbi_records) instead of spawning
// `bee backlog pbi list --json`. Same event-sourced fold over
// .bee/backlog.jsonl, one process instead of two, and the `--bee-cmd` escape
// hatch is no longer meaningful — it is accepted and ignored so any caller
// still passing it keeps working.

use serde_json::{Map, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use run::PaneTransport;

// The append-only wave ledger (D10): one row per wave, read side (occupancy)
// and write side (append_wave). `wave` below is the CLI verb that drives a
// real wave and appends to it; `occupancy` is the CLI verb that reads it.
mod wave_ledger;

// `bee herding wave` (D17) and `bee herding occupancy` — the caller that
// turns `herding.agent_command` into a running wave, and the CLI bridge to
// the ledger's read side. See `wave.rs` for both.
mod wave;

// `bee herding control-loop` (D8) — the Rust replacement for
// control-loop.sh. See control_loop.rs.
mod control_loop;

// The file mailbox worker-completion contract (herding-executor feature:
// mailbox layout and the self-contained-brief requirement, both locked in
// .bee/decisions.jsonl feature=herding-executor). See mailbox.rs.
mod mailbox;

// `bee herding run` (herding-executor D1/D2/D5/D6/D9) — the scope-A verb:
// spawn one bee-ignorant external agent into a pane, wait on the mailbox
// above with native health-check liveness, return one structured result.
// See run.rs.
mod run;

// The tmux implementation of run.rs's `PaneTransport` seam
// (tmux-herding-transport D2/D3/D4/D5): the same 13 operations, reached
// through `tmux` verbs instead of `herdr` ones, with pane status read from a
// bounded `capture-pane` and classified against marker lists held as config
// data. Selected by `herding.transport` (D1), never by env sniffing.
// See tmux.rs.
pub(crate) mod tmux;

// `bee herding pane …` / `agent-start` / `pane-id` / `result`
// (tmux-herding-cockpit D2) — the transport-neutral cockpit vocabulary a
// role document and bootstrap-cockpit.sh act through, over a
// `CockpitTransport` trait implemented for both `RealHerdr` and
// `RealTmux`. See pane_verbs.rs.
pub(crate) mod pane_verbs;

// The cross-PROCESS pane-split lock (herding-split-serialize): concurrent
// `bee herding run` processes serialize their pane split through an advisory
// lock file, because each spawn is its own OS process. See split_lock.rs.
mod split_lock;

// The interrupt and cancel verbs (herding-cockpit-completeness D1, D4):
// interrupt sends Escape and keeps the pane; cancel captures foreground pid,
// closes the pane, and confirms exit within 5 s. See job_verbs.rs.
mod job_verbs;

const ENABLE_BASENAME: &str = "bee-herding.enable";

pub fn try_native(args: &[OsString]) -> Option<ExitCode> {
    let strs: Vec<&str> = args.iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    if strs.first().copied() != Some("herding") {
        return None;
    }
    let (name, rest) = strs[1..].split_first()?;
    match *name {
        "classify-lane" => Some(classify_lane(rest)),
        "interlock" => Some(interlock(rest)),
        "status" => Some(status(rest)),
        "command-template" => Some(command_template(rest)),
        "herdr-result" => Some(herdr_result(rest)),
        "herdr-pane-id" => Some(herdr_pane_id(rest)),
        "pane" => Some(pane_verbs::pane(rest)),
        "agent-start" => Some(pane_verbs::agent_start(rest)),
        "pane-id" => Some(pane_verbs::pane_id(rest)),
        "result" => Some(pane_verbs::result(rest)),
        "wave" => Some(wave::wave(rest)),
        "occupancy" => Some(wave::occupancy(rest)),
        "record-worker" => Some(wave::record_worker(rest)),
        "run" => Some(run::run(rest)),
        "control-loop" => Some(control_loop::control_loop(rest)),
        "interrupt" => Some(job_verbs::interrupt(rest)),
        "cancel" => Some(job_verbs::cancel(rest)),
        _ => None,
    }
}

fn emit(obj: Map<String, Value>) {
    println!("{}", serde_json::to_string(&Value::Object(obj)).unwrap());
}

// ═══════════════════════════════════════════════════════════════════════════
// classify-lane
// ═══════════════════════════════════════════════════════════════════════════

/// One mode-gate risk flag, from bee-planning SKILL.md's "Mode Gate" section.
/// `hard_gate` marks D6's decisive subset (any single one classifies
/// high-risk); four or more of ANY flag does the same.
struct FlagRule {
    label: &'static str,
    hard_gate: bool,
    matcher: fn(&str) -> bool,
}

/// Case-insensitive substring alternation, over an already-lowercased haystack.
fn has_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| text.contains(n))
}

/// JS `\b<word>\b` — ASCII word boundaries ([A-Za-z0-9_]) on both sides, which
/// is what the .mjs's `/…/i` patterns meant. Non-ASCII letters (Vietnamese
/// diacritics) are NOT \w in JS either, so they count as boundaries here too.
fn has_word(text: &str, word: &str) -> bool {
    let is_w = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let bytes = text.as_bytes();
    let wlen = word.len();
    let mut from = 0usize;
    while let Some(off) = text[from..].find(word) {
        let start = from + off;
        let end = start + wlen;
        let before_ok = start == 0 || !is_w(prev_char(text, start));
        let after_ok = end == bytes.len() || !is_w(text[end..].chars().next().unwrap());
        if before_ok && after_ok {
            return true;
        }
        from = start + 1;
        if from >= text.len() {
            break;
        }
    }
    false
}

fn prev_char(text: &str, idx: usize) -> char {
    text[..idx].chars().next_back().unwrap_or(' ')
}

fn has_any_word(text: &str, words: &[&str]) -> bool {
    words.iter().any(|w| has_word(text, w))
}

/// JS `<a>[^\n]{0,40}<b>` — `a`, then at most 40 non-newline chars, then `b`.
fn has_near(text: &str, a: &str, b: &str) -> bool {
    let mut from = 0usize;
    while let Some(off) = text[from..].find(a) {
        let start = from + off + a.len();
        let tail = &text[start..];
        let window: String = tail.chars().take_while(|c| *c != '\n').take(40 + b.len()).collect();
        if let Some(pos) = window.find(b) {
            if window[..pos].chars().count() <= 40 && !window[..pos].contains('\n') {
                return true;
            }
        }
        from = from + off + 1;
        if from >= text.len() {
            break;
        }
    }
    false
}

const FLAG_RULES: &[FlagRule] = &[
    FlagRule {
        label: "auth",
        hard_gate: true,
        // \bauthentication\b | \bauth\b(?!ori) | đăng nhập | xác thực
        // (the (?!ori) is redundant: \bauth\b already excludes "authoriz…")
        matcher: |t| has_any_word(t, &["authentication", "auth"]) || has_any(t, &["đăng nhập", "xác thực"]),
    },
    FlagRule {
        label: "authorization",
        hard_gate: true,
        matcher: |t| {
            has_any_word(t, &["authorize", "authorization", "authorized", "authorizing", "authz"])
                || has_any(t, &["phân quyền", "ủy quyền", "uỷ quyền", "quyền truy cập"])
        },
    },
    FlagRule {
        label: "data model / data loss",
        hard_gate: true,
        matcher: |t| {
            has_any_word(t, &["data loss", "data model"])
                || has_any(
                    t,
                    &[
                        "drop table",
                        "xóa dữ liệu",
                        "xoá dữ liệu",
                        "mất dữ liệu",
                        "schema change",
                        "schema migration",
                        "mô hình dữ liệu",
                    ],
                )
        },
    },
    FlagRule {
        label: "audit/security",
        hard_gate: true,
        matcher: |t| {
            has_any_word(t, &["audit", "security"])
                || has_any(t, &["bảo mật", "an ninh", "lỗ hổng", "vulnerab"])
        },
    },
    FlagRule {
        label: "external systems / external provider",
        hard_gate: true,
        matcher: |t| {
            has_any(
                t,
                &[
                    "external provider",
                    "external system",
                    "external service",
                    "external api",
                    "third-party",
                    "third party",
                    "bên ngoài",
                    "nhà cung cấp",
                    "dịch vụ ngoài",
                ],
            )
        },
    },
    FlagRule {
        label: "public contracts",
        hard_gate: false,
        matcher: |t| {
            has_any(t, &["public contract", "breaking change", "api contract", "hợp đồng công khai"])
        },
    },
    FlagRule {
        label: "cross-platform",
        hard_gate: false,
        matcher: |t| {
            has_any(t, &["cross-platform", "đa nền tảng"])
                || has_near(t, "windows", "macos")
                || has_near(t, "windows", "linux")
                || has_near(t, "macos", "windows")
                || has_near(t, "macos", "linux")
        },
    },
    FlagRule {
        label: "changes behavior an existing test asserts",
        hard_gate: false,
        matcher: |t| {
            has_any(t, &["existing test", "covered contract", "kiểm thử hiện có", "test hiện có"])
        },
    },
    FlagRule {
        label: "weakening/deleting/replacing existing proof (validation removal)",
        hard_gate: true,
        matcher: |t| {
            has_any_word(t, &["weaken", "weakening"])
                || has_any(
                    t,
                    &[
                        "remove validation",
                        "skip validation",
                        "bỏ qua kiểm tra",
                        "gỡ bỏ kiểm tra",
                        "gỡ bỏ validation",
                        "xoá test",
                        "xóa test",
                        "xoá proof",
                        "xóa proof",
                    ],
                )
        },
    },
    FlagRule {
        label: "multi-domain",
        hard_gate: false,
        matcher: |t| has_any(t, &["multi-domain", "multiple domains", "nhiều domain", "đa lĩnh vực"]),
    },
];

const PBI_STATUSES: [&str; 5] = ["proposed", "in-flight", "parked", "done", "declined"];

fn result_object(
    pbi: &str,
    lane: &str,
    hard_gate_flags: Vec<String>,
    lane_safe: bool,
    reason: String,
) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("pbi".into(), Value::String(pbi.to_string()));
    m.insert("lane".into(), Value::String(lane.to_string()));
    m.insert(
        "hard_gate_flags".into(),
        Value::Array(hard_gate_flags.into_iter().map(Value::String).collect()),
    );
    m.insert("lane_safe".into(), Value::Bool(lane_safe));
    m.insert("reason".into(), Value::String(reason));
    m
}

fn unclassifiable(pbi: &str, reason: String) -> ExitCode {
    emit(result_object(pbi, "high-risk", Vec::new(), false, reason));
    ExitCode::SUCCESS
}

fn classify_lane(flags: &[&str]) -> ExitCode {
    let mut pbi: Option<&str> = None;
    let mut i = 0usize;
    while i < flags.len() {
        // `--bee-cmd X` is accepted and ignored: the fold is read in-process
        // now, so there is no child `bee` to point anywhere.
        if flags[i] == "--bee-cmd" {
            i += 2;
            continue;
        }
        if pbi.is_none() {
            pbi = Some(flags[i]);
        }
        i += 1;
    }
    let Some(pbi) = pbi else {
        return unclassifiable("", "no PBI id provided on the command line".to_string());
    };

    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return unclassifiable(pbi, format!("cannot read the PBI fold: cwd unreadable: {e}"));
        }
    };
    let root = match crate::roots::resolve_store_root_worktree(&cwd) {
        crate::roots::RootsWt::Go(r) => r.root,
        _ => {
            return unclassifiable(
                pbi,
                "cannot read the PBI fold: no bee repo root found (no .bee/onboarding.json or .git up the tree), or the worktree link is broken".to_string(),
            );
        }
    };
    let Some(records) = crate::verbs::backlog::fold_pbi_records(&root) else {
        return unclassifiable(
            pbi,
            "cannot read the PBI fold: .bee/backlog.jsonl has a line that could not be parsed as JSON".to_string(),
        );
    };

    let Some(record) = records.iter().find(|r| r.get("id").and_then(Value::as_str) == Some(pbi))
    else {
        return unclassifiable(pbi, format!("no matching PBI found for {pbi} in the fold"));
    };

    let title = record.get("title").and_then(Value::as_str).unwrap_or("");
    let cos = record.get("cos").and_then(Value::as_str).unwrap_or("");
    let text = [title, cos]
        .iter()
        .filter(|p| !p.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    let text = text.trim();
    if text.is_empty() {
        return unclassifiable(
            pbi,
            format!("record for {pbi} has empty or unparseable text (no title or cos found)"),
        );
    }
    match record.get("status") {
        None | Some(Value::Null) => {}
        Some(Value::String(s)) if PBI_STATUSES.contains(&s.as_str()) => {}
        Some(other) => {
            let shown = match other {
                Value::String(s) => s.clone(),
                v => v.to_string(),
            };
            return unclassifiable(
                pbi,
                format!("record for {pbi} has out-of-enum status \"{shown}\""),
            );
        }
    }

    let lowered = text.to_lowercase();
    let matched: Vec<&FlagRule> = FLAG_RULES.iter().filter(|r| (r.matcher)(&lowered)).collect();
    let hard: Vec<&&FlagRule> = matched.iter().filter(|r| r.hard_gate).collect();

    if !hard.is_empty() {
        let labels: Vec<String> = hard.iter().map(|r| r.label.to_string()).collect();
        let joined = labels.join(", ");
        emit(result_object(
            pbi,
            "high-risk",
            labels,
            false,
            format!("hard-gate flag matched: {joined}"),
        ));
        return ExitCode::SUCCESS;
    }

    if matched.len() >= 4 {
        let joined = matched.iter().map(|r| r.label).collect::<Vec<_>>().join(", ");
        emit(result_object(
            pbi,
            "high-risk",
            Vec::new(),
            false,
            format!(
                "{} mode-gate risk flags matched (4+ classifies high-risk): {joined}",
                matched.len()
            ),
        ));
        return ExitCode::SUCCESS;
    }

    // 0-1 flags -> tiny/small; 2-3 -> standard (bee-planning Mode Gate). The
    // tiny/small split further depends on a product-file count this command
    // cannot see from backlog text alone, so 0-1 flags reports the safer
    // (larger) of the two, "small".
    let lane = if matched.len() >= 2 { "standard" } else { "small" };
    let reason = if matched.is_empty() {
        "no mode-gate risk flags matched in title/cos text".to_string()
    } else {
        let joined = matched.iter().map(|r| r.label).collect::<Vec<_>>().join(", ");
        format!(
            "{} mode-gate risk flag(s) matched, below the high-risk threshold: {joined}",
            matched.len()
        )
    };
    emit(result_object(pbi, lane, Vec::new(), true, reason));
    ExitCode::SUCCESS
}

// ═══════════════════════════════════════════════════════════════════════════
// interlock
// ═══════════════════════════════════════════════════════════════════════════

fn interlock_object(
    enabled: bool,
    marker: Option<&str>,
    main_root: Option<&str>,
    reason: String,
) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("enabled".into(), Value::Bool(enabled));
    m.insert(
        "marker".into(),
        marker.map(|p| Value::String(p.to_string())).unwrap_or(Value::Null),
    );
    m.insert(
        "main_root".into(),
        main_root.map(|p| Value::String(p.to_string())).unwrap_or(Value::Null),
    );
    m.insert("reason".into(), Value::String(reason));
    m
}

/// Resolve the MAIN checkout root the same way bootstrap's §1 does: the shared
/// .git common dir, correct whether invoked from main or a linked worktree.
/// An explicit --main-root always wins.
pub(crate) fn resolve_main_root(explicit: Option<&str>) -> Option<PathBuf> {
    if let Some(p) = explicit {
        return Some(PathBuf::from(p));
    }
    let out = Command::new("git")
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        return None;
    }
    // strip the trailing "/.git" (or ".git") to get the worktree root
    Path::new(&s).parent().map(Path::to_path_buf)
}

pub(crate) fn enable_marker_state(explicit: Option<&str>) -> Map<String, Value> {
    let Some(main_root) = resolve_main_root(explicit) else {
        return interlock_object(
            false,
            None,
            None,
            "could not resolve the MAIN checkout root (no --main-root given and `git rev-parse --git-common-dir` failed) — refusing to enable dispatch".to_string(),
        );
    };

    let marker = main_root.join(".bee").join("tmp").join(ENABLE_BASENAME);
    // Node's path.join normalizes the WHOLE result to the platform separator,
    // including the forward slashes `git rev-parse --path-format=absolute`
    // hands back on win32. Rust's PathBuf::join only appends, so normalize the
    // rendered string to keep the marker path byte-identical to the .mjs's.
    let marker_str = if cfg!(windows) {
        marker.display().to_string().replace('/', "\\")
    } else {
        marker.display().to_string()
    };

    let main_root_str = main_root.display().to_string();

    if marker.exists() {
        interlock_object(
            true,
            Some(&marker_str),
            Some(&main_root_str),
            format!("owner enable marker present ({marker_str}) — dispatch may build a dispatchable set this iteration"),
        )
    } else {
        interlock_object(
            false,
            Some(&marker_str),
            Some(&main_root_str),
            format!("no owner enable marker at {marker_str} — dispatch MUST NOT build a dispatchable set (D10). The owner enables the loop with: touch {marker_str}"),
        )
    }
}

fn interlock(flags: &[&str]) -> ExitCode {
    let mut explicit: Option<&str> = None;
    let mut i = 0usize;
    while i < flags.len() {
        if flags[i] == "--main-root" {
            explicit = flags.get(i + 1).copied();
            i += 2;
            continue;
        }
        i += 1;
    }

    let obj = enable_marker_state(explicit);
    let enabled = obj.get("enabled").and_then(Value::as_bool) == Some(true);
    let main_root_missing = obj.get("main_root").map(Value::is_null).unwrap_or(true);
    emit(obj);

    if main_root_missing {
        ExitCode::from(1)
    } else if enabled {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(3)
    }
}

/// tmux-herding-transport D1: which terminal multiplexer bee reaches a worker
/// pane through. Selected by ONE config key (`herding.transport`) and never by
/// sniffing the environment — a session nested in both tools must not pick by
/// accident.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TransportKind {
    Herdr,
    Tmux,
}

impl TransportKind {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            TransportKind::Herdr => "herdr",
            TransportKind::Tmux => "tmux",
        }
    }
}

/// tmux-herding-transport D1: read `herding.transport` out of an already-parsed
/// `.bee/config.json`. Absent is herdr — the byte-identical default. Anything
/// that is not one of the two legal spellings is a typed refusal naming both,
/// never a silent fallback: a typo'd transport must not quietly run the other
/// one.
pub(crate) fn transport_kind(cfg: &Value) -> Result<TransportKind, String> {
    match cfg.get("herding").and_then(|h| h.get("transport")) {
        None => Ok(TransportKind::Herdr),
        Some(Value::String(s)) if s == "herdr" => Ok(TransportKind::Herdr),
        Some(Value::String(s)) if s == "tmux" => Ok(TransportKind::Tmux),
        Some(other) => Err(format!(
            "herding.transport is {other} — the only legal values are \"herdr\" and \"tmux\""
        )),
    }
}

/// `transport_kind` over `<main_root>/.bee/config.json`. A missing or
/// unparseable file reads as herdr — the same fail-open posture
/// `read_command_template_tokens` below takes for the same file, so a repo with
/// no config at all keeps the pre-tmux behavior exactly.
pub(crate) fn transport_kind_at(main_root: &Path) -> Result<TransportKind, String> {
    let path = main_root.join(".bee").join("config.json");
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return Ok(TransportKind::Herdr);
    };
    let Ok(cfg) = serde_json::from_str::<Value>(&raw) else {
        return Ok(TransportKind::Herdr);
    };
    transport_kind(&cfg)
}

/// The herdr-only spelling every pre-tmux caller and test uses: unchanged
/// behavior, now one delegation deep. Production reaches the probe through
/// `transport_state_for` with the configured kind, so this name survives for
/// its callers' sake (tests, and any herdr-only caller added later).
#[allow(dead_code)]
pub(crate) fn transport_state_with(lookup: impl Fn(&str) -> Option<String>) -> Map<String, Value> {
    transport_state_for(TransportKind::Herdr, lookup)
}

/// tmux-herding-transport D1: the transport-reachability probe for a KNOWN
/// transport. `kind` comes from the config key, never from the env — with the
/// key absent this is the herdr arm and the tmux variables are never read.
pub(crate) fn transport_state_for(
    kind: TransportKind,
    lookup: impl Fn(&str) -> Option<String>,
) -> Map<String, Value> {
    let (ready, reason, pane_val) = match kind {
        TransportKind::Herdr => {
            let herdr_env = lookup("HERDR_ENV");
            let pane_id = lookup("HERDR_PANE_ID").filter(|s| !s.trim().is_empty());

            match (herdr_env.as_deref(), pane_id) {
                (Some("1"), Some(pid)) => (
                    true,
                    format!("HERDR_ENV=1 and HERDR_PANE_ID={pid} are set"),
                    Value::String(pid),
                ),
                (Some("1"), None) => (
                    false,
                    "HERDR_ENV=1 is set but HERDR_PANE_ID is empty or not set".to_string(),
                    Value::Null,
                ),
                (Some(other), pid) => (
                    false,
                    format!("HERDR_ENV is {other:?} (expected '1')"),
                    pid.map(Value::String).unwrap_or(Value::Null),
                ),
                (None, pid) => (
                    false,
                    "HERDR_ENV is not set — this session is not inside a herdr pane".to_string(),
                    pid.map(Value::String).unwrap_or(Value::Null),
                ),
            }
        }
        TransportKind::Tmux => {
            // tmux exports $TMUX (socket,pid,session) to every pane and
            // $TMUX_PANE (the pane id) — both non-empty is "inside a pane".
            let tmux = lookup("TMUX").filter(|s| !s.trim().is_empty());
            let pane_id = lookup("TMUX_PANE").filter(|s| !s.trim().is_empty());

            match (tmux, pane_id) {
                (Some(_), Some(pane)) => (
                    true,
                    format!("TMUX and TMUX_PANE={pane} are set"),
                    Value::String(pane),
                ),
                (Some(_), None) => (
                    false,
                    "TMUX is set but TMUX_PANE is empty or not set".to_string(),
                    Value::Null,
                ),
                (None, pane) => (
                    false,
                    "TMUX is not set — this session is not inside a tmux pane".to_string(),
                    pane.map(Value::String).unwrap_or(Value::Null),
                ),
            }
        }
    };

    let mut m = Map::new();
    m.insert("ready".into(), Value::Bool(ready));
    m.insert("reason".into(), Value::String(reason));
    m.insert("pane_id".into(), pane_val);
    // Additive: `bee herding status --json` gains transport.kind; every key
    // above keeps its pre-tmux value.
    m.insert("kind".into(), Value::String(kind.as_str().to_string()));
    m
}

/// The probe `bee herding status` runs. The main root resolves exactly the way
/// the surrounding `status` does (`--main-root`, else the git common dir); an
/// unresolvable root falls open to herdr, and a bad `herding.transport` value
/// is reported as not-ready with the refusal text as the reason — never a
/// panic.
fn transport_state(explicit: Option<&str>) -> Map<String, Value> {
    let kind = match resolve_main_root(explicit) {
        Some(main_root) => transport_kind_at(&main_root),
        None => Ok(TransportKind::Herdr),
    };
    match kind {
        Ok(kind) => transport_state_for(kind, |k| std::env::var(k).ok()),
        Err(reason) => {
            let mut m = Map::new();
            m.insert("ready".into(), Value::Bool(false));
            m.insert("reason".into(), Value::String(reason));
            m.insert("pane_id".into(), Value::Null);
            m.insert("kind".into(), Value::Null);
            m
        }
    }
}

fn parse_brief_filename(name: &str) -> Option<u32> {
    let digits = name.strip_prefix("brief-")?.strip_suffix(".txt")?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse::<u32>().ok()
}

fn compute_job_status(
    bee_dir: &Path,
    job_id: &str,
    job_obj: &Map<String, Value>,
    result_round: Option<u32>,
    pane_id: Option<&str>,
    round: u32,
    now_ms: i64,
    transport: Option<&dyn PaneTransport>,
) -> String {
    // 1. A job with a result file shows its result status
    if let Some(res_round) = result_round {
        let path = mailbox::result_path(bee_dir, job_id, res_round);
        if let crate::fsutil::ReadJson::Parsed(Value::Object(map)) = crate::fsutil::read_json(&path) {
            if let Some(status) = map.get("status").and_then(Value::as_str) {
                return status.to_string();
            }
        }
        return "done".to_string();
    }

    // 2. D2 word: stale activity + Alive -> stalled
    let is_stale_activity = match std::fs::read_to_string(mailbox::activity_path(bee_dir, job_id)) {
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(v) => {
                let round_matches = v
                    .get("round")
                    .and_then(Value::as_u64)
                    .map(|r| r as u32)
                    .map_or(true, |r| r >= round);
                let at_ms = v
                    .get("at")
                    .and_then(Value::as_str)
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.timestamp_millis());
                round_matches
                    && at_ms.map_or(false, |at| {
                        now_ms.saturating_sub(at)
                            > mailbox::ACTIVITY_FRESHNESS_SECS.saturating_mul(1000)
                    })
            }
            Err(_) => false,
        },
        Err(_) => false,
    };

    let is_alive = match (transport, pane_id) {
        (Some(t), Some(p)) => matches!(t.process_info(p), run::Liveness::Alive { .. }),
        _ => false,
    };

    if is_stale_activity && is_alive {
        mailbox::transition_status(bee_dir, job_id, "stalled");
        return "stalled".to_string();
    }

    // 3. transition_status gives recovered once; else the existing word
    let last_status = job_obj.get("last_status").and_then(Value::as_str);
    if last_status == Some("stalled") {
        mailbox::transition_status(bee_dir, job_id, "recovered");
        return "recovered".to_string();
    }

    let existing_word = match std::fs::read_to_string(mailbox::activity_path(bee_dir, job_id))
        .ok()
        .and_then(|t| mailbox::parse_activity_text(&t, round, now_ms))
    {
        Some(mailbox::ActivityState::Working) => "working".to_string(),
        Some(mailbox::ActivityState::Blocked) | Some(mailbox::ActivityState::WaitingInput) => {
            "blocked".to_string()
        }
        Some(mailbox::ActivityState::Idle) => "idle".to_string(),
        Some(mailbox::ActivityState::Exited) => "exited".to_string(),
        None => {
            if let Some(t) = transport {
                t.agent_status(job_id).unwrap_or_else(|| "working".to_string())
            } else {
                "working".to_string()
            }
        }
    };
    mailbox::transition_status(bee_dir, job_id, &existing_word);
    existing_word
}

fn collect_jobs(
    bee_dir: &Path,
    _main_root: &Path,
    transport: Option<&dyn PaneTransport>,
) -> (Vec<Value>, Vec<String>) {
    let mailbox_base = bee_dir.join("mailbox");
    let rd = match std::fs::read_dir(&mailbox_base) {
        Ok(rd) => rd,
        Err(_) => return (Vec::new(), Vec::new()),
    };

    let mut job_dirs: Vec<PathBuf> = rd
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect();
    job_dirs.sort();

    let now_ms = chrono::Utc::now().timestamp_millis();
    let mut jobs_json = Vec::new();
    let mut plain_lines = Vec::new();

    for job_dir in job_dirs {
        let job_id = match job_dir.file_name().and_then(|n| n.to_str()) {
            Some(id) => id.to_string(),
            None => continue,
        };

        let jp = mailbox::job_path(bee_dir, &job_id);
        let job_obj = match crate::fsutil::read_json(&jp) {
            crate::fsutil::ReadJson::Parsed(Value::Object(m)) => m,
            _ => continue,
        };

        let pane_id = job_obj
            .get("pane_id")
            .and_then(Value::as_str)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let entries: Vec<String> = match std::fs::read_dir(&job_dir) {
            Ok(entries) => entries
                .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().into_owned()))
                .collect(),
            Err(_) => Vec::new(),
        };

        let brief_round = entries.iter().filter_map(|n| parse_brief_filename(n)).max();
        let result_round = mailbox::latest_result_round(&entries);
        let job_round = job_obj.get("round").and_then(Value::as_u64).map(|r| r as u32);
        let round = brief_round.or(result_round).or(job_round).unwrap_or(1);

        let (mark_str, mark_reason_str) = match mailbox::read_mark(bee_dir, &job_id) {
            Some((m, r)) => (Some(m.as_str().to_string()), r),
            None => (None, None),
        };

        let status = compute_job_status(
            bee_dir,
            &job_id,
            &job_obj,
            result_round,
            pane_id.as_deref(),
            round,
            now_ms,
            transport,
        );

        let mut job_map = Map::new();
        job_map.insert("job_id".into(), Value::String(job_id.clone()));
        job_map.insert(
            "pane_id".into(),
            pane_id
                .as_ref()
                .map(|p| Value::String(p.clone()))
                .unwrap_or(Value::Null),
        );
        job_map.insert("round".into(), Value::Number(round.into()));
        job_map.insert(
            "mark".into(),
            mark_str
                .as_ref()
                .map(|m| Value::String(m.clone()))
                .unwrap_or(Value::Null),
        );
        job_map.insert(
            "mark_reason".into(),
            mark_reason_str
                .as_ref()
                .map(|r| Value::String(r.clone()))
                .unwrap_or(Value::Null),
        );
        job_map.insert("status".into(), Value::String(status.clone()));

        jobs_json.push(Value::Object(job_map));

        let pane_disp = pane_id.as_deref().unwrap_or("null");
        let mark_disp = mark_str.as_deref().unwrap_or("null");
        let reason_disp = mark_reason_str.as_deref().unwrap_or("null");
        plain_lines.push(format!(
            "{job_id}: pane_id={pane_disp} round={round} mark={mark_disp} mark_reason={reason_disp} status={status}"
        ));
    }

    (jobs_json, plain_lines)
}

pub(crate) fn status_with_panes_and_transport(
    flags: &[&str],
    transport_override: Option<&dyn PaneTransport>,
    live_panes_override: Option<Option<HashSet<String>>>,
) -> (ExitCode, Value, Vec<String>) {
    let mut explicit: Option<&str> = None;
    let mut json = false;
    let mut i = 0usize;
    while i < flags.len() {
        if flags[i] == "--main-root" {
            explicit = flags.get(i + 1).copied();
            i += 2;
            continue;
        }
        if flags[i].starts_with("--main-root=") {
            explicit = Some(&flags[i]["--main-root=".len()..]);
            i += 1;
            continue;
        }
        if flags[i] == "--json" {
            json = true;
            i += 1;
            continue;
        }
        i += 1;
    }

    let main_root = resolve_main_root(explicit);

    let mut sweep_lines = Vec::new();
    if let Some(ref root) = main_root {
        let bee_dir = root.join(".bee");
        let live_panes_opt = match live_panes_override {
            Some(ref lp) => lp.clone(),
            None => wave::live_pane_ids(root),
        };
        match live_panes_opt {
            Some(ref live_panes) => {
                for marked_id in mailbox::mark_orphans(&bee_dir, live_panes) {
                    let msg = format!("herding: marked job {marked_id} interrupted (process_restarted)");
                    println!("{msg}");
                    sweep_lines.push(msg);
                }
            }
            None => {
                let msg = "herding: transport cannot list panes; skipping orphan sweep".to_string();
                println!("{msg}");
                sweep_lines.push(msg);
            }
        }
    }

    let mut obj = enable_marker_state(explicit);
    let transport = transport_state(explicit);
    let enabled = obj.get("enabled").and_then(Value::as_bool).unwrap_or(false);
    let transport_ready = transport.get("ready").and_then(Value::as_bool).unwrap_or(false);
    let transport_ready_str = if transport_ready { "ready" } else { "not ready" };
    let transport_reason = transport
        .get("reason")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    obj.insert("transport".into(), Value::Object(transport));

    let mut jobs_val = Vec::new();
    let mut job_plain_lines = Vec::new();

    if let Some(ref root) = main_root {
        let bee_dir = root.join(".bee");
        let transport_box = if transport_override.is_none() {
            job_verbs::transport_for_run(root).ok()
        } else {
            None
        };
        let t_ref = transport_override.or(transport_box.as_deref());
        let (j_val, j_lines) = collect_jobs(&bee_dir, root, t_ref);
        jobs_val = j_val;
        job_plain_lines = j_lines;
    }

    obj.insert("jobs".into(), Value::Array(jobs_val));

    if json {
        emit(obj.clone());
    } else {
        println!("herding: enabled={enabled} transport={transport_ready_str} ({transport_reason})");
        for line in &job_plain_lines {
            println!("{line}");
        }
    }

    (ExitCode::SUCCESS, Value::Object(obj), job_plain_lines)
}

fn status(flags: &[&str]) -> ExitCode {
    status_with_panes_and_transport(flags, None, None).0
}

// ═══════════════════════════════════════════════════════════════════════════
// the cockpit shell scripts' inline JSON readers
// ═══════════════════════════════════════════════════════════════════════════

/// Shared by the `command_template` CLI verb below (the shell scripts'
/// external reader) and `control_loop::resolve_iteration_argv` (the same
/// read done in-process, no child `bee` call): `herding.<key>` from
/// `<main_root>/.bee/config.json` as a JSON array of argv-token strings.
/// `None` covers every "fall back to the hardcoded default" case
/// control-loop.sh's `read_command_template` covered by printing nothing: a
/// missing file, a missing key, a non-array value, an empty array, a
/// non-string element, or an element containing a newline (the line-per-token
/// protocol `command_template` still prints over cannot carry one, so the
/// same rejection is kept here even though nothing crosses a line in the
/// in-process caller).
pub(crate) fn read_command_template_tokens(main_root: &Path, key: &str) -> Option<Vec<String>> {
    let path = main_root.join(".bee").join("config.json");
    let raw = std::fs::read_to_string(&path).ok()?;
    let cfg: Value = serde_json::from_str(&raw).ok()?;
    let Value::Array(tmpl) = cfg.get("herding")?.get(key)? else { return None };
    if tmpl.is_empty() {
        return None;
    }
    let mut out = Vec::with_capacity(tmpl.len());
    for t in tmpl {
        match t.as_str() {
            Some(s) if !s.contains('\n') => out.push(s.to_string()),
            _ => return None,
        }
    }
    Some(out)
}

/// control-loop.sh `read_command_template KEY`: print `herding.<KEY>` from
/// `<main-root>/.bee/config.json`, one argv token per line. A missing file, a
/// missing key, a non-array value, an empty array, a non-string element, or an
/// element containing a newline ALL print nothing and exit 0 — the caller then
/// falls back to its hardcoded default (D4: no config keys => byte-equivalent
/// command). Refusing newline-bearing tokens is load-bearing: the line-per-token
/// protocol cannot carry one.
fn command_template(flags: &[&str]) -> ExitCode {
    let mut key: Option<&str> = None;
    let mut explicit: Option<&str> = None;
    let mut i = 0usize;
    while i < flags.len() {
        if flags[i] == "--main-root" {
            explicit = flags.get(i + 1).copied();
            i += 2;
            continue;
        }
        if key.is_none() {
            key = Some(flags[i]);
        }
        i += 1;
    }
    let Some(key) = key else { return ExitCode::SUCCESS };
    let Some(main_root) = resolve_main_root(explicit) else { return ExitCode::SUCCESS };
    if let Some(tokens) = read_command_template_tokens(&main_root, key) {
        for s in tokens {
            println!("{s}");
        }
    }
    ExitCode::SUCCESS
}

pub(crate) fn read_stdin() -> String {
    use std::io::Read;
    let mut s = String::new();
    let _ = std::io::stdin().read_to_string(&mut s);
    s
}

/// bootstrap-cockpit.sh `json_result <dotted.path.under.result>`: read one herdr
/// JSON response on stdin and print the value at `result.<path>`, or fail loudly
/// (surfacing herdr's own `.error.message`). `--context NAME` supplies the
/// message prefix the shell script used to hardcode.
fn herdr_result(flags: &[&str]) -> ExitCode {
    result_reader(flags, "bee herding herdr-result", "herdr-result", "herdr")
}

/// `bee herding result <dotted.path>` (tmux-herding-cockpit D2) — the
/// transport-neutral twin of `herdr-result`. It reads the pane verbs' OWN
/// envelope (`{"ok":…,"transport":…,"result":{…}}`), whose `result` and
/// `error.message` keys are the same two the herdr reader already walks, so
/// both verbs share one reader rather than two that can drift.
pub(crate) fn envelope_result(flags: &[&str]) -> ExitCode {
    result_reader(flags, "bee herding result", "result", "transport")
}

/// The dotted-path walk both readers share: descend `result.<a>.<b>…`,
/// stopping at `Null` on the first hop that is missing rather than
/// distinguishing "absent" from "null" — the shell callers cannot tell the
/// two apart anyway.
pub(crate) fn walk_result_path(root: &Value, path: &str) -> Value {
    let mut v = root.get("result").cloned().unwrap_or(Value::Null);
    for key in path.split('.') {
        v = if v.is_null() { Value::Null } else { v.get(key).cloned().unwrap_or(Value::Null) };
    }
    v
}

/// The body of both readers. `usage_noun` and `subject` are the only things
/// that differ between them, so `herdr-result`'s messages stay byte-for-byte
/// what bootstrap-cockpit.sh has always printed.
fn result_reader(
    flags: &[&str],
    default_context: &str,
    usage_noun: &str,
    subject: &str,
) -> ExitCode {
    let mut path: Option<&str> = None;
    let mut context = default_context;
    let mut i = 0usize;
    while i < flags.len() {
        if flags[i] == "--context" {
            if let Some(c) = flags.get(i + 1) {
                context = c;
            }
            i += 2;
            continue;
        }
        if path.is_none() {
            path = Some(flags[i]);
        }
        i += 1;
    }
    let Some(path) = path else {
        eprintln!("{context}: {usage_noun} needs a dotted path under .result");
        return ExitCode::FAILURE;
    };
    let s = read_stdin();
    let Ok(r) = serde_json::from_str::<Value>(&s) else {
        eprintln!("{context}: unparseable {subject} output: {s}");
        return ExitCode::FAILURE;
    };
    if let Some(err) = r.get("error").filter(|v| !v.is_null()) {
        let msg = err
            .get("message")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| serde_json::to_string(err).unwrap_or_default());
        eprintln!("{context}: {subject} error: {msg}");
        return ExitCode::FAILURE;
    }
    let v = walk_result_path(&r, path);
    let printed = match &v {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        other => Some(other.to_string()),
    };
    match printed {
        Some(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        None => {
            eprintln!("{context}: {subject} response missing result.{path}: {s}");
            ExitCode::FAILURE
        }
    }
}

/// bootstrap-cockpit.sh `find_dispatch_pane`: read a `herdr pane list` response
/// on stdin and print the pane_id of the first pane carrying `--label`, or
/// nothing. SILENT on every parse trouble (always exit 0) — the idempotency
/// check is a refuse-if-sure check, never a reason to block a bootstrap over a
/// herdr response shape mismatch.
fn herdr_pane_id(flags: &[&str]) -> ExitCode {
    let mut label: Option<&str> = None;
    let mut i = 0usize;
    while i < flags.len() {
        if flags[i] == "--label" {
            label = flags.get(i + 1).copied();
            i += 2;
            continue;
        }
        i += 1;
    }
    let Some(label) = label else { return ExitCode::SUCCESS };
    let Ok(r) = serde_json::from_str::<Value>(&read_stdin()) else { return ExitCode::SUCCESS };
    if r.get("error").is_some_and(|v| !v.is_null()) {
        return ExitCode::SUCCESS;
    }
    let result = r.get("result").cloned().unwrap_or(Value::Null);
    let panes = match &result {
        Value::Array(a) => a.clone(),
        Value::Object(o) => match o.get("panes") {
            Some(Value::Array(a)) => a.clone(),
            _ => return ExitCode::SUCCESS,
        },
        _ => return ExitCode::SUCCESS,
    };
    if let Some(hit) = panes.iter().find(|p| p.get("label").and_then(Value::as_str) == Some(label))
    {
        println!("{}", hit.get("pane_id").and_then(Value::as_str).unwrap_or(""));
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels_for(text: &str) -> Vec<&'static str> {
        let lowered = text.to_lowercase();
        FLAG_RULES.iter().filter(|r| (r.matcher)(&lowered)).map(|r| r.label).collect()
    }

    #[test]
    fn word_boundaries_match_the_js_regex_semantics() {
        assert!(has_word("run an audit now", "audit"));
        assert!(!has_word("auditor of things", "audit"));
        // \bauth\b must not fire on "authorization" (the .mjs's (?!ori))
        assert!(!has_word("authorization rules", "auth"));
        assert!(has_word("auth flow", "auth"));
        // non-ASCII letters are not \w in JS either, so they are boundaries
        assert!(has_word("đauditđ", "audit"));
    }

    #[test]
    fn hard_gate_flags_fire_on_their_own_vocabulary() {
        assert_eq!(labels_for("Add authentication to the pane"), vec!["auth"]);
        assert_eq!(labels_for("phân quyền cho editor"), vec!["authorization"]);
        assert_eq!(labels_for("schema migration for cells"), vec!["data model / data loss"]);
        assert_eq!(labels_for("security review of hooks"), vec!["audit/security"]);
        assert_eq!(
            labels_for("call a third-party service"),
            vec!["external systems / external provider"]
        );
        assert_eq!(
            labels_for("skip validation in the guard"),
            vec!["weakening/deleting/replacing existing proof (validation removal)"]
        );
    }

    #[test]
    fn soft_flags_and_the_bounded_gap_behave() {
        assert_eq!(labels_for("a breaking change to the CLI"), vec!["public contracts"]);
        assert!(labels_for("windows and macos parity").contains(&"cross-platform"));
        // more than 40 chars between the two platform words: no match
        let far = format!("windows{}linux", "x".repeat(41));
        assert!(!labels_for(&far).contains(&"cross-platform"));
        assert_eq!(labels_for("rename a button label"), Vec::<&str>::new());
    }

    #[test]
    fn no_flags_classifies_small_and_safe() {
        let obj = result_object("p-1", "small", Vec::new(), true, "x".into());
        assert_eq!(obj.get("lane_safe"), Some(&Value::Bool(true)));
        // field order is the .mjs's, which role-dispatch.md quotes verbatim
        let keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        assert_eq!(keys, vec!["pbi", "lane", "hard_gate_flags", "lane_safe", "reason"]);
    }

    #[test]
    fn interlock_object_keeps_the_mjs_field_order() {
        let obj = interlock_object(false, None, None, "x".into());
        let keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        assert_eq!(keys, vec!["enabled", "marker", "main_root", "reason"]);
    }

    #[test]
    fn unknown_herding_subcommand_returns_none() {
        let args: Vec<OsString> = ["herding", "nope"].iter().map(OsString::from).collect();
        assert!(try_native(&args).is_none());
        let other: Vec<OsString> = ["status"].iter().map(OsString::from).collect();
        assert!(try_native(&other).is_none());
    }

    #[test]
    fn transport_state_probe_with_both_env_states() {
        // Ready state: HERDR_ENV == 1 and HERDR_PANE_ID is non-empty
        let ready = transport_state_with(|k| match k {
            "HERDR_ENV" => Some("1".to_string()),
            "HERDR_PANE_ID" => Some("w4:p7".to_string()),
            _ => None,
        });
        assert_eq!(ready.get("ready"), Some(&Value::Bool(true)));
        assert_eq!(ready.get("pane_id"), Some(&Value::String("w4:p7".to_string())));
        assert!(ready.get("reason").and_then(Value::as_str).unwrap().contains("HERDR_ENV=1 and HERDR_PANE_ID=w4:p7"));

        // Not ready state: HERDR_ENV missing
        let not_ready_missing = transport_state_with(|k| match k {
            "HERDR_PANE_ID" => Some("w4:p7".to_string()),
            _ => None,
        });
        assert_eq!(not_ready_missing.get("ready"), Some(&Value::Bool(false)));
        assert!(not_ready_missing.get("reason").and_then(Value::as_str).unwrap().contains("HERDR_ENV is not set"));

        // Not ready state: HERDR_ENV != 1
        let not_ready_other = transport_state_with(|k| match k {
            "HERDR_ENV" => Some("0".to_string()),
            "HERDR_PANE_ID" => Some("w4:p7".to_string()),
            _ => None,
        });
        assert_eq!(not_ready_other.get("ready"), Some(&Value::Bool(false)));

        // Not ready state: HERDR_PANE_ID empty
        let not_ready_empty_pane = transport_state_with(|k| match k {
            "HERDR_ENV" => Some("1".to_string()),
            "HERDR_PANE_ID" => Some("".to_string()),
            _ => None,
        });
        assert_eq!(not_ready_empty_pane.get("ready"), Some(&Value::Bool(false)));
        assert_eq!(not_ready_empty_pane.get("pane_id"), Some(&Value::Null));
    }

    // ── tmux-herding-transport D1: the config key and the tmux probe arm ────

    #[test]
    fn transport_kind_defaults_to_herdr_and_refuses_an_unknown_value() {
        // Absent key (and an absent `herding` block) => herdr, no env read.
        assert_eq!(transport_kind(&serde_json::json!({})), Ok(TransportKind::Herdr));
        assert_eq!(
            transport_kind(&serde_json::json!({"herding": {}})),
            Ok(TransportKind::Herdr)
        );
        // The two legal spellings.
        assert_eq!(
            transport_kind(&serde_json::json!({"herding": {"transport": "herdr"}})),
            Ok(TransportKind::Herdr)
        );
        assert_eq!(
            transport_kind(&serde_json::json!({"herding": {"transport": "tmux"}})),
            Ok(TransportKind::Tmux)
        );
        // Anything else refuses, naming exactly the two legal values.
        let err = transport_kind(&serde_json::json!({"herding": {"transport": "nope"}}))
            .unwrap_err();
        assert_eq!(
            err,
            "herding.transport is \"nope\" — the only legal values are \"herdr\" and \"tmux\""
        );
        let err_nonstring =
            transport_kind(&serde_json::json!({"herding": {"transport": 3}})).unwrap_err();
        assert!(err_nonstring.contains("\"herdr\""), "got {err_nonstring}");
        assert!(err_nonstring.contains("\"tmux\""), "got {err_nonstring}");
    }

    #[test]
    fn transport_kind_at_reads_the_config_and_fails_open() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // No .bee/config.json at all => herdr (same posture as
        // read_command_template_tokens).
        assert_eq!(transport_kind_at(root), Ok(TransportKind::Herdr));

        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("config.json"), "{ not json").unwrap();
        assert_eq!(transport_kind_at(root), Ok(TransportKind::Herdr));

        std::fs::write(
            root.join(".bee").join("config.json"),
            "{\"herding\":{\"transport\":\"tmux\"}}",
        )
        .unwrap();
        assert_eq!(transport_kind_at(root), Ok(TransportKind::Tmux));

        std::fs::write(
            root.join(".bee").join("config.json"),
            "{\"herding\":{\"transport\":\"nope\"}}",
        )
        .unwrap();
        assert!(transport_kind_at(root).is_err());
    }

    #[test]
    fn transport_state_for_tmux_reads_only_the_tmux_vars() {
        let env = |tmux: Option<&str>, pane: Option<&str>| {
            let tmux = tmux.map(str::to_string);
            let pane = pane.map(str::to_string);
            move |k: &str| match k {
                "TMUX" => tmux.clone(),
                "TMUX_PANE" => pane.clone(),
                // A herdr pane's variables are present and must not be read.
                "HERDR_ENV" => Some("1".to_string()),
                "HERDR_PANE_ID" => Some("w4:p7".to_string()),
                _ => None,
            }
        };

        // Ready: both set.
        let ready = transport_state_for(
            TransportKind::Tmux,
            env(Some("/tmp/tmux-1000/default,42,0"), Some("%3")),
        );
        assert_eq!(ready.get("ready"), Some(&Value::Bool(true)));
        assert_eq!(
            ready.get("reason"),
            Some(&Value::String("TMUX and TMUX_PANE=%3 are set".to_string()))
        );
        assert_eq!(ready.get("pane_id"), Some(&Value::String("%3".to_string())));
        assert_eq!(ready.get("kind"), Some(&Value::String("tmux".to_string())));

        // TMUX set, TMUX_PANE missing / empty.
        for pane in [None, Some(""), Some("   ")] {
            let m = transport_state_for(
                TransportKind::Tmux,
                env(Some("/tmp/tmux-1000/default,42,0"), pane),
            );
            assert_eq!(m.get("ready"), Some(&Value::Bool(false)));
            assert_eq!(
                m.get("reason"),
                Some(&Value::String(
                    "TMUX is set but TMUX_PANE is empty or not set".to_string()
                ))
            );
            assert_eq!(m.get("pane_id"), Some(&Value::Null));
        }

        // TMUX missing: not ready, and the pane id still rides along.
        let no_tmux = transport_state_for(TransportKind::Tmux, env(None, Some("%3")));
        assert_eq!(no_tmux.get("ready"), Some(&Value::Bool(false)));
        assert_eq!(
            no_tmux.get("reason"),
            Some(&Value::String(
                "TMUX is not set — this session is not inside a tmux pane".to_string()
            ))
        );
        assert_eq!(no_tmux.get("pane_id"), Some(&Value::String("%3".to_string())));
        assert_eq!(no_tmux.get("kind"), Some(&Value::String("tmux".to_string())));
    }

    #[test]
    fn transport_state_carries_the_kind_on_the_herdr_arm_too() {
        let herdr = transport_state_with(|k| match k {
            "HERDR_ENV" => Some("1".to_string()),
            "HERDR_PANE_ID" => Some("w4:p7".to_string()),
            // Present but never consulted on the herdr arm (D1: no auto-detect).
            "TMUX" => Some("/tmp/tmux-1000/default,42,0".to_string()),
            "TMUX_PANE" => Some("%3".to_string()),
            _ => None,
        });
        assert_eq!(herdr.get("kind"), Some(&Value::String("herdr".to_string())));
        assert_eq!(herdr.get("ready"), Some(&Value::Bool(true)));
        assert_eq!(
            herdr.get("reason"),
            Some(&Value::String("HERDR_ENV=1 and HERDR_PANE_ID=w4:p7 are set".to_string()))
        );
        // Additive key: it lands after the three pre-tmux keys, which keep
        // their order and their values.
        let keys: Vec<&str> = herdr.keys().map(String::as_str).collect();
        assert_eq!(keys, vec!["ready", "reason", "pane_id", "kind"]);
    }

    #[test]
    fn herding_status_subcommand_dispatches_in_try_native() {
        let args: Vec<OsString> = ["herding", "status", "--json"].iter().map(OsString::from).collect();
        assert_eq!(try_native(&args), Some(ExitCode::SUCCESS));
    }

    struct MockTransport {
        alive_pid: Option<u32>,
    }

    impl PaneTransport for MockTransport {
        fn name(&self) -> &'static str { "mock" }
        fn pane_current(&self) -> Result<String, String> { Ok("p1".into()) }
        fn pane_layout(&self, _pane_id: &str) -> Option<Vec<run::PaneGeom>> { None }
        fn pane_split(&self, _pane_id: &str, _dir: &str, _ratio: f64, _cwd: &Path) -> Result<String, String> { Ok("p1".into()) }
        fn tab_create(&self, _ws: &str, _cwd: &Path, _label: &str) -> Result<String, String> { Ok("p1".into()) }
        fn pane_run(&self, _pane: &str, _cmd: &str) -> Result<(), String> { Ok(()) }
        fn agent_start(&self, _job: &str, _kind: &str, _pane: &str, _args: &[String]) -> Result<(), String> { Ok(()) }
        fn agent_prompt(&self, _job: &str, _prompt: &str, _working: &str, _timeout_ms: u64) -> Result<(), String> { Ok(()) }
        fn agent_wait(&self, _job: &str, _timeout_ms: u64) -> Option<String> { None }
        fn agent_status(&self, _job: &str) -> Option<String> { None }
        fn pane_close(&self, _pane: &str) -> Result<(), String> { Ok(()) }
        fn pane_alive(&self, _pane: &str) -> bool { self.alive_pid.is_some() }
        fn pane_read(&self, _pane: &str) -> Result<String, String> { Ok(String::new()) }
        fn process_info(&self, _pane_id: &str) -> run::Liveness {
            match self.alive_pid {
                Some(pid) => run::Liveness::Alive { pid },
                None => run::Liveness::Absent,
            }
        }
        fn pane_send_key(&self, _pane: &str, _key: &str) -> Result<(), String> { Ok(()) }
    }

    #[test]
    fn status_lists_job_with_mark_null() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let root_str = root.to_str().unwrap();
        let bee_dir = root.join(".bee");
        let job_dir = bee_dir.join("mailbox").join("job-normal");
        std::fs::create_dir_all(&job_dir).unwrap();

        let job_spec = serde_json::json!({
            "job_id": "job-normal",
            "pane_id": "w4:p1",
            "round": 1
        });
        std::fs::write(job_dir.join("job.json"), serde_json::to_string(&job_spec).unwrap()).unwrap();

        let live_panes: HashSet<String> = ["w4:p1".to_string()].into_iter().collect();
        let (exit, val, _plain_lines) = status_with_panes_and_transport(
            &["--main-root", root_str, "--json"],
            None,
            Some(Some(live_panes)),
        );
        assert_eq!(exit, ExitCode::SUCCESS);

        let jobs = val.get("jobs").and_then(Value::as_array).expect("jobs array present");
        assert_eq!(jobs.len(), 1);
        let job = &jobs[0];
        assert_eq!(job.get("job_id"), Some(&Value::String("job-normal".to_string())));
        assert_eq!(job.get("pane_id"), Some(&Value::String("w4:p1".to_string())));
        assert_eq!(job.get("round"), Some(&serde_json::json!(1)));
        assert_eq!(job.get("mark"), Some(&Value::Null));
        assert_eq!(job.get("mark_reason"), Some(&Value::Null));
        assert_eq!(job.get("status"), Some(&Value::String("working".to_string())));

        // Check non-json plain output format
        let (_exit, _val, lines) = status_with_panes_and_transport(
            &["--main-root", root_str],
            None,
            Some(Some(["w4:p1".to_string()].into_iter().collect())),
        );
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0],
            "job-normal: pane_id=w4:p1 round=1 mark=null mark_reason=null status=working"
        );
    }

    #[test]
    fn status_marks_orphan_job_and_lists_it() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let root_str = root.to_str().unwrap();
        let bee_dir = root.join(".bee");
        let job_dir = bee_dir.join("mailbox").join("job-orphan");
        std::fs::create_dir_all(&job_dir).unwrap();

        let job_spec = serde_json::json!({
            "job_id": "job-orphan",
            "pane_id": "w4:dead",
            "round": 1
        });
        std::fs::write(job_dir.join("job.json"), serde_json::to_string(&job_spec).unwrap()).unwrap();

        // live_panes is empty, so w4:dead is missing
        let (exit, val, _lines) = status_with_panes_and_transport(
            &["--main-root", root_str, "--json"],
            None,
            Some(Some(HashSet::new())),
        );
        assert_eq!(exit, ExitCode::SUCCESS);

        let jobs = val.get("jobs").and_then(Value::as_array).expect("jobs array present");
        assert_eq!(jobs.len(), 1);
        let job = &jobs[0];
        assert_eq!(job.get("job_id"), Some(&Value::String("job-orphan".to_string())));
        assert_eq!(job.get("mark"), Some(&Value::String("interrupted".to_string())));
        assert_eq!(job.get("mark_reason"), Some(&Value::String("process_restarted".to_string())));

        let (mark, reason) = mailbox::read_mark(&bee_dir, "job-orphan").expect("mark exists on disk");
        assert_eq!(mark, mailbox::Mark::Interrupted);
        assert_eq!(reason, Some("process_restarted".to_string()));
    }

    #[test]
    fn status_skips_orphan_sweep_when_transport_cannot_list_panes() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let root_str = root.to_str().unwrap();
        let bee_dir = root.join(".bee");
        let job_dir = bee_dir.join("mailbox").join("job-orphan-skipped");
        std::fs::create_dir_all(&job_dir).unwrap();

        let job_spec = serde_json::json!({
            "job_id": "job-orphan-skipped",
            "pane_id": "w4:dead",
            "round": 1
        });
        std::fs::write(job_dir.join("job.json"), serde_json::to_string(&job_spec).unwrap()).unwrap();

        // live_panes_override = Some(None) represents transport cannot list panes
        let (exit, val, _lines) = status_with_panes_and_transport(
            &["--main-root", root_str, "--json"],
            None,
            Some(None),
        );
        assert_eq!(exit, ExitCode::SUCCESS);

        // Job was NOT marked on disk
        assert_eq!(mailbox::read_mark(&bee_dir, "job-orphan-skipped"), None);

        let jobs = val.get("jobs").and_then(Value::as_array).unwrap();
        assert_eq!(jobs[0].get("mark"), Some(&Value::Null));
    }

    #[test]
    fn status_computes_stalled_and_recovered_transitions() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let root_str = root.to_str().unwrap();
        let bee_dir = root.join(".bee");
        let job_dir = bee_dir.join("mailbox").join("job-stall-rec");
        std::fs::create_dir_all(&job_dir).unwrap();

        let job_spec = serde_json::json!({
            "job_id": "job-stall-rec",
            "pane_id": "w4:alive",
            "round": 1
        });
        std::fs::write(job_dir.join("job.json"), serde_json::to_string(&job_spec).unwrap()).unwrap();

        let mock_transport = MockTransport { alive_pid: Some(4242) };
        let live_panes: HashSet<String> = ["w4:alive".to_string()].into_iter().collect();

        // 1. Stale activity (>120s ago) + alive process -> stalled
        let stale_time = chrono::Utc::now() - chrono::Duration::seconds(mailbox::ACTIVITY_FRESHNESS_SECS + 30);
        let stale_act = serde_json::json!({
            "round": 1,
            "at": stale_time.to_rfc3339(),
            "state": "working"
        });
        std::fs::write(job_dir.join("activity.json"), serde_json::to_string(&stale_act).unwrap()).unwrap();

        let (_exit, val1, _lines) = status_with_panes_and_transport(
            &["--main-root", root_str, "--json"],
            Some(&mock_transport),
            Some(Some(live_panes.clone())),
        );
        let jobs1 = val1.get("jobs").and_then(Value::as_array).unwrap();
        assert_eq!(jobs1[0].get("status"), Some(&Value::String("stalled".to_string())));

        // 2. Activity resumes (fresh) -> recovered once
        let fresh_time = chrono::Utc::now();
        let fresh_act = serde_json::json!({
            "round": 1,
            "at": fresh_time.to_rfc3339(),
            "state": "working"
        });
        std::fs::write(job_dir.join("activity.json"), serde_json::to_string(&fresh_act).unwrap()).unwrap();

        let (_exit, val2, _lines) = status_with_panes_and_transport(
            &["--main-root", root_str, "--json"],
            Some(&mock_transport),
            Some(Some(live_panes.clone())),
        );
        let jobs2 = val2.get("jobs").and_then(Value::as_array).unwrap();
        assert_eq!(jobs2[0].get("status"), Some(&Value::String("recovered".to_string())));

        // 3. Next read -> working
        let (_exit, val3, _lines) = status_with_panes_and_transport(
            &["--main-root", root_str, "--json"],
            Some(&mock_transport),
            Some(Some(live_panes)),
        );
        let jobs3 = val3.get("jobs").and_then(Value::as_array).unwrap();
        assert_eq!(jobs3[0].get("status"), Some(&Value::String("working".to_string())));
    }

    #[test]
    fn status_shows_result_file_status() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let root_str = root.to_str().unwrap();
        let bee_dir = root.join(".bee");

        let job_done_dir = bee_dir.join("mailbox").join("job-done");
        std::fs::create_dir_all(&job_done_dir).unwrap();
        let job_spec = serde_json::json!({
            "job_id": "job-done",
            "pane_id": "w4:p1",
            "round": 1
        });
        std::fs::write(job_done_dir.join("job.json"), serde_json::to_string(&job_spec).unwrap()).unwrap();
        let result_done = serde_json::json!({
            "status": "done",
            "summary": "finished cleanly",
            "files_changed": [],
            "proof": "cargo test"
        });
        std::fs::write(job_done_dir.join("result-1.json"), serde_json::to_string(&result_done).unwrap()).unwrap();

        let (_exit, val, _lines) = status_with_panes_and_transport(
            &["--main-root", root_str, "--json"],
            None,
            Some(Some(HashSet::new())),
        );
        let jobs = val.get("jobs").and_then(Value::as_array).unwrap();
        assert_eq!(jobs[0].get("status"), Some(&Value::String("done".to_string())));
    }
}
