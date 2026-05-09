# tokimo-server CDN Roadmap

> 写于 2026-05-09。这是对 tokimo-server 项目"该做什么、为什么做"的反思笔记，不是承诺，不是排期。读它的人请把它当**判断 feature 优先级的尺子**，而不是任务清单。

## 1. 项目定位重申

tokimo-server **不是**通用 API gateway，**是**给"网络受限的 tokimo 用户"准备的：

```
[ tokimo 用户（中国/网络劣势） ]
              │ HTTPS
              ▼
[ tokimo-server （境外或近源部署） ]
              │
   ┌──────────┼──────────┬──────────┐
   ▼          ▼          ▼          ▼
TMDB     Spotify    Bangumi     19 个热榜源 ……
```

判断一个新 feature 该不该做，先问一句：

> **它能不能让一个网络劣势用户感知到更快 / 更稳？**

不能 → 后置或砍掉。

## 2. 现状盘点

### 已具备（强项）

| 能力 | 实现 |
|---|---|
| 上游适配 | 32 providers + 19 hot sources |
| 并发去重 | 两层 single-flight：进程内 DashMap + 跨进程 PostgreSQL `pg_advisory_xact_lock` |
| 持久缓存 | Sea-ORM + DB-backed TTL，metrics::cache_hit 标记 |
| 限流 | per-route token bucket，PostgreSQL 持久化 |
| 鉴权 | Bearer service key + Admin JWT |
| 资源存储 | Local / S3 / Aliyun OSS（**已搭好但没真正接入图片代理**） |
| 监控 | metrics 字段 + admin dashboard 数据接口 |

### 短板（按"目标对齐度"看的）

最大的短板是 **CDN 这层的"边缘"能力**几乎为零：

1. 所有请求最终都打到源站和 PG（即便是 cache hit 也走一次 DB query）。
2. 浏览器 / 公网 CDN 拿不到任何 cache hint，没法在边缘吃掉相同请求。
3. 大流量品类（图片）完全没接入：用户浏览器仍然直连 `image.tmdb.org` / `i.scdn.co`。代理项目漏掉了用户最直观的体验项。
4. 上游故障会穿透打爆（没有 negative cache）。
5. 首屏依赖"恰好别人之前请求过同样数据"——没有预热机制。

## 3. 10 项候选 feature（按 ROI 排序）

每条用统一格式：**目标 / 工作量 / 收益 / 备注**。

### 🔥 #1 HTTP cache headers + ETag + Last-Modified

- **目标**：所有 GET 路由按上游可缓存性输出 `Cache-Control: public, max-age=...` + `ETag` + `Last-Modified`。让 Cloudflare / 阿里 CDN / 浏览器能在边缘吃命中。
- **工作量**：低（半天）。在 axum middleware 或每个路由响应包装层加。
- **收益**：极高。一旦套上公网 CDN，99% 流量被边缘拦截，源站 + PG 负载几乎归零；这是 CDN 项目最划算的一刀。
- **备注**：不需要 ops 配合，先在 server 端把 header 出对；上 CDN 是后续的事，但前提是 server 先合规。

### 🔥 #2 图片代理 + on-the-fly resize / WebP 转码

- **目标**：TMDB 海报、Spotify 封面、JavBus 截图等图片资源走 tokimo-server 转发；首次访问下载到 S3/OSS，后续直接边缘出。支持 query 参数指定尺寸（`?w=500`），按需转 WebP/AVIF。
- **工作量**：中（~2 天）。需要新路由 + `image` crate 或 `libvips` FFI；S3 存储已就绪。
- **收益**：极高。**用户感知"快不快"≈"海报多久出来"**。当前方案根本没把图片纳入代理，等于漏掉最大流量品类。
- **备注**：选型权衡——`image` crate 纯 Rust（CPU 慢，无依赖），`libvips` 快 5-10× 但要装系统库；先用 `image` 上线，性能瓶颈了再换。

### 🔥 #3 失败响应 neg-cache

- **目标**：404/429/5xx 也短 TTL 缓存（30s ~ 5min）。
- **工作量**：极低。改 cache 写入逻辑，加一个 `cache_failure_ttl` 配置。
- **收益**：高。上游故障 / 限流时不会被穿透打爆；用户即便看到错误，至少错误是"立刻"返回的而不是 30 秒超时。
- **备注**：要小心 5xx 的 short TTL 不能太长，否则上游恢复后用户还在拿到错误。

### #4 预热 / 定时刷新

- **目标**：tokio-cron-scheduler 每天定时拉：top 100 movies metadata、19 hot sources、Bing wallpaper、currency rates。
- **工作量**：低（1 天）。
- **收益**：高。用户首屏永不冷启动；尤其是 hot sources 这种"每个用户都看 + 短 TTL"的列表，应该后台拉。
- **备注**：跟 #1 配合更好——预热填的缓存，被 CDN 边缘命中，用户首次访问就接近零延迟。

### #5 per-service-key quota + 用量看板

- **目标**：每个 service key 独立配额（次数 / 流量），admin 看板可视化。
- **工作量**：中。需要新表 + 中间件计数 + admin UI。
- **收益**：中。SaaS 化的硬前置；也能识别滥用 key。
- **备注**：阻塞用户登录 / 计费功能，但本身用户感知不到，所以 ROI 不如前 4。

### #6 upstream latency p50/p95/p99 + cache hit ratio Grafana

- **目标**：Prometheus exporter + Grafana dashboard。每个 provider 独立看 p50/p95/p99 + 命中率。
- **工作量**：中（dashboard 调起来挺花时间）。
- **收益**：中。出问题时看得出哪个 provider 在拖；可以对外承诺 SLA。
- **备注**：metrics 字段已有，缺的是 exporter 和 dashboard JSON。

### #7 多上游 fallback

- **目标**：TMDB 挂 → 自动 fallback 到 OMDb；JavBus 挂 → JavDB。需要 schema 归一化层。
- **工作量**：高。每个领域要设计统一 schema + adapter。
- **收益**：中。提高可用性，但代价是抽象层复杂度。
- **备注**：跟 #2 抢工作量，先放后期。

### #8 gzip / brotli 响应压缩 + 上游 keep-alive 池调优

- **目标**：tower-http `CompressionLayer` 接到所有 JSON 路由；reqwest `pool_idle_timeout` / `pool_max_idle_per_host` 调到合理值。
- **工作量**：极低（半小时）。
- **收益**：中。中国 ↔ 海外延迟敏感，每个 RTT 都贵；压缩响应能显著省流量。
- **备注**：和 #1 一起做最划算。

### #9 大文件 chunked streaming

- **目标**：字幕、图片大文件（>1MB）边下边发，不必整文件读进内存再返回。
- **工作量**：中。axum response stream + reqwest streaming body。
- **收益**：中。减少 TTFB；同时降内存占用。
- **备注**：跟 #2 部分重合（图片如果走 resize 必须先全读），需要分流 routing：原图 stream，变换走 buffered。

### #10 DB 索引审计 + 缓存表 zstd 压缩

- **目标**：用 EXPLAIN ANALYZE 审所有 cache 表查询，加覆盖索引；jsonb 改 bytea + zstd 存（省 50% 磁盘）。
- **工作量**：中。需要 migration + 兼容老数据迁移。
- **收益**：中（中长期）。短期看不出，但缓存表会一直膨胀，迟早做。
- **备注**：等数据量上来再做不迟。

## 4. 刻意排除（暂不做）

| 排除项 | 理由 |
|---|---|
| 多区域部署 / 边缘节点 | 这是 ops 问题不是 server 问题，租 Cloudflare 或阿里 CDN 即可，不必自己 multi-region |
| 用户登录 / 邀请码 / SaaS 计费 | 已在前面阶段 plan，优先级低于"先把核心代理能力做扎实" |
| 删主仓 hot_search.rs 改 thin proxy | 用户明确指示不动主仓 |
| fetch-as-blob 全量缓存大文件 | 与 chunked streaming 冲突，留到后期再权衡 |
| 通用 GraphQL gateway | 偏离定位，不做 |

## 5. 推进顺序建议

```
第 1 周末（半~1 天）：#1 + #3 + #8 三连击
                      └ 收益：用 CDN 后流量归零；上游故障不穿透；省 30%+ 流量

第 1 周（2 天）：     #2 图片代理
                    └ 收益：海报加载从 2s → 200ms（中国出口）

第 2 周（1 天）：     #4 预热
                    └ 收益：首屏永不冷启动

进入"对外可用"阶段后再考虑 #5/#6（运营能力）和 #7/#9/#10（深度优化）。
```

## 6. 衡量标准

每完成一项，能给出明确数字才算落地：

| 项 | 衡量指标 |
|---|---|
| #1 | 套 CDN 后边缘命中率 ≥ 95%（CDN 后台数据） |
| #2 | TMDB 海报 P95 加载时延（中国出口）从 X ms → Y ms |
| #3 | 上游 5xx 故障期间，源站 QPS 不超过正常值 1.5 倍 |
| #4 | 首页 hot sources 接口冷启动率 < 5% |
| #5 | admin 看板能看到每个 key 当日请求数 / 流量 |
| #6 | Grafana 能在 1 分钟内定位异常 provider |

## 7. 待用户决策的开放问题

1. 何时启动 #1-#4 实施？这一组下来约 4-5 天工时。
2. CDN 选型：Cloudflare 全球 vs 阿里 CDN 国内 vs 双活？需要 ops 方决策。
3. 图片 resize 用 `image` crate 还是 `libvips` FFI？默认建议先 `image` crate 上线，扛不住再换。
4. neg-cache 的 5xx TTL 设多长合适？默认建议 30 秒。
5. 预热任务用 in-process tokio-cron 还是独立 worker 进程？取决于多实例部署策略。
