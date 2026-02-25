import { useEffect, useState, useCallback } from "react";

type Theme = "dark" | "light";
type FontFamily = "doto" | "bebas";

export function useTheme() {
  const [theme, setThemeState] = useState<Theme>(() => {
    const saved = localStorage.getItem("heki-theme");
    if (saved === "light" || saved === "dark") return saved;
    return "dark";
  });

  const [font, setFontState] = useState<FontFamily>(() => {
    const saved = localStorage.getItem("heki-font");
    if (saved === "doto" || saved === "bebas") return saved;
    return "doto";
  });

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("heki-theme", theme);
  }, [theme]);

  useEffect(() => {
    document.documentElement.setAttribute("data-font", font);
    localStorage.setItem("heki-font", font);
  }, [font]);

  const setTheme = useCallback((t: Theme) => {
    setThemeState(t);
  }, []);

  const toggleTheme = useCallback(() => {
    setThemeState((prev) => (prev === "dark" ? "light" : "dark"));
  }, []);

  const toggleFont = useCallback(() => {
    setFontState((prev) => (prev === "doto" ? "bebas" : "doto"));
  }, []);

  return { theme, setTheme, toggleTheme, font, toggleFont };
}
