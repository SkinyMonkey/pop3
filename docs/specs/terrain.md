# Terrain System

## Terrain System

### Heightmap

- **Base Address:** 0x00888980 (g_Heightmap)
- **Size:** 128x128 cells, 2 bytes per cell
- **Access Formula:** `heightmap[(y & 0xFE) * 2 | (x & 0xFE00)] * 2`
- **Height interpolation:** Bilinear interpolation between 4 corners

### Cell Flags

- **Base Address:** 0x0088897c (g_CellFlags)
- **Size:** 128x128 cells, 4 bytes per cell
- **Bit 0:** Diagonal split direction for terrain triangles
- **Bit 1:** Has blocking scenery
- **Bit 19:** Has tree/resource

### Cell Object Lists

- **Base Address:** 0x00888982
- **Each cell has a linked list of objects via offset 0x20 (next object index)

### Terrain Functions

| Address    | Name                     | Description                    |
|------------|--------------------------|--------------------------------|
| 0x004e8e50 | Terrain_GetHeightAtPoint | Interpolates height at X,Y     |
| 0x004e9fe0 | Cell_UpdateFlags         | Updates cell flags from objects |
| 0x00424ed0 | Path_FindBestDirection   | Scans terrain for best path    |

---

## Appendix S: Terrain System

### Terrain_GetHeightAtPoint (0x004e8e50)

Returns interpolated terrain height at any world coordinate.

**Height Map:**
- 128x128 grid of height values
- Stored at g_Heightmap
- Each cell is 256 world units

**Interpolation:**
- Uses bilinear interpolation between 4 corner heights
- Cell flags at g_CellFlags determine triangle split direction
- Flag bit 0: Controls which diagonal splits the cell

**Coordinate System:**
```c
cellX = (param_1 & 0x1fe) >> 1;  // X grid cell
cellZ = (param_2 & 0x1fe) >> 1;  // Z grid cell
fracX = cellX & 0xff;            // Fractional within cell
fracZ = cellZ & 0xff;
```

### Cell Flags

Located at g_CellFlags, 4 bytes per cell:
- Bit 0: Triangle split direction
- Bit 1: Obstacle present
- Bit 19: Tree present in cell

---

## Appendix AP: Terrain Mesh Generation

### Heightmap to Geometry Pipeline

```
128x128 Heightmap (g_Heightmap[])
    ↓
Terrain_GetHeightAtPoint() [bilinear interpolation]
    ↓
FUN_0046dc10() [vertex generation per cell]
    ↓
Vertex data (DAT_00699a50/54, 32 bytes/vertex)
    ↓
FUN_0046e0f0() [triangle generation: 2 per cell]
    ↓
FUN_0046f6f0() [triangle commands with shading]
    ↓
Depth buckets (DAT_00699a64[], 0xe01 buckets)
    ↓
Render_ProcessCommandBuffer()
```

### Vertex Structure (32 bytes)

```c
struct TerrainVertex {
    uint32_t flags;         // +0x00
    int32_t  height;        // +0x08
    int32_t  screen_x;      // +0x0C
    int32_t  screen_y;      // +0x10
    int32_t  brightness;    // +0x14
    uint8_t  cell_flags;    // +0x18 (0x40=special, 0x80=flag)
};
```

### Triangle Generation (FUN_0046e0f0)

**Two triangles per cell, split based on cell flag bit 0:**
- **Bit 0 = 0:** Triangle1: (0,0)-(0,1)-(1,0), Triangle2: (0,1)-(1,1)-(1,0)
- **Bit 0 = 1:** Triangle1: (0,0)-(0,1)-(1,1), Triangle2: (0,0)-(1,1)-(1,0)

This alternating pattern prevents visible diagonal seams.

### Depth Sorting (0xe01 buckets)

**Bucket calculation:**
```c
bucket = (distance + offset) >> 4;  // clamped to 0-0xe00
```

Each bucket is a linked list of render commands, processed front-to-back.

### Isometric Projection (FUN_0046ea30)

```c
// Camera transformation (14-bit fixed-point)
screen_x = center_x + (cam_x * focal_length) >> z_depth;
screen_y = center_y - (cam_y * focal_length) >> z_depth;
```

**Key globals:**
- DAT_007b8fc0 - Focal length shift
- DAT_007b8ffc/fe - Screen center
- DAT_007b8fbc - Camera Z offset

### LOD System

**Distance thresholds at DAT_0069d5e0 (0-220):**
- Near terrain: Full detail
- Far terrain: Reduced geometry
- Transition: Distance-based fade

---

## Appendix CH: Terrain Height System

### Height Data Storage

Terrain heights are stored in two related arrays:

| Address | Array | Purpose |
|---------|-------|---------|
| 0x00888980 | g_Heightmap | Primary height values (256x256 cells, 2 bytes each) |
| 0x00889180 | g_HeightmapAdjacent | Adjacent cell heights for interpolation |
| 0x00889190 | g_HeightmapDiagonal | Diagonal cell heights |
| 0x00888990 | g_HeightmapNext | Next row heights |

### Terrain_GetHeightAtPoint @ 0x004e8e50

Performs bilinear height interpolation within a cell:

```c
int Terrain_GetHeightAtPoint(ushort world_x, ushort world_z) {
    // Extract cell coordinates and sub-cell position
    byte cell_x = (world_x >> 8) & 0xFE;
    byte cell_z = (world_z >> 8) & 0xFE;
    uint frac_x = (world_x & 0x1FE) >> 1;  // 0-255 within cell
    uint frac_z = (world_z & 0x1FE) >> 1;  // 0-255 within cell

    // Get cell index
    uint cell_idx = (cell_x * 2) | (cell_z << 8);

    // Get corner heights (optimization for interior cells)
    if ((cell_x & 0xFE) != 0xFE && (cell_z & 0xFE) != 0xFE) {
        h00 = g_Heightmap[cell_idx * 2];           // This cell
        h10 = g_HeightmapAdjacent[cell_idx * 2];   // X+1
        h11 = g_HeightmapDiagonal[cell_idx * 2];   // X+1, Z+1
        h01 = g_HeightmapNext[cell_idx * 2];       // Z+1
    } else {
        // Edge cells - fetch manually with wrapping
        // ...
    }

    // Check triangle split direction (bit 0 of cell flags)
    if ((g_CellFlags[cell_idx] & 1) == 0) {
        // Split: (0,0)-(1,0)-(1,1) and (0,0)-(1,1)-(0,1)
        if (frac_z < frac_x) {
            // Lower triangle
            return h00 + ((h01 - h00) * frac_x >> 8) + ((h11 - h01) * frac_z >> 8);
        } else {
            // Upper triangle
            return h00 + ((h11 - h10) * frac_x >> 8) + ((h10 - h00) * frac_z >> 8);
        }
    } else {
        // Alternate split direction
        if (frac_x + frac_z < 0x100) {
            // Lower triangle
            return h00 + ((h01 - h00) * frac_x >> 8) + ((h10 - h00) * frac_z >> 8);
        } else {
            // Upper triangle
            return h11 + ((h01 - h11) * (0x100 - frac_z) >> 8) +
                         ((h10 - h11) * (0x100 - frac_x) >> 8);
        }
    }
}
```

### Cell Flags Array

The g_CellFlags array (0x00888978) stores per-cell properties:

| Bit | Mask | Meaning |
|-----|------|---------|
| 0 | 0x0001 | Triangle split direction (0=NW-SE, 1=NE-SW) |
| 1 | 0x0002 | Has object standing on it |
| 3 | 0x0008 | Needs redraw |
| 9 | 0x0200 | Water surface |
| 19 | 0x80000 | Has building |
| 20 | 0x100000 | Building foundation |

### Height Modification

From `Terrain_ModifyHeight @ 0x004ea2e0`:

```c
bool Terrain_ModifyHeight(int cell_ptr, short target_height, short max_change, bool update_mesh) {
    short current = *(short*)(cell_ptr + 4);
    int diff = current - target_height;

    if (diff == 0) return true;

    // Clamp change to max_change per tick
    if (abs(diff) > max_change) {
        if (diff > 0) {
            *(short*)(cell_ptr + 4) = current - max_change;
        } else {
            *(short*)(cell_ptr + 4) = current + max_change;
        }
        return false;  // Not at target yet
    }

    *(short*)(cell_ptr + 4) = target_height;

    if (update_mesh) {
        // Trigger mesh regeneration for affected area
        FUN_004e8300(cell_index, 2, 1);
        FUN_004e8450();  // Update normals
        FUN_004e9800(1, cell_index, 1, -1);  // Update rendering
    }

    return true;
}
```

---

## Appendix CP: Cell Flags System

### g_CellFlags Array

| Address | Size | Purpose |
|---------|------|---------|
| g_CellFlags | 0x40000 | 256×256 cells × 16 bytes each |

### Cell Entry (16 bytes)

| Offset | Size | Field |
|--------|------|-------|
| +0x00 | 4 | General flags |
| +0x04 | 2 | Height value (10-bit, offset 0-1023) |
| +0x06 | 2 | Material/texture info |
| +0x08 | 1 | Cell type |
| +0x09 | 1 | Reserved |
| +0x0A | 2 | Object list head |
| +0x0C | 4 | Extended flags |

### Cell Flags Bit Meanings

| Bit | Mask | Meaning |
|-----|------|---------|
| 1 | 0x02 | Has blocking object |
| 18 | 0x80000 | Has special building |
| 14 | 0x4000 | Has shape object |

### Cell_UpdateFlags @ 0x004e9fe0

Updates cell passability based on objects in the cell:
```c
void Cell_UpdateFlags(uint cell_xy) {
    uint cell_index = (cell_xy & 0xFE) * 2 | (cell_xy & 0xFE00);
    uint* flags = &g_CellFlags[cell_index];

    bool has_blocker = false;
    bool has_building = false;

    // Scan object list for cell
    for (obj = object_list[cell_index]; obj != NULL; obj = obj->next) {
        if (obj->type == 5) {  // Building
            if (building_data[obj->subtype].flags & 0x80000) {
                has_blocker = true;
            }
            if (building_data[obj->subtype].flags2 & 0x10) {
                has_building = true;
            }
        }
    }

    // Update flags
    if (has_blocker) {
        *flags |= 0x02;
    } else {
        *flags &= ~0x02;
    }

    if (has_building) {
        *flags |= 0x80000;
    } else {
        *flags &= ~0x80000;
    }
}
```

### Terrain_CheckCellFlags @ 0x0046e040

Checks cell type for rendering decisions:
```c
byte Terrain_CheckCellFlags(int vertex_ptr) {
    int type_index = (*(byte*)(vertex_ptr + 0xC) & 0x0F) * 0x0E;

    if (terrain_types[type_index] & 0x01) {
        return 1;  // Water
    }

    if ((terrain_types[type_index] & 0x3C) == 0) {
        return 0;  // Normal land
    }

    return terrain_types[type_index + 0x0D] & 0x80;  // Special
}
```

---

## Appendix CQ: Terrain Render State Constants

### Terrain_SetupRenderState @ 0x0048ebb0

Sets global rendering parameters:

```c
void Terrain_SetupRenderState(void) {
    DAT_007f9180 = 0x2800;    // Terrain scale X
    DAT_007f9184 = 0x0CCC;    // Terrain scale Y (3276)
    DAT_007f9188 = 0x10000;   // Terrain scale Z (65536)

    // Check for water visibility
    if (FUN_00459570(0x0CCC) & 1) {
        DAT_007f9184 += 0x66;  // Boost Y for water (102)
    }
}
```

### Scale Constants

| Constant | Value | Decimal | Purpose |
|----------|-------|---------|---------|
| SCALE_X | 0x2800 | 10240 | Horizontal terrain scale |
| SCALE_Y | 0x0CCC | 3276 | Vertical terrain scale |
| SCALE_Z | 0x10000 | 65536 | Depth scale (1.0 in 16.16) |
| WATER_BOOST | 0x66 | 102 | Extra Y scale for water |

---

## Appendix CV: Terrain Cell Boundary Rendering

### Terrain_RenderCellBoundary @ 0x00475830

Handles special rendering for terrain transitions (coastlines, lava edges, etc.):

```c
int Terrain_RenderCellBoundary(uint* cell, int tri1, int tri2,
                                uint* v_data1, uint* v_data2, uint flag_mask) {
    if (tri1 == 0 && tri2 == 0) return 0;

    // Get cell coordinates from pointer arithmetic
    ushort cell_index = (cell - g_CellFlags) >> 4;
    short cell_x = ((cell_index & 0x7F) * 2) | ((cell_index >> 7) & 0x7F);

    // Check adjacent cells for boundary flags
    byte adjacency = 0;

    // Check 4 cardinal neighbors
    if ((g_CellFlags[NORTH] & flag_mask) == 0) adjacency |= 0x01;
    if ((g_CellFlags[SOUTH] & flag_mask) == 0) adjacency |= 0x04;
    if ((g_CellFlags[EAST] & flag_mask) == 0) adjacency |= 0x02;
    if ((g_CellFlags[WEST] & flag_mask) == 0) adjacency |= 0x08;

    // Determine edge pattern (16 patterns)
    byte rotation;
    int edge_type;
    switch (adjacency) {
        case 0x03: rotation = 3; edge_type = 6; break;  // NE corner
        case 0x05: rotation = 1; edge_type = 2; break;  // Horizontal edge
        case 0x06: rotation = 0; edge_type = 6; break;  // SE corner
        // ... 13 more patterns ...
        case 0x0F: edge_type = 7; break;  // All edges (isolated)
        default: rotation = 0; edge_type = 5; break;
    }

    // Check diagonal neighbors for more complex patterns
    if (rotation == 4) {
        // Check 4 diagonal neighbors
        // Results in 16 additional sub-patterns
    }

    // Handle special flags
    if (flag_mask & 0x1000) { /* water */ }
    if (flag_mask & 0x400) { edge_type += 0x30; }  // Lava
    if (flag_mask & 0x100) { edge_type += 0x40; }  // Ice

    // Create boundary triangles
    int count = 0;
    if (tri1 != 0) {
        Triangle_CreateWithRotatedUVs(tri1, edge_type, 0, 0);
        count++;
    }
    if (tri2 != 0) {
        Triangle_CreateWithRotatedUVs(tri2, edge_type, 0, 0);
        count++;
    }

    return count;
}
```

### Edge Pattern Lookup

The adjacency byte maps to rotation and edge type:

| Adjacency | Pattern | Rotation | Edge Type |
|-----------|---------|----------|-----------|
| 0x03 | NE corner | 3 | 6 (corner) |
| 0x05 | N+S edges | 1 | 2 (horizontal) |
| 0x06 | SE corner | 0 | 6 (corner) |
| 0x07 | NES | 1 | 3 (three-way) |
| 0x09 | NW corner | 2 | 6 (corner) |
| 0x0A | E+W edges | 0 | 2 (vertical) |
| 0x0B | NEW | 0 | 3 (three-way) |
| 0x0C | SW corner | 1 | 6 (corner) |
| 0x0D | NWS | 3 | 3 (three-way) |
| 0x0E | SWE | 2 | 3 (three-way) |
| 0x0F | All | - | 7 (island) |

---

## Appendix DJ: Heightmap and Terrain Height Interpolation

### Height Lookup

**Function:** `Terrain_GetHeightAtPoint @ 0x004e8e50`

**Global Data:**
- `g_Heightmap` - 256×256 height values (shorts)
- `g_CellFlags` - Per-cell flags affecting triangle split direction

**Height Interpolation Algorithm:**

The function performs bilinear interpolation across terrain cells, respecting the cell's triangle split direction.

```c
int Terrain_GetHeightAtPoint(ushort x, ushort z) {
    // Get cell indices (each cell is 256 units)
    uint cell_x = (x >> 8) & 0xFF;
    uint cell_z = (z >> 8) & 0xFF;
    uint frac_x = (x & 0x1FE) >> 1;  // Sub-cell fraction [0-255]
    uint frac_z = (z & 0x1FE) >> 1;

    // Get 4 corner heights
    short h00 = g_Heightmap[cell_z * 256 + cell_x];           // Top-left
    short h10 = g_Heightmap[cell_z * 256 + cell_x + 1];       // Top-right
    short h01 = g_Heightmap[(cell_z + 1) * 256 + cell_x];     // Bottom-left
    short h11 = g_Heightmap[(cell_z + 1) * 256 + cell_x + 1]; // Bottom-right

    // Check cell flag bit 0 for split direction
    if ((g_CellFlags[cell_index] & 1) == 0) {
        // Split: top-left to bottom-right diagonal
        if (frac_z < frac_x) {
            // Upper-right triangle
            return h00 + ((h10 - h00) * frac_x >> 8)
                       + ((h11 - h10) * frac_z >> 8);
        } else {
            // Lower-left triangle
            return h00 + ((h11 - h01) * frac_x >> 8)
                       + ((h01 - h00) * frac_z >> 8);
        }
    } else {
        // Split: top-right to bottom-left diagonal
        if (frac_x + frac_z < 256) {
            // Upper-left triangle
            return h00 + ((h10 - h00) * frac_x >> 8)
                       + ((h01 - h00) * frac_z >> 8);
        } else {
            // Lower-right triangle
            return h11 + ((h01 - h11) * (256 - frac_x) >> 8)
                       + ((h10 - h11) * (256 - frac_z) >> 8);
        }
    }
}
```

### Cell Flags

**Function:** `Cell_UpdateFlags @ 0x004e9fe0`

**Flag Bits:**
| Bit | Mask | Purpose |
|-----|------|---------|
| 0 | 0x01 | Triangle split direction (0=NW-SE, 1=NE-SW) |
| 1 | 0x02 | Has blocking object |
| 19 | 0x80000 | Has impassable structure |

---

