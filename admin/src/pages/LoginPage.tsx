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
    <div
      style={{
        display: "flex",
        justifyContent: "center",
        alignItems: "center",
        height: "100vh",
      }}
    >
      <Card title={t("login.cardTitle")} style={{ width: 400 }}>
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
