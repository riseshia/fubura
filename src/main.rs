use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// apply schedules to EventBridge Scheduler
    Apply,
    /// plan schedules from EventBridge Scheduler
    Plan,
    /// generate schedules bootstrap
    Init,
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Apply => {
            println!("apply called!")
        }
        Commands::Plan => {
            println!("plan called!")
        }
        Commands::Init => {
            println!("init called!")
        }
    }
}
