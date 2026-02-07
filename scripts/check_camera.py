#!/usr/bin/env python3
"""Validate camera debug JSONL output for coherence."""

import json
import math
import sys

def round_half_away_from_zero(x):
    """Match Rust's f32::round() which rounds half away from zero (not Python's banker's rounding)."""
    if x >= 0:
        return int(math.floor(x + 0.5))
    else:
        return int(math.ceil(x - 0.5))

def load_entries(path):
    entries = []
    with open(path) as f:
        for i, line in enumerate(f, 1):
            line = line.strip()
            if not line:
                continue
            try:
                entries.append(json.loads(line))
            except json.JSONDecodeError as e:
                print(f"FAIL: line {i}: invalid JSON: {e}")
                sys.exit(1)
    return entries

def check_orbit_radius(entries):
    """Eye XY distance to focus should match radius (within tolerance)."""
    errors = []
    for i, e in enumerate(entries):
        ex, ey, _ = e["eye"]
        fx, fy = e["focus"][0], e["focus"][1]
        r_actual = math.sqrt((ex - fx)**2 + (ey - fy)**2)
        r_expected = e["radius"] * math.cos(math.radians(e["angle_x"]))
        if abs(r_actual - r_expected) > 0.01:
            errors.append(f"  line {i+1}: orbit radius XY={r_actual:.4f} expected={r_expected:.4f} (event={e['event']})")
    return errors

def check_eye_above_min_z(entries):
    """Eye Z should always be >= min_z."""
    errors = []
    for i, e in enumerate(entries):
        ez = e["eye"][2]
        mz = e["min_z"]
        if ez < mz - 0.001:
            errors.append(f"  line {i+1}: eye_z={ez:.4f} < min_z={mz:.4f} (event={e['event']})")
    return errors

def check_angle_increments(entries):
    """Q/E should change angle_z by exactly 5, ArrowUp/Down angle_x by 5."""
    errors = []
    for i in range(1, len(entries)):
        prev, cur = entries[i-1], entries[i]
        ev = cur["event"]

        if ev == "KeyQ":
            expected_z = prev["angle_z"] - 5
            if cur["angle_z"] != expected_z:
                errors.append(f"  line {i+1}: Q: angle_z={cur['angle_z']} expected={expected_z}")
        elif ev == "KeyE":
            expected_z = prev["angle_z"] + 5
            if cur["angle_z"] != expected_z:
                errors.append(f"  line {i+1}: E: angle_z={cur['angle_z']} expected={expected_z}")
        elif ev == "ArrowUp":
            expected_x = min(prev["angle_x"] + 5, -30)
            if cur["angle_x"] != expected_x:
                errors.append(f"  line {i+1}: ArrowUp: angle_x={cur['angle_x']} expected={expected_x}")
        elif ev == "ArrowDown":
            expected_x = max(prev["angle_x"] - 5, -90)
            if cur["angle_x"] != expected_x:
                errors.append(f"  line {i+1}: ArrowDown: angle_x={cur['angle_x']} expected={expected_x}")

    return errors

def check_wasd_shift_direction(entries):
    """WASD shift changes should match the corrected rotation formula."""
    errors = []
    for i in range(1, len(entries)):
        prev, cur = entries[i-1], entries[i]
        ev = cur["event"]

        key_map = {"KeyW": (0, 1), "KeyS": (0, -1), "KeyA": (-1, 0), "KeyD": (1, 0)}
        if ev not in key_map:
            continue

        dx, dy = key_map[ev]
        az = math.radians(prev["angle_z"])
        gx = -dx * math.cos(az) - dy * math.sin(az)
        gy =  dx * math.sin(az) - dy * math.cos(az)
        expected_sx = round_half_away_from_zero(gx)
        expected_sy = round_half_away_from_zero(gy)

        actual_dsx = cur["shift"][0] - prev["shift"][0]
        actual_dsy = cur["shift"][1] - prev["shift"][1]

        # Handle toroidal wrapping (mod 128)
        if actual_dsx > 64: actual_dsx -= 128
        if actual_dsx < -64: actual_dsx += 128
        if actual_dsy > 64: actual_dsy -= 128
        if actual_dsy < -64: actual_dsy += 128

        if actual_dsx != expected_sx or actual_dsy != expected_sy:
            errors.append(
                f"  line {i+1}: {ev} at angle_z={prev['angle_z']}: "
                f"shift delta=({actual_dsx},{actual_dsy}) expected=({expected_sx},{expected_sy})"
            )

    return errors

def check_focus_constant(entries):
    """Focus point should remain constant across all entries."""
    if not entries:
        return []
    errors = []
    ref_focus = entries[0]["focus"]
    for i, e in enumerate(entries):
        if e["focus"] != ref_focus:
            errors.append(f"  line {i+1}: focus={e['focus']} expected={ref_focus}")
    return errors

def check_zoom_bounds(entries):
    """Zoom should stay within [0.3, 5.0]."""
    errors = []
    for i, e in enumerate(entries):
        z = e["zoom"]
        if z < 0.3 - 0.001 or z > 5.0 + 0.001:
            errors.append(f"  line {i+1}: zoom={z} out of bounds [0.3, 5.0]")
    return errors

def check_reset_state(entries):
    """After a 'reset' event, camera should be at default angles."""
    errors = []
    for i, e in enumerate(entries):
        if e["event"] == "reset":
            if e["angle_x"] != -55:
                errors.append(f"  line {i+1}: reset: angle_x={e['angle_x']} expected=-55")
            if e["angle_z"] != 0:
                errors.append(f"  line {i+1}: reset: angle_z={e['angle_z']} expected=0")
    return errors

def check_eye_z_consistency(entries):
    """eye_z should equal max(eye_z_orbit, min_z)."""
    errors = []
    for i, e in enumerate(entries):
        expected_z = max(e["eye_z_orbit"], e["min_z"])
        if abs(e["eye"][2] - expected_z) > 0.001:
            errors.append(
                f"  line {i+1}: eye_z={e['eye'][2]:.4f} expected max(orbit={e['eye_z_orbit']:.4f}, min_z={e['min_z']:.4f})={expected_z:.4f}"
            )
    return errors

def check_angle_x_clamping(entries):
    """angle_x should always be within [-90, -30]."""
    errors = []
    for i, e in enumerate(entries):
        ax = e["angle_x"]
        if ax < -90 or ax > -30:
            errors.append(f"  line {i+1}: angle_x={ax} out of bounds [-90, -30] (event={e['event']})")
    return errors

def check_shift_cancellation(entries):
    """Equal W+S or A+D presses should return shift to original value."""
    errors = []
    # Find sequences of consecutive same-axis keypresses that should cancel
    # Look for patterns: N forward presses followed by N backward presses
    i = 0
    while i < len(entries):
        e = entries[i]
        ev = e["event"]
        if ev not in ("KeyW", "KeyS", "KeyA", "KeyD"):
            i += 1
            continue
        # Determine the inverse key
        inverse = {"KeyW": "KeyS", "KeyS": "KeyW", "KeyA": "KeyD", "KeyD": "KeyA"}
        inv = inverse[ev]
        # Count consecutive presses of this key
        count = 0
        j = i
        while j < len(entries) and entries[j]["event"] == ev:
            count += 1
            j += 1
        # Check if followed by exactly `count` inverse presses
        inv_count = 0
        k = j
        while k < len(entries) and entries[k]["event"] == inv:
            inv_count += 1
            k += 1
        if inv_count == count and count >= 3:
            # Shift before first press should match shift after last inverse press
            shift_before = entries[i]["shift"]  # first entry is after the press, so use i-1
            if i > 0:
                shift_before = entries[i - 1]["shift"]
            shift_after = entries[k - 1]["shift"]
            if shift_before[0] != shift_after[0] or shift_before[1] != shift_after[1]:
                errors.append(
                    f"  lines {i+1}-{k}: {ev}x{count} + {inv}x{count} at angle_z={entries[i]['angle_z']}: "
                    f"shift before={shift_before} after={shift_after} (should match)"
                )
            i = k
        else:
            i = j
    return errors

def check_full_rotation(entries):
    """After 72 consecutive Q or E presses, eye XY should return near start."""
    errors = []
    i = 0
    while i < len(entries):
        e = entries[i]
        ev = e["event"]
        if ev not in ("KeyQ", "KeyE"):
            i += 1
            continue
        # Count consecutive presses
        count = 0
        j = i
        while j < len(entries) and entries[j]["event"] == ev:
            count += 1
            j += 1
        if count >= 72 and count % 72 == 0:
            # Eye XY before first press should match eye XY after last press
            if i > 0:
                eye_before = entries[i - 1]["eye"]
            else:
                eye_before = entries[i]["eye"]
            eye_after = entries[j - 1]["eye"]
            dx = abs(eye_before[0] - eye_after[0])
            dy = abs(eye_before[1] - eye_after[1])
            if dx > 0.02 or dy > 0.02:
                errors.append(
                    f"  lines {i+1}-{j}: {ev}x{count} (full rotation): "
                    f"eye XY before=({eye_before[0]:.4f},{eye_before[1]:.4f}) "
                    f"after=({eye_after[0]:.4f},{eye_after[1]:.4f}) delta=({dx:.4f},{dy:.4f})"
                )
        i = j
    return errors

def check_zoom_clamping(entries):
    """After zoom events, zoom should be clamped to [0.3, 5.0]."""
    errors = []
    for i, e in enumerate(entries):
        if e["event"].startswith("zoom"):
            z = e["zoom"]
            if z < 0.3 - 0.001 or z > 5.0 + 0.001:
                errors.append(f"  line {i+1}: zoom event resulted in zoom={z}, should be clamped to [0.3, 5.0]")
    return errors

def main():
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <jsonl_file>")
        sys.exit(1)

    entries = load_entries(sys.argv[1])
    if not entries:
        print("FAIL: no entries in log file")
        sys.exit(1)

    print(f"Loaded {len(entries)} entries")

    checks = [
        ("Orbit radius consistency", check_orbit_radius),
        ("Eye Z above min_z", check_eye_above_min_z),
        ("Angle increments (Q/E/arrows)", check_angle_increments),
        ("WASD shift direction", check_wasd_shift_direction),
        ("Focus constant", check_focus_constant),
        ("Zoom bounds", check_zoom_bounds),
        ("Reset state", check_reset_state),
        ("Eye Z = max(orbit, min_z)", check_eye_z_consistency),
        ("Angle X clamping [-90, -30]", check_angle_x_clamping),
        ("Shift cancellation (W+S, A+D)", check_shift_cancellation),
        ("Full rotation (72 Q/E)", check_full_rotation),
        ("Zoom clamping after zoom events", check_zoom_clamping),
    ]

    all_ok = True
    for name, fn in checks:
        errors = fn(entries)
        if errors:
            print(f"FAIL: {name}")
            for e in errors[:10]:  # cap output
                print(e)
            if len(errors) > 10:
                print(f"  ... and {len(errors) - 10} more")
            all_ok = False
        else:
            print(f"OK:   {name}")

    if all_ok:
        print("\nAll checks passed.")
        sys.exit(0)
    else:
        print("\nSome checks failed.")
        sys.exit(1)

if __name__ == "__main__":
    main()
