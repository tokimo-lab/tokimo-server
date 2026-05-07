import { Button, Card, Form, Input, message } from "antd";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router";
import { login } from "../api/client";

function LoginPage() {
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();
  const { t } = useTranslation();

  const onFinish = async (values: { bootstrap_key: string }) => {
    setLoading(true);
    try {
      const { token } = await login(values.bootstrap_key);
      localStorage.setItem("tokimo-admin-jwt", token);
      message.success(t("login.success"));
      navigate("/dashboard");
    } catch (error) {
      message.error(String(error));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="tks-login-page">
      <section className="tks-login-hero" aria-label="Tokimo Server">
        <svg
          aria-hidden="true"
          className="tks-login-blob"
          viewBox="0 0 360 360"
        >
          <defs>
            <linearGradient
              id="loginHeroGradient"
              x1="44"
              x2="316"
              y1="40"
              y2="320"
            >
              <stop offset="0" stopColor="#3b82f6" stopOpacity="0.12" />
              <stop offset="0.5" stopColor="#8b5cf6" stopOpacity="0.14" />
              <stop offset="1" stopColor="#ec4899" stopOpacity="0.12" />
            </linearGradient>
          </defs>
          <path
            d="M270 55c48 36 65 109 37 163-28 55-101 90-166 80-64-10-119-66-119-127C22 109 77 41 140 25c63-17 82-6 130 30Z"
            fill="url(#loginHeroGradient)"
          />
        </svg>
        <div className="tks-login-hero-content">
          <h1 className="tks-login-wordmark">
            <span className="gradient-text">Tokimo</span>
            <span className="tks-login-wordmark-server">Server</span>
          </h1>
          <p className="tks-login-tagline">
            Minimal admin controls for keys, providers, cache, and service
            health.
          </p>
        </div>
      </section>
      <section className="tks-login-panel" aria-label={t("login.cardTitle")}>
        <Card className="tks-login-card">
          <h2 className="tks-login-title">{t("login.cardTitle")}</h2>
          <p className="tks-login-subtitle">tokimo-server admin console</p>
          <Form onFinish={onFinish} layout="vertical" requiredMark={false}>
            <Form.Item
              label={t("login.bootstrapKeyLabel")}
              name="bootstrap_key"
              rules={[
                { required: true, message: t("login.bootstrapKeyRequired") },
              ]}
            >
              <Input.Password autoComplete="current-password" />
            </Form.Item>
            <Form.Item>
              <Button
                block
                className="tks-primary-button gradient-border-on-hover"
                htmlType="submit"
                loading={loading}
                type="primary"
              >
                {t("login.submit")}
              </Button>
            </Form.Item>
          </Form>
        </Card>
      </section>
    </div>
  );
}

export default LoginPage;
