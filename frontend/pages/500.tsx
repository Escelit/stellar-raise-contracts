import React from "react";
import { useRouter } from "next/router";

const ServerErrorPage = () => {
  const router = useRouter();

  return (
    <div style={styles.container}>
      <div style={styles.content}>
        <h1 style={styles.code}>500</h1>
        <h2 style={styles.title}>Internal Server Error</h2>
        <p style={styles.message}>
          Something went wrong on our end. Please try again or return home while
          we work on fixing the issue.
        </p>
        <div style={styles.actions}>
          <button style={styles.primaryButton} onClick={() => router.reload()}>
            Try Again
          </button>
          <button
            style={styles.secondaryButton}
            onClick={() => router.push("/")}
          >
            Return to Home
          </button>
        </div>
      </div>
    </div>
  );
};

const styles = {
  container: {
    display: "flex",
    justifyContent: "center",
    alignItems: "center",
    minHeight: "100vh",
    backgroundColor: "#f9fafb",
    fontFamily: "sans-serif",
  },
  content: {
    textAlign: "center" as const,
    padding: "2rem",
  },
  code: {
    fontSize: "6rem",
    fontWeight: "bold",
    color: "#dc2626",
    margin: 0,
  },
  title: {
    fontSize: "1.5rem",
    fontWeight: "600",
    color: "#111827",
    marginBottom: "1rem",
  },
  message: {
    fontSize: "1rem",
    color: "#6b7280",
    marginBottom: "2rem",
    maxWidth: "400px",
    margin: "0 auto 2rem auto",
  },
  actions: {
    display: "flex",
    gap: "1rem",
    justifyContent: "center",
  },
  primaryButton: {
    backgroundColor: "#dc2626",
    color: "#ffffff",
    border: "none",
    padding: "0.75rem 1.5rem",
    borderRadius: "0.5rem",
    fontSize: "1rem",
    cursor: "pointer",
  },
  secondaryButton: {
    backgroundColor: "#ffffff",
    color: "#374151",
    border: "1px solid #d1d5db",
    padding: "0.75rem 1.5rem",
    borderRadius: "0.5rem",
    fontSize: "1rem",
    cursor: "pointer",
  },
};

export default ServerErrorPage;
