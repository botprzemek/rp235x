pub trait OutputPin {
    fn set_high(&mut self);
    fn set_low(&mut self);

    #[inline(always)]
    fn set_level(&mut self, high: bool) {
        if high {
            self.set_high();
        } else {
            self.set_low();
        }
    }
}

pub struct LedOutputs<Output> {
    pub r1: Output,
    pub g1: Output,
    pub b1: Output,
    pub r2: Output,
    pub g2: Output,
    pub b2: Output,

    pub a: Output,
    pub b: Output,
    pub c: Output,
    pub d: Output,
    pub _e: Output,

    pub clk: Output,
    pub lat: Output,
    pub oe: Output,
}
