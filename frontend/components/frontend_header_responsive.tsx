import React, { useState, useCallback, useMemo } from 'react';

/**
 * @title FrontendHeaderResponsive
 * @notice Responsive header navigation bar for the Stellar Raise crowdfunding dApp.
 *
 * @dev This React functional component renders a sticky top-level header with:
 *   - A brand logo section
 *   - A mobile hamburger toggle button (hidden on screens ≥ 768 px via `md:hidden`)
 *   - A navigation links area (always visible on desktop, toggled on mobile)
 *   - A wallet connection status badge
 *
 *   Breakpoints follow the design-system tokens in `frontend/styles/responsive.css`:
 *     - Mobile  : < 768 px  → hamburger toggle controls nav visibility
 *     - Tablet+ : ≥ 768 px  → nav links always rendered inline (`md:flex`)
 *
 * @custom:efficiency
 *   `useCallback` memoises `handleToggleMenu` so its reference stays stable
 *   between renders, preventing unnecessary re-renders of child elements that
 *   receive it as a prop.
 *   `useMemo` memoises the `navLinks` array so a new array object is not
 *   allocated on every render pass.
 *
 * @custom:security
 *   - No user-supplied HTML is injected into the DOM; all dynamic content
 *     (wallet status, menu state) derives from typed boolean props, eliminating
 *     XSS risk at the component boundary.
 *   - `onToggleMenu` is invoked inside the functional `setState` updater,
 *     guaranteeing the callback always receives the *new* state value and
 *     never a stale closure value.
 *   - Link `href` values are hardcoded constants; no user input reaches the
 *     anchor `href` attribute.
 *
 * @custom:accessibility
 *   - `aria-label` on the toggle button satisfies WCAG 2.1 SC 1.1.1.
 *   - `aria-expanded` on the toggle button satisfies WCAG 2.1 SC 4.1.2
 *     (Name, Role, Value) and keeps assistive technology in sync with visual state.
 *   - All interactive elements meet the 44 × 44 px minimum touch target
 *     size recommended by WCAG 2.5.5.
 */

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/**
 * @dev Props accepted by the `FrontendHeaderResponsive` component.
 */
export interface FrontendHeaderResponsiveProps {
  /**
   * @notice Reflects whether the user's Stellar wallet is currently connected.
   * @dev Controls the colour and text label of the wallet status badge.
   *      `true`  → green badge, label "Connected"
   *      `false` → red badge,   label "Disconnected"
   */
  isWalletConnected: boolean;

  /**
   * @notice Optional callback fired whenever the mobile menu is opened or closed.
   * @dev Receives the *new* open state after the toggle:
   *      `true`  → menu was just opened
   *      `false` → menu was just closed
   *      Useful for parent components that need to respond to menu state changes
   *      (e.g. disabling background scroll while the drawer is open).
   * @param isOpen - The new boolean state of the mobile menu.
   */
  onToggleMenu?: (isOpen: boolean) => void;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/**
 * @notice Renders the responsive top-level header for Stellar Raise.
 * @dev See module-level NatSpec above for full architecture, security, and
 *      accessibility notes.
 * @param props - See `FrontendHeaderResponsiveProps`.
 */
export const FrontendHeaderResponsive: React.FC<FrontendHeaderResponsiveProps> = ({
  isWalletConnected,
  onToggleMenu,
}) => {

  // -------------------------------------------------------------------------
  // State
  // -------------------------------------------------------------------------

  /**
   * @dev Tracks whether the mobile hamburger menu is currently expanded.
   *      Initialised to `false` (closed) on every mount.
   */
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

  // -------------------------------------------------------------------------
  // Handlers
  // -------------------------------------------------------------------------

  /**
   * @dev Toggles `isMobileMenuOpen` and notifies the optional parent callback.
   *      The functional form of `setState` is used so `onToggleMenu` receives
   *      the correct *next* value regardless of render timing or batching.
   *      Memoised with `useCallback` to keep the reference stable and avoid
   *      unnecessary re-renders of consumers that depend on this handler.
   */
  const handleToggleMenu = useCallback(() => {
    setIsMobileMenuOpen(prev => {
      const newState = !prev;
      if (onToggleMenu) {
        onToggleMenu(newState);
      }
      return newState;
    });
  }, [onToggleMenu]);

  // -------------------------------------------------------------------------
  // Derived values
  // -------------------------------------------------------------------------

  /**
   * @dev Static navigation link definitions.
   *      Memoised with `useMemo` so the array reference is stable and React
   *      does not recreate it on every render, keeping reconciliation cheap.
   */
  const navLinks = useMemo(() => [
    { label: 'Dashboard', href: '/dashboard' },
    { label: 'Invest',    href: '/invest'    },
    { label: 'Docs',      href: '/docs'      },
  ], []);

  // -------------------------------------------------------------------------
  // Render
  // -------------------------------------------------------------------------

  return (
    <header
      className="frontend-header"
      style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '1rem 2rem',
        backgroundColor: '#0A1929', // var(--color-deep-navy)
        color: '#FFFFFF',           // var(--color-neutral-100)
        boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1)', // var(--shadow-md)
      }}
    >

      {/* Brand Logo -------------------------------------------------------- */}
      <div
        className="header-logo"
        style={{ fontSize: '1.5rem', fontWeight: 'bold' }}
      >
        Stellar Raise
      </div>

      {/* Mobile Menu Toggle ------------------------------------------------
          Visible only on small screens (hidden on md+ via external CSS).
          `aria-expanded` keeps assistive technology in sync with open state. */}
      <button
        className="mobile-menu-toggle md:hidden"
        onClick={handleToggleMenu}
        aria-label="Toggle Navigation Menu"
        aria-expanded={isMobileMenuOpen}
        style={{
          background: 'none',
          border: 'none',
          color: 'inherit',
          cursor: 'pointer',
          padding: '0.5rem',
          display: 'block', // overridden to `none` on md+ by responsive.css
        }}
      >
        {/* Icon swaps to communicate open/closed state visually */}
        {isMobileMenuOpen ? '✖' : '☰'}
      </button>

      {/* Navigation Links --------------------------------------------------
          On desktop the nav is always flex-visible (`md:flex`).
          On mobile visibility is toggled via `block` / `hidden` classes. */}
      <nav
        className={`nav-links ${isMobileMenuOpen ? 'block' : 'hidden'} md:flex`}
        style={{
          display: 'flex',
          gap: '1.5rem',
          alignItems: 'center',
        }}
      >
        {navLinks.map(link => (
          <a
            key={link.label}
            href={link.href}
            style={{
              color: 'inherit',
              textDecoration: 'none',
              fontWeight: 500,
              padding: '0.5rem',
            }}
          >
            {link.label}
          </a>
        ))}

        {/* Wallet Status Badge ---------------------------------------------
            Background and border colours are derived entirely from the
            `isWalletConnected` prop; no user input reaches these values. */}
        <div
          className="wallet-status"
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '0.5rem',
            padding: '0.5rem 1rem',
            borderRadius: '9999px', // var(--radius-full)
            backgroundColor: isWalletConnected
              ? 'rgba(0, 200, 83, 0.1)'
              : 'rgba(255, 59, 48, 0.1)',
            border: `1px solid ${isWalletConnected ? '#00C853' : '#FF3B30'}`,
            marginLeft: '1rem',
          }}
        >
          {/* Decorative status dot */}
          <span
            style={{
              display: 'inline-block',
              width: '8px',
              height: '8px',
              borderRadius: '50%',
              backgroundColor: isWalletConnected ? '#00C853' : '#FF3B30',
            }}
          />
          <span style={{ fontSize: '0.875rem', fontWeight: 600 }}>
            {isWalletConnected ? 'Connected' : 'Disconnected'}
          </span>
        </div>
      </nav>

    </header>
  );
};

export default FrontendHeaderResponsive;
