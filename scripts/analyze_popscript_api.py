#!/usr/bin/env python3
"""Analyze PopScript API - compare documented functions with our implementation."""

import re
from pathlib import Path

DOCS_SUMMARY = Path("scripts/popscript_docs/extracted_summary.txt")
POPSCRIPT_RS = Path("src/engine/ai/popscript.rs")

def get_documented_items() -> dict:
    """Get all documented items categorized by module."""
    summary = DOCS_SUMMARY.read_text()

    modules = {}
    current_module = None

    for line in summary.split('\n'):
        if line.endswith(':'):
            current_module = line[:-1]
            modules[current_module] = []
        elif line.strip().startswith('- ') and current_module:
            name = line.strip()[2:].strip()
            if name and not name.startswith('_'):
                modules[current_module].append(name)

    return modules

def get_our_functions() -> dict:
    """Get our Lua-exposed functions."""
    content = POPSCRIPT_RS.read_text()
    funcs = {'implemented': set(), 'stubs': set()}

    # Stubs
    stub_match = re.search(r'let stub_functions = \[(.*?)\];', content, re.DOTALL)
    if stub_match:
        funcs['stubs'].update(re.findall(r'"(\w+)"', stub_match.group(1)))

    # Implemented - look for globals.set("NAME", lua.create_function
    impl_pattern = r'globals\.set\(\s*"(\w+)"\s*,\s*lua\.create_function'
    for match in re.finditer(impl_pattern, content):
        funcs['implemented'].add(match.group(1))

    return funcs

def is_likely_lua_api(name: str) -> bool:
    """Heuristic: Lua API functions tend to be uppercase or have specific patterns."""
    # Internal C++ helpers usually start with lowercase
    if name[0].islower():
        return False
    # Constants are all uppercase with underscores
    if name.isupper() or (name.replace('_', '').isalpha() and name.isupper()):
        return True
    # Mixed case commands
    if '_' in name and name[0].isupper():
        return True
    return False

def main():
    modules = get_documented_items()
    our_funcs = get_our_functions()

    all_our_funcs = our_funcs['implemented'] | our_funcs['stubs']

    # Get documented items that look like Lua API (not internal C++ helpers)
    doc_lua_api = set()
    doc_by_module = {}

    for module, items in modules.items():
        lua_items = [i for i in items if is_likely_lua_api(i)]
        doc_lua_api.update(lua_items)
        if lua_items:
            doc_by_module[module] = lua_items

    print("=" * 70)
    print("PopScript API Analysis")
    print("=" * 70)
    print(f"\nDocumented items (all): {sum(len(v) for v in modules.values())}")
    print(f"Documented Lua API (filtered): {len(doc_lua_api)}")
    print(f"Our functions: {len(all_our_funcs)}")
    print(f"  - Implemented: {len(our_funcs['implemented'])}")
    print(f"  - Stubs: {len(our_funcs['stubs'])}")

    # Find overlap
    in_docs = all_our_funcs & doc_lua_api
    missing = doc_lua_api - all_our_funcs
    extra = all_our_funcs - doc_lua_api

    print(f"\nOverlap (we have + documented): {len(in_docs)}")
    print(f"Missing from us (in docs): {len(missing)}")
    print(f"Extra (we have, not in docs): {len(extra)}")

    # Show missing functions by module
    print("\n" + "=" * 70)
    print("MISSING FUNCTIONS (by module)")
    print("=" * 70)

    for module, items in sorted(doc_by_module.items()):
        module_missing = set(items) - all_our_funcs
        if module_missing:
            print(f"\n{module} ({len(module_missing)} missing):")
            for func in sorted(module_missing)[:20]:
                print(f"  - {func}")
            if len(module_missing) > 20:
                print(f"  ... and {len(module_missing) - 20} more")

    # Show extra functions (we have but not documented)
    print("\n" + "=" * 70)
    print("EXTRA FUNCTIONS (we have, not in official docs)")
    print("=" * 70)

    if extra:
        print(f"\n{len(extra)} functions that aren't in the official docs:")
        for func in sorted(extra):
            status = "IMPL" if func in our_funcs['implemented'] else "stub"
            print(f"  [{status}] {func}")

    # Show implemented functions that ARE documented
    print("\n" + "=" * 70)
    print("IMPLEMENTED FUNCTIONS (documented)")
    print("=" * 70)

    impl_documented = our_funcs['implemented'] & doc_lua_api
    if impl_documented:
        print(f"\n{len(impl_documented)} implemented functions that are documented:")
        for func in sorted(impl_documented):
            print(f"  {func}")

if __name__ == "__main__":
    main()
