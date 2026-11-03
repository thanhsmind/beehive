// The AGENTS.md render fence.
//
// `AGENTS.md` is the file agents actually load; the bee-owned part of it is
// RENDERED from `packages/bee/AGENTS.block.md` by `bee dev regen`. Nothing
// caught an edit to the source that was never regenerated. `rule_index_parity`
// compares the two surfaces' rule-marker SETS, so a body edit that leaves the
// markers alone passes it, and CI has no regen-then-diff step. A doctrine line
// added to the block and never rendered is a rule no agent ever reads.
//
// What is pinned: the bytes between `<!-- BEE:START -->` and `<!-- BEE:END -->`
// in `AGENTS.md` equal what `render_agents_block`
// (`packages/bee-rs/crates/bee/src/onboard/merge.rs`) produces from
// `packages/bee/AGENTS.block.md` — marker line, body with trailing whitespace
// trimmed, marker line, trailing newline. That is the same equality
// `onboard::plan` tests to decide `update_agents_block`.
//
// The PowerShell tail follows the REPOSITORY, never the machine. When the
// resolved host shell is PowerShell, `render_agents_block` appends
// `packages/bee/AGENTS.windows.md` INSIDE the same block. The fence reads only
// the repo half of that resolution — `host_shell` in `.bee/config.json`
// (decision f05e6583: the repository decides before the machine does): declared
// `"powershell"` pins the render WITH the tail; declared `"posix"`, absent, or
// unrecognised pins the posix render. `cfg!(windows)` is deliberately not
// consulted, so the pinned bytes are the same on a Windows workstation and on
// Linux CI. A repo whose maintainers regen on PowerShell declares `powershell`;
// leaving the key absent and regenerating on Windows turns this red —
// correctly: the committed doctrine would then depend on who last ran regen.
//
// Two marker details, taken from `merge.rs` rather than assumed. The AGENTS
// markers are matched by plain `find` — FIRST occurrence, not whole-line
// anchored. Only the gitignore pair is line-anchored, and only there does a
// `# BEE:START custom notes` decoy matter. This fence copies the loose form on
// purpose, so it sees the same block onboarding would splice.
//
// Shape, deliberately: pure filesystem, std only, NOTHING imported from the bee
// crate — the model is `rule_index_parity.rs` beside it. A fence that imported
// the renderer would agree with a broken renderer.

use std::path::PathBuf;

/// The generated file, and the source it is rendered from.
const AGENTS_MD: &str = "AGENTS.md";
const AGENTS_BLOCK: &str = "packages/bee/AGENTS.block.md";
/// The PowerShell tail, appended inside the block only when `host_shell` is
/// declared `"powershell"` (`onboard::merge::host_shell_is_powershell`).
const AGENTS_WINDOWS: &str = "packages/bee/AGENTS.windows.md";
const BEE_CONFIG: &str = ".bee/config.json";

/// `onboard::templates::MARKER_START` / `MARKER_END`.
const MARKER_START: &str = "<!-- BEE:START -->";
const MARKER_END: &str = "<!-- BEE:END -->";

const REGEN: &str = ".bee/bin/bee dev regen";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4).unwrap().to_path_buf()
}

fn read(rel: &str) -> String {
    let path = repo_root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {rel}: {e}"))
}

/// `extract_agents_block`, re-derived from text: first `MARKER_START`, first
/// `MARKER_END`, both markers included, one trailing newline.
fn extract_block(text: &str) -> Option<String> {
    let start = text.find(MARKER_START)?;
    let end = text.find(MARKER_END)?;
    if end < start {
        return None;
    }
    Some(format!("{}\n", &text[start..end + MARKER_END.len()]))
}

/// `render_agents_block(block, tail)` — body, then the tail after one blank
/// line when it is `Some` and non-blank, both with trailing whitespace trimmed.
fn render_block(body: &str, tail: Option<&str>) -> String {
    let extra = match tail.map(str::trim_end).unwrap_or_default() {
        s if s.trim().is_empty() => String::new(),
        s => format!("\n\n{s}"),
    };
    format!("{MARKER_START}\n{}{extra}\n{MARKER_END}\n", body.trim_end())
}

/// The repo half of `host_shell_is_powershell`: `"host_shell": "powershell"`
/// declared in `.bee/config.json`. No JSON crate — a std-only fence must not
/// agree with a broken parser either — so this is a plain key/value scan of a
/// flat, hand-written config: a top-level key on its own line is what it reads.
fn repo_declares_powershell() -> bool {
    let Ok(text) = std::fs::read_to_string(repo_root().join(BEE_CONFIG)) else {
        return false;
    };
    text.lines().any(|line| {
        let Some((key, value)) = line.trim().trim_end_matches(',').split_once(':') else {
            return false;
        };
        key.trim() == "\"host_shell\"" && value.trim() == "\"powershell\""
    })
}

/// Line number and both texts' lines at the first place they differ. Lines are
/// clipped so a wrapped paragraph does not bury the message.
fn first_difference(actual: &str, expected: &str) -> String {
    fn clip(line: &str) -> String {
        match line.char_indices().nth(90) {
            Some((i, _)) => format!("{}…", &line[..i]),
            None => line.to_string(),
        }
    }
    let mut a = actual.lines();
    let mut e = expected.lines();
    let mut n = 0usize;
    loop {
        n += 1;
        match (a.next(), e.next()) {
            (None, None) => return "the texts differ only in trailing bytes".to_string(),
            (x, y) if x == y => continue,
            (x, y) => {
                return format!(
                    "first difference at block line {n}:\n    {AGENTS_MD}:    {}\n    \
                     {AGENTS_BLOCK}: {}",
                    x.map(clip).unwrap_or_else(|| "<end of block>".into()),
                    y.map(clip).unwrap_or_else(|| "<end of block>".into()),
                );
            }
        }
    }
}

#[test]
fn agents_md_block_is_the_rendered_source_byte_for_byte() {
    let rendered_file = read(AGENTS_MD);
    let source = read(AGENTS_BLOCK);

    assert!(
        !source.trim().is_empty(),
        "{AGENTS_BLOCK} is empty, so this fence would be comparing nothing — the exact silence \
         it exists to break"
    );

    let actual = extract_block(&rendered_file).unwrap_or_else(|| {
        panic!(
            "{AGENTS_MD} carries no {MARKER_START} … {MARKER_END} block (or the end marker \
             precedes the start).\n\nThat block is where every agent reads bee's doctrine; \
             without it the rendered file and its source cannot be compared at all. FIX: run \
             `{REGEN}`."
        )
    });
    let tail = repo_declares_powershell().then(|| read(AGENTS_WINDOWS));
    let expected = render_block(&source, tail.as_deref());

    assert!(
        actual == expected,
        "the bee block in {AGENTS_MD} is not the render of {AGENTS_BLOCK}.\n\n{}\n\n{AGENTS_MD} \
         is GENERATED: {AGENTS_BLOCK} is the only place to edit, and the rendered copy must be \
         regenerated after every edit to it. FIX: run `{REGEN}` and commit the resulting \
         {AGENTS_MD}; never hand-edit the block inside {AGENTS_MD}.\n\n(The `{AGENTS_WINDOWS}` tail is expected \
         only when {BEE_CONFIG} declares \"host_shell\": \"powershell\" — a regen on a \
         PowerShell host without that key, or on a posix host with it, is the usual \
         difference.)",
        first_difference(&actual, &expected),
    );
}
