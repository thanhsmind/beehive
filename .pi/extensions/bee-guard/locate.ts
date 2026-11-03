import { execFileSync } from "node:child_process"
import { existsSync, statSync } from "node:fs"
import path from "node:path"

export const BINARY_NAMES = ["bee", "bee.exe"]

// ─── store + binary discovery (re-run on EVERY call, never cached) ──────────

/** Resolves the main checkout root for a directory. For a linked worktree,
 * this returns the main worktree root via `git rev-parse --git-common-dir`.
 * For a direct checkout, returns the directory itself. */
export function mainCheckoutRoot(directory: string): string {
  try {
    const commonDir = execFileSync(
      "git",
      ["-C", directory, "rev-parse", "--path-format=absolute", "--git-common-dir"],
      { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] },
    ).trim()
    if (commonDir) {
      const absCommonDir = path.isAbsolute(commonDir)
        ? commonDir
        : path.resolve(directory, commonDir)
      return path.dirname(absCommonDir)
    }
  } catch {
    // not a git repo, or git unavailable — the direct-project root above is all there is.
  }
  return directory
}

/** Every root this project's Pi session might find a bee store under, in
 * priority order: the project directory first, then (for a linked worktree
 * with no vendored store of its own) the main worktree root via
 * `git rev-parse --git-common-dir`. Mirrors the shell fallback chain in
 * packages/bee/hooks/claude-hooks.json. */
export function candidateRoots(directory: string): string[] {
  const main = mainCheckoutRoot(directory)
  return main === directory ? [directory] : [directory, main]
}

export function isDirectory(candidate: string): boolean {
  try {
    return statSync(candidate).isDirectory()
  } catch {
    return false
  }
}

/** The passivity check, per call. True when a `.bee` DIRECTORY exists at the
 * project root or at the main worktree root — the cheapest honest "is this a
 * bee repo" signal, and the one that flips the moment an in-session
 * `bee onboard` creates the store (no `/reload` needed). The binary is a
 * SEPARATE question, deliberately: directory present + binary missing is an
 * undecidable bee repo, not a bee-less one. */
export function beeStorePresent(directory: string): boolean {
  return candidateRoots(directory).some((root) => isDirectory(path.join(root, ".bee")))
}

/** The first bee binary that exists across the same roots, or null. Mirrors
 * candidateRoots' priority order exactly. */
export function resolveBeeBinary(directory: string): string | null {
  for (const root of candidateRoots(directory)) {
    for (const name of BINARY_NAMES) {
      const candidate = path.join(root, ".bee", "bin", name)
      if (existsSync(candidate)) return candidate
    }
  }
  return null
}


