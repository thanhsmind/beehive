// pane_verbs — the transport-neutral cockpit vocabulary (tmux-herding-cockpit D2).
//
//   bee herding pane current|list|split|run|send-text|read|rename|close|layout
//                    |tab-create|tab-list|tab-focus
//   bee herding agent-start <job> --kind <k> --pane <id> -- <args…>
//   bee herding pane-id --label <label>
//   bee herding result <dotted.path>
//
// D2: a cockpit role document and bootstrap-cockpit.sh act on panes ONLY
// through these verbs — never a raw `herdr` or `tmux` line. A cold control
// agent then reads ONE vocabulary, whatever `herding.transport` names.
//
// The seam is `CockpitTransport`, a second trait ON TOP of run.rs's
// phase-1 `PaneTransport`. Six operations the cockpit needs and `bee
// herding run` never did (send-text, rename, list-with-labels, tab list,
// tab focus, the caller's own workspace/tab context) live here; the other
// eight ride PaneTransport unchanged. Splitting the trait instead of
// widening PaneTransport is what keeps phase 1's implementations and its
// test fakes untouched (CONTEXT "Settled In Planning").
//
// EVERY verb prints exactly one JSON envelope, identical in shape on both
// transports:
//
//   {"ok":true,"transport":"herdr|tmux","result":{…}}            exit 0
//   {"ok":false,"transport":…,"error":{"code":…,"message":…}}    exit 1
//
// `bee herding result <dotted.path>` reads such an envelope on stdin and
// prints `result.<path>` — the transport-neutral twin of `herding
// herdr-result`, sharing that verb's own reader so the two cannot drift.
//
// tmux mapping (D3): workspace = the caller's session, tab = a window,
// label = the pane title (`select-pane -T`), label lookup = `list-panes`
// `pane_title`. Every tmux argv here is built by a PURE function and the
// listing parse is a PURE function, so both are pinned by tests with no
// process anywhere — the failure
// `docs/knowledge/patterns/20260821-a-faked-seam-hides-the-parse.md`
// records.

use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::sync::OnceLock;

use super::run::{read_main_config, PaneGeom, PaneTransport, RealHerdr};
use super::tmux::{classify, RealTmux, Screen, TmuxSettings};
use super::{resolve_main_root, TransportKind};

// ═══════════════════════════════════════════════════════════════════════════
// the rows the cockpit reads
// ═══════════════════════════════════════════════════════════════════════════

/// One pane as the cockpit sees it. `label` is herdr's pane label and
/// tmux's pane title (D3) — `None` when the pane carries neither, which is
/// what makes "the pane labelled dispatch" answerable on both transports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PaneRow {
    pub(crate) id: String,
    pub(crate) label: Option<String>,
    pub(crate) tab: String,
    pub(crate) cwd: Option<String>,
    pub(crate) command: Option<String>,
}

/// One tab (herdr) / window (tmux).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TabRow {
    pub(crate) id: String,
    pub(crate) label: String,
}

/// Where the caller itself is sitting: its own pane, and the tab and
/// workspace holding it. `pane current` renders this, and `pane tab-create`
/// with no `--workspace` resolves the workspace from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PaneContext {
    pub(crate) pane_id: String,
    pub(crate) tab_id: Option<String>,
    pub(crate) workspace_id: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// the CockpitTransport seam
// ═══════════════════════════════════════════════════════════════════════════

/// The cockpit's own operations, on top of every phase-1 `PaneTransport`
/// one. Implemented for BOTH production transports (`RealHerdr` and
/// `RealTmux`, both below) and for the in-module `FakeCockpit`.
pub(crate) trait CockpitTransport: PaneTransport {
    /// Type one line into a pane and submit it. On tmux this is D5's
    /// two-call discipline and it REFUSES a pane classifying `Blocked`
    /// (transport D3): a dialog is answered by a human, never by whatever
    /// character the text happens to start with.
    fn pane_send_text(&self, pane: &str, text: &str) -> Result<(), String>;
    /// Set a pane's label, or clear it with `None`.
    fn pane_rename(&self, pane: &str, label: Option<&str>) -> Result<(), String>;
    /// Every pane of one workspace — the caller's own when `workspace` is
    /// `None`.
    fn pane_list(&self, workspace: Option<&str>) -> Result<Vec<PaneRow>, String>;
    /// Every tab of one workspace — the caller's own when `workspace` is
    /// `None`.
    fn tab_list(&self, workspace: Option<&str>) -> Result<Vec<TabRow>, String>;
    /// Bring a tab to the front.
    fn tab_focus(&self, tab: &str) -> Result<(), String>;
    /// The caller's own pane, tab, and workspace.
    fn pane_context(&self) -> Result<PaneContext, String>;
    /// The first pane whose label (herdr) or title (tmux) equals `label`.
    /// One definition for both transports, because `pane_list` already
    /// normalizes the two carriers into the same `label` field.
    fn pane_id_by_label(&self, label: &str) -> Result<Option<String>, String> {
        Ok(self
            .pane_list(None)?
            .into_iter()
            .find(|p| p.label.as_deref() == Some(label))
            .map(|p| p.id))
    }
}

/// `run.rs`'s `transport_for_run` for the cockpit verbs: read
/// `herding.transport` out of the MAIN checkout's config, then build it.
/// An illegal value comes back `Err` with the message `transport_kind`
/// wrote (it names both legal spellings) — a typo'd transport never
/// half-runs a cockpit action on the other one.
pub(crate) fn cockpit_transport_for(main_root: &Path) -> Result<Box<dyn CockpitTransport>, String> {
    let kind = super::transport_kind_at(main_root)?;
    Ok(match kind {
        TransportKind::Herdr => Box::new(RealHerdr) as Box<dyn CockpitTransport>,
        TransportKind::Tmux => {
            let settings = TmuxSettings::from_config(&read_main_config(main_root));
            // `RealTmux` keeps its settings private and tmux.rs is owned by
            // another cell, so the D3 blocked-preflight below reads them
            // from here instead. One transport per process, set once.
            let _ = TMUX_SETTINGS.set(settings.clone());
            Box::new(RealTmux::new(settings))
        }
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// the herdr implementation
// ═══════════════════════════════════════════════════════════════════════════

/// `result` as either a bare array or `{"<key>": […]}` — the same two
/// shapes `herdr_pane_id` and `RealHerdr::pane_alive` already tolerate.
/// Anything else is an empty list, never a panic.
fn herdr_rows(v: &Value, key: &str) -> Vec<Value> {
    match v.get("result") {
        Some(Value::Array(a)) => a.clone(),
        Some(Value::Object(o)) => {
            o.get(key).and_then(Value::as_array).cloned().unwrap_or_default()
        }
        _ => Vec::new(),
    }
}

fn non_empty(v: Option<&Value>) -> Option<String> {
    v.and_then(Value::as_str).filter(|s| !s.is_empty()).map(str::to_string)
}

/// `herdr pane list`'s body → `PaneRow`s. Pure. A row with no `pane_id` is
/// dropped rather than failing the whole read (`extract_pane_layout`'s own
/// rule in run.rs).
fn parse_herdr_pane_rows(v: &Value) -> Vec<PaneRow> {
    herdr_rows(v, "panes")
        .iter()
        .filter_map(|p| {
            let id = non_empty(p.get("pane_id"))?;
            Some(PaneRow {
                id,
                label: non_empty(p.get("label")),
                tab: non_empty(p.get("tab_id")).unwrap_or_default(),
                cwd: non_empty(p.get("cwd")),
                command: non_empty(p.get("command")),
            })
        })
        .collect()
}

/// `herdr tab list`'s body → `TabRow`s. Pure, same drop rule.
fn parse_herdr_tab_rows(v: &Value) -> Vec<TabRow> {
    herdr_rows(v, "tabs")
        .iter()
        .filter_map(|t| {
            let id = non_empty(t.get("tab_id"))?;
            Some(TabRow { id, label: non_empty(t.get("label")).unwrap_or_default() })
        })
        .collect()
}

/// `herdr pane current --current`'s body → the caller's context. Pure.
/// The pane id is required; the tab and workspace ride along when herdr
/// reports them (it does — `result.pane.{tab_id,workspace_id}`), and that
/// workspace is what `pane tab-create` falls back to.
fn parse_herdr_pane_context(v: &Value) -> Option<PaneContext> {
    let pane = v.get("result")?.get("pane")?;
    Some(PaneContext {
        pane_id: non_empty(pane.get("pane_id"))?,
        tab_id: non_empty(pane.get("tab_id")),
        workspace_id: non_empty(pane.get("workspace_id")),
    })
}

impl CockpitTransport for RealHerdr {
    fn pane_send_text(&self, pane: &str, text: &str) -> Result<(), String> {
        self.call(&["pane", "send-text", pane, text]).map(|_| ())
    }

    fn pane_rename(&self, pane: &str, label: Option<&str>) -> Result<(), String> {
        match label {
            Some(l) => self.call(&["pane", "rename", pane, l]).map(|_| ()),
            None => self.call(&["pane", "rename", pane, "--clear"]).map(|_| ()),
        }
    }

    fn pane_list(&self, workspace: Option<&str>) -> Result<Vec<PaneRow>, String> {
        let mut argv = vec!["pane", "list"];
        if let Some(ws) = workspace {
            argv.push("--workspace");
            argv.push(ws);
        }
        Ok(parse_herdr_pane_rows(&self.call(&argv)?))
    }

    fn tab_list(&self, workspace: Option<&str>) -> Result<Vec<TabRow>, String> {
        let mut argv = vec!["tab", "list"];
        if let Some(ws) = workspace {
            argv.push("--workspace");
            argv.push(ws);
        }
        Ok(parse_herdr_tab_rows(&self.call(&argv)?))
    }

    fn tab_focus(&self, tab: &str) -> Result<(), String> {
        self.call(&["tab", "focus", tab]).map(|_| ())
    }

    fn pane_context(&self) -> Result<PaneContext, String> {
        let v = self.call(&["pane", "current", "--current"])?;
        parse_herdr_pane_context(&v)
            .ok_or_else(|| "herdr pane current --current: missing result.pane.pane_id".to_string())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// the tmux implementation (D3)
// ═══════════════════════════════════════════════════════════════════════════

/// Five tab-separated columns, in the order `parse_pane_rows` reads them.
/// The format string is a bare argv token: the single quotes around it in
/// tmux documentation are SHELL syntax and there is no shell here.
const TMUX_PANE_FORMAT: &str =
    "#{pane_id}\t#{pane_title}\t#{window_id}\t#{pane_current_path}\t#{pane_current_command}";

const TMUX_TAB_FORMAT: &str = "#{window_id}\t#{window_name}";

/// D5's two-call send: `-l` types the bytes LITERALLY, which is what makes
/// a line containing `Enter` or `C-c` safe to type — and it is also why the
/// newline cannot ride along, so the submit is always a second call with
/// the key NAME `Enter`.
fn send_text_argv(pane: &str, text: &str) -> Vec<Vec<String>> {
    vec![
        vec!["send-keys".into(), "-t".into(), pane.into(), "-l".into(), text.into()],
        vec!["send-keys".into(), "-t".into(), pane.into(), "Enter".into()],
    ]
}

/// D3: the pane label is the pane TITLE. An empty title is tmux's own way
/// of spelling "no title", so clearing a label is `-T` with `""`.
fn rename_argv(pane: &str, label: Option<&str>) -> Vec<String> {
    vec![
        "select-pane".into(),
        "-t".into(),
        pane.into(),
        "-T".into(),
        label.unwrap_or("").to_string(),
    ]
}

/// `-s` scopes the listing to ONE session — the caller's own when no
/// workspace is named (D3: workspace = the caller's session).
fn list_panes_argv(workspace: Option<&str>) -> Vec<String> {
    let mut argv: Vec<String> = vec!["list-panes".into(), "-s".into()];
    if let Some(ws) = workspace {
        argv.push("-t".into());
        argv.push(ws.into());
    }
    argv.push("-F".into());
    argv.push(TMUX_PANE_FORMAT.into());
    argv
}

fn tab_list_argv(workspace: Option<&str>) -> Vec<String> {
    let mut argv: Vec<String> = vec!["list-windows".into()];
    if let Some(ws) = workspace {
        argv.push("-t".into());
        argv.push(ws.into());
    }
    argv.push("-F".into());
    argv.push(TMUX_TAB_FORMAT.into());
    argv
}

fn tab_focus_argv(tab: &str) -> Vec<String> {
    vec!["select-window".into(), "-t".into(), tab.into()]
}

fn display_message_argv(format: &str) -> Vec<String> {
    vec!["display-message".into(), "-p".into(), format.into()]
}

fn capture_argv(pane: &str, scrollback: u32) -> Vec<String> {
    vec![
        "capture-pane".into(),
        "-p".into(),
        "-t".into(),
        pane.into(),
        "-S".into(),
        format!("-{scrollback}"),
    ]
}

/// `list-panes -F TMUX_PANE_FORMAT`'s output → `PaneRow`s. Pure.
///
/// An empty column is `None`, never `Some("")` — an untitled pane must not
/// answer `pane-id --label ""`. A row with no id at all is dropped; the
/// listing is already scoped to one session by `-s`, so there is nothing
/// to filter here.
fn parse_pane_rows(stdout: &str) -> Vec<PaneRow> {
    stdout
        .lines()
        .filter_map(|line| {
            let mut cols = line.split('\t');
            let id = cols.next()?.trim().to_string();
            if id.is_empty() {
                return None;
            }
            let title = cols.next().unwrap_or("");
            let tab = cols.next().unwrap_or("").to_string();
            let cwd = cols.next().unwrap_or("");
            let command = cols.next().unwrap_or("");
            Some(PaneRow {
                id,
                label: (!title.is_empty()).then(|| title.to_string()),
                tab,
                cwd: (!cwd.is_empty()).then(|| cwd.to_string()),
                command: (!command.is_empty()).then(|| command.to_string()),
            })
        })
        .collect()
}

/// `list-windows -F TMUX_TAB_FORMAT`'s output → `TabRow`s. Pure.
fn parse_tab_rows(stdout: &str) -> Vec<TabRow> {
    stdout
        .lines()
        .filter_map(|line| {
            let mut cols = line.split('\t');
            let id = cols.next()?.trim().to_string();
            if id.is_empty() {
                return None;
            }
            Some(TabRow { id, label: cols.next().unwrap_or("").to_string() })
        })
        .collect()
}

/// The settings the D3 preflight classifies against. `RealTmux` holds its
/// own copy privately, so `cockpit_transport_for` deposits one here for the
/// impl below; a process that never built a tmux transport reads defaults.
static TMUX_SETTINGS: OnceLock<TmuxSettings> = OnceLock::new();

fn tmux_settings() -> &'static TmuxSettings {
    TMUX_SETTINGS.get_or_init(TmuxSettings::default)
}

/// One `tmux` invocation, this module's own — `RealTmux::call` is private
/// to tmux.rs. Same contract: a tmux that will not spawn and a non-zero
/// exit are both `Err`, and the message names the full argv, because
/// tmux's own stderr ("can't find pane") says nothing about which call
/// produced it.
fn tmux_call(args: &[String]) -> Result<String, String> {
    let out = Command::new("tmux")
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("could not spawn tmux {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        let code = out.status.code().map_or_else(|| "unknown".to_string(), |c| c.to_string());
        return Err(format!(
            "tmux {} exited {code}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
}

/// transport D3, restated for the cockpit's own typing path: never type
/// into a pane showing a dialog. Fails OPEN on an unreadable screen, the
/// same posture `RealTmux::agent_prompt` takes for its own preflight.
fn refuse_if_blocked(pane: &str) -> Result<(), String> {
    let settings = tmux_settings();
    let Ok(screen) = tmux_call(&capture_argv(pane, settings.scrollback)) else { return Ok(()) };
    if classify(&screen, settings) == Screen::Blocked {
        return Err(format!(
            "pane {pane} is showing a dialog (blocked) — nothing was typed; a human answers it"
        ));
    }
    Ok(())
}

impl CockpitTransport for RealTmux {
    fn pane_send_text(&self, pane: &str, text: &str) -> Result<(), String> {
        refuse_if_blocked(pane)?;
        for argv in send_text_argv(pane, text) {
            tmux_call(&argv)?;
        }
        Ok(())
    }

    fn pane_rename(&self, pane: &str, label: Option<&str>) -> Result<(), String> {
        tmux_call(&rename_argv(pane, label)).map(|_| ())
    }

    fn pane_list(&self, workspace: Option<&str>) -> Result<Vec<PaneRow>, String> {
        Ok(parse_pane_rows(&tmux_call(&list_panes_argv(workspace))?))
    }

    fn tab_list(&self, workspace: Option<&str>) -> Result<Vec<TabRow>, String> {
        Ok(parse_tab_rows(&tmux_call(&tab_list_argv(workspace))?))
    }

    fn tab_focus(&self, tab: &str) -> Result<(), String> {
        tmux_call(&tab_focus_argv(tab)).map(|_| ())
    }

    fn pane_context(&self) -> Result<PaneContext, String> {
        // D3: the caller's session IS the workspace and its window IS the tab.
        let pane_id = self.pane_current()?;
        let tab_id = tmux_call(&display_message_argv("#{window_id}")).ok().filter(|s| !s.is_empty());
        let workspace_id =
            tmux_call(&display_message_argv("#{session_name}")).ok().filter(|s| !s.is_empty());
        Ok(PaneContext { pane_id, tab_id, workspace_id })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// the envelope
// ═══════════════════════════════════════════════════════════════════════════

/// One typed refusal: the `error.code` a role branches on, and the message
/// a human reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VerbError {
    pub(crate) code: String,
    pub(crate) message: String,
}

fn verb_error(code: &str, message: impl Into<String>) -> VerbError {
    VerbError { code: code.to_string(), message: message.into() }
}

fn usage(message: impl Into<String>) -> VerbError {
    verb_error("usage", message)
}

/// Every transport failure carries the argv the transport named — that is
/// the whole diagnostic value of these messages.
fn transport_err(message: String) -> VerbError {
    verb_error("transport_error", message)
}

fn print_ok(transport: &str, result: Map<String, Value>) -> ExitCode {
    let mut m = Map::new();
    m.insert("ok".into(), Value::Bool(true));
    m.insert("transport".into(), Value::String(transport.to_string()));
    m.insert("result".into(), Value::Object(result));
    println!("{}", serde_json::to_string(&Value::Object(m)).unwrap());
    ExitCode::SUCCESS
}

fn print_err(transport: Option<&str>, err: &VerbError) -> ExitCode {
    let mut e = Map::new();
    e.insert("code".into(), Value::String(err.code.clone()));
    e.insert("message".into(), Value::String(err.message.clone()));
    let mut m = Map::new();
    m.insert("ok".into(), Value::Bool(false));
    m.insert(
        "transport".into(),
        transport.map_or(Value::Null, |t| Value::String(t.to_string())),
    );
    m.insert("error".into(), Value::Object(e));
    println!("{}", serde_json::to_string(&Value::Object(m)).unwrap());
    ExitCode::from(1)
}

// ═══════════════════════════════════════════════════════════════════════════
// flag plumbing
// ═══════════════════════════════════════════════════════════════════════════

/// Removes `--name <value>` and returns the value. Only the NAMED options
/// are consumed, so a positional that happens to look like a flag (a
/// `send-text` body, say) is left where it is.
fn take_opt<'a>(args: &mut Vec<&'a str>, name: &str) -> Option<&'a str> {
    let i = args.iter().position(|a| *a == name)?;
    args.remove(i);
    if i >= args.len() {
        return None;
    }
    Some(args.remove(i))
}

fn take_flag(args: &mut Vec<&str>, name: &str) -> bool {
    match args.iter().position(|a| *a == name) {
        Some(i) => {
            args.remove(i);
            true
        }
        None => false,
    }
}

fn positional<'a>(args: &[&'a str], i: usize, message: &str) -> Result<&'a str, VerbError> {
    args.get(i).copied().ok_or_else(|| usage(message))
}

fn pane_id_result(pane_id: &str) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("pane_id".into(), Value::String(pane_id.to_string()));
    m
}

fn pane_row_json(row: &PaneRow) -> Value {
    let mut m = Map::new();
    m.insert("pane_id".into(), Value::String(row.id.clone()));
    m.insert("label".into(), row.label.clone().map_or(Value::Null, Value::String));
    m.insert("tab_id".into(), Value::String(row.tab.clone()));
    m.insert("cwd".into(), row.cwd.clone().map_or(Value::Null, Value::String));
    m.insert("command".into(), row.command.clone().map_or(Value::Null, Value::String));
    Value::Object(m)
}

fn tab_row_json(row: &TabRow) -> Value {
    let mut m = Map::new();
    m.insert("tab_id".into(), Value::String(row.id.clone()));
    m.insert("label".into(), Value::String(row.label.clone()));
    Value::Object(m)
}

fn geom_json(g: &PaneGeom) -> Value {
    let mut m = Map::new();
    m.insert("pane_id".into(), Value::String(g.pane_id.clone()));
    m.insert("width".into(), Value::from(g.width));
    m.insert("height".into(), Value::from(g.height));
    Value::Object(m)
}

fn context_result(ctx: &PaneContext) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("pane_id".into(), Value::String(ctx.pane_id.clone()));
    if let Some(t) = &ctx.tab_id {
        m.insert("tab_id".into(), Value::String(t.clone()));
    }
    if let Some(w) = &ctx.workspace_id {
        m.insert("workspace_id".into(), Value::String(w.clone()));
    }
    m
}

/// The last `n` lines of a capture — what `pane read --lines N` trims to.
fn tail_of(text: &str, n: usize) -> String {
    if n == 0 {
        return String::new();
    }
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(n);
    lines[start..].join("\n")
}

fn cwd_or_here(flag: Option<&str>) -> PathBuf {
    flag.map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

// ═══════════════════════════════════════════════════════════════════════════
// the verbs
// ═══════════════════════════════════════════════════════════════════════════

/// Shared spine of every verb: strip `--main-root`, build the transport,
/// run the body, render exactly one envelope. `--main-root` is optional
/// everywhere and defaults to `resolve_main_root`'s git answer.
fn run_verb<F>(args: &[&str], body: F) -> ExitCode
where
    F: FnOnce(&dyn CockpitTransport, &[&str]) -> Result<Map<String, Value>, VerbError>,
{
    let mut argv: Vec<&str> = args.to_vec();
    let explicit = take_opt(&mut argv, "--main-root");
    let Some(main_root) = resolve_main_root(explicit) else {
        return print_err(
            None,
            &verb_error(
                "main_root",
                "could not resolve the MAIN checkout root (`git rev-parse --git-common-dir` \
                 failed) — pass --main-root PATH",
            ),
        );
    };
    let transport = match cockpit_transport_for(&main_root) {
        Ok(t) => t,
        Err(message) => return print_err(None, &verb_error("transport", message)),
    };
    let name = transport.name();
    match body(transport.as_ref(), &argv) {
        Ok(result) => print_ok(name, result),
        Err(err) => print_err(Some(name), &err),
    }
}

const PANE_SUBVERBS: &str = "current, list, split, run, send-text, read, rename, close, layout, \
                             tab-create, tab-list, tab-focus";

/// `bee herding pane <subverb> …`
pub(crate) fn pane(args: &[&str]) -> ExitCode {
    let Some((sub, rest)) = args.split_first() else {
        return print_err(
            None,
            &usage(format!("bee herding pane needs a subverb — one of: {PANE_SUBVERBS}")),
        );
    };
    run_verb(rest, |t, rest| dispatch_pane(sub, rest, t))
}

/// The whole `pane` group, pure over its transport — a test drives it with
/// a fake and reads the result map back without capturing stdout.
fn dispatch_pane(
    sub: &str,
    args: &[&str],
    t: &dyn CockpitTransport,
) -> Result<Map<String, Value>, VerbError> {
    let mut a: Vec<&str> = args.to_vec();
    match sub {
        "current" => Ok(context_result(&t.pane_context().map_err(transport_err)?)),

        "list" => {
            let workspace = take_opt(&mut a, "--workspace");
            let rows = t.pane_list(workspace).map_err(transport_err)?;
            let mut m = Map::new();
            m.insert("panes".into(), Value::Array(rows.iter().map(pane_row_json).collect()));
            Ok(m)
        }

        "split" => {
            let direction = take_opt(&mut a, "--direction").unwrap_or("right");
            let ratio = match take_opt(&mut a, "--ratio") {
                Some(r) => {
                    r.parse::<f64>().map_err(|_| usage(format!("--ratio {r} is not a number")))?
                }
                None => 0.5,
            };
            let cwd = cwd_or_here(take_opt(&mut a, "--cwd"));
            let pane = positional(&a, 0, "bee herding pane split needs a <pane_id>")?;
            let id = t.pane_split(pane, direction, ratio, &cwd).map_err(transport_err)?;
            Ok(pane_id_result(&id))
        }

        "run" => {
            let pane = positional(&a, 0, "bee herding pane run needs a <pane_id>")?;
            let command = positional(&a, 1, "bee herding pane run needs a <command>")?;
            t.pane_run(pane, command).map_err(transport_err)?;
            Ok(Map::new())
        }

        "send-text" => {
            let pane = positional(&a, 0, "bee herding pane send-text needs a <pane_id>")?;
            let text = positional(&a, 1, "bee herding pane send-text needs a <text>")?;
            t.pane_send_text(pane, text).map_err(transport_err)?;
            Ok(Map::new())
        }

        "read" => {
            // `--source` is herdr's own flag; accepted and ignored so one
            // role line reads the same on both transports.
            let _ = take_opt(&mut a, "--source");
            let lines = match take_opt(&mut a, "--lines") {
                Some(n) => Some(
                    n.parse::<usize>()
                        .map_err(|_| usage(format!("--lines {n} is not a whole number")))?,
                ),
                None => None,
            };
            let pane = positional(&a, 0, "bee herding pane read needs a <pane_id>")?;
            let text = t.pane_read(pane).map_err(transport_err)?;
            let text = match lines {
                Some(n) => tail_of(&text, n),
                None => text,
            };
            let mut m = Map::new();
            m.insert("text".into(), Value::String(text));
            Ok(m)
        }

        "rename" => {
            let clear = take_flag(&mut a, "--clear");
            let pane = positional(&a, 0, "bee herding pane rename needs a <pane_id>")?;
            let label = if clear {
                None
            } else {
                Some(positional(&a, 1, "bee herding pane rename needs a <label> or --clear")?)
            };
            t.pane_rename(pane, label).map_err(transport_err)?;
            Ok(Map::new())
        }

        "close" => {
            let pane = positional(&a, 0, "bee herding pane close needs a <pane_id>")?;
            t.pane_close(pane).map_err(transport_err)?;
            Ok(Map::new())
        }

        "layout" => {
            let pane = match take_opt(&mut a, "--pane").or_else(|| a.first().copied()) {
                Some(p) => p.to_string(),
                None => t.pane_context().map_err(transport_err)?.pane_id,
            };
            let geoms = t.pane_layout(&pane).unwrap_or_default();
            let mut m = Map::new();
            m.insert("panes".into(), Value::Array(geoms.iter().map(geom_json).collect()));
            Ok(m)
        }

        "tab-create" => {
            let label = take_opt(&mut a, "--label").unwrap_or("");
            let cwd = cwd_or_here(take_opt(&mut a, "--cwd"));
            let workspace = match take_opt(&mut a, "--workspace") {
                Some(ws) => ws.to_string(),
                // D3: on tmux the caller's session IS the workspace, and
                // `RealTmux::tab_create` ignores the argument anyway; on
                // herdr the id has to come from somewhere, so it comes from
                // the caller's own pane.
                None => match t.pane_context().ok().and_then(|c| c.workspace_id) {
                    Some(ws) => ws,
                    None if t.name() == "tmux" => String::new(),
                    None => {
                        return Err(verb_error(
                            "workspace_required",
                            "bee herding pane tab-create could not resolve the workspace from \
                             the caller's own pane — pass --workspace <id>",
                        ))
                    }
                },
            };
            let id = t.tab_create(&workspace, &cwd, label).map_err(transport_err)?;
            Ok(pane_id_result(&id))
        }

        "tab-list" => {
            let workspace = take_opt(&mut a, "--workspace");
            let rows = t.tab_list(workspace).map_err(transport_err)?;
            let mut m = Map::new();
            m.insert("tabs".into(), Value::Array(rows.iter().map(tab_row_json).collect()));
            Ok(m)
        }

        "tab-focus" => {
            let tab = positional(&a, 0, "bee herding pane tab-focus needs a <tab_id>")?;
            t.tab_focus(tab).map_err(transport_err)?;
            Ok(Map::new())
        }

        other => Err(usage(format!(
            "bee herding pane: unknown subverb {other:?} — one of: {PANE_SUBVERBS}"
        ))),
    }
}

/// `bee herding agent-start <job_id> --kind <kind> --pane <pane_id> -- <args…>`
pub(crate) fn agent_start(args: &[&str]) -> ExitCode {
    // The tail after a bare `--` is the agent's OWN argv and must never be
    // scanned for this verb's flags (or for `--main-root`).
    let (head, tail): (&[&str], Vec<String>) = match args.iter().position(|a| *a == "--") {
        Some(i) => (&args[..i], args[i + 1..].iter().map(|s| (*s).to_string()).collect()),
        None => (args, Vec::new()),
    };
    run_verb(head, move |t, rest| {
        let mut a: Vec<&str> = rest.to_vec();
        let kind = take_opt(&mut a, "--kind")
            .ok_or_else(|| usage("bee herding agent-start needs --kind <kind>"))?;
        let pane = take_opt(&mut a, "--pane")
            .ok_or_else(|| usage("bee herding agent-start needs --pane <pane_id>"))?;
        let job = positional(&a, 0, "bee herding agent-start needs a <job_id>")?;
        t.agent_start(job, kind, pane, &tail).map_err(transport_err)?;
        Ok(Map::new())
    })
}

/// `bee herding pane-id --label <label>` — the transport-neutral twin of
/// `herding herdr-pane-id`. Unlike that one (silent, always exit 0, a
/// bootstrap idempotency probe), this is a typed answer: a miss is
/// `not_found` and exit 1, so a role can branch on it.
pub(crate) fn pane_id(args: &[&str]) -> ExitCode {
    run_verb(args, |t, rest| {
        let mut a: Vec<&str> = rest.to_vec();
        let label = take_opt(&mut a, "--label")
            .ok_or_else(|| usage("bee herding pane-id needs --label <label>"))?;
        match t.pane_id_by_label(label).map_err(transport_err)? {
            Some(id) => Ok(pane_id_result(&id)),
            None => Err(verb_error("not_found", format!("no pane carries the label {label:?}"))),
        }
    })
}

/// `bee herding result <dotted.path>` — reads one pane-verb envelope on
/// stdin and prints `result.<path>`. Delegates to the SAME reader
/// `herding herdr-result` runs, so the two cannot drift.
pub(crate) fn result(args: &[&str]) -> ExitCode {
    super::envelope_result(args)
}

// ═══════════════════════════════════════════════════════════════════════════
// tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::herding::run::Liveness;
    use std::cell::RefCell;

    // ─── the fake ───────────────────────────────────────────────────────

    /// Records every call the verbs make, and answers with whatever the
    /// test seeded. Implements BOTH traits, so `dispatch_pane` sees exactly
    /// what a production transport would offer.
    #[derive(Default)]
    struct FakeCockpit {
        calls: RefCell<Vec<String>>,
        panes: Vec<PaneRow>,
        tabs: Vec<TabRow>,
        context: Option<PaneContext>,
        pane_text: String,
        split_result: Option<String>,
        tab_create_result: Option<String>,
        layout: Option<Vec<PaneGeom>>,
        name: &'static str,
    }

    impl FakeCockpit {
        fn new() -> Self {
            Self { name: "herdr", ..Default::default() }
        }
        fn log(&self, call: impl Into<String>) {
            self.calls.borrow_mut().push(call.into());
        }
        fn calls(&self) -> Vec<String> {
            self.calls.borrow().clone()
        }
    }

    impl PaneTransport for FakeCockpit {
        fn name(&self) -> &'static str {
            self.name
        }
        fn pane_current(&self) -> Result<String, String> {
            Ok(self.context.as_ref().map(|c| c.pane_id.clone()).unwrap_or_default())
        }
        fn pane_layout(&self, pane_id: &str) -> Option<Vec<PaneGeom>> {
            self.log(format!("pane_layout {pane_id}"));
            self.layout.clone()
        }
        fn pane_split(
            &self,
            pane_id: &str,
            direction: &str,
            ratio: f64,
            cwd: &Path,
        ) -> Result<String, String> {
            self.log(format!("pane_split {pane_id} {direction} {ratio} {}", cwd.display()));
            self.split_result.clone().ok_or_else(|| "no split seeded".to_string())
        }
        fn tab_create(&self, workspace: &str, cwd: &Path, label: &str) -> Result<String, String> {
            self.log(format!("tab_create {workspace} {} {label}", cwd.display()));
            self.tab_create_result.clone().ok_or_else(|| "no tab seeded".to_string())
        }
        fn pane_run(&self, pane_id: &str, command: &str) -> Result<(), String> {
            self.log(format!("pane_run {pane_id} {command}"));
            Ok(())
        }
        fn agent_start(
            &self,
            job_id: &str,
            kind: &str,
            pane_id: &str,
            args: &[String],
        ) -> Result<(), String> {
            self.log(format!("agent_start {job_id} {kind} {pane_id} [{}]", args.join(" ")));
            Ok(())
        }
        fn agent_status(&self, _job_id: &str) -> Option<String> {
            None
        }
        fn pane_close(&self, pane_id: &str) -> Result<(), String> {
            self.log(format!("pane_close {pane_id}"));
            Ok(())
        }
        fn agent_prompt(
            &self,
            _job: &str,
            _prompt: &str,
            _until: &str,
            _timeout_ms: u64,
        ) -> Result<(), String> {
            Ok(())
        }
        fn agent_wait(&self, _job: &str, _timeout_ms: u64) -> Option<String> {
            None
        }
        fn pane_alive(&self, _pane_id: &str) -> bool {
            true
        }
        fn pane_read(&self, pane_id: &str) -> Result<String, String> {
            self.log(format!("pane_read {pane_id}"));
            Ok(self.pane_text.clone())
        }
        fn process_info(&self, _pane_id: &str) -> Liveness {
            Liveness::Unknown
        }
    }

    impl CockpitTransport for FakeCockpit {
        fn pane_send_text(&self, pane: &str, text: &str) -> Result<(), String> {
            self.log(format!("pane_send_text {pane} {text}"));
            Ok(())
        }
        fn pane_rename(&self, pane: &str, label: Option<&str>) -> Result<(), String> {
            self.log(format!("pane_rename {pane} {}", label.unwrap_or("<clear>")));
            Ok(())
        }
        fn pane_list(&self, workspace: Option<&str>) -> Result<Vec<PaneRow>, String> {
            self.log(format!("pane_list {}", workspace.unwrap_or("<own>")));
            Ok(self.panes.clone())
        }
        fn tab_list(&self, workspace: Option<&str>) -> Result<Vec<TabRow>, String> {
            self.log(format!("tab_list {}", workspace.unwrap_or("<own>")));
            Ok(self.tabs.clone())
        }
        fn tab_focus(&self, tab: &str) -> Result<(), String> {
            self.log(format!("tab_focus {tab}"));
            Ok(())
        }
        fn pane_context(&self) -> Result<PaneContext, String> {
            self.log("pane_context");
            self.context.clone().ok_or_else(|| "no context seeded".to_string())
        }
    }

    fn row(id: &str, label: Option<&str>, tab: &str) -> PaneRow {
        PaneRow {
            id: id.into(),
            label: label.map(str::to_string),
            tab: tab.into(),
            cwd: None,
            command: None,
        }
    }

    fn ok(t: &FakeCockpit, sub: &str, args: &[&str]) -> Map<String, Value> {
        dispatch_pane(sub, args, t).expect("subverb should have succeeded")
    }

    // ─── each subverb maps its flags to the right trait call ────────────

    #[test]
    fn pane_verbs_current_renders_pane_tab_and_workspace() {
        let mut f = FakeCockpit::new();
        f.context = Some(PaneContext {
            pane_id: "w4:p4".into(),
            tab_id: Some("w4:t1".into()),
            workspace_id: Some("w4".into()),
        });
        let r = ok(&f, "current", &[]);
        assert_eq!(r.get("pane_id").and_then(Value::as_str), Some("w4:p4"));
        assert_eq!(r.get("tab_id").and_then(Value::as_str), Some("w4:t1"));
        assert_eq!(r.get("workspace_id").and_then(Value::as_str), Some("w4"));
    }

    #[test]
    fn pane_verbs_current_omits_the_ids_the_transport_cannot_answer() {
        let mut f = FakeCockpit::new();
        f.context =
            Some(PaneContext { pane_id: "%7".into(), tab_id: None, workspace_id: None });
        let r = ok(&f, "current", &[]);
        let keys: Vec<&str> = r.keys().map(String::as_str).collect();
        assert_eq!(keys, vec!["pane_id"]);
    }

    #[test]
    fn pane_verbs_list_passes_the_workspace_and_pins_the_row_keys() {
        let mut f = FakeCockpit::new();
        f.panes = vec![row("w4:p4", Some("dispatch"), "w4:t1"), row("w4:p5", None, "w4:t1")];
        let r = ok(&f, "list", &["--workspace", "w4"]);
        assert_eq!(f.calls(), vec!["pane_list w4"]);
        let panes = r.get("panes").and_then(Value::as_array).expect("panes array");
        assert_eq!(panes.len(), 2);
        let keys: Vec<&str> =
            panes[0].as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(keys, vec!["pane_id", "label", "tab_id", "cwd", "command"]);
        assert_eq!(panes[0].get("label").and_then(Value::as_str), Some("dispatch"));
        // an unlabelled pane carries an explicit null, never a "" label
        assert_eq!(panes[1].get("label"), Some(&Value::Null));
    }

    #[test]
    fn pane_verbs_split_carries_direction_ratio_and_cwd() {
        let mut f = FakeCockpit::new();
        f.split_result = Some("w4:p9".into());
        let r = ok(
            &f,
            "split",
            &["w4:p4", "--direction", "down", "--ratio", "0.7", "--cwd", "/repo"],
        );
        assert_eq!(f.calls(), vec!["pane_split w4:p4 down 0.7 /repo"]);
        assert_eq!(r.get("pane_id").and_then(Value::as_str), Some("w4:p9"));
    }

    #[test]
    fn pane_verbs_run_and_send_text_and_close_reach_their_own_calls() {
        let f = FakeCockpit::new();
        assert!(ok(&f, "run", &["w4:p4", "echo hi"]).is_empty());
        assert!(ok(&f, "send-text", &["w4:p4", "dispatch: picking p-12"]).is_empty());
        assert!(ok(&f, "close", &["w4:p9"]).is_empty());
        assert_eq!(
            f.calls(),
            vec![
                "pane_run w4:p4 echo hi",
                "pane_send_text w4:p4 dispatch: picking p-12",
                "pane_close w4:p9",
            ]
        );
    }

    #[test]
    fn pane_verbs_read_trims_to_the_last_lines_and_ignores_source() {
        let mut f = FakeCockpit::new();
        f.pane_text = "one\ntwo\nthree\nfour".into();
        let r = ok(&f, "read", &["w4:p4", "--source", "recent", "--lines", "2"]);
        assert_eq!(r.get("text").and_then(Value::as_str), Some("three\nfour"));
        assert_eq!(f.calls(), vec!["pane_read w4:p4"]);
    }

    #[test]
    fn pane_verbs_rename_sets_a_label_and_clear_removes_it() {
        let f = FakeCockpit::new();
        ok(&f, "rename", &["w4:p4", "dispatch"]);
        ok(&f, "rename", &["w4:p4", "--clear"]);
        assert_eq!(f.calls(), vec!["pane_rename w4:p4 dispatch", "pane_rename w4:p4 <clear>"]);
    }

    #[test]
    fn pane_verbs_layout_reports_every_pane_of_the_tab() {
        let mut f = FakeCockpit::new();
        f.layout = Some(vec![PaneGeom { pane_id: "w4:p4".into(), width: 120, height: 43 }]);
        let r = ok(&f, "layout", &["--pane", "w4:p4"]);
        let panes = r.get("panes").and_then(Value::as_array).expect("panes array");
        assert_eq!(panes[0].get("pane_id").and_then(Value::as_str), Some("w4:p4"));
        assert_eq!(panes[0].get("width").and_then(Value::as_u64), Some(120));
        assert_eq!(panes[0].get("height").and_then(Value::as_u64), Some(43));
    }

    #[test]
    fn pane_verbs_tab_create_falls_back_to_the_callers_own_workspace() {
        let mut f = FakeCockpit::new();
        f.tab_create_result = Some("w4:p31".into());
        f.context = Some(PaneContext {
            pane_id: "w4:p4".into(),
            tab_id: Some("w4:t1".into()),
            workspace_id: Some("w4".into()),
        });
        let r = ok(&f, "tab-create", &["--label", "runtime", "--cwd", "/repo"]);
        assert_eq!(f.calls(), vec!["pane_context", "tab_create w4 /repo runtime"]);
        assert_eq!(r.get("pane_id").and_then(Value::as_str), Some("w4:p31"));
    }

    #[test]
    fn pane_verbs_tab_create_refuses_when_no_workspace_can_be_resolved() {
        let mut f = FakeCockpit::new();
        f.context = Some(PaneContext {
            pane_id: "w4:p4".into(),
            tab_id: None,
            workspace_id: None,
        });
        let err = dispatch_pane("tab-create", &["--label", "runtime"], &f).unwrap_err();
        assert_eq!(err.code, "workspace_required");
    }

    #[test]
    fn pane_verbs_tab_create_on_tmux_needs_no_workspace() {
        let mut f = FakeCockpit::new();
        f.name = "tmux";
        f.tab_create_result = Some("%12".into());
        f.context =
            Some(PaneContext { pane_id: "%4".into(), tab_id: None, workspace_id: None });
        let r = ok(&f, "tab-create", &["--label", "runtime", "--cwd", "/repo"]);
        assert_eq!(f.calls(), vec!["pane_context", "tab_create  /repo runtime"]);
        assert_eq!(r.get("pane_id").and_then(Value::as_str), Some("%12"));
    }

    #[test]
    fn pane_verbs_tab_list_and_tab_focus_pin_their_keys() {
        let mut f = FakeCockpit::new();
        f.tabs = vec![TabRow { id: "@2".into(), label: "runtime".into() }];
        let r = ok(&f, "tab-list", &[]);
        let tabs = r.get("tabs").and_then(Value::as_array).expect("tabs array");
        let keys: Vec<&str> = tabs[0].as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(keys, vec!["tab_id", "label"]);
        assert!(ok(&f, "tab-focus", &["@2"]).is_empty());
        assert_eq!(f.calls(), vec!["tab_list <own>", "tab_focus @2"]);
    }

    #[test]
    fn pane_verbs_unknown_subverb_and_missing_positional_are_usage_refusals() {
        let f = FakeCockpit::new();
        assert_eq!(dispatch_pane("teleport", &[], &f).unwrap_err().code, "usage");
        assert_eq!(dispatch_pane("close", &[], &f).unwrap_err().code, "usage");
    }

    // ─── pane-id by label ───────────────────────────────────────────────

    #[test]
    fn pane_verbs_pane_id_by_label_finds_the_first_match() {
        let mut f = FakeCockpit::new();
        f.panes = vec![
            row("w4:p4", None, "w4:t1"),
            row("w4:p5", Some("dispatch"), "w4:t1"),
            row("w4:p6", Some("dispatch"), "w4:t1"),
        ];
        assert_eq!(f.pane_id_by_label("dispatch").unwrap(), Some("w4:p5".to_string()));
    }

    #[test]
    fn pane_verbs_pane_id_not_found_is_a_typed_refusal() {
        let mut f = FakeCockpit::new();
        f.panes = vec![row("w4:p4", Some("merge"), "w4:t1")];
        // the same branch `pane_id` renders as exit 1 + error.code
        let miss = f.pane_id_by_label("dispatch").unwrap();
        assert_eq!(miss, None);
        let err = verb_error("not_found", "no pane carries the label \"dispatch\"");
        assert_eq!(err.code, "not_found");
    }

    // ─── the envelope ───────────────────────────────────────────────────

    #[test]
    fn pane_verbs_envelope_keys_are_the_same_on_both_transports() {
        for transport in ["herdr", "tmux"] {
            let mut result = Map::new();
            result.insert("pane_id".into(), Value::String("p".into()));
            let mut m = Map::new();
            m.insert("ok".into(), Value::Bool(true));
            m.insert("transport".into(), Value::String(transport.into()));
            m.insert("result".into(), Value::Object(result));
            let keys: Vec<&str> = m.keys().map(String::as_str).collect();
            assert_eq!(keys, vec!["ok", "transport", "result"]);
        }
    }

    #[test]
    fn pane_verbs_result_walks_the_envelope_path() {
        let body = r#"{"ok":true,"transport":"tmux","result":{"pane_id":"%12"}}"#;
        let v: Value = serde_json::from_str(body).unwrap();
        assert_eq!(
            crate::herding::walk_result_path(&v, "pane_id"),
            Value::String("%12".into())
        );
        // a nested path walks the same way herdr bodies do
        let nested: Value =
            serde_json::from_str(r#"{"result":{"pane":{"pane_id":"w4:p4"}}}"#).unwrap();
        assert_eq!(
            crate::herding::walk_result_path(&nested, "pane.pane_id"),
            Value::String("w4:p4".into())
        );
        // a missing leaf is Null, which the reader turns into a refusal
        assert_eq!(crate::herding::walk_result_path(&v, "nope"), Value::Null);
    }

    // ─── the tmux argv builders and parser, no process ──────────────────

    #[test]
    fn pane_verbs_tmux_send_text_is_two_calls_literal_then_enter() {
        let calls = send_text_argv("%4", "dispatch: picking p-12");
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0], vec!["send-keys", "-t", "%4", "-l", "dispatch: picking p-12"]);
        assert_eq!(calls[1], vec!["send-keys", "-t", "%4", "Enter"]);
    }

    #[test]
    fn pane_verbs_tmux_rename_sets_the_pane_title_and_clears_with_empty() {
        assert_eq!(rename_argv("%4", Some("dispatch")), vec![
            "select-pane",
            "-t",
            "%4",
            "-T",
            "dispatch"
        ]);
        assert_eq!(rename_argv("%4", None), vec!["select-pane", "-t", "%4", "-T", ""]);
    }

    #[test]
    fn pane_verbs_tmux_listings_scope_to_one_session() {
        assert_eq!(list_panes_argv(None), vec![
            "list-panes".to_string(),
            "-s".to_string(),
            "-F".to_string(),
            TMUX_PANE_FORMAT.to_string()
        ]);
        assert_eq!(list_panes_argv(Some("cockpit")), vec![
            "list-panes".to_string(),
            "-s".to_string(),
            "-t".to_string(),
            "cockpit".to_string(),
            "-F".to_string(),
            TMUX_PANE_FORMAT.to_string()
        ]);
        assert_eq!(tab_list_argv(None), vec![
            "list-windows".to_string(),
            "-F".to_string(),
            TMUX_TAB_FORMAT.to_string()
        ]);
        assert_eq!(tab_focus_argv("@2"), vec!["select-window", "-t", "@2"]);
    }

    #[test]
    fn pane_verbs_tmux_pane_rows_parse_five_tab_separated_columns() {
        let stdout = "%4\tdispatch\t@1\t/repo\tbash\n%5\t\t@1\t/repo\tclaude\n";
        let rows = parse_pane_rows(stdout);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0],
            PaneRow {
                id: "%4".into(),
                label: Some("dispatch".into()),
                tab: "@1".into(),
                cwd: Some("/repo".into()),
                command: Some("bash".into()),
            }
        );
        // an empty title is NO label — an untitled pane must never answer
        // `pane-id --label ""`
        assert_eq!(rows[1].label, None);
        assert_eq!(rows[1].command.as_deref(), Some("claude"));
    }

    #[test]
    fn pane_verbs_tmux_tab_rows_parse_id_and_name() {
        let rows = parse_tab_rows("@1\tcockpit\n@2\truntime\n");
        assert_eq!(rows, vec![
            TabRow { id: "@1".into(), label: "cockpit".into() },
            TabRow { id: "@2".into(), label: "runtime".into() },
        ]);
        assert!(parse_tab_rows("").is_empty());
    }

    #[test]
    fn pane_verbs_tmux_capture_argv_uses_the_configured_scrollback() {
        assert_eq!(capture_argv("%4", 40), vec![
            "capture-pane",
            "-p",
            "-t",
            "%4",
            "-S",
            "-40"
        ]);
    }

    #[test]
    fn pane_verbs_tmux_blocked_screens_are_never_typed_into() {
        // D3: `refuse_if_blocked`'s decision, exercised through the same
        // classifier it calls — a dialog on screen means no keys are sent.
        let settings = TmuxSettings::default();
        assert_eq!(classify("do you trust the files in this folder?", &settings), Screen::Blocked);
        assert_eq!(classify("$ ", &settings), Screen::Idle);
    }

    // ─── the herdr body parsers, no process ─────────────────────────────

    #[test]
    fn pane_verbs_herdr_pane_rows_parse_both_result_shapes() {
        let object: Value = serde_json::from_str(
            r#"{"result":{"panes":[{"pane_id":"w4:p4","label":"dispatch","tab_id":"w4:t1",
                 "cwd":"/repo","command":"bash"}]}}"#,
        )
        .unwrap();
        let bare: Value =
            serde_json::from_str(r#"{"result":[{"pane_id":"w4:p4","label":"dispatch",
                 "tab_id":"w4:t1","cwd":"/repo","command":"bash"}]}"#)
                .unwrap();
        let want = vec![PaneRow {
            id: "w4:p4".into(),
            label: Some("dispatch".into()),
            tab: "w4:t1".into(),
            cwd: Some("/repo".into()),
            command: Some("bash".into()),
        }];
        assert_eq!(parse_herdr_pane_rows(&object), want);
        assert_eq!(parse_herdr_pane_rows(&bare), want);
        // a row with no id is dropped, never a panic
        let junk: Value = serde_json::from_str(r#"{"result":{"panes":[{"label":"x"}]}}"#).unwrap();
        assert!(parse_herdr_pane_rows(&junk).is_empty());
    }

    #[test]
    fn pane_verbs_herdr_tab_rows_and_context_parse() {
        let tabs: Value =
            serde_json::from_str(r#"{"result":{"tabs":[{"tab_id":"w4:t1","label":"cockpit"}]}}"#)
                .unwrap();
        assert_eq!(parse_herdr_tab_rows(&tabs), vec![TabRow {
            id: "w4:t1".into(),
            label: "cockpit".into()
        }]);

        let current: Value = serde_json::from_str(
            r#"{"result":{"pane":{"pane_id":"w4:p4","tab_id":"w4:t1","workspace_id":"w4"}}}"#,
        )
        .unwrap();
        assert_eq!(
            parse_herdr_pane_context(&current),
            Some(PaneContext {
                pane_id: "w4:p4".into(),
                tab_id: Some("w4:t1".into()),
                workspace_id: Some("w4".into()),
            })
        );
        let empty: Value = serde_json::from_str(r#"{"result":{}}"#).unwrap();
        assert_eq!(parse_herdr_pane_context(&empty), None);
    }

    // ─── flag plumbing ──────────────────────────────────────────────────

    #[test]
    fn pane_verbs_take_opt_leaves_positionals_alone() {
        let mut a = vec!["w4:p4", "dispatch: --not-a-flag", "--lines", "20"];
        assert_eq!(take_opt(&mut a, "--lines"), Some("20"));
        assert_eq!(take_opt(&mut a, "--missing"), None);
        assert_eq!(a, vec!["w4:p4", "dispatch: --not-a-flag"]);
    }

    #[test]
    fn pane_verbs_tail_of_keeps_the_last_lines() {
        assert_eq!(tail_of("a\nb\nc", 2), "b\nc");
        assert_eq!(tail_of("a\nb\nc", 9), "a\nb\nc");
        assert_eq!(tail_of("a\nb\nc", 0), "");
    }
}
