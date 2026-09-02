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
// What is deliberately NOT pinned: the PowerShell tail. On a host whose shell
// resolves to PowerShell, `render_agents_block` appends
// `packages/bee/AGENTS.windows.md` INSIDE the same block. This repo declares no
// `host_shell` in `.bee/config.json` and its committed `AGENTS.md` is the posix
// render, so the fence pins that render and nothing else. A regen on a
// PowerShell host turns it red — correctly: that tail does not belong in this
// repo's committed doctrine.
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

/// `render_agents_block(block, None)` — the posix render.
fn render_block(body: &str) -> String {
    format!("{MARKER_START}\n{}\n{MARKER_END}\n", body.trim_end())
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
    let expected = render_block(&source);

    assert!(
        actual == expected,
        "the bee block in {AGENTS_MD} is not the render of {AGENTS_BLOCK}.\n\n{}\n\n{AGENTS_MD} \
         is GENERATED: {AGENTS_BLOCK} is the only place to edit, and the rendered copy must be \
         regenerated after every edit to it. FIX: run `{REGEN}` and commit the resulting \
         {AGENTS_MD}; never hand-edit the block inside {AGENTS_MD}.\n\n(If you regenerated on a \
         PowerShell host, the extra `packages/bee/AGENTS.windows.md` tail is the difference — \
         that tail is not part of this repo's committed doctrine.)",
        first_difference(&actual, &expected),
    );
}
