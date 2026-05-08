import type { Resources } from "./en";

const zh: Resources = {
  common: {
    login: "登录",
    logout: "退出登录",
    save: "保存",
    cancel: "取消",
    delete: "删除",
    create: "创建",
    close: "关闭",
    refresh: "刷新",
    loading: "加载中...",
    error: "错误",
    success: "成功",
    yes: "是",
    no: "否",
    language: "语言",
  },
  nav: {
    dashboard: "仪表盘",
    keys: "服务密钥",
    providers: "Provider 配置",
    cache: "缓存检查",
    settings: "设置",
    appTitle: "Tokimo 服务管理后台",
    serviceKeys: "服务密钥",
  },
  header: {
    logout: "登出",
    theme: {
      light: "浅色",
      dark: "深色",
    },
    language: {
      zh: "中文",
      en: "English",
    },
  },
  login: {
    cardTitle: "管理员登录",
    bootstrapKeyLabel: "Bootstrap 密钥",
    bootstrapKeyRequired: "请输入 Bootstrap 密钥",
    submit: "登录",
    success: "登录成功",
  },
  serviceKeys: {
    createBtn: "创建服务密钥",
    modalTitle: "创建服务密钥",
    tokenCreatedHint: "密钥已创建，请立即复制（仅展示一次）：",
    nameLabel: "名称",
    columns: {
      name: "名称",
      prefix: "前缀",
      enabled: "启用",
      created: "创建时间",
      action: "操作",
    },
    toasts: {
      created: "服务密钥已创建",
      deleted: "服务密钥已删除",
    },
  },
  providers: {
    title: "Provider 配置",
    description:
      "本服务接入的 {{count}} 个 Provider 适配器静态视图。鉴权所需环境变量在服务启动时从进程环境读取；为避免泄露密钥存在性，此处不展示运行时实际是否已配置。",
    readOnlyTitle: "只读视图",
    readOnlyDescription:
      "暂不支持运行时编辑 Provider 配置。请在服务的 .env / 部署清单中设置环境变量后重启。",
    columns: {
      provider: "Provider",
      prefix: "接口前缀",
      rateLimit: "限流",
      auth: "鉴权",
      envVars: "环境变量",
    },
    auth: {
      required: "必填",
      optional: "可选",
      none: "无需",
    },
    columns2: {
      sampleUrl: "示例 URL",
      action: "操作",
    },
    serviceKey: {
      label: "Service Key (Bearer)",
      placeholder: "tks_...",
      saved: "已保存到 localStorage",
      missing: "Service key 为空 — 请求很可能 401",
      copied: "URL 已复制",
      promptTitle: "需要 Service Key",
      promptDescription:
        "请输入 Bearer service key。会以 base64 编码后保存到 localStorage，不上传服务器。",
      promptSubmit: "保存并发送",
      promptRequired: "请输入 service key",
      clear: "清除",
      cleared: "已清除 service key",
    },
    test: {
      sendBtn: "发送",
      modalTitle: "响应 · {{provider}}",
      status: "状态",
      duration: "耗时",
      contentType: "Content-Type",
      body: "Body",
      sending: "发送中...",
      networkError: "网络错误",
      copyResponse: "复制响应",
      copiedResponse: "响应已复制",
    },
  },
  dashboard: {
    title: "仪表盘",
    retry: "重试",
    loading: "正在加载仪表盘...",
    empty: "暂无数据 — 请调用 Provider 后刷新。",
    cards: {
      keys: "服务密钥总数",
      providers: "活跃 Provider",
      cacheEntries: "缓存条目",
      calls24h: "24 小时调用",
      errorRate: "错误率 {{rate}}（{{errors}}/{{calls}}）",
    },
    subtitles: {
      active: "可用",
      configured: "已配置",
      totalRows: "总行数",
    },
    charts: {
      volume: "请求量",
      topProviders: "调用最多的 Provider",
      byProvider: "Provider 调用量",
      recentErrors: "最近错误",
      calls: "调用",
      errors: "错误",
      cacheHits: "缓存命中",
      cacheMisses: "缓存未命中",
      other: "其他",
      latency: "延迟 p50 / p95",
      p50: "p50",
      p95: "p95",
      cacheHit: "缓存命中率",
      heatmap: "Provider × 时段",
      errorsArea: "错误趋势",
      statusCodes: "状态码",
      statusOk: "2xx",
      status4xx: "4xx",
      status5xx: "5xx",
      cacheTables: "缓存表",
      rows: "行",
      avgTtl: "平均 TTL",
      heroCalls: "区间内总调用数",
      dragHint: "拖动手柄重新排序",
    },
    range: {
      "1h": "1 小时",
      "24h": "24 小时",
      "7d": "7 天",
    },
    refresh: {
      label: "刷新",
      off: "关闭",
      now: "立即刷新",
      interval: "自动刷新",
    },
    columns: {
      time: "时间",
      provider: "Provider",
      status: "状态",
      duration: "耗时",
    },
    relative: {
      justNow: "刚刚",
      minutesAgo: "{{count}} 分钟前",
      hoursAgo: "{{count}} 小时前",
      daysAgo: "{{count}} 天前",
    },
    units: {
      ms: "{{value}} 毫秒",
    },
  },
  cache: {
    title: "缓存检查器",
    description: "查看 / 清除 / 强制过期各 provider 的缓存",
    tablePlaceholder: "选择缓存表",
    searchPlaceholder: "搜索 id、key 或预览内容",
    confirmDeleteTitle: "确认删除这条缓存？",
    previewHint: "这里只展示前 200 个字符。完整缓存内容需要直接查询数据库。",
    previewModalTitle: "前 200 字符预览",
    columns: {
      id: "ID",
      key: "Key",
      fetchedAt: "抓取时间",
      rawPreview: "原始预览",
      operations: "操作",
    },
    actions: {
      viewFull: "查看完整",
      expire: "强制过期",
      delete: "删除",
    },
    ttl: {
      average: "平均剩余 TTL：{{value}}",
      expired: "已过期",
      empty: "无数据",
      days: "{{count}}天",
      hours: "{{count}}小时",
      minutes: "{{count}}分",
      seconds: "{{count}}秒",
    },
    relative: {
      justNow: "刚刚",
      minutesAgo: "{{count}} 分钟前",
      hoursAgo: "{{count}} 小时前",
      daysAgo: "{{count}} 天前",
    },
    toasts: {
      expired: "缓存已强制过期",
      deleted: "缓存已删除",
    },
  },
  backdoor: {
    toast: "再点 {{remaining}} 次开启清除工具",
    title: "清除统计 (Debug)",
    range: {
      "1h": "最近 1 小时",
      "24h": "最近 24 小时",
      "7d": "最近 7 天",
      all: "全部",
      custom: "自定义",
    },
    confirm: "清除",
    cancel: "取消",
    success: "已清除 {{count}} 条记录",
  },
  docsHub: {
    title: "文档中心",
    fabTooltip: "打开文档中心 (Cmd/Ctrl+/)",
    minimize: "最小化",
    expand: "展开",
    close: "关闭",
    empty: "当前页面暂未注册文档条目。",
    sectionsHeader: "章节",
    fieldsHeader: "字段",
    entryCount: "{{count}} 条目",
  },
  docs: {
    dashboard: {
      title: "运行总览",
      summary:
        "代理服务的实时仪表盘：在所选时间窗内聚合的请求量、错误率、延迟分位数、状态码分布以及各 Provider 流量。所有数据来自 `/api/admin/dashboard/*` 接口，由后端预聚合，前端不做二次计算。",
      sections: {
        overview: {
          title: "概览",
          body: "页面顶部是**全局控件**（时间范围 / 自动刷新 / 手动刷新），下面是 9 张可拖拽重排的图表卡片，最下方是「最近错误」表格。卡片顺序与自动刷新间隔保存在 localStorage（key 为 `tokimo-admin-dashboard-order-v1` / `-refresh-interval-v1`），刷新页面后保留。\n\n**所有图表受顶部时间范围统一控制**——单卡片不提供独立 range，避免互相错位的时间轴。",
        },
        layout: {
          title: "卡片布局",
          body: "9 张卡片在 lg 屏幕（≥1024px）上以 3 列网格展示，**Request Volume** 卡片跨 2 列宽显示。每张卡左上角的拖动手柄（六个点图标）可重排顺序；新顺序立即写入 localStorage。\n\n卡片状态机：**loading**（骨架）→ **error**（带 Retry 按钮）→ **empty**（无数据占位）→ **rendered**。",
        },
        refresh: {
          title: "刷新与缓存",
          body: "**自动刷新**：通过 React Query 的 `refetchInterval` 实现，可选 0（关闭）/ 10s / 30s / 60s，默认 30s。\n**手动刷新**：右上角圆形按钮（旋转图标）触发所有可见卡片同时 `refetch`，按钮在请求期间会持续旋转。\n**staleTime**：所有 dashboard query 的 `staleTime` 由 React Query 默认决定（0），即每次 `refetch` 都重新请求后端。",
        },
        backdoor: {
          title: "隐藏调试入口",
          body: "侧栏 Logo 处**连续 5 次快速点击**会弹出「清除统计」对话框（`backdoor.title`），可以按时间窗清空 metrics rollup（1h / 24h / 7d / all / 自定义）。**会永久重置数据**，仅供调试，不要在生产环境随便点。",
        },
      },
      fields: {
        "control-range": {
          label: "时间范围 (1h / 24h / 7d)",
          desc: "选择整页的聚合窗口。**对应 bucket 大小**：1h → 5 分钟桶；24h → 1 小时桶；7d → 1 天桶。值会作为 `range_secs` 与 `bucket_secs` 透传给所有 dashboard 接口。切换 range 会取消并重发所有正在进行中的 query。",
        },
        "control-refresh-interval": {
          label: "自动刷新 (Off / 10s / 30s / 60s)",
          desc: "设置 React Query 的 `refetchInterval`。Off 表示不自动刷新（只在切 range / 手动刷新时取数）。设置保存在 localStorage `tokimo-admin-dashboard-refresh-interval-v1`。\n\n注意：仅当浏览器标签页处于活动状态时才会触发自动 refetch（受 React Query 默认 `refetchIntervalInBackground: false` 控制）。",
        },
        "control-refresh-now": {
          label: "立即刷新（旋转图标按钮）",
          desc: "强制对页面上所有 dashboard query 触发 `refetch()`，无视 `staleTime`。按钮图标在任意 query 处于 fetching 状态时持续旋转。",
        },
        "card-volume": {
          label: "Request Volume（请求量）",
          desc: "时间序列折线图，X 轴为按 bucket 切片的时间，Y 轴为请求数。区分两条线：**calls**（成功 + 失败合计）与 **errors**。卡片标题处显示**所选时间窗内的总调用数**。\n\n数据源：`GET /api/admin/dashboard/timeseries?range_secs=...&bucket_secs=...`。该卡占据 2 列宽。",
        },
        "card-cache-ring": {
          label: "Cache Hit（缓存命中率环）",
          desc: "活动环 + 中央百分比文字，显示**最近 24 小时**整体缓存命中率（不受顶部 range 影响）。值来自 `dashboard/overview` 的 `cache_hit_ratio_24h` 字段，范围 0..1。\n\n判定为「命中」的依据：响应头含 `x-cache: HIT`，由代理层在写出响应时盖章。",
        },
        "card-top-providers": {
          label: "Top Providers（饼图）",
          desc: "按调用次数排名 Top-N 的 Provider 占比。**N = 10**，超过前 10 的 Provider 合并到「其他」切片中。N 之所以设为 10（不是 5）是为了避免长尾被一锅端到「其他」里看不出分布。\n\n数据源：`dashboard/by-provider`，按时间窗内 `calls` 倒序。",
        },
        "card-by-provider": {
          label: "Provider Calls（柱图）",
          desc: "横向柱状图，列出所有 Provider 在所选时间窗内的总调用数。卡标题显示 Provider 总数（即柱子条数）。与 Top Providers 饼图共用同一份 `dashboard/by-provider` 数据，但**不做截断**——所有 Provider 都会出现，便于对比尾部流量。",
        },
        "card-latency": {
          label: "Latency p50 / p95（延迟）",
          desc: "两条折线：**p50** = 中位数，**p95** = 95 分位数，单位毫秒。计算口径：每个时间桶内基于该桶所有请求的耗时样本（含成功与失败）做分位数。卡标题显示**最新桶**的 p95 值，副标题显示最新桶 p50。\n\n数据源：与 Volume 共用 `dashboard/timeseries`（响应里同时含 `p50_ms` / `p95_ms`）。",
        },
        "card-errors-area": {
          label: "Errors Trend（错误趋势面积图）",
          desc: "面积图，仅显示 `errors`（4xx + 5xx 合计）随时间的变化。与 Volume 折线的 `errors` 序列同源（`dashboard/timeseries.errors`），但单独成图便于在错误激增时一眼看到形态。卡标题显示时间窗内总错误数。",
        },
        "card-heatmap": {
          label: "Provider × Time（热力图）",
          desc: "二维热力图：Y 轴为 Provider，X 轴为时间桶，格子颜色深浅代表该时间桶该 Provider 的调用次数。用于快速定位「某个 Provider 在某段时间突然爆量」。\n\n数据源：`dashboard/heatmap?range_secs=...&bucket_secs=...`，响应是 `[{ ts, values: [{ provider, calls }] }]` 形态。",
        },
        "card-status-codes": {
          label: "Status Codes（状态码堆叠柱）",
          desc: "堆叠柱状图，每个时间桶 3 段：**2xx**（成功，绿）/ **4xx**（客户端错误，黄）/ **5xx**（服务端错误，红）。卡标题显示三段相加的总和。\n\n数据源：`dashboard/status-codes`。注意 502/504 这种 upstream 失败会被代理记成 5xx，与上游本身返回的 5xx 合并。",
        },
        "card-cache-tables": {
          label: "Cache Tables（缓存表列表）",
          desc: "纯文本列表（不是图表），按行数倒序列出每张缓存表 `cache_<provider>` 及其行数与平均剩余 TTL。点列表项不会跳转——只是状态展示，去缓存检查器页面进行操作。\n\n数据源：`/api/admin/cache/tables`，与「缓存检查器」页顶部下拉同源。",
        },
        "recent-errors-table": {
          label: "Recent Errors（最近错误表格）",
          desc: "页面底部的表格，列出最近的失败请求。**最多 50 条**（后端硬上限），按发生时间倒序。\n\n列：**Time**（相对时间，如「3 分钟前」）/ **Provider**（哪个 Provider 触发）/ **Status**（HTTP 状态码，如 502）/ **Duration**（耗时毫秒）。\n\n数据源：`dashboard/recent-errors`。错误记录保留时长由后端 metrics rollup 配置决定，超出会被清理。",
        },
      },
    },
    "provider-configs": {
      title: "Provider 配置",
      summary:
        "本服务接入的所有上游 API Provider 的**只读静态视图**。展示路由前缀、限流、鉴权要求与所需环境变量；底部配合 Service Key 输入框可以即时发起一次真实代理请求来探活。Provider 配置在代码中固化，运行时不可编辑。",
      sections: {
        overview: {
          title: "概览",
          body: "表格内容来自前端常量 `PROVIDERS`（`admin/src/pages/ProviderConfigsPage.tsx`），与后端实际注册路由保持人工同步。每行对应一个上游 Provider 的代理。\n\n**为什么是只读**：鉴权所需的环境变量（如 `TMDB_API_KEY`）在服务进程启动时从 env 读取，admin 不暴露这些值是否已配置——避免泄漏「某 key 是否存在」这种边信道信息。要修改 Provider 行为，请改 `crates/providers/` 与 `.env` 后**重启服务**。",
        },
        "service-key": {
          title: "Service Key 工作流",
          body: "顶部输入框接收一个 Bearer service key（格式 `tks_xxx`）。值会被 base64 编码保存到 `localStorage['tokimo-admin-svc-key']`，**不会**上传服务器。点行内「发送」时：\n\n1. 若 key 为空 → 弹出 `ServiceKeyPromptModal` 让你先填\n2. 浏览器直接 `fetch(sample, { Authorization: 'Bearer ' + key })`\n3. 响应在 `ProviderResponseModal` 中展示，**不经过 admin 后端**\n\n这就是为什么需要 service key：admin 不代为发起请求，避免把 admin session 的权限借给 Provider 调用。",
        },
        "response-modal": {
          title: "响应检查器",
          body: "「发送」按钮触发的 Modal 显示：**Status**（HTTP 状态码）、**Duration**（前端 `performance.now()` 测的端到端耗时，含网络）、**Content-Type**、**Body**（JSON 会被自动 pretty-print）。「复制响应」按钮把整个 body 写入剪贴板。\n\n失败时（CORS / 网络 / 超时）`status` 显示为 0，error message 单独展示。",
        },
        security: {
          title: "安全须知",
          body: "- Service key 是 base64 编码而非加密，**任何能访问该浏览器 localStorage 的人都能读到**。仅在受信终端使用。\n- 「示例 URL」请求会**真实**打到上游 Provider，会被计入 metrics、消耗你的 API quota、写入 cache。\n- 「清除」按钮只是从 localStorage 删除本地副本，服务器端的 service key 不受影响（要吊销请去「服务密钥」页）。",
        },
      },
      fields: {
        "column-provider": {
          label: "Provider",
          desc: "Provider 的展示名（如 `TMDB` / `OMDb` / `Bangumi`）。同时是路由命名空间的人类可读标签——表格里没有单独的 prefix 列，prefix 见下一列示例 URL 的开头部分（如 `/api/tmdb/...`）。",
        },
        "column-sample-url": {
          label: "示例 URL",
          desc: "一条经过精心选择的真实可调用的 sample 路径（如 `/api/tmdb/movie/550` 对应《搏击俱乐部》）。**点这一列文字不会触发请求**——只有右侧「发送」按钮会。鼠标 hover 显示完整 URL（路径较长时会被 ellipsis 截断）。\n\n该路径在「发送」时会被 `expandSample()` 处理（替换占位符等），实际请求 URL 即为此值。",
        },
        "column-rate-limit": {
          label: "限流",
          desc: "代理对该 Provider 设置的**单实例 token bucket** 上限（如 `10/s` / `1/s` / `30/s`）。这是**代理出口侧**的限流——保护上游不被打爆，与 Service Key 自身的限流是两套独立机制。\n\n超出会返回 `429`，并附带 `Retry-After` 头。值在 Rust 端 `crates/providers/<name>.rs` 中以代码常量定义，admin 只是文案展示。",
        },
        "column-auth": {
          label: "鉴权",
          desc: "**调用本代理时**是否需要 Bearer service key（不是上游 API 的鉴权）。三种取值：\n\n- `必填` (yes，红 tag) — 必须带 `Authorization: Bearer tks_...`，否则 401\n- `可选` (optional，黄 tag) — 不带也能调，但可能限流更严或返回精简数据\n- `无需` (no，绿 tag) — 完全开放，匿名可调（如部分公开数据 Provider）\n\n上游 Provider 自身是否需要 API key 由「环境变量」列体现，与本列**无关**。",
        },
        "column-env-vars": {
          label: "环境变量",
          desc: "上游 Provider 鉴权所需的进程级环境变量名列表（如 `TMDB_API_KEY`、`SPOTIFY_CLIENT_ID` + `SPOTIFY_CLIENT_SECRET`）。**空（—）** 表示该 Provider 不需要任何上游 key（如 Douban 走匿名公开页）。\n\n服务启动时若必填 env 缺失：调用该 Provider 会返回 `502 Bad Gateway` 并附带错误 `provider not configured: missing env XXX`。**admin 此处不显示运行时是否已配置**——只列出 schema。",
        },
        "column-action-send": {
          label: "操作 · 发送",
          desc: "点击发送按钮：用顶部输入框中的 service key（若为空则先弹 prompt）携带 `Authorization: Bearer ...` 直接 `fetch(sample)`。请求**由浏览器发出**，不经 admin 后端转发。\n\n用途：在不动写脚本的前提下快速探活某个 Provider，验证：上游可达 ✅ / service key 有效 ✅ / 鉴权 scope 够用 ✅ / 响应结构符合预期 ✅。响应在 `ProviderResponseModal` 中展示。",
        },
        "input-service-key": {
          label: "Service Key 输入框",
          desc: "接收形如 `tks_xxx.<sig>` 的 Bearer token。值用 base64 编码后存 `localStorage['tokimo-admin-svc-key']`，下次打开本页自动回填。**只用于本页发送测试请求**——admin 自身的鉴权走另一条 cookie/JWT 路径，不读这个值。",
        },
        "action-clear-key": {
          label: "清除按钮",
          desc: "从 localStorage 删除本地保存的 service key，并清空输入框。**不会**调用后端，**不会**吊销密钥本身。要吊销请去「服务密钥」页对该 key 删除。",
        },
        "response-status": {
          label: "响应 · Status",
          desc: "Modal 顶部显示的 HTTP 状态码。常见取值：`200` 成功 / `401` service key 无效 / `404` 路径不存在 / `429` 触发限流 / `502` 上游 env 未配置 / `0` fetch 抛错（网络 / CORS / 超时）。",
        },
        "response-duration": {
          label: "响应 · 耗时",
          desc: "前端 `performance.now()` 测得的**端到端耗时**（毫秒），从 `fetch()` 发起到完整 body 读完。包含浏览器 → 代理 → 上游 → 代理 → 浏览器全链路。**不等于**上游纯耗时；上游耗时可在 dashboard 延迟图中查看。",
        },
        "response-content-type": {
          label: "响应 · Content-Type",
          desc: "代理透传的响应头。若为 `application/json` 系列，body 会被自动 pretty-print（缩进 2）；其它类型按原文展示。",
        },
        "response-body": {
          label: "响应 · Body",
          desc: "响应主体。JSON 会自动美化；非 JSON / parse 失败按原文显示；错误情况下显示 fetch 抛出的 error message。「复制响应」按钮把整个 body 写入剪贴板。",
        },
      },
    },
    "service-keys": {
      title: "服务密钥",
      summary:
        "签发 / 查看 / 吊销下游服务调用本代理时使用的 Bearer Token。每个密钥有不可变的创建时间和**仅展示一次**的明文 token——丢失后只能重新签发。本页是 admin 唯一能直接产出可用 service key 的入口。",
      sections: {
        overview: {
          title: "概览",
          body: "表格列出所有已签发的服务密钥（不含明文 token，**只有 prefix**）。顶部「创建服务密钥」按钮弹窗签发新密钥；签发成功后明文 token 会一次性显示在同一弹窗里，关闭后**永久无法再次查看**。\n\n密钥仅用于**服务器到服务器**的调用，不要嵌到浏览器里——因为浏览器代码会把 token 暴露给最终用户。",
        },
        lifecycle: {
          title: "生命周期",
          body: "**创建**：admin 调用 `POST /api/admin/service-keys`，后端用 HMAC-SHA256 签发 `tks_<id>.<sig>` 形态的 token，**只在响应里**返回一次明文。后端只保存 `id` 与 `token_prefix`（前 N 字符）用于审计展示，**不存明文**也不存可逆形式。\n\n**吊销**：「删除」按钮调 `DELETE /api/admin/service-keys/{id}`。删除后该 key 的所有后续请求立即返回 401，**已经在途中的请求不受影响**。",
        },
        security: {
          title: "安全须知",
          body: "- 明文 token 只展示一次；关闭弹窗 = 永久遗失，**没有任何「重新查看」按钮**\n- 后端验签使用恒定时间比较，不会因 sig 不同长度泄漏信息\n- service key 的 scope / TTL **暂未实现**——任何已签发 key 拥有完整 Provider 调用权限，谨慎签发\n- `enabled = false` 等价于软吊销，但目前 admin UI **不展示切换开关**——仅靠删除来吊销",
        },
      },
      fields: {
        "column-name": {
          label: "名称",
          desc: "签发时填的人类可读名（如 `media-server-prod`、`my-laptop-dev`）。**仅用于辨识**，对鉴权无影响，可重复。建议带环境后缀以便审计。",
        },
        "column-token-prefix": {
          label: "前缀",
          desc: "明文 token 的前若干字符（典型为 `tks_<id>` 部分），用于在日志和列表里识别**是哪条 key**。完整明文不会落库——**这是有意为之的设计**——所以表格永远只能给你前缀。",
        },
        "column-enabled": {
          label: "启用",
          desc: "当前是否允许使用。值为 `是` / `否`。\n\n**目前 admin UI 没有切换按钮**——所有签发出来的 key 默认 `是`，要禁用只能通过「删除」按钮硬删。后端已支持 `enabled = false` 的软吊销，未来 UI 会补上 toggle。",
        },
        "column-created": {
          label: "创建时间",
          desc: "UTC ISO-8601 时间戳。审计用途；同时是签名 payload 的一部分（HMAC over `{id, scopes, created_at}`），不可篡改。",
        },
        "column-action-delete": {
          label: "操作 · 删除",
          desc: "硬删除该密钥。**立即生效，不可恢复**。点击直接调 `DELETE /api/admin/service-keys/{id}`，没有二次确认弹窗——下游服务会从下一次请求开始收到 401。要重新签发需走「创建」流程拿新 token。",
        },
        "action-create": {
          label: "创建服务密钥按钮",
          desc: "弹出创建表单。当前**只需填一个 `名称` 字段**——scope / TTL / 备注等高级字段未实现。提交后在同一个 Modal 内显示新签发的明文 token，需要立即复制。",
        },
        "form-name": {
          label: "表单 · 名称",
          desc: "必填。允许任意 UTF-8 字符串；建议短于 64 字符，否则 prefix 列展示会被截断。提交即落库，无法事后修改（要改名只能删了重签）。",
        },
        "token-reveal": {
          label: "新签发明文 token 展示",
          desc: "创建成功后弹窗内的只读 textarea，包含 `tks_<id>.<sig>` 完整明文。**仅展示一次**，关闭弹窗后即永久消失——后端从未保存这个 sig 段。点关闭前必须复制走，否则只能重签。",
        },
      },
    },
    "cache-inspector": {
      title: "缓存检查器",
      summary:
        "查看 PostgreSQL 中各 Provider 缓存表的内容；可以预览原始响应前 200 字符、强制行过期或硬删除单行。每张缓存表 `cache_<provider>` 的 schema 由对应 Sea-ORM entity 决定。",
      sections: {
        overview: {
          title: "概览",
          body: "顶部下拉**选缓存表**（每张表对应一个 Provider，如 `cache_tmdb` / `cache_omdb`），下拉项右侧括号里的数字是行数。表选定后下方分页表格加载该表前 50 行（按 `fetched_at` 倒序，最新在前）。\n\n**完整功能 v1**：仅本页可见的 UI 元素已通过后端 CRUD 接口实现。更深入的批量过期 / 模式匹配删除尚未提供 UI，需要直连数据库。",
        },
        ttl: {
          title: "TTL 与过期",
          body: "TTL 由 Provider 在写缓存时显式指定（不在每行单独存配置）；下拉旁的「平均剩余 TTL」是该表所有未过期行的平均值。\n\n**过期行不会自动删除**——查询时若 `now() > fetched_at + ttl` 则视为 stale，下次命中会回源；同时回源失败时仍可作为「软兜底」返回旧数据（具体策略见各 Provider 实现）。\n\n手动「强制过期」会把 `fetched_at` 调到足够古老，保证下一次请求一定回源。",
        },
        operations: {
          title: "可用操作与审计",
          body: "每行三个操作：\n\n- **查看完整** — 把 `raw_preview`（前 200 字）展示在 Modal 中。**整行 raw 不在表格响应里**，但 Modal 也只展示 200 字——完整 body 需要直连 DB（`SELECT raw FROM cache_<provider> WHERE id=...`）\n- **强制过期** — 调 `POST /api/admin/cache/{table}/{id}/refresh`，把 fetched_at 拨到很久以前\n- **删除** — 调 `DELETE /api/admin/cache/{table}/{id}`，行直接消失（带 Popconfirm 二次确认）\n\n三种操作都会写 admin audit log。**删除不可恢复**。",
        },
        limitations: {
          title: "已知局限",
          body: "- 「查看完整」实际只展示前 200 字符（与列表里 `raw_preview` 同源），名字略有误导——这是因为完整 body 可能上 MB 级，不适合走 admin 接口\n- 搜索框是**前端纯客户端过滤**当前页 50 行（匹配 id / key / preview），不是后端搜索；翻到下一页要重新输入\n- 没有按 TTL 颜色 Tag——表格只显示 `fetched_at` 时间和相对时间副标题，是否过期需要自行计算（或参考下拉旁的平均 TTL）",
        },
      },
      fields: {
        "selector-table": {
          label: "缓存表选择器",
          desc: "下拉菜单，列出所有 `cache_*` 表。选项格式 `表名 (行数)`，行数来自 `GET /api/admin/cache/tables` 接口。切换会重置分页到第 1 页并重新拉取行数据。",
        },
        "avg-ttl-display": {
          label: "平均剩余 TTL 展示",
          desc: "下拉右侧的灰字，格式如「平均剩余 TTL：2 天 5 小时」。值是该表所有**未过期**行剩余 TTL 的平均（`avg(ttl - (now() - fetched_at))`）。\n\n空表显示「无数据」；全部已过期显示「已过期」。最多显示 2 段单位（天 + 小时 / 小时 + 分 / 分 + 秒）。",
        },
        "input-search": {
          label: "搜索框",
          desc: "**纯前端过滤**当前页 50 行，匹配字段：`id`、`key`、`raw_preview`（任一包含搜索词即命中，大小写不敏感）。**不会触发后端查询**——翻页后需要重新输入。常用于在已加载的小窗口内快速定位某个 key。",
        },
        "action-refresh-list": {
          label: "刷新按钮",
          desc: "重新拉取表列表 + 当前选定表的当前页行。在表数据可能被其他进程改写后用来强制取最新。按钮在加载期间显示 loading。",
        },
        "column-id": {
          label: "ID",
          desc: "缓存行的主键，固定列。通常是 hash（如 SHA256 截断）或 Provider 自定义生成的 stable id。`fixed: left` 让其在水平滚动时不消失。",
        },
        "column-key": {
          label: "Key",
          desc: "规范化的请求 key，由 Provider id、路由、查询参数组合并大小写归一生成（如 `tmdb:movie/550?language=zh-CN`），保证「等价请求」落到同一行。鼠标 hover 完整内容。",
        },
        "column-fetched-at": {
          label: "抓取时间",
          desc: "上游响应被写入缓存的 UTC 时间。**主行**显示绝对时间 `YYYY-MM-DD HH:mm:ss`，**副行**显示相对时间（如「3 小时前」）。缓存年龄 = `now() - fetched_at`。",
        },
        "column-raw-preview": {
          label: "原始预览",
          desc: "缓存 body 的**前 200 字符**（已经在后端截断，admin 接口本身不传完整 body）。用 `<code>` + `line-clamp-2` 在表格内只显示 2 行；点「查看完整」打开预览 Modal。",
        },
        "column-operations": {
          label: "操作",
          desc: "右侧固定列，三个按钮：**查看完整** / **强制过期** / **删除**（带 Popconfirm 二次确认）。详见「可用操作与审计」章节。",
        },
        "action-view-full": {
          label: "操作 · 查看完整",
          desc: "打开 Preview Modal，展示该行 `raw_preview`（前 200 字）。Modal 内有提示：完整 body 需要直接查 DB。**不是真的完整**——名字略带历史感，未来会替换为「预览前 200 字」。",
        },
        "action-expire": {
          label: "操作 · 强制过期",
          desc: "调 `POST /api/admin/cache/{table}/{id}/refresh`，后端把 `fetched_at` 拨到足够古老（如 1970 年）。**不删行**——下一次相同请求 miss 后回源，新值直接覆盖旧行。适合在上游数据有更新但 cache 还没到 TTL 时强刷。",
        },
        "action-delete": {
          label: "操作 · 删除",
          desc: "硬删除该行。带 Popconfirm 二次确认。**不可恢复**：下次相同请求会冷启动回源（如果上游恰好挂了，软兜底也没东西可兜）。仅在确实需要清理脏数据时使用。",
        },
        "preview-modal": {
          label: "预览 Modal",
          desc: "宽度 720，展示 `raw_preview` 前 200 字。`<pre>` 渲染，保留换行，自动 wrap。最大高度 60vh，超出滚动。无复制按钮——浏览器直接选中复制即可。",
        },
      },
    },
  },
};

export default zh;
