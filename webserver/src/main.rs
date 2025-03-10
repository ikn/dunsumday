use std::path::PathBuf;
use clap::Parser;
use dunsumday::config;

mod configrefs;
mod constant;
mod api;
mod ui;
mod server;

#[derive(Parser)]
#[command(version, long_about = None)]
/// Webserver for dunsumday, a tool used to track completion of regular tasks.
struct Options {
    #[arg(short, long, value_name = "FILE",
          default_value = "/usr/local/etc/dunsumday/config.yaml")]
    /// Path to config file.
    config: PathBuf,
}

fn main() -> Result<(), String> {
    env_logger::init();
    let options = Options::parse();
    let cfg = Box::new(config::file::new(&options.config)?);
    server::run(Box::leak::<'static>(cfg))
}
