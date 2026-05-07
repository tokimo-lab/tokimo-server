const STORAGE_KEY = "tokimo-admin-service-key";

function encode(value: string): string {
  const bytes = new TextEncoder().encode(value);
  let bin = "";
  for (const b of bytes) {
    bin += String.fromCharCode(b);
  }
  return btoa(bin);
}

function decode(value: string): string {
  try {
    const bin = atob(value);
    const bytes = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i += 1) {
      bytes[i] = bin.charCodeAt(i);
    }
    return new TextDecoder().decode(bytes);
  } catch {
    return "";
  }
}

export function loadServiceKey(): string {
  const raw = localStorage.getItem(STORAGE_KEY);
  if (!raw) return "";
  return decode(raw);
}

export function saveServiceKey(value: string): void {
  if (!value) {
    localStorage.removeItem(STORAGE_KEY);
    return;
  }
  localStorage.setItem(STORAGE_KEY, encode(value));
}

export function clearServiceKey(): void {
  localStorage.removeItem(STORAGE_KEY);
}
