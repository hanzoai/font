#!/usr/bin/env python3
"""Set the attribution fields in every Glyphs source.

These fields land in the built font's `name` table and OS/2 `achVendID`, so
they travel with a bare .woff2 wherever it is served. Run after editing a
source in Glyphs, which rewrites the file from its own UI fields.
"""
import re
import sys
from pathlib import Path

FIELDS = {
    "copyrights": "Copyright 2026 Hanzo AI, Inc. (https://git.hanzo.ai/hanzoai/font)",
    "designers": "Hanzo AI, Inc.",
    "designerURL": "https://hanzo.ai",
    "manufacturers": "Hanzo AI, Inc.",
    "manufacturerURL": "https://hanzo.ai",
    "vendorID": "HNZO",
}

SOURCES = Path(__file__).resolve().parent.parent / "sources"


def patch(text, key, value):
    """Replace the first `value = ...;` after `key = <key>;`.

    Glyphs writes two shapes: a bare `value` for scalars, and a `values = (…)`
    list of per-language entries. Both begin with the same `value =` token, so
    one pattern covers each.
    """
    pattern = re.compile(
        r"(key = " + re.escape(key) + r";(?:.*?\n){0,6}?\s*value = )"
        r'("(?:[^"\\]|\\.)*"|[^;\n]+)(;)',
        re.DOTALL,
    )
    new, n = pattern.subn(lambda m: m.group(1) + f'"{value}"' + m.group(3),
                          text, count=1)
    if n != 1:
        raise SystemExit(f"{key}: expected 1 match, got {n}")
    return new


def main():
    files = sorted(SOURCES.glob("*.glyphspackage/fontinfo.plist"))
    if not files:
        raise SystemExit(f"no sources under {SOURCES}")
    for path in files:
        text = original = path.read_text(encoding="utf-8")
        for key, value in FIELDS.items():
            text = patch(text, key, value)
        if text != original:
            path.write_text(text, encoding="utf-8")
            print(f"patched {path.relative_to(SOURCES.parent)}")
        else:
            print(f"unchanged {path.relative_to(SOURCES.parent)}")


if __name__ == "__main__":
    sys.exit(main())
