# When bee itself is at fault

<!-- rule: agents-bee-defect-report -->
- When bee itself is at fault — a crash, a typed error on correct
  input, a wrong result, or a guard refusing work that is approved and
  correct — run `bee report issue --title --symptom --evidence
  --root-cause --command`, and add bee's exact output with `--output`.
  It files a scrubbed issue upstream on its own; add `--dry-run` first
  when unsure. One report per defect, never one per retry. After
  filing, tell the human the issue link in one line. On a gh error,
  tell the human once to run `gh auth login`, then keep working. Never
  patch, rebuild or hand-edit bee in this repo. Not a bee defect: a
  deny that names its remedy, a gate waiting for approval, or your own
  mistake — that one is `bee mailbox reflect`.
<!-- /rule -->
