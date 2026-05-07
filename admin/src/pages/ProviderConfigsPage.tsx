import {
  Alert,
  Button,
  Input,
  Space,
  Spin,
  Table,
  Tag,
  Tooltip,
  Typography,
  message,
} from "antd";
import { Suspense, lazy, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  clearServiceKey,
  loadServiceKey,
  saveServiceKey,
} from "../utils/serviceKey";

const ServiceKeyPromptModal = lazy(
  () => import("../components/ServiceKeyPromptModal"),
);
const ProviderResponseModal = lazy(
  () => import("../components/ProviderResponseModal"),
);

const { Title, Paragraph, Text } = Typography;

interface ProviderRow {
  key: string;
  provider: string;
  prefix: string;
  sample: string;
  rateLimit: string;
  authRequired: "yes" | "optional" | "no";
  envVars: string[];
}

const PROVIDERS: ProviderRow[] = [
  {
    key: "tmdb",
    provider: "TMDB",
    prefix: "/api/tmdb/...",
    sample: "/api/tmdb/movie/550",
    rateLimit: "10/s",
    authRequired: "yes",
    envVars: ["TMDB_API_KEY"],
  },
  {
    key: "omdb",
    provider: "OMDb",
    prefix: "/api/omdb/...",
    sample: "/api/omdb/title/tt1375666",
    rateLimit: "10/s",
    authRequired: "yes",
    envVars: ["OMDB_API_KEY"],
  },
  {
    key: "thetvdb",
    provider: "TheTVDB",
    prefix: "/api/thetvdb/...",
    sample: "/api/thetvdb/series/121361",
    rateLimit: "10/s",
    authRequired: "yes",
    envVars: ["THETVDB_API_KEY"],
  },
  {
    key: "bangumi",
    provider: "Bangumi",
    prefix: "/api/bangumi/...",
    sample: "/api/bangumi/subject/8",
    rateLimit: "10/s",
    authRequired: "yes",
    envVars: ["BANGUMI_USER_AGENT"],
  },
  {
    key: "fanart",
    provider: "Fanart",
    prefix: "/api/fanart/...",
    sample: "/api/fanart/movie/550",
    rateLimit: "10/s",
    authRequired: "yes",
    envVars: ["FANART_API_KEY"],
  },
  {
    key: "douban",
    provider: "Douban",
    prefix: "/api/douban/...",
    sample: "/api/douban/subject/26266893",
    rateLimit: "1/s",
    authRequired: "no",
    envVars: [],
  },
  {
    key: "spotify",
    provider: "Spotify",
    prefix: "/api/spotify/...",
    sample: "/api/spotify/track/4uLU6hMCjMI75M1A2tKUQC",
    rateLimit: "30/s",
    authRequired: "yes",
    envVars: ["SPOTIFY_CLIENT_ID", "SPOTIFY_CLIENT_SECRET"],
  },
  {
    key: "musicbrainz",
    provider: "MusicBrainz",
    prefix: "/api/musicbrainz/...",
    sample: "/api/musicbrainz/artist/cc197bad-dc9c-440d-a5b5-d52ba2e14234",
    rateLimit: "1/s",
    authRequired: "yes",
    envVars: ["MUSICBRAINZ_USER_AGENT"],
  },
  {
    key: "deezer",
    provider: "Deezer",
    prefix: "/api/deezer/...",
    sample: "/api/deezer/track/3135556",
    rateLimit: "30/s",
    authRequired: "no",
    envVars: [],
  },
  {
    key: "lrclib",
    provider: "LRCLIB",
    prefix: "/api/lrclib/...",
    sample: "/api/lrclib/get?artist=Coldplay&track=Yellow",
    rateLimit: "30/s",
    authRequired: "no",
    envVars: [],
  },
  {
    key: "qidian",
    provider: "Qidian",
    prefix: "/api/qidian/...",
    sample: "/api/qidian/search?q=%E6%96%97%E7%A0%B4%E8%8B%8D%E7%A9%B9",
    rateLimit: "1/s",
    authRequired: "no",
    envVars: [],
  },
  {
    key: "wikipedia",
    provider: "Wikipedia",
    prefix: "/api/wikipedia/summary",
    sample: "/api/wikipedia/summary?title=Linux&lang=en",
    rateLimit: "10/s",
    authRequired: "no",
    envVars: [],
  },
  {
    key: "openmeteo",
    provider: "Open-Meteo",
    prefix: "/api/openmeteo/...",
    sample: "/api/openmeteo/forecast?lat=40.71&lon=-74.01&days=3",
    rateLimit: "100/s",
    authRequired: "no",
    envVars: [],
  },
  {
    key: "nominatim",
    provider: "Nominatim",
    prefix: "/api/nominatim/...",
    sample: "/api/nominatim/search?q=Brandenburg+Gate",
    rateLimit: "1/s (TOS)",
    authRequired: "yes",
    envVars: ["NOMINATIM_USER_AGENT"],
  },
  {
    key: "geocoding",
    provider: "Geocoding (composite)",
    prefix: "/api/geocoding/...",
    sample: "/api/geocoding/forward?q=Berlin",
    rateLimit: "30/s",
    authRequired: "yes",
    envVars: ["NOMINATIM_USER_AGENT"],
  },
  {
    key: "holiday",
    provider: "Holiday (Timor + Nager)",
    prefix: "/api/holiday/:country/:year",
    sample: "/api/holiday/US/2024",
    rateLimit: "10/s",
    authRequired: "no",
    envVars: [],
  },
  {
    key: "assrt",
    provider: "Assrt",
    prefix: "/api/assrt/...",
    sample: "/api/assrt/search?q=Inception",
    rateLimit: "10/s",
    authRequired: "yes",
    envVars: ["ASSRT_API_KEY"],
  },
  {
    key: "github",
    provider: "GitHub Releases",
    prefix: "/api/github/releases/...",
    sample: "/api/github/releases/rust-lang/rust/latest",
    rateLimit: "30/s",
    authRequired: "optional",
    envVars: ["GITHUB_TOKEN"],
  },
  {
    key: "baidu_hot",
    provider: "Baidu Hot",
    prefix: "/api/hot/list",
    sample: "/api/hot/list?id=weibo",
    rateLimit: "per-source",
    authRequired: "no",
    envVars: [],
  },
  {
    key: "baidu_sports",
    provider: "Baidu Sports",
    prefix: "/api/sports/schedule",
    sample: "/api/sports/schedule?type=hot",
    rateLimit: "10/s",
    authRequired: "no",
    envVars: [],
  },
];

interface FetchResult {
  status: number;
  duration: number;
  contentType: string;
  body: string;
  error?: string;
}

function ProviderConfigsPage() {
  const { t } = useTranslation();
  const [serviceKey, setServiceKey] = useState<string>(() => loadServiceKey());
  const [active, setActive] = useState<ProviderRow | null>(null);
  const [result, setResult] = useState<FetchResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [promptOpen, setPromptOpen] = useState(false);
  const pendingRowRef = useRef<ProviderRow | null>(null);
  const [messageApi, contextHolder] = message.useMessage();

  useEffect(() => {
    saveServiceKey(serviceKey);
  }, [serviceKey]);

  const authTag = (v: ProviderRow["authRequired"]) => {
    switch (v) {
      case "yes":
        return <Tag color="red">{t("providers.auth.required")}</Tag>;
      case "optional":
        return <Tag color="gold">{t("providers.auth.optional")}</Tag>;
      case "no":
        return <Tag color="green">{t("providers.auth.none")}</Tag>;
    }
  };

  const fireRequest = async (row: ProviderRow, key: string) => {
    setActive(row);
    setResult(null);
    setLoading(true);
    const started = performance.now();
    try {
      const res = await fetch(row.sample, {
        headers: key ? { Authorization: `Bearer ${key}` } : undefined,
      });
      const text = await res.text();
      const ct = res.headers.get("content-type") ?? "";
      let body = text;
      if (ct.includes("application/json")) {
        try {
          body = JSON.stringify(JSON.parse(text), null, 2);
        } catch {
          /* keep raw */
        }
      }
      setResult({
        status: res.status,
        duration: Math.round(performance.now() - started),
        contentType: ct,
        body,
      });
    } catch (e) {
      setResult({
        status: 0,
        duration: Math.round(performance.now() - started),
        contentType: "",
        body: "",
        error: e instanceof Error ? e.message : String(e),
      });
    } finally {
      setLoading(false);
    }
  };

  const handleSend = (row: ProviderRow) => {
    if (!serviceKey) {
      pendingRowRef.current = row;
      setPromptOpen(true);
      return;
    }
    void fireRequest(row, serviceKey);
  };

  const handlePromptSubmit = (key: string) => {
    setServiceKey(key);
    setPromptOpen(false);
    const row = pendingRowRef.current;
    pendingRowRef.current = null;
    if (row) {
      void fireRequest(row, key);
    }
  };

  const handlePromptCancel = () => {
    pendingRowRef.current = null;
    setPromptOpen(false);
  };

  const handleClearKey = () => {
    clearServiceKey();
    setServiceKey("");
    messageApi.success(t("providers.serviceKey.cleared"));
  };

  const handleCloseResponse = () => {
    setActive(null);
    setResult(null);
  };

  const columns = [
    {
      title: t("providers.columns.provider"),
      dataIndex: "provider",
      key: "provider",
      width: 180,
    },
    {
      title: t("providers.columns2.sampleUrl"),
      dataIndex: "sample",
      key: "sample",
      ellipsis: { showTitle: false },
      render: (v: string) => (
        <Tooltip title={v} placement="topLeft">
          <Text
            code
            copyable={{
              text: v,
              tooltips: false,
              onCopy: () =>
                messageApi.success(t("providers.serviceKey.copied")),
            }}
            style={{ display: "inline-block", maxWidth: "100%" }}
            ellipsis
          >
            {v}
          </Text>
        </Tooltip>
      ),
    },
    {
      title: t("providers.columns.rateLimit"),
      dataIndex: "rateLimit",
      key: "rateLimit",
      width: 100,
    },
    {
      title: t("providers.columns.auth"),
      dataIndex: "authRequired",
      key: "authRequired",
      width: 90,
      render: authTag,
    },
    {
      title: t("providers.columns.envVars"),
      dataIndex: "envVars",
      key: "envVars",
      width: 220,
      render: (vars: string[]) =>
        vars.length === 0 ? (
          <span style={{ color: "#999" }}>—</span>
        ) : (
          <Space size={[4, 4]} wrap>
            {vars.map((v) => (
              <Tag key={v}>{v}</Tag>
            ))}
          </Space>
        ),
    },
    {
      title: t("providers.columns2.action"),
      key: "action",
      width: 100,
      render: (_: unknown, row: ProviderRow) => (
        <Button size="small" type="primary" onClick={() => handleSend(row)}>
          {t("providers.test.sendBtn")}
        </Button>
      ),
    },
  ];

  return (
    <div>
      {contextHolder}
      <Title level={3}>{t("providers.title")}</Title>
      <Paragraph type="secondary">
        {t("providers.description", { count: PROVIDERS.length })}
      </Paragraph>
      <Alert
        type="info"
        showIcon
        style={{ marginBottom: 16 }}
        message={t("providers.readOnlyTitle")}
        description={t("providers.readOnlyDescription")}
      />
      <Space.Compact style={{ width: "100%", marginBottom: 16 }}>
        <Input
          addonBefore={t("providers.serviceKey.label")}
          placeholder={t("providers.serviceKey.placeholder")}
          value={serviceKey}
          onChange={(e) => setServiceKey(e.target.value)}
          allowClear
        />
        <Button onClick={handleClearKey} disabled={!serviceKey}>
          {t("providers.serviceKey.clear")}
        </Button>
      </Space.Compact>
      <Table
        dataSource={PROVIDERS}
        columns={columns}
        pagination={false}
        size="small"
        scroll={{ x: 960 }}
      />
      <Suspense fallback={<Spin />}>
        <ServiceKeyPromptModal
          open={promptOpen}
          onSubmit={handlePromptSubmit}
          onCancel={handlePromptCancel}
        />
        <ProviderResponseModal
          open={active !== null}
          provider={active?.provider}
          sample={active?.sample}
          loading={loading}
          result={result}
          onClose={handleCloseResponse}
        />
      </Suspense>
    </div>
  );
}

export default ProviderConfigsPage;
