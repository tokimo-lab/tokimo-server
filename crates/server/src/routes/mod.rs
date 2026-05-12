pub mod admin;
pub mod animetosho;
pub mod assrt;
pub mod bangumi;
pub mod bing_wallpaper;
pub mod currency;
pub mod deezer;
pub mod douban;
pub mod fanart;
pub mod geocoding;
pub mod gestdown;
pub mod github;
pub mod hitokoto;
pub mod holiday;
pub mod hot;
pub mod itunes;
pub mod javbus;
pub mod javdb;
pub mod lrclib;
pub mod musicbrainz;
pub mod nominatim;
pub mod omdb;
pub mod openmeteo;
pub mod opensubtitles;
pub mod qidian;
pub mod regielive;
pub mod shooter;
pub mod sports;
pub mod spotify;
pub mod stashdb;
pub mod thetvdb;
pub mod tmdb;
pub mod tpdb;
pub mod wikipedia;
pub mod zenquotes;

use axum::{http::StatusCode, middleware, routing::get, Router};
use tower_http::compression::CompressionLayer;

use crate::{
    capabilities::capabilities_handler,
    middleware::{admin_auth, cache_headers, record_metrics, service_auth},
    AppState,
};

pub fn api_routes(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/capabilities", get(capabilities_handler))
        .nest("/admin", admin::public_routes())
        .nest(
            "/admin",
            admin::protected_routes().route_layer(middleware::from_fn_with_state(state.clone(), admin_auth)),
        )
        .nest("/tmdb", provider_routes(tmdb::routes(), &state))
        .nest("/omdb", provider_routes(omdb::routes(), &state))
        .nest("/thetvdb", provider_routes(thetvdb::routes(), &state))
        .nest("/bangumi", provider_routes(bangumi::routes(), &state))
        .nest("/currency", provider_routes(currency::routes(), &state))
        .nest("/fanart", provider_routes(fanart::routes(), &state))
        .nest("/douban", provider_routes(douban::routes(), &state))
        .nest("/javbus", provider_routes(javbus::routes(), &state))
        .nest("/javdb", provider_routes(javdb::routes(), &state))
        .nest("/tpdb", provider_routes(tpdb::routes(), &state))
        .nest("/stashdb", provider_routes(stashdb::routes(), &state))
        .nest("/hot", provider_routes(hot::routes(), &state))
        .nest("/sports", provider_routes(sports::routes(), &state))
        .nest("/spotify", provider_routes(spotify::routes(), &state))
        .nest("/musicbrainz", provider_routes(musicbrainz::routes(), &state))
        .nest("/deezer", provider_routes(deezer::routes(), &state))
        .nest("/lrclib", provider_routes(lrclib::routes(), &state))
        .nest("/openmeteo", provider_routes(openmeteo::routes(), &state))
        .nest("/nominatim", provider_routes(nominatim::routes(), &state))
        .nest("/qidian", provider_routes(qidian::routes(), &state))
        .nest("/wikipedia", provider_routes(wikipedia::routes(), &state))
        .nest("/holiday", provider_routes(holiday::routes(), &state))
        .nest("/geocoding", provider_routes(geocoding::routes(), &state))
        .nest("/assrt", provider_routes(assrt::routes(), &state))
        .nest("/github", provider_routes(github::routes(), &state))
        .nest("/hitokoto", provider_routes(hitokoto::routes(), &state))
        .nest("/itunes", provider_routes(itunes::routes(), &state))
        .nest("/zenquotes", provider_routes(zenquotes::routes(), &state))
        .nest("/bing", provider_routes(bing_wallpaper::routes(), &state))
        .nest("/opensubtitles", provider_routes(opensubtitles::routes(), &state))
        .nest("/regielive", provider_routes(regielive::routes(), &state))
        .nest("/gestdown", provider_routes(gestdown::routes(), &state))
        .nest("/shooter", provider_routes(shooter::routes(), &state))
        .nest("/animetosho", provider_routes(animetosho::routes(), &state))
        .fallback(api_not_found)
        .layer(middleware::from_fn(cache_headers))
        .layer(CompressionLayer::new())
        .layer(middleware::from_fn_with_state(state.clone(), record_metrics))
        .with_state(state)
}

async fn api_not_found() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}

fn provider_routes(routes: Router<AppState>, state: &AppState) -> Router<AppState> {
    routes.route_layer(middleware::from_fn_with_state(state.clone(), service_auth))
}

async fn health() -> &'static str {
    "OK"
}
