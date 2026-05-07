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
      <Card className="tks-card tks-login-card">
        <h1 className="tks-login-title">{t("login.cardTitle")}</h1>
        <p className="tks-login-subtitle">tokimo-server admin console</p>
        <Form onFinish={onFinish} layout="vertical">
          <Form.Item
            label={t("login.bootstrapKeyLabel")}
            name="bootstrap_key"
            rules={[
              { required: true, message: t("login.bootstrapKeyRequired") },
            ]}
          >
            <Input.Password />
          </Form.Item>
          <Form.Item>
            <Button type="primary" htmlType="submit" loading={loading} block>
              {t("login.submit")}
            </Button>
          </Form.Item>
        </Form>
      </Card>
    </div>
  );
}

export default LoginPage;
