import React, { useState } from "react";

type TooltipPosition = "top" | "bottom" | "left" | "right";

interface TooltipProps {
  content: string;
  position?: TooltipPosition;
  children: React.ReactNode;
}

const Tooltip = ({ content, position = "top", children }: TooltipProps) => {
  const [isVisible, setIsVisible] = useState(false);

  const show = () => setIsVisible(true);
  const hide = () => setIsVisible(false);

  return (
    <div
      style={styles.wrapper}
      onMouseEnter={show}
      onMouseLeave={hide}
      onFocus={show}
      onBlur={hide}
    >
      {children}
      {isVisible && (
        <div
          role="tooltip"
          aria-live="polite"
          style={{ ...styles.tooltip, ...positionStyles[position] }}
        >
          {content}
          <div style={{ ...styles.arrow, ...arrowStyles[position] }} />
        </div>
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  wrapper: {
    position: "relative",
    display: "inline-block",
  },
  tooltip: {
    position: "absolute",
    backgroundColor: "#1f2937",
    color: "#ffffff",
    padding: "0.4rem 0.75rem",
    borderRadius: "0.375rem",
    fontSize: "0.8rem",
    whiteSpace: "nowrap",
    zIndex: 9999,
    boxShadow: "0 4px 12px rgba(0,0,0,0.15)",
    pointerEvents: "none",
  },
  arrow: {
    position: "absolute",
    width: 0,
    height: 0,
  },
};

const positionStyles: Record<TooltipPosition, React.CSSProperties> = {
  top: { bottom: "calc(100% + 8px)", left: "50%", transform: "translateX(-50%)" },
  bottom: { top: "calc(100% + 8px)", left: "50%", transform: "translateX(-50%)" },
  left: { right: "calc(100% + 8px)", top: "50%", transform: "translateY(-50%)" },
  right: { left: "calc(100% + 8px)", top: "50%", transform: "translateY(-50%)" },
};

const arrowStyles: Record<TooltipPosition, React.CSSProperties> = {
  top: {
    top: "100%", left: "50%", transform: "translateX(-50%)",
    borderLeft: "5px solid transparent", borderRight: "5px solid transparent",
    borderTop: "5px solid #1f2937",
  },
  bottom: {
    bottom: "100%", left: "50%", transform: "translateX(-50%)",
    borderLeft: "5px solid transparent", borderRight: "5px solid transparent",
    borderBottom: "5px solid #1f2937",
  },
  left: {
    left: "100%", top: "50%", transform: "translateY(-50%)",
    borderTop: "5px solid transparent", borderBottom: "5px solid transparent",
    borderLeft: "5px solid #1f2937",
  },
  right: {
    right: "100%", top: "50%", transform: "translateY(-50%)",
    borderTop: "5px solid transparent", borderBottom: "5px solid transparent",
    borderRight: "5px solid #1f2937",
  },
};

export default Tooltip;
