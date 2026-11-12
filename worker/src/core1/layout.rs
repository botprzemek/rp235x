//! Fixed-capacity scoreboard text formatter for `no_std` HUB75 rendering on Core 1.

use basketball::{Game, Scorable, Team};
use heapless::String;
use hub75::{Color, DisplayBuffer};

/// Stack-allocated text buffers for formatting a [`Game`] onto a 64x32 HUB75 panel.
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
    /// Creates a new empty [`ScoreboardLayout`].
    #[must_use]
    pub const fn new() -> Self {
        Self {
            home_buffer: String::new(),
            guest_buffer: String::new(),
            quarter_buffer: String::new(),
            clock_buffer: String::new(),
        }
    }

    /// Formats `game` scores, quarter, and clock and renders them into `display`.
    /// When `is_connected` is false, draws a subtle blinking/static connection dot to indicate offline state.
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

        // Visual indicator of network connection state:
        // When disconnected/offline, turn on a 2x2 red pixel block in the top-right corner
        if !is_connected {
            display.set_pixel(WIDTH - 2, 1, hub75::RED);
            display.set_pixel(WIDTH - 1, 1, hub75::RED);
            display.set_pixel(WIDTH - 2, 2, hub75::RED);
            display.set_pixel(WIDTH - 1, 2, hub75::RED);
            display.prepare_bitplanes();
        }
    }

    /// Clears `display`, draws the scoreboard text fields, and prepares BCM bitplanes.
    /// If `color` is specified, fills background with `color` and cuts out text glyphs as a mask (dark text on bright background).
    pub fn draw_text_layout<const WIDTH: usize, const HEIGHT: usize, const HALF_HEIGHT: usize>(
        &self,
        display: &mut DisplayBuffer<WIDTH, HEIGHT, HALF_HEIGHT>,
        color: Color,
        home: &str,
        guest: &str,
        quarter: &str,
        clock: &str,
    ) {
        // Background mask: fill with background color
        display.fill(color);

        let home_x = 3;
        let guest_x = (WIDTH as i32) - 26;
        let quarter_x = 3;
        let clock_x = 22;

        // Cut out text glyphs (mask / off pixels)
        display.draw_text_mask(home, home_x, 3);
        display.draw_text_mask(guest, guest_x, 3);
        display.draw_text_mask(quarter, quarter_x, 20);
        display.draw_text_mask(clock, clock_x, 20);

        display.prepare_bitplanes();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoreboard_render_pixels() {
        let mut display = DisplayBuffer::<64, 32, 16>::default();
        let mut layout = ScoreboardLayout::new();
        let game = Game::empty();
        layout.render(&mut display, &game, hub75::RED, true);

        println!("home: '{}'", layout.home_buffer);
        println!("guest: '{}'", layout.guest_buffer);
        println!("quarter: '{}'", layout.quarter_buffer);
        println!("clock: '{}'", layout.clock_buffer);
    }
}
