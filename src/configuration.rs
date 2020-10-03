use crate::ruby::RSpecConfiguration;
use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub include: Vec<String>,
    pub rspec: RSpecConfiguration,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            include: vec![],
            rspec: RSpecConfiguration::default(),
        }
    }
}

impl Configuration {
    pub fn read_configuration() -> Result<Self, ConfigError> {
        let mut config = Config::try_from(&Configuration::default())?;
        config.merge(config::File::with_name("spec_detect"))?;
        config.try_into()
    }
}
