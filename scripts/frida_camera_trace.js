"use strict";

// Pop3 Camera Flyby Tracer (ASLR-aware, Wine-compatible)
// Attach: frida popTB.exe -l frida_camera_trace.js

var LOG_PATH = "camera_trace.log";
var frameCount = 0;

var logFile = new File(LOG_PATH, "w");

function logLine(s) {
    logFile.write(s + "\n");
    logFile.flush();
}

// ── ASLR-aware address computation ──────────────────────────────────

var DECOMP_BASE = 0x00400000;

// Try multiple ways to find the module base
var BASE = null;
var moduleName = "?";

try {
    var mod = Process.getModuleByName("popTB.exe");
    BASE = mod.base;
    moduleName = mod.name;
} catch(e1) {
    logLine("popTB.exe not found: " + e1);
    try {
        var mod = Process.getModuleByName("POPTB.EXE");
        BASE = mod.base;
        moduleName = mod.name;
    } catch(e2) {
        logLine("POPTB.EXE not found: " + e2);
        var modules = Process.enumerateModules();
        for (var i = 0; i < modules.length; i++) {
            var n = modules[i].name.toLowerCase();
            if (n.indexOf("pop") >= 0 || n.indexOf("poptb") >= 0) {
                BASE = modules[i].base;
                moduleName = modules[i].name;
                break;
            }
        }
    }
}

if (!BASE) {
    // Assume no ASLR for old Win32 exe
    BASE = ptr(DECOMP_BASE);
}

function va(decompiled_addr) {
    return BASE.add(decompiled_addr - DECOMP_BASE);
}

// ── Memory read helpers (use NativePointer methods directly) ─────────

function readI16(p) { return p.readS16(); }
function readU16(p) { return p.readU16(); }
function readI32(p) { return p.readS32(); }
function readU8(p)  { return p.readU8(); }

// ── Key global addresses (ASLR-adjusted) ────────────────────────────

var pFLYBY_BUF_X      = va(0x67c280);
var pFLYBY_BUF_Y      = va(0x67c282);
var pFLYBY_BUF_ANGLE  = va(0x67c284);
var pFLYBY_BUF_ZOOM   = va(0x67c27c);
var pFLYBY_FLAGS      = va(0x67c288);
var pFLYBY_ZOOM_DIR   = va(0x67c289);
var pFLYBY_ZOOM_DELAY = va(0x67c286);
var pSCROLL_ACC_X     = va(0x67c26c);
var pSCROLL_ACC_Y     = va(0x67c270);
var pSCROLL_ACC_ANGLE = va(0x67c274);
var pSCROLL_ACC_ZOOM  = va(0x67c278);
var pCAM_STATE_FLAGS  = va(0x67c2c6);
var pCAM_SCRIPT_CMD   = va(0x67c1e6);
var pCAM_SCRIPT_PARAM = va(0x67c1e7);
var pPLAYER_INDEX     = va(0x884c88);
var pCAM_STRUCT_BASE  = va(0x885760);

// ── Diagnostic: test memory read ────────────────────────────────────

logLine("=== Pop3 Camera Flyby Tracer v3 ===");
logLine("Module: " + moduleName + " base=" + BASE + " size=" + (mod ? mod.size : "?"));

// Raw memory read test
var testP = ptr(0x00884c88);
logLine("Raw read test at 0x00884c88 (NativePointer)...");
try {
    var v = testP.readU8();
    logLine("  readU8 = " + v + " SUCCESS");
} catch(e2) {
    logLine("  readU8 FAILED: " + e2.message);
}

var testP2 = BASE.add(0x00884c88 - DECOMP_BASE);
logLine("Offset read test at " + testP2 + " (BASE.add)...");
try {
    var v2 = testP2.readU8();
    logLine("  readU8 = " + v2 + " SUCCESS");
} catch(e3) {
    logLine("  readU8 FAILED: " + e3.message);
}

// Try Memory.readU8 as alternative
try {
    var v3 = ptr(0x00884c88).readU8();
    logLine("  ptr.readU8 alt = " + v3 + " SUCCESS");
} catch(e4) {
    logLine("  ptr.readU8 alt FAILED: " + e4.message);
}

logLine("pFLYBY_BUF_X=" + pFLYBY_BUF_X);
logLine("pPLAYER_INDEX=" + pPLAYER_INDEX);

// ── Camera struct access ────────────────────────────────────────────

function getCameraStructAddr() {
    var idx = readU8(pPLAYER_INDEX);
    var offset = idx * 3173;
    return pCAM_STRUCT_BASE.add(offset);
}

function readCameraStruct(label) {
    try {
        var cam = getCameraStructAddr();
        var s = label + "=" +
            "pos_x=" + readI16(cam.add(0x24)) +
            " pos_y=" + readI16(cam.add(0x26)) +
            " angle_z=" + readU16(cam.add(0x32)) +
            " zoom=" + readI16(cam.add(0x34));
        try { s += " follow_ptr=" + cam.add(0x89d).readPointer(); } catch(e) {}
        return s;
    } catch(e) {
        return label + "=ERR(" + e.message + ")";
    }
}

function readFlybyBuffer(label) {
    try {
        return label + "=" +
            "x=" + readI16(pFLYBY_BUF_X) +
            " y=" + readI16(pFLYBY_BUF_Y) +
            " angle=" + readU16(pFLYBY_BUF_ANGLE) +
            " zoom_acc=" + readI32(pFLYBY_BUF_ZOOM) +
            " flags=0x" + readU8(pFLYBY_FLAGS).toString(16) +
            " zoom_dir=" + readU8(pFLYBY_ZOOM_DIR) +
            " zoom_delay=" + readI16(pFLYBY_ZOOM_DELAY);
    } catch(e) {
        return label + "=ERR(" + e.message + ")";
    }
}

function readScrollAccum(label) {
    try {
        return label + "=" +
            "acc_x=" + readI32(pSCROLL_ACC_X) +
            " acc_y=" + readI32(pSCROLL_ACC_Y) +
            " acc_angle=" + readI32(pSCROLL_ACC_ANGLE) +
            " acc_zoom=" + readI32(pSCROLL_ACC_ZOOM);
    } catch(e) {
        return label + "=ERR(" + e.message + ")";
    }
}

// ── Hook: FUN_00457380 — Init flyby buffer from camera ──────────────

Interceptor.attach(va(0x00457380), {
    onEnter: function(args) {
        try {
            var cam = getCameraStructAddr();
            this.pre_cam_x = readI16(cam.add(0x24));
            this.pre_cam_y = readI16(cam.add(0x26));
            this.pre_cam_angle = readU16(cam.add(0x32));
            this.pre_cam_zoom = readI16(cam.add(0x34));
        } catch(e) {
            this.pre_cam_x = 0; this.pre_cam_y = 0;
            this.pre_cam_angle = 0; this.pre_cam_zoom = 0;
        }
    },
    onLeave: function(retval) {
        var s = "[" + frameCount + "] FLYBY_INIT:" +
            " cam_x=" + this.pre_cam_x +
            " cam_y=" + this.pre_cam_y +
            " cam_angle=" + this.pre_cam_angle +
            " cam_zoom=" + this.pre_cam_zoom +
            " ->" +
            " buf_x=" + readI16(pFLYBY_BUF_X) +
            " buf_y=" + readI16(pFLYBY_BUF_Y) +
            " buf_angle=" + readU16(pFLYBY_BUF_ANGLE) +
            " buf_zoom_acc=" + readI32(pFLYBY_BUF_ZOOM);
        logLine(s);
    }
});

// ── Hook: Tick_ProcessCameraScript ──────────────────────────────────

Interceptor.attach(va(0x00456c00), {
    onEnter: function(args) {
        this.cmd = readU8(pCAM_SCRIPT_CMD);
        this.param = readU8(pCAM_SCRIPT_PARAM);
        this.pre_flyby = readFlybyBuffer("pre");
    },
    onLeave: function(retval) {
        var s = "[" + frameCount + "] CAM_SCRIPT:" +
            " cmd=" + this.cmd +
            " param=" + this.param +
            " " + this.pre_flyby +
            " ->" +
            " " + readFlybyBuffer("post");
        if (this.cmd === 10) {
            s += " [CASE_9_FLYBY]";
            s += " " + readScrollAccum("scroll");
            s += " " + readCameraStruct("cam");
        }
        if (this.cmd === 11) {
            s += " [CASE_10_C2BE]";
        }
        logLine(s);
    }
});

// ── Hook: Tick_UpdateCameraMotion ───────────────────────────────────

Interceptor.attach(va(0x00456fd0), {
    onEnter: function(args) {
        try {
            this.pre_cam = readCameraStruct("pre_cam");
            this.pre_buf = readFlybyBuffer("pre_buf");
            this.pre_scroll = readScrollAccum("pre_scroll");
            this.pre_state = "state=0x" + readU16(pCAM_STATE_FLAGS).toString(16);
        } catch(e) {
            this.pre_cam = "?"; this.pre_buf = "?";
            this.pre_scroll = "?"; this.pre_state = "?";
        }
    },
    onLeave: function(retval) {
        frameCount++;
        var s = "[" + frameCount + "] CAM_MOTION:" +
            " " + this.pre_cam +
            " " + this.pre_buf +
            " " + this.pre_scroll +
            " " + this.pre_state +
            " ->" +
            " " + readCameraStruct("post_cam") +
            " " + readFlybyBuffer("post_buf") +
            " " + readScrollAccum("post_scroll");
        logLine(s);
    }
});

// ── Hook: FUN_004245c0 — Init camera from shaman position ───────────

Interceptor.attach(va(0x004245c0), {
    onEnter: function(args) {
        this.camPtr = args[0];
        try {
            var personPtr = this.camPtr.add(0x89d).readPointer();
            if (personPtr && !personPtr.isNull()) {
                var raw = "";
                for (var i = 0; i < 8; i++) {
                    var b = personPtr.add(0x3d + i).readU8();
                    raw += " " + (b < 16 ? "0" : "") + b.toString(16);
                }
                this.person_raw = raw;
                this.person_x = readI16(personPtr.add(0x3d));
                this.person_y = readI16(personPtr.add(0x3f));
            } else {
                this.person_raw = "null";
                this.person_x = -1;
                this.person_y = -1;
            }
            this.pre_cam_x = readI16(this.camPtr.add(0x24));
            this.pre_cam_y = readI16(this.camPtr.add(0x26));
        } catch(e) {
            this.person_raw = "err:" + e.message;
            this.person_x = -1;
            this.person_y = -1;
            this.pre_cam_x = -1;
            this.pre_cam_y = -1;
        }
    },
    onLeave: function(retval) {
        try {
            var post_x = readI16(this.camPtr.add(0x24));
            var post_y = readI16(this.camPtr.add(0x26));
            var post_angle = readU16(this.camPtr.add(0x32));
            logLine("[" + frameCount + "] CAM_INIT_SHAMAN:" +
                " person_bytes=" + this.person_raw +
                " person_x=" + this.person_x +
                " person_y=" + this.person_y +
                " pre:cam_x=" + this.pre_cam_x + " cam_y=" + this.pre_cam_y +
                " post:cam_x=" + post_x + " cam_y=" + post_y + " angle=" + post_angle);
        } catch(e) {
            logLine("[" + frameCount + "] CAM_INIT_SHAMAN: onLeave err: " + e.message);
        }
    }
});

// ── Hook: Math_MovePointByAngle ──────────────────────────────────────

Interceptor.attach(va(0x004d4b20), {
    onEnter: function(args) {
        this.pointPtr = args[0];
        this.angle = args[1].toInt32() & 0xFFFF;
        this.distance = args[2].toInt32();
        try {
            this.pre_x = readI16(this.pointPtr);
            this.pre_y = readI16(this.pointPtr.add(2));
        } catch(e) {
            this.pre_x = 0; this.pre_y = 0;
        }
    },
    onLeave: function(retval) {
        try {
            var post_x = readI16(this.pointPtr);
            var post_y = readI16(this.pointPtr.add(2));
            logLine("[" + frameCount + "] MOVE_BY_ANGLE:" +
                " pre=(" + this.pre_x + "," + this.pre_y + ")" +
                " angle=" + this.angle +
                " dist=" + this.distance +
                " post=(" + post_x + "," + post_y + ")" +
                " dx=" + (post_x - this.pre_x) +
                " dy=" + (post_y - this.pre_y));
        } catch(e) {}
    }
});

// ── Hook: Game_RestartOrReplay ──────────────────────────────────────

Interceptor.attach(va(0x004562c0), {
    onEnter: function(args) {
        logLine("[" + frameCount + "] RESTART_OR_REPLAY:" +
            " param1=" + args[0].toInt32() +
            " param2=" + args[1].toInt32() +
            " " + readCameraStruct("cam") +
            " " + readFlybyBuffer("buf") +
            " " + readScrollAccum("scroll") +
            " state=0x" + readU16(pCAM_STATE_FLAGS).toString(16));
    }
});

// ── Hook: Input_ApplyCameraMovement ──────────────────────────────────

Interceptor.attach(va(0x004ab6e0), {
    onEnter: function(args) {
        logLine("[" + frameCount + "] INPUT_CAM_MOVE:" +
            " " + readCameraStruct("cam") +
            " " + readFlybyBuffer("buf") +
            " state=0x" + readU16(pCAM_STATE_FLAGS).toString(16));
    }
});

// ── Hook: Camera_Initialize ─────────────────────────────────────────

Interceptor.attach(va(0x00422130), {
    onEnter: function(args) {
        logLine("[" + frameCount + "] CAMERA_INITIALIZE:" +
            " " + readCameraStruct("cam_before") +
            " " + readFlybyBuffer("buf"));
    }
});

// ── Periodic full state dump ≈ every 1s ─────────────────────────────

setInterval(function() {
    try {
        logLine("[" + frameCount + "] PERIODIC:" +
            " " + readCameraStruct("cam") +
            " " + readFlybyBuffer("buf") +
            " " + readScrollAccum("scroll") +
            " state=0x" + readU16(pCAM_STATE_FLAGS).toString(16) +
            " script_cmd=" + readU8(pCAM_SCRIPT_CMD) +
            " player_idx=" + readU8(pPLAYER_INDEX));
    } catch(e) {
        logLine("[" + frameCount + "] PERIODIC: ERR " + e.message);
    }
}, 1000);

// ── Startup diagnostic ──────────────────────────────────────────────

logLine("Test read player_index=" + readU8(pPLAYER_INDEX));
logLine("Test read flyby_buf_x=" + readI16(pFLYBY_BUF_X));
logLine("Test read state_flags=" + readU16(pCAM_STATE_FLAGS).toString(16));
try { logLine("Camera struct: " + readCameraStruct("init")); } catch(e) { logLine("Camera struct: ERR " + e); }
try { logLine("Flyby buffer: " + readFlybyBuffer("init")); } catch(e) { logLine("Flyby buffer: ERR " + e); }
logLine("=========================================");