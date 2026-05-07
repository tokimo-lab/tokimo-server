import { Alert, List, Tag, Typography } from "antd";

const { Title, Paragraph } = Typography;

const CACHE_TABLES: Array<{ table: string; provider: string }> = [
  { table: "tmdb_movies", provider: "TMDB" },
  { table: "tmdb_tv", provider: "TMDB" },
  { table: "tmdb_seasons", provider: "TMDB" },
  { table: "tmdb_episodes", provider: "TMDB" },
  { table: "tmdb_persons", provider: "TMDB" },
  { table: "omdb_titles", provider: "OMDb" },
  { table: "thetvdb_entries", provider: "TheTVDB" },
  { table: "bangumi_subjects", provider: "Bangumi" },
  { table: "fanart_entries", provider: "Fanart" },
  { table: "douban_entries", provider: "Douban" },
  { table: "spotify_entries", provider: "Spotify" },
  { table: "musicbrainz_entries", provider: "MusicBrainz" },
  { table: "deezer_entries", provider: "Deezer" },
  { table: "lrclib_entries", provider: "LRCLIB" },
  { table: "qidian_entries", provider: "Qidian" },
  { table: "wikipedia_entries", provider: "Wikipedia" },
  { table: "openmeteo_entries", provider: "Open-Meteo" },
  { table: "nominatim_entries", provider: "Nominatim" },
  { table: "holiday_entries", provider: "Holiday" },
  { table: "assrt_entries", provider: "Assrt" },
  { table: "github_releases_entries", provider: "GitHub Releases" },
];

function CacheInspectorPage() {
  return (
    <div>
      <Title level={3}>Cache Inspector</Title>
      <Alert
        type="warning"
        showIcon
        style={{ marginBottom: 16 }}
        message="Coming soon"
        description={
          <>
            The admin <code>/api/admin/cache</code> endpoint is currently a stub returning an empty list.
            Once a per-table inspect endpoint lands (planned: <code>GET /api/admin/cache/:table?limit=50</code>{" "}
            and <code>POST /api/admin/cache/:table/:id/invalidate</code>), this page will render last-N rows
            with <code>id · fetched_at · size_estimate</code> plus a "force refresh" action.
          </>
        }
      />
      <Paragraph>
        <strong>Provider cache tables</strong> currently persisted by the workspace (one row per upstream
        resource, plus a TTL column for expiry):
      </Paragraph>
      <List
        size="small"
        bordered
        dataSource={CACHE_TABLES}
        renderItem={(item) => (
          <List.Item>
            <code>{item.table}</code>
            <Tag style={{ marginLeft: 12 }}>{item.provider}</Tag>
          </List.Item>
        )}
      />
    </div>
  );
}

export default CacheInspectorPage;
