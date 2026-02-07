//! Sprite Animation Demo
//!
//! A minimal demo to test and debug sprite animations from Populous: The Beginning.
//! Shows all 8 directions in a circle, with keyboard controls to test animation.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::window::PresentMode;
use populous_authentic_demo::animation_data::CHARACTER_ANIMATIONS;

// =============================================================================
// SPRITE CONSTANTS
// =============================================================================

const SPRITE_DATA_PATH: &str = "/Users/adriencandiotti/Library/Containers/com.isaacmarovitz.Whisky/Bottles/74820C9D-5F8C-4BFE-B5DB-90E1DE818D3F/drive_c/GOG Games/Populous - The Beginning/data";
const SPRITE_FILE: &str = "HSPR0-0.DAT";

/// Animation format: 5 stored directions + 3 mirrored
/// Directions 5-7 are mirrored from 3, 2, 1
const SHAMAN_STORED_DIRECTIONS: usize = 5;   // 5 stored directions (0-4)
const SHAMAN_NUM_DIRECTIONS: usize = 8;      // Display 8 directions (5-7 mirrored)
const SHAMAN_ANIM_SPEED: f32 = 0.08;  // Seconds per frame

// =============================================================================
// SPRITE LOADING
// =============================================================================

struct SpriteFrame {
    width: u16,
    height: u16,
    pixels: Vec<Vec<u8>>,
}

fn load_game_palette() -> Option<Vec<[u8; 4]>> {
    let palette_path = format!("{}/pal0-0.dat", SPRITE_DATA_PATH);
    let palette_data = std::fs::read(&palette_path).ok()?;

    if palette_data.len() < 1024 {
        return None;
    }

    let mut palette = Vec::with_capacity(256);
    for i in 0..256 {
        let offset = i * 4;
        palette.push([
            palette_data[offset],
            palette_data[offset + 1],
            palette_data[offset + 2],
            255,
        ]);
    }
    Some(palette)
}

fn load_sprite_frame(frame_index: usize) -> Option<SpriteFrame> {
    let path = format!("{}/{}", SPRITE_DATA_PATH, SPRITE_FILE);
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
        data[header_offset + 4],
        data[header_offset + 5],
        data[header_offset + 6],
        data[header_offset + 7],
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

#[allow(dead_code)]
fn sprite_frame_to_image(frame: &SpriteFrame, palette: &[[u8; 4]]) -> Image {
    let width = frame.width as usize;
    let height = frame.height as usize;
    let mut rgba = Vec::with_capacity(width * height * 4);

    for row in &frame.pixels {
        for &index in row {
            if index == 255 {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let color = palette.get(index as usize).unwrap_or(&[255, 0, 255, 255]);
                rgba.extend_from_slice(&[color[0], color[1], color[2], 255]);
            }
        }
    }

    Image::new(
        bevy::render::render_resource::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        rgba,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// Convert sprite frame to image, padding to target dimensions (centered)
fn sprite_frame_to_image_padded(frame: &SpriteFrame, palette: &[[u8; 4]], target_w: usize, target_h: usize) -> Image {
    let width = frame.width as usize;
    let height = frame.height as usize;

    // Calculate centering offsets
    let offset_x = (target_w - width) / 2;
    let offset_y = (target_h - height) / 2;

    let mut rgba = vec![0u8; target_w * target_h * 4];  // All transparent

    for (y, row) in frame.pixels.iter().enumerate() {
        for (x, &index) in row.iter().enumerate() {
            let target_x = offset_x + x;
            let target_y = offset_y + y;
            let pixel_offset = (target_y * target_w + target_x) * 4;

            if index == 255 {
                // Already transparent (0, 0, 0, 0)
            } else {
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
// COMPONENTS AND RESOURCES
// =============================================================================

#[derive(Component)]
struct AnimatedSprite {
    direction: u8,
    current_frame: u8,
    anim_timer: f32,
}

/// Returns (source_direction, is_mirrored) for a given display direction
fn get_source_direction(dir: u8) -> (usize, bool) {
    match dir {
        0 => (0, false),  // South - no mirror
        1 => (1, false),  // SW - no mirror
        2 => (2, false),  // West - no mirror
        3 => (3, false),  // NW - no mirror
        4 => (4, false),  // North - no mirror
        5 => (3, true),   // NE - mirror of NW (dir 3)
        6 => (2, true),   // East - mirror of West (dir 2)
        7 => (1, true),   // SE - mirror of SW (dir 1)
        _ => (0, false),
    }
}

#[derive(Component)]
struct DirectionLabel {
    #[allow(dead_code)]
    direction: u8,
    world_pos: Vec3,
}

#[derive(Component)]
struct CharacterLabel;


#[derive(Resource)]
struct ShamanAnimFrames {
    frames: Vec<Vec<Handle<StandardMaterial>>>,
    frame_width: f32,
    frame_height: f32,
    frames_per_dir: usize,
}

/// Cached palette for reloading
#[derive(Resource)]
struct CachedPalette(Vec<[u8; 4]>);

/// Helper function to get animation data as a tuple for compatibility
fn get_character_anim(index: usize) -> (usize, &'static str, usize) {
    let anim = &CHARACTER_ANIMATIONS[index];
    (anim.sprite_start as usize, anim.name, anim.frames_per_dir as usize)
}

/// Get the number of available character animations
fn character_anim_count() -> usize {
    CHARACTER_ANIMATIONS.len()
}

#[derive(Resource)]
struct AnimationSettings {
    speed: f32,
    paused: bool,
    current_character: usize,  // Index into CHARACTER_ANIMATIONS
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            speed: SHAMAN_ANIM_SPEED,
            paused: false,
            current_character: 0,
        }
    }
}

// =============================================================================
// SYSTEMS
// =============================================================================

/// Load character animation frames starting at given frame index
fn load_character_frames(
    anim_start: usize,
    frames_per_dir: usize,
    palette: &[[u8; 4]],
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) -> Option<(Vec<Vec<Handle<StandardMaterial>>>, f32, f32, usize)> {
    // First pass: find max dimensions across all frames for this character
    let mut max_width: u16 = 0;
    let mut max_height: u16 = 0;

    for dir in 0..SHAMAN_STORED_DIRECTIONS {
        for frame_idx in 0..frames_per_dir {
            let global_frame = anim_start + dir * frames_per_dir + frame_idx;
            if let Some(frame) = load_sprite_frame(global_frame) {
                max_width = max_width.max(frame.width);
                max_height = max_height.max(frame.height);
            }
        }
    }

    if max_width == 0 || max_height == 0 {
        return None;
    }

    // Load the 5 stored directions
    let mut stored_frames: Vec<Vec<Handle<Image>>> = Vec::new();

    for dir in 0..SHAMAN_STORED_DIRECTIONS {
        let mut dir_frames: Vec<Handle<Image>> = Vec::new();

        for frame_idx in 0..frames_per_dir {
            let global_frame = anim_start + dir * frames_per_dir + frame_idx;
            let frame = load_sprite_frame(global_frame)?;

            let image = sprite_frame_to_image_padded(&frame, palette, max_width as usize, max_height as usize);
            let image_handle = images.add(image);
            dir_frames.push(image_handle);
        }

        stored_frames.push(dir_frames);
    }

    // Build all 8 directions (mirroring 5-7 from 3-1)
    let mut all_frames: Vec<Vec<Handle<StandardMaterial>>> = Vec::new();

    for dir in 0..SHAMAN_NUM_DIRECTIONS {
        let mut dir_frames: Vec<Handle<StandardMaterial>> = Vec::new();
        let (source_dir, _mirror) = get_source_direction(dir as u8);

        for frame_idx in 0..frames_per_dir {
            let image_handle = stored_frames[source_dir][frame_idx].clone();

            let material = materials.add(StandardMaterial {
                base_color_texture: Some(image_handle),
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

    Some((all_frames, max_width as f32, max_height as f32, frames_per_dir))
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

    // Cache palette for character switching
    commands.insert_resource(CachedPalette(palette.clone()));

    // Load initial character (shaman)
    let (anim_start, char_name, frames_per_dir) = get_character_anim(0);
    let Some((all_frames, frame_width, frame_height, fpd)) = load_character_frames(
        anim_start, frames_per_dir, &palette, &mut images, &mut materials
    ) else {
        eprintln!("Failed to load character frames");
        return;
    };

    println!("Loaded {} - {}x{} ({} frames/dir)", char_name, frame_width, frame_height, fpd);

    // Store animation frames resource
    commands.insert_resource(ShamanAnimFrames {
        frames: all_frames.clone(),
        frame_width,
        frame_height,
        frames_per_dir: fpd,
    });

    // Create sprite mesh
    let scale = 4.0;
    let sprite_mesh = meshes.add(Rectangle::new(frame_width * scale, frame_height * scale));

    // Spawn 8 sprites in a circle (5 stored + 3 mirrored)
    let radius = 200.0;
    for dir in 0..SHAMAN_NUM_DIRECTIONS {
        let angle = dir as f32 * std::f32::consts::TAU / SHAMAN_NUM_DIRECTIONS as f32;
        let x = angle.sin() * radius;
        let z = angle.cos() * radius;

        // Spawn sprite
        commands.spawn((
            Mesh3d(sprite_mesh.clone()),
            MeshMaterial3d(all_frames[dir][0].clone()),
            Transform::from_xyz(x, 0.0, z),
            AnimatedSprite {
                direction: dir as u8,
                current_frame: 0,
                anim_timer: 0.0,
            },
        ));

        // Spawn 2D UI label for this direction
        commands.spawn((
            Text::new(format!("Dir {}", dir)),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 0.0)),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            DirectionLabel {
                direction: dir as u8,
                world_pos: Vec3::new(x, 0.0, z),
            },
        ));
    }

    // Camera - looking down at the circle
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 400.0, 400.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Lighting
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1000.0,
    });

    // Info text
    commands.spawn((
        Text::new("Sprite Animation Demo\n\nSpace: Pause/Resume\nUp/Down: Animation speed\nLeft/Right: Frame step (paused)\nTab/N/P: Switch character\nQ/E: Rotate camera"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    // Character label (top center)
    let (_, char_name, _) = get_character_anim(0);
    commands.spawn((
        Text::new(format!("{} (frame {})", char_name, anim_start)),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Percent(50.0),
            ..default()
        },
        CharacterLabel,
    ));
}

fn update_animation(
    time: Res<Time>,
    settings: Res<AnimationSettings>,
    anim_frames: Res<ShamanAnimFrames>,
    mut sprites: Query<(&mut AnimatedSprite, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    if settings.paused {
        return;
    }

    let frames_per_dir = anim_frames.frames_per_dir;

    for (mut sprite, mut material) in sprites.iter_mut() {
        sprite.anim_timer += time.delta_secs();

        if sprite.anim_timer >= settings.speed {
            sprite.anim_timer -= settings.speed;
            sprite.current_frame = (sprite.current_frame + 1) % frames_per_dir as u8;

            let dir = sprite.direction as usize;
            let frame = sprite.current_frame as usize;
            material.0 = anim_frames.frames[dir][frame].clone();
        }
    }
}

fn update_sprite_facing(
    anim_frames: Res<ShamanAnimFrames>,
    mut sprites: Query<(&AnimatedSprite, &mut Transform, &mut MeshMaterial3d<StandardMaterial>)>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<AnimatedSprite>)>,
) {
    let Ok(cam_transform) = camera_query.get_single() else { return };

    for (sprite, mut transform, mut material) in sprites.iter_mut() {
        // Get source direction and mirror flag
        let (source_dir, mirrored) = get_source_direction(sprite.direction);

        // Full billboard - face camera in all axes (not just Y rotation)
        // This prevents foreshortening when camera looks down at angle
        let direction = cam_transform.translation - transform.translation;
        let yaw = direction.x.atan2(direction.z);
        let horizontal_dist = (direction.x.powi(2) + direction.z.powi(2)).sqrt();
        let pitch = (-direction.y).atan2(horizontal_dist);
        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);

        // Apply mirror by flipping X scale
        let x_scale = if mirrored { -1.0 } else { 1.0 };
        transform.scale = Vec3::new(x_scale, 1.0, 1.0);

        // Update material for source direction and frame
        let frame = sprite.current_frame as usize % anim_frames.frames_per_dir;
        material.0 = anim_frames.frames[source_dir][frame].clone();
    }
}

fn update_direction_labels(
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut labels: Query<(&DirectionLabel, &mut Node)>,
) {
    let Ok((camera, camera_transform)) = camera_query.get_single() else { return };

    for (label, mut node) in labels.iter_mut() {
        // Project world position to screen
        if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, label.world_pos) {
            node.left = Val::Px(screen_pos.x - 20.0);
            node.top = Val::Px(screen_pos.y + 60.0);  // Below the sprite
        }
    }
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<AnimationSettings>,
    anim_frames: Res<ShamanAnimFrames>,
    mut sprites: Query<(&mut AnimatedSprite, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    // Pause/resume
    if keyboard.just_pressed(KeyCode::Space) {
        settings.paused = !settings.paused;
        println!("Animation {}", if settings.paused { "paused" } else { "running" });
    }

    // Speed controls
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        settings.speed = (settings.speed - 0.02).max(0.02);
        println!("Animation speed: {:.2}s per frame", settings.speed);
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        settings.speed = (settings.speed + 0.02).min(0.5);
        println!("Animation speed: {:.2}s per frame", settings.speed);
    }

    let frames_per_dir = anim_frames.frames_per_dir;

    // Manual frame step when paused
    if settings.paused {
        if keyboard.just_pressed(KeyCode::ArrowRight) {
            for (mut sprite, mut material) in sprites.iter_mut() {
                sprite.current_frame = (sprite.current_frame + 1) % frames_per_dir as u8;
                let dir = sprite.direction as usize;
                let frame = sprite.current_frame as usize;
                material.0 = anim_frames.frames[dir][frame].clone();
            }
            println!("Frame step forward");
        }
        if keyboard.just_pressed(KeyCode::ArrowLeft) {
            for (mut sprite, mut material) in sprites.iter_mut() {
                sprite.current_frame = if sprite.current_frame == 0 {
                    frames_per_dir as u8 - 1
                } else {
                    sprite.current_frame - 1
                };
                let dir = sprite.direction as usize;
                let frame = sprite.current_frame as usize;
                material.0 = anim_frames.frames[dir][frame].clone();
            }
            println!("Frame step backward");
        }
    }
}

fn handle_character_switch(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<AnimationSettings>,
    mut anim_frames: ResMut<ShamanAnimFrames>,
    palette: Res<CachedPalette>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sprites: Query<(&mut AnimatedSprite, &mut MeshMaterial3d<StandardMaterial>)>,
    mut label_query: Query<&mut Text, With<CharacterLabel>>,
) {
    // Tab or N/P to switch character
    let switch = if keyboard.just_pressed(KeyCode::Tab) || keyboard.just_pressed(KeyCode::KeyN) {
        Some(1i32)  // Next
    } else if keyboard.just_pressed(KeyCode::KeyP) {
        Some(-1i32)  // Previous
    } else {
        None
    };

    if let Some(delta) = switch {
        let new_idx = (settings.current_character as i32 + delta)
            .rem_euclid(character_anim_count() as i32) as usize;

        let (anim_start, char_name, frames_per_dir) = get_character_anim(new_idx);

        if let Some((frames, width, height, fpd)) = load_character_frames(
            anim_start, frames_per_dir, &palette.0, &mut images, &mut materials
        ) {
            anim_frames.frames = frames;
            anim_frames.frame_width = width;
            anim_frames.frame_height = height;
            anim_frames.frames_per_dir = fpd;
            settings.current_character = new_idx;

            // Reset all sprites to frame 0 with new materials
            for (mut sprite, mut material) in sprites.iter_mut() {
                sprite.current_frame = 0;
                let (source_dir, _) = get_source_direction(sprite.direction);
                material.0 = anim_frames.frames[source_dir][0].clone();
            }

            // Update on-screen label
            if let Ok(mut text) = label_query.get_single_mut() {
                *text = Text::new(format!("{} (frame {}, {} fpd)", char_name, anim_start, fpd));
            }

            println!("Switched to: {} (frame {}, {} frames/dir)", char_name, anim_start, fpd);
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
        // Rotate camera around Y axis while looking at center
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
                title: "Sprite Animation Demo".into(),
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
            handle_character_switch,
            rotate_camera,
            update_animation,
            update_sprite_facing,
            update_direction_labels,
        ))
        .run();
}
