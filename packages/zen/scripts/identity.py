"""What makes a face ours, said once.

Two scripts write identity into a binary — `name.py` names the hand-exported Zen
Pixel cuts, `notice.py` puts the upstream line back on the fontmake-built
families — and both need the same answer to "whose is this?". Held in one place
they cannot drift apart, and `check` reads the same answer back.

Run it over a directory to verify rather than mutate:

    python3 packages/zen/scripts/identity.py packages/zen/dist/fonts

It exits non-zero on the first face that is not ours, which is why
`copy-npm-fonts` ends with it: the moment a face becomes something the package
would publish is the moment to refuse it.

The check is worth having because the failure it catches is invisible. A face
relabelled family-only — nameID 1 rewritten to Zen while the vendor tag stays
VRCL and nameID 0 names one holder — answers "Zen" to every question a
stylesheet, a grep or a font menu asks. Four repositories in the estate carried
Geist under our name that way. Vendor and notice are where the bytes admit what
they are.
"""
import json
import pathlib
import sys

from fontTools.ttLib import TTFont

UPSTREAM = 'Copyright 2024 The Geist Project Authors (https://github.com/vercel/geist-font)'
OURS = 'Copyright 2026 Hanzo AI, Inc. (https://git.hanzo.ai/hanzoai/font)'
NOTICE = f'{UPSTREAM}\n{OURS}'
VENDOR = 'https://hanzo.ai'
TAG = 'HNZO'
FACES = ('.otf', '.ttf', '.woff2')

# The Arabic companions are the one set of faces here that are NOT ours, and they
# are checked for exactly that: each must still say the family, notice and vendor
# its upstream gave it. Relabelling one as Zen would be the failure this file
# exists to catch, pointed the other way. The table is the one arabic.py builds
# from, so the two cannot disagree about which files are companions.
COMPANIONS = {
    face['file']: face
    for face in json.loads((pathlib.Path(__file__).resolve().parent / 'arabic.json').read_text())['faces']
}

FAMILIES = {'Zen', 'Zen Mono'} | {
    f'Zen Pixel {cut}' for cut in ('Circle', 'Grid', 'Line', 'Square', 'Triangle')
}


def complaints(path: pathlib.Path) -> list[str]:
    """Everything wrong with one face's identity. Empty means it is ours."""
    f = TTFont(str(path))
    out = []
    # A static cut outside the four styles a family can group puts its weight in
    # nameID 1 — "Zen Thin" — and the family they all share in nameID 16. Where
    # the two would say the same thing, 16 is absent. So 16 is the family when
    # it is there, and 1 is the family when it is not.
    family = f['name'].getDebugName(16) or f['name'].getDebugName(1) or ''
    # The variable Zen Pixel carries all five cuts on one axis, so it names the
    # family they share rather than any one of them.
    if family not in FAMILIES and family != 'Zen Pixel':
        out.append(f'family {family!r}')
    tag = f['OS/2'].achVendID
    if tag != TAG:
        out.append(f'vendor {tag!r}')
    # Every nameID 0 record, not just the one a reader happens to pick: the
    # notice lives in a Windows record and a Mac one, and a face that rewrote
    # only the record you read agrees with itself right up until something
    # reads the other. OFL §2 binds each copy to the notice, and a .woff2
    # travels alone.
    notices = [r.toUnicode() for r in f['name'].names if r.nameID == 0]
    if not notices:
        out.append('no copyright at all')
    for holder, who in ((UPSTREAM, 'upstream'), (OURS, 'ours')):
        if any(holder not in n for n in notices):
            out.append(f'no {who} copyright')
    return out


def companion(path: pathlib.Path, face: dict) -> list[str]:
    """Everything wrong with a companion's identity. Empty means it is still theirs."""
    f = TTFont(str(path))
    out = []
    family = f['name'].getDebugName(16) or f['name'].getDebugName(1) or ''
    if family != face['family']:
        out.append(f'family {family!r}, upstream says {face["family"]!r}')
    if f['OS/2'].achVendID == TAG:
        out.append(f'vendor {TAG} on a face that is not ours')
    notices = [r.toUnicode() for r in f['name'].names if r.nameID == 0]
    if not notices or any(face['notice'] not in n for n in notices):
        out.append('upstream copyright missing')
    if path.parent.name != 'arabic':
        out.append('outside fonts/arabic')
    return out


def main() -> int:
    roots = [pathlib.Path(a) for a in sys.argv[1:]] or [
        pathlib.Path(__file__).resolve().parents[1] / 'dist' / 'fonts'
    ]
    seen = 0
    bad = []
    for r in roots:
        for path in sorted(r.rglob('*')):
            if path.suffix not in FACES:
                continue
            seen += 1
            face = COMPANIONS.get(path.name)
            if said := (companion(path, face) if face else complaints(path)):
                bad.append(f'  {path.name:<34} {", ".join(said)}')
    if not seen:
        print(f'no faces under {", ".join(str(r) for r in roots)}', file=sys.stderr)
        return 1
    if bad:
        print(f'{len(bad)} of {seen} faces do not say what they are:', file=sys.stderr)
        print('\n'.join(bad), file=sys.stderr)
        return 1
    print(f'all {seen} faces check out: ours name themselves Zen, tag {TAG}, and carry both '
          f'notices; the {len(COMPANIONS)} Arabic companions keep their own')
    return 0


if __name__ == '__main__':
    sys.exit(main())
