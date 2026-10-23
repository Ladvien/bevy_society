# Social-psychology corpus (home-still)

Snapshot: 2026-10-23 21:29 UTC (`summary.json`, `generated_at`). Partial: conversion and embedding are still in progress and are not complete.

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
| unconvertible | 0 |
| failed_transient | 7 |
| converted | 19 |
| embedded | 8 |
| awaiting_conversion | 68 |

Downloaded + skipped_existing + paywalled + failed_transient = 300. Every selected DOI is in exactly one of `acquired.tsv`, `paywalled.tsv`, `failed-transient.tsv`.

## Files
- `acquired.tsv`: stored PDFs (downloaded or already stored), with conversion and embedding state.
- `awaiting-conversion.tsv`: the acquired subset not yet converted.
- `paywalled.tsv`: no open-access PDF found. Rows carry `already_in_corpus`.
- `unconvertible.tsv`: empty (no refusals recorded).
- `failed-transient.tsv`: timeouts or rate limits that failed after 3 retries.
- `candidate-pool.tsv`: all screened candidates with scores and the `selected` flag.

## Known gaps
- 96 selected seed rows have empty titles (Crossref and Semantic Scholar rate-limited). Selection and downloads depend only on DOIs. Titles can be filled from cache.
- Scribe on big is VRAM-gated (`backend_unavailable`). Conversion runs on bmb. Awaiting counts are reported as they stood at the snapshot.
- `hs` cannot resolve `nats://three:4222` from big_mac. The script builds a temp HOME in `tools/corpus/cache/hs-home` that points NATS at `192.168.1.103:4222`. The real config is untouched.

## Using the corpus
- Search: `distill_search` (semantic, per chunk) or `abstract_search` (per paper).
- Read: `markdown_read` with the stem (for example `10.48550_arxiv.2304.03442`).

## Re-running the conversion probe
The probe is an omp eval cell (MCP tools are not reachable from the script). It writes `tools/corpus/cache/catalog-state.json`. The cell is not included in this snapshot; the `report` subcommand reads its output.
