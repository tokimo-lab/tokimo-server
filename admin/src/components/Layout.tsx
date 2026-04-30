import { Layout as AntLayout, Menu } from "antd";
import { Link, Outlet, useNavigate } from "react-router";

const { Header, Content, Sider } = AntLayout;

function Layout() {
  const navigate = useNavigate();

  const handleLogout = () => {
    localStorage.removeItem("tokimo-admin-jwt");
    navigate("/login");
  };

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
        <div style={{ color: "white", fontSize: 20 }}>Tokimo Server Admin</div>
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
          Logout
        </button>
      </Header>
      <AntLayout>
        <Sider width={200}>
          <Menu
            mode="inline"
            defaultSelectedKeys={["keys"]}
            style={{ height: "100%", borderRight: 0 }}
            items={[
              { key: "keys", label: <Link to="/keys">Service Keys</Link> },
              {
                key: "providers",
                label: <Link to="/providers">Provider Configs</Link>,
              },
              { key: "cache", label: <Link to="/cache">Cache Inspector</Link> },
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
