//! Public AI-friendly capability discovery endpoint.
//!
//! `GET /api/capabilities` returns a structured JSON document describing
//! every upstream provider this server proxies: route layout, sample URLs,
//! per-provider 24h health stats, and an AI-targeted integration hint.
//!
//! The module is split into:
//!
//! * [`inventory`] — static `ProviderInfo` table (33+ providers, endpoints,
//!   short summaries and AI hints).
//! * [`handler`]   — request handler + 30s in-memory response cache + the
//!   24h stats aggregation that fans out per-provider `availability`.
//!
//! The route is wired in [`crate::routes::api_routes`] outside `service_auth`
//! and added to `record_metrics::SKIP_PREFIXES` so polling never pollutes
//! provider call stats.

pub mod inventory;
