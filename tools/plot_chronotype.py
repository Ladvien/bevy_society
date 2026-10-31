#!/usr/bin/env python3
"""When the village sleeps, with and without body clocks.

    tools/plot_chronotype.py <run-dir> <sleep.png> <day.png> <table.md>     # paths relative to <run-dir>

Reads `<run-dir>/evidence/{before,after}/<key>/{sleep.txt,summary.json,series.csv}` for the keys it finds (`v7`, `v42`, `v11`,
`v99` for the 24-person village, `t7`, `t99` for the 60-person town, `t40s3` for a 40-person town) and, when they exist,
`<run-dir>/evidence/variants/<name>/` for the two controls of `eval.md`. `before` is the society as it was (every person on the
village's clock, a sleeper can be approached: `--sleepers-available`, byte-identical to the last release); `after` gives each person
a `Chronotype` with a standard deviation of 1.5 h (`--chronotype-sd 1.5`).

Writes

- `sleep.png`: top row the share of the village asleep at each clock hour (red before, blue after); bottom row the clock hour each
  sleep began at;
- `day.png`: what the village was doing at each clock hour, averaged over the run (a stacked share of villager-ticks), before and
  after, for the first key;
- a markdown table of the numbers a review quotes (the CSVs themselves are ignored by git and regenerate with the command in
  `eval.md`).

`sleep.txt` is the harness's `SleepLedger`: `hour asleep_pct began woke`, then the totals. `series.csv` has one row per tick with a
`doing` column of `activity=count` pairs, and `summary.json` gives the ticks and game-days a clock hour is cut from.
"""
import csv
import json
import re
import sys
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

KEYS = ["v7", "v42", "v11", "v99", "t7", "t99", "t40s3", "v7d240"]
NAMES = {
    "v7": "village s7",
    "v42": "village s42",
    "v11": "village s11",
    "v99": "village s99 (fresh eyes)",
    "t7": "town 60 s7",
    "t99": "town 60 s99 (fresh eyes)",
    "t40s3": "town 40 s3 (fresh eyes)",
    "v7d240": "village s7, 240 s day (6 days)",
}
PLOTTED = ["v7d240", "v7", "t7", "v99"]
START_HOUR = 15.0  # `village-evening` starts at 15:00 (`docs/loop/scenarios.yaml`)
RED, BLUE = "#c0392b", "#2c6fbb"


def save_small(fig, path, colours=64):
    """Writes the figure as a palette PNG so the committed file stays under the 100 KB a loop run allows each file."""
    import io

    from PIL import Image

    buffer = io.BytesIO()
    fig.savefig(buffer, dpi=100, format="png")
    buffer.seek(0)
    Image.open(buffer).convert("RGB").quantize(colors=colours).save(path, optimize=True)


def read_sleep(path):
    """`(asleep_pct[24], began[24], woke[24], totals text)` from a `sleep.txt`."""
    asleep, began, woke = [0.0] * 24, [0] * 24, [0] * 24
    tail = []
    for line in Path(path).read_text().splitlines():
        parts = line.split()
        if len(parts) == 4 and parts[0].isdigit():
            hour = int(parts[0])
            asleep[hour], began[hour], woke[hour] = float(parts[1]), int(parts[2]), int(parts[3])
        elif line.strip() and not line.startswith("hour"):
            tail.append(line.strip())
    return asleep, began, woke, tail


def stats(side_dir):
    """Everything the table quotes for one run."""
    asleep, began, woke, tail = read_sleep(side_dir / "sleep.txt")
    summary = json.loads((side_dir / "summary.json").read_text())
    total = max(sum(began), 1)
    out = {
        "asleep": asleep,
        "began": began,
        "woke": woke,
        "busiest_hour_share": 100.0 * max(began) / total,
        "mixed_hours": sum(1 for s in asleep if 10.0 < s < 90.0),
        "peak": max(asleep),
        "trough": min(asleep),
        "summary": summary,
    }
    for line in tail:
        m = re.match(r"sleeps started (\d+), completed (\d+), interrupted (\d+)", line)
        if m:
            out["started"], out["completed"], out["interrupted"] = map(int, m.groups())
        m = re.match(r"exchanges aimed at somebody asleep: (\d+) of (\d+)", line)
        if m:
            out["slept_on"], out["exchanges"] = map(int, m.groups())
    return out


def by_clock_hour(side_dir):
    """What the village was doing at each clock hour: `{activity: share of villager-ticks}` per hour of the day."""
    summary = json.loads((side_dir / "summary.json").read_text())
    people = summary["people"]
    # `game_days` is the clock's `now_days`, which started at `START_HOUR`; a game-day is the ticks of the elapsed days.
    per_day = summary["ticks"] / (summary["game_days"] - START_HOUR / 24.0)
    sums = [dict() for _ in range(24)]
    counted = [0] * 24
    with open(side_dir / "series.csv") as handle:
        for row in csv.DictReader(handle):
            tick = int(row["tick"])
            hour = int((START_HOUR + 24.0 * (tick / per_day)) % 24.0)
            counted[hour] += people
            for pair in row["doing"].split():
                name, _, count = pair.rpartition("=")
                if name.startswith("travelling:"):
                    name = "walking"
                sums[hour][name] = sums[hour].get(name, 0) + int(count)
    return [{k: v / max(counted[h], 1) for k, v in sums[h].items()} for h in range(24)]


def find(base, key):
    path = base / key
    return path if (path / "sleep.txt").exists() else None


def main():
    run = Path(sys.argv[1])
    sleep_png, day_png, table_md = (run / name for name in sys.argv[2:5])
    before_dir, after_dir = run / "evidence" / "before", run / "evidence" / "after"
    keys = [k for k in KEYS if find(before_dir, k) and find(after_dir, k)]
    plotted = [k for k in PLOTTED if k in keys]

    fig, axes = plt.subplots(2, len(plotted), figsize=(4.6 * len(plotted), 7.0), squeeze=False, sharex=True)
    hours = list(range(24))
    for col, key in enumerate(plotted):
        before, after = stats(find(before_dir, key)), stats(find(after_dir, key))
        top, bottom = axes[0][col], axes[1][col]
        top.step(hours, before["asleep"], where="mid", color=RED, label="before: one clock")
        top.step(hours, after["asleep"], where="mid", color=BLUE, label="after: body clocks (sd 1.5 h)")
        top.set_title(NAMES[key])
        top.set_ylim(0, 100)
        top.grid(alpha=0.25)
        bottom.bar([h - 0.2 for h in hours], before["began"], width=0.4, color=RED)
        bottom.bar([h + 0.2 for h in hours], after["began"], width=0.4, color=BLUE)
        bottom.set_xlabel("clock hour")
        bottom.grid(alpha=0.25)
        if col == 0:
            top.set_ylabel("% of the village asleep")
            bottom.set_ylabel("sleeps begun in that hour")
            top.legend(loc="upper center", fontsize=8)
        bottom.set_xticks(range(0, 24, 3))
    fig.tight_layout()
    save_small(fig, sleep_png)
    plt.close(fig)

    # What the village does by the clock, first plotted key: before and after, stacked.
    key = plotted[0]
    shares = [by_clock_hour(find(before_dir, key)), by_clock_hour(find(after_dir, key))]
    names = sorted({n for side in shares for hour in side for n in hour})
    order = ["sleep", "walking"] + [n for n in names if n not in ("sleep", "walking")]
    palette = plt.get_cmap("tab20")
    fig, axes = plt.subplots(1, 2, figsize=(12.5, 4.4), sharey=True)
    for ax, side, title in zip(axes, shares, ["before: one clock", "after: body clocks (sd 1.5 h)"]):
        stack = [[side[h].get(n, 0.0) for h in hours] for n in order]
        ax.stackplot(hours, stack, labels=order, colors=[palette(i) for i in range(len(order))])
        ax.set_title(f"{NAMES[key]}: {title}")
        ax.set_xlabel("clock hour")
        ax.set_xlim(0, 23)
        ax.set_ylim(0, 1)
    axes[0].set_ylabel("share of villager-ticks")
    axes[1].legend(loc="center left", bbox_to_anchor=(1.0, 0.5), fontsize=8)
    fig.tight_layout()
    save_small(fig, day_png)
    plt.close(fig)

    lines = [
        "| run | sleeps begun in the busiest hour | clock hours 10-90 % asleep | asleep at peak / trough | sleeps interrupted | exchanges aimed at a sleeper | friends | rivals | insults | rumor reach |",
        "|---|---|---|---|---|---|---|---|---|---|",
    ]

    def row(label, s):
        sm = s["summary"]
        interrupted = 100.0 * s["interrupted"] / max(s["completed"] + s["interrupted"], 1)
        return (
            f"| {label} | {s['busiest_hour_share']:.0f} % | {s['mixed_hours']} | {s['peak']:.0f} % / {s['trough']:.0f} % "
            f"| {s['interrupted']} of {s['completed'] + s['interrupted']} ({interrupted:.0f} %) "
            f"| {s['slept_on']} of {s['exchanges']} | {sm['friends']} | {sm['rivals']} | {sm['insults']} | {sm['rumor_reach']} |"
        )

    for key in keys:
        lines.append(row(f"{NAMES[key]}, before", stats(find(before_dir, key))))
        lines.append(row(f"{NAMES[key]}, after", stats(find(after_dir, key))))
        variants = run / "evidence" / "variants"
        for name, label in (("sleepers-open", "body clocks, sleepers can be approached"), ("no-clocks", "no body clocks, sleepers cannot be approached")):
            path = variants / f"{name}-{key}"
            if (path / "sleep.txt").exists():
                lines.append(row(f"{NAMES[key]}, {label}", stats(path)))
    table_md.write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
