const API_BASE = "/api";
const JWT_KEY = "tokimo-admin-jwt";

export interface CacheTable {
  name: string;
  row_count: number;
  avg_ttl_remaining_seconds: number | null;
}

export interface CacheRow {
  id: string;
  key: string;
  fetched_at: string;
  raw_preview: string | null;
}

export interface CacheListResponse {
  table: string;
  limit: number;
  offset: number;
  rows: CacheRow[];
}

function getHeaders(): HeadersInit {
  const token = localStorage.getItem(JWT_KEY);
  return {
    "Content-Type": "application/json",
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
  };
}

async function readJson<T>(res: Response, fallbackMessage: string): Promise<T> {
  if (!res.ok) {
    throw new Error(fallbackMessage);
  }
  return res.json() as Promise<T>;
}

export async function listCacheTables(): Promise<CacheTable[]> {
  const res = await fetch(`${API_BASE}/admin/cache/tables`, {
    headers: getHeaders(),
  });
  return readJson<CacheTable[]>(res, "Failed to fetch cache tables");
}

export async function listCacheRows(
  table: string,
  limit: number,
  offset: number,
): Promise<CacheListResponse> {
  const params = new URLSearchParams({
    limit: String(limit),
    offset: String(offset),
  });
  const res = await fetch(
    `${API_BASE}/admin/cache/${encodeURIComponent(table)}?${params.toString()}`,
    { headers: getHeaders() },
  );
  return readJson<CacheListResponse>(res, "Failed to fetch cache rows");
}

async function ensureOk(res: Response, fallbackMessage: string): Promise<void> {
  if (res.ok) return;

  const detail = await res.text().catch(() => "");
  throw new Error(detail ? `${fallbackMessage}: ${detail}` : fallbackMessage);
}

export async function deleteCacheRow(table: string, id: string): Promise<void> {
  const res = await fetch(
    `${API_BASE}/admin/cache/${encodeURIComponent(table)}/${encodeURIComponent(id)}`,
    {
      method: "DELETE",
      headers: getHeaders(),
    },
  );
  await ensureOk(res, "Failed to delete cache row");
}

export async function refreshCacheRow(
  table: string,
  id: string,
): Promise<void> {
  const res = await fetch(
    `${API_BASE}/admin/cache/${encodeURIComponent(table)}/${encodeURIComponent(id)}/refresh`,
    {
      method: "POST",
      headers: getHeaders(),
    },
  );
  await ensureOk(res, "Failed to refresh cache row");
}
