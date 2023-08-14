use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrderF32(f32);

impl From<f32> for OrderF32 {
    fn from(value: f32) -> Self {
        OrderF32(value)
    }
}

impl From<OrderF32> for f32 {
    fn from(value: OrderF32) -> Self {
        value.0
    }
}

impl Hash for OrderF32 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let bits: u32 = self.0.to_bits();
        bits.hash(state);
    }
}
