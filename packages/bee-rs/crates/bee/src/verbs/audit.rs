use crate::audit;
use crate::verbs::decisions::{decisions_prelude, feature_for_stamp};
use crate::verbs::reservations::{finish, keys_known, parse_flags, Err2, Flags, Out, Pre};
use crate::verbs::state_group::resolve_mutation_target;
use crate::verbs::state_group::Target;
use serde_json::json;
use std::ffi::OsString;
use std::process::ExitCode;
use std::time::Instant;

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "audit" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    let toks: Vec<&str> =
        args[2..].iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    let (flags, use_json) = parse_flags(&toks)?;
    match verb {
        "log" => run_log(flags, use_json, t0),
        "list" => run_list(flags, use_json, t0),
        _ => None,
    }
}

fn run_log(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["kind", "feature", "note", "cell"]) {
        return None;
    }
    let ctx = match decisions_prelude("audit log", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    let out = (|| {
        let Some(kind) = flags.truthy_str("kind") else {
            return Ok(Out::Thrown(
                "audit log: --kind is required (bee audit log --kind <kind> --feature <f> --note \"<text>\").".into(),
            ));
        };
        if !audit::KINDS.contains(&kind) {
            return Ok(Out::Thrown(audit::unknown_kind_message("audit log", kind)));
        }
        let Some(note) = flags.truthy_str("note") else {
            return Ok(Out::Thrown("audit log: --note is required.".into()));
        };
        let feature = match flags.truthy_str("feature") {
            Some(f) => f.to_string(),
            None => {
                let target = resolve_mutation_target(&ctx.root, None, "audit log", false)?;
                let resolved = match &target {
                    Target::Default { target_feature, .. } => target_feature.clone(),
                    lane => feature_for_stamp(lane),
                };
                match resolved {
                    Some(f) => f,
                    None => {
                        return Ok(Out::Thrown(
                            "audit log: --feature is required — no bound lane or workflow feature to default to.".into(),
                        ))
                    }
                }
            }
        };
        let row = audit::append(
            &ctx.root,
            kind,
            &feature,
            flags.truthy_str("cell"),
            note,
            "bee audit log",
            None,
        )
        .map_err(|_| Err2::Ex)?;
        let id = row["id"].as_str().unwrap_or_default().to_string();
        Ok(Out::Emit(
            json!({ "entry": row }),
            format!("Logged audit entry {id} ({kind}, {feature})."),
            0,
        ))
    })();
    finish(&ctx, out)
}

fn run_list(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["kind", "feature"]) {
        return None;
    }
    let ctx = match decisions_prelude("audit list", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    let out = (|| {
        let kind = flags.truthy_str("kind");
        if let Some(k) = kind {
            if !audit::KINDS.contains(&k) {
                return Ok(Out::Thrown(audit::unknown_kind_message("audit list", k)));
            }
        }
        let entries = audit::rows(&ctx.root, kind, flags.truthy_str("feature"));
        let text = if entries.is_empty() {
            "No audit entries.".to_string()
        } else {
            entries
                .iter()
                .map(|e| {
                    let field = |k: &str| e.get(k).and_then(|v| v.as_str()).unwrap_or("");
                    format!(
                        "{} {} {} — {}",
                        field("ts"),
                        field("kind"),
                        field("feature"),
                        field("note")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        Ok(Out::Emit(json!({ "entries": entries }), text, 0))
    })();
    finish(&ctx, out)
}
