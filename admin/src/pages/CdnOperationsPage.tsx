import { Button, Card, Modal, Statistic, Table, Tag, message } from "antd";
import type { TableProps } from "antd";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  type CdnOverview,
  type CdnTableStatus,
  type CleanupRunStats,
  type TableCleanupResult,
  getCdnOverview,
  getCdnTables,
  getLastCleanup,
  runCdnCleanup,
} from "../api/cdn";

function formatDateTime(value: string | null): string {
  if (!value) return "-";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  const pad = (part: number) => String(part).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(
    date.getDate(),
  )} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(
    date.getSeconds(),
  )}`;
}

function getErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function CdnOperationsPage() {
  const { t } = useTranslation();
  const [overview, setOverview] = useState<CdnOverview | null>(null);
  const [tables, setTables] = useState<CdnTableStatus[]>([]);
  const [lastCleanup, setLastCleanup] = useState<CleanupRunStats | null>(null);
  const [overviewLoading, setOverviewLoading] = useState(false);
  const [tablesLoading, setTablesLoading] = useState(false);
  const [cleanupRunning, setCleanupRunning] = useState(false);
  const [cleanupResult, setCleanupResult] = useState<CleanupRunStats | null>(
    null,
  );
  const [resultModalOpen, setResultModalOpen] = useState(false);

  const loadOverview = useCallback(async () => {
    setOverviewLoading(true);
    try {
      const data = await getCdnOverview();
      setOverview(data);
    } catch (error) {
      message.error(getErrorMessage(error));
    } finally {
      setOverviewLoading(false);
    }
  }, []);

  const loadTables = useCallback(async () => {
    setTablesLoading(true);
    try {
      const data = await getCdnTables();
      setTables(data);
    } catch (error) {
      message.error(getErrorMessage(error));
    } finally {
      setTablesLoading(false);
    }
  }, []);

  const loadLastCleanup = useCallback(async () => {
    try {
      const data = await getLastCleanup();
      setLastCleanup(data.last_run);
    } catch (error) {
      message.error(getErrorMessage(error));
    }
  }, []);

  useEffect(() => {
    void loadOverview();
    void loadTables();
    void loadLastCleanup();

    const interval = setInterval(() => {
      void loadLastCleanup();
    }, 30000);

    return () => clearInterval(interval);
  }, [loadOverview, loadTables, loadLastCleanup]);

  const handleRunCleanup = () => {
    Modal.confirm({
      title: t("cdn.cleanup.confirmTitle"),
      content: t("cdn.cleanup.confirmContent"),
      okText: t("common.yes"),
      cancelText: t("common.cancel"),
      onOk: async () => {
        setCleanupRunning(true);
        try {
          const result = await runCdnCleanup();
          setCleanupResult(result);
          setResultModalOpen(true);
          message.success(t("cdn.cleanup.success"));
          void loadTables();
          void loadLastCleanup();
        } catch (error) {
          message.error(getErrorMessage(error));
        } finally {
          setCleanupRunning(false);
        }
      },
    });
  };

  const tableColumns: TableProps<CdnTableStatus>["columns"] = [
    {
      title: t("cdn.tables.columns.table"),
      dataIndex: "table",
      key: "table",
      fixed: "left",
      width: 200,
    },
    {
      title: t("cdn.tables.columns.tier"),
      dataIndex: "tier",
      key: "tier",
      width: 100,
      render: (tier: string) => {
        const colors: Record<string, string> = {
          volatile: "red",
          short: "orange",
          medium: "blue",
          permanent: "green",
          ttl: "purple",
        };
        return <Tag color={colors[tier] || "default"}>{tier}</Tag>;
      },
    },
    {
      title: t("cdn.tables.columns.rows"),
      dataIndex: "row_count",
      key: "row_count",
      width: 120,
      render: (count: number) => count.toLocaleString(),
    },
    {
      title: t("cdn.tables.columns.oldest"),
      dataIndex: "oldest_fetched_at",
      key: "oldest_fetched_at",
      width: 180,
      render: formatDateTime,
    },
    {
      title: t("cdn.tables.columns.newest"),
      dataIndex: "newest_fetched_at",
      key: "newest_fetched_at",
      width: 180,
      render: formatDateTime,
    },
    {
      title: t("cdn.tables.columns.retention"),
      dataIndex: "retention_secs",
      key: "retention_secs",
      width: 120,
      render: (secs: number | null) => {
        if (secs === null) return "-";
        const days = Math.floor(secs / 86400);
        return `${days}d`;
      },
    },
  ];

  const resultColumns: TableProps<TableCleanupResult>["columns"] = [
    {
      title: t("cdn.cleanup.result.table"),
      dataIndex: "table",
      key: "table",
    },
    {
      title: t("cdn.cleanup.result.tier"),
      dataIndex: "tier",
      key: "tier",
    },
    {
      title: t("cdn.cleanup.result.deleted"),
      dataIndex: "rows_deleted",
      key: "rows_deleted",
      render: (count: number) => count.toLocaleString(),
    },
    {
      title: t("cdn.cleanup.result.duration"),
      dataIndex: "duration_ms",
      key: "duration_ms",
      render: (ms: number) => `${ms}ms`,
    },
    {
      title: t("cdn.cleanup.result.status"),
      dataIndex: "error",
      key: "error",
      render: (error: string | null) =>
        error ? (
          <Tag color="red">{error}</Tag>
        ) : (
          <Tag color="green">{t("cdn.cleanup.result.ok")}</Tag>
        ),
    },
  ];

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-1 mb-2">
        <h1 className="text-2xl font-semibold">{t("cdn.title")}</h1>
        <p className="text-sm text-fg-muted-light dark:text-fg-muted-dark">
          {t("cdn.description")}
        </p>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
        <Card loading={overviewLoading}>
          <Statistic
            title={t("cdn.stats.lz4Columns")}
            value={overview?.compression.lz4 ?? 0}
            suffix={`/ ${overview?.compression.total_jsonb_columns ?? 0}`}
          />
        </Card>
        <Card loading={overviewLoading}>
          <Statistic
            title={t("cdn.stats.fetchedAtIndexes")}
            value={overview?.indexes.fetched_at_indexes ?? 0}
          />
        </Card>
        <Card loading={overviewLoading}>
          <Statistic
            title={t("cdn.stats.lastCleanup")}
            value={
              lastCleanup?.finished_at
                ? formatDateTime(lastCleanup.finished_at)
                : t("cdn.stats.noRun")
            }
            valueStyle={{ fontSize: "16px" }}
          />
          {lastCleanup && (
            <div className="mt-2 text-sm text-fg-muted-light dark:text-fg-muted-dark">
              {t("cdn.stats.rowsDeleted", {
                count: lastCleanup.total_rows_deleted,
              })}
            </div>
          )}
        </Card>
      </div>

      <Card
        title={t("cdn.tables.title")}
        extra={
          <Button
            type="primary"
            loading={cleanupRunning}
            onClick={handleRunCleanup}
          >
            {t("cdn.cleanup.runButton")}
          </Button>
        }
      >
        <Table
          columns={tableColumns}
          dataSource={tables}
          loading={tablesLoading}
          rowKey="table"
          pagination={false}
          scroll={{ x: 1000 }}
          size="small"
        />
      </Card>

      <Card title={t("cdn.lastRun.title")}>
        {lastCleanup ? (
          <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
            <div>
              <div className="text-xs text-fg-muted-light dark:text-fg-muted-dark">
                {t("cdn.lastRun.started")}
              </div>
              <div className="font-medium">
                {formatDateTime(lastCleanup.started_at)}
              </div>
            </div>
            <div>
              <div className="text-xs text-fg-muted-light dark:text-fg-muted-dark">
                {t("cdn.lastRun.finished")}
              </div>
              <div className="font-medium">
                {formatDateTime(lastCleanup.finished_at)}
              </div>
            </div>
            <div>
              <div className="text-xs text-fg-muted-light dark:text-fg-muted-dark">
                {t("cdn.lastRun.totalDeleted")}
              </div>
              <div className="font-medium">
                {lastCleanup.total_rows_deleted.toLocaleString()}
              </div>
            </div>
            <div>
              <div className="text-xs text-fg-muted-light dark:text-fg-muted-dark">
                {t("cdn.lastRun.status")}
              </div>
              <div className="font-medium">
                {lastCleanup.error ? (
                  <Tag color="red">{lastCleanup.error}</Tag>
                ) : (
                  <Tag color="green">{t("cdn.lastRun.ok")}</Tag>
                )}
              </div>
            </div>
          </div>
        ) : (
          <div className="text-sm text-fg-muted-light dark:text-fg-muted-dark">
            {t("cdn.stats.noRun")}
          </div>
        )}
      </Card>

      <Modal
        title={t("cdn.cleanup.resultTitle")}
        open={resultModalOpen}
        onCancel={() => setResultModalOpen(false)}
        footer={[
          <Button key="close" onClick={() => setResultModalOpen(false)}>
            {t("common.close")}
          </Button>,
        ]}
        width={800}
      >
        {cleanupResult && (
          <div className="flex flex-col gap-4">
            <div className="grid grid-cols-3 gap-4">
              <div>
                <div className="text-xs text-fg-muted-light dark:text-fg-muted-dark">
                  {t("cdn.cleanup.result.started")}
                </div>
                <div>{formatDateTime(cleanupResult.started_at)}</div>
              </div>
              <div>
                <div className="text-xs text-fg-muted-light dark:text-fg-muted-dark">
                  {t("cdn.cleanup.result.finished")}
                </div>
                <div>{formatDateTime(cleanupResult.finished_at)}</div>
              </div>
              <div>
                <div className="text-xs text-fg-muted-light dark:text-fg-muted-dark">
                  {t("cdn.cleanup.result.totalDeleted")}
                </div>
                <div>{cleanupResult.total_rows_deleted.toLocaleString()}</div>
              </div>
            </div>
            <Table
              columns={resultColumns}
              dataSource={cleanupResult.per_table}
              rowKey="table"
              pagination={false}
              size="small"
            />
          </div>
        )}
      </Modal>
    </div>
  );
}

export default CdnOperationsPage;
