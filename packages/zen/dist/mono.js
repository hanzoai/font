import localFont from "next/font/local";

export const ZenMono = localFont({
  src: "./fonts/zen-mono/ZenMono-Variable.woff2",
  variable: "--font-zen-mono",
  adjustFontFallback: false,
  fallback: [
    "ui-monospace",
    "SFMono-Regular",
    "Roboto Mono",
    "Menlo",
    "Monaco",
    "Liberation Mono",
    "DejaVu Sans Mono",
    "Courier New",
    "monospace",
  ],
  weight: "100 900",
});
