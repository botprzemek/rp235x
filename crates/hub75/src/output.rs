#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinLevel {
    Low,
    High,
}

impl From<bool> for PinLevel {
    fn from(is_high: bool) -> Self {
        if is_high { Self::High } else { Self::Low }
    }
}

pub trait OutputPin {

    fn set_high(&mut self);

    fn set_low(&mut self);

    #[inline(always)]
    fn set_level(&mut self, level: PinLevel) {
        match level {
            PinLevel::High => self.set_high(),
            PinLevel::Low => self.set_low(),
        }
    }
}

#[non_exhaustive]
pub struct LedOutputs<P> {
    pub r1: P,
    pub g1: P,
    pub b1: P,
    pub r2: P,
    pub g2: P,
    pub b2: P,

    pub a: P,
    pub b: P,
    pub c: P,
    pub d: P,
    pub e: P,

    pub clk: P,
    pub lat: P,
    pub oe: P,
}
