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
    <div className="grid min-h-screen grid-cols-1 bg-bg-light dark:bg-bg-dark md:grid-cols-[minmax(0,3fr)_minmax(360px,2fr)]">
      <section
        className="relative flex min-h-[38vh] flex-1 items-center justify-center overflow-hidden px-6 py-10 md:min-h-screen md:p-10"
        aria-label="Tokimo Server"
      >
        <div
          className="gradient-bg absolute top-[14%] right-[8%] h-[360px] w-[360px] rounded-full opacity-20 blur-3xl"
          aria-hidden="true"
        />
        <div className="relative z-10 max-w-xl">
          <h1 className="m-0 text-[clamp(40px,7vw,88px)] leading-[0.95] font-bold tracking-[-0.06em]">
            <span className="gradient-text">Tokimo</span>
            <span className="block text-fg-light dark:text-fg-dark">
              Server
            </span>
          </h1>
          <p className="mt-5 max-w-md text-base leading-7 text-fg-muted-light dark:text-fg-muted-dark">
            Minimal admin controls for keys, providers, cache, and service
            health.
          </p>
        </div>
      </section>
      <section
        className="flex items-center justify-center px-6 pb-8 md:border-l md:border-border-light md:p-8 dark:md:border-border-dark"
        aria-label={t("login.cardTitle")}
      >
        <Card
          className="w-full max-w-sm rounded-card border border-border-light bg-panel-light shadow-sm dark:border-border-dark dark:bg-panel-dark"
          classNames={{ body: "p-8" }}
        >
          <h2 className="m-0 text-2xl font-semibold tracking-[-0.03em] text-fg-light dark:text-fg-dark">
            {t("login.cardTitle")}
          </h2>
          <p className="mt-2 mb-6 text-sm text-fg-muted-light dark:text-fg-muted-dark">
            tokimo-server admin console
          </p>
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
                className="gradient-ring-hover transition-all"
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
