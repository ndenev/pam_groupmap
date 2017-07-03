use std::fs::File;
use std::io;
use std::io::prelude::*;
use toml;
use std::collections::BTreeMap;

// Connection and Operation timeouts in seconds.
const LDAP_CONN_TIMEOUT: u64 = 2;
const LDAP_OP_TIMEOUT: u64 = 5;

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

fn default_conn_timeout() -> u64 {
    LDAP_CONN_TIMEOUT
}

fn default_op_timeout() -> u64 {
    LDAP_OP_TIMEOUT
}

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
    #[serde(default = "default_conn_timeout")]
    pub conn_timeout: u64,
    #[serde(default = "default_op_timeout")]
    pub op_timeout: u64,
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