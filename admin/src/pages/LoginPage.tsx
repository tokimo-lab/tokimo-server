import { Button, Card, Form, Input, message } from "antd";
import { useState } from "react";
import { useNavigate } from "react-router";
import { login } from "../api/client";

function LoginPage() {
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

  const onFinish = async (values: { bootstrap_key: string }) => {
    setLoading(true);
    try {
      const { token } = await login(values.bootstrap_key);
      localStorage.setItem("tokimo-admin-jwt", token);
      message.success("Login successful");
      navigate("/");
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
      <Card title="Admin Login" style={{ width: 400 }}>
        <Form onFinish={onFinish} layout="vertical">
          <Form.Item
            label="Bootstrap Key"
            name="bootstrap_key"
            rules={[{ required: true, message: "Please input bootstrap key" }]}
          >
            <Input.Password />
          </Form.Item>
          <Form.Item>
            <Button type="primary" htmlType="submit" loading={loading} block>
              Login
            </Button>
          </Form.Item>
        </Form>
      </Card>
    </div>
  );
}

export default LoginPage;
