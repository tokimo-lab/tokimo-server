import { Form, Input, Modal, Typography } from "antd";
import { useEffect } from "react";
import { useTranslation } from "react-i18next";

const { Paragraph } = Typography;

interface Props {
  open: boolean;
  onSubmit: (value: string) => void;
  onCancel: () => void;
}

interface FormValues {
  key: string;
}

function ServiceKeyPromptModal({ open, onSubmit, onCancel }: Props) {
  const { t } = useTranslation();
  const [form] = Form.useForm<FormValues>();

  useEffect(() => {
    if (open) {
      form.resetFields();
    }
  }, [open, form]);

  const handleOk = async () => {
    const values = await form.validateFields();
    onSubmit(values.key.trim());
  };

  return (
    <Modal
      title={t("providers.serviceKey.promptTitle")}
      open={open}
      onOk={handleOk}
      onCancel={onCancel}
      okText={t("providers.serviceKey.promptSubmit")}
      cancelText={t("common.cancel")}
      destroyOnClose
    >
      <Paragraph type="secondary">
        {t("providers.serviceKey.promptDescription")}
      </Paragraph>
      <Form form={form} layout="vertical" preserve={false}>
        <Form.Item
          name="key"
          label={t("providers.serviceKey.label")}
          rules={[
            {
              required: true,
              message: t("providers.serviceKey.promptRequired"),
            },
          ]}
        >
          <Input.Password
            placeholder={t("providers.serviceKey.placeholder")}
            autoFocus
            onPressEnter={handleOk}
          />
        </Form.Item>
      </Form>
    </Modal>
  );
}

export default ServiceKeyPromptModal;
