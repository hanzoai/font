# Zen Sans, Zen Mono & Zen Pixel

Zen is the Hanzo type family, under the SIL Open Font License. Every copyright the
licence requires is in `LICENSE.txt`, which ships beside the binaries.

Zen Sans is a sans-serif typeface designed for legibility and simplicity. It is modern, geometric, and based on the principles of classic Swiss typography. It is designed to be used in body copy, headlines, logos, posters, and other large display sizes.

Zen Mono is a monospaced typeface, crafted to be the perfect partner to Zen Sans. It is designed to be used in code editors, diagrams, terminals, and other text-based interfaces where code is rendered.

Zen Pixel is a display typeface family featuring five unique pixel-based variants, each with a distinct visual style. It is designed for decorative use in headlines, logos, and other display contexts where a pixelated aesthetic is desired.

### Installation

```sh
pnpm add @hanzo/font
```

### Using with Next.js

`ZenSans` is exported from `@hanzo/font/sans`, `ZenMono` can be found in `@hanzo/font/mono`, and Zen Pixel variants are available from `@hanzo/font/pixel`. All are `NextFontWithVariable` instances. You can learn more by [reading the `next/font` docs](https://nextjs.org/docs/app/building-your-application/optimizing/fonts).

#### Zen Pixel Variants

Zen Pixel includes five distinct variants, each exported separately:

| Export               | CSS Variable                  | Description              |
| -------------------- | ----------------------------- | ------------------------ |
| `ZenPixelSquare`   | `--font-zen-pixel-square`   | Square pixel shapes      |
| `ZenPixelGrid`     | `--font-zen-pixel-grid`     | Grid-based pixel pattern |
| `ZenPixelCircle`   | `--font-zen-pixel-circle`   | Circular pixel shapes    |
| `ZenPixelTriangle` | `--font-zen-pixel-triangle` | Triangular pixel shapes  |
| `ZenPixelLine`     | `--font-zen-pixel-line`     | Line-based pixel pattern |

```jsx
import {
  ZenPixelSquare,
  ZenPixelGrid,
  ZenPixelCircle,
  ZenPixelTriangle,
  ZenPixelLine,
} from "@hanzo/font/pixel";
```

#### App Router

In `app/layout.js`:

```jsx
import { ZenSans } from "@hanzo/font/sans";

export default function RootLayout({ children }) {
  return (
    <html lang="en" className={ZenSans.className}>
      <body>{children}</body>
    </html>
  );
}
```

#### Pages Router

In `pages/_app.js`:

```jsx
import { ZenSans } from "@hanzo/font/sans";

export default function MyApp({ Component, pageProps }) {
  return (
    <main className={ZenSans.className}>
      <Component {...pageProps} />
    </main>
  );
}
```

If you're using a version of Next.js that's older than 15, then in `next.config.js` or `next.config.mjs` add:

```diff js
/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
+  transpilePackages: ["@hanzo/font"],
};

export default nextConfig;
```

This is required to fix errors like:

- `TypeError: next_font_local__WEBPACK_IMPORTED_MODULE_0___default(...) is not a function`
- `SyntaxError: Cannot use import statement outside a module`

#### With Tailwind CSS

All Zen fonts can be used through CSS variables.

- `ZenSans`: `--font-zen-sans`
- `ZenMono`: `--font-zen-mono`
- `ZenPixelSquare`: `--font-zen-pixel-square`
- `ZenPixelGrid`: `--font-zen-pixel-grid`
- `ZenPixelCircle`: `--font-zen-pixel-circle`
- `ZenPixelTriangle`: `--font-zen-pixel-triangle`
- `ZenPixelLine`: `--font-zen-pixel-line`

In `app/layout.js`:

```jsx
import { ZenSans } from "@hanzo/font/sans";
import { ZenMono } from "@hanzo/font/mono";
import { ZenPixelSquare } from "@hanzo/font/pixel";

export default function RootLayout({ children }) {
  return (
    <html
      lang="en"
      className={`${ZenSans.variable} ${ZenMono.variable} ${ZenPixelSquare.variable}`}
    >
      <body>{children}</body>
    </html>
  );
}
```

##### Tailwind CSS V4

Then in `tailwind.css`:

```css
@theme {
  /* rest of your theme config */

  --font-sans: var(--font-zen-sans);
  --font-mono: var(--font-zen-mono);
  --font-pixel-square: var(--font-zen-pixel-square);
  --font-pixel-grid: var(--font-zen-pixel-grid);
  --font-pixel-circle: var(--font-zen-pixel-circle);
  --font-pixel-triangle: var(--font-zen-pixel-triangle);
  --font-pixel-line: var(--font-zen-pixel-line);

  /* rest of your theme config */
}
```

##### Tailwind CSS V3

Then in `tailwind.config.js`:

```javascript
module.exports = {
  theme: {
    extend: {
      fontFamily: {
        sans: ["var(--font-zen-sans)"],
        mono: ["var(--font-zen-mono)"],
        "pixel-square": ["var(--font-zen-pixel-square)"],
        "pixel-grid": ["var(--font-zen-pixel-grid)"],
        "pixel-circle": ["var(--font-zen-pixel-circle)"],
        "pixel-triangle": ["var(--font-zen-pixel-triangle)"],
        "pixel-line": ["var(--font-zen-pixel-line)"],
      },
    },
  },
};
```

### License

The Zen font family is free and open sourced under the [SIL Open Font License](../../LICENSE.txt).
