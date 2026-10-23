"""The Arabic companions: fetched from upstream, cut to the Arabic blocks, shipped.

Zen draws Latin and Cyrillic and no Arabic, so a page in Arabic used to fall back
to whatever face each browser picked. The companions in `arabic.json` fill that
gap. They are NOT ours and are never renamed: each keeps its own family, notice
and vendor, which is what `identity.py` checks them for. Each is pinned by
commit and checksum, so a rebuild cannot quietly change what ships.

What happens to each face, and why:

- Every axis but weight is pinned (Noto Sans Arabic's width, at 100). The page
  asks for a weight and nothing else, and a second axis doubles the bytes.
- The cmap is cut to the Arabic blocks in `range`, plus U+200C-200F so the
  joiners shape inside the face. The joiners stay OUT of the declared
  unicode-range: an English page with an emoji ZWJ sequence would otherwise
  fetch the Arabic face for one invisible character.
- Layout features are kept whole, and hinting goes: the subset is drawn by a
  browser that ignores it.

Run from the repository root (needs the network once per face):

    python3 packages/zen/scripts/arabic.py
"""
import hashlib
import io
import json
import pathlib
import sys
import urllib.request

from fontTools import subset
from fontTools.ttLib import TTFont
from fontTools.varLib import instancer

HERE = pathlib.Path(__file__).resolve().parent
TABLE = json.loads((HERE / 'arabic.json').read_text())
OUT = HERE.parent / 'dist' / 'fonts' / 'arabic'
JOINERS = range(0x200C, 0x2010)


def fetch(url: str, sha256: str) -> bytes:
    with urllib.request.urlopen(url, timeout=60) as r:
        data = r.read()
    got = hashlib.sha256(data).hexdigest()
    if got != sha256:
        raise SystemExit(f'{url}\n  sha256 {got}\n  wanted {sha256}')
    return data


def codepoints(spec: str) -> set[int]:
    out: set[int] = set()
    for part in spec.split(','):
        lo, _, hi = part.strip().removeprefix('U+').partition('-')
        out.update(range(int(lo, 16), int(hi or lo, 16) + 1))
    return out


def cut(face: dict) -> bytes:
    font = TTFont(io.BytesIO(fetch(face['source'], face['sha256'])))
    if face['pin']:
        # Through bytes and back: the instancer leaves gvar half-loaded, and the
        # subsetter then asks it for glyphs it no longer has.
        pinned = io.BytesIO()
        instancer.instantiateVariableFont(font, face['pin']).save(pinned)
        font = TTFont(io.BytesIO(pinned.getvalue()))
    axes = {a.axisTag: (a.minValue, a.maxValue) for a in font['fvar'].axes}
    lo, hi = (float(w) for w in face['weight'].split())
    if axes != {'wght': (lo, hi)}:
        raise SystemExit(f"{face['family']}: axes {axes}, declared wght {face['weight']}")
    options = subset.Options()
    options.layout_features = ['*']
    options.name_IDs = ['*']
    options.name_languages = ['*']
    options.name_legacy = True
    options.hinting = False
    options.flavor = 'woff2'
    sub = subset.Subsetter(options)
    sub.populate(unicodes=codepoints(TABLE['range']) | set(JOINERS))
    sub.subset(font)
    buf = io.BytesIO()
    font.flavor = 'woff2'
    font.save(buf)
    return buf.getvalue()


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    lic = TABLE['license']
    (OUT / lic['file']).write_bytes(fetch(lic['source'], lic['sha256']))
    for face in TABLE['faces']:
        data = cut(face)
        (OUT / face['file']).write_bytes(data)
        print(f"{face['file']:<34} {len(data) / 1024:6.1f} KB  {face['family']}")
    return 0


if __name__ == '__main__':
    sys.exit(main())
