use serde::{Deserialize, Serialize};

use crate::{cli::StrKeyVal, jsonnet_evaluator};

use super::SsConfig;

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub ss_configs: Vec<SsConfig>,
}

impl Config {
    pub fn load_from_path(config: &str, ext_str: &[StrKeyVal]) -> Config {
        let config_value = jsonnet_evaluator::eval(config, ext_str).unwrap();

        serde_json::from_value(config_value).unwrap_or_else(|e| {
            panic!("failed to parse config file with error: {}", e);
        })
    }
}
