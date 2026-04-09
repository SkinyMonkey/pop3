use std::collections::HashSet;

/// Original game stores angles in 1/16th degree units (0-4095 for full circle).
/// Our Camera uses whole degrees. Divide by this to convert.
pub const FLYBY_ANGLE_SCALE: f32 = 16.0;

#[derive(Debug, Clone)]
pub struct FlybyKeyframe {
    pub tick: u32,
    pub value: i16,
}

#[derive(Debug, Clone)]
pub struct FlybyTooltip {
    pub tick: u32,
    pub tooltip_id: i32,
}

#[derive(Debug, Clone)]
pub struct FlybyEndTarget {
    pub world_x: i16,
    pub world_y: i16,
    pub angle_z: i16,
    pub angle_x: i16,
    pub zoom: i16,
}

#[derive(Debug, Clone)]
pub struct FlybyCameraOutput {
    pub angle_z: i16,
    pub angle_x: i16,
    pub zoom: i16,
    pub world_x: i16,
    pub world_y: i16,
}

#[derive(Debug, Clone)]
pub struct FlybyState {
    pub pos_x_keyframes: Vec<FlybyKeyframe>,
    pub pos_y_keyframes: Vec<FlybyKeyframe>,
    pub angle_keyframes: Vec<FlybyKeyframe>,
    pub zoom_keyframes: Vec<FlybyKeyframe>,
    pub tooltips: Vec<FlybyTooltip>,
    pub end_target: Option<FlybyEndTarget>,
    pub allow_interrupt: bool,
    pub start_tick: u32,
    pub active: bool,
    pub finished: bool,
    pub shown_tooltips: HashSet<u32>,
}

impl FlybyState {
    pub fn new() -> Self {
        Self {
            pos_x_keyframes: Vec::new(),
            pos_y_keyframes: Vec::new(),
            angle_keyframes: Vec::new(),
            zoom_keyframes: Vec::new(),
            tooltips: Vec::new(),
            end_target: None,
            allow_interrupt: false,
            start_tick: 0,
            active: false,
            finished: false,
            shown_tooltips: HashSet::new(),
        }
    }

    pub fn start(&mut self, tick: u32) {
        self.pos_x_keyframes.sort_by_key(|kf| kf.tick);
        self.pos_y_keyframes.sort_by_key(|kf| kf.tick);
        self.angle_keyframes.sort_by_key(|kf| kf.tick);
        self.zoom_keyframes.sort_by_key(|kf| kf.tick);
        // Keyframe ticks are relative (0-based from script). Offset by start_tick
        // so that update(game_tick, ...) works with absolute tick values.
        for kf in &mut self.pos_x_keyframes {
            kf.tick += tick;
        }
        for kf in &mut self.pos_y_keyframes {
            kf.tick += tick;
        }
        for kf in &mut self.angle_keyframes {
            kf.tick += tick;
        }
        for kf in &mut self.zoom_keyframes {
            kf.tick += tick;
        }
        // Also offset tooltip ticks
        for t in &mut self.tooltips {
            t.tick += tick;
        }
        self.start_tick = tick;
        self.active = true;
    }

    pub fn update(&mut self, tick: u32, defaults: &FlybyCameraOutput) -> Option<FlybyCameraOutput> {
        if !self.active {
            return None;
        }

        let max_tick = self
            .angle_keyframes
            .iter()
            .chain(self.zoom_keyframes.iter())
            .chain(self.pos_x_keyframes.iter())
            .chain(self.pos_y_keyframes.iter())
            .map(|kf| kf.tick)
            .max();

        match max_tick {
            None => {
                self.active = false;
                self.finished = true;
                return None;
            }
            Some(mt) if tick > mt => {
                self.active = false;
                self.finished = true;
                return None;
            }
            _ => {}
        }

        Some(FlybyCameraOutput {
            angle_z: interpolate_keyframes(&self.angle_keyframes, tick, defaults.angle_z),
            angle_x: 0,
            zoom: interpolate_keyframes(&self.zoom_keyframes, tick, defaults.zoom),
            world_x: interpolate_keyframes(&self.pos_x_keyframes, tick, defaults.world_x),
            world_y: interpolate_keyframes(&self.pos_y_keyframes, tick, defaults.world_y),
        })
    }

    pub fn interrupt(&mut self) {
        self.active = false;
        self.finished = true;
    }

    pub fn pending_tooltips(&mut self, tick: u32) -> Vec<i32> {
        let mut result = Vec::new();
        for tooltip in &self.tooltips {
            if tooltip.tick == tick && !self.shown_tooltips.contains(&tooltip.tick) {
                result.push(tooltip.tooltip_id);
                self.shown_tooltips.insert(tooltip.tick);
            }
        }
        result
    }
}

pub fn interpolate_keyframes(keyframes: &[FlybyKeyframe], tick: u32, default: i16) -> i16 {
    if keyframes.is_empty() {
        return default;
    }

    if tick <= keyframes[0].tick {
        return keyframes[0].value;
    }

    if tick >= keyframes[keyframes.len() - 1].tick {
        return keyframes[keyframes.len() - 1].value;
    }

    for i in 0..keyframes.len() - 1 {
        if tick >= keyframes[i].tick && tick <= keyframes[i + 1].tick {
            return lerp_keyframe(&keyframes[i], &keyframes[i + 1], tick);
        }
    }

    default
}

fn lerp_keyframe(a: &FlybyKeyframe, b: &FlybyKeyframe, tick: u32) -> i16 {
    let total = (b.tick - a.tick) as f32;
    if total == 0.0 {
        return b.value;
    }
    let progress = (tick - a.tick) as f32 / total;
    let result = a.value as f32 + (b.value as f32 - a.value as f32) * progress;
    result.round() as i16
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ai::dispatch::build_flyby_state_from_events;
    use crate::engine::ai::{FlybyEvent, FlybyEventKind};
    use crate::engine::command::GameCommand;

    fn kf(tick: u32, value: i16) -> FlybyKeyframe {
        FlybyKeyframe { tick, value }
    }

    #[test]
    fn flyby_state_new_is_inactive() {
        let state = FlybyState::new();
        assert!(!state.active);
        assert!(!state.finished);
    }

    #[test]
    fn flyby_state_start_activates() {
        let mut state = FlybyState::new();
        state.start(100);
        assert!(state.active);
        assert_eq!(state.start_tick, 100);
    }

    #[test]
    fn flyby_start_sorts_keyframes() {
        let mut state = FlybyState::new();
        state.angle_keyframes = vec![kf(20, 90), kf(5, 45), kf(15, 60)];
        state.start(0);
        assert_eq!(state.angle_keyframes[0].tick, 5);
        assert_eq!(state.angle_keyframes[1].tick, 15);
        assert_eq!(state.angle_keyframes[2].tick, 20);
    }

    #[test]
    fn interpolate_before_first_keyframe_uses_first_value() {
        let keyframes = vec![kf(100, 200)];
        assert_eq!(interpolate_keyframes(&keyframes, 50, 0), 200);
    }

    #[test]
    fn interpolate_at_keyframe_uses_exact_value() {
        let keyframes = vec![kf(100, 200)];
        assert_eq!(interpolate_keyframes(&keyframes, 100, 0), 200);
    }

    #[test]
    fn interpolate_between_keyframes_lerps() {
        let keyframes = vec![kf(100, 100), kf(200, 200)];
        assert_eq!(interpolate_keyframes(&keyframes, 150, 0), 150);
    }

    #[test]
    fn interpolate_after_last_keyframe_uses_last_value() {
        let keyframes = vec![kf(100, 200)];
        assert_eq!(interpolate_keyframes(&keyframes, 300, 0), 200);
    }

    #[test]
    fn flyby_update_interpolates_all_channels() {
        let mut state = FlybyState::new();
        state.pos_x_keyframes = vec![kf(0, 8), kf(100, 28)];
        state.pos_y_keyframes = vec![kf(0, 10), kf(100, 30)];
        state.angle_keyframes = vec![kf(0, 0), kf(100, 1072)];
        state.zoom_keyframes = vec![kf(0, 0), kf(100, -500)];
        state.start(0);

        let defaults = FlybyCameraOutput {
            angle_z: 0,
            angle_x: 0,
            zoom: 0,
            world_x: 0,
            world_y: 0,
        };

        let out = state.update(50, &defaults).unwrap();
        assert_eq!(out.world_x, 18);
        assert_eq!(out.world_y, 20);
        assert_eq!(out.angle_z, 536);
        assert_eq!(out.zoom, -250);
    }

    #[test]
    fn flyby_update_returns_none_when_past_all_keyframes() {
        let mut state = FlybyState::new();
        state.angle_keyframes = vec![kf(100, 45), kf(200, 90)];
        state.start(0);

        let defaults = FlybyCameraOutput {
            angle_z: 0,
            angle_x: 0,
            zoom: 0,
            world_x: 0,
            world_y: 0,
        };

        assert!(state.update(100, &defaults).is_some());
        assert!(state.update(200, &defaults).is_some());
        assert!(state.update(300, &defaults).is_none());
        assert!(state.finished);
    }

    #[test]
    fn flyby_interrupt_stops_and_marks_finished() {
        let mut state = FlybyState::new();
        state.start(0);
        state.interrupt();
        assert!(!state.active);
        assert!(state.finished);
    }

    #[test]
    fn flyby_pending_tooltips_returns_at_correct_tick() {
        let mut state = FlybyState::new();
        state.tooltips = vec![FlybyTooltip {
            tick: 50,
            tooltip_id: 5,
        }];
        state.start(0);

        assert_eq!(state.pending_tooltips(50), vec![5]);
        assert_eq!(state.pending_tooltips(60), Vec::<i32>::new());
    }

    #[test]
    fn flyby_pending_tooltips_deduplicates_already_shown() {
        let mut state = FlybyState::new();
        state.tooltips = vec![FlybyTooltip {
            tick: 50,
            tooltip_id: 5,
        }];
        state.start(0);

        assert_eq!(state.pending_tooltips(50), vec![5]);
        assert_eq!(state.pending_tooltips(50), Vec::<i32>::new());
    }

    // ---- Phase 7: Integration tests for full flyby pipeline ----

    #[test]
    fn integration_level1_flyby_produces_correct_camera_at_each_tick() {
        let events = vec![
            FlybyEvent {
                kind: FlybyEventKind::CreateNew,
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEventPos {
                    x: 8,
                    y: 28,
                    tick: 4,
                },
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEventPos {
                    x: 2,
                    y: 28,
                    tick: 81,
                },
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEventPos {
                    x: 252,
                    y: 254,
                    tick: 126,
                },
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEventPos {
                    x: 12,
                    y: 238,
                    tick: 181,
                },
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEventPos {
                    x: 20,
                    y: 216,
                    tick: 221,
                },
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEventAngle { angle: 0, tick: 5 },
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEventAngle {
                    angle: 1072,
                    tick: 46,
                },
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEventAngle {
                    angle: 681,
                    tick: 87,
                },
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEventAngle {
                    angle: 744,
                    tick: 134,
                },
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEventAngle {
                    angle: 54,
                    tick: 170,
                },
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEventAngle {
                    angle: 1438,
                    tick: 219,
                },
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEventZoom {
                    zoom: -100,
                    tick: 10,
                },
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEventZoom { zoom: 10, tick: 67 },
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEventZoom {
                    zoom: 80,
                    tick: 165,
                },
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEventZoom { zoom: 0, tick: 202 },
            },
            FlybyEvent {
                kind: FlybyEventKind::AllowInterrupt,
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEndTarget {
                    world_x: 20,
                    world_y: 216,
                    angle_z: 1438,
                },
            },
            FlybyEvent {
                kind: FlybyEventKind::Start,
            },
        ];

        // Build from events starting at game_tick=73 (matching actual game)
        let commands = build_flyby_state_from_events(&events, 73);
        assert_eq!(commands.len(), 1);

        let mut state = match &commands[0] {
            GameCommand::StartFlyby(s) => s.clone(),
            other => panic!("expected StartFlyby, got {:?}", other),
        };

        let defaults = FlybyCameraOutput {
            angle_z: 0,
            angle_x: 0,
            zoom: 0,
            world_x: 0,
            world_y: 0,
        };

        // Keyframe ticks are relative (5, 46, ...) but start() offsets them by 73.
        // So angle keyframe at relative tick 5 becomes absolute tick 78.
        // At absolute tick 78 (= 73 + 5), angle should be 0 (exact keyframe)
        let out = state.update(78, &defaults).unwrap();
        assert_eq!(out.angle_z, 0);

        // At absolute tick 119 (= 73 + 46), angle should be 1072
        let out = state.update(119, &defaults).unwrap();
        assert_eq!(out.angle_z, 1072);

        // At absolute tick 300 (past all keyframes), flyby should be finished
        let result = state.update(300, &defaults);
        assert!(result.is_none());
        assert!(state.finished);
    }

    #[test]
    fn integration_angle_conversion_to_degrees() {
        assert_eq!((1072.0_f32 / FLYBY_ANGLE_SCALE).round() as i16, 67);
        assert_eq!((1438.0_f32 / FLYBY_ANGLE_SCALE).round() as i16, 90);
        assert_eq!((0.0_f32 / FLYBY_ANGLE_SCALE).round() as i16, 0);
        assert_eq!((681.0_f32 / FLYBY_ANGLE_SCALE).round() as i16, 43);
    }

    #[test]
    fn integration_stop_command_clears_flyby() {
        let start_events = vec![
            FlybyEvent {
                kind: FlybyEventKind::CreateNew,
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEventAngle {
                    angle: 500,
                    tick: 10,
                },
            },
            FlybyEvent {
                kind: FlybyEventKind::Start,
            },
        ];
        let stop_events = vec![FlybyEvent {
            kind: FlybyEventKind::Stop,
        }];

        let start_cmds = build_flyby_state_from_events(&start_events, 0);
        assert_eq!(start_cmds.len(), 1);
        assert!(matches!(&start_cmds[0], GameCommand::StartFlyby(_)));

        let stop_cmds = build_flyby_state_from_events(&stop_events, 0);
        assert_eq!(stop_cmds.len(), 1);
        assert!(matches!(&stop_cmds[0], GameCommand::StopFlyby));
    }

    #[test]
    fn integration_interrupt_allows_early_exit() {
        let mut state = FlybyState::new();
        state.angle_keyframes = vec![
            FlybyKeyframe { tick: 0, value: 0 },
            FlybyKeyframe {
                tick: 200,
                value: 1600,
            },
        ];
        state.allow_interrupt = true;
        state.start(0);

        let defaults = FlybyCameraOutput {
            angle_z: 0,
            angle_x: 0,
            zoom: 0,
            world_x: 0,
            world_y: 0,
        };

        let out = state.update(100, &defaults).unwrap();
        assert!(out.angle_z > 0);
        assert!(out.angle_z < 1600);

        state.interrupt();
        assert!(!state.active);
        assert!(state.finished);
        assert!(state.update(150, &defaults).is_none());
    }

    #[test]
    fn integration_position_interpolation_matches_original_game() {
        let mut state = FlybyState::new();
        state.pos_x_keyframes = vec![
            FlybyKeyframe { tick: 4, value: 8 },
            FlybyKeyframe { tick: 81, value: 2 },
        ];
        state.pos_y_keyframes = vec![
            FlybyKeyframe { tick: 4, value: 28 },
            FlybyKeyframe {
                tick: 81,
                value: 28,
            },
        ];
        state.start(0);

        let defaults = FlybyCameraOutput {
            angle_z: 0,
            angle_x: 0,
            zoom: 0,
            world_x: 0,
            world_y: 0,
        };

        let out = state.update(4, &defaults).unwrap();
        assert_eq!(out.world_x, 8);
        assert_eq!(out.world_y, 28);

        let out = state.update(81, &defaults).unwrap();
        assert_eq!(out.world_x, 2);
        assert_eq!(out.world_y, 28);

        let out = state.update(42, &defaults).unwrap();
        let expected_x = interpolate_keyframes(&state.pos_x_keyframes, 42, 0);
        assert!((out.world_x - expected_x).abs() <= 1);
    }

    #[test]
    fn integration_zoom_interpolation_negative_zoom() {
        let mut state = FlybyState::new();
        state.zoom_keyframes = vec![
            FlybyKeyframe {
                tick: 10,
                value: -100,
            },
            FlybyKeyframe {
                tick: 67,
                value: 10,
            },
        ];
        state.start(0);

        let defaults = FlybyCameraOutput {
            angle_z: 0,
            angle_x: 0,
            zoom: 0,
            world_x: 0,
            world_y: 0,
        };

        let out = state.update(10, &defaults).unwrap();
        assert_eq!(out.zoom, -100);

        let out = state.update(67, &defaults).unwrap();
        assert_eq!(out.zoom, 10);

        let out = state.update(38, &defaults).unwrap();
        assert!(
            out.zoom < 0 && out.zoom > -100,
            "zoom at midpoint should be between -100 and 0, got {}",
            out.zoom
        );
    }

    #[test]
    fn integration_start_tick_offsets_keyframes() {
        // When flyby starts at game_tick=100, keyframe at relative tick 5
        // should activate at absolute tick 105, not tick 5.
        let mut state = FlybyState::new();
        state.angle_keyframes = vec![
            FlybyKeyframe {
                tick: 5,
                value: 500,
            },
            FlybyKeyframe {
                tick: 50,
                value: 1000,
            },
        ];
        state.start(100);

        let defaults = FlybyCameraOutput {
            angle_z: 0,
            angle_x: 0,
            zoom: 0,
            world_x: 0,
            world_y: 0,
        };

        // Before keyframe tick (absolute 104 < 105): should use first keyframe value
        let out = state.update(104, &defaults).unwrap();
        assert_eq!(out.angle_z, 500);

        // At exact keyframe tick (absolute 105): should use exact value
        let out = state.update(105, &defaults).unwrap();
        assert_eq!(out.angle_z, 500);

        // At second keyframe (absolute 150 = 100 + 50): should use 1000
        let out = state.update(150, &defaults).unwrap();
        assert_eq!(out.angle_z, 1000);

        // After all keyframes (absolute 200 > 150): finished
        assert!(state.update(200, &defaults).is_none());
        assert!(state.finished);
    }
}
