pub struct BitplaneBuffer<const WIDTH: usize, const HALF_HEIGHT: usize> {
    pub data: [[[u8; WIDTH]; 8]; HALF_HEIGHT],
}

impl<const WIDTH: usize, const HALF_HEIGHT: usize> BitplaneBuffer<WIDTH, HALF_HEIGHT> {
    pub const fn new() -> Self {
        Self {
            data: [[[0; WIDTH]; 8]; HALF_HEIGHT],
        }
    }

    #[inline(always)]
    pub fn row_slice(&self, row: usize, bit: usize) -> &[u8] {
        &self.data[row][bit]
    }
}
