/**
 * ScrollToTop Component
 * Displays a floating action button in the bottom right corner
 * Button appears only when user scrolls past a threshold (300px)
 * Clicking the button smoothly scrolls back to the top
 */

class ScrollToTop {
  constructor(options = {}) {
    this.threshold = options.threshold || 300;
    this.button = null;
    this.isVisible = false;
    this.init();
  }

  init() {
    this.createButton();
    this.attachEventListeners();
  }

  createButton() {
    this.button = document.createElement("button");
    this.button.setAttribute("aria-label", "Scroll to top");
    this.button.classList.add("scroll-to-top");
    this.button.innerHTML = "↑";
    this.button.style.cssText = `
      position: fixed;
      bottom: 2rem;
      right: 2rem;
      background-color: #4f46e5;
      color: #ffffff;
      border: none;
      border-radius: 50%;
      width: 3rem;
      height: 3rem;
      font-size: 1.25rem;
      cursor: pointer;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
      transition: opacity 0.3s ease, transform 0.3s ease;
      z-index: 1000;
      display: flex;
      align-items: center;
      justify-content: center;
      opacity: 0;
      visibility: hidden;
    `;

    this.button.addEventListener("click", () => this.scrollToTop());
    document.body.appendChild(this.button);
  }

  attachEventListeners() {
    window.addEventListener("scroll", () => this.handleScroll());
  }

  handleScroll() {
    const shouldBeVisible = window.scrollY > this.threshold;

    if (shouldBeVisible && !this.isVisible) {
      this.show();
    } else if (!shouldBeVisible && this.isVisible) {
      this.hide();
    }
  }

  show() {
    this.isVisible = true;
    this.button.style.opacity = "1";
    this.button.style.visibility = "visible";
  }

  hide() {
    this.isVisible = false;
    this.button.style.opacity = "0";
    this.button.style.visibility = "hidden";
  }

  scrollToTop() {
    window.scrollTo({
      top: 0,
      behavior: "smooth",
    });
  }

  destroy() {
    if (this.button && this.button.parentNode) {
      this.button.parentNode.removeChild(this.button);
    }
    window.removeEventListener("scroll", () => this.handleScroll());
  }
}

// Initialize when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => {
    new ScrollToTop({ threshold: 300 });
  });
} else {
  new ScrollToTop({ threshold: 300 });
}
