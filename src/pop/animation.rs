use std::path::Path;
use std::fs::File;
use std::io::Read;
use core::mem::size_of;

use crate::pop::types::{BinDeserializer, from_reader, ImageInfo};

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct VeleRaw {
    pub sprite_index: u16,
    pub coord_x: i16,
    pub coord_y: i16,
    pub flags: u16,
    pub next_index: u16,
}

impl BinDeserializer for VeleRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<VeleRaw, {size_of::<VeleRaw>()}, R>(reader)
    }
}

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct VfraRaw {
    pub index: u16,
    pub width: u8,
    pub height: u8,
    pub f3: u8,
    pub f4: u8,
    pub next_vfra: u16,
}

impl BinDeserializer for VfraRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<VfraRaw, {size_of::<VfraRaw>()}, R>(reader)
    }
}

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct VstartRaw {
    pub index: u16,
    pub f1: u8,
    pub f2: u8,
}

impl BinDeserializer for VstartRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<VstartRaw, {size_of::<VstartRaw>()}, R>(reader)
    }
}

/******************************************************************************/

#[derive(Debug)]
pub struct AnimationsData {
    pub vele: Vec<VeleRaw>,
    pub vfra: Vec<VfraRaw>,
    pub vstart: Vec<VstartRaw>,
}

impl AnimationsData {

    pub fn from_reader<R: Read>(reader_vele: &mut R
                               , reader_vfra: &mut R
                               , reader_vstart: &mut R) -> Self {
        AnimationsData {
            vele: VeleRaw::from_reader_vec(reader_vele),
            vfra: VfraRaw::from_reader_vec(reader_vfra),
            vstart: VstartRaw::from_reader_vec(reader_vstart),
        }
    }

    pub fn from_path(path: &Path) -> Self {
        let mut file_vele = File::options().read(true).open(path.join("VELE-0.ANI")).unwrap();
        let mut file_vfra = File::options().read(true).open(path.join("VFRA-0.ANI")).unwrap();
        let mut file_vstart = File::options().read(true).open(path.join("VSTART-0.ANI")).unwrap();
        Self::from_reader(&mut file_vele, &mut file_vfra, &mut file_vstart)
    }
}

use crate::pop::psfb::ContainerPSFB;

/******************************************************************************/

pub const DIRS_PER_ANIM: usize = 8;
pub const STORED_DIRECTIONS: usize = 5;
pub const NUM_TRIBES: usize = 4;

/// Idle animation indices from g_PersonAnimationTable (RE'd from original binary)
/// Format: (subtype, animation_index)
pub const UNIT_IDLE_ANIMS: [(u8, usize); 6] = [
    (2, 15),  // Brave
    (3, 16),  // Warrior
    (4, 17),  // Preacher
    (5, 18),  // Spy
    (6, 19),  // Firewarrior
    (7, 20),  // Shaman
];

/******************************************************************************/

pub enum ElementRotate {
    NoRotate,
    RotateHorizontal,
    RotateVertical,
}

#[derive(Debug, Copy, Clone)]
pub struct AnimationElement {
    pub sprite_index: usize,
    pub coord_x: i16,
    pub coord_y: i16,
    pub tribe: u8,
    pub flags: u16,
    pub uvar5: u16,
    pub original_flags: u16,
}

#[derive(Debug, Clone)]
pub struct AnimationFrame {
    pub index: usize,
    pub width: usize,
    pub height: usize,
    pub sprites: Vec<AnimationElement>,
}

pub struct AnimationSequence {
    pub index: usize,
    pub frames: Vec<AnimationFrame>,
}

impl AnimationElement {
    pub fn get_tribe(&self) -> u8 {
        self.tribe
    }

    pub fn is_hidden(&self) -> bool {
        false
    }

    pub fn is_common(&self) -> bool {
        !(self.is_tribe_specific() || self.is_type_specific())
    }

    pub fn is_tribe_specific(&self) -> bool {
        self.uvar5 == 1
    }

    pub fn is_type_specific(&self) -> bool {
        self.uvar5 > 1
    }

    pub fn get_rotate(&self) -> ElementRotate {
        if (self.flags & 0x1) != 0 {
            ElementRotate::RotateHorizontal
        } else if (self.flags & 0x2) != 0 {
            ElementRotate::RotateVertical
        } else {
            ElementRotate::NoRotate
        }
    }

    pub fn from_data(index: u16, vele: &[VeleRaw]) -> Vec<Self> {
        let mut sprites = Vec::new();
        let mut vele_index = index as usize;
        while vele_index != 0 {
            let vele_sprite = &vele[vele_index];
            sprites.push(AnimationElement{
                sprite_index: (vele_sprite.sprite_index as usize / 6).saturating_sub(1),
                coord_x: vele_sprite.coord_x,
                coord_y: vele_sprite.coord_y,
                tribe: (vele_sprite.flags >> 9) as u8,
                flags: vele_sprite.flags & 0x1f,
                uvar5: (vele_sprite.flags & 0x1f0) >> 4,
                original_flags: vele_sprite.flags,
            });
            vele_index = vele_sprite.next_index as usize;
            if sprites.len() > 255 {
                break;
            }
        }
        sprites
    }
}

impl AnimationFrame {
    pub fn get_permutations(&self, with_tribe: bool, with_type: bool) -> Vec<Vec<AnimationElement>> {
        let mut common_elems = Vec::new();
        let mut tribe_elems = Vec::new();
        let mut type_elems = Vec::new();
        for elem in &self.sprites {
            if elem.is_hidden() {
                continue;
            }
            if elem.is_common() {
                common_elems.push(*elem);
                if with_type {
                    type_elems.push(*elem);
                }
                if with_tribe {
                    tribe_elems.push(*elem);
                }
            } else if with_tribe && elem.is_tribe_specific() {
                tribe_elems.push(*elem);
            } else if with_type && elem.is_type_specific() {
                type_elems.push(*elem);
            }
        }
        let mut res = Vec::new();
        if tribe_elems.is_empty() && type_elems.is_empty() {
            res.push(common_elems);
        } else if !tribe_elems.is_empty() {
            for tribe_elem in tribe_elems {
                let mut res_tribe = common_elems.clone();
                res_tribe.push(tribe_elem);
                if type_elems.is_empty() {
                    res.push(res_tribe);
                } else {
                    for type_elem in &type_elems {
                        let mut res_type = res_tribe.clone();
                        res_type.push(*type_elem);
                        res.push(res_type);
                    }
                }
            }
        } else {
            for type_elem in &type_elems {
                let mut res_type = common_elems.clone();
                res_type.push(*type_elem);
                res.push(res_type);
            }
        }
        res
    }

    pub fn from_data(index: u16, vfra: &[VfraRaw], vele: &[VeleRaw]) -> Vec<Self> {
        let mut frames = Vec::new();
        let mut vfra_index = index as usize;
        while vfra_index != 0 {
            let vfra_frame = &vfra[vfra_index];
            frames.push(AnimationFrame{
                index: vfra_index,
                width: vfra_frame.width as usize,
                height: vfra_frame.height as usize,
                sprites: AnimationElement::from_data(vfra_frame.index, vele),
            });
            vfra_index = vfra_frame.next_vfra as usize;
            if frames.len() > 255 {
                break;
            }
            if vfra_index == (index as usize) {
                break;
            }
        }
        frames
    }
}

impl ImageInfo for AnimationFrame {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl AnimationSequence {
    pub fn from_data(anim_data: &AnimationsData) -> Vec<Self> {
        let mut res = Vec::<Self>::with_capacity(anim_data.vstart.len());
        for (index, vstart) in (0..).zip(&anim_data.vstart) {
            let frames = AnimationFrame::from_data(vstart.index, &anim_data.vfra, &anim_data.vele);
            res.push(AnimationSequence{index, frames});
        }
        res
    }

    pub fn get_frames(anim_seq_vec: &Vec<Self>) -> Vec<AnimationFrame> {
        let mut frames = Vec::new();
        for anim_seq in anim_seq_vec {
            frames.extend_from_slice(&anim_seq.frames);
        }
        frames
    }
}

/******************************************************************************/
// Sprite compositing helpers (used by main renderer and unit_viewer)
/******************************************************************************/

/// Check if an element should be rendered given the current tribe and unit combo.
/// Layer 0 (base): always render
/// Layer 1 (tribe): render if element's tribe matches selected tribe
/// Layer 2+ (type): render only if (uvar5, tribe) matches selected unit combo
pub fn should_render_element(
    elem: &AnimationElement,
    tribe: u8,
    unit_combo: Option<(u16, u16)>,
) -> bool {
    if elem.is_hidden() {
        return false;
    }
    if elem.is_common() {
        return true;
    }
    if elem.is_tribe_specific() {
        return elem.get_tribe() == tribe;
    }
    if elem.is_type_specific() {
        return match unit_combo {
            Some((layer, high)) => elem.uvar5 == layer && elem.tribe as u16 == high,
            None => false,
        };
    }
    true
}

/// Discover available unit combos (layer_type, element_tribe) from an animation's elements.
pub fn discover_unit_combos(
    sequences: &[AnimationSequence],
    base: usize,
) -> Vec<(u16, u16)> {
    let mut combos: Vec<(u16, u16)> = Vec::new();
    for dir in 0..STORED_DIRECTIONS {
        let seq_idx = base + dir;
        if seq_idx >= sequences.len() { continue; }
        for frame in &sequences[seq_idx].frames {
            for elem in &frame.sprites {
                if elem.is_type_specific() {
                    let combo = (elem.uvar5, elem.tribe as u16);
                    if !combos.contains(&combo) {
                        combos.push(combo);
                    }
                }
            }
        }
    }
    combos.sort();
    combos
}

/// Composite a single animation frame's elements into an RGBA bitmap.
pub fn composite_frame(
    elements: &[AnimationElement],
    container: &ContainerPSFB,
    palette: &[[u8; 4]],
    tribe: u8,
    unit_combo: Option<(u16, u16)>,
    frame_width: usize,
    frame_height: usize,
    offset_x: i32,
    offset_y: i32,
) -> Vec<u8> {
    let fw = frame_width;
    let fh = frame_height;
    let mut rgba = vec![0u8; fw * fh * 4];

    for elem in elements {
        if !should_render_element(elem, tribe, unit_combo) {
            continue;
        }

        let sprite_index = elem.sprite_index;
        let image = match container.get_image(sprite_index) {
            Some(img) => img,
            None => continue,
        };
        let info = match container.get_info(sprite_index) {
            Some(i) => i,
            None => continue,
        };

        let sw = info.width as usize;
        let sh = info.height as usize;

        let h_flip = matches!(elem.get_rotate(), ElementRotate::RotateHorizontal);
        let v_flip = matches!(elem.get_rotate(), ElementRotate::RotateVertical);

        let ox = (elem.coord_x as i32 - offset_x) as isize;
        let oy = (elem.coord_y as i32 - offset_y) as isize;

        for y in 0..sh {
            for x in 0..sw {
                let src_x = if h_flip { sw - 1 - x } else { x };
                let src_y = if v_flip { sh - 1 - y } else { y };
                let src = image.data[src_y * sw + src_x];
                if src == 255 { continue; }

                let dst_x = ox + x as isize;
                let dst_y = oy + y as isize;
                if dst_x < 0 || dst_y < 0 || dst_x >= fw as isize || dst_y >= fh as isize {
                    continue;
                }

                let dst_off = (dst_y as usize * fw + dst_x as usize) * 4;
                let c = palette.get(src as usize).unwrap_or(&[255, 0, 255, 255]);
                rgba[dst_off] = c[0];
                rgba[dst_off + 1] = c[1];
                rgba[dst_off + 2] = c[2];
                rgba[dst_off + 3] = 255;
            }
        }
    }

    rgba
}

/// Compute a bounding box across ALL animations for consistent frame sizing.
/// Returns (min_x, min_y, max_x, max_y) encompassing all non-hidden elements.
pub fn compute_global_bbox(
    sequences: &[AnimationSequence],
    container: &ContainerPSFB,
) -> (i32, i32, i32, i32) {
    let mut min_x: i32 = 0;
    let mut min_y: i32 = 0;
    let mut max_x: i32 = 1;
    let mut max_y: i32 = 1;

    for seq in sequences {
        for frame in &seq.frames {
            for elem in &frame.sprites {
                if elem.is_hidden() { continue; }
                if let Some(info) = container.get_info(elem.sprite_index) {
                    let ex = elem.coord_x as i32;
                    let ey = elem.coord_y as i32;
                    min_x = min_x.min(ex);
                    min_y = min_y.min(ey);
                    max_x = max_x.max(ex + info.width as i32);
                    max_y = max_y.max(ey + info.height as i32);
                }
            }
        }
    }

    (min_x, min_y, max_x, max_y)
}

/// Build a sprite atlas for an animation with all 4 tribes.
/// Layout: rows = 4 tribes × 5 stored directions, cols = frames.
/// Returns (atlas_w, atlas_h, rgba, frame_w, frame_h, frames_per_dir) or None.
/// `unit_combo_override`: `None` = auto-detect first combo, `Some(x)` = use `x` as the combo.
/// `bbox_override`: when `Some`, use the provided (min_x, min_y, max_x, max_y) instead of
/// computing a per-animation bounding box. Pass a global bbox for consistent sizing across animations.
pub fn build_tribe_atlas(
    sequences: &[AnimationSequence],
    container: &ContainerPSFB,
    palette: &[[u8; 4]],
    anim_index: usize,
    unit_combo_override: Option<Option<(u16, u16)>>,
    bbox_override: Option<(i32, i32, i32, i32)>,
) -> Option<(u32, u32, Vec<u8>, u32, u32, u32)> {
    let base = anim_index * DIRS_PER_ANIM;

    // Count max frames per direction
    let mut max_frames = 0usize;
    for dir in 0..STORED_DIRECTIONS {
        let seq_idx = base + dir;
        if seq_idx >= sequences.len() { continue; }
        max_frames = max_frames.max(sequences[seq_idx].frames.len());
    }
    if max_frames == 0 { return None; }

    // Resolve unit combo BEFORE bounding box so we can filter elements
    let unit_combo = match unit_combo_override {
        Some(combo) => combo,
        None => {
            let combos = discover_unit_combos(sequences, base);
            combos.first().copied()
        }
    };

    let (min_x, min_y, max_x, max_y) = if let Some(bbox) = bbox_override {
        bbox
    } else {
        // Compute bounding box across rendered elements only.
        // Type-specific elements (uvar5 > 1) are excluded when unit_combo is None,
        // matching bevy_demo5 which filters elements before compositing.
        let mut min_x: i32 = 0;
        let mut min_y: i32 = 0;
        let mut max_x: i32 = 1;
        let mut max_y: i32 = 1;

        for dir in 0..STORED_DIRECTIONS {
            let seq_idx = base + dir;
            if seq_idx >= sequences.len() { continue; }
            for frame in &sequences[seq_idx].frames {
                for elem in &frame.sprites {
                    if elem.is_hidden() { continue; }
                    if elem.is_type_specific() {
                        match unit_combo {
                            Some((layer, high)) => {
                                if elem.uvar5 != layer || elem.tribe as u16 != high { continue; }
                            }
                            None => continue,
                        }
                    }
                    if let Some(info) = container.get_info(elem.sprite_index) {
                        let ex = elem.coord_x as i32;
                        let ey = elem.coord_y as i32;
                        min_x = min_x.min(ex);
                        min_y = min_y.min(ey);
                        max_x = max_x.max(ex + info.width as i32);
                        max_y = max_y.max(ey + info.height as i32);
                    }
                }
            }
        }
        (min_x, min_y, max_x, max_y)
    };

    let fw = ((max_x - min_x) as u32).max(1).min(512);
    let fh = ((max_y - min_y) as u32).max(1).min(512);
    let total_rows = (NUM_TRIBES * STORED_DIRECTIONS) as u32;
    let atlas_w = fw * max_frames as u32;
    let atlas_h = fh * total_rows;

    if atlas_w == 0 || atlas_h == 0 { return None; }

    let mut rgba = vec![0u8; (atlas_w * atlas_h * 4) as usize];

    for tribe in 0..NUM_TRIBES {
        for dir in 0..STORED_DIRECTIONS {
            let seq_idx = base + dir;
            if seq_idx >= sequences.len() { continue; }

            let dir_frames = &sequences[seq_idx].frames;
            if dir_frames.is_empty() { continue; }
            for f in 0..max_frames {
                let frame = &dir_frames[f % dir_frames.len()];

                let frame_rgba = composite_frame(
                    &frame.sprites,
                    container,
                    palette,
                    tribe as u8,
                    unit_combo,
                    fw as usize,
                    fh as usize,
                    min_x,
                    min_y,
                );

                let cell_x = f as u32 * fw;
                let row = (tribe * STORED_DIRECTIONS + dir) as u32;
                let cell_y = row * fh;
                for y in 0..fh {
                    let src_row = (y * fw * 4) as usize;
                    let dst_row = ((cell_y + y) * atlas_w + cell_x) as usize * 4;
                    let len = (fw * 4) as usize;
                    rgba[dst_row..dst_row + len]
                        .copy_from_slice(&frame_rgba[src_row..src_row + len]);
                }
            }
        }
    }

    Some((atlas_w, atlas_h, rgba, fw, fh, max_frames as u32))
}

/******************************************************************************/
