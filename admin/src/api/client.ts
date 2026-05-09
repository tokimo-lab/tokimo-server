const API_BASE = "/api";
const JWT_KEY = "tokimo-admin-jwt";

function getHeaders(): HeadersInit {
  const token = localStorage.getItem(JWT_KEY);
  return {
    "Content-Type": "application/json",
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
  };
}

export async function login(bootstrapKey: string): Promise<{ token: string }> {
  const res = await fetch(`${API_BASE}/admin/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ bootstrap_key: bootstrapKey }),
  });

  if (!res.ok) {
    throw new Error("Login failed");
  }

  return res.json();
}

export async function listServiceKeys() {
  const res = await fetch(`${API_BASE}/admin/service-keys`, {
    headers: getHeaders(),
  });

  if (!res.ok) {
    throw new Error("Failed to fetch service keys");
  }

  return res.json();
}

export async function createServiceKey(data: {
  name: string;
  scopes?: unknown;
  expires_at?: string;
}) {
  const res = await fetch(`${API_BASE}/admin/service-keys`, {
    method: "POST",
    headers: getHeaders(),
    body: JSON.stringify(data),
  });

  if (!res.ok) {
    throw new Error("Failed to create service key");
  }

  return res.json();
}

export async function deleteServiceKey(id: string) {
  const res = await fetch(`${API_BASE}/admin/service-keys`, {
    method: "DELETE",
    headers: getHeaders(),
    body: JSON.stringify({ id }),
  });

  if (!res.ok) {
    throw new Error("Failed to delete service key");
  }

  return res.json();
}

export async function listProviderConfigs() {
  const res = await fetch(`${API_BASE}/admin/provider-configs`, {
    headers: getHeaders(),
  });

  if (!res.ok) {
    throw new Error("Failed to fetch provider configs");
  }

  return res.json();
}

export interface AdminProvider {
  key: string;
  category: string;
  prefix: string;
  sample: string;
  rate_limit: string;
  auth_required: "yes" | "optional" | "no";
  env_keys: string[];
  env_status: Record<string, boolean>;
  ttl_seconds: number;
  has_ttl: boolean;
  enabled: boolean;
  i18n_name_key: string;
  i18n_desc_key: string;
}

export async function listAdminProviders(): Promise<AdminProvider[]> {
  const res = await fetch(`${API_BASE}/admin/providers`, {
    headers: getHeaders(),
  });

  if (!res.ok) {
    throw new Error("Failed to fetch admin providers");
  }

  return res.json();
}

export async function patchAdminProvider(
  key: string,
  body: { ttl_seconds?: number; enabled?: boolean },
): Promise<{ ok: true }> {
  const res = await fetch(
    `${API_BASE}/admin/providers/${encodeURIComponent(key)}`,
    {
      method: "PATCH",
      headers: getHeaders(),
      body: JSON.stringify(body),
    },
  );

  if (!res.ok) {
    throw new Error("Failed to update provider config");
  }

  return res.json();
}

export async function listCache() {
  const res = await fetch(`${API_BASE}/admin/cache`, {
    headers: getHeaders(),
  });

  if (!res.ok) {
    throw new Error("Failed to fetch cache entries");
  }

  return res.json();
}

export interface DashboardOverview {
  total_keys: number;
  total_providers: number;
  cache_entries_total: number;
  calls_24h: number;
  errors_24h: number;
  cache_hit_ratio_24h: number;
}

export interface DashboardTimeseriesPoint {
  ts: number;
  calls: number;
  errors: number;
  hits: number;
  misses: number;
  p50_ms?: number;
  p95_ms?: number;
}

export interface DashboardHeatmapValue {
  provider: string;
  calls: number;
}

export interface DashboardHeatmapBucket {
  ts: number;
  values: DashboardHeatmapValue[];
}

export interface DashboardHeatmap {
  providers: string[];
  buckets: DashboardHeatmapBucket[];
}

export interface DashboardStatusCodePoint {
  ts: number;
  ok_2xx: number;
  client_4xx: number;
  server_5xx: number;
}

export interface DashboardProviderStats {
  provider: string;
  calls: number;
  errors: number;
  p50_ms: number;
  p95_ms: number;
  hit_ratio: number;
}

export interface DashboardRecentError {
  ts: number;
  provider: string;
  status: number;
  duration_ms: number;
}

export async function getDashboardOverview(): Promise<DashboardOverview> {
  const res = await fetch(`${API_BASE}/admin/dashboard/overview`, {
    headers: getHeaders(),
  });

  if (!res.ok) {
    throw new Error("Failed to fetch dashboard overview");
  }

  return res.json();
}

export async function getDashboardTimeseries(
  rangeSecs: number,
  bucketSecs: number,
): Promise<DashboardTimeseriesPoint[]> {
  const params = new URLSearchParams({
    range_secs: String(rangeSecs),
    bucket_secs: String(bucketSecs),
  });
  const res = await fetch(
    `${API_BASE}/admin/dashboard/timeseries?${params.toString()}`,
    {
      headers: getHeaders(),
    },
  );

  if (!res.ok) {
    throw new Error("Failed to fetch dashboard timeseries");
  }

  return res.json();
}

export async function getDashboardByProvider(
  rangeSecs: number,
): Promise<DashboardProviderStats[]> {
  const params = new URLSearchParams({ range_secs: String(rangeSecs) });
  const res = await fetch(
    `${API_BASE}/admin/dashboard/by-provider?${params.toString()}`,
    {
      headers: getHeaders(),
    },
  );

  if (!res.ok) {
    throw new Error("Failed to fetch dashboard provider stats");
  }

  return res.json();
}

export async function getDashboardRecentErrors(
  limit: number,
): Promise<DashboardRecentError[]> {
  const params = new URLSearchParams({ limit: String(limit) });
  const res = await fetch(
    `${API_BASE}/admin/dashboard/recent-errors?${params.toString()}`,
    {
      headers: getHeaders(),
    },
  );

  if (!res.ok) {
    throw new Error("Failed to fetch dashboard recent errors");
  }

  return res.json();
}

export async function getDashboardHeatmap(
  rangeSecs: number,
  bucketSecs: number,
): Promise<DashboardHeatmap> {
  const params = new URLSearchParams({
    range_secs: String(rangeSecs),
    bucket_secs: String(bucketSecs),
  });
  const res = await fetch(
    `${API_BASE}/admin/dashboard/heatmap?${params.toString()}`,
    {
      headers: getHeaders(),
    },
  );

  if (!res.ok) {
    throw new Error("Failed to fetch dashboard heatmap");
  }

  return res.json();
}

export async function getDashboardStatusCodes(
  rangeSecs: number,
  bucketSecs: number,
): Promise<DashboardStatusCodePoint[]> {
  const params = new URLSearchParams({
    range_secs: String(rangeSecs),
    bucket_secs: String(bucketSecs),
  });
  const res = await fetch(
    `${API_BASE}/admin/dashboard/status-codes?${params.toString()}`,
    {
      headers: getHeaders(),
    },
  );

  if (!res.ok) {
    throw new Error("Failed to fetch dashboard status codes");
  }

  return res.json();
}

export interface ClearDashboardMetricsResponse {
  cleared_buckets: number;
  since_ts_ms?: number;
  until_ts_ms?: number;
}

export async function clearDashboardMetrics(params: {
  since_ts_ms?: number;
  until_ts_ms?: number;
}): Promise<ClearDashboardMetricsResponse> {
  const res = await fetch(`${API_BASE}/admin/dashboard/clear-metrics`, {
    method: "POST",
    headers: getHeaders(),
    body: JSON.stringify(params),
  });

  if (!res.ok) {
    throw new Error("Failed to clear dashboard metrics");
  }

  return res.json();
}
