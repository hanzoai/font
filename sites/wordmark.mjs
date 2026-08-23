/* Wordmarks, cut from Zen.
 *
 * Two of the three brands already set their wordmark in live type — Hanzo's is
 * `<text font-family="Zen" font-weight="600">Hanzo</text>` beside the H mark, and
 * Zoo's is two spans (800 / 300) beside the Venn. Lux's is the odd one: 633 bytes
 * of drawn geometry, which is why LUX CREDIT set in Zen sits next to it rather
 * than with it. Zen reaches it — `zen-wide` was refitted to that mark and lands
 * within 2px of its width — and the U wants `_shape.py`'s flatten on top, since
 * Zen's round forms overshoot the baseline by 16 units and the drawn mark is flat.
 *
 * This emits the wordmark AS Zen outlines, so a brand can adopt one file and have
 * every lockup beside it match by construction instead of by eye. It writes real
 * path data — not a `<text>` element — because a logo must render identically
 * where the font is absent: an email client, a partner's deck, a favicon.
 *
 * Run: node sites/wordmark.mjs   ->  sites/dist/wordmark/*.svg
 */
import { execFileSync } from 'node:child_process'
import { mkdirSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))
const OUT = join(HERE, 'dist', 'wordmark')
const PY = join(HERE, '_outline.py')

/* The Lux mark IS the wide preset — it does not restate it.
   Copying 845/1.56/-0.04 here is how the two drifted: the preset was refitted
   from Druk to the drawn LUX and this file kept cutting the old width, so a
   brand adopting the file got a mark 83px wider than the CSS beside it. There
   is one definition of the voice and it lives with the other four. */
const { PRESETS } = await import('../packages/zen/dist/presets.js')
const wide = PRESETS.wide

/* Each entry is the setting the brand already uses, so these are not new
   decisions — they are the existing lockups made portable.
     hanzo  what brand/assets/logo/wordmark.svg already states
     zoo    lowercase — z, o and o are all x-height and the two o are the same
            circle, so the mark is symmetrical by construction rather than by
            optical adjustment. Caps would need the Z kerned against two round
            forms; lowercase needs nothing. */
const MARKS = [
  { name: 'lux',   text: 'LUX',   wght: wide.wght, scaleX: wide.scaleX, track: wide.track },
  { name: 'hanzo', text: 'Hanzo', wght: 600, scaleX: 1.00, track: -0.0286 },
  { name: 'zoo',   text: 'zoo',   wght: 800, scaleX: 1.00, track: -0.025 },
]

mkdirSync(OUT, { recursive: true })
for (const m of MARKS) {
  const svg = execFileSync('python3',
    [PY, m.text, String(m.wght), String(m.scaleX), String(m.track)],
    { encoding: 'utf8', maxBuffer: 8 << 20 })
  writeFileSync(join(OUT, `${m.name}.svg`), svg)
  console.log(`  ${m.name.padEnd(6)} ${m.text.padEnd(6)} wght ${m.wght} · ${m.scaleX}× · ${m.track}em   ${(svg.length / 1024).toFixed(1)} KB`)
}
