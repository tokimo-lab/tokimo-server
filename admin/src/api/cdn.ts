const API_BASE = "/api";
const JWT_KEY = "tokimo-admin-jwt";

export interface CdnCompressionDetail {
  table: string;
  column: string;
  compression: string;
}

export interface CdnCompressionInfo {
  total_jsonb_columns: number;
  lz4: number;
  pglz: number;
  details: CdnCompressionDetail[];
}

export interface CdnIndexesInfo {
  fetched_at_indexes: number;
  missing_on_tables: string[];
}

export interface CdnOverview {
  compression: CdnCompressionInfo;
  indexes: CdnIndexesInfo;
}

export interface CdnTableStatus {
  table: string;
  tier: string;
  retention_secs: number | null;
  row_count: number;
  oldest_fetched_at: string | null;
  newest_fetched_at: string | null;
}

export interface TableCleanupResult {
  table: string;
  tier: string;
  rows_deleted: number;
  duration_ms: number;
  error: string | null;
}

export interface CleanupRunStats {
  started_at: string | null;
  finished_at: string | null;
  total_rows_deleted: number;
  per_table: TableCleanupResult[];
  error: string | null;
}

export interface CdnCleanupLastResponse {
  last_run: CleanupRunStats | null;
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
    const detail = await res.text().catch(() => "");
    throw new Error(detail ? `${fallbackMessage}: ${detail}` : fallbackMessage);
  }
  return res.json() as Promise<T>;
}

export async function getCdnOverview(): Promise<CdnOverview> {
  const res = await fetch(`${API_BASE}/admin/cdn/overview`, {
    headers: getHeaders(),
  });
  return readJson<CdnOverview>(res, "Failed to fetch CDN overview");
}

export async function getCdnTables(): Promise<CdnTableStatus[]> {
  const res = await fetch(`${API_BASE}/admin/cdn/tables`, {
    headers: getHeaders(),
  });
  return readJson<CdnTableStatus[]>(res, "Failed to fetch CDN tables");
}

export async function runCdnCleanup(): Promise<CleanupRunStats> {
  const res = await fetch(`${API_BASE}/admin/cdn/cleanup/run`, {
    method: "POST",
    headers: getHeaders(),
  });
  return readJson<CleanupRunStats>(res, "Failed to run CDN cleanup");
}

export async function getLastCleanup(): Promise<CdnCleanupLastResponse> {
  const res = await fetch(`${API_BASE}/admin/cdn/cleanup/last`, {
    headers: getHeaders(),
  });
  return readJson<CdnCleanupLastResponse>(res, "Failed to fetch last cleanup");
}
