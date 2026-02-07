# Faithful

Faithful is a project dedicated to the game "Populous: The Beginning" (Pop3/PopTB). It tries to fulfill two goals:
 - To implement a modern renderer for Pop3
 - To implement utilities to convert Pop3 resources to modern formats

The renderer stays as close to the original as practically possible, hence the project name "faithful".

## Building

Faithful is written in Rust. Build with:

```bash
cargo build --release
```

### Dependencies

- [wgpu](https://wgpu.rs/) for cross-platform GPU rendering (Metal/Vulkan/DX12)
- [winit](https://github.com/rust-windowing/winit) for windowing
- [cgmath](https://github.com/rustgd/cgmath) for math
- [clap](https://github.com/clap-rs/clap) for CLI arguments

## Executables

### faithful (main renderer)

A wgpu-based 3D renderer for viewing Populous levels. It reads original game files directly and renders them using modern graphics APIs.

#### Landscape rendering

The landscape is a toroidal 128x128 grid with height mapping. There are 4 texture rendering modes (switchable at runtime):

 - **Full GPU** - texture generation entirely on GPU using original game resources
 - **CPU/GPU hybrid** - palette indices on CPU, colors on GPU
 - **Full CPU** - texture generated on CPU, sent to GPU
 - **Height gradient** - simple height-based coloring (no game resources needed)

Additional landscape features:
 - Toroidal wrapping (seamless world edges)
 - Curvature distortion for a spherical planet effect (toggleable)
 - Water animation
 - Sunlight simulation

#### Sky rendering

Panoramic 512x512 sky background with horizontal scrolling. Each landscape type has its own sky variant, rendered using the game's original palette system.

#### 3D object rendering

Buildings are rendered as full 3D meshes loaded from original game object files:
 - Huts (Small/Medium/Large per tribe)
 - Guard Tower, Boat Hut, Balloon Hut
 - Spy Training, Temple, Firewarrior Training, Warrior Training
 - Vault of Knowledge

Trees are rendered as 3D scenery objects (6 types including fruit tree variants).

Other objects (units, creatures, vehicles) are shown as color-coded markers.

#### Sprite rendering

 - Shaman spawn markers rendered as billboard sprites at reincarnation sites for all 4 tribes
 - 8-direction sprite system with proper mirroring (5 stored directions, 3 mirrored)
 - Sprite atlas system (5 rows x 8 columns for directions x animation frames)
 - Direction computed from camera angle and unit facing using the original game formula

#### Camera

Orbit camera with:
 - Q/E: rotate (yaw)
 - Up/Down arrows: tilt (pitch, -30 to -90 degrees)
 - WASD: pan terrain (screen-relative, toroidal scrolling)
 - Mouse wheel: zoom
 - Space: center on blue tribe's shaman spawn
 - C: toggle curvature, [ / ]: adjust curvature scale

#### Level navigation

 - 25 levels supported
 - B/V: next/previous level
 - N/M: next/previous shader variant
 - O: toggle object markers

#### CLI options

```
--base PATH       Pop3 game directory path
--level N         Start with specific level (1-255)
--landtype TYPE   Override landscape type
--cpu             Enable CPU texture rendering mode
--cpu-full        Enable full CPU rendering
--light X;Y       Configure sunlight parameters
--debug           Enable debug logging
--script PATH     Replay key events from script file
```

### pop_obj_view

Standalone 3D object viewer for inspecting individual game models.

### sprite_viewer

Sprite viewer for browsing sprite atlases and animations.

### sky_viewer

Sky texture viewer for browsing sky variants across landscape types.

### pop_res

2D resource viewer and level renderer. Outputs levels as BMP images. See the `scripts/` directory for usage examples.

## Project structure

```
src/
  main.rs           Main renderer
  landscape.rs      Terrain mesh generation
  view.rs           Camera/MVP matrices
  model.rs          Mesh model traits
  gpu/              wgpu abstraction (context, pipeline, buffer, texture)
  pop/              Populous game format parsers (levels, units, objects, sprites, animations)
  geometry/         Procedural mesh generation (cube, sphere, circle)
  bin/              Additional viewer executables
shaders/            WGSL shaders for all rendering modes
scripts/            Testing and resource extraction tools
docs/               Documentation and reverse engineering notes
```

## Reverse engineering

Reverse engineering notes for the original game binary are available in [docs/RE_NOTES.md](docs/RE_NOTES.md).
