#!/usr/bin/env python3
"""Plot and tabulate `Tuning::kin_premium`: do relatives get the loaf, and does anything else move?

    /tmp/plotenv/bin/python tools/plot_kin.py <run-dir> <out-plot.png> <out-sweep.png> <out-table.md>

Reads `<run-dir>/evidence/{before,after}/<run>/{kin.txt,summary.json}` for each run that exists
(`before` is `--kin-premium 0`: the relatives exist and nobody favours them; `after` is the default premium), and
`<run-dir>/evidence/sweep/p*/` plus `before/v7` and `after/v7` for the premium sweep. `kin.txt` is the harness's
`KinLedger`: accepted shares by what the receiver is to the giver, the share of ordered pairs each relation is
(what a pick blind to kin would give it), and the bread variance within and between households.
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
    ("f3", "town 40 s3, of 3 + cousins"),
    ("c7", "village s7 + cousins"),
    ("c42", "village s42 + cousins"),
)
SWEEP = (("before/v7", 0.0), ("sweep/p05", 0.5), ("after/v7", 1.0), ("sweep/p2", 2.0), ("sweep/p4", 4.0))
OUTCOMES = ("friends", "rivals", "insults", "interrupted_endings", "shares")


def save(fig, out, dpi=110):
    """Save `fig` as a 64-colour PNG, so a committed plot stays under the 100 KB evidence limit."""
    fig.savefig(out, dpi=dpi)
    Image.open(out).convert("RGB").quantize(colors=64).save(out, optimize=True)


def kin(path):
    """Parse `kin.txt` into {relation: (shares, share_of_shares, share_of_pairs)} plus the bread variances."""
    text = pathlib.Path(path).read_text()
    rows = {}
    for line in text.splitlines()[2:]:
        m = re.match(r"(\S+)\s+(\d+)\s+([\d.]+)%\s+([\d.]+)%", line)
        if m:
            rows[m.group(1)] = (int(m.group(2)), float(m.group(3)), float(m.group(4)))
    v = re.search(r"within households ([\d.]+), between household means ([\d.]+), within / \(within \+ between\) ([\d.]+)", text)
    total = re.search(r"accepted shares: (\d+)", text)
    return {
        "rows": rows,
        "within": float(v.group(1)) if v else None,
        "between": float(v.group(2)) if v else None,
        "pooling": float(v.group(3)) if v else None,
        "shares": int(total.group(1)) if total else 0,
    }


def relatives(k):
    """(share of shares to any relative, share of ordered pairs that are relatives), in percent."""
    got = sum(v[1] for r, v in k["rows"].items() if r != "stranger")
    chance = sum(v[2] for r, v in k["rows"].items() if r != "stranger")
    return got, chance


def load(base, side, run):
    d = base / "evidence" / side / run
    if not (d / "kin.txt").exists():
        return None
    return {"kin": kin(d / "kin.txt"), "summary": json.loads((d / "summary.json").read_text())}


def main():
    base = pathlib.Path(sys.argv[1])
    out_plot, out_sweep, out_table = sys.argv[2:5]
    pairs = []
    for run, label in RUNS:
        b, a = load(base, "before", run), load(base, "after", run)
        if b and a:
            pairs.append((run, label, b, a))

    fig, axes = plt.subplots(1, 3, figsize=(17, 4.6))
    xs = range(len(pairs))
    w = 0.38
    ax = axes[0]
    ax.bar([x - w / 2 for x in xs], [relatives(p[2]["kin"])[0] for p in pairs], w, label="premium 0 (before)", color="#999999")
    ax.bar([x + w / 2 for x in xs], [relatives(p[3]["kin"])[0] for p in pairs], w, label="premium 1 (after)", color="#2a7f62")
    ax.plot(list(xs), [relatives(p[3]["kin"])[1] for p in pairs], "k_", markersize=26, mew=2, label="share of pairs that are kin")
    ax.set_xticks(list(xs))
    ax.set_xticklabels([p[1] for p in pairs], fontsize=7, rotation=35, ha="right")
    ax.set_ylabel("% of accepted shares given to a relative")
    ax.set_title("Who gets the loaf")
    ax.legend(fontsize=7)

    ax = axes[1]
    ax.bar([x - w / 2 for x in xs], [p[2]["kin"]["pooling"] for p in pairs], w, color="#999999")
    ax.bar([x + w / 2 for x in xs], [p[3]["kin"]["pooling"] for p in pairs], w, color="#2a7f62")
    ax.set_xticks(list(xs))
    ax.set_xticklabels([p[1] for p in pairs], fontsize=7, rotation=35, ha="right")
    ax.set_ylabel("bread variance within / (within + between households)")
    ax.set_title("Do households pool? (lower = more alike inside)")

    ax = axes[2]
    shown = [
        ("friends", "friend ties"),
        ("rivals", "rival ties"),
        ("insults", "insults"),
        ("interrupted_endings", "interrupted endings"),
    ]
    for i, (key, label) in enumerate(shown):
        ratios = [p[3]["summary"].get(key, 0) / max(p[2]["summary"].get(key, 0), 1) for p in pairs]
        ax.scatter([i] * len(ratios), ratios, color="#2a7f62", s=18)
        ax.hlines(sum(ratios) / max(len(ratios), 1), i - 0.3, i + 0.3, color="k")
    ax.axhline(1.0, color="#999999", lw=1)
    ax.set_xticks(range(len(shown)))
    ax.set_xticklabels([s[1] for s in shown], fontsize=8)
    ax.set_ylabel("after / before, one dot per run")
    ax.set_title("Does anything else move?")
    fig.tight_layout()
    save(fig, out_plot)

    sweep = []
    for rel, premium in SWEEP:
        side, run = rel.split("/")
        r = load(base, side, run)
        if r:
            sweep.append((premium, r))
    if sweep:
        fig, axes = plt.subplots(1, 2, figsize=(9, 3.6))
        axes[0].plot([s[0] for s in sweep], [relatives(s[1]["kin"])[0] for s in sweep], "o-", color="#2a7f62")
        axes[0].axhline(relatives(sweep[0][1]["kin"])[1], color="#999999", ls="--", label="share of pairs that are kin")
        axes[0].set_xlabel("Tuning::kin_premium")
        axes[0].set_ylabel("% of accepted shares to a relative")
        axes[0].legend(fontsize=7)
        axes[1].plot([s[0] for s in sweep], [s[1]["kin"]["pooling"] for s in sweep], "o-", color="#2a7f62")
        axes[1].set_xlabel("Tuning::kin_premium")
        axes[1].set_ylabel("within / (within + between)")
        fig.suptitle("village, seed 7, households of 4")
        fig.tight_layout()
        save(fig, out_sweep)

    lines = [
        "# Kin premium, `before` (`--kin-premium 0`) against `after` (default 1.0)\n",
        "| run | accepted shares | to a relative, before / after | share of pairs that are kin | within / (within + between) | friends | rivals | insults | interrupted |",
        "|---|---|---|---|---|---|---|---|---|",
    ]
    for run, label, b, a in pairs:
        gb, chance = relatives(b["kin"])
        ga, _ = relatives(a["kin"])
        s = lambda key: f"{b['summary'].get(key, 0)} / {a['summary'].get(key, 0)}"
        lines.append(
            f"| {label.replace(chr(10), ' ')} | {b['kin']['shares']} / {a['kin']['shares']} | {gb:.1f} % / **{ga:.1f} %** | {chance:.1f} % | "
            f"{b['kin']['pooling']:.3f} / {a['kin']['pooling']:.3f} | {s('friends')} | {s('rivals')} | {s('insults')} | {s('interrupted_endings')} |"
        )
    if sweep:
        lines += ["", "## Sweep (village, seed 7, households of 4)\n", "| premium | accepted shares | to a relative | within / (within + between) | friends | rivals |", "|---|---|---|---|---|---|"]
        for premium, r in sweep:
            g, _ = relatives(r["kin"])
            lines.append(
                f"| {premium} | {r['kin']['shares']} | {g:.1f} % | {r['kin']['pooling']:.3f} | {r['summary'].get('friends', 0)} | {r['summary'].get('rivals', 0)} |"
            )
    pathlib.Path(out_table).write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
