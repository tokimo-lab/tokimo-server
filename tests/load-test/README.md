# Load test

Mixed cache-hit / cache-miss load tester for tokimo-server.

Pure Node (>=18, uses built-in `fetch` + `AbortController`). No deps to install.

## Run

```bash
node tests/load-test/load-test.mjs
```

Defaults: 500 requests · 16 concurrent · 70% repeat (cache-hit) / 30% unique (cache-miss) · target `http://localhost:5680` · service key `tks_0qcWQe7cIigPMxnpHfojnVjOAZcMwuU8`.

Override via env:

```bash
BASE_URL=http://localhost:5680 \
SERVICE_KEY=tks_xxx \
TOTAL=1000 CONCURRENCY=32 REPEAT_RATIO=0.8 \
  node tests/load-test/load-test.mjs
```

## What it does

For each of 23 representative provider endpoints (tmdb / omdb / thetvdb / bangumi / douban / fanart / spotify / musicbrainz / deezer / lrclib / wikipedia / qidian / openmeteo / nominatim / geocoding / holiday / github / currency / assrt / hot / sports), the planner draws either:

- **repeat** — pick a URL from a small fixed pool (3 entries). After the first miss, subsequent hits should land in DB cache.
- **unique** — generate a URL with a random param so the cache always misses.

The 500 requests are shuffled across all endpoints so the load is interleaved.

## Output

- Per-endpoint table (total / ok / err / repeat / unique / p50 / p95)
- Overall (total / duration / rps / p50 / p95 / p99)
- Open the admin Dashboard to verify cache hit ratio / error rate / latency charts respond.

## Notes

- A few endpoints (omdb / thetvdb / spotify / fanart / assrt / etc.) need their API key in env to actually succeed — without keys those rows will mostly show `err`. Cache hit metrics still get recorded for the second-and-later identical requests because the server caches the upstream error response too (per endpoint policy).
- This is a smoke / observability tool, not a benchmark.
