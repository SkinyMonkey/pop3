#!/usr/bin/env python3
"""Fetch PopScript documentation from populous3.info and save locally.
Uses only standard library - no external dependencies."""

import urllib.request
from html.parser import HTMLParser
from pathlib import Path

BASE_URL = "https://www.populous3.info/script3_doc/"
OUTPUT_DIR = Path(__file__).parent / "popscript_docs"

class LinkExtractor(HTMLParser):
    """Extract href links from HTML."""
    def __init__(self):
        super().__init__()
        self.links = []

    def handle_starttag(self, tag, attrs):
        if tag == 'a':
            for attr, value in attrs:
                if attr == 'href' and value.endswith('.html'):
                    self.links.append(value)

def fetch_page(url: str) -> str:
    """Fetch a page and return its HTML content."""
    print(f"Fetching: {url}")
    req = urllib.request.Request(url, headers={
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
    })
    with urllib.request.urlopen(req, timeout=30) as resp:
        return resp.read().decode('utf-8')

def save_html(content: str, path: Path):
    """Save HTML content to file."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding='utf-8')

def main():
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    # Fetch index page
    index_html = fetch_page(BASE_URL)
    save_html(index_html, OUTPUT_DIR / "index.html")

    # Doxygen sites have standard pages - fetch them all
    doxygen_pages = [
        'functions.html', 'functions_func.html', 'functions_vars.html',
        'globals.html', 'globals_func.html', 'globals_defs.html',
        'modules.html', 'namespaces.html', 'files.html'
    ]

    pages = set()
    for page in doxygen_pages:
        pages.add(BASE_URL + page)

    # Also extract any links from index
    parser = LinkExtractor()
    parser.feed(index_html)
    for href in parser.links:
        if not href.startswith('http'):
            pages.add(BASE_URL + href)

    print(f"Found {len(pages)} documentation pages to fetch")

    # Fetch each page
    fetched = 0
    for page_url in sorted(pages):
        try:
            html = fetch_page(page_url)
            filename = page_url.replace(BASE_URL, '')
            save_html(html, OUTPUT_DIR / filename)
            fetched += 1
        except Exception as e:
            print(f"Error fetching {page_url}: {e}")

    print(f"\nDocumentation saved to: {OUTPUT_DIR}")
    print(f"Total pages fetched: {fetched}")

if __name__ == "__main__":
    main()
