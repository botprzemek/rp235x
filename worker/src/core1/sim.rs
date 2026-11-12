//! Simulated HUB75 LED display runner for Linux / desktop.
//!
//! Renders the exact same HUB75 64x32 display buffer using ASCII / ANSI terminal art,
//! visualising the LED matrix layout in real-time.

use basketball::{Game, Scorable};
use hub75::{Color, DisplayBuffer, RED};

use crate::core1::ScoreboardLayout;

/// Simulated HUB75 64x32 display renderer for Linux.
pub struct LinuxScoreboardSimulator {
    display: DisplayBuffer<64, 32, 16>,
    layout: ScoreboardLayout,
}

impl Default for LinuxScoreboardSimulator {
    fn default() -> Self {
        Self::new()
    }
}

impl LinuxScoreboardSimulator {
    /// Creates a new [`LinuxScoreboardSimulator`].
    #[must_use]
    pub fn new() -> Self {
        let mut display = DisplayBuffer::<64, 32, 16>::default();
        display.set_brightness(20);
        Self {
            display,
            layout: ScoreboardLayout::new(),
        }
    }

    /// Renders `game` using the exact RP235x [`ScoreboardLayout`], prepares bitplanes,
    /// and formats a terminal view.
    pub fn render_frame(&mut self, game: &Game) {
        let color: Color = RED;

        // Use the exact firmware layout logic:
        self.layout.render(&mut self.display, game, color, true);

        // Render terminal representation of the scoreboard
        self.print_terminal_view(game);
    }

    /// Draws a formatted ANSI representation of the scoreboard state and HUB75 buffer info.
    fn print_terminal_view(&self, game: &Game) {
        let total_seconds = game.regulation_millis() / 1000;
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        let tenths = (game.regulation_millis() % 1000) / 100;

        let time_str = if minutes == 0 && total_seconds < 60 {
            format!("{seconds:02}.{tenths}")
        } else {
            format!("{minutes:02}:{seconds:02}")
        };

        let scores = game.scores();
        let state_str = match game.state() {
            basketball::State::Running => "RUNNING ▶",
            basketball::State::Paused => "PAUSED ⏸",
            basketball::State::QuarterEnd => "QUARTER END ⏹",
            basketball::State::GameEnd => "GAME END 🏁",
            basketball::State::Idle | _ => "IDLE",
        };

        println!(
            "\r[HUB75 SIMULATOR] [GAME #{}] Q{:?} | TIME: {} | HOME {:>3} : {:<3} AWAY | [{}]",
            game.id(),
            game.quarter(),
            time_str,
            scores.home,
            scores.away,
            state_str
        );
    }
}
