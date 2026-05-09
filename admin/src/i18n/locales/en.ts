const en = {
  common: {
    login: "Login",
    logout: "Logout",
    save: "Save",
    cancel: "Cancel",
    delete: "Delete",
    create: "Create",
    close: "Close",
    refresh: "Refresh",
    loading: "Loading...",
    error: "Error",
    success: "Success",
    yes: "Yes",
    no: "No",
    language: "Language",
  },
  nav: {
    dashboard: "Dashboard",
    keys: "Service Keys",
    providers: "Provider Configs",
    cache: "Cache Inspector",
    settings: "Settings",
    appTitle: "Tokimo Server Admin",
    serviceKeys: "Service Keys",
  },
  header: {
    logout: "Logout",
    theme: {
      light: "Light",
      dark: "Dark",
    },
    language: {
      zh: "中文",
      en: "English",
    },
  },
  login: {
    cardTitle: "Admin Login",
    bootstrapKeyLabel: "Bootstrap Key",
    bootstrapKeyRequired: "Please input bootstrap key",
    submit: "Login",
    success: "Login successful",
  },
  serviceKeys: {
    createBtn: "Create Service Key",
    modalTitle: "Create Service Key",
    tokenCreatedHint:
      "Token created successfully. Copy it now (it won't be shown again):",
    nameLabel: "Name",
    columns: {
      name: "Name",
      prefix: "Prefix",
      enabled: "Enabled",
      created: "Created",
      action: "Action",
    },
    toasts: {
      created: "Service key created",
      deleted: "Service key deleted",
    },
  },
  providers: {
    title: "Provider Configurations",
    description:
      "{{count}} provider adapters wired into this server. Edit cache TTL per provider; auth env vars are read from the server process environment at startup (only configured/missing status is surfaced, never the secret value).",
    loadError: "Failed to load providers",
    retry: "Retry",
    columns: {
      name: "Provider",
      category: "Category",
      prefix: "Endpoint Prefix",
      rateLimit: "Rate Limit",
      auth: "Auth",
      envVars: "Env Vars",
      ttl: "Cache TTL",
    },
    categories: {
      book: "Book",
      currency: "Currency",
      geo: "Geo",
      metadata: "Metadata",
      music: "Music",
      news: "News",
      quote: "Quote",
      sports: "Sports",
      subtitle: "Subtitle",
      tools: "Tools",
      wallpaper: "Wallpaper",
    },
    envStatus: {
      configured: "Configured",
      missing: "Not Set",
    },
    ttl: {
      edit: "Edit",
      save: "Save",
      cancel: "Cancel",
      seconds: "seconds",
      permanent: "Permanent",
      permanentHint: "This provider does not expose a configurable TTL.",
      updated: "TTL updated",
      updateFailed: "Failed to update TTL",
      zeroHint: "0 = no cache",
    },
    auth: {
      required: "required",
      optional: "optional",
      none: "none",
    },
    columns2: {
      sampleUrl: "Sample URL",
      action: "Action",
    },
    serviceKey: {
      label: "Service Key (Bearer)",
      placeholder: "tks_...",
      saved: "Saved to localStorage",
      missing: "Service key is empty — request will likely 401",
      copied: "URL copied",
      promptTitle: "Service Key required",
      promptDescription:
        "Enter your Bearer service key. It will be base64-encoded and persisted in localStorage; no server round-trip.",
      promptSubmit: "Save & Send",
      promptRequired: "Please input service key",
      clear: "Clear",
      cleared: "Service key cleared",
    },
    test: {
      sendBtn: "Send",
      modalTitle: "Response · {{provider}}",
      status: "Status",
      duration: "Duration",
      contentType: "Content-Type",
      body: "Body",
      sending: "Sending...",
      networkError: "Network error",
      copyResponse: "Copy response",
      copiedResponse: "Response copied",
    },

    tmdb: {
      name: "TMDB",
      description:
        "Movie and TV metadata from The Movie Database; requires configured API key.",
    },
    omdb: {
      name: "OMDb",
      description:
        "IMDb-linked movie metadata from the Open Movie Database; requires configured API key.",
    },
    thetvdb: {
      name: "TheTVDB",
      description:
        "Series and episode metadata from TheTVDB; requires configured API key.",
    },
    bangumi: {
      name: "Bangumi",
      description:
        "Anime and media metadata from Bangumi; requires configured user-agent.",
    },
    fanart: {
      name: "Fanart.tv",
      description:
        "Artwork assets for movies and shows from Fanart.tv; requires configured API key.",
    },
    douban: {
      name: "Douban",
      description:
        "Chinese movie and TV search metadata from Douban for localized cataloging.",
    },
    spotify: {
      name: "Spotify",
      description:
        "Track, album, and artist metadata from Spotify; requires configured API key.",
    },
    musicbrainz: {
      name: "MusicBrainz",
      description:
        "Open music metadata for artists, releases, and recordings; requires configured user-agent.",
    },
    deezer: {
      name: "Deezer",
      description:
        "Music search and track metadata from Deezer for cross-source enrichment.",
    },
    lrclib: {
      name: "LrcLib",
      description:
        "Lyrics lookup by artist and track from LrcLib for music subtitle scenarios.",
    },
    qidian: {
      name: "Qidian",
      description:
        "Online novel search metadata from Qidian for book discovery workflows.",
    },
    wikipedia: {
      name: "Wikipedia",
      description:
        "Article summary and encyclopedia context from Wikipedia by title and language.",
    },
    openmeteo: {
      name: "Open-Meteo",
      description:
        "Weather forecast data by coordinates from Open-Meteo for weather widgets.",
    },
    nominatim: {
      name: "Nominatim",
      description:
        "OpenStreetMap geocoding and place search via Nominatim; requires configured user-agent.",
    },
    geocoding: {
      name: "Geocoding",
      description:
        "Unified forward geocoding endpoint backed by OpenStreetMap data; requires configured user-agent.",
    },
    holiday: {
      name: "Holiday",
      description:
        "Public holiday calendars by country and year for planning and reminder features.",
    },
    assrt: {
      name: "ASSRT",
      description:
        "Chinese subtitle search from ASSRT; requires configured API key.",
    },
    opensubtitles: {
      name: "OpenSubtitles",
      description:
        "Subtitle search by IMDb ID and language from OpenSubtitles; requires configured API key.",
    },
    regielive: {
      name: "RegieLive",
      description:
        "Romanian subtitle search source for multilingual subtitle discovery.",
    },
    gestdown: {
      name: "Gestdown",
      description:
        "TV subtitle index and show lookup from Gestdown for subtitle sourcing.",
    },
    shooter: {
      name: "Shooter",
      description:
        "Hash-based Chinese subtitle matching from Shooter for local media files.",
    },
    animetosho: {
      name: "AnimeTosho",
      description:
        "Anime subtitle and release search from AnimeTosho for anime-focused workflows.",
    },
    hot: {
      name: "Hot List",
      description:
        "Aggregated real-time trending topics from supported platforms for news hotspots.",
    },
    sports: {
      name: "Sports Schedule",
      description:
        "Sports event schedules and hot matches for scoreboards and daily sports views.",
    },
    currency: {
      name: "Currency Rates",
      description:
        "Exchange-rate queries for target currencies for conversion and finance widgets.",
    },
    github: {
      name: "GitHub Releases",
      description:
        "Repository release metadata from GitHub; optional token improves rate limits.",
    },
    hitokoto: {
      name: "Hitokoto",
      description:
        "Random short quote sentences for daily inspiration and lightweight content cards.",
    },
    zenquotes: {
      name: "ZenQuotes",
      description:
        "Inspirational quote feed from ZenQuotes for quote and motivation displays.",
    },
    bing: {
      name: "Bing Wallpaper",
      description:
        "Daily wallpaper metadata from Bing for background and wallpaper features.",
    },
  },
  dashboard: {
    title: "Dashboard",
    retry: "Retry",
    loading: "Loading dashboard...",
    empty: "No data yet — call some providers and refresh",
    cards: {
      keys: "Total Service Keys",
      providers: "Active Providers",
      cacheEntries: "Cache Entries",
      calls24h: "24h Calls",
      errorRate: "Error rate {{rate}} ({{errors}}/{{calls}})",
    },
    subtitles: {
      active: "Active",
      configured: "Configured",
      totalRows: "Total Rows",
    },
    charts: {
      volume: "Request Volume",
      topProviders: "Top Providers",
      byProvider: "Provider Calls",
      recentErrors: "Recent Errors",
      calls: "Calls",
      errors: "Errors",
      cacheHits: "Cache Hits",
      cacheMisses: "Cache Misses",
      other: "Other",
      latency: "Latency p50 / p95",
      p50: "p50",
      p95: "p95",
      cacheHit: "Cache Hit",
      heatmap: "Provider × Time",
      errorsArea: "Errors Trend",
      statusCodes: "Status Codes",
      statusOk: "2xx",
      status4xx: "4xx",
      status5xx: "5xx",
      cacheTables: "Cache Tables",
      rows: "rows",
      avgTtl: "avg TTL",
      heroCalls: "Total calls in range",
      dragHint: "Drag handle to reorder",
    },
    range: {
      "1h": "1h",
      "24h": "24h",
      "7d": "7d",
    },
    refresh: {
      label: "Refresh",
      off: "Off",
      now: "Refresh now",
      interval: "Auto-refresh",
    },
    columns: {
      time: "Time",
      provider: "Provider",
      status: "Status",
      duration: "Duration",
    },
    relative: {
      justNow: "Just now",
      minutesAgo: "{{count}}m ago",
      hoursAgo: "{{count}}h ago",
      daysAgo: "{{count}}d ago",
    },
    units: {
      ms: "{{value}} ms",
    },
  },
  cache: {
    title: "Cache Inspector",
    description: "View / clear / force-expire provider cache entries",
    tablePlaceholder: "Select cache table",
    searchPlaceholder: "Search id, key, or preview",
    confirmDeleteTitle: "Delete this cache row?",
    previewHint:
      "This preview only contains the first 200 characters. Query the database directly to inspect the full cached payload.",
    previewModalTitle: "Preview first 200 chars",
    columns: {
      id: "ID",
      key: "Key",
      fetchedAt: "Fetched at",
      rawPreview: "Raw preview",
      operations: "Operations",
    },
    actions: {
      viewFull: "View full",
      expire: "Expire",
      delete: "Delete",
    },
    ttl: {
      average: "Average TTL remaining: {{value}}",
      expired: "Expired",
      empty: "No rows",
      days: "{{count}}d",
      hours: "{{count}}h",
      minutes: "{{count}}m",
      seconds: "{{count}}s",
    },
    relative: {
      justNow: "Just now",
      minutesAgo: "{{count}}m ago",
      hoursAgo: "{{count}}h ago",
      daysAgo: "{{count}}d ago",
    },
    toasts: {
      expired: "Cache row expired",
      deleted: "Cache row deleted",
    },
  },
  backdoor: {
    toast: "Tap {{remaining}} more time(s) to unlock metrics tools",
    title: "Clear metrics (Debug)",
    range: {
      "1h": "Last 1 hour",
      "24h": "Last 24 hours",
      "7d": "Last 7 days",
      all: "All",
      custom: "Custom",
    },
    confirm: "Clear",
    cancel: "Cancel",
    success: "Cleared {{count}} records",
  },
  docsHub: {
    title: "Docs Hub",
    fabTooltip: "Open Docs Hub (Cmd/Ctrl+/)",
    minimize: "Minimize",
    expand: "Expand",
    close: "Close",
    empty: "No documentation registered for the current page.",
    sectionsHeader: "Sections",
    fieldsHeader: "Fields",
    entryCount: "{{count}} entries",
  },
  docs: {
    "dashboard-overview": {
      title: "Dashboard · Top Bar Controls",
      summary:
        "Range / auto-refresh / manual refresh strip at the top — drives every chart on the page.",
      sections: {
        layout: {
          title: "Page layout",
          body: "Three layers: **global controls** at the top, **9 draggable chart cards** in the middle (3-column lg grid; Volume spans 2 columns), and a **Recent Errors** table at the bottom. Card order and the auto-refresh interval are persisted in localStorage (`tokimo-admin-dashboard-order-v1` / `-refresh-interval-v1`) and survive reloads.\n\n**Every chart is driven by the top-level range** — there is no per-card time control by design, so axes always line up. Card state machine: **loading** (skeleton) → **error** (with Retry) → **empty** → **rendered**.",
        },
        refresh: {
          title: "Refresh & cache",
          body: "**Auto-refresh** uses React Query's `refetchInterval`. Choices: 0 (Off) / 10s / 30s / 60s, default 30s.\n**Manual refresh**: the spinning circular button on the top-right calls `refetch()` on every visible card simultaneously and keeps spinning while any query is in flight.\n**staleTime**: dashboard queries leave React Query at default (0), so each `refetch` always hits the network.",
        },
        backdoor: {
          title: "Hidden debug entry",
          body: "**Five rapid clicks on the sidebar logo** open a hidden “Clear metrics” dialog (`backdoor.title`). It can wipe metric rollups by window (1h / 24h / 7d / all / custom). It **permanently resets metrics** — debugging only, not for casual use in production.",
        },
      },
      fields: {
        "control-range": {
          label: "Range (1h / 24h / 7d)",
          desc: "Selects the aggregation window for the entire page. **Bucket sizes are inferred**: 1h → 5-minute buckets; 24h → 1-hour buckets; 7d → 1-day buckets. The value is sent as `range_secs` and `bucket_secs` to every dashboard endpoint. Switching range cancels in-flight queries and re-fetches.",
        },
        "control-refresh-interval": {
          label: "Auto-refresh (Off / 10s / 30s / 60s)",
          desc: "Sets React Query's `refetchInterval`. `Off` means no polling — only range changes / manual refresh fetch. Persisted to localStorage `tokimo-admin-dashboard-refresh-interval-v1`.\n\nNote: auto-refetch only fires when the tab is foregrounded (React Query default `refetchIntervalInBackground: false`).",
        },
        "control-refresh-now": {
          label: "Refresh now (spinning icon button)",
          desc: "Forces every dashboard query on the page to `refetch()`, ignoring `staleTime`. The icon spins as long as any query is in `fetching` state.",
        },
      },
    },
    "dashboard-card-volume": {
      title: "Request Volume",
      summary: "Time-series line chart of calls vs errors.",
      fields: {
        chart: {
          label: "Request Volume",
          desc: "Time-series line chart, X = bucketed time, Y = request count. Two series: **calls** (success + error combined) and **errors**. The card title shows the **total calls in range**.\n\nSource: `GET /api/admin/dashboard/timeseries?range_secs=...&bucket_secs=...`. This card is 2 columns wide.",
        },
      },
    },
    "dashboard-card-cache-ring": {
      title: "Cache Hit Ring",
      summary: "Activity ring + central percentage of last-24h hit ratio.",
      fields: {
        chart: {
          label: "Cache Hit (activity ring)",
          desc: "Ring + central percentage showing the **last 24h** overall cache hit ratio (not affected by the top-level range). Value comes from `dashboard/overview.cache_hit_ratio_24h`, range 0..1.\n\n“Hit” is determined by the response header `x-cache: HIT`, stamped by the proxy when serving a cached response.",
        },
      },
    },
    "dashboard-card-top-providers": {
      title: "Top Providers Pie",
      summary: "Top-10 providers by call count.",
      fields: {
        chart: {
          label: "Top Providers (pie)",
          desc: "Pie chart of the top-N providers by call count. **N = 10**; everything beyond rank 10 is collapsed into an “Other” slice. We picked 10 (rather than 5) so the long tail isn't all dumped into Other and lose its shape.\n\nSource: `dashboard/by-provider`, sorted by `calls` descending within the selected range.",
        },
      },
    },
    "dashboard-card-by-provider": {
      title: "Provider Calls Column",
      summary: "Vertical bars of all providers' call counts.",
      fields: {
        chart: {
          label: "Provider Calls (column)",
          desc: "Vertical column chart listing **every** provider's total call count in the range. The card title shows the count of bars (= active providers). Shares the `dashboard/by-provider` payload with the pie but **does not truncate** — useful for spotting tail-end traffic.",
        },
      },
    },
    "dashboard-card-latency": {
      title: "Latency p50 / p95",
      summary: "Two latency percentile lines, ms.",
      fields: {
        chart: {
          label: "Latency p50 / p95",
          desc: "Two lines: **p50** (median) and **p95** (95th percentile), unit ms. Computed per bucket from per-request timing samples (success + error included). The card title shows the **latest bucket's p95**, the subtitle the latest bucket's p50.\n\nSource: same `dashboard/timeseries` payload (`p50_ms` / `p95_ms` per point).",
        },
      },
    },
    "dashboard-card-errors-area": {
      title: "Errors Trend Area",
      summary: "Area chart of errors over time.",
      fields: {
        chart: {
          label: "Errors Trend (area)",
          desc: "Area chart of `errors` only (4xx + 5xx combined) over time. Same series as the `errors` line in Volume — but isolated so an error spike's shape is obvious. The card title shows total errors in range.",
        },
      },
    },
    "dashboard-card-heatmap": {
      title: "Provider × Time Heatmap",
      summary: "2D intensity grid: provider × time bucket.",
      fields: {
        chart: {
          label: "Provider × Time (heatmap)",
          desc: "2D heatmap: Y axis = provider, X axis = time bucket, cell colour = call count for that (provider, bucket). Useful for spotting “provider X spiked at hour Y”.\n\nSource: `dashboard/heatmap?range_secs=...&bucket_secs=...`. Payload is `[{ ts, values: [{ provider, calls }] }]`.",
        },
      },
    },
    "dashboard-card-status-codes": {
      title: "Status Codes",
      summary: "Stacked bars of 2xx / 4xx / 5xx per bucket.",
      fields: {
        chart: {
          label: "Status Codes (stacked column)",
          desc: "Stacked bars per bucket with three segments: **2xx** (success, green) / **4xx** (client error, yellow) / **5xx** (server error, red). The card title shows the sum.\n\nSource: `dashboard/status-codes`. Note: upstream failures like 502/504 are recorded by the proxy as 5xx and merged with upstream-originated 5xx.",
        },
      },
    },
    "dashboard-card-cache-tables": {
      title: "Cache Tables List",
      summary: "Plain list of `cache_<provider>` tables, sorted by row count.",
      fields: {
        chart: {
          label: "Cache Tables (list)",
          desc: "Plain list (not a chart). Shows every `cache_<provider>` table sorted by row count, with row count and average remaining TTL. Clicking does **not** navigate — this is a status display; perform actions in the Cache Inspector page.\n\nSource: `/api/admin/cache/tables`, same as the table dropdown on the Cache Inspector page.",
        },
      },
    },
    "dashboard-recent-errors-table": {
      title: "Recent Errors Table",
      summary: "Bottom table: most recent failing requests, max 50.",
      fields: {
        table: {
          label: "Recent Errors (table)",
          desc: "Bottom table listing the most recent failing requests. **Hard-capped at 50 rows** (server-side), most recent first.\n\nColumns: **Time** (relative, e.g. “3m ago”) / **Provider** / **Status** (HTTP code, e.g. 502) / **Duration** (ms).\n\nSource: `dashboard/recent-errors`. Retention is governed by the metrics rollup config; older entries are pruned.",
        },
      },
    },
    "provider-configs-overview": {
      title: "Provider Configs · Overview",
      summary: "Page purpose, service-key workflow, security notes.",
      sections: {
        overview: {
          title: "Overview",
          body: "Read-only inventory of every upstream API provider wired into this proxy. The table is rendered from the front-end constant `PROVIDERS` (`admin/src/pages/ProviderConfigsPage.tsx`), kept in sync manually with backend route registrations.\n\n**Why read-only**: env vars used for upstream auth (e.g. `TMDB_API_KEY`) are read from the server process at startup. The admin deliberately does not expose whether a given env var is populated — that would leak a side-channel about secret presence. To change provider behaviour, edit `crates/providers/` and `.env`, then **restart the server**.",
        },
        "service-key": {
          title: "Service Key workflow",
          body: "The top input takes a Bearer service key (format `tks_xxx`). Its value is base64-encoded and stored in `localStorage['tokimo-admin-svc-key']`; **it is never sent to the admin server**. When you click Send on a row:\n\n1. If the key is empty → `ServiceKeyPromptModal` opens to collect one.\n2. The browser directly issues `fetch(sample, { Authorization: 'Bearer ' + key })`.\n3. The response is shown in `ProviderResponseModal`, **not routed through the admin backend**.\n\nWhy a separate key: the admin will not borrow your admin session's privileges to call provider APIs. You explicitly carry your own service-to-service token.",
        },
        security: {
          title: "Security notes",
          body: "- Service key is base64-encoded, **not encrypted** — anyone with access to the browser's localStorage can read it. Use only on trusted endpoints.\n- The sample request hits the upstream provider for real: it counts toward metrics, consumes API quota and writes the cache.\n- The Clear button only removes the local copy from localStorage. It does **not** revoke the key on the server (revoke from the Service Keys page).",
        },
      },
      fields: {
        "input-service-key": {
          label: "Service Key input",
          desc: "Accepts a `tks_xxx.<sig>` Bearer token. Value is base64-encoded into `localStorage['tokimo-admin-svc-key']` and re-loaded next visit. **Used only to fire test requests on this page** — admin's own auth runs on a separate cookie/JWT path and ignores this value.",
        },
        "action-clear-key": {
          label: "Clear button",
          desc: "Removes the locally cached service key from localStorage and empties the input. **Does not call the server. Does not revoke the key.** To revoke server-side go to the Service Keys page and delete the entry.",
        },
      },
    },
    "provider-configs-table": {
      title: "Providers Table",
      summary:
        "Live provider rows from admin API: metadata, env readiness, and test actions.",
      fields: {
        "column-name": {
          label: "Name",
          desc: "Provider display name resolved via i18n key (`i18n_name_key`) with fallback to provider `key`. Rows are loaded dynamically from `/api/admin/providers`, not from a static frontend constant.",
        },
        "column-category": {
          label: "Category",
          desc: "Logical provider group (e.g. movie, music, anime). Used to quickly scan capability domains; rendered from backend `category` and translated with `providers.categories.*` when available.",
        },
        "column-prefix": {
          label: "Prefix",
          desc: "Proxy route prefix for this provider (for example `/api/tmdb`). The cell uses tooltip + code style so long prefixes remain readable without widening the table.",
        },
        "column-rate-limit": {
          label: "Rate Limit",
          desc: "Current outbound throttle policy shown per provider (`rate_limit`, e.g. `10/s`). This value comes from backend provider metadata and helps explain `429` behavior during probe requests.",
        },
        "column-auth": {
          label: "Auth",
          desc: "Whether proxy calls require a Bearer service key (`yes | optional | no`). Rendered as colored tags to show strict/optional/open access at a glance.",
        },
        "column-env-vars": {
          label: "Env Vars",
          desc: "Required upstream env keys (`env_keys`) rendered as tags. Each tag color reflects runtime `env_status`: green = configured, gray = missing. `—` means no env dependency for that provider.",
        },
        "column-ttl": {
          label: "TTL",
          desc: "Provider cache TTL in seconds. If `has_ttl` is true, value is editable inline and saved via `PATCH /api/admin/providers/{key}`; if false, the table shows a Permanent tag with tooltip hint for non-expiring cache behavior.",
        },
        "column-sample-url": {
          label: "Sample URL",
          desc: "Probe URL template used by the Send action. The final request URL is produced by `expandSample()` (placeholder expansion such as `{TODAY}`), and full value is available in tooltip when truncated.",
        },
        "column-action-send": {
          label: "Action · Send",
          desc: "Sends a browser-side probe request for the row's sample URL. Uses the top service key input (prompts if empty) and opens `ProviderResponseModal` with status / latency / body for quick diagnostics.",
        },
      },
    },
    "provider-test-response-modal": {
      title: "Provider Response Modal",
      summary: "Modal triggered by the Send button.",
      sections: {
        overview: {
          title: "Overview",
          body: "Modal shows status / duration / content-type / body; JSON is auto pretty-printed and a Copy button is provided. On failure (CORS / network / timeout), `status` is `0` and the error message is shown separately.",
        },
      },
      fields: {
        "response-status": {
          label: "Status",
          desc: "HTTP status code shown at the top of the response modal. Common values: `200` ok / `401` invalid service key / `404` no such route / `429` rate-limited / `502` upstream env missing / `0` fetch threw (network / CORS / timeout).",
        },
        "response-duration": {
          label: "Duration",
          desc: "End-to-end latency measured by `performance.now()` (ms) — from `fetch()` start until the body finishes streaming. Includes browser → proxy → upstream → proxy → browser. **Not the same as upstream latency**; for that, see the dashboard's Latency card.",
        },
        "response-content-type": {
          label: "Content-Type",
          desc: "Forwarded from the proxy response. If the type is `application/json`-ish, the body is auto-pretty-printed (2-space indent); otherwise shown verbatim.",
        },
        "response-body": {
          label: "Body",
          desc: "Response payload. JSON is pretty-printed; non-JSON or parse failures are shown as raw text; on fetch errors the thrown error message is displayed instead. The Copy button writes the whole body to the clipboard.",
        },
      },
    },
    "service-keys-overview": {
      title: "Service Keys · Overview",
      summary:
        "Issue Bearer tokens for downstream services to call this proxy.",
      sections: {
        overview: {
          title: "Overview",
          body: "The table lists every issued service key (the plaintext is **not** stored — only the prefix). The “Create service key” button at the top opens a modal to mint a new one; on success the plaintext is rendered once in that same modal. **After you close it the plaintext is gone forever.**\n\nKeys are intended for **server-to-server traffic only** — never embed them in browser apps where end users could exfiltrate them.",
        },
        lifecycle: {
          title: "Lifecycle",
          body: "**Create**: admin POSTs to `/api/admin/service-keys`; the server signs `tks_<id>.<sig>` with HMAC-SHA256 and returns the plaintext **only in this response**. The DB only persists `id` and `token_prefix` for audit display — **never** the secret half, in any reversible form.\n\n**Revoke**: the Delete button calls `DELETE /api/admin/service-keys/{id}`. After deletion all subsequent requests using that key get 401; **in-flight requests are not aborted**.",
        },
        security: {
          title: "Security notes",
          body: "- The plaintext is shown once. Closing the modal = permanent loss; **there is no “re-reveal” button anywhere**.\n- The server uses constant-time comparison for verification, no length-leaks.\n- Per-key scope and TTL are **not yet implemented** — every issued key has full provider-call permissions. Issue conservatively.\n- `enabled = false` is a soft revoke supported by the backend, but the admin UI **has no toggle** today — revoke = delete.",
        },
      },
      fields: {
        "action-create": {
          label: "Create service key button",
          desc: "Opens the create-form modal. Currently the form **only requires `name`** — scope / TTL / notes are not yet implemented. After submission the freshly minted plaintext token is shown inline in the same modal — copy it immediately.",
        },
      },
    },
    "service-keys-table": {
      title: "Service Keys Table",
      summary: "Issued keys (prefix only, no plaintext).",
      fields: {
        "column-name": {
          label: "Name",
          desc: "Human-readable label set at creation (e.g. `media-server-prod`, `my-laptop-dev`). **Identification only**, no auth effect, may collide with another key. Recommend including an env suffix for audit clarity.",
        },
        "column-token-prefix": {
          label: "Prefix",
          desc: "The first few chars of the plaintext token (typically the `tks_<id>` portion). Used to identify **which key** in logs and lists. The full plaintext is never persisted — by design — so the table can only ever show the prefix.",
        },
        "column-enabled": {
          label: "Enabled",
          desc: "Whether the key is currently usable. Renders as `Yes` / `No`.\n\n**There is no toggle in the admin UI today** — every issued key defaults to `Yes`; the only way to disable is the Delete button (hard revoke). The backend supports `enabled = false` for soft revoke and a UI toggle is planned.",
        },
        "column-created": {
          label: "Created",
          desc: "UTC ISO-8601 timestamp of issuance. Used for audit; also part of the signed payload (HMAC over `{id, scopes, created_at}`), so it cannot be tampered with after the fact.",
        },
        "column-action-delete": {
          label: "Action · Delete",
          desc: "Hard-deletes the key. **Effect immediate, not recoverable.** Hits `DELETE /api/admin/service-keys/{id}` directly — no second confirmation. Downstream services will start getting 401 from the next request. To restore access you must re-issue and ship a new token.",
        },
      },
    },
    "service-key-create-modal": {
      title: "Create Service Key · Form",
      summary: "Single-field create form (name only).",
      fields: {
        "form-name": {
          label: "Name",
          desc: "Required. Any UTF-8 string; keep below 64 chars or the Prefix column display will be truncated. Persisted on submit and **cannot be renamed afterwards** — to rename you must delete and re-issue.",
        },
      },
    },
    "service-key-token-reveal-modal": {
      title: "New Token One-time Reveal",
      summary: "Plaintext token shown exactly once after creation.",
      sections: {
        warning: {
          title: "Important warning",
          body: "Plaintext is shown once on success. **Once you close the modal the server cannot reproduce it** — the sig segment was never persisted, and there is no “re-reveal” button anywhere. Copy before closing or you must re-issue.",
        },
      },
      fields: {
        "token-reveal": {
          label: "Plaintext textarea",
          desc: "Read-only textarea containing the full `tks_<id>.<sig>`. Use the browser's native select-and-copy.",
        },
      },
    },
    "cache-inspector-overview": {
      title: "Cache Inspector · Overview",
      summary: "Table picker / search / refresh toolbar; TTL & ops policy.",
      sections: {
        overview: {
          title: "Overview",
          body: "The top dropdown picks the **cache table** — one per provider, e.g. `cache_tmdb` / `cache_omdb`. The number in parentheses is the row count. Once a table is selected the paginated grid below loads the first 50 rows, sorted by `fetched_at` descending (newest first).\n\n**Functional v1**: every visible UI element on this page maps to a backend CRUD endpoint. Bulk expiration / pattern-based delete are not exposed in the UI — go straight to the database for those.",
        },
        ttl: {
          title: "TTL & expiry",
          body: "TTL is set explicitly by each provider when it writes the cache (no per-row config stored). The “avg remaining TTL” next to the dropdown is the mean across non-expired rows in that table.\n\n**Stale rows are not auto-deleted** — at query time, if `now() > fetched_at + ttl` the row is considered stale and the next request re-fetches upstream; if the upstream call fails, the stale row may still be served as a soft fallback (provider-specific).\n\nForce-expire pulls `fetched_at` far enough into the past that the next request is guaranteed to re-fetch.",
        },
        limitations: {
          title: "Known limitations",
          body: "- “View full” actually shows only the first 200 chars (same `raw_preview` field as the table). The label is mildly misleading — full bodies can be MB-sized and are not appropriate to ship over admin endpoints.\n- The search box is **client-side only** — it filters the currently loaded 50 rows by `id`/`key`/`raw_preview` substring, **not** the backend. Paginating clears the filter visually but state is reset.\n- There are no TTL colour tags. The grid only renders `fetched_at` plus a relative time subtitle; staleness is computed by you (or inferred from the avg-TTL display).",
        },
      },
      fields: {
        "selector-table": {
          label: "Cache table selector",
          desc: "Dropdown listing every `cache_*` table. Option label format `name (row_count)`, sourced from `GET /api/admin/cache/tables`. Switching resets pagination to page 1 and re-fetches rows.",
        },
        "avg-ttl-display": {
          label: "Average TTL display",
          desc: "Muted text next to the dropdown, e.g. “Average TTL remaining: 2d 5h”. Computed as `avg(ttl - (now() - fetched_at))` over non-expired rows.\n\nEmpty table → “No rows”. All-stale → “Expired”. At most 2 unit segments (d+h / h+m / m+s).",
        },
        "input-search": {
          label: "Search box",
          desc: "**Pure client-side filter** over the currently loaded 50 rows. Matches case-insensitively against `id`, `key` and `raw_preview` (substring on any). **Does not hit the backend** — paginating means re-typing. Useful for quickly locating a key inside the loaded window.",
        },
        "action-refresh-list": {
          label: "Refresh button",
          desc: "Re-fetches both the table list and the currently loaded page rows. Use when other writers may have changed the cache out of band. Shows a loading state while a fetch is in flight.",
        },
      },
    },
    "cache-entries-table": {
      title: "Cache Entries Table",
      summary:
        "Paginated rows of the selected cache table with three row actions.",
      sections: {
        operations: {
          title: "Operations & audit",
          body: "Three per-row actions:\n\n- **View full** — shows `raw_preview` (first 200 chars) in a modal. The full body is **not** in the list response, and the modal also shows only 200 chars — for the actual full body, query the DB directly (`SELECT raw FROM cache_<provider> WHERE id=...`).\n- **Expire** — hits `POST /api/admin/cache/{table}/{id}/refresh`, which rewinds `fetched_at` far into the past.\n- **Delete** — hits `DELETE /api/admin/cache/{table}/{id}`; the row vanishes (Popconfirm asks once).\n\nAll three write to the admin audit log. **Deletes are not recoverable.**",
        },
      },
      fields: {
        "column-id": {
          label: "ID",
          desc: "Cache row primary key. Pinned column. Typically a hash (truncated SHA256) or a provider-defined stable id. `fixed: left` keeps it visible during horizontal scroll.",
        },
        "column-key": {
          label: "Key",
          desc: "The canonicalised request key, built from provider id + route + query params with case normalisation (e.g. `tmdb:movie/550?language=zh-CN`). Equivalent requests collapse to the same row. Hover for the full content.",
        },
        "column-fetched-at": {
          label: "Fetched at",
          desc: "UTC time the upstream response was written to the cache. **Main line** is the absolute time `YYYY-MM-DD HH:mm:ss`; **subtitle** is relative (e.g. “3h ago”). Cache age = `now() - fetched_at`.",
        },
        "column-raw-preview": {
          label: "Raw preview",
          desc: "The first 200 chars of the cached body (already truncated server-side; the admin endpoint never ships the full body). Rendered with `<code>` + `line-clamp-2` so it stays compact in the grid; click View full for the modal.",
        },
        "column-operations": {
          label: "Operations",
          desc: "Right-pinned column with three buttons: **View full** / **Expire** / **Delete** (the last has a Popconfirm). See the “Operations & audit” section.",
        },
        "action-view-full": {
          label: "View full",
          desc: "Opens the preview modal showing `raw_preview` (first 200 chars). The modal carries a hint that the full body must be queried from the DB. **Not literally full** — the label is historical; future revision will rename to “Preview first 200 chars”.",
        },
        "action-expire": {
          label: "Expire",
          desc: "Hits `POST /api/admin/cache/{table}/{id}/refresh`; the server rewinds `fetched_at` to a very old timestamp (e.g. 1970). **The row is not deleted** — the next matching request misses, re-fetches and overwrites. Use when upstream has been updated but the cache TTL hasn't elapsed.",
        },
        "action-delete": {
          label: "Delete",
          desc: "Hard-deletes the row. Popconfirm asks once. **Not recoverable**: the next matching request cold-starts (and if upstream is down at that moment, soft-fallback has nothing to fall back to). Use only to clean up genuinely bad data.",
        },
      },
    },
    "cache-entry-preview-modal": {
      title: "Cache Preview Modal",
      summary: "Inspect a single row's `raw_preview` (first 200 chars).",
      fields: {
        "preview-modal": {
          label: "Preview modal",
          desc: "Width 720, shows the first 200 chars of `raw_preview`. Renders inside `<pre>` so newlines are preserved and long lines wrap. Max height 60vh, content scrolls. No copy button — use the browser's native select-and-copy.",
        },
      },
    },
  },
};

export default en;
export type Resources = typeof en;
