#!/usr/bin/env python3
"""Pop3 Camera Flyby Tracer — Python host.
Usage:
    python3 frida_camera_trace_host.py <pid_or_name>

Examples:
    python3 frida_camera_trace_host.py popTB.exe
    python3 frida_camera_trace_host.py 1234

Output: camera_trace.log in the current directory.
"""

import sys
import frida

LOG_PATH = "camera_trace.log"

def main():
    target = sys.argv[1] if len(sys.argv) > 1 else "popTB.exe"
    outfile = open(LOG_PATH, "w", buffering=1)  # line-buffered

    def on_message(message, data):
        if message["type"] == "send":
            outfile.write(message["payload"] + "\n")
        elif message["type"] == "error":
            outfile.write("ERROR: " + str(message.get("description", message)) + "\n")

    try:
        session = frida.attach(target)
    except frida.ProcessNotFoundError:
        print(f"Process '{target}' not found. Is the game running?")
        sys.exit(1)

    with open("frida_camera_trace.js") as f:
        script_code = f.read()

    script = session.create_script(script_code)
    script.on("message", on_message)
    script.load()

    print(f"Tracing '{target}' → {LOG_PATH}. Press Ctrl+C to stop.")
    try:
        sys.stdin.read()
    except KeyboardInterrupt:
        pass

    script.unload()
    session.detach()
    outfile.close()
    print(f"Log saved to {LOG_PATH}")

if __name__ == "__main__":
    main()