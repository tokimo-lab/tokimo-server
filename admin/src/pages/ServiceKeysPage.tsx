import { Button, Form, Input, Modal, Space, Table, message } from "antd";
import { useEffect, useState } from "react";
import {
  createServiceKey,
  deleteServiceKey,
  listServiceKeys,
} from "../api/client";

interface ServiceKey {
  id: string;
  name: string;
  token_prefix: string;
  enabled: boolean;
  created_at: string;
  token?: string;
}

function ServiceKeysPage() {
  const [keys, setKeys] = useState<ServiceKey[]>([]);
  const [loading, setLoading] = useState(false);
  const [modalVisible, setModalVisible] = useState(false);
  const [createdToken, setCreatedToken] = useState<string | null>(null);
  const [form] = Form.useForm();

  const loadKeys = async () => {
    setLoading(true);
    try {
      const data = await listServiceKeys();
      setKeys(data);
    } catch (error) {
      message.error(String(error));
    } finally {
      setLoading(false);
    }
  };

  // biome-ignore lint/correctness/useExhaustiveDependencies: loadKeys is stable
  useEffect(() => {
    loadKeys();
  }, []);

  const handleCreate = async (values: { name: string }) => {
    try {
      const result = await createServiceKey(values);
      setCreatedToken(result.token);
      message.success("Service key created");
      form.resetFields();
      loadKeys();
    } catch (error) {
      message.error(String(error));
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await deleteServiceKey(id);
      message.success("Service key deleted");
      loadKeys();
    } catch (error) {
      message.error(String(error));
    }
  };

  const columns = [
    { title: "Name", dataIndex: "name", key: "name" },
    { title: "Prefix", dataIndex: "token_prefix", key: "token_prefix" },
    {
      title: "Enabled",
      dataIndex: "enabled",
      key: "enabled",
      render: (v: boolean) => (v ? "Yes" : "No"),
    },
    { title: "Created", dataIndex: "created_at", key: "created_at" },
    {
      title: "Action",
      key: "action",
      render: (_: unknown, record: ServiceKey) => (
        <Button danger size="small" onClick={() => handleDelete(record.id)}>
          Delete
        </Button>
      ),
    },
  ];

  return (
    <div>
      <div style={{ marginBottom: 16 }}>
        <Button type="primary" onClick={() => setModalVisible(true)}>
          Create Service Key
        </Button>
      </div>
      <Table
        dataSource={keys}
        columns={columns}
        loading={loading}
        rowKey="id"
      />

      <Modal
        title="Create Service Key"
        open={modalVisible}
        onCancel={() => {
          setModalVisible(false);
          setCreatedToken(null);
        }}
        footer={null}
      >
        {createdToken ? (
          <div>
            <p>
              Token created successfully. Copy it now (it won't be shown again):
            </p>
            <Input.TextArea value={createdToken} rows={3} readOnly />
            <Button
              type="primary"
              onClick={() => {
                setModalVisible(false);
                setCreatedToken(null);
              }}
              style={{ marginTop: 16 }}
            >
              Close
            </Button>
          </div>
        ) : (
          <Form form={form} onFinish={handleCreate} layout="vertical">
            <Form.Item label="Name" name="name" rules={[{ required: true }]}>
              <Input />
            </Form.Item>
            <Form.Item>
              <Space>
                <Button type="primary" htmlType="submit">
                  Create
                </Button>
                <Button onClick={() => setModalVisible(false)}>Cancel</Button>
              </Space>
            </Form.Item>
          </Form>
        )}
      </Modal>
    </div>
  );
}

export default ServiceKeysPage;
