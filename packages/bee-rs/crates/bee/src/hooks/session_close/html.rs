// the matrix HTML renderer
//
// Split out of the single 2.8k-line hooks/session_close.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, ReadJson};
use crate::hooks::adapter::{emit_hook_output, encode_block, log_crash, now_iso, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::jsjson::{self, js_to_string};
use crate::state::{bypass_level, read_config_raw};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

// ── HTML rendering (renderMatrixHtml) ──────────────────────────────────────

pub(crate) fn js_math_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// Number#toFixed for the report's 1-2 digit uses (round half away from zero
/// approximated on the scaled double).
pub(crate) fn js_to_fixed(v: f64, digits: u32) -> String {
    let scale = 10f64.powi(digits as i32);
    let y = v * scale;
    let fl = y.floor();
    let n = if y - fl >= 0.5 { fl + 1.0 } else { fl };
    format!("{:.*}", digits as usize, n / scale)
}

pub(crate) fn fmt_tokens(v: f64) -> String {
    if v >= 1e9 {
        format!("{}B", js_to_fixed(v / 1e9, 2))
    } else if v >= 1e6 {
        format!("{}M", js_to_fixed(v / 1e6, 2))
    } else if v >= 1e3 {
        format!("{}k", js_to_fixed(v / 1e3, 1))
    } else {
        jsjson::js_f64_to_string(v)
    }
}

pub(crate) fn cache_pct(total: f64, cached: f64) -> String {
    if total > 0.0 {
        format!("{}%", jsjson::js_f64_to_string(js_math_round(cached / total * 100.0)))
    } else {
        "—".to_string()
    }
}

pub(crate) fn humanize_ms(ms: f64) -> String {
    if !ms.is_finite() || ms <= 0.0 {
        return "0s".to_string();
    }
    let s = js_math_round(ms / 1000.0) as i64;
    let h = s / 3600;
    let m = (s % 3600) / 60;
    let sec = s % 60;
    let mut parts = Vec::new();
    if h != 0 {
        parts.push(format!("{h}h"));
    }
    if m != 0 {
        parts.push(format!("{m}m"));
    }
    if sec != 0 || parts.is_empty() {
        parts.push(format!("{sec}s"));
    }
    parts.join("")
}

pub(crate) fn esc(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#39;".to_string(),
            other => other.to_string(),
        })
        .collect()
}

pub(crate) fn short_model(model: &str) -> String {
    let m = model.strip_prefix("claude-").unwrap_or(model);
    // /-\d{6,}$/ — a trailing -<6+ digits> run.
    if let Some(dash) = m.rfind('-') {
        let tail = &m[dash + 1..];
        if tail.len() >= 6 && tail.chars().all(|c| c.is_ascii_digit()) {
            return m[..dash].to_string();
        }
    }
    m.to_string()
}

pub(crate) fn fmt_date(ms: Option<f64>) -> Result<String, String> {
    match ms {
        None => Ok("—".to_string()),
        Some(v) => {
            let iso = ms_to_iso(v).map_err(|_| "RangeError: Invalid time value".to_string())?;
            Ok(iso[..16].replacen('T', " ", 1))
        }
    }
}

pub(crate) fn render_matrix_html(projects: &[ProjectAgg], generated_at: &str) -> Result<String, String> {
    let mut totals_models = ModelMap::default();
    let mut t_sessions = 0.0;
    let mut t_running = 0.0;
    let mut t_total = 0.0;
    let mut t_new = 0.0;
    let mut t_cached = 0.0;
    for p in projects {
        t_sessions += p.sessions;
        t_running += p.running_time_ms;
        t_total += p.total_tokens;
        t_new += p.new_tokens;
        t_cached += p.cached_tokens;
        let models_value = p.models.to_value();
        add_raw_models(&mut totals_models, Some(&models_value));
    }
    totals_models.finalize();

    let mut rows_html: Vec<String> = Vec::new();
    for (i, p) in projects.iter().enumerate() {
        let mut sorted_models: Vec<&(String, ModelAcc)> = p.models.0.iter().collect();
        sorted_models
            .sort_by(|a, b| b.1.total.partial_cmp(&a.1.total).unwrap_or(std::cmp::Ordering::Equal));
        let models_rows = sorted_models
            .iter()
            .map(|(m, v)| {
                format!(
                    "<tr><td class=\"mdl\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td></tr>",
                    esc(&short_model(m)),
                    fmt_tokens(v.total),
                    fmt_tokens(v.new_t),
                    fmt_tokens(v.cached)
                )
            })
            .collect::<Vec<_>>()
            .join("");
        let model_names = {
            let names =
                p.models.0.iter().map(|(m, _)| short_model(m)).collect::<Vec<_>>().join(", ");
            if names.is_empty() {
                "—".to_string()
            } else {
                names
            }
        };
        let title = if p.paths.is_empty() { p.project.clone() } else { p.paths.join(", ") };
        rows_html.push(format!(
            "<tbody class=\"proj\">\n  <tr class=\"row\" data-i=\"{i}\">\n    <td class=\"name\" title=\"{}\">{}</td>\n    <td class=\"num\">{}</td>\n    <td class=\"num\">{}</td>\n    <td class=\"num strong\">{}</td>\n    <td class=\"num\">{}</td>\n    <td class=\"num\">{}</td>\n    <td class=\"num\">{}</td>\n    <td class=\"num\">{}/{}</td>\n    <td class=\"models\">{}</td>\n    <td class=\"num\">{}</td>\n  </tr>\n  <tr class=\"detail\"><td colspan=\"10\"><table class=\"mtx\"><thead><tr><th>model</th><th>total</th><th>new</th><th>cached</th></tr></thead><tbody>{}</tbody></table></td></tr>\n</tbody>",
            esc(&title),
            esc(&p.project),
            jsjson::js_f64_to_string(p.sessions),
            esc(&humanize_ms(p.running_time_ms)),
            fmt_tokens(p.total_tokens),
            fmt_tokens(p.new_tokens),
            fmt_tokens(p.cached_tokens),
            cache_pct(p.total_tokens, p.cached_tokens),
            jsjson::js_f64_to_string(p.parallel_sessions),
            jsjson::js_f64_to_string(p.sessions),
            esc(&model_names),
            esc(&fmt_date(p.last_ms)?),
            models_rows
        ));
    }
    let summary = [
        ("projects", jsjson::js_f64_to_string(projects.len() as f64)),
        ("sessions", jsjson::js_f64_to_string(t_sessions)),
        ("active time", humanize_ms(t_running)),
        ("total tokens", fmt_tokens(t_total)),
        ("new", fmt_tokens(t_new)),
        ("cached", fmt_tokens(t_cached)),
        ("cache %", cache_pct(t_total, t_cached)),
    ]
    .iter()
    .map(|(k, v)| format!("<div class=\"card\"><div class=\"k\">{}</div><div class=\"v\">{}</div></div>", esc(k), esc(v)))
    .collect::<Vec<_>>()
    .join("");
    let rows = rows_html.join("\n");
    let rows_or_empty = if rows.is_empty() {
        "<tbody><tr><td class=\"empty\" colspan=\"10\">No sessions found yet. Do some work, then reopen this page.</td></tr></tbody>".to_string()
    } else {
        rows
    };
    let generated = esc(&generated_at.chars().take(19).collect::<String>().replacen('T', " ", 1));
    Ok(format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>bee performance</title>
<style>
:root{{--bg:#f7f8fa;--fg:#1a1d23;--muted:#6b7280;--card:#fff;--line:#e5e7eb;--accent:#b45309;--rowhover:#f0f1f4;}}
@media (prefers-color-scheme: dark){{:root{{--bg:#0f1115;--fg:#e6e8eb;--muted:#9aa1ab;--card:#171a21;--line:#262b34;--accent:#f59e0b;--rowhover:#1c2029;}}}}
*{{box-sizing:border-box}}
body{{margin:0;background:var(--bg);color:var(--fg);font:14px/1.5 system-ui,-apple-system,Segoe UI,Roboto,sans-serif;padding:24px;}}
h1{{font-size:20px;margin:0 0 4px}}
.sub{{color:var(--muted);font-size:12px;margin-bottom:20px}}
.cards{{display:flex;flex-wrap:wrap;gap:12px;margin-bottom:24px}}
.card{{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:12px 16px;min-width:110px}}
.card .k{{color:var(--muted);font-size:11px;text-transform:uppercase;letter-spacing:.04em}}
.card .v{{font-size:20px;font-weight:600;margin-top:2px}}
.wrap{{overflow-x:auto;border:1px solid var(--line);border-radius:10px;background:var(--card)}}
table.matrix{{border-collapse:collapse;width:100%;min-width:820px}}
table.matrix thead th{{position:sticky;top:0;background:var(--card);text-align:right;padding:10px 12px;font-size:11px;text-transform:uppercase;letter-spacing:.04em;color:var(--muted);border-bottom:1px solid var(--line);cursor:pointer;white-space:nowrap}}
table.matrix thead th:first-child{{text-align:left}}
.row td{{padding:10px 12px;border-bottom:1px solid var(--line);text-align:right;white-space:nowrap}}
.row td.name{{text-align:left;font-weight:600;max-width:340px;overflow:hidden;text-overflow:ellipsis}}
.row td.models{{text-align:left;color:var(--muted);font-size:12px;max-width:220px;overflow:hidden;text-overflow:ellipsis}}
.row td.strong{{font-weight:700;color:var(--accent)}}
.num{{font-variant-numeric:tabular-nums}}
.row:hover{{background:var(--rowhover)}}
.row{{cursor:pointer}}
.detail{{display:none}}
.detail.open{{display:table-row}}
.detail td{{padding:0 12px 12px 24px;border-bottom:1px solid var(--line)}}
table.mtx{{border-collapse:collapse;margin:6px 0}}
table.mtx th,table.mtx td{{padding:3px 14px 3px 0;text-align:right;font-size:12px;color:var(--muted)}}
table.mtx th:first-child,table.mtx td.mdl{{text-align:left;color:var(--fg)}}
.empty{{padding:40px;text-align:center;color:var(--muted)}}
</style>
</head>
<body>
<h1>bee performance</h1>
<div class="sub">{count} project(s) · generated {generated} UTC · active time excludes idle</div>
<div class="cards">{summary}</div>
<div class="wrap">
<table class="matrix">
<thead><tr>
<th data-sort="name">Project</th><th data-sort="num">Sessions</th><th data-sort="num">Active</th>
<th data-sort="num">Total</th><th data-sort="num">New</th><th data-sort="num">Cached</th><th data-sort="num">Cache%</th>
<th data-sort="num">Parallel</th><th data-sort="name">Models</th><th data-sort="num">Last active</th>
</tr></thead>
{rows_or_empty}
</table>
</div>
<script>
// expand a project row to show its per-model breakdown
document.querySelectorAll('tr.row').forEach(function(r){{
  r.addEventListener('click',function(){{
    var d=r.parentNode.querySelector('tr.detail');
    if(d) d.classList.toggle('open');
  }});
}});
</script>
</body>
</html>
"#,
        count = jsjson::js_f64_to_string(projects.len() as f64),
    ))
}

/// maybePerfRefresh — best-effort; Err(msg) => logCrash(source 'perf-refresh').
pub(crate) fn perf_refresh(root: &Path, session_id: Option<&str>) -> Result<(), String> {
    if let Some(transcript) = resolve_transcript_for(root, session_id) {
        if let Some(rollup) = rollup_transcript(&transcript) {
            let record = session_record(&rollup)?;
            upsert_session_records(&[record])?;
        }
    }
    if !read_session_records().is_empty() {
        let projects = build_matrix_from_log();
        let html = render_matrix_html(&projects, &now_iso())?;
        let out = global_perf_dir().join("performance.html");
        if let Some(dir) = out.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("Error: {e}"))?;
        }
        std::fs::write(&out, html).map_err(|e| format!("Error: {e}"))?;
    }
    Ok(())
}
