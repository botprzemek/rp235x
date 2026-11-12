use basketball::{Game, Scorable, Team};
use heapless::String;
use hub75::{Color, DisplayBuffer};

pub struct ScoreboardLayout {
    home_buffer: String<8>,
    guest_buffer: String<8>,
    quarter_buffer: String<8>,
    clock_buffer: String<8>,
}

impl Default for ScoreboardLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl ScoreboardLayout {

    #[must_use]
    pub const fn new() -> Self {
        Self {
            home_buffer: String::new(),
            guest_buffer: String::new(),
            quarter_buffer: String::new(),
            clock_buffer: String::new(),
        }
    }

    pub fn render<const WIDTH: usize, const HEIGHT: usize, const HALF_HEIGHT: usize>(
        &mut self,
        display: &mut DisplayBuffer<WIDTH, HEIGHT, HALF_HEIGHT>,
        game: &Game,
        color: Color,
        is_connected: bool,
    ) {
        use core::fmt::Write;

        self.home_buffer.clear();
        self.guest_buffer.clear();
        self.quarter_buffer.clear();
        self.clock_buffer.clear();

        let total_seconds = game.regulation_millis() / 1000;
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;

        let _ = write!(self.home_buffer, "{:03}", game.score(Team::Home));
        let _ = write!(self.guest_buffer, "{:03}", game.score(Team::Away));
        let _ = write!(self.quarter_buffer, "{:?}", game.quarter());
        let _ = write!(self.clock_buffer, "{minutes:02}:{seconds:02}");

        self.draw_text_layout(
            display,
            color,
            &self.home_buffer,
            &self.guest_buffer,
            &self.quarter_buffer,
            &self.clock_buffer,
        );

        if !is_connected {
            display.set_pixel(WIDTH - 2, 1, hub75::RED);
            display.set_pixel(WIDTH - 1, 1, hub75::RED);
            display.set_pixel(WIDTH - 2, 2, hub75::RED);
            display.set_pixel(WIDTH - 1, 2, hub75::RED);
            display.prepare_bitplanes();
        }
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

        display.fill(color);

        let home_x = 3;
        let guest_x = (WIDTH as i32) - 26;
        let quarter_x = 3;
        let clock_x = 22;

        display.draw_text_mask(home, home_x, 3);
        display.draw_text_mask(guest, guest_x, 3);
        display.draw_text_mask(quarter, quarter_x, 20);
        display.draw_text_mask(clock, clock_x, 20);

        display.prepare_bitplanes();
    }
}
