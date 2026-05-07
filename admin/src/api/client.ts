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
  range: string,
  bucket: string,
): Promise<DashboardTimeseriesPoint[]> {
  const params = new URLSearchParams({ range, bucket });
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
  range: string,
): Promise<DashboardProviderStats[]> {
  const params = new URLSearchParams({ range });
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
