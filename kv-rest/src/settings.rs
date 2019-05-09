use config::ConfigError;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub host: String,
    pub port: u16,
}

impl Settings {
    pub fn new() -> Result<Settings, ConfigError> {
        let mut s = config::Config::new();

        s.set_default("host", "127.0.0.1");
        s.set_default("port", "1111");
        s.merge(config::Environment::with_prefix("KV"));

        s.try_into()
    }
}
