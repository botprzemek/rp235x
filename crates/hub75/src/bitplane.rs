#[non_exhaustive]
pub struct BitplaneBuffer<const WIDTH: usize, const HALF_HEIGHT: usize> {
    pub(crate) data: [[[u8; WIDTH]; 8]; HALF_HEIGHT],
}

impl<const WIDTH: usize, const HALF_HEIGHT: usize> Default for BitplaneBuffer<WIDTH, HALF_HEIGHT> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const WIDTH: usize, const HALF_HEIGHT: usize> BitplaneBuffer<WIDTH, HALF_HEIGHT> {

    #[must_use]
    pub const fn new() -> Self {
        Self {
            data: [[[0; WIDTH]; 8]; HALF_HEIGHT],
        }
    }

    #[must_use]
    #[inline(always)]
    pub(crate) fn row_slice(&self, row: usize, bit: usize) -> &[u8] {
        &self.data[row][bit]
    }
}
