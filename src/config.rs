use chrono::Duration;
use color_eyre::Result;
use croner::Cron;
use serde::Deserialize;
use tracing::debug;

#[derive(Debug, Deserialize)]
pub struct RuntimeConfig {
    pub token: String,
    pub machine_id: String,
    #[serde(default = "defaults::cache_ttl")]
    pub cache_ttl: Duration,
    #[serde(default = "defaults::update_cron")]
    pub update_cron: Cron,
}

impl RuntimeConfig {
    pub fn try_from_env() -> Result<Self> {
        debug!("loading runtime configuration");
        // TODO: Improve error message for missing value.
        Ok(envy::from_env::<RuntimeConfig>()?)
    }
}

mod defaults {
    use chrono::Duration;
    use croner::Cron;
    use std::str::FromStr;
    use tracing::trace;

    pub fn cache_ttl() -> Duration {
        trace!("using default cache TTL of 6 hours");
        Duration::hours(6)
    }

    pub fn update_cron() -> Cron {
        trace!("using default update cron of `* * * * *`");
        // TODO: Don't use `unwrap`.
        Cron::from_str("* * * * *").unwrap()
    }
}
