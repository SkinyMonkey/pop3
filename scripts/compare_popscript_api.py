#!/usr/bin/env python3
"""Compare PopScript API from documentation with our implementation."""

from html.parser import HTMLParser
from pathlib import Path
import re

DOCS_DIR = Path(__file__).parent / "popscript_docs"

class FunctionExtractor(HTMLParser):
    """Extract function names from Doxygen HTML."""
    def __init__(self):
        super().__init__()
        self.functions = []
        self.in_function = False

    def handle_starttag(self, tag, attrs):
        if tag == 'a' and any(attr == 'class' for attr, _ in attrs):
            for attr, value in attrs:
                if attr == 'href' and value.startswith('_module_'):
                    self.in_function = True

    def handle_data(self, data):
        if self.in_function and '()' in data:
            func_name = data.strip().replace('()', '')
            if func_name and not func_name.startswith('_'):
                self.functions.append(func_name)
        self.in_function = False

def extract_functions_from_file(filepath: Path) -> list[str]:
    """Extract function names from HTML file."""
    if not filepath.exists():
        return []

    content = filepath.read_text(encoding='utf-8')
    parser = FunctionExtractor()
    try:
        parser.feed(content)
    except:
        pass
    return parser.functions

def get_all_documented_functions() -> set[str]:
    """Get all documented functions from HTML files."""
    functions = set()

    # Also parse globals_func.html directly with regex
    globals_func = DOCS_DIR / "globals_func.html"
    if globals_func.exists():
        content = globals_func.read_text(encoding='utf-8')
        # Match patterns like: function_name()</a>
        matches = re.findall(r'(\w+)\(\)', content)
        functions.update(matches)

    # Parse all HTML files
    for html_file in DOCS_DIR.glob("*.html"):
        funcs = extract_functions_from_file(html_file)
        functions.update(funcs)

    return functions

def get_our_functions() -> set[str]:
    """Extract PopScript functions from our implementation."""
    popscript_file = Path(__file__).parent.parent / "src" / "engine" / "ai" / "popscript.rs"
    if not popscript_file.exists():
        return set()

    content = popscript_file.read_text(encoding='utf-8')
    functions = set()

    # Find stub_functions array
    stub_match = re.search(r'let stub_functions = \[(.*?)\];', content, re.DOTALL)
    if stub_match:
        stub_content = stub_match.group(1)
        stubs = re.findall(r'"(\w+)"', stub_content)
        functions.update(stubs)

    # Find implemented functions (globals.set("NAME", ...))
    impl_matches = re.findall(r'globals\.set\(\s*"(\w+)"', content)
    functions.update(impl_matches)

    return functions

def main():
    print("=" * 60)
    print("PopScript API Comparison")
    print("=" * 60)

    doc_functions = get_all_documented_functions()
    our_functions = get_our_functions()

    print(f"\nDocumented functions: {len(doc_functions)}")
    print(f"Our functions: {len(our_functions)}")

    if doc_functions:
        print("\n--- Documented Functions ---")
        for func in sorted(doc_functions):
            in_ours = func in our_functions
            status = "OK" if in_ours else "MISSING"
            print(f"  [{status}] {func}")

    if our_functions:
        print("\n--- Our Functions ---")
        for func in sorted(our_functions):
            in_docs = func in doc_functions
            status = "OK" if in_docs else "NOT IN DOCS"
            print(f"  [{status}] {func}")

    missing = doc_functions - our_functions
    extra = our_functions - doc_functions

    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)
    print(f"Missing from our implementation: {len(missing)}")
    for func in sorted(missing):
        print(f"  - {func}")

    print(f"\nExtra (not in docs): {len(extra)}")
    for func in sorted(extra):
        print(f"  + {func}")

if __name__ == "__main__":
    main()
