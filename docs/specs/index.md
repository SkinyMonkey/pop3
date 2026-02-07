# Populous: The Beginning - Reverse Engineering Specs

Reverse engineering notes for the original game binary, organized by system.

## Game Systems

- [Core Data Structures](core_data_structures.md) - ModelType enums, subtypes, object instance layout, constants, global variables, memory addresses
- [Object System](object_system.md) - Object lifecycle, state machine, creation/destruction, linked lists, internal objects, scenery
- [Person Units](person_units.md) - Unit system, person state machine, conversion, training, tribes, population
- [Buildings](buildings.md) - Building types, construction, combat, Ghidra analysis, placement during level loading
- [Spells](spells.md) - Spell system, effects, configuration, shield, mana generation
- [Combat and Pathfinding](combat_and_pathfinding.md) - Damage, combat mechanics, pathfinding, kill tracking
- [AI Scripting](ai_scripting.md) - AI bytecode scripting, game tick system, decision trees
- [Terrain](terrain.md) - Terrain mesh, height system, cell flags, render state, boundaries

## Rendering

- [Rendering](rendering.md) - Core rendering pipeline, matrices, projection, rasterizer, lighting, shaders, DirectDraw, Ghidra analysis (R.1-R.60)
- [Sprites and Animation](sprites_and_animation.md) - Sprite format, RLE, loading, rendering, animation frames, UV rotation
- [Textures and Palettes](textures_and_palettes.md) - Data tables, texture loading, palette system, shading LUTs, 3D shape/model format
- [Water and Effects](water_and_effects.md) - Water rendering/animation, particle system, visual effects, swarm effects, selection highlights

## Interface and Infrastructure

- [UI and Input](ui_and_input.md) - Game loop, input/key bindings, camera, minimap, HUD, menus, font rendering, discovery system
- [Audio](audio.md) - Sound system
- [Level, Save and Network](level_save_network.md) - Level loading, save/load, network sync, victory/defeat, game timing
- [Utilities](utilities.md) - File I/O, math helpers, memory management, RNG, flying physics, debug/cheat, vehicles

## Meta

- [RE Session Meta](re_meta.md) - Renamed function lists, coverage summaries, session summaries, statistics
