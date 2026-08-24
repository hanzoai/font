/* Wordmarks, cut from Zen.
 *
 * Two of the three brands already set their wordmark in live type — Hanzo's is
 * `<text font-family="Zen" font-weight="600">Hanzo</text>` beside the H mark, and
 * Zoo's is two spans (800 / 300) beside the Venn. Lux's is the odd one: 633 bytes
 * of drawn geometry, which is why LUX CREDIT set in Zen used to sit next to it
 * rather than with it. Zen reaches it now — see `zen lux`, and the note below.
 *
 * This emits each wordmark AS Zen outlines, so a brand can adopt one file and have
 * every lockup beside it match by construction instead of by eye. It writes real
 * path data — not a `<text>` element — because a logo must render identically
 * where the font is absent: an email client, a partner's deck, a favicon.
 *
 * The cutting is `crates/zen`, invoked as `zen mark`. It used to be two Python
 * scripts, `_outline.py` for the plain cut and `_shape.py` for the shaped one —
 * the same job twice, and only one of them could produce the Lux mark.
 *
 * Run: node sites/wordmark.mjs   ->  sites/dist/wordmark/*.svg
 */
import { execFileSync } from 'node:child_process'
import { existsSync, mkdirSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))
const ROOT = join(HERE, '..')
const OUT = join(HERE, 'dist', 'wordmark')
const FONT = join(ROOT, 'packages/zen/dist/fonts/zen-sans/Zen-Variable.ttf')
const ZEN = join(ROOT, 'crates/zen/target/release/zen')

if (!existsSync(ZEN)) {
  console.error(`no ${ZEN}\n  cargo build --release --manifest-path crates/zen/Cargo.toml`)
  process.exit(2)
}

/* Each entry is the setting the brand uses, so these are not new decisions.
 *
 *   lux    FITTED to the drawn mark, feature by feature, by `zen lux`. Every
 *          measurement lands within 3.2% — stems were 22% fat and bars 21-29%
 *          thin at the old 845/1.40. `thicken` is what made it reachable: the
 *          drawn LUX is nearly monoline (bar/stem 0.90 against Zen's 0.58) and
 *          no weight or width setting gets there.
 *   hanzo  what brand/assets/logo/wordmark.svg already states
 *   zoo    lowercase — z, o and o are all x-height and the two o are the same
 *          circle, so the mark is symmetrical by construction rather than by
 *          optical adjustment. Caps would need the Z kerned against two round
 *          forms; lowercase needs nothing.
 */
const MARKS = [
  { name: 'lux',   text: 'LUX',   wght: 675, scaleX: 1.460, track: -0.040, flatten: 1, thicken: 72 },
  { name: 'hanzo', text: 'Hanzo', wght: 600, scaleX: 1.000, track: -0.0286, flatten: 0, thicken: 0 },
  { name: 'zoo',   text: 'zoo',   wght: 800, scaleX: 1.000, track: -0.025, flatten: 0, thicken: 0 },
]

mkdirSync(OUT, { recursive: true })
for (const m of MARKS) {
  const svg = execFileSync(ZEN, ['mark', FONT, m.text,
    String(m.wght), String(m.scaleX), String(m.track), String(m.flatten), String(m.thicken)],
    { encoding: 'utf8', maxBuffer: 8 << 20 })
  writeFileSync(join(OUT, `${m.name}.svg`), svg)
  console.log(`  ${m.name.padEnd(6)} ${m.text.padEnd(6)} wght ${m.wght} · ${m.scaleX}× · ` +
    `${m.track}em${m.thicken ? ` · thicken ${m.thicken}` : ''}   ${(svg.length / 1024).toFixed(1)} KB`)
}
