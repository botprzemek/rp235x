//! Binary-coded modulation (BCM) bitplane buffer for HUB75 LED panels.

/// Precomputed BCM bitplanes for a HUB75 matrix of `WIDTH` columns and `HALF_HEIGHT` scan rows.
#[non_exhaustive]
pub struct BitplaneBuffer<const WIDTH: usize, const HALF_HEIGHT: usize> {
    pub data: [[[u8; WIDTH]; 8]; HALF_HEIGHT],
}

impl<const WIDTH: usize, const HALF_HEIGHT: usize> Default for BitplaneBuffer<WIDTH, HALF_HEIGHT> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const WIDTH: usize, const HALF_HEIGHT: usize> BitplaneBuffer<WIDTH, HALF_HEIGHT> {
    /// Creates a zero-initialized [`BitplaneBuffer`].
    #[must_use]
    pub const fn new() -> Self {
        Self {
            data: [[[0; WIDTH]; 8]; HALF_HEIGHT],
        }
    }

    /// Returns the column bitmasks for the given scan `row` and BCM `bit` plane.
    #[must_use]
    #[inline(always)]
    pub fn row_slice(&self, row: usize, bit: usize) -> &[u8] {
        &self.data[row][bit]
    }
}
