use basketball::{Game, Scorable, Team};
use core::fmt::Write;
use heapless::String;
use hub75::{Color, DisplayBuffer};

pub struct ScoreboardLayout {
    home_buf: String<8>,
    guest_buf: String<8>,
    quarter_buf: String<8>,
    clock_buf: String<8>,
}

impl Default for ScoreboardLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl ScoreboardLayout {
    pub const fn new() -> Self {
        Self {
            home_buf: String::new(),
            guest_buf: String::new(),
            quarter_buf: String::new(),
            clock_buf: String::new(),
        }
    }

    pub fn render<const WIDTH: usize, const HEIGHT: usize, const HALF_HEIGHT: usize>(
        &mut self,
        display: &mut DisplayBuffer<WIDTH, HEIGHT, HALF_HEIGHT>,
        game: &Game,
        color: Color,
    ) {
        self.home_buf.clear();
        self.guest_buf.clear();
        self.quarter_buf.clear();
        self.clock_buf.clear();

        let total_seconds = game.regulation_millis() / 1000;
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;

        let _ = write!(self.home_buf, "{:03}", game.get_score(Team::Home));
        let _ = write!(self.guest_buf, "{:03}", game.get_score(Team::Away));
        let _ = write!(self.quarter_buf, "{:?}", game.quarter());
        let _ = write!(self.clock_buf, "{:02}:{:02}", minutes, seconds);

        self.draw_text_layout(
            display,
            color,
            &self.home_buf,
            &self.guest_buf,
            &self.quarter_buf,
            &self.clock_buf,
        );
    }

    pub fn draw_text_layout<const WIDTH: usize, const HEIGHT: usize, const HALF_HEIGHT: usize>(
        &self,
        display: &mut DisplayBuffer<WIDTH, HEIGHT, HALF_HEIGHT>,
        color: Color,
        home: &str,
        guest: &str,
        quarter: &str,
        clock: &str,
    ) {
        display.clear();

        let home_x = 3;
        let guest_x = (WIDTH as i32) - 26;
        let quarter_x = 3;
        let clock_x = 22;

        display.draw_text(home, home_x, 3, color);
        display.draw_text(guest, guest_x, 3, color);
        display.draw_text(quarter, quarter_x, 20, color);
        display.draw_text(clock, clock_x, 20, color);

        display.prepare_bitplanes();
    }
}
