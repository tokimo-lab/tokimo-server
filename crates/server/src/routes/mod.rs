pub mod admin;
pub mod hot;
pub mod sports;
pub mod tmdb;

use axum::{middleware, routing::get, Router};

use crate::{
    middleware::{admin_auth, service_auth},
    AppState,
};

pub fn api_routes(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .nest(
            "/admin",
            admin::routes().route_layer(middleware::from_fn_with_state(state.clone(), admin_auth)),
        )
        .nest(
            "/tmdb",
            tmdb::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/hot",
            hot::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .nest(
            "/sports",
            sports::routes().route_layer(middleware::from_fn_with_state(state.clone(), service_auth)),
        )
        .with_state(state)
}

async fn health() -> &'static str {
    "OK"
}
