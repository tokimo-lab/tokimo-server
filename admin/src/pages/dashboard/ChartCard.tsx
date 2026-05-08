import { GripVertical } from "lucide-react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";

type Props = {
  metric: string;
  value: ReactNode;
  hint?: ReactNode;
  trailing?: ReactNode;
  children: ReactNode;
  dragHandleProps?: Record<string, unknown>;
  className?: string;
};

/**
 * Apple Health style chart card:
 * - small uppercase metric label top-left
 * - large hero number
 * - chart area below
 */
export function ChartCard({
  metric,
  value,
  hint,
  trailing,
  children,
  dragHandleProps,
  className,
}: Props) {
  const { t } = useTranslation();
  const cls = [
    "group relative flex h-full min-h-[360px] flex-col rounded-2xl",
    "border border-border-light bg-panel-light p-5",
    "shadow-[0_1px_2px_0_rgba(0,0,0,0.04),0_1px_3px_0_rgba(0,0,0,0.06)]",
    "transition-shadow hover:shadow-md",
    "dark:border-border-dark dark:bg-panel-dark",
    "dark:shadow-[0_1px_2px_0_rgba(0,0,0,0.4),0_1px_3px_0_rgba(0,0,0,0.5)]",
    className ?? "",
  ].join(" ");

  return (
    <div className={cls}>
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-1.5">
            {dragHandleProps ? (
              <button
                type="button"
                aria-label={t("dashboard.charts.dragHint")}
                className="-ml-1 flex h-5 w-4 cursor-grab touch-none items-center justify-center rounded text-fg-muted-light/40 opacity-0 transition-opacity hover:text-fg-muted-light active:cursor-grabbing group-hover:opacity-100 dark:text-fg-muted-dark/40 dark:hover:text-fg-muted-dark"
                {...dragHandleProps}
              >
                <GripVertical size={14} />
              </button>
            ) : null}
            <div className="text-[11px] font-semibold tracking-[0.08em] text-fg-muted-light uppercase dark:text-fg-muted-dark">
              {metric}
            </div>
          </div>
          <div className="mt-1.5 text-3xl font-semibold tracking-[-0.03em] text-fg-light tabular-nums dark:text-fg-dark">
            {value}
          </div>
          {hint ? (
            <div className="mt-1 text-xs text-fg-muted-light dark:text-fg-muted-dark">
              {hint}
            </div>
          ) : null}
        </div>
        {trailing}
      </div>
      <div className="mt-4 flex min-h-0 flex-1 flex-col">{children}</div>
    </div>
  );
}
