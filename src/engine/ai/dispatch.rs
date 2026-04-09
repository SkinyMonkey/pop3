// AI command dispatch — translates pending AI commands into real game state mutations.
//
// Replaces the log-only handlers in app.rs with actual unit movement,
// building placement queuing, training initiation, and spell TODO markers.

use super::flyby::{FlybyEndTarget, FlybyKeyframe, FlybyState, FlybyTooltip};
use super::{
    target::score_person_target, AiPendingCommands, AiSystem, FlybyEvent, FlybyEventKind,
    MarkerEntry,
};
use crate::engine::buildings::training::{start_training, training_output_subtype};
use crate::engine::buildings::BuildingState;
use crate::engine::command::GameCommand;
use crate::engine::movement::WorldCoord;
use crate::engine::units::coordinator::UnitCoordinator;

/// Resolve a marker ID to a world position using the marker entries table.
fn resolve_marker(marker_entries: &[MarkerEntry], marker_id: i32) -> Option<WorldCoord> {
    marker_entries
        .iter()
        .find(|m| m.marker == marker_id)
        .map(|m| WorldCoord {
            x: m.x as i16,
            z: m.y as i16,
        })
}

/// Dispatch all pending AI commands for a single AI tribe, producing real game state changes.
///
/// Called once per AI tribe per tick, after tick_update_ai has executed scripts.
/// - Attack: moves idle units toward enemy positions using target scoring
/// - Build: queues building type into AiBuildingPlacement
/// - Train: finds matching training buildings and initiates conversion
/// - Spell: logs with TODO marker (Phase 4 dependency)
/// - Move: moves units toward marker position
/// - Convert: logs with TODO (requires spell-like behavior)
/// - ShamanMove: moves shaman unit toward marker position
pub fn dispatch_ai_commands(
    cmds: &AiPendingCommands,
    marker_entries: &[MarkerEntry],
    coordinator: &mut UnitCoordinator,
    ai_system: &mut AiSystem,
    ai_tribe: u8,
) {
    // Attack commands: find best target, move idle units toward it
    for atk in &cmds.attacks {
        if atk.tribe_id != ai_tribe {
            continue;
        }
        let target_pos = if let Some((x, z)) = atk.marker {
            WorldCoord {
                x: x as i16,
                z: z as i16,
            }
        } else {
            // Find best enemy target using scoring
            find_best_attack_target(coordinator, atk.target_tribe)
                .unwrap_or(WorldCoord { x: 0, z: 0 })
        };
        if target_pos.x != 0 || target_pos.z != 0 {
            coordinator.order_move_tribe(ai_tribe, target_pos, atk.num_people);
            log::info!(
                "AI tribe {} attacks tribe {} with {} people -> ({}, {})",
                ai_tribe,
                atk.target_tribe,
                atk.num_people,
                target_pos.x,
                target_pos.z
            );
        }
    }

    // Build commands: queue into AiBuildingPlacement
    for bld in &cmds.builds {
        if bld.tribe_id != ai_tribe {
            continue;
        }
        ai_system
            .building_placement_mut(ai_tribe)
            .queue_building(bld.building_type);
        log::info!(
            "AI tribe {} queues building type {} at ({}, {})",
            ai_tribe,
            bld.building_type,
            bld.marker_x,
            bld.marker_y
        );
    }

    // Train commands: find matching training buildings and start conversion
    for trn in &cmds.trains {
        if trn.tribe_id != ai_tribe {
            continue;
        }
        let mut trained = 0u32;
        // Two-phase: collect matching building handles, then mutate
        let handles: Vec<u16> = coordinator
            .pool()
            .buildings()
            .filter(|(_h, hdr, bd)| {
                hdr.tribe == ai_tribe
                    && bd.state == BuildingState::Active
                    && bd.conversion_countdown == 0
                    && training_output_subtype(bd.building_subtype) == Some(trn.unit_type)
            })
            .map(|(h, _hdr, _bd)| h)
            .take(trn.count as usize)
            .collect();

        for handle in handles {
            if let Some((_hdr, bd)) = coordinator.pool_mut().building_by_handle_mut(handle) {
                if start_training(bd) {
                    trained += 1;
                }
            }
        }
        if trained > 0 {
            log::info!(
                "AI tribe {} initiated training of {} units of type {}",
                ai_tribe,
                trained,
                trn.unit_type
            );
        }
    }

    // Spell commands: TODO Phase 4 spell system
    for spl in &cmds.spells {
        if spl.tribe_id != ai_tribe {
            continue;
        }
        log::info!(
            "AI spell {} at ({}, {}) -- TODO: Phase 4 spell system",
            spl.spell_type,
            spl.target_x,
            spl.target_y
        );
    }

    // Move commands: move units toward marker position
    for mv in &cmds.moves {
        if mv.tribe_id != ai_tribe {
            continue;
        }
        if let Some(target) = resolve_marker(marker_entries, mv.marker) {
            coordinator.order_move_tribe(ai_tribe, target, mv.num_people);
            log::info!(
                "AI tribe {} moves {} people to marker {} -> ({}, {})",
                ai_tribe,
                mv.num_people,
                mv.marker,
                target.x,
                target.z
            );
        }
    }

    // Convert commands: TODO requires spell-like behavior
    for cnv in &cmds.converts {
        if cnv.tribe_id != ai_tribe {
            continue;
        }
        log::info!(
            "AI tribe {} converts at marker {} -- TODO: convert wild",
            ai_tribe,
            cnv.marker
        );
    }

    // Shaman move commands: move shaman toward marker position
    for sm in &cmds.shaman_moves {
        if sm.tribe_id != ai_tribe {
            continue;
        }
        if let Some(target) = resolve_marker(marker_entries, sm.marker) {
            // Move only the shaman (subtype 7) — use order_move_tribe with max 1
            coordinator.order_move_shaman(ai_tribe, target);
            log::info!(
                "AI tribe {} shaman moves to marker {} -> ({}, {})",
                ai_tribe,
                sm.marker,
                target.x,
                target.z
            );
        }
    }
}

/// Find the best enemy unit to attack using target scoring from target.rs.
fn find_best_attack_target(coordinator: &UnitCoordinator, target_tribe: u8) -> Option<WorldCoord> {
    let units = coordinator.units();
    let mut best_score = 0i32;
    let mut best_pos = None;

    for unit in units {
        if !unit.alive || unit.tribe_index != target_tribe {
            continue;
        }
        // Use 0 nearby defenders and 0 distance for initial scoring
        // (distance penalty would require knowing attacker position which varies)
        let score = score_person_target(unit.subtype, 0, 0);
        if score > best_score {
            best_score = score;
            best_pos = Some(WorldCoord {
                x: unit.movement.position.x,
                z: unit.movement.position.z,
            });
        }
    }

    best_pos
}

/// Convert a sequence of flyby events from PopScript into a FlybyState.
///
/// Events are processed in order: CreateNew resets state, SetEvent* adds keyframes,
/// SetEndTarget sets the final camera state, AllowInterrupt flags interruptibility,
/// Start triggers activation, Stop is returned as a separate GameCommand.
///
/// Returns a Vec<GameCommand> which may contain StartFlyby and/or StopFlyby.
pub fn build_flyby_state_from_events(events: &[FlybyEvent], game_tick: u32) -> Vec<GameCommand> {
    let mut state = FlybyState::new();
    let mut created = false;
    let mut commands = Vec::new();

    for event in events {
        match &event.kind {
            FlybyEventKind::CreateNew => {
                state = FlybyState::new();
                created = true;
            }
            FlybyEventKind::SetEventPos { x, y, tick } => {
                state.pos_x_keyframes.push(FlybyKeyframe {
                    tick: *tick,
                    value: *x,
                });
                state.pos_y_keyframes.push(FlybyKeyframe {
                    tick: *tick,
                    value: *y,
                });
            }
            FlybyEventKind::SetEventAngle { angle, tick } => {
                state.angle_keyframes.push(FlybyKeyframe {
                    tick: *tick,
                    value: *angle,
                });
            }
            FlybyEventKind::SetEventZoom { zoom, tick } => {
                state.zoom_keyframes.push(FlybyKeyframe {
                    tick: *tick,
                    value: *zoom,
                });
            }
            FlybyEventKind::SetEventIntPoint { .. } => {}
            FlybyEventKind::SetEventTooltip { tooltip_id, tick } => {
                state.tooltips.push(FlybyTooltip {
                    tick: *tick,
                    tooltip_id: *tooltip_id,
                });
            }
            FlybyEventKind::SetEndTarget {
                world_x,
                world_y,
                angle_z,
            } => {
                state.end_target = Some(FlybyEndTarget {
                    world_x: *world_x,
                    world_y: *world_y,
                    angle_z: *angle_z,
                    angle_x: 0,
                    zoom: 0,
                });
            }
            FlybyEventKind::AllowInterrupt => {
                state.allow_interrupt = true;
            }
            FlybyEventKind::Start => {
                if created {
                    state.start(game_tick);
                    commands.push(GameCommand::StartFlyby(state.clone()));
                }
            }
            FlybyEventKind::Stop => {
                commands.push(GameCommand::StopFlyby);
            }
        }
    }

    commands
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ai::building_ai::AiBuildingPlacement;

    // Helper to create a minimal UnitCoordinator with test units
    fn make_coordinator_with_units(units_data: &[(u8, u8, bool, u8)]) -> UnitCoordinator {
        // (tribe_index, subtype, alive, state_byte)
        // state_byte: 1=Idle, 6=Wander, 5=GoToPoint
        use crate::data::units::ModelType;
        use crate::engine::movement::PersonMovement;
        use crate::engine::units::person_state::PersonState;
        use crate::engine::units::unit::Unit;

        let mut coord = UnitCoordinator::new();
        for (i, &(tribe, subtype, alive, state_byte)) in units_data.iter().enumerate() {
            let state = match state_byte {
                1 => PersonState::Idle,
                6 => PersonState::Wander,
                5 => PersonState::GoToPoint,
                _ => PersonState::Idle,
            };
            let mut unit = Unit {
                id: i,
                model_type: ModelType::Person,
                subtype,
                tribe_index: tribe,
                movement: PersonMovement::default(),
                cell_x: 0.0,
                cell_y: 0.0,
                state,
                prev_state: PersonState::Idle,
                state_timer: 0,
                state_counter: 0,
                health: 100,
                max_health: 100,
                target_unit: None,
                attacker_unit: None,
                alive,
                home_pos: WorldCoord { x: 0, z: 0 },
                behavior_flags: 0,
                anim: crate::engine::units::animation::AnimationState::default(),
                guard_position: None,
                gather_target: None,
                wander_duration: 0,
                wander_range: 0,
                linked_obj_id: None,
                bloodlust: false,
                shielded: false,
                building_handle: None,
                wood_carried: 0,
            };
            unit.movement.position = WorldCoord {
                x: (i as i16 + 1) * 100,
                z: (i as i16 + 1) * 100,
            };
            coord.push_unit_for_test(unit);
        }
        coord
    }

    #[test]
    fn order_move_tribe_moves_matching_idle_units() {
        let mut coord = make_coordinator_with_units(&[
            (1, 2, true, 1), // tribe 1, brave, alive, idle -> should move
            (1, 2, true, 1), // tribe 1, brave, alive, idle -> should move
            (0, 2, true, 1), // tribe 0, brave, alive, idle -> wrong tribe
        ]);
        let target = WorldCoord { x: 500, z: 500 };
        let moved = coord.order_move_tribe(1, target, 10);
        assert_eq!(moved, 2);
    }

    #[test]
    fn order_move_tribe_skips_dead_units() {
        let mut coord = make_coordinator_with_units(&[
            (1, 2, false, 1), // tribe 1, dead -> skip
            (1, 2, true, 1),  // tribe 1, alive, idle -> should move
        ]);
        let target = WorldCoord { x: 500, z: 500 };
        let moved = coord.order_move_tribe(1, target, 10);
        assert_eq!(moved, 1);
    }

    #[test]
    fn order_move_tribe_skips_non_idle_units() {
        let mut coord = make_coordinator_with_units(&[
            (1, 2, true, 5), // tribe 1, GoToPoint -> skip (already moving)
            (1, 2, true, 1), // tribe 1, Idle -> should move
            (1, 2, true, 6), // tribe 1, Wander -> should move
        ]);
        let target = WorldCoord { x: 500, z: 500 };
        let moved = coord.order_move_tribe(1, target, 10);
        assert_eq!(moved, 2);
    }

    #[test]
    fn order_move_tribe_respects_num_people_limit() {
        let mut coord = make_coordinator_with_units(&[
            (1, 2, true, 1), // tribe 1, idle
            (1, 2, true, 1), // tribe 1, idle
            (1, 2, true, 1), // tribe 1, idle
        ]);
        let target = WorldCoord { x: 500, z: 500 };
        let moved = coord.order_move_tribe(1, target, 2);
        assert_eq!(moved, 2); // only 2 despite 3 available
    }

    #[test]
    fn dispatch_attack_no_idle_units_no_changes() {
        let mut coord = make_coordinator_with_units(&[
            (1, 2, true, 5), // tribe 1, already moving
        ]);
        let mut ai = AiSystem::new().unwrap();
        let cmds = AiPendingCommands {
            attacks: vec![super::super::AiAttackCommand {
                tribe_id: 1,
                target_tribe: 0,
                num_people: 5,
                attack_type: 0,
                marker: Some((500, 500)),
            }],
            builds: vec![],
            spells: vec![],
            trains: vec![],
            moves: vec![],
            converts: vec![],
            shaman_moves: vec![],
            flyby_events: vec![],
        };
        dispatch_ai_commands(&cmds, &[], &mut coord, &mut ai, 1);
        // Unit should still be in GoToPoint (unchanged)
        assert_eq!(
            coord.units()[0].state,
            crate::engine::units::person_state::PersonState::GoToPoint
        );
    }

    #[test]
    fn dispatch_build_queues_building_type() {
        let mut coord = UnitCoordinator::new();
        let mut ai = AiSystem::new().unwrap();
        let cmds = AiPendingCommands {
            attacks: vec![],
            builds: vec![super::super::AiBuildCommand {
                tribe_id: 1,
                building_type: 4, // drum tower
                marker_x: 100,
                marker_y: 200,
            }],
            spells: vec![],
            trains: vec![],
            moves: vec![],
            converts: vec![],
            shaman_moves: vec![],
            flyby_events: vec![],
        };
        dispatch_ai_commands(&cmds, &[], &mut coord, &mut ai, 1);
        assert_eq!(ai.building_placement_mut(1).priority_queue.len(), 1);
        assert_eq!(ai.building_placement_mut(1).priority_queue[0].1, 4);
    }

    #[test]
    fn dispatch_train_starts_training_in_matching_building() {
        use crate::data::units::ModelType;
        use crate::engine::buildings::types::{BuildingData, BuildingState as BS, BuildingSubtype};
        use crate::engine::movement::WorldCoord;

        let mut coord = UnitCoordinator::new();
        // Create a warrior training building owned by tribe 1
        let handle = coord
            .pool_mut()
            .create(
                ModelType::Building,
                BuildingSubtype::WarriorTrain as u8,
                1, // tribe 1
                WorldCoord { x: 0, z: 0 },
            )
            .unwrap();
        // Set building to Active state with training flag
        if let Some((_hdr, bd)) = coord.pool_mut().building_by_handle_mut(handle) {
            bd.state = BS::Active;
            bd.building_subtype = BuildingSubtype::WarriorTrain;
            bd.behavior_flags = 0x01;
            bd.conversion_countdown = 0;
        }

        let mut ai = AiSystem::new().unwrap();
        let cmds = AiPendingCommands {
            attacks: vec![],
            builds: vec![],
            spells: vec![],
            trains: vec![super::super::AiTrainCommand {
                tribe_id: 1,
                unit_type: 3, // warrior
                count: 1,
            }],
            moves: vec![],
            converts: vec![],
            shaman_moves: vec![],
            flyby_events: vec![],
        };
        dispatch_ai_commands(&cmds, &[], &mut coord, &mut ai, 1);
        // Building should now have a conversion countdown set
        if let Some((_hdr, bd)) = coord.pool_mut().building_by_handle_mut(handle) {
            assert!(bd.conversion_countdown > 0, "Training should have started");
        }
    }

    #[test]
    fn dispatch_spell_logs_todo_no_state_change() {
        let mut coord = UnitCoordinator::new();
        let mut ai = AiSystem::new().unwrap();
        let cmds = AiPendingCommands {
            attacks: vec![],
            builds: vec![],
            spells: vec![super::super::AiSpellCommand {
                tribe_id: 1,
                spell_type: 5,
                target_x: 100,
                target_y: 200,
            }],
            trains: vec![],
            moves: vec![],
            converts: vec![],
            shaman_moves: vec![],
            flyby_events: vec![],
        };
        // Should not panic
        dispatch_ai_commands(&cmds, &[], &mut coord, &mut ai, 1);
    }

    // ---- Flyby dispatch tests ----

    #[test]
    fn build_flyby_state_from_empty_events() {
        let commands = build_flyby_state_from_events(&[], 0);
        assert!(commands.is_empty());
    }

    #[test]
    fn build_flyby_state_start_flyby_command() {
        let events = vec![
            FlybyEvent {
                kind: FlybyEventKind::CreateNew,
            },
            FlybyEvent {
                kind: FlybyEventKind::Start,
            },
        ];
        let commands = build_flyby_state_from_events(&events, 100);
        assert_eq!(commands.len(), 1);
        match &commands[0] {
            GameCommand::StartFlyby(state) => {
                assert!(state.active);
                assert_eq!(state.start_tick, 100);
            }
            other => panic!("expected StartFlyby, got {:?}", other),
        }
    }

    #[test]
    fn build_flyby_state_with_keyframes() {
        let events = vec![
            FlybyEvent {
                kind: FlybyEventKind::CreateNew,
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEventPos {
                    x: 8,
                    y: 28,
                    tick: 0,
                },
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEventAngle {
                    angle: 1072,
                    tick: 0,
                },
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEventZoom {
                    zoom: -500,
                    tick: 0,
                },
            },
            FlybyEvent {
                kind: FlybyEventKind::AllowInterrupt,
            },
            FlybyEvent {
                kind: FlybyEventKind::Start,
            },
        ];
        let commands = build_flyby_state_from_events(&events, 200);
        assert_eq!(commands.len(), 1);
        match &commands[0] {
            GameCommand::StartFlyby(state) => {
                assert!(state.active);
                assert!(state.allow_interrupt);
                assert_eq!(state.pos_x_keyframes.len(), 1);
                assert_eq!(state.pos_x_keyframes[0].value, 8);
                assert_eq!(state.angle_keyframes[0].value, 1072);
                assert_eq!(state.zoom_keyframes[0].value, -500);
            }
            other => panic!("expected StartFlyby, got {:?}", other),
        }
    }

    #[test]
    fn build_flyby_state_stop_command() {
        let events = vec![FlybyEvent {
            kind: FlybyEventKind::Stop,
        }];
        let commands = build_flyby_state_from_events(&events, 0);
        assert_eq!(commands.len(), 1);
        assert!(matches!(&commands[0], GameCommand::StopFlyby));
    }

    #[test]
    fn build_flyby_state_start_then_stop() {
        let events = vec![
            FlybyEvent {
                kind: FlybyEventKind::CreateNew,
            },
            FlybyEvent {
                kind: FlybyEventKind::Start,
            },
            FlybyEvent {
                kind: FlybyEventKind::Stop,
            },
        ];
        let commands = build_flyby_state_from_events(&events, 0);
        assert_eq!(commands.len(), 2);
        assert!(matches!(&commands[0], GameCommand::StartFlyby(_)));
        assert!(matches!(&commands[1], GameCommand::StopFlyby));
    }

    #[test]
    fn build_flyby_state_set_end_target() {
        let events = vec![
            FlybyEvent {
                kind: FlybyEventKind::CreateNew,
            },
            FlybyEvent {
                kind: FlybyEventKind::SetEndTarget {
                    world_x: 28,
                    world_y: 8,
                    angle_z: 1438,
                },
            },
            FlybyEvent {
                kind: FlybyEventKind::Start,
            },
        ];
        let commands = build_flyby_state_from_events(&events, 0);
        assert_eq!(commands.len(), 1);
        match &commands[0] {
            GameCommand::StartFlyby(state) => {
                assert!(state.end_target.is_some());
                let et = state.end_target.as_ref().unwrap();
                assert_eq!(et.world_x, 28);
                assert_eq!(et.angle_z, 1438);
            }
            other => panic!("expected StartFlyby, got {:?}", other),
        }
    }

    #[test]
    fn build_flyby_start_without_create_is_ignored() {
        let events = vec![FlybyEvent {
            kind: FlybyEventKind::Start,
        }];
        let commands = build_flyby_state_from_events(&events, 0);
        assert!(commands.is_empty());
    }
}
