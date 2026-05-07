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
];

function readInitialCollapsed() {
  return localStorage.getItem(SIDER_COLLAPSED_STORAGE_KEY) === "true";
}

function Layout() {
  const navigate = useNavigate();
  const location = useLocation();
  const { t, i18n } = useTranslation();
  const { mode, toggleMode } = useAdminTheme();
  const [collapsed, setCollapsed] = useState(readInitialCollapsed);
  const isDark = mode === "dark";

  const selectedKey =
    routes.find((item) => location.pathname.startsWith(item.path))?.key ??
    "dashboard";

  const handleLogout = () => {
    localStorage.removeItem("tokimo-admin-jwt");
    navigate("/login");
  };

  const currentLang = i18n.language?.startsWith("zh") ? "zh" : "en";

  return (
    <AntLayout className="tks-admin-shell">
      <Sider
        className="tks-glass tks-admin-sider"
        collapsible
        collapsed={collapsed}
        collapsedWidth={64}
        onCollapse={(nextCollapsed) => {
          localStorage.setItem(
            SIDER_COLLAPSED_STORAGE_KEY,
            String(nextCollapsed),
          );
          setCollapsed(nextCollapsed);
        }}
        theme={mode}
        width={220}
      >
        <Menu
          className="tks-admin-menu"
          mode="inline"
          selectedKeys={[selectedKey]}
          theme={mode}
          items={routes.map((item) => ({
            key: item.key,
            icon: item.icon,
            label: <Link to={item.path}>{t(`nav.${item.key}`)}</Link>,
          }))}
        />
      </Sider>
      <AntLayout className="tks-admin-main">
        <Header className="tks-glass tks-admin-header">
          <div className="tks-admin-header-left">
            <div className="tks-brand">
              <span className="tks-brand-mark" aria-hidden="true" />
              <span className="tks-brand-text">tokimo-server</span>
            </div>
            <Breadcrumb items={[{ title: t(`nav.${selectedKey}`) }]} />
          </div>
          <div className="tks-admin-header-actions">
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
            <Button
              aria-label={t(
                `header.theme.${mode === "dark" ? "light" : "dark"}`,
              )}
              icon={isDark ? <SunOutlined /> : <MoonOutlined />}
              onClick={toggleMode}
              type="text"
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
          <Content className="tks-card tks-admin-content">
            <Outlet />
          </Content>
        </AntLayout>
      </AntLayout>
    </AntLayout>
  );
}

export default Layout;
