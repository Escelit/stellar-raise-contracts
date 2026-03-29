import React, { useState, useEffect } from "react";

const ScrollToTop = () => {
  const [isVisible, setIsVisible] = useState(false);

  const SCROLL_THRESHOLD = 300;

  useEffect(() => {
    const handleScroll = () => {
      setIsVisible(window.scrollY > SCROLL_THRESHOLD);
    };

    window.addEventListener("scroll", handleScroll);
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  const scrollToTop = () => {
    window.scrollTo({
      top: 0,
      behavior: "smooth",
    });
  };

  if (!isVisible) return null;

  return (
    <button
      onClick={scrollToTop}
      style={styles.button}
      aria-label="Scroll to top"
    >
      ↑
    </button>
  );
};

const styles: { button: React.CSSProperties } = {
  button: {
    position: "fixed",
const styles = {
  button: {
    position: "fixed",
    bottom: "2rem",
    right: "2rem",
    backgroundColor: "#4f46e5",
    color: "#ffffff",
    border: "none",
    borderRadius: "50%",
    width: "3rem",
    height: "3rem",
    fontSize: "1.25rem",
    cursor: "pointer",
    boxShadow: "0 4px 12px rgba(0, 0, 0, 0.15)",
    transition: "opacity 0.3s ease, transform 0.3s ease",
    zIndex: 1000,
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
  } as React.CSSProperties,
  },
};

export default ScrollToTop;
