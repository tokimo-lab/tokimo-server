import { Button, Modal, Space, Tag, Typography, message } from "antd";
import { useTranslation } from "react-i18next";

const { Paragraph, Text } = Typography;

export interface FetchResult {
  status: number;
  duration: number;
  contentType: string;
  body: string;
  error?: string;
}

interface Props {
  open: boolean;
  provider?: string;
  sample?: string;
  loading: boolean;
  result: FetchResult | null;
  onClose: () => void;
}

function ProviderResponseModal({
  open,
  provider,
  sample,
  loading,
  result,
  onClose,
}: Props) {
  const { t } = useTranslation();
  const [messageApi, contextHolder] = message.useMessage();

  const handleCopy = async () => {
    if (!result) return;
    try {
      await navigator.clipboard.writeText(result.body || result.error || "");
      messageApi.success(t("providers.test.copiedResponse"));
    } catch {
      messageApi.error("Clipboard not available");
    }
  };

  return (
    <Modal
      title={provider ? t("providers.test.modalTitle", { provider }) : ""}
      open={open}
      onCancel={onClose}
      width={800}
      destroyOnClose
      footer={[
        <Button key="copy" disabled={!result || loading} onClick={handleCopy}>
          {t("providers.test.copyResponse")}
        </Button>,
        <Button key="close" type="primary" onClick={onClose}>
          {t("common.close")}
        </Button>,
      ]}
    >
      {contextHolder}
      {sample && (
        <Paragraph>
          <Text code>GET {sample}</Text>
        </Paragraph>
      )}
      {loading && <div>{t("providers.test.sending")}</div>}
      {result && (
        <div>
          <Paragraph>
            <Space size="large">
              <span>
                <Text strong>{t("providers.test.status")}:</Text>{" "}
                <Tag
                  color={
                    result.status >= 200 && result.status < 300
                      ? "green"
                      : result.status === 0
                        ? "default"
                        : "red"
                  }
                >
                  {result.status === 0
                    ? t("providers.test.networkError")
                    : result.status}
                </Tag>
              </span>
              <span>
                <Text strong>{t("providers.test.duration")}:</Text>{" "}
                {result.duration} ms
              </span>
              {result.contentType && (
                <span>
                  <Text strong>{t("providers.test.contentType")}:</Text>{" "}
                  <Text code>{result.contentType}</Text>
                </span>
              )}
            </Space>
          </Paragraph>
          <Text strong>{t("providers.test.body")}:</Text>
          <pre
            style={{
              background: "#f5f5f5",
              padding: 12,
              borderRadius: 4,
              maxHeight: 400,
              overflow: "auto",
              fontSize: 12,
              marginTop: 8,
            }}
          >
            {result.error ?? result.body}
          </pre>
        </div>
      )}
    </Modal>
  );
}

export default ProviderResponseModal;
