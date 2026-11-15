# ppb-34c: check of the pstack unslop merge in ppb-34

Checked against commit `4b9b7a055` (ppb-34). The source is the pstack skill
`unslop` at `/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/unslop/SKILL.md`,
the only file in that skill. The bee side is `skills/bee-unslop/SKILL.md`, with
the voice rules it cites in `skills/bee-technical-writing/SKILL.md` ("Vary the
rhythm"). No file other than this report was edited.

## Verdict

The merge holds. Every pstack unslop rule is present in bee-unslop, or left out
with a named bee home or a named bee difference. None is missing. The port notes
in `port-notes/merge-ppb-34.md` list the same split.

- **pstack-only tool names:** none. A scan of `skills/bee-unslop/SKILL.md` for
  `/loop`, `Graphite`, `bugbot`, `poteto`, `arena`, `architect`, `Cursor`,
  `disable-model-invocation` and `Task tool` matches only the provenance line
  `> Ported from pstack (cursor/plugins, pstack/skills/unslop)`. That line names
  the source path, not a tool, and predates ppb-34.
- **bee rules removed:** none. The ppb-34 diff to `skills/bee-unslop/SKILL.md`
  has two removed lines, and both come back rewritten: Process step 3
  "Self-audit" is renumbered to step 4 with the same text, and rule 13 keeps its
  old text with one sentence inserted. Every other change is an insertion. Rules
  32 and 33, the Scope section, the Headless section, the stable-rule-number
  note and the closing hive handoff are unchanged. The numbered rule count went
  from 28 to 33.
- **Other checks:** the bee-unslop description is unchanged at 265 characters,
  under 300. `skills/bee-unslop/LICENSE.pstack` is byte-identical to the pstack
  `LICENSE`. `SKILL.md` and `LICENSE.pstack` match the source byte for byte in
  all five rendered trees (`.claude-plugin`, `.codex-plugin`, `.agents`,
  `.opencode`, `.claude`).
- **Rules 1 to 31 text:** a line-by-line compare of each numbered pattern rule
  finds all 31 identical to pstack. The only numbered line that differs is
  Process step 3, "Add soul" in pstack and "Add voice" in bee.

## Rule by rule

P = present in bee-unslop. L = left out, with the bee home or the bee difference.
M = missing.

### Frontmatter and opening

| # | pstack rule | Status | Where in bee |
|---|---|---|---|
| F1 | name `unslop` | L | bee name `bee-unslop`; the bee prefix is the bee naming rule |
| F2 | description "Must always apply" | L | bee difference: bee routes by its description and the Communication contract, and the Scope section limits what the rules touch (port notes, "Left out because bee differs") |
| F3 | "Edit text to remove AI patterns and add human voice." | P | line 15 keeps "Edit text to remove AI patterns."; the voice half lives in Process step 3 and the Adding voice section. The intro wording predates ppb-34 |

### Process

| # | pstack rule | Status | Where in bee |
|---|---|---|---|
| P1 | Scan for the patterns | P | Process step 1 |
| P2 | Rewrite, preserve meaning, match tone | P | Process step 2 |
| P3 | Add soul (see next section) | P | Process step 3, renamed "Add voice" |
| P4 | Self-audit: "What makes this obviously AI generated?" | P | Process step 4 |

### Adding soul

| # | pstack rule | Status | Where in bee |
|---|---|---|---|
| S0 | Removing patterns is half the job; voiceless writing is as obvious | P | Adding voice, opening paragraph |
| S1 | Have opinions | L | `bee-technical-writing` "Vary the rhythm": "Have a view where the mode allows it"; bee bounds it by mode |
| S2 | Vary rhythm | L | `bee-technical-writing` "Vary the rhythm": mix sentence lengths |
| S3 | Acknowledge complexity | P | Adding voice, first bullet, extended with "name the cost next to the benefit" |
| S4 | Use "I" when it fits | P | Adding voice, second bullet |
| S5 | Let some mess in | P | Adding voice, third bullet, bounded by Scope so verbatim lines are never loosened |
| S6 | Be specific | L | `bee-technical-writing` "Vary the rhythm": "Be specific over sterile" |

The three L rows were checked in `skills/bee-technical-writing/SKILL.md`
lines 27 to 34; each rule is there. Adding voice cites that section by name.

### Patterns 1 to 31

| # | pstack rule | Status | Where in bee |
|---|---|---|---|
| 1 | Puffery | P | rule 1, brought in by ppb-34 |
| 2 | Name-dropping | P | rule 2, brought in by ppb-34 |
| 3 | Superficial -ing phrases | P | rule 3 |
| 4 | Promotional language | P | rule 4, brought in by ppb-34 |
| 5 | Vague attributions | P | rule 5 |
| 6 | Formulaic challenges | P | rule 6, brought in by ppb-34 |
| 7 | AI vocabulary | P | rule 7 |
| 8 | Fancy ways to say "is" | P | rule 8 |
| 9 | "Not just X, but Y." | P | rule 9 |
| 10 | Rule of three | P | rule 10 |
| 11 | Synonym cycling | P | rule 11 |
| 12 | False ranges | P | rule 12 |
| 13 | Em dash overuse, with the parentheses-trade sentence | P | rule 13; the trade sentence brought in by ppb-34 |
| 14 | Colon overuse | P | rule 14 |
| 15 | Boldface overuse | P | rule 15 |
| 16 | Inline-header lists | P | rule 16 |
| 17 | Title case headings | P | rule 17 |
| 18 | Decorative emojis | P | rule 18 |
| 19 | Curly quotes | P | rule 19 |
| 20 | Chatbot phrases | P | rule 20 |
| 21 | Cutoff disclaimers | P | rule 21, brought in by ppb-34 |
| 22 | Sycophantic tone | P | rule 22 |
| 23 | Filler phrases | P | rule 23 |
| 24 | Excessive hedging | P | rule 24 |
| 25 | Generic conclusions | P | rule 25 |
| 26 | Abstract metaphor nouns | P | rule 26 |
| 27 | Say what it does, not how it feels | P | rule 27 |
| 28 | Shorten or split dense sentences | P | rule 28 |
| 29 | Active voice | P | rule 29 |
| 30 | Cut adverbs | P | rule 30 |
| 31 | Prefer the plain word | P | rule 31 |

## Count

45 source rules: 40 present, 5 left out, 0 missing. Left out: F1, F2, S1,
S2 and S6. Present: F3, P1 to P4, S0, S3 to S5 and rules 1 to 31.

## Side finding

The ppb-34 upstream summary lists commit `a64a9c7f6` beside `4b9b7a055`. That
commit is "Check the pstack teach merge into bee-teach" (ppb-33c) and does not
touch `skills/bee-unslop/`. The ppb-34 work is `4b9b7a055` alone.
