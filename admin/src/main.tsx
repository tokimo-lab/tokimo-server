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

const sharedToken: ThemeConfig["token"] = {
  fontFamily: "Inter, -apple-system, system-ui, sans-serif",
  borderRadius: 8,
  borderRadiusLG: 10,
  borderRadiusSM: 6,
  motionDurationMid: "0.24s",
  motionDurationSlow: "0.32s",
  motionEaseInOut: "cubic-bezier(0.32, 0.72, 0, 1)",
  colorPrimary: "#FF5CA1",
};

const lightToken: ThemeConfig["token"] = {
  colorBgContainer: "#FFFFFF",
  colorBgLayout: "#F8F5F0",
  colorBorder: "rgba(0,0,0,0.10)",
  colorBorderSecondary: "rgba(0,0,0,0.06)",
  colorPrimaryHover: "#FF8A3D",
  colorText: "rgba(0,0,0,0.92)",
  colorTextSecondary: "rgba(0,0,0,0.62)",
  colorTextTertiary: "rgba(0,0,0,0.40)",
  controlItemBgHover: "rgba(255, 92, 161, 0.08)",
};

const darkToken: ThemeConfig["token"] = {
  colorBgContainer: "#1F1F26",
  colorBgLayout: "#14141A",
  colorBorder: "rgba(255,255,255,0.10)",
  colorBorderSecondary: "rgba(255,255,255,0.06)",
  colorPrimaryHover: "#FF8A3D",
  colorText: "rgba(255,255,255,0.94)",
  colorTextSecondary: "rgba(255,255,255,0.62)",
  colorTextTertiary: "rgba(255,255,255,0.40)",
  controlItemBgHover: "rgba(255, 110, 173, 0.14)",
};

function LocalizedRoot() {
  const { i18n } = useTranslation();
  const { mode } = useAdminTheme();
  const antdLocale = i18n.language?.startsWith("zh") ? zhCN : enUS;
  const modeToken = mode === "dark" ? darkToken : lightToken;

  return (
    <ConfigProvider
      locale={antdLocale}
      theme={{
        algorithm:
          mode === "dark" ? theme.darkAlgorithm : theme.defaultAlgorithm,
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
      <LocalizedRoot />
    </AdminThemeProvider>
  </React.StrictMode>,
);
