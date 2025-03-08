use std::path::PathBuf;
use clap::Parser;
use dunsumday::config::{self, Config};
use self::server::ConfigFactory;

mod configrefs;
mod constant;
mod api;
mod ui;
mod server;

struct AppConfigFactory {
    pub path: PathBuf,
}

impl ConfigFactory for AppConfigFactory {
    fn get(&self) -> Result<Box<dyn Config>, String> {
        Ok(Box::new(config::file::new(&self.path)?))
    }
}

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
    let cfg_factory = Box::leak::<'static>(Box::new(
        AppConfigFactory { path: options.config }));
    server::run(cfg_factory)
}
