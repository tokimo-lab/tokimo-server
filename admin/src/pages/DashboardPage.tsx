import {
  ApiOutlined,
  ArrowUpOutlined,
  DatabaseOutlined,
  KeyOutlined,
  ThunderboltOutlined,
} from "@ant-design/icons";
import { useQuery } from "@tanstack/react-query";
import {
  Alert,
  Button,
  Card,
  Col,
  Row,
  Segmented,
  Skeleton,
  Statistic,
  Table,
  type TableProps,
  Typography,
} from "antd";
import { Suspense, lazy, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  type DashboardRecentError,
  getDashboardByProvider,
  getDashboardOverview,
  getDashboardRecentErrors,
  getDashboardTimeseries,
} from "../api/client";
import { useAdminTheme } from "../theme";

const Line = lazy(() =>
  import("@ant-design/plots").then((module) => ({ default: module.Line })),
);
const Pie = lazy(() =>
  import("@ant-design/plots").then((module) => ({ default: module.Pie })),
);
const Column = lazy(() =>
  import("@ant-design/plots").then((module) => ({ default: module.Column })),
);

const RANGE_BUCKET: Record<string, string> = {
  "1h": "5m",
  "24h": "1h",
  "7d": "1d",
};
const LINE_GRADIENT = "l(0) 0:#3b82f6 0.5:#8b5cf6 1:#ec4899";
const PIE_COLORS = [
  "#8b5cf6",
  "#3b82f6",
  "#ec4899",
  "#06b6d4",
  "#10b981",
  "#f59e0b",
];
const RANGE_OPTIONS = [
  { labelKey: "dashboard.range.1h", value: "1h" },
  { labelKey: "dashboard.range.24h", value: "24h" },
  { labelKey: "dashboard.range.7d", value: "7d" },
] as const;

const tableWrapperClass =
  "overflow-hidden rounded-lg border border-border-light bg-panel-light dark:border-border-dark dark:bg-panel-dark [&_.ant-table]:!bg-transparent [&_.ant-table-cell]:!border-border-light dark:[&_.ant-table-cell]:!border-border-dark [&_.ant-table-thead>tr>th]:!bg-panel-light dark:[&_.ant-table-thead>tr>th]:!bg-panel-dark [&_.ant-table-thead>tr>th]:!text-fg-muted-light dark:[&_.ant-table-thead>tr>th]:!text-fg-muted-dark [&_.ant-table-tbody>tr:hover>td]:!bg-zinc-100 dark:[&_.ant-table-tbody>tr:hover>td]:!bg-[#18181c]";
const cardClass =
  "rounded-lg border border-border-light bg-panel-light shadow-none transition-shadow hover:shadow-sm dark:border-border-dark dark:bg-panel-dark";
const chartBodyClass = "min-h-[320px]";
const shortChartClass = "min-h-[260px]";

type ChartPalette = {
  axis: string;
  border: string;
  text: string;
  muted: string;
};

type PieDatum = {
  provider: string;
  calls: number;
  percent?: number;
};

function getChartPalette(resolvedMode: "light" | "dark"): ChartPalette {
  return resolvedMode === "dark"
    ? {
        axis: "#9a9aa3",
        border: "#1f1f23",
        muted: "#9a9aa3",
        text: "#ededed",
      }
    : {
        axis: "#5e5e66",
        border: "#e5e5e7",
        muted: "#5e5e66",
        text: "#1a1a1a",
      };
}

function toDate(ts: number) {
  return new Date(ts < 1_000_000_000_000 ? ts * 1000 : ts);
}

function formatTime(ts: number) {
  return toDate(ts).toLocaleString(undefined, {
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    month: "2-digit",
  });
}

function formatRelativeTime(
  ts: number,
  t: ReturnType<typeof useTranslation>["t"],
) {
  const diffMs = Math.max(Date.now() - toDate(ts).getTime(), 0);
  const minute = 60_000;
  const hour = 60 * minute;
  const day = 24 * hour;

  if (diffMs < minute) return t("dashboard.relative.justNow");
  if (diffMs < hour) {
    return t("dashboard.relative.minutesAgo", {
      count: Math.floor(diffMs / minute),
    });
  }
  if (diffMs < day) {
    return t("dashboard.relative.hoursAgo", {
      count: Math.floor(diffMs / hour),
    });
  }
  return t("dashboard.relative.daysAgo", { count: Math.floor(diffMs / day) });
}

function formatNumber(value: number) {
  return new Intl.NumberFormat().format(value);
}

function ErrorState({
  error,
  onRetry,
}: { error: unknown; onRetry: () => void }) {
  const { t } = useTranslation();

  return (
    <Alert
      type="error"
      showIcon
      message={t("common.error")}
      description={error instanceof Error ? error.message : String(error)}
      action={<Button onClick={onRetry}>{t("dashboard.retry")}</Button>}
    />
  );
}

function EmptyDashboard() {
  const { t } = useTranslation();

  return (
    <div className="flex min-h-[180px] flex-col items-center justify-center gap-3 text-center">
      <div className="gradient-bg h-12 w-[72px] rounded-lg opacity-20" />
      <Typography.Text className="text-xs text-fg-muted-light dark:text-fg-muted-dark">
        {t("dashboard.empty")}
      </Typography.Text>
    </div>
  );
}

function DashboardPage() {
  const { t } = useTranslation();
  const { resolvedMode } = useAdminTheme();
  const [range, setRange] = useState("24h");
  const chartPalette = getChartPalette(resolvedMode);
  const chartKey = `${resolvedMode}-${chartPalette.axis}-${chartPalette.border}`;
  const bucket = RANGE_BUCKET[range];

  const overviewQuery = useQuery({
    queryKey: ["dashboard", "overview"],
    queryFn: getDashboardOverview,
  });
  const timeseriesQuery = useQuery({
    queryKey: ["dashboard", "timeseries", range, bucket],
    queryFn: () => getDashboardTimeseries(range, bucket),
  });
  const providersQuery = useQuery({
    queryKey: ["dashboard", "providers", "24h"],
    queryFn: () => getDashboardByProvider("24h"),
  });
  const recentErrorsQuery = useQuery({
    queryKey: ["dashboard", "recent-errors", 10],
    queryFn: () => getDashboardRecentErrors(10),
  });

  const overview = overviewQuery.data;
  const errorRate = overview?.calls_24h
    ? overview.errors_24h / overview.calls_24h
    : 0;

  const lineMetricLabels = useMemo(
    () => ({
      calls: t("dashboard.charts.calls"),
      errors: t("dashboard.charts.errors"),
      hits: t("dashboard.charts.cacheHits"),
      misses: t("dashboard.charts.cacheMisses", {
        defaultValue: "Cache misses",
      }),
    }),
    [t],
  );

  const lineData = useMemo(
    () =>
      (timeseriesQuery.data ?? []).flatMap((point) => [
        {
          metric: lineMetricLabels.calls,
          time: formatTime(point.ts),
          value: point.calls,
        },
        {
          metric: lineMetricLabels.errors,
          time: formatTime(point.ts),
          value: point.errors,
        },
        {
          metric: lineMetricLabels.hits,
          time: formatTime(point.ts),
          value: point.hits,
        },
        {
          metric: lineMetricLabels.misses,
          time: formatTime(point.ts),
          value: point.misses,
        },
      ]),
    [lineMetricLabels, timeseriesQuery.data],
  );

  const sortedProviders = useMemo(
    () => [...(providersQuery.data ?? [])].sort((a, b) => b.calls - a.calls),
    [providersQuery.data],
  );

  const pieData = useMemo(() => {
    const top = sortedProviders.slice(0, 8);
    const otherCalls = sortedProviders
      .slice(8)
      .reduce((total, item) => total + item.calls, 0);
    return otherCalls > 0
      ? [...top, { provider: t("dashboard.charts.other"), calls: otherCalls }]
      : top;
  }, [sortedProviders, t]);

  const columnData = useMemo(
    () => sortedProviders.slice(0, 10),
    [sortedProviders],
  );

  const lineConfig = {
    axis: {
      x: { labelFill: chartPalette.axis, lineStroke: chartPalette.border },
      y: { gridStroke: chartPalette.border, labelFill: chartPalette.axis },
    },
    background: "transparent",
    colorField: "metric",
    data: lineData,
    height: 300,
    legend: { color: { itemLabelFill: chartPalette.text } },
    point: { size: 0 },
    scale: {
      color: {
        domain: [
          lineMetricLabels.calls,
          lineMetricLabels.errors,
          lineMetricLabels.hits,
          lineMetricLabels.misses,
        ],
        range: [LINE_GRADIENT, "#f43f5e", "#10b981", "#9a9aa3"],
      },
    },
    seriesField: "metric",
    style: { lineWidth: 2 },
    theme: { colors: ["#8b5cf6", "#f43f5e", "#10b981", "#9a9aa3"] },
    xField: "time",
    yField: "value",
  };

  const pieConfig = {
    angleField: "calls",
    background: "transparent",
    colorField: "provider",
    data: pieData,
    height: 260,
    innerRadius: 0.5,
    label: {
      fill: chartPalette.muted,
      fontSize: 12,
      text: (datum: PieDatum) => {
        const percent = datum.percent ?? 0;
        return `${datum.provider} ${new Intl.NumberFormat(undefined, {
          maximumFractionDigits: 0,
          style: "percent",
        }).format(percent)}`;
      },
    },
    legend: { color: { itemLabelFill: chartPalette.text, position: "right" } },
    scale: { color: { range: PIE_COLORS } },
    state: { active: { scale: 1.04 } },
    theme: { colors: PIE_COLORS },
  };

  const columnConfig = {
    axis: {
      x: { labelFill: chartPalette.axis, lineStroke: chartPalette.border },
      y: { gridStroke: chartPalette.border, labelFill: chartPalette.axis },
    },
    background: "transparent",
    color: "#8b5cf6",
    data: columnData,
    height: 260,
    legend: false,
    style: {
      fill: "#8b5cf6",
      radiusTopLeft: 4,
      radiusTopRight: 4,
    },
    theme: { colors: ["#8b5cf6"] },
    xField: "provider",
    yField: "calls",
  };

  const columns: TableProps<DashboardRecentError>["columns"] = [
    {
      key: "ts",
      render: (_, record) => formatRelativeTime(record.ts, t),
      title: t("dashboard.columns.time"),
    },
    {
      dataIndex: "provider",
      key: "provider",
      render: (value: DashboardRecentError["provider"]) => (
        <span className="font-semibold text-fg-light dark:text-fg-dark">
          {value}
        </span>
      ),
      title: t("dashboard.columns.provider"),
    },
    {
      dataIndex: "status",
      key: "status",
      title: t("dashboard.columns.status"),
    },
    {
      dataIndex: "duration_ms",
      key: "duration_ms",
      render: (value: DashboardRecentError["duration_ms"]) =>
        t("dashboard.units.ms", { value }),
      title: t("dashboard.columns.duration"),
    },
  ];

  const statCards = [
    {
      icon: <KeyOutlined />,
      key: "keys",
      subtitle: t("dashboard.subtitles.active"),
      title: t("dashboard.cards.keys"),
      value: overview?.total_keys ?? 0,
    },
    {
      icon: <ApiOutlined />,
      key: "providers",
      subtitle: t("dashboard.subtitles.configured"),
      title: t("dashboard.cards.providers"),
      value: overview?.total_providers ?? 0,
    },
    {
      icon: <DatabaseOutlined />,
      key: "cache",
      subtitle: t("dashboard.subtitles.totalRows"),
      title: t("dashboard.cards.cacheEntries"),
      value: overview?.cache_entries_total ?? 0,
    },
    {
      icon: <ThunderboltOutlined />,
      key: "calls",
      subtitle: (
        <Typography.Text type={errorRate > 0.01 ? "danger" : "secondary"}>
          {t("dashboard.cards.errorRate", {
            calls: formatNumber(overview?.calls_24h ?? 0),
            errors: formatNumber(overview?.errors_24h ?? 0),
            rate: new Intl.NumberFormat(undefined, {
              maximumFractionDigits: 2,
              style: "percent",
            }).format(errorRate),
          })}
        </Typography.Text>
      ),
      title: t("dashboard.cards.calls24h"),
      value: overview?.calls_24h ?? 0,
    },
  ];

  return (
    <div className="mx-auto flex w-full max-w-[1200px] flex-col gap-6">
      <Typography.Title
        level={2}
        className="m-0 text-2xl tracking-[-0.03em] text-fg-light dark:text-fg-dark"
      >
        {t("dashboard.title")}
      </Typography.Title>

      {overviewQuery.isError ? (
        <ErrorState
          error={overviewQuery.error}
          onRetry={() => overviewQuery.refetch()}
        />
      ) : (
        <Row gutter={[24, 24]}>
          {statCards.map((card) => (
            <Col xs={24} sm={12} xl={6} key={card.key}>
              <Card className={cardClass} classNames={{ body: "p-5" }}>
                {overviewQuery.isLoading ? (
                  <Skeleton active paragraph={false} />
                ) : (
                  <div className="flex items-start gap-4">
                    <span className="inline-flex h-9 w-9 flex-none items-center justify-center rounded-input border border-border-light bg-zinc-100 text-fg-muted-light dark:border-border-dark dark:bg-[#18181c] dark:text-fg-muted-dark">
                      {card.icon}
                    </span>
                    <div className="min-w-0">
                      <Statistic
                        title={
                          <span className="inline-flex items-center gap-1 text-xs font-semibold tracking-[0.08em] text-fg-muted-light uppercase dark:text-fg-muted-dark">
                            <ArrowUpOutlined className="text-[10px] text-brand-500" />
                            {card.title}
                          </span>
                        }
                        value={card.value}
                        formatter={(value) => (
                          <span className="gradient-text text-[28px] leading-none font-semibold tracking-[-0.04em]">
                            {formatNumber(Number(value))}
                          </span>
                        )}
                      />
                      <div className="mt-2 text-xs text-fg-muted-light dark:text-fg-muted-dark">
                        {card.subtitle}
                      </div>
                    </div>
                  </div>
                )}
              </Card>
            </Col>
          ))}
        </Row>
      )}

      {overview?.calls_24h === 0 ? <EmptyDashboard /> : null}

      <Row gutter={[24, 24]}>
        <Col xs={24} xl={16}>
          <Card
            className={cardClass}
            classNames={{
              body: "bg-panel-light dark:bg-panel-dark",
              header:
                "min-h-12 border-border-light text-sm font-semibold dark:border-border-dark",
            }}
            title={
              <div className="flex items-center justify-between gap-3">
                <span>{t("dashboard.charts.volume")}</span>
                <Segmented
                  value={range}
                  onChange={(value) => setRange(String(value))}
                  options={RANGE_OPTIONS.map((item) => ({
                    label: t(item.labelKey),
                    value: item.value,
                  }))}
                />
              </div>
            }
          >
            <div className={chartBodyClass}>
              {timeseriesQuery.isError ? (
                <ErrorState
                  error={timeseriesQuery.error}
                  onRetry={() => timeseriesQuery.refetch()}
                />
              ) : timeseriesQuery.isLoading ? (
                <Skeleton active />
              ) : lineData.length === 0 ? (
                <EmptyDashboard />
              ) : (
                <Suspense fallback={<Skeleton active />}>
                  <Line key={`line-${chartKey}`} {...lineConfig} />
                </Suspense>
              )}
            </div>
          </Card>
        </Col>
        <Col xs={24} xl={8}>
          <Card
            className={cardClass}
            classNames={{
              body: "bg-panel-light dark:bg-panel-dark",
              header:
                "min-h-12 border-border-light text-sm font-semibold dark:border-border-dark",
            }}
            title={t("dashboard.charts.topProviders")}
          >
            <div className={shortChartClass}>
              {providersQuery.isError ? (
                <ErrorState
                  error={providersQuery.error}
                  onRetry={() => providersQuery.refetch()}
                />
              ) : providersQuery.isLoading ? (
                <Skeleton active />
              ) : pieData.length === 0 ? (
                <EmptyDashboard />
              ) : (
                <Suspense fallback={<Skeleton active />}>
                  <Pie key={`pie-${chartKey}`} {...pieConfig} />
                </Suspense>
              )}
            </div>
          </Card>
        </Col>
      </Row>

      <Row gutter={[24, 24]}>
        <Col xs={24} xl={14}>
          <Card
            className={cardClass}
            classNames={{
              body: "bg-panel-light dark:bg-panel-dark",
              header:
                "min-h-12 border-border-light text-sm font-semibold dark:border-border-dark",
            }}
            title={t("dashboard.charts.byProvider")}
          >
            <div className={shortChartClass}>
              {providersQuery.isError ? (
                <ErrorState
                  error={providersQuery.error}
                  onRetry={() => providersQuery.refetch()}
                />
              ) : providersQuery.isLoading ? (
                <Skeleton active />
              ) : columnData.length === 0 ? (
                <EmptyDashboard />
              ) : (
                <Suspense fallback={<Skeleton active />}>
                  <Column key={`column-${chartKey}`} {...columnConfig} />
                </Suspense>
              )}
            </div>
          </Card>
        </Col>
        <Col xs={24} xl={10}>
          <Card
            className={cardClass}
            classNames={{
              body: "bg-panel-light dark:bg-panel-dark p-0",
              header:
                "min-h-12 border-border-light text-sm font-semibold dark:border-border-dark",
            }}
            title={t("dashboard.charts.recentErrors")}
          >
            {recentErrorsQuery.isError ? (
              <div className="p-6">
                <ErrorState
                  error={recentErrorsQuery.error}
                  onRetry={() => recentErrorsQuery.refetch()}
                />
              </div>
            ) : (
              <div className={tableWrapperClass}>
                <Table
                  columns={columns}
                  dataSource={recentErrorsQuery.data ?? []}
                  loading={recentErrorsQuery.isLoading}
                  pagination={false}
                  rowKey={(record) =>
                    `${record.ts}-${record.provider}-${record.status}`
                  }
                  size="small"
                />
              </div>
            )}
          </Card>
        </Col>
      </Row>
    </div>
  );
}
export default DashboardPage;
