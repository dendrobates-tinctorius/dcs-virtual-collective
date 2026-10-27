use crate::config::{parse_axis, Config};
use anyhow::Result;
use evdev::{
    uinput::VirtualDevice, uinput::VirtualDeviceBuilder, AbsInfo, AbsoluteAxisType,
    AttributeSet, Device, EventType, InputEvent, Key, UinputAbsSetup,
};
use std::thread;
use std::time::Duration;

pub struct CollectiveSimulator {
  gamepad: Device,
  virtual_device: VirtualDevice,
  target_axis: AbsoluteAxisType,
  virtual_axis_type: AbsoluteAxisType,
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

    // Register a full standard gamepad button set so Steam Input fully maps the device
    let keys = AttributeSet::from_iter([
        Key::BTN_SOUTH,
        Key::BTN_EAST,
        Key::BTN_NORTH,
        Key::BTN_WEST,
        Key::BTN_TL,
        Key::BTN_TR,
        Key::BTN_SELECT,
        Key::BTN_START,
        Key::BTN_MODE,
        Key::BTN_THUMBL,
        Key::BTN_THUMBR,
    ]);

    let standard_axes = [
        AbsoluteAxisType::ABS_X,
        AbsoluteAxisType::ABS_Y,
        AbsoluteAxisType::ABS_RX,
        AbsoluteAxisType::ABS_RY,
        AbsoluteAxisType::ABS_Z,
        AbsoluteAxisType::ABS_RZ,
    ];

    let mut builder = VirtualDeviceBuilder::new()?
        .name("DCS-Virtual-Collective")
        .with_keys(&keys)?;

    for axis in standard_axes {
      let abs_setup = UinputAbsSetup::new(axis, abs_info);
      builder = builder.with_absolute_axis(&abs_setup)?;
    }

    let mut virtual_device = builder.build()?;

    // Initialize all axes to proper defaults
    for axis in standard_axes {
      let init_val = if axis == AbsoluteAxisType::ABS_X
          || axis == AbsoluteAxisType::ABS_Y
          || axis == AbsoluteAxisType::ABS_RX
          || axis == AbsoluteAxisType::ABS_RY
      {
        16384
      } else {
        0
      };
      let init_event = InputEvent::new(EventType::ABSOLUTE, axis.0, init_val);
      virtual_device.emit(&[init_event])?;
    }

    let center_val: i32 = 0;
    let half_span: f32 = 32767.0;
    let deadzone: f32 = half_span * config.deadzone_percent;

    Ok(Self {
      gamepad,
      virtual_device,
      target_axis,
      virtual_axis_type,
      center_val,
      half_span,
      deadzone,
      step_multiplier: config.step_multiplier,
      physical_y: center_val,
      virtual_value: 0.0,
    })
  }

  pub fn run(&mut self) -> Result<()> {
    println!("Rust Virtual Collective active (Full Profile Mode)!");
    println!("Press Ctrl+C to stop.");

    let max_val = 32767.0f32;
    let mut last_emitted_value: f32 = -1.0;

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
      }

      if (self.virtual_value - last_emitted_value).abs() >= 1.0 {
        let event = InputEvent::new(
            EventType::ABSOLUTE,
            self.virtual_axis_type.0,
            self.virtual_value as i32,
        );
        self.virtual_device.emit(&[event])?;
        last_emitted_value = self.virtual_value;
      }

      thread::sleep(Duration::from_millis(10));
    }
  }
}