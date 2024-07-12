use similar::TextDiff;

use crate::types::{Schedule, SsConfig, StateMachine};

fn print_resource_diff(remote: &str, local: &str) {
    let diff = TextDiff::from_lines(remote, local);

    diff.unified_diff()
        .header("remote", "local")
        .to_writer(std::io::stdout())
        .unwrap();
}

pub fn print_config_diff(
    local_config: &SsConfig,
    remote_sfn: &Option<StateMachine>,
    remote_schedule: &Option<Schedule>,
) {
    let remote_sfn_json_string = serde_json::to_string_pretty(&remote_sfn).unwrap();
    let local_sfn_json_string = serde_json::to_string_pretty(&local_config.state).unwrap();

    let mut has_no_diff = true;

    if local_sfn_json_string != remote_sfn_json_string {
        has_no_diff = false;
        print_resource_diff(&remote_sfn_json_string, &local_sfn_json_string);
    }

    if let Some(local_schedule) = &local_config.schedule {
        let local_schedule_json_string = serde_json::to_string_pretty(&local_schedule).unwrap();

        match remote_schedule {
            Some(remote_schedule) => {
                let remote_schedule_json_string =
                    serde_json::to_string_pretty(&remote_schedule).unwrap();

                if remote_schedule_json_string != local_schedule_json_string {
                    has_no_diff = false;
                    print_resource_diff(&remote_schedule_json_string, &local_schedule_json_string);
                }
            }
            None => {
                print_resource_diff("null", &local_schedule_json_string);
            }
        }
    } else {
        todo!("no local schedule & remote schedule exists/not exists case");
    }

    if has_no_diff {
        println!("no difference");
    }
}
