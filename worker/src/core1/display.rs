#[derive(Clone, Copy)]
pub struct DisplayState {
    pub home_score: u16,
    pub away_score: u16,
    pub quarter: game::Quarter,
    pub minutes: u32,
    pub seconds: u32,
    pub _hundredths: u32,
}

impl Default for DisplayState {
    fn default() -> Self {
        Self {
            home_score: 0,
            away_score: 0,
            quarter: game::Quarter::None,
            minutes: 0,
            seconds: 0,
            _hundredths: 0,
        }
    }
}
