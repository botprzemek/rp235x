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
    pub(crate) r1: P,
    pub(crate) g1: P,
    pub(crate) b1: P,
    pub(crate) r2: P,
    pub(crate) g2: P,
    pub(crate) b2: P,

    pub(crate) a: P,
    pub(crate) b: P,
    pub(crate) c: P,
    pub(crate) d: P,
    pub(crate) e: P,

    pub(crate) clk: P,
    pub(crate) lat: P,
    pub(crate) oe: P,
}
