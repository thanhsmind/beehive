---
type: bee.pattern
title: An allowlist that names a client grants every subcommand it has
description: An allowlist that names a client grants every subcommand it has
tags: [permissions, allowlist, tools, read-only, boundary]
timestamp: 2026-08-28
bee:
  id: pattern-20260828-an-allowlist-that-names-a-client-grants-every-subcommand-it-has
  lifecycle: active
  areas: [bee-herding]
  sources: ["slp-supervisor-heartbeat cell sup-1 — the supervisor's read-only tool surface, recorded deviation, 2026-08-27"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "for any role declared read-only, read its allowlist entries as the union of every subcommand each named binary accepts — an entry naming a multiplexer or database client is a write grant; replace it with the narrow verbs the product already owns"
---

# An allowlist that names a client grants every subcommand it has

A role was specified as **read only**. Its tool allowlist reached the terminal
multiplexer the obvious way: allow the multiplexer client. The role only meant
to read pane text.

That client also sends keystrokes and kills panes. The allowlist said *read*
and granted *write*, and nothing in the wording showed it. Two transports meant
two such entries, so the same hole existed twice, spelled differently.

The fix was not a tighter pattern on the client. It was to route the read
through the product's own narrow verbs — the ones that can only list and read —
so both transports resolved to the **same** allowlist string, and neither
carried a subcommand nobody asked for.

The general shape: **an allowlist entry is a grant of everything the named
program can do, not of the thing you happened to want.** A binary is a
namespace. Read every entry as the union of its subcommands, and treat any
general-purpose client — a multiplexer, a package manager, a database shell, a
version-control binary — as a write grant until proven otherwise.

Two tells that the boundary is wrong:

- The read-only surface names a program whose own documentation has a "send",
  "delete", "exec", or "kill" verb.
- The same logical permission is spelled differently on two backends. One of
  the spellings is almost always wider than the other.
