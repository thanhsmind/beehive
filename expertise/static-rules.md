# How to turn a check into a static rule

A mistake that a check can stop belongs as high in the stack as a check
can sit. A reviewer's comment, a line of prose, and a test that runs only
when someone remembers are all weaker than a tool that refuses the wrong
code before anyone reads it. This guide covers the case where a past
mistake's fix lives at the check layer and you must decide what kind of
check to write.

## Where to look

| Situation / goal | Entry |
|---|---|
| Choosing what kind of rule to write | Pick the highest home that can hold the rule |
| Finding where a rule goes in the project's linter | Where the rule lives |
| Proving the rule works | Show it red, then green |
| Staging the mistake without leaving it in the tree | Stage the red proof |
| The rule exists but CI never fails on it | Make the rule run |
| The rule needs a tool the project does not run | Ask before adding a tool |
| Deciding whether a rule is worth writing | When not to write a rule |

## Pick the highest home that can hold the rule

Try the four homes in this order and take the first that can express the
rule:

1. **A type the compiler enforces.** Make the wrong value impossible to
   build: a distinct type for a validated id, an enum in place of a string
   flag, a constructor that refuses bad input. The compiler runs on every
   build, so nothing needs to be remembered or wired.
2. **A rule in the linter the project already runs.** Right for "never call
   this function", "never import that module here", "never write this
   shape of expression".
3. **A test.** Right when the rule needs to run code or compare behavior,
   which a syntax-level rule cannot see.
4. **A hook or guard.** Right when the mistake happens outside the code,
   such as a command, a file path, or a commit, and no source-level tool
   sees it.

A project with no compiler-checked types starts at the lint home.

## Where the rule lives

Name the file your project's linter already reads, then add the rule
there.

- **ESLint**: `no-restricted-syntax`, `no-restricted-imports`, or a local
  rule in the project's own plugin.
- **Clippy**: `disallowed-methods` and `disallowed-types` in `clippy.toml`.
- **Ruff**: `banned-api` under `flake8-tidy-imports`.
- **golangci-lint**: `forbidigo` for names, `depguard` for imports.
- **A pattern rule tool**: ast-grep or Semgrep, when the project's linter
  cannot express the rule. Each takes a rule file that matches code by
  shape.

Other ecosystems keep their rules in .NET banned-API analyzers,
ErrorProne or ArchUnit for Java, detekt for Kotlin, RuboCop custom cops,
PHPStan custom rules, SwiftLint `custom_rules`, and Credo for Elixir.

## Show it red, then green

A rule never seen red proves only that it loads. A linter that ignores
your config, a pattern with a typo, and a rule scoped to the wrong
directory all pass silently on every file, including the one that
holds the mistake.

Prove the rule in this order:

1. Run the rule on the code that made the mistake. It must fail, and the
   failure must name the mistake.
2. Apply the fix. Run the rule again. It must pass.

Record both runs, with the command and what each showed.

## Stage the red proof

The past mistake is usually already fixed, so the red run needs a
stand-in. Rebuild the mistake as a small fixture, or as a scratch edit to
a real file that reintroduces it. Run the rule and read the failure. Then
remove the fixture or revert the edit, so the mistake does not stay in the
tree and the rule does not fail on its own proof. Run the rule once more
on the clean tree to see green.

If the rule must keep a fixture to stay honest, as a rule-tool test
directory does, keep it where that tool expects test cases and nowhere
the project's own lint run reads.

## Make the rule run

A rule counts only when CI sees it. The check must be a member of the
project's `commands.test` list, because that list is what the test run
and the proof on a cap come from. A linter the project runs only in an
editor, or a script nobody calls, never fails a build.

If the rule rides on a linter the project already lists, it runs already.
If it needs its own command, add the command to `commands.test`, placed
before the tests.

When the project has no `commands.test` list, or its value is the sentinel
`none`, stop. Take the question to the owner instead of writing a rule
that nothing runs.

## Ask before adding a tool

A linter, analyzer, or rule tool the project does not already run is the
owner's decision. Each one adds a dependency, a config file, a CI step, and
a new way for the build to fail. Describe the mistake, name the tool and
the alternative of a test, and ask. Never install it on your own.

## When not to write a rule

- **A one-off.** A mistake nobody would repeat costs more to guard than to
  fix. Fix it and move on.
- **A rule with false positives.** A rule that flags correct code teaches
  people to switch it off, and a switched-off rule protects nothing. If
  you cannot write the pattern narrowly enough, use a test, or leave the
  rule to prose.
