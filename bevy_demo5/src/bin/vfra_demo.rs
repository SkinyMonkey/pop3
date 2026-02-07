//! VFRA Animation Demo
//!
//! Tests loading animations via the VSTART → VFRA chain instead of hardcoded offsets.
//! This demo uses the actual animation files to discover unit animations.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::window::PresentMode;
use populous_authentic_demo::{GAME_PATH, SpriteFrame};

// =============================================================================
// PATHS AND CONSTANTS
// =============================================================================

fn data_path() -> String {
    format!("{}/data", GAME_PATH)
}

const NUM_DIRECTIONS: usize = 8;
const STORED_DIRECTIONS: usize = 5;
const ANIM_SPEED: f32 = 0.08;

// =============================================================================
// ANIMATION FILE STRUCTURES
// =============================================================================

/// VSTART entry (4 bytes) - verified from Ghidra decompilation
#[derive(Debug, Clone, Copy)]
struct VstartEntry {
    vfra_index: u16,   // Index into VFRA linked list
    mirror_ref: u16,   // Mirror reference for directions 5-7
}

/// VFRA entry (8 bytes) - verified from Ghidra decompilation
#[derive(Debug, Clone, Copy)]
struct VfraEntry {
    first_element: u16,  // Sprite frame index or VELE element
    width: u8,
    height: u8,
    flags: u16,          // 0x0100 = animation start marker
    next_frame: u16,     // Next VFRA index (linked list)
}

/// Parsed animation from VSTART/VFRA chain
#[derive(Debug, Clone)]
struct ParsedAnimation {
    anim_index: usize,
    name: String,
    directions: Vec<Vec<u16>>,  // [direction][frame] -> sprite index
}

// =============================================================================
// FILE LOADING
// =============================================================================

fn load_vstart() -> Option<Vec<VstartEntry>> {
    let path = format!("{}/VSTART-0.ANI", data_path());
    let data = std::fs::read(&path).ok()?;

    let entry_count = data.len() / 4;
    let mut entries = Vec::with_capacity(entry_count);

    for i in 0..entry_count {
        let offset = i * 4;
        let vfra_index = u16::from_le_bytes([data[offset], data[offset + 1]]);
        let mirror_ref = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
        entries.push(VstartEntry { vfra_index, mirror_ref });
    }

    println!("Loaded VSTART: {} entries ({} animations)", entry_count, entry_count / 8);
    Some(entries)
}

fn load_vfra() -> Option<Vec<VfraEntry>> {
    let path = format!("{}/VFRA-0.ANI", data_path());
    let data = std::fs::read(&path).ok()?;

    let entry_count = data.len() / 8;
    let mut entries = Vec::with_capacity(entry_count);

    for i in 0..entry_count {
        let offset = i * 8;
        entries.push(VfraEntry {
            first_element: u16::from_le_bytes([data[offset], data[offset + 1]]),
            width: data[offset + 2],
            height: data[offset + 3],
            flags: u16::from_le_bytes([data[offset + 4], data[offset + 5]]),
            next_frame: u16::from_le_bytes([data[offset + 6], data[offset + 7]]),
        });
    }

    println!("Loaded VFRA: {} entries", entry_count);
    Some(entries)
}

/// Follow VFRA linked list to get all frames for a direction
fn follow_vfra_chain(vfra: &[VfraEntry], start_index: u16, max_frames: usize) -> Vec<u16> {
    let mut frames = Vec::new();
    let mut current = start_index as usize;
    let mut visited = std::collections::HashSet::new();

    while current < vfra.len() && frames.len() < max_frames {
        if visited.contains(&current) {
            break; // Loop detected
        }
        visited.insert(current);

        let entry = &vfra[current];
        frames.push(entry.first_element);

        // Follow to next frame
        let next = entry.next_frame as usize;
        if next == start_index as usize {
            break; // Complete loop
        }
        current = next;
    }

    frames
}

/// Parse animation from VSTART index
fn parse_animation(anim_index: usize, vstart: &[VstartEntry], vfra: &[VfraEntry]) -> Option<ParsedAnimation> {
    let base = anim_index * 8;
    if base + 8 > vstart.len() {
        return None;
    }

    let mut directions = Vec::new();

    // Parse all 8 directions
    for dir in 0..NUM_DIRECTIONS {
        let entry = &vstart[base + dir];
        let frames = follow_vfra_chain(vfra, entry.vfra_index, 32);
        directions.push(frames);
    }

    // Skip empty animations
    if directions[0].is_empty() {
        return None;
    }

    Some(ParsedAnimation {
        anim_index,
        name: format!("Animation {}", anim_index),
        directions,
    })
}

/// Get sprite count from HSPR file
fn get_sprite_count() -> Option<usize> {
    let path = format!("{}/HSPR0-0.DAT", data_path());
    let data = std::fs::read(&path).ok()?;
    populous_authentic_demo::psfb_sprite_count(&data)
}

// =============================================================================
// SPRITE LOADING (uses shared lib)
// =============================================================================

fn load_game_palette() -> Option<Vec<[u8; 4]>> {
    let path = format!("{}/pal0-0.dat", data_path());
    populous_authentic_demo::load_palette_raw(&path)
}

fn load_sprite_frame(frame_index: usize) -> Option<SpriteFrame> {
    let path = format!("{}/HSPR0-0.DAT", data_path());
    let data = std::fs::read(&path).ok()?;
    populous_authentic_demo::psfb_load_frame(&data, frame_index)
}

fn sprite_frame_to_image_padded(frame: &SpriteFrame, palette: &[[u8; 4]], target_w: usize, target_h: usize) -> Image {
    let width = frame.width as usize;
    let height = frame.height as usize;

    let offset_x = (target_w - width) / 2;
    let offset_y = (target_h - height) / 2;

    let mut rgba = vec![0u8; target_w * target_h * 4];

    for (y, row) in frame.pixels.iter().enumerate() {
        for (x, &index) in row.iter().enumerate() {
            let target_x = offset_x + x;
            let target_y = offset_y + y;
            let pixel_offset = (target_y * target_w + target_x) * 4;

            if index != 255 {
                let color = palette.get(index as usize).unwrap_or(&[255, 0, 255, 255]);
                rgba[pixel_offset] = color[0];
                rgba[pixel_offset + 1] = color[1];
                rgba[pixel_offset + 2] = color[2];
                rgba[pixel_offset + 3] = 255;
            }
        }
    }

    Image::new(
        bevy::render::render_resource::Extent3d {
            width: target_w as u32,
            height: target_h as u32,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        rgba,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

// =============================================================================
// BEVY COMPONENTS AND RESOURCES
// =============================================================================

#[derive(Component)]
struct AnimatedSprite {
    direction: u8,
    current_frame: u8,
    anim_timer: f32,
}

#[derive(Component)]
struct DirectionLabel {
    world_pos: Vec3,
}

#[derive(Component)]
struct InfoLabel;

#[derive(Resource)]
struct AnimationData {
    animations: Vec<ParsedAnimation>,
    current_anim: usize,
    sprite_count: usize,
}

#[derive(Resource)]
struct LoadedFrames {
    frames: Vec<Vec<Handle<StandardMaterial>>>,  // [direction][frame]
    frame_width: f32,
    frame_height: f32,
}

#[derive(Resource)]
struct CachedPalette(Vec<[u8; 4]>);

#[derive(Resource)]
struct AnimationSettings {
    speed: f32,
    paused: bool,
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            speed: ANIM_SPEED,
            paused: false,
        }
    }
}

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

fn load_animation_frames(
    anim: &ParsedAnimation,
    palette: &[[u8; 4]],
    sprite_count: usize,
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) -> Option<(Vec<Vec<Handle<StandardMaterial>>>, f32, f32)> {
    // Find max dimensions across all frames
    let mut max_width: u16 = 0;
    let mut max_height: u16 = 0;

    // Only check stored directions (0-4)
    for dir in 0..STORED_DIRECTIONS {
        for &sprite_idx in &anim.directions[dir] {
            if (sprite_idx as usize) < sprite_count {
                if let Some(frame) = load_sprite_frame(sprite_idx as usize) {
                    max_width = max_width.max(frame.width);
                    max_height = max_height.max(frame.height);
                }
            }
        }
    }

    if max_width == 0 || max_height == 0 {
        return None;
    }

    // Load stored directions
    let mut stored_frames: Vec<Vec<Handle<Image>>> = Vec::new();

    for dir in 0..STORED_DIRECTIONS {
        let mut dir_frames: Vec<Handle<Image>> = Vec::new();

        for &sprite_idx in &anim.directions[dir] {
            if (sprite_idx as usize) >= sprite_count {
                // Skip VELE references for now
                continue;
            }

            if let Some(frame) = load_sprite_frame(sprite_idx as usize) {
                let image = sprite_frame_to_image_padded(&frame, palette, max_width as usize, max_height as usize);
                dir_frames.push(images.add(image));
            }
        }

        stored_frames.push(dir_frames);
    }

    // Check we have frames
    if stored_frames[0].is_empty() {
        return None;
    }

    // Build all 8 directions
    let mut all_frames: Vec<Vec<Handle<StandardMaterial>>> = Vec::new();

    for dir in 0..NUM_DIRECTIONS {
        let (source_dir, _mirror) = get_source_direction(dir as u8);
        let mut dir_frames = Vec::new();

        for image_handle in &stored_frames[source_dir] {
            let material = materials.add(StandardMaterial {
                base_color_texture: Some(image_handle.clone()),
                base_color: Color::WHITE,
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                ..default()
            });
            dir_frames.push(material);
        }

        all_frames.push(dir_frames);
    }

    Some((all_frames, max_width as f32, max_height as f32))
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Load palette
    let Some(palette) = load_game_palette() else {
        eprintln!("Failed to load palette");
        return;
    };
    commands.insert_resource(CachedPalette(palette.clone()));

    // Load animation files
    let Some(vstart) = load_vstart() else {
        eprintln!("Failed to load VSTART");
        return;
    };
    let Some(vfra) = load_vfra() else {
        eprintln!("Failed to load VFRA");
        return;
    };
    let Some(sprite_count) = get_sprite_count() else {
        eprintln!("Failed to get sprite count");
        return;
    };
    println!("HSPR0-0.DAT has {} sprites", sprite_count);

    // Parse all animations
    let num_anims = vstart.len() / 8;
    let mut animations = Vec::new();

    for i in 0..num_anims {
        if let Some(anim) = parse_animation(i, &vstart, &vfra) {
            // Check if first frame is within sprite bank
            if !anim.directions[0].is_empty() && (anim.directions[0][0] as usize) < sprite_count {
                animations.push(anim);
            }
        }
    }

    println!("Found {} animations with valid HSPR sprites", animations.len());

    if animations.is_empty() {
        eprintln!("No valid animations found");
        return;
    }

    // Load first animation
    let Some((frames, frame_width, frame_height)) = load_animation_frames(
        &animations[0], &palette, sprite_count, &mut images, &mut materials
    ) else {
        eprintln!("Failed to load animation frames");
        return;
    };

    let frames_per_dir = frames[0].len();
    println!("Loaded animation 0: {}x{}, {} frames/dir", frame_width, frame_height, frames_per_dir);

    // Get first sprite for label
    let first_sprite = animations[0].directions[0][0];

    commands.insert_resource(AnimationData {
        animations,
        current_anim: 0,
        sprite_count,
    });

    commands.insert_resource(LoadedFrames {
        frames: frames.clone(),
        frame_width,
        frame_height,
    });

    // Create sprite mesh
    let scale = 4.0;
    let sprite_mesh = meshes.add(Rectangle::new(frame_width * scale, frame_height * scale));

    // Spawn 8 sprites in a circle
    let radius = 200.0;
    for dir in 0..NUM_DIRECTIONS {
        let angle = dir as f32 * std::f32::consts::TAU / NUM_DIRECTIONS as f32;
        let x = angle.sin() * radius;
        let z = angle.cos() * radius;

        let (source_dir, _) = get_source_direction(dir as u8);

        commands.spawn((
            Mesh3d(sprite_mesh.clone()),
            MeshMaterial3d(frames[source_dir][0].clone()),
            Transform::from_xyz(x, 0.0, z),
            AnimatedSprite {
                direction: dir as u8,
                current_frame: 0,
                anim_timer: 0.0,
            },
        ));

        commands.spawn((
            Text::new(format!("Dir {}", dir)),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::srgb(1.0, 1.0, 0.0)),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            DirectionLabel {
                world_pos: Vec3::new(x, 0.0, z),
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

    // Info text
    commands.spawn((
        Text::new("VFRA Animation Demo\n\nSpace: Pause/Resume\nTab/N/P: Switch animation\nQ/E: Rotate camera"),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    // Animation label
    commands.spawn((
        Text::new(format!("Animation 0 (frame {})", first_sprite)),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::srgb(1.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Percent(50.0),
            ..default()
        },
        InfoLabel,
    ));
}

fn update_animation(
    time: Res<Time>,
    settings: Res<AnimationSettings>,
    loaded: Res<LoadedFrames>,
    mut sprites: Query<(&mut AnimatedSprite, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    if settings.paused || loaded.frames.is_empty() {
        return;
    }

    for (mut sprite, mut material) in sprites.iter_mut() {
        let (source_dir, _) = get_source_direction(sprite.direction);

        // Skip if this direction has no frames
        if loaded.frames[source_dir].is_empty() {
            continue;
        }

        let frames_per_dir = loaded.frames[source_dir].len();

        sprite.anim_timer += time.delta_secs();

        if sprite.anim_timer >= settings.speed {
            sprite.anim_timer -= settings.speed;
            sprite.current_frame = (sprite.current_frame + 1) % frames_per_dir as u8;

            let frame = sprite.current_frame as usize;
            material.0 = loaded.frames[source_dir][frame].clone();
        }
    }
}

fn update_sprite_facing(
    loaded: Res<LoadedFrames>,
    mut sprites: Query<(&AnimatedSprite, &mut Transform, &mut MeshMaterial3d<StandardMaterial>)>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<AnimatedSprite>)>,
) {
    if loaded.frames.is_empty() || loaded.frames[0].is_empty() {
        return;
    }

    let Ok(cam_transform) = camera_query.get_single() else { return };

    for (sprite, mut transform, mut material) in sprites.iter_mut() {
        let (source_dir, mirrored) = get_source_direction(sprite.direction);

        let direction = cam_transform.translation - transform.translation;
        let yaw = direction.x.atan2(direction.z);
        let horizontal_dist = (direction.x.powi(2) + direction.z.powi(2)).sqrt();
        let pitch = (-direction.y).atan2(horizontal_dist);
        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);

        let x_scale = if mirrored { -1.0 } else { 1.0 };
        transform.scale = Vec3::new(x_scale, 1.0, 1.0);

        // Guard against empty frame lists
        if !loaded.frames[source_dir].is_empty() {
            let frame = sprite.current_frame as usize % loaded.frames[source_dir].len();
            material.0 = loaded.frames[source_dir][frame].clone();
        }
    }
}

fn update_direction_labels(
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut labels: Query<(&DirectionLabel, &mut Node)>,
) {
    let Ok((camera, camera_transform)) = camera_query.get_single() else { return };

    for (label, mut node) in labels.iter_mut() {
        if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, label.world_pos) {
            node.left = Val::Px(screen_pos.x - 20.0);
            node.top = Val::Px(screen_pos.y + 60.0);
        }
    }
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<AnimationSettings>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        settings.paused = !settings.paused;
        println!("Animation {}", if settings.paused { "paused" } else { "running" });
    }
}

fn handle_animation_switch(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut anim_data: ResMut<AnimationData>,
    mut loaded: ResMut<LoadedFrames>,
    palette: Res<CachedPalette>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sprites: Query<(&mut AnimatedSprite, &mut MeshMaterial3d<StandardMaterial>)>,
    mut label_query: Query<&mut Text, With<InfoLabel>>,
) {
    let switch = if keyboard.just_pressed(KeyCode::Tab) || keyboard.just_pressed(KeyCode::KeyN) {
        Some(1i32)
    } else if keyboard.just_pressed(KeyCode::KeyP) {
        Some(-1i32)
    } else {
        None
    };

    if let Some(delta) = switch {
        let new_idx = (anim_data.current_anim as i32 + delta)
            .rem_euclid(anim_data.animations.len() as i32) as usize;

        // Clone what we need from anim before mutating
        let anim = anim_data.animations[new_idx].clone();
        let first_sprite = anim.directions[0].first().copied().unwrap_or(0);

        if let Some((frames, width, height)) = load_animation_frames(
            &anim, &palette.0, anim_data.sprite_count, &mut images, &mut materials
        ) {
            loaded.frames = frames;
            loaded.frame_width = width;
            loaded.frame_height = height;
            anim_data.current_anim = new_idx;

            for (mut sprite, mut material) in sprites.iter_mut() {
                sprite.current_frame = 0;
                let (source_dir, _) = get_source_direction(sprite.direction);
                if !loaded.frames[source_dir].is_empty() {
                    material.0 = loaded.frames[source_dir][0].clone();
                }
            }

            if let Ok(mut text) = label_query.get_single_mut() {
                *text = Text::new(format!(
                    "Animation {} (sprite {}, {} frames)",
                    new_idx, first_sprite, loaded.frames[0].len()
                ));
            }

            println!("Switched to animation {} (sprite {})", new_idx, first_sprite);
        }
    }
}

fn rotate_camera(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(mut transform) = camera_query.get_single_mut() else { return };

    let mut rotation = 0.0;
    if keyboard.pressed(KeyCode::KeyQ) {
        rotation += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyE) {
        rotation -= 1.0;
    }

    if rotation != 0.0 {
        let current_pos = transform.translation;
        let radius = (current_pos.x * current_pos.x + current_pos.z * current_pos.z).sqrt();
        let current_angle = current_pos.x.atan2(current_pos.z);
        let new_angle = current_angle + rotation * time.delta_secs();

        transform.translation.x = new_angle.sin() * radius;
        transform.translation.z = new_angle.cos() * radius;
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
                title: "VFRA Animation Demo".into(),
                resolution: (800.0, 600.0).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .init_resource::<AnimationSettings>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_input,
            handle_animation_switch,
            rotate_camera,
            update_animation,
            update_sprite_facing,
            update_direction_labels,
        ))
        .run();
}
