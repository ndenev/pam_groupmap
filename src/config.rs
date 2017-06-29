use std::fs::File;
use std::io;
use std::io::prelude::*;
use toml;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct ConfigError(String);
impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> ConfigError {
        let msg = format!("Error reading config file: {}", e);
        ConfigError(msg)
    }
}
impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> ConfigError {
        let msg = format!("Error parsing config file: {}", e);
        ConfigError(msg)
    }
}

pub type Mappings = BTreeMap<String, String>;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub ldap: LdapConfig,
    pub mappings: Mappings,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct LdapConfig {
    pub uri: String,
    pub user: String,
    pub pass: String,
    pub user_base_dn: String,
    pub group_base_dn: String,
    pub uid_attribute: String,
    pub group_attribute: String,
}

impl Config {
    pub fn load(config_file: &str) -> Result<Config, ConfigError> {
        let mut file = File::open(config_file)?;
        let mut cfg_text = String::new();
        file.read_to_string(&mut cfg_text)?;
        Ok(toml::from_str(&cfg_text)?)
    }
}