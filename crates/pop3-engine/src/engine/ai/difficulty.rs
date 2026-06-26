/// Difficulty scaling constants from original binary.
/// Original: COMPUTER_MANA_ADJUST @ 0x005aa8b9

pub const DEFAULT_MANA_ADJUST: u32 = 100; // 100% = no change

/// Apply mana adjustment for computer tribes.
/// adjusted_mana = base_mana * adjust / 100
pub fn apply_mana_adjust(base_mana: u32, mana_adjust: u32) -> u32 {
    (base_mana as u64 * mana_adjust as u64 / 100) as u32
}

/// Training cost bands: diminishing returns as unit count increases.
/// Original binary uses 6 bands with increasing multipliers.
pub fn training_cost_band(unit_count: u32) -> u32 {
    match unit_count {
        0..=3 => 100,
        4..=7 => 120,
        8..=11 => 140,
        12..=15 => 160,
        16..=20 => 180,
        _ => 200,
    }
}

/// Apply training cost with band multiplier.
/// base_cost * band_multiplier / 100
pub fn adjusted_training_cost(base_cost: u32, current_count: u32) -> u32 {
    base_cost * training_cost_band(current_count) / 100
}

/// Per-unit-type AI training mana costs (separate from human costs).
/// Original: CP_TRAIN_MANA_* constants
pub struct AiTrainingCosts {
    pub warrior: u32,
    pub spy: u32,
    pub preacher: u32,
    pub super_warrior: u32,
}

impl AiTrainingCosts {
    pub fn default_costs() -> Self {
        Self {
            warrior: 3000,
            spy: 5000,
            preacher: 8000,
            super_warrior: 12000,
        }
    }
}

pub struct DifficultyScaling {
    pub mana_adjust: u32,
    pub training_costs: AiTrainingCosts,
}

impl DifficultyScaling {
    pub fn normal() -> Self {
        Self {
            mana_adjust: DEFAULT_MANA_ADJUST,
            training_costs: AiTrainingCosts::default_costs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_mana_adjust_150_percent() {
        assert_eq!(apply_mana_adjust(1000, 150), 1500);
    }

    #[test]
    fn apply_mana_adjust_no_change() {
        assert_eq!(apply_mana_adjust(1000, 100), 1000);
    }

    #[test]
    fn training_cost_band_0_to_3() {
        assert_eq!(training_cost_band(0), 100);
        assert_eq!(training_cost_band(3), 100);
    }

    #[test]
    fn training_cost_band_4_to_7() {
        assert_eq!(training_cost_band(4), 120);
        assert_eq!(training_cost_band(5), 120);
        assert_eq!(training_cost_band(7), 120);
    }

    #[test]
    fn training_cost_band_21_plus() {
        assert_eq!(training_cost_band(21), 200);
        assert_eq!(training_cost_band(50), 200);
    }

    #[test]
    fn training_cost_band_all_ranges() {
        assert_eq!(training_cost_band(8), 140);
        assert_eq!(training_cost_band(12), 160);
        assert_eq!(training_cost_band(16), 180);
    }

    #[test]
    fn adjusted_training_cost_applies_band() {
        // base 3000, count 5 => band 120% => 3600
        assert_eq!(adjusted_training_cost(3000, 5), 3600);
    }

    #[test]
    fn default_training_costs() {
        let costs = AiTrainingCosts::default_costs();
        assert_eq!(costs.warrior, 3000);
        assert_eq!(costs.spy, 5000);
        assert_eq!(costs.preacher, 8000);
        assert_eq!(costs.super_warrior, 12000);
    }

    #[test]
    fn difficulty_scaling_normal() {
        let d = DifficultyScaling::normal();
        assert_eq!(d.mana_adjust, 100);
    }
}
