/// Top-level game state machine.
/// Original: g_GameState at 0x00877598
///
/// The game transitions through these states during its lifecycle:
/// Frontend → Loading → InGame → (Outro or back to Frontend)
/// Multiplayer is entered from Frontend for network games.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    /// Main menu / frontend. Handles menu rendering and navigation.
    /// Original: GameState_Frontend at 0x004baa40
    Frontend,

    /// Level loading sequence. Loads terrain, textures, units, AI scripts.
    /// Original: GameState_Loading at 0x0041fab0
    Loading,

    /// Active gameplay. Simulation tick loop runs here.
    /// Original: GameState_InGame at 0x004ddd20
    InGame,

    /// Victory/defeat outro sequence.
    /// Original: GameState_Outro at 0x004bae70
    Outro,

    /// Multiplayer lobby and session management.
    /// Original: GameState_Multiplayer at 0x004c03d0
    Multiplayer,
}

impl GameState {
    /// Convert from the raw byte value stored in the binary.
    pub fn from_raw(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Frontend),
            1 => Some(Self::Loading),
            2 => Some(Self::InGame),
            3 => Some(Self::Outro),
            4 => Some(Self::Multiplayer),
            _ => None,
        }
    }

    /// Convert to the raw byte value.
    pub fn to_raw(self) -> u8 {
        match self {
            Self::Frontend => 0,
            Self::Loading => 1,
            Self::InGame => 2,
            Self::Outro => 3,
            Self::Multiplayer => 4,
        }
    }
}
