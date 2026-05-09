import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Alert,
  Button,
  Input,
  InputNumber,
  Spin,
  Table,
  Tag,
  Tooltip,
  message,
} from "antd";
import { Suspense, lazy, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  type AdminProvider,
  listAdminProviders,
  patchAdminProvider,
} from "../api/client";
import { useDocsRegister } from "../system/docs";
import {
  clearServiceKey,
  loadServiceKey,
  saveServiceKey,
} from "../utils/serviceKey";

const ServiceKeyPromptModal = lazy(
  () => import("../components/ServiceKeyPromptModal"),
);
const ProviderResponseModal = lazy(
  () => import("../components/ProviderResponseModal"),
);

interface FetchResult {
  status: number;
  duration: number;
  contentType: string;
  body: string;
  error?: string;
}

function expandSample(sample: string): string {
  const today = new Date().toISOString().slice(0, 10);
  return sample.replace(/\{TODAY\}/g, today);
}

function humanizeTtl(seconds: number): string {
  if (seconds <= 0) return "0s";
  if (seconds % 86400 === 0) return `${seconds / 86400}d`;
  if (seconds % 3600 === 0) return `${seconds / 3600}h`;
  if (seconds % 60 === 0) return `${seconds / 60}m`;
  return `${seconds}s`;
}

function ProviderConfigsPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [serviceKey, setServiceKey] = useState<string>(() => loadServiceKey());
  const [active, setActive] = useState<AdminProvider | null>(null);
  const [result, setResult] = useState<FetchResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [promptOpen, setPromptOpen] = useState(false);
  const pendingRowRef = useRef<AdminProvider | null>(null);
  const [messageApi, contextHolder] = message.useMessage();

  const [editing, setEditing] = useState<{ key: string; value: number } | null>(
    null,
  );

  const providersQuery = useQuery({
    queryKey: ["admin", "providers"],
    queryFn: () => listAdminProviders(),
    staleTime: 30_000,
  });

  const ttlMutation = useMutation({
    mutationFn: (vars: { key: string; ttl_seconds: number }) =>
      patchAdminProvider(vars.key, { ttl_seconds: vars.ttl_seconds }),
    onSuccess: () => {
      messageApi.success(t("providers.ttl.updated"));
      setEditing(null);
      queryClient.invalidateQueries({ queryKey: ["admin", "providers"] });
    },
    onError: (err) => {
      messageApi.error(
        `${t("providers.ttl.updateFailed")}: ${err instanceof Error ? err.message : String(err)}`,
      );
    },
  });

  useEffect(() => {
    saveServiceKey(serviceKey);
  }, [serviceKey]);

  const authTag = (v: AdminProvider["auth_required"]) => {
    switch (v) {
      case "yes":
        return <Tag color="red">{t("providers.auth.required")}</Tag>;
      case "optional":
        return <Tag color="gold">{t("providers.auth.optional")}</Tag>;
      case "no":
        return <Tag color="green">{t("providers.auth.none")}</Tag>;
    }
  };

  const fireRequest = async (row: AdminProvider, key: string) => {
    setActive(row);
    setResult(null);
    setLoading(true);
    const url = expandSample(row.sample);
    const started = performance.now();
    try {
      const res = await fetch(url, {
        headers: key ? { Authorization: `Bearer ${key}` } : undefined,
      });
      const text = await res.text();
      const ct = res.headers.get("content-type") ?? "";
      let body = text;
      if (ct.includes("application/json")) {
        try {
          body = JSON.stringify(JSON.parse(text), null, 2);
        } catch {
          /* keep raw */
        }
      }
      setResult({
        status: res.status,
        duration: Math.round(performance.now() - started),
        contentType: ct,
        body,
      });
    } catch (e) {
      setResult({
        status: 0,
        duration: Math.round(performance.now() - started),
        contentType: "",
        body: "",
        error: e instanceof Error ? e.message : String(e),
      });
    } finally {
      setLoading(false);
    }
  };

  const handleSend = (row: AdminProvider) => {
    if (!serviceKey) {
      pendingRowRef.current = row;
      setPromptOpen(true);
      return;
    }
    void fireRequest(row, serviceKey);
  };

  const handlePromptSubmit = (key: string) => {
    setServiceKey(key);
    setPromptOpen(false);
    const row = pendingRowRef.current;
    pendingRowRef.current = null;
    if (row) {
      void fireRequest(row, key);
    }
  };

  const handlePromptCancel = () => {
    pendingRowRef.current = null;
    setPromptOpen(false);
  };

  const handleClearKey = () => {
    clearServiceKey();
    setServiceKey("");
    messageApi.success(t("providers.serviceKey.cleared"));
  };

  const handleCloseResponse = () => {
    setActive(null);
    setResult(null);
  };

  const columns = [
    {
      title: t("providers.columns.name"),
      dataIndex: "key",
      key: "name",
      width: 180,
      render: (_: unknown, row: AdminProvider) => (
        <span className="font-medium">
          {t(row.i18n_name_key, { defaultValue: row.key })}
        </span>
      ),
    },
    {
      title: t("providers.columns.category"),
      dataIndex: "category",
      key: "category",
      width: 110,
      render: (v: string) => (
        <span className="text-xs text-fg-muted-light dark:text-fg-muted-dark">
          {t(`providers.categories.${v}`, { defaultValue: v })}
        </span>
      ),
    },
    {
      title: t("providers.columns.prefix"),
      dataIndex: "prefix",
      key: "prefix",
      width: 200,
      ellipsis: { showTitle: false },
      render: (v: string) => (
        <Tooltip title={v} placement="topLeft">
          <code className="text-xs text-fg-muted-light dark:text-fg-muted-dark">
            {v}
          </code>
        </Tooltip>
      ),
    },
    {
      title: t("providers.columns.rateLimit"),
      dataIndex: "rate_limit",
      key: "rateLimit",
      width: 100,
    },
    {
      title: t("providers.columns.auth"),
      dataIndex: "auth_required",
      key: "auth",
      width: 90,
      render: authTag,
    },
    {
      title: t("providers.columns.envVars"),
      key: "envVars",
      width: 220,
      render: (_: unknown, row: AdminProvider) => {
        if (row.env_keys.length === 0) {
          return (
            <span className="text-fg-muted-light dark:text-fg-muted-dark">
              —
            </span>
          );
        }
        return (
          <div className="flex flex-wrap gap-1">
            {row.env_keys.map((k) => {
              const ok = row.env_status[k] === true;
              return (
                <Tooltip
                  key={k}
                  title={t(
                    ok
                      ? "providers.envStatus.configured"
                      : "providers.envStatus.missing",
                  )}
                >
                  <Tag color={ok ? "green" : "default"} className="text-xs">
                    {k}
                  </Tag>
                </Tooltip>
              );
            })}
          </div>
        );
      },
    },
    {
      title: t("providers.columns.ttl"),
      key: "ttl",
      width: 220,
      render: (_: unknown, row: AdminProvider) => {
        if (!row.has_ttl) {
          return (
            <Tooltip title={t("providers.ttl.permanentHint")}>
              <Tag color="default">{t("providers.ttl.permanent")}</Tag>
            </Tooltip>
          );
        }

        const isEditing = editing?.key === row.key;
        if (isEditing) {
          const changed = editing.value !== row.ttl_seconds;
          return (
            <div className="flex items-center gap-1">
              <InputNumber
                size="small"
                min={0}
                value={editing.value}
                onChange={(v) =>
                  setEditing({
                    key: row.key,
                    value: typeof v === "number" ? v : 0,
                  })
                }
                style={{ width: 90 }}
              />
              <Tooltip title={t("providers.ttl.zeroHint")}>
                <span className="text-xs text-fg-muted-light dark:text-fg-muted-dark">
                  {t("providers.ttl.seconds")}
                </span>
              </Tooltip>
              <Button
                size="small"
                type="primary"
                disabled={!changed || ttlMutation.isPending}
                loading={ttlMutation.isPending}
                onClick={() =>
                  ttlMutation.mutate({
                    key: row.key,
                    ttl_seconds: editing.value,
                  })
                }
              >
                {t("providers.ttl.save")}
              </Button>
              <Button
                size="small"
                onClick={() => setEditing(null)}
                disabled={ttlMutation.isPending}
              >
                {t("providers.ttl.cancel")}
              </Button>
            </div>
          );
        }
        return (
          <div className="flex items-center gap-2">
            <Tooltip title={`${row.ttl_seconds}s`}>
              <span className="font-mono text-xs">
                {humanizeTtl(row.ttl_seconds)}
              </span>
            </Tooltip>
            <Button
              size="small"
              onClick={() => {
                if (!row.has_ttl) return;
                setEditing({ key: row.key, value: row.ttl_seconds });
              }}
            >
              {t("providers.ttl.edit")}
            </Button>
          </div>
        );
      },
    },
    {
      title: t("providers.columns2.sampleUrl"),
      dataIndex: "sample",
      key: "sample",
      ellipsis: { showTitle: false },
      render: (v: string) => (
        <Tooltip title={v} placement="topLeft">
          <code className="min-w-0 break-all text-xs text-fg-muted-light dark:text-fg-muted-dark">
            {v}
          </code>
        </Tooltip>
      ),
    },
    {
      title: t("providers.columns2.action"),
      key: "action",
      width: 100,
      render: (_: unknown, row: AdminProvider) => (
        <Button size="small" type="primary" onClick={() => handleSend(row)}>
          {t("providers.test.sendBtn")}
        </Button>
      ),
    },
  ];

  const overviewRef = useRef<HTMLDivElement>(null);
  const tableRef = useRef<HTMLDivElement>(null);
  const responseModalRef = useRef<HTMLDivElement>(null);

  useDocsRegister(
    useMemo(
      () => ({
        id: "provider-configs-overview",
        sections: [
          { key: "overview" },
          { key: "service-key" },
          { key: "security" },
        ],
        fields: [
          { key: "input-service-key", type: "Bearer · localStorage" },
          { key: "action-clear-key", type: "button" },
        ],
        anchorRef: overviewRef,
      }),
      [],
    ),
  );
  useDocsRegister(
    useMemo(
      () => ({
        id: "provider-configs-table",
        fields: [
          { key: "column-name", type: "string · i18n" },
          { key: "column-category", type: "string · i18n" },
          { key: "column-prefix", type: "string" },
          { key: "column-rate-limit", type: "string · token-bucket" },
          { key: "column-auth", type: "required | optional | none" },
          { key: "column-env-vars", type: "Tag[] · configured/missing" },
          { key: "column-ttl", type: "u64 · seconds (editable)" },
          { key: "column-sample-url", type: "path" },
          { key: "column-action-send", type: "button" },
        ],
        anchorRef: tableRef,
      }),
      [],
    ),
  );
  useDocsRegister(
    useMemo(
      () => ({
        id: "provider-test-response-modal",
        sections: [{ key: "overview" }],
        fields: [
          { key: "response-status", type: "i32 · HTTP code" },
          { key: "response-duration", type: "ms · performance.now()" },
          { key: "response-content-type", type: "string" },
          { key: "response-body", type: "string · auto-pretty" },
        ],
        anchorRef: responseModalRef,
      }),
      [],
    ),
  );

  const providers = providersQuery.data ?? [];

  return (
    <div className="mx-auto w-full max-w-7xl flex flex-col gap-4">
      {contextHolder}
      <div ref={overviewRef}>
        <header className="mb-4">
          <h1 className="text-2xl font-semibold text-fg-light dark:text-fg-dark">
            {t("providers.title")}
          </h1>
          <p className="mt-1 text-sm text-fg-muted-light dark:text-fg-muted-dark">
            {t("providers.description", { count: providers.length })}
          </p>
        </header>
        <div className="mb-4 flex w-full gap-2">
          <Input
            addonBefore={t("providers.serviceKey.label")}
            placeholder={t("providers.serviceKey.placeholder")}
            value={serviceKey}
            onChange={(e) => setServiceKey(e.target.value)}
            allowClear
            className="min-w-0 flex-1"
          />
          <Button onClick={handleClearKey} disabled={!serviceKey}>
            {t("providers.serviceKey.clear")}
          </Button>
        </div>
      </div>
      <div ref={tableRef}>
        {providersQuery.isLoading ? (
          <div className="flex justify-center py-10">
            <Spin />
          </div>
        ) : providersQuery.isError ? (
          <Alert
            type="error"
            showIcon
            message={t("providers.loadError")}
            description={
              providersQuery.error instanceof Error
                ? providersQuery.error.message
                : String(providersQuery.error)
            }
            action={
              <Button size="small" onClick={() => providersQuery.refetch()}>
                {t("providers.retry")}
              </Button>
            }
          />
        ) : (
          <Table
            dataSource={providers}
            rowKey="key"
            columns={columns}
            pagination={false}
            size="small"
            sticky
            scroll={{ x: 1400, y: "calc(100vh - 320px)" }}
            className="[&_.ant-table-tbody>tr]:transition-colors [&_.ant-table-tbody>tr:hover>td]:bg-fill-tertiary-light dark:[&_.ant-table-tbody>tr:hover>td]:bg-fill-tertiary-dark"
          />
        )}
      </div>
      <Suspense fallback={<Spin />}>
        <ServiceKeyPromptModal
          open={promptOpen}
          onSubmit={handlePromptSubmit}
          onCancel={handlePromptCancel}
        />
        <ProviderResponseModal
          open={active !== null}
          provider={
            active
              ? t(active.i18n_name_key, { defaultValue: active.key })
              : undefined
          }
          sample={active ? expandSample(active.sample) : undefined}
          loading={loading}
          result={result}
          onClose={handleCloseResponse}
          anchorRef={responseModalRef}
        />
      </Suspense>
    </div>
  );
}

export default ProviderConfigsPage;
