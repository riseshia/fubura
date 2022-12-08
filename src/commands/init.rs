use std::io::Write;
use std::fs::File;

use std::collections::HashMap;

fn bootstrap_files<'a>() -> HashMap<&'a str, &'a str> {
    HashMap::from([
        ("eber-config.jsonnet", r#"{
      targetScheduleGroups: [
        "example-group"
      ]
    }"#),
        ("example-group.jsonnet", "[]"),
    ])
}

fn write_to_file(file_path: &str, content: &str) -> std::io::Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn try_boostrap_file(file_path: &str, content: &str) {
    if std::fs::metadata(file_path).is_ok() {
        let mut input = String::new();

        println!("'{}' is exist, do you want override it?", file_path);
        print!("Type 'yes' if you want: ");
        std::io::stdout().flush().unwrap();

        std::io::stdin().read_line(&mut input).unwrap();

        if input.trim() != "yes" {
            println!("Skip generate file: {}", file_path);
            return
        }
    }

    match write_to_file(file_path, content) {
        Ok(_) => println!("Generated file: {}", file_path),
        Err(error) => println!("Error writing file {} with error: {}", file_path, error)
    }
}

pub struct InitCommand;

impl InitCommand {
    pub fn run() {
        let target_files = bootstrap_files();

        for (file_path, content) in target_files.iter() {
            try_boostrap_file(file_path, content)
        }
    }
}
