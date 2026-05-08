import {
  DndContext,
  type DragEndEvent,
  PointerSensor,
  closestCenter,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import {
  SortableContext,
  arrayMove,
  rectSortingStrategy,
  useSortable,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Alert,
  Button,
  Segmented,
  Skeleton,
  Table,
  type TableProps,
  Tooltip,
  Typography,
} from "antd";
import { RefreshCw } from "lucide-react";
import { Suspense, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  type DashboardRecentError,
  getDashboardByProvider,
  getDashboardHeatmap,
  getDashboardOverview,
  getDashboardRecentErrors,
  getDashboardStatusCodes,
  getDashboardTimeseries,
} from "../api/client";
import { useDocsRegister } from "../system/docs";
import { useAdminTheme } from "../theme";
import { ActivityRing } from "./dashboard/ActivityRing";
import { ChartCard } from "./dashboard/ChartCard";
import {
  Area,
  Column,
  Heatmap,
  Line,
  Pie,
  errorsAreaConfig,
  heatmapConfig,
  latencyLineConfig,
  pieConfig,
  providerColumnConfig,
  statusCodesConfig,
  timeseriesLineConfig,
} from "./dashboard/charts";

type RangeKey = "1h" | "24h" | "7d";
const RANGE_SECS: Record<RangeKey, { range: number; bucket: number }> = {
  "1h": { range: 3600, bucket: 300 },
  "24h": { range: 86400, bucket: 3600 },
  "7d": { range: 604800, bucket: 86400 },
};

const ORDER_STORAGE_KEY = "tokimo-admin-dashboard-order-v1";
const REFRESH_INTERVAL_STORAGE_KEY =
  "tokimo-admin-dashboard-refresh-interval-v1";

const REFRESH_INTERVAL_OPTIONS: ReadonlyArray<number> = [0, 10, 30, 60];
const DEFAULT_REFRESH_INTERVAL = 30;

function loadRefreshInterval(): number {
  try {
    const raw = localStorage.getItem(REFRESH_INTERVAL_STORAGE_KEY);
    if (raw == null) return DEFAULT_REFRESH_INTERVAL;
    const n = Number(raw);
    if (!Number.isFinite(n)) return DEFAULT_REFRESH_INTERVAL;
    return REFRESH_INTERVAL_OPTIONS.includes(n) ? n : DEFAULT_REFRESH_INTERVAL;
  } catch {
    return DEFAULT_REFRESH_INTERVAL;
  }
}

const CHART_IDS = [
  "chart-timeseries",
  "chart-cache-ring",
  "chart-top-providers",
  "chart-by-provider",
  "chart-latency",
  "chart-errors-area",
  "chart-heatmap",
  "chart-status-codes",
  "chart-cache-tables",
] as const;
type ChartId = (typeof CHART_IDS)[number];

const DEFAULT_ORDER: ChartId[] = [
  "chart-timeseries",
  "chart-cache-ring",
  "chart-top-providers",
  "chart-by-provider",
  "chart-latency",
  "chart-errors-area",
  "chart-heatmap",
  "chart-status-codes",
  "chart-cache-tables",
];

/** charts that should span 2 columns on lg+ screens */
const WIDE_CHARTS: ReadonlySet<ChartId> = new Set(["chart-timeseries"]);

function loadOrder(): ChartId[] {
  try {
    const raw = localStorage.getItem(ORDER_STORAGE_KEY);
    if (!raw) return DEFAULT_ORDER;
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return DEFAULT_ORDER;
    const valid = parsed.every(
      (id): id is ChartId =>
        typeof id === "string" && (CHART_IDS as readonly string[]).includes(id),
    );
    if (!valid || parsed.length !== CHART_IDS.length) return DEFAULT_ORDER;
    const set = new Set(parsed as ChartId[]);
    if (set.size !== CHART_IDS.length) return DEFAULT_ORDER;
    return parsed as ChartId[];
  } catch {
    return DEFAULT_ORDER;
  }
}

function toMs(ts: number): number {
  return ts < 1_000_000_000_000 ? ts * 1000 : ts;
}

function formatTime(ts: number, range: RangeKey): string {
  const date = new Date(toMs(ts));
  if (range === "7d") {
    return date.toLocaleDateString(undefined, {
      day: "2-digit",
      month: "2-digit",
    });
  }
  if (range === "24h") {
    return date.toLocaleTimeString(undefined, {
      hour: "2-digit",
      minute: "2-digit",
    });
  }
  return date.toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatNumber(value: number) {
  return new Intl.NumberFormat().format(value);
}

function formatTtlShort(seconds: number | null | undefined): string {
  if (seconds === null || seconds === undefined) return "—";
  if (seconds <= 0) return "0";
  const d = Math.floor(seconds / 86400);
  if (d > 0) return `${d}d`;
  const h = Math.floor(seconds / 3600);
  if (h > 0) return `${h}h`;
  const m = Math.floor(seconds / 60);
  if (m > 0) return `${m}m`;
  return `${seconds}s`;
}

function formatRelative(
  ts: number,
  t: ReturnType<typeof useTranslation>["t"],
): string {
  const diff = Math.max(Date.now() - toMs(ts), 0);
  const minute = 60_000;
  const hour = 60 * minute;
  const day = 24 * hour;
  if (diff < minute) return t("dashboard.relative.justNow");
  if (diff < hour)
    return t("dashboard.relative.minutesAgo", {
      count: Math.floor(diff / minute),
    });
  if (diff < day)
    return t("dashboard.relative.hoursAgo", {
      count: Math.floor(diff / hour),
    });
  return t("dashboard.relative.daysAgo", { count: Math.floor(diff / day) });
}

function ChartFallback() {
  return <Skeleton active paragraph={{ rows: 4 }} />;
}

function ChartError({
  error,
  onRetry,
}: {
  error: unknown;
  onRetry: () => void;
}) {
  const { t } = useTranslation();
  return (
    <Alert
      type="error"
      showIcon
      message={t("common.error")}
      description={error instanceof Error ? error.message : String(error)}
      action={
        <Button size="small" onClick={onRetry}>
          {t("dashboard.retry")}
        </Button>
      }
    />
  );
}

function EmptyChart() {
  const { t } = useTranslation();
  return (
    <div className="flex flex-1 flex-col items-center justify-center text-xs text-fg-muted-light dark:text-fg-muted-dark">
      {t("dashboard.empty")}
    </div>
  );
}

type SortableProps = {
  id: ChartId;
  children: (handleProps: Record<string, unknown>) => React.ReactNode;
};

function SortableSlot({ id, children }: SortableProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id });

  const style: React.CSSProperties = {
    transform: CSS.Translate.toString(transform),
    transition,
    zIndex: isDragging ? 10 : "auto",
    opacity: isDragging ? 0.85 : 1,
  };

  const wide = WIDE_CHARTS.has(id);
  const className = wide ? "lg:col-span-2" : "lg:col-span-1";

  return (
    <div ref={setNodeRef} style={style} className={`col-span-1 ${className}`}>
      {children({ ...attributes, ...listeners })}
    </div>
  );
}

function DashboardPage() {
  const { t } = useTranslation();
  const { resolvedMode } = useAdminTheme();
  const [range, setRange] = useState<RangeKey>("24h");
  const { range: rangeSecs, bucket: bucketSecs } = RANGE_SECS[range];
  const themeKey = resolvedMode;

  const [order, setOrder] = useState<ChartId[]>(() => loadOrder());
  useEffect(() => {
    try {
      localStorage.setItem(ORDER_STORAGE_KEY, JSON.stringify(order));
    } catch {
      // ignore storage errors (private mode etc.)
    }
  }, [order]);

  const [refreshInterval, setRefreshInterval] = useState<number>(() =>
    loadRefreshInterval(),
  );
  useEffect(() => {
    try {
      localStorage.setItem(
        REFRESH_INTERVAL_STORAGE_KEY,
        String(refreshInterval),
      );
    } catch {
      // ignore storage errors (private mode etc.)
    }
  }, [refreshInterval]);

  const refetchInterval =
    refreshInterval > 0 ? refreshInterval * 1000 : (false as const);
  const sharedRefetch = {
    refetchInterval,
    refetchIntervalInBackground: false,
  };

  const qc = useQueryClient();
  const [spinning, setSpinning] = useState(false);
  const handleManualRefresh = async () => {
    setSpinning(true);
    try {
      await qc.invalidateQueries({ queryKey: ["dashboard"] });
      await qc.refetchQueries({ queryKey: ["dashboard"], type: "active" });
    } finally {
      setSpinning(false);
    }
  };

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 4 } }),
  );

  const overviewQuery = useQuery({
    queryKey: ["dashboard", "overview"],
    queryFn: getDashboardOverview,
    ...sharedRefetch,
  });
  const timeseriesQuery = useQuery({
    queryKey: ["dashboard", "timeseries", rangeSecs, bucketSecs],
    queryFn: () => getDashboardTimeseries(rangeSecs, bucketSecs),
    ...sharedRefetch,
  });
  const providersQuery = useQuery({
    queryKey: ["dashboard", "by-provider", rangeSecs],
    queryFn: () => getDashboardByProvider(rangeSecs),
    ...sharedRefetch,
  });
  const heatmapQuery = useQuery({
    queryKey: ["dashboard", "heatmap", rangeSecs, bucketSecs],
    queryFn: () => getDashboardHeatmap(rangeSecs, bucketSecs),
    retry: false,
    ...sharedRefetch,
  });
  const statusQuery = useQuery({
    queryKey: ["dashboard", "status-codes", rangeSecs, bucketSecs],
    queryFn: () => getDashboardStatusCodes(rangeSecs, bucketSecs),
    retry: false,
    ...sharedRefetch,
  });
  const recentErrorsQuery = useQuery({
    queryKey: ["dashboard", "recent-errors"],
    queryFn: () => getDashboardRecentErrors(10),
    ...sharedRefetch,
  });
  const cacheTablesQuery = useQuery({
    queryKey: ["dashboard", "cache-tables"],
    queryFn: async () => {
      const { listCacheTables } = await import("../api/cache");
      return listCacheTables();
    },
    ...sharedRefetch,
  });

  const overview = overviewQuery.data;
  const timeseries = timeseriesQuery.data ?? [];

  const totalCallsInRange = useMemo(
    () => timeseries.reduce((sum, p) => sum + p.calls, 0),
    [timeseries],
  );
  const totalErrorsInRange = useMemo(
    () => timeseries.reduce((sum, p) => sum + p.errors, 0),
    [timeseries],
  );

  const lineLabels = {
    calls: t("dashboard.charts.calls"),
    errors: t("dashboard.charts.errors"),
    hits: t("dashboard.charts.cacheHits"),
    misses: t("dashboard.charts.cacheMisses"),
  };
  const lineData = useMemo(
    () =>
      timeseries.flatMap((p) => [
        {
          metric: lineLabels.calls,
          time: formatTime(p.ts, range),
          value: p.calls,
        },
        {
          metric: lineLabels.hits,
          time: formatTime(p.ts, range),
          value: p.hits,
        },
        {
          metric: lineLabels.misses,
          time: formatTime(p.ts, range),
          value: p.misses,
        },
        {
          metric: lineLabels.errors,
          time: formatTime(p.ts, range),
          value: p.errors,
        },
      ]),
    [
      timeseries,
      range,
      lineLabels.calls,
      lineLabels.hits,
      lineLabels.misses,
      lineLabels.errors,
    ],
  );

  const sortedProviders = useMemo(
    () => [...(providersQuery.data ?? [])].sort((a, b) => b.calls - a.calls),
    [providersQuery.data],
  );

  const pieData = useMemo(() => {
    const top = sortedProviders.slice(0, 10);
    const otherCalls = sortedProviders
      .slice(10)
      .reduce((sum, p) => sum + p.calls, 0);
    return otherCalls > 0
      ? [...top, { provider: t("dashboard.charts.other"), calls: otherCalls }]
      : top;
  }, [sortedProviders, t]);

  const pieTotal = useMemo(
    () => pieData.reduce((sum, d) => sum + d.calls, 0),
    [pieData],
  );

  const columnLabels = {
    calls: t("dashboard.charts.calls"),
    errors: t("dashboard.charts.errors"),
  };
  const columnData = useMemo(
    () =>
      sortedProviders.slice(0, 8).flatMap((p) => [
        { provider: p.provider, metric: columnLabels.calls, value: p.calls },
        { provider: p.provider, metric: columnLabels.errors, value: p.errors },
      ]),
    [sortedProviders, columnLabels.calls, columnLabels.errors],
  );

  const latencyLabels = {
    p50: t("dashboard.charts.p50"),
    p95: t("dashboard.charts.p95"),
  };
  const latencyData = useMemo(
    () =>
      timeseries.flatMap((p) => [
        {
          metric: latencyLabels.p50,
          time: formatTime(p.ts, range),
          value: p.p50_ms ?? 0,
        },
        {
          metric: latencyLabels.p95,
          time: formatTime(p.ts, range),
          value: p.p95_ms ?? 0,
        },
      ]),
    [timeseries, range, latencyLabels.p50, latencyLabels.p95],
  );
  const lastPoint =
    timeseries.length > 0 ? timeseries[timeseries.length - 1] : undefined;
  const latestP95 = lastPoint?.p95_ms ?? 0;
  const latestP50 = lastPoint?.p50_ms ?? 0;

  const errorsAreaData = useMemo(
    () =>
      timeseries.map((p) => ({
        time: formatTime(p.ts, range),
        errors: p.errors,
      })),
    [timeseries, range],
  );

  const heatmapData = useMemo(() => {
    const buckets = heatmapQuery.data?.buckets ?? [];
    return buckets.flatMap((b) =>
      b.values.map((v) => ({
        time: formatTime(b.ts, range),
        provider: v.provider,
        calls: v.calls,
      })),
    );
  }, [heatmapQuery.data, range]);

  const statusLabels = {
    ok: t("dashboard.charts.statusOk"),
    c4xx: t("dashboard.charts.status4xx"),
    c5xx: t("dashboard.charts.status5xx"),
  };
  const statusData = useMemo(
    () =>
      (statusQuery.data ?? []).flatMap((p) => [
        {
          time: formatTime(p.ts, range),
          metric: statusLabels.ok,
          value: p.ok_2xx,
        },
        {
          time: formatTime(p.ts, range),
          metric: statusLabels.c4xx,
          value: p.client_4xx,
        },
        {
          time: formatTime(p.ts, range),
          metric: statusLabels.c5xx,
          value: p.server_5xx,
        },
      ]),
    [
      statusQuery.data,
      range,
      statusLabels.ok,
      statusLabels.c4xx,
      statusLabels.c5xx,
    ],
  );

  const cacheTables = cacheTablesQuery.data ?? [];
  const totalCacheRows = cacheTables.reduce(
    (sum, tbl) => sum + tbl.row_count,
    0,
  );

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (!over || active.id === over.id) return;
    setOrder((prev) => {
      const oldIdx = prev.indexOf(active.id as ChartId);
      const newIdx = prev.indexOf(over.id as ChartId);
      if (oldIdx < 0 || newIdx < 0) return prev;
      return arrayMove(prev, oldIdx, newIdx);
    });
  };

  const renderChart = (
    id: ChartId,
    handleProps: Record<string, unknown>,
  ): React.ReactNode => {
    switch (id) {
      case "chart-timeseries":
        return (
          <ChartCard
            key={id}
            metric={t("dashboard.charts.volume")}
            value={formatNumber(totalCallsInRange)}
            hint={t("dashboard.charts.heroCalls")}
            dragHandleProps={handleProps}
          >
            {timeseriesQuery.isError ? (
              <ChartError
                error={timeseriesQuery.error}
                onRetry={() => timeseriesQuery.refetch()}
              />
            ) : timeseriesQuery.isLoading ? (
              <ChartFallback />
            ) : timeseries.length === 0 ? (
              <EmptyChart />
            ) : (
              <Suspense fallback={<ChartFallback />}>
                <Line
                  key={`ts-${themeKey}`}
                  {...timeseriesLineConfig(lineData, resolvedMode, lineLabels)}
                />
              </Suspense>
            )}
          </ChartCard>
        );

      case "chart-cache-ring": {
        const ratio = overview?.cache_hit_ratio_24h ?? 0;
        return (
          <ChartCard
            key={id}
            metric={t("dashboard.charts.cacheHit")}
            value={`${(ratio * 100).toFixed(1)}%`}
            hint={t("dashboard.subtitles.totalRows")}
            dragHandleProps={handleProps}
          >
            {overviewQuery.isLoading ? (
              <ChartFallback />
            ) : (
              <div className="flex flex-1 items-center justify-center">
                <div className="relative">
                  <ActivityRing value={ratio} size={180} stroke={18} />
                  <div className="absolute inset-0 flex flex-col items-center justify-center">
                    <span className="text-3xl font-semibold tracking-[-0.04em] text-fg-light tabular-nums dark:text-fg-dark">
                      {(ratio * 100).toFixed(0)}%
                    </span>
                    <span className="mt-0.5 text-[10px] font-semibold tracking-[0.08em] text-fg-muted-light uppercase dark:text-fg-muted-dark">
                      {t("dashboard.charts.cacheHit")}
                    </span>
                  </div>
                </div>
              </div>
            )}
          </ChartCard>
        );
      }

      case "chart-top-providers":
        return (
          <ChartCard
            key={id}
            metric={t("dashboard.charts.topProviders")}
            value={formatNumber(pieTotal)}
            hint={t("dashboard.charts.calls")}
            dragHandleProps={handleProps}
          >
            {providersQuery.isError ? (
              <ChartError
                error={providersQuery.error}
                onRetry={() => providersQuery.refetch()}
              />
            ) : providersQuery.isLoading ? (
              <ChartFallback />
            ) : pieData.length === 0 ? (
              <EmptyChart />
            ) : (
              <Suspense fallback={<ChartFallback />}>
                <Pie
                  key={`pie-${themeKey}`}
                  {...pieConfig(pieData, resolvedMode)}
                />
              </Suspense>
            )}
          </ChartCard>
        );

      case "chart-by-provider":
        return (
          <ChartCard
            key={id}
            metric={t("dashboard.charts.byProvider")}
            value={formatNumber(sortedProviders.length)}
            hint={t("dashboard.cards.providers")}
            dragHandleProps={handleProps}
          >
            {providersQuery.isError ? (
              <ChartError
                error={providersQuery.error}
                onRetry={() => providersQuery.refetch()}
              />
            ) : providersQuery.isLoading ? (
              <ChartFallback />
            ) : columnData.length === 0 ? (
              <EmptyChart />
            ) : (
              <Suspense fallback={<ChartFallback />}>
                <Column
                  key={`col-${themeKey}`}
                  {...providerColumnConfig(
                    columnData,
                    resolvedMode,
                    columnLabels,
                  )}
                />
              </Suspense>
            )}
          </ChartCard>
        );

      case "chart-latency":
        return (
          <ChartCard
            key={id}
            metric={t("dashboard.charts.latency")}
            value={`${Math.round(latestP95)}ms`}
            hint={`p50 ${Math.round(latestP50)}ms`}
            dragHandleProps={handleProps}
          >
            {timeseriesQuery.isError ? (
              <ChartError
                error={timeseriesQuery.error}
                onRetry={() => timeseriesQuery.refetch()}
              />
            ) : timeseriesQuery.isLoading ? (
              <ChartFallback />
            ) : latencyData.length === 0 ? (
              <EmptyChart />
            ) : (
              <Suspense fallback={<ChartFallback />}>
                <Line
                  key={`lat-${themeKey}`}
                  {...latencyLineConfig(
                    latencyData,
                    resolvedMode,
                    latencyLabels,
                  )}
                />
              </Suspense>
            )}
          </ChartCard>
        );

      case "chart-errors-area":
        return (
          <ChartCard
            key={id}
            metric={t("dashboard.charts.errorsArea")}
            value={formatNumber(totalErrorsInRange)}
            hint={t("dashboard.charts.errors")}
            dragHandleProps={handleProps}
          >
            {timeseriesQuery.isError ? (
              <ChartError
                error={timeseriesQuery.error}
                onRetry={() => timeseriesQuery.refetch()}
              />
            ) : timeseriesQuery.isLoading ? (
              <ChartFallback />
            ) : errorsAreaData.length === 0 ? (
              <EmptyChart />
            ) : (
              <Suspense fallback={<ChartFallback />}>
                <Area
                  key={`err-${themeKey}`}
                  {...errorsAreaConfig(errorsAreaData, resolvedMode)}
                />
              </Suspense>
            )}
          </ChartCard>
        );

      case "chart-heatmap":
        return (
          <ChartCard
            key={id}
            metric={t("dashboard.charts.heatmap")}
            value={formatNumber(heatmapQuery.data?.providers.length ?? 0)}
            hint={t("dashboard.cards.providers")}
            dragHandleProps={handleProps}
          >
            {heatmapQuery.isError ? (
              <ChartError
                error={heatmapQuery.error}
                onRetry={() => heatmapQuery.refetch()}
              />
            ) : heatmapQuery.isLoading ? (
              <ChartFallback />
            ) : heatmapData.length === 0 ? (
              <EmptyChart />
            ) : (
              <Suspense fallback={<ChartFallback />}>
                <Heatmap
                  key={`heat-${themeKey}`}
                  {...heatmapConfig(heatmapData, resolvedMode)}
                />
              </Suspense>
            )}
          </ChartCard>
        );

      case "chart-status-codes":
        return (
          <ChartCard
            key={id}
            metric={t("dashboard.charts.statusCodes")}
            value={formatNumber(
              (statusQuery.data ?? []).reduce(
                (sum, p) => sum + p.ok_2xx + p.client_4xx + p.server_5xx,
                0,
              ),
            )}
            hint={t("dashboard.charts.calls")}
            dragHandleProps={handleProps}
          >
            {statusQuery.isError ? (
              <ChartError
                error={statusQuery.error}
                onRetry={() => statusQuery.refetch()}
              />
            ) : statusQuery.isLoading ? (
              <ChartFallback />
            ) : statusData.length === 0 ? (
              <EmptyChart />
            ) : (
              <Suspense fallback={<ChartFallback />}>
                <Column
                  key={`status-${themeKey}`}
                  {...statusCodesConfig(statusData, resolvedMode, statusLabels)}
                />
              </Suspense>
            )}
          </ChartCard>
        );

      case "chart-cache-tables":
        return (
          <ChartCard
            key={id}
            metric={t("dashboard.charts.cacheTables")}
            value={formatNumber(totalCacheRows)}
            hint={t("dashboard.charts.rows")}
            dragHandleProps={handleProps}
            bodyClassName="max-h-[260px] overflow-y-auto"
          >
            {cacheTablesQuery.isError ? (
              <ChartError
                error={cacheTablesQuery.error}
                onRetry={() => cacheTablesQuery.refetch()}
              />
            ) : cacheTablesQuery.isLoading ? (
              <ChartFallback />
            ) : cacheTables.length === 0 ? (
              <EmptyChart />
            ) : (
              <ul className="flex flex-1 flex-col gap-1.5 pr-1">
                {cacheTables
                  .slice()
                  .sort((a, b) => b.row_count - a.row_count)
                  .map((tbl) => (
                    <li
                      key={tbl.name}
                      className="flex h-8 shrink-0 items-center justify-between rounded-lg px-2 text-xs hover:bg-fill-tertiary-light dark:hover:bg-fill-tertiary-dark"
                    >
                      <span className="truncate font-mono text-fg-light dark:text-fg-dark">
                        {tbl.name}
                      </span>
                      <span className="ml-3 flex shrink-0 items-baseline gap-2 text-fg-muted-light tabular-nums dark:text-fg-muted-dark">
                        <span className="font-semibold text-fg-light dark:text-fg-dark">
                          {formatNumber(tbl.row_count)}
                        </span>
                        <span className="text-[10px]">
                          {formatTtlShort(tbl.avg_ttl_remaining_seconds)}
                        </span>
                      </span>
                    </li>
                  ))}
              </ul>
            )}
          </ChartCard>
        );
    }
  };

  const recentErrorColumns: TableProps<DashboardRecentError>["columns"] = [
    {
      key: "ts",
      title: t("dashboard.columns.time"),
      render: (_, r) => formatRelative(r.ts, t),
    },
    {
      key: "provider",
      dataIndex: "provider",
      title: t("dashboard.columns.provider"),
      render: (v: string) => (
        <span className="font-semibold text-fg-light dark:text-fg-dark">
          {v}
        </span>
      ),
    },
    {
      key: "status",
      dataIndex: "status",
      title: t("dashboard.columns.status"),
    },
    {
      key: "duration_ms",
      dataIndex: "duration_ms",
      title: t("dashboard.columns.duration"),
      render: (v: number) => t("dashboard.units.ms", { value: v }),
    },
  ];

  const containerRef = useRef<HTMLDivElement>(null);
  useDocsRegister(
    useMemo(
      () => ({
        id: "dashboard",
        sections: [{ key: "overview" }, { key: "metrics" }, { key: "refresh" }],
        fields: [
          { key: "range", type: "enum" },
          { key: "interval", type: "i64" },
        ],
        anchorRef: containerRef,
      }),
      [],
    ),
  );

  return (
    <div
      ref={containerRef}
      className="mx-auto flex w-full max-w-[1280px] flex-col gap-6"
    >
      <div className="flex items-end justify-between gap-4">
        <Typography.Title
          level={2}
          className="m-0 text-2xl font-semibold tracking-[-0.03em] text-fg-light dark:text-fg-dark"
        >
          {t("dashboard.title")}
        </Typography.Title>
        <div className="flex items-center gap-3">
          <Segmented
            value={range}
            onChange={(v) => setRange(v as RangeKey)}
            options={[
              { label: t("dashboard.range.1h"), value: "1h" },
              { label: t("dashboard.range.24h"), value: "24h" },
              { label: t("dashboard.range.7d"), value: "7d" },
            ]}
          />
          <Segmented
            value={refreshInterval}
            onChange={(v) => setRefreshInterval(Number(v))}
            options={[
              { label: t("dashboard.refresh.off"), value: 0 },
              { label: "10s", value: 10 },
              { label: "30s", value: 30 },
              { label: "60s", value: 60 },
            ]}
          />
          <Tooltip title={t("dashboard.refresh.now")}>
            <Button
              icon={
                <RefreshCw
                  size={16}
                  className={spinning ? "animate-spin" : undefined}
                />
              }
              onClick={handleManualRefresh}
            />
          </Tooltip>
        </div>
      </div>

      {overviewQuery.isError ? (
        <ChartError
          error={overviewQuery.error}
          onRetry={() => overviewQuery.refetch()}
        />
      ) : null}

      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragEnd={handleDragEnd}
      >
        <SortableContext items={order} strategy={rectSortingStrategy}>
          <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
            {order.map((id) => (
              <SortableSlot key={id} id={id}>
                {(handle) => renderChart(id, handle)}
              </SortableSlot>
            ))}
          </div>
        </SortableContext>
      </DndContext>

      <div className="rounded-2xl border border-border-light bg-panel-light p-5 shadow-[0_1px_2px_0_rgba(0,0,0,0.04),0_1px_3px_0_rgba(0,0,0,0.06)] dark:border-border-dark dark:bg-panel-dark">
        <div className="mb-3 text-[11px] font-semibold tracking-[0.08em] text-fg-muted-light uppercase dark:text-fg-muted-dark">
          {t("dashboard.charts.recentErrors")}
        </div>
        {recentErrorsQuery.isError ? (
          <ChartError
            error={recentErrorsQuery.error}
            onRetry={() => recentErrorsQuery.refetch()}
          />
        ) : (
          <div className="max-h-[400px] overflow-y-auto">
            <Table
              columns={recentErrorColumns}
              dataSource={recentErrorsQuery.data ?? []}
              loading={recentErrorsQuery.isLoading}
              pagination={false}
              rowKey={(r) => `${r.ts}-${r.provider}-${r.status}`}
              size="small"
              className="[&_.ant-table]:!bg-transparent [&_.ant-table-cell]:!border-border-light dark:[&_.ant-table-cell]:!border-border-dark [&_.ant-table-thead>tr>th]:!bg-transparent [&_.ant-table-thead>tr>th]:!text-fg-muted-light dark:[&_.ant-table-thead>tr>th]:!text-fg-muted-dark"
            />
          </div>
        )}
      </div>
    </div>
  );
}

export default DashboardPage;
