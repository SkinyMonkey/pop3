# Audio System

## Sound System

### Sound_Play (0x00417300)

Full 3D positional audio system:

```c
int Sound_Play(object, soundId, flags)
```

**Parameters:**
- `object` - Source object for 3D positioning (0 for global)
- `soundId` - Sound effect ID from SDT file
- `flags` - Playback flags (see below)

**Sound Flags:**
| Bit | Description |
|-----|-------------|
| 0   | 2D sound (no positioning) |
| 2   | Looping sound |
| 4   | Priority sound |
| 11  | Ignore pause state |

**3D Audio:**
- Calculates distance from camera position
- Max audible distance: 0x9000000 (squared)
- Volume based on distance falloff
- Uses separate RNG (DAT_0088420a) for variation

### Sound Data Table (0x005a5c70)

Per-sound entry (0x0C bytes):
- Offset 0x00: Base sample index
- Offset 0x02: Low quality sample index
- Offset 0x05: Sample variation count
- Offset 0x06: Low quality variation count
- Offset 0x07: Priority
- Offset 0x08: Pitch variation
- Offset 0x0A: Volume
- Offset 0x0B: Flags

### Sound Categories

| Range | Category |
|-------|----------|
| 0x1C, 0x50, 0xC2, 0xC6 | Ambient/looping |
| 0x73-0x86 | Unit acknowledgements |
| 0x1E-0x21, 0xC8-0xCA, 0xD5 | UI/menu sounds |

---

## Appendix AA: Sound System

### Sound_Play (0x00417300)

**Parameters:**
- `param_1`: Source object (or 0 for global sounds)
- `param_2`: Sound ID
- `param_3`: Flags

**Sound Flags:**
- Bit 0: Ambient/looping sound
- Bit 2: UI sound (no distance attenuation)
- Bit 4: Use object's sound group
- Bit 11: Force play even when paused

**Distance Attenuation:**
```c
// Max audible distance: 0x9000000 squared units
distance = Math_DistanceSquared(object->pos, camera->pos);
if (distance >= 0x9000001) return 0;  // Too far

// Volume scales with distance
volume = ((0x9000000 - distance) / 0x900) * soundBaseVolume;
```

**Sound Categories (by ID):**

| ID Range | Category |
|----------|----------|
| 0x1C, 0x50, 0xC2, 0xC6 | Ambient loops |
| 0x19 | Shaman-related |
| 0x73-0x86 | Combat sounds (one per object) |
| 0x1E, 0xC8 | Sound group 1 |
| 0x1F, 0xC9 | Sound group 2 |
| 0x20, 0xCA, 0xD5 | Sound group 4 |
| 0x21 | Sound group 3 |

**Random Variation:**
- Sound index varies by ±range within sound bank
- Pitch varies by ±DAT_005a5c78[soundId]%

### SDT File Format

Sound bank files loaded by Sound_LoadSDT (0x00418c00):
- Low quality variant: Sound_LoadSDTLowQuality (0x00418f40)

---

## Appendix BA: Audio System (Complete)

### Sound_Play Function (0x00417300)

**Signature:**
```c
int Sound_Play(int entity, ushort sound_id, ushort flags)
```

**Parameters:**
- `entity`: Object pointer (0 = global/listener-relative)
- `sound_id`: Sound effect ID from SDT file
- `flags`: Playback options

### Sound Control Structure (0x2a bytes)

```c
struct SoundControl {
    void*    prev;           // +0x00: Previous sound (linked list)
    void*    next;           // +0x04: Next sound
    uint16_t sound_id;       // +0x06: Sound ID
    uint16_t entity_id;      // +0x0c: Source entity
    uint16_t sound_id2;      // +0x10: Sound ID copy
    uint16_t volume;         // +0x12: Current volume
    uint8_t  amplitude;      // +0x14: Amplitude factor
    uint8_t  pan;            // +0x15: Stereo pan (0-127)
    uint8_t  final_volume;   // +0x16: Calculated volume
    uint8_t  base_volume;    // +0x17: Base volume (100)
    uint16_t flags;          // +0x18: State flags
    uint16_t category;       // +0x1a: Sound category
    void*    wave_buffer;    // +0x1c: Wave data pointer
    void*    user_data;      // +0x20: User data
    uint32_t start_time;     // +0x24: GetTickCount start
};
```

### Sound Flags (offset +0x18)

| Flag | Meaning |
|------|---------|
| 0x01 | One-shot (stop when done) |
| 0x02 | Paused state |
| 0x04 | Special audio processing |
| 0x08 | Finished/marked for deletion |
| 0x10 | Entity-linked (follows source) |
| 0x20 | Loop state |
| 0x40 | Loopable |
| 0x100 | Active/playing |
| 0x200 | Forced playback |

### Sound Data Table (0x005a5c70)

Entry size: 0xc bytes per sound ID

| Offset | Size | Purpose |
|--------|------|---------|
| 0x00-0x01 | short | Base pitch |
| 0x02-0x03 | short | High quality variant ID |
| 0x04-0x05 | short | Low quality variant ID |
| 0x06 | byte | Low quality variant count |
| 0x07 | byte | Base amplitude |
| 0x08 | byte | Volume variation |
| 0x09 | byte | Variant count |

### SDT File Loading (0x00418c00)

**Files Loaded:**
- `data/SOUND/soundd2.sdt` (0x0057b8f8) - High quality
- `data/SOUND/soundd2low.sdt` (0x0057b920) - Low quality
- `data/SOUND/popdrones22.sdt` (0x0057b900) - Ambient drones
- `data/SOUND/popfight.sf2` (0x0057b910) - SoundFont for MIDI

**Loader Types (FUN_0053a470):**
| Type | Purpose | Function |
|------|---------|----------|
| 1 | Wave stream | FUN_00541ae0() |
| 2 | MIDI/SoundFont | FUN_00541900() |
| 3 | Sample | FUN_00541680() |
| 4 | CD audio | FUN_005413a0() |
| 5 | MIDI sequencer | FUN_00540460() |
| 6 | SoundFont manager | FUN_005408e0() |

### 3D Positional Audio (FUN_004183b0)

**Distance Calculation:**
```c
distance_sq = Math_DistanceSquared(entity_pos, camera_pos);

if (distance_sq >= 0x9000000) {  // ~31,622 units max
    volume = 0;
    return 0;
}

// Volume attenuation
volume = ((0x9000000 - distance_sq) / 0x900) * amplitude >> 16;

// Pan calculation (stereo positioning)
angle = Math_Atan2(camera_x - entity_x, -(camera_y - entity_y));
pan_angle = (angle & 0x7ff) - (camera_angle + 0x200) & 0x7ff;
pan = (pan_angle < 0x400) ? (pan_angle >> 3) : ((0x7ff - pan_angle) >> 3);
```

### QSWaveMix.dll Functions (24 total)

**Initialization:**
- QSWaveMixInitEx (0xc5)
- QSWaveMixActivate (0xc6)
- QSWaveMixOpenChannel (0xc7)
- QSWaveMixPump (0xc8)

**Channel Control:**
- QSWaveMixConfigureChannel (0xbe)
- QSWaveMixEnableChannel (0xbf)
- QSWaveMixOpenWaveEx (0xc0)
- QSWaveMixFreeWave (0xc1)
- QSWaveMixPlayEx (0xc2)
- QSWaveMixStopChannel (0xbd)
- QSWaveMixPauseChannel (0xbc)
- QSWaveMixRestartChannel (0xbb)

**3D Positioning:**
- QSWaveMixSetPosition (0xba)
- QSWaveMixSetSourcePosition (0xb9)
- QSWaveMixSetListenerPosition (0xb2)
- QSWaveMixSetSourceVelocity (0xb8)
- QSWaveMixSetListenerVelocity (0xb7)

**Audio Properties:**
- QSWaveMixSetVolume (0xb3)
- QSWaveMixSetFrequency (0xb6)
- QSWaveMixSetDistanceMapping (0xb4)
- QSWaveMixSetSourceCone (0xb5)

### Sound System Globals

| Address | Purpose |
|---------|---------|
| 0x00885405 | Active sounds linked list head |
| 0x00885409 | QSWaveMix primary session |
| 0x0088540d | QSWaveMix MIDI session |
| 0x0088420a | RNG state for pitch/volume variation |
| 0x0087e347 | Sound system initialized |
| 0x0087e348 | Sound playback enabled |
| 0x0087e34a | Wave mixer ready |
| 0x0087e34b | Drone sounds loaded |
| 0x0087e34c | Master volume level |
| 0x0087e34e | SoundFont ready |
| 0x0087e357 | MIDI session active |
| 0x0087e359 | Current music track ID |
| 0x0057b8fc | Primary sound table (SoundD2.SDT) |
| 0x0057b904 | Drone ambient (PopDrones22.SDT) |
| 0x0057b914 | SoundFont data (PopFight.SF2) |

---


### Audio System (QSWaveMix)

QSWaveMix is a 3D positional audio library.

| Address    | Name                  | Description                              |
|------------|----------------------|------------------------------------------|
| 0x00418c00 | Sound_LoadSDT         | Loads sound data table (high quality)    |
| 0x00418f40 | Sound_LoadSDTLowQuality | Loads low quality sound data           |

**Audio Imports (QSWaveMix.dll):**
- `QSWaveMixInitEx` - Initialize audio system
- `QSWaveMixPlayEx` - Play sound with 3D position
- `QSWaveMixSetListenerPosition` - Set camera/listener position
- `QSWaveMixSetSourcePosition` - Set sound source position
- `QSWaveMixSetVolume` - Set volume
- `QSWaveMixOpenChannel` / `QSWaveMixCloseSession`

**MIDI for Music (winmm.dll):**
- `midiOutOpen` / `midiOutClose` - MIDI playback
- `mciSendCommandA` - Media Control Interface

**Sound Files:**
- `soundd2.sdt` / `soundd2low.sdt` - Main sound effects
- `popfightnew.sdt` - Combat sounds
- `popdrones22.sdt` - Ambient drones
- `popdrum022.sdt` - Drum sounds
- `popfight.sf2` - SoundFont for music

