/// Target scoring constants matching AI_FindBestAttackTarget @ 0x004b9770
pub const SCORE_SHAMAN: i32 = 1000;
pub const SCORE_SUPER_WARRIOR: i32 = 20;
pub const SCORE_PREACHER: i32 = 15;
pub const SCORE_WARRIOR: i32 = 10;
pub const SCORE_SPY: i32 = 5;
pub const SCORE_EXPOSED_SHAMAN_BONUS: i32 = 500;
pub const EXPOSED_SHAMAN_DEFENDER_THRESHOLD: u32 = 3;

pub const SCORE_BUILDING_SUPER_TRAIN: i32 = 250;
pub const SCORE_BUILDING_TEMPLE: i32 = 200;
pub const SCORE_BUILDING_WARRIOR_TRAIN: i32 = 180;
pub const SCORE_BUILDING_DRUM_TOWER: i32 = 150;
pub const SCORE_BUILDING_DEFAULT: i32 = 50;

pub const DISTANCE_PENALTY_DIVISOR: i32 = 100;

/// Threat assessment weights matching AI_AssessThreat @ 0x0041ba40
pub const THREAT_SHAMAN: i32 = 50;
pub const THREAT_SUPER_WARRIOR: i32 = 20;
pub const THREAT_PREACHER: i32 = 15;
pub const THREAT_WARRIOR: i32 = 10;
pub const THREAT_SPY: i32 = 5;

/// Person subtypes matching original binary constants
pub const SUBTYPE_BRAVE: u8 = 2;
pub const SUBTYPE_WARRIOR: u8 = 3;
pub const SUBTYPE_PREACHER: u8 = 4;
pub const SUBTYPE_SPY: u8 = 5;
pub const SUBTYPE_SUPER_WARRIOR: u8 = 6;
pub const SUBTYPE_SHAMAN: u8 = 7;

/// Building subtypes
pub const BLDG_DRUM_TOWER: u8 = 4;
pub const BLDG_TEMPLE: u8 = 5;
pub const BLDG_SPY_TRAIN: u8 = 6;
pub const BLDG_WARRIOR_TRAIN: u8 = 7;
pub const BLDG_SUPER_TRAIN: u8 = 8;

pub fn score_person_target(subtype: u8, nearby_defenders: u32, distance: i32) -> i32 {
    let base = match subtype {
        SUBTYPE_SHAMAN => SCORE_SHAMAN,
        SUBTYPE_SUPER_WARRIOR => SCORE_SUPER_WARRIOR,
        SUBTYPE_PREACHER => SCORE_PREACHER,
        SUBTYPE_WARRIOR => SCORE_WARRIOR,
        SUBTYPE_SPY => SCORE_SPY,
        _ => 0,
    };
    let bonus = if subtype == SUBTYPE_SHAMAN && nearby_defenders < EXPOSED_SHAMAN_DEFENDER_THRESHOLD {
        SCORE_EXPOSED_SHAMAN_BONUS
    } else {
        0
    };
    (base + bonus - distance / DISTANCE_PENALTY_DIVISOR).max(0)
}

pub fn score_building_target(building_subtype: u8, distance: i32) -> i32 {
    let base = match building_subtype {
        BLDG_SUPER_TRAIN => SCORE_BUILDING_SUPER_TRAIN,
        BLDG_TEMPLE => SCORE_BUILDING_TEMPLE,
        BLDG_WARRIOR_TRAIN => SCORE_BUILDING_WARRIOR_TRAIN,
        BLDG_DRUM_TOWER => SCORE_BUILDING_DRUM_TOWER,
        _ => SCORE_BUILDING_DEFAULT,
    };
    (base - distance / DISTANCE_PENALTY_DIVISOR).max(0)
}

pub fn assess_threat(unit_counts: &[(u8, u32)]) -> i32 {
    unit_counts.iter().map(|(subtype, count)| {
        let weight = match *subtype {
            SUBTYPE_SHAMAN => THREAT_SHAMAN,
            SUBTYPE_SUPER_WARRIOR => THREAT_SUPER_WARRIOR,
            SUBTYPE_PREACHER => THREAT_PREACHER,
            SUBTYPE_WARRIOR => THREAT_WARRIOR,
            SUBTYPE_SPY => THREAT_SPY,
            _ => 0,
        };
        weight * (*count as i32)
    }).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_shaman_with_distance() {
        // SHAMAN base 1000, 0 defenders < 3 so exposed bonus 500, distance 100
        // => 1000 + 500 - 100/100 = 1499
        assert_eq!(score_person_target(SUBTYPE_SHAMAN, 0, 100), 1499);
    }

    #[test]
    fn score_shaman_with_enough_defenders() {
        // SHAMAN base 1000, 5 defenders >= 3 so no exposed bonus, distance 100
        // => 1000 - 100/100 = 999
        assert_eq!(score_person_target(SUBTYPE_SHAMAN, 5, 100), 999);
    }

    #[test]
    fn score_exposed_shaman_bonus() {
        // SHAMAN with < 3 defenders => 1000 + 500 = 1500
        assert_eq!(score_person_target(SUBTYPE_SHAMAN, 2, 0), 1500);
    }

    #[test]
    fn score_warrior_with_distance() {
        // WARRIOR base 10, distance 200 => 10 - 200/100 = 8
        assert_eq!(score_person_target(SUBTYPE_WARRIOR, 5, 200), 8);
    }

    #[test]
    fn score_building_super_train() {
        assert_eq!(score_building_target(BLDG_SUPER_TRAIN, 0), 250);
    }

    #[test]
    fn score_building_drum_tower_with_distance() {
        // DRUM_TOWER base 150, distance 300 => 150 - 3 = 147
        assert_eq!(score_building_target(BLDG_DRUM_TOWER, 300), 147);
    }

    #[test]
    fn assess_threat_mixed_units() {
        // 2 warriors (2*10=20) + 1 shaman (50) = 70
        let units = &[(SUBTYPE_WARRIOR, 2), (SUBTYPE_SHAMAN, 1)];
        assert_eq!(assess_threat(units), 70);
    }

    #[test]
    fn score_spy_clamped_to_zero() {
        // SPY base 5, distance 500 => 5 - 5 = 0 (min 0)
        assert_eq!(score_person_target(SUBTYPE_SPY, 10, 500), 0);
    }
}
