#!/usr/bin/env python3
"""Build the bevy_society social-psychology corpus in home-still (stdlib only).

  search    run every sweep query through `hs paper search`, screen titles, score, select
            300 papers (seeds + round-robin sweep) and write docs/research/corpus/candidate-pool.tsv
  download  `hs paper download --doi` for every selected DOI (resumable); writes cache/downloads.json
  report    write acquired / paywalled / unconvertible / awaiting-conversion / failed-transient TSVs
            and summary.json from the pool, cache/downloads.json and cache/catalog-state.json

hs runs with a temporary HOME (cache/hs-home) whose .home-still/config.yaml points NATS at
192.168.1.103, because big_mac cannot resolve the hostname `three`. The real ~/.home-still is
never modified: every entry is symlinked except config.yaml, which is copied with the replacement.
"""

from __future__ import annotations

import argparse
import concurrent.futures as cf
import datetime as dt
import hashlib
import json
import math
import os
import re
import subprocess
import threading
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CORPUS = ROOT / "tools" / "corpus"
CACHE = CORPUS / "cache"
OUT = ROOT / "docs" / "research" / "corpus"
POOL = OUT / "candidate-pool.tsv"
DOWNLOADS = CACHE / "downloads.json"
CATALOG_STATE = CACHE / "catalog-state.json"

TARGET_TOTAL = 300
PROBE_QUERY = "bounded confidence opinion dynamics"
PROVIDERS = ["crossref", "semanticscholar", "arxiv", "europepmc"]
WORKERS = 4
RETRIES = 3
RETRY_SLEEP_S = 30
NATS_FROM = "nats://three:4222"
NATS_TO = "nats://192.168.1.103:4222"

PAYWALL = re.compile(r"No open-access PDF found|No PDF could be downloaded", re.I)
TRANSIENT = re.compile(
    r"timed? ?out|timeout|HTTP (429|5\d\d)|status (429|5\d\d)|rate.?limit|"
    r"connection (reset|refused|error)|temporarily|\bNATS\b|\bDNS\b|event bus",
    re.I,
)
DONE = {"downloaded", "skipped_existing", "paywalled"}

MODULE_ORDER = [
    "personality-mood",
    "needs-utility",
    "relationships",
    "opinion-influence",
    "rumor-reputation",
    "groups-status",
    "game-social-ai",
]
MODULE_INDEX = {m: i for i, m in enumerate(MODULE_ORDER)}
SCREEN_PATTERNS = {
    "personality-mood": r"emotion|affect|mood|personality|apprais|feeling",
    "needs-utility": r"utility|need|motivat|behavio(u)?r selection|routine|decision",
    "relationships": r"friend|relationship|tie|balance|homophily|attraction|trust|familiar|network",
    "opinion-influence": r"opinion|influence|conform|consensus|contagion|polariz|attitude",
    "rumor-reputation": r"rumou?r|gossip|reputation|reciproc|cooperat|punish|trust",
    "groups-status": r"norm|group|identity|hierarch|status|dominan|segregat|coalition|faction",
    "game-social-ai": r"game|character|narrative|story|npc|drama|agent",
}
SCREEN = {m: re.compile(p, re.I) for m, p in SCREEN_PATTERNS.items()}

POOL_HEADER = ["doi", "module", "title", "year", "citations", "providers", "queries", "score", "selected"]
HS_HOME: Path | None = None


class HsError(RuntimeError):
    """An hs call failed, or the provider reported rate limiting."""


# ---------------------------------------------------------------- plumbing


def ensure_hs_home() -> Path:
    """Build cache/hs-home/.home-still pointing NATS at a reachable address; the real config is untouched."""
    home = CACHE / "hs-home"
    dot = home / ".home-still"
    real = Path.home() / ".home-still"
    dot.mkdir(parents=True, exist_ok=True)
    for entry in real.iterdir():
        if entry.name == "config.yaml":
            continue
        link = dot / entry.name
        if not (link.exists() or link.is_symlink()):
            link.symlink_to(entry)
    text = (real / "config.yaml").read_text()
    if NATS_FROM not in text:
        raise SystemExit(f"{real / 'config.yaml'} does not name {NATS_FROM}; the hs-home workaround is obsolete")
    cfg = dot / "config.yaml"
    cfg.unlink(missing_ok=True)
    cfg.write_text(text.replace(NATS_FROM, NATS_TO))
    os.chmod(cfg, 0o600)
    return home


def hs(args: list[str], timeout: float = 600) -> tuple[int, str, str]:
    env = dict(os.environ, HOME=str(HS_HOME))
    try:
        proc = subprocess.run(["hs", *args], capture_output=True, text=True, timeout=timeout, env=env)
    except subprocess.TimeoutExpired:
        return 124, "", f"timeout after {timeout}s"
    return proc.returncode, proc.stdout, proc.stderr


def parse_json(text: str) -> dict:
    return json.loads(text[text.find("{"):])


def load_json(path: Path, default):
    return json.loads(path.read_text()) if path.exists() else default


def save_json(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(path.name + ".tmp")
    tmp.write_text(json.dumps(obj, indent=2, sort_keys=True) + "\n")
    os.replace(tmp, path)


def clean(value) -> str:
    text = re.sub(r"<[^>]+>", "", str(value or ""))
    return re.sub(r"\s+", " ", text).replace("\t", " ").strip()


def norm_doi(value: str) -> str:
    value = (value or "").strip()
    value = re.sub(r"^(https?://(dx\.)?doi\.org/|doi:)", "", value, flags=re.I)
    return value.lower()


def read_tsv(path: Path) -> list[dict]:
    with path.open() as fh:
        header = fh.readline().rstrip("\n").split("\t")
        rows = []
        for line in fh:
            cells = line.rstrip("\n").split("\t")
            if len(cells) != len(header):
                raise SystemExit(f"{path}: expected {len(header)} columns, got {len(cells)}: {line[:80]!r}")
            rows.append(dict(zip(header, (c.strip() for c in cells))))
    return rows


def write_tsv(path: Path, header: list[str], rows: list[list]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = ["\t".join(header)]
    for row in rows:
        lines.append("\t".join(clean(cell) for cell in row))
    path.write_text("\n".join(lines) + "\n")


def read_seeds() -> list[dict]:
    seeds = read_tsv(CORPUS / "seeds.tsv")
    for s in seeds:
        if s["module"] not in MODULE_INDEX:
            raise SystemExit(f"seeds.tsv: unknown module {s['module']!r}")
    return seeds


def read_queries() -> list[dict]:
    counter: dict[str, int] = {}
    out = []
    for row in read_tsv(CORPUS / "queries.tsv"):
        module = row["module"]
        if module not in MODULE_INDEX:
            raise SystemExit(f"queries.tsv: unknown module {module!r}")
        counter[module] = counter.get(module, 0) + 1
        out.append({"module": module, "query": row["query"], "n": counter[module]})
    return out


# ---------------------------------------------------------------- search


def cache_file(kind: str, key: str) -> Path:
    folder = CACHE / kind
    folder.mkdir(parents=True, exist_ok=True)
    return folder / (hashlib.sha1(key.encode()).hexdigest()[:20] + ".json")


def hs_search(provider: str, query: str, n: int, typ: str | None = None) -> tuple[list[dict], bool]:
    """Return (papers, ok). ok is False when the provider reported failures; such responses are not cached."""
    key = json.dumps([provider, query, n, typ])
    path = cache_file("search", key)
    if path.exists():
        return json.loads(path.read_text())["papers"], True
    args = ["paper", "search", query, "-p", provider, "-n", str(n), "--output", "json"]
    if typ:
        args += ["-t", typ]
    rc, out, err = hs(args, timeout=300)
    if rc != 0:
        raise HsError(f"{provider} rc={rc}: {(err or out).strip()[-300:]}")
    data = parse_json(out)
    failures = data.get("provider_failures") or []
    if "rate limited" in (err + json.dumps(failures)).lower():
        raise HsError(f"{provider} rate limited")
    papers = data.get("papers") or []
    ok = not failures
    if ok:
        path.write_text(json.dumps({"papers": papers}))
    return papers, ok


def normalize(p: dict) -> dict:
    year = str(p.get("publication_date") or p.get("publication_year") or p.get("year") or "")[:4]
    try:
        citations = int(p.get("cited_by_count", p.get("citations", 0)) or 0)
    except (TypeError, ValueError):
        citations = 0
    return {
        "doi": norm_doi(p.get("doi") or p.get("id") or ""),
        "title": clean(p.get("title")),
        "year": year if re.fullmatch(r"\d{4}", year) else "",
        "citations": citations,
    }


def new_cand() -> dict:
    return {"title": "", "year": "", "citations": 0, "providers": set(), "queries": set(), "hits": {}}


def take(cands: dict, row: dict, provider: str, qid: str, module: str, screen: bool) -> None:
    key = row["doi"]
    if not key or (screen and not SCREEN[module].search(row["title"])):
        return
    cand = cands.setdefault(key, new_cand())
    if screen:
        cand["hits"][module] = cand["hits"].get(module, 0) + 1
    cand["title"] = cand["title"] or row["title"]
    cand["year"] = cand["year"] or row["year"]
    cand["citations"] = max(cand["citations"], row["citations"])
    cand["providers"].add(provider)
    cand["queries"].add(qid)


def score(cand: dict) -> float:
    return len(cand["providers"]) + math.log10(1 + cand["citations"])


def run_task(task: dict):
    try:
        papers, _ = hs_search(task["provider"], task["query"], task["n"], task["typ"])
        return task, papers, None
    except Exception as exc:  # one failed query must not stop the sweep; it is recorded
        return task, [], f"{type(exc).__name__}: {exc}"


def cmd_search(_args) -> None:
    global HS_HOME
    HS_HOME = ensure_hs_home()
    seeds = read_seeds()
    queries = read_queries()

    try:
        _, openalex_ok = hs_search("openalex", PROBE_QUERY, 5)
    except HsError as exc:
        openalex_ok = False
        print(f"openalex probe failed, not used: {exc}", flush=True)
    providers = PROVIDERS + (["openalex"] if openalex_ok else [])
    print(f"providers: {','.join(providers)}", flush=True)

    tasks = [
        {"kind": "seed", "module": s["module"], "qid": "seed", "query": s["doi"],
         "provider": "crossref", "n": 5, "typ": "doi"}
        for s in seeds
    ]
    for q in queries:
        for provider in providers:
            tasks.append({"kind": "sweep", "module": q["module"], "qid": f"{q['module']}#{q['n']}",
                          "query": q["query"], "provider": provider, "n": 25, "typ": None})

    with cf.ThreadPoolExecutor(WORKERS) as pool:
        results = list(pool.map(run_task, tasks))

    cands: dict[str, dict] = {}
    for s in seeds:
        cands[norm_doi(s["doi"])] = new_cand()

    failures = []
    for task, papers, err in results:
        if err:
            failures.append({"module": task["module"], "qid": task["qid"], "provider": task["provider"],
                             "query": task["query"], "error": err[:300]})
            continue
        for p in papers:
            row = normalize(p)
            if task["kind"] == "seed":
                if row["doi"] == norm_doi(task["query"]):
                    take(cands, row, task["provider"], "seed", task["module"], screen=False)
            else:
                take(cands, row, task["provider"], task["qid"], task["module"], screen=True)

    local_missing = []
    for q in queries:
        path = CACHE / "local-openalex" / f"{q['module']}-{q['n']}.json"
        if not path.exists():
            local_missing.append(path.name)
            continue
        data = json.loads(path.read_text())
        papers = data["papers"] if isinstance(data, dict) and "papers" in data else data
        for p in papers:
            take(cands, normalize(p), "local-openalex", f"{q['module']}#{q['n']}", q["module"], screen=True)
    if local_missing:
        print(f"local-openalex files missing for {len(local_missing)} queries (run the MCP dump first)", flush=True)

    seed_keys = {norm_doi(s["doi"]): s for s in seeds}
    sweep_by_module: dict[str, list[tuple[str, dict]]] = {m: [] for m in MODULE_ORDER}
    for key, cand in cands.items():
        if key in seed_keys or not cand["hits"]:
            continue
        module = max(cand["hits"].items(), key=lambda kv: (kv[1], -MODULE_INDEX[kv[0]]))[0]
        cand["module"] = module
        sweep_by_module[module].append((key, cand))
    for module in MODULE_ORDER:
        sweep_by_module[module].sort(key=lambda kc: (-score(kc[1]), kc[0]))

    target = TARGET_TOTAL - len(seeds)
    chosen: list[tuple[str, dict]] = []
    pointer = {m: 0 for m in MODULE_ORDER}
    while len(chosen) < target:
        progressed = False
        for module in MODULE_ORDER:
            if len(chosen) >= target:
                break
            if pointer[module] < len(sweep_by_module[module]):
                chosen.append(sweep_by_module[module][pointer[module]])
                pointer[module] += 1
                progressed = True
        if not progressed:
            break
    if len(chosen) < target:
        print(f"warning: only {len(chosen)} of {target} sweep slots could be filled", flush=True)
    chosen_keys = {key for key, _ in chosen}

    rows = []
    for s in seeds:
        cand = cands[norm_doi(s["doi"])]
        rows.append([s["doi"], s["module"], cand["title"], cand["year"], cand["citations"],
                     ",".join(sorted(cand["providers"])), len(cand["queries"]), f"{score(cand):.4f}", 1])
    for key, cand in chosen:
        rows.append([key, cand["module"], cand["title"], cand["year"], cand["citations"],
                     ",".join(sorted(cand["providers"])), len(cand["queries"]), f"{score(cand):.4f}", 1])
    for module in MODULE_ORDER:
        for key, cand in sweep_by_module[module]:
            if key not in chosen_keys:
                rows.append([key, module, cand["title"], cand["year"], cand["citations"],
                             ",".join(sorted(cand["providers"])), len(cand["queries"]), f"{score(cand):.4f}", 0])
    write_tsv(POOL, POOL_HEADER, rows)

    missing_titles = [s["doi"] for s in seeds if not cands[norm_doi(s["doi"])]["title"]]
    print(f"candidates (pool rows): {len(rows)}; selected: {len(seeds)} seeds + {len(chosen)} sweep "
          f"= {len(seeds) + len(chosen)}", flush=True)
    for module in MODULE_ORDER:
        n_sweep = sum(1 for key, _ in chosen if cands[key]["module"] == module)
        print(f"  {module}: seeds {sum(1 for s in seeds if s['module'] == module)}, "
              f"sweep selected {n_sweep} of {len(sweep_by_module[module])} screened", flush=True)
    print(f"search failures: {len(failures)} (cache/search-failures.json); seeds without a crossref title: "
          f"{len(missing_titles)}", flush=True)
    save_json(CACHE / "search-failures.json", failures)


# ---------------------------------------------------------------- download


def fetch_one(doi: str, module: str) -> dict:
    stem_guess = doi.lower().replace("/", "_")
    err = ""

    def result(stem: str, status: str, error: str) -> dict:
        return {"doi": doi, "module": module, "stem": stem, "status": status, "error": error}

    for attempt in range(RETRIES + 1):
        rc, out, errtxt = hs(["paper", "download", "--doi", doi, "--output", "json", "-y"], timeout=900)
        text = (out + "\n" + errtxt).strip()
        if rc == 0:
            try:
                info = parse_json(out)
            except ValueError:
                info = {}
            file_path = info.get("file_path")
            if file_path:
                status = "skipped_existing" if info.get("skipped") else "downloaded"
                return result(Path(file_path).stem.lower(), status, "")
            err = "rc=0 without file_path: " + text[-300:]
        else:
            err = text[-400:]
        if PAYWALL.search(text) and not TRANSIENT.search(text):
            return result(stem_guess, "paywalled", err)
        if attempt < RETRIES:
            time.sleep(RETRY_SLEEP_S)
    return result(stem_guess, "failed-transient", err)


def cmd_download(_args) -> None:
    global HS_HOME
    HS_HOME = ensure_hs_home()
    pool = read_tsv(POOL)
    selected = [r for r in pool if r["selected"] == "1"]
    state: dict = load_json(DOWNLOADS, {})
    todo = [r for r in selected if state.get(r["doi"], {}).get("status") not in DONE]
    print(f"{len(selected)} selected, {len(selected) - len(todo)} already done, {len(todo)} to fetch", flush=True)

    lock = threading.Lock()

    def work(row: dict) -> None:
        res = fetch_one(row["doi"], row["module"])
        with lock:
            state[row["doi"]] = res
            save_json(DOWNLOADS, state)
            print(f"{res['status']:>16}  {row['module']:<18} {row['doi']}", flush=True)

    with cf.ThreadPoolExecutor(WORKERS) as pool_exec:
        list(pool_exec.map(work, todo))

    tally: dict[str, int] = {}
    for row in selected:
        status = state.get(row["doi"], {}).get("status", "not-fetched")
        tally[status] = tally.get(status, 0) + 1
    print("tally:", json.dumps(tally, sort_keys=True), flush=True)


# ---------------------------------------------------------------- report


def refusal_reason(value) -> str:
    return value if isinstance(value, str) and value else "refused"


def cmd_report(_args) -> None:
    pool = read_tsv(POOL)
    seeds = read_seeds()
    queries = read_queries()
    downloads: dict = load_json(DOWNLOADS, {})
    if CATALOG_STATE.exists():
        catalog = load_json(CATALOG_STATE, {})
    else:
        catalog = {}
        print("warning: cache/catalog-state.json missing; converted and embedded counted as 0", flush=True)

    seed_keys = {norm_doi(s["doi"]) for s in seeds}
    acquired, awaiting, paywalled, unconvertible, failed = [], [], [], [], []
    placement: dict[str, list[str]] = {}
    stored_states = []
    status_counts: dict[str, int] = {}

    for row in pool:
        if row["selected"] != "1" or row["doi"] not in downloads:
            continue
        entry = downloads[row["doi"]]
        doi, module, stem, status = row["doi"], row["module"], entry["stem"], entry["status"]
        state = catalog.get(stem, {})
        status_counts[status] = status_counts.get(status, 0) + 1
        if status in ("downloaded", "skipped_existing"):
            converted = bool(state.get("converted"))
            embedded = bool(state.get("embedded"))
            stored_states.append((converted, embedded))
            if state.get("refused"):
                unconvertible.append([doi, module, stem, refusal_reason(state.get("refused"))])
                placement.setdefault(norm_doi(doi), []).append("unconvertible")
            else:
                acquired.append([doi, module, stem, status, int(converted), int(embedded)])
                placement.setdefault(norm_doi(doi), []).append("acquired")
                if not converted:
                    awaiting.append([doi, module, stem, int(norm_doi(doi) in seed_keys)])
        elif status == "paywalled":
            in_corpus = bool(state.get("converted") or state.get("embedded"))
            paywalled.append([doi, module, stem, int(in_corpus)])
            placement.setdefault(norm_doi(doi), []).append("paywalled")
        elif status == "failed-transient":
            failed.append([doi, module, entry.get("error", "")[:200]])
            placement.setdefault(norm_doi(doi), []).append("failed-transient")

    problems = []
    for seed in seeds:
        places = placement.get(norm_doi(seed["doi"]), [])
        if len(places) != 1:
            problems.append(f"{seed['doi']}: {places or 'not downloaded'}")
    if problems:
        print("seed placement violations:", *problems, sep="\n  ", flush=True)

    write_tsv(OUT / "acquired.tsv", ["doi", "module", "stem", "download", "converted", "embedded"], acquired)
    write_tsv(OUT / "paywalled.tsv", ["doi", "module", "stem", "already_in_corpus"], paywalled)
    write_tsv(OUT / "unconvertible.tsv", ["doi", "module", "stem", "reason"], unconvertible)
    write_tsv(OUT / "awaiting-conversion.tsv", ["doi", "module", "stem", "seed"], awaiting)
    write_tsv(OUT / "failed-transient.tsv", ["doi", "module", "error"], failed)

    summary = {
        "queries": len(queries),
        "candidates": len(pool),
        "selected": sum(1 for r in pool if r["selected"] == "1"),
        "downloaded": status_counts.get("downloaded", 0),
        "skipped_existing": status_counts.get("skipped_existing", 0),
        "paywalled": len(paywalled),
        "unconvertible": len(unconvertible),
        "converted": sum(1 for c, _ in stored_states if c),
        "embedded": sum(1 for _, e in stored_states if e),
        "awaiting_conversion": len(awaiting),
        "failed_transient": len(failed),
        "generated_at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds"),
    }
    (OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2), flush=True)
    if problems:
        raise SystemExit(2)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="cmd", required=True)
    sub.add_parser("search", help="search, screen, score and write candidate-pool.tsv").set_defaults(fn=cmd_search)
    sub.add_parser("download", help="download every selected DOI (resumable)").set_defaults(fn=cmd_download)
    sub.add_parser("report", help="write deliverable TSVs and summary.json").set_defaults(fn=cmd_report)
    args = parser.parse_args()
    args.fn(args)


if __name__ == "__main__":
    main()
