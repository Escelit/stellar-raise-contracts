import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { CssVariablesUsage, CssVariablesMap, THEME } from '../utils/css_variables_usage';

type ThemeMode = 'light' | 'dark' | 'system';

interface ThemeContextType {
  mode: ThemeMode;
  setMode: (mode: ThemeMode) => void;
  cssVars: CssVariablesUsage;
  isDark: boolean;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export const useTheme = () => {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useTheme must be used within ThemeProvider');
  }
  return context;
};

interface ThemeProviderProps {
  children: ReactNode;
}

const lightTheme: CssVariablesMap = {
  '--color-primary-blue': '#0066FF',
  '--color-neutral-100': '#FFFFFF',
  // Add more light theme vars
};

const darkTheme: CssVariablesMap = {
  '--color-primary-blue': '#4B90FF',
  '--color-neutral-100': '#1A1A1A',
  // Add more dark theme vars
};

export const ThemeProvider: React.FC<ThemeProviderProps> = ({ children }) => {
  const [mode, setMode] = useState<ThemeMode>('system');
  const [isDark, setIsDark] = useState(false);

  const cssVars = new CssVariablesUsage();

  useEffect(() => {
    const applyTheme = () => {
      const root = document.documentElement;
      const themeMap = mode === 'dark' ? darkTheme : lightTheme;
cssVars.setMultiple(themeMap as CssVariablesMap);
      setIsDark(mode === 'dark' || (mode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches));
    };

    applyTheme();

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    mediaQuery.addEventListener('change', applyTheme);

    return () => mediaQuery.removeEventListener('change', applyTheme);
  }, [mode]);

  return (
  <ThemeContext.Provider value={{ mode, setMode, cssVars, isDark }}>
      {children}
    </ThemeContext.Provider>
  );
};

