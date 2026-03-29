import React from "react";

interface EmptyStateProps {
  title?: string;
  message?: string;
}

const EmptyIllustration = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="180"
    height="180"
    viewBox="0 0 180 180"
    fill="none"
    aria-hidden="true"
  >
    {/* Background circle */}
    <circle cx="90" cy="90" r="90" fill="#f3f4f6" />

    {/* Empty box */}
    <rect x="50" y="60" width="80" height="70" rx="8" fill="#e5e7eb" />
    <rect x="50" y="60" width="80" height="20" rx="8" fill="#d1d5db" />

    {/* Box lid open */}
    <path
      d="M45 60 Q90 30 135 60"
      stroke="#9ca3af"
      strokeWidth="3"
      fill="none"
      strokeLinecap="round"
    />

    {/* Stars / sparkles */}
    <circle cx="40" cy="50" r="4" fill="#c4b5fd" />
    <circle cx="140" cy="45" r="3" fill="#c4b5fd" />
    <circle cx="130" cy="110" r="3" fill="#a5b4fc" />

    {/* Bottom shadow */}
    <ellipse cx="90" cy="148" rx="40" ry="6" fill="#e5e7eb" />
  </svg>
);

const EmptyState = ({
  title = "No campaigns found",
  message = "There are no active campaigns right now. Check back later or create one to get started.",
}: EmptyStateProps) => {
  return (
    <div style={styles.container}>
      <EmptyIllustration />
      <h2 style={styles.title}>{title}</h2>
      <p style={styles.message}>{message}</p>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    justifyContent: "center",
    padding: "4rem 2rem",
    textAlign: "center",
    minHeight: "400px",
  },
  title: {
    fontSize: "1.25rem",
    fontWeight: "600",
    color: "#111827",
    marginTop: "1.5rem",
    marginBottom: "0.5rem",
  },
  message: {
    fontSize: "0.95rem",
    color: "#6b7280",
    maxWidth: "360px",
    lineHeight: "1.6",
  },
};

export default EmptyState;
