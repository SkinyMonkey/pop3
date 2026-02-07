use std::io::Read;
use core::mem::size_of;

use crate::pop::types::{BinDeserializer, from_reader};

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

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct UnitRaw {
    pub subtype: u8,       // byte 0: subtype within model (e.g. Brave=2, Shaman=7)
    pub model: u8,         // byte 1: model type (1=Person, 2=Building, 3=Creature, ...)
    tribe_index: u8,       // byte 2: owner tribe (0-3, or 255=unowned)
    loc_x: u16,            // bytes 3-4: world X position
    loc_y: u16,            // bytes 5-6: world Y position
    angle: u32,            // bytes 7-10: rotation angle (game uses angle/512 for buildings)
    f2: u16,
    f3: u16,
    fd: [u8; 40],
}

impl UnitRaw {
    pub fn tribe_index(&self) -> u8 { self.tribe_index }
    pub fn loc_x(&self) -> u16 { self.loc_x }
    pub fn loc_y(&self) -> u16 { self.loc_y }
    pub fn model_type(&self) -> Option<ModelType> { ModelType::from_u8(self.model) }
    pub fn angle(&self) -> u32 { self.angle }
    pub fn f2(&self) -> u16 { self.f2 }
    pub fn f3(&self) -> u16 { self.f3 }
    pub fn fd(&self) -> &[u8; 40] { &self.fd }
}

impl BinDeserializer for UnitRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<UnitRaw, {size_of::<UnitRaw>()}, R>(reader)
    }
}

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct TribeConfigRaw {
    pub data: [u8; 16],
}

impl BinDeserializer for TribeConfigRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<TribeConfigRaw, {size_of::<TribeConfigRaw>()}, R>(reader)
    }
}

/******************************************************************************/

/// Returns the OBJS object index for a completed building.
/// tribe_index: 0-3 (Blue, Red, Yellow, Green)
///
/// Huts use 3 consecutive indices per tribe (Small/Medium/Large).
/// Other buildings use 1 index per tribe in blocks of 4.
pub fn building_obj_index(subtype: u8, tribe_index: u8) -> Option<usize> {
    let tribe = tribe_index.min(3) as usize;
    match subtype {
        1  => Some(145 + tribe * 3),     // Small Hut (style 1)
        2  => Some(146 + tribe * 3),     // Medium Hut (style 1)
        3  => Some(147 + tribe * 3),     // Large Hut (style 1)
        13 => Some(117 + tribe),         // Guard Tower
        9  => Some(125 + tribe),         // Balloon Hut (Airship)
        11 => Some(121 + tribe),         // Boat Hut
        6  => Some(129 + tribe),         // Spy Training
        5  => Some(133 + tribe),         // Preacher Training (Temple)
        8  => Some(137 + tribe),         // FW Training
        7  => Some(141 + tribe),         // Warrior Training
        19 => Some(190),                 // Vault of Knowledge
        _  => None,
    }
}

/// Returns the OBJS object index for any model type + subtype combination.
/// Currently only Building types are mapped.
/// General/Scenery types (vault, stone heads, etc.) also have 3D meshes
/// but their OBJS index depends on runtime state (discovery type from
/// the parameter stack mechanism in the game engine).
pub fn object_3d_index(model_type: &ModelType, subtype: u8, tribe_index: u8) -> Option<usize> {
    match model_type {
        ModelType::Building => building_obj_index(subtype, tribe_index),
        _ => None,
    }
}

/******************************************************************************/
