# release

1. Run the repo's release script end to end — in bee, `scripts/release.sh
   <VERSION>`. The script IS the release; a hand-walked checklist is a step
   that gets skipped.
2. Let the declared suite run BEFORE anything is tagged. Skipping it is a flag
   you own out loud, never a quiet shortcut.
3. Wait for the script's final `OK` line — tag pushed, release CI green,
   published assets verified.
4. Re-run the same version when a run dies mid-flight; the script is
   idempotent and picks the release back up.

A release commit without that `OK` line is not a release.

Proof line: the script's final `OK` line, verbatim.
