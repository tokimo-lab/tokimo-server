# tokimo-providers

Upstream third-party API adapters used by tokimo-server's cache+single-flight
layer. Each module wraps a single upstream (TMDB, OMDb, TheTVDB, Bangumi,
Fanart.tv, Douban, …) and exposes pure async functions returning
`CoreResult<T>`; persistence and rate-limiting live in the server crate.

## Source: Copy + Adapt from `tokimo-lab/tokimo`

The original provider clients live in the **private** monorepo at:

    tokimo/packages/rust-client-api/src/metadata_providers/<provider>.rs

Because tokimo-server is **public** and tokimo is **private**, we cannot add
tokimo as a git submodule or `[path]` cargo dep. Instead, for each provider
we **copy the relevant `.rs` file(s)** into this crate and adapt them.

### What to replace

| Upstream symbol | Replacement |
|---|---|
| `crate::error::ClientError` (rust-client-api) | `tokimo_core::CoreError` |
| `ClientError::Http(e)` | `CoreError::Upstream(e)` |
| `ClientError::Api { status, message }` | `CoreError::Provider(format!(...))` |
| `ClientError::NotFound` | `CoreError::NotFound` |
| `tokimo_web_fetch::*` (private cloudflare-aware fetcher) | plain `reqwest::Client` (drop CF challenge support — public service does not need it) |
| Internal helper types pulled in via `use crate::...` | inline-copy the minimum needed types into the provider module |

### What to keep verbatim

- Upstream URL templates and query-string conventions
- Anti-scrape headers (User-Agent, Referer, Cookie strings, …) — copy as-is,
  do **not** "improve"; behaviour drift is the #1 cause of silent breakage.
- Response parsing logic (regex, scraper selectors, JSON paths)
- Pagination / rate-limit hint handling

### What to add (per provider)

1. **DB migration** — `crates/migration/src/m20250101_0000XX_create_<provider>_<table>.rs`
2. **Sea-ORM entity** — `crates/server/src/db/entities/<provider>_*.rs`
3. **Provider module** — `crates/providers/src/<provider>.rs` with `pub async fn fetch_*`
4. **Route handler** — `crates/server/src/routes/<provider>.rs` following
   `routes/tmdb.rs`: fast-path DB check → `rate_limiter.acquire(<bucket>)` →
   `single_flight.do_once(&cache_key, ...)` with re-check inside → upstream
   call → persist → respond.

## Why no submodule / no `[path]` dep

- tokimo is private; embedding it (or even a path reference) into a public
  crate's `Cargo.toml` would leak its existence and break `cargo publish`-style
  builds for downstream consumers of tokimo-server.
- Copy + adapt keeps each adapter self-contained and lets tokimo-server diverge
  freely (drop Cloudflare bits, add caching hooks, swap error types) without
  coordinating with the upstream monorepo.
