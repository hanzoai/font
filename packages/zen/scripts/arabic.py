"""The Arabic companions: fetched from upstream, checked, shipped.

Zen draws Latin and Cyrillic and no Arabic, so a page in Arabic used to fall back
to whatever face each browser picked. The companions in `arabic.json` fill that
gap. They are NOT ours and are never renamed: each keeps its own family, notice
and vendor, which is what `identity.py` checks them for. Each file is pinned by
commit and checksum, so a rebuild cannot quietly change what ships.

A companion must draw every letter in `needs`: the Arabic-block characters that
every Arabic-script catalog @hanzo/i18n ships writes (Arabic, Persian, Dari, Urdu,
Pashto, Sindhi, Sorani Kurdish, Uyghur, Shahmukhi Punjabi), plus both sets of
digits. A face missing one sets that letter in another face, mid-word, where the
two do not join. Cairo, Readex Pro and Rubik were measured and each misses letters
Pashto, Sindhi, Kurdish or Uyghur write, so none is here.

What happens to a face, and why:

- `cut` faces are cut to `range`. Every axis but weight is pinned (Noto Sans
  Arabic's width, at 100): the page asks for a weight and nothing else, and a
  second axis doubles the bytes. The cmap keeps the Arabic blocks plus
  U+200C-200F, so the joiners shape inside the face; the joiners stay OUT of the
  declared unicode-range, or an English page with an emoji ZWJ sequence would
  fetch the Arabic face for one invisible character. Layout features are kept
  whole, and hinting goes: the subset is drawn by a browser that ignores it.
- A face that is NOT cut ships as its foundry published it. IBM Plex Sans Arabic
  is one: the OFL reserves the name "Plex", a subset is a Modified Version, and a
  Modified Version may not carry a Reserved Font Name. IBM's own woff2 is used
  byte for byte, and the declared unicode-range still keeps its Latin unused.

Run from the repository root (needs the network once per file):

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


NEEDS = codepoints(TABLE['needs'])


def cut(face: dict, f: dict) -> bytes:
    font = TTFont(io.BytesIO(fetch(f['source'], f['sha256'])))
    if face['pin']:
        # Through bytes and back: the instancer leaves gvar half-loaded, and the
        # subsetter then asks it for glyphs it no longer has.
        pinned = io.BytesIO()
        instancer.instantiateVariableFont(font, face['pin']).save(pinned)
        font = TTFont(io.BytesIO(pinned.getvalue()))
    axes = {a.axisTag: (a.minValue, a.maxValue) for a in font['fvar'].axes}
    lo, hi = (float(w) for w in f['weight'].split())
    if axes != {'wght': (lo, hi)}:
        raise SystemExit(f"{face['family']}: axes {axes}, declared wght {f['weight']}")
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


def whole(face: dict, f: dict) -> bytes:
    data = fetch(f['source'], f['sha256'])
    font = TTFont(io.BytesIO(data))
    if font.flavor != 'woff2':
        raise SystemExit(f"{f['file']}: {face['family']} ships uncut, so its source must already be woff2")
    if font['OS/2'].usWeightClass != int(f['weight']):
        raise SystemExit(f"{f['file']}: weight class {font['OS/2'].usWeightClass}, declared {f['weight']}")
    return data


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    for name, lic in TABLE['licenses'].items():
        (OUT / name).write_bytes(fetch(lic['source'], lic['sha256']))
    for face in TABLE['faces']:
        for f in face['files']:
            data = (cut if face['cut'] else whole)(face, f)
            missing = NEEDS - set(TTFont(io.BytesIO(data)).getBestCmap())
            if missing:
                letters = ''.join(chr(c) for c in sorted(missing))
                raise SystemExit(f"{f['file']}: {face['family']} does not draw {letters}")
            (OUT / f['file']).write_bytes(data)
            print(f"{f['file']:<36} {len(data) / 1024:6.1f} KB  {face['family']} {f['weight']}")
    return 0


if __name__ == '__main__':
    sys.exit(main())
