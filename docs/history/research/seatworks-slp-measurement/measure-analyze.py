import json, os, re, glob
S = os.path.dirname(os.path.abspath(__file__))
DENY = re.compile(r"guard|denied|refused|intake gate", re.I)
STATE_READ = re.compile(r"(cat|jq|head|sed -n)[^|;&]*\.bee/(state|lanes|claims|cells|runtime)|json\.load\(open\([^)]*\.bee/")
HAND_STATE_WRITE = re.compile(r"(rm|mv|>|tee)\s[^|;&]*\.bee/(state|lanes|claims|cells|runtime)|json\.dump[^\n]*\.bee/")
def text_of(r):
    if isinstance(r, dict) and isinstance(r.get("content"), list):
        return "\n".join(x.get("text", "") for x in r["content"] if isinstance(x, dict))
    return json.dumps(r)
def capped_cells(repo):
    n = 0
    for f in glob.glob(os.path.join(repo, ".bee", "cells", "**", "*.json"), recursive=True):
        try:
            if json.load(open(f)).get("status") == "capped":
                n += 1
        except Exception:
            pass
    return n
rows = []
for d in sorted(glob.glob(os.path.join(S, "measure", "*"))):
    label = os.path.basename(d)
    p = os.path.join(d, "stdout.jsonl")
    if not os.path.exists(p) or label.startswith("spike"):
        continue
    order, ends, err = [], {}, None
    for line in open(p, errors="replace"):
        try:
            e = json.loads(line)
        except Exception:
            continue
        t = e.get("type")
        if t == "tool_execution_start":
            order.append(e)
        elif t == "tool_execution_end":
            ends[e.get("toolCallId")] = e
        elif t == "message_end" and e.get("message", {}).get("stopReason") == "error":
            err = e["message"].get("errorMessage")
    denials = repeats = state_reads = state_writes = bee_calls = 0
    prev_deny = None
    for s in order:
        args = json.dumps(s.get("args"))
        if ".bee/bin/bee" in args or re.search(r"\bbee [a-z]", args):
            bee_calls += 1
        if STATE_READ.search(args):
            state_reads += 1
        if HAND_STATE_WRITE.search(args):
            state_writes += 1
        e = ends.get(s.get("toolCallId"))
        out = text_of(e.get("result")) if e else ""
        if e and e.get("isError") and DENY.search(out):
            denials += 1
            key = out.strip()[:80]
            if key == prev_deny:
                repeats += 1
            prev_deny = key
        else:
            prev_deny = None
    run = {}
    rp = os.path.join(d, "run.txt")
    if os.path.exists(rp):
        for tok in open(rp).read().split():
            if "=" in tok:
                k, v = tok.split("=", 1)
                run[k] = v
    rows.append({"run": label, "tool_calls": len(order), "bee_calls": bee_calls,
                 "guard_denials": denials, "same_denial_twice": repeats,
                 "hand_state_reads": state_reads, "hand_state_writes": state_writes,
                 "capped_cells": capped_cells(os.path.join(d, "repo")),
                 "script_on_main": run.get("main_has_script"), "executable": run.get("main_script_exec"),
                 "seconds": run.get("seconds"), "model_error": err})
print(json.dumps(rows, indent=1))
