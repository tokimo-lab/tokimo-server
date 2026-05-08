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
        "代理服务的实时运行视图：在所选时间窗内聚合的请求量、错误率、延迟分位数以及各 Provider 占比。",
      sections: {
        overview: {
          title: "概览",
          body: "总览面板汇总了**所有已配置 Provider** 的指标，并以时间序列、热力图和环形图渲染。数据来自 `dashboard_*` 一组接口，前端缓存 15 秒。顶部的时间范围切换会同步影响所有图表——这是有意为之的设计，单卡片不提供独立时间控件。",
        },
        metrics: {
          title: "指标说明",
          body: "每张图直接对应一个后端 rollup：\n\n- **活动环** — 时间窗内成功 / 失败请求数\n- **时间序列** — 按桶聚合的请求量\n- **延迟** — 基于单请求样本计算的 p50 / p95 / p99\n- **热力图** — 小时 × 星期的错误密度\n- **Provider 柱图** — 流量 Top-N Provider\n\n所有指标都在服务器端完成聚合，前端不会对原始记录二次计算。",
        },
        refresh: {
          title: "刷新与缓存",
          body: "React Query 每 30 秒轮询一次，`staleTime` 为 15 秒。**刷新**按钮会强制对所有可见卡片触发 `refetch`。标题处还有一个隐藏的调试入口（连续 5 次快速点击）允许运维清空 rollup 窗口——会**永久重置**指标，请谨慎使用。",
        },
      },
      fields: {
        range: {
          label: "range",
          desc: "选定的时间窗。会作为 `range` 查询参数透传给所有 dashboard 接口。允许值：`1h`、`24h`、`7d`。",
        },
        interval: {
          label: "bucket",
          desc: "服务端根据时间窗推导出的桶大小（秒）。前端不可控制，图表直接读取响应里的 `bucket` 字段，使 x 轴刻度与服务端一致。",
        },
      },
    },
    "provider-configs": {
      title: "Provider 配置",
      summary:
        "代理已接入的上游 API Provider 只读清单。展示每个 Provider 的路由健康度、近 24 小时流量以及示例请求 URL。",
      sections: {
        overview: {
          title: "概览",
          body: "Provider 定义来自代码与配置文件，**无法在此页面编辑**。表格是一个**运行时镜像**：展示服务器当前注册的 Provider，以及从指标 rollup 中读取的近 24 小时计数。",
        },
        "sample-url": {
          title: "示例 URL",
          body: "点击行打开响应检查器。该模态会回放该 Provider **最近一次成功的上游调用**（若全部失败则展示最近一次失败响应），方便在不打扰生产流量的前提下验证鉴权头、响应结构与限流字段。",
        },
      },
      fields: {
        provider: {
          label: "provider",
          desc: "内部 Provider 标识（如 `tmdb`、`omdb`）。对应路由前缀 `/providers/{id}/...`。",
        },
        status: {
          label: "status",
          desc: "依据最近 5 分钟流量推导的路由健康度。`healthy` = 成功率 > 95%；`degraded` = 50–95%；`down` = < 50% 或无流量。",
        },
        "24h_calls": {
          label: "24h calls",
          desc: "近 24 小时内路由到该 Provider 的请求总数，包含缓存命中与未命中。",
        },
        hit_ratio: {
          label: "hit ratio",
          desc: "近 24 小时缓存命中率。值偏低通常意味着缓存冷启动或 TTL 过短——可结合「缓存检查器」交叉验证。",
        },
      },
    },
    "service-keys": {
      title: "服务密钥",
      summary:
        "签发、查看与吊销下游服务调用本代理时使用的长期 API Token。每个 Token 拥有固定的 scope 集合与不可变的签发时间。",
      sections: {
        overview: {
          title: "概览",
          body: "服务密钥是由 admin 签发的 JWT 风格 Bearer Token。它们仅用于**服务器到服务器**的调用，请勿嵌入到浏览器应用中。Token 创建后，原始字符串**只会展示一次**——请立即复制，否则只能重新签发。",
        },
        "token-format": {
          title: "Token 格式",
          body: "Token 由服务端 HMAC-SHA256 私钥签名，格式为 `tks_<id>.<sig>`。`id` 段是数据库主键；`sig` 段是对 `{id, scopes, created_at}` 计算的 HMAC。验签使用恒定时间比较。",
        },
        scopes: {
          title: "权限范围",
          body: "每个密钥都附带一份显式的 scope 白名单（例如 `cache:read`、`dashboard:read`、`providers:write`）。Scope 由路由守卫层校验；空 scope 集合等价于一个能通过认证但无法访问任何资源的 Token。",
        },
      },
      fields: {
        token: {
          label: "token",
          desc: "原始 Bearer 字符串。表中只持久化前缀 `tks_<id>`，秘密部分不会落库——丢失后只能重新签发。",
        },
        created_at: {
          label: "created_at",
          desc: "UTC 签发时间。用于审计，并可作为时间限定过期策略的输入。",
        },
        scopes: {
          label: "scopes",
          desc: "逗号分隔的权限集合。鉴权中间件在每个请求上读取；scope 不匹配会返回 `403 forbidden`。",
        },
      },
    },
    "cache-inspector": {
      title: "缓存检查器",
      summary:
        "查看存储在 PostgreSQL 中的 Provider 响应缓存。可浏览缓存行、预览原始响应体、强制过期或删除单行用于调试。",
      sections: {
        overview: {
          title: "概览",
          body: "每个 Provider 有一张独立的缓存表，命名为 `cache_<provider>`。行的主键是规范化后的请求 URL + 查询串。检查器在服务端按每页 50 行分页，原始响应体在打开预览前不会加载。",
        },
        ttl: {
          title: "TTL",
          body: "TTL 是**按 Provider 配置**的，部署时确定。`TTL` 列显示距离该行被判定为过期的剩余秒数。过期行在上游调用失败时仍可被服务，因此缓存同时承担「软兜底」角色。",
        },
        operations: {
          title: "可用操作",
          body: "每行支持的操作：\n\n- **Refresh** — 将 `fetched_at` 拨回足够久之前，迫使下次命中此 key 时重新请求上游\n- **Delete** — 直接删除此行\n- **Preview** — 在模态中查看缓存的原始响应体\n\n以上三种操作都会写入审计日志；删除**不可恢复**。",
        },
      },
      fields: {
        fetched_at: {
          label: "fetched_at",
          desc: "上游响应被写入缓存的 UTC 时间。缓存年龄列计算为 `now() - fetched_at`。",
        },
        ttl_seconds: {
          label: "ttl_seconds",
          desc: "该行剩余 TTL（秒）。负值表示行已过期，下一次未命中时会重新请求上游。",
        },
        key: {
          label: "key",
          desc: "规范化的请求键。由 Provider id、路由与查询参数组合生成，并做大小写归一，使等价请求落在同一行。",
        },
      },
    },
  },
};

export default zh;
