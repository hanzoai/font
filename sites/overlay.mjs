/* Superimpose a Zen cut on the drawn mark, so the fit is READ rather than scored.
 *
 * `zen lux` reports seven numbers and every one can land inside 3% while the
 * mark still looks wrong — a number says how far apart two edges are, not which
 * way the eye reads the shape between them. The overlay says it directly: drawn
 * in magenta, cut in cyan, the agreement in white where they coincide.
 *
 * Both marks are normalised to the same cap and set on the same baseline and
 * left edge, because that is how a logo is swapped in practice — you replace the
 * artwork and everything around it stays put.
 *
 * Run: node sites/overlay.mjs [text] [wght] [scaleX] [track] [thicken] [diag] [ux]
 *   -> sites/dist/overlay/<text>.svg
 */
import { execFileSync } from 'node:child_process'
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))
const ROOT = join(HERE, '..')
const OUT = join(HERE, 'dist', 'overlay')
const FONT = join(ROOT, 'packages/zen/dist/fonts/zen-sans/Zen-Variable.ttf')
const ZEN = join(ROOT, 'crates/zen/target/release/zen')
const DRAWN = process.env.LUX_DRAWN || '/home/z/work/lux/logo/svg/lux-wordmark-white.svg'

const [text = 'LUX', wght = 650, scaleX = 1.5503, track = -0.095,
       thicken = 90, diag = 750, ux = -0.02, foot = 10.13, lu = -0.00494] = process.argv.slice(2)

// Every path/polygon in a file, as one flat list of subpaths in its own units.
const shapes = (svg) => {
  const out = []
  for (const m of svg.matchAll(/<path[^>]*\sd="([^"]+)"/g)) out.push({ kind: 'd', v: m[1] })
  for (const m of svg.matchAll(/<polygon[^>]*\spoints="([^"]+)"/g)) out.push({ kind: 'p', v: m[1] })
  return out
}
const viewBox = (svg) => (svg.match(/viewBox="([^"]+)"/) || [])[1]?.split(/[\s,]+/).map(Number)

// The drawn mark: y DOWN, cap = the viewBox height.
const drawn = readFileSync(DRAWN, 'utf8')
const dvb = viewBox(drawn)
if (!dvb) throw new Error(`${DRAWN} declares no viewBox`)

// The cut: `zen mark` frames on ink, so its viewBox height IS the cap.
const cut = execFileSync(ZEN, ['mark', FONT, text, String(wght), String(scaleX),
  String(track), '1', String(thicken), String(diag), '1.0', String(ux),
  String(foot), String(lu)],
  { encoding: 'utf8', maxBuffer: 8 << 20 })
const cvb = viewBox(cut)

// One cap height for both, and the same origin, so the only difference left on
// screen is shape.
const CAP = 1000
const layer = (svg, vb, fill, opacity) => {
  const k = CAP / vb[3]
  const body = shapes(svg).map((s) => s.kind === 'd'
    ? `<path d="${s.v}"/>`
    : `<polygon points="${s.v}"/>`).join('')
  return `<g transform="translate(${-vb[0] * k} ${-vb[1] * k}) scale(${k})" ` +
         `fill="${fill}" fill-opacity="${opacity}" fill-rule="nonzero">${body}</g>`
}

const w = Math.max(dvb[2] * (CAP / dvb[3]), cvb[2] * (CAP / cvb[3]))
const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${-20} ${-20} ${w + 40} ${CAP + 40}">` +
  `<rect x="${-20}" y="${-20}" width="${w + 40}" height="${CAP + 40}" fill="#000"/>` +
  layer(drawn, dvb, '#ff00aa', 1) +
  layer(cut, cvb, '#00e5ff', 0.55) +
  `</svg>`

mkdirSync(OUT, { recursive: true })
const p = join(OUT, `${text.toLowerCase().replace(/[^a-z0-9]+/g, '-')}.svg`)
writeFileSync(p, svg)
console.log(`${p}  drawn ${dvb.join(' ')}  cut ${cvb.join(' ')}`)
