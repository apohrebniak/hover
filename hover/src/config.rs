use config::{ConfigError, FileFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DiscoveryConfig {
    pub multicast_group: String,
    pub multicast_port: u16,
    pub fanout: u32,
    pub rate_ms: u64,
    pub probe_timeout_ms: u64,
    pub probe_req_timeout_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BroadcastConfig {
    pub fanout: u32,
    pub rate_ms: u64,
    pub message_keep: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HoverConfig {
    pub address: String,
    pub port: u16,
    pub discovery: DiscoveryConfig,
    pub broadcast: BroadcastConfig,
}

impl HoverConfig {
    pub fn default() -> Result<HoverConfig, ConfigError> {
        let mut conf = config::Config::default();

        apply_default(&mut conf);
        conf.merge(config::Environment::with_prefix("hover"))?;
        conf.try_into()
    }

    pub fn from_file(path: &str) -> Result<HoverConfig, ConfigError> {
        let mut conf = config::Config::default();

        apply_default(&mut conf);
        conf.merge(config::File::new(path, config::FileFormat::Yaml))?;
        conf.merge(config::Environment::with_prefix("hover"))?;
        conf.try_into()
    }
}

fn apply_default(conf: &mut config::Config) {
    conf.set_default("address", "127.0.0.1").unwrap();
    conf.set_default("port", "6202").unwrap();
    conf.set_default("discovery.multicast_group", "228.0.0.1")
        .unwrap();
    conf.set_default("discovery.multicast_port", "2403")
        .unwrap();
    conf.set_default("discovery.fanout", "2").unwrap();
    conf.set_default("discovery.rate_ms", "500").unwrap();
    conf.set_default("discovery.probe_timeout_ms", "500")
        .unwrap();
    conf.set_default("discovery.probe_req_timeout_ms", "700")
        .unwrap();
    conf.set_default("broadcast.fanout", "2").unwrap();
    conf.set_default("broadcast.rate_ms", "500").unwrap();
    conf.set_default("broadcast.message_keep", "500").unwrap();
}
