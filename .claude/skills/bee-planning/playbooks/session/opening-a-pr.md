# opening-a-pr

You own how the work reaches review. Land it through the worktree, the cap
commits, the draft PR and the merge path bee already has, and write the words a
reviewer will read.

Invoked at the end of every other playbook. In bee a PR is not opened by hand
for each task: the `ship_visibility` rule opens a draft PR on a feature's first
capped cell and pushes on every later cap, and `bee worktree merge` and `bee
close` are the way it lands. This playbook is what you do inside that path.

1. **Work from a worktree off main.** Code-touching work runs in its own
   worktree, never on main directly, and a worker inherits the worktree of the
   cell it holds. Two workers never share a branch: each gets its own worktree.
   A dirty branch with unrelated work: patch the unrelated edits out, start a
   fresh worktree and apply only what belongs. A snarled worktree: reset from
   main and redo it minimally. Both beat untangling, because an untangled
   history hides what changed.
2. **Commit liberally, then leave small ordered commits.** Each commit is a
   future review unit: landable on its own, ordered to tell the story. Before
   the cap, amend when the fix belongs in the commit you just made and add a new
   commit when it is separable. After the cap, add a new commit, because the cap
   recorded that commit's proof. Every cap commit ends with the `cell: <id>`
   trailer.
3. **Write the PR title, PR description and commit body with
   `bee-technical-writing`, then `bee-unslop`.** That skill holds the title
   shape, the section order and the layers; this playbook does not restate them,
   so the two cannot drift apart. Attach a video or a screenshot when it proves a
   claim, and leave out `## Summary` and `## Test plan` boilerplate. Use one word for each action, keep articles,
   and avoid `-ing` when a plain verb works, because a reviewer reads a
   description once and an ambiguous word costs a round trip. A commit body does
   not restate its subject. Write no comment in the code you are about to put up
   for review, because bee's rule is that the why goes to a knowledge concept or
   the decisions log, and run `bee-comment-review` over the diff before review
   to catch the ones that slipped in.
4. **Keep the PR narrow.** Prefer five narrow features to one large one, and
   keep each ordered so a reviewer can read it bottom-up. Follow-ups branch
   from current main after the first merges; branch from main only for
   independent work. Start substantial work from current main, because a branch
   that starts behind pays the drift later in a conflict.
5. **Keep it a draft until the owner says otherwise.** The `ship_visibility`
   rule opens the PR as a draft when a GitHub remote and `gh` exist, and the
   draft stays a draft: it is a window onto the work, not a ship decision.
   Marking it ready, and merging it, are the owner's call or the verb's, never
   a step here, because a PR that lands itself skips the proof-line and uat
   checks. Run `gh pr view <n>` before you refer to a PR's status, because
   memory of it goes stale the moment a check finishes. Pushing publishes, so
   the secret scan and commit hygiene apply unchanged, and `ship_visibility`
   never overrides a repo's no-push policy.
6. **Land through the bee path.** On green the merge is `bee worktree merge`
   from the main checkout, without asking, as `uat_stop = "close"` already does,
   and the `uat` door of `bee close` stays the owner's: uat is never
   auto-approved at any bypass level. A feature's own landing is `bee feature
   finish`, which runs that merge, then the scribing stamp, then `bee close`,
   and stops at the first refusal.
7. **Opening a PR does not start a babysit.** Post the URL and keep building.
   Finish the phase or the program first. Run a separate `babysit` pass only
   when the owner asks for one after the whole program exists, because a babysit
   for each new PR stalls the build and spends checks on commits that later
   waves restart. Push back when review feedback drifts from the intent.
8. **A worker that pushes returns the URL and nothing more.** It runs
   `bee-interrogate` and `bee-comment-review` over its diff, returns the PR URL
   and does not babysit. Return to the parent, because the lead owns the
   frontier and two agents on one PR is the failure `babysit` step 3 names.

Reply: the PR URL and its state from `gh pr view`, the commits in order, and
what remains before the merge path runs.

Proof line: `gh pr view <n> --json state,isDraft` and `git log --oneline
main..HEAD`, verbatim.
