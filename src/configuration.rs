use crate::ruby::rspec::RSpecConfiguration;
use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub branch: String,
    pub include: Vec<String>,
    pub rspec: RSpecConfiguration,
    pub map: HashMap<String, Vec<(String, String)>>,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            branch: String::from("master"),
            include: vec![],
            rspec: RSpecConfiguration::default(),
            map: HashMap::new(),
        }
    }
}

impl Configuration {
    pub fn read_configuration() -> Result<Self, ConfigError> {
        let mut config = Config::try_from(&Configuration::default())?;
        config.merge(config::File::with_name("spec_detect").required(false))?;
        config.set("rspec.use_bundler", true)?;
        config.try_into()
    }
}
