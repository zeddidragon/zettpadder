use rdev;
use toml::{Value};
use std::collections::{BTreeMap};
use crate::function::{Function};

#[derive(Debug, Copy, Clone)]
pub enum Mapping {
    Noop,
    Emit(rdev::EventType),
    MouseX(f64),
    MouseY(f64),
    FlickX,
    FlickY,
    NegPos(rdev::EventType, rdev::EventType),
    Layer(u8),
    Trigger(usize),
}

impl Mapping {
    fn released(self) -> Option<Mapping> {
        use Mapping::{Emit, Layer};
        use rdev::EventType::{
            KeyPress,
            KeyRelease,
            ButtonPress,
            ButtonRelease,
            Wheel };
        match self {
            Emit(KeyPress(key)) => {
                Some(Emit(KeyRelease(key)))
            },
            Emit(ButtonPress(btn)) => {
                Some(Emit(ButtonRelease(btn)))
            },
            Emit(Wheel { delta_x: _, delta_y: _ }) => {
                None
            },
            Layer(_) => {
                Some(Layer(0))
            },
            _ => {
                println!("Don't know how to release: {:?}", self);
                None
            },
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Binding {
    pub mapping: Mapping,
    pub deadzone_on: f64,
    pub deadzone_off: f64,
}

impl Binding {
    pub fn get_mapping(self, value: f64, prev: f64) -> Option<Mapping> {
        use Mapping::{Emit, NegPos, Layer};
        let on = self.deadzone_on;
        let off = self.deadzone_off;
        match self.mapping {
            Emit(_) => {
                if prev < on && value >= on {
                    Some(self.mapping)
                } else if prev > off && value <= off {
                    self.mapping.released()
                } else {
                    None
                }
            },
            Layer(_) => {
                if prev < on && value >= on {
                    Some(self.mapping)
                } else if prev > off && value <= off {
                    Some(Mapping::Layer(0))
                } else {
                    None
                }
            },
            NegPos(neg, pos) => {
                let (mapping, value, prev) =
                    if value < 0.0 || prev < 0.0 {
                        (Some(Mapping::Emit(neg)), -value, -prev)
                    } else if value > 0.0 || prev > 0.0 {
                        (Some(Mapping::Emit(pos)), value, prev)
                    } else {
                        (None, 0.0, 0.0)
                    };
                if let Some(mapping) = mapping {
                    let contained = Binding {
                        mapping: mapping,
                        deadzone_on: on,
                        deadzone_off: off,
                    };
                    contained.get_mapping(value, prev)
                } else {
                    None
                }
            },
            _ => {
                Some(self.mapping)
            }
        }
    }
}

pub fn parse_mappings(
    layer: u16,
    v: Value,
    map: &mut BTreeMap<u16, Binding>,
    functions: &mut Vec<Function>,
) {
    if let Value::Table(table) = v {
        for (button, mapping) in table {
            let input = parse_input(&button) as u16;
            let mut parsed = parse_output(&mapping);
            let input = input + 256 * layer;
            let (deadzone_on, deadzone_off) = parse_deadzone(&mapping, parsed);
            if let Some(function) = parse_function(&mapping) {
                let idx = functions.len();
                functions.push(function);
                parsed = Mapping::Trigger(idx);
            };
            let binding = Binding {
                mapping: parsed,
                deadzone_on: deadzone_on,
                deadzone_off: deadzone_off,
            };
            map.insert(input, binding);
        }
    }
}

pub fn parse_layers(
    v: Value,
    map: &mut BTreeMap<u16, Binding>,
    functions: &mut Vec<Function>,
) {
    if let Value::Table(table) = v {
        for (layer, mappings) in table {
            if let Ok(layer) = layer.parse::<u16>() {
                parse_mappings(layer, mappings, map, functions);
            } else {
                println!("Didn't understand layer: {:?}", layer);
            }
        }
    }
}

fn parse_input(v: &String) -> u8 {
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
            println!("Unknown: {}", unknown);
            panic!("Unknown: {}", unknown);
        },
    }
}

pub fn parse_output(v: &Value) -> Mapping {
    use Mapping::{Emit, Layer, Noop};
    use rdev::EventType::{KeyPress, ButtonPress, Wheel};
    match v {
        Value::String(v) => {
            match v.as_str() {
                "Alt" => Emit(KeyPress(rdev::Key::Alt)),
                "AltGr" => Emit(KeyPress(rdev::Key::AltGr)),
                "Backspace" => Emit(KeyPress(rdev::Key::Backspace)),
                "CapsLock" => Emit(KeyPress(rdev::Key::CapsLock)),
                "ControlLeft" => Emit(KeyPress(rdev::Key::ControlLeft)),
                "ControlRight" => Emit(KeyPress(rdev::Key::ControlRight)),
                "Delete" => Emit(KeyPress(rdev::Key::Delete)),
                "End" => Emit(KeyPress(rdev::Key::End)),
                "Escape" => Emit(KeyPress(rdev::Key::Escape)),
                "F1" => Emit(KeyPress(rdev::Key::F1)),
                "F2" => Emit(KeyPress(rdev::Key::F2)),
                "F3" => Emit(KeyPress(rdev::Key::F3)),
                "F4" => Emit(KeyPress(rdev::Key::F4)),
                "F5" => Emit(KeyPress(rdev::Key::F5)),
                "F6" => Emit(KeyPress(rdev::Key::F6)),
                "F7" => Emit(KeyPress(rdev::Key::F7)),
                "F8" => Emit(KeyPress(rdev::Key::F8)),
                "F9" => Emit(KeyPress(rdev::Key::F9)),
                "F10" => Emit(KeyPress(rdev::Key::F10)),
                "F11" => Emit(KeyPress(rdev::Key::F11)),
                "F12" => Emit(KeyPress(rdev::Key::F12)),
                "Home" => Emit(KeyPress(rdev::Key::Home)),
                "UpArrow" => Emit(KeyPress(rdev::Key::UpArrow)),
                "LeftArrow" => Emit(KeyPress(rdev::Key::LeftArrow)),
                "DownArrow" => Emit(KeyPress(rdev::Key::DownArrow)),
                "RightArrow" => Emit(KeyPress(rdev::Key::RightArrow)),
                "MetaLeft" => Emit(KeyPress(rdev::Key::MetaLeft)),
                "MetaRight" => Emit(KeyPress(rdev::Key::MetaRight)),
                "PageDown" => Emit(KeyPress(rdev::Key::PageDown)),
                "PageUp" => Emit(KeyPress(rdev::Key::PageUp)),
                "Return" => Emit(KeyPress(rdev::Key::Return)),
                "Shift" => Emit(KeyPress(rdev::Key::ShiftLeft)),
                "ShiftLeft" => Emit(KeyPress(rdev::Key::ShiftLeft)),
                "ShiftRight" => Emit(KeyPress(rdev::Key::ShiftRight)),
                "Space" => Emit(KeyPress(rdev::Key::Space)),
                "Tab" => Emit(KeyPress(rdev::Key::Tab)),
                "PrintScreen" => Emit(KeyPress(rdev::Key::PrintScreen)),
                "ScrollLock" => Emit(KeyPress(rdev::Key::ScrollLock)),
                "Pause" => Emit(KeyPress(rdev::Key::Pause)),
                "NumLock" => Emit(KeyPress(rdev::Key::NumLock)),
                "BackQuote" => Emit(KeyPress(rdev::Key::BackQuote)),
                "Num1" => Emit(KeyPress(rdev::Key::Num1)),
                "Num2" => Emit(KeyPress(rdev::Key::Num2)),
                "Num3" => Emit(KeyPress(rdev::Key::Num3)),
                "Num4" => Emit(KeyPress(rdev::Key::Num4)),
                "Num5" => Emit(KeyPress(rdev::Key::Num5)),
                "Num6" => Emit(KeyPress(rdev::Key::Num6)),
                "Num7" => Emit(KeyPress(rdev::Key::Num7)),
                "Num8" => Emit(KeyPress(rdev::Key::Num8)),
                "Num9" => Emit(KeyPress(rdev::Key::Num9)),
                "Num0" => Emit(KeyPress(rdev::Key::Num0)),
                "Minus" => Emit(KeyPress(rdev::Key::Minus)),
                "Equal" => Emit(KeyPress(rdev::Key::Equal)),
                "LeftBracket" => Emit(KeyPress(rdev::Key::LeftBracket)),
                "RightBracket" => Emit(KeyPress(rdev::Key::RightBracket)),
                "KeyA" => Emit(KeyPress(rdev::Key::KeyA)),
                "KeyB" => Emit(KeyPress(rdev::Key::KeyB)),
                "KeyC" => Emit(KeyPress(rdev::Key::KeyC)),
                "KeyD" => Emit(KeyPress(rdev::Key::KeyD)),
                "KeyE" => Emit(KeyPress(rdev::Key::KeyE)),
                "KeyF" => Emit(KeyPress(rdev::Key::KeyF)),
                "KeyG" => Emit(KeyPress(rdev::Key::KeyG)),
                "KeyH" => Emit(KeyPress(rdev::Key::KeyH)),
                "KeyI" => Emit(KeyPress(rdev::Key::KeyI)),
                "KeyJ" => Emit(KeyPress(rdev::Key::KeyJ)),
                "KeyK" => Emit(KeyPress(rdev::Key::KeyK)),
                "KeyL" => Emit(KeyPress(rdev::Key::KeyL)),
                "KeyM" => Emit(KeyPress(rdev::Key::KeyM)),
                "KeyN" => Emit(KeyPress(rdev::Key::KeyN)),
                "KeyO" => Emit(KeyPress(rdev::Key::KeyO)),
                "KeyP" => Emit(KeyPress(rdev::Key::KeyP)),
                "KeyQ" => Emit(KeyPress(rdev::Key::KeyQ)),
                "KeyR" => Emit(KeyPress(rdev::Key::KeyR)),
                "KeyS" => Emit(KeyPress(rdev::Key::KeyS)),
                "KeyT" => Emit(KeyPress(rdev::Key::KeyT)),
                "KeyU" => Emit(KeyPress(rdev::Key::KeyU)),
                "KeyV" => Emit(KeyPress(rdev::Key::KeyV)),
                "KeyW" => Emit(KeyPress(rdev::Key::KeyW)),
                "KeyX" => Emit(KeyPress(rdev::Key::KeyX)),
                "KeyY" => Emit(KeyPress(rdev::Key::KeyY)),
                "KeyZ" => Emit(KeyPress(rdev::Key::KeyZ)),
                "SemiColon" => Emit(KeyPress(rdev::Key::SemiColon)),
                "Quote" => Emit(KeyPress(rdev::Key::Quote)),
                "BackSlash" => Emit(KeyPress(rdev::Key::BackSlash)),
                "IntlBackslash" => Emit(KeyPress(rdev::Key::IntlBackslash)),
                "Comma" => Emit(KeyPress(rdev::Key::Comma)),
                "Dot" => Emit(KeyPress(rdev::Key::Dot)),
                "Slash" => Emit(KeyPress(rdev::Key::Slash)),
                "Insert" => Emit(KeyPress(rdev::Key::Insert)),
                "KpReturn" => Emit(KeyPress(rdev::Key::KpReturn)),
                "KpMinus" => Emit(KeyPress(rdev::Key::KpMinus)),
                "KpPlus" => Emit(KeyPress(rdev::Key::KpPlus)),
                "KpMultiply" => Emit(KeyPress(rdev::Key::KpMultiply)),
                "KpDivide" => Emit(KeyPress(rdev::Key::KpDivide)),
                "Kp0" => Emit(KeyPress(rdev::Key::Kp0)),
                "Kp1" => Emit(KeyPress(rdev::Key::Kp1)),
                "Kp2" => Emit(KeyPress(rdev::Key::Kp2)),
                "Kp3" => Emit(KeyPress(rdev::Key::Kp3)),
                "Kp4" => Emit(KeyPress(rdev::Key::Kp4)),
                "Kp5" => Emit(KeyPress(rdev::Key::Kp5)),
                "Kp6" => Emit(KeyPress(rdev::Key::Kp6)),
                "Kp7" => Emit(KeyPress(rdev::Key::Kp7)),
                "Kp8" => Emit(KeyPress(rdev::Key::Kp8)),
                "Kp9" => Emit(KeyPress(rdev::Key::Kp9)),
                "KpDelete" => Emit(KeyPress(rdev::Key::KpDelete)),
                "Function" => Emit(KeyPress(rdev::Key::Function)),

                // Aliases for buttons for user convenience
                "Ctrl" => Emit(KeyPress(rdev::Key::ControlLeft)),
                "Del" => Emit(KeyPress(rdev::Key::Delete)),
                "Enter" => Emit(KeyPress(rdev::Key::Return)),
                "Up" => Emit(KeyPress(rdev::Key::UpArrow)),
                "Down" => Emit(KeyPress(rdev::Key::DownArrow)),
                "Left" => Emit(KeyPress(rdev::Key::LeftArrow)),
                "Right" => Emit(KeyPress(rdev::Key::RightArrow)),
                "WinRight" => Emit(KeyPress(rdev::Key::MetaRight)),
                "SuperRight" => Emit(KeyPress(rdev::Key::MetaRight)),
                "WinLeft" => Emit(KeyPress(rdev::Key::MetaLeft)),
                "SuperLeft" => Emit(KeyPress(rdev::Key::MetaLeft)),
                ";" => Emit(KeyPress(rdev::Key::SemiColon)),
                "\"" => Emit(KeyPress(rdev::Key::Quote)),
                "\\" => Emit(KeyPress(rdev::Key::BackSlash)),
                "," => Emit(KeyPress(rdev::Key::Comma)),
                "." => Emit(KeyPress(rdev::Key::Dot)),
                "/" => Emit(KeyPress(rdev::Key::Slash)),
                "[" => Emit(KeyPress(rdev::Key::LeftBracket)),
                "]" => Emit(KeyPress(rdev::Key::RightBracket)),
                "A" => Emit(KeyPress(rdev::Key::KeyA)),
                "B" => Emit(KeyPress(rdev::Key::KeyB)),
                "C" => Emit(KeyPress(rdev::Key::KeyC)),
                "D" => Emit(KeyPress(rdev::Key::KeyD)),
                "E" => Emit(KeyPress(rdev::Key::KeyE)),
                "F" => Emit(KeyPress(rdev::Key::KeyF)),
                "G" => Emit(KeyPress(rdev::Key::KeyG)),
                "H" => Emit(KeyPress(rdev::Key::KeyH)),
                "I" => Emit(KeyPress(rdev::Key::KeyI)),
                "J" => Emit(KeyPress(rdev::Key::KeyJ)),
                "K" => Emit(KeyPress(rdev::Key::KeyK)),
                "L" => Emit(KeyPress(rdev::Key::KeyL)),
                "M" => Emit(KeyPress(rdev::Key::KeyM)),
                "N" => Emit(KeyPress(rdev::Key::KeyN)),
                "O" => Emit(KeyPress(rdev::Key::KeyO)),
                "P" => Emit(KeyPress(rdev::Key::KeyP)),
                "Q" => Emit(KeyPress(rdev::Key::KeyQ)),
                "R" => Emit(KeyPress(rdev::Key::KeyR)),
                "S" => Emit(KeyPress(rdev::Key::KeyS)),
                "T" => Emit(KeyPress(rdev::Key::KeyT)),
                "U" => Emit(KeyPress(rdev::Key::KeyU)),
                "V" => Emit(KeyPress(rdev::Key::KeyV)),
                "W" => Emit(KeyPress(rdev::Key::KeyW)),
                "X" => Emit(KeyPress(rdev::Key::KeyX)),
                "Y" => Emit(KeyPress(rdev::Key::KeyY)),
                "Z" => Emit(KeyPress(rdev::Key::KeyZ)),
                "1" => Emit(KeyPress(rdev::Key::Num1)),
                "2" => Emit(KeyPress(rdev::Key::Num2)),
                "3" => Emit(KeyPress(rdev::Key::Num3)),
                "4" => Emit(KeyPress(rdev::Key::Num4)),
                "5" => Emit(KeyPress(rdev::Key::Num5)),
                "6" => Emit(KeyPress(rdev::Key::Num6)),
                "7" => Emit(KeyPress(rdev::Key::Num7)),
                "8" => Emit(KeyPress(rdev::Key::Num8)),
                "9" => Emit(KeyPress(rdev::Key::Num9)),
                "0" => Emit(KeyPress(rdev::Key::Num0)),
                "M1" => Emit(ButtonPress(rdev::Button::Left)),
                "M2" => Emit(ButtonPress(rdev::Button::Right)),
                "M3" => Emit(ButtonPress(rdev::Button::Middle)),
                "M4" => Emit(ButtonPress(rdev::Button::Unknown(4))),
                "M5" => Emit(ButtonPress(rdev::Button::Unknown(5))),
                "M6" => Emit(ButtonPress(rdev::Button::Unknown(6))),
                "M7" => Emit(ButtonPress(rdev::Button::Unknown(7))),
                "M8" => Emit(ButtonPress(rdev::Button::Unknown(8))),
                "M9" => Emit(ButtonPress(rdev::Button::Unknown(9))),
                "M10" => Emit(ButtonPress(rdev::Button::Unknown(10))),
                "M11" => Emit(ButtonPress(rdev::Button::Unknown(11))),
                "M12" => Emit(ButtonPress(rdev::Button::Unknown(12))),
                "M13" => Emit(ButtonPress(rdev::Button::Unknown(13))),
                "M14" => Emit(ButtonPress(rdev::Button::Unknown(14))),
                "M15" => Emit(ButtonPress(rdev::Button::Unknown(15))),
                "M16" => Emit(ButtonPress(rdev::Button::Unknown(16))),
                "ScrollUp" => Emit(Wheel { delta_x: 0, delta_y: 1 }),
                "ScrollDown" => Emit(Wheel { delta_x: 0, delta_y: -1 }),
                "MouseX" => Mapping::MouseX(1.0),
                "MouseY" => Mapping::MouseY(1.0),
                "FlickX" => Mapping::FlickX,
                "FlickY" => Mapping::FlickY,
                "Layer1" => Layer(1),
                "Layer2" => Layer(2),
                "Layer3" => Layer(3),
                "Layer4" => Layer(4),
                "Layer5" => Layer(5),
                "Layer6" => Layer(6),
                "Layer7" => Layer(7),
                "Layer8" => Layer(8),
                "Layer9" => Layer(9),
                "Layer10" => Layer(10),
                "Layer11" => Layer(11),
                "Layer12" => Layer(12),
                "Layer13" => Layer(13),
                "Layer14" => Layer(14),
                "Layer15" => Layer(15),
                "Layer16" => Layer(16),
                _ => {
                    println!("Unrecognized key: {:?}", v);
                    Noop
                },
            }
        },
        Value::Array(arr) => {
            if arr.len() == 2 {
                let neg = parse_output(&arr[0]);
                let pos = parse_output(&arr[1]);
                match(neg, pos) {
                    (Emit(e1), Emit(e2)) => {
                        Mapping::NegPos(e1, e2)
                    },
                    (n, p) => {
                        println!("Unable to interpret, ({:?}, {:?}", n, p);
                        Noop
                    },
                }
            } else {
                println!("Array maps must be exactly sized 2, got {:?}", v);
                Noop
            }
        },
        Value::Table(table) => {
            if let Some(action) = table.get("action") {
            if let Value::String(action_str) = action {
                return match action_str.as_str() {
                    "MouseX" => {
                        let sensitivity : f64 = match table.get("sensitivity") {
                            Some(Value::Float(x)) => *x,
                            _ => 1.0,
                        };
                        Mapping::MouseX(sensitivity)
                    },
                    "MouseY" => {
                        let sensitivity : f64 = match table.get("sensitivity") {
                            Some(Value::Float(x)) => *x,
                            _ => 1.0,
                        };
                        Mapping::MouseY(sensitivity)
                    },
                    "NegPos" => {
                        if let Some(Value::Array(arr)) = table.get("keys") {
                            let neg = parse_output(&arr[0]);
                            let pos = parse_output(&arr[1]);
                            if let (Emit(e1), Emit(e2)) = (neg, pos) {
                                return Mapping::NegPos(e1, e2)
                            }
                        }
                        println!("Unable to interpret, ({:?}", action);
                        Noop
                    },
                    "Layer" => {
                        let layer = match table.get("layer") {
                            Some(Value::Integer(x)) => *x as u8,
                            v => panic!("Value out of range: {:?}", v),
                        };
                        Mapping::Layer(layer)
                    },
                    _ => {
                        parse_output(action)
                    },
                }
            } }

            println!("Missing string field: 'action'\n{:?}", v);
            Noop
        },
        v => {
            println!("Unrecognized mapping: {:?}", v);
            Noop
        },
    }
}

fn parse_deadzone(v: &Value, action: Mapping) -> (f64, f64) {
    let default_on =
        match action {
            Mapping::FlickX => { 0.0 },
            Mapping::FlickY => { 0.0 },
            _ => 0.125,
        };
    if let Value::Table(table) = v {
        let on : f64 =
            if let Some(Value::Float(v)) = table.get("deadzoneOn") { *v }
            else if let Some(Value::Float(v)) = table.get("deadzone") { *v }
            else { default_on };
        let off : f64 = 
            if let Some(Value::Float(v)) = table.get("deadzoneOff") { *v }
            else { on * 0.8 };
        return (on, off)
    }
    (default_on, default_on * 0.8 )
}

fn parse_function(v: &Value) -> Option<Function> {
    None
}
