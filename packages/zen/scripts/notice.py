"""Put the upstream copyright line back on the built families.

`name.py` names the hand-exported Zen Pixel cuts and writes nameID 0 as the
upstream notice above ours, because OFL §2 binds each copy of the Font Software
to the notice and a .woff2 served to a browser IS a copy travelling alone. The
fontmake-built families take the other path — their fields come from the Glyphs
sources via `scripts/set-fontinfo.py` — and that path wrote only our line, so
Zen and Zen Mono shipped without the notice the licence at the root already
carries. This puts them in agreement.

Sources are fixed too (set-fontinfo.py), so a rebuild produces this directly;
this exists for the binaries already built and committed.

    python3 packages/zen/scripts/notice.py

Idempotent: a file already carrying both lines is left alone.
"""
import pathlib
import sys

from fontTools.ttLib import TTFont

ROOT = pathlib.Path(__file__).resolve().parents[3]
UPSTREAM = 'Copyright 2024 The Geist Project Authors (https://github.com/vercel/geist-font)'
OURS = 'Copyright 2026 Hanzo AI, Inc. (https://git.hanzo.ai/hanzoai/font)'
NOTICE = f'{UPSTREAM}\n{OURS}'


def fix(path: pathlib.Path) -> bool:
    f = TTFont(str(path))
    have = f['name'].getDebugName(0) or ''
    if UPSTREAM in have and OURS in have:
        return False
    f['name'].setName(NOTICE, 0, 3, 1, 0x409)
    f['name'].setName(NOTICE, 0, 1, 0, 0)
    f.save(str(path))
    return True


def main() -> int:
    roots = [ROOT / 'fonts', ROOT / 'packages' / 'zen' / 'dist' / 'fonts']
    seen = fixed = 0
    for r in roots:
        if not r.is_dir():
            continue
        for path in sorted(r.rglob('*')):
            if path.suffix not in ('.otf', '.ttf', '.woff2'):
                continue
            seen += 1
            if fix(path):
                fixed += 1
    if not seen:
        print('no built faces found — run `make build` first', file=sys.stderr)
        return 1
    print(f'{fixed} of {seen} faces gained the upstream notice'
          if fixed else f'all {seen} faces already carry both notices')
    return 0


if __name__ == '__main__':
    sys.exit(main())
