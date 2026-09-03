use crate::{
    config::RuntimeConfig,
    error::AppError,
    utils::{format_duration, get_ip4},
};
use chrono::{DateTime, Local, TimeDelta};
use cloudflare::{
    endpoints::{
        account::user::GetUserTokenStatus,
        dns::dns::{
            DnsContent, ListDnsRecords, ListDnsRecordsOrder, ListDnsRecordsParams, UpdateDnsRecord,
            UpdateDnsRecordParams,
        },
        zones::zone::{ListZones, ListZonesOrder, ListZonesParams},
    },
    framework::{
        Environment, OrderDirection,
        auth::Credentials,
        client::{ClientConfig as CloudflareConfig, async_api::Client as CloudflareClient},
    },
};
use color_eyre::Result;
use std::net::Ipv4Addr;
use tokio::time::sleep;
use tracing::{debug, info};

pub struct App {
    api_client: CloudflareClient,
    config: RuntimeConfig,

    cached_ip4: Ipv4Addr,
    cached_last_time: DateTime<Local>,
}

impl App {
    pub fn new(config: RuntimeConfig) -> Result<Self> {
        let credentials = Credentials::UserAuthToken {
            token: config.token.clone(),
        };

        Ok(Self {
            api_client: CloudflareClient::new(
                credentials,
                CloudflareConfig::default(),
                Environment::Production,
            )?,
            config,
            cached_ip4: Ipv4Addr::new(0, 0, 0, 0),
            cached_last_time: DateTime::default(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.verify_token().await?;

        info!("cache TTL: {}", format_duration(&self.config.cache_ttl));
        info!("machine identifier: {}", self.config.machine_id);
        info!("update cron: {}", self.config.update_cron);

        loop {
            let now = Local::now();
            let next = self.config.update_cron.find_next_occurrence(&now, false)?;

            let ip4 = get_ip4().await?;

            if ip4 != self.cached_ip4
                || now - self.cached_last_time > TimeDelta::from(self.config.cache_ttl)
            {
                tracing::info!("updating with the IP: {}", ip4);
                self.cached_ip4 = ip4;
                self.cached_last_time = now;
                self.update_records().await?;
            }
            sleep((next - now).to_std()?).await;
        }
    }

    async fn verify_token(&self) -> Result<()> {
        let endpoint = GetUserTokenStatus {};
        let res = self.api_client.request(&endpoint).await;

        match res {
            Ok(status) => match status.result.status.as_str() {
                "active" => {
                    info!("successfully verified API token");
                    Ok(())
                }
                "disabled" => Err(AppError::InvalidToken("disabled".into()))?,
                "expired" => Err(AppError::InvalidToken("expired".into()))?,
                _ => todo!(),
            },
            Err(_) => todo!(),
        }
    }

    async fn update_records(&self) -> Result<()> {
        // list all zones that we have access to
        let zones = {
            let res = self
                .api_client
                .request(&ListZones {
                    params: ListZonesParams {
                        name: None,
                        status: None,
                        page: None,
                        per_page: Some(50),
                        order: Some(ListZonesOrder::Name),
                        direction: Some(OrderDirection::Ascending),
                        search_match: None,
                    },
                })
                .await;
            match res {
                Ok(r) => r.result,
                _ => todo!(),
            }
        };

        for zone in zones {
            let records = {
                let res = self
                    .api_client
                    .request(&ListDnsRecords {
                        zone_identifier: &zone.id,
                        params: ListDnsRecordsParams {
                            record_type: None,
                            name: None,
                            page: None,
                            per_page: Some(500),
                            order: Some(ListDnsRecordsOrder::Name),
                            direction: Some(OrderDirection::Ascending),
                            search_match: None,
                        },
                    })
                    .await;
                match res {
                    Ok(r) => r.result,
                    _ => todo!(),
                }
            };

            for record in records {
                if !self.config.machine_id.is_empty()
                    && record
                        .comment
                        .clone()
                        .unwrap_or("".to_string())
                        .contains(format!("DDNS_ID={}", &self.config.machine_id).as_str())
                {
                    // Update the record

                    let res = self
                        .api_client
                        .request(&UpdateDnsRecord {
                            zone_identifier: &zone.id,
                            identifier: &record.id,
                            params: UpdateDnsRecordParams {
                                ttl: Some(record.ttl),
                                proxied: Some(record.proxied),
                                name: &record.name,
                                content: DnsContent::A {
                                    content: self.cached_ip4,
                                },
                                comment: record.comment.as_deref(),
                                tags: Some(&record.tags),
                            },
                        })
                        .await;
                    match res {
                        Ok(_) => {
                            debug!("updated record {}", record.name);
                        }
                        _ => todo!(),
                    }
                }
            }
        }

        Ok(())
    }
}
