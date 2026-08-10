---
type: bee.pattern
title: Non-ASCII in a .ps1 without BOM is a parse-time bomb on Windows PowerShell 5.1
description: Non-ASCII in a .ps1 without BOM is a parse-time bomb on Windows PowerShell 5.1
tags: [failure, windows, powershell, encoding, cross-platform]
timestamp: 2026-07-14
bee:
  id: pattern-20260714-non-ascii-in-a-ps1-without-bom-is
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT33", "original feature: installer-hardening"]
  polarity: pitfall
  critical: false
---

# Non-ASCII in a .ps1 without BOM is a parse-time bomb on Windows PowerShell 5.1

install.ps1 shipped unrunnable: six em-dashes in a UTF-8-no-BOM file. PS 5.1 decodes
no-BOM files as cp1252, so `—` (E2 80 94) ends in 0x94 = `"` (smart right-double-quote),
which PowerShell honors as a STRING TERMINATOR — one comment dash cascaded into ~10 parse
errors and the whole script never ran (reported as "codex doesn't understand bee": skills
were simply never installed on Windows). Keep .ps1 files pure ASCII and guard it with a
byte-level test (any platform, no pwsh needed); a WSL host can prove real parses via
`powershell.exe` interop + `Parser::ParseFile`.
