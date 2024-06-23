pub struct ExportCommand;

impl ExportCommand {
    pub async fn run(config: &str, sfn_arn: &str, schedule_arn: &Option<String>) {
        println!("export called with");
        println!("- export path: {}", config);
        println!("- target sfn arn: {}", sfn_arn);
        println!("- target schedule arn: {:?}", schedule_arn);
    }
}
