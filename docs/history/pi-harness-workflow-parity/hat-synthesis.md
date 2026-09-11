# Plan consult: Pi harness workflow parity

## Returns

- `hat-user-impact` returned through herding as job `job-1789133085436`.
- `hat-value` returned through herding as job `job-1789133330059`.
- `hat-facts-gaps`, `hat-alternatives`, and `hat-risks` were dropped. Their configured Agent transport is not available in this Pi session.
- Validation repair ran again as job `job-1789133624229`. It found no new issue after regeneration moved to the final cell.
- All returned prompts lost their questions. Each worker recovered by reading the active feature documents. This is direct evidence for cell `pihp-2`.

## Accepted

Use a separate conditional `purpose` variable in all three non-cell templates. Keep advisor `brief` for `--brief-file`.

## Dismissed

- Finding: allow a null or absent cell `verify` value. Dismissed because new cells require a non-empty `verify`. Historical caps stay compatible on read.
- Finding: let full bypass create the preview automatically. Dismissed because bypass changes the approval actor only. It does not skip preparation.

## Result

No blocker remains in the plan. The exact cell packet covers each locked decision and each accepted clarification.
