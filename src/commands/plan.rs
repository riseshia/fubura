use anyhow::Result;

use crate::context::FuburaContext;
use crate::differ::diff;
use crate::types::{Config, DiffResult};

pub struct PlanCommand;

impl PlanCommand {
    pub async fn run(context: &FuburaContext, config: &Config) -> Result<()> {
        let diff_result = diff(context, config).await?;

        if let Some(json_diff_path) = &context.json_diff_path {
            write_result_to_path(json_diff_path, &diff_result)?;
        }

        Ok(())
    }
}

fn write_result_to_path(output_path: &str, diff_result: &DiffResult) -> Result<()> {
    let json_diff = serde_json::to_string_pretty(diff_result).unwrap();
    std::fs::write(output_path, json_diff)?;
    Ok(())
}
