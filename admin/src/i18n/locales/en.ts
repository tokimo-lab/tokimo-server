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
      "Static view of the {{count}} provider adapters wired into this server. Auth env vars are read from the server process environment at startup; live status of which env vars are actually populated is not surfaced here to avoid leaking secret presence.",
    readOnlyTitle: "Read-only view",
    readOnlyDescription:
      "Editing provider configuration at runtime is not yet supported. Set env vars in the server's .env / deployment manifest and restart.",
    columns: {
      provider: "Provider",
      prefix: "Endpoint Prefix",
      rateLimit: "Rate Limit",
      auth: "Auth",
      envVars: "Env Vars",
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
    dashboard: {
      title: "Dashboard",
      summary:
        "Live operational view of the upstream proxy: aggregated request volume, error rates, latency percentiles and provider breakdown over the selected time range.",
      sections: {
        overview: {
          title: "Overview",
          body: "The dashboard aggregates metrics across **all configured providers** and renders them as time series, heatmaps and rings. Data is pulled from `dashboard_*` endpoints and cached on the client for 15s. Use the range switch to scope every panel simultaneously — there is no per-card time control by design.",
        },
        metrics: {
          title: "Metrics",
          body: "Each chart maps directly to a backend rollup:\n\n- **Activity Ring** — success vs error count for the range\n- **Timeseries** — bucketed request volume\n- **Latency** — p50 / p95 / p99 derived from per-request samples\n- **Heatmap** — hour × day error density\n- **Provider column** — top-N providers by traffic\n\nAll metrics are computed by the server; the admin UI never re-aggregates raw rows.",
        },
        refresh: {
          title: "Refresh & cache",
          body: "React Query polls each panel every 30s with `staleTime` 15s. The **Refresh** button forces a `refetch` on every visible card. There is also a hidden debug action (5 fast clicks on the title) that lets ops clear the rollup window — use with care, this resets metrics persistently.",
        },
      },
      fields: {
        range: {
          label: "range",
          desc: "Selected time window. Drives the `range` query parameter sent to every dashboard endpoint. Allowed values: `1h`, `24h`, `7d`.",
        },
        interval: {
          label: "bucket",
          desc: "Server-side bucket size (seconds) inferred from the range. Not user-controllable; charts read this from the API response so x-axis ticks always align.",
        },
      },
    },
    "provider-configs": {
      title: "Provider Configs",
      summary:
        "Read-only inventory of upstream API providers wired into the proxy. Lists routing status, observed traffic and a sample request URL for each provider.",
      sections: {
        overview: {
          title: "Overview",
          body: "Provider definitions live in code and configuration — they cannot be edited from this page. The table is a **runtime mirror**: it shows whatever the server reports as currently registered, plus 24h traffic counters scraped from the metrics rollup.",
        },
        "sample-url": {
          title: "Sample URL",
          body: "Click a row to open the response inspector. The modal replays the **last successful upstream call** (or the most recent failed one, if none succeeded) so you can verify auth headers, response shape, and rate-limit fields without touching production.",
        },
      },
      fields: {
        provider: {
          label: "provider",
          desc: "Internal provider id (e.g. `tmdb`, `omdb`). Maps to the route prefix `/providers/{id}/...`.",
        },
        status: {
          label: "status",
          desc: "Routing health derived from the last 5 minutes of traffic. `healthy` = success ratio > 95%; `degraded` = 50–95%; `down` = < 50% or no traffic.",
        },
        "24h_calls": {
          label: "24h calls",
          desc: "Total requests routed to this provider in the last 24 hours, including cache hits and misses.",
        },
        hit_ratio: {
          label: "hit ratio",
          desc: "Cache hit percentage in the last 24 hours. Low values usually mean either cold cache or aggressive TTL — cross-check with the Cache Inspector.",
        },
      },
    },
    "service-keys": {
      title: "Service Keys",
      summary:
        "Issue, view and revoke long-lived API tokens used by downstream services to call this proxy. Each key carries a fixed scope set and an immutable creation timestamp.",
      sections: {
        overview: {
          title: "Overview",
          body: "Service keys are JWT-style bearer tokens minted by the admin. They are intended for **server-to-server traffic only**; do not embed them in browser apps. Once a key is created, the raw token is shown **exactly once** — copy it immediately or rotate.",
        },
        "token-format": {
          title: "Token format",
          body: "Tokens are signed with the server's HMAC-SHA256 key and use the format `tks_<id>.<sig>`. The `id` segment is the database primary key; the `sig` segment is the HMAC over `{id, scopes, created_at}`. Verification is constant-time.",
        },
        scopes: {
          title: "Scopes",
          body: "Every key carries an explicit allowlist of scopes (e.g. `cache:read`, `dashboard:read`, `providers:write`). Scopes are checked at the route guard layer; an empty scope set yields a key that can authenticate but cannot access any resource.",
        },
      },
      fields: {
        token: {
          label: "token",
          desc: "Raw bearer string. Only the prefix `tks_<id>` is persisted in the table — the secret half is never stored, so a lost token must be re-issued.",
        },
        created_at: {
          label: "created_at",
          desc: "UTC timestamp of issuance. Used for audit and (optionally) for time-bounded expiry policies.",
        },
        scopes: {
          label: "scopes",
          desc: "Comma-separated permission set. Read by the auth middleware on every request; mismatched scopes return `403 forbidden`.",
        },
      },
    },
    "cache-inspector": {
      title: "Cache Inspector",
      summary:
        "Inspect the per-provider response cache stored in PostgreSQL. Browse rows, preview raw bodies, force-expire stale entries and delete individual rows for debugging.",
      sections: {
        overview: {
          title: "Overview",
          body: "Each provider has a dedicated cache table named `cache_<provider>`. Rows are keyed by the canonicalised request URL plus query string. The inspector paginates rows server-side at 50 per page and never loads the raw body until you open the preview modal.",
        },
        ttl: {
          title: "TTL",
          body: "TTL is **per-provider**, configured at deploy time. The `TTL` column shows the remaining seconds until the row is considered stale. Stale rows are still served if the upstream call fails, so the cache also acts as a soft fallback.",
        },
        operations: {
          title: "Operations",
          body: "Available actions per row:\n\n- **Refresh** — set `fetched_at` far enough in the past to force the next request to re-fetch upstream\n- **Delete** — drop the row entirely\n- **Preview** — view the cached raw response body in a modal\n\nAll three operations write an audit entry; deletes are not recoverable.",
        },
      },
      fields: {
        fetched_at: {
          label: "fetched_at",
          desc: "UTC timestamp at which the upstream response was written into the cache. The cache age column is computed as `now() - fetched_at`.",
        },
        ttl_seconds: {
          label: "ttl_seconds",
          desc: "Per-row remaining time-to-live in seconds. Negative values indicate a stale row that will be re-fetched on the next miss.",
        },
        key: {
          label: "key",
          desc: "Canonical request key. Built from the provider id, route, and query parameters; case-normalised so equivalent requests collapse to the same row.",
        },
      },
    },
  },
};

export default en;
export type Resources = typeof en;
