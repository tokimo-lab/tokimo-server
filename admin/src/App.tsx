import { BrowserRouter, Navigate, Route, Routes } from "react-router";
import Layout from "./components/Layout";
import CacheInspectorPage from "./pages/CacheInspectorPage";
import DashboardPage from "./pages/DashboardPage";
import LoginPage from "./pages/LoginPage";
import ProviderConfigsPage from "./pages/ProviderConfigsPage";
import ServiceKeysPage from "./pages/ServiceKeysPage";
import SettingsPage from "./pages/SettingsPage";

function App() {
  const token = localStorage.getItem("tokimo-admin-jwt");

  if (!token && window.location.pathname !== "/admin/login") {
    window.location.replace("/admin/login");
    return null;
  }

  return (
    <BrowserRouter basename="/admin">
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/" element={<Layout />}>
          <Route index element={<Navigate to="/dashboard" replace />} />
          <Route path="dashboard" element={<DashboardPage />} />
          <Route path="keys" element={<ServiceKeysPage />} />
          <Route path="providers" element={<ProviderConfigsPage />} />
          <Route path="cache" element={<CacheInspectorPage />} />
          <Route path="settings" element={<SettingsPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
