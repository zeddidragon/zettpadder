use rdev;

#[derive(Debug, Copy, Clone)]
pub enum Mapping {
    // Specifically do nothing, but still capture the input
    Noop,
    // Emit and rdev Event
    Emit(rdev::EventType),
    // Initiate mouse velocity
    MouseX(f64),
    MouseY(f64),
    // Flick Stick. Used to turn towards your stick direction.
    FlickX(f64),
    FlickY(f64),
    // Compass movement. Used to warp to a point on screen from center offset by radius.
    CompassX(f64, f64),
    CompassY(f64, f64),
    // For axes.
    // Negative values on the axis emits the first event.
    // Positive values emit the second event instead.
    NegPos(rdev::EventType, rdev::EventType),
    // Switch layer for as long as button is held
    Layer(u8),
    // Send value to macro with specified ID
    Trigger(usize),
    // Send value to ring with specified ID
    RingX(usize),
    RingY(usize),
    // Signifies a delay in amacro
    Delay, 
}

impl Mapping {
    pub fn released(self) -> Option<Mapping> {
        use Mapping::{
            Emit,
            Layer,
            MouseX,
            MouseY,
            FlickX,
            FlickY,
            CompassX,
            CompassY,
        };
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
            Layer(_) => {
                Some(Layer(0))
            },
            Emit(KeyRelease(_)) => { None },
            Emit(ButtonRelease(_)) => { None },
            Emit(Wheel { delta_x: _, delta_y: _ }) => { None },
            MouseX(_) => {
                Some(MouseX(0.0))
            },
            MouseY(_) => {
                Some(MouseY(0.0))
            },
            FlickX(_) => {
                Some(FlickX(0.0))
            },
            FlickY(_) => {
                Some(FlickY(0.0))
            },
            CompassX(x, _) => {
                Some(CompassX(x, 0.0))
            },
            CompassY(y, _) => {
                Some(CompassY(y, 0.0))
            },
            Mapping::Delay => {
                Some(Mapping::Delay)
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

fn anti_deadzone(v: f64, dz: f64) -> f64 {
    v.signum() * (v.abs() - dz) / (1.0 - dz)
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
            Mapping::MouseX(v) => {
                if value.abs() >= on {
                    Some(Mapping::MouseX(v * anti_deadzone(value, on)))
                } else {
                    self.mapping.released()
                }
            },
            Mapping::MouseY(v) => {
                if value.abs() >= on {
                    Some(Mapping::MouseY(v * anti_deadzone(value, on)))
                } else {
                    self.mapping.released()
                }
            },
            Mapping::FlickX(v) => {
                if value.abs() >= on {
                    Some(Mapping::FlickX(v * anti_deadzone(value, on)))
                } else {
                    self.mapping.released()
                }
            },
            Mapping::FlickY(v) => {
                if value.abs() >= on {
                    Some(Mapping::FlickY(v * anti_deadzone(value, on)))
                } else {
                    self.mapping.released()
                }
            },
            Mapping::CompassX(x, r) => {
                if value.abs() >= on {
                    let r = r * anti_deadzone(value, on);
                    Some(Mapping::CompassX(x, r))
                } else {
                    self.mapping.released()
                }
            },
            Mapping::CompassY(y, r) => {
                if value.abs() >= on {
                    let r = r * anti_deadzone(value, on);
                    Some(Mapping::CompassY(y, r))
                } else {
                    self.mapping.released()
                }
            },
            _ => {
                Some(self.mapping)
            }
        }
    }
}
