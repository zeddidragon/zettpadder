use rdev;
use crossbeam_channel::{Sender};
use crate::zettpadder::{ZpMsg};

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
    pub deadzone_on: Option<f64>,
    pub deadzone_off: Option<f64>,
}

impl Binding {
    pub fn new(mapping: Mapping) -> Self {
        Binding {
            mapping: mapping,
            deadzone_on: None,
            deadzone_off: None,
        }
    }

    pub fn get_mapping(self, value: f64, prev: f64) -> Option<Mapping> {
        use Mapping::{Emit, NegPos, Layer};
        let on = self.deadzone_on.unwrap_or(0.125);
        let off = self.deadzone_off.unwrap_or(on * 0.8);
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
                        deadzone_on: self.deadzone_on,
                        deadzone_off: self.deadzone_off,
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

fn send(sender: &Sender<ZpMsg>, msg: ZpMsg) {
    match sender.send(msg) {
        Err(err) => {
            println!("Unable to relay event: {:?}\n{:?}", msg, err);
        },
        _ => {},
    };
}
