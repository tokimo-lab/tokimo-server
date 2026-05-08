import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { App as AntdApp, ConfigProvider, theme } from "antd";
import enUS from "antd/locale/en_US";
import zhCN from "antd/locale/zh_CN";
import React from "react";
import ReactDOM from "react-dom/client";
import { useTranslation } from "react-i18next";
import "@fontsource/inter/variable.css";
import "./styles/reset.css";
import "./styles/index.css";
import App from "./App";
import "./i18n";
import { DocsProvider, DocsRoot } from "./system/docs";
import { AdminThemeProvider, useAdminTheme } from "./theme";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchInterval: 30_000,
      staleTime: 15_000,
    },
  },
});

function LocalizedRoot() {
  const { i18n } = useTranslation();
  const { resolvedMode } = useAdminTheme();
  const antdLocale = i18n.language?.startsWith("zh") ? zhCN : enUS;
  const isDark = resolvedMode === "dark";

  return (
    <ConfigProvider
      locale={antdLocale}
      theme={{
        algorithm: isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
        token: {
          colorPrimary: "#8b5cf6",
          colorPrimaryHover: "#a78bfa",
          colorPrimaryActive: "#7c3aed",
          colorBgBase: isDark ? "#08080b" : "#fafafa",
          colorBgContainer: isDark ? "#111114" : "#ffffff",
          colorBorder: isDark ? "#1f1f23" : "#e5e5e7",
          colorBorderSecondary: isDark ? "#16161a" : "#efefef",
          colorText: isDark ? "#ededed" : "#1a1a1a",
          colorTextSecondary: isDark ? "#9a9aa3" : "#5e5e66",
          borderRadius: 8,
          borderRadiusSM: 6,
          borderRadiusLG: 10,
          fontFamily: "Inter, system-ui, -apple-system, sans-serif",
          fontSize: 13,
          motionDurationFast: "0.15s",
          motionDurationMid: "0.2s",
          boxShadow: "0 1px 2px rgba(0,0,0,0.04)",
          boxShadowSecondary: "0 2px 8px rgba(0,0,0,0.06)",
        },
        components: {
          Table: {
            cellPaddingBlock: 8,
            cellPaddingInline: 12,
            cellPaddingBlockSM: 6,
            cellPaddingInlineSM: 10,
            headerBg: isDark ? "#111114" : "#fafafa",
          },
        },
      }}
    >
      <AntdApp>
        <DocsProvider>
          <App />
          <DocsRoot />
        </DocsProvider>
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
