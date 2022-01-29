use rdev;
use toml::{Value};
use std::collections::{HashMap, BTreeMap};

#[derive(Debug)]
pub enum Mapping {
    Noop,
    KeyPress(rdev::Key),
}

pub fn parse_mappings(map : &mut BTreeMap<u8, Mapping>, v : Value) {
    if let Value::Table(table) = v { for (button, mapping) in table {
        let input = parse_input(&button);
        let parsed = parse_output(&mapping);
        match parsed {
            Mapping::KeyPress(key) => {
                map.insert(input, parsed);
            },
            _ => {
                println!("Don't know how to map {} => {:?}", button, mapping)
            },
        }
    } }
}

fn parse_input(v : &String) -> u8 {
    match v.as_str() {
        "None" => 0x00,
        "Exit" => 0x01,
        "ActionA" => 0x02,
        "ActionB" => 0x03,
        "ActionC" => 0x04,
        "ActionH" => 0x05,
        "ActionV" => 0x06,
        "ActionD" => 0x07,
        "MenuL" => 0x08,
        "MenuR" => 0x09,
        "Joy" => 0x0A,
        "Cam" => 0x0B,
        "BumperL" => 0x0C,
        "BumperR" => 0x0D,
        "TriggerL" => 0x0E,
        "TriggerR" => 0x0F,
        "Up" => 0x10,
        "Down" => 0x11,
        "Left" => 0x12,
        "Right" => 0x13,
        "HatUp" => 0x14,
        "HatDown" => 0x15,
        "HatLeft" => 0x16,
        "HatRight" => 0x17,
        "MicUp" => 0x18,
        "MicDown" => 0x19,
        "MicLeft" => 0x1A,
        "MicRight" => 0x1B,
        "PovUp" => 0x1C,
        "PovDown" => 0x1D,
        "PovLeft" => 0x1E,
        "PovRight" => 0x1F,
        "JoyX" => 0x20,
        "JoyY" => 0x21,
        "JoyZ" => 0x22,
        "CamX" => 0x23,
        "CamY" => 0x24,
        "CamZ" => 0x25,
        "Slew" => 0x26,
        "Throttle" => 0x27,
        "ThrottleL" => 0x28,
        "ThrottleR" => 0x29,
        "Volume" => 0x2A,
        "Wheel" => 0x2B,
        "Rudder" => 0x2C,
        "Gas" => 0x2D,
        "Brake" => 0x2E,
        "MicPush" => 0x2F,
        "Trigger" => 0x30,
        "Bumper" => 0x31,
        "ActionL" => 0x32,
        "ActionM" => 0x33,
        "ActionR" => 0x34,
        "Pinky" => 0x35,
        "PinkyForward" => 0x36,
        "PinkyBackward" => 0x37,
        "FlapsUp" => 0x38,
        "FlapsDown" => 0x39,
        "BoatForward" => 0x3A,
        "BoatBackward" => 0x3B,
        "AutopilotPath" => 0x3C,
        "AutopilotAlt" => 0x3D,
        "EngineMotorL" => 0x3E,
        "EngineMotorR" => 0x3F,
        "EngineFuelFlowL" => 0x40,
        "EngineFuelFlowR" => 0x41,
        "EngineIgnitionL" => 0x42,
        "EngineIgnitionR" => 0x43,
        "SpeedbrakeBackward" => 0x44,
        "SpeedbrakeForward" => 0x45,
        "ChinaBackward" => 0x46,
        "ChinaForward" => 0x47,
        "Apu" => 0x48,
        "RadarAltimeter" => 0x49,
        "LandingGearSilence" => 0x4A,
        "Eac" => 0x4B,
        "AutopilotToggle" => 0x4C,
        "ThrottleButton" => 0x4D,
        "MouseX" => 0x4E,
        "MouseY" => 0x4F,
        "Mouse" => 0x50,
        "PaddleLeft" => 0x51,
        "PaddleRight" => 0x52,
        "PinkyLeft" => 0x53,
        "PinkyRight" => 0x54,
        "Context" => 0x55,
        "Dpi" => 0x56,
        "ScrollX" => 0x57,
        "ScrollY" => 0x58,
        "Scroll" => 0x59,
        "TrimUp" => 0x5A,
        "TrimDown" => 0x5B,
        "TrimLeft" => 0x5C,
        "TrimRight" => 0x5D,
        unknown => {
            if let Ok(num) = unknown.parse::<u8>() {
                return num & !0x80
            }
            panic!("Unknown: {}", unknown)
        },
    }
}

pub fn parse_output(v : &Value) -> Mapping {
    match v {
        Value::String(v) => {
            match v.as_str() {
                "Alt" => Mapping::KeyPress(rdev::Key::Alt),
                "AltGr" => Mapping::KeyPress(rdev::Key::AltGr),
                "Backspace" => Mapping::KeyPress(rdev::Key::Backspace),
                "CapsLock" => Mapping::KeyPress(rdev::Key::CapsLock),
                "ControlLeft" => Mapping::KeyPress(rdev::Key::ControlLeft),
                "ControlRight" => Mapping::KeyPress(rdev::Key::ControlRight),
                "Delete" => Mapping::KeyPress(rdev::Key::Delete),
                "End" => Mapping::KeyPress(rdev::Key::End),
                "Escape" => Mapping::KeyPress(rdev::Key::Escape),
                "F1" => Mapping::KeyPress(rdev::Key::F1),
                "F2" => Mapping::KeyPress(rdev::Key::F2),
                "F3" => Mapping::KeyPress(rdev::Key::F3),
                "F4" => Mapping::KeyPress(rdev::Key::F4),
                "F5" => Mapping::KeyPress(rdev::Key::F5),
                "F6" => Mapping::KeyPress(rdev::Key::F6),
                "F7" => Mapping::KeyPress(rdev::Key::F7),
                "F8" => Mapping::KeyPress(rdev::Key::F8),
                "F9" => Mapping::KeyPress(rdev::Key::F9),
                "F10" => Mapping::KeyPress(rdev::Key::F10),
                "F11" => Mapping::KeyPress(rdev::Key::F11),
                "F12" => Mapping::KeyPress(rdev::Key::F12),
                "Home" => Mapping::KeyPress(rdev::Key::Home),
                "UpArrow" => Mapping::KeyPress(rdev::Key::UpArrow),
                "LeftArrow" => Mapping::KeyPress(rdev::Key::LeftArrow),
                "DownArrow" => Mapping::KeyPress(rdev::Key::DownArrow),
                "RightArrow" => Mapping::KeyPress(rdev::Key::RightArrow),
                "MetaLeft" => Mapping::KeyPress(rdev::Key::MetaLeft),
                "MetaRight" => Mapping::KeyPress(rdev::Key::MetaRight),
                "PageDown" => Mapping::KeyPress(rdev::Key::PageDown),
                "PageUp" => Mapping::KeyPress(rdev::Key::PageUp),
                "Return" => Mapping::KeyPress(rdev::Key::Return),
                "Shift" => Mapping::KeyPress(rdev::Key::ShiftLeft),
                "ShiftLeft" => Mapping::KeyPress(rdev::Key::ShiftLeft),
                "ShiftRight" => Mapping::KeyPress(rdev::Key::ShiftRight),
                "Space" => Mapping::KeyPress(rdev::Key::Space),
                "Tab" => Mapping::KeyPress(rdev::Key::Tab),
                "PrintScreen" => Mapping::KeyPress(rdev::Key::PrintScreen),
                "ScrollLock" => Mapping::KeyPress(rdev::Key::ScrollLock),
                "Pause" => Mapping::KeyPress(rdev::Key::Pause),
                "NumLock" => Mapping::KeyPress(rdev::Key::NumLock),
                "BackQuote" => Mapping::KeyPress(rdev::Key::BackQuote),
                "Num1" => Mapping::KeyPress(rdev::Key::Num1),
                "Num2" => Mapping::KeyPress(rdev::Key::Num2),
                "Num3" => Mapping::KeyPress(rdev::Key::Num3),
                "Num4" => Mapping::KeyPress(rdev::Key::Num4),
                "Num5" => Mapping::KeyPress(rdev::Key::Num5),
                "Num6" => Mapping::KeyPress(rdev::Key::Num6),
                "Num7" => Mapping::KeyPress(rdev::Key::Num7),
                "Num8" => Mapping::KeyPress(rdev::Key::Num8),
                "Num9" => Mapping::KeyPress(rdev::Key::Num9),
                "Num0" => Mapping::KeyPress(rdev::Key::Num0),
                "Minus" => Mapping::KeyPress(rdev::Key::Minus),
                "Equal" => Mapping::KeyPress(rdev::Key::Equal),
                "LeftBracket" => Mapping::KeyPress(rdev::Key::LeftBracket),
                "RightBracket" => Mapping::KeyPress(rdev::Key::RightBracket),
                "KeyA" => Mapping::KeyPress(rdev::Key::KeyA),
                "KeyB" => Mapping::KeyPress(rdev::Key::KeyB),
                "KeyC" => Mapping::KeyPress(rdev::Key::KeyC),
                "KeyD" => Mapping::KeyPress(rdev::Key::KeyD),
                "KeyE" => Mapping::KeyPress(rdev::Key::KeyE),
                "KeyF" => Mapping::KeyPress(rdev::Key::KeyF),
                "KeyG" => Mapping::KeyPress(rdev::Key::KeyG),
                "KeyH" => Mapping::KeyPress(rdev::Key::KeyH),
                "KeyI" => Mapping::KeyPress(rdev::Key::KeyI),
                "KeyJ" => Mapping::KeyPress(rdev::Key::KeyJ),
                "KeyK" => Mapping::KeyPress(rdev::Key::KeyK),
                "KeyL" => Mapping::KeyPress(rdev::Key::KeyL),
                "KeyM" => Mapping::KeyPress(rdev::Key::KeyM),
                "KeyN" => Mapping::KeyPress(rdev::Key::KeyN),
                "KeyO" => Mapping::KeyPress(rdev::Key::KeyO),
                "KeyP" => Mapping::KeyPress(rdev::Key::KeyP),
                "KeyQ" => Mapping::KeyPress(rdev::Key::KeyQ),
                "KeyR" => Mapping::KeyPress(rdev::Key::KeyR),
                "KeyS" => Mapping::KeyPress(rdev::Key::KeyS),
                "KeyT" => Mapping::KeyPress(rdev::Key::KeyT),
                "KeyU" => Mapping::KeyPress(rdev::Key::KeyU),
                "KeyV" => Mapping::KeyPress(rdev::Key::KeyV),
                "KeyW" => Mapping::KeyPress(rdev::Key::KeyW),
                "KeyX" => Mapping::KeyPress(rdev::Key::KeyX),
                "KeyY" => Mapping::KeyPress(rdev::Key::KeyY),
                "KeyZ" => Mapping::KeyPress(rdev::Key::KeyZ),
                "SemiColon" => Mapping::KeyPress(rdev::Key::SemiColon),
                "Quote" => Mapping::KeyPress(rdev::Key::Quote),
                "BackSlash" => Mapping::KeyPress(rdev::Key::BackSlash),
                "IntlBackslash" => Mapping::KeyPress(rdev::Key::IntlBackslash),
                "Comma" => Mapping::KeyPress(rdev::Key::Comma),
                "Dot" => Mapping::KeyPress(rdev::Key::Dot),
                "Slash" => Mapping::KeyPress(rdev::Key::Slash),
                "Insert" => Mapping::KeyPress(rdev::Key::Insert),
                "KpReturn" => Mapping::KeyPress(rdev::Key::KpReturn),
                "KpMinus" => Mapping::KeyPress(rdev::Key::KpMinus),
                "KpPlus" => Mapping::KeyPress(rdev::Key::KpPlus),
                "KpMultiply" => Mapping::KeyPress(rdev::Key::KpMultiply),
                "KpDivide" => Mapping::KeyPress(rdev::Key::KpDivide),
                "Kp0" => Mapping::KeyPress(rdev::Key::Kp0),
                "Kp1" => Mapping::KeyPress(rdev::Key::Kp1),
                "Kp2" => Mapping::KeyPress(rdev::Key::Kp2),
                "Kp3" => Mapping::KeyPress(rdev::Key::Kp3),
                "Kp4" => Mapping::KeyPress(rdev::Key::Kp4),
                "Kp5" => Mapping::KeyPress(rdev::Key::Kp5),
                "Kp6" => Mapping::KeyPress(rdev::Key::Kp6),
                "Kp7" => Mapping::KeyPress(rdev::Key::Kp7),
                "Kp8" => Mapping::KeyPress(rdev::Key::Kp8),
                "Kp9" => Mapping::KeyPress(rdev::Key::Kp9),
                "KpDelete" => Mapping::KeyPress(rdev::Key::KpDelete),
                "Function" => Mapping::KeyPress(rdev::Key::Function),

                // Aliases for buttons for user convenience
                "Ctrl" => Mapping::KeyPress(rdev::Key::ControlLeft),
                "Del" => Mapping::KeyPress(rdev::Key::Delete),
                "Enter" => Mapping::KeyPress(rdev::Key::Return),
                "Up" => Mapping::KeyPress(rdev::Key::UpArrow),
                "Down" => Mapping::KeyPress(rdev::Key::DownArrow),
                "Left" => Mapping::KeyPress(rdev::Key::LeftArrow),
                "Right" => Mapping::KeyPress(rdev::Key::RightArrow),
                "WinRight" => Mapping::KeyPress(rdev::Key::MetaRight),
                "SuperRight" => Mapping::KeyPress(rdev::Key::MetaRight),
                "WinLeft" => Mapping::KeyPress(rdev::Key::MetaLeft),
                "SuperLeft" => Mapping::KeyPress(rdev::Key::MetaLeft),
                ";" => Mapping::KeyPress(rdev::Key::SemiColon),
                "\"" => Mapping::KeyPress(rdev::Key::Quote),
                "\\" => Mapping::KeyPress(rdev::Key::BackSlash),
                "," => Mapping::KeyPress(rdev::Key::Comma),
                "." => Mapping::KeyPress(rdev::Key::Dot),
                "/" => Mapping::KeyPress(rdev::Key::Slash),
                "[" => Mapping::KeyPress(rdev::Key::LeftBracket),
                "]" => Mapping::KeyPress(rdev::Key::RightBracket),
                "A" => Mapping::KeyPress(rdev::Key::KeyA),
                "B" => Mapping::KeyPress(rdev::Key::KeyB),
                "C" => Mapping::KeyPress(rdev::Key::KeyC),
                "D" => Mapping::KeyPress(rdev::Key::KeyD),
                "E" => Mapping::KeyPress(rdev::Key::KeyE),
                "F" => Mapping::KeyPress(rdev::Key::KeyF),
                "G" => Mapping::KeyPress(rdev::Key::KeyG),
                "H" => Mapping::KeyPress(rdev::Key::KeyH),
                "I" => Mapping::KeyPress(rdev::Key::KeyI),
                "J" => Mapping::KeyPress(rdev::Key::KeyJ),
                "K" => Mapping::KeyPress(rdev::Key::KeyK),
                "L" => Mapping::KeyPress(rdev::Key::KeyL),
                "M" => Mapping::KeyPress(rdev::Key::KeyM),
                "N" => Mapping::KeyPress(rdev::Key::KeyN),
                "O" => Mapping::KeyPress(rdev::Key::KeyO),
                "P" => Mapping::KeyPress(rdev::Key::KeyP),
                "Q" => Mapping::KeyPress(rdev::Key::KeyQ),
                "R" => Mapping::KeyPress(rdev::Key::KeyR),
                "S" => Mapping::KeyPress(rdev::Key::KeyS),
                "T" => Mapping::KeyPress(rdev::Key::KeyT),
                "U" => Mapping::KeyPress(rdev::Key::KeyU),
                "V" => Mapping::KeyPress(rdev::Key::KeyV),
                "W" => Mapping::KeyPress(rdev::Key::KeyW),
                "X" => Mapping::KeyPress(rdev::Key::KeyX),
                "Y" => Mapping::KeyPress(rdev::Key::KeyY),
                "Z" => Mapping::KeyPress(rdev::Key::KeyZ),
                "1" => Mapping::KeyPress(rdev::Key::Num1),
                "2" => Mapping::KeyPress(rdev::Key::Num2),
                "3" => Mapping::KeyPress(rdev::Key::Num3),
                "4" => Mapping::KeyPress(rdev::Key::Num4),
                "5" => Mapping::KeyPress(rdev::Key::Num5),
                "6" => Mapping::KeyPress(rdev::Key::Num6),
                "7" => Mapping::KeyPress(rdev::Key::Num7),
                "8" => Mapping::KeyPress(rdev::Key::Num8),
                "9" => Mapping::KeyPress(rdev::Key::Num9),
                "0" => Mapping::KeyPress(rdev::Key::Num0),
                _ => {
                    println!("Unrecognized key: {:?}", v);
                    Mapping::Noop
                },
            }
        },
        v => {
            println!("Unrecognized mapping: {:?}", v);
            Mapping::Noop
        },
    }
}
