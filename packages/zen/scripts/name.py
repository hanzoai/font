"""Give the hand-exported binaries the same identity the built ones get.

`make build` compiles Zen and Zen Mono from `sources/`, so those come out of
fontmake already carrying family "Zen"/"Zen Mono", vendor HNZO and our copyright
— the Glyphs sources hold those fields and `scripts/set-fontinfo.py` keeps them
there. Zen Pixel is the exception: it is drawn on a virtual master, gftools
builds only its variable, and its five static cuts are exported by hand from
Glyphs. An export skips the source fields, so those files arrive naming the
upstream family, and a face is bound to whatever family the CSS declares — so
@font-face reads Zen while every font menu, inspector and PDF embed reads the
old name.

Run after a hand export, then commit the result:

    python3 packages/zen/scripts/name.py

Idempotent: run it over the whole tree and the built files are already right.

nameID 0 carries the upstream copyright line above ours. OFL §2 binds each copy
to the notice, and family, uniqueID, version, manufacturer, designer and the
vendor URLs are not the notice — those are ours. §3 reserves nothing here: the
upstream block declares no Reserved Font Name, so the family name is free.
"""
import pathlib
import sys
from fontTools.ttLib import TTFont

ROOT = pathlib.Path(__file__).resolve().parents[3]
# The hand exports. Everything else under fonts/ comes out of fontmake already
# naming itself, including ZenPixel's variable, which gftools does build.
EXPORTS = [ROOT / 'fonts' / 'ZenPixel' / d for d in ('otf', 'ttf', 'webfonts')]

UPSTREAM = 'Copyright 2024 The Geist Project Authors (https://github.com/vercel/geist-font)'
OURS = 'Copyright 2026 Hanzo AI, Inc. (https://git.hanzo.ai/hanzoai/font)'
VENDOR = 'https://hanzo.ai'


def retag(path: pathlib.Path) -> tuple[str, str]:
    """Name one cut. Each is its own family, so its subfamily is Regular.

    Zen Pixel's five cuts are five drawings of one alphabet, not five weights of
    one drawing — nothing orders them, so a font menu should list them side by
    side rather than bury four in a style submenu. That makes nameID 1 the whole
    name and nameID 2 plain Regular, and leaves the typographic pair (16/17) for
    families that actually have styles to group.
    """
    f = TTFont(str(path))
    cut = path.stem.split('-')[-1]
    family = f'Zen Pixel {cut}'
    ver = (f['name'].getDebugName(5) or 'Version 1.000').replace('Version ', '')
    was = f['name'].getDebugName(1)
    names = {
        0: f'{UPSTREAM}\n{OURS}',
        1: family,
        2: 'Regular',
        3: f'{ver};HNZO;ZenPixel-{cut}',
        4: family,
        6: f'ZenPixel-{cut}',
        8: 'Hanzo AI, Inc.',
        9: 'Hanzo AI, Inc.',
        11: VENDOR,
        12: VENDOR,
    }
    for nid, val in names.items():
        f['name'].setName(val, nid, 3, 1, 0x409)      # windows/unicode/en-US
        f['name'].setName(val, nid, 1, 0, 0)          # mac/roman, where present
    for nid in (16, 17):
        f['name'].removeNames(nid)
    f['OS/2'].achVendID = 'HNZO'
    f.save(str(path))
    return was, family


def main() -> int:
    moved = 0
    for d in EXPORTS:
        if not d.is_dir():
            print(f'no {d} — run `make build` first', file=sys.stderr)
            return 1
        for path in sorted(d.iterdir()):
            if path.suffix not in ('.otf', '.ttf', '.woff2'):
                continue
            was, now = retag(path)
            if was != now:
                moved += 1
                print(f'  {path.name:<32} {was} -> {now}')
    print(f'{moved} renamed' if moved else 'every cut already names itself Zen Pixel')
    return 0


if __name__ == '__main__':
    sys.exit(main())
