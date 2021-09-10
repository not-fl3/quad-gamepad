#![allow(dead_code)]

use std::collections::HashMap;

use crate::GamepadButton;

const MAPPINGS: &str = include_str!("mappings.txt");

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    Linux,
    Windows,
    Mac,
    Android,
    IOS,
}

const MAX_BTNS: usize = 140;

#[derive(Debug, Clone)]
pub struct Mapping {
    pub guid: String,
    pub name: String,
    pub platform: Platform,
    pub buttons: [GamepadButton; MAX_BTNS],
}

impl Mapping {
    pub fn new(guid: &str) -> Mapping {
        use GamepadButton::*;

        let mut buttons = [Unknown; MAX_BTNS];
        buttons[0] = A;
        buttons[1] = B;
        buttons[2] = X;
        buttons[3] = Y;
        buttons[4] = BumperLeft;
        buttons[5] = BumperRight;
        buttons[6] = ThumbLeft;
        buttons[7] = ThumbRight;
        buttons[8] = Back;
        buttons[9] = Select;
        buttons[10] = Start;
        buttons[11] = DpadUp;
        buttons[12] = DpadDown;
        buttons[13] = DpadRight;
        buttons[14] = DpadLeft;

        Mapping {
            guid: guid.to_owned(),
            name: "unknown".to_string(),
            platform: Platform::Linux,
            buttons,
        }
    }
}

pub type MappingsMap = HashMap<String, Mapping>;

// this should be a (proc?) macro
pub fn read_mappings_file(target_platform: Platform) -> MappingsMap {
    let mut mappings_map = HashMap::new();

    for line in MAPPINGS.lines() {
        if line.starts_with("#") || line.is_empty() {
            continue;
        }

        let mut tokens = line.split(',');

        let guid = tokens.next().unwrap();
        let name = tokens.next().unwrap();

        let mut mapping = Mapping::new(guid);
        mapping.name = name.to_owned();

        while let Some(pair) = tokens.next() {
            if pair.is_empty() {
                break;
            }
            let mut pair = pair.split(':');
            let key = pair.next().unwrap();
            let value = pair.next().unwrap();

            match key.as_ref() {
                "platform" if value == "Windows" => mapping.platform = Platform::Windows,
                "platform" if value == "Linux" => mapping.platform = Platform::Linux,
                "platform" if value == "Mac OS X" => mapping.platform = Platform::Mac,
                "platform" if value == "Android" => mapping.platform = Platform::Android,
                "platform" if value == "iOS" => mapping.platform = Platform::IOS,
                "platform" => panic!("{:?}", value),
                "a" => {
                    let ix: usize = value[1..].parse().unwrap();
                    mapping.buttons[ix] = GamepadButton::A;
                }
                "b" => {
                    let ix: usize = value[1..].parse().unwrap();
                    mapping.buttons[ix] = GamepadButton::B;
                }
                "x" => {
                    let ix: usize = value[1..].parse().unwrap();
                    mapping.buttons[ix] = GamepadButton::X;
                }
                "y" => {
                    let ix: usize = value[1..].parse().unwrap();
                    mapping.buttons[ix] = GamepadButton::Y;
                }
                "back" => {
                    let ix: usize = value[1..].parse().unwrap();
                    mapping.buttons[ix] = GamepadButton::Back;
                }
                "start" => {
                    let ix: usize = value[1..].parse().unwrap();
                    mapping.buttons[ix] = GamepadButton::Start;
                }
                _ => {}
            }
        }

        if target_platform == mapping.platform {
            mappings_map.insert(guid.to_owned(), mapping);
        }
    }
    mappings_map
}
