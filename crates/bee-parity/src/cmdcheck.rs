//! `--cmd-check`: the GENERIC command-parity arm (rpl-1).
//!
//! `--status-check` proves one command. This arm proves *any* command: a
//! scenario is an argv list, and every scenario runs the SAME argv through
//! both runtimes over independent clones of one `queen-bench --generate`
//! fixture, diffing stdout, stderr, exit code and the resulting store tree.
//! Every ledger cell in this slice (`rpl-2` … `rpl-10`) registers its group's
//! scenarios here.
//!
//! # Four properties this arm has to have, and why
//!
//! **1. The verify must DISCRIMINATE (`--group`).** A bare `--cmd-check`
//! would exit 0 on the seam's own smoke scenarios alone, so a later worker
//! who ported a group and registered ZERO scenarios would still get green.
//! There is no bare mode: a selector is mandatory, `--cmd-check --group
//! <name>` runs only that group and EXITS NON-ZERO when the group has
//! registered nothing, and the registered count per group is printed on
//! every run so the number is visible in the verify output.
//!
//! **2. The negative control is PER-SCENARIO.** See
//! [`crate::mutate::MutationTarget`]: a `state.json` `phase` flip cannot fire
//! for any ledger verb, because no ledger verb prints `phase`. Each scenario
//! declares the store it actually reads and how to perturb it, plus WHICH
//! output channel the control must fire on — asserting on the channel, not
//! merely on "some diff", is what keeps a tree-only detection from passing
//! for an output comparison that has been normalized into silence. A
//! scenario with no declared mutation target is REFUSED at registration.
//!
//! **3. Per-scenario SEEDING rides on top of `--generate`.** The generated
//! fixture is the authority for the BASELINE store and is never hand-
//! authored — but its ledger content is monotone filler: every decisions row
//! is type `decide` with an identical date and no tags
//! (`queen-bench/src/fixture.rs:287`), every backlog row is type `proposal`
//! (`:296`), ids are zero-padded so byte order equals numeric order, and
//! there is no capture queue, intent store, decisions archive or lanes
//! directory at all. A scenario asserting anything about a superseded,
//! redacted, non-ASCII, tagged, dual-schema or non-zero-padded row would be
//! non-discriminating against that store. So a scenario MAY seed additional
//! rows onto its clone — additive only, on top of `--generate`, never
//! instead of it.
//!
//! **4. STDERR is in the diff surface.** Half the refusal texts this slice
//! must reproduce never touch stdout at all (`bee.mjs:6985` `emitError`
//! writes to stderr unless `--json` was asked for). See
//! [`crate::runner`] and [`crate::normalize::strip_runtime_stderr_artifacts`].

use std::path::Path;

use crate::mutate::{self, MutationTarget};
use crate::{clone, differ, normalize, rootsafety, runner};

/// The groups this arm knows about. A `--group` outside this set is an
/// error, not an empty run — a typo'd group name must never read as "zero
/// scenarios, nothing to do".
pub const KNOWN_GROUPS: &[&str] = &["seam", "intent", "capture", "decisions", "backlog", "reviews", "feedback"];

/// Which output channel a scenario's negative control must fire on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Channel {
    Stdout,
    Stderr,
}

impl Channel {
    fn label(self) -> &'static str {
        match self {
            Channel::Stdout => "stdout",
            Channel::Stderr => "stderr",
        }
    }
}

/// A positive-content assertion. A zero diff between two EMPTY outputs is not
/// parity, so every scenario must state something its output really contains
/// (or exactly equals) on BOTH legs.
#[derive(Clone, Copy)]
pub struct Assertion {
    pub channel: Channel,
    pub kind: AssertKind,
    pub text: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AssertKind {
    /// The channel's NORMALIZED content must equal this exactly.
    Equals,
    /// The channel's NORMALIZED content must contain this.
    Contains,
}

/// Additive seeding applied to every clone of a scenario, after the golden
/// copy and before any run.
pub type SeedFn = fn(&Path) -> Result<(), String>;

/// One command-parity scenario.
pub struct Scenario {
    pub group: &'static str,
    pub name: &'static str,
    pub argv: Vec<String>,
    /// Pins `resolveSessionId`'s env chain for both runtimes.
    pub session_id: Option<&'static str>,
    pub seed: Option<SeedFn>,
    /// REQUIRED. See [`ScenarioSet::register`].
    pub mutation: Option<MutationTarget>,
    pub control_channel: Channel,
    pub expect_exit: i32,
    pub assertions: Vec<Assertion>,
}

impl Scenario {
    pub fn label(&self) -> String {
        format!("{}/{}", self.group, self.name)
    }
}

/// The registration table for scenarios.
#[derive(Default)]
pub struct ScenarioSet {
    scenarios: Vec<Scenario>,
}

impl ScenarioSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// REFUSES a scenario with no declared mutation target, an unknown
    /// group, an empty argv, or no positive-content assertion.
    ///
    /// The mutation refusal is the load-bearing one: silently skipping the
    /// negative control for a scenario that did not declare one would let a
    /// whole group's parity rest on a comparison nobody ever proved could
    /// fail.
    pub fn register(&mut self, scenario: Scenario) -> Result<(), String> {
        if !KNOWN_GROUPS.contains(&scenario.group) {
            return Err(format!(
                "scenario `{}` declares unknown group `{}` (known: {})",
                scenario.name,
                scenario.group,
                KNOWN_GROUPS.join(", ")
            ));
        }
        if scenario.mutation.is_none() {
            return Err(format!(
                "scenario `{}` declares NO mutation target — refusing to register it. Every scenario must name the store it reads and how to perturb it, or its negative control cannot fire and its zero-diff result proves nothing.",
                scenario.label()
            ));
        }
        if scenario.argv.is_empty() {
            return Err(format!("scenario `{}` has an empty argv", scenario.label()));
        }
        if scenario.assertions.is_empty() {
            return Err(format!(
                "scenario `{}` declares no positive-content assertion — a zero diff between two empty outputs is not parity",
                scenario.label()
            ));
        }
        if self.scenarios.iter().any(|s| s.group == scenario.group && s.name == scenario.name) {
            return Err(format!("scenario `{}` is already registered", scenario.label()));
        }
        self.scenarios.push(scenario);
        Ok(())
    }

    pub fn count_for(&self, group: &str) -> usize {
        self.scenarios.iter().filter(|s| s.group == group).count()
    }

    pub fn len(&self) -> usize {
        self.scenarios.len()
    }

    pub fn is_empty(&self) -> bool {
        self.scenarios.is_empty()
    }

    pub fn select(&self, group: Option<&str>) -> Vec<&Scenario> {
        self.scenarios
            .iter()
            .filter(|s| group.map(|g| s.group == g).unwrap_or(true))
            .collect()
    }

    /// The per-group registered-scenario table, printed on every run so the
    /// number a verify actually exercised is visible in its own output.
    pub fn render_counts(&self) -> String {
        let width = KNOWN_GROUPS.iter().map(|g| g.len()).max().unwrap_or(0);
        let mut lines = vec!["registered scenarios by group:".to_string()];
        for group in KNOWN_GROUPS {
            lines.push(format!("  {group:<width$}  {}", self.count_for(group), width = width));
        }
        lines.join("\n")
    }
}

// ─── seeding helpers shared by scenarios ───────────────────────────────────

fn write_file(path: &Path, body: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create_dir_all {}: {e}", parent.display()))?;
    }
    std::fs::write(path, body).map_err(|e| format!("write {}: {e}", path.display()))
}

// ─── the seam's own scenarios ──────────────────────────────────────────────

const REGISTRY_DUMP: &str = ".bee/cache/command-registry.json";

/// The lane feature slug the numeric-key scenario seeds. The lane FILE stem,
/// the lane id and the feature are deliberately the same string — that is
/// the shape `buildLaneSummary` resolves an ACTIVE lane through, mirroring
/// [`crate::enrich`]'s own lane record.
const NUMKEY_FEATURE: &str = "rpl1-numeric-keys";
const NUMKEY_SESSION: &str = "rpl1-numeric-keys-session";

/// Seed a lane record whose `approved_gates` carries NUMERIC-STRING KEYS,
/// plus a live session bound to it so `buildLaneSummary` resolves it as the
/// ACTIVE lane and serializes the record IN FULL.
///
/// This is a row `queen-bench --generate` never produces — the generated
/// fixture has no `.bee/lanes/` directory at all — which is exactly why
/// per-scenario seeding has to be allowed (obligation (D)).
///
/// It is also the ONLY live proof available for the numeric-string-key
/// question (obligation (c)): `JSON.stringify` hoists integer-like keys into
/// ascending numeric order ahead of every string key, while `serde_json` with
/// `preserve_order` emits insertion order. Measured against the frozen
/// oracle before `queen_bee::jsonout` existed, this seed produced
/// `{"1","2","10","context","shape","execution","review","zeta"}` from mjs and
/// `{"context","shape","execution","review","10","2","zeta","1"}` from
/// `queen-bee` — a real byte-compatibility break in an already-ported
/// command, not a hypothetical.
fn seed_numeric_string_keys(root: &Path) -> Result<(), String> {
    let bee = root.join(".bee");
    write_file(
        &bee.join("lanes").join(format!("{NUMKEY_FEATURE}.json")),
        &format!(
            "{{\"schema_version\":\"1.0\",\"phase\":\"scribing\",\"feature\":\"{NUMKEY_FEATURE}\",\"mode\":\"tiny\",\"approved_gates\":{{\"10\":true,\"2\":false,\"zeta\":true,\"1\":true}},\"summary\":\"\",\"next_action\":\"\",\"created_at\":null}}"
        ),
    )?;
    let heartbeat = crate::enrich::iso_now_millis();
    write_file(
        &bee.join("sessions").join(format!("{NUMKEY_SESSION}.json")),
        &format!(
            "{{\"id\":\"{NUMKEY_SESSION}\",\"started_at\":\"{heartbeat}\",\"last_heartbeat\":\"{heartbeat}\",\"lane\":\"{NUMKEY_FEATURE}\"}}"
        ),
    )?;
    Ok(())
}

fn mutate_numeric_lane(root: &Path) -> Result<(), String> {
    mutate::replace_exactly_once(
        root,
        &format!(".bee/lanes/{NUMKEY_FEATURE}.json"),
        "\"phase\":\"scribing\"",
        "\"phase\":\"rpl1-seeded-mutation\"",
    )
}

/// Rename one registry entry so the nearest-match SUGGESTION changes.
/// `reservations.list` is what the unmutated registry suggests for
/// `zzznotagroup.list`; once it is gone under that name, whatever the
/// nearest match becomes, it is not `reservations.list`.
fn mutate_registry_entry_name(root: &Path) -> Result<(), String> {
    mutate::replace_exactly_once(
        root,
        REGISTRY_DUMP,
        "\"name\": \"reservations.list\"",
        "\"name\": \"reservations.lisz\"",
    )
}

/// Rename the `cells.show` entry so the `--help --json` manifest changes.
fn mutate_registry_help_payload(root: &Path) -> Result<(), String> {
    mutate::replace_exactly_once(root, REGISTRY_DUMP, "\"name\": \"cells.show\"", "\"name\": \"cells.shzw\"")
}

/// The `cells.show` entry's own header, up to and including the opening of
/// its `properties` object. Anchored across several lines deliberately: the
/// inner `"description": "Cell id, …"` line alone is NOT unique in the dump,
/// and a mutation that lands on the wrong entry proves nothing about the
/// scenario that declared it.
const CELLS_SHOW_PROPERTIES_ANCHOR: &str = concat!(
    "\"name\": \"cells.show\",\n",
    "      \"invoke\": \"bee cells show\",\n",
    "      \"description\": \"Show one cell by id, including its full trace.\",\n",
    "      \"parameters\": {\n",
    "        \"type\": \"object\",\n",
    "        \"properties\": {"
);

/// Inject an extra property into `cells.show`'s schema so the KNOWN-FLAG
/// list in the unknown-flag refusal changes. That refusal is stderr-only, so
/// this mutation can only be seen by a harness that diffs stderr.
fn mutate_registry_known_flags(root: &Path) -> Result<(), String> {
    mutate::replace_exactly_once(
        root,
        REGISTRY_DUMP,
        CELLS_SHOW_PROPERTIES_ANCHOR,
        &format!(
            "{CELLS_SHOW_PROPERTIES_ANCHOR}\n          \"zzz-injected\": {{\n            \"type\": \"string\"\n          }},"
        ),
    )
}

/// Rewrite `cells.show`'s first example so the ` Example: …` tail of the
/// missing-required-flag refusal changes.
fn mutate_registry_example(root: &Path) -> Result<(), String> {
    mutate::replace_exactly_once(
        root,
        REGISTRY_DUMP,
        "bee cells show --id demo-1 --json",
        "bee cells show --id mutated-1 --json",
    )
}

fn argv(tokens: &[&str]) -> Vec<String> {
    tokens.iter().map(|s| (*s).to_string()).collect()
}

/// Build the full scenario table. Later cells add their group's scenarios
/// here; `rpl-1` registers only the `seam` group.
pub fn all_scenarios() -> Result<ScenarioSet, String> {
    let mut set = ScenarioSet::new();

    // ── smoke 1: the enumeration surface, byte-for-byte over 116 entries ──
    set.register(Scenario {
        group: "seam",
        name: "help-json",
        argv: argv(&["--help", "--json"]),
        session_id: None,
        seed: None,
        mutation: Some(MutationTarget { store: REGISTRY_DUMP, apply: mutate_registry_help_payload }),
        control_channel: Channel::Stdout,
        expect_exit: 0,
        assertions: vec![
            Assertion { channel: Channel::Stdout, kind: AssertKind::Contains, text: "\"schema_version\": \"1.0\"" },
            Assertion { channel: Channel::Stdout, kind: AssertKind::Contains, text: "\"name\": \"cells.show\"" },
            Assertion { channel: Channel::Stdout, kind: AssertKind::Contains, text: "\"name\": \"feedback.rank\"" },
            Assertion { channel: Channel::Stderr, kind: AssertKind::Equals, text: "" },
        ],
    })?;

    // ── smoke 2: an unknown-group refusal, on stdout, exit 1 ─────────────
    set.register(Scenario {
        group: "seam",
        name: "unknown-group-refusal",
        argv: argv(&["zzznotagroup", "list"]),
        session_id: None,
        seed: None,
        mutation: Some(MutationTarget { store: REGISTRY_DUMP, apply: mutate_registry_entry_name }),
        control_channel: Channel::Stdout,
        expect_exit: 1,
        assertions: vec![
            Assertion {
                channel: Channel::Stdout,
                kind: AssertKind::Equals,
                text: "Unknown command \"zzznotagroup.list\". Did you mean \"reservations.list\"?\n",
            },
            Assertion { channel: Channel::Stderr, kind: AssertKind::Equals, text: "" },
        ],
    })?;

    // ── the unknown-FLAG refusal: registry-derived, and STDERR-ONLY ──────
    //
    // must_have: "unknown-flag ... refusal text is byte-identical to mjs for
    // at least one real command, proven by a --cmd-check scenario" AND
    // "runner::RunResult captures stderr and differ::diff_legs diffs it,
    // proven by a scenario whose only difference would be on stderr". This
    // scenario is both: its stdout is EMPTY on both legs, so a stdout-only
    // harness would have declared it parity without comparing a single byte
    // that matters — and its negative control fires on stderr too.
    set.register(Scenario {
        group: "seam",
        name: "unknown-flag-refusal-stderr",
        argv: argv(&["cells", "show", "--id", "foo", "--bogus", "x"]),
        session_id: None,
        seed: None,
        mutation: Some(MutationTarget { store: REGISTRY_DUMP, apply: mutate_registry_known_flags }),
        control_channel: Channel::Stderr,
        expect_exit: 1,
        assertions: vec![
            Assertion { channel: Channel::Stdout, kind: AssertKind::Equals, text: "" },
            Assertion {
                channel: Channel::Stderr,
                kind: AssertKind::Equals,
                text: "cells show: unknown flag --bogus (known: help, id, json).\n",
            },
        ],
    })?;

    // ── the missing-required-flag refusal, on stdout, with its Example tail ─
    set.register(Scenario {
        group: "seam",
        name: "missing-required-flag-refusal",
        argv: argv(&["cells", "show"]),
        session_id: None,
        seed: None,
        mutation: Some(MutationTarget { store: REGISTRY_DUMP, apply: mutate_registry_example }),
        control_channel: Channel::Stdout,
        expect_exit: 1,
        assertions: vec![
            Assertion {
                channel: Channel::Stdout,
                kind: AssertKind::Equals,
                text: "Invalid call to \"cells.show\": required, missing (--id). Example: bee cells show --id demo-1 --json\n",
            },
            Assertion { channel: Channel::Stderr, kind: AssertKind::Equals, text: "" },
        ],
    })?;

    // ── numeric-string object keys, over a SEEDED row ────────────────────
    set.register(Scenario {
        group: "seam",
        name: "numeric-string-keys",
        argv: argv(&["status", "--json"]),
        session_id: Some(NUMKEY_SESSION),
        seed: Some(seed_numeric_string_keys),
        mutation: Some(MutationTarget {
            store: ".bee/lanes/rpl1-numeric-keys.json",
            apply: mutate_numeric_lane,
        }),
        control_channel: Channel::Stdout,
        expect_exit: 0,
        assertions: vec![
            // The seeded lane must have REACHED the payload, or the key
            // ordering this scenario exists to pin was never serialized.
            Assertion { channel: Channel::Stdout, kind: AssertKind::Contains, text: "\"zeta\": true" },
            Assertion { channel: Channel::Stdout, kind: AssertKind::Contains, text: "\"10\": true" },
            Assertion {
                channel: Channel::Stdout,
                kind: AssertKind::Contains,
                text: "\"feature\": \"rpl1-numeric-keys\"",
            },
            Assertion { channel: Channel::Stderr, kind: AssertKind::Equals, text: "" },
        ],
    })?;

    Ok(set)
}

// ─── the arm itself ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selector {
    pub group: Option<String>,
}

/// Parse `--cmd-check`'s own flags. A selector is MANDATORY: there is no
/// bare mode that could quietly pass on somebody else's scenarios.
pub fn parse_selector(args: &[String]) -> Result<Selector, String> {
    let mut group: Option<String> = None;
    let mut all = false;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--all" => all = true,
            "--group" => {
                i += 1;
                group = Some(
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| "--group requires a value".to_string())?,
                );
            }
            other => return Err(format!("unknown --cmd-check flag `{other}` (expected --all or --group <name>)")),
        }
        i += 1;
    }
    match (all, group) {
        (true, Some(_)) => Err("--all and --group are mutually exclusive".to_string()),
        (true, None) => Ok(Selector { group: None }),
        (false, Some(g)) => {
            if !KNOWN_GROUPS.contains(&g.as_str()) {
                return Err(format!("unknown group `{g}` (known: {})", KNOWN_GROUPS.join(", ")));
            }
            Ok(Selector { group: Some(g) })
        }
        (false, None) => Err(
            "--cmd-check requires a selector: `--all`, or `--group <name>` for one group. A bare run would exit 0 on whatever scenarios somebody ELSE registered, which is not a verify."
                .to_string(),
        ),
    }
}

/// Run one scenario end to end: parity plus its own negative control.
#[allow(clippy::too_many_arguments)]
pub fn check_one_scenario(
    repo_root: &Path,
    bins: &runner::Binaries,
    golden: &Path,
    scenario: &Scenario,
    track: &mut dyn FnMut(std::path::PathBuf) -> std::path::PathBuf,
) -> Result<String, String> {
    let label = scenario.label();
    let slug = format!("{}-{}", scenario.group, scenario.name);
    let root_mjs = track(fresh_dir(&format!("cmd-{slug}-mjs")));
    let root_rust = track(fresh_dir(&format!("cmd-{slug}-rust")));
    let root_mutated = track(fresh_dir(&format!("cmd-{slug}-mutated")));

    for dir in [&root_mjs, &root_rust, &root_mutated] {
        crate::assert_temp_outside_repo(repo_root, dir)?;
        clone::copy_tree(golden, dir)?;
        if let Some(seed) = scenario.seed {
            seed(dir).map_err(|e| format!("[{label}] seeding {}: {e}", dir.display()))?;
        }
    }

    rootsafety::assert_structural_safety(repo_root, &root_mjs)?;
    let run_mjs = runner::run_argv(bins, runner::Runtime::Mjs, &scenario.argv, &root_mjs, scenario.session_id)?;
    rootsafety::assert_structural_safety(repo_root, &root_rust)?;
    let run_rust =
        runner::run_argv(bins, runner::Runtime::QueenBee, &scenario.argv, &root_rust, scenario.session_id)?;

    // Exit codes are checked against the DECLARED value, independently of
    // "diff empty" — two identical failures are not parity, and two
    // identical successes at the wrong exit code are not either.
    for (who, run) in [("mjs", &run_mjs), ("queen-bee", &run_rust)] {
        if run.exit_code != scenario.expect_exit {
            return Err(format!(
                "[{label}] {who} exited {} but the scenario declares {} — argv `{}`\nstdout:\n{}\nstderr:\n{}",
                run.exit_code,
                scenario.expect_exit,
                scenario.argv.join(" "),
                crate::truncate(&run.stdout),
                crate::truncate(&run.stderr)
            ));
        }
    }

    // Positive content, on BOTH legs.
    for (who, run) in [("mjs", &run_mjs), ("queen-bee", &run_rust)] {
        assert_content(&label, who, run, &scenario.assertions)?;
    }

    let parity = differ::diff_legs(&run_mjs, &run_rust)?;
    if !parity.is_clean() {
        return Err(format!(
            "[{label}] argv `{}` — mjs vs queen-bee reported a diff: {}",
            run_mjs.argv.join(" "),
            parity.describe()
        ));
    }

    // PER-SCENARIO NEGATIVE CONTROL. The mutation perturbs the store THIS
    // scenario reads, and the control must fire on the channel this scenario
    // declares — never merely "somewhere".
    let mutation = scenario
        .mutation
        .ok_or_else(|| format!("[{label}] has no mutation target (registration should have refused it)"))?;
    (mutation.apply)(&root_mutated).map_err(|e| format!("[{label}] seeding the mutation: {e}"))?;
    crate::assert_temp_outside_repo(repo_root, &root_mutated)?;
    rootsafety::assert_structural_safety(repo_root, &root_mutated)?;
    let run_mutated =
        runner::run_argv(bins, runner::Runtime::QueenBee, &scenario.argv, &root_mutated, scenario.session_id)?;
    let mutation_diff = differ::diff_legs(&run_mjs, &run_mutated)?;
    let fired = match scenario.control_channel {
        Channel::Stdout => mutation_diff.stdout_diff.is_some(),
        Channel::Stderr => mutation_diff.stderr_diff.is_some(),
    };
    if !fired {
        return Err(format!(
            "[{label}] argv `{}` — seeded-mutation check FAILED: perturbing {} produced ZERO {} diff (channels that DID move: {}), so this scenario's {} comparison cannot detect a real divergence and its zero-diff parity result above cannot be trusted. Full mutation diff: {}",
            scenario.argv.join(" "),
            mutation.store,
            scenario.control_channel.label(),
            mutation_diff.output_channels_differing(),
            scenario.control_channel.label(),
            mutation_diff.describe()
        ));
    }

    Ok(format!(
        "[{label}] argv `{}` — zero diff (stdout {} B, stderr {} B, exit {}); control fired on {} via {}",
        scenario.argv.join(" "),
        run_mjs.stdout.len(),
        run_mjs.stderr.len(),
        run_mjs.exit_code,
        scenario.control_channel.label(),
        mutation.store
    ))
}

fn assert_content(
    label: &str,
    who: &str,
    run: &differ::RunResult,
    assertions: &[Assertion],
) -> Result<(), String> {
    let root_str = run.root.display().to_string();
    let stdout = normalize::normalize(&run.stdout, &root_str);
    let stderr = normalize::normalize_stderr(&run.stderr, &root_str, run.runtime)
        .map_err(|e| format!("[{label}] {who}: {e}"))?;
    for a in assertions {
        let actual = match a.channel {
            Channel::Stdout => &stdout,
            Channel::Stderr => &stderr,
        };
        let ok = match a.kind {
            AssertKind::Equals => actual == a.text,
            AssertKind::Contains => actual.contains(a.text),
        };
        if !ok {
            let verb = if a.kind == AssertKind::Equals { "equal" } else { "contain" };
            return Err(format!(
                "[{label}] {who}: {} did not {verb} the declared content.\n--- expected ---\n{}\n--- actual ---\n{}",
                a.channel.label(),
                a.text,
                crate::truncate(actual)
            ));
        }
    }
    Ok(())
}

fn fresh_dir(label: &str) -> std::path::PathBuf {
    crate::fresh_temp_dir(label)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_mutation() -> MutationTarget {
        fn noop(_: &Path) -> Result<(), String> {
            Ok(())
        }
        MutationTarget { store: ".bee/state.json", apply: noop }
    }

    fn base(name: &'static str) -> Scenario {
        Scenario {
            group: "seam",
            name,
            argv: argv(&["status", "--json"]),
            session_id: None,
            seed: None,
            mutation: Some(dummy_mutation()),
            control_channel: Channel::Stdout,
            expect_exit: 0,
            assertions: vec![Assertion { channel: Channel::Stdout, kind: AssertKind::Contains, text: "{" }],
        }
    }

    #[test]
    fn registration_refuses_a_scenario_with_no_mutation_target() {
        let mut set = ScenarioSet::new();
        let mut s = base("no-control");
        s.mutation = None;
        let err = set.register(s).unwrap_err();
        assert!(err.contains("declares NO mutation target"), "{err}");
        assert!(set.is_empty(), "a refused scenario must not be registered");
    }

    #[test]
    fn registration_refuses_an_unknown_group() {
        let mut set = ScenarioSet::new();
        let mut s = base("bad-group");
        s.group = "nope";
        assert!(set.register(s).unwrap_err().contains("unknown group"));
    }

    #[test]
    fn registration_refuses_a_scenario_with_no_positive_assertion() {
        let mut set = ScenarioSet::new();
        let mut s = base("no-assert");
        s.assertions = vec![];
        assert!(set.register(s).unwrap_err().contains("no positive-content assertion"));
    }

    #[test]
    fn registration_refuses_a_duplicate() {
        let mut set = ScenarioSet::new();
        set.register(base("dup")).unwrap();
        assert!(set.register(base("dup")).unwrap_err().contains("already registered"));
    }

    #[test]
    fn a_bare_cmd_check_is_refused_outright() {
        let err = parse_selector(&[]).unwrap_err();
        assert!(err.contains("requires a selector"), "{err}");
    }

    #[test]
    fn an_unknown_group_selector_is_refused_not_treated_as_empty() {
        let err = parse_selector(&argv(&["--group", "nosuchgroup"])).unwrap_err();
        assert!(err.contains("unknown group"), "{err}");
    }

    #[test]
    fn all_and_group_are_mutually_exclusive() {
        assert!(parse_selector(&argv(&["--all", "--group", "seam"])).is_err());
    }

    #[test]
    fn every_ledger_group_still_has_zero_scenarios_registered() {
        // rpl-1 is the seam only. This is also the pin that makes
        // `--cmd-check --group intent` (rpl-2's verify) fail TODAY, which is
        // the whole point of obligation (A).
        let set = all_scenarios().expect("the shipped scenario table registers cleanly");
        for group in KNOWN_GROUPS.iter().filter(|g| **g != "seam") {
            assert_eq!(set.count_for(group), 0, "group {group} should have no scenarios yet");
        }
        assert!(set.count_for("seam") >= 5, "the seam must carry its own smoke scenarios");
    }

    #[test]
    fn the_counts_table_names_every_known_group() {
        let set = all_scenarios().unwrap();
        let rendered = set.render_counts();
        for group in KNOWN_GROUPS {
            assert!(rendered.contains(group), "counts table must name {group}: {rendered}");
        }
    }
}
