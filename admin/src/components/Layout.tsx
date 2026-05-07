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
  Avatar,
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
    <AntLayout style={{ minHeight: "100vh" }}>
      <Sider
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
          mode="inline"
          selectedKeys={[selectedKey]}
          theme={mode}
          style={{ borderRight: 0, paddingTop: 16 }}
          items={routes.map((item) => ({
            key: item.key,
            icon: item.icon,
            label: <Link to={item.path}>{t(`nav.${item.key}`)}</Link>,
          }))}
        />
      </Sider>
      <AntLayout>
        <Header
          style={{
            alignItems: "center",
            background: isDark ? "#141414" : "#fff",
            borderBottom: `1px solid ${isDark ? "#303030" : "#f0f0f0"}`,
            display: "flex",
            justifyContent: "space-between",
            padding: "0 24px",
          }}
        >
          <div style={{ alignItems: "center", display: "flex", gap: 24 }}>
            <div
              style={{
                color: isDark ? "#fff" : "#111827",
                fontSize: 18,
                fontWeight: 700,
                letterSpacing: 0.2,
              }}
            >
              tokimo-server
            </div>
            <Breadcrumb
              items={[{ title: t(`nav.${selectedKey}`) }]}
              style={{ margin: 0 }}
            />
          </div>
          <div style={{ alignItems: "center", display: "flex", gap: 12 }}>
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
                shape="circle"
                type="text"
                icon={<Avatar size={28} icon={<UserOutlined />} />}
              />
            </Dropdown>
          </div>
        </Header>
        <AntLayout
          style={{
            background: isDark ? "#000" : "#f5f7fb",
            padding: 24,
          }}
        >
          <Content
            style={{
              background: isDark ? "#141414" : "#fff",
              border: `1px solid ${isDark ? "#303030" : "#edf0f5"}`,
              borderRadius: 16,
              boxShadow: isDark
                ? "0 16px 40px rgba(0, 0, 0, 0.35)"
                : "0 16px 40px rgba(15, 23, 42, 0.06)",
              margin: 0,
              minHeight: 280,
              padding: 24,
            }}
          >
            <Outlet />
          </Content>
        </AntLayout>
      </AntLayout>
    </AntLayout>
  );
}

export default Layout;
