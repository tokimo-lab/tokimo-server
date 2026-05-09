use anyhow::Context;

#[derive(Clone)]
pub struct AppConfig {
    pub listen: String,
    pub public_base_url: String,
    pub admin_bootstrap_key: String,
    pub jwt_secret: String,
    pub tmdb_api_key: Option<String>,
    pub omdb_api_key: Option<String>,
    pub thetvdb_api_key: Option<String>,
    pub bangumi_user_agent: Option<String>,
    pub fanart_api_key: Option<String>,
    pub spotify_client_id: Option<String>,
    pub spotify_client_secret: Option<String>,
    pub musicbrainz_user_agent: Option<String>,
    pub nominatim_user_agent: Option<String>,
    pub assrt_api_key: Option<String>,
    pub opensubtitles_api_key: Option<String>,
    pub github_token: Option<String>,
    pub javbus_base_url: Option<String>,
    pub javbus_cookie: Option<String>,
    pub javdb_base_url: Option<String>,
    pub javdb_cookie: Option<String>,
    pub tpdb_api_key: Option<String>,
    pub tpdb_base_url: Option<String>,
    pub stashdb_api_key: Option<String>,
    pub stashdb_base_url: Option<String>,
    pub cors_allowed_origins: Vec<String>,
    pub storage_backend: String,
    pub storage_local_root: String,
    pub database_url: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            listen: std::env::var("SERVER_LISTEN").unwrap_or_else(|_| "0.0.0.0:5680".into()),
            public_base_url: std::env::var("SERVER_PUBLIC_BASE_URL").unwrap_or_else(|_| "http://localhost:5680".into()),
            admin_bootstrap_key: std::env::var("SERVER_ADMIN_BOOTSTRAP_KEY")
                .context("SERVER_ADMIN_BOOTSTRAP_KEY not set")?,
            jwt_secret: std::env::var("SERVER_JWT_SECRET").context("SERVER_JWT_SECRET not set")?,
            tmdb_api_key: std::env::var("TMDB_API_KEY").ok(),
            omdb_api_key: std::env::var("OMDB_API_KEY").ok(),
            thetvdb_api_key: std::env::var("THETVDB_API_KEY").ok(),
            bangumi_user_agent: std::env::var("BANGUMI_USER_AGENT").ok(),
            fanart_api_key: std::env::var("FANART_API_KEY").ok(),
            spotify_client_id: std::env::var("SPOTIFY_CLIENT_ID").ok(),
            spotify_client_secret: std::env::var("SPOTIFY_CLIENT_SECRET").ok(),
            musicbrainz_user_agent: std::env::var("MUSICBRAINZ_USER_AGENT").ok(),
            nominatim_user_agent: std::env::var("NOMINATIM_USER_AGENT").ok(),
            assrt_api_key: std::env::var("ASSRT_API_KEY").ok(),
            opensubtitles_api_key: std::env::var("OPENSUBTITLES_API_KEY").ok(),
            github_token: std::env::var("GITHUB_TOKEN").ok(),
            javbus_base_url: std::env::var("JAVBUS_BASE_URL").ok(),
            javbus_cookie: std::env::var("JAVBUS_COOKIE").ok(),
            javdb_base_url: std::env::var("JAVDB_BASE_URL").ok(),
            javdb_cookie: std::env::var("JAVDB_COOKIE").ok(),
            tpdb_api_key: std::env::var("TPDB_API_KEY").ok(),
            tpdb_base_url: std::env::var("TPDB_BASE_URL").ok(),
            stashdb_api_key: std::env::var("STASHDB_API_KEY").ok(),
            stashdb_base_url: std::env::var("STASHDB_BASE_URL").ok(),
            cors_allowed_origins: std::env::var("SERVER_CORS_ALLOWED_ORIGINS")
                .unwrap_or_default()
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect(),
            storage_backend: std::env::var("STORAGE_BACKEND").unwrap_or_else(|_| "local".into()),
            storage_local_root: std::env::var("STORAGE_LOCAL_ROOT").unwrap_or_else(|_| "./storage".into()),
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL not set")?,
        })
    }
}
