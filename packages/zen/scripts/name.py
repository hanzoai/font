"""Make the Zen binaries identify as Zen.

The files shipped as Zen were byte-identical upstream with a new FILENAME. Inside,
the name table still read family "Geist", uniqueID "1.800;VRCL;Geist-Regular",
manufacturer "Basement.studio, Vercel, ...". @font-face hid it — a face is bound to
whatever family name the CSS declares — so the CSS said Zen while every font
inspector, every OS font menu and every PDF embed said Geist.

WHAT THE LICENCE ALLOWS, precisely, because this is the whole question:

  §3 reserves nothing here. The upstream copyright block declares NO Reserved Font
  Name — it is two plain copyright lines — so renaming the family is unrestricted.
  (A comment in @hanzo/ui claimed an RFN clause forced the rename. There is none;
  the rename was a branding decision, and a good one, but not a legal requirement.)

  §2 is the binding one: "each copy contains the above copyright notice and this
  license". The COPYRIGHT NOTICE is not ours to drop. Family, uniqueID, version,
  manufacturer, designer and the vendor URLs are not the copyright notice, so those
  become ours. nameID 0 keeps the upstream line and gains ours beneath it.

  §2 also says the notice may live in "stand-alone text files, human-readable
  headers or ... machine-readable metadata fields". So ONE carrier satisfies it.
  We keep two anyway — nameID 0 and the OFL text file — because a woff2 that gets
  copied out of its directory would otherwise travel with no notice at all.

So the floor is one line, in the licence, where a licence belongs. Everywhere else
— package descriptions, READMEs, code comments, docs prose — it is ours to remove,
and §2 does not reach any of them.

Run: python3 scripts/name.py
"""
import pathlib
from fontTools.ttLib import TTFont

HERE = pathlib.Path(__file__).resolve().parent.parent
FONTS = HERE / 'dist' / 'fonts'

UPSTREAM = 'Copyright 2024 The Geist Project Authors (https://github.com/vercel/geist-font)'
OURS = 'Copyright 2026 Hanzo AI, Inc.'
VENDOR = 'https://hanzo.ai'

FAMILIES = {'zen-sans': 'Zen', 'zen-mono': 'Zen Mono', 'zen-pixel': None}  # pixel: per-file


def family_for(d: str, stem: str) -> str:
    if d == 'zen-pixel':
        return 'Zen Pixel ' + stem.split('-')[-1]
    return FAMILIES[d]


def subfamily(stem: str) -> str:
    tail = stem.split('-')[-1]
    return 'Regular' if tail in ('Variable', 'Regular') else tail


def retag(path: pathlib.Path, family: str, sub: str) -> dict:
    f = TTFont(str(path))
    full = family if sub == 'Regular' else f'{family} {sub}'
    ver = f['name'].getDebugName(5) or 'Version 1.800'
    # nameID 0 keeps the upstream notice and gains ours. Everything else is ours.
    new = {
        0:  f'{UPSTREAM}\n{OURS}',
        1:  family,
        2:  sub,
        3:  f'{ver.replace("Version ", "")};HNZO;{family.replace(" ", "")}-{sub}',
        4:  full,
        6:  f'{family.replace(" ", "")}-{sub}',
        8:  'Hanzo AI, Inc.',
        9:  'Hanzo AI, Inc.',
        11: VENDOR,
        12: VENDOR,
        16: family,
        17: sub,
    }
    before = f['name'].getDebugName(1)
    for nid, val in new.items():
        f['name'].setName(val, nid, 3, 1, 0x409)      # windows/unicode/en-US
        f['name'].setName(val, nid, 1, 0, 0)          # mac/roman, where present
    f.save(str(path))
    return {'file': path.name, 'was': before, 'now': family}


def main():
    changed = []
    for d in sorted(p for p in FONTS.iterdir() if p.is_dir()):
        for path in sorted(d.glob('*.ttf')) + sorted(d.glob('*.woff2')):
            fam = family_for(d.name, path.stem)
            changed.append(retag(path, fam, subfamily(path.stem)))
    for c in changed:
        print(f"  {c['file']:<28} {c['was']} -> {c['now']}")
    print(f'\n{len(changed)} files retagged')


main()
