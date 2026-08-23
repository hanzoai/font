/* The three wordmarks, cut from Zen, each beside its mark.
 *
 * Run after wordmark.mjs: node sites/showcase.mjs -> sites/dist/wordmarks.html
 */
import { readFileSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))
const WM = join(HERE, 'dist', 'wordmark')
const FONTS = join(HERE, '..', 'packages', 'zen', 'dist', 'fonts')

const read = (p) => readFileSync(p, 'utf8')
const b64 = (p) => readFileSync(p).toString('base64')
const SANS = b64(join(FONTS, 'zen-sans', 'Zen-Variable.woff2'))

/* Strip the outer <svg> so a mark can be inlined at a size the page controls. */
const inner = (s) => s.replace(/^[\s\S]*?<svg[^>]*>/, '').replace(/<\/svg>\s*$/, '')
const box = (s) => (s.match(/viewBox="([^"]*)"/) || [])[1]

const zenwm = (n) => {
  const s = read(join(WM, `${n}.svg`))
  return { vb: box(s), body: inner(s) }
}

const LUX_REAL = read('/home/z/work/lux/logo/svg/lux-wordmark-white.svg')
const HANZO_MARK = `<path d="M22.21 67V44.6369H0V67H22.21Z"/><path d="M66.7038 22.3184H22.2534L0.0878906 44.6367H44.4634L66.7038 22.3184Z"/><path d="M22.21 0H0V22.3184H22.21V0Z"/><path d="M66.7198 0H44.5098V22.3184H66.7198V0Z"/><path d="M66.7198 67V44.6369H44.5098V67H66.7198Z"/>`
const ZOO_MARK = inner(read('/home/z/work/zoo/zoo.industries/public/zoo-logo.svg'))

const lux = zenwm('lux'), hanzo = zenwm('hanzo'), zoo = zenwm('zoo')

const HTML = `<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Wordmarks in Zen</title>
<style>
@font-face{font-family:'Zen';src:url(data:font/woff2;base64,${SANS}) format('woff2');
  font-weight:100 900;font-style:normal;font-display:block}
:root{--bg:#08080a;--fg:#f3f3f0;--dim:#8c8c87;--faint:#585854;
  --line:rgba(255,255,255,.085);--line2:rgba(255,255,255,.17);--sunk:#000;--ok:#79a866}
*{box-sizing:border-box}
html{-webkit-text-size-adjust:100%}
body{margin:0;background:var(--bg);color:var(--fg);font-family:'Zen',sans-serif;
  font-size:16px;line-height:1.62;letter-spacing:-.004em;-webkit-font-smoothing:antialiased}
.page{max-width:1080px;margin:0 auto;padding:0 30px 120px}
@media(max-width:720px){.page{padding:0 18px 70px}}
h1{margin:0;font-size:clamp(38px,7vw,72px);font-weight:200;letter-spacing:-.046em;line-height:1.05}
h2{margin:0 0 8px;font-size:clamp(23px,3.3vw,30px);font-variation-settings:'wght' 500;letter-spacing:-.03em}
p{margin:0 0 16px;max-width:68ch;color:var(--dim)}
strong{font-weight:600;color:var(--fg)}
code{font-size:.92em;background:rgba(255,255,255,.06);padding:1px 6px;border-radius:2px}
.eyebrow{display:block;font-size:11.5px;letter-spacing:.16em;text-transform:uppercase;
  color:var(--faint);font-variation-settings:'wght' 600;margin-bottom:11px}
header{padding:78px 0 50px;border-bottom:1px solid var(--line)}
header p{font-size:clamp(17px,2.2vw,21px);line-height:1.5;max-width:58ch;margin-top:22px;
  font-variation-settings:'wght' 300;letter-spacing:-.014em}
section{padding:54px 0;border-bottom:1px solid var(--line)}
section:last-of-type{border-bottom:0}
.stage{background:var(--sunk);border:1px solid var(--line);border-radius:3px;
  padding:44px 38px;margin-top:22px;overflow-x:auto}
.lockup{display:flex;align-items:center;gap:26px;color:var(--fg)}
.lockup svg{display:block;height:var(--h,64px);width:auto}
.lockup .mark{height:var(--m,64px);flex:none}
.note{font-size:13.5px;color:var(--dim);max-width:64ch;line-height:1.6;margin-top:15px}
.two{display:grid;gap:14px}
@media(min-width:840px){.two{grid-template-columns:1fr 1fr}}
.cell{background:var(--sunk);border:1px solid var(--line);border-radius:3px;padding:34px 28px}
.cell .cap{font-size:11px;letter-spacing:.1em;text-transform:uppercase;color:var(--faint);
  font-variation-settings:'wght' 600;margin-bottom:20px}
.cell svg{display:block;height:56px;width:auto;color:var(--fg)}
.stack{position:relative;height:78px}
.stack svg{position:absolute;left:0;top:0;height:78px}
.stack .a{color:#e8564a;opacity:.85}
.stack .b{color:#4a9fe8;opacity:.75;mix-blend-mode:screen}
table{border-collapse:collapse;width:100%;font-size:14.5px;margin-top:20px}
th,td{text-align:right;padding:9px 14px;border-bottom:1px solid var(--line)}
th:first-child,td:first-child{text-align:left;padding-left:0}
td:last-child,th:last-child{padding-right:0}
thead th{font-size:11px;letter-spacing:.11em;text-transform:uppercase;color:var(--faint);
  font-variation-settings:'wght' 600;border-bottom-color:var(--line2)}
tbody tr:last-child td{border-bottom:0}
.pill{font-size:12px;padding:1px 8px;border-radius:2px;font-variation-settings:'wght' 600}
.y{color:var(--ok);background:rgba(121,168,102,.13)}
.n{color:#c65c4c;background:rgba(198,92,76,.13)}
.fam{display:flex;flex-wrap:wrap;align-items:baseline;gap:14px 26px;margin-top:14px}
.fam span{font-variation-settings:'wght' 845;transform:scaleX(1.56);transform-origin:left center;
  display:inline-block;letter-spacing:-.04em;font-size:clamp(22px,3.4vw,40px);white-space:nowrap}
</style>

<div class="page">
<header>
  <span class="eyebrow">@hanzo/font · one family, three marks</span>
  <h1>Wordmarks in Zen</h1>
  <p>Each of the three, cut from the same variable file as real outlines — not a
  <code>&lt;text&gt;</code> element, so the mark is right in an email client, a
  partner's deck and a favicon, where no font resolves.</p>
</header>

<section>
  <span class="eyebrow">Hanzo</span>
  <h2>Already Zen</h2>
  <p><code>brand/assets/logo/wordmark.svg</code> sets it as live type at
  <strong>wght 600, −0.0286em</strong>. This is the same setting, converted to
  outlines so it no longer depends on the font resolving.</p>
  <div class="stage"><div class="lockup" style="--h:62px;--m:58px">
    <svg class="mark" viewBox="0 0 67 67" fill="currentColor">${HANZO_MARK}</svg>
    <svg viewBox="${hanzo.vb}" fill="currentColor">${hanzo.body}</svg>
  </div></div>
</section>

<section>
  <span class="eyebrow">Zoo</span>
  <h2>Lowercase, and symmetrical for it</h2>
  <p>Set lowercase: <strong>z</strong>, <strong>o</strong> and <strong>o</strong> are
  all x-height, and the two <strong>o</strong> are the same circle. The mark is even
  by construction rather than by optical correction — caps would need the Z kerned
  against two round forms and the whole thing tuned by hand. It also sits better
  beside a mark built from three circles.</p>
  <div class="stage"><div class="lockup" style="--h:58px;--m:66px">
    <svg class="mark" viewBox="0 0 1024 1024">${ZOO_MARK}</svg>
    <svg viewBox="${zoo.vb}" fill="currentColor">${zoo.body}</svg>
  </div></div>
  <p class="note">wght 800, −0.025em — the weights <code>components/Logo.tsx</code>
  already renders, so nothing about the brand changes except that the letters are
  now Zen and portable.</p>
</section>

<section>
  <span class="eyebrow">Lux</span>
  <h2>The one that was drawn</h2>
  <p>Lux's wordmark is 633 bytes of geometry someone drew — not type, and so not
  reachable by any font setting. That is why <strong>LUX CREDIT</strong> set in Zen
  sits <em>next to</em> the mark rather than <em>with</em> it.</p>
  <div class="two">
    <div class="cell"><div class="cap">Drawn — what ships today</div>
      <svg viewBox="0 0 63 17" fill="currentColor">${inner(LUX_REAL).replace(/fill="#ffffff"/g, '')}</svg></div>
    <div class="cell"><div class="cap">Zen Wide — 845 · 1.56× · −0.04em</div>
      <svg viewBox="${lux.vb}" fill="currentColor">${lux.body}</svg></div>
  </div>
  <div class="cell" style="margin-top:14px"><div class="cap">Overlaid — drawn in red, Zen in blue</div>
    <div class="stack">
      <svg class="a" viewBox="0 0 63 17" fill="currentColor">${inner(LUX_REAL).replace(/fill="#ffffff"/g, '')}</svg>
      <svg class="b" viewBox="${lux.vb}" fill="currentColor">${lux.body}</svg>
    </div></div>

  <table>
    <thead><tr><th>Ratio</th><th>Drawn</th><th>Zen 900 max</th><th></th></tr></thead>
    <tbody>
      <tr><td>stem ÷ cap</td><td>0.298</td><td>0.276</td><td><span class="pill y">scaleX reaches it</span></td></tr>
      <tr><td>bar ÷ cap</td><td>0.266</td><td>0.221</td><td><span class="pill n">20% short</span></td></tr>
      <tr><td>bar ÷ stem</td><td>0.892</td><td>0.801</td><td><span class="pill n">wrong direction</span></td></tr>
      <tr><td>X ÷ cap</td><td>1.429</td><td>1.045</td><td><span class="pill y">scaleX reaches it</span></td></tr>
    </tbody>
  </table>
  <p class="note"><strong>The bar settles it.</strong> A horizontal bar's thickness is
  a vertical measurement, so <code>scaleX</code> cannot touch it — the only lever is
  weight, and Zen's heaviest master is already 20% under. Worse, Zen thins its
  horizontals as it gets heavier (0.977 → 0.801) while the drawn mark stays
  near-monoline at 0.892, so the gap widens exactly where it would need to close.
  No setting of this font is the drawn mark.</p>
</section>

<section>
  <span class="eyebrow">The decision</span>
  <h2>Adopt the cut, or keep the drawing</h2>
  <p>Zen Wide is a <strong>12.5% residual</strong> on the word LUX — a strong
  resemblance, and close enough that the two read as one family at a glance. Lux has
  two honest options and they lead somewhere different:</p>
  <p><strong>Adopt it.</strong> Ship the Zen cut as the wordmark. Every lockup beside
  it then matches by construction, because they are the same outlines:</p>
  <div class="stage"><div class="fam">
    <span>LUX</span><span>LUX CREDIT</span><span>LUX EXCHANGE</span><span>LUX NETWORK</span>
  </div></div>
  <p class="note" style="margin-bottom:26px">One file, one setting, and a new property
  costs nothing — no drawing, no approval, no second asset to keep in step.</p>
  <p><strong>Keep the drawing.</strong> The mark stays exactly as it is, and Zen Wide
  is what sits beside it. That is what ships today, and the 12.5% is the price —
  visible if you set them adjacent at the same size, invisible in a header where only
  one appears.</p>
  <p class="note">Closing the gap for real is neither of these: it is a heavier
  master drawn for Zen, with the horizontals held near-monoline at display weight.
  That is outline work, not a setting, and it would let the family reach the drawn
  mark rather than resemble it.</p>
</section>
</div>
`

writeFileSync(join(HERE, 'dist', 'wordmarks.html'), HTML)
console.log(`  wordmarks.html  ${(HTML.length / 1024).toFixed(0)} KB`)
