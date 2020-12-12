#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Number {
    value: f64,
}

impl Number {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl From<Number> for f64 {
    fn from(number: Number) -> Self {
        number.value
    }
}

impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Self { value }
    }
}