/* Optical kerning, computed from the outlines.
 *
 * A font's kern table is cut for TEXT. Set the same pairs at display size and the
 * round-to-round joins open up — S|O is the classic one — because the eye judges
 * the AREA of white between two shapes, and area grows with the square of size
 * while the kern grows linearly. Uniform tracking cannot fix it: tightening
 * everything to close S|O crushes H|I.
 *
 * So measure the area. For each scanline across the band where two glyphs overlap
 * vertically, the white between them is (advance − right edge of A) + (left edge of
 * B). Sum that, clip how far into a bay we are willing to look — otherwise T's open
 * shoulder swamps every pair it appears in — and compare against a target.
 *
 * The target is LEARNED from the font's own kern table rather than picked. Zen
 * kerns 4,592 pairs by hand; the median white those pairs settle on is what the
 * designer decided a good join looks like, so that is the number to aim the
 * unkerned pairs at. Anything else is substituting my taste for theirs.
 *
 * Run: node scripts/kern.mjs "SOVEREIGN" [wght]
 */
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import { execFileSync } from 'node:child_process'

const HERE = dirname(fileURLToPath(import.meta.url))
const PY = join(HERE, '_kern.py')
const text = process.argv[2] ?? 'SOVEREIGN'
const wght = process.argv[3] ?? '845'

// The outline work is fontTools' — shelling to it beats reimplementing glyf parsing.
const out = execFileSync('python3', [PY, text, wght], { encoding: 'utf8' })
process.stdout.write(out)
