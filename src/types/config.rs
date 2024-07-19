use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{cli::StrKeyVal, jsonnet_evaluator};

use super::SsConfig;

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub ss_configs: Vec<SsConfig>,
}

impl Config {
    pub fn load_from_path(config: &str, ext_str: &[StrKeyVal]) -> Config {
        let config_value = jsonnet_evaluator::eval(config, ext_str).unwrap();

        serde_json::from_value(config_value).unwrap_or_else(|e| {
            eprintln!("failed to parse config file with error: {}", e);
            std::process::exit(1);
        })
    }

    pub fn target_ss_configs(&self, targets: &Option<Vec<String>>) -> Vec<&SsConfig> {
        if let Some(targets) = &targets {
            let targets = targets.iter().collect::<HashSet<_>>();

            self.ss_configs
                .iter()
                .filter(|ss_config| targets.contains(&ss_config.state.name))
                .collect::<Vec<_>>()
        } else {
            self.ss_configs.iter().collect::<Vec<_>>()
        }
    }
}
