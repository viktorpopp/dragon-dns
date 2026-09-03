use crate::{app::App, config::RuntimeConfig};
use color_eyre::Result;
use std::{env, path::Path};

mod app;
mod config;
mod error;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    if Path::new(".env").exists() {
        dotenvy::dotenv()?;
    }
    tracing_subscriber::fmt()
        .with_env_filter(format!(
            "{},rustls_platform_verifier=info,reqwest=info,hyper_util=info,h2=info",
            env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string())
        ))
        .init();

    let config = RuntimeConfig::try_from_env()?;
    let mut app = App::new(config)?;

    app.run().await
}
