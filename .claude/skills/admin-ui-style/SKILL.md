---
name: admin-ui-style
description: |
  Force-trigger this skill for any tokimo-server admin UI change: pages, components, layout, Tailwind classes, Ant Design theme, visual polish, empty states, dashboards, login, navigation, or style documentation. Linear is the visual ground truth: flat minimal panels, subtle 1px borders, low shadows, medium-tight density, and restrained brand accents only.
---

# admin-ui-style

## Visual ground truth

The admin UI follows Linear-style minimal product UI, not marketing UI:

- Flat background, flat panels, 1px subtle borders.
- Near-invisible shadows; never use heavy elevation.
- Medium-tight density with clear hierarchy.
- Dual theme follows `prefers-color-scheme` by default and may be explicitly set to light/dark/system.
- Brand color is violet `#8b5cf6`; hover `#a78bfa`; active `#7c3aed`.
- Brand gradient is an accent only: `linear-gradient(135deg, #3b82f6 0%, #8b5cf6 50%, #ec4899 100%)`.

## Tailwind v4 source of truth

Use Tailwind CSS v4 only.

- `admin/src/styles/index.css` is the CSS-first Tailwind entry and owns `@import "tailwindcss"`, `@theme`, `@custom-variant dark`, base setup, and the shared gradient utilities.
- `admin/src/styles/reset.css` is the only reset file and should stay tiny and framework-neutral.
- Do not create component/page CSS files. No custom CSS files except `reset.css` and `index.css`.
- Do not use deprecated Phase 8 v3 legacy tks-prefixed CSS variables; they are removed.
- Do not use JSX inline style props; use Tailwind `className` utilities.
- Add new `@theme` tokens only when the value is reused 3+ times. Otherwise use Tailwind arbitrary values.

Frozen values:

| Token intent | Light | Dark |
|---|---:|---:|
| Background | `#fafafa` | `#08080b` |
| Panel | `#ffffff` | `#111114` |
| Border | `#e5e5e7` | `#1f1f23` |
| Muted text | `#5e5e66` | `#9a9aa3` |
| Primary | `#8b5cf6` | `#8b5cf6` |
| Hover | `#a78bfa` | `#a78bfa` |
| Active | `#7c3aed` | `#7c3aed` |
| Card radius | `8px` | `8px` |
| Button/input radius | `6px` | `6px` |

## Tailwind v4 usage tips

- Use semantic theme utilities from `@theme`: `bg-bg-light`, `dark:bg-bg-dark`, `bg-panel-light`, `dark:bg-panel-dark`, `border-border-light`, `dark:border-border-dark`, `text-fg-muted-light`, `dark:text-fg-muted-dark`.
- Use `dark:` variants; dark mode is driven by `[data-theme="dark"]` via `@custom-variant`.
- Use arbitrary values for one-off precision: `h-[52px]`, `w-[3px]`, `text-[28px]`, `bg-[#18181c]`.
- Use Tailwind arbitrary variants to target Ant Design internals only when needed, e.g. `[&_.ant-table]:!bg-transparent`.
- Use shared utilities only for whitelisted gradients: `.gradient-text`, `.gradient-bg`, `.gradient-ring-hover`.

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
- No translucent panels; panels must be solid light/dark panel colors.
- No mesh background, radial ambient background, gradient body background, or gradient large surfaces.
- No multi-color rainbow cards, tables, panels, headers, or page backgrounds.
- No inline JSX style props; use Tailwind classes.
- No legacy tks-prefixed CSS variables.
- No icon library except `@ant-design/icons`.
- No heavy shadows; max shadow is `0 4px 12px rgba(0,0,0,0.08)` or lighter.
- No animation longer than `300ms`.
- No Ant Design default blue `colorPrimary`; it must be `#8b5cf6`.
- No new dependencies for visual work.

## Ant Design requirements

`ConfigProvider` must align with Tailwind theme tokens:

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
- `fontFamily`: `Inter, system-ui, -apple-system, sans-serif`.
- `fontSize`: `13`.
- `borderRadius`: `8`; `borderRadiusSM`: `6`; `borderRadiusLG`: `10`.
- Motion duration uses `0.15s` to `0.2s`.

## Before / after corrections

### Deprecated Phase 8 v3 variable to Tailwind v4

Before:

```tsx
<Card className="tks-card" />
```

After:

```tsx
<Card className="rounded-lg border border-border-light bg-panel-light dark:border-border-dark dark:bg-panel-dark" />
```

### Inline style to Tailwind utility

Before:

```tsx
<Card inline-style-prop />
```

After:

```tsx
<Card className="rounded-lg border border-border-light bg-panel-light p-6 dark:border-border-dark dark:bg-panel-dark" />
```

### Gradient surface to whitelisted accent

Before:

```tsx
<div className="min-h-screen bg-[radial-gradient(circle,#3b82f6,#ec4899)]" />
```

After:

```tsx
<div className="min-h-screen bg-bg-light dark:bg-bg-dark">
  <h1 className="gradient-text">Tokimo</h1>
</div>
```
