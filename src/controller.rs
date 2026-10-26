use crate::config::{parse_axis, Config};
use anyhow::Result;
use evdev::{
    uinput::VirtualDevice, uinput::VirtualDeviceBuilder, AbsInfo, Device,
    EventType, InputEvent,
};
use std::thread;
use std::time::Duration;

pub struct CollectiveSimulator {
  gamepad: Device,
  virtual_device: VirtualDevice,
  target_axis: evdev::AbsoluteAxisType,
  virtual_axis_type: evdev::AbsoluteAxisType,
  center_val: i32,
  half_span: f32,
  deadzone: f32,
  step_multiplier: f32,
  physical_y: i32,
  virtual_value: f32,
}

impl CollectiveSimulator {
  pub fn new(config: &Config) -> Result<Self> {
    let gamepad = Device::open(&config.gamepad_path)?;
    println!(
        "Connected to physical device: {}",
        gamepad.name().unwrap_or("Unknown")
    );

    let target_axis = parse_axis(&config.target_axis);
    let virtual_axis_type = parse_axis(&config.virtual_axis);

    let abs_info = AbsInfo::new(0, 0, 32767, 0, 0, 0);
    let virtual_device = VirtualDeviceBuilder::new()?
        .name("DCS-Virtual-Collective")
        .with_absolute_axis(&abs_info)?
        .build()?;

    let (center_val, half_span, deadzone) =
        if let Ok(Some(info)) = gamepad.get_abs_info(target_axis) {
          let center = (info.minimum() + info.maximum()) / 2;
          let span = (info.maximum() - info.minimum()) as f32 / 2.0;
          (center, span, span * config.deadzone_percent)
        } else {
          (128, 128.0, 128.0 * config.deadzone_percent)
        };

    let physical_y = center_val;

    Ok(Self {
      gamepad,
      virtual_device,
      target_axis,
      virtual_axis_type,
      center_val,
      half_span,
      deadzone,
      step_multiplier: config.step_multiplier,
      physical_y,
      virtual_value: 0.0,
    })
  }

  pub fn run(&mut self) -> Result<()> {
    println!("Rust Virtual Collective active!");
    println!("Press Ctrl+C to stop.");

    let max_val = 32767.0f32;

    loop {
      if let Ok(events) = self.gamepad.fetch_events() {
        for ev in events {
          if ev.event_type() == EventType::ABSOLUTE
              && ev.code() == self.target_axis.0
          {
            self.physical_y = ev.value();
          }
        }
      }

      let deviation = self.physical_y - self.center_val;
      if (deviation as f32).abs() > self.deadzone {
        let normalized_dev = ((deviation as f32).abs() - self.deadzone)
            / (self.half_span - self.deadzone);
        let step = normalized_dev * self.step_multiplier;

        if deviation > 0 {
          self.virtual_value += step;
        } else {
          self.virtual_value -= step;
        }

        self.virtual_value = self.virtual_value.clamp(0.0, max_val);

        let event = InputEvent::new(
            EventType::ABSOLUTE,
            self.virtual_axis_type.0,
            self.virtual_value as i32,
        );
        self.virtual_device.emit(&[event])?;
      }

      thread::sleep(Duration::from_millis(10));
    }
  }
}