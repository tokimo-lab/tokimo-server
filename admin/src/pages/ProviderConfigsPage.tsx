import { Alert, Space, Table, Tag, Typography } from "antd";

const { Title, Paragraph } = Typography;

interface ProviderRow {
  key: string;
  provider: string;
  prefix: string;
  rateLimit: string;
  authRequired: "yes" | "optional" | "no";
  envVars: string[];
}

const PROVIDERS: ProviderRow[] = [
  { key: "tmdb", provider: "TMDB", prefix: "/api/tmdb/...", rateLimit: "10/s", authRequired: "yes", envVars: ["TMDB_API_KEY"] },
  { key: "omdb", provider: "OMDb", prefix: "/api/omdb/...", rateLimit: "10/s", authRequired: "yes", envVars: ["OMDB_API_KEY"] },
  { key: "thetvdb", provider: "TheTVDB", prefix: "/api/thetvdb/...", rateLimit: "10/s", authRequired: "yes", envVars: ["THETVDB_API_KEY"] },
  { key: "bangumi", provider: "Bangumi", prefix: "/api/bangumi/...", rateLimit: "10/s", authRequired: "yes", envVars: ["BANGUMI_USER_AGENT"] },
  { key: "fanart", provider: "Fanart", prefix: "/api/fanart/...", rateLimit: "10/s", authRequired: "yes", envVars: ["FANART_API_KEY"] },
  { key: "douban", provider: "Douban", prefix: "/api/douban/...", rateLimit: "1/s", authRequired: "no", envVars: [] },
  { key: "spotify", provider: "Spotify", prefix: "/api/spotify/...", rateLimit: "30/s", authRequired: "yes", envVars: ["SPOTIFY_CLIENT_ID", "SPOTIFY_CLIENT_SECRET"] },
  { key: "musicbrainz", provider: "MusicBrainz", prefix: "/api/musicbrainz/...", rateLimit: "1/s", authRequired: "yes", envVars: ["MUSICBRAINZ_USER_AGENT"] },
  { key: "deezer", provider: "Deezer", prefix: "/api/deezer/...", rateLimit: "30/s", authRequired: "no", envVars: [] },
  { key: "lrclib", provider: "LRCLIB", prefix: "/api/lrclib/...", rateLimit: "30/s", authRequired: "no", envVars: [] },
  { key: "qidian", provider: "Qidian", prefix: "/api/qidian/...", rateLimit: "1/s", authRequired: "no", envVars: [] },
  { key: "wikipedia", provider: "Wikipedia", prefix: "/api/wikipedia/summary", rateLimit: "10/s", authRequired: "no", envVars: [] },
  { key: "openmeteo", provider: "Open-Meteo", prefix: "/api/openmeteo/...", rateLimit: "100/s", authRequired: "no", envVars: [] },
  { key: "nominatim", provider: "Nominatim", prefix: "/api/nominatim/...", rateLimit: "1/s (TOS)", authRequired: "yes", envVars: ["NOMINATIM_USER_AGENT"] },
  { key: "geocoding", provider: "Geocoding (composite)", prefix: "/api/geocoding/...", rateLimit: "30/s", authRequired: "yes", envVars: ["NOMINATIM_USER_AGENT"] },
  { key: "holiday", provider: "Holiday (Timor + Nager)", prefix: "/api/holiday/:country/:year", rateLimit: "10/s", authRequired: "no", envVars: [] },
  { key: "assrt", provider: "Assrt", prefix: "/api/assrt/...", rateLimit: "10/s", authRequired: "yes", envVars: ["ASSRT_API_KEY"] },
  { key: "github", provider: "GitHub Releases", prefix: "/api/github/releases/...", rateLimit: "30/s", authRequired: "optional", envVars: ["GITHUB_TOKEN"] },
  { key: "baidu_hot", provider: "Baidu Hot", prefix: "/api/hot/list", rateLimit: "per-source", authRequired: "no", envVars: [] },
  { key: "baidu_sports", provider: "Baidu Sports", prefix: "/api/sports/schedule", rateLimit: "10/s", authRequired: "no", envVars: [] },
];

function authTag(v: ProviderRow["authRequired"]) {
  switch (v) {
    case "yes":
      return <Tag color="red">required</Tag>;
    case "optional":
      return <Tag color="gold">optional</Tag>;
    case "no":
      return <Tag color="green">none</Tag>;
  }
}

function ProviderConfigsPage() {
  const columns = [
    { title: "Provider", dataIndex: "provider", key: "provider" },
    {
      title: "Endpoint Prefix",
      dataIndex: "prefix",
      key: "prefix",
      render: (v: string) => <code>{v}</code>,
    },
    { title: "Rate Limit", dataIndex: "rateLimit", key: "rateLimit" },
    {
      title: "Auth",
      dataIndex: "authRequired",
      key: "authRequired",
      render: authTag,
    },
    {
      title: "Env Vars",
      dataIndex: "envVars",
      key: "envVars",
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
  ];

  return (
    <div>
      <Title level={3}>Provider Configurations</Title>
      <Paragraph type="secondary">
        Static view of the {PROVIDERS.length} provider adapters wired into this server. Auth env vars are
        read from the server process environment at startup; live status of which env vars are actually
        populated is not surfaced here to avoid leaking secret presence.
      </Paragraph>
      <Alert
        type="info"
        showIcon
        style={{ marginBottom: 16 }}
        message="Read-only view"
        description="Editing provider configuration at runtime is not yet supported. Set env vars in the server's .env / deployment manifest and restart."
      />
      <Table dataSource={PROVIDERS} columns={columns} pagination={false} size="small" />
    </div>
  );
}

export default ProviderConfigsPage;
