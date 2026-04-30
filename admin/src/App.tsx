import { BrowserRouter, Navigate, Route, Routes } from "react-router";
import Layout from "./components/Layout";
import CacheInspectorPage from "./pages/CacheInspectorPage";
import LoginPage from "./pages/LoginPage";
import ProviderConfigsPage from "./pages/ProviderConfigsPage";
import ServiceKeysPage from "./pages/ServiceKeysPage";

function App() {
  const token = localStorage.getItem("tokimo-admin-jwt");

  if (!token && window.location.pathname !== "/admin/login") {
    return <Navigate to="/admin/login" replace />;
  }

  return (
    <BrowserRouter basename="/admin">
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/" element={<Layout />}>
          <Route index element={<Navigate to="/keys" replace />} />
          <Route path="keys" element={<ServiceKeysPage />} />
          <Route path="providers" element={<ProviderConfigsPage />} />
          <Route path="cache" element={<CacheInspectorPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
