# Spec drop sup-20261028-c9d4 (from waggledance@09d94dd), as received

## Owner brief: bee writes a decision-ready brief for every backlog item and feature

<!-- bee:not-a-deferral: the spec drop as received; it describes the deferral gap, it promises nothing -->
### Why
The owner reads every bee project's work on work.gogl.be. agent-works mirrors each repo's features and backlog items onto that board. Today a backlog item carries only a title and a one-sentence `cos`. Example: p-bea1cb2a in agent-works says only "A comment by a person on a mirrored task reaches the mapped project's leader as a dispatch; deferred from wd-mirror shaping". The owner said (2026-10-28): "nó quá ngắn … tôi đọc không hiểu, dẫn tới không approve hay có action khác được". The why, the tradeoff and the reason for deferral exist only in the deferring feature's CONTEXT.md and plan.md, and nobody surfaces them.
<!-- /bee:not-a-deferral -->

The waggledance leader and agent-works agreed on a brief format, and the owner approved it. Right now waggledance is backfilling about 98 briefs by hand, and agent-works's mirror is being changed to read them. Both are patches. The real fix belongs in bee: every new item is born with a brief, in every repo.

### The brief contract (keep it byte-compatible; consumers already read it)
- Path: `docs/briefs/<id>.md`, where `<id>` is the PBI id or the feature slug.
- Frontmatter: flat `key: value` lines between `---` lines, no nesting.
  - `title`: at most 80 characters, in the project's owner language (Vietnamese for this owner). It states the outcome.
  - `audience`: `owner` or `internal`.
  - `size`: `S`, `M` or `L`.
  - `ask`: `build`, `park`, `drop`, `answer` or `test`.
- Body: six paragraphs, each starting with a bold label, 120 to 220 words in total.
  - **Là gì:** what happens today, and what happens after the work.
  - **Vì sao:** the pain it removes, and who feels it.
  - **Nguồn gốc:** one line.
  - **Chi phí và rủi ro:** the size, one or two risks, and what it depends on.
  - **Anh cần làm:** two or three options, plus one recommendation with a one-line reason.
  - **Chuẩn xong:** two to four observable bullets.
- The body has no ids, file paths or function names. Consumers add a technical footer.
- Labels and owner language: decide how bee configures them (for example, `brief.language` in `.bee/config.json`). The label set above is what consumers parse today, so keep it as the default for `vi`.

<!-- bee:not-a-deferral: the spec drop as received; it describes the deferral gap, it promises nothing -->
### What to build in bee
1. **PBI add.** `bee backlog pbi add` writes or requires the brief. The agent that adds the item has the context, so it writes the brief at that moment. Choose a flag shape, for example `--brief-file`, or the skill writing the file before the call. If an item is added with no brief, it is recorded and flagged. It is never silently accepted.
2. **Shaping lock.** bee-shaping's Lock writes `docs/briefs/<feature>.md` for the feature it locks, drawn from the Feature Boundary and the Locked Decisions.
3. **Deferral.** When shaping or planning defers scope into a backlog item, the brief carries the reason for the deferral and its cost in "Chi phí và rủi ro". This is the exact gap behind p-bea1cb2a.
4. **Check.** A doctor row, a `bee backlog` check, or both, flags any live item (proposed, ready or in-flight) or active feature that has no brief or a malformed one. Optionally, give the brief a `sources_digest` and flag it as stale when its sources change.
5. **Folding.** A PBI that is in-flight under an active feature is shown inside that feature, not as its own item. Make sure this holds wherever bee itself lists items for the owner, and document it so consumers apply the same rule.
6. **Docs and skills.** Update the skills (bee-shaping, bee-planning deferral, PBI add) and the knowledge so that cold agents write good briefs. Include one good example and one bad one; the bad one is the one-sentence cos above.
<!-- /bee:not-a-deferral -->

### Constraints
- Keep the path and format above unchanged. waggledance (/api work-items, planned) and agent-works (integrations/waggledance/mirror.mjs, in flight) read exactly this.
- Do not backfill other repos. waggledance is backfilling its own, and each repo's leader will backfill after this release.
- Keep the cost small. The brief is written once, at creation, by the agent that holds the context. Never write it with an LLM call on every read.

### Contract
Run this repo's full working chain end to end; merge only through this repo's own `uat_stop` configuration and gate approvals; report the release version that carries it.
