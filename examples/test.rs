use quad_gamepad::*;

use std::thread;
use std::time::Duration;

pub fn main() {
    let mut context = ControllerContext::new().unwrap();

    context.update();

    for i in 0..MAX_DEVICES {
        let status = context.state(i).status;
        if status == ControllerStatus::Connected {
            println!("{:?}", context.info(i));
        }
    }

    loop {
        context.update();

        for i in 0..MAX_DEVICES {
            let state = context.state(i);
            if state.status == ControllerStatus::Connected {
                let info = context.info(i);

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
