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
