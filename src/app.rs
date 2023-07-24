use axum::routing;

#[derive(Clone)]
pub struct AppState {}

pub fn router(state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/", routing::get(root))
        .with_state(state)
}

#[tracing::instrument(level = "debug")]
async fn root() -> &'static str {
    "hello ferris.land!"
}
