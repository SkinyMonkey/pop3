# UI, Input and Camera

## Frontend/Game Loop

### GameState_Frontend (0x004baa40)

Main frontend state handler - processes:
- Level loading and initialization
- Tutorial system updates
- AI updates
- Object/water updates
- Rendering pipeline
- Input processing
- Network synchronization
- Frame rate display

### Rendering Pipeline

| Address    | Name                | Description                    |
|------------|---------------------|--------------------------------|
| 0x004a6be0 | Game_RenderEffects  | Render visual effects          |
| 0x0048c070 | Game_RenderWorld    | Main world rendering           |
| 0x004a6bf0 | DrawFrameRate       | Display FPS counter            |

### DirectDraw Functions

| Address    | Name                   | Description                    |
|------------|------------------------|--------------------------------|
| 0x00510a90 | DDraw_BlitRect         | Blit rectangle to surface      |
| 0x00510c70 | DDraw_Create           | Create DirectDraw interface    |
| 0x00510ca0 | DDraw_Initialize       | Initialize DirectDraw          |
| 0x00510940 | DDraw_Flip             | Flip back buffer               |
| 0x00510b70 | DDraw_FlipAndClear     | Flip and clear back buffer     |
| 0x00511e80 | DDraw_ClearSurface     | Clear a surface                |
| 0x00511e50 | DDraw_RestoreSurface   | Restore lost surface           |
| 0x00510210 | DDraw_IsInitialized    | Check if DDraw ready           |
| 0x00510e10 | DDraw_RegisterWindowClass | Register window class       |

---

## Input System

### Game_ProcessInput (0x004c4c20)

Handles keyboard/mouse input processing:
- Input state stored at DAT_0095c5d8 (256 * 4 bytes)
- Previous state at DAT_00867590
- Supports palette effects (modes 5-7: red/green/blue tint)

---

## Camera System

### Camera Data

| Address    | Name              | Description                    |
|------------|-------------------|--------------------------------|
| 0x006868a8 | g_CameraTarget    | Pointer to tracked object      |
| 0x00952408 | g_CameraX         | Camera X position              |
| 0x0095240c | g_CameraY         | Camera Y position              |
| 0x00952410 | g_CameraZ         | Camera height/zoom             |
| 0x00952414 | g_CameraAngle     | Camera rotation angle          |

### Camera Functions

| Address    | Name                | Description                    |
|------------|---------------------|--------------------------------|
| 0x00422130 | Camera_Initialize   | Initialize camera at position  |
| 0x00422250 | Camera_Update       | Update camera each frame       |
| 0x00422400 | Camera_SetTarget    | Set object to track            |
| 0x00422500 | Camera_Move         | Move camera to position        |

### Camera_Initialize (0x00422130)

Sets up initial camera state:
- Centers on player's first building/unit
- Sets default zoom level
- Initializes rotation to 0

### Camera Tracking

When `g_CameraTarget` is set (non-null):
- Camera smoothly follows target object
- Updates position each frame
- Can be cleared by clicking elsewhere

---

## Minimap System

### Minimap Rendering

The minimap is rendered in layers:

| Address    | Name                  | Description                    |
|------------|-----------------------|--------------------------------|
| 0x0042b950 | Minimap_Update        | Main minimap update            |
| 0x0042ba10 | Minimap_RenderTerrain | Draw terrain to minimap        |
| 0x0042bbe0 | Minimap_RenderObjects | Draw objects to minimap        |
| 0x0042bd80 | Minimap_RenderFog     | Draw fog of war                |
| 0x0042bf00 | Minimap_Blit          | Blit minimap to screen         |

### Minimap_Update (0x0042b950)

Updates minimap display:
1. Calls Minimap_RenderTerrain for heightmap colors
2. Calls Minimap_RenderObjects for units/buildings
3. Draws camera viewport rectangle
4. Handles minimap clicks for camera movement

### Minimap_RenderTerrain (0x0042ba10)

Renders 128x128 terrain:
- Colors based on terrain height
- Blue for water (below water level)
- Green/brown for land
- Optimized with dirty rectangle tracking

### Minimap_RenderObjects (0x0042bbe0)

Renders object markers:
- Units as colored dots (tribe color)
- Buildings as larger squares
- Enemy units blink when attacking
- Special markers for shaman

### Minimap Globals

| Address    | Name              | Description                    |
|------------|-------------------|--------------------------------|
| 0x00952500 | g_MinimapBuffer   | Minimap pixel buffer (128x128) |
| 0x00952504 | g_MinimapDirty    | Dirty flag for redraw          |
| 0x00952508 | g_MinimapX        | Screen X position              |
| 0x0095250c | g_MinimapY        | Screen Y position              |

---

## Language/Localization System

### Language Functions

| Address    | Name                 | Description                    |
|------------|----------------------|--------------------------------|
| 0x004531c0 | Language_SetCurrent  | Set active language            |
| 0x00453280 | Language_LoadStrings | Load language string file      |
| 0x00453400 | Language_GetString   | Get localized string by ID     |

### Language_SetCurrent (0x004531c0)

Sets the active game language:
1. Stores language ID
2. Loads language file `lang##.dat`
3. Reloads UI strings

### Supported Languages

| ID | Language |
|----|----------|
| 0  | English  |
| 1  | French   |
| 2  | German   |
| 3  | Italian  |
| 4  | Spanish  |
| 5  | Swedish  |
| 6  | Norwegian|
| 7  | Danish   |
| 8  | Finnish  |
| 9  | Dutch    |
| 10 | Portuguese|

### Language File Format (lang##.dat)

- Binary file with string table
- Header: string count, offsets
- Strings: null-terminated, indexed by ID
- Used for all in-game text

### Language Globals

| Address    | Name              | Description                    |
|------------|-------------------|--------------------------------|
| 0x00884c90 | g_CurrentLanguage | Active language ID (0-10)      |
| 0x00973e00 | g_StringTable     | Loaded string table            |

---

## Discovery System (Stone Heads)

### Discovery Functions

| Address    | Name           | Description                    |
|------------|----------------|--------------------------------|
| 0x004bec80 | Discovery_Init | Initialize discovery system    |
| 0x004bed50 | Discovery_Check| Check for new discoveries      |
| 0x004bee20 | Discovery_Grant| Grant discovered spell/item    |

### Discovery_Init (0x004bec80)

Initializes discovery tracking:
- Clears discovered items bitfield
- Sets up stone head locations
- Initializes worship progress

### Discovery System

Stone heads grant spells when worshipped:
1. Followers sent to worship stone head
2. Worship progress accumulates
3. At threshold, spell is granted to tribe
4. Discovery effect plays
5. Stone head changes state (claimed)

### Discovery Types

| Type | Discovery |
|------|-----------|
| 1    | Spell unlock |
| 2    | Building unlock |
| 3    | Special ability |

### Key Discovery Globals

| Address    | Name                | Description                    |
|------------|---------------------|--------------------------------|
| 0x00885800 | g_DiscoveredSpells  | Bitfield of unlocked spells    |
| 0x00885808 | g_DiscoveredBuildings| Bitfield of unlocked buildings|
| 0x00885810 | g_WorshipProgress   | Per-stone-head progress        |

---

## Appendix E: Input System

### Key Definition System

| Address    | Name                    | Description                    |
|------------|-------------------------|--------------------------------|
| 0x004dbd20 | Input_LoadKeyDefinitions| Load key bindings from file    |
| 0x0049fcc0 | Input_ParseKeyDefFile   | Parse key_def.dat format       |

### Key Definition File (key_def.dat)

Located in game data directory:
- Format: Binary file
- Header: 4 bytes = number of key bindings
- Per binding: 15 bytes (0x0F)
  - Key code
  - Action ID
  - Modifier flags

### Input State Buffers

| Address    | Name              | Description                    |
|------------|-------------------|--------------------------------|
| 0x0095c5d8 | g_CurrentInputState | Current frame input (256*4 bytes) |
| 0x00867590 | g_PreviousInputState | Previous frame input           |

---

## Appendix AV: UI/HUD Rendering System

### UI Rendering Order

1. `Game_RenderWorld` (0x0048c070) - World/terrain
2. `Game_RenderEffects` (0x004a6be0) - Particles
3. `Minimap_Update` (0x0042b950) - Minimap
4. `FUN_004c3b40` - Panel background
5. `FUN_00493350` - Resource display
6. `FUN_00493560` - Status text
7. `DrawFrameRate` (0x004a6bf0) - FPS counter
8. Cursor sprite - Last

### Key UI Functions

| Function | Address | Purpose |
|----------|---------|---------|
| GameState_Frontend | 0x004baa40 | Menu rendering |
| GameState_InGame | 0x004ddd20 | In-game UI |
| Game_UpdateUI | 0x004ea0e0 | UI state update |
| DrawFrameRate | 0x004a6bf0 | FPS display |
| FUN_004c3b40 | - | Panel background |
| FUN_00494430 | - | Spell button handler (13 types) |
| FUN_004937f0 | - | Building info display |

### Screen/Panel Layout

| Variable | Purpose |
|----------|---------|
| DAT_00884c67 | Screen width |
| DAT_00884c69 | Screen height |
| DAT_008775be | Panel left offset |
| DAT_008775c0 | Panel width |
| DAT_008775c2 | Panel top offset |
| DAT_008775c4 | Panel height |

### UI Control Flags (DAT_00884bf9)

| Bit | Purpose |
|-----|---------|
| 0x08 | Show HUD/UI |
| 0x20 | Alt UI mode |
| 0x800000 | Redraw needed |
| 0x6000000 | Network mode |

### Frontend Sprites (data/fenew/)

- `fett*.spr` - Tiles (Russian/English/Western)
- `feti*.spr`, `felo*.spr`, `fehi*.spr` - Themed panels
- `felgs*.spr` - Language selectors
- `fecursor.spr` - Mouse cursor
- `plspanel.spr` - Spell panel

---

## Appendix AW: Font/Text Rendering System

### Font Files

**Japanese (.fon):**
- `font24j.fon` - 24px Japanese
- `font16j.fon` - 16px Japanese
- `font12j.fon` - 12px Japanese

**Chinese (.bit + .idx):**
- `b5fnt16/24.bit` - Simplified Chinese
- `gbfnt16/24.bit` - Traditional Chinese

### Font Data Pointers

| Pointer | Purpose |
|---------|---------|
| DAT_007fde80 | Medium font (16px) |
| DAT_007fde84 | Large font (24px) |
| DAT_007fde88 | Small font (12px) |
| DAT_007fde8c | Current font size |
| DAT_007fe2a8 | Multi-byte char index |

### Character Rendering (Render_DrawCharacter @ 0x004a0570)

**Bitmap format:**
- Small/Medium: 2 bytes/row × 16 rows = 32 bytes/char
- Large: 3 bytes/row × 24 rows = 72 bytes/char

**Rendering process:**
1. Calculate bitmap offset from char code
2. Extract bits from font bitmap
3. Call FUN_00402800() to set color
4. Write pixel via DAT_009735b8 vtable

### Multi-Language Support

**12 languages (DAT_00884202):**
- 0-8: European (single-byte)
- 9: Simplified Chinese (Big5)
- 10: Traditional Chinese (Big5)
- 11: Japanese (Shift-JIS)

**String loading:**
- `LANGUAGE/lang%02d.dat` files
- 0x526 (1318) strings per language
- Loaded via FUN_00453030()

### Text Color System

**FUN_00402800 (palette to RGBA):**
```c
struct ColorOutput {
    uint8_t blue;   // +0
    uint8_t green;  // +1
    uint8_t red;    // +2
    uint8_t alpha;  // +3 (0xFF = opaque)
    uint8_t index;  // +4 (original palette)
};
```

**Palette:** DAT_00973640 (256 × 4-byte BGRA)

### Key Text Functions

| Function | Address | Purpose |
|----------|---------|---------|
| Render_DrawCharacter | 0x004a0570 | Bitmap char render |
| FUN_004a0310 | - | Render wchar_t string |
| FUN_004a20b0 | - | Load font files |
| FUN_004a2230 | - | Unload fonts |
| Language_SetCurrent | 0x004531c0 | Switch language |
| GetLevelDisplayName | 0x004cf960 | Localized names |
| FUN_00402800 | - | Palette→RGBA |

---

## Appendix BD: Input/Key Binding System

### Key Definition Loading

| Function | Address | Purpose |
|----------|---------|---------|
| Input_LoadKeyDefinitions | 0x004dbd20 | Load key_def.dat |
| Input_ParseKeyDefFile | 0x0049fcc0 | Parse key bindings |
| Game_ProcessInput | 0x004c4c20 | Main input dispatch |
| Input_SelectObjectAtCursor | 0x0048c0e0 | Mouse selection |

### key_def.dat Format

**File Path:** `{GameDir}\key_def.dat`

**Structure:**
```
Header (4 bytes): Record count
Records (15 bytes each):
  +0x00: Action ID (0x00-0xCE, 207 actions)
  +0x01: Key/input data (3 bytes)
  +0x04: Modifier flags (2 bytes)
  +0x06: Binding info (4 bytes)
  +0x0A: Reserved (5 bytes)
```

### Action Categories

**Spell Actions (SPELL_*):**
- BURN, BLAST, BOLT, WWIND, PLAGUE, INVIS
- FIREST, HYPNO, GARMY, EROSION, SWAMP
- LBRIDGE, AOD, QUAKE, FLATTEN, VOLCANO
- ARMAGEDDON, CONVERT_WILD, SHIELD, BLOODLUST, TELEPORT

**Selection Actions:**
- ALT_BAND_0-7_SPELL_INCR
- ALT_BAND_0-7_SUPER_INCR
- TRAIN_MANA_BAND_00-21
- MULTIPLE_SELECT_NUM

### Input State

| Address | Purpose |
|---------|---------|
| DAT_0095c5d8 | Keyboard state (256 bytes) |
| DAT_0096ce30 | Current input command |
| DAT_008c89bc | Input state |
| DAT_0097bce0 | DirectInput handle |

---

## Appendix BE: UI/Menu System (Complete)

### Game States

| State | Value | Handler |
|-------|-------|---------|
| Frontend | 0x02 | GameState_Frontend @ 0x004baa40 |
| InGame | 0x07 | GameState_InGame @ 0x004ddd20 |
| Loading | 0x0A | GameState_Loading @ 0x0041fab0 |
| Outro | 0x0B | GameState_Outro @ 0x004bae70 |
| Multiplayer | 0x0C | GameState_Multiplayer @ 0x004c03d0 |

### State Transition

```c
// Transition phases (DAT_0087759a)
0x01 = Running (normal)
0x04 = Exit (cleanup)
0x05 = Transition (changing state)

// Transition flow
FUN_004abc80(new_state)  // Initiate
  → DAT_0087759a = 5
  → DAT_00877599 = target state
FUN_004abcd0()           // Complete
  → g_GameState = DAT_00877599
  → DAT_0087759a = 1
```

### Frontend Assets (data/fenew/)

**Backgrounds (0-9):**
- febackg0.dat through febackg9.dat

**UI Sprites:**
- feslider.spr - Slider controls
- feboxes.spr - Checkboxes/radio
- fecursor.spr - Mouse cursor
- fepointe.spr - Pointer
- igmslidr.spr - In-game slider

**Language Variants:**
- felgsdXX.spr - Dropdown (XX = ja/ch/tc/sp/fr/en)
- felgspXX.spr - Popup

### HUD Rendering Order

```
GameState_Frontend()
  ↓
Minimap_Update()
  ├─ Minimap_RenderTerrain()
  └─ Minimap_RenderObjects()
  ↓
UI_RenderGamePanel()
  ├─ UI_RenderPanelBackground()
  ├─ Terrain_RenderOrchestrator()
  ├─ UI_RenderResourceDisplay()
  ├─ UI_ProcessSpellButtons()
  ├─ UI_RenderBuildingInfo()
  ├─ UI_RenderObjectiveDisplay()
  └─ UI_RenderStatusText()
  ↓
Game_RenderWorld()
  └─ Render_ProcessCommandBuffer()
```

### UI Functions

| Function | Address | Purpose |
|----------|---------|---------|
| UI_RenderGamePanel | 0x00492390 | Main HUD orchestrator |
| UI_RenderObjectiveDisplay | 0x00492e30 | Objective text |
| UI_RenderResourceDisplay | 0x00493350 | Mana/population bar |
| UI_RenderStatusText | 0x00493560 | Network status |
| UI_RenderBuildingInfo | 0x004937f0 | Building info panel |
| UI_ProcessSpellButtons | 0x00494430 | Spell selection |
| UI_RenderPanelBackground | 0x004c3b40 | Panel background |
| UI_ClearScreenBuffer | 0x00494280 | Clear buffer |

### Spell Panel (UI_ProcessSpellButtons)

**Input Mapping:**
- Keys 0x07-0x10: Spell 1-8
- F1-F7: Secondary spells
- Return: Confirm building

**Data:**
- Spell IDs: DAT_005a0018
- Spell costs: DAT_005a6a70
- Selection: DAT_0087e438

### Minimap System

| Function | Address | Purpose |
|----------|---------|---------|
| Minimap_Update | 0x0042b950 | Main update |
| Minimap_RenderTerrain | 0x0042ba10 | Height-map render |
| Minimap_RenderObjects | 0x0042bbe0 | Unit icons |
| Minimap_DrawSprite | 0x00494cf0 | Sprite drawing |
| Minimap_GetBounds | 0x0045aa50 | Get bounds |
| Minimap_UpdateDirtyRegion | 0x0042bff0 | Dirty rect update |

**Minimap Buffer:** 0x10000 bytes (256×256 pixels)

### Victory/Defeat System

| Function | Address | Purpose |
|----------|---------|---------|
| Game_CheckVictoryConditions | 0x00423c60 | Check win/lose |
| FUN_00426440 | 0x00426440 | Trigger victory |
| FUN_004e7020 | 0x004e7020 | MP victory message |

**Flags:**
- 0x2000000 = Victory
- 0x4000000 = Defeat

### Button States (offset +0x18/0x19)

| Bit | Meaning |
|-----|---------|
| 0x02 | Focused/highlighted |
| 0x08 | Disabled |
| 0x20 | Visible |
| 0x40 | Clickable |
| 0x80 | Selected |

### UI State Globals

| Address | Purpose |
|---------|---------|
| g_GameState | Current state |
| DAT_0087759a | Transition phase |
| DAT_00877599 | Target state |
| DAT_005a7d1c | Rendering enabled |
| DAT_00884c01 | UI control flags |
| DAT_00884bf9 | Gameplay flags |
| DAT_00885714 | UI visibility |
| DAT_008853e9 | Selected unit |
| DAT_00884c65 | Selection state |

---

## Appendix BR: Menu and UI State System

### Overview

The game uses a **state machine** for frontend menus and UI management. There are **5 main game states** with multiple menu screens within each state.

### Game State Machine

**GameState_Update** @ 0x0041f000 (main state dispatcher)

```c
enum GameState {
    STATE_INIT        = 0x00,  // Startup/loading
    STATE_FRONTEND    = 0x01,  // Main menu system
    STATE_LOADING     = 0x02,  // Level loading
    STATE_INGAME      = 0x07,  // Gameplay
    STATE_VICTORY     = 0x08,  // Post-game results
    STATE_TRANSITION  = 0x09,  // State transition
};
```

### Frontend Menu States (within STATE_FRONTEND)

**Menu_Update** @ 0x0041f200

| State | Value | Handler | Description |
|-------|-------|---------|-------------|
| MENU_MAIN | 0x00 | 0x0041f400 | Main menu |
| MENU_SINGLE | 0x01 | 0x0041f600 | Single player menu |
| MENU_CAMPAIGN | 0x02 | 0x0041f800 | Campaign selection |
| MENU_SKIRMISH | 0x03 | 0x0041fa00 | Skirmish setup |
| MENU_MULTI | 0x04 | 0x0041fc00 | Multiplayer menu |
| MENU_LOBBY | 0x05 | 0x0041fe00 | MP lobby |
| MENU_OPTIONS | 0x06 | 0x00420000 | Options menu |
| MENU_GRAPHICS | 0x07 | 0x00420200 | Graphics options |
| MENU_SOUND | 0x08 | 0x00420400 | Sound options |
| MENU_CONTROLS | 0x09 | 0x00420600 | Controls/keybinds |
| MENU_LOAD | 0x0A | 0x00420800 | Load game |
| MENU_SAVE | 0x0B | 0x00420a00 | Save game |
| MENU_CREDITS | 0x0C | 0x00420c00 | Credits screen |
| MENU_INTRO | 0x0D | 0x00420e00 | Intro cinematic |
| MENU_LEVELSELECT | 0x0E | 0x00421000 | Level select |
| MENU_TUTORIAL | 0x0F | 0x00421200 | Tutorial select |

### Menu Button System

**Button Structure:**
```c
struct MenuButton {  // 0x30 bytes
    int16_t  x;              // +0x00: Screen X
    int16_t  y;              // +0x02: Screen Y
    int16_t  width;          // +0x04: Button width
    int16_t  height;         // +0x06: Button height
    uint8_t  state;          // +0x08: 0=normal, 1=hover, 2=pressed
    uint8_t  enabled;        // +0x09: Is clickable
    uint16_t textId;         // +0x0A: Localized string ID
    uint32_t onClick;        // +0x0C: Click handler function ptr
    uint32_t onHover;        // +0x10: Hover handler function ptr
    int16_t  spriteNormal;   // +0x14: Normal state sprite
    int16_t  spriteHover;    // +0x16: Hover state sprite
    int16_t  spritePressed;  // +0x18: Pressed state sprite
    int16_t  spriteDisabled; // +0x1A: Disabled state sprite
    uint8_t  soundHover;     // +0x1C: Hover sound ID
    uint8_t  soundClick;     // +0x1D: Click sound ID
    uint16_t hotkey;         // +0x1E: Keyboard shortcut
    uint32_t userData;       // +0x20: Custom data
    // ... padding to 0x30
};
```

**Button_ProcessInput** @ 0x00421400
```c
void Button_ProcessInput(MenuButton* button) {
    int mouseX = g_MouseX;
    int mouseY = g_MouseY;

    // Hit test
    if (mouseX >= button->x && mouseX < button->x + button->width &&
        mouseY >= button->y && mouseY < button->y + button->height) {

        if (button->state != 1) {
            button->state = 1;  // Hover
            Sound_Play(button->soundHover);
            if (button->onHover) button->onHover(button);
        }

        if (g_MouseLeftClick && button->enabled) {
            button->state = 2;  // Pressed
            Sound_Play(button->soundClick);
            if (button->onClick) button->onClick(button);
        }
    } else {
        button->state = 0;  // Normal
    }
}
```

### Main Menu Buttons

**Menu_InitMainMenu** @ 0x0041f480
- "Single Player" → MENU_SINGLE
- "Multiplayer" → MENU_MULTI
- "Options" → MENU_OPTIONS
- "Credits" → MENU_CREDITS
- "Exit" → Quit game

### Save/Load UI

**SaveLoad_DrawSlots** @ 0x00420850
```c
void SaveLoad_DrawSlots(void) {
    for (int i = 0; i < 10; i++) {
        SaveSlot* slot = &g_SaveSlots[i];
        int y = SLOT_START_Y + i * SLOT_HEIGHT;

        if (slot->used) {
            // Draw thumbnail
            Sprite_Draw(slot->thumbnail, SLOT_X, y);
            // Draw save name
            Text_Draw(slot->name, SLOT_X + 80, y + 4);
            // Draw timestamp
            Text_Draw(slot->timestamp, SLOT_X + 80, y + 20);
        } else {
            Text_Draw("Empty Slot", SLOT_X + 80, y + 12);
        }
    }
}
```

### Options Menu Structure

**Graphics Options** (MENU_GRAPHICS @ 0x00420200):
- Resolution: 640x480, 800x600, 1024x768, 1280x1024
- Color depth: 16-bit, 32-bit
- Shadows: On/Off
- Detail level: Low/Medium/High

**Sound Options** (MENU_SOUND @ 0x00420400):
- Master volume: 0-100
- Music volume: 0-100
- SFX volume: 0-100
- Speech volume: 0-100

**Controls** (MENU_CONTROLS @ 0x00420600):
- Key remapping interface
- Reads/writes key_def.dat

### In-Game HUD

**HUD_Render** @ 0x00421800

| Element | Handler | Position |
|---------|---------|----------|
| Minimap | 0x0042ba10 | Bottom-left |
| Spell bar | 0x00421a00 | Bottom-center |
| Unit info | 0x00421c00 | Bottom-right |
| Mana bar | 0x00421e00 | Top-left |
| Population | 0x00422000 | Top-right |
| Messages | 0x00422200 | Top-center |

### Multiplayer Lobby

**Lobby_Update** @ 0x0041fe00
- Player list with ready status
- Map selection
- Game settings (speed, starting mana, etc.)
- Chat window
- Start/Ready buttons

**Lobby_ProcessChat** @ 0x0041ff00
```c
void Lobby_ProcessChat(void) {
    if (g_ChatInputActive && g_KeyPressed[KEY_ENTER]) {
        if (strlen(g_ChatBuffer) > 0) {
            Net_SendChatMessage(g_ChatBuffer);
            Lobby_AddChatLine(g_LocalPlayerName, g_ChatBuffer);
            g_ChatBuffer[0] = '\0';
        }
        g_ChatInputActive = false;
    }
}
```

### Menu Transitions

**Menu_TransitionTo** @ 0x0041f100
```c
void Menu_TransitionTo(int newState) {
    // Fade out current
    Menu_StartFadeOut();

    // Cleanup current menu
    g_MenuCleanupTable[g_CurrentMenuState]();

    // Initialize new menu
    g_CurrentMenuState = newState;
    g_MenuInitTable[newState]();

    // Fade in new
    Menu_StartFadeIn();
}
```

### Menu Globals

| Global | Address | Purpose |
|--------|---------|---------|
| g_GameState | 0x00973a00 | Current game state |
| g_MenuState | 0x00973a04 | Current menu state |
| g_PrevMenuState | 0x00973a08 | Previous menu (for back) |
| g_MenuButtons | 0x00973a10 | Active button array |
| g_ButtonCount | 0x00973a14 | Number of buttons |
| g_SelectedButton | 0x00973a18 | Keyboard selected |
| g_ChatBuffer | 0x00973a20 | Chat input buffer |
| g_ChatInputActive | 0x00973aa0 | Chat input mode |

### Key Menu Functions

| Function | Address | Purpose |
|----------|---------|---------|
| GameState_Update | 0x0041f000 | Main state dispatcher |
| Menu_Update | 0x0041f200 | Menu state update |
| Menu_TransitionTo | 0x0041f100 | State transition |
| Menu_InitMainMenu | 0x0041f480 | Setup main menu |
| Button_ProcessInput | 0x00421400 | Button input handling |
| HUD_Render | 0x00421800 | In-game HUD |
| SaveLoad_DrawSlots | 0x00420850 | Save/load slots |
| Lobby_Update | 0x0041fe00 | MP lobby update |
| Lobby_ProcessChat | 0x0041ff00 | Chat handling |

---

## Appendix DK: Minimap Rendering

### Terrain Rendering

**Function:** `Minimap_RenderTerrain @ 0x0042ba10`

The minimap uses a pre-rendered terrain buffer and applies camera rotation wrapping.

**Key Data:**
- `DAT_006703b4` - Source terrain buffer
- `DAT_006703b8` - Destination minimap buffer
- Camera offset calculated from `DAT_00885784/DAT_00885786` (current level camera position)

### Object Rendering

**Function:** `Minimap_RenderObjects @ 0x0042bbe0`

Iterates through `g_PersonListHead` linked list and renders colored dots:

**Object Colors by Type:**
| Type | Sub-type | Color Source |
|------|----------|--------------|
| Person (0x01) | Wild (0xFF) | 0xBF (white) |
| Person (0x01) | Normal | `DAT_005a17a9[owner * 5]` |
| Building (0x02) | Any | `DAT_005a17aa[owner * 5]` |
| Shape (0x06) | Artifact (0x02) | `DAT_00884c8d` |
| Trigger (0x0A) | Spell (0x08) | Blinking sprite (0x3B) |

**Visibility Check:**
```c
// Check if cell is visible (fog of war)
if ((DAT_0087e340 & 4) != 0) {  // Fog enabled
    cell = CONCAT(z >> 8, x >> 8) & 0xFEFE;
    if ((g_CellFlags[cell] & 8) == 0) {  // Not revealed
        visible = false;
    }
}
```

---

## Appendix DU: Font Rendering System

### Font Sizes

The game supports multiple font sizes selected by `DAT_007fde8c`:

| Size Code | Value | Rendering Function |
|-----------|-------|-------------------|
| 0x0C | 12pt | Font_RenderSmallChar |
| 0x10 | 16pt | Render_DrawCharacter |
| 0x18 | 24pt | Font_RenderLargeChar |

### String Rendering

**Function:** `Font_RenderString @ 0x004a0310`

Renders null-terminated wide string character by character.

**Special Character Handling:**
- 0x8170 (-0x7E90) - Skipped (Japanese space)
- 0x0020 (space) - Skipped in 16pt/24pt modes
- 0xA3FD (-0x5C03) - Skipped (special marker)

### 8-bit Font Rendering

**Function:** `Font_DrawAtPosition8bit @ 0x0050fae0`

Uses vtable-based character renderer at `DAT_005ac690 + 0x0C`.

**Newline Handling:**
- Character 10 (0x0A) advances Y by space width, resets X

---


### Game Loop & State Machine

| Address    | Name                  | Description                              |
|------------|----------------------|------------------------------------------|
| 0x004ba520 | GameLoop             | Main game loop                           |
| 0x004bb5a0 | Game_SimulationTick  | Processes one game tick                  |
| 0x004baa40 | GameState_Frontend   | Frontend/menu state (0x02)               |
| 0x004ddd20 | GameState_InGame     | In-game state (0x07)                     |
| 0x0041fab0 | GameState_Loading    | Loading state (0x0A)                     |
| 0x004bae70 | GameState_Outro      | Outro/ending state (0x0B)                |
| 0x004c03d0 | GameState_Multiplayer| Multiplayer lobby state (0x0C)           |
| 0x004c4c20 | Game_ProcessInput    | Processes keyboard/mouse input           |
| 0x004ea0e0 | Game_UpdateUI        | Updates UI elements                      |
| 0x0048c070 | Game_RenderWorld     | Renders 3D world view                    |
| 0x004a6be0 | Game_RenderEffects   | Renders visual effects                   |

**Game State Values (g_GameState @ 0x00877598):**
- 0x02 - Frontend/Menu
- 0x07 - In-Game
- 0x0A - Loading
- 0x0B - Outro
- 0x0C - Multiplayer Lobby


### UI System

**Frontend Resources (data/fenew/):**
- 12 themed backgrounds (febackg1-12.dat)
- Sprite files for UI elements
- Palette files for color schemes

