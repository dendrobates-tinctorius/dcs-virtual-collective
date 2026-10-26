mod config;
mod controller;

use anyhow::Result;
use config::Config;
use controller::CollectiveSimulator;

fn main() -> Result<()> {
  let config = Config::load_or_create("config.json")?;
  let mut simulator = CollectiveSimulator::new(&config)?;
  simulator.run()?;
  Ok(())
}