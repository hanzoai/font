import localFont from "next/font/local";

export const ZenMonoNonVariable = localFont({
  src: [
    {
      path: "./fonts/zen-mono/ZenMono-Thin.woff2",
      weight: "100",
      style: "normal",
    },
    {
      path: "./fonts/zen-mono/ZenMono-UltraLight.woff2",
      weight: "200",
      style: "normal",
    },
    {
      path: "./fonts/zen-mono/ZenMono-Light.woff2",
      weight: "300",
      style: "normal",
    },
    {
      path: "./fonts/zen-mono/ZenMono-Regular.woff2",
      weight: "400",
      style: "normal",
    },
    {
      path: "./fonts/zen-mono/ZenMono-Medium.woff2",
      weight: "500",
      style: "normal",
    },
    {
      path: "./fonts/zen-mono/ZenMono-SemiBold.woff2",
      weight: "600",
      style: "normal",
    },
    {
      path: "./fonts/zen-mono/ZenMono-Bold.woff2",
      weight: "700",
      style: "normal",
    },
    {
      path: "./fonts/zen-mono/ZenMono-Black.woff2",
      weight: "800",
      style: "normal",
    },
    {
      path: "./fonts/zen-mono/ZenMono-UltraBlack.woff2",
      weight: "900",
      style: "normal",
    },
  ],
  variable: "--font-zen-mono-non-variable",
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
});
