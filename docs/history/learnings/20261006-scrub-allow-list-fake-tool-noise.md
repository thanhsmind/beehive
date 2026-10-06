---
date: 2026-10-06
feature: bee-report-issues
categories: [failure, pattern]
severity: standard
tags: [scrub, privacy, test-fakes, gh]
---

# Learning: A Path Scrub Needs an Allow-List Start Rule

**Category:** failure
**Severity:** standard
**Tags:** [scrub, privacy, outbound-text]
**Applicable-when:** writing any scrub that must find secrets or host paths inside free text before it leaves the machine.

## What Happened

`bee report issue` (packages/bee-rs/crates/bee/src/verbs/report.rs) first rewrote a path only at the start of a whitespace token. Judge round one leaked `cfg=/srv/x`. The fix listed the characters that may stand before a path (`PATH_LEAD`). Judge round two then leaked `2>/srv/x`, `-C/srv/x` and `@/srv/x`. Round three leaked a path glued to a GitHub link by `,`, because the link skip ran to the next space. Each round was a new denylist gap.

## Root Cause

Every rule was a list of the shapes the author thought of. A public-tracker scrub fails on the shape nobody listed.

## Recommendation

When a scrub must find a token inside free text, define the start as "any character that cannot be part of a word" (here: not ASCII alphanumeric), and define every skip (a URL) as a bounded run of legal characters. Then probe it with an adversarial judge that invents shapes, not with the author's own test list.

# Learning: A Fake External Tool Must Print What the Real One Prints

**Category:** failure
**Severity:** standard
**Tags:** [test-fakes, gh, hermetic-tests]
**Applicable-when:** a test fakes an external CLI (gh, git, a cloud CLI) through an env-var override.

## What Happened

Both edges of the feature parsed `gh issue list --json` stdout as pure JSON, and every fake gh in the tests printed clean JSON. All 4605 tests passed. The first live run on main gave `gh output unparseable`, because this machine's `~/.local/bin/gh` is a mise wrapper that prints `mise ... tools: gh@2.101.0` on stdout before the JSON. Dedupe would have fallen through to create duplicate public issues.

## Root Cause

The fake encoded the author's idea of the tool, not what the tool on a real machine prints. Only the live run saw the gap.

## Recommendation

When a test fakes an external CLI, give the fake at least one case that prints a wrapper-style noise line before the real output, and parse from the first line that has the expected shape. Drive the real tool once before calling the feature done.
