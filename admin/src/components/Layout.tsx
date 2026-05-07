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
import { useAdminTheme } from "../theme";

const { Header, Content, Sider } = AntLayout;
const SIDER_COLLAPSED_STORAGE_KEY = "tks_admin_sider_collapsed";

const routes = [
  { key: "dashboard", path: "/dashboard", icon: <DashboardOutlined /> },
  { key: "keys", path: "/keys", icon: <KeyOutlined /> },
  { key: "providers", path: "/providers", icon: <ApiOutlined /> },
  { key: "cache", path: "/cache", icon: <DatabaseOutlined /> },
  { key: "settings", path: "/settings", icon: <SettingOutlined /> },
] as const;

const themeLabels = {
  dark: "Dark theme",
  light: "Light theme",
  system: "System theme",
};

function readInitialCollapsed() {
  return localStorage.getItem(SIDER_COLLAPSED_STORAGE_KEY) === "true";
}

function Layout() {
  const navigate = useNavigate();
  const location = useLocation();
  const { t, i18n } = useTranslation();
  const { mode, resolvedMode, toggleMode } = useAdminTheme();
  const [collapsed, setCollapsed] = useState(readInitialCollapsed);

  const selectedKey =
    routes.find((item) => location.pathname.startsWith(item.path))?.key ??
    "dashboard";

  const handleLogout = () => {
    localStorage.removeItem("tokimo-admin-jwt");
    navigate("/login");
  };

  const currentLang = i18n.language?.startsWith("zh") ? "zh" : "en";
  const themeIcon =
    resolvedMode === "dark" ? <MoonOutlined /> : <SunOutlined />;

  return (
    <AntLayout className="tks-admin-shell">
      <Sider
        className="tks-admin-sider"
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
        <div className="tks-sider-brand">
          <span className="tks-brand-mark" aria-hidden="true" />
          {!collapsed ? (
            <span className="tks-brand-wordmark">
              <span className="gradient-text">Tokimo</span>{" "}
              <span className="tks-brand-server">Server</span>
            </span>
          ) : null}
        </div>
        <Menu
          className="tks-admin-menu"
          mode="inline"
          selectedKeys={[selectedKey]}
          theme={resolvedMode}
          items={routes.map((item) => ({
            key: item.key,
            icon: item.icon,
            label: <Link to={item.path}>{t(`nav.${item.key}`)}</Link>,
          }))}
        />
      </Sider>
      <AntLayout className="tks-admin-main">
        <Header className="tks-admin-header">
          <div className="tks-admin-header-left">
            <Breadcrumb items={[{ title: t(`nav.${selectedKey}`) }]} />
          </div>
          <div className="tks-admin-header-actions">
            <Button
              aria-label={`${themeLabels[mode]}: click to cycle light, dark, system`}
              icon={themeIcon}
              onClick={toggleMode}
              type="text"
            >
              {mode === "system" ? "System" : null}
            </Button>
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
                  <span className="tks-avatar-mark">
                    <UserOutlined />
                  </span>
                }
                shape="circle"
                type="text"
              />
            </Dropdown>
          </div>
        </Header>
        <AntLayout className="tks-admin-content-wrap">
          <Content className="tks-admin-content">
            <Outlet />
          </Content>
        </AntLayout>
      </AntLayout>
    </AntLayout>
  );
}

export default Layout;
