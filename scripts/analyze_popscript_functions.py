#!/usr/bin/env python3
"""Analyze PopScript functions from our implementation and RE docs."""

import re
from pathlib import Path

def get_our_functions() -> dict:
    """Extract PopScript functions from our implementation."""
    popscript_file = Path(__file__).parent.parent / "src" / "engine" / "ai" / "popscript.rs"
    if not popscript_file.exists():
        return {}

    content = popscript_file.read_text(encoding='utf-8')
    functions = {}

    # Find stub_functions array
    stub_match = re.search(r'let stub_functions = \[(.*?)\];', content, re.DOTALL)
    if stub_match:
        stub_content = stub_match.group(1)
        stubs = re.findall(r'"(\w+)"', stub_content)
        for stub in stubs:
            functions[stub] = {'status': 'stub', 'implemented': False}

    # Find implemented functions (globals.set("NAME", ...))
    # Pattern: globals.set("FUNCTION_NAME", lua.create_function(...))
    impl_pattern = r'globals\.set\(\s*"(\w+)",\s*lua\.create_function'
    for match in re.finditer(impl_pattern, content):
        func_name = match.group(1)
        if func_name not in functions:
            functions[func_name] = {'status': 'implemented', 'implemented': True}
        else:
            # It was a stub, now implemented
            functions[func_name] = {'status': 'implemented', 'implemented': True}

    return functions

def get_re_documented_functions() -> dict:
    """Extract functions from our RE documentation."""
    doc_file = Path(__file__).parent.parent / "docs" / "specs" / "ai_scripting.md"
    if not doc_file.exists():
        return {}

    content = doc_file.read_text(encoding='utf-8')

    # Look for function tables and lists
    functions = {}

    # Pattern: | AI_FunctionName | 0x... | Description |
    table_pattern = r'\|\s*(AI_\w+|FUN_\w+)\s*\|\s*(0x[0-9a-fA-F]+)\s*\|\s*([^|]+)\|'
    for match in re.finditer(table_pattern, content):
        func_name = match.group(1)
        address = match.group(2)
        description = match.group(3).strip()
        functions[func_name] = {
            'address': address,
            'description': description,
            'source': 'RE docs'
        }

    return functions

def get_constants() -> set:
    """Extract constants registered for Lua."""
    constants_file = Path(__file__).parent.parent / "src" / "engine" / "ai" / "constants.rs"
    if not constants_file.exists():
        return set()

    content = constants_file.read_text(encoding='utf-8')
    constants = set()

    # Pattern: ("CONST_NAME", value)
    for match in re.finditer(r'\("(\w+)",\s*(-?\d+)\)', content):
        constants.add(match.group(1))

    return constants

def main():
    print("=" * 70)
    print("PopScript Function Analysis")
    print("=" * 70)

    our_functions = get_our_functions()
    re_functions = get_re_documented_functions()
    constants = get_constants()

    print(f"\nOur Lua functions: {len(our_functions)}")
    print(f"  - Implemented: {sum(1 for f in our_functions.values() if f['implemented'])}")
    print(f"  - Stubs: {sum(1 for f in our_functions.values() if not f['implemented'])}")

    print(f"\nRE documented functions: {len(re_functions)}")
    print(f"Constants: {len(constants)}")

    # Show implemented functions
    implemented = sorted([k for k, v in our_functions.items() if v['implemented']])
    print(f"\n--- Implemented Functions ({len(implemented)}) ---")
    for func in implemented:
        print(f"  {func}")

    # Show stub functions
    stubs = sorted([k for k, v in our_functions.items() if not v['implemented']])
    print(f"\n--- Stub Functions ({len(stubs)}) ---")
    for stub in stubs:
        print(f"  {stub}")

    # Show RE functions
    if re_functions:
        print(f"\n--- RE Documented Functions ({len(re_functions)}) ---")
        for func, info in sorted(re_functions.items()):
            print(f"  {func} @ {info['address']}: {info['description'][:50]}")

    # Show constants
    if constants:
        print(f"\n--- Constants ({len(constants)}) ---")
        for const in sorted(constants)[:50]:
            print(f"  {const}")
        if len(constants) > 50:
            print(f"  ... and {len(constants) - 50} more")

if __name__ == "__main__":
    main()
