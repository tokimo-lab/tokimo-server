import { Alert, List, Tag } from "antd";
import { useTranslation } from "react-i18next";

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
    <div className="space-y-6">
      <div>
        <h1 className="mb-1 text-2xl font-semibold text-fg-light dark:text-fg-dark">
          {t("cache.title")}
        </h1>
        <p className="text-sm text-fg-muted-light dark:text-fg-muted-dark">
          {t("cache.tablesIntro")}
        </p>
      </div>
      <Alert
        type="warning"
        showIcon
        message={t("cache.comingSoonTitle")}
        description={
          <div className="space-y-2">
            <div>
              {t("cache.comingSoonDescriptionPrefix")}
              <code className="mx-1 rounded bg-fill-tertiary-light px-1.5 py-0.5 text-xs dark:bg-fill-tertiary-dark">
                /api/admin/cache
              </code>
              {t("cache.comingSoonDescriptionMiddle")}
            </div>
            <div className="space-y-1">
              <code className="block break-all rounded bg-fill-tertiary-light px-2 py-1 text-xs dark:bg-fill-tertiary-dark">
                GET /api/admin/cache/:table?limit=50
              </code>
              {t("cache.comingSoonDescriptionAnd")}
              <code className="block break-all rounded bg-fill-tertiary-light px-2 py-1 text-xs dark:bg-fill-tertiary-dark">
                POST /api/admin/cache/:table/:id/invalidate
              </code>
            </div>
            <div>
              {t("cache.comingSoonDescriptionSuffix")}
              <code className="mx-1 rounded bg-fill-tertiary-light px-1.5 py-0.5 text-xs dark:bg-fill-tertiary-dark">
                id · fetched_at · size_estimate
              </code>
              {t("cache.comingSoonDescriptionTail")}
            </div>
          </div>
        }
      />
      <List
        bordered
        dataSource={CACHE_TABLES}
        className="rounded-md [&_.ant-list-item]:py-3 [&_.ant-list-item]:transition-colors [&_.ant-list-item:hover]:bg-fill-tertiary-light dark:[&_.ant-list-item:hover]:bg-fill-tertiary-dark"
        renderItem={(item) => (
          <List.Item className="flex items-center justify-between">
            <code className="font-mono text-sm font-medium">{item.table}</code>
            <Tag>{item.provider}</Tag>
          </List.Item>
        )}
      />
    </div>
  );
}

export default CacheInspectorPage;
