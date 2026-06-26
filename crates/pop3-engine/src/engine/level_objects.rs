// Level object extraction: level data -> renderable object placements.
use crate::data::level::LevelRes;
use crate::data::units::ModelType;

pub struct LevelObject {
    pub cell_x: f32,
    pub cell_y: f32,
    pub model_type: ModelType,
    #[allow(dead_code)]
    pub subtype: u8,
    pub tribe_index: u8,
    pub angle: u32,
}

pub fn extract_level_objects(level_res: &LevelRes) -> Vec<LevelObject> {
    let n = level_res.landscape.land_size() as f32;
    let mut objects = Vec::new();
    for unit in &level_res.units {
        let model_type = match unit.model_type() {
            Some(mt) if mt.is_visible() => mt,
            _ => continue,
        };
        if unit.loc_x() == 0 && unit.loc_y() == 0 {
            continue;
        }
        let bevy_x = ((unit.loc_x() >> 8) / 2) as f32 + 0.5;
        let bevy_z = ((unit.loc_y() >> 8) / 2) as f32 + 0.5;
        let cell_x = bevy_z;
        let cell_y = (n - 1.0) - bevy_x;
        eprintln!("[extract] type={:?} subtype={} tribe={} angle={} loc=({},{})",
            model_type, unit.subtype, unit.tribe_index(), unit.angle(),
            unit.loc_x(), unit.loc_y());
        objects.push(LevelObject {
            cell_x,
            cell_y,
            model_type,
            subtype: unit.subtype,
            tribe_index: unit.tribe_index(),
            angle: unit.angle(),
        });
    }
    objects
}

