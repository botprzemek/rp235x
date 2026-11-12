use basketball::{Game, Scorable};
use hub75::{Color, DisplayBuffer, RED};

use crate::core1::ScoreboardLayout;

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

    #[must_use]
    pub fn new() -> Self {
        let mut display = DisplayBuffer::<64, 32, 16>::default();
        display.set_brightness(20);
        Self {
            display,
            layout: ScoreboardLayout::new(),
        }
    }

    pub fn render_frame(&mut self, game: &Game) {
        let color: Color = RED;

        self.layout.render(&mut self.display, game, color, true);

        self.print_terminal_view(game);
    }

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
