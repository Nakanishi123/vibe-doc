import { createContext, useContext, type ReactNode } from "react";

export type ColorTheme = "light" | "dark";

const ThemeContext = createContext<ColorTheme | undefined>(undefined);

export function ThemeProvider({ children, value }: { children: ReactNode; value: ColorTheme }) {
  return <ThemeContext value={value}>{children}</ThemeContext>;
}

export function useTheme(): ColorTheme {
  const theme = useContext(ThemeContext);
  if (!theme) throw new Error("useTheme must be used within ThemeProvider");
  return theme;
}
