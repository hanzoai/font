/* The brand font sites, from one generator.
 *
 * One typeface, one page shape, a preset pack per brand. The sites are the same
 * page with a different pack, because that IS the claim: the difference between
 * two surfaces is a handful of numbers over one file, not two families. Writing
 * the sites by hand would quietly contradict the thing they exist to show.
 *
 * A brand's page speaks for that brand only. Nothing here reads across —
 * a pack names its own settings and its own voice, never another's.
 *
 * The faces are embedded as data URIs so the page is one file with no asset
 * paths to get wrong, and renders the same from disk, from S3 and from an
 * artifact host. Two things sit beside it instead, because neither is needed to
 * READ the page: the wasm cutter, and the variable fonts as .ttf. The page is
 * set in woff2 — the wire format — and a cutter reads sfnt, so it fetches the
 * ttf of the same build at the moment someone asks for a file.
 *
 * Run: node sites/build.mjs   ->  sites/dist/<brand>/
 */
import { readFileSync, writeFileSync, mkdirSync, copyFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { PRESETS } from '../packages/zen/dist/presets.js'

const HERE = dirname(fileURLToPath(import.meta.url))
const PKG = join(HERE, '..', 'packages', 'zen', 'dist')
const FONTS = join(PKG, 'fonts')
const b64 = (p) => readFileSync(p).toString('base64')

/* One build, two representations: woff2 sets the page, ttf feeds the cutter. */
const FILE = { sans: ['zen-sans', 'Zen'], mono: ['zen-mono', 'ZenMono'], pixel: ['zen-pixel', 'ZenPixel'] }
const FACE = Object.fromEntries(Object.entries(FILE).map(
  ([k, [dir, stem]]) => [k, b64(join(FONTS, dir, `${stem}-Variable.woff2`))]))

/* Zen Pixel's five cuts are five stops on ONE axis, so the variable file holds
   all of them and the space between. `ELSH` is the element — the shape the
   bitmap is drawn out of. */
const CUTS = [['Square', 1], ['Circle', 20], ['Grid', 40], ['Triangle', 60], ['Line', 80]]

/* The stylistic sets. A set does not run the same way in every family — Mono
   ships the slashed zero and ss09 takes the slash OUT, where Sans ships the
   plain one and ss09 puts it in — so each family gets its own sentence, and the
   presence of a sentence is what says the set exists there. The toggle shows
   the letter both ways regardless, which is the part that cannot be wrong. */
const FORMS = [
  { tag: 'ss01', letters: 'a', on: {
    sans: 'a, single-storey', mono: 'a, single-storey', pixel: 'a, single-storey' } },
  { tag: 'ss02', letters: 'a', on: {
    sans: 'a, the other single-storey', mono: 'a, the other single-storey',
    pixel: 'a, the other single-storey' } },
  { tag: 'ss03', letters: 'l', on: {
    sans: 'l, with a tail', mono: 'l, a tail instead of a foot', pixel: 'l, with a tail' } },
  { tag: 'ss04', letters: 'R', on: {
    sans: 'R, straight leg', mono: 'R, straight leg', pixel: 'R, straight leg' } },
  { tag: 'ss05', letters: 'I', on: { sans: 'I, with serifs', pixel: 'I, with serifs' } },
  { tag: 'ss06', letters: 'G', on: {
    sans: 'G, with a spur', mono: 'G, with a spur', pixel: 'G, with a spur' } },
  { tag: 'ss07', letters: '→', on: { sans: 'the arrows, redrawn', mono: 'the arrows, redrawn' } },
  { tag: 'ss08', letters: 'Ä', on: {
    sans: 'accents, round dots', mono: 'accents, round dots' } },
  { tag: 'ss09', letters: '0', on: {
    sans: 'the zero takes a slash, the one loses its foot',
    mono: 'the zero drops its slash', pixel: 'the zero drops its slash' } },
]

const SIZES = [['Display', 64], ['Heading', 34], ['Subhead', 22], ['Body', 16], ['Small', 13]]

const CODE = `const cut = await zen.cut({
  weight: 497,      // anywhere on 100–900
  forms: ['ss01'],  // baked into the outlines
})
// 0O o · 1lI · rn m · ;:, · {}[]()
for (let i = 0; i <= 0xFF; i++) draw(i)`

/* Each brand names the settings it uses, the ground it sits on, and the one
   stylistic set that carries its voice. The estate default is `normal`, so a
   brand opts into a letterform rather than inheriting one. */
const BRANDS = {
  hanzo: {
    host: 'font.hanzo.ai', name: 'Hanzo', family: 'Zen',
    lede: 'One variable file, five presets and five pixel cuts — every Hanzo ' +
          'surface reads its type from the same source.',
    bg: '#050506', fg: '#ededea', dim: '#8b8b86', faint: '#5b5b57', accent: '#9a9a94',
    line: 'rgba(255,255,255,.085)', line2: 'rgba(255,255,255,.17)', sunk: '#000',
    raised: '#101014', glow: 'rgba(255,255,255,.06)',
    feature: 'ss01', featureNote: 'single-storey a — the neo-grotesque register',
    display: 'air', body: 'book', pixel: 'Square',
    headline: 'The open AI cloud',
    uses: ['air', 'book', 'medium'],
    specimen: 'Hanzo',
  },
  lux: {
    host: 'font.lux.network', name: 'Lux', family: 'Zen',
    lede: 'The typeface behind the network. Monochrome, monumental, and the same ' +
          'single file every other surface sets its type from.',
    bg: '#050506', fg: '#f6f6f4', dim: '#8b8b87', faint: '#5a5a56', accent: '#c8c8c4',
    line: 'rgba(255,255,255,.09)', line2: 'rgba(255,255,255,.18)', sunk: '#000',
    raised: '#0f0f13', glow: 'rgba(255,255,255,.06)',
    feature: 'ss04', featureNote: 'straight-leg R — squared, to sit under the wordmark',
    display: 'wide', body: 'book', pixel: null,
    headline: 'SOVEREIGN',
    uses: ['wide', 'book', 'medium'],
    specimen: 'LUX',
  },
  zoo: {
    host: 'font.zoo.ngo', name: 'Zoo', family: 'Zen',
    lede: 'The typeface behind open research. Big, round and warm — set on daylight ' +
          'rather than black, because the audience is everyone.',
    bg: '#fdf4e3', fg: '#191712', dim: '#5d564a', faint: '#8b8375', accent: '#e0622d',
    line: 'rgba(0,0,0,.10)', line2: 'rgba(0,0,0,.20)', sunk: '#fffaf0',
    raised: '#f7eeda', glow: 'rgba(0,0,0,.05)',
    feature: null, featureNote: 'Zen as drawn — the humanist default, and the warmest',
    display: 'round', body: 'book', pixel: 'Grid',
    headline: 'Science for everyone!',
    uses: ['round', 'book', 'medium'],
    specimen: 'Zoo',
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

function page(b) {
  const feat = b.feature ? `font-feature-settings:'${b.feature}' 1;` : ''

  const presetRows = Object.entries(PRESETS).map(([n, p]) => {
    const mine = b.uses.includes(n)
    return `<tr${mine ? ' class="mine"' : ''} data-preset="${n}" tabindex="0">
      <td><code>.zen-${n}</code>${mine ? ' <span class="tag">in use</span>' : ''}</td>
      <td>${p.wght}</td><td>${p.scaleX === 1 ? '—' : num(p.scaleX)}</td>
      <td>${p.track ? num(p.track) + 'em' : '—'}</td>
      <td class="l">${p.note}</td></tr>`
  }).join('\n')

  const scale = SIZES.map(([lab, s]) =>
    `<div class="row"><span class="lab">${lab} · ${s}px</span>
     <span class="spec" style="font-size:${s}px">${b.name}${s >= 34 ? '' : ' sets type in Zen'}</span></div>`
  ).join('\n')

  const forms = FORMS.map((f) =>
    `<button class="form" data-tag="${f.tag}" data-on="${JSON.stringify(f.on).replace(/"/g, '&quot;')}"
      type="button"><span class="off">${f.letters}</span><span class="on"
      style="font-feature-settings:'${f.tag}' 1">${f.letters}</span
      ><em>${f.tag}</em></button>`).join('\n')

  const cuts = CUTS.map(([n, v]) =>
    `<button class="cut" data-elsh="${v}" type="button"><span
      style="font-variation-settings:'ELSH' ${v}">Aa</span><em>${n}</em></button>`).join('\n')

  return `<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>${b.family} · ${b.name}</title>
<meta name="description" content="${b.family}, the ${b.name} typeface — one variable file, brand presets, self-hosted. Cut your own static font in the browser.">
<style>
@font-face{font-family:'Zen';src:url(data:font/woff2;base64,${FACE.sans}) format('woff2');
  font-weight:100 900;font-style:normal;font-display:block}
@font-face{font-family:'Zen Mono';src:url(data:font/woff2;base64,${FACE.mono}) format('woff2');
  font-weight:100 900;font-style:normal;font-display:block}
@font-face{font-family:'Zen Pixel';src:url(data:font/woff2;base64,${FACE.pixel}) format('woff2');
  font-display:block}
:root{--bg:${b.bg};--fg:${b.fg};--dim:${b.dim};--faint:${b.faint};--ac:${b.accent};
  --line:${b.line};--line2:${b.line2};--sunk:${b.sunk};--raised:${b.raised};--glow:${b.glow}}
*{box-sizing:border-box}
html{-webkit-text-size-adjust:100%;scroll-behavior:smooth}
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
button{font:inherit;color:inherit;background:none;border:0;cursor:pointer}
.eyebrow{display:block;font-size:11.5px;letter-spacing:.16em;text-transform:uppercase;
  color:var(--faint);font-variation-settings:'wght' 600;margin-bottom:12px}
header{padding:80px 0 54px;border-bottom:1px solid var(--line);overflow:hidden}
.hero{${css(PRESETS[b.display])};font-size:clamp(42px,${b.display === 'wide' ? 6.4 : 8.6}vw,${b.display === 'wide' ? 84 : 104}px);
  line-height:${b.display === 'round' ? '.96' : '1.02'};display:block;margin-bottom:26px;
  animation:breathe 14s ease-in-out infinite}
@keyframes breathe{
  0%,100%{font-variation-settings:'wght' ${PRESETS[b.display].wght}}
  50%{font-variation-settings:'wght' ${Math.min(900, PRESETS[b.display].wght + 320)}}}
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
tbody tr{cursor:pointer;transition:background .18s}
tbody tr:hover,tbody tr:focus-visible{background:var(--glow);outline:0}
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

/* ── the editor ─────────────────────────────────────────────── */
.editor{border:1px solid var(--line2);border-radius:4px;overflow:hidden;background:var(--sunk)}
.tabs{display:flex;border-bottom:1px solid var(--line)}
.tabs button{padding:13px 20px;font-size:12px;letter-spacing:.11em;text-transform:uppercase;
  color:var(--faint);font-variation-settings:'wght' 600;border-bottom:1px solid transparent;
  margin-bottom:-1px;transition:color .18s,border-color .18s}
.tabs button[aria-selected=true]{color:var(--fg);border-bottom-color:var(--ac)}
.canvas{padding:44px 26px;min-height:190px;display:flex;align-items:center;
  overflow:hidden;border-bottom:1px solid var(--line)}
#type{outline:0;width:100%;line-height:1.1;word-break:break-word;
  transform-origin:left center;caret-color:var(--ac)}
.knobs{display:grid;grid-template-columns:repeat(auto-fit,minmax(215px,1fr));gap:2px;
  background:var(--line);border-bottom:1px solid var(--line)}
.knob{background:var(--sunk);padding:15px 20px 17px}
.knob label{display:flex;justify-content:space-between;font-size:10.5px;letter-spacing:.1em;
  text-transform:uppercase;color:var(--faint);font-variation-settings:'wght' 600;margin-bottom:9px}
.knob label b{color:var(--fg);font-variation-settings:'wght' 500;letter-spacing:0;
  text-transform:none;font-size:12px}
input[type=range]{-webkit-appearance:none;appearance:none;width:100%;height:2px;
  background:var(--line2);outline:0}
input[type=range]::-webkit-slider-thumb{-webkit-appearance:none;width:13px;height:13px;
  border-radius:50%;background:var(--fg);cursor:grab;border:0}
input[type=range]::-moz-range-thumb{width:13px;height:13px;border-radius:50%;
  background:var(--fg);cursor:grab;border:0}
.forms{display:flex;flex-wrap:wrap;gap:2px;background:var(--line);border-bottom:1px solid var(--line)}
.form{flex:1 0 78px;background:var(--sunk);padding:12px 8px 9px;text-align:center;
  transition:background .18s,opacity .18s}
.form:hover{background:var(--glow)}
.form[hidden]{display:none}
.form span{display:block;font-size:29px;line-height:1.15;transition:opacity .2s}
.form .on{margin-top:-1.15em;color:var(--ac);opacity:0}
.form em{display:block;font-family:'Zen Mono',monospace;font-style:normal;font-size:10px;
  letter-spacing:.06em;color:var(--faint);margin-top:4px}
.form[aria-pressed=true] .off{opacity:0}
.form[aria-pressed=true] .on{opacity:1}
.form[aria-pressed=true] em{color:var(--ac)}
.take{display:flex;flex-wrap:wrap;gap:2px;background:var(--line)}
.take button{flex:1 1 200px;background:var(--sunk);padding:16px;font-size:13px;
  font-variation-settings:'wght' 500;transition:background .18s}
.take button:hover{background:var(--glow)}
.take button:disabled{color:var(--faint);cursor:default;background:var(--sunk)}
.take small{display:block;font-size:11px;color:var(--faint);font-variation-settings:'wght' 400;
  margin-top:3px;letter-spacing:0}

.cuts{display:flex;flex-wrap:wrap;gap:2px;background:var(--line);
  border:1px solid var(--line);border-radius:3px;overflow:hidden}
.cut{flex:1 0 96px;background:var(--sunk);padding:20px 10px 12px;text-align:center;
  font-family:'Zen Pixel';font-size:31px;line-height:1;transition:background .18s,color .18s}
.cut:hover,.cut[aria-pressed=true]{background:var(--glow);color:var(--ac)}
.cut em{display:block;font-family:'Zen',sans-serif;font-style:normal;font-size:10.5px;
  letter-spacing:.1em;text-transform:uppercase;color:var(--faint);margin-top:11px}
.morph{font-family:'Zen Pixel';font-size:clamp(34px,7vw,72px);line-height:1.2;
  color:var(--ac);margin:22px 0 6px;display:block;
  animation:elsh 18s linear infinite}
@keyframes elsh{
  0%,100%{font-variation-settings:'ELSH' 1}
  25%{font-variation-settings:'ELSH' 20}
  50%{font-variation-settings:'ELSH' 40}
  75%{font-variation-settings:'ELSH' 60}
  90%{font-variation-settings:'ELSH' 80}}
.pxline{font-family:'Zen Pixel';font-size:31px;color:var(--ac);margin-top:14px}

.term{background:var(--sunk);border:1px solid var(--line);border-radius:3px;overflow:hidden}
.term .bar{display:flex;gap:7px;padding:12px 15px;border-bottom:1px solid var(--line)}
.term .bar i{width:9px;height:9px;border-radius:50%;background:var(--line2)}
.term pre{margin:0;padding:20px 22px;overflow-x:auto;font-family:'Zen Mono',monospace;
  font-size:13.5px;line-height:1.75;color:var(--dim);tab-size:2}
.term b{color:var(--fg);font-weight:500}
.term u{color:var(--ac);text-decoration:none}
.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(128px,1fr));gap:2px;
  background:var(--line);margin-top:2px}
.grid div{background:var(--sunk);padding:15px 16px;font-family:'Zen Mono',monospace;font-size:19px}
.grid span{display:block;font-family:'Zen',sans-serif;font-size:10.5px;letter-spacing:.1em;
  text-transform:uppercase;color:var(--faint);margin-top:7px}

.live .reveal{opacity:0;transform:translateY(14px);transition:opacity .7s ease,transform .7s ease}
.live .reveal.seen{opacity:1;transform:none}
@media(prefers-reduced-motion:reduce){
  *,*::before,*::after{animation:none!important;transition:none!important}
  .live .reveal{opacity:1;transform:none}
  html{scroll-behavior:auto}}
</style>
<script>document.documentElement.classList.add('live')</script>

<div class="page">
<header>
  <span class="eyebrow">${b.host}</span>
  <span class="hero">${b.headline}</span>
  <p>${b.lede}</p>
  ${b.pixel ? `<div class="pxline">${b.name.toUpperCase()} ${b.family.toUpperCase()}</div>` : ''}
</header>

<section class="reveal">
  <div class="head"><span class="eyebrow">01 · Cut your own</span>
    <h2>Move it, then take it</h2></div>
  <p>Every setting below is live type, not a picture of it. When it reads right,
  take the file: the cutter runs <strong>in this tab</strong> — it walks the
  outlines at your weight, bakes in the letterforms you picked, and hands back a
  static font. Nothing is uploaded, and no server sees the setting.</p>

  <div class="editor">
    <div class="tabs" role="tablist">
      <button role="tab" data-family="sans" aria-selected="true">Sans</button>
      <button role="tab" data-family="mono" aria-selected="false">Mono</button>
      <button role="tab" data-family="pixel" aria-selected="false">Pixel</button>
    </div>
    <div class="canvas"><div id="type" contenteditable spellcheck="false">${b.specimen}</div></div>
    <div class="knobs">
      <div class="knob" id="k-wght"><label for="wght">Weight <b id="v-wght">400</b></label>
        <input id="wght" type="range" min="100" max="900" step="1" value="400"></div>
      <div class="knob" id="k-elsh" hidden><label for="elsh">Element <b id="v-elsh">1</b></label>
        <input id="elsh" type="range" min="1" max="80" step="1" value="1"></div>
      <div class="knob"><label for="size">Size <b id="v-size">86px</b></label>
        <input id="size" type="range" min="14" max="180" step="1" value="86"></div>
      <div class="knob"><label for="track">Tracking <b id="v-track">0em</b></label>
        <input id="track" type="range" min="-100" max="60" step="1" value="0"></div>
      <div class="knob" id="k-width"><label for="width">Width <b id="v-width">1.00</b></label>
        <input id="width" type="range" min="70" max="200" step="1" value="100"></div>
    </div>
    <div class="forms">${forms}</div>
    <div class="take">
      <button id="take-cut">Take this cut<small id="cut-note">a static .ttf at these settings</small></button>
      <button id="take-var">Take the variable font<small id="var-note">one file, weight 100–900</small></button>
    </div>
  </div>
  <p class="note"><strong>Width and tracking travel with the file.</strong>
  <code>transform:scaleX()</code> stretches what is painted and leaves the layout box
  where it was; a font that is really wider carries the width in its advances too.
  So a cut taken here needs no CSS to keep the setting you gave it.</p>
</section>

<section class="reveal">
  <div class="head"><span class="eyebrow">02 · The pack</span>
    <h2>${b.name}'s presets</h2></div>
  <p>Five presets ship with the family. ${b.name} uses
  <strong>${b.uses.map((u) => '.zen-' + u).join('</strong>, <strong>')}</strong> —
  the others are there when a surface needs them. Each is one point in Zen's
  parameter space. <strong>Click a row</strong> to load it above.</p>
  <div class="scroll"><table>
    <thead><tr><th>Preset</th><th>wght</th><th>scaleX</th><th>track</th><th class="l">What it is for</th></tr></thead>
    <tbody>${presetRows}</tbody>
  </table></div>
  <p class="note"><strong>Voice:</strong> ${b.feature ? `<code>${b.feature}</code> — ` : ''}${b.featureNote}.
  The estate default is <code>normal</code>, so a brand opts into its letterform.</p>
</section>

<section class="reveal">
  <div class="head"><span class="eyebrow">03 · Zen Mono</span>
    <h2>Cut for the terminal</h2></div>
  <p>Every glyph is 0.6 em wide, so a column of code is a column. The pairs a
  screen of code turns on are kept apart out of the box: <strong>the zero comes
  slashed</strong>, and <code>1</code>, <code>l</code> and <code>I</code> each
  stand on a foot, so none of them is a bare stroke.</p>
  <div class="term">
    <div class="bar"><i></i><i></i><i></i></div>
    <pre>${CODE.replace(/</g, '&lt;')}</pre>
  </div>
  <div class="grid">
    <div>0 O o<span>zero, O, o</span></div>
    <div>1 l I |<span>one, l, I, bar</span></div>
    <div>rn m<span>r n, m</span></div>
    <div>; : , .<span>the small marks</span></div>
    <div>{ } [ ] ( )<span>the brackets</span></div>
    <div>=&gt; !== &lt;=<span>the operators</span></div>
  </div>
  <p class="note">The sets go the other way here, because the defaults already
  favour code: <code>ss09</code> takes the slash back out of the zero and
  <code>ss03</code> swaps the l's foot for a tail. Both are in the editor above,
  and both survive into a cut you take.</p>
</section>

<section class="reveal">
  <div class="head"><span class="eyebrow">04 · Zen Pixel</span>
    <h2>Five cuts, one axis</h2></div>
  <p>Pixel is drawn out of an element — a square, a circle, a triangle — and the
  element is an <strong>axis</strong>, not five files. The named cuts are five stops
  on it, and everything between them is a cut too.</p>
  <span class="morph">${b.name.toUpperCase()} ZEN PIXEL</span>
  <div class="cuts">${cuts}</div>
  <p class="note">The strip above runs the axis end to end. Pick a stop to load it
  into the editor, or drag <strong>Element</strong> there for the space between.</p>
</section>

<section class="reveal">
  <div class="head"><span class="eyebrow">05 · The scale</span>
    <h2>One file, every size</h2></div>
  <div class="stack">${scale}</div>
  <p class="note">Zen is variable on <code>wght</code> 100–900, so one file covers every
  weight — no nine static cuts, and no second request when a page needs a bolder line.</p>
</section>

<section class="reveal">
  <div class="head"><span class="eyebrow">06 · The set</span>
    <h2>Characters</h2></div>
  <div class="glyphs">ABCDEFGHIJKLMNOPQRSTUVWXYZ<br>abcdefghijklmnopqrstuvwxyz<br>
    0123456789 &amp;@#$%^*()[]{}&lt;&gt;/\\|+−=~ ‘’“” — –</div>
  <div class="glyphs" style="margin-top:2px;font-family:'Zen Mono'">
    Zen Mono &nbsp; const zen = 0xFF1l0O; &nbsp; i++ =&gt; !== &lt;=</div>
</section>

<section class="reveal">
  <div class="head"><span class="eyebrow">07 · Using it</span>
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
</div>

<script type="module">
${editor(b)}
</script>
`
}

/* The editor. Plain module JS — the page has no framework and needs none, and
   the cutter is fetched only when someone first asks for a file. */
const editor = (b) => `
const $ = (id) => document.getElementById(id)
const type = $('type'), knobs = { wght: $('wght'), elsh: $('elsh'), size: $('size'), track: $('track'), width: $('width') }
const PRESETS = ${JSON.stringify(PRESETS)}
const CUTS = ${JSON.stringify(Object.fromEntries(CUTS))}
const FACE = { sans: 'Zen', mono: 'Zen Mono', pixel: 'Zen Pixel' }
const state = { family: 'sans', forms: new Set() }

function draw() {
  const px = +knobs.size.value, track = +knobs.track.value / 1000, width = +knobs.width.value / 100
  const pixel = state.family === 'pixel'
  const axis = pixel ? \`'ELSH' \${knobs.elsh.value}\` : \`'wght' \${knobs.wght.value}\`
  Object.assign(type.style, {
    fontFamily: \`'\${FACE[state.family]}', \` + (state.family === 'mono' ? 'monospace' : 'sans-serif'),
    fontVariationSettings: axis,
    fontFeatureSettings: [...state.forms].map((t) => \`'\${t}' 1\`).join(',') || 'normal',
    fontSize: px + 'px',
    letterSpacing: track + 'em',
    transform: width === 1 ? 'none' : \`scaleX(\${width})\`,
  })
  $('v-wght').textContent = knobs.wght.value
  $('v-elsh').textContent = knobs.elsh.value
  $('v-size').textContent = px + 'px'
  $('v-track').textContent = track.toFixed(3).replace(/0+$/, '').replace(/\\.$/, '') + 'em'
  $('v-width').textContent = width.toFixed(2)
  $('k-wght').hidden = pixel
  $('k-elsh').hidden = !pixel
  $('k-width').hidden = pixel
  $('cut-note').textContent = pixel
    ? 'a static .ttf at this element' : 'a static .ttf at these settings'
  $('var-note').textContent = pixel
    ? 'one file, all five elements' : 'one file, weight 100–900'
  for (const f of document.querySelectorAll('.form')) {
    const name = JSON.parse(f.dataset.on)[state.family]
    f.hidden = !name
    f.title = name || ''
    f.style.fontFamily = \`'\${FACE[state.family]}'\`
    if (!name && state.forms.delete(f.dataset.tag)) f.setAttribute('aria-pressed', 'false')
  }
}

for (const k of Object.values(knobs)) k.addEventListener('input', draw)

for (const tab of document.querySelectorAll('[role=tab]')) {
  tab.onclick = () => {
    state.family = tab.dataset.family
    for (const t of document.querySelectorAll('[role=tab]')) t.setAttribute('aria-selected', String(t === tab))
    draw()
  }
}

for (const f of document.querySelectorAll('.form')) {
  f.setAttribute('aria-pressed', 'false')
  f.onclick = () => {
    const on = !state.forms.delete(f.dataset.tag)
    if (on) state.forms.add(f.dataset.tag)
    f.setAttribute('aria-pressed', String(on))
    draw()
  }
}

for (const row of document.querySelectorAll('tr[data-preset]')) {
  const load = () => {
    const p = PRESETS[row.dataset.preset]
    state.family = 'sans'
    for (const t of document.querySelectorAll('[role=tab]')) t.setAttribute('aria-selected', String(t.dataset.family === 'sans'))
    knobs.wght.value = p.wght
    knobs.track.value = Math.round(p.track * 1000)
    knobs.width.value = Math.round(p.scaleX * 100)
    draw()
    document.querySelector('.editor').scrollIntoView({ block: 'center' })
  }
  row.onclick = load
  row.onkeydown = (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); load() } }
}

for (const cut of document.querySelectorAll('.cut')) {
  cut.onclick = () => {
    state.family = 'pixel'
    for (const t of document.querySelectorAll('[role=tab]')) t.setAttribute('aria-selected', String(t.dataset.family === 'pixel'))
    knobs.elsh.value = cut.dataset.elsh
    for (const c of document.querySelectorAll('.cut')) c.setAttribute('aria-pressed', String(c === cut))
    draw()
    document.querySelector('.editor').scrollIntoView({ block: 'center' })
  }
}

/* Reveal on scroll. The page is readable without it — the class only ever
   removes the offset, so a browser with no observer shows everything. */
if ('IntersectionObserver' in window) {
  const seen = new IntersectionObserver((rows) => {
    for (const r of rows) if (r.isIntersecting) { r.target.classList.add('seen'); seen.unobserve(r.target) }
  }, { rootMargin: '-40px' })
  for (const s of document.querySelectorAll('.reveal')) seen.observe(s)
} else {
  for (const s of document.querySelectorAll('.reveal')) s.classList.add('seen')
}

/* The page is set in woff2 and the cutter reads sfnt, so the ttf of the same
   build sits beside the page and is fetched the first time it is needed. */
const FILE = { sans: 'Zen', mono: 'ZenMono', pixel: 'ZenPixel' }
const SOURCE = {}
async function bytes(family) {
  if (!SOURCE[family]) {
    const r = await fetch(\`./fonts/\${FILE[family]}-Variable.ttf\`)
    if (!r.ok) throw new Error(\`no source for \${family}\`)
    SOURCE[family] = new Uint8Array(await r.arrayBuffer())
  }
  return SOURCE[family]
}

const NAMES = [[100,'Thin'],[200,'ExtraLight'],[300,'Light'],[400,'Regular'],[500,'Medium'],
  [600,'SemiBold'],[700,'Bold'],[800,'ExtraBold'],[900,'Black']]
const near = (rows, v) => rows.reduce((a, r) => Math.abs(r[0] - v) < Math.abs(a[0] - v) ? r : a)[1]

function save(data, name) {
  const url = URL.createObjectURL(new Blob([data], { type: 'font/ttf' }))
  const a = Object.assign(document.createElement('a'), { href: url, download: name })
  document.body.append(a); a.click(); a.remove()
  setTimeout(() => URL.revokeObjectURL(url), 4000)
}

let zen
$('take-cut').onclick = async (e) => {
  const button = e.currentTarget, was = button.firstChild.textContent
  button.firstChild.textContent = 'Cutting…'
  try {
    if (!zen) {
      zen = await import('./wasm/zen.js')
      await zen.default({ module_or_path: './wasm/zen_bg.wasm' })
    }
    const pixel = state.family === 'pixel'
    const width = pixel ? 1 : +knobs.width.value / 100
    const track = +knobs.track.value / 1000
    const at = pixel ? 'ELSH=' + knobs.elsh.value : 'wght=' + knobs.wght.value
    const style = pixel ? 'Regular' : near(NAMES, +knobs.wght.value)
    const family = pixel
      ? 'Zen Pixel ' + near(Object.entries(CUTS).map(([n, v]) => [v, n]), +knobs.elsh.value)
      : FACE[state.family] + (width === 1 ? '' : ' Wide')
    const file = zen.cut(await bytes(state.family), at, width, track,
      [...state.forms].join(','), family, style,
      'Copyright 2026 Hanzo AI, Inc. (https://font.hanzo.ai)')
    save(file, family.replace(/ /g, '') + '-' + style + '.ttf')
  } catch (err) {
    button.firstChild.textContent = 'Could not cut it'
    console.error(err)
    return setTimeout(() => { button.firstChild.textContent = was }, 2600)
  }
  button.firstChild.textContent = was
}

$('take-var').onclick = async () => {
  save(await bytes(state.family), FILE[state.family] + '-Variable.ttf')
}

draw()
`

mkdirSync(join(HERE, 'dist'), { recursive: true })
for (const [key, b] of Object.entries(BRANDS)) {
  const dir = join(HERE, 'dist', key)
  mkdirSync(join(dir, 'wasm'), { recursive: true })
  mkdirSync(join(dir, 'fonts'), { recursive: true })
  const html = page(b)
  writeFileSync(join(dir, 'index.html'), html)
  for (const f of ['zen.js', 'zen_bg.wasm']) copyFileSync(join(PKG, 'wasm', f), join(dir, 'wasm', f))
  for (const [d, stem] of Object.values(FILE)) {
    copyFileSync(join(FONTS, d, `${stem}-Variable.ttf`), join(dir, 'fonts', `${stem}-Variable.ttf`))
  }
  console.log(`  ${b.host.padEnd(20)} ${(html.length / 1024).toFixed(0)} KB  ${dir}`)
}
