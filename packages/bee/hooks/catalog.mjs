// hooks/catalog.mjs — the single logical hook catalog for bee's runtime
// parity (cell codex-parity-2, docs/history/codex-runtime-parity/approach.md
// section 1 "Plugin-first distribution and one active source", decisions
// D1/D2 in CONTEXT.md).
//
// hooks/hooks.json is rendered from this catalog as the CODEX default
// projection: the Codex plugin manifest (.codex-plugin/plugin.json) omits an
// explicit "hooks" path and Codex loads hooks/hooks.json from its default
// plugin-root location (approach.md: "Make hooks/hooks.json the Codex
// default projection so the Codex manifest can omit a redundant hooks
// field."). hooks/claude-hooks.json is rendered as the CLAUDE projection and
// is wired explicitly by .claude-plugin/plugin.json's "hooks" field — the
// two projections and that manifest switch land in one atomic commit (P1
// repair, plan-review.md) so no intermediate state leaves Claude consuming
// the Codex projection.
//
// Runtime differences are explicit data, never hand-maintained projection
// drift. Both runtimes carry a native pre-spawn model guard, but on the tool
// matcher each runtime actually exposes: Claude guards Agent|Task, Codex
// guards spawn_agent (the collaboration-spawn tool name observed reaching
// PreToolUse on codex-cli 0.144.4 — codex-native-runtime-v2 D4, capability
// matrix row D1). Codex alone has the post-start / stop native-subagent audit
// hooks because those are the events Codex exposes today. See
// ALLOWED_DIFFERENCES and the drift-check test in hooks/test_hook_contracts.mjs.
// No wrapper .mjs is forked per runtime — only this catalog's projection
// output differs; see ALLOWED_DIFFERENCES and the drift-check test in
// hooks/test_hook_contracts.mjs.
//
// Version-parity guard and publisher metadata (BEE_VERSION, both plugin
// manifests' "version"/publisher fields) are explicitly OUT of scope here —
// they belong to the Distribution slice (approach.md "Likely Files" item 3).

export const RUNTIMES = Object.freeze({ CLAUDE: "claude", CODEX: "codex" });

// A projection is rendered for a TARGET as well as a runtime (cell
// codex-parity-6a):
//
//   "plugin" (DEFAULT) — the hook file ships inside the plugin root, so the
//     host exports ${CLAUDE_PLUGIN_ROOT} and the command resolves against it.
//     This is what hooks/hooks.json and hooks/claude-hooks.json are rendered
//     as; keeping it the default is what makes those two checked-in
//     projections, and every existing caller, byte-identical across this cell.
//
//   "repo" — the SOURCE-REPOSITORY fallback (.codex/hooks.json). Codex loads
//     a project's own .codex/hooks.json with NO plugin root exported, and the
//     old hand-authored file resolved through "$CLAUDE_PROJECT_DIR" — a
//     Claude-only variable that Codex never sets (0 occurrences in the shipped
//     Codex binary). With it unset the command collapsed to
//     `node /.bee/bin/hooks/bee-state-sync.mjs` and every hook died with
//     MODULE_NOT_FOUND (docs/history/codex-runtime-parity/reports/
//     diagnosis-codex-stop-hooks.md). The repo target instead resolves the git
//     root from Codex's session cwd, which the official hooks contract
//     guarantees (decision d91a8398) and which Codex's `$SHELL -lc` command
//     runner makes possible.
export const TARGETS = Object.freeze({ PLUGIN: "plugin", REPO: "repo" });

const BOTH = Object.freeze([RUNTIMES.CLAUDE, RUNTIMES.CODEX]);
const CLAUDE_ONLY = Object.freeze([RUNTIMES.CLAUDE]);
const CODEX_ONLY = Object.freeze([RUNTIMES.CODEX]);

// The PINNED fail-open diagnostic for the repo transport. It goes to STDERR
// and the command writes NOTHING to stdout: stdout on a Stop hook must parse
// as a JSON systemMessage, so a diagnostic on stdout would break that same
// command. Fail-open must be VISIBLE, never silent (spec R2) — a bare
// `[ -n "$r" ] || exit 0` is silent and violates it. Cell codex-parity-6b
// asserts this literal mechanically; do not paraphrase it.
export const REPO_TRANSPORT_UNAVAILABLE_DIAGNOSTIC =
  "bee: hook transport unavailable (no git root)";

// Pre-wrapper transport setup for the repo target: resolve the git root from
// the session cwd. `git rev-parse` fails the same way whether git is absent
// from PATH or the cwd is not a repository — both leave $r empty, both take
// the visible fail-open arm (exit 0), and NEITHER reaches node. That arm is
// load-bearing, not decorative: without it, the same command in a non-git cwd
// crashes with exactly the MODULE_NOT_FOUND this cell repairs.
function repoCommand(script) {
  return [
    'r="$(git rev-parse --show-toplevel 2>/dev/null)"',
    `[ -n "$r" ] || { echo "${REPO_TRANSPORT_UNAVAILABLE_DIAGNOSTIC}" >&2; exit 0; }`,
    `exec node "$r"/.bee/bin/hooks/${script} --source=repo`,
  ].join("\n");
}

function commandFor(script, target) {
  if (target === TARGETS.REPO) return repoCommand(script);
  return `node "\${CLAUDE_PLUGIN_ROOT}/packages/bee/hooks/${script}"`;
}

// Windows override for the repo target's Codex projection (Codex hook schema
// optional `commandWindows` field, run with the session cwd as working dir —
// no `$SHELL -lc`, so the POSIX `command` string above is Windows-broken). A
// bare cwd-relative `node .bee/bin/hooks/<f> --source=repo` (the pre-cell
// form) breaks the instant Codex's session cwd is a NESTED subdirectory of
// the repo (MODULE_NOT_FOUND) — cwd-relative resolution assumes the session
// cwd IS the repo root, which is not guaranteed. windowsBootstrap below
// resolves the real git root first, exactly like the POSIX command's own
// `git rev-parse --show-toplevel` step, so both transports share the same
// cwd-independence guarantee.
//
// The bootstrap is `node -e SCRIPT <script>`: SCRIPT is plain JS using ONLY
// single quotes internally and NO `$`, `%`, or backtick character anywhere —
// cmd.exe and powershell.exe both parse the outer double-quoted `-e`
// argument identically as long as none of those three characters appear
// inside it (cmd treats `%` as a variable sigil, PowerShell treats `$` and
// backtick specially even inside double quotes). `node -e` is cross-platform,
// so this same string is also directly executable on POSIX — the contract
// test runs it for real on Linux CI instead of only pattern-matching it.
//
// SCRIPT itself: resolve the git root via `execSync('git rev-parse
// --show-toplevel')`; on failure (no git root — outside any repo) exit 0
// SILENTLY, matching the POSIX command's fail-open semantics (no git root =
// transport unavailable) but with no diagnostic on this arm, since Windows
// carries no stderr-diagnostic contract today. On success, spawnSync the
// real hook (process.execPath, so it runs under the SAME node the bootstrap
// itself is running under) with stdio 'inherit' — this MUST be inherited,
// never piped, because hooks read their JSON payload from stdin — then
// process.exit with the child's status (1 if the spawn itself errored).
function windowsBootstrap(script) {
  const body = [
    "var cp=require('child_process');",
    "var path=require('path');",
    "var hook=process.argv[1];",
    "var root='';",
    "try{root=cp.execSync('git rev-parse --show-toplevel',{stdio:['ignore','pipe','ignore']}).toString().trim();}catch(e){root='';}",
    "if(!root){process.exit(0);}",
    "var target=path.join(root,'.bee','bin','hooks',hook);",
    "var r=cp.spawnSync(process.execPath,[target,'--source=repo'],{stdio:'inherit'});",
    "if(r.error){process.exit(1);}",
    "process.exit(r.status===null?1:r.status);",
  ].join("");
  return 'node -e "' + body + '" ' + script;
}

function commandWindowsFor(script, target) {
  if (target === TARGETS.REPO) return windowsBootstrap(script);
  return undefined;
}

// Catalog entries carry the logical (script, statusMessage) pair; the concrete
// command string is a function of the TARGET and is produced only at render
// time. Nothing in the catalog is hand-authored per runtime or per target.
function cmd(script, statusMessage) {
  return { script, statusMessage };
}

// One entry per lifecycle event bee wires today. `groups` is the ordered
// list of hook-group objects that event carries in the checked-in hooks.json
// shape (each optionally matcher-scoped); each group's `runtimes` says which
// projection(s) render it.
const CATALOG = Object.freeze([
  {
    event: "SessionStart",
    groups: [
      {
        runtimes: BOTH,
        matcher: "startup|resume|clear|compact",
        hooks: [cmd("bee-session-init.mjs", "bee: session bootstrap")],
      },
    ],
  },
  {
    event: "UserPromptSubmit",
    groups: [
      {
        runtimes: BOTH,
        hooks: [cmd("bee-prompt-context.mjs", "bee: phase reminder")],
      },
    ],
  },
  {
    event: "PreToolUse",
    groups: [
      {
        runtimes: BOTH,
        matcher: "Edit|Write|MultiEdit|Bash|Read|Glob|Grep|AskUserQuestion",
        hooks: [cmd("bee-write-guard.mjs", "bee: write guard")],
      },
      {
        runtimes: CLAUDE_ONLY,
        matcher: "Agent|Task",
        hooks: [cmd("bee-model-guard.mjs", "bee: model-tier guard")],
      },
      {
        // Codex-native spawn guard (codex-native-runtime-v2 D4, decision 0023
        // parity). Codex exposes agent spawns through PreToolUse as tool_name
        // "spawn_agent" (spike codex-cli 0.144.4, capability-matrix row D1),
        // never as "Agent"/"Task", so this is a SEPARATE Codex-only group, not
        // a superset of the Claude matcher above. The SAME bee-model-guard.mjs
        // handles it through an isolated Codex branch keyed on the observed
        // envelope (tool_input.agent_type "worker" + tool_input.message, marker
        // anchored at the START of message). This gates the built-in worker
        // spawn only — it is NOT D8 custom agents (deferred).
        runtimes: CODEX_ONLY,
        matcher: "spawn_agent",
        hooks: [cmd("bee-model-guard.mjs", "bee: model-tier guard")],
      },
    ],
  },
  {
    event: "PostToolUse",
    groups: [
      {
        // Superset, never a swap (D4, codex-native-runtime-v2): update_plan
        // is Codex's native plan-tool call; TaskCreate/TaskUpdate/TodoWrite
        // stay wired for Claude (and any Codex build that still emits them).
        // bee-state-sync.mjs itself filters on NEITHER tool_name NOR
        // tool_input - the matcher below is the only filter - so no wrapper
        // change was needed to add update_plan.
        runtimes: BOTH,
        matcher: "update_plan|TaskCreate|TaskUpdate|TodoWrite",
        hooks: [cmd("bee-state-sync.mjs", "bee: state sync")],
      },
      {
        // No matcher = every tool (renders like the matcher-less
        // UserPromptSubmit entry above). Passive measurement only — see
        // hooks/bee-tools-logger.mjs; this hook can never deny or block.
        runtimes: BOTH,
        hooks: [cmd("bee-tools-logger.mjs", "bee: tools logger")],
      },
    ],
  },
  {
    event: "SubagentStart",
    groups: [
      {
        runtimes: CODEX_ONLY,
        hooks: [cmd("bee-codex-subagent-audit.mjs", "bee: subagent start audit")],
      },
    ],
  },
  {
    event: "SubagentStop",
    groups: [
      {
        runtimes: BOTH,
        hooks: [
          cmd("bee-state-sync.mjs", "bee: state sync"),
          cmd("bee-chain-nudge.mjs", "bee: chain nudge"),
        ],
      },
      {
        runtimes: CODEX_ONLY,
        hooks: [cmd("bee-codex-subagent-audit.mjs", "bee: subagent stop audit")],
      },
    ],
  },
  {
    event: "PreCompact",
    groups: [
      {
        runtimes: BOTH,
        hooks: [cmd("bee-session-close.mjs", "bee: pre-compact flush check")],
      },
    ],
  },
  {
    event: "Stop",
    groups: [
      {
        runtimes: BOTH,
        hooks: [
          cmd("bee-state-sync.mjs", "bee: state sync"),
          cmd("bee-session-close.mjs", "bee: session close check"),
        ],
      },
    ],
  },
]);

function assertRuntime(runtime) {
  if (runtime !== RUNTIMES.CLAUDE && runtime !== RUNTIMES.CODEX) {
    throw new Error(
      `catalog.mjs renderProjection: unknown runtime "${runtime}" (expected "claude" or "codex")`,
    );
  }
}

function assertTarget(target) {
  if (target !== TARGETS.PLUGIN && target !== TARGETS.REPO) {
    throw new Error(
      `catalog.mjs renderProjection: unknown target "${target}" (expected "plugin" or "repo")`,
    );
  }
}

// Render one projection ("claude" | "codex") for one target ("plugin" |
// "repo") as the plain hooks.json object — no runtime/target metadata leaks
// into the output, only the matcher/hooks shape Claude Code and Codex both
// already load. `plugin` is the DEFAULT target, so every pre-existing caller
// (and both checked-in plugin projections) render exactly as before.
export function renderProjection(runtime, { target = TARGETS.PLUGIN } = {}) {
  assertRuntime(runtime);
  assertTarget(target);
  const hooks = {};
  for (const { event, groups } of CATALOG) {
    const rendered = groups
      .filter((g) => g.runtimes.includes(runtime))
      .map((g) => {
        const out = {};
        if (g.matcher !== undefined) out.matcher = g.matcher;
        out.hooks = g.hooks.map((h) => {
          const entry = {
            type: "command",
            command: commandFor(h.script, target),
          };
          // commandWindows is CODEX-REPO-ONLY: the Codex plugin manifest
          // omits codex hooks (only .codex/hooks.json, the repo target,
          // loads), and Claude has no such field in its hook schema.
          if (runtime === RUNTIMES.CODEX && target === TARGETS.REPO) {
            entry.commandWindows = commandWindowsFor(h.script, target);
          }
          entry.statusMessage = h.statusMessage;
          return entry;
        });
        return out;
      });
    if (rendered.length > 0) hooks[event] = rendered;
  }
  return { hooks };
}

// Deterministic, byte-stable JSON text for a projection: 2-space indent plus
// exactly one trailing newline — matches the checked-in file formatting
// exactly, so the drift-check test can compare rendered text to disk
// byte-for-byte.
export function renderProjectionText(runtime, { target = TARGETS.PLUGIN } = {}) {
  return `${JSON.stringify(renderProjection(runtime, { target }), null, 2)}\n`;
}

// The full set of event names the catalog defines, in declaration order.
export const EVENTS = Object.freeze(CATALOG.map((entry) => entry.event));

// The approved differences between the two rendered projections. Anything
// not covered here is drift and must fail the drift-check test in
// hooks/test_hook_contracts.mjs.
export const ALLOWED_DIFFERENCES = Object.freeze([
  {
    id: "model-tier-guard-claude-only",
    runtime: RUNTIMES.CLAUDE,
    event: "PreToolUse",
    matcher: "Agent|Task",
    script: "bee-model-guard.mjs",
    description:
      'bee-model-guard.mjs on the PreToolUse matcher "Agent|Task" is Claude-only: ' +
      '"Agent"/"Task" are the tool names Claude uses for subagent dispatch. This ' +
      "is a MATCHER difference, not a capability gap — Codex exposes the same guard " +
      'on its own spawn tool name (see "model-tier-guard-codex-spawn" below; ' +
      "codex-native-runtime-v2 D4, capability-matrix row D1).",
  },
  {
    id: "model-tier-guard-codex-spawn",
    runtime: RUNTIMES.CODEX,
    event: "PreToolUse",
    matcher: "spawn_agent",
    script: "bee-model-guard.mjs",
    description:
      'bee-model-guard.mjs on the PreToolUse matcher "spawn_agent" is Codex-only: ' +
      'Codex exposes collaboration spawn as tool_name "spawn_agent" (never ' +
      '"Agent"/"Task"), observed reaching PreToolUse on codex-cli 0.144.4. The ' +
      "guard runs an isolated Codex branch keyed on the observed envelope " +
      "(agent_type \"worker\" + message; marker anchored at the start of message) " +
      "and fails open on every unobserved shape (codex-native-runtime-v2 D4, " +
      "decision 0023 parity; NOT D8 custom agents).",
  },
  {
    id: "subagent-start-audit-codex-only",
    runtime: RUNTIMES.CODEX,
    event: "SubagentStart",
    matcher: null,
    script: "bee-codex-subagent-audit.mjs",
    description:
      "Codex exposes SubagentStart only after the native subagent has started; " +
      "bee records bounded bootstrap/audit evidence and claims no pre-spawn authority " +
      "(CONTEXT.md decisions D1/D2).",
  },
  {
    id: "subagent-stop-audit-codex-only",
    runtime: RUNTIMES.CODEX,
    event: "SubagentStop",
    matcher: null,
    script: "bee-codex-subagent-audit.mjs",
    description:
      "Codex pairs SubagentStop with the same bounded audit handler used at start; " +
      "Claude keeps its existing state-sync and chain-nudge behavior unchanged " +
      "(CONTEXT.md decisions D1/D2).",
  },
]);
