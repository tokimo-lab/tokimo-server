import {
  type ReactNode,
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";

export type AdminThemeMode = "light" | "dark" | "system";
export type ResolvedAdminThemeMode = "light" | "dark";

const THEME_STORAGE_KEY = "tks_admin_theme";

const mediaQuery = "(prefers-color-scheme: dark)";

type AdminThemeContextValue = {
  mode: AdminThemeMode;
  resolvedMode: ResolvedAdminThemeMode;
  setMode: (mode: AdminThemeMode) => void;
  toggleMode: () => void;
};

const AdminThemeContext = createContext<AdminThemeContextValue | null>(null);

function readInitialTheme(): AdminThemeMode {
  const stored = localStorage.getItem(THEME_STORAGE_KEY);
  return stored === "light" || stored === "dark" || stored === "system"
    ? stored
    : "system";
}

function getSystemMode(): ResolvedAdminThemeMode {
  return window.matchMedia(mediaQuery).matches ? "dark" : "light";
}

export function AdminThemeProvider({ children }: { children: ReactNode }) {
  const [mode, setModeState] = useState<AdminThemeMode>(readInitialTheme);
  const [systemMode, setSystemMode] =
    useState<ResolvedAdminThemeMode>(getSystemMode);

  useEffect(() => {
    const query = window.matchMedia(mediaQuery);
    const handleChange = () => setSystemMode(query.matches ? "dark" : "light");

    handleChange();
    query.addEventListener("change", handleChange);
    return () => query.removeEventListener("change", handleChange);
  }, []);

  const resolvedMode = mode === "system" ? systemMode : mode;

  useEffect(() => {
    if (mode === "system") {
      document.documentElement.removeAttribute("data-theme");
      return;
    }

    document.documentElement.dataset.theme = mode;
  }, [mode]);

  const setMode = useCallback((nextMode: AdminThemeMode) => {
    localStorage.setItem(THEME_STORAGE_KEY, nextMode);
    setModeState(nextMode);
  }, []);

  const toggleMode = useCallback(() => {
    const nextMode =
      mode === "light" ? "dark" : mode === "dark" ? "system" : "light";
    setMode(nextMode);
  }, [mode, setMode]);

  const value = useMemo<AdminThemeContextValue>(
    () => ({
      mode,
      resolvedMode,
      setMode,
      toggleMode,
    }),
    [mode, resolvedMode, setMode, toggleMode],
  );

  return (
    <AdminThemeContext.Provider value={value}>
      {children}
    </AdminThemeContext.Provider>
  );
}

export function useAdminTheme() {
  const context = useContext(AdminThemeContext);
  if (!context)
    throw new Error("useAdminTheme must be used within AdminThemeProvider");
  return context;
}
