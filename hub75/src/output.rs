//! Output pin trait and HUB75 signal bundle definition.

/// Logic level of a digital output pin.
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

/// Digital output pin abstraction for driving HUB75 signals.
pub trait OutputPin {
    /// Drives the pin to a logic high state.
    fn set_high(&mut self);

    /// Drives the pin to a logic low state.
    fn set_low(&mut self);

    /// Drives the pin to the given [`PinLevel`].
    #[inline(always)]
    fn set_level(&mut self, level: PinLevel) {
        match level {
            PinLevel::High => self.set_high(),
            PinLevel::Low => self.set_low(),
        }
    }
}

/// Collection of digital output pins required to drive a HUB75 LED panel.
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
