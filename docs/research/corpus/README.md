# Social-psychology corpus (home-still)

Snapshot: 2026-10-23 23:44 UTC (`summary.json`, `generated_at`). Partial: 63 of the 86 acquired papers still await conversion, and 22 of them are embedded.

## Scope
- 300 papers selected from 1480 candidates: 99 seeds from `tools/corpus/seeds.tsv` (the plan's DOI list) plus 201 sweep papers from 42 queries in `tools/corpus/queries.tsv` (six per module).
- Candidates came from crossref, semanticscholar, arxiv, europepmc, and the local OpenAlex MCP index. The OpenAlex web API was rate-limited and skipped.
- Generator: `tools/corpus/build_corpus.py` (`search`, `download`, `report`). Caches under `tools/corpus/cache/` are git-ignored.

## Stage counts (`summary.json`)
| stage | count |
|---|---|
| queries | 42 |
| candidates | 1480 |
| selected | 300 |
| downloaded | 78 |
| skipped_existing | 9 |
| paywalled | 206 |
| unconvertible | 1 |
| failed_transient | 7 |
| converted | 23 |
| embedded | 22 |
| awaiting_conversion | 63 |

Downloaded + skipped_existing + paywalled + failed_transient = 300. Every selected DOI is in exactly one of `acquired.tsv`, `paywalled.tsv`, `unconvertible.tsv`, `failed-transient.tsv`.

## Files
- `acquired.tsv`: stored PDFs (downloaded or already stored), with conversion and embedding state.
- `awaiting-conversion.tsv`: the acquired subset not yet converted.
- `paywalled.tsv`: no open-access PDF found. Rows carry `already_in_corpus`.
- `unconvertible.tsv`: one seed refused by scribe, `10.1038/nature04131` (permanent VLM repetition loop, 17 truncations). Refused seeds are not in `awaiting-conversion.tsv`.
- `failed-transient.tsv`: timeouts or rate limits that failed after 3 retries.
- `candidate-pool.tsv`: all screened candidates with scores and the `selected` flag.

## Known gaps
- 96 selected seed rows have empty titles (Crossref and Semantic Scholar rate-limited). Selection and downloads depend only on DOIs. Titles can be filled from cache.
- Scribe state as seen by the final probe run (`system_status`): `192.168.1.110:7435` (big) healthy and idle with 9 of 12 slots free; `bmb.local:7433` healthy with 3 converting and 3 of 6 slots free; `pipeline.queued_conversions` 3164. No `backend_unavailable` response was seen, so the 30-minute stop rule did not fire.
- `hs` cannot resolve `nats://three:4222` from big_mac. The script builds a temp HOME in `tools/corpus/cache/hs-home` that points NATS at `192.168.1.103:4222`. The real config is untouched.
- `scribe_convert` returns `already_converted: true` with a `markdown_key` for `10.1073_pnas.0903616106` and `10.1146_annurev.psych.54.101601.145059`, but the catalog has no `conversion` block for either (`annurev` has a `markdown_path`, `pnas` does not). The probe counts both as converted. The missing catalog stamp is a home-still gap, not fixed here.

## Using the corpus
- Search: `distill_search` (semantic, per chunk) or `abstract_search` (per paper).
- Read: `markdown_read` with the stem (for example `10.48550_arxiv.2304.03442`).

## Re-running the conversion probe
The probe is an omp eval cell (Python; MCP tools are reachable only from inside the eval kernel). Run it from the repo root. It writes `tools/corpus/cache/catalog-state.json` (`stem → {converted, embedded, refused}`). A stem counts as converted when the catalog has a `conversion` block, a `markdown_path`, or `scribe_convert` answered `already_converted`. Then run `python3 tools/corpus/build_corpus.py report`.

```python
import asyncio, json, os, pathlib
from datetime import datetime, timedelta, timezone

os.chdir(os.environ.get("CORPUS_ROOT", os.getcwd()))
ROOT = pathlib.Path("tools/corpus")
CACHE = ROOT / "cache"
downloads = json.loads((CACHE / "downloads.json").read_text())
seed_dois = {l.split("\t")[1].strip().lower()
             for l in (ROOT / "seeds.tsv").read_text().splitlines()[1:] if l.strip()}

def parse(resp):
    if resp.get("hasError"):
        return None
    try:
        return json.loads(resp["text"])
    except (ValueError, KeyError):
        return None

stems = {}
for doi, entry in downloads.items():
    stem = entry.get("stem") or doi.lower().replace("/", "_")
    stems[stem] = (doi.lower() in seed_dois,
                   entry.get("status") in ("downloaded", "skipped_existing"))

gate = asyncio.Semaphore(8)
convert_sem = asyncio.Semaphore(2)

async def limited(coro):
    async with gate:
        return await coro

async def read(stem):
    return parse(await tool.mcp__home_still_catalog_read({"stem": stem}))

async def indexed(stem):
    r = parse(await tool.mcp__home_still_distill_exists({"doc_id": stem}))
    return bool(r and r.get("indexed"))

names = list(stems)
entries = dict(zip(names, await asyncio.gather(*(limited(read(s)) for s in names))))

st = {"since": None, "stop": False}
outcomes = {"ok": 0, "already_converted": 0, "backend_unavailable": 0,
            "other_error": 0, "skipped_after_stop": 0}
refused, markdown_present, sample_errors = {}, set(), {}

async def convert(stem):
    if st["stop"]:
        outcomes["skipped_after_stop"] += 1
        return
    async with convert_sem:
        if st["stop"]:
            outcomes["skipped_after_stop"] += 1
            return
        r = await tool.mcp__home_still_scribe_convert({"stem": stem})
        text = r.get("text", "") or ""
        now = datetime.now(timezone.utc)
        if r.get("hasError") and "backend_unavailable" in text:
            outcomes["backend_unavailable"] += 1
            sample_errors.setdefault("backend_unavailable", text.splitlines()[0][:200])
            st["since"] = st["since"] or now
            if now - st["since"] >= timedelta(minutes=30):
                st["stop"] = True
            return
        st["since"] = None
        if r.get("hasError"):
            outcomes["other_error"] += 1
            first = text.splitlines()[0][:200] if text else ""
            sample_errors.setdefault(stem, first)
            if "refused" in text or "permanent:" in text:
                refused[stem] = first
            return
        body = parse(r) or {}
        if body.get("already_converted"):
            outcomes["already_converted"] += 1
            markdown_present.add(stem)
        else:
            outcomes["ok"] += 1

need = [s for s, (is_seed, acq) in stems.items()
        if is_seed and acq and not (entries.get(s) or {}).get("conversion")]
await asyncio.gather(*(convert(s) for s in need))
for s in need:
    entries[s] = await read(s)

flags = dict(zip(names, await asyncio.gather(*(limited(indexed(s)) for s in names))))

state = {}
for stem in sorted(names):
    entry = entries.get(stem) or {}
    state[stem] = {
        "converted": bool(entry.get("conversion") or entry.get("markdown_path") or stem in markdown_present),
        "embedded": flags[stem],
        "refused": refused.get(stem, False),
    }
(CACHE / "catalog-state.json").write_text(json.dumps(state, indent=2, sort_keys=True) + "\n")

status = parse(await tool.mcp__home_still_system_status({})) or {}
print(json.dumps({
    "stems": len(state),
    "converted": sum(v["converted"] for v in state.values()),
    "embedded": sum(v["embedded"] for v in state.values()),
    "refused": sum(1 for v in state.values() if v["refused"]),
    "convert_outcomes": outcomes,
    "sample_errors": sample_errors,
    "stopped_backend_unavailable_30m": st["stop"],
    "scribe": status.get("scribe_instances"),
    "pipeline_queued_conversions": (status.get("pipeline") or {}).get("queued_conversions"),
}, indent=2))
```

The cell ran with the clone root as its working directory; the only later changes are the `os.chdir` default and the final `scribe` print, which shows the raw `scribe_instances`.

## Seed-bucket check
All 99 seeds in `seeds.tsv` appear in exactly one of `acquired.tsv`, `paywalled.tsv`, `unconvertible.tsv` or `failed-transient.tsv`: 0 misplaced. The bucket counts are 23 acquired, 75 paywalled, 1 unconvertible and 0 failed-transient. `report` agrees and exits 0.

## Done-criterion
`distill_search "bounded confidence opinion dynamics"` (limit 10, metadata only), re-run at commit time. Distinct doc_ids in rank order:

1. `10.48550_arXiv.2005.12798` (score 0.54): not in `acquired.tsv`.
2. `10.18564_jasss.3521` (Flache 2017): in `acquired.tsv`.
3. `10.48550_arXiv.2606.04227`: not in `acquired.tsv`.
4. `10.1103_revmodphys.81.591` (Castellano 2009): in `acquired.tsv`.

Both target papers are in the top four. The top two doc_ids are one acquired paper and one that is not in `acquired.tsv`. Neither non-acquired hit is a seed.
