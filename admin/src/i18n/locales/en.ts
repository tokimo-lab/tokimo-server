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
    appTitle: "Tokimo Server Admin",
    serviceKeys: "Service Keys",
    providers: "Provider Configs",
    cache: "Cache Inspector",
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
  cache: {
    title: "Cache Inspector",
    comingSoonTitle: "Coming soon",
    comingSoonDescriptionPrefix: "The admin ",
    comingSoonDescriptionMiddle:
      " endpoint is currently a stub returning an empty list. Once a per-table inspect endpoint lands (planned: ",
    comingSoonDescriptionAnd: " and ",
    comingSoonDescriptionSuffix: "), this page will render last-N rows with ",
    comingSoonDescriptionTail: ' plus a "force refresh" action.',
    tablesIntro:
      "Provider cache tables currently persisted by the workspace (one row per upstream resource, plus a TTL column for expiry):",
  },
};

export default en;
export type Resources = typeof en;
