import React from "react";
import { render, screen, fireEvent } from "@testing-library/react";
import SubmitButton, { SubmitButtonProps } from "./SubmitButton";

// @testing-library/react is not in devDependencies; we use a minimal render shim
// via jest-environment-jsdom + ts-jest already configured in jest.config.json.
// If @testing-library/react is unavailable, fall back to ReactDOM.render.

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Render helper that returns the button element */
function renderBtn(props: Partial<SubmitButtonProps> = {}) {
  const defaults: SubmitButtonProps = { label: "Contribute" };
  const { container } = render(<SubmitButton {...defaults} {...props} />);
  return container.querySelector("button") as HTMLButtonElement;
}

// ---------------------------------------------------------------------------
// Idle state
// ---------------------------------------------------------------------------

describe("idle state (default)", () => {
  it("renders the provided label", () => {
    renderBtn();
    expect(screen.getByText("Contribute")).toBeTruthy();
  });

  it("is not disabled", () => {
    const btn = renderBtn();
    expect(btn.disabled).toBe(false);
  });

  it("fires onClick when clicked", () => {
    const onClick = jest.fn();
    const btn = renderBtn({ onClick });
    fireEvent.click(btn);
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("has data-state='idle'", () => {
    const btn = renderBtn();
    expect(btn.getAttribute("data-state")).toBe("idle");
  });

  it("has type='submit' by default", () => {
    const btn = renderBtn();
    expect(btn.type).toBe("submit");
  });

  it("respects explicit type='button'", () => {
    const btn = renderBtn({ type: "button" });
    expect(btn.type).toBe("button");
  });

  it("applies btn--primary class", () => {
    const btn = renderBtn();
    expect(btn.className).toContain("btn--primary");
  });

  it("applies btn--full when fullWidth=true", () => {
    const btn = renderBtn({ fullWidth: true });
    expect(btn.className).toContain("btn--full");
  });

  it("does not apply btn--full by default", () => {
    const btn = renderBtn();
    expect(btn.className).not.toContain("btn--full");
  });

  it("appends custom className", () => {
    const btn = renderBtn({ className: "my-custom" });
    expect(btn.className).toContain("my-custom");
  });
});

// ---------------------------------------------------------------------------
// Loading state
// ---------------------------------------------------------------------------

describe("loading state", () => {
  it("shows default loading label", () => {
    renderBtn({ state: "loading" });
    expect(screen.getByText("Processing…")).toBeTruthy();
  });

  it("shows custom loadingLabel when provided", () => {
    renderBtn({ state: "loading", loadingLabel: "Sending…" });
    expect(screen.getByText("Sending…")).toBeTruthy();
  });

  it("is disabled", () => {
    const btn = renderBtn({ state: "loading" });
    expect(btn.disabled).toBe(true);
  });

  it("has aria-busy='true'", () => {
    const btn = renderBtn({ state: "loading" });
    expect(btn.getAttribute("aria-busy")).toBe("true");
  });

  it("does NOT fire onClick while loading", () => {
    const onClick = jest.fn();
    const btn = renderBtn({ state: "loading", onClick });
    fireEvent.click(btn);
    expect(onClick).not.toHaveBeenCalled();
  });

  it("renders a spinner SVG", () => {
    const { container } = render(<SubmitButton label="Go" state="loading" />);
    expect(container.querySelector("svg")).toBeTruthy();
  });

  it("has data-state='loading'", () => {
    const btn = renderBtn({ state: "loading" });
    expect(btn.getAttribute("data-state")).toBe("loading");
  });
});

// ---------------------------------------------------------------------------
// Success state
// ---------------------------------------------------------------------------

describe("success state", () => {
  it("shows default success label", () => {
    renderBtn({ state: "success" });
    expect(screen.getByText("Success!")).toBeTruthy();
  });

  it("shows custom successLabel when provided", () => {
    renderBtn({ state: "success", successLabel: "Funded!" });
    expect(screen.getByText("Funded!")).toBeTruthy();
  });

  it("is disabled (prevents re-submission)", () => {
    const btn = renderBtn({ state: "success" });
    expect(btn.disabled).toBe(true);
  });

  it("does NOT fire onClick on success", () => {
    const onClick = jest.fn();
    const btn = renderBtn({ state: "success", onClick });
    fireEvent.click(btn);
    expect(onClick).not.toHaveBeenCalled();
  });

  it("has data-state='success'", () => {
    const btn = renderBtn({ state: "success" });
    expect(btn.getAttribute("data-state")).toBe("success");
  });

  it("applies submit-btn--success class", () => {
    const btn = renderBtn({ state: "success" });
    expect(btn.className).toContain("submit-btn--success");
  });
});

// ---------------------------------------------------------------------------
// Error state
// ---------------------------------------------------------------------------

describe("error state", () => {
  it("shows default error label", () => {
    renderBtn({ state: "error" });
    expect(screen.getByText("Failed – try again")).toBeTruthy();
  });

  it("shows custom errorLabel when provided", () => {
    renderBtn({ state: "error", errorLabel: "Wallet rejected" });
    expect(screen.getByText("Wallet rejected")).toBeTruthy();
  });

  it("is NOT disabled – user can retry", () => {
    const btn = renderBtn({ state: "error" });
    expect(btn.disabled).toBe(false);
  });

  it("fires onClick when clicked in error state", () => {
    const onClick = jest.fn();
    const btn = renderBtn({ state: "error", onClick });
    fireEvent.click(btn);
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("applies btn--danger class", () => {
    const btn = renderBtn({ state: "error" });
    expect(btn.className).toContain("btn--danger");
  });

  it("has data-state='error'", () => {
    const btn = renderBtn({ state: "error" });
    expect(btn.getAttribute("data-state")).toBe("error");
  });
});

// ---------------------------------------------------------------------------
// Externally disabled (deadline passed / goal met edge cases)
// ---------------------------------------------------------------------------

describe("disabled prop (external edge cases)", () => {
  it("is disabled when disabled=true in idle state", () => {
    const btn = renderBtn({ disabled: true });
    expect(btn.disabled).toBe(true);
  });

  it("does NOT fire onClick when disabled=true", () => {
    const onClick = jest.fn();
    const btn = renderBtn({ disabled: true, onClick });
    fireEvent.click(btn);
    expect(onClick).not.toHaveBeenCalled();
  });

  it("is disabled when disabled=true in error state (e.g. deadline passed)", () => {
    const btn = renderBtn({ state: "error", disabled: true });
    expect(btn.disabled).toBe(true);
  });

  it("has aria-disabled='true' when disabled", () => {
    const btn = renderBtn({ disabled: true });
    expect(btn.getAttribute("aria-disabled")).toBe("true");
  });
});

// ---------------------------------------------------------------------------
// Accessibility
// ---------------------------------------------------------------------------

describe("accessibility", () => {
  it("has aria-live='polite' for state announcements", () => {
    const btn = renderBtn();
    expect(btn.getAttribute("aria-live")).toBe("polite");
  });

  it("aria-busy is false when not loading", () => {
    const btn = renderBtn({ state: "idle" });
    expect(btn.getAttribute("aria-busy")).toBe("false");
  });

  it("spinner SVG is aria-hidden", () => {
    const { container } = render(<SubmitButton label="Go" state="loading" />);
    const svg = container.querySelector("svg");
    expect(svg?.getAttribute("aria-hidden")).toBe("true");
  });
});
