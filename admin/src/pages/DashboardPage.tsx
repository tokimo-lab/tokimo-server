import {
  ApiOutlined,
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
import { Suspense, lazy, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  type DashboardRecentError,
  getDashboardByProvider,
  getDashboardOverview,
  getDashboardRecentErrors,
  getDashboardTimeseries,
} from "../api/client";
import { useAdminTheme } from "../theme";
import "./DashboardPage.css";

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
const CHART_COLORS = ["#FF8A3D", "#FF5CA1", "#8B5CF6"];
const RANGE_OPTIONS = [
  { labelKey: "dashboard.range.1h", value: "1h" },
  { labelKey: "dashboard.range.24h", value: "24h" },
  { labelKey: "dashboard.range.7d", value: "7d" },
] as const;

type ChartTokenColors = {
  axis: string;
  border: string;
  text: string;
};
const INITIAL_CHART_TOKEN_COLORS: ChartTokenColors = {
  axis: "",
  border: "",
  text: "",
};

function getChartTokenColors(): ChartTokenColors {
  const styles = getComputedStyle(document.documentElement);
  return {
    axis: styles.getPropertyValue("--tks-fg-secondary").trim(),
    border: styles.getPropertyValue("--tks-border-subtle").trim(),
    text: styles.getPropertyValue("--tks-fg-primary").trim(),
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
    <div className="tks-dashboard-empty">
      <div className="tks-dashboard-empty-blob" />
      <Typography.Text className="tks-dashboard-empty-text">
        {t("dashboard.empty")}
      </Typography.Text>
    </div>
  );
}

function DashboardPage() {
  const { t } = useTranslation();
  const { mode } = useAdminTheme();
  const [range, setRange] = useState("24h");
  const [chartTokens, setChartTokens] = useState(INITIAL_CHART_TOKEN_COLORS);

  useEffect(() => {
    let frameId = 0;

    const updateChartTokens = () => {
      if (document.documentElement.dataset.theme !== mode) {
        frameId = window.requestAnimationFrame(updateChartTokens);
        return;
      }

      setChartTokens(getChartTokenColors());
    };

    frameId = window.requestAnimationFrame(updateChartTokens);

    return () => window.cancelAnimationFrame(frameId);
  }, [mode]);

  const chartKey = `${mode}-${chartTokens.axis}-${chartTokens.border}-${chartTokens.text}`;
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

  const lineData = useMemo(
    () =>
      (timeseriesQuery.data ?? []).flatMap((point) => [
        {
          metric: t("dashboard.charts.calls"),
          time: formatTime(point.ts),
          value: point.calls,
        },
        {
          metric: t("dashboard.charts.errors"),
          time: formatTime(point.ts),
          value: point.errors,
        },
        {
          metric: t("dashboard.charts.cacheHits"),
          time: formatTime(point.ts),
          value: point.hits,
        },
      ]),
    [timeseriesQuery.data, t],
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
      x: { labelFill: chartTokens.axis, lineStroke: chartTokens.border },
      y: { gridStroke: chartTokens.border, labelFill: chartTokens.axis },
    },
    colorField: "metric",
    data: lineData,
    height: 300,
    legend: { color: { itemLabelFill: chartTokens.text } },
    scale: { color: { range: CHART_COLORS } },
    seriesField: "metric",
    xField: "time",
    yField: "value",
  };

  const pieConfig = {
    angleField: "calls",
    colorField: "provider",
    data: pieData,
    height: 260,
    label: { text: "provider", fill: chartTokens.axis },
    legend: { color: { itemLabelFill: chartTokens.text } },
    scale: { color: { range: CHART_COLORS } },
  };

  const columnConfig = {
    axis: {
      x: { labelFill: chartTokens.axis, lineStroke: chartTokens.border },
      y: { gridStroke: chartTokens.border, labelFill: chartTokens.axis },
    },
    colorField: "provider",
    data: columnData,
    height: 260,
    legend: false,
    scale: { color: { range: CHART_COLORS } },
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
        <span className="tks-dashboard-provider-cell">{value}</span>
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
    <div className="tks-dashboard-page">
      <Typography.Title level={2} className="tks-dashboard-title">
        {t("dashboard.title")}
      </Typography.Title>

      {overviewQuery.isError ? (
        <ErrorState
          error={overviewQuery.error}
          onRetry={() => overviewQuery.refetch()}
        />
      ) : (
        <Row gutter={[24, 24]} className="tks-dashboard-stat-row">
          {statCards.map((card) => (
            <Col xs={24} sm={12} xl={6} key={card.key}>
              <Card className="tks-dashboard-card tks-dashboard-stat-card">
                {overviewQuery.isLoading ? (
                  <Skeleton active paragraph={false} />
                ) : (
                  <>
                    <span className="tks-dashboard-stat-icon">{card.icon}</span>
                    <div className="tks-dashboard-stat-content">
                      <Statistic
                        title={card.title}
                        value={card.value}
                        formatter={(value) => formatNumber(Number(value))}
                      />
                      <div className="tks-dashboard-stat-subtitle">
                        {card.subtitle}
                      </div>
                    </div>
                  </>
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
            className="tks-dashboard-card tks-dashboard-chart-card"
            title={
              <div className="tks-dashboard-card-title">
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
            <div className="tks-dashboard-chart-body">
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
            className="tks-dashboard-card tks-dashboard-chart-card"
            title={t("dashboard.charts.topProviders")}
          >
            <div className="tks-dashboard-short-chart">
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
            className="tks-dashboard-card tks-dashboard-chart-card"
            title={t("dashboard.charts.byProvider")}
          >
            <div className="tks-dashboard-short-chart">
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
            className="tks-dashboard-card tks-dashboard-chart-card"
            title={t("dashboard.charts.recentErrors")}
          >
            {recentErrorsQuery.isError ? (
              <ErrorState
                error={recentErrorsQuery.error}
                onRetry={() => recentErrorsQuery.refetch()}
              />
            ) : (
              <Table
                className="tks-dashboard-table"
                columns={columns}
                dataSource={recentErrorsQuery.data ?? []}
                loading={recentErrorsQuery.isLoading}
                pagination={false}
                rowKey={(record) =>
                  `${record.ts}-${record.provider}-${record.status}`
                }
                size="small"
              />
            )}
          </Card>
        </Col>
      </Row>
    </div>
  );
}
export default DashboardPage;
