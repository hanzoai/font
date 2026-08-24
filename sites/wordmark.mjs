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
 * THE U TUCKS UNDER THE X. The drawn mark overlaps those two letters — the X
 * starts before the U ends — and no uniform tracking produces an overlap, so it
 * takes a pair kern (`ux`). Tightening one pair shortens the run, so `scaleX`
 * rises from 1.540 to 1.576 to hold the mark at the drawn width.
 *
 * THE MARK IS THICKENED; A LOCKUP IS NOT. `thicken` is exact on a horizontal bar
 * and steps a terminal that is not flat, so LUX takes it cleanly — L, U and X
 * terminate flat — while C, G, S and A notch. "LUX CREDIT" and its siblings are
 * set at the same weight, width and tracking with `thicken: 0`; their bars run a
 * little lighter than the mark's, which is the ordinary relationship between a
 * logotype and the words beside it.
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
  { name: 'lux',   text: 'LUX',   wght: 625, scaleX: 1.576, track: -0.095, flatten: 1, thicken: 78, diag: 750, ux: -0.04 },
  { name: 'hanzo', text: 'Hanzo', wght: 600, scaleX: 1.000, track: -0.0286, flatten: 0, thicken: 0, diag: 0, ux: 0 },
  { name: 'zoo',   text: 'zoo',   wght: 800, scaleX: 1.000, track: -0.025, flatten: 0, thicken: 0, diag: 0, ux: 0 },

  // The Lux family lockups. Same voice as the mark, no thicken — see above.
  ...['LUX CREDIT', 'LUX FINANCE', 'LUX BANK', 'LUX FUND', 'LUX EXCHANGE'].map((text) => ({
    name: text.toLowerCase().replace(/ /g, '-'), text,
    wght: 625, scaleX: 1.576, track: -0.095, flatten: 1, thicken: 0, diag: 750, ux: -0.04,
  })),
]

mkdirSync(OUT, { recursive: true })
for (const m of MARKS) {
  const svg = execFileSync(ZEN, ['mark', FONT, m.text,
    String(m.wght), String(m.scaleX), String(m.track), String(m.flatten),
    String(m.thicken), String(m.diag), '1.0', String(m.ux ?? 0)],
    { encoding: 'utf8', maxBuffer: 8 << 20 })
  writeFileSync(join(OUT, `${m.name}.svg`), svg)
  console.log(`  ${m.name.padEnd(6)} ${m.text.padEnd(6)} wght ${m.wght} · ${m.scaleX}× · ` +
    `${m.track}em${m.thicken ? ` · thicken ${m.thicken}` : ''}   ${(svg.length / 1024).toFixed(1)} KB`)
}
