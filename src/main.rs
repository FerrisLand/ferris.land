#![deny(clippy::pedantic, rust_2018_idioms)]
#![allow(clippy::module_name_repetitions)]

mod app;
mod config;

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    init_logger().map_err(ServerError::Log)?;
    let config = config::Config::load_from_env().map_err(|raw| ServerError::Env(raw))?;
    let state = app::AppState {};
    let router = app::router(state);
    bind(&config, router).await.map_err(ServerError::Bind)
}

fn init_logger() -> Result<(), LogError> {
    use std::str::FromStr;
    use tracing_subscriber::{
        filter::Targets, layer::SubscriberExt, util::SubscriberInitExt, Registry,
    };

    let value = std::env::var("RUST_LOG");
    let value = value.as_deref().unwrap_or("info");
    let filter = Targets::from_str(value).map_err(|raw| LogError { raw })?;
    let layer = tracing_forest::ForestLayer::default();
    Registry::default().with(filter).with(layer).init();
    Ok(())
}

#[derive(Debug, thiserror::Error)]
#[error("failed to parse `RUST_LOG` environent variable")]
struct LogError {
    #[source]
    raw: tracing_subscriber::filter::ParseError,
}

async fn bind(config: &config::Config, router: axum::Router) -> Result<(), BindError> {
    let addr = std::net::SocketAddr::new(config.server.host, config.server.port);
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .map_err(|raw| BindError { raw, addr })
}

#[derive(Debug, thiserror::Error)]
#[error("failed to bind server to `{addr}`")]
struct BindError {
    #[source]
    raw: hyper::Error,
    addr: std::net::SocketAddr,
}

#[derive(thiserror::Error)]
enum ServerError {
    #[error("failed to intitialize logger")]
    Log(#[source] LogError),
    #[error("failed to load environment variables")]
    Env(#[source] config::ParseEnvError),
    #[error("failed to bind server")]
    Bind(#[source] BindError),
}

// For some odd reason, errors are printed from the `Debug` impl, so we override
// it to print the recursive `std::error::Error::source` context.
//
// For example:
// ```
// Error: failed to intitialize logger
//
// Context:
//     0: failed to parse `RUST_LOG` environent variable
//     1: error parsing level filter: expected one of "off", "error", "warn",
//        "info", "debug", "trace", or a number 0-5
// ```
impl std::fmt::Debug for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::error::Error as _;
        write!(f, "{self}")?;
        if self.source().is_some() {
            f.write_str("\n\nContext:")?;
            write_source(f, self.source(), 0)?;
        }
        Ok(())
    }
}

fn write_source(
    f: &mut std::fmt::Formatter<'_>,
    error: Option<&(dyn std::error::Error + 'static)>,
    count: u8,
) -> std::fmt::Result {
    match error {
        Some(error) => {
            write!(f, "\n    {count}: {error}")?;
            write_source(f, error.source(), count + 1)
        }
        None => Ok(()),
    }
}
