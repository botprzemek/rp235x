#[cfg(test)]
mod tests {
    use crate::{Discipline, Match, ModifiableScore, Quarter, Scorable, Score, State, Team};
    use core::time::Duration;

    #[test]
    fn test_match_type_state_lifecycle() {
        let match_idle = Match::new(Discipline::Fiba5v5);

        // Can score in Idle
        let mut match_idle = match_idle;
        match_idle.add_score(Team::Home, Score::FieldGoal3);
        assert_eq!(match_idle.game().score(Team::Home), 3);

        // Start match -> enters Running
        let mut match_running = match_idle.start();
        match_running.add_score(Team::Away, Score::FieldGoal2);
        assert_eq!(match_running.game().score(Team::Away), 2);

        // Tick 1s
        match_running = match_running
            .tick(Duration::from_secs(1))
            .expect("running continues");

        // Pause match -> enters Paused
        let mut match_paused = match_running.pause();
        match_paused.add_score(Team::Home, Score::FreeThrow);
        assert_eq!(match_paused.game().score(Team::Home), 4);

        // Resume -> Running
        let match_running = match_paused.resume();

        // Tick to quarter end -> enters QuarterBreak
        let rem = match_running.game().regulation_millis();
        let match_qbreak = match_running
            .tick(Duration::from_millis(rem as u64 + 100))
            .expect_err("quarter ends");
        assert_eq!(match_qbreak.game().state(), State::QuarterEnd);

        // Advance to Q2 -> Paused
        let match_q2_paused = match_qbreak.advance_quarter().expect("enters next quarter");
        assert_eq!(match_q2_paused.game().quarter(), Quarter::Q2);
    }
}
