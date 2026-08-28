# Zen

![The word Zen set twice — in Zen at a thin display weight, and beside it in Zen Pixel's triangle cut — over the line "Sans · Mono · Pixel" and, in Zen Mono, "one variable file · 100–900 · font.hanzo.ai".](./documentation/img/zen-banner--light.svg#gh-light-mode-only)
![The word Zen set twice — in Zen at a thin display weight, and beside it in Zen Pixel's triangle cut — over the line "Sans · Mono · Pixel" and, in Zen Mono, "one variable file · 100–900 · font.hanzo.ai".](./documentation/img/zen-banner--dark.svg#gh-dark-mode-only)

# Zen, Zen Mono & Zen Pixel

Zen is the Hanzo typeface. **Sans** is a geometric sans-serif in the Swiss line — one
variable file across weight 100–900, drawn for headlines, wordmarks and text alike.
**Mono** is its fixed-width companion, cut for code editors, terminals and diagrams.
**Pixel** is a display family of five cuts — Square, Grid, Circle, Triangle, Line —
one alphabet drawn five ways.

### Installing

Fonts to install on a machine are in [`fonts/`](./fonts): `otf/` and `ttf/` for the
static cuts, `variable/` for the one-file version of each family, `webfonts/` for
`.woff2`. `make build` regenerates all of it from `sources/`.

For a web or app surface, take the package instead — it carries the faces, the
stylesheet and the presets:

```sh
pnpm add @hanzo/font
```

```js
import '@hanzo/font/css'          // the faces
import '@hanzo/font/presets.css'  // the presets
```

Then name the role rather than the face: `var(--font-sans)`, `var(--font-mono)`.
Full instructions are in the [package README](./packages/zen/README.md), and
[font.hanzo.ai](https://font.hanzo.ai) is the live specimen.

### Building

CI builds the fonts on every push — look under Actions for the latest. To build on
your own machine:

- `make build` produces the font files.
- `make test` runs [Fontspector](https://github.com/fonttools/fontspector)'s quality checks.
- `make proof` writes HTML proofs.

`crates/zen` is the type tooling behind the presets — it reads outlines, measures
optical white between pairs, and fits one setting to a target shape. It compiles to
a native binary and to wasm from the same source, which is what lets the specimen
page run the fitter live.

### License

This Font Software is licensed under the SIL Open Font License, Version 1.1, in
[`LICENSE.txt`](./LICENSE.txt), which ships beside the binaries. The license is also
available with a FAQ at https://scripts.sil.org/OFL

### Repository layout

Structured as [Unified Font Repository v0.3](https://github.com/unified-font-repository/Unified-Font-Repository),
adjusted for the Google Fonts build workflow.
