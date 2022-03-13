use crate::mapping::Mapping;
use crate::coords::Coords;

const RING_X_POSITIVE : u8 = 0x01;
const RING_X_NEGATIVE : u8 = 0x02;
const RING_Y_POSITIVE : u8 = 0x04;
const RING_Y_NEGATIVE : u8 = 0x08;
const RING_R_OUTER: u8 = 0x10;

#[derive(Debug, Copy, Clone)]
pub struct Ring {
    pub xy: Coords,
    pub state: u8,
    pub mx: Mapping,
    pub my: Mapping,
    pub mr: Mapping,
    pub deadzone_on: f64,
    pub deadzone_off: f64,
    pub ring_on: f64,
    pub ring_off: f64,
}

#[derive(Default)]
pub struct RingFactory {
    mx: Option<Mapping>,
    my: Option<Mapping>,
    mr: Option<Mapping>,
    deadzone_on: Option<f64>,
    deadzone_off: Option<f64>,
    ring_on: Option<f64>,
    ring_off: Option<f64>,
}

impl RingFactory {
    pub fn build(&self) -> Result<Ring, ()> {
        let mx = self.mx.unwrap_or(Mapping::Noop);
        let my = self.my.unwrap_or(Mapping::Noop);
        let mr = self.mr.unwrap_or(Mapping::Noop);
        let dz_on = self.deadzone_on.unwrap_or(0.125);
        let dz_off = self.deadzone_off.unwrap_or(dz_on * 0.8);

        let ring_on = self.ring_on.unwrap_or((1.0 - dz_on) * 0.5 + dz_on);
        let ring_off = self.ring_off.unwrap_or(ring_on * 0.8);

        Ok(Ring {
            xy: Coords::default(),
            state: 0,
            mx: mx,
            my: my,
            mr: mr,
            deadzone_on: dz_on,
            deadzone_off: dz_off,
            ring_on: ring_on,
            ring_off: ring_off,
        })
    }

    pub fn clear(&mut self) {
        self.mx = None;
        self.my = None;
        self.mr = None;
        self.deadzone_on = None;
        self.deadzone_off = None;
        self.ring_on = None;
        self.ring_off = None;
    }

    pub fn bind_x(&mut self, m: Mapping) -> &Self {
        self.mx = Some(m);
        self
    }

    pub fn bind_y(&mut self, m: Mapping) -> &Self {
        self.my = Some(m);
        self
    }

    pub fn bind_r(&mut self, m: Mapping) -> &Self {
        self.mr = Some(m);
        self
    }

    pub fn with_deadzone_on(&mut self, v: f64) -> &Self {
        self.deadzone_on = Some(v);
        self
    }

    pub fn with_deadzone_off(&mut self, v: f64) -> &Self {
        self.deadzone_off = Some(v);
        self
    }

    pub fn with_ring_on(&mut self, v: f64) -> &Self {
        self.ring_on = Some(v);
        self
    }

    pub fn with_ring_off(&mut self, v: f64) -> &Self {
        self.ring_off = Some(v);
        self
    }
}

impl Ring {
    pub fn nudge_x(&mut self, value: f64) -> Option<Mapping> {
        use Mapping::{Emit, NegPos, Layer};
        self.xy.x = value;
        let on = self.deadzone_on;
        let off = self.deadzone_off;
        let is_pos = self.state & RING_X_POSITIVE == RING_X_POSITIVE;
        let is_neg = self.state & RING_X_NEGATIVE == RING_X_NEGATIVE;
        let is_off = !(is_pos || is_neg);
        match self.mx {
            Emit(_) => {
                let aval = value.abs();
                if is_off && aval >= on {
                    self.state ^= RING_X_POSITIVE;
                    Some(self.mx)
                } else if !is_off && aval <= off {
                    self.state ^= RING_X_POSITIVE;
                    self.mx.released()
                } else {
                    None
                }
            },
            Layer(_) => {
                let aval = value.abs();
                if is_off && aval >= on {
                    self.state ^= RING_X_POSITIVE;
                    Some(self.mx)
                } else if is_off && aval <= off {
                    self.state ^= RING_X_POSITIVE;
                    Some(Mapping::Layer(0))
                } else {
                    None
                }
            },
            NegPos(neg, pos) => {
                if is_pos && value < off {
                    self.state &= !RING_X_POSITIVE;
                    Mapping::Emit(pos).released()
                } else if is_neg && value > -off {
                    self.state &= !RING_X_NEGATIVE;
                    Mapping::Emit(neg).released()
                } else if value >= on {
                    self.state |= RING_X_POSITIVE;
                    Some(Mapping::Emit(pos))
                } else if value <= -on {
                    self.state |= RING_X_NEGATIVE;
                    Some(Mapping::Emit(neg))
                } else {
                    None
                }
            },
            _ => {
                Some(self.mx)
            }
        }
    }

    pub fn nudge_y(&mut self, value: f64) -> Option<Mapping> {
        use Mapping::{Emit, NegPos, Layer};
        self.xy.y = value;
        let on = self.deadzone_on;
        let off = self.deadzone_off;
        let is_pos = self.state & RING_Y_POSITIVE == RING_Y_POSITIVE;
        let is_neg = self.state & RING_Y_NEGATIVE == RING_Y_NEGATIVE;
        let is_off = !(is_pos || is_neg);
        match self.my {
            Emit(_) => {
                let aval = value.abs();
                if is_off && aval >= on {
                    self.state ^= RING_Y_POSITIVE;
                    Some(self.my)
                } else if !is_off && aval <= off {
                    self.state ^= RING_Y_POSITIVE;
                    self.my.released()
                } else {
                    None
                }
            },
            Layer(_) => {
                let aval = value.abs();
                if is_off && aval >= on {
                    self.state ^= RING_Y_POSITIVE;
                    Some(self.my)
                } else if is_off && aval <= off {
                    self.state ^= RING_Y_POSITIVE;
                    Some(Mapping::Layer(0))
                } else {
                    None
                }
            },
            NegPos(neg, pos) => {
                if is_pos && value < off {
                    self.state &= !RING_Y_POSITIVE;
                    Mapping::Emit(pos).released()
                } else if is_neg && value > -off {
                    self.state &= !RING_Y_NEGATIVE;
                    Mapping::Emit(neg).released()
                } else if value >= on {
                    self.state |= RING_Y_POSITIVE;
                    Some(Mapping::Emit(pos))
                } else if value <= -on {
                    self.state |= RING_Y_NEGATIVE;
                    Some(Mapping::Emit(neg))
                } else {
                    None
                }
            },
            _ => {
                Some(self.my)
            }
        }
    }

    pub fn check_ring(&mut self) -> Option<Mapping> {
        use Mapping::{Emit, Layer};
        let is_outer = self.state & RING_R_OUTER == RING_R_OUTER;
        let value = self.xy.len();
        match self.mr {
            Emit(_) => {
                if !is_outer && value >= self.ring_on {
                    self.state ^= RING_R_OUTER;
                    Some(self.mr)
                } else if is_outer && value <= self.ring_off {
                    self.state ^= RING_R_OUTER;
                    self.mr.released()
                } else {
                    None
                }
            },
            Layer(_) => {
                if !is_outer && value >= self.ring_on {
                    self.state ^= RING_R_OUTER;
                    Some(self.mr)
                } else if is_outer && value <= self.ring_off {
                    self.state ^= !RING_R_OUTER;
                    Some(Mapping::Layer(0))
                } else {
                    None
                }
            },
            _ => {
                Some(self.mr)
            }
        }
    }
}
