#!/usr/bin/env python3
"""Plot and tabulate the week: what a village does on each day of the week, with and without the weekly overlay.

    /tmp/plotenv/bin/python tools/plot_weekly.py <run-dir> <out-plot.png> <out-table.md>

Reads `<run-dir>/evidence/{before,after}/<run>/{week.txt,summary.json}` for each run that exists. `week.txt` is the
harness's `WeekLedger`: a row per weekday (and a `workdays` and a `rest` summary row) over the full game-days only, as
shares of villager-ticks (work, company, leisure, sleep, food, other), exchanges per villager-day (approaches, different
people), the share of approaches to a friend and the share accepted, and the mean `social` and
`fun` need. `before` is the society without a week, `after` the same seed on `examples/assets/weekly.society.ron`.
"""
import json
import pathlib
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
COLS = (
    "days", "work", "company", "leisure", "sleep", "food", "other",
    "approach", "distinct", "friend", "accept", "social", "fun",
)


def save(fig, out, dpi=110):
    """Save `fig` as a 64-colour PNG, so a committed plot stays under the 100 KB evidence limit."""
    fig.savefig(out, dpi=dpi)
    Image.open(out).convert("RGB").quantize(colors=64).save(out, optimize=True)


def parse(path):
    """`week.txt` -> {row label: {column: float}}."""
    rows = {}
    for line in path.read_text().splitlines()[1:]:
        parts = line.split()
        if len(parts) == len(COLS) + 1:
            rows[parts[0]] = dict(zip(COLS, map(float, parts[1:])))
    return rows


def load(run, sub, key):
    base = run / "evidence" / sub / key
    if not (base / "week.txt").exists():
        return None
    return {"week": parse(base / "week.txt"), "summary": json.loads((base / "summary.json").read_text())}


def present(run):
    return [(k, t, load(run, "before", k), load(run, "after", k)) for k, t in RUNS if load(run, "before", k) and load(run, "after", k)]


def panels(run, out):
    runs = present(run)
    if not runs:
        return
    spec = (
        ("work", "work (% of villager-ticks)"),
        ("company", "company (% of villager-ticks)"),
        ("distinct", "different people met, per villager-day"),
        ("friend", "approaches to a friend (%)"),
        ("approach", "approaches begun, per villager-day"),
        ("social", "mean `social` need"),
    )
    fig, axes = plt.subplots(2, 3, figsize=(12, 6.2))
    days = [str(d) for d in range(7)]
    for ax, (col, title) in zip(axes.flat, spec):
        for key, _, before, after in runs:
            ax.plot(range(7), [before["week"][d][col] for d in days], color="#c0392b", alpha=0.25, lw=1)
            ax.plot(range(7), [after["week"][d][col] for d in days], color="#1f77b4", alpha=0.25, lw=1)
        before_mean = [sum(b["week"][d][col] for _, _, b, _ in runs) / len(runs) for d in days]
        after_mean = [sum(a["week"][d][col] for _, _, _, a in runs) / len(runs) for d in days]
        ax.plot(range(7), before_mean, "o-", color="#c0392b", lw=2.2, label="no week")
        ax.plot(range(7), after_mean, "s-", color="#1f77b4", lw=2.2, label="weekly overlay")
        ax.axvspan(4.5, 6.5, color="#f1c40f", alpha=0.12)
        ax.set_title(title, fontsize=9)
        ax.set_xticks(range(7))
        ax.set_xlabel("weekday (5 and 6 are the rest days)")
        ax.grid(alpha=0.3)
    axes[0][0].legend(fontsize=8, loc="best")
    fig.suptitle(f"A week, {len(runs)} runs (thin lines: each run; thick: mean). Full game-days only.", fontsize=10)
    fig.tight_layout()
    save(fig, out)


def table(run, out):
    runs = present(run)
    lines = [
        "| run | pack | row | work % | company % | leisure % | sleep % | approach/day | different/day | friend % | accept % | social | fun |",
        "|---|---|---|---|---|---|---|---|---|---|---|---|---|",
    ]
    for key, title, before, after in runs:
        for pack, r in (("no week", before), ("weekly", after)):
            for row in ("workdays", "rest"):
                w = r["week"][row]
                lines.append(
                    f"| {title} | {pack} | {row} | {w['work']:.1f} | {w['company']:.1f} | {w['leisure']:.1f} | {w['sleep']:.1f} "
                    f"| {w['approach']:.2f} | {w['distinct']:.2f} | {w['friend']:.1f} | {w['accept']:.1f} "
                    f"| {w['social']:.3f} | {w['fun']:.3f} |"
                )
    lines += ["", "| run | friends before / after | rivals | insults | refusals / acceptances |", "|---|---|---|---|---|"]
    for key, title, before, after in runs:
        b, a = before["summary"], after["summary"]
        lines.append(
            f"| {title} | {b['friends']} / {a['friends']} | {b['rivals']} / {a['rivals']} | {b['insults']} / {a['insults']} "
            f"| {b['refusals']}/{b['acceptances']} / {a['refusals']}/{a['acceptances']} |"
        )
    pathlib.Path(out).write_text("\n".join(lines) + "\n")


def main():
    run = pathlib.Path(sys.argv[1])
    panels(run, run / sys.argv[2])
    table(run, run / sys.argv[3])


if __name__ == "__main__":
    main()
