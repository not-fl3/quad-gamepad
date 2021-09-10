mod hid;
mod io_kit;

use super::super::{
    ControllerInfo, ControllerState, ControllerStatus, DEFAULT_CONTROLLER_STATE, MAX_ANALOG,
    MAX_DEVICES, MAX_DIGITAL,
};

use crate::mapping::Mapping;

pub struct ControllerContext {
    info: Vec<ControllerInfo>,
    state: Vec<ControllerState>,
    hid: hid::HID,
    mappings: crate::mapping::MappingsMap,
}

// Helper function for convert Vec to array
fn to_digital_state_array(mapping: &Mapping, state: &Vec<bool>) -> [bool; MAX_DIGITAL] {
    let mut arr = [false; MAX_DIGITAL];
    for i in 0..state.len() {
        arr[mapping.buttons[i] as usize] = state[i];
    }
    arr
}

fn to_analog_state_array(state: &Vec<f32>) -> [f32; MAX_ANALOG] {
    let mut arr = [0.0; MAX_ANALOG];
    for (place, element) in arr.iter_mut().zip(state.iter()) {
        *place = *element;
    }
    arr
}

impl ControllerContext {
    pub fn new() -> Option<Self> {
        let mut info = Vec::new();
        let mut state = Vec::new();
        for _ in 0..MAX_DEVICES {
            info.push(ControllerInfo::new());
            state.push(ControllerState::new());
        }

        let hid = hid::HID::new();

        match hid {
            Ok(hid) => Some({
                let mappings = crate::mapping::read_mappings_file(crate::mapping::Platform::Mac);
                let mut context = Self {
                    info,
                    state,
                    hid,
                    mappings,
                };
                context.scan_controllers();
                context
            }),
            Err(err) => {
                println!("Error on create HID. reason: {:?}", err);
                None
            }
        }
    }

    fn scan_controllers(&mut self) -> usize {
        self.hid.detect_devices();

        let ndev = self.hid.num_devices();
        let state = self.hid.hid_state();
        let devices = state.devices.borrow();

        for (i, dev) in devices.iter().enumerate() {
            if i >= MAX_DEVICES {
                break;
            }

            if let Some(d) = dev.upgrade() {
                use crate::GamepadButton;

                let d = d.borrow();

                self.info[i] = ControllerInfo {
                    name: d.name.clone(),
                    analog_count: d.axes.len(),
                    buttons: vec![
                        GamepadButton::A,
                        GamepadButton::B,
                        GamepadButton::X,
                        GamepadButton::Y,
                        GamepadButton::Back,
                        GamepadButton::Start,
                        GamepadButton::BumperLeft,
                        GamepadButton::BumperRight,
                        GamepadButton::ThumbLeft,
                        GamepadButton::ThumbRight,
                    ],
                };
                self.state[i].status = ControllerStatus::Connected;
            } else {
                self.info[i] = ControllerInfo::new();
                self.state[i].status = ControllerStatus::Disconnected;
            }
        }

        ndev
    }

    pub fn update(&mut self) {
        for index in 0..MAX_DEVICES {
            self.hid.update(index);

            let state = self.hid.hid_state();
            let devices = state.devices.borrow();

            if index >= devices.len() || index >= MAX_DEVICES {
                return;
            }

            let dev = &devices[index];

            if let Some(d) = dev.upgrade() {
                let dev_bor = d.borrow();

                let mapping = self
                    .mappings
                    .get(&dev_bor.guid)
                    .cloned()
                    .unwrap_or_else(|| {
                        println!("No mapping for {}, falling back to default!", &dev_bor.guid);
                        crate::mapping::Mapping::new(&dev_bor.guid)
                    });

                self.state[index].sequence = dev_bor.state.sequence;
                self.state[index].analog_state = to_analog_state_array(&dev_bor.state.analog_state);
                self.state[index].digital_state_prev = self.state[index].digital_state;
                self.state[index].digital_state =
                    to_digital_state_array(&mapping, &dev_bor.state.digital_state);
            } else {
                self.state[index].status = ControllerStatus::Disconnected;
            }
        }
    }

    /// Get current information of Controller
    pub fn info(&self, index: usize) -> ControllerInfo {
        if index < MAX_DEVICES {
            self.info[index].clone()
        } else {
            ControllerInfo::new()
        }
    }

    /// Get current state of Controller
    pub fn state(&self, index: usize) -> &ControllerState {
        if index < MAX_DEVICES {
            &self.state[index]
        } else {
            &DEFAULT_CONTROLLER_STATE
        }
    }
}
