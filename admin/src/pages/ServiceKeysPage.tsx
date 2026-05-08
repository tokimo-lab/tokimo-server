import { Button, Form, Input, Modal, Table, message } from "antd";
import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  createServiceKey,
  deleteServiceKey,
  listServiceKeys,
} from "../api/client";
import { useDocsRegister } from "../system/docs";

interface ServiceKey {
  id: string;
  name: string;
  token_prefix: string;
  enabled: boolean;
  created_at: string;
  token?: string;
}

function ServiceKeysPage() {
  const { t } = useTranslation();
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
      message.success(t("serviceKeys.toasts.created"));
      form.resetFields();
      loadKeys();
    } catch (error) {
      message.error(String(error));
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await deleteServiceKey(id);
      message.success(t("serviceKeys.toasts.deleted"));
      loadKeys();
    } catch (error) {
      message.error(String(error));
    }
  };

  const columns = [
    {
      title: t("serviceKeys.columns.name"),
      dataIndex: "name",
      key: "name",
    },
    {
      title: t("serviceKeys.columns.prefix"),
      dataIndex: "token_prefix",
      key: "token_prefix",
    },
    {
      title: t("serviceKeys.columns.enabled"),
      dataIndex: "enabled",
      key: "enabled",
      render: (v: boolean) => (v ? t("common.yes") : t("common.no")),
    },
    {
      title: t("serviceKeys.columns.created"),
      dataIndex: "created_at",
      key: "created_at",
    },
    {
      title: t("serviceKeys.columns.action"),
      key: "action",
      render: (_: unknown, record: ServiceKey) => (
        <Button danger size="small" onClick={() => handleDelete(record.id)}>
          {t("common.delete")}
        </Button>
      ),
    },
  ];

  const containerRef = useRef<HTMLDivElement>(null);
  useDocsRegister(
    useMemo(
      () => ({
        id: "service-keys",
        sections: [
          { key: "overview" },
          { key: "token-format" },
          { key: "scopes" },
        ],
        fields: [
          { key: "token", type: "string" },
          { key: "created_at", type: "timestamp" },
          { key: "scopes", type: "string[]" },
        ],
        anchorRef: containerRef,
      }),
      [],
    ),
  );

  return (
    <div ref={containerRef} className="mx-auto w-full max-w-7xl">
      <header className="mb-4">
        <h1 className="text-2xl font-semibold text-fg-light dark:text-fg-dark">
          {t("nav.serviceKeys")}
        </h1>
      </header>
      <div className="mb-4">
        <Button type="primary" onClick={() => setModalVisible(true)}>
          {t("serviceKeys.createBtn")}
        </Button>
      </div>
      <Table
        dataSource={keys}
        columns={columns}
        loading={loading}
        rowKey="id"
        size="small"
        sticky
        scroll={{ y: "calc(100vh - 260px)" }}
        className="[&_.ant-table-tbody>tr]:transition-colors [&_.ant-table-tbody>tr:hover>td]:bg-fill-tertiary-light dark:[&_.ant-table-tbody>tr:hover>td]:bg-fill-tertiary-dark"
      />

      <Modal
        title={t("serviceKeys.modalTitle")}
        open={modalVisible}
        onCancel={() => {
          setModalVisible(false);
          setCreatedToken(null);
        }}
        footer={null}
      >
        {createdToken ? (
          <div>
            <p>{t("serviceKeys.tokenCreatedHint")}</p>
            <Input.TextArea
              value={createdToken}
              rows={3}
              readOnly
              className="mt-4 min-w-0 break-all font-mono text-xs"
            />
            <Button
              type="primary"
              onClick={() => {
                setModalVisible(false);
                setCreatedToken(null);
              }}
              block
              className="mt-4"
            >
              {t("common.close")}
            </Button>
          </div>
        ) : (
          <Form form={form} onFinish={handleCreate} layout="vertical">
            <Form.Item
              label={t("serviceKeys.nameLabel")}
              name="name"
              rules={[{ required: true }]}
            >
              <Input />
            </Form.Item>
            <Form.Item className="mb-0">
              <div className="flex items-center gap-2">
                <Button type="primary" htmlType="submit">
                  {t("common.create")}
                </Button>
                <Button onClick={() => setModalVisible(false)}>
                  {t("common.cancel")}
                </Button>
              </div>
            </Form.Item>
          </Form>
        )}
      </Modal>
    </div>
  );
}

export default ServiceKeysPage;
