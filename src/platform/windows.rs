use std::mem;

use winapi::shared::winerror::ERROR_SUCCESS;
use winapi::um::xinput::{
    self, XINPUT_CAPABILITIES as XCapabilities, XINPUT_FLAG_GAMEPAD, XINPUT_GAMEPAD_A,
    XINPUT_GAMEPAD_B, XINPUT_GAMEPAD_BACK, XINPUT_GAMEPAD_LEFT_SHOULDER, XINPUT_GAMEPAD_LEFT_THUMB,
    XINPUT_GAMEPAD_RIGHT_SHOULDER, XINPUT_GAMEPAD_RIGHT_THUMB, XINPUT_GAMEPAD_START,
    XINPUT_GAMEPAD_X, XINPUT_GAMEPAD_Y, XINPUT_STATE as XState,
};

use super::super::{
    ControllerInfo, ControllerState, ControllerStatus, DEFAULT_CONTROLLER_STATE, MAX_DIGITAL,
};

use crate::GamepadButton;

pub struct GamePad {
    info: ControllerInfo,
    state: ControllerState,
    buttons_map: [u16; MAX_DIGITAL],
    sequence: u32,
}

impl GamePad {
    pub fn new(capabilities: &XCapabilities) -> GamePad {
        let mut name = String::from("XBOX360");
        match capabilities.SubType {
            xinput::XINPUT_DEVSUBTYPE_GAMEPAD => name.push_str(" gamepad"),
            xinput::XINPUT_DEVSUBTYPE_WHEEL => name.push_str(" wheel"),
            xinput::XINPUT_DEVSUBTYPE_ARCADE_STICK => name.push_str(" arcade stick"),
            xinput::XINPUT_DEVSUBTYPE_FLIGHT_SICK => name.push_str(" flight stick"),
            xinput::XINPUT_DEVSUBTYPE_DANCE_PAD => name.push_str(" dance pad"),
            xinput::XINPUT_DEVSUBTYPE_GUITAR => name.push_str(" guitar"),
            xinput::XINPUT_DEVSUBTYPE_DRUM_KIT => name.push_str(" drum"),
            _ => (),
        };
        name.push_str(" controller");

        let mut buttons = vec![];
        let mut buttons_map = [0; MAX_DIGITAL as usize];
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_A != 0 {
            buttons.push(GamepadButton::A);
            buttons_map[GamepadButton::A as usize] = XINPUT_GAMEPAD_A;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_B != 0 {
            buttons.push(GamepadButton::B);
            buttons_map[GamepadButton::B as usize] = XINPUT_GAMEPAD_B;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_X != 0 {
            buttons.push(GamepadButton::X);
            buttons_map[GamepadButton::X as usize] = XINPUT_GAMEPAD_X;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_Y != 0 {
            buttons.push(GamepadButton::Y);
            buttons_map[GamepadButton::Y as usize] = XINPUT_GAMEPAD_Y;
        }

        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_START != 0 {
            buttons.push(GamepadButton::Start);
            buttons_map[GamepadButton::Start as usize] = XINPUT_GAMEPAD_START;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_BACK != 0 {
            buttons.push(GamepadButton::Back);
            buttons_map[GamepadButton::Back as usize] = XINPUT_GAMEPAD_BACK;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_LEFT_THUMB != 0 {
            buttons.push(GamepadButton::ThumbLeft);
            buttons_map[GamepadButton::ThumbLeft as usize] = XINPUT_GAMEPAD_LEFT_THUMB;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_RIGHT_THUMB != 0 {
            buttons.push(GamepadButton::ThumbRight);
            buttons_map[GamepadButton::ThumbRight as usize] = XINPUT_GAMEPAD_RIGHT_THUMB;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_LEFT_SHOULDER != 0 {
            buttons.push(GamepadButton::BumperLeft);
            buttons_map[GamepadButton::BumperLeft as usize] = XINPUT_GAMEPAD_LEFT_SHOULDER;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_RIGHT_SHOULDER != 0 {
            buttons.push(GamepadButton::BumperRight);
            buttons_map[GamepadButton::BumperRight as usize] = XINPUT_GAMEPAD_RIGHT_SHOULDER;
        }

        let mut analog_count = 0;
        if capabilities.Gamepad.bLeftTrigger != 0 {
            analog_count += 1;
        }
        if capabilities.Gamepad.bRightTrigger != 0 {
            analog_count += 1;
        }
        if capabilities.Gamepad.sThumbLX != 0 {
            analog_count += 1;
        }
        if capabilities.Gamepad.sThumbLY != 0 {
            analog_count += 1;
        }
        if capabilities.Gamepad.sThumbRX != 0 {
            analog_count += 1;
        }
        if capabilities.Gamepad.sThumbRY != 0 {
            analog_count += 1;
        }

        GamePad {
            info: ControllerInfo {
                name,
                buttons,
                analog_count,
            },
            state: ControllerState {
                status: ControllerStatus::Connected,
                ..ControllerState::new()
            },
            buttons_map,
            sequence: 0,
        }
    }

    pub fn update(&mut self, state: &XState) {
        self.state.digital_state_prev = self.state.digital_state;

        if state.dwPacketNumber == self.sequence {
            // no change in state
            return;
        }

        self.sequence = state.dwPacketNumber;
        for button in &self.info.buttons {
            self.state.digital_state[*button as usize] =
                state.Gamepad.wButtons & self.buttons_map[*button as usize] != 0;
        }
        self.state.analog_state[0] =
            (state.Gamepad.sThumbLX as i32 + 32768) as f32 / 65535.0 * 2.0 - 1.0;
        self.state.analog_state[1] =
            (state.Gamepad.sThumbLY as i32 + 32768) as f32 / 65535.0 * 2.0 - 1.0;
        self.state.analog_state[2] =
            (state.Gamepad.sThumbRX as i32 + 32768) as f32 / 65535.0 * 2.0 - 1.0;
        self.state.analog_state[3] =
            (state.Gamepad.sThumbRY as i32 + 32768) as f32 / 65535.0 * 2.0 - 1.0;
    }
}
pub struct ControllerContext {
    gamepads: [Option<GamePad>; 4],
}

impl ControllerContext {
    pub fn new() -> Option<Self> {
        unsafe { xinput::XInputEnable(1) };

        Some(Self {
            gamepads: [None, None, None, None],
        })
    }

    pub fn update(&mut self, index: usize) {
        if index >= 4 {
            return;
        }

        let mut state = unsafe { mem::zeroed::<XState>() };
        let val = unsafe { xinput::XInputGetState(index as u32, &mut state) };

        if val == ERROR_SUCCESS {
            if self.gamepads[index].is_none() {
                let mut capabilities = unsafe { mem::zeroed::<XCapabilities>() };
                if unsafe {
                    xinput::XInputGetCapabilities(
                        index as u32,
                        XINPUT_FLAG_GAMEPAD,
                        &mut capabilities,
                    )
                } == ERROR_SUCCESS
                {
                    let gamepad = GamePad::new(&capabilities);
                    self.gamepads[index] = Some(gamepad);
                }
            }

            if let Some(ref mut gamepad) = &mut self.gamepads[index] {
                gamepad.update(&state);
            }
        } else {
            self.gamepads[index] = None;
        }
    }

    pub fn info(&self, index: usize) -> ControllerInfo {
        if let Some(Some(gamepad)) = self.gamepads.get(index) {
            gamepad.info.clone()
        } else {
            ControllerInfo::new()
        }
    }

    pub fn state(&self, index: usize) -> &ControllerState {
        if let Some(Some(gamepad)) = self.gamepads.get(index) {
            &gamepad.state
        } else {
            &DEFAULT_CONTROLLER_STATE
        }
    }
}
