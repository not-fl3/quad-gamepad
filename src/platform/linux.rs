// https://github.com/glfw/glfw/blob/master/src/linux_joystick.c

use crate::{ControllerInfo, ControllerState, ControllerStatus, DEFAULT_CONTROLLER_STATE};

use std::path::PathBuf;

mod ioctl;
mod linux_input;

use self::ioctl::{InputAbsInfo, InputEvent, InputId};
use self::linux_input::*;

fn is_bit_set(bit: usize, arr: &[u8]) -> bool {
    return (arr[bit / 8] & (1 << ((bit as usize) % 8))) != 0;
}

struct GamePad {
    fd: libc::c_int,
    info: ControllerInfo,
    state: ControllerState,
    axis_map: [i32; ABS_CNT as usize],
    axis_info: [InputAbsInfo; ABS_CNT as usize],
    buttons_map: [usize; (KEY_CNT - BTN_MISC) as usize],
    mapping: crate::mapping::Mapping,
}

impl GamePad {
    unsafe fn poll_abs_info(&mut self) {
        for code in &self.axis_map {
            if *code != -1 {
                if libc::ioctl(
                    self.fd,
                    ioctl::eviocgabs(*code as _),
                    &mut self.axis_info[*code as usize],
                ) < 0
                {
                    continue;
                }
            }
        }
    }

    unsafe fn poll(&mut self) {
        let mut e = InputEvent::default();

        if libc::read(
            self.fd,
            &mut e as *mut _ as *mut _,
            std::mem::size_of_val(&e),
        ) < 0
        {
            // handle disconnect
            return;
        }

        if e.type_ == EV_KEY as _ {
            let code = e.code as usize - BTN_MISC as usize;
            self.state.digital_state[self.mapping.buttons[self.buttons_map[code]] as usize] =
                e.value != 0;
        }
        if e.type_ == EV_ABS as _ {
            let info = self.axis_info[e.code as usize];
            let value = if e.code >= ABS_HAT0X as _ && e.code <= ABS_HAT3Y as _ {
                e.value as f32
            } else {
                ((e.value as f32 - info.minimum as f32)
                    / (info.maximum as f32 - info.minimum as f32)
                    - 0.5)
                    * 2.
            };
            self.state.analog_state[self.axis_map[e.code as usize] as usize] = value;
        }
    }
}

unsafe fn open_joystick_device(
    mappings: &crate::mapping::MappingsMap,
    path: PathBuf,
) -> Option<GamePad> {
    use std::os::unix::ffi::OsStrExt;

    let fd = libc::open(
        path.as_os_str().as_bytes().as_ptr() as _,
        libc::O_RDONLY | libc::O_NONBLOCK,
    );
    if fd == -1 {
        return None;
    }

    let mut ev_bits: [u8; (EV_CNT as usize + 7) / 8] = [0; (EV_CNT as usize + 7) / 8];
    let mut key_bits: [u8; (KEY_CNT as usize + 7) / 8] = [0; (KEY_CNT as usize + 7) / 8];
    let mut abs_bits: [u8; (ABS_CNT as usize + 7) / 8] = [0; (ABS_CNT as usize + 7) / 8];

    let eviocgbit: u64 = ioctl::eviocgbit(0, std::mem::size_of_val(&ev_bits) as _);
    let eviocgbit_ev_key: u64 =
        ioctl::eviocgbit(EV_KEY as _, std::mem::size_of_val(&key_bits) as _);
    let eviocgbit_ev_abs: u64 =
        ioctl::eviocgbit(EV_ABS as _, std::mem::size_of_val(&abs_bits) as _);

    let mut id: InputId = InputId::default();

    if libc::ioctl(fd, eviocgbit, ev_bits.as_mut_ptr()) < 0
        || libc::ioctl(fd, eviocgbit_ev_key, key_bits.as_mut_ptr()) < 0
        || libc::ioctl(fd, eviocgbit_ev_abs, abs_bits.as_mut_ptr()) < 0
        || libc::ioctl(fd, ioctl::eviocgid(), &mut id as *mut _) < 0
    {
        libc::close(fd);
        println!("ioctl failed, bad");
        return None;
    }

    // Ensure this device supports the events expected of a joystick
    if !is_bit_set(EV_KEY as _, &ev_bits) || !is_bit_set(EV_ABS as _, &ev_bits) {
        libc::close(fd);
        return None;
    }

    // Retrieve joystick name
    let mut name_bytes: [u8; 256] = [0; 256];
    let eviocgname: u64 = ioctl::eviocgname(256);
    let name = if libc::ioctl(fd, eviocgname, name_bytes.as_mut_ptr()) >= 0 {
        std::ffi::CStr::from_ptr(name_bytes.as_ptr() as *const _)
            .to_string_lossy()
            .into_owned()
    } else {
        "Unknown".to_string()
    };
    println!("Found gamepad {:?}: {:?}", path, name);
    println!("input_id: {:?}", id);

    // Generate a joystick GUID that matches the SDL 2.0.5+ one
    #[rustfmt::skip]
    let guid = if id.vendor != 0 && id.product != 0 && id.version != 0 {
        format!(
            "{:02x}{:02x}0000{:02x}{:02x}0000{:02x}{:02x}0000{:02x}{:02x}0000",
            id.bustype & 0xff, id.bustype >> 8,
            id.vendor & 0xff,  id.vendor >> 8,
            id.product & 0xff, id.product >> 8,
            id.version & 0xff, id.version >> 8,
        )
    } else {
        format!(
            "{:2x}{:2x}0000{:2x}{:2x}{:2x}{:2x}{:2x}{:2x}{:2x}{:2x}{:2x}{:2x}{:2x}00",
            id.bustype & 0xff, id.bustype >> 8,
            name_bytes[0], name_bytes[1], name_bytes[2], name_bytes[3],
            name_bytes[4], name_bytes[5], name_bytes[6], name_bytes[7],
            name_bytes[8], name_bytes[9], name_bytes[10])
    };

    if !mappings.contains_key(&guid) {
        println!("No mapping for {}, falling back to default!", guid);
    }
    let mapping = mappings
        .get(&guid)
        .cloned()
        .unwrap_or_else(|| crate::mapping::Mapping::new(&guid));

    let mut digital_count = 0;
    let mut buttons = vec![];
    let mut buttons_map = [0; (KEY_CNT - BTN_MISC) as usize];
    for code in BTN_MISC..KEY_CNT {
        if !is_bit_set(code as _, &key_bits) {
            continue;
        }

        buttons_map[(code - BTN_MISC) as usize] = digital_count;
        buttons.push(mapping.buttons[digital_count]);
        digital_count += 1;
    }

    let mut analog_count = 0;
    let mut axis_map = [-1; ABS_CNT as usize];
    let mut axis_info = [InputAbsInfo::default(); ABS_CNT as usize];

    for code in 0..ABS_CNT {
        if !is_bit_set(code as _, &abs_bits) {
            continue;
        }

        if code >= ABS_HAT0X && code <= ABS_HAT3Y {
            axis_map[code as usize] = analog_count as i32;
            analog_count += 1;
        } else {
            if libc::ioctl(
                fd,
                ioctl::eviocgabs(code as _),
                &mut axis_info[code as usize],
            ) < 0
            {
                continue;
            }
            axis_map[code as usize] = analog_count as i32;
            analog_count += 1;
        }
    }

    let mut gamepad = GamePad {
        fd,
        info: ControllerInfo {
            name,
            buttons,
            analog_count,
        },
        state: ControllerState::new(),
        axis_info,
        axis_map,
        buttons_map,
        mapping,
    };
    gamepad.state.status = ControllerStatus::Connected;
    gamepad.poll_abs_info();

    Some(gamepad)
}

unsafe fn platform_init_joysticks(mappings: &crate::mapping::MappingsMap) -> Vec<GamePad> {
    let dirname = "/dev/input";

    let mut res = vec![];

    for entry in std::fs::read_dir(dirname).unwrap() {
        let path = entry.unwrap().path();
        let file_name = path.file_name().unwrap().to_str().unwrap();

        if file_name.starts_with("event") {
            if let Some(gamepad) = open_joystick_device(mappings, path) {
                res.push(gamepad);
            }
        }
    }
    res
}

pub struct ControllerContext {
    gamepads: Vec<GamePad>,
}

impl ControllerContext {
    pub fn new() -> Option<Self> {
        let mappings = crate::mapping::read_mappings_file();
        Some(ControllerContext {
            gamepads: unsafe { platform_init_joysticks(&mappings) },
        })
    }

    /// Update controller state by index
    pub fn update(&mut self, index: usize) {
        if let Some(ref mut gamepad) = self.gamepads.get_mut(index) {
            gamepad.state.digital_state_prev = gamepad.state.digital_state;

            unsafe {
                gamepad.poll();
            }
        }
    }

    pub fn info(&self, index: usize) -> ControllerInfo {
        if let Some(ref gamepad) = self.gamepads.get(index) {
            gamepad.info.clone()
        } else {
            ControllerInfo::new()
        }
    }
    pub fn state(&self, index: usize) -> &ControllerState {
        if let Some(ref gamepad) = self.gamepads.get(index) {
            &gamepad.state
        } else {
            &DEFAULT_CONTROLLER_STATE
        }
    }
}
