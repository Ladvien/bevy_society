#!/usr/bin/env python3
"""Turn the nine-run grid into the numbers and plots the iteration's eval.md cites.

Usage: analyze.py <run-dir>

Reads `<run>/{before,after}/*/series.csv` + `summary.json`, and writes
`<run>/evidence/{plots/*.png,grid.md,narratives/}`.
"""
import csv
import json
import pathlib
import sys
from collections import Counter, defaultdict

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt


def doing_share(series_path):
    """Share of person-ticks per activity, from the `doing` column, excluding travelling."""
    totals = Counter()
    ticks = 0
    with open(series_path) as fh:
        for row in csv.DictReader(fh):
            ticks += 1
            field = row.get("doing") or ""
            for part in field.split(" "):
                if "=" not in part:
                    continue
                name, count = part.rsplit("=", 1)
                if name.startswith("travelling:"):
                    continue
                try:
                    totals[name] += int(count)
                except ValueError:
                    continue
    total = sum(totals.values()) or 1
    return {name: count / total for name, count in totals.items()}, total, ticks


def load(root):
    """Every run under `root` as (side, seed, people, summary, doing shares)."""
    out = []
    root = pathlib.Path(root)
    for run in sorted(root.glob("before/*")) + sorted(root.glob("after/*")):
        summary = json.loads((run / "summary.json").read_text())
        shares, person_ticks, ticks = doing_share(run / "series.csv")
        side = run.parent.name
        seed, people = run.name.split("_")
        out.append(
            dict(
                side=side,
                seed=int(seed[1:]),
                people=int(people[1:]),
                summary=summary,
                shares=shares,
                person_ticks=person_ticks,
                ticks=ticks,
                path=run,
            )
        )
    return out


def rivals_pct(run):
    s = run["summary"]
    people = s["people"]
    pairs = people * (people - 1) / 2
    return 100.0 * s["rivals"] / pairs if pairs else 0.0


def mean(values):
    values = list(values)
    return sum(values) / len(values) if values else float("nan")


def stderr(values):
    values = list(values)
    if len(values) < 2:
        return float("nan")
    m = mean(values)
    var = sum((v - m) ** 2 for v in values) / (len(values) - 1)
    return (var / len(values)) ** 0.5


ACTIVITIES = ["work", "craft", "sleep", "eat", "chat", "gossip", "relax", "snub", "share", "bake"]


def write_grid(runs, out_dir):
    lines = []
    lines.append("| side | seed | people | rivals | pairs | rival % | work share | craft share | sleep share |")
    lines.append("|---|---|---|---|---|---|---|---|---|")
    for run in runs:
        s = run["summary"]
        pairs = s["people"] * (s["people"] - 1) // 2
        lines.append(
            "| {} | {} | {} | {} | {} | {:.2f}% | {:.2f}% | {:.2f}% | {:.2f}% |".format(
                run["side"],
                run["seed"],
                s["people"],
                s["rivals"],
                pairs,
                rivals_pct(run),
                100 * run["shares"].get("work", 0.0),
                100 * run["shares"].get("craft", 0.0),
                100 * run["shares"].get("sleep", 0.0),
            )
        )

    lines.append("")
    lines.append("| people | side | rival % (mean) | work share (mean) | friends (mean) | interrupted (mean) |")
    lines.append("|---|---|---|---|---|---|")
    for people in sorted({run["people"] for run in runs}):
        for side in ("before", "after"):
            group = [run for run in runs if run["side"] == side and run["people"] == people]
            if not group:
                continue
            lines.append(
                "| {} | {} | {:.2f}% | {:.2f}% | {:.1f} | {:.1f} |".format(
                    people,
                    side,
                    mean(rivals_pct(r) for r in group),
                    100 * mean(r["shares"].get("work", 0.0) for r in group),
                    mean(r["summary"]["friends"] for r in group),
                    mean(r["summary"]["interrupted_endings"] for r in group),
                )
            )
    text = "\n".join(lines)
    (out_dir / "grid.md").write_text(text + "\n")
    print(text)
    return text


def plot_rival_density(runs, out_dir):
    fig, ax = plt.subplots(figsize=(7, 4.5))
    for side, colour, marker in (("before", "#c0392b", "o"), ("after", "#2471a3", "s")):
        groups = sorted({r["people"] for r in runs if r["side"] == side})
        xs = [g for g in groups if any(r["side"] == side and r["people"] == g for r in runs)]
        ys = [mean(rivals_pct(r) for r in runs if r["side"] == side and r["people"] == g) for g in xs]
        errs = [stderr(rivals_pct(r) for r in runs if r["side"] == side and r["people"] == g) for g in xs]
        ax.errorbar(xs, ys, yerr=errs, label=side, color=colour, marker=marker, capsize=4)
    ax.axhspan(7, 8, color="#f4d03f", alpha=0.3, label="target band 7-8%")
    ax.set_xlabel("villagers (fixed 30 seats)")
    ax.set_ylabel("rival pairs, % of all pairs")
    ax.set_title("Rival density against population")
    ax.set_xticks([24, 36, 48])
    ax.legend()
    ax.grid(alpha=0.3)
    fig.tight_layout()
    fig.savefig(out_dir / "rival_density.png", dpi=140)
    plt.close(fig)


def plot_work_share(runs, out_dir):
    fig, ax = plt.subplots(figsize=(7, 4.5))
    for side, colour, marker in (("before", "#c0392b", "o"), ("after", "#2471a3", "s")):
        groups = sorted({r["people"] for r in runs if r["side"] == side})
        xs = list(groups)
        ys = [
            100 * mean(r["shares"].get("work", 0.0) for r in runs if r["side"] == side and r["people"] == g)
            for g in xs
        ]
        errs = [
            100 * stderr(r["shares"].get("work", 0.0) for r in runs if r["side"] == side and r["people"] == g)
            for g in xs
        ]
        ax.errorbar(xs, ys, yerr=errs, label=side, color=colour, marker=marker, capsize=4)
    ax.axhline(10, color="#7d6608", linestyle="--", alpha=0.6, label="baseline 24-person work share")
    ax.set_xlabel("villagers (fixed 30 seats)")
    ax.set_ylabel("`work`, % of all doing person-ticks")
    ax.set_title("Work survives a full workshop")
    ax.set_xticks([24, 36, 48])
    ax.legend()
    ax.grid(alpha=0.3)
    fig.tight_layout()
    fig.savefig(out_dir / "work_share.png", dpi=140)
    plt.close(fig)


def plot_activity_mix(runs, out_dir):
    """Every activity's share at 48 people, before and after: 'an activity vanished' made visible."""
    fig, ax = plt.subplots(figsize=(9, 4.5))
    width = 0.38
    for i, side in enumerate(("before", "after")):
        group = [r for r in runs if r["side"] == side and r["people"] == 48]
        if not group:
            continue
        heights = [100 * mean(r["shares"].get(a, 0.0) for r in group) for a in ACTIVITIES]
        xs = [j + (i - 0.5) * width for j in range(len(ACTIVITIES))]
        ax.bar(xs, heights, width=width, label=side)
    ax.set_xticks(range(len(ACTIVITIES)))
    ax.set_xticklabels(ACTIVITIES, rotation=30)
    ax.set_ylabel("% of doing person-ticks at 48 villagers")
    ax.set_title("The activity mix when the village is over capacity")
    ax.legend()
    ax.grid(alpha=0.3, axis="y")
    fig.tight_layout()
    fig.savefig(out_dir / "activity_mix_48.png", dpi=140)
    plt.close(fig)


def plot_series(runs, out_dir):
    """Rival count over time, one panel per population, both sides overlaid at the same seed."""
    people_values = sorted({r["people"] for r in runs})
    fig, axes = plt.subplots(1, len(people_values), figsize=(5 * len(people_values), 4), sharey=True)
    if len(people_values) == 1:
        axes = [axes]
    for ax, people in zip(axes, people_values):
        for side, colour in (("before", "#c0392b"), ("after", "#2471a3")):
            for run in runs:
                if run["side"] != side or run["people"] != people:
                    continue
                ticks, rivals = [], []
                with open(run["path"] / "series.csv") as fh:
                    for row in csv.DictReader(fh):
                        ticks.append(int(row["tick"]))
                        rivals.append(int(row["rivals"]))
                ax.plot(ticks, rivals, color=colour, alpha=0.75, linewidth=1.2, label=side)
        ax.set_title("{} villagers".format(people))
        ax.set_xlabel("tick")
        ax.grid(alpha=0.3)
    axes[0].set_ylabel("rival pairs")
    handles, labels = axes[0].get_legend_handles_labels()
    unique = dict(zip(labels, handles))
    axes[0].legend(unique.values(), unique.keys())
    fig.suptitle("Rival pairs over the run, three seeds overlaid")
    fig.tight_layout()
    fig.savefig(out_dir / "rivals_over_time.png", dpi=140)
    plt.close(fig)


def main():
    run_dir = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "docs/loop/runs/2026-12-29-01")
    evidence = run_dir / "evidence"
    out_dir = evidence / "plots"
    out_dir.mkdir(parents=True, exist_ok=True)
    runs = load(evidence)
    if not runs:
        print("no runs found under", evidence)
        return
    write_grid(runs, out_dir)
    plot_rival_density(runs, out_dir)
    plot_work_share(runs, out_dir)
    plot_activity_mix(runs, out_dir)
    plot_series(runs, out_dir)
    print("wrote", out_dir)


if __name__ == "__main__":
    main()
