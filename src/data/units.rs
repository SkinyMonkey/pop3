use std::io::Read;
use core::mem::size_of;

use crate::data::types::{BinDeserializer, from_reader};

/******************************************************************************/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ModelType {
    Person   = 1,
    Building = 2,
    Creature = 3,
    Vehicle  = 4,
    Scenery  = 5,
    General  = 6,
    Effect   = 7,
    Shot     = 8,
    Shape    = 9,
    Internal = 10,
    Spell    = 11,
}

impl ModelType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1  => Some(Self::Person),
            2  => Some(Self::Building),
            3  => Some(Self::Creature),
            4  => Some(Self::Vehicle),
            5  => Some(Self::Scenery),
            6  => Some(Self::General),
            7  => Some(Self::Effect),
            8  => Some(Self::Shot),
            9  => Some(Self::Shape),
            10 => Some(Self::Internal),
            11 => Some(Self::Spell),
            _  => None,
        }
    }

    pub fn is_visible(&self) -> bool {
        matches!(self,
            Self::Person | Self::Building | Self::Creature |
            Self::Vehicle | Self::Scenery | Self::General | Self::Shape)
    }
}

/******************************************************************************/

/// Raw on-disk THINGSAVE entry (55 bytes).
///
/// Cross-reference: `pop.h:1116-1131` (Pop-World-Editor). Field layout:
/// - byte 0  `Model`  — subtype within type (e.g. Brave=2, Shaman=7). Exposed as `subtype`.
/// - byte 1  `Type`   — top-level type enum (1=Person … 11=Spell). Exposed as `model` (ModelType).
/// - byte 2  `Owner`  — **signed** i8: -1 = neutral, 0..=3 = tribe index. Use [`owner`] not [`tribe_index`].
/// - bytes 3-4  `PosX` (i16 little-endian) — stored here as u16, callers cast.
/// - bytes 5-6  `PosZ` (i16 little-endian)
/// - bytes 7-54 — 48-byte union: see [`ThingPayload`] (BUILDINGSAVE / SCENERYSAVE / GENERALSAVE / TRIGGERSAVE).
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct UnitRaw {
    pub subtype: u8,       // byte 0: THINGSAVE.Model
    pub model: u8,         // byte 1: THINGSAVE.Type
    tribe_index: u8,       // byte 2: THINGSAVE.Owner (signed: 0xFF = -1 = neutral)
    loc_x: u16,            // bytes 3-4: THINGSAVE.PosX
    loc_y: u16,            // bytes 5-6: THINGSAVE.PosZ
    angle: u32,            // bytes 7-10: BUILDINGSAVE.Angle (for buildings only — other variants overlay this region)
    f2: u16,
    f3: u16,
    fd: [u8; 40],
}

impl UnitRaw {
    /// Raw owner byte. **Returns 255 for neutral objects** (the on-disk value is -1 as i8).
    /// Prefer [`owner`] which decodes the signed sentinel into `Option<u8>`.
    pub fn tribe_index(&self) -> u8 { self.tribe_index }

    /// Decoded owner: `None` if the on-disk byte is negative (neutral / unowned),
    /// otherwise `Some(0..=3)` for a tribe slot. Matches `pop.h:1120` `SBYTE Owner`.
    pub fn owner(&self) -> Option<u8> {
        let signed = self.tribe_index as i8;
        if signed < 0 { None } else { Some(signed as u8) }
    }

    pub fn loc_x(&self) -> u16 { self.loc_x }
    pub fn loc_y(&self) -> u16 { self.loc_y }
    pub fn model_type(&self) -> Option<ModelType> { ModelType::from_u8(self.model) }
    pub fn angle(&self) -> u32 { self.angle }
    pub fn f2(&self) -> u16 { self.f2 }
    pub fn f3(&self) -> u16 { self.f3 }
    pub fn fd(&self) -> &[u8; 40] { &self.fd }

    /// Returns the 48-byte union payload as the variant matching `self.model`.
    /// Cross-reference: `pop.h:1116-1131` (THINGSAVE).
    pub fn payload(&self) -> ThingPayload {
        // Reconstruct the contiguous 48-byte union region (bytes 7..55) from the
        // packed fields. `repr(C, packed)` guarantees layout matches the on-disk struct.
        let mut buf = [0u8; 48];
        buf[0..4].copy_from_slice(&self.angle.to_le_bytes());
        buf[4..6].copy_from_slice(&self.f2.to_le_bytes());
        buf[6..8].copy_from_slice(&self.f3.to_le_bytes());
        buf[8..48].copy_from_slice(&self.fd);

        match self.model_type() {
            Some(ModelType::Building) => ThingPayload::Building(BuildingSave::from_bytes(&buf)),
            Some(ModelType::Scenery)  => ThingPayload::Scenery(ScenerySave::from_bytes(&buf)),
            Some(ModelType::General)  => ThingPayload::General(GeneralSave::from_bytes(&buf)),
            // Triggers are encoded as General-type with TriggerType set; the editor uses a
            // dedicated TRIGGERSAVE variant for "trigger" model objects. We don't have a
            // way to distinguish without context, so callers should use [`as_trigger`] explicitly.
            _ => ThingPayload::Other(buf),
        }
    }

    /// Force-decode the union region as a TRIGGERSAVE. Use when caller knows from
    /// level context (e.g. trigger thing-index lists) that the object is a trigger.
    pub fn as_trigger(&self) -> TriggerSave {
        let mut buf = [0u8; 48];
        buf[0..4].copy_from_slice(&self.angle.to_le_bytes());
        buf[4..6].copy_from_slice(&self.f2.to_le_bytes());
        buf[6..8].copy_from_slice(&self.f3.to_le_bytes());
        buf[8..48].copy_from_slice(&self.fd);
        TriggerSave::from_bytes(&buf)
    }
}

impl BinDeserializer for UnitRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<UnitRaw, {size_of::<UnitRaw>()}, R>(reader)
    }
}

/******************************************************************************/

/// THINGSAVE union variants (`pop.h:1116-1131`).
/// The 48-byte union region (bytes 7..55 of THINGSAVE) is interpreted by `Type`.
#[derive(Debug, Clone, Copy)]
pub enum ThingPayload {
    Building(BuildingSave),
    Scenery(ScenerySave),
    General(GeneralSave),
    /// Raw bytes — used for Trigger objects (caller must decode via [`UnitRaw::as_trigger`])
    /// and for types without a structured payload (Person, Creature, etc.).
    Other([u8; 48]),
}

/// `pop.h:1072` `struct BUILDINGSAVE { SLONG Angle; }`.
#[derive(Debug, Clone, Copy)]
pub struct BuildingSave {
    pub angle: i32,
}

impl BuildingSave {
    fn from_bytes(buf: &[u8; 48]) -> Self {
        Self { angle: i32::from_le_bytes(buf[0..4].try_into().unwrap()) }
    }
}

/// `pop.h:1078` `struct SCENERYSAVE`.
#[derive(Debug, Clone, Copy)]
pub struct ScenerySave {
    pub portal_status: u8,
    pub portal_level: u8,
    pub portal_type: u8,
    pub angle: i16,
    pub user_id: u8,
    pub island_alt: i16,
    pub island_num: u8,
    pub bridge_num: u8,
}

impl ScenerySave {
    fn from_bytes(buf: &[u8; 48]) -> Self {
        Self {
            portal_status: buf[0],
            portal_level:  buf[1],
            portal_type:   buf[2],
            angle:         i16::from_le_bytes(buf[3..5].try_into().unwrap()),
            user_id:       buf[5],
            island_alt:    i16::from_le_bytes(buf[6..8].try_into().unwrap()),
            island_num:    buf[8],
            bridge_num:    buf[9],
        }
    }
}

/// `pop.h:1091` `struct GENERALSAVE`.
#[derive(Debug, Clone, Copy)]
pub struct GeneralSave {
    pub discovery_type: u8,
    pub discovery_model: u8,
    pub availability_type: u8,
    pub trigger_type: u8,
    pub mana_amt: i32,
}

impl GeneralSave {
    fn from_bytes(buf: &[u8; 48]) -> Self {
        Self {
            discovery_type:    buf[0],
            discovery_model:   buf[1],
            availability_type: buf[2],
            trigger_type:      buf[3],
            mana_amt:          i32::from_le_bytes(buf[4..8].try_into().unwrap()),
        }
    }
}

/// `pop.h:1101` `struct TRIGGERSAVE`.
#[derive(Debug, Clone, Copy)]
pub struct TriggerSave {
    pub trigger_type: u8,
    pub cell_radius: u8,
    pub random_value: u8,
    pub num_occurences: i8,
    pub trigger_count: u16,
    pub thing_idxs: [u16; 10],
    pub pray_time: i16,
    pub start_inactive: u8,
    pub create_player_owned: u8,
    pub inactive_time: i16,
}

impl TriggerSave {
    fn from_bytes(buf: &[u8; 48]) -> Self {
        let mut thing_idxs = [0u16; 10];
        for (i, idx) in thing_idxs.iter_mut().enumerate() {
            let o = 6 + i * 2;
            *idx = u16::from_le_bytes(buf[o..o+2].try_into().unwrap());
        }
        Self {
            trigger_type:        buf[0],
            cell_radius:         buf[1],
            random_value:        buf[2],
            num_occurences:      buf[3] as i8,
            trigger_count:       u16::from_le_bytes(buf[4..6].try_into().unwrap()),
            thing_idxs,
            pray_time:           i16::from_le_bytes(buf[26..28].try_into().unwrap()),
            start_inactive:      buf[28],
            create_player_owned: buf[29],
            inactive_time:       i16::from_le_bytes(buf[30..32].try_into().unwrap()),
        }
    }
}

/******************************************************************************/

/// `pop.h:1062` `struct PLAYERSAVEINFO`. 16 bytes per player, 4 in a level (one per tribe slot).
///
/// **Was previously misnamed `TribeConfigRaw`.** The actual tribe configuration
/// (starting spells / buildings / vehicles) lives in the `.hdr` file as
/// `PLAYERTHINGS DefaultThings`, not in this struct.
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct PlayerSaveInfo {
    pub start_pos_x: i16,
    pub start_pos_y: i16,
    pub _future1: i32,
    pub _future2: i32,
    pub _future3: i32,
}

impl BinDeserializer for PlayerSaveInfo {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<PlayerSaveInfo, {size_of::<PlayerSaveInfo>()}, R>(reader)
    }
}

/******************************************************************************/


/// Returns the OBJS object index for a completed building.
/// `owner`: `Some(0..=3)` for a tribe, `None` for neutral / out-of-range.
///
/// Huts use 3 consecutive indices per tribe (Small/Medium/Large).
/// Other buildings use 1 index per tribe in blocks of 4.
///
/// Bug fix vs the previous `tribe_index.min(3)` behaviour: a neutral building
/// (owner = -1 as i8 on disk) used to map to tribe 3 (Green). Now returns `None`
/// for neutrals so callers can omit the mesh rather than mis-coloring it.
pub fn building_obj_index(subtype: u8, owner: Option<u8>) -> Option<usize> {
    // Vault of Knowledge (subtype 18) is owner-independent; everything else needs a valid tribe.
    if subtype == 18 {
        return Some(190);
    }
    let tribe = owner.filter(|&t| t < 4)? as usize;
    match subtype {
        1  => Some(145 + tribe * 3),     // Small Hut
        2  => Some(146 + tribe * 3),     // Medium Hut
        3  => Some(147 + tribe * 3),     // Large Hut
        4  => Some(117 + tribe),         // Guard Tower (DrumTower)
        5  => Some(133 + tribe),         // Temple (Preacher Training)
        6  => Some(129 + tribe),         // Spy Training
        7  => Some(141 + tribe),         // Warrior Training
        8  => Some(137 + tribe),         // Firewarrior Training
        13 => Some(121 + tribe),         // Boat Hut
        15 => Some(125 + tribe),         // Balloon Hut (Airship)
        _  => None,
    }
}

/// Returns the OBJS object index for a scenery object.
/// Derived from the scenery data table at 0x5a0790 (field +0x0a per 0x18-byte entry)
/// and Object_InitShapeData @ 0x4bd5b0 which applies subtype-specific overrides.
pub fn scenery_obj_index(subtype: u8) -> Option<usize> {
    match subtype {
        1  => Some(13),  // Mass Tree
        2  => Some(14),  // Special Tree 1
        3  => Some(15),  // Special Tree 2
        4  => Some(16),  // Mass Fruit Tree
        5  => Some(17),  // Special Fruit Tree 1
        6  => Some(18),  // Special Fruit Tree 2
        7  => Some(2),   // Tree variant 7
        8  => Some(3),   // Tree variant 8
        9  => Some(45),  // Stone Head (worship site)
        10 => Some(5),   // Obelisk
        11 => Some(23),  // Totem Pole
        12 => Some(30),  // Discovery Pillar (Reincarnation Site)
        14 => Some(26),  // Additional Tree (Object_InitShapeData forces 0x1a)
        15 => Some(12),  // Bridge/Island
        16 => Some(18),  // Portal/Trigger scenery
        18 => Some(44),  // Vegetation
        19 => Some(39),  // Sub-level Scenery
        _  => None,      // 0, 13 (position-variant), 17 (no model)
    }
}

/// Returns the OBJS object index for any model type + subtype combination.
pub fn object_3d_index(model_type: &ModelType, subtype: u8, owner: Option<u8>) -> Option<usize> {
    match model_type {
        ModelType::Building => building_obj_index(subtype, owner),
        ModelType::Scenery => scenery_obj_index(subtype),
        _ => None,
    }
}

/******************************************************************************/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_decodes_negative_byte_as_neutral() {
        // Construct UnitRaw with tribe_index byte = 0xFF (= -1 as i8 = neutral).
        let mut bytes = [0u8; 55];
        bytes[2] = 0xFF;
        let raw: UnitRaw = unsafe { std::ptr::read(bytes.as_ptr() as *const UnitRaw) };
        assert_eq!(raw.owner(), None, "0xFF (-1) must decode as neutral");
        assert_eq!(raw.tribe_index(), 0xFF, "raw byte still accessible via tribe_index()");
    }

    #[test]
    fn owner_decodes_tribe_byte_as_some() {
        for t in 0..=3u8 {
            let mut bytes = [0u8; 55];
            bytes[2] = t;
            let raw: UnitRaw = unsafe { std::ptr::read(bytes.as_ptr() as *const UnitRaw) };
            assert_eq!(raw.owner(), Some(t), "tribe byte {} must decode as Some({})", t, t);
        }
    }

    #[test]
    fn owner_decodes_high_byte_as_neutral() {
        // Any byte with high bit set (0x80..=0xFF) is a negative i8 — treat as neutral.
        let mut bytes = [0u8; 55];
        bytes[2] = 0x80;
        let raw: UnitRaw = unsafe { std::ptr::read(bytes.as_ptr() as *const UnitRaw) };
        assert_eq!(raw.owner(), None);
    }

    #[test]
    fn building_obj_index_neutral_returns_none_for_tribed_buildings() {
        // Small Hut (subtype 1) with no owner — used to return Some(154) (Green); now returns None.
        assert_eq!(building_obj_index(1, None), None);
        // Same for Drum Tower (subtype 4).
        assert_eq!(building_obj_index(4, None), None);
    }

    #[test]
    fn building_obj_index_vault_is_owner_independent() {
        // Vault of Knowledge (subtype 18) is the same OBJ regardless of owner.
        assert_eq!(building_obj_index(18, None),        Some(190));
        assert_eq!(building_obj_index(18, Some(0)),     Some(190));
        assert_eq!(building_obj_index(18, Some(3)),     Some(190));
    }

    #[test]
    fn building_obj_index_per_tribe_huts() {
        // Small Hut: 145, 148, 151, 154 for Blue/Red/Yellow/Green.
        assert_eq!(building_obj_index(1, Some(0)), Some(145));
        assert_eq!(building_obj_index(1, Some(1)), Some(148));
        assert_eq!(building_obj_index(1, Some(2)), Some(151));
        assert_eq!(building_obj_index(1, Some(3)), Some(154));
    }

    #[test]
    fn building_obj_index_rejects_out_of_range_owner() {
        // owner = 4 (invalid tribe slot) must not collapse to tribe 3.
        assert_eq!(building_obj_index(1, Some(4)), None);
        assert_eq!(building_obj_index(1, Some(99)), None);
    }

    #[test]
    fn payload_decodes_building_angle() {
        let mut bytes = [0u8; 55];
        bytes[0] = 1;                                                // Model (subtype)
        bytes[1] = ModelType::Building as u8;                        // Type
        bytes[2] = 0;                                                // Owner = tribe 0
        bytes[7..11].copy_from_slice(&0x0000_0200i32.to_le_bytes()); // BuildingSave.Angle = 0x200 (90°)
        let raw: UnitRaw = unsafe { std::ptr::read(bytes.as_ptr() as *const UnitRaw) };
        match raw.payload() {
            ThingPayload::Building(b) => assert_eq!(b.angle, 0x0000_0200),
            other => panic!("expected Building variant, got {:?}", other),
        }
    }

    #[test]
    fn payload_decodes_scenery_fields() {
        let mut bytes = [0u8; 55];
        bytes[1] = ModelType::Scenery as u8;
        bytes[7]  = 2;                                                  // portal_status
        bytes[8]  = 3;                                                  // portal_level
        bytes[9]  = 4;                                                  // portal_type
        bytes[10..12].copy_from_slice(&0x0100i16.to_le_bytes());        // angle (45° = 0x100)
        bytes[12] = 7;                                                  // user_id
        bytes[13..15].copy_from_slice(&(-200i16).to_le_bytes());        // island_alt
        bytes[15] = 1;                                                  // island_num
        bytes[16] = 9;                                                  // bridge_num
        let raw: UnitRaw = unsafe { std::ptr::read(bytes.as_ptr() as *const UnitRaw) };
        match raw.payload() {
            ThingPayload::Scenery(s) => {
                assert_eq!(s.portal_status, 2);
                assert_eq!(s.portal_level, 3);
                assert_eq!(s.portal_type, 4);
                assert_eq!(s.angle, 0x0100);
                assert_eq!(s.user_id, 7);
                assert_eq!(s.island_alt, -200);
                assert_eq!(s.island_num, 1);
                assert_eq!(s.bridge_num, 9);
            }
            other => panic!("expected Scenery variant, got {:?}", other),
        }
    }

    #[test]
    fn payload_decodes_general_mana_amt() {
        let mut bytes = [0u8; 55];
        bytes[1] = ModelType::General as u8;
        bytes[7] = 5; // discovery_type
        bytes[8] = 6; // discovery_model
        bytes[9] = 1; // availability_type (AVAILABILITY_PERMANENT)
        bytes[10] = 0; // trigger_type
        bytes[11..15].copy_from_slice(&50_000i32.to_le_bytes()); // mana_amt
        let raw: UnitRaw = unsafe { std::ptr::read(bytes.as_ptr() as *const UnitRaw) };
        match raw.payload() {
            ThingPayload::General(g) => {
                assert_eq!(g.discovery_type, 5);
                assert_eq!(g.discovery_model, 6);
                assert_eq!(g.availability_type, 1);
                assert_eq!(g.mana_amt, 50_000);
            }
            other => panic!("expected General variant, got {:?}", other),
        }
    }

    #[test]
    fn as_trigger_decodes_thing_idxs_and_pray_time() {
        let mut bytes = [0u8; 55];
        bytes[1] = ModelType::General as u8; // triggers are encoded as General in the type byte
        bytes[7]  = 3;                                                       // trigger_type
        bytes[8]  = 5;                                                       // cell_radius
        bytes[9]  = 50;                                                      // random_value
        bytes[10] = (-3i8) as u8;                                            // num_occurences
        bytes[11..13].copy_from_slice(&42u16.to_le_bytes());                 // trigger_count
        // thing_idxs[0..2]: encode two referenced object indices
        bytes[13..15].copy_from_slice(&100u16.to_le_bytes());
        bytes[15..17].copy_from_slice(&200u16.to_le_bytes());
        bytes[33..35].copy_from_slice(&(-1i16).to_le_bytes());               // pray_time
        bytes[35] = 1;                                                       // start_inactive
        bytes[36] = 0;                                                       // create_player_owned
        bytes[37..39].copy_from_slice(&500i16.to_le_bytes());                // inactive_time
        let raw: UnitRaw = unsafe { std::ptr::read(bytes.as_ptr() as *const UnitRaw) };
        let trig = raw.as_trigger();
        assert_eq!(trig.trigger_type, 3);
        assert_eq!(trig.cell_radius, 5);
        assert_eq!(trig.random_value, 50);
        assert_eq!(trig.num_occurences, -3);
        assert_eq!(trig.trigger_count, 42);
        assert_eq!(trig.thing_idxs[0], 100);
        assert_eq!(trig.thing_idxs[1], 200);
        assert_eq!(trig.pray_time, -1);
        assert_eq!(trig.start_inactive, 1);
        assert_eq!(trig.inactive_time, 500);
    }
}
