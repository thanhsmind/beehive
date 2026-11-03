# Execution order and native dispatch evidence

Decision a95e7d19-4372-4d3b-a032-8b97d4952813 changes only the scheduling edge: cpc-2 no longer waits for cpc-1 completion. The leader keeps every acceptance requirement; installed native-hook proof still blocks integration. cpc-2 code files are disjoint from the running hook-discovery work.

Current callable native spawn schema:
- required task_name and message;
- optional fork_turns, model, reasoning_effort;
- no agent_type, subagent_type, sandbox or per-child permission field;
- full-history fork inherits parent model/effort and rejects overrides; use none or a numeric fork for explicit model/effort.
Do not infer an available model override from a role marker. Test the actual policy fields and transport mismatch using the shared configured-role resolver. Prepared model-shaped read-only jobs need a real restricted CLI transport, not a prompt promise. Configured external transports remain their configured path.

Official current hook reference: https://learn.chatgpt.com/docs/hooks (read 2026-09-12).
It documents shell/unified-exec matching as Bash, patch matching by apply_patch/Edit/Write, and native spawn matching by spawn_agent/Agent. It also distinguishes project-layer trust from individual hook trust. Treat these as documentation evidence, not observations from the installed canary.

Leader live diagnostic:
- Codex 0.154.0 ran a read-only pwd command with --sandbox read-only --ephemeral and the already-approved per-invocation hook-trust option; exit 0.
- Evidence: /tmp/bee-cpc-root-6HMYd0/evidence/20260912-220834-1304278/002-timeout-60-codex-exec-sandbox-read-only-dangerously-bypass-h.{cmd,out,err,exit}
- Global hook events appeared, but the temporary project's added recorder did not run. This is NOT installed-project proof.
- A Cloudflare MCP authentication warning occurred, but the Codex command still completed successfully. That warning alone is not a Codex auth failure.
- Do not change real user config, trust records or credentials. Use existing verification helpers for sandbox writes; a direct attempt to mkdir outside the physical worktree was refused and abandoned.

Before code: write and run RED regressions, including wrong direct model/effort/full-fork settings, role transport mismatch, and read-only fallback behavior. Preserve Claude. Do not introduce a second resolver. Keep proof labels honest and preserve evidence.

