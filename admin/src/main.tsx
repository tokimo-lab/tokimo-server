import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { App as AntdApp, ConfigProvider, theme } from "antd";
import type { ThemeConfig } from "antd";
import enUS from "antd/locale/en_US";
import zhCN from "antd/locale/zh_CN";
import React from "react";
import ReactDOM from "react-dom/client";
import { useTranslation } from "react-i18next";
import "./styles/index.css";
import App from "./App";
import "./i18n";
import { AdminThemeProvider, useAdminTheme } from "./theme";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchInterval: 30_000,
      staleTime: 15_000,
    },
  },
});

const sharedToken: ThemeConfig["token"] = {
  borderRadius: 8,
  borderRadiusLG: 10,
  borderRadiusSM: 6,
  boxShadow: "0 1px 2px rgba(0,0,0,0.04)",
  boxShadowSecondary: "0 2px 8px rgba(0,0,0,0.06)",
  colorPrimary: "#8b5cf6",
  colorPrimaryActive: "#7c3aed",
  colorPrimaryHover: "#a78bfa",
  fontFamily:
    "Inter, system-ui, -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif",
  fontSize: 13,
  motionDurationFast: "0.15s",
  motionDurationMid: "0.2s",
  motionDurationSlow: "0.3s",
  motionEaseInOut: "cubic-bezier(0.4, 0, 0.2, 1)",
  motionEaseOut: "cubic-bezier(0.4, 0, 0.2, 1)",
};

const lightToken: ThemeConfig["token"] = {
  colorBgBase: "#fafafa",
  colorBgContainer: "#ffffff",
  colorBgLayout: "#fafafa",
  colorBorder: "#e5e5e7",
  colorBorderSecondary: "#efefef",
  colorText: "#1a1a1a",
  colorTextSecondary: "#5e5e66",
  colorTextTertiary: "#8a8a93",
  controlItemBgHover: "#f4f4f5",
};

const darkToken: ThemeConfig["token"] = {
  colorBgBase: "#08080b",
  colorBgContainer: "#111114",
  colorBgLayout: "#08080b",
  colorBorder: "#1f1f23",
  colorBorderSecondary: "#16161a",
  colorText: "#ededed",
  colorTextSecondary: "#9a9aa3",
  colorTextTertiary: "#6f6f78",
  controlItemBgHover: "#18181c",
};

function LocalizedRoot() {
  const { i18n } = useTranslation();
  const { resolvedMode } = useAdminTheme();
  const antdLocale = i18n.language?.startsWith("zh") ? zhCN : enUS;
  const isDark = resolvedMode === "dark";
  const modeToken = isDark ? darkToken : lightToken;

  return (
    <ConfigProvider
      locale={antdLocale}
      theme={{
        algorithm: isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
        token: {
          ...sharedToken,
          ...modeToken,
        },
      }}
    >
      <AntdApp>
        <App />
      </AntdApp>
    </ConfigProvider>
  );
}

const rootEl = document.getElementById("root");
if (!rootEl) throw new Error("Root element not found");
const root = ReactDOM.createRoot(rootEl);
root.render(
  <React.StrictMode>
    <AdminThemeProvider>
      <QueryClientProvider client={queryClient}>
        <LocalizedRoot />
      </QueryClientProvider>
    </AdminThemeProvider>
  </React.StrictMode>,
);
