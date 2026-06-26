#!/usr/bin/env python3
"""Fetch all PopScript documentation pages from populous3.info and extract functions."""

import re
from pathlib import Path
import urllib.request

BASE_URL = "https://www.populous3.info/script3_doc/"
DOCS_DIR = Path(__file__).parent / "popscript_docs"

# All module files from files.html - URL format uses _module___name_8h.html pattern
MODULE_FILES = {
    "LbColour.h": "_lb_colour_8h.html",
    "Module_Commands.h": "_module___commands_8h.html",
    "Module_Control.h": "_module___control_8h.html",
    "Module_DataTypes.h": "_module___data_types_8h.html",
    "Module_Defines.h": "_module___defines_8h.html",
    "Module_Draw.h": "_module___draw_8h.html",
    "Module_Game.h": "_module___game_8h.html",
    "Module_GameStates.h": "_module___game_states_8h.html",
    "Module_Globals.h": "_module___globals_8h.html",
    "Module_Helpers.h": "_module___helpers_8h.html",
    "Module_Level.h": "_module___level_8h.html",
    "Module_Map.h": "_module___map_8h.html",
    "Module_MapWho.h": "_module___map_who_8h.html",
    "Module_Objects.h": "_module___objects_8h.html",
    "Module_Person.h": "_module___person_8h.html",
    "Module_Players.h": "_module___players_8h.html",
    "Module_PopScript.h": "_module___pop_script_8h.html",
    "Module_Sound.h": "_module___sound_8h.html",
    "Module_StringTools.h": "_module___string_tools_8h.html",
    "Module_System.h": "_module___system_8h.html",
    "No_Module.h": "_no___module_8h.html",
    "ObjectList.h": "_object_list_8h.html",
    "ObjectProxy.h": "_object_proxy_8h.html",
    "Pop3Keys.h": "_pop3_keys_8h.html",
    "script3_profiler.h": "script3__profiler_8h.html",
    "TbRect.h": "_tb_rect_8h.html",
}

def fetch_file(display_name: str, url_filename: str) -> str | None:
    """Fetch a single HTML file."""
    url = BASE_URL + url_filename
    print(f"Fetching {display_name}...")
    req = urllib.request.Request(url, headers={
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'
    })
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            return resp.read().decode('utf-8')
    except Exception as e:
        print(f"  Error: {e}")
        return None

def extract_functions(html: str) -> list[str]:
    """Extract function names from Doxygen HTML."""
    functions = []
    # Pattern 1: <a class="el" href="_module___commands_8h_aXXX.html#aXXX">function_name</a>
    pattern1 = r'<a\s+class="el"[^>]*href="[^"]*_8[ah]_[a-f0-9]+\.html#[a-f0-9]+"[^>]*>(\w+)</a>'
    for match in re.finditer(pattern1, html):
        func_name = match.group(1).strip()
        if func_name and not func_name.startswith('_'):
            functions.append(func_name)

    # Pattern 2: globals_func.html style - function names in plain links
    pattern2 = r'<a[^>]*href="[^"]*_8[ah]_[a-f0-9]+\.html"[^>]*>\s*(\w+)\s*\(\s*\)\s*</a>'
    for match in re.finditer(pattern2, html):
        func_name = match.group(1).strip()
        if func_name and func_name not in functions:
            functions.append(func_name)

    return functions

def extract_constants(html: str) -> list[str]:
    """Extract constant names from Doxygen HTML."""
    constants = []
    # Match #define statements
    pattern = r'#define\s+(\w+)'
    for match in re.finditer(pattern, html):
        constants.append(match.group(1))
    return constants

def main():
    DOCS_DIR.mkdir(exist_ok=True)

    all_functions = []
    all_constants = []
    module_functions = {}

    for display_name, url_filename in MODULE_FILES.items():
        html = fetch_file(display_name, url_filename)
        if html:
            # Save raw HTML
            output_path = DOCS_DIR / display_name.replace('.h', '.html')
            output_path.write_text(html, encoding='utf-8')

            # Extract functions and constants
            funcs = extract_functions(html)
            consts = extract_constants(html)

            module_functions[display_name] = funcs
            all_functions.extend(funcs)
            all_constants.extend(consts)

            if funcs or consts:
                print(f"  -> {len(funcs)} functions, {len(consts)} constants")

    print(f"\n{'='*60}")
    print(f"Total: {len(set(all_functions))} unique functions, {len(set(all_constants))} constants")

    # Write summary
    summary_path = DOCS_DIR / "extracted_summary.txt"
    with open(summary_path, 'w', encoding='utf-8') as f:
        f.write("=== FUNCTIONS BY MODULE ===\n\n")
        for module, funcs in sorted(module_functions.items()):
            if funcs:
                f.write(f"{module}:\n")
                for func in sorted(funcs):
                    f.write(f"  - {func}\n")
                f.write("\n")

        f.write(f"\n=== ALL FUNCTIONS ({len(set(all_functions))}) ===\n")
        for func in sorted(set(all_functions)):
            f.write(f"{func}\n")

        f.write(f"\n=== CONSTANTS ({len(set(all_constants))}) ===\n")
        for const in sorted(set(all_constants)):
            f.write(f"{const}\n")

    print(f"Summary written to {summary_path}")

    # Show missing from our implementation
    print(f"\n{'='*60}")
    print("Checking against our implementation...")

    our_file = Path(__file__).parent.parent / "src" / "engine" / "ai" / "popscript.rs"
    our_functions = set()
    if our_file.exists():
        content = our_file.read_text(encoding='utf-8')
        # Find stub functions
        import re
        stub_match = re.search(r'let stub_functions = \[(.*?)\];', content, re.DOTALL)
        if stub_match:
            our_functions.update(re.findall(r'"(\w+)"', stub_match.group(1)))
        # Find implemented functions
        our_functions.update(re.findall(r'globals\.set\(\s*"(\w+)"', content))

        doc_functions = set(all_functions)
        missing = doc_functions - our_functions
        extra = our_functions - doc_functions

        if missing:
            print(f"\nMissing from our implementation ({len(missing)}):")
            for func in sorted(missing):
                print(f"  - {func}")

        if extra:
            print(f"\nExtra (not in docs) ({len(extra)}):")
            for func in sorted(extra):
                print(f"  + {func}")

if __name__ == "__main__":
    main()
