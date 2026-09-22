# The comment guard and the comment ratchet

No comment may be written in code in this repository. Two layers hold the rule.
The **write guard** refuses an Edit, a Write, a heredoc shell write or an
apply_patch that ADDS a comment line to a code file, and names the remedy in the
same message. The **ratchet** — a test in the declared suite plus the committed
`.bee/comment-baseline.json` — goes red when any file's comment count rises above
its baseline. One config key, `no_code_comments` (boolean, default false), turns
the guard on; a host that installs bee gets the code and refuses nothing until it
opts in.

## Sub-features

- `comment-guard-edit` refuses an Edit whose new text adds a comment line under a
  code root, naming the file, the line number and the line.
- `comment-guard-write` refuses a `Write` of a whole new code file carrying a
  comment, doc comments (`///`, `//!`) included.
- `comment-guard-multiedit` refuses a MultiEdit and names the offending edit.
- `comment-guard-heredoc` refuses a Bash heredoc body redirected into a code path
  when the body adds a comment line.
- `comment-guard-apply-patch` refuses an apply_patch by its ADDED `+` lines.
- `comment-guard-off` stays silent with the key absent or `false`.
- `comment-guard-move` allows a comment that was only moved, reindented or
  deleted — the comparison is old versus new, not "the file has comments".
- `comment-guard-exceptions` allows a `#!` line, a `SAFETY:` line on an unsafe
  block, and a license header at file top.
- `comment-guard-scope` never touches a `.md`, `.json` or `.toml`, and never
  touches a path outside the code roots.
- `comment-baseline-check` reports every file above its baseline and exits 1.
- `comment-baseline-write` lowers and drops entries and REFUSES to raise one.

## How to get to it (user POV)

- With `no_code_comments: true` in `.bee/config.json`, add `// x` to any `.rs`
  file under `packages/bee-rs/crates/<crate>/src/` and the edit is refused before
  it lands, exit 2 on stderr.
- Run `bee dev comment-baseline --check` from the bee source checkout to read the
  ratchet by hand; CI runs the same fence as the unit test
  `devtools::comment_baseline::tests::every_code_file_is_at_or_below_its_comment_baseline`.
- Run `bee dev comment-baseline --write` after deleting comments to lower the
  baseline.

## Driving it with control-bee

Preconditions:

- A launched sandbox, `control-bee doctor` fully `ok`.
- The binary under test is the one `control-bee build` produced — the comment arm
  and the `dev comment-baseline` verb both ship in it.

- **Turn the key on in the sandbox.** The sandbox's own config decides, so write
  it there. bee ships no `config set` verb, so put the file:
  `printf '{"no_code_comments": true}\n' | control-bee put .bee/config.json`.
  Read it back with `control-bee sh -- cat .bee/config.json`.
- **Give the guard a code file to guard.** Run
  `printf 'fn f() {}\n' | control-bee put packages/bee-rs/crates/bee/src/x.rs`.
- **Drive the refusal with a crafted hook payload.** The hook reads one JSON
  object on stdin, exactly as `packages/bee-rs/crates/bee/tests/hook_contracts.rs`
  (`a_comment_guard_deny_reaches_the_host_as_exit_two_on_stderr`) drives it. Run
  `printf '{"tool_name":"Edit","tool_input":{"file_path":"packages/bee-rs/crates/bee/src/x.rs","old_string":"fn f() {}","new_string":"// note\nfn f() {}"},"cwd":"<sandbox path>"}' | control-bee cli -- hook write-guard`.
  The `.exit` file holds `2`, stdout is EMPTY, and stderr opens with
  `bee comment guard denied this write:` naming
  `packages/bee-rs/crates/bee/src/x.rs:1`, quoting `"// note"`, then the `FIX:`
  sentence with the two homes for the why, the owning concept for a public item's
  description, and `bee backlog add` for a workaround. Read the sandbox path from
  `control-bee paths`.
- **The same drive from a hooked session.** In a real Claude/Codex session with
  the hook installed, an Edit that adds `/// x` to a `.rs` file under a code root
  is refused by the host with that same text. Use this when you want the refusal
  as the USER meets it; use the crafted payload when you want it repeatable.
- **The key off is silent.** Re-run the same payload after
  `printf '{}\n' | control-bee put .bee/config.json`. The `.exit` file holds `0`
  and stderr carries no `bee comment guard` line.
- **A moved comment passes.** Send an Edit whose `old_string` already holds
  `// note` and whose `new_string` holds the same `// note` on another line. The
  `.exit` file holds `0`.
- **Read the ratchet.** `bee dev comment-baseline` is a SOURCE-checkout verb: it
  refuses in the sandbox with
  `bee dev comment-baseline: the dev surface runs in a bee SOURCE checkout … FIX: cd into the bee checkout`.
  Drive the refusal there once —
  `control-bee cli -- dev comment-baseline --check` — then run the verb itself
  from the real checkout: `.bee/bin/bee dev comment-baseline --check`. A clean
  tree prints `comment-baseline --check: <n> file(s) at or below baseline` and
  exits 0.
- **A rise is named and refused.** Append a comment line to any code file in the
  source checkout, re-run `--check`, and read the per-file
  `<path>: <count> comment line(s), baseline <n>` line plus the remedy sentence
  `bee dev comment-baseline --write lowers a count, never raises one.` Delete the
  line again; never fix a rise with `--write`.
- **Proof.** Run `control-bee snapshot comment-guard`. The evidence dir holds the
  refusal's `.exit` file at `2` with the denial text on stderr, the key-off run at
  `0`, and the sandbox's `.bee/config.json` showing the key that decided.

## Gotchas

- **The guard counts a line that only LOOKS like a comment.** The predicate is a
  line predicate, not a parser: a line starting with `//` or `#` inside a string
  literal counts. One such line exists in the repo today
  (`hooks/session_close/html.rs`). Fix a false red by rewriting the string, never
  by an allowlist.
- **Hosts default to off.** `no_code_comments` absent reads as `false`. Onboarding
  ships the hook and the doctrine line, never the key and never the baseline — the
  baseline is this repository's own.
- **An UNGRANTED worktree reads main's config.** The guard resolves its store the
  way every other check does: a GRANTED feature worktree reads its own
  `.bee/config.json` (so an in-flight branch that predates the key is not
  refused), while an ungranted one reads main's — where the key is `true`.
- **A Bash write with no readable body is never refused.** A `sed -i`, a `cp`, a
  redirect from a command's output carries no text the guard can read. That shape
  passes, by design; the ratchet catches whatever reaches the tree.
- **The baseline never rises.** `--write` lowers and drops entries and refuses to
  raise one, naming the file. A count that rose because a branch merged is fixed
  by DELETING the merged comment lines, not by re-seeding.
- **The seed happened once.** The first `--write` seeded from the per-file maximum
  over main and every unmerged `wt/*` branch, so a sibling feature merging later
  cannot red CI. A later `--write` folds those branches in again and reports
  `the unmerged branches were not folded in — <gap>` when git could not be read.
- **`--check` and `--write` are exclusive.** Neither flag, or both, prints
  `usage: bee dev comment-baseline (--check | --write)` and exits 1.
- **A failing `--check` writes to stderr, a passing one to stdout.** Assert on the
  recorded `.exit` file, not on the stream.
