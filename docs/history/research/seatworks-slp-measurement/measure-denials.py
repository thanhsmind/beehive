import json, os, re, glob, collections
S = os.path.dirname(os.path.abspath(__file__))
DENY = re.compile(r"guard|denied|refused|intake gate", re.I)
def text_of(r):
    if isinstance(r, dict) and isinstance(r.get("content"), list):
        return "\n".join(x.get("text", "") for x in r["content"] if isinstance(x, dict))
    return json.dumps(r)
def kind(out):
    o = out
    for k, pat in [("cd-then-write", r"a `cd` earlier"), ("unexpanded-var", r"unexpanded shell syntax"),
                   ("concurrent-git", r"concurrent-worker git guard"), ("cli-shape", r"CLI-shape guard"),
                   ("containment", r"canonically contained"), ("worktree-first", r"worktree-first guard"),
                   ("intake-gate", r"intake gate"), ("control-plane-in-worktree", r"refused inside a granted feature worktree"),
                   ("worker-outward", r"worker-outward"), ("gate-refused", r"gate: approval refused|not approved"),
                   ("claim", r"CLAIMED|claim:"), ("preview", r"preview packet"), ("other-refused", r"refused"), ("other-denied", r"denied")]:
        if re.search(pat, o):
            return k
    return "other"
per = {}
followed = collections.Counter()
for d in sorted(glob.glob(os.path.join(S, "measure", "*"))):
    label = os.path.basename(d)
    p = os.path.join(d, "stdout.jsonl")
    if not os.path.exists(p) or label.startswith("spike"):
        continue
    order, ends = [], {}
    for line in open(p, errors="replace"):
        try:
            e = json.loads(line)
        except Exception:
            continue
        if e.get("type") == "tool_execution_start":
            order.append(e)
        elif e.get("type") == "tool_execution_end":
            ends[e.get("toolCallId")] = e
    c = collections.Counter()
    for i, s in enumerate(order):
        e = ends.get(s.get("toolCallId"))
        out = text_of(e.get("result")) if e else ""
        if e and e.get("isError") and DENY.search(out):
            k = kind(out)
            c[k] += 1
            nxt = order[i + 1] if i + 1 < len(order) else None
            nerr = ends.get(nxt.get("toolCallId")) if nxt else None
            ok_next = nerr is not None and not nerr.get("isError")
            followed[(label.split("-")[0], ok_next)] += 1
    per[label] = dict(c)
for k, v in per.items():
    print(k, v)
print("next call after a denial succeeded (version, ok):", dict(followed))
