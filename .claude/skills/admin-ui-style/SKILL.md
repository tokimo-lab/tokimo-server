---
name: admin-ui-style
description: |
  tokimo-server admin 前端 UI 风格规范（强制）。Apple HIG / iCloud Web 风：玻璃毛玻璃 + 高对比 + 多彩渐变点缀（橙→粉→紫）。涵盖色板、间距、圆角、动画、字体、组件边界、antd override、施工中组件、不要做清单、违规代码修复参考。触发词：写组件 / 改样式 / 加页面 / UI 调整 / 配色 / 动画 / 玻璃 / 主题 / 施工中。
---

## 整体语言

admin 前端遵循 Apple HIG / iCloud Web 风：柔和纯色页面背景、清晰内容层级、必要位置使用玻璃毛玻璃、多彩暖色渐变只做点缀。后台界面优先可读性和业务效率，不做营销页式大面积渐变或强装饰。

## 设计 token 来源

唯一来源是 `admin/src/styles/tokens.css`。使用下列 token，不新增 `--tks-ambient-*`、`--tks-text-*` 或同义 alias。

- `--tks-space-1: 8px;`
- `--tks-space-2: 16px;`
- `--tks-space-3: 24px;`
- `--tks-space-4: 32px;`
- `--tks-space-5: 48px;`
- `--tks-space-6: 64px;`
- `--tks-radius-sm: 6px;`
- `--tks-radius-md: 8px;`
- `--tks-radius-lg: 10px;`
- `--tks-radius-xl: 14px;`
- `--tks-ease-spring: cubic-bezier(0.32, 0.72, 0, 1);`
- `--tks-duration-fast: 160ms;`
- `--tks-duration-base: 240ms;`
- `--tks-duration-slow: 320ms;`
- `--tks-font-sans: "Inter", -apple-system, BlinkMacSystemFont, "SF Pro Text", system-ui, sans-serif;`
- `--tks-font-mono: "JetBrains Mono", "SF Mono", ui-monospace, "Cascadia Mono", monospace;`
- `--tks-gradient-warm: linear-gradient(135deg, #FF8A3D 0%, #FF5CA1 50%, #8B5CF6 100%);`
- `--tks-gradient-warm-soft: linear-gradient(135deg, rgba(255,138,61,0.16) 0%, rgba(255,92,161,0.16) 50%, rgba(139,92,246,0.16) 100%);`
- Light：`--tks-bg-app: #F8F5F0;`、`--tks-bg-elevated: #FFFFFF;`、`--tks-bg-glass: rgba(255, 255, 255, 0.62);`、`--tks-bg-glass-strong: rgba(255, 255, 255, 0.78);`、`--tks-border-subtle: rgba(0, 0, 0, 0.06);`、`--tks-border-base: rgba(0, 0, 0, 0.10);`、`--tks-fg-primary: rgba(0, 0, 0, 0.92);`、`--tks-fg-secondary: rgba(0, 0, 0, 0.62);`、`--tks-fg-muted: rgba(0, 0, 0, 0.40);`、`--tks-shadow-card: 0 1px 2px rgba(0, 0, 0, 0.04), 0 8px 24px rgba(0, 0, 0, 0.06);`、`--tks-shadow-glass: 0 8px 32px rgba(0, 0, 0, 0.08);`
- Dark：`--tks-bg-app: #14141A;`、`--tks-bg-elevated: #1F1F26;`、`--tks-bg-glass: rgba(28, 28, 36, 0.62);`、`--tks-bg-glass-strong: rgba(28, 28, 36, 0.78);`、`--tks-border-subtle: rgba(255, 255, 255, 0.06);`、`--tks-border-base: rgba(255, 255, 255, 0.10);`、`--tks-fg-primary: rgba(255, 255, 255, 0.94);`、`--tks-fg-secondary: rgba(255, 255, 255, 0.62);`、`--tks-fg-muted: rgba(255, 255, 255, 0.40);`、`--tks-shadow-card: 0 1px 2px rgba(0, 0, 0, 0.30), 0 8px 24px rgba(0, 0, 0, 0.40);`、`--tks-shadow-glass: 0 8px 32px rgba(0, 0, 0, 0.50);`

## 色板

页面底色使用 `--tks-bg-app`，内容卡片使用 `--tks-bg-elevated`，玻璃层使用 `--tks-bg-glass` / `--tks-bg-glass-strong`。文字只用 `--tks-fg-primary`、`--tks-fg-secondary`、`--tks-fg-muted`，边框只用 `--tks-border-subtle` / `--tks-border-base`。

## 字体规则

全局字体走 `--tks-font-sans`。代码、日志、终端、token 示例走 `--tks-font-mono`。不要在 TSX inline style 中重复写 font-family。

## 间距

间距只用 `--tks-space-1` 到 `--tks-space-6`。表单小间距优先 `--tks-space-1` / `--tks-space-2`，页面级 padding 优先 `--tks-space-5` / `--tks-space-6`。

## 圆角

控件和标签使用 `--tks-radius-sm` / `--tks-radius-md`，主卡片使用 `--tks-radius-lg`，大容器最多使用 `--tks-radius-xl`。不要用 16px、24px 等脱离 token 的随手圆角。

## 毛玻璃用法

毛玻璃用于 header、sider、浮层或轻量悬浮面板，不用于主内容卡片。标准片段：

```css
.tks-glass {
  background: var(--tks-bg-glass);
  border: 1px solid var(--tks-border-subtle);
  box-shadow: var(--tks-shadow-glass);
  backdrop-filter: blur(24px) saturate(180%);
  -webkit-backdrop-filter: blur(24px) saturate(180%);
}
```

## 多彩渐变

`--tks-gradient-warm` 和 `--tks-gradient-warm-soft` 只用于 logo、avatar、blob、徽章等小面积点缀。body、app、登录页、主布局背景必须是 `--tks-bg-app` 这类柔和纯色，不使用 radial ambient 背景。

## 动画规则

动效使用 `--tks-duration-fast`、`--tks-duration-base`、`--tks-duration-slow` 和 `--tks-ease-spring`。禁止 `transition: all 0.5s`，只声明实际变化的属性。

## 组件层级

布局外壳使用 `--tks-bg-app`；导航、header 可叠 `.tks-glass`；业务内容用 `.tks-card`；卡片内部再分组时只加边框或间距，不继续叠加毛玻璃。

## antd ConfigProvider override 模板

```tsx
const lightToken = {
  colorBgLayout: "#F8F5F0",
  colorBgContainer: "#FFFFFF",
  colorBorder: "rgba(0,0,0,0.10)",
  colorBorderSecondary: "rgba(0,0,0,0.06)",
  colorText: "rgba(0,0,0,0.92)",
  colorTextSecondary: "rgba(0,0,0,0.62)",
  colorTextTertiary: "rgba(0,0,0,0.40)",
};

const darkToken = {
  colorBgLayout: "#14141A",
  colorBgContainer: "#1F1F26",
  colorBorder: "rgba(255,255,255,0.10)",
  colorBorderSecondary: "rgba(255,255,255,0.06)",
  colorText: "rgba(255,255,255,0.94)",
  colorTextSecondary: "rgba(255,255,255,0.62)",
  colorTextTertiary: "rgba(255,255,255,0.40)",
};
```

## 施工中组件

未完成页面统一使用 `ConstructionPage`，传入 `pageName` 和可选 `estimate`。施工卡片宽度使用 `width: min(100%, 560px)`，可配 `min-width: min(100%, 480px)`，确保移动端不横向溢出。

## i18n

可见文案优先接入现有 i18n；临时施工文案可以保留组件内部固定文案，但不要把真实业务页面的新中文/英文散落在 TSX 中。

## Lint

样式或前端改动后运行 admin 现有 lint/typecheck 命令。若只做 token 文档更新，至少跑 targeted grep 检查违规 token、旧 rgba、硬编码白色 avatar。

## 不要做（DON'T LIST）

- 不要新增 UI 框架或样式依赖。
- 不要绕开 Ant Design 重写表单、表格、Modal、Dropdown 等业务组件。
- 不要在 TSX 里硬编码颜色、阴影、圆角、动画时长。
- 不要新增 `--tks-ambient-*`、`--tks-text-*` 或同义 token alias。
- 不要给 body/app/login 使用 radial gradient 或 ambient 背景。
- 不要在主内容卡片上使用 backdrop-filter。
- 不要使用 `transition: all`。

## 违规代码修复参考

Before：
```tsx
<Content style={{ background: "#fff", borderRadius: 16, padding: 24 }}>
  <Outlet />
</Content>
```
After：
```tsx
<Content className="tks-card tks-admin-content">
  <Outlet />
</Content>
```

Before：
```css
.panel {
  transition: all 0.5s ease;
}
```
After：
```css
.panel {
  transition: opacity var(--tks-duration-base) var(--tks-ease-spring),
    transform var(--tks-duration-base) var(--tks-ease-spring);
}
```

Before：
```tsx
function SettingsPage() {
  return <div>Coming Soon</div>;
}
```
After：
```tsx
function SettingsPage() {
  return <ConstructionPage pageName="Settings" />;
}
```

Before：
```css
.login {
  background: radial-gradient(circle, var(--tks-ambient-warm-soft), transparent), var(--tks-bg-app);
}
```
After：
```css
.login {
  background: var(--tks-bg-app);
}
```
