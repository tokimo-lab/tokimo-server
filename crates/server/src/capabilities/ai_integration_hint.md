# Integrating tokimo-server with AI agents

This service is a **read-mostly API proxy + cache**. All providers below take simple HTTP GET with query/path params and return JSON.

## How to call

```
curl -H "Authorization: Bearer <service_key>" \
  https://<host>/api/tmdb/movie/550
```

## Tips for AI agents

- Always check `stats_24h.availability` before relying on a provider. If `down` or `degraded`, fall back to alternates listed in the same `category`.
- Responses include cache info via `X-Cache: HIT|MISS` and `ETag` for conditional requests.
- Hot-search aggregator: use `/api/hot/list?id=<source>` with one of the 19 source ids; results are pre-warmed every ~5 min.
- Errors follow `{"error": "..."}` with HTTP status reflecting upstream state (4xx for client / 5xx for upstream).
- Negative cache: 4xx/5xx are cached briefly (Cache-Control: public, max-age=30~60), so transient upstream failures won't immediately re-hit.

## When to use this service vs going to upstream directly

- ✅ Use it when: you want caching, when upstream is geo-restricted, when you need uniform auth.
- ❌ Don't use it when: you need write/mutation operations (this service is read-only).

## Provider category overview

See `categories` for groupings. Use the same category to find a fallback when one provider degrades.
