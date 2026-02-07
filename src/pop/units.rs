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
    loc_x: u16,
    loc_y: u16,
    f1: u32,
    f2: u16,
    f3: u16,
    fd: [u8; 40],
}

impl UnitRaw {
    pub fn tribe_index(&self) -> u8 { self.tribe_index }
    pub fn loc_x(&self) -> u16 { self.loc_x }
    pub fn loc_y(&self) -> u16 { self.loc_y }
    pub fn model_type(&self) -> Option<ModelType> { ModelType::from_u8(self.model) }
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
