import {
  type ReactNode,
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import type { DocDef } from "./types";

const STORAGE_KEY = "tokimo-admin-docs-hub-v1";

const DEFAULT_WIDTH = 480;
const DEFAULT_HEIGHT = 600;

interface PanelLayout {
  x: number;
  y: number;
  width: number;
  height: number;
}

function loadLayout(): PanelLayout {
  if (typeof window === "undefined") {
    return {
      x: 0,
      y: 0,
      width: DEFAULT_WIDTH,
      height: DEFAULT_HEIGHT,
    };
  }
  const fallback: PanelLayout = {
    x: Math.max(window.innerWidth - DEFAULT_WIDTH - 24, 24),
    y: Math.max(window.innerHeight - DEFAULT_HEIGHT - 24, 24),
    width: DEFAULT_WIDTH,
    height: DEFAULT_HEIGHT,
  };
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return fallback;
    const parsed = JSON.parse(raw) as Partial<PanelLayout>;
    return {
      x: typeof parsed.x === "number" ? parsed.x : fallback.x,
      y: typeof parsed.y === "number" ? parsed.y : fallback.y,
      width: typeof parsed.width === "number" ? parsed.width : fallback.width,
      height:
        typeof parsed.height === "number" ? parsed.height : fallback.height,
    };
  } catch {
    return fallback;
  }
}

function persistLayout(layout: PanelLayout) {
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(layout));
  } catch {
    // ignore quota errors
  }
}

export interface DocsContextValue {
  registered: Map<string, DocDef>;
  register: (def: DocDef) => () => void;
  hoveredId: string | null;
  setHoveredId: (id: string | null) => void;
  open: boolean;
  setOpen: (v: boolean) => void;
  minimized: boolean;
  setMinimized: (v: boolean) => void;
  layout: PanelLayout;
  setLayout: (l: PanelLayout) => void;
  selectedId: string | null;
  setSelectedId: (id: string | null) => void;
}

export const DocsContext = createContext<DocsContextValue | null>(null);

export function useDocsContext(): DocsContextValue {
  const ctx = useContext(DocsContext);
  if (!ctx) {
    throw new Error("useDocsContext must be used inside <DocsProvider>");
  }
  return ctx;
}

export function DocsProvider({ children }: { children: ReactNode }) {
  const [registered, setRegistered] = useState<Map<string, DocDef>>(
    () => new Map(),
  );
  const [hoveredId, setHoveredId] = useState<string | null>(null);
  const [open, setOpen] = useState(false);
  const [minimized, setMinimized] = useState(false);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [layout, setLayoutState] = useState<PanelLayout>(() => loadLayout());

  const setLayout = useCallback((next: PanelLayout) => {
    setLayoutState(next);
    persistLayout(next);
  }, []);

  const register = useCallback((def: DocDef) => {
    setRegistered((prev) => {
      const next = new Map(prev);
      next.set(def.id, def);
      return next;
    });
    return () => {
      setRegistered((prev) => {
        if (!prev.has(def.id)) return prev;
        const next = new Map(prev);
        next.delete(def.id);
        return next;
      });
    };
  }, []);

  // Auto-select the first available doc if nothing is selected, or fall back
  // when current selection unmounts.
  useEffect(() => {
    if (registered.size === 0) return;
    if (selectedId && registered.has(selectedId)) return;
    const first = registered.keys().next().value;
    setSelectedId(first ?? null);
  }, [registered, selectedId]);

  const value = useMemo<DocsContextValue>(
    () => ({
      registered,
      register,
      hoveredId,
      setHoveredId,
      open,
      setOpen,
      minimized,
      setMinimized,
      layout,
      setLayout,
      selectedId,
      setSelectedId,
    }),
    [
      registered,
      register,
      hoveredId,
      open,
      minimized,
      layout,
      setLayout,
      selectedId,
    ],
  );

  return <DocsContext.Provider value={value}>{children}</DocsContext.Provider>;
}
