use std::{fs::read_to_string, path::PathBuf};

use clap::Parser;
use server::{GetConfig, ServerConfig, run};

#[derive(Parser)]
#[command(name = "micro-url", version, about, long_about = None)]
struct Cli {
    /// Optional path to the assets directory
    #[arg(short, long, value_name = "ASSETS_PATH")]
    assets: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "CONFIG_FILE")]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let mut config: ServerConfig = if let Some(config) = cli.config {
        let config_content = read_to_string(config).expect("Unable to read config file");
        toml::from_str(&config_content).expect("Unable to parse config")
    } else {
        ServerConfig::from_env()
    };

    if let Some(assets) = cli.assets {
        config.assets_path = assets;
    }

    run(config.to_owned()).await
}
