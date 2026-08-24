/* The three brand font sites, from one generator.
 *
 *   font.hanzo.ai · font.lux.network · font.zoo.ngo
 *
 * One typeface, three voices. The sites are the same page with a different preset
 * pack, because that IS the claim: the difference between the brands is a handful
 * of numbers over one file, not three families. Writing three hand-authored sites
 * would quietly contradict the thing they exist to demonstrate.
 *
 * The faces are embedded as data URIs so a page is one file with no asset paths to
 * get wrong on a static plane, and so it renders identically from disk, from S3 and
 * from an artifact host.
 *
 * Run: node sites/build.mjs   ->  sites/dist/{hanzo,lux,zoo}/index.html
 */
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { PRESETS } from '../packages/zen/dist/presets.js'

const HERE = dirname(fileURLToPath(import.meta.url))
const FONTS = join(HERE, '..', 'packages', 'zen', 'dist', 'fonts')
const b64 = (p) => readFileSync(p).toString('base64')

const SANS = b64(join(FONTS, 'zen-sans', 'Zen-Variable.woff2'))
const MONO = b64(join(FONTS, 'zen-mono', 'ZenMono-Variable.woff2'))
const PX = {
  Square: b64(join(FONTS, 'zen-pixel', 'ZenPixel-Square.woff2')),
  Grid: b64(join(FONTS, 'zen-pixel', 'ZenPixel-Grid.woff2')),
}

/* Each brand names the presets it uses and the ground it sits on. `feature` is the
   one stylistic set that carries its voice — the estate default is `normal`, so a
   brand opts in rather than inheriting somebody else's letterform. */
const BRANDS = {
  hanzo: {
    host: 'font.hanzo.ai', name: 'Hanzo', family: 'Zen',
    lede: 'One variable file, five presets and five pixel cuts — every Hanzo ' +
          'surface reads its type from the same source.',
    bg: '#050506', fg: '#ededea', dim: '#8b8b86', faint: '#5b5b57', accent: '#9a9a94',
    line: 'rgba(255,255,255,.085)', line2: 'rgba(255,255,255,.17)', sunk: '#000',
    raised: '#101014',
    feature: 'ss01', featureNote: 'single-storey a — the neo-grotesque register',
    display: 'air', body: 'book', pixel: 'Square',
    headline: 'The open AI cloud',
    uses: ['air', 'book', 'medium'],
  },
  lux: {
    host: 'font.lux.network', name: 'Lux', family: 'Zen',
    lede: 'The typeface behind the network. Monochrome, monumental, and the same ' +
          'single file every other Lux surface sets its type from.',
    bg: '#050506', fg: '#f6f6f4', dim: '#8b8b87', faint: '#5a5a56', accent: '#c8c8c4',
    line: 'rgba(255,255,255,.09)', line2: 'rgba(255,255,255,.18)', sunk: '#000',
    raised: '#0f0f13',
    feature: 'ss04', featureNote: 'straight-leg R — squared, to sit under the wordmark',
    display: 'wide', body: 'book', pixel: null,
    headline: 'SOVEREIGN',
    uses: ['wide', 'book', 'medium'],
  },
  zoo: {
    host: 'font.zoo.ngo', name: 'Zoo', family: 'Zen',
    lede: 'The typeface behind open research. Big, round and warm — set on daylight ' +
          'rather than black, because the audience is everyone.',
    bg: '#fdf4e3', fg: '#191712', dim: '#5d564a', faint: '#8b8375', accent: '#e0622d',
    line: 'rgba(0,0,0,.10)', line2: 'rgba(0,0,0,.20)', sunk: '#fffaf0',
    raised: '#f7eeda',
    feature: null, featureNote: 'Zen as drawn — the humanist default, and the warmest',
    display: 'round', body: 'book', pixel: 'Grid',
    headline: 'Science for everyone!',
    uses: ['round', 'book', 'medium'],
    /* Zoo's wordmark is TYPE, not artwork — `components/Logo.tsx` renders the
       Venn mark beside two live spans, `ZOO` at extrabold and the property name
       at light. So unlike Lux, whose mark is fixed SVG geometry that no font
       change can touch, moving Zoo to Zen MOVES ITS WORDMARK. That is worth
       showing rather than discovering. */
    wordmark: { lead: 'ZOO', tail: 'INDUSTRIES', lead_w: 800, tail_w: 300, track: -0.025 },
  },
}

/* The number, not a rounding of it. This table is where a reader copies the
   value FROM, so a rounding here becomes a different number in their stylesheet
   — toFixed(2) printed 1.49 for a preset the CSS and the crate both set to
   1.4861. */
const num = (n) => String(n)

const css = (p) => {
  const out = [`font-variation-settings:'wght' ${p.wght}`]
  if (p.track) out.push(`letter-spacing:${p.track}em`)
  if (p.scaleX !== 1) out.push(
    `transform:scaleX(${p.scaleX})`, 'transform-origin:left center', 'display:inline-block')
  return out.join(';')
}

const SIZES = [['Display', 64], ['Heading', 34], ['Subhead', 22], ['Body', 16], ['Small', 13]]

function page(key, b) {
  const px = b.pixel
    ? `@font-face{font-family:'Zen Pixel';src:url(data:font/woff2;base64,${PX[b.pixel]}) format('woff2');font-display:block}`
    : ''
  const feat = b.feature ? `font-feature-settings:'${b.feature}' 1;` : ''

  const presetRows = Object.entries(PRESETS).map(([n, p]) => {
    const mine = b.uses.includes(n)
    return `<tr${mine ? ' class="mine"' : ''}>
      <td><code>.zen-${n}</code>${mine ? ' <span class="tag">in use</span>' : ''}</td>
      <td>${p.wght}</td><td>${p.scaleX === 1 ? '—' : num(p.scaleX)}</td>
      <td>${p.track ? num(p.track) + 'em' : '—'}</td>
      <td class="l">${p.note}</td></tr>`
  }).join('\n')

  const scale = SIZES.map(([lab, s]) =>
    `<div class="row"><span class="lab">${lab} · ${s}px</span>
     <span class="spec" style="font-size:${s}px">${b.name} ${s >= 34 ? '' : 'sets type in Zen'}</span></div>`
  ).join('\n')

  const wideNote = b.display === 'wide'
    ? `<p class="note"><strong>Zen Wide transforms</strong>, so its layout box stays the
       untransformed width — give it room or clip its container. It is set to sit with
       <strong>the LUX wordmark</strong>: same width, same stem, within
       ${(PRESETS.wide.within * 100).toFixed(1)}% on every measurement. It was fitted to a
       licensed display face first, which is a different target, and beside the very
       mark it was meant to accompany it ran 22% fat in the stems.</p>
      <p class="note">The wordmark itself is <strong>set, not drawn</strong>, and it
       needs one thing this preset cannot: the drawn LUX is nearly monoline — its bars
       are 0.90 of its stems where Zen's are 0.58 — and no weight reaches that, because
       weight moves bars and stems together. Cutting the mark adds the bars back as an
       outline operation. Type set beside it therefore carries slightly lighter
       horizontals than the mark does, which is the ordinary relationship between a
       logo and its companion text.</p>` : ''

  const wm = b.wordmark
  const wmSection = !wm ? '' : `
<section>
  <div class="head"><span class="eyebrow">05 · The wordmark</span>
    <h2>Set, not drawn</h2></div>
  <p>${b.name}'s wordmark is <strong>live type</strong> — the mark beside two spans,
  the name at ${wm.lead_w} and the property at ${wm.tail_w}. It is not artwork, so it is not
  frozen: changing the family changes the wordmark, which is why it belongs on this
  page rather than in a folder of SVGs.</p>
  <div class="stack" style="padding:34px 24px">
    <div class="lockup"><span class="lead">${wm.lead}</span><span class="tail">&nbsp;${wm.tail}</span></div>
    <div class="lockup" style="font-size:26px;margin-top:22px"
      ><span class="lead">${wm.lead}</span><span class="tail">&nbsp;LABS</span></div>
  </div>
  <p class="note">Two weights off one axis — no second file, and every property that
  reuses the lockup inherits the same pair. The tracking is ${wm.track}em, tight enough
  that the name reads as one object rather than two words.</p>
</section>`

  return `<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>${b.family} · ${b.name}</title>
<meta name="description" content="${b.family}, the ${b.name} typeface — one variable file, brand presets, self-hosted.">
<style>
@font-face{font-family:'Zen';src:url(data:font/woff2;base64,${SANS}) format('woff2');
  font-weight:100 900;font-style:normal;font-display:block}
@font-face{font-family:'Zen Mono';src:url(data:font/woff2;base64,${MONO}) format('woff2');
  font-weight:100 900;font-style:normal;font-display:block}
${px}
:root{--bg:${b.bg};--fg:${b.fg};--dim:${b.dim};--faint:${b.faint};--ac:${b.accent};
  --line:${b.line};--line2:${b.line2};--sunk:${b.sunk};--raised:${b.raised}}
*{box-sizing:border-box}
html{-webkit-text-size-adjust:100%}
body{margin:0;background:var(--bg);color:var(--fg);
  font-family:'Zen',ui-sans-serif,system-ui,sans-serif;
  font-size:16px;line-height:1.62;letter-spacing:-.004em;${feat}
  font-variant-numeric:tabular-nums;-webkit-font-smoothing:antialiased}
.page{max-width:1080px;margin:0 auto;padding:0 30px 130px}
@media(max-width:720px){.page{padding:0 18px 80px}}
h1,h2{margin:0;line-height:1.06;text-wrap:balance}
h2{font-size:clamp(23px,3.3vw,31px);font-variation-settings:'wght' 500;letter-spacing:-.03em}
p{margin:0 0 17px;max-width:68ch;color:var(--dim)}
strong{font-weight:600;color:var(--fg)}
code,.mono{font-family:'Zen Mono',ui-monospace,monospace;font-size:.92em}
a{color:var(--fg)}
.eyebrow{display:block;font-size:11.5px;letter-spacing:.16em;text-transform:uppercase;
  color:var(--faint);font-variation-settings:'wght' 600;margin-bottom:12px}
header{padding:80px 0 54px;border-bottom:1px solid var(--line);overflow:hidden}
.hero{${css(PRESETS[b.display])};font-size:clamp(42px,${b.display === 'wide' ? 6.4 : 8.6}vw,${b.display === 'wide' ? 84 : 104}px);
  line-height:${b.display === 'round' ? '.96' : '1.02'};display:block;margin-bottom:26px}
header p{font-size:clamp(17px,2.2vw,21px);line-height:1.5;max-width:58ch;
  font-variation-settings:'wght' 300;letter-spacing:-.014em;color:var(--dim)}
section{padding:58px 0;border-bottom:1px solid var(--line)}
section:last-of-type{border-bottom:0}
.head{margin-bottom:30px}
.note{font-size:13.5px;color:var(--dim);max-width:64ch;line-height:1.6;margin-top:16px}
.scroll{overflow-x:auto}
table{border-collapse:collapse;width:100%;min-width:600px;font-size:14.5px}
th,td{text-align:right;padding:9px 14px;border-bottom:1px solid var(--line)}
th:first-child,td:first-child,td.l,th.l{text-align:left}
th:first-child,td:first-child{padding-left:0}
td:last-child,th:last-child{padding-right:0}
thead th{font-size:11px;letter-spacing:.11em;text-transform:uppercase;color:var(--faint);
  font-variation-settings:'wght' 600;border-bottom-color:var(--line2)}
tbody tr:last-child td{border-bottom:0}
tr.mine td{color:var(--fg)}
.tag{font-size:10.5px;letter-spacing:.08em;text-transform:uppercase;color:var(--ac);
  border:1px solid var(--line2);border-radius:2px;padding:1px 6px;margin-left:8px;
  font-variation-settings:'wght' 600}
td.l{color:var(--dim);font-size:13.5px}
.stack{background:var(--sunk);border:1px solid var(--line);border-radius:3px;
  padding:6px 24px;overflow-x:auto}
.row{display:flex;align-items:baseline;gap:24px;padding:15px 0;border-bottom:1px solid var(--line)}
.row:last-child{border-bottom:0}
.lab{font-size:10.5px;letter-spacing:.1em;text-transform:uppercase;color:var(--faint);
  font-variation-settings:'wght' 600;min-width:112px;flex:none}
.spec{white-space:nowrap;line-height:1.25}
.install{background:var(--raised);border:1px solid var(--line);border-radius:3px;
  padding:20px 22px;font-size:13.5px;line-height:1.9;color:var(--dim);overflow-x:auto}
.install code{color:var(--fg)}
.glyphs{background:var(--sunk);border:1px solid var(--line);border-radius:3px;
  padding:24px;font-size:23px;line-height:1.85;color:var(--dim);word-break:break-word}
.pxline{font-family:'Zen Pixel';font-size:31px;color:var(--ac);margin-top:14px}
.lockup{font-size:clamp(30px,5vw,54px);line-height:1;text-transform:uppercase;
  letter-spacing:${wm ? wm.track : 0}em;white-space:nowrap}
.lockup .lead{font-variation-settings:'wght' ${wm ? wm.lead_w : 800}}
.lockup .tail{font-variation-settings:'wght' ${wm ? wm.tail_w : 300}}
@media(prefers-reduced-motion:reduce){*{transition:none!important}}
</style>

<div class="page">
<header>
  <span class="eyebrow">${b.host}</span>
  <span class="hero">${b.headline}</span>
  <p>${b.lede}</p>
  ${b.pixel ? `<div class="pxline">${b.name.toUpperCase()} ${b.family.toUpperCase()}</div>` : ''}
</header>

<section>
  <div class="head"><span class="eyebrow">01 · The pack</span>
    <h2>${b.name}'s presets</h2></div>
  <p>Five settings ship with the family. ${b.name} uses
  <strong>${b.uses.map((u) => '.zen-' + u).join('</strong>, <strong>')}</strong> —
  and the others are there when a surface needs them. Each is a point in Zen's own
  parameter space, and three were fitted rather than chosen — so those swaps hold on
  measurement rather than on taste.</p>
  <div class="scroll"><table>
    <thead><tr><th>Preset</th><th>wght</th><th>scaleX</th><th>track</th><th class="l">What it is for</th></tr></thead>
    <tbody>${presetRows}</tbody>
  </table></div>
  <p class="note"><strong>Voice:</strong> ${b.feature ? `<code>${b.feature}</code> — ` : ''}${b.featureNote}.
  The estate default is <code>normal</code>, so a brand opts into its letterform rather
  than inheriting another's.</p>
  ${wideNote}
</section>

<section>
  <div class="head"><span class="eyebrow">02 · The scale</span>
    <h2>One file, every size</h2></div>
  <div class="stack">${scale}</div>
  <p class="note">Zen is variable on <code>wght</code> 100–900, so one file covers every
  weight — no nine static cuts, and no second request when a page needs a bolder line.</p>
</section>

<section>
  <div class="head"><span class="eyebrow">03 · The set</span>
    <h2>Characters</h2></div>
  <div class="glyphs">ABCDEFGHIJKLMNOPQRSTUVWXYZ<br>abcdefghijklmnopqrstuvwxyz<br>
    0123456789 &amp;@#$%^*()[]{}&lt;&gt;/\\|+−=~ ‘’“” — –</div>
  <div class="glyphs" style="margin-top:12px;font-family:'Zen Mono'">
    Zen Mono &nbsp; const zen = 0xFF1l0O; &nbsp; i++ =&gt; !== &lt;=</div>
</section>

<section>
  <div class="head"><span class="eyebrow">04 · Using it</span>
    <h2>Two imports</h2></div>
  <div class="install">
    <code>pnpm add @hanzo/font</code><br><br>
    <code>import '@hanzo/font/css'</code> &nbsp;<span style="color:var(--faint)">the faces</span><br>
    <code>import '@hanzo/font/presets.css'</code> &nbsp;<span style="color:var(--faint)">the five presets</span>
  </div>
  <p class="note">Then name the role, never the face: <code>var(--font-sans)</code> and
  <code>var(--font-mono)</code>. A surface on <code>@hanzo/design</code> gets both with
  the token layer and needs no font import at all — which is what keeps every ${b.name}
  property on one version of the type as the family evolves.</p>
</section>
${wmSection}
</div>
`
}

mkdirSync(join(HERE, 'dist'), { recursive: true })
for (const [key, b] of Object.entries(BRANDS)) {
  const dir = join(HERE, 'dist', key)
  mkdirSync(dir, { recursive: true })
  const html = page(key, b)
  writeFileSync(join(dir, 'index.html'), html)
  console.log(`  ${b.host.padEnd(20)} ${(html.length / 1024).toFixed(0)} KB  ${dir}`)
}
