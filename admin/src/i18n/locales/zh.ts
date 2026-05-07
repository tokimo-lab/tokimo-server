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
  cache: {
    title: "缓存查看器",
    comingSoonTitle: "即将推出",
    comingSoonDescriptionPrefix: "管理接口 ",
    comingSoonDescriptionMiddle:
      " 当前是返回空列表的占位实现。待按表查询接口上线后（计划：",
    comingSoonDescriptionAnd: " 与 ",
    comingSoonDescriptionSuffix: "），本页将展示最近 N 条记录，包含 ",
    comingSoonDescriptionTail: "，并提供「强制刷新」操作。",
    tablesIntro:
      "工作区当前持久化的 Provider 缓存表（每行对应一个上游资源，并附带 TTL 过期时间列）：",
  },
};

export default zh;
