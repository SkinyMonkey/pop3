# Camera Test Suite

Script-driven tests that replay key sequences through the `--script` flag and validate the resulting camera state log with `check_camera.py`.

## How it works

1. The `--script <file>` flag feeds commands from a text file instead of keyboard input (one command per line).
2. Every state-changing event writes a JSON line to `/tmp/faithful_debug.jsonl` with: timestamp, angles, zoom, radius, eye position, focus, shift, min_z.
3. `check_camera.py` reads the JSONL and runs 12 invariant checks against it.

## Running

Single test:

```bash
cargo run --release -- --base '<pop_path>' --level 1 --script scripts/test_camera.txt
python3 scripts/check_camera.py /tmp/faithful_debug.jsonl
```

All tests:

```bash
bash scripts/run_all_tests.sh '<pop_path>'
```

## Script format

- One command per line
- Blank lines and `#` comments are ignored
- Key names: `W`, `S`, `A`, `D`, `Q`, `E`, `ArrowUp`, `ArrowDown`, `R`, `T`, `Space`
- Zoom: `zoom <float>` (e.g. `zoom 2.5`)

## Test scripts

### `test_camera.txt` -- General sanity

Basic coverage of all controls: pan (WASD), rotate (Q/E), tilt (arrows), reset (R), center (Space), zoom, and a full 360 rotation. This is the baseline test that exercises every input type at least once.

### `test_angle_clamping.txt` -- Tilt boundaries

Verifies `angle_x` is clamped to [-90, -30]:
- From default -55, presses ArrowUp 10 times (would reach -5 unclamped, must stop at -30)
- Then ArrowDown 20 times (would reach -130, must stop at -90)
- Then ArrowUp 12 back to -30, plus 5 more at the boundary (must stay -30)
- Then ArrowDown 15 to -90, plus 3 more at the boundary (must stay -90)

Catches: off-by-one in clamp logic, clamp applied after increment vs before.

### `test_wasd_quadrants.txt` -- Pan direction at cardinal angles

Tests WASD at angle_z = 0, 90, 180, 270 (reset, then 18 E presses per quadrant). At each angle, presses W/S/A/D and the check verifies the shift delta matches the rotation formula:

```
gx = -dx*cos(az) - dy*sin(az)
gy =  dx*sin(az) - dy*cos(az)
```

Catches: wrong sin/cos signs, swapped axes, direction inversion at specific quadrants.

### `test_zoom_clamp.txt` -- Zoom boundary enforcement

- `zoom 0.1` (below min 0.3, must clamp)
- Pan and rotate at min zoom to verify radius is correct
- `zoom 10.0` (above max 5.0, must clamp)
- Pan and rotate at max zoom
- Tests exact boundaries: `zoom 0.3`, `zoom 5.0`
- Restores `zoom 1.0`

Catches: unclamped zoom affecting orbit radius, division by zero at extreme zoom.

### `test_shift_cancel.txt` -- Opposite directions cancel

At angle_z = 0, 90, and 135:
- W x5 then S x5 (shift must return to original)
- A x5 then D x5 (shift must return to original)

Catches: asymmetric rounding in shift deltas, toroidal wrapping errors, direction formula producing different magnitudes for opposite keys.

### `test_full_rotation.txt` -- 360 rotation invariant

- Pans to a non-zero shift, then:
  - Q x72 (360 clockwise) -- eye XY must return to start
  - E x72 (360 counter-clockwise) -- eye XY must return to start
  - E x144 (720) -- tests large accumulated angle_z values

Catches: floating-point drift in eye position after full rotation, trig errors with large angle values.

### `test_top_view.txt` -- Top-down view (T key)

- T sets angle_x = -90 directly
- Pan with WASD at top-down (cos(-90) ~ 0, so orbit radius XY ~ 0)
- Rotate with Q at top-down
- ArrowUp x6 to recover from top-down back toward -30
- Pan again at recovered angle

Catches: division by zero or degenerate geometry when looking straight down, inability to recover from top-down via arrow keys.

### `test_combined_stress.txt` -- Comprehensive stress test

1379 commands across 17 phases, each targeting a specific bug class:

| Phase | Commands | What it stresses |
|-------|----------|-----------------|
| 1 | Basic pan + rotate | Sanity baseline |
| 2 | ArrowUp to -30 clamp + 40 pans | Terrain clipping at shallow tilt |
| 3 | ArrowDown interleaved with WASD | State changes mid-tilt transition |
| 4 | T then immediate mixed ops | Top-down edge cases |
| 5 | Pan at 15, 35, 50, 85 degrees | Trig at non-axis-aligned angles |
| 6 | WASD at exactly 90, 180, 270, 360 | cos/sin = exactly 0 or 1 |
| 7 | 6 zoom levels with rotate + pan | Zoom affecting orbit radius and direction |
| 8 | 16x rapid W/S, A/D, Q/E, Up/Down | Jitter from rapid alternation |
| 9 | All 4 WASD + Q/E interleaved | Rapid mixed state transitions |
| 10 | Reset/Space spam between moves | State reset mid-operation |
| 11 | 20 pans + max zoom + shallow tilt | Worst-case terrain clipping |
| 12 | Q from 0 into negative angles | Negative angle_z in trig |
| 13 | 72 E (360) + 78 E (angle_z = 750) | Large accumulated angle values |
| 14 | T + zoom + gradual ArrowUp recovery | Combined boundary conditions |
| 15 | 130 W + 130 D (wraps 128 grid) | Toroidal shift modular arithmetic |
| 16 | Rotate + tilt + pan + zoom every step | All parameters changing simultaneously |
| 17 | W x10/S x10 cancel + Q x72 rotation | Final invariant verification |

## Validation checks (`check_camera.py`)

All 12 checks run against every test script. A check reports FAIL with the first 10 offending log lines.

| # | Check | What it verifies |
|---|-------|-----------------|
| 1 | **Orbit radius consistency** | Eye XY distance to focus matches `radius * cos(angle_x)` within 0.01 |
| 2 | **Eye Z above min_z** | Eye never goes below terrain surface + 0.05 margin |
| 3 | **Angle increments** | Q/E change angle_z by exactly 5; ArrowUp/Down change angle_x by 5 (with clamp) |
| 4 | **WASD shift direction** | Shift delta matches the rotation formula, accounting for toroidal wrapping at 128 |
| 5 | **Focus constant** | Focus point never changes (orbit camera invariant) |
| 6 | **Zoom bounds** | Zoom stays within [0.3, 5.0] at all times |
| 7 | **Reset state** | After R, angle_x = -55 and angle_z = 0 |
| 8 | **Eye Z = max(orbit, min_z)** | Eye Z equals the larger of the orbit height and the terrain min_z |
| 9 | **Angle X clamping** | angle_x stays within [-90, -30] at all times |
| 10 | **Shift cancellation** | N forward presses followed by N backward presses return shift to original |
| 11 | **Full rotation** | 72 consecutive Q or E presses return eye XY to starting position within 0.02 |
| 12 | **Zoom clamping** | Zoom events with out-of-range values produce clamped results |
