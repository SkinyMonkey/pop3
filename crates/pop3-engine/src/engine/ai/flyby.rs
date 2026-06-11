//! Flyby cinematic camera playback.
//!
//! Faithful to the original's event scheduler (popTB.exe 0x4dac00,
//! `Flyby_TickScheduler` — see re_meta.md → Flyby) and its per-channel handlers:
//! a flyby is four independent channels (pos_x, pos_y, angle, zoom). Each
//! script event means "at `start_tick`, begin moving this channel from its
//! *current* value to `target`, arriving `duration` ticks later".
//!
//! - Positions are half-cell coordinates; deltas take the shortest toroidal
//!   path (binary wraps at half the map, see FUN_004daf50).
//! - Angles are 2048 units per turn; deltas take the shortest path
//!   (Math_AngleDifference / Math_GetRotationDirection in FUN_004db200).
//! - Each segment is eased (the binary builds a trapezoidal velocity
//!   profile in FUN_004db950; we approximate with smoothstep).

use std::collections::HashSet;

/// Original game angles: 2048 units per full circle (sin/cos LUTs are
/// indexed with `& 0x7ff`, see Camera_ApplyRotation / Math_MovePointByAngle).
pub const FLYBY_ANGLE_MODULUS: i32 = 2048;

/// Flyby positions are half-cell coordinates; a 128-cell map wraps at 256.
pub const FLYBY_DEFAULT_POS_MODULUS: i32 = 256;

/// One scripted channel event: start moving toward `target` at `start_tick`
/// (relative to flyby start), arriving `duration` ticks later.
#[derive(Debug, Clone)]
pub struct FlybyChannelEvent {
    pub start_tick: u32,
    pub target: i16,
    pub duration: u32,
}

/// An in-flight animation segment.
#[derive(Debug, Clone)]
struct Segment {
    start_value: f32,
    delta: f32,
    start_tick: f32,
    duration: f32,
}

/// One animated camera channel (pos_x / pos_y / angle / zoom).
#[derive(Debug, Clone)]
pub struct FlybyChannel {
    pub events: Vec<FlybyChannelEvent>,
    next_event: usize,
    value: f32,
    seg: Option<Segment>,
    modulus: Option<i32>,
}

impl FlybyChannel {
    fn new(modulus: Option<i32>) -> Self {
        Self {
            events: Vec::new(),
            next_event: 0,
            value: 0.0,
            seg: None,
            modulus,
        }
    }

    pub fn push_event(&mut self, start_tick: u32, target: i16, duration: u32) {
        self.events.push(FlybyChannelEvent {
            start_tick,
            target,
            duration,
        });
    }

    fn sort_events(&mut self) {
        self.events.sort_by_key(|e| e.start_tick);
    }

    fn set_value(&mut self, v: f32) {
        self.value = self.wrap(v);
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    fn wrap(&self, v: f32) -> f32 {
        match self.modulus {
            Some(m) => v.rem_euclid(m as f32),
            None => v,
        }
    }

    /// Shortest signed path from the current value to `target`
    /// (toroidal/circular when a modulus is set).
    fn delta_to(&self, target: f32) -> f32 {
        match self.modulus {
            Some(m) => {
                let mf = m as f32;
                let mut d = (target - self.value).rem_euclid(mf);
                if d >= mf / 2.0 {
                    d -= mf;
                }
                d
            }
            None => target - self.value,
        }
    }

    /// Advance to relative tick `t`, activating due events in order.
    fn advance(&mut self, t: f32) {
        while self.next_event < self.events.len()
            && self.events[self.next_event].start_tick as f32 <= t
        {
            let ev = self.events[self.next_event].clone();
            self.next_event += 1;
            // Bring the previous segment up to this event's start so the new
            // segment captures the channel's value at activation time.
            self.advance_segment(ev.start_tick as f32);
            self.seg = Some(Segment {
                start_value: self.value,
                delta: self.delta_to(ev.target as f32),
                start_tick: ev.start_tick as f32,
                duration: (ev.duration.max(1)) as f32,
            });
        }
        self.advance_segment(t);
    }

    fn advance_segment(&mut self, t: f32) {
        if let Some(seg) = &self.seg {
            let p = ((t - seg.start_tick) / seg.duration).clamp(0.0, 1.0);
            // Smoothstep ease; the original uses a trapezoidal velocity
            // profile (FUN_004db950) — both accelerate in and decelerate out.
            let e = p * p * (3.0 - 2.0 * p);
            self.value = self.wrap(seg.start_value + seg.delta * e);
            if p >= 1.0 {
                self.seg = None;
            }
        }
    }

    /// All events consumed and no segment still animating.
    fn done(&self) -> bool {
        self.next_event >= self.events.len() && self.seg.is_none()
    }
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

/// Camera state in flyby script units (half-cells / 2048-angle / zoom units).
#[derive(Debug, Clone)]
pub struct FlybyCameraOutput {
    pub angle_z: i16,
    pub angle_x: i16,
    pub zoom: i16,
    pub world_x: i16,
    pub world_y: i16,
}

/// Fractional camera output for per-frame evaluation. The original camera
/// works in world units (512 per cell), so sub-half-cell precision is
/// needed for smooth motion.
#[derive(Debug, Clone)]
pub struct FlybyCameraOutputF {
    pub angle_z: f32,
    pub angle_x: f32,
    pub zoom: f32,
    pub world_x: f32,
    pub world_y: f32,
}

#[derive(Debug, Clone)]
pub struct FlybyState {
    pub pos_x: FlybyChannel,
    pub pos_y: FlybyChannel,
    pub angle: FlybyChannel,
    pub zoom: FlybyChannel,
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
            pos_x: FlybyChannel::new(Some(FLYBY_DEFAULT_POS_MODULUS)),
            pos_y: FlybyChannel::new(Some(FLYBY_DEFAULT_POS_MODULUS)),
            angle: FlybyChannel::new(Some(FLYBY_ANGLE_MODULUS)),
            zoom: FlybyChannel::new(None),
            tooltips: Vec::new(),
            end_target: None,
            allow_interrupt: false,
            start_tick: 0,
            active: false,
            finished: false,
            shown_tooltips: HashSet::new(),
        }
    }

    /// Set the toroidal wrap for the position channels (2 × map width in
    /// half-cells; 256 for a 128-cell map).
    pub fn set_pos_modulus(&mut self, m: i32) {
        self.pos_x.modulus = Some(m);
        self.pos_y.modulus = Some(m);
    }

    /// Seed the channels with the camera state at flyby start. Mirrors the
    /// original: each channel animates from the camera's current value
    /// (FUN_004daf50 / FUN_004db200 read the live camera struct).
    pub fn set_initial(&mut self, initial: &FlybyCameraOutput) {
        self.pos_x.set_value(initial.world_x as f32);
        self.pos_y.set_value(initial.world_y as f32);
        self.angle.set_value(initial.angle_z as f32);
        self.zoom.set_value(initial.zoom as f32);
    }

    pub fn start(&mut self, tick: u32) {
        self.pos_x.sort_events();
        self.pos_y.sort_events();
        self.angle.sort_events();
        self.zoom.sort_events();
        // Tooltip ticks are relative; offset to absolute for pending_tooltips.
        for t in &mut self.tooltips {
            t.tick += tick;
        }
        self.start_tick = tick;
        self.active = true;
    }

    /// Per-frame evaluation at a fractional absolute game tick. Returns the
    /// camera state while the flyby plays (including one final output the
    /// moment all channels complete), then None.
    pub fn update_f(&mut self, tick: f32) -> Option<FlybyCameraOutputF> {
        if !self.active {
            return None;
        }

        let no_events = self.pos_x.events.is_empty()
            && self.pos_y.events.is_empty()
            && self.angle.events.is_empty()
            && self.zoom.events.is_empty();
        if no_events {
            self.active = false;
            self.finished = true;
            return None;
        }

        let rel = (tick - self.start_tick as f32).max(0.0);
        self.pos_x.advance(rel);
        self.pos_y.advance(rel);
        self.angle.advance(rel);
        self.zoom.advance(rel);

        if self.pos_x.done() && self.pos_y.done() && self.angle.done() && self.zoom.done() {
            // Emit the final (exact-target) state once, then deactivate.
            self.active = false;
            self.finished = true;
        }

        Some(FlybyCameraOutputF {
            angle_z: self.angle.value(),
            angle_x: 0.0,
            zoom: self.zoom.value(),
            world_x: self.pos_x.value(),
            world_y: self.pos_y.value(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ai::dispatch::build_flyby_state_from_events;
    use crate::engine::ai::{FlybyEvent, FlybyEventKind};
    use crate::engine::command::GameCommand;

    fn initial(x: i16, y: i16, angle: i16, zoom: i16) -> FlybyCameraOutput {
        FlybyCameraOutput {
            angle_z: angle,
            angle_x: 0,
            zoom,
            world_x: x,
            world_y: y,
        }
    }

    #[test]
    fn flyby_state_new_is_inactive() {
        let state = FlybyState::new();
        assert!(!state.active);
        assert!(!state.finished);
    }

    #[test]
    fn empty_flyby_finishes_immediately() {
        let mut state = FlybyState::new();
        state.start(10);
        assert!(state.update_f(10.0).is_none());
        assert!(state.finished);
    }

    #[test]
    fn channel_holds_initial_value_before_first_event() {
        // Level 1: first pos event fires at relative tick 4 — before that
        // the camera must hold where it was (the village), not snap.
        let mut state = FlybyState::new();
        state.pos_x.push_event(4, 8, 80);
        state.set_initial(&initial(36, 215, 0, 0));
        state.start(73);

        let out = state.update_f(73.0).unwrap();
        assert!((out.world_x - 36.0).abs() < 1e-4);
        let out = state.update_f(76.9).unwrap();
        assert!((out.world_x - 36.0).abs() < 1e-4);
    }

    #[test]
    fn channel_eases_from_current_to_target_over_duration() {
        // FLYBY_SET_EVENT_POS(8, 28, 4, 80): START moving at tick 4, arrive
        // at tick 84 — not "be at (8,28) at tick 4".
        let mut state = FlybyState::new();
        state.pos_x.push_event(4, 8, 80);
        state.set_initial(&initial(36, 0, 0, 0));
        state.start(0);

        let out = state.update_f(4.0).unwrap();
        assert!((out.world_x - 36.0).abs() < 1e-4, "still at start when firing");

        let out = state.update_f(44.0).unwrap();
        assert!(
            out.world_x < 36.0 && out.world_x > 8.0,
            "midway between, got {}",
            out.world_x
        );

        let out = state.update_f(84.0).unwrap();
        assert!((out.world_x - 8.0).abs() < 1e-4, "exactly at target at end");
    }

    #[test]
    fn ease_is_smooth_not_linear() {
        // Smoothstep: slower near the endpoints than in the middle.
        let mut state = FlybyState::new();
        state.zoom.push_event(0, 100, 100);
        state.set_initial(&initial(0, 0, 0, 0));
        state.start(0);

        let v10 = state.update_f(10.0).unwrap().zoom;
        let v50 = state.update_f(50.0).unwrap().zoom;
        assert!(v10 < 10.0, "ease-in slower than linear, got {v10}");
        assert!((v50 - 50.0).abs() < 1e-3, "midpoint is half, got {v50}");
    }

    #[test]
    fn angle_takes_shortest_wrapped_path() {
        // 2000 → 100 (mod 2048): +148 across the seam, not -1900.
        let mut state = FlybyState::new();
        state.angle.push_event(0, 100, 10);
        state.set_initial(&initial(0, 0, 2000, 0));
        state.start(0);

        let v = state.update_f(5.0).unwrap().angle_z;
        assert!(
            (2000.0..2048.0).contains(&v) || (0.0..=100.0).contains(&v),
            "midway across the seam, got {v}"
        );
        let v = state.update_f(10.0).unwrap().angle_z;
        assert!((v - 100.0).abs() < 1e-3);
    }

    #[test]
    fn position_wraps_across_map_seam() {
        // Level 1: from x≈2 to x=252 crosses the toroidal seam (-6, not +250).
        let mut state = FlybyState::new();
        state.pos_x.push_event(0, 252, 10);
        state.set_initial(&initial(2, 0, 0, 0));
        state.start(0);

        let v = state.update_f(5.0).unwrap().world_x;
        assert!(
            v > 252.0 || v < 2.0,
            "must travel through the seam, got {v}"
        );
        let v = state.update_f(10.0).unwrap().world_x;
        assert!((v - 252.0).abs() < 1e-3);
    }

    #[test]
    fn retarget_mid_flight_captures_current_value() {
        // Level 1 pos events overlap: (…,4,80) runs to 84 but the next event
        // fires at 81. The new segment starts from wherever the camera is.
        let mut state = FlybyState::new();
        state.zoom.push_event(0, 100, 80);
        state.zoom.push_event(40, 0, 40);
        state.set_initial(&initial(0, 0, 0, 0));
        state.start(0);

        // At tick 40 the first segment is halfway (value 50); the second
        // event retargets to 0 from there.
        let v = state.update_f(60.0).unwrap().zoom;
        assert!(v > 0.0 && v < 50.0, "easing back down from ~50, got {v}");
        let v = state.update_f(80.0).unwrap().zoom;
        assert!(v.abs() < 1e-3, "arrived at retarget, got {v}");
    }

    #[test]
    fn update_emits_final_state_then_none() {
        let mut state = FlybyState::new();
        state.angle.push_event(0, 512, 10);
        state.set_initial(&initial(0, 0, 0, 0));
        state.start(0);

        let out = state.update_f(15.0).unwrap();
        assert!((out.angle_z - 512.0).abs() < 1e-3);
        assert!(state.finished);
        assert!(state.update_f(16.0).is_none());
    }

    #[test]
    fn fractional_ticks_move_between_integer_ticks() {
        let mut state = FlybyState::new();
        state.zoom.push_event(0, 100, 10);
        state.set_initial(&initial(0, 0, 0, 0));
        state.start(0);

        let a = state.update_f(5.0).unwrap().zoom;
        let b = state.update_f(5.5).unwrap().zoom;
        assert!(b > a, "value advances within a tick: {a} → {b}");
    }

    #[test]
    fn flyby_interrupt_stops_and_marks_finished() {
        let mut state = FlybyState::new();
        state.zoom.push_event(0, 100, 10);
        state.start(0);
        state.interrupt();
        assert!(!state.active);
        assert!(state.finished);
        assert!(state.update_f(5.0).is_none());
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

    // ---- Integration: full level-1 flyby through the dispatch builder ----

    fn level1_events() -> Vec<FlybyEvent> {
        // Exact values from cpscr010.dat (with the duration args the binary
        // reads — AI_ExecuteScriptCommand cases 0x4b9-0x4be).
        let mut ev = vec![
            FlybyEvent {
                kind: FlybyEventKind::CreateNew,
            },
            FlybyEvent {
                kind: FlybyEventKind::AllowInterrupt,
            },
        ];
        for (x, y, tick, dur) in [
            (8, 28, 4, 80),
            (2, 28, 81, 44),
            (252, 254, 126, 45),
            (12, 238, 181, 30),
            (20, 216, 221, 45),
        ] {
            ev.push(FlybyEvent {
                kind: FlybyEventKind::SetEventPos {
                    x,
                    y,
                    tick,
                    duration: dur,
                },
            });
        }
        for (angle, tick, dur) in [
            (0, 5, 35),
            (1072, 46, 40),
            (681, 87, 45),
            (744, 134, 35),
            (54, 170, 48),
            (1438, 219, 45),
        ] {
            ev.push(FlybyEvent {
                kind: FlybyEventKind::SetEventAngle {
                    angle,
                    tick,
                    duration: dur,
                },
            });
        }
        for (zoom, tick, dur) in [(-100, 10, 35), (10, 67, 25), (80, 165, 36), (0, 202, 63)] {
            ev.push(FlybyEvent {
                kind: FlybyEventKind::SetEventZoom {
                    zoom,
                    tick,
                    duration: dur,
                },
            });
        }
        ev.push(FlybyEvent {
            kind: FlybyEventKind::SetEndTarget {
                world_x: 20,
                world_y: 216,
                angle_z: 1438,
            },
        });
        ev.push(FlybyEvent {
            kind: FlybyEventKind::Start,
        });
        ev
    }

    #[test]
    fn integration_level1_flyby_trajectory() {
        let commands = build_flyby_state_from_events(&level1_events(), 73);
        assert_eq!(commands.len(), 1);
        let mut state = match &commands[0] {
            GameCommand::StartFlyby(s) => s.clone(),
            other => panic!("expected StartFlyby, got {:?}", other),
        };
        // Camera starts on the village (shaman at half-cells (17, 215)-ish).
        state.set_initial(&initial(17, 215, 1536, 0));

        // Before the first pos event fires (abs 73+4=77): still on village.
        let out = state.update_f(75.0).unwrap();
        assert!((out.world_x - 17.0).abs() < 1e-3);
        assert!((out.world_y - 215.0).abs() < 1e-3);

        // Approach runs ticks 4..84: at rel 44 (abs 117) we're in transit.
        let out = state.update_f(117.0).unwrap();
        assert!(
            (out.world_x - 8.0).abs() > 0.5 || (out.world_y - 28.0).abs() > 0.5,
            "still approaching the camp at rel 44"
        );

        // First pos segment lands at rel 84 (abs 157)… but the second pos
        // event already fired at rel 81 retargeting toward (2,28) — snapped
        // to cell center (3,29). By rel 125 (81+44, abs 198) it arrives.
        let out = state.update_f(198.0).unwrap();
        assert!((out.world_x - 3.0).abs() < 1.0, "got {}", out.world_x);
        assert!((out.world_y - 29.0).abs() < 1.0, "got {}", out.world_y);

        // Final pos event: start rel 221, duration 45 → rel 266 (abs 339).
        // Target (20,216) snaps to cell center (21,217).
        let out = state.update_f(339.0).unwrap();
        assert!((out.world_x - 21.0).abs() < 1e-3);
        assert!((out.world_y - 217.0).abs() < 1e-3);
        assert!((out.angle_z - 1438.0).abs() < 1e-3);
        assert!(out.zoom.abs() < 1e-3);

        // All channels complete shortly after; flyby ends.
        assert!(state.finished || state.update_f(340.0).is_none());
        assert!(state.finished);
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
                    duration: 20,
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
}
