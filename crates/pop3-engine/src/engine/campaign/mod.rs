pub mod objectives;

pub const TOTAL_LEVELS: u32 = 25;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CampaignEvent {
    None,
    Victory(u32),
    Defeat(u32),
}

#[derive(Debug, Clone)]
pub struct CampaignState {
    pub current_level: u32,
    pub completed_levels: [bool; TOTAL_LEVELS as usize],
    pub campaign_complete: bool,
    /// Whether victory/defeat has been acknowledged (to avoid repeated events)
    outcome_acknowledged: bool,
}

impl CampaignState {
    pub fn new() -> Self {
        Self {
            current_level: 1,
            completed_levels: [false; TOTAL_LEVELS as usize],
            campaign_complete: false,
            outcome_acknowledged: false,
        }
    }

    /// Check victory/defeat flags and return campaign event.
    /// Called after each tick when in campaign mode.
    pub fn check_progress(&mut self, has_won: bool, has_lost: bool) -> CampaignEvent {
        if self.outcome_acknowledged {
            return CampaignEvent::None;
        }
        if has_won {
            self.outcome_acknowledged = true;
            CampaignEvent::Victory(self.current_level)
        } else if has_lost {
            self.outcome_acknowledged = true;
            CampaignEvent::Defeat(self.current_level)
        } else {
            CampaignEvent::None
        }
    }

    /// Advance to next level after victory.
    pub fn advance_level(&mut self) {
        let idx = (self.current_level - 1) as usize;
        if idx < TOTAL_LEVELS as usize {
            self.completed_levels[idx] = true;
        }
        if self.current_level < TOTAL_LEVELS {
            self.current_level += 1;
        } else {
            self.campaign_complete = true;
        }
        self.outcome_acknowledged = false;
    }

    /// Retry the current level after defeat.
    pub fn retry_level(&mut self) {
        self.outcome_acknowledged = false;
    }

    pub fn is_level_completed(&self, level: u32) -> bool {
        let idx = (level - 1) as usize;
        self.completed_levels.get(idx).copied().unwrap_or(false)
    }

    pub fn total_levels(&self) -> u32 {
        TOTAL_LEVELS
    }

    /// Set current level (for level select or load game).
    pub fn set_level(&mut self, level: u32) {
        self.current_level = level.clamp(1, TOTAL_LEVELS);
        self.outcome_acknowledged = false;
    }

    /// Level data file name for current level.
    /// Original format: levl20XX.dat where XX is zero-padded level number.
    pub fn level_filename(&self) -> String {
        format!("levl20{:02}.dat", self.current_level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_starts_at_level_1() {
        let state = CampaignState::new();
        assert_eq!(state.current_level, 1);
        assert!(!state.campaign_complete);
        for i in 0..TOTAL_LEVELS as usize {
            assert!(!state.completed_levels[i]);
        }
    }

    #[test]
    fn test_advance_level_from_1() {
        let mut state = CampaignState::new();
        state.advance_level();
        assert_eq!(state.current_level, 2);
        assert!(state.completed_levels[0]); // level 1 completed
    }

    #[test]
    fn test_advance_level_from_25_sets_complete() {
        let mut state = CampaignState::new();
        state.current_level = 25;
        state.advance_level();
        assert_eq!(state.current_level, 25); // stays at 25
        assert!(state.campaign_complete);
        assert!(state.completed_levels[24]); // level 25 completed
    }

    #[test]
    fn test_check_progress_won() {
        let mut state = CampaignState::new();
        let event = state.check_progress(true, false);
        assert_eq!(event, CampaignEvent::Victory(1));
    }

    #[test]
    fn test_check_progress_lost() {
        let mut state = CampaignState::new();
        let event = state.check_progress(false, true);
        assert_eq!(event, CampaignEvent::Defeat(1));
    }

    #[test]
    fn test_check_progress_no_flags() {
        let mut state = CampaignState::new();
        let event = state.check_progress(false, false);
        assert_eq!(event, CampaignEvent::None);
    }

    #[test]
    fn test_check_progress_acknowledged_returns_none() {
        let mut state = CampaignState::new();
        state.check_progress(true, false); // acknowledge victory
        let event = state.check_progress(true, false); // second call
        assert_eq!(event, CampaignEvent::None);
    }

    #[test]
    fn test_retry_level_keeps_current() {
        let mut state = CampaignState::new();
        state.current_level = 5;
        state.retry_level();
        assert_eq!(state.current_level, 5);
    }

    #[test]
    fn test_is_level_completed() {
        let mut state = CampaignState::new();
        assert!(!state.is_level_completed(1));
        state.advance_level(); // completes level 1
        assert!(state.is_level_completed(1));
        assert!(!state.is_level_completed(2));
    }

    #[test]
    fn test_total_levels() {
        let state = CampaignState::new();
        assert_eq!(state.total_levels(), 25);
    }

    #[test]
    fn test_level_filename() {
        let mut state = CampaignState::new();
        assert_eq!(state.level_filename(), "levl2001.dat");
        state.current_level = 15;
        assert_eq!(state.level_filename(), "levl2015.dat");
    }

    #[test]
    fn test_set_level() {
        let mut state = CampaignState::new();
        state.set_level(10);
        assert_eq!(state.current_level, 10);
        state.set_level(0); // clamped to 1
        assert_eq!(state.current_level, 1);
        state.set_level(100); // clamped to 25
        assert_eq!(state.current_level, 25);
    }
}
