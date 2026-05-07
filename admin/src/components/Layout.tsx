import { Layout as AntLayout, Menu, Segmented } from "antd";
import { useTranslation } from "react-i18next";
import { Link, Outlet, useNavigate } from "react-router";

const { Header, Content, Sider } = AntLayout;

function Layout() {
  const navigate = useNavigate();
  const { t, i18n } = useTranslation();

  const handleLogout = () => {
    localStorage.removeItem("tokimo-admin-jwt");
    navigate("/login");
  };

  const currentLang = i18n.language?.startsWith("zh") ? "zh" : "en";

  return (
    <AntLayout style={{ minHeight: "100vh" }}>
      <Header
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          color: "white",
        }}
      >
        <div style={{ color: "white", fontSize: 20 }}>{t("nav.appTitle")}</div>
        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
          <Segmented
            size="small"
            value={currentLang}
            onChange={(val) => {
              void i18n.changeLanguage(String(val));
            }}
            options={[
              { label: "中文", value: "zh" },
              { label: "English", value: "en" },
            ]}
          />
          <button
            type="button"
            onClick={handleLogout}
            style={{
              background: "none",
              border: "1px solid white",
              color: "white",
              padding: "4px 16px",
              cursor: "pointer",
            }}
          >
            {t("common.logout")}
          </button>
        </div>
      </Header>
      <AntLayout>
        <Sider width={200}>
          <Menu
            mode="inline"
            defaultSelectedKeys={["keys"]}
            style={{ height: "100%", borderRight: 0 }}
            items={[
              {
                key: "keys",
                label: <Link to="/keys">{t("nav.serviceKeys")}</Link>,
              },
              {
                key: "providers",
                label: <Link to="/providers">{t("nav.providers")}</Link>,
              },
              {
                key: "cache",
                label: <Link to="/cache">{t("nav.cache")}</Link>,
              },
            ]}
          />
        </Sider>
        <AntLayout style={{ padding: "24px" }}>
          <Content
            style={{
              background: "white",
              padding: 24,
              margin: 0,
              minHeight: 280,
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
