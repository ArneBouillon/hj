use std::cmp::Ordering;

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct NonNan(f32);

impl NonNan {
    pub fn new(val: f32) -> Option<NonNan> {
        if val.is_nan() {
            None
        } else {
            Some(NonNan(val))
        }
    }

    pub fn value(self) -> f32 {
        self.0
    }

    pub fn zero() -> Self {
        Self::new(0.).unwrap()
    }

    pub fn try_add(self, other: Self) -> Option<Self> {
        let sum = self.0 + other.0;
        if sum.is_nan() {
            None
        } else {
            Some(NonNan(sum))
        }
    }
}

impl Eq for NonNan {}

impl Ord for NonNan {
    fn cmp(&self, other: &NonNan) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
