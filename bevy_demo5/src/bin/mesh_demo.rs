//! 3D Mesh Viewer Demo
//!
//! Loads and displays every 3D mesh from Populous: The Beginning's object files.
//! Format based on hrttf111/faithful project and Ghidra analysis.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::image::{ImageSampler, ImageSamplerDescriptor, ImageFilterMode, ImageAddressMode};
use bevy::window::PresentMode;
use populous_authentic_demo::{GAME_PATH, palette_decrypt};

const XYZ_SCALE: f32 = 1.0 / 300.0;
const UV_SCALE: f32 = 4.768372e-7; // 1/2^21, from faithful

// Atlas layout: 8 columns x 32 rows of 256x256 tiles
const ATLAS_COLS: u32 = 8;
const ATLAS_ROWS: u32 = 32;
const TILE_SIZE: u32 = 256;
const ATLAS_WIDTH: u32 = ATLAS_COLS * TILE_SIZE;  // 2048
const ATLAS_HEIGHT: u32 = ATLAS_ROWS * TILE_SIZE; // 8192

// =============================================================================
// FILE STRUCTURES
// =============================================================================

#[derive(Debug, Clone, Copy)]
struct ObjEntry {
    flags: u16,
    facs_num: u16,
    pnts_num: u16,
    morph_index: u8,
    coord_scale: u32,
    facs_ptr: u32,    // 1-based index into FACS array
    facs_ptr_end: u32,
    pnts_ptr: u32,    // 1-based index into PNTS array
    pnts_ptr_end: u32,
}

#[derive(Debug, Clone, Copy)]
struct Point {
    x: i16,
    y: i16,
    z: i16,
}

#[derive(Debug, Clone, Copy)]
struct Face {
    color_idx: u16,
    tex_index: i16,
    num_points: u8,
    render_flags: u8,
    uvs: [[u32; 2]; 4], // 4 vertices x (u, v)
    indices: [u16; 4],
}

// =============================================================================
// FILE LOADING
// =============================================================================

/// Try to find and read a file with case-insensitive matching
fn read_file_ci(dir: &str, name: &str) -> Option<Vec<u8>> {
    // Try exact name first
    let exact = format!("{}/{}", dir, name);
    if let Ok(data) = std::fs::read(&exact) {
        return Some(data);
    }
    // Try lowercase
    let lower = format!("{}/{}", dir, name.to_lowercase());
    if let Ok(data) = std::fs::read(&lower) {
        return Some(data);
    }
    // Try uppercase
    let upper = format!("{}/{}", dir, name.to_uppercase());
    if let Ok(data) = std::fs::read(&upper) {
        return Some(data);
    }
    // Try title case (first letter upper, rest lower)
    let title = format!("{}/{}{}", dir, &name[..1].to_uppercase(), &name[1..].to_lowercase());
    if let Ok(data) = std::fs::read(&title) {
        return Some(data);
    }
    None
}

fn parse_objs(data: &[u8]) -> Vec<ObjEntry> {
    let count = data.len() / 54;
    let mut entries = Vec::with_capacity(count);
    for i in 0..count {
        let off = i * 54;
        entries.push(ObjEntry {
            flags: u16::from_le_bytes([data[off], data[off + 1]]),
            facs_num: u16::from_le_bytes([data[off + 2], data[off + 3]]),
            pnts_num: u16::from_le_bytes([data[off + 4], data[off + 5]]),
            morph_index: data[off + 7],
            coord_scale: u32::from_le_bytes([data[off + 0x0C], data[off + 0x0D], data[off + 0x0E], data[off + 0x0F]]),
            facs_ptr: u32::from_le_bytes([data[off + 0x10], data[off + 0x11], data[off + 0x12], data[off + 0x13]]),
            facs_ptr_end: u32::from_le_bytes([data[off + 0x14], data[off + 0x15], data[off + 0x16], data[off + 0x17]]),
            pnts_ptr: u32::from_le_bytes([data[off + 0x18], data[off + 0x19], data[off + 0x1a], data[off + 0x1b]]),
            pnts_ptr_end: u32::from_le_bytes([data[off + 0x1c], data[off + 0x1d], data[off + 0x1e], data[off + 0x1f]]),
        });
    }
    entries
}

fn parse_pnts(data: &[u8]) -> Vec<Point> {
    let count = data.len() / 6;
    let mut points = Vec::with_capacity(count);
    for i in 0..count {
        let off = i * 6;
        points.push(Point {
            x: i16::from_le_bytes([data[off], data[off + 1]]),
            y: i16::from_le_bytes([data[off + 2], data[off + 3]]),
            z: i16::from_le_bytes([data[off + 4], data[off + 5]]),
        });
    }
    points
}

fn parse_facs(data: &[u8]) -> Vec<Face> {
    let count = data.len() / 60;
    let mut faces = Vec::with_capacity(count);
    for i in 0..count {
        let off = i * 60;
        let mut uvs = [[0u32; 2]; 4];
        for v in 0..4 {
            let uv_off = off + 8 + v * 8;
            uvs[v][0] = u32::from_le_bytes([data[uv_off], data[uv_off + 1], data[uv_off + 2], data[uv_off + 3]]);
            uvs[v][1] = u32::from_le_bytes([data[uv_off + 4], data[uv_off + 5], data[uv_off + 6], data[uv_off + 7]]);
        }
        faces.push(Face {
            color_idx: u16::from_le_bytes([data[off], data[off + 1]]),
            tex_index: i16::from_le_bytes([data[off + 2], data[off + 3]]),
            num_points: data[off + 6],
            render_flags: data[off + 7],
            uvs,
            indices: [
                u16::from_le_bytes([data[off + 0x28], data[off + 0x29]]),
                u16::from_le_bytes([data[off + 0x2a], data[off + 0x2b]]),
                u16::from_le_bytes([data[off + 0x2c], data[off + 0x2d]]),
                u16::from_le_bytes([data[off + 0x2e], data[off + 0x2f]]),
            ],
        });
    }
    faces
}

/// A single bank of object data (OBJS/PNTS/FACS)
struct Bank {
    points: Vec<Point>,
    faces: Vec<Face>,
}

/// A single bank of object data (OBJS/PNTS/FACS)
struct BankObjs {
    objs: Vec<ObjEntry>,
}

/// Load all 9 banks (0-8) of OBJS/PNTS/FACS data
fn load_all_banks() -> Option<(Vec<BankObjs>, Vec<Bank>)> {
    let objects_dir = format!("{}/objects", GAME_PATH);

    let mut bank_objs = Vec::new();
    let mut banks = Vec::new();

    for bank_id in 0..=8 {
        let objs_name = format!("OBJS0-{}.DAT", bank_id);
        let pnts_name = format!("PNTS0-{}.DAT", bank_id);
        let facs_name = format!("FACS0-{}.DAT", bank_id);

        let objs = read_file_ci(&objects_dir, &objs_name)
            .map(|d| parse_objs(&d))
            .unwrap_or_default();
        let points = read_file_ci(&objects_dir, &pnts_name)
            .map(|d| parse_pnts(&d))
            .unwrap_or_default();
        let faces = read_file_ci(&objects_dir, &facs_name)
            .map(|d| parse_facs(&d))
            .unwrap_or_default();

        println!("Bank {}: {} objs, {} points, {} faces", bank_id, objs.len(), points.len(), faces.len());
        bank_objs.push(BankObjs { objs });
        banks.push(Bank { points, faces });
    }

    Some((bank_objs, banks))
}

fn load_palette() -> Option<Vec<[u8; 4]>> {
    let path = format!("{}/data/pal0-0.dat", GAME_PATH);
    populous_authentic_demo::load_palette_raw(&path)
}

// =============================================================================
// TEXTURE ATLAS
// =============================================================================

/// Build a texture atlas from BL320 files.
/// Atlas is 8 cols x 32 rows of 256x256 tiles = 2048x8192 RGBA.
/// BL320 files are 8bpp paletted, 4 tiles per file (262144 bytes).
fn build_texture_atlas() -> Option<(Vec<u8>, u32, u32)> {
    let atlas_w = ATLAS_WIDTH;
    let atlas_h = ATLAS_HEIGHT;
    let mut atlas = vec![0u8; (atlas_w * atlas_h * 4) as usize];

    // BL320 file suffixes: 0-9, A-Z (36 files, 4 tiles each = 144 tiles)
    let suffixes: Vec<char> = ('0'..='9').chain('A'..='Z').collect();

    let mut tiles_loaded = 0u32;

    for (file_idx, suffix) in suffixes.iter().enumerate() {
        let bl_path = format!("{}/data/BL320-{}.DAT", GAME_PATH, suffix);
        let pal_path = format!("{}/data/pal0-{}.dat", GAME_PATH, suffix);

        let bl_data = match std::fs::read(&bl_path) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let mut pal_data = match std::fs::read(&pal_path) {
            Ok(d) if d.len() >= 1024 => d,
            _ => continue,
        };

        // Decrypt palette using XOR rotating mask (faithful pls::decode)
        palette_decrypt(&mut pal_data);

        // Parse palette (256 colors x 4 bytes: R, G, B, A)
        // For BL320 texture tiles, use full opacity for all colors.
        // Only palette index 0 is typically transparent (used as color key).
        let mut pal = [[0u8; 4]; 256];
        for i in 0..256 {
            let off = i * 4;
            pal[i] = [pal_data[off], pal_data[off + 1], pal_data[off + 2], 255];
        }
        // Index 0 is the transparent color key
        pal[0][3] = 0;

        // Each file has 4 tiles of 256x256
        let tiles_in_file = bl_data.len() / (TILE_SIZE * TILE_SIZE) as usize;
        for tile_in_file in 0..tiles_in_file.min(4) {
            let tile_idx = file_idx * 4 + tile_in_file;
            if tile_idx >= (ATLAS_COLS * ATLAS_ROWS) as usize {
                break;
            }

            let col = (tile_idx as u32) % ATLAS_COLS;
            let row = (tile_idx as u32) / ATLAS_COLS;

            let src_offset = tile_in_file * (TILE_SIZE * TILE_SIZE) as usize;

            for py in 0..TILE_SIZE {
                for px in 0..TILE_SIZE {
                    let src_idx = src_offset + (py * TILE_SIZE + px) as usize;
                    if src_idx >= bl_data.len() {
                        continue;
                    }
                    let pal_idx = bl_data[src_idx] as usize;
                    let color = pal[pal_idx];

                    let atlas_x = col * TILE_SIZE + px;
                    let atlas_y = row * TILE_SIZE + py;
                    let dst_idx = ((atlas_y * atlas_w + atlas_x) * 4) as usize;
                    atlas[dst_idx..dst_idx + 4].copy_from_slice(&color);
                }
            }

            tiles_loaded += 1;
        }
    }

    println!("Atlas: loaded {} tiles from BL320 files ({}x{})", tiles_loaded, atlas_w, atlas_h);

    // Also fill a small solid white region at the very end of the atlas
    // for flat-shaded faces to sample from (we'll use UV = tiny region in last row)
    // Actually, we'll use vertex colors for flat-shaded faces, so no need.

    Some((atlas, atlas_w, atlas_h))
}

// =============================================================================
// MESH BUILDING
// =============================================================================

fn build_mesh_for_object(
    obj: &ObjEntry,
    all_points: &[Point],
    all_faces: &[Face],
    palette: &[[u8; 4]],
    textured: bool,
) -> Option<Mesh> {
    if obj.facs_num == 0 || obj.pnts_num == 0 {
        return None;
    }

    if obj.pnts_ptr == 0 || obj.facs_ptr == 0 {
        return None;
    }
    let pnts_start = (obj.pnts_ptr - 1) as usize;
    let pnts_end = (obj.pnts_ptr_end - 1) as usize;
    let facs_start = (obj.facs_ptr - 1) as usize;
    let facs_end = (obj.facs_ptr_end - 1) as usize;
    let num_local_points = pnts_end - pnts_start;

    if pnts_end > all_points.len() || facs_end > all_faces.len() {
        return None;
    }

    let local_points = &all_points[pnts_start..pnts_end];

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut tex_uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for f_idx in facs_start..facs_end {
        let face = &all_faces[f_idx];
        let n = face.num_points as usize;
        if n < 3 || n > 4 {
            continue;
        }

        let is_textured = textured && face.tex_index >= 0 && (face.tex_index as u32) < ATLAS_COLS * ATLAS_ROWS;

        // Get face color from palette
        let pal_idx = (face.color_idx as usize) % 256;
        let c = palette[pal_idx];
        let color = if is_textured {
            [1.0, 1.0, 1.0, 1.0] // white tint for textured faces
        } else {
            [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0, 1.0]
        };

        let mut verts = Vec::with_capacity(n);
        let mut face_uvs = Vec::with_capacity(n);
        let mut valid = true;

        for vi in 0..n {
            let local_idx = face.indices[vi] as usize;
            if local_idx >= num_local_points {
                valid = false;
                break;
            }
            let p = &local_points[local_idx];
            verts.push([
                p.x as f32 * XYZ_SCALE,
                p.y as f32 * XYZ_SCALE,
                p.z as f32 * XYZ_SCALE,
            ]);

            if is_textured {
                // Convert raw UV to tile-local [0..1]
                let u_local = face.uvs[vi][0] as f32 * UV_SCALE;
                let v_local = face.uvs[vi][1] as f32 * UV_SCALE;

                // Map to atlas coordinates using faithful's shader formula:
                // row = tex_id / 8, col = tex_id % 8
                // atlas_u = (col + uv.x) / 8
                // atlas_v = (row + uv.y) / 32
                let tex_id = face.tex_index as u32;
                let col = tex_id % ATLAS_COLS;
                let row = tex_id / ATLAS_COLS;

                let atlas_u = (col as f32 + u_local) / ATLAS_COLS as f32;
                let atlas_v = (row as f32 + v_local) / ATLAS_ROWS as f32;

                face_uvs.push([atlas_u, atlas_v]);
            } else {
                face_uvs.push([0.0, 0.0]);
            }
        }

        if !valid {
            continue;
        }

        let base = positions.len() as u32;

        for i in 0..n {
            positions.push(verts[i]);
            colors.push(color);
            tex_uvs.push(face_uvs[i]);
        }

        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 2);

        if n == 4 {
            indices.push(base + 2);
            indices.push(base + 3);
            indices.push(base);
        }
    }

    if positions.is_empty() {
        return None;
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, tex_uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh.compute_normals();

    Some(mesh)
}

fn compute_bounds(obj: &ObjEntry, all_points: &[Point]) -> (Vec3, f32) {
    let pnts_start = (obj.pnts_ptr - 1) as usize;
    let pnts_end = (obj.pnts_ptr_end - 1) as usize;

    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);

    for i in pnts_start..pnts_end {
        let p = &all_points[i];
        let v = Vec3::new(
            p.x as f32 * XYZ_SCALE,
            p.y as f32 * XYZ_SCALE,
            p.z as f32 * XYZ_SCALE,
        );
        min = min.min(v);
        max = max.max(v);
    }

    let center = (min + max) * 0.5;
    let extent = (max - min).length();
    (center, extent)
}

// =============================================================================
// BEVY RESOURCES AND COMPONENTS
// =============================================================================

/// Which bank resolved for a given object
#[derive(Debug, Clone)]
struct ResolvedObject {
    obj_idx: usize,
    bank_id: usize,
}

#[derive(Resource)]
struct MeshData {
    bank_objs: Vec<BankObjs>,
    banks: Vec<Bank>,
    palette: Vec<[u8; 4]>,
    valid_objects: Vec<ResolvedObject>,
}

#[derive(Resource)]
struct TextureAtlasHandle(Handle<Image>);

#[derive(Resource)]
struct CurrentObject {
    list_idx: usize,
    needs_reload: bool,
    show_textures: bool,
}

#[derive(Component)]
struct MeshDisplay;

#[derive(Component)]
struct InfoLabel;

// =============================================================================
// SYSTEMS
// =============================================================================

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let Some((bank_objs, banks)) = load_all_banks() else {
        eprintln!("Failed to load object banks");
        return;
    };
    let Some(palette) = load_palette() else {
        eprintln!("Failed to load palette");
        return;
    };

    // Build texture atlas
    let atlas_handle = if let Some((atlas_data, w, h)) = build_texture_atlas() {
        let mut image = Image::new(
            bevy::render::render_resource::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            atlas_data,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        );
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            mag_filter: ImageFilterMode::Nearest,
            min_filter: ImageFilterMode::Nearest,
            mipmap_filter: ImageFilterMode::Nearest,
            address_mode_u: ImageAddressMode::ClampToEdge,
            address_mode_v: ImageAddressMode::ClampToEdge,
            ..default()
        });
        images.add(image)
    } else {
        Handle::default()
    };

    // Resolve each object: each bank has its own OBJS, and its data is in that bank's PNTS/FACS
    let mut valid_objects: Vec<ResolvedObject> = Vec::new();
    for (bank_id, bo) in bank_objs.iter().enumerate() {
        let bank = &banks[bank_id];
        for (obj_idx, o) in bo.objs.iter().enumerate() {
            if o.facs_num == 0 || o.pnts_num == 0 || o.pnts_ptr == 0 || o.facs_ptr == 0 {
                continue;
            }
            let pnts_end = (o.pnts_ptr_end - 1) as usize;
            let facs_end = (o.facs_ptr_end - 1) as usize;

            if pnts_end <= bank.points.len() && facs_end <= bank.faces.len() {
                valid_objects.push(ResolvedObject { obj_idx, bank_id });
            }
        }
    }

    let total_objs: usize = bank_objs.iter().map(|b| b.objs.len()).sum();
    println!(
        "{} valid objects out of {} total (across {} banks)",
        valid_objects.len(),
        total_objs,
        banks.len()
    );

    commands.insert_resource(MeshData {
        bank_objs,
        banks,
        palette,
        valid_objects,
    });

    commands.insert_resource(TextureAtlasHandle(atlas_handle));

    commands.insert_resource(CurrentObject {
        list_idx: 0,
        needs_reload: true,
        show_textures: true,
    });

    commands.spawn((
        Mesh3d(Handle::default()),
        MeshMaterial3d::<StandardMaterial>(Handle::default()),
        Transform::default(),
        MeshDisplay,
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Lighting
    commands.spawn((
        DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
    });

    // UI - Help
    commands.spawn((
        Text::new("Mesh Demo\n\nN/P/Tab: Next/Prev object\n+/-: Jump 10\nQ/E: Rotate\nUp/Down: Zoom\nT: Toggle textures"),
        TextFont {
            font_size: 14.0,
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

    // UI - Info label
    commands.spawn((
        Text::new("Loading..."),
        TextFont {
            font_size: 20.0,
            ..default()
        },
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

fn load_current_object(
    data: Res<MeshData>,
    atlas: Res<TextureAtlasHandle>,
    mut current: ResMut<CurrentObject>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut display: Query<
        (
            &mut Mesh3d,
            &mut MeshMaterial3d<StandardMaterial>,
            &mut Transform,
        ),
        With<MeshDisplay>,
    >,
    mut label: Query<&mut Text, With<InfoLabel>>,
) {
    if !current.needs_reload {
        return;
    }
    current.needs_reload = false;

    if data.valid_objects.is_empty() {
        return;
    }

    let resolved = &data.valid_objects[current.list_idx];
    let obj = &data.bank_objs[resolved.bank_id].objs[resolved.obj_idx];
    let bank = &data.banks[resolved.bank_id];

    if let Some(mesh) = build_mesh_for_object(
        obj,
        &bank.points,
        &bank.faces,
        &data.palette,
        current.show_textures,
    ) {
        let (center, extent) = compute_bounds(obj, &bank.points);
        let scale = if extent > 0.0 { 3.0 / extent } else { 1.0 };

        let mesh_handle = meshes.add(mesh);

        let material_handle = if current.show_textures {
            materials.add(StandardMaterial {
                base_color_texture: Some(atlas.0.clone()),
                base_color: Color::WHITE,
                cull_mode: None,
                alpha_mode: AlphaMode::Mask(0.5),
                ..default()
            })
        } else {
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
                cull_mode: None,
                ..default()
            })
        };

        if let Ok((mut mesh3d, mut mat, mut transform)) = display.get_single_mut() {
            mesh3d.0 = mesh_handle;
            mat.0 = material_handle;
            transform.translation = -center * scale;
            transform.scale = Vec3::splat(scale);
        }

        // Count textured vs flat faces for this object
        let facs_start = (obj.facs_ptr - 1) as usize;
        let facs_end = (obj.facs_ptr_end - 1) as usize;
        let tex_count = (facs_start..facs_end)
            .filter(|&i| bank.faces[i].tex_index >= 0)
            .count();

        let label_text = format!(
            "Obj {}/{} (idx {} bank {}) | {} verts | {} faces ({} tex) | flags=0x{:04x} | {}",
            current.list_idx + 1,
            data.valid_objects.len(),
            resolved.obj_idx,
            resolved.bank_id,
            obj.pnts_num,
            obj.facs_num,
            tex_count,
            obj.flags,
            if current.show_textures { "TEX" } else { "COLOR" },
        );
        if let Ok(mut text) = label.get_single_mut() {
            *text = Text::new(label_text.clone());
        }
        println!("{}", label_text);
    } else {
        if let Ok(mut text) = label.get_single_mut() {
            *text = Text::new(format!(
                "Obj {} bank {} - failed to build mesh",
                resolved.obj_idx, resolved.bank_id
            ));
        }
    }
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut current: ResMut<CurrentObject>,
    data: Res<MeshData>,
) {
    let total = data.valid_objects.len();
    if total == 0 {
        return;
    }

    let mut new_idx: Option<usize> = None;

    if keyboard.just_pressed(KeyCode::KeyN) || keyboard.just_pressed(KeyCode::Tab) {
        new_idx = Some((current.list_idx + 1) % total);
    }
    if keyboard.just_pressed(KeyCode::KeyP) {
        new_idx = Some((current.list_idx + total - 1) % total);
    }
    if keyboard.just_pressed(KeyCode::Equal) {
        new_idx = Some((current.list_idx + 10) % total);
    }
    if keyboard.just_pressed(KeyCode::Minus) {
        new_idx = Some((current.list_idx + total - 10) % total);
    }

    // Toggle textures
    if keyboard.just_pressed(KeyCode::KeyT) {
        current.show_textures = !current.show_textures;
        current.needs_reload = true;
    }

    if let Some(idx) = new_idx {
        current.list_idx = idx;
        current.needs_reload = true;
    }
}

fn orbit_camera(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(mut transform) = camera.get_single_mut() else {
        return;
    };

    let mut rot = 0.0;
    if keyboard.pressed(KeyCode::KeyQ) {
        rot += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyE) {
        rot -= 1.0;
    }

    if rot != 0.0 {
        let pos = transform.translation;
        let radius = (pos.x.powi(2) + pos.z.powi(2)).sqrt();
        let angle = pos.x.atan2(pos.z) + rot * time.delta_secs();
        transform.translation.x = angle.sin() * radius;
        transform.translation.z = angle.cos() * radius;
        *transform = transform.looking_at(Vec3::ZERO, Vec3::Y);
    }

    let mut zoom = 0.0;
    if keyboard.pressed(KeyCode::ArrowUp) {
        zoom -= 2.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        zoom += 2.0;
    }
    if zoom != 0.0 {
        let dir = transform.translation.normalize();
        transform.translation += dir * zoom * time.delta_secs();
        let dist = transform.translation.length();
        if dist < 1.0 {
            transform.translation = dir * 1.0;
        }
        if dist > 50.0 {
            transform.translation = dir * 50.0;
        }
    }
}

fn auto_rotate(
    time: Res<Time>,
    mut display: Query<&mut Transform, With<MeshDisplay>>,
) {
    if let Ok(mut transform) = display.get_single_mut() {
        transform.rotate_y(time.delta_secs() * 0.5);
    }
}

// =============================================================================
// MAIN
// =============================================================================

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Populous Mesh Viewer".into(),
                resolution: (900.0, 700.0).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                load_current_object,
                handle_input,
                orbit_camera,
                auto_rotate,
            ),
        )
        .run();
}
