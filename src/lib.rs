#[cfg(target_os = "linux")]
extern crate libc;

#[cfg(target_os = "windows")]
extern crate winapi;

#[cfg(target_os = "macos")]
extern crate core_foundation;
#[cfg(target_os = "macos")]
extern crate libc;

mod mapping;
mod platform;

pub use self::platform::*;

pub const MAX_DEVICES: usize = 8;
pub const MAX_DIGITAL: usize = 16;
pub const MAX_ANALOG: usize = 8;

#[derive(Debug, Clone)]
pub struct ControllerInfo {
    pub name: String,
    pub buttons: Vec<GamepadButton>,
    pub analog_count: usize,
}

impl ControllerInfo {
    pub fn new() -> Self {
        Self {
            name: "null".to_owned(),
            analog_count: 0,
            buttons: vec![],
        }
    }
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum GamepadButton {
    /// Also Cross
    A = 0,
    /// Also Circle
    B,
    /// Also Square
    X,
    /// Also Triangle
    Y,
    DpadUp,
    DpadDown,
    DpadRight,
    DpadLeft,
    BumperLeft,
    BumperRight,
    ThumbLeft,
    ThumbRight,
    Select,
    Start,
    Back,
    Unknown,
    Max,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerStatus {
    Disconnected,
    Connected,
}

#[derive(Debug)]
pub struct ControllerState {
    pub status: ControllerStatus,
    pub sequence: usize,
    pub digital_state_prev: [bool; GamepadButton::Max as usize],
    pub digital_state: [bool; GamepadButton::Max as usize],
    pub analog_state: [f32; MAX_ANALOG],
}

impl ControllerState {
    pub fn new() -> Self {
        Self {
            status: ControllerStatus::Disconnected,
            sequence: 0,
            digital_state: [false; GamepadButton::Max as usize],
            digital_state_prev: [false; GamepadButton::Max as usize],
            analog_state: [0.0; MAX_ANALOG],
        }
    }
}

const DEFAULT_CONTROLLER_STATE: ControllerState = ControllerState {
    status: ControllerStatus::Disconnected,
    sequence: 0,
    digital_state: [false; GamepadButton::Max as usize],
    digital_state_prev: [false; GamepadButton::Max as usize],
    analog_state: [0.0; MAX_ANALOG],
};
