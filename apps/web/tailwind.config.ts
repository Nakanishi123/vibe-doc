import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        surface: {
          DEFAULT: "#f8fafc",
          raised: "#ffffff",
          muted: "#eef2f7",
        },
        ink: {
          DEFAULT: "#172033",
          muted: "#5b667a",
          soft: "#7b8495",
        },
        action: {
          DEFAULT: "#0f766e",
          soft: "#ccfbf1",
          border: "#5eead4",
        },
        attention: {
          DEFAULT: "#b45309",
          soft: "#fef3c7",
        },
      },
      fontFamily: {
        sans: [
          "Inter",
          "ui-sans-serif",
          "system-ui",
          "-apple-system",
          "BlinkMacSystemFont",
          "Segoe UI",
          "sans-serif",
        ],
      },
    },
  },
  plugins: [],
} satisfies Config;
