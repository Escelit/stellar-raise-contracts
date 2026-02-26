import React, { useState } from "react";

const SunIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="18"
    height="18"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    aria-hidden="true"
  >
    <circle cx="12" cy="12" r="5" />
    <line x1="12" y1="1" x2="12" y2="3" />
    <line x1="12" y1="21" x2="12" y2="23" />
    <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
    <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
    <line x1="1" y1="12" x2="3" y2="12" />
    <line x1="21" y1="12" x2="23" y2="12" />
    <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
    <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
  </svg>
);

const MoonIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="18"
    height="18"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    aria-hidden="true"
  >
    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
  </svg>
);

const ThemeToggle = () => {
  const [isDark, setIsDark] = useState(false);

  const toggle = () => setIsDark((prev) => !prev);

  return (
    <button
      onClick={toggle}
      style={{
        ...styles.button,
        backgroundColor: isDark ? "#1f2937" : "#f3f4f6",
        color: isDark ? "#facc15" : "#4f46e5",
      }}
      aria-label={isDark ? "Switch to light mode" : "Switch to dark mode"}
      title={isDark ? "Switch to light mode" : "Switch to dark mode"}
      type="button"
    >
      <span style={styles.iconWrapper}>{isDark ? <MoonIcon /> : <SunIcon />}</span>
      <span style={styles.label}>{isDark ? "Dark" : "Light"}</span>
    </button>
  );
};

const styles: Record<string, React.CSSProperties> = {
  button: {
    display: "inline-flex",
    alignItems: "center",
    gap: "0.5rem",
    padding: "0.5rem 1rem",
    borderRadius: "999px",
    border: "1px solid #d1d5db",
    cursor: "pointer",
    fontSize: "0.875rem",
    fontWeight: "500",
    transition: "background-color 0.3s ease, color 0.3s ease",
    outline: "none",
  },
  iconWrapper: {
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    transition: "transform 0.3s ease",
  },
  label: {
    userSelect: "none",
  },
};

export default ThemeToggle;
