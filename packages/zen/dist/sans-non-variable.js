import localFont from "next/font/local";

export const ZenSansNonVariable = localFont({
  src: [
    {
      path: "./fonts/zen-sans/Zen-Thin.woff2",
      weight: "100",
      style: "normal",
    },
    {
      path: "./fonts/zen-sans/Zen-UltraLight.woff2",
      weight: "200",
      style: "normal",
    },
    {
      path: "./fonts/zen-sans/Zen-Light.woff2",
      weight: "300",
      style: "normal",
    },
    {
      path: "./fonts/zen-sans/Zen-Regular.woff2",
      weight: "400",
      style: "normal",
    },
    {
      path: "./fonts/zen-sans/Zen-Medium.woff2",
      weight: "500",
      style: "normal",
    },
    {
      path: "./fonts/zen-sans/Zen-SemiBold.woff2",
      weight: "600",
      style: "normal",
    },
    {
      path: "./fonts/zen-sans/Zen-Bold.woff2",
      weight: "700",
      style: "normal",
    },
    {
      path: "./fonts/zen-sans/Zen-Black.woff2",
      weight: "800",
      style: "normal",
    },
    {
      path: "./fonts/zen-sans/Zen-UltraBlack.woff2",
      weight: "900",
      style: "normal",
    },
  ],
  variable: "--font-zen-sans-non-variable",
});
