# gmr-2 evidence — home-prefix containment bypass closed

## The hole, before the fix

Live probes against the vendored hook (`.bee/bin/hooks/bee-write-guard.mjs`),
one payload per spelling on stdin, cwd = repo root. All four name the **same
destination file**:

```
echo hi > /home/<user>/.claude/lesson.md   exit=2   denied (containment)
echo hi > ~/.claude/lesson.md              exit=0   ALLOWED
echo hi > $HOME/.claude/lesson.md          exit=0   ALLOWED
```

A leading home reference is neither absolute nor contains `..`, so
`canonicalRelPath` resolved it against cwd as a literal `~` / `$HOME` directory
and produced an in-repo relative path that passed containment and flowed into
`checkWrite`.

## After the fix

```
exit=2  <- ~/.claude/lesson.md
exit=2  <- $HOME/.claude/lesson.md
exit=2  <- ${HOME}/.claude/lesson.md
exit=2  <- /home/<user>/.claude/lesson.md
```

Non-regression, same run:

```
exit=0  <- bare ~            (BROAD_TARGETS still owns it, behavior unchanged)
exit=0  <- docs/note.md      (in-repo path unaffected)
```

## Suite

```
PASS run_verify: 108 suite(s), concurrency=5, wall=75947ms
EXIT=0
```

108 rather than 109: `scripts/tests/test_rust_workspace.mjs` was removed with
the Rust runtime. No guard test was edited to reach green.

## Design note

Deny-outright, not expand-then-contain:

- expanding would make the wall's verdict depend on an environment variable,
  while the security boundary is meant to be declaration;
- the tokenizer has already discarded the quoting that decides whether bash
  would expand at all, so expansion would be guessing;
- deny has no resolution step that can fail open.

The consequence for the follow-up work: a declared `guards.memory_root` will
honor **absolute** spellings only.
