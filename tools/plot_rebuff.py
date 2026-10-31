#!/usr/bin/env python3
"""Plot and tabulate `Tuning::rebuff_memory_hours`: how soon a person goes back to someone they approached.

    /tmp/plotenv/bin/python tools/plot_rebuff.py <run-dir> <out-plot.png> <out-sweep.png> <out-table.md>

Reads `<run-dir>/evidence/{before,after}/<run>/{reapproach.txt,summary.json}` for each run that exists, and
`<run-dir>/evidence/sweep/h<hours>/` for the memory sweep. `reapproach.txt` is the harness's
`ReapproachLedger`: for each earlier outcome (refused, accepted), the share of approaches followed by another
to the same person within 1 / 3 / 6 / 12 / 24 hours. The plot answers: after a refusal, does the person stop
going back to the one who refused, and do they go back later? The table is what the review cites.
"""
import json
import pathlib
import re
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
from PIL import Image

RUNS = (
    ("v7", "village s7"),
    ("v42", "village s42"),
    ("v11", "village s11"),
    ("v99", "village s99"),
    ("t7", "town 60 s7"),
    ("t99", "town 60 s99"),
    ("t40s3", "town 40 s3"),
)
WINDOWS = (1, 3, 6, 12, 24)


def save(fig, out, dpi=110):
    """Save `fig` as a 64-colour PNG, so a committed plot stays under the 100 KB evidence limit."""
    fig.savefig(out, dpi=dpi)
    Image.open(out).convert("RGB").quantize(colors=64).save(out, optimize=True)


def parse(path):
    """`reapproach.txt` -> {'refused': {window: not_an_insult_pct}, 'accepted': {...}, counts, diversity}."""
    text = path.read_text()
    out = {"refused": {}, "accepted": {}, "n": {}}
    section = None
    for line in text.splitlines():
        m = re.match(r"after a (refused|accepted) approach \((\d+) of them\)", line)
        if m:
            section = m.group(1)
            out["n"][section] = int(m.group(2))
            continue
        m = re.match(r"\s+(\d+)\s+([\d.]+)%\s+([\d.]+)%\s+([\d.]+)%", line)
        if m and section:
            out[section][int(m.group(1))] = float(m.group(3))
            continue
        m = re.search(
            r"approaches per person-day ([\d.]+), different people per person-day ([\d.]+) \(([\d.]+)% ", line
        )
        if m:
            out["per_day"] = float(m.group(1))
            out["distinct"] = float(m.group(2))
            out["same_day_repeat"] = float(m.group(3))
    return out


def load(run, sub, key):
    base = run / "evidence" / sub / key
    if not (base / "reapproach.txt").exists():
        return None
    r = parse(base / "reapproach.txt")
    s = json.loads((base / "summary.json").read_text())
    r["summary"] = s
    return r


def bars(run, out):
    present = [(k, t) for k, t in RUNS if load(run, "before", k) and load(run, "after", k)]
    if not present:
        return
    fig, axes = plt.subplots(1, len(present), figsize=(3.0 * len(present), 3.6), sharey=True)
    if len(present) == 1:
        axes = [axes]
    xs = range(len(WINDOWS))
    for ax, (key, title) in zip(axes, present):
        b, a = load(run, "before", key), load(run, "after", key)
        ax.bar([x - 0.27 for x in xs], [b["refused"][w] for w in WINDOWS], 0.27, color="#c0392b", label="refused, before")
        ax.bar([x for x in xs], [a["refused"][w] for w in WINDOWS], 0.27, color="#1f77b4", label="refused, after")
        ax.bar([x + 0.27 for x in xs], [b["accepted"][w] for w in WINDOWS], 0.27, color="#999999", label="accepted, before")
        ax.set_xticks(list(xs))
        ax.set_xticklabels([f"{w}h" for w in WINDOWS])
        ax.set_title(title, fontsize=9)
        ax.grid(alpha=0.3, axis="y")
    axes[0].set_ylabel("% of approaches followed by another\nto the same person (not an insult)")
    axes[0].legend(fontsize=7, loc="upper left")
    fig.suptitle("Going back to someone: within this many hours of the last approach", fontsize=10)
    fig.tight_layout()
    save(fig, out)


def sweep(run, out):
    hours, rows = [], []
    base = load(run, "before", "v7")
    if base is None:
        return
    points = [(0, base)]
    for h in (3, 6, 12, 24):
        r = load(run, "sweep", f"h{h}")
        if r:
            points.append((h, r))
    hours = [h for h, _ in points]
    fig, axes = plt.subplots(1, 3, figsize=(11, 3.3))
    ax = axes[0]
    ax.plot(hours, [r["refused"][1] for _, r in points], "o-", color="#1f77b4", label="within 1 h")
    ax.plot(hours, [r["refused"][24] for _, r in points], "s-", color="#c0392b", label="within 24 h")
    ax.set_title("after a refusal: back to the same person (%)", fontsize=9)
    ax = axes[1]
    ax.plot(hours, [r["summary"]["refusals"] / (r["summary"]["refusals"] + r["summary"]["acceptances"]) * 100 for _, r in points], "o-", color="#8e44ad")
    ax.set_title("refused share of all approaches (%)", fontsize=9)
    ax = axes[2]
    ax.plot(hours, [r["summary"]["friends"] for _, r in points], "o-", color="#27ae60", label="friend ties")
    ax.plot(hours, [r["summary"]["rivals"] for _, r in points], "s-", color="#e67e22", label="rival ties")
    ax.plot(hours, [r["summary"]["insults"] for _, r in points], "^-", color="#7f8c8d", label="insults")
    ax.set_title("village seed 7", fontsize=9)
    for ax in axes:
        ax.set_xlabel("rebuff_memory_hours (0 = off)")
        ax.grid(alpha=0.3)
        ax.legend(fontsize=7)
    fig.tight_layout()
    save(fig, out)


def means(path):
    """Mean pleasure and mean `social`, `rest`, `food`, `fun` need over the whole run, from `series.csv`."""
    import csv

    cols = ("pleasure", "need0", "need1", "need2", "need3")
    sums, n = [0.0] * len(cols), 0
    with open(path) as f:
        for row in csv.DictReader(f):
            for i, c in enumerate(cols):
                sums[i] += float(row[c])
            n += 1
    return [s / max(n, 1) for s in sums]


def table(run, out):
    head = (
        "| run | arm | approaches (accepted / refused) | refused % | back to a refuser, not an insult, 1 h / 6 h / 24 h | "
        "back after an accepted, 1 h / 24 h | approaches / different people per person-day | friend / rival ties | insults | "
        "mean pleasure | mean social / fun need |\n"
        "|---|---|---|---|---|---|---|---|---|---|---|\n"
    )
    lines = [head]
    for key, title in RUNS:
        for sub in ("before", "after"):
            r = load(run, sub, key)
            if r is None:
                continue
            s = r["summary"]
            acc, ref = s["acceptances"], s["refusals"]
            m = means(run / "evidence" / sub / key / "series.csv")
            lines.append(
                f"| {title} | {sub} | {acc} / {ref} | {100 * ref / (acc + ref):.1f} | "
                f"{r['refused'][1]:.1f} / {r['refused'][6]:.1f} / {r['refused'][24]:.1f} | "
                f"{r['accepted'][1]:.1f} / {r['accepted'][24]:.1f} | {r['per_day']:.2f} / {r['distinct']:.2f} | "
                f"{s['friends']} / {s['rivals']} | {s['insults']} | {m[0]:+.3f} | {m[1]:.3f} / {m[4]:.3f} |\n"
            )
    sw = [(0, load(run, "before", "v7"))] + [(h, load(run, "sweep", f"h{h}")) for h in (3, 6, 12, 24)]
    if all(r for _, r in sw):
        lines.append("\nSweep, village seed 7 (hours = `rebuff_memory_hours`, 0 = off):\n\n")
        lines.append("| hours | refused % | back to a refuser 1 h / 6 h / 24 h | approaches / different people per person-day | friend / rival ties | insults | rumor reach |\n|---|---|---|---|---|---|---|\n")
        for h, r in sw:
            s = r["summary"]
            lines.append(
                f"| {h} | {100 * s['refusals'] / (s['refusals'] + s['acceptances']):.1f} | "
                f"{r['refused'][1]:.1f} / {r['refused'][6]:.1f} / {r['refused'][24]:.1f} | {r['per_day']:.2f} / {r['distinct']:.2f} | "
                f"{s['friends']} / {s['rivals']} | {s['insults']} | {s['rumor_reach']} |\n"
            )
    out.write_text("".join(lines))


def main():
    if len(sys.argv) != 5:
        print(__doc__)
        sys.exit(1)
    run = pathlib.Path(sys.argv[1])
    bars(run, pathlib.Path(sys.argv[2]))
    sweep(run, pathlib.Path(sys.argv[3]))
    table(run, pathlib.Path(sys.argv[4]))


if __name__ == "__main__":
    main()
