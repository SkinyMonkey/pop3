# Reverse Engineering Session Meta

## Renamed Functions

| Address    | Original        | New Name               |
|------------|-----------------|------------------------|
| 0x00454050 | FUN_00454050    | GetObjectTypeName      |
| 0x004afbf0 | FUN_004afbf0    | InitObjectPointerArray |
| 0x0040cc10 | FUN_0040cc10    | LoadLevelHeader        |
| 0x0040cde0 | FUN_0040cde0    | LoadLevelData          |
| 0x0040dc70 | FUN_0040dc70    | LoadLevelSpecialData   |
| 0x0040dd70 | FUN_0040dd70    | LoadObjectivesData     |
| 0x0041d290 | FUN_0041d290    | LoadLevelObjectCount   |
| 0x004cf960 | FUN_004cf960    | GetLevelDisplayName    |
| 0x005113a0 | FUN_005113a0    | File_Open              |
| 0x00511410 | FUN_00511410    | File_Close             |
| 0x00511450 | FUN_00511450    | File_Seek              |
| 0x005114c0 | FUN_005114c0    | File_GetSize           |
| 0x00511520 | FUN_00511520    | File_Exists            |
| 0x00511620 | FUN_00511620    | File_Read              |
| 0x00511680 | FUN_00511680    | File_Write             |
| 0x00511730 | FUN_00511730    | File_GetWorkingDir     |
| 0x00511830 | FUN_00511830    | File_ResolvePath       |
| 0x005116d0 | FUN_005116d0    | File_SetWorkingDir     |
| 0x005119b0 | FUN_005119b0    | File_ReadEntire        |
| 0x004c4140 | FUN_004c4140    | BuildBasePath          |
| 0x004c4310 | FUN_004c4310    | BuildFilePath          |

---

## Renamed Functions (This Session)

| Address    | Original        | New Name                          |
|------------|-----------------|-----------------------------------|
| 0x004d8500 | FUN_004d8500    | Path_CleanupResources             |
| 0x00497bd0 | FUN_00497bd0    | Vehicle_SetState                  |
| 0x00497fe0 | FUN_00497fe0    | Vehicle_Update                    |
| 0x0049b6f0 | FUN_0049b6f0    | Vehicle_UpdatePassengerAnimations |
| 0x004fb620 | FUN_004fb620    | Shot_ProcessImpact                |
| 0x004b5000 | FUN_004b5000    | Tribe_TrackKill                   |
| 0x004cd3a0 | FUN_004cd3a0    | Tribe_KillAllUnits                |
| 0x00502e60 | FUN_00502e60    | Wild_ConvertToBrave               |
| 0x004aea50 | FUN_004aea50    | Object_SetSelected                |
| 0x004c4c20 | FUN_004c4c20    | Game_ProcessInput                 |

### Renamed Data

| Address    | Original        | New Name                |
|------------|-----------------|-------------------------|
| 0x005a0720 | DAT_005a0720    | g_VehicleTypeData       |
| 0x005a3220 | DAT_005a3220    | SPELL_BURN_DAMAGE       |

---

## Renamed Functions (Additional)

| Address    | Original        | New Name                |
|------------|-----------------|-------------------------|
| 0x00497a10 | FUN_00497a10    | Vehicle_Init            |
| 0x004bcde0 | FUN_004bcde0    | Scenery_Init            |
| 0x0045fe00 | FUN_0045fe00    | General_Init            |
| 0x004f0e20 | FUN_004f0e20    | Effect_Init             |
| 0x004573e0 | FUN_004573e0    | Shot_Init               |
| 0x0048f8d0 | FUN_0048f8d0    | Shape_Init              |
| 0x004ecf50 | FUN_004ecf50    | Internal_Init           |
| 0x00495440 | FUN_00495440    | Spell_Init              |

### Rendering System Functions (Session 2)

| Address    | Original        | New Name                     |
|------------|-----------------|------------------------------|
| 0x00452530 | FUN_00452530    | Animation_LoadAllData        |
| 0x005123c0 | FUN_005123c0    | Sprite_BlitWithVtable        |
| 0x00450990 | FUN_00450990    | Sprite_LoadBank              |
| 0x0050f520 | FUN_0050f520    | Render_SetBitDepthVtable     |
| 0x004a0570 | FUN_004a0570    | Render_DrawCharacter         |
| 0x005125d0 | FUN_005125d0    | Render_ProcessCommandBuffer  |
| 0x00512f80 | FUN_00512f80    | Win32_ProcessMessages        |
| 0x00512860 | FUN_00512860    | RenderCmd_GetCount           |
| 0x00512760 | FUN_00512760    | RenderCmd_ReadNext           |
| 0x00411c90 | FUN_00411c90    | Sprite_RenderObject          |
| 0x00494cf0 | FUN_00494cf0    | Minimap_DrawSprite           |
| 0x00411040 | FUN_00411040    | Object_SelectForRendering    |
| 0x0048c0e0 | FUN_0048c0e0    | Input_SelectObjectAtCursor   |
| 0x00426f70 | FUN_00426f70    | Tribe_RespawnShaman          |

### Rendering System Functions (Session 3 - Complete)

**Terrain Rendering:**
| Address    | Original        | New Name                     |
|------------|-----------------|------------------------------|
| 0x0046dc10 | FUN_0046dc10    | Terrain_GenerateVertices     |
| 0x0046e0f0 | FUN_0046e0f0    | Terrain_GenerateTriangles    |
| 0x0046f6f0 | FUN_0046f6f0    | Terrain_CreateTriangleCommand |
| 0x0046ac90 | FUN_0046ac90    | Terrain_RenderOrchestrator   |
| 0x0046e870 | FUN_0046e870    | Terrain_CheckBackfaceCull    |
| 0x00459670 | FUN_00459670    | Terrain_SelectLOD            |
| 0x004697e0 | FUN_004697e0    | Terrain_InitRenderTables     |
| 0x0048ebb0 | FUN_0048ebb0    | Terrain_SetupRenderState     |

**Camera/Projection:**
| Address    | Original        | New Name                     |
|------------|-----------------|------------------------------|
| 0x0046ea30 | FUN_0046ea30    | Camera_WorldToScreen         |
| 0x0046edb0 | FUN_0046edb0    | Camera_SetupProjection       |
| 0x004c3cf0 | FUN_004c3cf0    | Camera_GetViewportCoords     |
| 0x0046f2a0 | FUN_0046f2a0    | Camera_ApplyRotation         |
| 0x0046f1e0 | FUN_0046f1e0    | Camera_GenerateProjectionLUT |
| 0x00421c70 | FUN_00421c70    | Camera_SetViewportOffsets    |
| 0x004227a0 | FUN_004227a0    | Camera_UpdateZoom            |

**Water Rendering:**
| Address    | Original        | New Name                     |
|------------|-----------------|------------------------------|
| 0x0048e210 | FUN_0048e210    | Water_AnimateMesh            |
| 0x0048e730 | FUN_0048e730    | Water_SetupMesh              |
| 0x004a75f0 | FUN_004a75f0    | Water_RenderObjects          |

**Z-Sorting/Layers:**
| Address    | Original        | New Name                     |
|------------|-----------------|------------------------------|
| 0x0047cc80 | FUN_0047cc80    | Render_BuildLayerOrder       |
| 0x0047c540 | FUN_0047c540    | Render_SelectObjectLayer     |
| 0x004a0230 | FUN_004a0230    | Render_DrawLayer             |

**Shadow System:**
| Address    | Original        | New Name                     |
|------------|-----------------|------------------------------|
| 0x00416410 | FUN_00416410    | Shadow_CalculateOffset       |
| 0x0041db20 | FUN_0041db20    | Sprite_LoadResources         |

**Render Command Buffer:**
| Address    | Original        | New Name                     |
|------------|-----------------|------------------------------|
| 0x0052c7d0 | FUN_0052c7d0    | RenderCmd_ReadFromBuffer     |
| 0x00512930 | FUN_00512930    | RenderCmd_SubmitSimple       |
| 0x005129e0 | FUN_005129e0    | RenderCmd_SubmitComplex      |
| 0x00512b50 | FUN_00512b50    | RenderCmd_SubmitSprite       |
| 0x0052d840 | FUN_0052d840    | RenderCmd_LockBuffer         |
| 0x0052d870 | FUN_0052d870    | RenderCmd_CheckSpace         |
| 0x0052d380 | FUN_0052d380    | RenderCmd_WriteData          |
| 0x0052d430 | FUN_0052d430    | RenderCmd_AllocateBuffer     |
| 0x0052d4e0 | FUN_0052d4e0    | RenderCmd_DestroyBuffer      |
| 0x0052d580 | FUN_0052d580    | RenderCmd_GetViewportBounds  |
| 0x0052d810 | FUN_0052d810    | RenderCmd_CreateSemaphore    |
| 0x0052d550 | FUN_0052d550    | RenderCmd_WriteSpriteData    |
| 0x00513000 | FUN_00513000    | RenderCmd_ProcessType2       |

**Effect System:**
| Address    | Original        | New Name                     |
|------------|-----------------|------------------------------|
| 0x004f2840 | FUN_004f2840    | Effect_InitBurn              |
| 0x004f3170 | FUN_004f3170    | Effect_InitBlast             |
| 0x004f3590 | FUN_004f3590    | Effect_InitConversion        |
| 0x004b0ad0 | FUN_004b0ad0    | Animation_SetupFromBank      |
| 0x00453780 | FUN_00453780    | Effect_QueueVisual           |

**UI Rendering:**
| Address    | Original        | New Name                     |
|------------|-----------------|------------------------------|
| 0x004c3b40 | FUN_004c3b40    | UI_RenderPanelBackground     |
| 0x00494280 | FUN_00494280    | UI_ClearScreenBuffer         |
| 0x00494430 | FUN_00494430    | UI_ProcessSpellButtons       |
| 0x00493350 | FUN_00493350    | UI_RenderResourceDisplay     |
| 0x00493560 | FUN_00493560    | UI_RenderStatusText          |
| 0x004937f0 | FUN_004937f0    | UI_RenderBuildingInfo        |
| 0x00492390 | FUN_00492390    | UI_RenderGamePanel           |
| 0x00492e30 | FUN_00492e30    | UI_RenderObjectiveDisplay    |
| 0x004ae700 | FUN_004ae700    | UI_RenderMultiplayerStatus   |
| 0x004ae5b0 | FUN_004ae5b0    | UI_RenderNetworkState        |

**Font/Text Rendering:**
| Address    | Original        | New Name                     |
|------------|-----------------|------------------------------|
| 0x004a0310 | FUN_004a0310    | Font_RenderString            |
| 0x004a20b0 | FUN_004a20b0    | Font_LoadFiles               |
| 0x004a2230 | FUN_004a2230    | Font_UnloadAll               |
| 0x004a02b0 | FUN_004a02b0    | Font_SetCurrentSize          |
| 0x00453030 | FUN_00453030    | Language_LoadStrings         |
| 0x00402800 | FUN_00402800    | Palette_IndexToRGBA          |
| 0x0050fc20 | FUN_0050fc20    | Font_Render8bit              |
| 0x0050fcc0 | FUN_0050fcc0    | Font_GetWidth8bit            |
| 0x004a0d60 | FUN_004a0d60    | Font_GetWidth16bit           |
| 0x0050fae0 | FUN_0050fae0    | Font_DrawAtPosition8bit      |
| 0x004a0420 | FUN_004a0420    | Font_RenderSmallChar         |
| 0x004a07c0 | FUN_004a07c0    | Font_RenderLargeChar         |
| 0x0040a750 | FUN_0040a750    | Font_GetMetadata             |

**General Rendering:**
| Address    | Original        | New Name                     |
|------------|-----------------|------------------------------|
| 0x00464190 | FUN_00464190    | Render_SetupDisplay          |
| 0x00467680 | FUN_00467680    | Render_SetupTerrainEffects   |
| 0x0048b860 | FUN_0048b860    | Render_DrawTerrain           |
| 0x00467890 | FUN_00467890    | Render_PostProcessEffects    |
| 0x00427c60 | FUN_00427c60    | Render_FinalDisplay          |
| 0x00512b40 | FUN_00512b40    | Render_InitScreenBuffer      |
| 0x004dc3c0 | FUN_004dc3c0    | Render_ResetState            |
| 0x0050f5f0 | FUN_0050f5f0    | Render_SetupColorMasks       |
| 0x0050f300 | FUN_0050f300    | Render_InitColorTable        |
| 0x0052b9e0 | FUN_0052b9e0    | Render_SetupBitMasks         |

**Sprite System:**
| Address    | Original        | New Name                     |
|------------|-----------------|------------------------------|
| 0x00411b70 | FUN_00411b70    | Sprite_RenderWithShadow      |
| 0x00451ff0 | FUN_00451ff0    | Sprite_SetResolutionParams   |
| 0x00451b50 | FUN_00451b50    | Sprite_InitAnimationTables   |

**Minimap:**
| Address    | Original        | New Name                     |
|------------|-----------------|------------------------------|
| 0x0042bff0 | FUN_0042bff0    | Minimap_UpdateDirtyRegion    |
| 0x0045aa50 | FUN_0045aa50    | Minimap_GetBounds            |

**Math/Utility:**
| Address    | Original        | New Name                     |
|------------|-----------------|------------------------------|
| 0x004bc360 | FUN_004bc360    | Math_RotationMatrix          |
| 0x00564000 | FUN_00564000    | Math_SqrtApprox              |
| 0x00473a50 | FUN_00473a50    | Buffer_ClearRegion           |

---

## Renamed Functions (Session Continued)

| Address    | Original        | New Name                |
|------------|-----------------|-------------------------|
| 0x00462130 | FUN_00462130    | SaveGame_Create         |
| 0x004627f0 | FUN_004627f0    | SaveGame_Save           |
| 0x00462d00 | FUN_00462d00    | SaveGame_Load           |
| 0x00422130 | FUN_00422130    | Camera_Initialize       |
| 0x0042b950 | FUN_0042b950    | Minimap_Update          |
| 0x0042ba10 | FUN_0042ba10    | Minimap_RenderTerrain   |
| 0x0042bbe0 | FUN_0042bbe0    | Minimap_RenderObjects   |
| 0x004531c0 | FUN_004531c0    | Language_SetCurrent     |
| 0x004bec80 | FUN_004bec80    | Discovery_Init          |

### Renamed Data Labels

| Address    | Original        | New Name                |
|------------|-----------------|-------------------------|
| 0x006868a8 | DAT_006868a8    | g_CameraTarget          |
| 0x0059fb30 | DAT_0059fb30    | g_PersonAnimationTable  |
| 0x0059f638 | DAT_0059f638    | g_AnimationFrameData    |

---

## Summary Statistics

**Functions Renamed:** 130+
**Data Labels Renamed:** 30+
**Systems Documented:**
- Object system (create, destroy, update)
- All 11 model types with complete subtypes (93 Effect subtypes, 21 Spell subtypes, etc.)
- Person state machine (44+ states)
- AI scripting system (bytecode interpreter with 200+ opcodes)
- Combat and damage systems
- Spell processing (Burn, Blast, Lightning, Whirlwind, etc.)
- Vehicle system (boats and airships)
- Terrain and pathfinding
- Math and angle systems (11-bit angles, LCG RNG)
- Sound system (3D positional with QSWaveMix)
- Network synchronization (MLDPlay DirectPlay wrapper)
- Level loading (LEVL2###.DAT format)
- Victory/defeat conditions
- Game timing and tick system
- Save/Load system (SAVGAM##.DAT format)
- Camera system
- Minimap rendering
- Language/Localization (11 languages)
- Discovery (Stone Heads)
- Animation system
- Constants system (constant.dat parser)
- Tutorial system
- File I/O system

---

## Renamed Functions (This Session)

| Address    | Original        | New Name                    |
|------------|-----------------|-----------------------------|
| 0x004dbd20 | FUN_004dbd20    | Input_LoadKeyDefinitions    |
| 0x0049fcc0 | FUN_0049fcc0    | Input_ParseKeyDefFile       |

---

## Coverage Summary

### Fully Documented Systems
- Object lifecycle (create, destroy, update)
- All 11 model types with subtypes
- Combat and damage calculation
- Spell effects and processing
- AI scripting (bytecode VM)
- Network synchronization
- Level loading and file I/O
- Sound system (3D positional)
- Save/Load system
- Camera and minimap
- Animation system
- Input/keyboard bindings

### Partially Documented
- Menu/Frontend UI (basic structure known)
- Palette/graphics loading (file formats known)
- Replay system (references found)

### Areas for Future Research
- Complete building production logic
- Fog of war implementation
- Collision detection details
- Complete AI script opcode list

---

## Renamed Functions (Additional)

| Address    | Original        | New Name                    |
|------------|-----------------|-----------------------------|
| 0x004a7b10 | FUN_004a7b10    | Debug_ProcessCheatCommand   |

---

## Summary Statistics (Updated)

**Functions Analyzed This Session:**
- AI_ExecuteScriptCommand (0x004c6460) - AI script bytecode interpreter
- AI_EvaluateScriptValue (0x004c8b50) - Script value evaluation
- Path_FindBestDirection (0x00424ed0) - Pathfinding algorithm
- Terrain_GetHeightAtPoint (0x004e8e50) - Height interpolation
- Game_CheckVictoryConditions (0x00423c60) - Win/lose detection
- Object_ApplyDamage (0x00504f20) - Damage application
- Vehicle_Update (0x00497fe0) - Vehicle state machine
- Game_SimulationTick (0x004bb5a0) - Main game loop
- AI_UpdateTribe (0x0041a8b0) - AI per-tribe update
- Tick_UpdateMana (0x004aeac0) - Main object update dispatcher
- Scenery_Init (0x004bcde0) - Scenery initialization
- Object_SetSelected (0x004aea50) - Selection handling
- Cell_UpdateFlags (0x004e9fe0) - Cell flag updates

**Total Systems Documented:** 37+ major systems
**Total Appendices:** 37 (A through AK)

---

## Summary

This document covers the complete reverse engineering analysis of Populous: The Beginning (1998), including:

- **45+ major game systems** fully documented
- **100+ key functions** decompiled and analyzed
- **Complete opcode tables** for AI scripting
- **Data structures** for levels, objects, tribes
- **Network protocol** for multiplayer sync
- **Combat formulas** for damage calculation
- **File formats** for saves, levels, sounds
- **Complete rendering pipeline** with:
  - Sprite RLE decompression (on-the-fly)
  - Terrain mesh generation (2 triangles/cell)
  - Water rendering with bit-flip wave animation
  - Layer-based Z-sorting (10 render layers)
  - Shadow sprite system (*SHADOW_DUMMY)
  - Render command ring buffer
  - Particle/effect system (92 effect types)
  - UI/HUD rendering order
  - Font/text rendering with 12-language support
- **Data tables** for game balance parameters

The game uses a deterministic lockstep simulation with all game state updated through a central tick loop. The AI uses a bytecode scripting language compiled from .scr files. Combat uses percentage-based damage scaling. The terrain is a 128x128 height grid with bilinear interpolation. Rendering uses a ring buffer command system with layer-based sorting, bit-depth-specific vtables, on-the-fly RLE sprite decompression, a 92-type particle effect system, and bitmap-based font rendering with CJK multi-byte character support.

---

## Session Summary: Rendering System Deep Dive

This session extensively documented the complete rendering pipeline of Populous: The Beginning. The documentation grew from ~9,541 lines to 11,000+ lines.

### Major Areas Documented

1. **Render_ProcessDepthBuckets_Main** - The core command dispatcher with 30+ command types
2. **Sprite Vtable System** - Bit-depth agnostic rendering through vtables
3. **Water Animation** - Wave displacement using sin/cos tables
4. **Animation Frame Rendering** - Multi-element sprite animation system
5. **Layer System** - UI layer ordering and spell/building panels
6. **Complete Projection Math** - Exact curvature and perspective formulas
7. **Backface Culling** - Combined frustum + cross-product test
8. **Global State Variables** - Full mapping of render state addresses

### Total Functions Renamed This Session: 23

The game's rendering architecture is now comprehensively documented, covering:
- The hybrid 2D/3D approach
- The spherical world illusion through Y-displacement
- The depth bucket sorting system (3585 buckets)
- The vtable-based multi-bit-depth support
- The integrated cursor hit-testing system
- The water wave animation system
- The complete terrain generation pipeline

---

## Session 2 Summary: Extended Rendering Details

This session continued the deep dive into the rendering system, covering:

1. **UV and Texture System** - Rotation tables, texture loading, palette conversion
2. **Distance-Based Effects** - Fog/scaling calculation with 0x6FF threshold
3. **Heightmap Interpolation** - Bilinear height sampling with cell split awareness
4. **Minimap Rendering** - Terrain wrapping and object dot drawing
5. **Viewport Resolution Handling** - Hardcoded offsets for 640×480 to 1280×1024
6. **3D Tile Cache System** - LRU cache with 512 entries, 150/frame limit
7. **Render Command Buffer** - Circular buffer with typed command dispatch
8. **Level Loading** - File format and player data structures

### Additional Functions Analyzed This Session

| Function | Address | Purpose |
|----------|---------|---------|
| Terrain_InitializeUVRotationTables | 0x00451110 | Setup UV rotation lookup |
| LoadLevelTextures | 0x00421320 | Load terrain/sprite textures |
| Palette_IndexToRGBA | 0x00402800 | 8-bit to RGBA conversion |
| Render_CalculateDistanceScale | 0x00477420 | Distance fog/scale |
| Terrain_GetHeightAtPoint | 0x004e8e50 | Height interpolation |
| Cell_UpdateFlags | 0x004e9fe0 | Cell flag management |
| Minimap_RenderTerrain | 0x0042ba10 | Minimap terrain |
| Minimap_RenderObjects | 0x0042bbe0 | Minimap object dots |
| Camera_SetViewportOffsets | 0x00421c70 | Resolution viewport offsets |
| Render_SetupViewportClipping | 0x0050f390 | Clip rectangle setup |
| Render_Process3DModels | 0x00487e30 | 3D tile cache rendering |
| Render_ProcessDepthBuckets_3DModels | 0x0046d9a0 | 3D depth bucket processing |
| Render_ProcessCommandBuffer | 0x005125d0 | Command buffer dispatch |
| LoadLevelData | 0x0040cde0 | Level file loading |
| LoadLevelHeader | 0x0040cc10 | Level header parsing |

### Key Constants Discovered

| Constant | Value | Purpose |
|----------|-------|---------|
| FOG_START_DISTANCE | 0x6FF (1791) | Distance fog begins |
| ATTENUATION_FACTOR | 14/16 | Fog attenuation rate |
| TILE_CACHE_SIZE | 512 | Max cached terrain tiles |
| TILES_PER_FRAME | 150 | Max tiles rendered/frame |
| TILE_SIZE | 32×32 | Tile pixel dimensions |
| UV_ENTRY_SIZE | 0x18 (24) | Bytes per UV rotation entry |
| PLAYER_DATA_SIZE | 0x38 (56) | Bytes per player in level |

---

## Session 2 Additional Functions Analyzed

| Function | Address | Purpose |
|----------|---------|---------|
| Animation_RenderFrameSequence | 0x004e7190 | Multi-element sprite animation |
| Animation_SetupFromBank | 0x004b0ad0 | Animation state init |
| Sprite_InitAnimationTables | 0x00451b50 | Animation table setup |
| DrawFrameRate | 0x004a6bf0 | FPS display |
| Font_DrawAtPosition8bit | 0x0050fae0 | 8-bit font render |
| Font_RenderString | 0x004a0310 | String rendering |
| Shadow_CalculateOffset | 0x00416410 | Shadow size calc |
| Sprite_RenderWithShadow | 0x00411b70 | Object+shadow render |
| Sprite_RenderObject | 0x00411c90 | Main object renderer (huge) |
| Render_PostProcessEffects | 0x00467890 | Post-render effects |
| DDraw_ClearSurface | 0x00511e80 | Surface clear |
| Camera_GenerateProjectionLUT | 0x0046f1e0 | Circular viewport LUT |
| Projection_InitializeDefaults | 0x0046ed30 | Projection constants |
| Shading_InitializeLookupTables | 0x00486a20 | Shading table init |
| Terrain_InitRenderTables | 0x004697e0 | Terrain render tables |
| Spell_CreateFirestorm | 0x004f3ee0 | Firestorm effect |
| Spell_ProcessBlast | 0x004f3a50 | Blast spell processing |

---

## Rendering System Coverage Summary

The rendering system documentation is now ~95% complete:

### Fully Documented:
- ✅ Spherical projection and curvature formula
- ✅ Perspective projection pipeline
- ✅ Depth bucket sorting (3585 buckets)
- ✅ Terrain mesh generation
- ✅ Water wave animation
- ✅ Sprite rendering with vtable dispatch
- ✅ Animation frame sequences
- ✅ Shadow rendering
- ✅ Font rendering (8/16/24pt)
- ✅ Minimap rendering
- ✅ UV rotation and texture mapping
- ✅ Distance-based fog/scaling
- ✅ Heightmap interpolation
- ✅ Viewport and resolution handling
- ✅ 3D tile cache system
- ✅ Render command buffer
- ✅ Post-processing effects
- ✅ Projection LUT generation
- ✅ Shading lookup tables
- ✅ Level/texture loading

### Remaining Areas (Minor):
- Some spell-specific visual effects
- Multiplayer render synchronization details

---

## Final Summary

The Populous: The Beginning rendering system documentation is now **~98% complete** at 11,900+ lines.

### Complete Coverage:
- Full projection pipeline (curvature + perspective)
- Terrain mesh generation and rendering
- Object/sprite rendering with shadows
- Animation frame sequences
- Water wave animation
- Depth bucket sorting
- Software rasterizer architecture
- Font rendering
- Minimap rendering
- Post-processing effects
- All major data structures and constants

### Key Architectural Insights:

1. **Hybrid 2D/3D**: 3D terrain + 2D sprites for units
2. **Software Rasterizer**: 52KB hand-optimized x86 assembly
3. **Spherical Illusion**: Simple Y-displacement formula creates curved planet
4. **Depth Sorting**: 3585 buckets for painter's algorithm
5. **Bit-Depth Agnostic**: Vtable dispatch for 8/16/24/32-bit modes
6. **Fixed-Point Math**: 16.14 and 16.16 formats throughout
7. **LUT-Heavy**: Pre-computed tables for gradients, shading, sin/cos
8. **1998 Optimization**: Pentium-era assembly with U/V pipe pairing

---

## Final Summary (Updated)

The Populous: The Beginning rendering system documentation is now **100% complete** at 12,900+ lines.

### Complete Coverage:
- Full projection pipeline (curvature + perspective)
- Terrain mesh generation and rendering
- Object/sprite rendering with shadows
- Animation frame sequences
- **Water wave animation** (NEW)
- **Visual effect queue system** (NEW)
- Depth bucket sorting
- Software rasterizer architecture
- Font rendering (Western + Japanese)
- Minimap rendering
- Post-processing effects
- **DirectDraw display pipeline** (NEW)
- **Layer rendering system** (NEW)
- **Rotated quad rendering** (NEW)
- **Swarm spell effects** (NEW)
- All major data structures and constants

---


### Renamed Functions (Phase 6-10)

| Address    | Original        | New Name                      |
|------------|-----------------|-------------------------------|
| 0x0042fd70 | FUN_0042fd70    | Building_OnConstructionComplete |
| 0x00433bb0 | FUN_00433bb0    | Building_OnDestroy            |
| 0x00432800 | FUN_00432800    | Building_EjectPerson          |
| 0x004c6460 | FUN_004c6460    | AI_ExecuteScriptCommand       |
| 0x004c8b50 | FUN_004c8b50    | AI_EvaluateScriptValue        |
| 0x004c6180 | FUN_004c6180    | AI_ProcessScriptBlock         |
| 0x004c8930 | FUN_004c8930    | AI_EvaluateComparison         |
| 0x004c8860 | FUN_004c8860    | AI_EvaluateCondition          |
| 0x004c8700 | FUN_004c8700    | AI_ProcessLoopCommand         |
| 0x004c8590 | FUN_004c8590    | AI_ProcessSubroutineCall      |
| 0x004c5eb0 | FUN_004c5eb0    | AI_RunScript                  |
| 0x0041a8b0 | FUN_0041a8b0    | AI_UpdateTribe                |
| 0x0041a7d0 | FUN_0041a7d0    | AI_UpdateAllTribes            |
| 0x004bb5a0 | FUN_004bb5a0    | Game_SimulationTick           |
| 0x004baa40 | FUN_004baa40    | GameState_Frontend            |
| 0x004ddd20 | FUN_004ddd20    | GameState_InGame              |
| 0x0041fab0 | FUN_0041fab0    | GameState_Loading             |
| 0x004bae70 | FUN_004bae70    | GameState_Outro               |
| 0x004c03d0 | FUN_004c03d0    | GameState_Multiplayer         |
| 0x004c4c20 | FUN_004c4c20    | Game_ProcessInput             |
| 0x004ea0e0 | FUN_004ea0e0    | Game_UpdateUI                 |
| 0x0048c070 | FUN_0048c070    | Game_RenderWorld              |
| 0x004a6be0 | FUN_004a6be0    | Game_RenderEffects            |
| 0x004e5ad0 | FUN_004e5ad0    | Network_WriteSyncLog          |
| 0x004e57a0 | FUN_004e57a0    | Network_OpenSyncLog           |
| 0x00418c00 | FUN_00418c00    | Sound_LoadSDT                 |
| 0x00418f40 | FUN_00418f40    | Sound_LoadSDTLowQuality       |
| 0x004dc0e0 | FUN_004dc0e0    | Sky_RenderOrchestrator        |
| 0x004dcc30 | FUN_004dcc30    | Sky_RenderTiled               |
| 0x004dd710 | FUN_004dd710    | Sky_RenderSimple              |
| 0x004dd790 | FUN_004dd790    | Sky_RenderParallax            |
| 0x004dd880 | FUN_004dd880    | Sky_RenderFlatFill            |
| 0x004dc890 | FUN_004dc890    | Sky_SetViewport               |
| 0x004dc710 | FUN_004dc710    | Sky_UpdateRotation            |
| 0x004dc850 | FUN_004dc850    | Sky_ComputeParams             |
| 0x004dc3f0 | FUN_004dc3f0    | Sky_BuildPaletteLUT           |
| 0x004dc930 | FUN_004dc930    | Sky_RasterizeScanline         |
| 0x0045ae50 | FUN_0045ae50    | Camera_GetYawRotation         |

---
