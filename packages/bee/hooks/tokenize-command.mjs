// tokenize-command.mjs — pure, side-effect-free shell tokenizer for
// bee-write-guard.mjs's CLI-shape check (checkCliShape/splitCliSegments).
//
// Extracted out of bee-write-guard.mjs into its own file so it can be
// imported directly by tests without triggering that file's top-level
// `main()` execution. Mirrors guards.mjs's tokenize(): separators split
// into their own token even glued to adjacent text (`2>/dev/null;` used to
// swallow the `;` into the redirect target, denying ordinary read-only
// commands), adjacent quoted/unquoted segments with no space merge into one
// word (bash word-splitting — without this, a DIRECT_EDIT_DENY path could
// be split across quotes to evade containment), and a backslash escapes the
// next character literally. Duplicated from guards.mjs's tokenize() rather
// than imported for the same reason resolveCliCommandName's registry-shape
// logic is duplicated in bee-write-guard.mjs (repo-root lib/*.mjs only,
// never re-entering guards mid-parse) — kept in sync by hand; guards.mjs's
// tokenize() is the source of truth for this exact algorithm. A tokenizer-
// equivalence test in guards.test.mjs is what actually catches hand-sync
// drift between the two copies.
export function tokenizeCommand(command) {
  const str = String(command || "");
  const tokens = [];
  let current = "";
  let hasCurrent = false;
  const flush = () => {
    if (hasCurrent) {
      tokens.push(current);
      current = "";
      hasCurrent = false;
    }
  };
  let i = 0;
  while (i < str.length) {
    const ch = str[i];
    if (ch === " " || ch === "\t" || ch === "\n" || ch === "\r") {
      flush();
      i += 1;
      continue;
    }
    if (ch === "\\" && i + 1 < str.length) {
      current += str[i + 1];
      hasCurrent = true;
      i += 2;
      continue;
    }
    if (ch === '"' || ch === "'") {
      const close = str.indexOf(ch, i + 1);
      const end = close === -1 ? str.length : close;
      current += str.slice(i + 1, end);
      hasCurrent = true;
      i = end + 1;
      continue;
    }
    if ((ch === "&" && str[i + 1] === "&") || (ch === "|" && str[i + 1] === "|")) {
      flush();
      tokens.push(ch + ch);
      i += 2;
      continue;
    }
    if (ch === ";" || ch === "&" || ch === "|") {
      flush();
      tokens.push(ch);
      i += 1;
      continue;
    }
    current += ch;
    hasCurrent = true;
    i += 1;
  }
  flush();
  return tokens;
}
