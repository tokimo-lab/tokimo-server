---
name: admin-ui-style
description: |
  Force-trigger this skill for any tokimo-server admin UI change: pages, components, layout, CSS, Ant Design theme, visual polish, empty states, dashboards, login, navigation, or style documentation. Linear is the visual ground truth: flat minimal panels, subtle 1px borders, low shadows, medium-tight density, and restrained brand accents only.
---

# admin-ui-style

## Visual ground truth

The admin UI follows Linear-style minimal product UI, not marketing UI:

- Flat background, flat panels, 1px subtle borders.
- Near-invisible shadows; never use heavy elevation.
- Medium-tight density with clear hierarchy.
- Dual theme follows `prefers-color-scheme` by default and may be explicitly set to light/dark.
- Brand color is violet `#8b5cf6`; hover `#a78bfa`; active `#7c3aed`.
- Brand gradient is an accent only: `linear-gradient(135deg, #3b82f6 0%, #8b5cf6 50%, #ec4899 100%)`.

## Token source

Use `admin/src/styles/tokens.css` as the only visual token source.

Required tokens:

- Surfaces: `--tks-bg`, `--tks-panel`, `--tks-panel-hover`.
- Borders: `--tks-border`, `--tks-border-strong`.
- Text: `--tks-fg`, `--tks-fg-muted`, `--tks-fg-subtle`.
- Brand: `--tks-primary`, `--tks-primary-hover`, `--tks-primary-active`, `--tks-focus-ring`.
- Status: `--tks-success`, `--tks-warning`, `--tks-danger`.
- Radius: `--tks-radius-sm`, `--tks-radius-md`, `--tks-radius-lg`, `--tks-radius-pill`.
- Shadow: `--tks-shadow-sm`, `--tks-shadow-md`, `--tks-shadow-lg`.
- Spacing: `--tks-space-1` through `--tks-space-8`.
- Type: `--tks-text-xs`, `--tks-text-sm`, `--tks-text-base`, `--tks-text-lg`, `--tks-text-xl`, `--tks-text-2xl`, `--tks-text-3xl`.
- Motion: `--tks-easing`, `--tks-duration-fast`, `--tks-duration`.
- Gradient: `--tks-gradient-brand`, `--tks-gradient-brand-soft`.

Frozen values:

| Token intent | Light | Dark |
|---|---:|---:|
| Background | `#fafafa` | `#08080b` |
| Panel | `#ffffff` | `#111114` |
| Primary | `#8b5cf6` | `#8b5cf6` |
| Hover | `#a78bfa` | `#a78bfa` |
| Active | `#7c3aed` | `#7c3aed` |
| Card radius | `8px` | `8px` |
| Button/input radius | `6px` | `6px` |
| Pill radius | `999px` | `999px` |

## Gradient placement whitelist

The brand gradient may appear in exactly these six places:

1. Logo / wordmark text or mark.
2. Primary button hover / active border, not the default resting button fill.
3. Stat card number via `background-clip: text`.
4. Chart primary line or pie active slice.
5. Empty state and login hero illustration.
6. Sider active left indicator as a 3px strip.

Anywhere else requires removing the gradient.

## Strict DO NOT list

- No glassmorphism.
- No `backdrop-filter` or `-webkit-backdrop-filter`.
- No `rgba()` translucent panels; panels must be solid `--tks-panel`.
- No mesh background, radial ambient background, gradient body background, or gradient large surfaces.
- No multi-color rainbow cards, tables, panels, headers, or page backgrounds.
- No inline `style={{}}`; use CSS classes and tokens.
- No icon library except `@ant-design/icons`.
- No heavy shadows; max shadow is `0 4px 12px rgba(0,0,0,0.08)` or token equivalent.
- No animation longer than `300ms`.
- No Ant Design default blue `colorPrimary`; it must be `#8b5cf6`.
- No new dependencies for visual work.

## Ant Design requirements

`ConfigProvider` must align with tokens:

- `algorithm`: `isDark ? theme.darkAlgorithm : theme.defaultAlgorithm`.
- `colorPrimary`: `#8b5cf6`.
- `colorPrimaryHover`: `#a78bfa`.
- `colorPrimaryActive`: `#7c3aed`.
- `colorBgBase`: dark `#08080b`, light `#fafafa`.
- `colorBgContainer`: dark `#111114`, light `#ffffff`.
- `colorBorder`: dark `#1f1f23`, light `#e5e5e7`.
- `colorBorderSecondary`: dark `#16161a`, light `#efefef`.
- `colorText`: dark `#ededed`, light `#1a1a1a`.
- `colorTextSecondary`: dark `#9a9aa3`, light `#5e5e66`.
- `fontFamily`: Inter plus system fallback.
- `fontSize`: `13`.
- `borderRadius`: `8`; `borderRadiusSM`: `6`; `borderRadiusLG`: `10`.
- Motion uses `150ms` to `200ms` and `cubic-bezier(0.4, 0, 0.2, 1)`.

## Before / after corrections

### Glass panel to Linear panel

Before:

```css
.header {
  background: rgba(255, 255, 255, 0.62);
  backdrop-filter: blur(24px);
  box-shadow: 0 24px 80px rgba(0, 0, 0, 0.18);
}
```

After:

```css
.header {
  background: var(--tks-panel);
  border-bottom: 1px solid var(--tks-border);
  box-shadow: none;
}
```

### Inline style to token class

Before:

```tsx
<Card style={{ background: "#fff", borderRadius: 16, padding: 24 }} />
```

After:

```tsx
<Card className="admin-card" />
```

```css
.admin-card {
  background: var(--tks-panel);
  border: 1px solid var(--tks-border);
  border-radius: var(--tks-radius-md);
  padding: var(--tks-space-6);
}
```

### Gradient surface to whitelisted accent

Before:

```css
.page {
  background: radial-gradient(circle, #3b82f6, #ec4899);
}
```

After:

```css
.page {
  background: var(--tks-bg);
}

.logo {
  background: var(--tks-gradient-brand);
  background-clip: text;
  color: transparent;
}
```

### Ant Design primary correction

Before:

```tsx
<ConfigProvider theme={{ token: { colorPrimary: "#1677ff" } }}>
```

After:

```tsx
<ConfigProvider theme={{ token: { colorPrimary: "#8b5cf6" } }}>
```
