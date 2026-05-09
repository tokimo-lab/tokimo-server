# tokimo-server

An adapter, cache, and CDN-fronting service for third-party APIs (TMDB, Baidu Hot Search, Baidu Sports, and more). Provides normalized data persistence, single-flight request deduplication, rate limiting, and asset storage.

## Features

- 🔐 **Authentication**: HTTP Bearer token validation + Admin JWT
- 🚀 **Single-Flight**: Two-tier deduplication of concurrent identical requests — process-local (DashMap) + cross-process (PostgreSQL `pg_advisory_xact_lock`)
- 🌊 **Rate Limiting**: Token-bucket rate limiter persisted to PostgreSQL
- 💾 **Caching**: Database-backed caching with TTL
- 📦 **Asset Storage**: Local filesystem · S3-compatible (AWS S3 / MinIO) · Aliyun OSS
- 🎬 **31 Provider Adapters**: video metadata (TMDB, OMDb, TheTVDB, Bangumi, Fanart, Douban, JavBus, JavDB, ThePornDB, StashDB) · music (Spotify, MusicBrainz, Deezer, LRCLIB) · books (Qidian) · encyclopedia (Wikipedia) · geo/weather (Open-Meteo, Nominatim, Geocoding) · holidays (Timor + Nager) · subtitles (Assrt, OpenSubtitles, RegieLive, Gestdown) · releases (GitHub) · trending (Baidu Hot, Baidu Sports) · quotes (Hitokoto, ZenQuotes) · wallpaper (Bing) · currency (exchange rates)
- 🔥 **Hot Search Aggregator**: Multi-source trending topics (Weibo, Bilibili, Baidu, GitHub Trending, Hacker News, V2EX)

## Tech Stack

| Layer | Technologies |
|-------|-------------|
| Backend | Rust · Axum 0.7 · Sea-ORM 1.x · PostgreSQL 16 |
| Frontend | React 19 · Vite 6 · Antd 5 · TypeScript 5 · Biome |
| Infra | Docker · GitHub Actions |

## Architecture

```mermaid
graph TB
    Client[Client]
    Admin[Admin UI]
    
    subgraph Server["tokimo-server (Axum)"]
        Auth[Auth Middleware]
        Routes[Route Handlers]
        SF[Single Flight]
        RL[Rate Limiter]
        Cache[Cache Layer]
        Storage[Storage Layer]
    end
    
    subgraph Providers["Providers (31 adapters)"]
        Video[Video: TMDB · OMDb · TheTVDB · Bangumi · Fanart · Douban · JavBus · JavDB · ThePornDB · StashDB]
        Music[Music: Spotify · MusicBrainz · Deezer · LRCLIB]
        Geo[Geo/Weather: Open-Meteo · Nominatim · Geocoding · Holiday]
        Subs[Subtitles: Assrt · OpenSubtitles · RegieLive · Gestdown]
        Misc[Misc: Wikipedia · Qidian · GitHub Releases · Baidu Hot/Sports · Hitokoto · ZenQuotes · Bing Wallpaper · Currency]
    end
    
    DB[(PostgreSQL)]
    FS[Local Storage]
    
    Client -->|Bearer Token| Auth
    Admin -->|JWT| Auth
    Auth --> Routes
    Routes --> SF
    SF --> RL
    RL --> Providers
    Routes --> Cache
    Routes --> Storage
    Cache --> DB
    RL --> DB
    Storage --> FS
    Providers -->|Upstream| External[External APIs]
```

### Cross-process single-flight

Multi-instance deployments dedup concurrent identical requests in two tiers. The **inner** tier is a process-local `DashMap` that lets same-process callers wait on a single in-flight task with no PG round-trip. The **outer** tier wraps the local layer with a transaction-scoped PostgreSQL advisory lock (`pg_advisory_xact_lock(xxh3_64(key))`) so that across N instances only one process actually hits the upstream API. The lock auto-releases on transaction commit (or connection drop on panic), so it can never leak. Race contract: handlers MUST re-check the provider's persistent table or shared cache as the first action inside the single-flight closure — cross-process losers wake up after the lock is released and short-circuit on this re-check instead of re-running the upstream call.

### 跨进程 Single-Flight

多实例部署下采用两层去重。**内层**是进程内的 `DashMap`，让同一进程的并发调用合并为一次 in-flight 任务，零 PG 往返。**外层**在内层之上叠加 PostgreSQL `pg_advisory_xact_lock(xxh3_64(key))`，把跨 N 个进程的并发请求收敛到只有一个进程真正打上游。锁挂在一个专用事务里，事务 commit / 连接断开时自动释放，不会泄漏。竞态契约：handler 在 single-flight 闭包里的第一步**必须**重新查 provider 持久表或共享缓存——跨进程的"输家"在锁释放后醒来，依靠这次重查直接拿到结果，而不是再打一次上游。

## Provider Status

All 31 adapters below follow the same pattern: typed adapter → DB cache table → cross-process single-flight → rate-limited upstream call.

| Provider | Endpoints (representative) | Rate Limit | Auth |
|----------|---------------------------|------------|------|
| TMDB | `/api/tmdb/{movie,tv,season,episode,person,image}/...` | 10/s | `TMDB_API_KEY` |
| OMDb | `/api/omdb/...` | 10/s | `OMDB_API_KEY` |
| TheTVDB | `/api/thetvdb/...` | 10/s | `THETVDB_API_KEY` |
| Bangumi | `/api/bangumi/...` | 10/s | `BANGUMI_USER_AGENT` |
| Fanart | `/api/fanart/...` | 10/s | `FANART_API_KEY` |
| Douban | `/api/douban/...` | 1/s | scraping (no key) |
| JavBus | `/api/javbus/search?video_id=` | 5/s | `JAVBUS_BASE_URL` (optional `JAVBUS_COOKIE`) |
| JavDB | `/api/javdb/search?video_id=` | 5/s | `JAVDB_BASE_URL` (optional `JAVDB_COOKIE`) |
| ThePornDB | `/api/tpdb/search?video_id=` | 5/s | `TPDB_API_KEY` + `TPDB_BASE_URL` |
| StashDB | `/api/stashdb/search?video_id=` | 5/s | `STASHDB_BASE_URL` (optional `STASHDB_API_KEY`) |
| Spotify | `/api/spotify/...` | 30/s | `SPOTIFY_CLIENT_ID` + `SPOTIFY_CLIENT_SECRET` |
| MusicBrainz | `/api/musicbrainz/...` | 1/s | `MUSICBRAINZ_USER_AGENT` |
| Deezer | `/api/deezer/...` | 30/s | none |
| LRCLIB | `/api/lrclib/...` | 30/s | none |
| Qidian | `/api/qidian/book/:id`, `/api/qidian/search` | 1/s | scraping (no key) |
| Wikipedia | `/api/wikipedia/summary?title=&lang=` | 10/s | none |
| Open-Meteo | `/api/openmeteo/{forecast,air-quality}` | 100/s | none |
| Nominatim | `/api/nominatim/{search,reverse}` | **1/s** (TOS) | `NOMINATIM_USER_AGENT` |
| Geocoding | `/api/geocoding/{forward,reverse}` (composite) | 30/s | reuses Nominatim UA |
| Holiday | `/api/holiday/:country/:year` (Timor + Nager merged) | 10/s | none |
| Assrt | `/api/assrt/{search,sub/:id/detail}` | 10/s | `ASSRT_API_KEY` |
| OpenSubtitles | `/api/opensubtitles/search` | 10/s | `OPENSUBTITLES_API_KEY` |
| RegieLive | `/api/regielive/search` | 10/s | none (hardcoded Bazarr UA + key) |
| Gestdown | `/api/gestdown/{shows/search,subtitles}` | 10/s | none |
| GitHub Releases | `/api/github/releases/:owner/:repo/{latest,list}` | 30/s | optional `GITHUB_TOKEN` |
| Baidu Hot | `/api/hot/list?id=...` | per-source | none |
| Baidu Sports | `/api/sports/schedule?...` | 10/s | none |
| Hitokoto | `/api/hitokoto/sentence` | 10/s | none |
| ZenQuotes | `/api/zenquotes/random` | 10/s | none |
| Bing Wallpaper | `/api/bing/wallpaper` | 10/s | none |
| Currency | `/api/currency/rates` | 10/s | none |

## Environment Variables

Server / database / storage env vars are listed in [Configuration](#configuration). Below are the **provider auth** env vars; absent variables disable the corresponding routes (or fall back to anonymous mode for providers that support it).

| Variable | Required by | Notes |
|----------|-------------|-------|
| `TMDB_API_KEY` | required | TMDB v3 API key |
| `OMDB_API_KEY` | required | OMDb apikey |
| `THETVDB_API_KEY` | required | TheTVDB v4 API key (server exchanges for JWT) |
| `BANGUMI_USER_AGENT` | required | Bangumi requires a descriptive UA per their TOS |
| `FANART_API_KEY` | required | fanart.tv project API key |
| `SPOTIFY_CLIENT_ID` | required | Spotify app client id (client_credentials flow) |
| `SPOTIFY_CLIENT_SECRET` | required | paired with `SPOTIFY_CLIENT_ID` |
| `MUSICBRAINZ_USER_AGENT` | required | MusicBrainz requires a contact UA per their TOS |
| `NOMINATIM_USER_AGENT` | required | OSM Nominatim requires a contact UA; **also reused by `/api/geocoding`** |
| `ASSRT_API_KEY` | required | assrt.net subtitle API token |
| `OPENSUBTITLES_API_KEY` | required for `/api/opensubtitles` | OpenSubtitles consumer key — register at https://www.opensubtitles.com/en/consumers |
| `GITHUB_TOKEN` | optional | raises GitHub anonymous rate limit (60/h → 5000/h) |

## Quick Start

### Development (with Docker)

```bash
# 1. Start dev database
docker compose -f docker/docker-compose.dev.yml up -d

# 2. Copy and edit .env
cp .env.example .env
# Edit DATABASE_URL, TMDB_API_KEY, etc.

# 3. Run migrations
cargo run -p tokimo-migration -- up

# 4. Start server
cargo run -p tokimo-server

# 5. Build admin UI
cd admin
pnpm install
pnpm build
cd ..

# Access http://localhost:5680/admin
```

### Production (Docker Compose)

```bash
docker compose -f docker/docker-compose.yml up -d
```

## Authentication

### Admin Login

```bash
curl -X POST http://localhost:5680/api/admin/login \
  -H "Content-Type: application/json" \
  -d '{"bootstrap_key":"YOUR_BOOTSTRAP_KEY"}'
# Returns: {"token":"JWT_TOKEN"}
```

### Create Service Key

```bash
curl -X POST http://localhost:5680/api/admin/service-keys \
  -H "Authorization: Bearer JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"my-app"}'
# Returns: {"token":"tks_...","id":"..."}
```

### Use Service Key

```bash
curl http://localhost:5680/api/tmdb/movie/550 \
  -H "Authorization: Bearer tks_..."
```

## Configuration

| Environment Variable | Required | Default | Description |
|---------------------|----------|---------|-------------|
| `SERVER_LISTEN` | No | `0.0.0.0:5680` | Server listen address |
| `SERVER_PUBLIC_BASE_URL` | No | `http://localhost:5680` | Public URL for assets |
| `SERVER_ADMIN_BOOTSTRAP_KEY` | Yes | - | Admin bootstrap key |
| `SERVER_JWT_SECRET` | Yes | - | JWT signing secret |
| `SERVER_CORS_ALLOWED_ORIGINS` | No | `` (permissive) | Comma-separated CORS origins |
| `DATABASE_URL` | Yes | - | PostgreSQL connection string |
| `STORAGE_BACKEND` | No | `local` | `local` / `s3` / `oss` |
| `STORAGE_LOCAL_ROOT` | Yes (local) | `./storage` | Local storage root path |
| `STORAGE_LOCAL_PUBLIC_BASE` | Yes (local) | - | Public URL prefix for assets |
| `TMDB_API_KEY` | No | - | TMDB API key (required for TMDB endpoints) |
| `RUST_LOG` | No | `info,tokimo_server=debug,sqlx=warn` | Log level |

### Storage backends

Selected via `STORAGE_BACKEND`. DB always stores object **keys**; the URL is assembled at response time via `Storage::url_for(key)` (async).

| Backend | When to use | Required env vars |
|---|---|---|
| `local` | Single-node dev / self-hosted with reverse proxy | `STORAGE_LOCAL_ROOT`, `STORAGE_LOCAL_PUBLIC_BASE` |
| `s3` | AWS S3 / MinIO / any S3-compatible service | `STORAGE_S3_BUCKET`, `STORAGE_S3_REGION`, `STORAGE_S3_ACCESS_KEY_ID`, `STORAGE_S3_SECRET_ACCESS_KEY`, optional `STORAGE_S3_ENDPOINT` (omit for AWS), `STORAGE_S3_PUBLIC_BASE` (when public), `STORAGE_S3_PRESIGN_TTL_SECONDS` (default `0`) |
| `oss` | Aliyun OSS (S3-compatible protocol) | `STORAGE_OSS_BUCKET`, `STORAGE_OSS_REGION`, `STORAGE_OSS_ACCESS_KEY_ID`, `STORAGE_OSS_SECRET_ACCESS_KEY`, optional `STORAGE_OSS_ENDPOINT` (default `https://oss-cn-hangzhou.aliyuncs.com`), `STORAGE_OSS_PUBLIC_BASE`, `STORAGE_OSS_PRESIGN_TTL_SECONDS` |

`PRESIGN_TTL_SECONDS=0` ⇒ bucket is treated as public; `url_for` returns `{public_base}/{key}`. `>0` ⇒ bucket is private; `url_for` returns a presigned GET URL valid for that many seconds.

## GitHub Secrets

For CI workflows:

| Secret | Required For | Description |
|--------|-------------|-------------|
| `TMDB_API_KEY` | Live API tests | TMDB API key for integration tests |

## Contributing

### Adding a Provider

1. Create `crates/providers/src/my_provider.rs`
2. Implement fetching logic with error handling
3. Add route handler in `crates/server/src/routes/my_provider.rs`
4. Register route in `routes/mod.rs`
5. Add database migration if needed
6. Document in README provider status table

### Code Guidelines

- No `.unwrap()` / `.expect()` in non-test code
- Always propagate errors with `?`
- DB stores object **keys**, never URLs
- URLs assembled via `Storage::url_for(key).await` at response time
- Wrap upstream calls in `tracing::info_span!("upstream", provider=..., ...)`

---

## 中文文档

一个用于第三方 API（TMDB、百度热搜、百度体育等）的适配器、缓存和 CDN 前置服务。提供标准化数据持久化、单飞请求去重、速率限制和资源存储。

### 特性

- 🔐 **认证**：HTTP Bearer token 验证 + 管理员 JWT
- 🚀 **单飞机制**：去重并发的相同请求（进程内）
- 🌊 **速率限制**：令牌桶算法速率限制器，持久化到 PostgreSQL
- 💾 **缓存**：数据库支持的带 TTL 缓存
- 📦 **资源存储**：本地文件系统 · S3 兼容（AWS S3 / MinIO）· 阿里云 OSS
- 🎬 **TMDB 集成**：电影元数据 + 图片下载
- 🔥 **热搜聚合器**：多源热门话题（微博、B站、百度、GitHub Trending、Hacker News、V2EX）
- ⚽ **百度体育**：赛事日程获取 + 自动预热

### 技术栈

| 层级 | 技术 |
|-----|------|
| 后端 | Rust · Axum 0.7 · Sea-ORM 1.x · PostgreSQL 16 |
| 前端 | React 19 · Vite 6 · Antd 5 · TypeScript 5 · Biome |
| 基础设施 | Docker · GitHub Actions |

### Provider 状态

（同上表）

### 快速开始

#### 开发环境（使用 Docker）

```bash
# 1. 启动开发数据库
docker compose -f docker/docker-compose.dev.yml up -d

# 2. 复制并编辑 .env
cp .env.example .env
# 编辑 DATABASE_URL、TMDB_API_KEY 等

# 3. 运行迁移
cargo run -p tokimo-migration -- up

# 4. 启动服务器
cargo run -p tokimo-server

# 5. 构建管理界面
cd admin
pnpm install
pnpm build
cd ..

# 访问 http://localhost:5680/admin
```

#### 生产环境（Docker Compose）

```bash
docker compose -f docker/docker-compose.yml up -d
```

### 认证流程

#### 管理员登录

```bash
curl -X POST http://localhost:5680/api/admin/login \
  -H "Content-Type: application/json" \
  -d '{"bootstrap_key":"YOUR_BOOTSTRAP_KEY"}'
# 返回: {"token":"JWT_TOKEN"}
```

#### 创建服务密钥

```bash
curl -X POST http://localhost:5680/api/admin/service-keys \
  -H "Authorization: Bearer JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"my-app"}'
# 返回: {"token":"tks_...","id":"..."}
```

#### 使用服务密钥

```bash
curl http://localhost:5680/api/tmdb/movie/550 \
  -H "Authorization: Bearer tks_..."
```

### 配置说明

（环境变量同上表）

#### 存储后端

通过 `STORAGE_BACKEND` 选择。数据库只存对象 **key**，URL 在响应时通过 `Storage::url_for(key)`（async）组装。

| 后端 | 适用场景 | 必填环境变量 |
|---|---|---|
| `local` | 单机开发 / 反向代理自部署 | `STORAGE_LOCAL_ROOT`、`STORAGE_LOCAL_PUBLIC_BASE` |
| `s3` | AWS S3 / MinIO / 其他 S3 兼容服务 | `STORAGE_S3_BUCKET`、`STORAGE_S3_REGION`、`STORAGE_S3_ACCESS_KEY_ID`、`STORAGE_S3_SECRET_ACCESS_KEY`、可选 `STORAGE_S3_ENDPOINT`（AWS 留空）、`STORAGE_S3_PUBLIC_BASE`（公有桶必填）、`STORAGE_S3_PRESIGN_TTL_SECONDS`（默认 `0`） |
| `oss` | 阿里云 OSS（S3 兼容协议） | `STORAGE_OSS_BUCKET`、`STORAGE_OSS_REGION`、`STORAGE_OSS_ACCESS_KEY_ID`、`STORAGE_OSS_SECRET_ACCESS_KEY`、可选 `STORAGE_OSS_ENDPOINT`（默认 `https://oss-cn-hangzhou.aliyuncs.com`）、`STORAGE_OSS_PUBLIC_BASE`、`STORAGE_OSS_PRESIGN_TTL_SECONDS` |

`PRESIGN_TTL_SECONDS=0` ⇒ 公有桶，`url_for` 返回 `{public_base}/{key}`；`>0` ⇒ 私有桶，`url_for` 返回有效期为该秒数的预签名 GET URL。


### GitHub Secrets

用于 CI 工作流：

| Secret | 用途 | 说明 |
|--------|-----|-----|
| `TMDB_API_KEY` | Live API 测试 | TMDB API 密钥用于集成测试 |

### 贡献指南

#### 添加新 Provider

1. 创建 `crates/providers/src/my_provider.rs`
2. 实现带错误处理的获取逻辑
3. 在 `crates/server/src/routes/my_provider.rs` 添加路由处理器
4. 在 `routes/mod.rs` 注册路由
5. 如需要添加数据库迁移
6. 在 README provider 状态表中记录

#### 代码规范

- 非测试代码不使用 `.unwrap()` / `.expect()`
- 始终用 `?` 传播错误
- 数据库存储对象**键值**，不存 URL
- 响应时通过 `Storage::url_for(key).await` 组装 URL
- 上游调用包裹在 `tracing::info_span!("upstream", provider=..., ...)` 中

## License

MIT
