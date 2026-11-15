# cited-claims — plan

Lane: small. Class: feature. Situation: `skills/bee-planning/playbooks/feature/feature.md`.
Architect skipped: no new data shape — D1 is doctrine text, D2 widens one
string check beside the existing one.

## Throughput

- Blocking first steps: none.
- Independent workstreams: the doctrine text (cc-1) and the Stop check
  (cc-2) touch disjoint files and run in parallel.
- Shared mutable state: none.
- Smallest safe decomposition: two implementer cells; cc-2 gets a tester
  cell (cc-3), cc-1 gets a docs checker cell (cc-4).

## Cell map

| Cell | Role | Does | Decisions |
|---|---|---|---|
| cc-1 | docs | doctrine and Claim labels teach numbered citations with a Sources block | D1 |
| cc-2 | code | Stop hook counts a cited reply as checked and names the citation form | D2 |
| cc-3 | test | proves the cited, legacy and uncited cases on transcript fixtures | D2 |
| cc-4 | test | checks the doctrine text, pointers and render parity | D1 |

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | the doctrine asks for a [verified] tag | read | packages/bee/LEADER.md:199 | - Tag each technical claim about code or behavior `[verified]` (read or |
| 2 | the full form shows an inline verified example | read | skills/bee-hive/references/routing-and-contracts.md:311 | **Claim labels.** Every technical claim about code or behavior in a reply carries a label. |
| 3 | the Stop check only accepts the [verified marker | read | packages/bee-rs/crates/bee/src/hooks/session_close/claims.rs:37 | let labelled = text.contains("[verified"); |
| 4 | the warning text names the [verified] label | read | packages/bee-rs/crates/bee/src/hooks/session_close/claims.rs:32 | const WARNING: &str = "bee: this reply agrees with a proposal but carries no [verified] label |

## Proof

Each cell runs its filtered cargo tests or parity checks. The full suite
runs once with `bee test` before `bee worktree merge`.

## Cells — current slice (preview)

```json
[
  {
    "id": "cc-1",
    "feature": "cited-claims",
    "title": "Teach numbered source citations in the claim doctrine",
    "lane": "small",
    "role": "docs",
    "deps": [],
    "decisions": ["D1", "06a46ab6-577b-42c5-8692-6430b02b23fe"],
    "files": [
      "packages/bee/LEADER.md",
      ".bee/LEADER.md",
      "skills/bee-hive/references/routing-and-contracts.md",
      "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
      ".claude-plugin/skills/bee-hive/",
      ".codex-plugin/skills/bee-hive/",
      ".claude/skills/bee-hive/",
      ".agents/skills/bee-hive/",
      ".opencode/skills/bee-hive/",
      "docs/history/codex-harness-hardening/release-manifest.json",
      ".bee/onboarding.json"
    ],
    "read_first": [
      "docs/history/cited-claims/CONTEXT.md",
      "packages/bee/LEADER.md",
      "skills/bee-hive/references/routing-and-contracts.md"
    ],
    "action": "D1. In packages/bee/LEADER.md rule agents-verify-before-agreeing (around line 199), replace the [verified]/[unverified] tag bullet: a technical claim the session checked carries a numbered citation [1], [2] in the text, and the reply ends with a Sources block (heading Sources: or Nguồn:), one line per number, [n] path:line or [n] the command run with its result; one source keeps one number on reuse; a claim from general knowledge or inference keeps the inline [unverified] marker and gets no number, and is never stated as fact. Change 'until its key assumption is [verified]' to 'until its key assumption has a citation', and the Stop hook sentence to say it warns when a reply agrees with no citation. Copy the same bytes to .bee/LEADER.md (it is the onboarded copy and must stay byte-identical). In skills/bee-hive/references/routing-and-contracts.md section Claim labels and pushback, rewrite the Claim labels paragraph and its text example to the citation form (one example reply with two citations and its Sources block, one [unverified] line), and the First reaction paragraph to 'Until the key assumption has a citation'. Keep the section heading and the Pushback text unchanged. Update the spoken line of agents-verify-before-agreeing in docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md to say cite each checked code claim with a numbered source listed under the reply. Run bee dev regen to re-render the skill trees, then restore .claude/skills hub-served skills other than bee-hive if regen touched them. One commit. Bee commands you may run: bee dev regen, bee dev release-manifest --check, bee onboard --repo-root . --json, bee knowledge check. Run no dispatch verb, not even its help.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test rule_index_parity && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pointer_integrity && .bee/bin/bee dev release-manifest --check && .bee/bin/bee onboard --repo-root . --json",
    "must_haves": {
      "truths": [
        "LEADER.md rule agents-verify-before-agreeing asks for numbered citations with a Sources block under the reply",
        "the Claim labels section shows one example reply with numbered citations and its Sources block",
        "an unchecked claim keeps the [unverified] marker and is never stated as fact"
      ],
      "artifacts": [
        {"path": "packages/bee/LEADER.md", "substantive": "citation form in agents-verify-before-agreeing"},
        {"path": "skills/bee-hive/references/routing-and-contracts.md", "substantive": "Claim labels citation form and example"}
      ],
      "key_links": [
        ".bee/LEADER.md stays byte-identical to packages/bee/LEADER.md"
      ],
      "prohibitions": [
        "No new numbered rule",
        "No code edits",
        "No change to the Pushback text"
      ]
    },
    "affects_skills": ["skills/bee-hive/references/routing-and-contracts.md"],
    "affects_specs": ["docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md"]
  },
  {
    "id": "cc-2",
    "feature": "cited-claims",
    "title": "Count a reply with numbered sources as checked in the Stop hook",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D2", "4c6f834c-58fd-46d0-b26a-b3158a759ff1"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/session_close/claims.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs",
      "docs/knowledge/areas/hook-runtime/overview.md"
    ],
    "read_first": [
      "docs/history/cited-claims/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/hooks/session_close/claims.rs",
      "docs/knowledge/areas/hook-runtime/overview.md"
    ],
    "action": "D2. In claims.rs unverified_agreement_warning, the reply counts as checked when the lowercased last-turn text contains the legacy '[verified' marker, or when it is cited: some inline [n] (n one or more digits) appears AND, after a line whose trimmed text is 'sources:' or 'nguồn:' (also accept a markdown heading or bold form of those words, e.g. '## sources' or '**nguồn:**'), some line starts (after trim) with that same [n]. Keep it a small pure function over the text; no regex crate if not already a dependency of bee. Rewrite WARNING to name the citation form: the reply agrees with a proposal but cites no source — cite the key assumption as [1] with a Sources line, then agree with what the check showed; keep the rule id agents-verify-before-agreeing. The conditional markers still silence it. Advisory, Stop only, never a block. Add unit tests in session_close/tests.rs for a cited agreement (quiet), a citation with no Sources block (warns), a Vietnamese Nguồn: block (quiet), and keep the existing legacy [verified] test. Sync the Stop agreement paragraph of docs/knowledge/areas/hook-runtime/overview.md (around line 89) citing cited-claims D2. One commit. Bee commands you may run: bee knowledge check. Run no dispatch verb, not even its help.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- session_close",
    "must_haves": {
      "truths": [
        "an agreeing reply with an inline [1] and a Sources block line starting with [1] prints no agreement warning",
        "an agreeing reply with an inline [1] and no Sources block prints the agreement warning",
        "an agreeing reply with the legacy [verified marker prints no agreement warning",
        "the agreement warning never blocks the Stop"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/session_close/claims.rs", "substantive": "cited-reply check in unverified_agreement_warning"}
      ],
      "key_links": [
        "mod.rs adds claims warnings to the Stop parts only on the Done path"
      ],
      "prohibitions": [
        "No code comments",
        "No block or exit-code change in the Stop hook",
        "No new config key"
      ]
    },
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/overview.md"]
  },
  {
    "id": "cc-3",
    "feature": "cited-claims",
    "title": "Prove the cited-reply check on transcript fixtures",
    "lane": "standard",
    "role": "test",
    "deps": ["cc-2"],
    "decisions": ["D2", "4c6f834c-58fd-46d0-b26a-b3158a759ff1"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs"
    ],
    "read_first": [
      "docs/history/cited-claims/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/hooks/session_close/claims.rs"
    ],
    "action": "Test only, no product code. Build agreeing-reply fixtures in both the Claude shape and the Codex shape. Cover: inline [1] with a Sources: line [1] (quiet); inline [2] with a Nguồn: heading and a [2] line (quiet); inline [1] but the Sources block lists only [3] (warns); inline [1] with no Sources heading (warns); an array index like v[1] in prose with no Sources block (warns); legacy [verified] (quiet); a conditional 'checking first' reply (quiet). Drive the Stop hook entry end to end where existing session_close tests do and assert it never blocks. Show each new test red against a mutation of the cc-2 check. A red against the product is a finding returned as blocked. Bee commands you may run: none beyond the verify. Run no dispatch verb, not even its help.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- session_close",
    "must_haves": {
      "truths": [
        "a test fails if a cited reply with a matching Sources line still warns",
        "a test fails if a citation without a matching Sources line goes quiet",
        "a test fails if the agreement warning blocks the Stop"
      ],
      "artifacts": [],
      "key_links": [],
      "prohibitions": [
        "No code comments",
        "No product code edits"
      ]
    },
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "cc-4",
    "feature": "cited-claims",
    "title": "Check the citation doctrine text and its skill pointers",
    "lane": "small",
    "role": "test",
    "deps": ["cc-1"],
    "decisions": ["D1", "06a46ab6-577b-42c5-8692-6430b02b23fe"],
    "files": [
      "docs/history/cited-claims/cc-4-check.md"
    ],
    "read_first": [
      "docs/history/cited-claims/CONTEXT.md",
      "packages/bee/LEADER.md",
      "skills/bee-hive/references/routing-and-contracts.md"
    ],
    "action": "Docs checker, no edits to skills, doctrine or code. Read the cc-1 commit. Confirm LEADER.md rule agents-verify-before-agreeing and the Claim labels section teach numbered citations with a Sources block under the reply, keep [unverified] for unchecked claims, and no longer ask for a bare [verified] tag; that .bee/LEADER.md is byte-identical to packages/bee/LEADER.md; that the Pushback text is unchanged; and that the rendered bee-hive copies match the source. Write docs/history/cited-claims/cc-4-check.md with what you checked, the command output summary and a pass or fail verdict. A fail is returned as blocked with the evidence. Bee commands you may run: bee dev release-manifest --check. Run no dispatch verb, not even its help.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test rule_index_parity && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pointer_integrity && .bee/bin/bee dev release-manifest --check",
    "judge_obligation_ack": "docs checker over one rule paragraph, no guard logic",
    "must_haves": {
      "truths": [
        "the checker report states pass only when both doctrine homes teach the citation form and .bee/LEADER.md matches its source",
        "rule index parity and pointer integrity tests pass"
      ],
      "artifacts": [
        {"path": "docs/history/cited-claims/cc-4-check.md", "substantive": "check record with verdict"}
      ],
      "key_links": [],
      "prohibitions": [
        "No edits outside the check record"
      ]
    },
    "affects_skills": [],
    "affects_specs": []
  }
]
```
