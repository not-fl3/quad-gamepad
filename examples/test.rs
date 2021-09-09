use quad_gamepad::*;

use std::thread;
use std::time::Duration;

pub fn main() {
    let mut controller = ControllerContext::new().unwrap();

    for i in 0..MAX_DEVICES {
        controller.update(i);
        let status = controller.state(i).status;
        if status == ControllerStatus::Connected {
            println!("{:?}", controller.info(i));
        }
    }

    loop {
        for i in 0..MAX_DEVICES {
            controller.update(i);
            let state = controller.state(i);
            if state.status == ControllerStatus::Connected {
                let info = controller.info(i);

                for button in info.buttons {
                    if state.digital_state[button as usize]
                        && !state.digital_state_prev[button as usize]
                    {
                        println!("{:?}", button);
                    }

                    for (axis, value) in state.analog_state.iter().enumerate() {
                        if value.abs() >= 0.01 {
                            println!("axis {} = {}", axis, value);
                        }
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(10));
    }
}
