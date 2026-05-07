import { App as AntdApp, ConfigProvider, theme } from "antd";
import enUS from "antd/locale/en_US";
import zhCN from "antd/locale/zh_CN";
import React from "react";
import ReactDOM from "react-dom/client";
import { useTranslation } from "react-i18next";
import App from "./App";
import "./i18n";
import { AdminThemeProvider, useAdminTheme } from "./theme";

function LocalizedRoot() {
  const { i18n } = useTranslation();
  const { mode } = useAdminTheme();
  const antdLocale = i18n.language?.startsWith("zh") ? zhCN : enUS;

  return (
    <ConfigProvider
      locale={antdLocale}
      theme={{
        algorithm:
          mode === "dark" ? theme.darkAlgorithm : theme.defaultAlgorithm,
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
      <LocalizedRoot />
    </AdminThemeProvider>
  </React.StrictMode>,
);
