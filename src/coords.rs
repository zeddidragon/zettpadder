use std::ops;

#[derive(Debug, Copy, Clone)]
pub struct Coords {
    pub x: f64,
    pub y: f64,
}

impl Coords {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
        } }

    pub fn len(&self) -> f64 {
        self.x.powi(2) + self.y.powi(2).sqrt()
    }

    pub fn manhattan(&self) -> f64 {
        self.x.abs() + self.y.abs()
    }

    pub fn angle(&self) -> f64 {
        self.x.atan2(-self.y)
    }
}

impl ops::Add<Coords> for Coords {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Coords {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl ops::AddAssign<Coords> for Coords {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl ops::Mul<f64> for Coords {
    type Output = Self;

    fn mul(self, other: f64) -> Self {
        Coords {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl ops::MulAssign<f64> for Coords {
    fn mul_assign(&mut self, other: f64) {
        self.x *= other;
        self.y *= other;
    }
}

impl ops::Div<f64> for Coords {
    type Output = Self;

    fn div(self, other: f64) -> Self {
        Coords {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl ops::DivAssign<f64> for Coords {
    fn div_assign(&mut self, other: f64) {
        self.x /= other;
        self.y /= other;
    }
}
