#!/usr/bin/env node
// Guard: the OKF coverage gate's ground truth must be DERIVED and honest.
//
// f2-1b (F8, supersedes F1). The gate shipped by okf-5/okf-6 compared a
// hand-authored ANCHOR_REGISTRY against hand-authored claims — the exact
// defect this repo promoted as a critical pattern ("a coverage gate derives
// its ground truth; it never compares two hand-authored lists"). The advisor
// consult then MEASURED a second, worse defect: the extractor is
// format-blind. docs/specs/onboarding.md's 22 `- **R1** —` rules inventory as
// R0 and its behaviors as B0, so "lost" anchors read as anchors that never
// existed. Reproducing 26 and 47 proved nothing: advisor-protocol is the very
// file those regexes were written against.
//
// This suite asserts the property that closes both holes:
//
//   NO EXTRACTION RESULT MAY READ AS A PASS UNLESS IT WAS POSITIVELY VERIFIED.
//
// Concretely — a pin is {commit, path, blob_sha, scheme, expected_counts} and
// ALL of it is asserted at check time; an empty extraction, a sha mismatch, a
// count mismatch, an undeclared scheme, and an unresolvable source are each a
// typed exit-1 failure, never a quiet 0/0 green. Plus the visibility that
// makes format-blindness detectable at all: unparsed-block reporting, proven
// here against onboarding.md, which MUST report a non-zero count. A clean
// parse there would mean the extractor is still blind and this suite is
// lying.

import { spawnSync } from "node:child_process";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..", "..");
const MIGRATE = path.join(REPO_ROOT, "scripts", "okf_migrate.mjs");
const TMP = path.join(REPO_ROOT, ".bee", "tmp", "test_okf_pins");

let failures = 0;
let checks = 0;

function ok(cond, label, detail) {
  checks += 1;
  if (cond) return true;
  failures += 1;
  console.error(`FAIL test_okf_pins: ${label}`);
  if (detail !== undefined) console.error(`      ${typeof detail === "string" ? detail : JSON.stringify(detail)}`);
  return false;
}

function eq(actual, expected, label) {
  return ok(
    JSON.stringify(actual) === JSON.stringify(expected),
    label,
    `expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`,
  );
}

/** git's own blob id for a byte string — no git process needed. */
function blobSha(text) {
  const body = Buffer.from(text, "utf8");
  return crypto
    .createHash("sha1")
    .update(Buffer.concat([Buffer.from(`blob ${body.length}\0`, "utf8"), body]))
    .digest("hex");
}

function runCli(args) {
  const r = spawnSync(process.execPath, [MIGRATE, ...args], { cwd: REPO_ROOT, encoding: "utf8" });
  return { code: r.status, out: r.stdout || "", err: r.stderr || "" };
}

function codes(result) {
  return (result?.issues || []).map((i) => i.code).sort();
}

// ─── load the module under test ─────────────────────────────────────────────

let M;
try {
  M = await import(pathToFileURL(MIGRATE).href);
} catch (error) {
  console.error(`FAIL test_okf_pins: could not import scripts/okf_migrate.mjs: ${error.message}`);
  process.exit(1);
}

for (const name of ["PIN_REGISTRY", "derivePin", "derivePinForArea", "extractByScheme", "resolvePinnedSource", "inventorySpec", "inventoryPatterns"]) {
  ok(typeof M[name] !== "undefined", `scripts/okf_migrate.mjs must export ${name}`);
}
if (failures > 0) {
  console.error("FAIL test_okf_pins: the derived-pin API is missing — nothing further can be asserted");
  process.exit(1);
}

fs.rmSync(TMP, { recursive: true, force: true });
fs.mkdirSync(TMP, { recursive: true });

// ─── 1. the hand-authored anchor lists are gone ─────────────────────────────

{
  const src = fs.readFileSync(MIGRATE, "utf8");
  const codeOnly = src
    .split("\n")
    .filter((l) => !/^\s*(\/\/|\*|\/\*)/.test(l))
    .join("\n");
  ok(
    !/\bANCHOR_REGISTRY\b/.test(codeOnly),
    "ANCHOR_REGISTRY's hand-authored anchor list is gone from okf_migrate.mjs (F8: only the pin map remains)",
  );
  ok(
    !/\bPATTERNS_ANCHOR_REGISTRY\b/.test(codeOnly),
    "PATTERNS_ANCHOR_REGISTRY's hand-authored anchor list is gone from okf_migrate.mjs",
  );
  ok(typeof M.PIN_REGISTRY === "object" && M.PIN_REGISTRY !== null, "PIN_REGISTRY is the replacement");
}

// ─── 2. every pin declares all five parts ───────────────────────────────────

for (const [area, pin] of Object.entries(M.PIN_REGISTRY)) {
  if (pin.scheme === null) {
    ok(
      typeof pin.refusal === "string" && pin.refusal.length > 0,
      `pin "${area}" declares no scheme, so it must carry a refusal reason (F9/S5 owns choosing one)`,
    );
    continue;
  }
  for (const field of ["commit", "path", "blob_sha", "scheme", "expected_counts", "source_copy"]) {
    ok(pin[field] !== undefined && pin[field] !== null, `pin "${area}" declares ${field}`);
  }
  ok(/^[0-9a-f]{40}$/.test(pin.blob_sha || ""), `pin "${area}" blob_sha is a full 40-hex blob id`, pin.blob_sha);
  ok(
    typeof pin.expected_counts?.total === "number",
    `pin "${area}" asserts a total (a pin that expects nothing proves nothing)`,
  );
}

// ─── 3. the two shipped areas still derive their shipped counts ─────────────

const advisor = await M.derivePinForArea("advisor-protocol");
ok(advisor.ok, "derived advisor-protocol pin is green", advisor.issues);
eq(
  {
    behaviors: advisor.counts?.behaviors,
    rules: advisor.counts?.rules,
    edges: advisor.counts?.edges,
    pointers: advisor.counts?.pointers,
    total: advisor.counts?.total,
  },
  { behaviors: 4, rules: 9, edges: 6, pointers: 7, total: 26 },
  "advisor-protocol derives 26 anchors (4 B / 9 R / 6 E / 7 P) from the pinned blob",
);
ok(advisor.anchors?.all?.length === 26, "advisor-protocol's derived anchor list has 26 members", advisor.anchors?.all?.length);
ok(advisor.counts?.unparsed_blocks === 0, "advisor-protocol's pinned blob parses with zero unparsed blocks", advisor.counts?.unparsed_blocks);

const patterns = await M.derivePinForArea("critical-patterns");
ok(patterns.ok, "derived critical-patterns pin is green", patterns.issues);
ok(patterns.counts?.total === 47, "critical-patterns derives 47 anchors from the pinned blob", patterns.counts?.total);
ok(patterns.anchors?.all?.length === 47, "critical-patterns' derived anchor list has 47 members", patterns.anchors?.all?.length);

// ─── 4. the committed verbatim source copies exist and hash to blob_sha ─────

for (const [area, pin] of Object.entries(M.PIN_REGISTRY)) {
  if (pin.scheme === null) continue;
  const abs = path.join(REPO_ROOT, ...pin.source_copy.split("/"));
  if (!ok(fs.existsSync(abs), `pin "${area}"'s committed source copy exists at ${pin.source_copy} (shallow-clone fallback)`)) continue;
  ok(
    blobSha(fs.readFileSync(abs, "utf8")) === pin.blob_sha,
    `pin "${area}"'s committed source copy hashes to blob_sha`,
    `${blobSha(fs.readFileSync(abs, "utf8"))} != ${pin.blob_sha}`,
  );
}

// ─── 5. a WRONG SHA exits 1 ─────────────────────────────────────────────────

{
  const bad = { ...M.PIN_REGISTRY["advisor-protocol"], blob_sha: "0".repeat(40) };
  const r = await M.derivePin(bad, "advisor-protocol");
  ok(r.ok === false, "a pin whose blob_sha does not match the pinned commit:path is REFUSED");
  ok(codes(r).includes("PIN_SHA_MISMATCH"), "wrong sha reports the typed code PIN_SHA_MISMATCH", codes(r));

  const cli = runCli(["--derive", JSON.stringify(bad)]);
  ok(cli.code === 1, "CLI --derive exits 1 on a wrong sha", `exit ${cli.code}`);
  ok(/PIN_SHA_MISMATCH/.test(cli.err + cli.out), "CLI names PIN_SHA_MISMATCH", cli.err + cli.out);
}

// ─── 6. right sha + WRONG expected_counts exits 1 ───────────────────────────

{
  const bad = {
    ...M.PIN_REGISTRY["advisor-protocol"],
    expected_counts: { ...M.PIN_REGISTRY["advisor-protocol"].expected_counts, total: 25, rules: 8 },
  };
  const r = await M.derivePin(bad, "advisor-protocol");
  ok(r.ok === false, "a pin with the right sha but the wrong expected_counts is REFUSED (a mismatch, not only an empty set)");
  ok(codes(r).includes("PIN_COUNT_MISMATCH"), "count mismatch reports the typed code PIN_COUNT_MISMATCH", codes(r));

  const cli = runCli(["--derive", JSON.stringify(bad)]);
  ok(cli.code === 1, "CLI --derive exits 1 on a count mismatch", `exit ${cli.code}`);
  ok(/PIN_COUNT_MISMATCH/.test(cli.err + cli.out), "CLI names PIN_COUNT_MISMATCH", cli.err + cli.out);
}

// ─── 7. an EMPTY extraction exits 1 — never 0/0 green ───────────────────────

{
  const empty = "# A spec with no anchors at all\n\nJust prose.\n";
  const rel = ".bee/tmp/test_okf_pins/empty-source.md";
  fs.writeFileSync(path.join(REPO_ROOT, rel), empty);
  const pin = {
    commit: "0".repeat(40),
    path: "docs/specs/does-not-exist.md",
    blob_sha: blobSha(empty),
    scheme: "ba-nine-section",
    // unparsed_blocks is mandatory since f2-2 — declared here so this fixture
    // still proves what it was written to prove (an EMPTY extraction is
    // refused), rather than tripping the earlier PIN_INCOMPLETE guard.
    expected_counts: { total: 0, unparsed_blocks: 0 },
    source_copy: rel,
  };
  const r = await M.derivePin(pin, "empty-fixture");
  ok(r.ok === false, "an EMPTY extraction is REFUSED even when expected_counts says total 0 — 0/0 is never a pass");
  ok(codes(r).includes("PIN_EMPTY_EXTRACTION"), "empty extraction reports the typed code PIN_EMPTY_EXTRACTION", codes(r));

  const cli = runCli(["--derive", JSON.stringify(pin)]);
  ok(cli.code === 1, "CLI --derive exits 1 on an empty extraction", `exit ${cli.code}`);
  ok(/PIN_EMPTY_EXTRACTION/.test(cli.err + cli.out), "CLI names PIN_EMPTY_EXTRACTION", cli.err + cli.out);
}

// ─── 8. shallow-clone fallback: unresolvable blob + committed copy PASSES ───

{
  const real = M.PIN_REGISTRY["advisor-protocol"];
  const pin = { ...real, commit: "0".repeat(40) }; // `git show <sha>:<path>` cannot resolve this
  const r = await M.derivePin(pin, "advisor-protocol");
  ok(r.ok, "an unresolvable commit falls back to the committed source copy and still verifies", r.issues);
  ok(r.via === "committed-copy", "the fallback reports via=committed-copy (not a silent skip)", r.via);
  ok(r.counts?.total === 26, "the fallback derives the same 26 anchors", r.counts?.total);

  const cli = runCli(["--derive", JSON.stringify(pin)]);
  ok(cli.code === 0, "CLI --derive exits 0 via the committed-copy fallback", `exit ${cli.code} ${cli.err}`);
  ok(/committed-copy/.test(cli.out + cli.err), "CLI names the committed-copy fallback", cli.out);
}

// ─── 9. missing BOTH is exit 1, never a skip ────────────────────────────────

{
  const real = M.PIN_REGISTRY["advisor-protocol"];
  const pin = { ...real, commit: "0".repeat(40), source_copy: ".bee/tmp/test_okf_pins/absent.md" };
  const r = await M.derivePin(pin, "advisor-protocol");
  ok(r.ok === false, "an unresolvable commit with NO committed copy is REFUSED (never skipped)");
  ok(codes(r).includes("PIN_UNRESOLVED"), "missing both reports the typed code PIN_UNRESOLVED", codes(r));

  const cli = runCli(["--derive", JSON.stringify(pin)]);
  ok(cli.code === 1, "CLI --derive exits 1 when neither the blob nor the copy resolves", `exit ${cli.code}`);
}

// ─── 10. a drifted committed copy is refused ────────────────────────────────

{
  const drifted = "# not the pinned source\n";
  const rel = ".bee/tmp/test_okf_pins/drifted.md";
  fs.writeFileSync(path.join(REPO_ROOT, rel), drifted);
  const real = M.PIN_REGISTRY["advisor-protocol"];
  const pin = { ...real, commit: "0".repeat(40), source_copy: rel };
  const r = await M.derivePin(pin, "advisor-protocol");
  ok(r.ok === false, "a committed source copy that does not hash to blob_sha is REFUSED");
  ok(codes(r).includes("PIN_SHA_MISMATCH"), "a drifted copy reports PIN_SHA_MISMATCH", codes(r));
}

// ─── 11. an area with NO declared scheme is refused, never passed 0/0 ───────
//
// f2-5: decision-memory is no longer in this set — its migration cell
// authored its pin (9 rules, 0 unparsed).
// f2-11: neither is worktree-parallelism — the LAST unscheme'd area, now
// pinned under the `narrative-sections` scheme (section 27 below). Every
// registered area therefore declares a scheme today, so the PIN_NO_SCHEME
// refusal is asserted against an ad-hoc pin instead of a registry entry: the
// property under test is "an undecided shape is refused BY NAME rather than
// passed 0/0", and it must keep holding for the next area that arrives without
// one — not only while some area happened to still be undecided.

{
  const r2 = await M.derivePin(
    { ...M.PIN_REGISTRY["advisor-protocol"], scheme: null, refusal: "shape not decided yet" },
    "some-unscheme'd-area",
  );
  ok(r2.ok === false, "an area with no declared anchor scheme is REFUSED");
  ok(codes(r2).includes("PIN_NO_SCHEME"), "an unscheme'd area reports PIN_NO_SCHEME", codes(r2));
  ok(
    /shape not decided yet/.test((r2.issues || []).map((i) => i.message).join(" ")),
    "the PIN_NO_SCHEME refusal carries the pin's own reason, so the refusal names WHY",
    r2.issues,
  );
  ok(
    Object.values(M.PIN_REGISTRY).every((p) => typeof p.scheme === "string" && p.scheme.length > 0),
    "every REGISTERED area now declares a scheme (f2-11 closed the last unscheme'd one)",
    Object.entries(M.PIN_REGISTRY).filter(([, p]) => !p.scheme).map(([a]) => a),
  );

  const r3 = await M.derivePinForArea("no-such-area-at-all");
  ok(r3.ok === false, "an area with no pin at all is REFUSED");
  ok(codes(r3).includes("PIN_UNKNOWN_AREA"), "an unpinned area reports PIN_UNKNOWN_AREA", codes(r3));

  const r4 = await M.derivePin({ ...M.PIN_REGISTRY["advisor-protocol"], scheme: "invented-scheme" }, "x");
  ok(r4.ok === false, "an unknown scheme name is REFUSED");
  ok(codes(r4).includes("PIN_UNKNOWN_SCHEME"), "an unknown scheme reports PIN_UNKNOWN_SCHEME", codes(r4));
}

// f2-5: decision-memory's own coverage gate now exits 0 — asserted alongside
// the other two shipped areas in section "f2-3/f2-5 pin assertions" below.
{
  const cli = runCli(["--check", "decision-memory"]);
  ok(cli.code === 0, "`--check decision-memory` exits 0 now that its pin (9 rules, 0 unparsed) is authored", `exit ${cli.code}: ${cli.err}${cli.out}`);
  ok(/^PASS okf_migrate --check decision-memory: /m.test(cli.out), "the chain PASS wording names decision-memory", cli.out);
}

// f2-6: verify-pipeline's own coverage gate now exits 0 — 14 anchors (0 B /
// 5 R / 4 E / 5 P), 7 unparsed blocks (the unnumbered Behaviors & Operations
// bullets, never invented into ids — D10), split across two topic concepts.
{
  const cli = runCli(["--check", "verify-pipeline"]);
  ok(cli.code === 0, "`--check verify-pipeline` exits 0 now that its pin (14 anchors, 7 unparsed) is authored", `exit ${cli.code}: ${cli.err}${cli.out}`);
  ok(/^PASS okf_migrate --check verify-pipeline: /m.test(cli.out), "the chain PASS wording names verify-pipeline", cli.out);
}

// ─── 12. unparsed-block reporting makes format-blindness VISIBLE ────────────

// f2-9 moved WHICH FILE holds those bytes, and nothing else: `docs/specs/
// onboarding.md` is now a D37 pointer stub, so the pre-migration source — the
// exact bytes this section has always measured, hash-verified against the pin's
// blob_sha — is read from the committed copy instead. The property under test
// (a real spec MUST report a non-zero unparsed-block count, or the extractor is
// still blind) is unchanged; weakening it to fit a stub would be the assertion
// following the convenience instead of the truth.
const ONBOARDING_SOURCE = "docs/history/okf-migration-f2/sources/onboarding.md";

{
  const onboarding = fs.readFileSync(path.join(REPO_ROOT, ...ONBOARDING_SOURCE.split("/")), "utf8");
  const inv = M.inventorySpec(onboarding);
  ok(
    typeof inv.unparsed?.blocks?.total === "number",
    "inventorySpec reports an unparsed-block count",
    inv.unparsed,
  );
  ok(
    inv.unparsed.blocks.total > 0,
    `${ONBOARDING_SOURCE} (the pinned pre-migration onboarding spec) reports a NON-ZERO unparsed-block count — a clean parse here would prove the extractor is still blind`,
    inv.unparsed?.blocks,
  );
  // f2-4 moved which part of onboarding is invisible, and the assertion moves
  // with the truth — never the other way round. Its 28 `- **R1** —` rules are
  // now SEEN (that is the widening), while its 20 unnumbered bold-lead
  // behaviour paragraphs (`**Detect (every run).**`) still carry no id and so
  // remain unparsed BY DESIGN: the classifier reads ids, it never invents them.
  ok(
    inv.rules.length >= 27 && inv.rules.includes("R20b"),
    "onboarding's bold-wrapped rule ids are now classified, letter suffix included (f2-4) — this is what the narrow classifier could not see",
    inv.rules?.length,
  );
  ok(
    inv.unparsed.blocks.behaviors > 0,
    "onboarding's unnumbered bold-lead behaviors are STILL reported as unparsed — the widening never fabricated ids for them",
    inv.unparsed?.blocks,
  );
  ok(
    typeof inv.unparsed.lines?.total === "number" && inv.unparsed.lines.total > 0,
    "inventorySpec also reports unclassified non-blank body lines per section",
    inv.unparsed?.lines,
  );

  const cli = runCli(["--inventory", ONBOARDING_SOURCE]);
  ok(cli.code === 0, `\`--inventory ${ONBOARDING_SOURCE}\` runs`, cli.err);
  let parsed = null;
  try {
    parsed = JSON.parse(cli.out);
  } catch { /* reported below */ }
  ok(parsed?.unparsed?.blocks?.total > 0, "the --inventory JSON surfaces the non-zero unparsed-block count", parsed?.unparsed);
}

// ─── 13. the shipped chain gates still PASS, from the derived path ──────────

{
  const cli = runCli(["--check", "advisor-protocol"]);
  ok(cli.code === 0, "`--check advisor-protocol` still exits 0", cli.err);
  ok(
    /^PASS okf_migrate --check advisor-protocol: /m.test(cli.out),
    "`--check advisor-protocol` keeps the PASS wording the chain entry depends on",
    cli.out,
  );
  const cli2 = runCli(["--check-patterns"]);
  ok(cli2.code === 0, "`--check-patterns` still exits 0", cli2.err);
  ok(
    /^PASS okf_migrate --check-patterns: /m.test(cli2.out),
    "`--check-patterns` keeps the PASS wording the chain entry depends on",
    cli2.out,
  );
}

// ─── 14. `--verify-pins` asserts every pinned area in one shot ──────────────

{
  const cli = runCli(["--verify-pins"]);
  ok(cli.code === 0, "`--verify-pins` exits 0 with the registry as shipped", cli.err);
  ok(/advisor-protocol/.test(cli.out) && /critical-patterns/.test(cli.out), "`--verify-pins` reports each pinned area", cli.out);
}

// ─── 15. the REPAIRED-pin branch is asserted, never a hole (f2-10) ──────────
//
// hook-runtime shipped the rule id `R14` TWICE — two genuinely different
// rules — so it could not be pinned as it stood: anchors are keyed by id, so
// the first R14's text was silently overwritten by the second's and became
// unmeasurable by the F11 floor forever, while set-equality could not see the
// pair's second member at all. Neither rule may be dropped or merged, so the
// SOURCE was repaired (second occurrence renumbered `R14a`) before the pin was
// captured. Those bytes then exist in no commit's tree, which is the one
// legitimate reason a pin's blob_sha may differ from what commit:path
// resolves to.
//
// The whole risk of that branch is that it becomes a way to launder drift.
// So it is asserted from both sides here: the declared repair must name the
// EXACT provenance blob and carry a reason, and an undeclared or misdeclared
// disagreement is still PIN_SHA_MISMATCH exactly as it was before this
// existed. The property the repair exists to create — every derived id
// distinct — is asserted directly rather than inferred from the counts, since
// counts are precisely what a duplicate id slips past.

{
  const real = M.PIN_REGISTRY["hook-runtime"];
  ok(
    typeof real.repaired_from === "string" && /^[0-9a-f]{40}$/.test(real.repaired_from),
    "the hook-runtime pin names the provenance blob it was repaired from",
    real.repaired_from,
  );
  ok(
    typeof real.repair_reason === "string" && real.repair_reason.includes("R14a"),
    "the hook-runtime pin states the repair in prose, naming the id it introduced",
    real.repair_reason?.slice(0, 80),
  );

  const derived = await M.derivePinForArea("hook-runtime");
  ok(derived.ok, "the repaired hook-runtime pin derives green", derived.issues);
  ok(
    derived.via === "committed-copy",
    "a repaired pin resolves via its committed copy — the resolvable blob is the PROVENANCE, never the pinned source",
    derived.via,
  );

  const all = derived.anchors?.all || [];
  ok(all.length === 81, "hook-runtime derives 81 anchors from the repaired blob", all.length);
  ok(
    new Set(all).size === all.length,
    "every derived hook-runtime anchor id is DISTINCT — the property the repair exists to create, and the one a count can never prove",
    `${all.length} anchors, ${new Set(all).size} distinct`,
  );
  ok(
    all.includes("R14") && all.includes("R14a"),
    "both former-R14 rules survive as separate anchors — neither was dropped nor merged to remove the duplicate",
    all.filter((a) => a.startsWith("R14")),
  );
  for (const id of ["R14", "R14a"]) {
    const text = derived.anchors?.texts?.get(id);
    ok(
      typeof text === "string" && text.length > 0,
      `${id} has its own extracted text, so the F11 floor can measure it individually`,
      typeof text,
    );
  }
  ok(
    derived.anchors.texts.get("R14").includes("gate-bypass") &&
      derived.anchors.texts.get("R14a").includes("command-shape"),
    "the two texts are the two DIFFERENT rules, not one rule read twice",
  );

  // An undeclared disagreement is still refused — the pre-f2-10 behaviour,
  // unchanged. (Section 5 asserts this for an unrepaired pin; here it is
  // asserted for the very pin that DOES carry a repair, with the declaration
  // removed, so the branch cannot be reached by accident.)
  const undeclared = { ...real };
  delete undeclared.repaired_from;
  delete undeclared.repair_reason;
  const rU = await M.derivePin(undeclared, "hook-runtime");
  ok(rU.ok === false, "the same pin WITHOUT a declared repair is REFUSED — a disagreement is never inferred to be one");
  ok(codes(rU).includes("PIN_SHA_MISMATCH"), "an undeclared disagreement reports PIN_SHA_MISMATCH", codes(rU));

  const wrongProvenance = { ...real, repaired_from: "0".repeat(40) };
  const rW = await M.derivePin(wrongProvenance, "hook-runtime");
  ok(rW.ok === false, "a repair naming the WRONG provenance blob is REFUSED — drifting provenance is as loud as a drifting pin");
  ok(codes(rW).includes("PIN_SHA_MISMATCH"), "a wrong repaired_from reports PIN_SHA_MISMATCH", codes(rW));

  const noReason = { ...real, repair_reason: "   " };
  const rR = await M.derivePin(noReason, "hook-runtime");
  ok(rR.ok === false, "a repair with no stated reason is REFUSED — the repair must be readable, not merely flagged");
  ok(codes(rR).includes("PIN_SHA_MISMATCH"), "an unexplained repair reports PIN_SHA_MISMATCH", codes(rR));

  // And the no-op: declaring a repair on a pin whose blob_sha already agrees
  // with history changes nothing at all.
  const advisorWithRepair = {
    ...M.PIN_REGISTRY["advisor-protocol"],
    repaired_from: "0".repeat(40),
    repair_reason: "not a real repair",
  };
  const rN = await M.derivePin(advisorWithRepair, "advisor-protocol");
  ok(rN.ok, "a repair declaration on an UNREPAIRED pin is inert — the branch is only reachable on a real disagreement", rN.issues);
  ok(rN.via === "git", "the unrepaired pin still resolves via git", rN.via);
}

// ═══════════════════════════════════════════════════════════════════════════
// f2-2 (F11/F12 + the f2-1b judge's residual gap)
//
// Set-equality proves an anchor was CLAIMED. It cannot prove the anchor was
// carried — an owner concept that summarises its anchor away still hits the
// count. So three more guards, each asserted here before it exists:
//
//   F11  a per-anchor fidelity floor (normalized token overlap >= 0.60)
//   F12  per-area drift telemetry + whole-bundle invariants every check
//   ---  expected_counts.unparsed_blocks made MANDATORY on every pin
//
// The tuning rule is inverted from the usual instinct and is asserted as such
// below: advisor-protocol's 26 anchors and the 47 patterns must ALL clear the
// floor with ZERO edits to their concepts. A failure here means the
// NORMALIZATION is wrong — never the migrated content, never the threshold.
// ═══════════════════════════════════════════════════════════════════════════

for (const name of [
  "FIDELITY_FLOOR",
  "normalizeTokens",
  "tokenOverlap",
  "fidelityReport",
  "areaFidelity",
  "collectTelemetry",
  "telemetryIssues",
  "bundleInvariantIssues",
  "runBundleInvariants",
  // f4-7: the coverage walk and the honest denominator it feeds.
  "collectClaims",
  "anchorOwningConcepts",
]) {
  ok(typeof M[name] !== "undefined", `scripts/okf_migrate.mjs must export ${name} (f2-2)`);
}

// ─── 15. a pin omitting expected_counts.unparsed_blocks is REFUSED ──────────
//
// The f2-1b judge's residual gap: unparsed_blocks is the ONLY assertion that
// stops a future extractor regression from reclassifying anchors as prose
// while the totals still add up. A pin that omits it opts out of that guard
// silently, so omitting it is now PIN_INCOMPLETE.

{
  const real = M.PIN_REGISTRY["advisor-protocol"];
  const { unparsed_blocks: _dropped, ...withoutUnparsed } = real.expected_counts;
  const bad = { ...real, expected_counts: withoutUnparsed };
  const r = await M.derivePin(bad, "advisor-protocol");
  ok(r.ok === false, "a pin whose expected_counts omits unparsed_blocks is REFUSED");
  ok(codes(r).includes("PIN_INCOMPLETE"), "a pin omitting unparsed_blocks reports PIN_INCOMPLETE", codes(r));
  ok(
    (r.issues || []).some((i) => /unparsed_blocks/.test(i.message)),
    "the PIN_INCOMPLETE message NAMES unparsed_blocks",
    (r.issues || []).map((i) => i.message),
  );

  const cli = runCli(["--derive", JSON.stringify(bad)]);
  ok(cli.code === 1, "CLI --derive exits 1 on a pin missing unparsed_blocks", `exit ${cli.code}`);
  ok(/PIN_INCOMPLETE/.test(cli.err + cli.out), "CLI names PIN_INCOMPLETE for the missing unparsed_blocks", cli.err + cli.out);

  // every pin in the registry that CAN be gated declares it
  for (const [area, pin] of Object.entries(M.PIN_REGISTRY)) {
    if (pin.scheme === null) continue;
    ok(
      typeof pin.expected_counts?.unparsed_blocks === "number",
      `pin "${area}" declares expected_counts.unparsed_blocks (now mandatory)`,
      pin.expected_counts,
    );
  }
}

// ─── 16. the extraction carries anchor TEXT, not only anchor ids ────────────
//
// A fidelity floor is impossible without the anchor's own bytes. The texts
// must come from the same pinned blob the ids come from — never from the
// post-migration stub, which no longer holds the content.

{
  const adv = await M.derivePinForArea("advisor-protocol");
  const texts = adv.anchors?.texts;
  ok(texts instanceof Map, "the ba-nine-section extraction returns an anchor->text Map", typeof texts);
  ok(texts?.size === 26, "advisor-protocol yields 26 anchor texts (one per derived anchor)", texts?.size);
  ok(
    /dispatcher offers the adviser/.test(texts?.get("B1") || ""),
    "B1's anchor text is its own block from the pinned blob",
    (texts?.get("B1") || "").slice(0, 80),
  );
  ok(
    /two per claim/.test(texts?.get("R7") || ""),
    "R7's anchor text is its own bullet from the pinned blob",
    texts?.get("R7"),
  );
  ok(
    (texts?.get("B1") || "").split("\n").length > 1,
    "an anchor's text includes its continuation lines, not just its first line",
    texts?.get("B1"),
  );

  const pat = await M.derivePinForArea("critical-patterns");
  ok(pat.anchors?.texts?.size === 47, "critical-patterns yields 47 anchor texts", pat.anchors?.texts?.size);
  ok(
    (pat.anchors?.texts?.get("PAT1") || "").startsWith("## ["),
    "a PAT anchor's text starts at its dated heading",
    (pat.anchors?.texts?.get("PAT1") || "").slice(0, 60),
  );
}

// ─── 17. normalization + overlap are total and symmetric-free ──────────────

{
  ok(M.FIDELITY_FLOOR === 0.6, "the fidelity floor is 0.60 (F11) — never lowered to make a shipped area pass", M.FIDELITY_FLOOR);
  const t = M.normalizeTokens("**Read-only** `--runtime` advice, never a *gate*.");
  ok(t instanceof Set, "normalizeTokens returns a Set", typeof t);
  ok(t.has("read") && t.has("only"), "markdown emphasis and punctuation are stripped before splitting", [...t]);
  ok(t.has("runtime"), "a backticked flag normalizes to its word", [...t]);
  ok(!t.has("a") && t.has("never"), "a small stopword set is dropped; meaning-bearing words like `never` are NOT", [...t]);
  ok(M.tokenOverlap("alpha beta gamma", "alpha beta gamma delta") === 1, "full retention scores 1.0");
  ok(M.tokenOverlap("alpha beta gamma delta", "alpha beta") === 0.5, "half the anchor tokens retained scores 0.5", M.tokenOverlap("alpha beta gamma delta", "alpha beta"));
}

// ─── 18. THE TUNING RULE: both shipped areas clear 0.60 UNEDITED ────────────
//
// If either of these goes red, the normalization is wrong. Fixing it by
// editing a migrated concept, or by lowering the floor, would make this suite
// assert nothing at all.

const fidelityStats = {};
for (const area of ["advisor-protocol", "critical-patterns"]) {
  const f = await M.areaFidelity(area);
  ok(f.ok, `areaFidelity("${area}") resolves`, f.issues);
  fidelityStats[area] = f.stats;
  ok(
    f.issues.length === 0,
    `${area}: every anchor clears the ${M.FIDELITY_FLOOR} fidelity floor with ZERO concept edits — a failure here means the NORMALIZATION is wrong, never the content`,
    f.issues,
  );
  ok(f.rows.length === (area === "advisor-protocol" ? 26 : 47), `${area}: one fidelity row per anchor`, f.rows.length);
  ok(f.stats.min >= M.FIDELITY_FLOOR, `${area}: min overlap ${f.stats.min?.toFixed(3)} >= ${M.FIDELITY_FLOOR}`, f.stats);
  ok(
    f.stats.median >= 0.75,
    `${area}: median overlap ${f.stats.median?.toFixed(3)} sits well clear of the floor — a median barely above 0.60 would mean the normalization is too strict`,
    f.stats,
  );
}
console.log(
  `      fidelity margin (F11, floor ${M.FIDELITY_FLOOR}): ` +
    Object.entries(fidelityStats)
      .map(([a, s]) => `${a} n=${s.n} min=${s.min.toFixed(3)} median=${s.median.toFixed(3)} max=${s.max.toFixed(3)}`)
      .join(" | "),
);

// ─── 19. a concept that SUMMARISES its anchor away falls below the floor ────
//
// The whole point of F11: this fixture passes set-equality (the anchor is
// claimed by exactly one concept) and still fails, because the content is
// gone. The failure must name the anchor, its owner, and the measured ratio.

{
  const anchorText =
    "- R4 — High-risk execution approval requires a live consult record; staleness is event-based (four events), never a time limit, and the approval verb refuses a missing or stale record with a corrective message naming each failed condition.";
  const faithful =
    "High-risk execution approval requires a live consult record. Staleness is event-based — four events — never a time limit. The approval verb refuses a missing or stale record with a corrective message naming each failed condition.";
  const summarised = "The orchestrator checks freshness before approving.";

  const good = M.fidelityReport({
    expected: ["R4"],
    texts: new Map([["R4", anchorText]]),
    claims: new Map([["R4", ["docs/knowledge/areas/x/faithful.md"]]]),
    bodies: new Map([["docs/knowledge/areas/x/faithful.md", faithful]]),
  });
  ok(good.issues.length === 0, "a faithful concept clears the floor", good.issues);

  const bad = M.fidelityReport({
    expected: ["R4"],
    texts: new Map([["R4", anchorText]]),
    claims: new Map([["R4", ["docs/knowledge/areas/x/summarised.md"]]]),
    bodies: new Map([["docs/knowledge/areas/x/summarised.md", summarised]]),
  });
  ok(bad.issues.length === 1, "a summarised concept FAILS the floor even though set-equality is satisfied", bad.issues);
  const msg = bad.issues[0] || "";
  ok(/\bR4\b/.test(msg), "the fidelity failure NAMES the anchor", msg);
  ok(/summarised\.md/.test(msg), "the fidelity failure NAMES the owning concept", msg);
  ok(/0\.\d{2,}/.test(msg), "the fidelity failure quotes the MEASURED ratio", msg);
  ok(/0\.6/.test(msg), "the fidelity failure quotes the floor it fell below", msg);
  ok(bad.rows[0].ratio < M.FIDELITY_FLOOR, "the row carries the sub-floor ratio", bad.rows[0]);
  console.log(`      summarised-fixture failure: ${msg}`);

  // The discriminating case: not an empty stub, but a plausible-looking
  // paraphrase that keeps the gist and drops the specifics — the shape
  // anchor-shaped compliance actually takes in a long re-authoring run.
  const paraphrased =
    "High-risk execution approval requires a live consult record before the gate opens.";
  const partial = M.fidelityReport({
    expected: ["R4"],
    texts: new Map([["R4", anchorText]]),
    claims: new Map([["R4", ["docs/knowledge/areas/x/paraphrased.md"]]]),
    bodies: new Map([["docs/knowledge/areas/x/paraphrased.md", paraphrased]]),
  });
  ok(partial.issues.length === 1, "a plausible paraphrase that drops the specifics ALSO fails the floor", partial.rows);
  ok(
    partial.rows[0].ratio > 0 && partial.rows[0].ratio < M.FIDELITY_FLOOR,
    `the floor discriminates rather than merely detecting absence — the paraphrase scores ${partial.rows[0]?.ratio?.toFixed(3)}, between 0 and the floor`,
    partial.rows[0],
  );
  console.log(`      paraphrase-fixture failure: ${partial.issues[0]}`);
}

// ─── 20. drift telemetry (F12) reports always, fails only with a median ─────

{
  const tel = await M.collectTelemetry();
  ok(Array.isArray(tel) && tel.length >= 2, "collectTelemetry returns one row per gateable pinned area", tel.length);
  for (const row of tel) {
    ok(typeof row.anchors_per_concept === "number", `${row.area}: anchors_per_concept is reported`, row);
    ok(typeof row.concepts_per_100_source_lines === "number", `${row.area}: concepts_per_100_source_lines is reported`, row);
    ok(row.source_lines > 0, `${row.area}: source line count comes from the pinned blob`, row);
    // f4-7: the measured denominator is anchor-OWNING concepts; the directory
    // file count is reported beside it and can only ever be >= it.
    ok(
      Number.isInteger(row.concepts) && row.concepts > 0 && Number.isInteger(row.concept_files) && row.concept_files >= row.concepts,
      `${row.area}: the denominator is anchor-owning concepts, reported beside the area's file count`,
      row,
    );
    ok(
      row.anchors_per_concept === row.anchors / row.concepts &&
        row.concepts_per_100_source_lines === (row.concepts * 100) / row.source_lines,
      `${row.area}: BOTH ratios use the anchor-owning denominator, not the file count`,
      row,
    );
  }
  console.log(
    `      telemetry (F12): ` +
      tel.map((r) => `${r.area} anchors/concept=${r.anchors_per_concept.toFixed(2)} concepts/100L=${r.concepts_per_100_source_lines.toFixed(2)} (${r.concepts} owning of ${r.concept_files} files)`).join(" | "),
  );

  // f4-7: the band and the sample floor are the two knobs a "fix" could turn to
  // make a red vanish. Pinned here as literals so turning either one is a test
  // failure, not a quiet edit.
  ok(M.TELEMETRY_MIN_SAMPLES === 3, "TELEMETRY_MIN_SAMPLES is 3 — unmoved", M.TELEMETRY_MIN_SAMPLES);
  {
    // The band itself, probed from outside: 2.01x fails, 2.00x passes, 0.49x
    // fails, 0.50x passes. Reading the constants would prove nothing; these
    // probe the behaviour.
    const base = [
      { area: "a", anchors_per_concept: 10, concepts_per_100_source_lines: 2 },
      { area: "b", anchors_per_concept: 10, concepts_per_100_source_lines: 2 },
      { area: "c", anchors_per_concept: 10, concepts_per_100_source_lines: 2 },
    ];
    const at = (v) => M.telemetryIssues({ current: { area: "probe", anchors_per_concept: v, concepts_per_100_source_lines: 2 }, samples: base });
    ok(at(20).length === 0, "the band's upper edge is exactly 2x — 2.00x passes", at(20));
    ok(at(20.1).length > 0, "2.01x fails — the band was not widened upward", at(20.1));
    ok(at(5).length === 0, "the band's lower edge is exactly 0.5x — 0.50x passes", at(5));
    ok(at(4.9).length > 0, "0.49x fails — the band was not widened downward", at(4.9));
  }

  // Fewer than three pinned areas: no median exists, so telemetry REPORTS and
  // never fails — even for a wild outlier. This is the state the repo is in
  // today (2 gateable pins), so the chain must not go red on it.
  const wild = { area: "wild", anchors_per_concept: 99, concepts_per_100_source_lines: 0.01 };
  const two = [
    { area: "a", anchors_per_concept: 6, concepts_per_100_source_lines: 2 },
    { area: "b", anchors_per_concept: 6.5, concepts_per_100_source_lines: 2.1 },
  ];
  // Two pinned areas in total (the outlier IS one of them): a two-sample
  // "median" is a coin flip, so telemetry reports and never fails.
  ok(
    M.telemetryIssues({ current: wild, samples: [two[0], wild] }).length === 0,
    "with fewer than 3 pinned areas telemetry reports only — never fails (no median exists yet)",
    M.telemetryIssues({ current: wild, samples: [two[0], wild] }),
  );
  ok(
    M.telemetryIssues({ current: tel[0], samples: tel }).length === 0,
    "the repo's current 2-pin state produces no telemetry failure",
    M.telemetryIssues({ current: tel[0], samples: tel }),
  );

  const three = [
    ...two,
    { area: "c", anchors_per_concept: 6.2, concepts_per_100_source_lines: 2.05 },
  ];
  ok(
    M.telemetryIssues({ current: three[0], samples: [...three, wild] }).length === 0,
    "an in-band area passes once a median exists",
    M.telemetryIssues({ current: three[0], samples: [...three, wild] }),
  );
  const outlierIssues = M.telemetryIssues({ current: wild, samples: [...three, wild] });
  ok(outlierIssues.length > 0, "an OUTLIER area fails once >= 3 pinned areas give a median", outlierIssues);
  ok(/wild/.test(outlierIssues.join(" ")), "the telemetry failure names the outlier area", outlierIssues);
  ok(
    /anchors_per_concept/.test(outlierIssues.join(" ")) && /median/.test(outlierIssues.join(" ")),
    "the telemetry failure names the metric and the median it drifted from",
    outlierIssues,
  );
  ok(
    M.telemetryIssues({
      current: { area: "low", anchors_per_concept: 1, concepts_per_100_source_lines: 2 },
      samples: [...three, { area: "low", anchors_per_concept: 1, concepts_per_100_source_lines: 2 }],
    }).length > 0,
    "the <0.5x side of the band fails too, not only >2x",
  );
}

// ─── 20b. the corrected denominator still BITES, both directions (f4-7) ─────
//
// f4-7 changed what F12's denominator measures: anchor-OWNING concepts, not
// every .md in the area directory. A "fix" that merely made a red go away would
// look identical from the green side, so the guard is asserted from the side
// that matters — a deliberately BADLY DECOMPOSED migration must still be caught,
// in both directions, against the repo's own real running median.
//
// The fixtures are real files walked by the real `collectClaims`, not a
// hand-built claims Map: if the walk and the denominator ever disagree the
// fixture stops reproducing the shape it is named for. They live under .bee/tmp/
// rather than docs/knowledge/ on purpose — a fixture area dropped into the
// bundle would be seen by the whole-bundle invariants of every concurrently
// running suite.
//
// The dumping-ground fixture is also the proof that the OLD denominator was
// blind: 24 anchors in ONE concept, in a directory of 6 files. Counting files
// scores it 4.00 anchors/concept — comfortably in band, the fault invisible.
// Counting anchor-owning concepts scores it 24.00 and fires.

{
  const source = "docs/specs/okf-profile.md";
  const anchorPattern = M.AREA_ANCHOR_PATTERN;
  const anchors = Array.from({ length: 24 }, (_, i) => `B${i + 1}`);
  const SOURCE_LINES = 612; // okf-profile's pinned source length
  const fixtureRoot = path.join(REPO_ROOT, ".bee", "tmp", "f4-7-decomposition-fixtures");

  const write = (dirRel, name, owned) => {
    const abs = path.join(REPO_ROOT, ...dirRel.split("/"), name);
    fs.mkdirSync(path.dirname(abs), { recursive: true });
    const sources = owned.map((a) => `"${source}#${a}"`).join(", ");
    fs.writeFileSync(
      abs,
      `---\nbee:\n  id: ${name.replace(/\.md$/, "")}\n  sources: [${sources}]\n---\n\nfixture body\n`,
    );
  };

  // The real comparison population, computed by the real collector.
  const nine = (await M.collectTelemetry()).filter((r) => r.scheme === "ba-nine-section");
  const rowFor = async (dirRel) => {
    const { claims } = await M.collectClaims({ dir: dirRel, source, anchorPattern });
    const concepts = M.anchorOwningConcepts(claims, anchors).size;
    const files = fs.readdirSync(path.join(REPO_ROOT, ...dirRel.split("/"))).filter((n) => n.endsWith(".md")).length;
    return {
      area: dirRel.split("/").pop(),
      concepts,
      files,
      anchors_per_concept: anchors.length / concepts,
      concepts_per_100_source_lines: (concepts * 100) / SOURCE_LINES,
      // What the pre-f4-7 denominator would have scored the same directory.
      old_anchors_per_concept: anchors.length / files,
      old_concepts_per_100_source_lines: (files * 100) / SOURCE_LINES,
    };
  };

  try {
    // (a) DUMPING GROUND: all 24 anchors swallowed by one concept, beside five
    //     concepts of new truth that own nothing from the pinned source.
    const dumpDir = ".bee/tmp/f4-7-decomposition-fixtures/dumping-ground";
    write(dumpDir, "everything.md", anchors);
    for (let i = 0; i < 5; i += 1) write(dumpDir, `new-truth-${i}.md`, []);

    // (b) SHREDDED: the opposite fault — one anchor per concept.
    const shredDir = ".bee/tmp/f4-7-decomposition-fixtures/shredded";
    anchors.forEach((a, i) => write(shredDir, `piece-${i}.md`, [a]));

    // (c) HEALTHY: okf-profile's real shape (5 owning concepts) plus the two
    //     new-truth concepts it really grew. The control on the control — if
    //     this failed, the fixtures would prove nothing about decomposition.
    const healthyDir = ".bee/tmp/f4-7-decomposition-fixtures/healthy";
    [[0, 5], [5, 10], [10, 15], [15, 20], [20, 24]].forEach(([a, b], i) =>
      write(healthyDir, `part-${i}.md`, anchors.slice(a, b)),
    );
    write(healthyDir, "new-truth-a.md", []);
    write(healthyDir, "new-truth-b.md", []);

    const dump = await rowFor(dumpDir);
    const shred = await rowFor(shredDir);
    const healthy = await rowFor(healthyDir);

    ok(dump.concepts === 1 && dump.files === 6, "the dumping-ground fixture has ONE anchor-owning concept among six files", dump);
    ok(shred.concepts === 24 && shred.files === 24, "the shredded fixture has one anchor-owning concept per anchor", shred);
    ok(healthy.concepts === 5 && healthy.files === 7, "the healthy fixture reproduces okf-profile's real shape", healthy);

    const dumpIssues = M.telemetryIssues({ current: dump, samples: [...nine, dump] });
    ok(dumpIssues.length > 0, "NEGATIVE CONTROL: a dumping-ground concept is STILL an outlier under the corrected denominator", dumpIssues);
    ok(
      /anchors_per_concept/.test(dumpIssues.join(" ")),
      "the dumping-ground fixture fires on anchors_per_concept — the metric whose stated purpose is exactly this fault",
      dumpIssues,
    );

    const shredIssues = M.telemetryIssues({ current: shred, samples: [...nine, shred] });
    ok(shredIssues.length > 0, "NEGATIVE CONTROL: concepts shredded one-anchor-each are STILL an outlier", shredIssues);

    ok(
      M.telemetryIssues({ current: healthy, samples: [...nine, healthy] }).length === 0,
      "a well-decomposed fixture of the same directory size passes — the fixtures discriminate rather than always firing",
      M.telemetryIssues({ current: healthy, samples: [...nine, healthy] }),
    );

    // The blindness the correction removed, stated as an assertion rather than
    // as prose: scored by DIRECTORY FILE COUNT, the dumping ground passes.
    ok(
      M.telemetryIssues({
        current: {
          area: "dumping-ground-scored-the-old-way",
          anchors_per_concept: dump.old_anchors_per_concept,
          concepts_per_100_source_lines: dump.old_concepts_per_100_source_lines,
        },
        samples: nine,
      }).length === 0,
      `the OLD denominator was blind to it: file-count scores the same dumping ground ${dump.old_anchors_per_concept.toFixed(2)} anchors/concept — in band, fault invisible`,
      dump,
    );

    console.log(
      `      f4-7 negative control: dumping-ground ${dump.anchors_per_concept.toFixed(2)} anchors/concept (file-count would say ${dump.old_anchors_per_concept.toFixed(2)}) | shredded ${shred.anchors_per_concept.toFixed(2)} | healthy ${healthy.anchors_per_concept.toFixed(2)}`,
    );
    console.log(`      f4-7 dumping-ground failure: ${dumpIssues[0]}`);
    console.log(`      f4-7 shredded failure: ${shredIssues[0]}`);
  } finally {
    fs.rmSync(fixtureRoot, { recursive: true, force: true });
  }
}

// ─── 21. whole-bundle invariants run on EVERY check ─────────────────────────

{
  const healthy = M.runBundleInvariants();
  ok(healthy.issues.length === 0, "the bundle as shipped is healthy (authority unique, zero not_canonical, indexes fresh)", healthy.issues);

  const doctored = M.bundleInvariantIssues({
    check: {
      okf: { errors: [] },
      profile: {
        warnings: [
          { file: "docs/knowledge/areas/x/a.md", code: "not_canonical", message: "frontmatter parse→re-emit differs" },
          { file: "docs/knowledge/areas/x/b.md", code: "duplicate_authoritative_for", message: "claimed by 2 concepts" },
          { file: "docs/knowledge/areas/x/c.md", code: "missing_profile_field", message: "still just a warning" },
        ],
      },
    },
    index: { ok: true, output: "0 stale" },
  });
  ok(doctored.some((i) => /not_canonical/.test(i)), "a not_canonical concept makes the bundle unhealthy", doctored);
  ok(doctored.some((i) => /duplicate_authoritative_for/.test(i)), "a duplicated authority makes the bundle unhealthy", doctored);
  ok(
    !doctored.some((i) => /missing_profile_field/.test(i)),
    "an unrelated profile warning does NOT fail the gate (D13 stays intact)",
    doctored,
  );
  ok(
    M.bundleInvariantIssues({ check: { okf: { errors: [{ file: "z.md", code: "empty_type", message: "x" }] }, profile: { warnings: [] } }, index: { ok: true, output: "" } }).length > 0,
    "any OKF error makes the bundle unhealthy",
  );
  ok(
    M.bundleInvariantIssues({ check: { okf: { errors: [] }, profile: { warnings: [] } }, index: { ok: false, output: "2 stale" } }).some((i) => /index/i.test(i)),
    "a stale generated index makes the bundle unhealthy",
  );
}

// ─── 22. end to end: a doctored bundle turns the shipped gate RED ───────────
//
// Not a unit-level assertion of the pure function — the real CLI, over the
// real bundle, with one deliberately non-canonical concept dropped in. If
// this passes green the invariants are computed but not wired.

{
  const fixtureRel = "docs/knowledge/areas/advisor-protocol/zz-f2-2-unhealthy-fixture.md";
  const fixtureAbs = path.join(REPO_ROOT, ...fixtureRel.split("/"));
  const before = runCli(["--check", "advisor-protocol"]);
  ok(before.code === 0, "the gate is green before the bundle is doctored", before.err);
  try {
    fs.writeFileSync(
      fixtureAbs,
      // hand-authored, deliberately outside the canonical emitted form:
      // key order and quoting differ, so parse→re-emit cannot round-trip.
      "---\nbee:\n  id: zz-f2-2-fixture\ntype: bee.pattern\ntitle: 'doctored fixture'\n---\n\nA deliberately non-canonical concept.\n",
    );
    const red = runCli(["--check", "advisor-protocol"]);
    ok(red.code === 1, "a doctored bundle turns `--check advisor-protocol` RED", `exit ${red.code}\n${red.out}${red.err}`);
    ok(
      /BUNDLE UNHEALTHY/.test(red.err + red.out),
      "the gate names the bundle invariant it failed",
      red.err + red.out,
    );
  } finally {
    fs.rmSync(fixtureAbs, { force: true });
  }
  const after = runCli(["--check", "advisor-protocol"]);
  ok(after.code === 0, "the gate is green again once the fixture is removed", after.err);
}

// ─── 23. the shipped gates report fidelity + telemetry, keeping PASS wording ─

{
  const cli = runCli(["--check", "advisor-protocol"]);
  ok(cli.code === 0, "`--check advisor-protocol` still exits 0 with F11/F12 live", cli.err);
  ok(/^PASS okf_migrate --check advisor-protocol: /m.test(cli.out), "the chain PASS wording is unchanged", cli.out);
  ok(/fidelity/i.test(cli.out), "`--check` reports the fidelity margin", cli.out);
  ok(/min /.test(cli.out) && /median /.test(cli.out), "`--check` quotes min and median overlap so the margin is visible", cli.out);
  ok(/anchors_per_concept/.test(cli.out), "`--check` reports drift telemetry", cli.out);

  const cli2 = runCli(["--check-patterns"]);
  ok(cli2.code === 0, "`--check-patterns` still exits 0 with F11/F12 live", cli2.err);
  ok(/^PASS okf_migrate --check-patterns: /m.test(cli2.out), "the chain PASS wording is unchanged", cli2.out);
  ok(/fidelity/i.test(cli2.out) && /median /.test(cli2.out), "`--check-patterns` reports the fidelity margin", cli2.out);

  const fid = runCli(["--fidelity", "advisor-protocol"]);
  ok(fid.code === 0, "`--fidelity <area>` reports the per-anchor table", fid.err);
  let parsedFid = null;
  try {
    parsedFid = JSON.parse(fid.out);
  } catch { /* reported below */ }
  ok(parsedFid?.rows?.length === 26, "`--fidelity` emits one row per anchor", parsedFid?.rows?.length);
  ok(typeof parsedFid?.stats?.median === "number", "`--fidelity` emits the median", parsedFid?.stats);
}

// ═══════════════════════════════════════════════════════════════════════════
// f2-4 — THE ID-FORM WIDENING, AND THE NO-OP THAT BOUNDS IT
//
// f2-3 returned [BLOCKED] rather than force a scheme, and its evidence exposed
// a systemic defect: the shipped classifier required a BARE id at the head of
// a block (`**B1 — …`, `- R1 — …`), so five of the nine remaining areas —
// which write the same anchors with the id bold-wrapped — derived R0 while
// carrying real, numbered rules. doctrine-layer inventoried 20 anchors with 21
// UNPARSED blocks. decision-memory was filed by the planning sweep as
// "shapeless, 0 anchors" and slated for a bespoke scheme; it is not shapeless
// at all — its 9 rules were invisible. A verdict of "no structure here" that is
// really "this reader cannot see the structure" is the same lie the derived
// gate exists to prevent, one level up.
//
// The widening recognises ID FORMS ONLY, and the property that bounds it is a
// STRICT NO-OP on both shipped pins. If a widening moves 26 or 47, it is too
// broad and must be narrowed — never the other way around, and never by
// touching expected_counts.
// ═══════════════════════════════════════════════════════════════════════════

// ─── 24. every id form observed across the nine remaining areas is classified ─

{
  // Every line below is a form MEASURED in docs/specs/, with its source cited.
  const FIXTURE = [
    "# Fixture",
    "",
    "## Behaviors & Operations",
    "",
    "**B1 — Bare bold id.** The shipped form; advisor-protocol writes all four this way.",
    "",
    "**B3a — Letter-suffixed id.** doctrine-layer L89, hook-runtime L95, workflow-state L207.",
    "",
    "**B7** — Id closed before the dash.",
    "",
    "**Detect (every run).** An unnumbered bold-lead paragraph — onboarding L96. It carries",
    "no id, so it MUST stay unparsed: inventing a positional id would fabricate structure",
    "the source never had (D10) and would collide with the source's own B-ids.",
    "",
    "## Business Rules",
    "",
    "- R1 — Bare bullet id, the shipped form.",
    "- R8a — Bare and letter-suffixed; hook-runtime L412.",
    "- **R2** — Bold-wrapped id; doctrine-layer L213, onboarding L337.",
    "- **R20b** — Bold-wrapped and letter-suffixed; onboarding L478.",
    "- **R3** (D1) — Bold-wrapped with a citation before the dash; feedback-digest L259.",
    "- **R7 (not yet implemented — P24)** — A citation carrying its own em dash; onboarding L365.",
    "- **R4 — The whole rule bolded through the dash** (D7). decision-memory L16.",
    "- **Something unnumbered** — a bolded bullet with no id; verify-pipeline L19. Stays unparsed.",
    "",
  ].join("\n");

  const inv = M.inventorySpec(FIXTURE);
  eq(
    inv.behaviors,
    ["B1", "B3a", "B7"],
    "inventorySpec classifies bare, letter-suffixed, and bold-closed behavior ids",
  );
  eq(
    inv.rules,
    ["R1", "R8a", "R2", "R20b", "R3", "R7", "R4"],
    "inventorySpec classifies bare, bold-wrapped, letter-suffixed, citation-carrying, and dash-inside-bold rule ids",
  );
  ok(
    inv.unparsed.blocks.behaviors === 1,
    "an unnumbered bold-lead behavior paragraph stays UNPARSED — the widening reads ids, it never invents them",
    inv.unparsed.blocks,
  );
  ok(
    inv.unparsed.blocks.rules === 1,
    "an unnumbered bolded rule bullet stays UNPARSED",
    inv.unparsed.blocks,
  );
  // The anchor TEXT must still travel with the widened ids (F11 depends on it).
  ok(
    /Bold-wrapped id/.test(inv.texts.get("R2") || ""),
    "a widened rule id still carries its anchor text, so the fidelity floor can measure it",
    inv.texts.get("R2"),
  );
  ok(
    /Letter-suffixed id/.test(inv.texts.get("B3a") || ""),
    "a letter-suffixed behavior id still carries its anchor text",
    inv.texts.get("B3a"),
  );
}

// ─── 25. THE SAFETY PROPERTY: the widening is a strict no-op on both pins ────

{
  const a = await M.derivePinForArea("advisor-protocol");
  ok(a.ok, "STRICT NO-OP: advisor-protocol's pin is still green after the widening", a.issues);
  eq(
    {
      behaviors: a.counts?.behaviors,
      rules: a.counts?.rules,
      edges: a.counts?.edges,
      pointers: a.counts?.pointers,
      total: a.counts?.total,
      unparsed_blocks: a.counts?.unparsed_blocks,
    },
    { behaviors: 4, rules: 9, edges: 6, pointers: 7, total: 26, unparsed_blocks: 0 },
    "STRICT NO-OP: the widened classifier still derives exactly 26 {4,9,6,7} with 0 unparsed blocks from advisor-protocol's PINNED blob",
  );
  const p = await M.derivePinForArea("critical-patterns");
  ok(p.ok, "STRICT NO-OP: critical-patterns' pin is still green after the widening", p.issues);
  eq(
    { patterns: p.counts?.patterns, total: p.counts?.total, unparsed_blocks: p.counts?.unparsed_blocks },
    { patterns: 47, total: 47, unparsed_blocks: 0 },
    "STRICT NO-OP: critical-patterns still derives exactly 47 with 0 unparsed blocks",
  );
  // ... and the pins themselves were not "adjusted" to make that true.
  ok(
    M.PIN_REGISTRY["advisor-protocol"].expected_counts.total === 26 &&
      M.PIN_REGISTRY["advisor-protocol"].expected_counts.unparsed_blocks === 0 &&
      M.PIN_REGISTRY["critical-patterns"].expected_counts.total === 47,
    "the no-op was proven against UNCHANGED expected_counts — never by relaxing the pins",
    { advisor: M.PIN_REGISTRY["advisor-protocol"].expected_counts, patterns: M.PIN_REGISTRY["critical-patterns"].expected_counts },
  );
}

// ─── 26. the widening is bounded: it may not classify a bare prose block ────

{
  const NOT_ANCHORS = [
    "## Business Rules",
    "",
    "- **Reserve** — this bullet names no numbered id, so it is prose to the classifier.",
    "- R1 - an ASCII hyphen is not the em dash the scheme uses.",
    "- **RULES** — an all-caps word starting with R is not an id.",
    "",
  ].join("\n");
  const inv = M.inventorySpec(NOT_ANCHORS);
  eq(inv.rules, [], "the widened rule pattern still refuses unnumbered, hyphenated, and word-shaped bullets");
  ok(inv.unparsed.blocks.rules === 3, "all three refused bullets are REPORTED as unparsed blocks", inv.unparsed.blocks);
}

// ─── 27. the THIRD scheme: `narrative-sections` (f2-11, F9/S5) ──────────────
//
// worktree-parallelism is the one area of the eleven that genuinely has no
// numbered anchors — not because the extractor is blind (f2-4 fixed that and
// re-confirmed this file afterwards), but because the source truly carries no
// `B*`/`R*`/`E*`/`P*` ids and no nine-section headings to derive them from.
// F9 forbids forcing it into `ba-nine-section`, and D10 forbids inventing
// numbered ids the source never had. So the anchors are the source's OWN `## `
// headings, slugified — derived mechanically from the heading text, exactly as
// `flat-pattern-list` already derives one anchor per `## [YYYYMMDD] …`
// heading. The headings ARE the ground truth; nothing here is invented.
//
// The three properties this scheme must hold, all asserted below:
//   a `## ` heading becomes an anchor · a `###` subheading does NOT · a source
//   with zero `## ` headings is REFUSED rather than passed 0/0.

{
  for (const name of ["inventoryNarrativeSections", "slugifyHeading", "NARRATIVE_ANCHOR_PATTERN"]) {
    ok(typeof M[name] !== "undefined", `scripts/okf_migrate.mjs must export ${name} (f2-11)`);
  }
  ok(
    typeof M.SCHEMES === "object" && typeof M.SCHEMES["narrative-sections"] === "function",
    "the `narrative-sections` scheme is registered in SCHEMES",
    Object.keys(M.SCHEMES || {}),
  );

  // (a) a `## ` heading becomes an anchor, id derived from the heading TEXT.
  const SRC = [
    "# Spec — Something",
    "",
    "**Area:** a preamble block that belongs to no section.",
    "**Status:** shipped.",
    "",
    "## The trust model (the load-bearing rule)",
    "",
    "A worktree gets its own store ONLY when it is granted.",
    "",
    "### A subheading, which is NOT an anchor",
    "",
    "Nested prose that must travel with the section above it.",
    "",
    "## Boundary (out of scope)",
    "",
    "- Rollout to onboarded host repos — deferred.",
    "",
  ].join("\n");

  const inv = M.inventoryNarrativeSections(SRC);
  eq(
    inv.sections,
    ["S-the-trust-model-the-load-bearing-rule", "S-boundary-out-of-scope"],
    "each `## ` heading becomes ONE anchor, id slugified from the heading's own text (never a numbered id — D10)",
  );
  eq(inv.all, inv.sections, "the narrative inventory's `all` is its section list, in document order");
  ok(
    M.slugifyHeading("Entering: `worktree new --feature <slug>` (D7, GH #21)") ===
      "entering-worktree-new-feature-slug-d7-gh-21",
    "slugifyHeading collapses backticks, punctuation and angle brackets into a single-hyphen slug",
    M.slugifyHeading("Entering: `worktree new --feature <slug>` (D7, GH #21)"),
  );

  // (b) a `###` subheading is NOT an anchor — and it is REPORTED, never silently
  //     dropped, exactly as the BA scheme reports its unnumbered bold-leads.
  ok(
    !inv.all.some((a) => /subheading/.test(a)),
    "a `###` subheading is NOT an anchor under narrative-sections",
    inv.all,
  );
  ok(
    inv.unparsed.blocks.subheadings === 1,
    "the `###` subheading is REPORTED as an unparsed block (visible, never invented into an anchor)",
    inv.unparsed.blocks,
  );
  ok(
    inv.unparsed.blocks.preamble === 2,
    "block starts before the first `## ` heading are reported as unparsed preamble blocks",
    inv.unparsed.blocks,
  );
  ok(
    inv.unparsed.blocks.total === 3 && inv.unparsed.lines.total > 0,
    "the unparsed report totals both kinds and counts unclassified body lines",
    inv.unparsed,
  );

  // (c) the anchor's TEXT is its section body up to the NEXT `## ` heading —
  //     so a `###` subheading's prose travels with the section it sits in and
  //     the fidelity floor can measure it.
  const trust = inv.texts.get("S-the-trust-model-the-load-bearing-rule");
  ok(
    /A worktree gets its own store/.test(trust) &&
      /### A subheading/.test(trust) &&
      /Nested prose that must travel/.test(trust) &&
      !/Boundary \(out of scope\)/.test(trust),
    "an anchor's text is the heading plus its body up to the next `## ` heading, subheading content included",
    trust,
  );

  // (d) ZERO `## ` headings is a REFUSAL, never a 0/0 pass. This is the hole
  //     the whole gate exists to close: a scheme that returns an empty set for
  //     a file it cannot read converts lost content into content that never
  //     existed.
  let threw = null;
  try {
    M.extractByScheme("narrative-sections", "# Title only\n\nSome prose and no `## ` heading at all.\n");
  } catch (error) {
    threw = error;
  }
  ok(threw !== null, "a source with ZERO `## ` headings is REFUSED by the scheme, never passed 0/0");
  ok(threw?.code === "PIN_EMPTY_EXTRACTION", "the zero-heading refusal is typed PIN_EMPTY_EXTRACTION", threw?.code);
  ok(
    /## /.test(String(threw?.message || "")),
    "the refusal names what it could not find, so the reader knows the scheme did not fit",
    threw?.message,
  );

  // …and the same refusal reaches derivePin as an exit-1 typed failure rather
  // than a green pin, even when the pin obligingly declares `total: 0`.
  const emptyPath = path.join(TMP, "no-headings.md");
  const emptyText = "# Title only\n\nProse, and not one `## ` heading.\n";
  fs.writeFileSync(emptyPath, emptyText);
  const emptyPin = {
    kind: "area",
    path: "docs/specs/nothing.md",
    blob_sha: blobSha(emptyText),
    scheme: "narrative-sections",
    expected_counts: { sections: 0, total: 0, unparsed_blocks: 0 },
    source_copy: path.relative(REPO_ROOT, emptyPath).split(path.sep).join("/"),
  };
  const emptyResult = await M.derivePin(emptyPin, "no-headings");
  ok(emptyResult.ok === false, "a narrative pin over a heading-less source is exit-1, never a 0/0 green");
  ok(
    codes(emptyResult).includes("PIN_EMPTY_EXTRACTION"),
    "the heading-less narrative pin reports PIN_EMPTY_EXTRACTION",
    codes(emptyResult),
  );

  // (e) two headings that slugify to the SAME id are refused — the f2-10
  //     duplicate-`R14` hazard, closed at the scheme level this time rather
  //     than discovered in a source. Anchors are keyed by id, so a collision
  //     would silently overwrite the first section's text and be unmeasurable
  //     by the fidelity floor forever.
  let dupThrew = null;
  try {
    M.extractByScheme(
      "narrative-sections",
      "## The trust model!\n\nfirst\n\n## The trust model?\n\nsecond\n",
    );
  } catch (error) {
    dupThrew = error;
  }
  ok(dupThrew !== null, "two headings slugifying to the same anchor id are REFUSED, never silently collapsed");
  ok(
    dupThrew?.code === "PIN_DUPLICATE_ANCHOR",
    "the collision refusal is typed PIN_DUPLICATE_ANCHOR and names the id",
    `${dupThrew?.code}: ${dupThrew?.message}`,
  );

  // (f) the scheme reads HEADINGS, and only headings — a numbered id sitting in
  //     the body never becomes an anchor of its own.
  const numbered = M.inventoryNarrativeSections(
    "## A section\n\n- **R1** — a numbered rule in the body is NOT a narrative anchor.\n",
  );
  eq(numbered.all, ["S-a-section"], "a `B*`/`R*` id in the body is never promoted to an anchor by this scheme");
}

// ─── 28. worktree-parallelism's own gate, and the STRICT NO-OP (f2-11) ──────
//
// The safety property, identical in form to f2-4's classifier widening and
// f2-10's repaired-pin branch: adding a scheme may not move ANY existing pin.
// All nine pre-existing pins must still derive their exact expected_counts,
// and the proof is run against a FROZEN table here — never against
// PIN_REGISTRY's own numbers, which is how a relaxed pin would launder itself
// into a green.

{
  const FROZEN = {
    "advisor-protocol": { behaviors: 4, rules: 9, edges: 6, pointers: 7, total: 26, unparsed_blocks: 0 },
    "doctrine-layer": { behaviors: 10, rules: 17, edges: 5, pointers: 7, total: 39, unparsed_blocks: 2 },
    "critical-patterns": { patterns: 47, total: 47, unparsed_blocks: 0 },
    "decision-memory": { behaviors: 0, rules: 9, edges: 0, pointers: 0, total: 9, unparsed_blocks: 0 },
    "verify-pipeline": { behaviors: 0, rules: 5, edges: 4, pointers: 5, total: 14, unparsed_blocks: 7 },
    "performance-log": { behaviors: 0, rules: 11, edges: 5, pointers: 7, total: 23, unparsed_blocks: 10 },
    "feedback-digest": { behaviors: 0, rules: 15, edges: 6, pointers: 8, total: 29, unparsed_blocks: 26 },
    onboarding: { behaviors: 0, rules: 28, edges: 15, pointers: 15, total: 58, unparsed_blocks: 20 },
    "hook-runtime": { behaviors: 22, rules: 24, edges: 17, pointers: 18, total: 81, unparsed_blocks: 8 },
  };
  for (const [area, want] of Object.entries(FROZEN)) {
    const derived = await M.derivePinForArea(area);
    ok(derived.ok, `STRICT NO-OP: ${area}'s pin is still green after the narrative-sections scheme was added`, derived.issues);
    const got = Object.fromEntries(Object.keys(want).map((k) => [k, derived.counts?.[k]]));
    eq(got, want, `STRICT NO-OP: ${area} still derives exactly ${JSON.stringify(want)} from its pinned blob`);
    const declared = M.PIN_REGISTRY[area].expected_counts;
    const declaredSubset = Object.fromEntries(Object.keys(want).map((k) => [k, declared[k]]));
    eq(declaredSubset, want, `the no-op was proven against UNCHANGED expected_counts for ${area} — never by relaxing the pin`);
  }

  // The new area's own pin, and its own chain gate.
  const wt = await M.derivePinForArea("worktree-parallelism");
  ok(wt.ok, "worktree-parallelism's narrative-sections pin resolves, hashes, extracts and matches its counts", wt.issues);
  ok(
    M.PIN_REGISTRY["worktree-parallelism"].scheme === "narrative-sections",
    "worktree-parallelism declares the narrative-sections scheme",
    M.PIN_REGISTRY["worktree-parallelism"].scheme,
  );
  ok(
    (wt.anchors?.all || []).every((a) => new RegExp(`^${M.NARRATIVE_ANCHOR_PATTERN}$`).test(a)),
    "every derived narrative anchor matches the pattern the stub map and the claim matcher read",
    wt.anchors?.all,
  );
  ok(
    (wt.anchors?.all || []).length === new Set(wt.anchors?.all || []).size,
    "every derived narrative anchor id is DISTINCT (the f2-10 collision hazard, closed by construction)",
    wt.anchors?.all,
  );

  const cli = runCli(["--check", "worktree-parallelism"]);
  ok(cli.code === 0, "`--check worktree-parallelism` exits 0 under the narrative-sections scheme", `exit ${cli.code}: ${cli.err}${cli.out}`);
  ok(/^PASS okf_migrate --check worktree-parallelism: /m.test(cli.out), "the chain PASS wording names worktree-parallelism", cli.out);

  const pins = runCli(["--verify-pins"]);
  ok(pins.code === 0, "`--verify-pins` is green across all ten pins", `exit ${pins.code}: ${pins.err}${pins.out}`);
  ok(
    !/SKIP-REFUSED/.test(pins.out),
    "no pin is skipped any more — every registered area is gateable (f2-11 closed the last refusal)",
    pins.out,
  );
}

// ─── 29. telemetry comparability is keyed on SHAPE, and that is a no-op ─────
//
// f2-3 restricted the F12 median to pins of the same shape after the
// flat-pattern-list migration (one anchor IS one concept by construction)
// reported permanent "drift" against nine-section areas that no cell had
// touched. `narrative-sections` is a third shape with the opposite skew —
// whole SECTIONS are the anchors, so a 225-line source yields 10 of them where
// a nine-section area of the same length yields 23 — and pooling it would
// repeat exactly that defect. The comparability key is therefore the pin's
// SCHEME, which is what "same shape" has always meant. Proven a strict no-op:
// the eight ba-nine-section areas group exactly as the eight `kind: "area"`
// pins did, and critical-patterns stays alone exactly as `kind: "patterns"`
// did.

{
  const rows = await M.collectTelemetry();
  ok(rows.every((r) => typeof r.scheme === "string"), "every telemetry row carries its pin's scheme", rows.map((r) => r.scheme));
  const byScheme = (s) => rows.filter((r) => r.scheme === s).map((r) => r.area).sort();
  const byKind = (k) => rows.filter((r) => (r.kind || "area") === k).map((r) => r.area).sort();
  eq(
    byScheme("ba-nine-section"),
    byKind("area").filter((a) => a !== "worktree-parallelism"),
    "STRICT NO-OP: grouping by scheme reproduces the previous `kind: area` grouping for every pre-existing pin",
  );
  eq(byScheme("flat-pattern-list"), byKind("patterns"), "STRICT NO-OP: critical-patterns is alone under either key");
  eq(byScheme("narrative-sections"), ["worktree-parallelism"], "the third shape is its own comparison population");
  ok(
    byScheme("narrative-sections").length < 3,
    "with one narrative-sections pin there is no median yet, so its telemetry REPORTS ONLY — never fails on a coin flip",
    byScheme("narrative-sections").length,
  );
}

// ─── 30. workflow-state: THREE duplicate ids repaired before pinning (f2-12) ─
//
// The second repaired pin, and the worse case. docs/specs/workflow-state.md
// shipped R19, R20 AND R21 twice each inside one `## Business Rules` section:
// the fresh-session-handoff triple (planned-next preconditions / auto-resume
// authority / the work puller) and the chain-integrity triple (learning
// capture is never settable / the sync demands executed work / the terminal
// state demands zero spec debt). Six genuinely distinct rules, three
// collisions, so 140 anchors carried only 137 distinct ids — and the counts
// added up perfectly the whole time, which is exactly why a count can never
// prove this property. The SECOND occurrence of each id in document order was
// renumbered R19a/R20a/R21a before the pin was captured.
//
// Asserted here from BOTH sides, as f2-10's single repair is: the property the
// repair creates (every derived id distinct, all six rules individually
// measurable), the property it repaired (the PROVENANCE blob really does carry
// 140 anchors with 137 distinct ids — measured, never asserted), and the
// refusals that stop the branch from becoming a way to launder drift.

{
  const real = M.PIN_REGISTRY["workflow-state"];
  ok(
    typeof real.repaired_from === "string" && /^[0-9a-f]{40}$/.test(real.repaired_from),
    "the workflow-state pin names the provenance blob it was repaired from",
    real.repaired_from,
  );
  ok(
    typeof real.repair_reason === "string" &&
      ["R19a", "R20a", "R21a"].every((id) => real.repair_reason.includes(id)),
    "the workflow-state pin states the repair in prose, naming all three ids it introduced",
    real.repair_reason?.slice(0, 120),
  );

  const derived = await M.derivePinForArea("workflow-state");
  ok(derived.ok, "the repaired workflow-state pin derives green", derived.issues);
  ok(
    derived.via === "committed-copy",
    "a repaired pin resolves via its committed copy — the resolvable blob is the PROVENANCE, never the pinned source",
    derived.via,
  );

  const all = derived.anchors?.all || [];
  ok(all.length === 140, "workflow-state derives 140 anchors from the repaired blob", all.length);
  ok(
    new Set(all).size === all.length,
    "every derived workflow-state anchor id is DISTINCT — the property three collisions denied, and the one a count can never prove",
    `${all.length} anchors, ${new Set(all).size} distinct`,
  );
  for (const id of ["R19", "R20", "R21", "R19a", "R20a", "R21a"]) {
    ok(all.includes(id), `${id} survives as its own anchor — no rule was dropped or merged to remove a duplicate`, all.filter((a) => a.startsWith(id.slice(0, 3))));
    const text = derived.anchors?.texts?.get(id);
    ok(
      typeof text === "string" && text.length > 0,
      `${id} has its own extracted text, so the F11 floor can measure it individually`,
      typeof text,
    );
  }
  const t = (id) => derived.anchors.texts.get(id) || "";
  ok(
    /planned-next handoff/.test(t("R19")) && /learning-capture phase/.test(t("R19a")),
    "the R19 pair is the two DIFFERENT rules (planned-next preconditions vs. the unsettable learning-capture phase), not one rule read twice",
  );
  ok(
    /fresh-session boundary/.test(t("R20")) && /knowledge sync/.test(t("R20a")),
    "the R20 pair is the two DIFFERENT rules (auto-resume authority vs. recording a knowledge sync)",
  );
  ok(
    /work puller/.test(t("R21")) && /terminal state/.test(t("R21a")),
    "the R21 pair is the two DIFFERENT rules (the work puller's authority vs. entering the terminal state)",
  );

  // The defect, measured on the PROVENANCE bytes rather than described. This
  // is what makes "the repair was necessary" evidence instead of a claim.
  const prov = spawnSync("git", ["cat-file", "blob", real.repaired_from], {
    cwd: REPO_ROOT,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  if (ok(prov.status === 0, "the provenance blob is readable from git", prov.stderr)) {
    const before = M.inventorySpec(prov.stdout);
    ok(
      before.all.length === 140 && new Set(before.all).size === 137,
      "the PRE-repair source really did carry 140 anchors with only 137 distinct ids — three ids collapsed, and every count still added up",
      `${before.all.length} anchors, ${new Set(before.all).size} distinct`,
    );
    eq(
      ["R19", "R20", "R21"],
      [...new Set(before.all.filter((a, i) => before.all.indexOf(a) !== i))].sort(),
      "the three collided ids were exactly R19, R20 and R21",
    );
  }

  // The launder-proofing, identical in form to section 15's.
  const undeclared = { ...real };
  delete undeclared.repaired_from;
  delete undeclared.repair_reason;
  const rU = await M.derivePin(undeclared, "workflow-state");
  ok(rU.ok === false, "the same pin WITHOUT a declared repair is REFUSED — a disagreement is never inferred to be one");
  ok(codes(rU).includes("PIN_SHA_MISMATCH"), "an undeclared disagreement reports PIN_SHA_MISMATCH", codes(rU));

  const wrongProvenance = { ...real, repaired_from: "0".repeat(40) };
  const rW = await M.derivePin(wrongProvenance, "workflow-state");
  ok(rW.ok === false, "a repair naming the WRONG provenance blob is REFUSED — drifting provenance is as loud as a drifting pin");
  ok(codes(rW).includes("PIN_SHA_MISMATCH"), "a wrong repaired_from reports PIN_SHA_MISMATCH", codes(rW));

  const noReason = { ...real, repair_reason: "   " };
  const rR = await M.derivePin(noReason, "workflow-state");
  ok(rR.ok === false, "a repair with no stated reason is REFUSED — the repair must be readable, not merely flagged");
  ok(codes(rR).includes("PIN_SHA_MISMATCH"), "an unexplained repair reports PIN_SHA_MISMATCH", codes(rR));

  // STRICT NO-OP: adding an eleventh pin moves no pin that existed before it.
  const pins = runCli(["--verify-pins"]);
  ok(pins.code === 0, "`--verify-pins` is green across all eleven pins", `exit ${pins.code}: ${pins.err}${pins.out}`);
  ok(!/SKIP-REFUSED/.test(pins.out), "no pin is skipped — every registered area is still gateable", pins.out);
  ok(
    /PASS workflow-state: 140 anchors/.test(pins.out),
    "`--verify-pins` reports workflow-state's own 140 anchors",
    pins.out.split("\n").filter((l) => /workflow-state/.test(l)).join(" | "),
  );
}

// ─── done ───────────────────────────────────────────────────────────────────

fs.rmSync(TMP, { recursive: true, force: true });

if (failures > 0) {
  console.error(`\nFAIL test_okf_pins: ${failures} of ${checks} assertions failed`);
  process.exit(1);
}
console.log(`PASS test_okf_pins: ${checks} assertions — pins are content-addressed and fully asserted (sha, scheme, counts), empty/mismatched/unresolvable/unscheme'd extractions all exit 1, the committed-source fallback verifies via blob hash, onboarding.md's unparsed-block count is non-zero (format-blindness is visible), and the three schemes stay strict no-ops on each other's pins (narrative-sections derives a source's own "## " headings, refuses a heading-less source and a slug collision, and never touches the nine numbered pins)`);
