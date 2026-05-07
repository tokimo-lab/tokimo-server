import { Button, Modal, Tag, message } from "antd";
import { useTranslation } from "react-i18next";

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
      width={880}
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
      <div className="min-w-0 space-y-4">
        {sample && (
          <div>
            <code className="break-all text-xs text-fg-muted-light dark:text-fg-muted-dark">
              GET {sample}
            </code>
          </div>
        )}
        {loading && (
          <div className="py-12 text-center text-fg-muted-light dark:text-fg-muted-dark">
            {t("providers.test.sending")}
          </div>
        )}
        {result && (
          <div className="space-y-3">
            <div className="flex flex-wrap items-center gap-4">
              <span className="flex items-center gap-2">
                <span className="text-sm font-medium">
                  {t("providers.test.status")}:
                </span>
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
              <span className="text-sm">
                <span className="font-medium">
                  {t("providers.test.duration")}:
                </span>{" "}
                {result.duration} ms
              </span>
              {result.contentType && (
                <span className="min-w-0 text-sm">
                  <span className="font-medium">
                    {t("providers.test.contentType")}:
                  </span>{" "}
                  <code className="break-all text-xs">
                    {result.contentType}
                  </code>
                </span>
              )}
            </div>
            <div>
              <div className="mb-2 text-sm font-medium">
                {t("providers.test.body")}:
              </div>
              <pre className="max-h-96 min-w-0 overflow-auto whitespace-pre-wrap break-all rounded-md bg-fill-tertiary-light p-3 text-xs dark:bg-fill-tertiary-dark">
                {result.error ?? result.body}
              </pre>
            </div>
          </div>
        )}
      </div>
    </Modal>
  );
}

export default ProviderResponseModal;
