// ─── tool -> hook mapping (the only "rule" in this file) ───────────────────

export type MappedCall = {
  hook: "write-guard" | null
  tool_name: string
  tool_input: Record<string, unknown>
  /** true only when tool_input is the caller's own input forwarded verbatim —
   * the one shape a bee `updatedInput` repair can be applied onto. */
  passthrough: boolean
}

/** Pi 0.84.3's COMPLETE built-in tool registry, enumerated from the installed
 * binary rather than guessed. Two independent version-matched anchors agree on
 * the same eight names:
 *   - docs/settings.md `defaultTools`: "Available built-ins are `read`,
 *     `bash`, `powershell`, `edit`, `write`, `grep`, `find`, and `ls`";
 *   - docs/extensions.md "Overriding Built-in Tools": the same eight;
 * and each argument shape below was read off the binary's own typebox schemas
 * (`bashSchema` {command,timeout}, `readSchema` {path,offset,limit},
 * `writeSchema` {path,content}, `editSchema` {path,edits[]}, `grepSchema`
 * {pattern,path,glob,ignoreCase,literal,context,limit}, `findSchema`
 * {pattern,path,limit}, `lsSchema` {path,limit}; `powershell` shares the shell
 * tool's schema with `bash`).
 *
 * Kept as a NAMED LIST, not a switch default, because the fail-safe below
 * depends on knowing exactly which names are enumerated.
 *
 * Re-verified 2026-10-02 against Pi 1.0.0: `docs/settings.md` carries the
 * identical eight names, and no changelog entry between 0.84.3 and 1.0.0
 * adds or removes a built-in. The list is still
 * complete at the ceiling named in the header. */
export const PI_BUILTIN_TOOLS = [
  "bash",
  "powershell",
  "read",
  "write",
  "edit",
  "grep",
  "find",
  "ls",
] as const

export const BEE_STAGE_TOOLS = new Set<string>([
  ...PI_BUILTIN_TOOLS,
  "codemode",
  "tool_search",
  "verdict",
  "bee_dispatch",
  "bee_advisor",
  "bee_steer",
])

/** Field names a custom tool might carry a write target under, in probe
 * order — used ONLY by the fail-safe route below. */
export const PATH_FIELDS = [
  "file_path",
  "filePath",
  "path",
  "file",
  "target",
  "destination",
  "dest",
  "output",
  "outputPath",
]

export function firstString(input: any, keys: string[]): string | undefined {
  if (!input || typeof input !== "object") return undefined
  for (const key of keys) {
    const value = input[key]
    if (typeof value === "string" && value.length > 0) return value
  }
  return undefined
}

/** Which Pi tool bee's blocking hook sees as what, and the field-name
 * translation into the PreToolUse shape bee already reads
 * (packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs).
 *
 * This function NEVER returns null. A name outside PI_BUILTIN_TOOLS — a custom
 * `pi.registerTool` tool from a sibling extension, a tool added by a future Pi
 * release, an override of a built-in — is routed to write-guard as a
 * WRITE-CAPABLE call, because a tool this file does not recognise is a
 * TypeScript-side allow otherwise, and that is the one bypass this file exists
 * to close. bee still owns the verdict; the fail-safe only decides which
 * SHAPE the unknown call is presented in, never whether it passes.
 *
 * bee's write-capable set is `Edit | Write | MultiEdit | Bash | apply_patch`
 * (write_guard/main.rs:64-67), so the fail-safe picks between the two shapes
 * that carry a target: a `command` string routes as Bash, everything else
 * routes as Write with the first path-shaped field found, and the raw input
 * rides along untouched so no field is hidden from a future detector. */
export function mapToolCall(tool: string, input: any): MappedCall {
  const args = (input && typeof input === "object" ? input : {}) as Record<string, unknown>

  switch (tool) {
    case "bash":
    case "powershell":
      // Both shell tools share `{ command, timeout }`; bee reads `command`.
      return {
        hook: "write-guard",
        tool_name: "Bash",
        tool_input: { command: args.command },
        passthrough: false,
      }

    case "write":
      return {
        hook: "write-guard",
        tool_name: "Write",
        tool_input: { file_path: args.path, content: args.content },
        passthrough: false,
      }

    case "edit":
      // Pi's `edit` takes an ARRAY of replacements (`edits: [{oldText,
      // newText}]`), which is Claude's MultiEdit shape, not its single-edit
      // Edit shape — MultiEdit is the honest name here, and bee treats all
      // three write-tool names identically (write_guard/main.rs:64, reading
      // only `file_path`). The entries are translated into bee's own field
      // names so a future detector reading them finds the shape it expects.
      return {
        hook: "write-guard",
        tool_name: "MultiEdit",
        tool_input: {
          file_path: args.path,
          edits: Array.isArray(args.edits)
            ? args.edits.map((edit: any) => ({
                old_string: edit?.oldText,
                new_string: edit?.newText,
              }))
            : undefined,
        },
        passthrough: false,
      }

    case "read":
      // path -> file_path. offset/limit are forwarded ONLY when Pi actually
      // supplied them: bee's "unbounded read" size-denial check
      // (write_guard/main.rs:121-124) fires only when tool_name === "Read" AND
      // tool_input has NEITHER an "offset" NOR a "limit" key. JSON.stringify
      // drops an `undefined` property outright, so an omitted Pi argument
      // reads on bee's side as truly absent, not merely falsy — the
      // presence/absence signal survives the translation exactly.
      return {
        hook: "write-guard",
        tool_name: "Read",
        tool_input: { file_path: args.path, offset: args.offset, limit: args.limit },
        passthrough: false,
      }

    case "grep":
      // bee's read-guard resolves the target from tool_input.file_path OR
      // .path (write_guard/main.rs:110) — Pi's grep already uses "path" for
      // the same purpose, so no rename is needed for the one field bee reads.
      // Pi's `glob` filter is Claude Grep's `include`; both ride along unread
      // but harmless.
      return {
        hook: "write-guard",
        tool_name: "Grep",
        tool_input: { path: args.path, pattern: args.pattern, include: args.glob },
        passthrough: false,
      }

    case "find":
      return {
        hook: "write-guard",
        tool_name: "Glob",
        tool_input: { path: args.path, pattern: args.pattern },
        passthrough: false,
      }

    case "ls":
      return {
        hook: "write-guard",
        tool_name: "Glob",
        tool_input: { path: args.path },
        passthrough: false,
      }

    case "verdict":
    case "bee_dispatch":
    case "bee_advisor":
    case "bee_steer":
      return {
        hook: "write-guard",
        tool_name: tool,
        tool_input: args,
        passthrough: false,
      }

    case "codemode":
    case "tool_search":
      return { hook: null, tool_name: tool, tool_input: args, passthrough: false }

    default: {
      // FAIL-SAFE. Never a silent allow: bee decides, on the write-capable
      // shape (or read-only web fetch) that best fits the unknown arguments.
      const command = firstString(args, ["command"])
      if (command !== undefined) {
        return {
          hook: "write-guard",
          tool_name: "Bash",
          tool_input: { command },
          passthrough: false,
        }
      }
      const pathTarget = firstString(args, PATH_FIELDS)
      if (pathTarget !== undefined) {
        return {
          hook: "write-guard",
          tool_name: "Write",
          tool_input: { ...args, file_path: pathTarget },
          passthrough: false,
        }
      }
      const url = firstString(args, ["url", "urls"])
      if (url !== undefined) {
        return {
          hook: "write-guard",
          tool_name: "WebFetch",
          tool_input: { ...args, url },
          passthrough: false,
        }
      }
      return {
        hook: "write-guard",
        tool_name: "Write",
        tool_input: { ...args, file_path: "" },
        passthrough: false,
      }
    }
  }
}

