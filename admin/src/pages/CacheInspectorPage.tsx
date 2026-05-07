import { Alert, List, Tag, Typography } from "antd";
import { useTranslation } from "react-i18next";

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
  const { t } = useTranslation();

  return (
    <div>
      <Title level={3}>{t("cache.title")}</Title>
      <Alert
        type="warning"
        showIcon
        className="tks-admin-section-gap"
        message={t("cache.comingSoonTitle")}
        description={
          <>
            {t("cache.comingSoonDescriptionPrefix")}
            <code>/api/admin/cache</code>
            {t("cache.comingSoonDescriptionMiddle")}
            <code>GET /api/admin/cache/:table?limit=50</code>
            {t("cache.comingSoonDescriptionAnd")}
            <code>POST /api/admin/cache/:table/:id/invalidate</code>
            {t("cache.comingSoonDescriptionSuffix")}
            <code>id · fetched_at · size_estimate</code>
            {t("cache.comingSoonDescriptionTail")}
          </>
        }
      />
      <Paragraph>{t("cache.tablesIntro")}</Paragraph>
      <List
        size="small"
        bordered
        dataSource={CACHE_TABLES}
        renderItem={(item) => (
          <List.Item>
            <code>{item.table}</code>
            <Tag className="tks-admin-inline-gap-left">{item.provider}</Tag>
          </List.Item>
        )}
      />
    </div>
  );
}

export default CacheInspectorPage;
