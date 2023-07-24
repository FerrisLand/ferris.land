use std::{net::IpAddr, str::FromStr};

#[derive(Debug)]
pub struct Config {
    pub server: Server,
}

impl Config {
    pub fn load_from_env() -> Result<Self, ParseEnvError> {
        dotenvy::dotenv().ok();
        let config = Self {
            server: Server::load_from_env()?,
        };
        tracing::info!("loaded config:\n{config:#?}");
        Ok(config)
    }
}

#[derive(Debug)]
pub struct Server {
    pub host: IpAddr,
    pub port: u16,
}

impl Server {
    fn load_from_env() -> Result<Self, ParseEnvError> {
        let host = var("FERRISLAND_SERVER_HOST", "0.0.0.0")?;
        let port = var("FERRISLAND_SERVER_PORT", "3000")?;
        Ok(Server { host, port })
    }
}

fn var<T: FromStr>(name: &'static str, default: &'static str) -> Result<T, ParseEnvError> {
    let value = std::env::var(name).unwrap_or(default.to_owned());
    value.parse().map_err(|_| ParseEnvError {
        var: name,
        expected: std::any::type_name::<T>(),
        value,
    })
}

#[derive(Debug, thiserror::Error)]
#[error("failed to parse environment variable `{var}` (value: `{value}`) into `{expected}`")]
pub struct ParseEnvError {
    var: &'static str,
    expected: &'static str,
    value: String,
}
