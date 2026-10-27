# DCS Virtual Collective Simulator

A lightweight Rust utility designed for Linux (built for **openSUSE Tumbleweed** but should work with other distributions running **DCS World** via **Proton/Wine**) that transforms a spring-loaded physical gamepad axis (such as a right-stick Y-axis) into a **persistent, position-retaining virtual helicopter collective**.

---

## Project Idea & Problem Solved

Flying helicopters (like the **CH-47 Chinook** or **AH-64 Apache**) in flight simulators requires a collective lever that stays in place wherever you leave it. Using a standard gamepad analog stick creates a major challenge: **spring-recenter**. The moment you let go of the physical stick, it snaps back to center, which normally forces your collective back to zero or 50%. On button mapped to collective increase/decrease, value is applied in steps of ~10%.

This tool bridges that gap:
* **Spring Retention:** It reads your physical gamepad input and incrementally adjusts a virtual collective value based on how far you push the stick from center.
* **Position Locking:** When you let go of the stick, it snaps back to center, but the virtual collective **locks in place precisely where you left it**, exactly like a real helicopter collective lever.
* **Steam Input Harmony:** It creates a virtual device (`DCS-Virtual-Collective`) with a full standard gamepad profile, allowing Steam Input to manage your button modifier shifts on your physical gamepad while passing the clean analog axis data directly into DCS World.
* **No "Step-up" behavior**: Allows precise control over each % of collective.

---

## Tech Stack & Architecture

* **Language:** Rust
* **Core Libraries:** 
  * `evdev` — For reading raw input events from physical gamepads.
  * `uinput` — For registering and emitting events from a virtual game controller (`DCS-Virtual-Collective`).
  * `serde` / `serde_json` — For modular, customizable configuration file management (`config.json`).
* **Platform Compatibility:** Linux (evdev/uinput subsystem), Proton, Wine, Steam Input.

---

## Prerequisites & Permissions

Because the tool creates a virtual input device via the Linux kernel's `uinput` module, your user account must have permission to access `/dev/input/uinput`.

### 1. Set up Udev Rules
Create a udev rule to grant your user group write access to `uinput`:
```bash
sudo nano /etc/udev/rules.d/99-uinput.rules
```
Add the following line:
```text
KERNEL=="uinput", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"
```

### 2. Add Your User to the Input Group
```bash
sudo usermod -aG input $USER
```

### 3. Reload Rules
```bash
sudo udevadm control --reload-rules && sudo udevadm trigger
```
*(Note: Log out and log back in, or reboot your system, for group permission changes to take full effect).*

---

## Configuration (`config.json`)

On its first run, the tool automatically generates a default `config.json` file in the project directory:

```json
{
  "gamepad_path": "/dev/input/event3",
  "target_axis": "ABS_RY",
  "virtual_axis": "ABS_RY",
  "step_multiplier": 350.0,
  "deadzone_percent": 0.10
}
```

* **`gamepad_path`**: The system event node for your physical gamepad (e.g., `/dev/input/eventX`).
* **`target_axis`**: The physical stick axis to read (e.g., `ABS_RY` for right stick Y).
* **`virtual_axis`**: The virtual output axis (`ABS_RY`).
* **`step_multiplier`**: Speed at which the collective increases/decreases when the stick is deflected.
* **`deadzone_percent`**: Inner deadzone to prevent stick drift at rest.

---

## Instructions to Run

### 1. Clone & Build
Ensure you have Rust and Cargo installed, then build the release binary:
```bash
cargo build --release
```

### 2. Run the Tool
Launch the simulator directly from your user account (no `sudo` required once your udev rules are active):
```bash
cargo run --release
```
You should see confirmation that it has connected to your physical gamepad and activated the virtual collective device.

### 3. Bind in DCS World
1. Launch DCS World.
2. Open **Controls Settings**.
3. Locate the controller named **`DCS-Virtual-Collective`**.
4. Bind your helicopter's **Collective** axis strictly to **`Axis RY`** on the virtual device.
5. Keep your button modifiers and alt-shift states bound to your physical gamepad (which Steam Input manages concurrently).