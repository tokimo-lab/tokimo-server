import { lazy } from "react";
import type { ResolvedAdminThemeMode } from "../../theme";
import {
  CATEGORICAL,
  PINK_ACCENT,
  VIOLET,
  VIOLET_LIGHT,
  VIOLET_PALE,
  VIOLET_RAMP,
  appleHealthAxis,
  getPalette,
} from "./theme";

export const Line = lazy(() =>
  import("@ant-design/plots").then((m) => ({ default: m.Line })),
);
export const Pie = lazy(() =>
  import("@ant-design/plots").then((m) => ({ default: m.Pie })),
);
export const Column = lazy(() =>
  import("@ant-design/plots").then((m) => ({ default: m.Column })),
);
export const Area = lazy(() =>
  import("@ant-design/plots").then((m) => ({ default: m.Area })),
);
export const Heatmap = lazy(() =>
  import("@ant-design/plots").then((m) => ({ default: m.Heatmap })),
);

type Mode = ResolvedAdminThemeMode;

export function timeseriesLineConfig(
  data: Array<{ time: string; value: number; metric: string }>,
  mode: Mode,
  labels: { calls: string; errors: string; hits: string; misses: string },
) {
  const palette = getPalette(mode);
  return {
    data,
    xField: "time",
    yField: "value",
    colorField: "metric",
    height: 260,
    autoFit: true,
    background: "transparent",
    smooth: true,
    style: { lineWidth: 2.5 },
    point: { size: 0 },
    axis: appleHealthAxis(palette),
    scale: {
      color: {
        domain: [labels.calls, labels.hits, labels.misses, labels.errors],
        range: [VIOLET, VIOLET_LIGHT, VIOLET_PALE, PINK_ACCENT],
      },
    },
    legend: {
      color: { itemLabelFill: palette.text, itemLabelFontSize: 11 },
    },
    theme: { type: mode === "dark" ? "classicDark" : "classic" },
  };
}

export function pieConfig(
  data: Array<{ provider: string; calls: number }>,
  mode: Mode,
) {
  const palette = getPalette(mode);
  return {
    data,
    angleField: "calls",
    colorField: "provider",
    innerRadius: 0.65,
    height: 260,
    autoFit: true,
    background: "transparent",
    label: false as const,
    scale: { color: { range: VIOLET_RAMP } },
    state: { active: { scale: 1.04 } },
    legend: {
      color: {
        itemLabelFill: palette.text,
        itemLabelFontSize: 11,
        position: "right",
      },
    },
    theme: { type: mode === "dark" ? "classicDark" : "classic" },
  };
}

export function providerColumnConfig(
  data: Array<{ provider: string; metric: string; value: number }>,
  mode: Mode,
  labels: { calls: string; errors: string },
) {
  const palette = getPalette(mode);
  return {
    data,
    xField: "provider",
    yField: "value",
    colorField: "metric",
    height: 260,
    autoFit: true,
    background: "transparent",
    group: true,
    style: { radiusTopLeft: 4, radiusTopRight: 4, maxWidth: 22 },
    axis: appleHealthAxis(palette),
    scale: {
      color: {
        domain: [labels.calls, labels.errors],
        range: [VIOLET, PINK_ACCENT],
      },
    },
    legend: { color: { itemLabelFill: palette.text, itemLabelFontSize: 11 } },
    theme: { type: mode === "dark" ? "classicDark" : "classic" },
  };
}

export function latencyLineConfig(
  data: Array<{ time: string; value: number; metric: string }>,
  mode: Mode,
  labels: { p50: string; p95: string },
) {
  const palette = getPalette(mode);
  return {
    data,
    xField: "time",
    yField: "value",
    colorField: "metric",
    height: 200,
    autoFit: true,
    background: "transparent",
    smooth: true,
    style: { lineWidth: 2.5 },
    point: { size: 0 },
    axis: appleHealthAxis(palette),
    scale: {
      color: {
        domain: [labels.p50, labels.p95],
        range: [VIOLET, VIOLET_PALE],
      },
    },
    legend: { color: { itemLabelFill: palette.text, itemLabelFontSize: 11 } },
    theme: { type: mode === "dark" ? "classicDark" : "classic" },
  };
}

export function errorsAreaConfig(
  data: Array<{ time: string; errors: number }>,
  mode: Mode,
) {
  const palette = getPalette(mode);
  return {
    data,
    xField: "time",
    yField: "errors",
    height: 200,
    autoFit: true,
    background: "transparent",
    shapeField: "smooth",
    style: {
      fill: "linear-gradient(180deg, rgba(251,113,133,0.4) 0%, rgba(251,113,133,0) 100%)",
      fillOpacity: 1,
      lineWidth: 2,
      stroke: PINK_ACCENT,
    },
    line: { style: { stroke: PINK_ACCENT, lineWidth: 2 } },
    axis: appleHealthAxis(palette),
    legend: false as const,
    theme: { type: mode === "dark" ? "classicDark" : "classic" },
  };
}

export function heatmapConfig(
  data: Array<{ time: string; provider: string; calls: number }>,
  mode: Mode,
) {
  const palette = getPalette(mode);
  return {
    data,
    xField: "time",
    yField: "provider",
    colorField: "calls",
    height: 240,
    autoFit: true,
    background: "transparent",
    style: { inset: 1.5 },
    scale: {
      color: { range: ["rgba(139,92,246,0.04)", VIOLET] },
    },
    axis: appleHealthAxis(palette),
    legend: { color: { itemLabelFill: palette.text, itemLabelFontSize: 11 } },
    theme: { type: mode === "dark" ? "classicDark" : "classic" },
  };
}

export function statusCodesConfig(
  data: Array<{ time: string; metric: string; value: number }>,
  mode: Mode,
  labels: { ok: string; c4xx: string; c5xx: string },
) {
  const palette = getPalette(mode);
  return {
    data,
    xField: "time",
    yField: "value",
    colorField: "metric",
    height: 240,
    autoFit: true,
    background: "transparent",
    stack: true,
    style: { radiusTopLeft: 3, radiusTopRight: 3, maxWidth: 24 },
    axis: appleHealthAxis(palette),
    scale: {
      color: {
        domain: [labels.ok, labels.c4xx, labels.c5xx],
        range: [VIOLET, VIOLET_LIGHT, PINK_ACCENT],
      },
    },
    legend: { color: { itemLabelFill: palette.text, itemLabelFontSize: 11 } },
    theme: { type: mode === "dark" ? "classicDark" : "classic" },
  };
}

export { CATEGORICAL };
