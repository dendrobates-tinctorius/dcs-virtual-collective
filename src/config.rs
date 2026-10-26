use anyhow::Result;
use evdev::AbsoluteAxisType;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
  pub gamepad_path: String,
  pub target_axis: String,
  pub virtual_axis: String,
  pub step_multiplier: f32,
  pub deadzone_percent: f32,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      gamepad_path: "/dev/input/event3".to_string(),
      target_axis: "ABS_RY".to_string(),   // Physical gamepad right stick Y
      virtual_axis: "ABS_RY".to_string(),  // Virtual device output axis RY
      step_multiplier: 350.0,
      deadzone_percent: 0.10,
    }
  }
}

impl Config {
  pub fn load_or_create(path: &str) -> Result<Self> {
    if Path::new(path).exists() {
      let data = fs::read_to_string(path)?;
      if data.trim().is_empty() {
        let default_config = Config::default();
        let serialized = serde_json::to_string_pretty(&default_config)?;
        fs::write(path, serialized)?;
        println!("Configuration file '{}' was empty. Restored defaults.", path);
        return Ok(default_config);
      }
      let config: Config = serde_json::from_str(&data)?;
      Ok(config)
    } else {
      let default_config = Config::default();
      let data = serde_json::to_string_pretty(&default_config)?;
      fs::write(path, data)?;
      println!("Generated default configuration file: '{}'.", path);
      Ok(default_config)
    }
  }
}

pub fn parse_axis(name: &str) -> AbsoluteAxisType {
  match name.to_uppercase().as_str() {
    "ABS_X" => AbsoluteAxisType::ABS_X,
    "ABS_Y" => AbsoluteAxisType::ABS_Y,
    "ABS_Z" => AbsoluteAxisType::ABS_Z,
    "ABS_RX" => AbsoluteAxisType::ABS_RX,
    "ABS_RY" => AbsoluteAxisType::ABS_RY,
    "ABS_RZ" => AbsoluteAxisType::ABS_RZ,
    "ABS_THROTTLE" => AbsoluteAxisType::ABS_THROTTLE,
    "ABS_RUDDER" => AbsoluteAxisType::ABS_RUDDER,
    "ABS_WHEEL" => AbsoluteAxisType::ABS_WHEEL,
    "ABS_GAS" => AbsoluteAxisType::ABS_GAS,
    "ABS_BRAKE" => AbsoluteAxisType::ABS_BRAKE,
    _ => AbsoluteAxisType::ABS_RY,
  }
}