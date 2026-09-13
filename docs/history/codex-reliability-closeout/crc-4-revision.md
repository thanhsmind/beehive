# crc-4 test contract correction

Continue the same assigned cell. Source parser fix is correct and should remain one added flag name.
Your tests seed and assert gates.shape.approved / gates.execution.approved. Those are not the actual lane approval fields. Current lanes carry approved_gates.shape and approved_gates.execution (plain booleans), as the real state output confirms. Preserving an inert gates object would not detect accidental actual approval.
Update fixtures and assertions to approved_gates, and test both default and lane preview paths. Include both false and already-true approvals so preview preserves state rather than merely clearing it. Use before/after real fields or state readers, not an invented shape. Keep existing alias failure and successful subcommand checks. No need for production changes beyond the correct one-line parser fix.
Run the focused CLI test suite. Append the correction and output to preview.md, preserve old evidence. Finish through the shared control-plane CLI with an exact report or name its refusal. Do not rewrite sibling commits; integration will consolidate this cell's commits.
