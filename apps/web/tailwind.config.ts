import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        surface: {
          DEFAULT: "rgb(var(--color-surface) / <alpha-value>)",
          raised: "rgb(var(--color-surface-raised) / <alpha-value>)",
          muted: "rgb(var(--color-surface-muted) / <alpha-value>)",
          wash: "rgb(var(--color-surface-wash) / <alpha-value>)",
        },
        ink: {
          DEFAULT: "rgb(var(--color-ink) / <alpha-value>)",
          muted: "rgb(var(--color-ink-muted) / <alpha-value>)",
          soft: "rgb(var(--color-ink-soft) / <alpha-value>)",
        },
        action: {
          DEFAULT: "rgb(var(--color-action) / <alpha-value>)",
          soft: "rgb(var(--color-action-soft) / <alpha-value>)",
          border: "rgb(var(--color-action-border) / <alpha-value>)",
          strong: "rgb(var(--color-action-strong) / <alpha-value>)",
        },
        attention: {
          DEFAULT: "rgb(var(--color-attention) / <alpha-value>)",
          soft: "rgb(var(--color-attention-soft) / <alpha-value>)",
          border: "rgb(var(--color-attention-border) / <alpha-value>)",
        },
        line: {
          DEFAULT: "rgb(var(--color-line) / <alpha-value>)",
          soft: "rgb(var(--color-line-soft) / <alpha-value>)",
        },
        field: {
          DEFAULT: "rgb(var(--color-field) / <alpha-value>)",
        },
        danger: {
          DEFAULT: "rgb(var(--color-danger) / <alpha-value>)",
          soft: "rgb(var(--color-danger-soft) / <alpha-value>)",
          border: "rgb(var(--color-danger-border) / <alpha-value>)",
        },
      },
      fontFamily: {
        display: [
          "Georgia",
          "Iowan Old Style",
          "Palatino Linotype",
          "serif",
        ],
        sans: [
          "Aptos",
          "Source Sans 3",
          "ui-sans-serif",
          "system-ui",
          "-apple-system",
          "sans-serif",
        ],
      },
    },
  },
  plugins: [],
} satisfies Config;
