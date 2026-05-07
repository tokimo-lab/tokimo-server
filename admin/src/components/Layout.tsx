import {
  ApiOutlined,
  DashboardOutlined,
  DatabaseOutlined,
  KeyOutlined,
  LogoutOutlined,
  MoonOutlined,
  SettingOutlined,
  SunOutlined,
  UserOutlined,
} from "@ant-design/icons";
import {
  Layout as AntLayout,
  Breadcrumb,
  Button,
  Dropdown,
  Menu,
  Segmented,
} from "antd";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, Outlet, useLocation, useNavigate } from "react-router";
import { type AdminThemeMode, useAdminTheme } from "../theme";

const { Header, Content, Sider } = AntLayout;
const SIDER_COLLAPSED_STORAGE_KEY = "tks_admin_sider_collapsed";

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

  const selectedKey =
    routes.find((item) => location.pathname.startsWith(item.path))?.key ??
    "dashboard";

  const handleLogout = () => {
    localStorage.removeItem("tokimo-admin-jwt");
    navigate("/login");
  };

  const currentLang = i18n.language?.startsWith("zh") ? "zh" : "en";

  return (
    <AntLayout className="min-h-screen bg-bg-light dark:bg-bg-dark">
      <Sider
        className="border-r border-border-light bg-white dark:border-border-dark dark:bg-[#111114] [&_.ant-layout-sider-children]:flex [&_.ant-layout-sider-children]:flex-col [&_.ant-layout-sider-trigger]:!border-t [&_.ant-layout-sider-trigger]:!border-border-light [&_.ant-layout-sider-trigger]:!bg-white [&_.ant-layout-sider-trigger]:!text-fg-muted-light dark:[&_.ant-layout-sider-trigger]:!border-border-dark dark:[&_.ant-layout-sider-trigger]:!bg-[#111114] dark:[&_.ant-layout-sider-trigger]:!text-fg-muted-dark"
        collapsible
        collapsed={collapsed}
        collapsedWidth={56}
        onCollapse={(nextCollapsed) => {
          localStorage.setItem(
            SIDER_COLLAPSED_STORAGE_KEY,
            String(nextCollapsed),
          );
          setCollapsed(nextCollapsed);
        }}
        theme={resolvedMode}
        width={240}
      >
        <div className="flex h-[52px] items-center gap-2 border-b border-border-light px-4 whitespace-nowrap dark:border-border-dark">
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
        </div>
        <Menu
          className="border-0 bg-transparent px-2 py-3 [&_.ant-menu-item]:relative [&_.ant-menu-item]:my-0.5 [&_.ant-menu-item]:h-9 [&_.ant-menu-item]:rounded-input [&_.ant-menu-item]:leading-9 [&_.ant-menu-item]:text-fg-muted-light dark:[&_.ant-menu-item]:text-fg-muted-dark [&_.ant-menu-item:hover]:!bg-zinc-100 dark:[&_.ant-menu-item:hover]:!bg-[#18181c] [&_.ant-menu-item:hover]:!text-fg-light dark:[&_.ant-menu-item:hover]:!text-fg-dark [&_.ant-menu-item-selected]:!bg-zinc-100 dark:[&_.ant-menu-item-selected]:!bg-[#18181c] [&_.ant-menu-item-selected]:!text-fg-light dark:[&_.ant-menu-item-selected]:!text-fg-dark"
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
      </Sider>
      <AntLayout className="min-h-screen bg-bg-light dark:bg-bg-dark">
        <Header className="flex h-[52px] items-center justify-between border-b border-border-light bg-panel-light px-6 leading-[52px] dark:border-border-dark dark:bg-panel-dark">
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
        </Header>
        <AntLayout className="bg-bg-light dark:bg-bg-dark">
          <Content className="m-0 min-h-[calc(100vh-52px)] bg-bg-light p-6 dark:bg-bg-dark">
            <Outlet />
          </Content>
        </AntLayout>
      </AntLayout>
    </AntLayout>
  );
}

export default Layout;
