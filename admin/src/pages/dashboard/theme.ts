import type { ResolvedAdminThemeMode } from "../../theme";

export const VIOLET = "#8b5cf6";
export const VIOLET_LIGHT = "#a78bfa";
export const VIOLET_PALE = "#c4b5fd";
export const VIOLET_FAINT = "#ddd6fe";
export const PINK_ACCENT = "#fb7185";

export const VIOLET_RAMP = [
  VIOLET,
  VIOLET_LIGHT,
  VIOLET_PALE,
  VIOLET_FAINT,
  "#7c3aed",
  "#6d28d9",
];

export const CATEGORICAL = [
  VIOLET,
  VIOLET_LIGHT,
  "#9ca3af",
  VIOLET_PALE,
  "#6b7280",
  VIOLET_FAINT,
];

export type ChartPalette = {
  axisLabel: string;
  grid: string;
  text: string;
  tooltipBg: string;
  tooltipFg: string;
  tooltipBorder: string;
};

export function getPalette(mode: ResolvedAdminThemeMode): ChartPalette {
  return mode === "dark"
    ? {
        axisLabel: "#9a9aa3",
        grid: "#27272a",
        text: "#ededed",
        tooltipBg: "rgba(24,24,28,0.95)",
        tooltipFg: "#ededed",
        tooltipBorder: "#27272a",
      }
    : {
        axisLabel: "#9a9aa3",
        grid: "#f4f4f5",
        text: "#1a1a1a",
        tooltipBg: "rgba(255,255,255,0.98)",
        tooltipFg: "#1a1a1a",
        tooltipBorder: "#e5e5e7",
      };
}

/** Apple-Health-style axis: only horizontal grid, no vertical lines, no axis line. */
export function appleHealthAxis(palette: ChartPalette) {
  return {
    x: {
      labelFill: palette.axisLabel,
      labelFontSize: 11,
      line: false as const,
      tick: false as const,
      gridStroke: "transparent",
    },
    y: {
      labelFill: palette.axisLabel,
      labelFontSize: 11,
      line: false as const,
      tick: false as const,
      gridStroke: palette.grid,
      gridLineWidth: 1,
    },
  };
}
