# Native transcript shape observation

Observed on the current Codex 0.154.0 session, 2026-09-12. A jq projection emitted field names, message roles, phases and content types only. It did not emit user text, tool arguments or encrypted content.

- session_meta.payload has id and session_id, cli_version, cwd and source.
- response_item.payload.type=message has role=assistant, phase=commentary or phase=final_answer, and content[].type=output_text.
- event_msg.payload.type=task_started names a turn_id.
- event_msg.payload.type=task_complete includes turn_id and last_agent_message.
- event_msg.payload.type=token_count includes info.total_token_usage and info.last_token_usage.
- Both usage objects contain input_tokens, cached_input_tokens, cache_write_input_tokens, output_tokens, reasoning_output_tokens and total_tokens.
- response_item also carries tool calls, tool outputs, reasoning and agent_message variants. Their presence alone is not final text.

This is observed transcript structure, not proof that Stop fires after these records flush. The installed-hook canary must establish event timing. Do not add reasoning_output_tokens to output_tokens without proving they are disjoint. Do not add every cumulative total event.

