//! VELE Compositing Demo
//!
//! Properly renders animations using the VELE element chain system.
//! Each frame can consist of multiple sprite layers composited together.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::window::PresentMode;
use std::collections::HashMap;

// =============================================================================
// PATHS AND CONSTANTS
// =============================================================================

const DATA_PATH: &str = "/Users/adriencandiotti/Library/Containers/com.isaacmarovitz.Whisky/Bottles/74820C9D-5F8C-4BFE-B5DB-90E1DE818D3F/drive_c/GOG Games/Populous - The Beginning/data";

const NUM_DIRECTIONS: usize = 8;
const STORED_DIRECTIONS: usize = 5;

// =============================================================================
// ANIMATION FILE STRUCTURES (from Ghidra analysis)
// =============================================================================

/// VSTART entry (4 bytes) - animation start indices
#[derive(Debug, Clone, Copy)]
struct VstartEntry {
    vfra_index: u16,
    _mirror_ref: u16,
}

/// VFRA entry (8 bytes) - frame linked list
#[derive(Debug, Clone, Copy)]
struct VfraEntry {
    first_element: u16,  // Index into VELE or direct sprite
    _width: u8,
    _height: u8,
    flags: u16,
    next_frame: u16,
}

/// VELE entry (10 bytes) - sprite element with offset
#[derive(Debug, Clone, Copy)]
struct VeleEntry {
    sprite_idx: u16,
    x_offset: i16,
    y_offset: i16,
    flip_flags: u16,
    next_element: u16,
}

// =============================================================================
// FILE LOADING
// =============================================================================

fn load_vstart() -> Option<Vec<VstartEntry>> {
    let path = format!("{}/VSTART-0.ANI", DATA_PATH);
    let data = std::fs::read(&path).ok()?;

    let entry_count = data.len() / 4;
    let mut entries = Vec::with_capacity(entry_count);

    for i in 0..entry_count {
        let offset = i * 4;
        entries.push(VstartEntry {
            vfra_index: u16::from_le_bytes([data[offset], data[offset + 1]]),
            _mirror_ref: u16::from_le_bytes([data[offset + 2], data[offset + 3]]),
        });
    }

    println!("Loaded VSTART: {} entries ({} animations × 8 dirs)", entry_count, entry_count / 8);
    Some(entries)
}

fn load_vfra() -> Option<Vec<VfraEntry>> {
    let path = format!("{}/VFRA-0.ANI", DATA_PATH);
    let data = std::fs::read(&path).ok()?;

    let entry_count = data.len() / 8;
    let mut entries = Vec::with_capacity(entry_count);

    for i in 0..entry_count {
        let offset = i * 8;
        entries.push(VfraEntry {
            first_element: u16::from_le_bytes([data[offset], data[offset + 1]]),
            _width: data[offset + 2],
            _height: data[offset + 3],
            flags: u16::from_le_bytes([data[offset + 4], data[offset + 5]]),
            next_frame: u16::from_le_bytes([data[offset + 6], data[offset + 7]]),
        });
    }

    println!("Loaded VFRA: {} entries", entry_count);
    Some(entries)
}

fn load_vele() -> Option<Vec<VeleEntry>> {
    let path = format!("{}/VELE-0.ANI", DATA_PATH);
    let data = std::fs::read(&path).ok()?;

    let entry_count = data.len() / 10;
    let mut entries = Vec::with_capacity(entry_count);

    for i in 0..entry_count {
        let offset = i * 10;
        entries.push(VeleEntry {
            sprite_idx: u16::from_le_bytes([data[offset], data[offset + 1]]),
            x_offset: i16::from_le_bytes([data[offset + 2], data[offset + 3]]),
            y_offset: i16::from_le_bytes([data[offset + 4], data[offset + 5]]),
            flip_flags: u16::from_le_bytes([data[offset + 6], data[offset + 7]]),
            next_element: u16::from_le_bytes([data[offset + 8], data[offset + 9]]),
        });
    }

    println!("Loaded VELE: {} entries", entry_count);
    Some(entries)
}

fn load_palette() -> Option<Vec<[u8; 4]>> {
    let path = format!("{}/pal0-0.dat", DATA_PATH);
    let data = std::fs::read(&path).ok()?;

    if data.len() < 1024 {
        return None;
    }

    let mut palette = Vec::with_capacity(256);
    for i in 0..256 {
        let offset = i * 4;
        palette.push([data[offset], data[offset + 1], data[offset + 2], 255]);
    }
    Some(palette)
}

fn get_sprite_count() -> Option<usize> {
    let path = format!("{}/HSPR0-0.DAT", DATA_PATH);
    let data = std::fs::read(&path).ok()?;

    if data.len() < 8 || &data[0..4] != b"PSFB" {
        return None;
    }

    Some(u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize)
}

// =============================================================================
// SPRITE LOADING
// =============================================================================

struct SpriteFrame {
    width: u16,
    height: u16,
    pixels: Vec<Vec<u8>>,
}

fn load_sprite_frame(frame_index: usize) -> Option<SpriteFrame> {
    let path = format!("{}/HSPR0-0.DAT", DATA_PATH);
    let data = std::fs::read(&path).ok()?;

    if data.len() < 8 || &data[0..4] != b"PSFB" {
        return None;
    }

    let frame_count = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
    if frame_index >= frame_count {
        return None;
    }

    let header_offset = 8 + frame_index * 8;
    let width = u16::from_le_bytes([data[header_offset], data[header_offset + 1]]);
    let height = u16::from_le_bytes([data[header_offset + 2], data[header_offset + 3]]);
    let pixel_offset = u32::from_le_bytes([
        data[header_offset + 4], data[header_offset + 5],
        data[header_offset + 6], data[header_offset + 7],
    ]) as usize;

    let mut pos = pixel_offset;
    let mut pixels = vec![vec![255u8; width as usize]; height as usize];

    for row in 0..height as usize {
        let mut col = 0usize;
        while pos < data.len() {
            let val = data[pos] as i8;
            pos += 1;

            if val == 0 {
                break;
            } else if val > 0 {
                for _ in 0..val {
                    if col < width as usize && pos < data.len() {
                        pixels[row][col] = data[pos];
                        pos += 1;
                        col += 1;
                    }
                }
            } else {
                col += (-val) as usize;
            }
        }
    }

    Some(SpriteFrame { width, height, pixels })
}

// =============================================================================
// ANIMATION PARSING
// =============================================================================

/// A parsed animation with all its frames for all directions
struct ParsedAnimation {
    index: usize,
    directions: Vec<Vec<AnimationFrame>>,  // [direction][frame]
}

/// A single animation frame (may have multiple VELE elements)
struct AnimationFrame {
    elements: Vec<FrameElement>,
}

/// A single element within a frame
struct FrameElement {
    sprite_idx: u16,
    x_offset: i16,
    y_offset: i16,
    flip: bool,
}

fn parse_animation(
    anim_index: usize,
    vstart: &[VstartEntry],
    vfra: &[VfraEntry],
    vele: &[VeleEntry],
    sprite_count: usize,
) -> Option<ParsedAnimation> {
    let base = anim_index * 8;
    if base + 8 > vstart.len() {
        return None;
    }

    let mut directions = Vec::new();

    // Only parse the 5 stored directions (0-4), we'll mirror 5-7 later
    for dir in 0..STORED_DIRECTIONS {
        let start_entry = &vstart[base + dir];
        let mut frames = Vec::new();

        // Follow VFRA linked list
        let start_vfra = start_entry.vfra_index as usize;
        let mut vfra_idx = start_vfra;
        let mut frame_count = 0;
        let max_frames = 100; // Safety limit

        if dir == 0 {
            println!("  Dir {}: VSTART vfra_index={}", dir, start_vfra);
        }

        loop {
            if vfra_idx >= vfra.len() || frame_count >= max_frames {
                break;
            }

            let vfra_entry = &vfra[vfra_idx];

            let mut elements = Vec::new();
            let first_elem = vfra_entry.first_element as usize;

            // first_element is a VELE index
            // VELE sprite_idx on disk uses formula: actual_sprite = file_val / 6 - 1
            // Skip shadow layer (sprite 22) and use the body sprite (second element)
            if first_elem < vele.len() {
                let mut vele_idx = first_elem;
                let mut vele_visited = std::collections::HashSet::new();

                while vele_idx < vele.len() && !vele_visited.contains(&vele_idx) {
                    vele_visited.insert(vele_idx);
                    let vele_entry = &vele[vele_idx];

                    let actual_sprite = (vele_entry.sprite_idx as usize) / 6 - 1;
                    if actual_sprite < sprite_count {
                        elements.push(FrameElement {
                            sprite_idx: actual_sprite as u16,
                            x_offset: vele_entry.x_offset,
                            y_offset: vele_entry.y_offset,
                            flip: vele_entry.flip_flags & 1 != 0,
                        });
                    }

                    if vele_entry.next_element == 0 || vele_entry.next_element as usize >= vele.len() {
                        break;
                    }
                    vele_idx = vele_entry.next_element as usize;
                }
                if dir == 0 && frame_count < 3 {
                    let sprite_list: Vec<u16> = elements.iter().map(|e| e.sprite_idx).collect();
                    println!("    Frame {}: VELE[{}] -> sprites {:?}",
                             frame_count, first_elem, sprite_list);
                }
            }

            if !elements.is_empty() {
                frames.push(AnimationFrame { elements });
            }

            frame_count += 1;

            // Move to next frame
            let next = vfra_entry.next_frame as usize;

            // Stop if we've looped back to start
            if next == start_vfra {
                break;
            }

            vfra_idx = next;
        }

        if dir == 0 {
            println!("  Dir 0: {} frames loaded", frames.len());
        }

        directions.push(frames);
    }

    // Mirror directions 5-7 from 3, 2, 1
    // For now, just clone the source direction's frames
    for _ in STORED_DIRECTIONS..NUM_DIRECTIONS {
        // We'll handle mirroring in the rendering, just need empty vecs
        directions.push(Vec::new());
    }

    // Skip empty animations
    if directions[0].is_empty() {
        return None;
    }

    Some(ParsedAnimation {
        index: anim_index,
        directions,
    })
}

// =============================================================================
// COMPOSITING
// =============================================================================

fn composite_frame(
    frame: &AnimationFrame,
    palette: &[[u8; 4]],
    sprite_cache: &mut HashMap<u16, Option<SpriteFrame>>,
) -> Option<(Image, i32, i32)> {
    if frame.elements.is_empty() {
        return None;
    }

    // Find bounding box
    let mut min_x: i32 = 0;
    let mut min_y: i32 = 0;
    let mut max_x: i32 = 0;
    let mut max_y: i32 = 0;

    for elem in &frame.elements {
        let sprite = sprite_cache
            .entry(elem.sprite_idx)
            .or_insert_with(|| load_sprite_frame(elem.sprite_idx as usize));

        if let Some(sprite) = sprite {
            let x1 = elem.x_offset as i32;
            let y1 = elem.y_offset as i32;
            let x2 = x1 + sprite.width as i32;
            let y2 = y1 + sprite.height as i32;

            min_x = min_x.min(x1);
            min_y = min_y.min(y1);
            max_x = max_x.max(x2);
            max_y = max_y.max(y2);
        }
    }

    let width = (max_x - min_x).max(1) as usize;
    let height = (max_y - min_y).max(1) as usize;

    // Create canvas
    let mut rgba = vec![0u8; width * height * 4];

    // Composite each element
    for elem in &frame.elements {
        let sprite = sprite_cache.get(&elem.sprite_idx).and_then(|s| s.as_ref());

        if let Some(sprite) = sprite {
            let base_x = (elem.x_offset as i32 - min_x) as usize;
            let base_y = (elem.y_offset as i32 - min_y) as usize;

            for (sy, row) in sprite.pixels.iter().enumerate() {
                for (sx, &pixel_idx) in row.iter().enumerate() {
                    if pixel_idx == 255 {
                        continue; // Transparent
                    }

                    let dx = if elem.flip {
                        base_x + (sprite.width as usize - 1 - sx)
                    } else {
                        base_x + sx
                    };
                    let dy = base_y + sy;

                    if dx < width && dy < height {
                        let offset = (dy * width + dx) * 4;
                        let color = palette.get(pixel_idx as usize).unwrap_or(&[255, 0, 255, 255]);
                        rgba[offset] = color[0];
                        rgba[offset + 1] = color[1];
                        rgba[offset + 2] = color[2];
                        rgba[offset + 3] = 255;
                    }
                }
            }
        }
    }

    let image = Image::new(
        bevy::render::render_resource::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        rgba,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );

    Some((image, -min_x, -min_y))
}

// =============================================================================
// BEVY COMPONENTS AND RESOURCES
// =============================================================================

#[derive(Resource)]
struct AnimationFiles {
    vstart: Vec<VstartEntry>,
    vfra: Vec<VfraEntry>,
    vele: Vec<VeleEntry>,
    palette: Vec<[u8; 4]>,
    sprite_count: usize,
}

#[derive(Resource)]
struct CurrentAnimation {
    index: usize,
    parsed: Option<ParsedAnimation>,
    frame_materials: Vec<Vec<Handle<StandardMaterial>>>,  // [direction][frame]
    is_static_pose: bool,
}

#[derive(Resource)]
struct AnimationSettings {
    speed: f32,
    paused: bool,
    game_tick: u32,
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            speed: 0.1,  // Slower - ~10 FPS
            paused: false,
            game_tick: 0,
        }
    }
}

#[derive(Component)]
struct AnimatedSprite {
    direction: u8,
    current_frame: usize,
    timer: f32,
}

#[derive(Component)]
struct InfoLabel;

fn get_source_direction(dir: u8) -> (usize, bool) {
    match dir {
        0 => (0, false),
        1 => (1, false),
        2 => (2, false),
        3 => (3, false),
        4 => (4, false),
        5 => (3, true),
        6 => (2, true),
        7 => (1, true),
        _ => (0, false),
    }
}

// =============================================================================
// SYSTEMS
// =============================================================================

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Load all animation files
    let Some(vstart) = load_vstart() else {
        eprintln!("Failed to load VSTART");
        return;
    };
    let Some(vfra) = load_vfra() else {
        eprintln!("Failed to load VFRA");
        return;
    };
    let Some(vele) = load_vele() else {
        eprintln!("Failed to load VELE");
        return;
    };
    let Some(palette) = load_palette() else {
        eprintln!("Failed to load palette");
        return;
    };
    let Some(sprite_count) = get_sprite_count() else {
        eprintln!("Failed to get sprite count");
        return;
    };

    println!("HSPR has {} sprites", sprite_count);

    commands.insert_resource(AnimationFiles {
        vstart,
        vfra,
        vele,
        palette,
        sprite_count,
    });

    // Start with animation 15 (Brave Idle from lookup table)
    let anim_index = 15;
    commands.insert_resource(CurrentAnimation {
        index: anim_index,
        parsed: None,
        frame_materials: Vec::new(),
        is_static_pose: false,
    });

    // Create placeholder mesh (will be replaced when animation loads)
    let sprite_mesh = meshes.add(Rectangle::new(100.0, 100.0));

    // Create placeholder material
    let placeholder = materials.add(StandardMaterial {
        base_color: Color::srgba(0.5, 0.5, 0.5, 0.5),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Spawn 8 sprites in a circle
    let radius = 200.0;
    for dir in 0..NUM_DIRECTIONS {
        let angle = dir as f32 * std::f32::consts::TAU / NUM_DIRECTIONS as f32;
        let x = angle.sin() * radius;
        let z = angle.cos() * radius;

        commands.spawn((
            Mesh3d(sprite_mesh.clone()),
            MeshMaterial3d(placeholder.clone()),
            Transform::from_xyz(x, 0.0, z),
            AnimatedSprite {
                direction: dir as u8,
                current_frame: 0,
                timer: 0.0,
            },
        ));
    }

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 400.0, 400.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1000.0,
    });

    // UI
    commands.spawn((
        Text::new("VELE Demo\n\nSpace: Pause\nN/P/Tab: Change anim\n+/-: Jump 10\n1-4: Quick anims\nUp/Down: Speed\nQ/E: Rotate"),
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    commands.spawn((
        Text::new("Loading..."),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::srgb(1.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        InfoLabel,
    ));
}

fn load_current_animation(
    files: Res<AnimationFiles>,
    mut current: ResMut<CurrentAnimation>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut label: Query<&mut Text, With<InfoLabel>>,
) {
    // Only load if not already loaded
    if current.parsed.is_some() {
        return;
    }

    println!("Parsing animation {}...", current.index);

    let parsed = parse_animation(
        current.index,
        &files.vstart,
        &files.vfra,
        &files.vele,
        files.sprite_count,
    );

    if let Some(ref anim) = parsed {
        let mut sprite_cache: HashMap<u16, Option<SpriteFrame>> = HashMap::new();
        let mut frame_materials = Vec::new();

        // First pass: find max dimensions for consistent sizing
        let mut max_width = 0i32;
        let mut max_height = 0i32;
        for dir in 0..STORED_DIRECTIONS {
            for frame in &anim.directions[dir] {
                for elem in &frame.elements {
                    let sprite = sprite_cache
                        .entry(elem.sprite_idx)
                        .or_insert_with(|| load_sprite_frame(elem.sprite_idx as usize));
                    if let Some(s) = sprite {
                        let x2 = elem.x_offset as i32 + s.width as i32;
                        let y2 = elem.y_offset as i32 + s.height as i32;
                        max_width = max_width.max(x2);
                        max_height = max_height.max(y2);
                    }
                }
            }
        }

        // Create materials for each direction
        for dir in 0..NUM_DIRECTIONS {
            let (source_dir, _) = get_source_direction(dir as u8);
            let mut dir_materials = Vec::new();

            if source_dir < anim.directions.len() {
                for frame in &anim.directions[source_dir] {
                    if let Some((image, _, _)) = composite_frame(frame, &files.palette, &mut sprite_cache) {
                        let image_handle = images.add(image);
                        let material = materials.add(StandardMaterial {
                            base_color_texture: Some(image_handle),
                            base_color: Color::WHITE,
                            unlit: true,
                            alpha_mode: AlphaMode::Blend,
                            cull_mode: None,
                            ..default()
                        });
                        dir_materials.push(material);
                    }
                }
            }

            frame_materials.push(dir_materials);
        }

        let frame_count = anim.directions[0].len();
        let elem_count: usize = anim.directions[0].iter().map(|f| f.elements.len()).sum();

        // Detect static pose: single frame that loops to itself
        let is_static = frame_count == 1;
        current.is_static_pose = is_static;

        let label_str = if is_static {
            format!("Anim {} | STATIC POSE", current.index)
        } else {
            format!("Anim {} | {} frames | {} elements", current.index, frame_count, elem_count)
        };

        if let Ok(mut text) = label.get_single_mut() {
            *text = Text::new(label_str);
        }

        println!("Loaded animation {}: {} frames in dir 0, {} total elements{}",
                 current.index, frame_count, elem_count,
                 if is_static { " (STATIC POSE)" } else { "" });

        current.frame_materials = frame_materials;
    } else {
        if let Ok(mut text) = label.get_single_mut() {
            *text = Text::new(format!("Anim {} - empty/invalid", current.index));
        }
        println!("Animation {} is empty or invalid", current.index);
        current.frame_materials = Vec::new();
    }

    current.parsed = parsed;
}

fn update_animation(
    time: Res<Time>,
    settings: Res<AnimationSettings>,
    current: Res<CurrentAnimation>,
    mut sprites: Query<(&mut AnimatedSprite, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    if settings.paused || current.frame_materials.is_empty() || current.is_static_pose {
        return;
    }

    for (mut sprite, mut material) in sprites.iter_mut() {
        let dir = sprite.direction as usize;
        if dir >= current.frame_materials.len() || current.frame_materials[dir].is_empty() {
            continue;
        }

        let frame_count = current.frame_materials[dir].len();

        sprite.timer += time.delta_secs();
        if sprite.timer >= settings.speed {
            sprite.timer -= settings.speed;
            sprite.current_frame = (sprite.current_frame + 1) % frame_count;
            material.0 = current.frame_materials[dir][sprite.current_frame].clone();
        }
    }
}

fn update_sprite_facing(
    current: Res<CurrentAnimation>,
    mut sprites: Query<(&AnimatedSprite, &mut Transform, &mut MeshMaterial3d<StandardMaterial>)>,
    camera: Query<&Transform, (With<Camera3d>, Without<AnimatedSprite>)>,
) {
    if current.frame_materials.is_empty() {
        return;
    }

    let Ok(cam) = camera.get_single() else { return };

    for (sprite, mut transform, mut material) in sprites.iter_mut() {
        let (source_dir, mirrored) = get_source_direction(sprite.direction);

        // Billboard
        let dir = cam.translation - transform.translation;
        let yaw = dir.x.atan2(dir.z);
        let hdist = (dir.x.powi(2) + dir.z.powi(2)).sqrt();
        let pitch = (-dir.y).atan2(hdist);
        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);

        // Mirror
        transform.scale.x = if mirrored { -1.0 } else { 1.0 };

        // Update material
        if source_dir < current.frame_materials.len() &&
           !current.frame_materials[source_dir].is_empty() {
            let frame = sprite.current_frame % current.frame_materials[source_dir].len();
            material.0 = current.frame_materials[source_dir][frame].clone();
        }
    }
}

/// Check if an animation is a static pose (single frame, self-loop) without fully parsing it
fn is_static_pose(anim_index: usize, vstart: &[VstartEntry], vfra: &[VfraEntry]) -> bool {
    let base = anim_index * 8;
    if base >= vstart.len() {
        return true;
    }
    let start_vfra = vstart[base].vfra_index as usize;
    if start_vfra >= vfra.len() {
        return true;
    }
    // Static pose = next_frame points back to itself
    vfra[start_vfra].next_frame as usize == start_vfra
}

/// Find the next non-static animation in a given direction
fn find_next_animated(
    from: usize, delta: i32, max_anims: usize,
    vstart: &[VstartEntry], vfra: &[VfraEntry],
) -> usize {
    let mut idx = from;
    for _ in 0..max_anims {
        idx = (idx as i32 + delta).rem_euclid(max_anims as i32) as usize;
        if !is_static_pose(idx, vstart, vfra) {
            return idx;
        }
    }
    from // fallback if everything is static
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<AnimationSettings>,
    mut current: ResMut<CurrentAnimation>,
    files: Res<AnimationFiles>,
    mut sprites: Query<&mut AnimatedSprite>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        settings.paused = !settings.paused;
        println!("Paused: {}", settings.paused);
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) {
        settings.speed = (settings.speed - 0.02).max(0.02);
        println!("Speed: {:.2}s/frame", settings.speed);
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        settings.speed = (settings.speed + 0.02).min(0.5);
        println!("Speed: {:.2}s/frame", settings.speed);
    }

    // Change animation
    let max_anims = files.vstart.len() / 8;
    let mut new_idx: Option<usize> = None;

    // N/P/Tab to cycle (auto-skip static poses)
    if keyboard.just_pressed(KeyCode::KeyN) || keyboard.just_pressed(KeyCode::Tab) {
        new_idx = Some(find_next_animated(current.index, 1, max_anims, &files.vstart, &files.vfra));
    } else if keyboard.just_pressed(KeyCode::KeyP) {
        new_idx = Some(find_next_animated(current.index, -1, max_anims, &files.vstart, &files.vfra));
    }

    // Number keys for quick jump to known animations
    // 1=15 (Brave Idle), 2=20 (Shaman Idle), 3=21 (Brave Walk), 4=26 (Shaman Walk)
    if keyboard.just_pressed(KeyCode::Digit1) {
        new_idx = Some(15);
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        new_idx = Some(20);
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        new_idx = Some(21);
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        new_idx = Some(26);
    }

    // +/- for bigger jumps (also skip static poses)
    if keyboard.just_pressed(KeyCode::Equal) {
        let target = (current.index + 10) % max_anims;
        new_idx = Some(find_next_animated(target.max(1) - 1, 1, max_anims, &files.vstart, &files.vfra));
    }
    if keyboard.just_pressed(KeyCode::Minus) {
        let target = (current.index + max_anims - 10) % max_anims;
        new_idx = Some(find_next_animated(target + 1, -1, max_anims, &files.vstart, &files.vfra));
    }

    if let Some(idx) = new_idx {
        if idx < max_anims && idx != current.index {
            current.index = idx;
            current.parsed = None;  // Trigger reload
            current.frame_materials.clear();
            current.is_static_pose = false;

            for mut sprite in sprites.iter_mut() {
                sprite.current_frame = 0;
                sprite.timer = 0.0;
            }
        }
    }
}

fn rotate_camera(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(mut transform) = camera.get_single_mut() else { return };

    let mut rot = 0.0;
    if keyboard.pressed(KeyCode::KeyQ) { rot += 1.0; }
    if keyboard.pressed(KeyCode::KeyE) { rot -= 1.0; }

    if rot != 0.0 {
        let pos = transform.translation;
        let radius = (pos.x.powi(2) + pos.z.powi(2)).sqrt();
        let angle = pos.x.atan2(pos.z) + rot * time.delta_secs();
        transform.translation.x = angle.sin() * radius;
        transform.translation.z = angle.cos() * radius;
        *transform = transform.looking_at(Vec3::ZERO, Vec3::Y);
    }
}

// =============================================================================
// MAIN
// =============================================================================

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "VELE Compositing Demo".into(),
                resolution: (800.0, 600.0).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .init_resource::<AnimationSettings>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            load_current_animation,
            handle_input,
            rotate_camera,
            update_animation,
            update_sprite_facing,
        ))
        .run();
}
