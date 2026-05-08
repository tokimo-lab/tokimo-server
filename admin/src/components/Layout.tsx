import {
  ApiOutlined,
  DashboardOutlined,
  DatabaseOutlined,
  KeyOutlined,
  LeftOutlined,
  LogoutOutlined,
  MoonOutlined,
  RightOutlined,
  SettingOutlined,
  SunOutlined,
  UserOutlined,
} from "@ant-design/icons";
import { useQueryClient } from "@tanstack/react-query";
import {
  Breadcrumb,
  Button,
  DatePicker,
  Dropdown,
  Menu,
  Modal,
  Segmented,
  message,
} from "antd";
import dayjs, { type Dayjs } from "dayjs";
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, Outlet, useLocation, useNavigate } from "react-router";
import { clearDashboardMetrics } from "../api/client";
import { type AdminThemeMode, useAdminTheme } from "../theme";

const SIDER_COLLAPSED_STORAGE_KEY = "tks_admin_sider_collapsed";

const BACKDOOR_CLICK_WINDOW_MS = 1500;
const BACKDOOR_UNLOCK_THRESHOLD = 5;
const BACKDOOR_HINT_THRESHOLD = 3;

type BackdoorRangeMode = "1h" | "24h" | "7d" | "all" | "custom";

type RangeValue = [Dayjs | null, Dayjs | null] | null;

const routes = [
  { key: "dashboard", path: "/dashboard", icon: <DashboardOutlined /> },
  { key: "keys", path: "/keys", icon: <KeyOutlined /> },
  { key: "providers", path: "/providers", icon: <ApiOutlined /> },
  { key: "cache", path: "/cache", icon: <DatabaseOutlined /> },
  { key: "settings", path: "/settings", icon: <SettingOutlined /> },
] as const;

function readInitialCollapsed() {
  return localStorage.getItem(SIDER_COLLAPSED_STORAGE_KEY) === "true";
}

function Layout() {
  const navigate = useNavigate();
  const location = useLocation();
  const { t, i18n } = useTranslation();
  const { mode, resolvedMode, setMode } = useAdminTheme();
  const [collapsed, setCollapsed] = useState(readInitialCollapsed);
  const queryClient = useQueryClient();

  const clickCountRef = useRef(0);
  const lastClickAtRef = useRef(0);
  const [backdoorOpen, setBackdoorOpen] = useState(false);
  const [backdoorMode, setBackdoorMode] = useState<BackdoorRangeMode>("1h");
  const [customRange, setCustomRange] = useState<RangeValue>(null);
  const [clearing, setClearing] = useState(false);

  const handleLogoClick = () => {
    const now = Date.now();
    if (now - lastClickAtRef.current > BACKDOOR_CLICK_WINDOW_MS) {
      clickCountRef.current = 1;
    } else {
      clickCountRef.current += 1;
    }
    lastClickAtRef.current = now;

    const count = clickCountRef.current;
    if (count >= BACKDOOR_UNLOCK_THRESHOLD) {
      clickCountRef.current = 0;
      lastClickAtRef.current = 0;
      setBackdoorMode("1h");
      setCustomRange(null);
      setBackdoorOpen(true);
      return;
    }
    if (count >= BACKDOOR_HINT_THRESHOLD) {
      const remaining = BACKDOOR_UNLOCK_THRESHOLD - count;
      message.info(t("backdoor.toast", { remaining }));
    }
  };

  const handleBackdoorConfirm = async () => {
    const now = Date.now();
    let payload: { since_ts_ms?: number; until_ts_ms?: number } = {};
    switch (backdoorMode) {
      case "1h":
        payload = { since_ts_ms: now - 3_600_000 };
        break;
      case "24h":
        payload = { since_ts_ms: now - 86_400_000 };
        break;
      case "7d":
        payload = { since_ts_ms: now - 604_800_000 };
        break;
      case "all":
        payload = {};
        break;
      case "custom": {
        if (
          !customRange ||
          !Array.isArray(customRange) ||
          !customRange[0] ||
          !customRange[1]
        ) {
          return;
        }
        const [start, end] = customRange as [Dayjs, Dayjs];
        payload = {
          since_ts_ms: start.valueOf(),
          until_ts_ms: end.valueOf(),
        };
        break;
      }
    }

    setClearing(true);
    try {
      const result = await clearDashboardMetrics(payload);
      message.success(t("backdoor.success", { count: result.cleared_buckets }));
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
      setBackdoorOpen(false);
    } catch (err) {
      message.error(String(err));
    } finally {
      setClearing(false);
    }
  };

  const selectedKey =
    routes.find((item) => location.pathname.startsWith(item.path))?.key ??
    "dashboard";

  const handleLogout = () => {
    localStorage.removeItem("tokimo-admin-jwt");
    navigate("/login");
  };

  const currentLang = i18n.language?.startsWith("zh") ? "zh" : "en";

  const toggleCollapsed = () => {
    const nextCollapsed = !collapsed;
    localStorage.setItem(SIDER_COLLAPSED_STORAGE_KEY, String(nextCollapsed));
    setCollapsed(nextCollapsed);
  };

  return (
    <div className="flex min-h-screen w-full bg-bg-light dark:bg-bg-dark">
      <aside
        className={`flex min-h-screen flex-none flex-col overflow-hidden border-r border-border-light bg-white transition-[width] duration-200 dark:border-border-dark dark:bg-[#111114] ${
          collapsed ? "w-[56px]" : "w-[240px]"
        }`}
      >
        <div className="flex h-[52px] flex-none items-center gap-2 border-b border-border-light px-4 whitespace-nowrap dark:border-border-dark">
          <button
            type="button"
            onClick={handleLogoClick}
            aria-label="Tokimo"
            className="flex cursor-pointer items-center gap-2 border-0 bg-transparent p-0 text-left outline-none focus-visible:ring-2 focus-visible:ring-violet-500/40"
          >
            <span
              className="gradient-bg h-5 w-5 flex-none rounded-input"
              aria-hidden="true"
            />
            {!collapsed ? (
              <span className="overflow-hidden text-lg leading-none font-semibold tracking-[-0.02em]">
                <span className="gradient-text">Tokimo</span>{" "}
                <span className="text-fg-muted-light dark:text-fg-muted-dark">
                  Server
                </span>
              </span>
            ) : null}
          </button>
        </div>
        <Menu
          className="border-0 bg-transparent px-2 py-3 [&_.ant-menu-item]:relative [&_.ant-menu-item]:my-0.5 [&_.ant-menu-item]:h-9 [&_.ant-menu-item]:rounded-input [&_.ant-menu-item]:leading-9 [&_.ant-menu-item]:text-fg-muted-light dark:[&_.ant-menu-item]:text-fg-muted-dark [&_.ant-menu-item:hover]:!bg-zinc-100 dark:[&_.ant-menu-item:hover]:!bg-[#18181c] [&_.ant-menu-item:hover]:!text-fg-light dark:[&_.ant-menu-item:hover]:!text-fg-dark [&_.ant-menu-item-selected]:!bg-zinc-100 dark:[&_.ant-menu-item-selected]:!bg-[#18181c] [&_.ant-menu-item-selected]:!text-fg-light dark:[&_.ant-menu-item-selected]:!text-fg-dark"
          inlineCollapsed={collapsed}
          mode="inline"
          selectedKeys={[selectedKey]}
          theme={resolvedMode}
          items={routes.map((item) => ({
            key: item.key,
            icon: item.icon,
            label: (
              <Link
                to={item.path}
                className="relative block cursor-pointer before:gradient-bg before:absolute before:top-1/2 before:-left-8 before:h-5 before:w-[3px] before:-translate-y-1/2 before:rounded-r-full before:opacity-0 aria-[current=page]:before:opacity-100"
                aria-current={selectedKey === item.key ? "page" : undefined}
              >
                {t(`nav.${item.key}`)}
              </Link>
            ),
          }))}
        />
        <button
          type="button"
          className="mt-auto flex h-12 w-full cursor-pointer items-center justify-center border-t border-border-light bg-white text-fg-muted-light hover:bg-zinc-100 dark:border-border-dark dark:bg-[#111114] dark:text-fg-muted-dark dark:hover:bg-[#18181c]"
          aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
          onClick={toggleCollapsed}
        >
          {collapsed ? <RightOutlined /> : <LeftOutlined />}
        </button>
      </aside>
      <div className="min-h-screen flex-1 bg-bg-light dark:bg-bg-dark">
        <header className="flex h-[52px] items-center justify-between border-b border-border-light bg-panel-light px-6 leading-[52px] dark:border-border-dark dark:bg-panel-dark">
          <div className="flex min-w-0 items-center gap-4">
            <Breadcrumb
              className="text-xs text-fg-muted-light dark:text-fg-muted-dark"
              items={[{ title: t(`nav.${selectedKey}`) }]}
            />
          </div>
          <div className="flex items-center gap-2">
            <Segmented
              size="small"
              value={mode}
              onChange={(value) => setMode(value as AdminThemeMode)}
              options={[
                { label: <SunOutlined />, value: "light" },
                { label: <MoonOutlined />, value: "dark" },
                { label: "System", value: "system" },
              ]}
            />
            <Segmented
              size="small"
              value={currentLang}
              onChange={(val) => {
                void i18n.changeLanguage(String(val));
              }}
              options={[
                { label: t("header.language.zh"), value: "zh" },
                { label: t("header.language.en"), value: "en" },
              ]}
            />
            <Dropdown
              menu={{
                items: [
                  {
                    key: "logout",
                    icon: <LogoutOutlined />,
                    label: t("header.logout"),
                    onClick: handleLogout,
                  },
                ],
              }}
              placement="bottomRight"
              trigger={["click"]}
            >
              <Button
                icon={
                  <span className="inline-flex h-7 w-7 items-center justify-center rounded-full border border-border-light bg-zinc-100 text-fg-muted-light dark:border-border-dark dark:bg-[#18181c] dark:text-fg-muted-dark">
                    <UserOutlined />
                  </span>
                }
                shape="circle"
                type="text"
              />
            </Dropdown>
          </div>
        </header>
        <main className="m-0 min-h-[calc(100vh-52px)] bg-bg-light p-6 dark:bg-bg-dark">
          <Outlet />
        </main>
      </div>
      <Modal
        title={t("backdoor.title")}
        open={backdoorOpen}
        onCancel={() => setBackdoorOpen(false)}
        onOk={handleBackdoorConfirm}
        confirmLoading={clearing}
        okText={t("backdoor.confirm")}
        cancelText={t("backdoor.cancel")}
        okButtonProps={{
          danger: true,
          disabled:
            backdoorMode === "custom" &&
            (!customRange ||
              !Array.isArray(customRange) ||
              !customRange[0] ||
              !customRange[1]),
        }}
        destroyOnClose
      >
        <div className="flex flex-col gap-4 py-2">
          <Segmented<BackdoorRangeMode>
            block
            value={backdoorMode}
            onChange={(val) => setBackdoorMode(val)}
            options={[
              { label: t("backdoor.range.1h"), value: "1h" },
              { label: t("backdoor.range.24h"), value: "24h" },
              { label: t("backdoor.range.7d"), value: "7d" },
              { label: t("backdoor.range.all"), value: "all" },
              { label: t("backdoor.range.custom"), value: "custom" },
            ]}
          />
          {backdoorMode === "custom" ? (
            <DatePicker.RangePicker
              showTime
              value={customRange}
              onChange={(val) => setCustomRange(val)}
              presets={[
                {
                  label: t("backdoor.range.1h"),
                  value: [dayjs().subtract(1, "hour"), dayjs()],
                },
                {
                  label: t("backdoor.range.24h"),
                  value: [dayjs().subtract(24, "hour"), dayjs()],
                },
                {
                  label: t("backdoor.range.7d"),
                  value: [dayjs().subtract(7, "day"), dayjs()],
                },
              ]}
            />
          ) : null}
        </div>
      </Modal>
    </div>
  );
}

export default Layout;
