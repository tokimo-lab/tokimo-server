pub mod admin;
pub mod assrt;
pub mod bangumi;
pub mod deezer;
pub mod douban;
pub mod fanart;
pub mod geocoding;
pub mod github;
pub mod holiday;
pub mod hot;
pub mod lrclib;
pub mod musicbrainz;
pub mod nominatim;
pub mod omdb;
pub mod openmeteo;
pub mod qidian;
pub mod sports;
pub mod spotify;
pub mod thetvdb;
pub mod tmdb;
pub mod wikipedia;

use axum::{middleware, routing::get, Router};

use crate::{
    middleware::{admin_auth, service_auth},
    AppState,
};

pub fn api_routes(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .nest("/admin", admin::public_routes())
        .nest(
            "/admin",
            admin::protected_routes().route_layer(middleware::from_fn_with_state(state.clone(), admin_auth)),
        )
        .nest(
            "/tmdb",
            tmdb::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/omdb",
            omdb::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/thetvdb",
            thetvdb::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/bangumi",
            bangumi::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/fanart",
            fanart::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/douban",
            douban::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/hot",
            hot::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/sports",
            sports::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/spotify",
            spotify::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/musicbrainz",
            musicbrainz::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/deezer",
            deezer::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/lrclib",
            lrclib::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/openmeteo",
            openmeteo::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/nominatim",
            nominatim::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/qidian",
            qidian::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/wikipedia",
            wikipedia::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/holiday",
            holiday::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/geocoding",
            geocoding::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/assrt",
            assrt::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/github",
            github::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .with_state(state)
}

async fn health() -> &'static str {
    "OK"
}
