/// Trait definitions for subsystem dependencies called by the tick loop.
///
/// Each trait corresponds to one of the Tick_Update* calls in
/// Game_SimulationTick (0x004bb5a0). Subsystems implement these traits
/// to plug into the game loop. NoOp implementations are provided for
/// viewer mode where no game logic runs.

/// Terrain modification processing.
/// Original: Tick_UpdateTerrain at 0x0048bda0
pub trait TerrainTick {
    fn tick_update_terrain(&mut self);
}

/// Object system update (all object types).
/// Original: Tick_UpdateObjects at 0x004a7550
pub trait ObjectTick {
    fn tick_update_objects(&mut self);
}

/// Water simulation and effects.
/// Original: Tick_UpdateWater at 0x0048bf10
pub trait WaterTick {
    fn tick_update_water(&mut self);
}

/// Network message processing.
/// Original: Tick_ProcessNetworkMessages at 0x004a76b0
pub trait NetworkTick {
    /// Process incoming network messages. Returns true if the game should
    /// continue processing this tick, false to abort (e.g. waiting for sync).
    fn tick_process_network(&mut self) -> bool;
}

/// Pending player action queue processing.
/// Original: Tick_ProcessPendingActions at 0x004a6f60
pub trait ActionTick {
    fn tick_process_actions(&mut self);
}

/// Per-tribe game time updates.
/// Original: Tick_UpdateGameTime at 0x004a7ac0
pub trait GameTimeTick {
    fn tick_update_game_time(&mut self);
}

/// Single-player per-tick update.
/// Original: Tick_UpdateSinglePlayer at 0x00456500
pub trait SinglePlayerTick {
    fn tick_update_single_player(&mut self);
}

/// Tutorial mode per-tick update.
/// Original: Tick_UpdateTutorial at 0x00469320
pub trait TutorialTick {
    fn tick_update_tutorial(&mut self);
}

/// AI tribe update (all computer-controlled tribes).
/// Original: AI_UpdateAllTribes at 0x0041a7d0
pub trait AiTick {
    fn tick_update_ai(&mut self);
}

/// Population spawning.
/// Original: Tick_UpdatePopulation at 0x004198f0
pub trait PopulationTick {
    fn tick_update_population(&mut self);
}

/// Main update dispatcher (misnamed "mana" in original).
/// Handles object state updates, movement, building combat, cleanup.
/// Original: Tick_UpdateMana at 0x004aeac0
pub trait ManaTick {
    fn tick_update_mana(&mut self);
}

// --- No-op implementations for viewer mode ---

/// No-op stub that does nothing. Used when a subsystem isn't implemented yet.
pub struct NoOp;

impl TerrainTick for NoOp {
    fn tick_update_terrain(&mut self) {}
}

impl ObjectTick for NoOp {
    fn tick_update_objects(&mut self) {}
}

impl WaterTick for NoOp {
    fn tick_update_water(&mut self) {}
}

impl NetworkTick for NoOp {
    fn tick_process_network(&mut self) -> bool {
        true
    }
}

impl ActionTick for NoOp {
    fn tick_process_actions(&mut self) {}
}

impl GameTimeTick for NoOp {
    fn tick_update_game_time(&mut self) {}
}

impl SinglePlayerTick for NoOp {
    fn tick_update_single_player(&mut self) {}
}

impl TutorialTick for NoOp {
    fn tick_update_tutorial(&mut self) {}
}

impl AiTick for NoOp {
    fn tick_update_ai(&mut self) {}
}

impl PopulationTick for NoOp {
    fn tick_update_population(&mut self) {}
}

impl ManaTick for NoOp {
    fn tick_update_mana(&mut self) {}
}
