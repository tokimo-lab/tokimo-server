import {
  Button,
  Input,
  Modal,
  Popconfirm,
  Select,
  Table,
  type TableProps,
  Tooltip,
  message,
} from "antd";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  type CacheRow,
  type CacheTable,
  deleteCacheRow,
  listCacheRows,
  listCacheTables,
  refreshCacheRow,
} from "../api/cache";

const PAGE_SIZE = 50;

function formatDateTime(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  const pad = (part: number) => String(part).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(
    date.getDate(),
  )} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(
    date.getSeconds(),
  )}`;
}

function formatRelative(
  value: string,
  t: ReturnType<typeof useTranslation>["t"],
) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "";

  const diffMs = Math.max(Date.now() - date.getTime(), 0);
  const minute = 60_000;
  const hour = 60 * minute;
  const day = 24 * hour;

  if (diffMs < minute) return t("cache.relative.justNow");
  if (diffMs < hour) {
    return t("cache.relative.minutesAgo", {
      count: Math.floor(diffMs / minute),
    });
  }
  if (diffMs < day) {
    return t("cache.relative.hoursAgo", { count: Math.floor(diffMs / hour) });
  }
  return t("cache.relative.daysAgo", { count: Math.floor(diffMs / day) });
}

function formatTtl(
  seconds: number | null,
  t: ReturnType<typeof useTranslation>["t"],
) {
  if (seconds === null) return t("cache.ttl.empty");
  if (seconds <= 0) return t("cache.ttl.expired");

  const days = Math.floor(seconds / 86_400);
  const hours = Math.floor((seconds % 86_400) / 3_600);
  const minutes = Math.floor((seconds % 3_600) / 60);
  const remainingSeconds = seconds % 60;
  const parts: string[] = [];

  if (days > 0) parts.push(t("cache.ttl.days", { count: days }));
  if (hours > 0) parts.push(t("cache.ttl.hours", { count: hours }));
  if (minutes > 0) parts.push(t("cache.ttl.minutes", { count: minutes }));
  if (remainingSeconds > 0 || parts.length === 0) {
    parts.push(t("cache.ttl.seconds", { count: remainingSeconds }));
  }

  return t("cache.ttl.average", { value: parts.slice(0, 2).join(" ") });
}

function getErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function CacheInspectorPage() {
  const { t } = useTranslation();
  const [tables, setTables] = useState<CacheTable[]>([]);
  const [selectedTableName, setSelectedTableName] = useState<string>();
  const [rows, setRows] = useState<CacheRow[]>([]);
  const [page, setPage] = useState(1);
  const [search, setSearch] = useState("");
  const [tablesLoading, setTablesLoading] = useState(false);
  const [rowsLoading, setRowsLoading] = useState(false);
  const [previewRow, setPreviewRow] = useState<CacheRow | null>(null);

  const selectedTable = tables.find(
    (table) => table.name === selectedTableName,
  );

  const loadTables = useCallback(async () => {
    setTablesLoading(true);
    try {
      const nextTables = await listCacheTables();
      setTables(nextTables);
      setSelectedTableName((current) => current ?? nextTables[0]?.name);
    } catch (error) {
      message.error(getErrorMessage(error));
    } finally {
      setTablesLoading(false);
    }
  }, []);

  const loadRows = useCallback(async (table: string, nextPage: number) => {
    setRowsLoading(true);
    try {
      const offset = (nextPage - 1) * PAGE_SIZE;
      const response = await listCacheRows(table, PAGE_SIZE, offset);
      setRows(response.rows);
    } catch (error) {
      message.error(getErrorMessage(error));
    } finally {
      setRowsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadTables();
  }, [loadTables]);

  useEffect(() => {
    if (!selectedTableName) return;
    void loadRows(selectedTableName, page);
  }, [loadRows, page, selectedTableName]);

  const filteredRows = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    if (!keyword) return rows;
    return rows.filter((row) =>
      [row.id, row.key, row.raw_preview ?? ""].some((value) =>
        value.toLowerCase().includes(keyword),
      ),
    );
  }, [rows, search]);

  const handleRefresh = async () => {
    await loadTables();
    if (selectedTableName) await loadRows(selectedTableName, page);
  };

  const handleExpire = async (row: CacheRow) => {
    if (!selectedTableName) return;
    try {
      await refreshCacheRow(selectedTableName, row.id);
      message.success(t("cache.toasts.expired"));
      await handleRefresh();
    } catch (error) {
      message.error(getErrorMessage(error));
    }
  };

  const handleDelete = async (row: CacheRow) => {
    if (!selectedTableName) return;
    try {
      await deleteCacheRow(selectedTableName, row.id);
      message.success(t("cache.toasts.deleted"));
      await handleRefresh();
    } catch (error) {
      message.error(getErrorMessage(error));
    }
  };

  const columns: TableProps<CacheRow>["columns"] = [
    {
      title: t("cache.columns.id"),
      dataIndex: "id",
      key: "id",
      fixed: "left",
      ellipsis: true,
      width: 220,
    },
    {
      title: t("cache.columns.key"),
      dataIndex: "key",
      key: "key",
      ellipsis: true,
      width: 260,
      render: (value: string) => <Tooltip title={value}>{value}</Tooltip>,
    },
    {
      title: t("cache.columns.fetchedAt"),
      dataIndex: "fetched_at",
      key: "fetched_at",
      width: 180,
      render: (value: string) => (
        <div>
          <div>{formatDateTime(value)}</div>
          <div className="text-xs text-fg-muted-light dark:text-fg-muted-dark">
            {formatRelative(value, t)}
          </div>
        </div>
      ),
    },
    {
      title: t("cache.columns.rawPreview"),
      dataIndex: "raw_preview",
      key: "raw_preview",
      render: (value: string | null) => (
        <code className="text-xs break-all line-clamp-2">{value ?? ""}</code>
      ),
    },
    {
      title: t("cache.columns.operations"),
      key: "operations",
      fixed: "right",
      width: 240,
      render: (_: unknown, record: CacheRow) => (
        <div className="flex items-center gap-2">
          <Button size="small" onClick={() => setPreviewRow(record)}>
            {t("cache.actions.viewFull")}
          </Button>
          <Button size="small" onClick={() => void handleExpire(record)}>
            {t("cache.actions.expire")}
          </Button>
          <Popconfirm
            title={t("cache.confirmDeleteTitle")}
            okText={t("common.delete")}
            cancelText={t("common.cancel")}
            onConfirm={() => void handleDelete(record)}
          >
            <Button danger size="small">
              {t("cache.actions.delete")}
            </Button>
          </Popconfirm>
        </div>
      ),
    },
  ];

  const preview = previewRow?.raw_preview ?? "";

  return (
    <div className="mx-auto w-full max-w-7xl px-8 py-8">
      <header className="mb-8">
        <h1 className="text-2xl font-semibold text-fg-light dark:text-fg-dark">
          {t("cache.title")}
        </h1>
        <p className="mt-1 text-sm text-fg-muted-light dark:text-fg-muted-dark">
          {t("cache.description")}
        </p>
      </header>

      <section className="mb-6">
        <div className="flex flex-wrap items-center gap-3">
          <Select
            className="min-w-72"
            loading={tablesLoading}
            options={tables.map((table) => ({
              label: `${table.name} (${table.row_count})`,
              value: table.name,
            }))}
            value={selectedTableName}
            onChange={(value) => {
              setSelectedTableName(value);
              setPage(1);
            }}
            placeholder={t("cache.tablePlaceholder")}
          />
          <div className="text-sm text-fg-muted-light dark:text-fg-muted-dark">
            {formatTtl(selectedTable?.avg_ttl_remaining_seconds ?? null, t)}
          </div>
          <Input.Search
            className="min-w-72 max-w-md"
            allowClear
            value={search}
            onChange={(event) => {
              setSearch(event.target.value);
              setPage(1);
            }}
            placeholder={t("cache.searchPlaceholder")}
          />
          <Button
            loading={tablesLoading || rowsLoading}
            onClick={() => void handleRefresh()}
          >
            {t("common.refresh")}
          </Button>
        </div>
      </section>

      <section className="mb-6">
        <Table
          rowKey="id"
          columns={columns}
          dataSource={filteredRows}
          loading={rowsLoading}
          scroll={{ x: 1180 }}
          pagination={{
            current: page,
            pageSize: PAGE_SIZE,
            showSizeChanger: false,
            total: selectedTable?.row_count ?? 0,
            onChange: (nextPage) => setPage(nextPage),
          }}
          className="[&_.ant-table-tbody>tr]:transition-colors [&_.ant-table-tbody>tr:hover>td]:bg-fill-tertiary-light dark:[&_.ant-table-tbody>tr:hover>td]:bg-fill-tertiary-dark [&_.ant-table-tbody>tr>td]:py-3"
        />
      </section>

      <Modal
        title={t("cache.previewModalTitle")}
        open={previewRow !== null}
        width={720}
        footer={null}
        onCancel={() => setPreviewRow(null)}
      >
        <p className="mb-3 text-sm text-fg-muted-light dark:text-fg-muted-dark">
          {t("cache.previewHint")}
        </p>
        <pre className="overflow-x-auto whitespace-pre-wrap break-all bg-fill-tertiary-light dark:bg-fill-tertiary-dark p-3 rounded-md text-xs max-h-[60vh] overflow-y-auto">
          {preview}
        </pre>
      </Modal>
    </div>
  );
}

export default CacheInspectorPage;
