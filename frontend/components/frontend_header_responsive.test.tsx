// import React from 'react';
// import { render, screen, fireEvent } from '@testing-library/react';
// import { FrontendHeaderResponsive } from './frontend_header_responsive';

// describe('FrontendHeaderResponsive Config & Styling', () => {
//   beforeEach(() => {
//     jest.clearAllMocks();
//   });

//   it('renders the branding logo correctly', () => {
//     render(<FrontendHeaderResponsive isWalletConnected={false} />);
//     expect(screen.getByText('Stellar Raise')).toBeTruthy();
//   });

//   it('renders navigation links reliably (gas-efficient rendering)', () => {
//     // Ensuring the memoized links mount as expected
//     render(<FrontendHeaderResponsive isWalletConnected={false} />);
//     expect(screen.getByText('Dashboard')).toBeTruthy();
//     expect(screen.getByText('Invest')).toBeTruthy();
//     expect(screen.getByText('Docs')).toBeTruthy();
//   });

//   it('displays the correct wallet status when disconnected', () => {
//     render(<FrontendHeaderResponsive isWalletConnected={false} />);
//     const statusText = screen.getByText('Disconnected');
//     expect(statusText).toBeTruthy();
    
//     // Testing visual styling applied via dynamic properties
//     const container = statusText.closest('.wallet-status');
//     expect(container).toHaveStyle({ border: '1px solid #FF3B30' });
//   });

//   it('displays the correct wallet status when connected', () => {
//     render(<FrontendHeaderResponsive isWalletConnected={true} />);
//     const statusText = screen.getByText('Connected');
//     expect(statusText).toBeTruthy();
    
//     // Testing visual styling applied via dynamic properties
//     const container = statusText.closest('.wallet-status');
//     expect(container).toHaveStyle({ border: '1px solid #00C853' });
//   });

//   it('toggles mobile menu and invokes the callback properly', () => {
//     const handleToggle = jest.fn();
//     render(
//       <FrontendHeaderResponsive 
//         isWalletConnected={true} 
//         onToggleMenu={handleToggle} 
//       />
//     );

//     // Get the mobile menu toggle button by its aria-label
//     const toggleButton = screen.getByRole('button', { name: "Toggle Navigation Menu" });
    
//     // Initial state check
//     expect(toggleButton).toHaveAttribute('aria-expanded', 'false');
//     expect(toggleButton).toHaveTextContent('☰');

//     // Click to open
//     fireEvent.click(toggleButton);

//     // Verify state changed and callback invoked with 'true'
//     expect(toggleButton).toHaveAttribute('aria-expanded', 'true');
//     expect(toggleButton).toHaveTextContent('✖');
//     expect(handleToggle).toHaveBeenCalledTimes(1);
//     expect(handleToggle).toHaveBeenCalledWith(true);

//     // Click to close
//     fireEvent.click(toggleButton);

//     // Verify state reverted and callback invoked with 'false'
//     expect(toggleButton).toHaveAttribute('aria-expanded', 'false');
//     expect(toggleButton).toHaveTextContent('☰');
//     expect(handleToggle).toHaveBeenCalledTimes(2);
//     expect(handleToggle).toHaveBeenCalledWith(false);
//   });

//   it('toggles mobile menu internally without a provided callback', () => {
//     // Render without onToggleMenu to hit the branch missing the callback
//     render(<FrontendHeaderResponsive isWalletConnected={false} />);
    
//     const toggleButton = screen.getByRole('button', { name: "Toggle Navigation Menu" });
    
//     expect(toggleButton).toHaveAttribute('aria-expanded', 'false');
    
//     // We should be able to click it and just change internal state without throwing
//     fireEvent.click(toggleButton);
//     expect(toggleButton).toHaveAttribute('aria-expanded', 'true');
//   });
// });



/**
 * @file frontend_header_responsive.test.tsx
 * @title Test Suite – FrontendHeaderResponsive
 *
 * @notice Comprehensive unit tests for the responsive header component.
 *
 * @dev Coverage targets (≥ 95 %):
 *   - Rendering: logo, nav links, mobile toggle button, wallet badge
 *   - Props: isWalletConnected (both branches), onToggleMenu (with/without)
 *   - Interactions: toggle open, toggle close, repeated toggles
 *   - Callback correctness: receives new state, not stale state
 *   - Edge cases: no callback provided, multiple rapid clicks
 *
 * @custom:security-notes
 *   - Callback stale-closure test (4.4) confirms `onToggleMenu` always
 *     receives the new state value, never a stale one.
 *   - No test injects raw HTML; all assertions use plain text content,
 *     consistent with the component's XSS-safe design.
 *
 * @custom:test-output
 *   Run: `npm test -- --run` (single pass, no watch)
 *   Expected: all tests pass, ≥ 95% statement/branch/function/line coverage.
 */

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { FrontendHeaderResponsive } from './frontend_header_responsive';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/**
 * @dev Convenience render wrapper – reduces boilerplate in each test.
 */
const renderHeader = (props: Partial<React.ComponentProps<typeof FrontendHeaderResponsive>> = {}) =>
  render(<FrontendHeaderResponsive isWalletConnected={false} {...props} />);

/**
 * @dev Returns the hamburger toggle button by its accessible name.
 */
const getToggleBtn = () =>
  screen.getByRole('button', { name: /toggle navigation menu/i });

// ---------------------------------------------------------------------------
// 1. Rendering
// ---------------------------------------------------------------------------

describe('1. Rendering', () => {
  it('1.1 renders the brand logo text', () => {
    renderHeader();
    expect(screen.getByText('Stellar Raise')).toBeTruthy();
  });

  it('1.2 renders all three default navigation links', () => {
    renderHeader();
describe('FrontendHeaderResponsive Config & Styling', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

/**
 * @dev Convenience render wrapper – reduces boilerplate in each test.
 */
const renderHeader = (props: Partial<React.ComponentProps<typeof FrontendHeaderResponsive>> = {}) =>
  render(<FrontendHeaderResponsive isWalletConnected={false} {...props} />);

/**
 * @dev Returns the hamburger toggle button by its accessible name.
 */
const getToggleBtn = () =>
  screen.getByRole('button', { name: /toggle navigation menu/i });

// ---------------------------------------------------------------------------
// 1. Rendering
// ---------------------------------------------------------------------------

describe('1. Rendering', () => {
  it('1.1 renders the brand logo text', () => {
    renderHeader();
    expect(screen.getByText('Stellar Raise')).toBeTruthy();
  });

  it('1.2 renders all three default navigation links', () => {
    renderHeader();
    expect(screen.getByText('Dashboard')).toBeTruthy();
    expect(screen.getByText('Invest')).toBeTruthy();
    expect(screen.getByText('Docs')).toBeTruthy();
  });

  it('1.3 renders the mobile menu toggle button', () => {
    renderHeader();
    expect(getToggleBtn()).toBeTruthy();
  });

  it('1.4 renders the wallet status badge', () => {
    renderHeader();
    // Badge is present in either connected or disconnected state
    expect(
      screen.getByText('Disconnected') || screen.queryByText('Connected'),
    ).toBeTruthy();
  });

  it('1.5 nav links are anchor elements with correct hrefs', () => {
    renderHeader();
    expect(screen.getByRole('link', { name: 'Dashboard' })).toHaveAttribute('href', '/dashboard');
    expect(screen.getByRole('link', { name: 'Invest' })).toHaveAttribute('href', '/invest');
    expect(screen.getByRole('link', { name: 'Docs' })).toHaveAttribute('href', '/docs');
  });
});

// ---------------------------------------------------------------------------
// 2. Wallet Status Badge
// ---------------------------------------------------------------------------

describe('2. Wallet Status Badge', () => {
  it('2.1 shows "Disconnected" label when isWalletConnected is false', () => {
    renderHeader({ isWalletConnected: false });
    expect(screen.getByText('Disconnected')).toBeTruthy();
  });

  it('2.2 shows "Connected" label when isWalletConnected is true', () => {
    renderHeader({ isWalletConnected: true });
    expect(screen.getByText('Connected')).toBeTruthy();
  });

  it('2.3 applies red border style when wallet is disconnected', () => {
    renderHeader({ isWalletConnected: false });
    const badge = screen.getByText('Disconnected').closest('.wallet-status');
    expect(badge).toHaveStyle({ border: '1px solid #FF3B30' });
  });

  it('2.4 applies green border style when wallet is connected', () => {
    renderHeader({ isWalletConnected: true });
    const badge = screen.getByText('Connected').closest('.wallet-status');
    expect(badge).toHaveStyle({ border: '1px solid #00C853' });
  });

  it('2.5 applies red background tint when wallet is disconnected', () => {
    renderHeader({ isWalletConnected: false });
    const badge = screen.getByText('Disconnected').closest('.wallet-status');
    expect(badge).toHaveStyle({ backgroundColor: 'rgba(255, 59, 48, 0.1)' });
  });

  it('2.6 applies green background tint when wallet is connected', () => {
    renderHeader({ isWalletConnected: true });
    const badge = screen.getByText('Connected').closest('.wallet-status');
    expect(badge).toHaveStyle({ backgroundColor: 'rgba(0, 200, 83, 0.1)' });
  });
});

// ---------------------------------------------------------------------------
// 3. Mobile Menu Toggle – State
// ---------------------------------------------------------------------------

describe('3. Mobile Menu Toggle – State', () => {
  it('3.1 menu starts closed: aria-expanded is "false"', () => {
    renderHeader();
    expect(getToggleBtn()).toHaveAttribute('aria-expanded', 'false');
  });

  it('3.2 shows hamburger icon (☰) when menu is closed', () => {
    renderHeader();
    expect(getToggleBtn()).toHaveTextContent('☰');
  });

  it('3.3 clicking toggle opens the menu: aria-expanded becomes "true"', () => {
    renderHeader();
    fireEvent.click(getToggleBtn());
    expect(getToggleBtn()).toHaveAttribute('aria-expanded', 'true');
  });

  it('3.4 shows close icon (✖) when menu is open', () => {
    renderHeader();
    fireEvent.click(getToggleBtn());
    expect(getToggleBtn()).toHaveTextContent('✖');
  });

  it('3.5 clicking toggle again closes the menu: aria-expanded returns "false"', () => {
    renderHeader();
    fireEvent.click(getToggleBtn()); // open
    fireEvent.click(getToggleBtn()); // close
    expect(getToggleBtn()).toHaveAttribute('aria-expanded', 'false');
  });

  it('3.6 hamburger icon is restored after menu is closed', () => {
    renderHeader();
    fireEvent.click(getToggleBtn()); // open
    fireEvent.click(getToggleBtn()); // close
    expect(getToggleBtn()).toHaveTextContent('☰');
  });

  it('3.7 nav acquires "block" class when menu is open', () => {
    renderHeader();
    fireEvent.click(getToggleBtn());
    const nav = document.querySelector('.nav-links');
    expect(nav?.className).toContain('block');
  });

  it('3.8 nav acquires "hidden" class when menu is closed', () => {
    renderHeader();
    const nav = document.querySelector('.nav-links');
    expect(nav?.className).toContain('hidden');
  });
});

// ---------------------------------------------------------------------------
// 4. onToggleMenu Callback
// ---------------------------------------------------------------------------

describe('4. onToggleMenu Callback', () => {
  it('4.1 fires once with `true` when menu is opened', () => {
    const spy = jest.fn();
    renderHeader({ onToggleMenu: spy });
    fireEvent.click(getToggleBtn());
    expect(spy).toHaveBeenCalledTimes(1);
    expect(spy).toHaveBeenCalledWith(true);
  });

  it('4.2 fires once with `false` when menu is closed', () => {
    const spy = jest.fn();
    renderHeader({ onToggleMenu: spy });
    fireEvent.click(getToggleBtn()); // open  → spy(true)
    fireEvent.click(getToggleBtn()); // close → spy(false)
    expect(spy).toHaveBeenCalledTimes(2);
    expect(spy).toHaveBeenNthCalledWith(2, false);
  });

  it('4.3 does not throw when onToggleMenu is not provided', () => {
    expect(() => {
      renderHeader(); // no onToggleMenu prop
      fireEvent.click(getToggleBtn());
    }).not.toThrow();
  });

  it('4.4 callback always receives the NEW state (stale-closure safety)', () => {
    // Security note: confirms the setState-updater pattern is working correctly.
    // The callback must receive the value *after* the toggle, not before.
    const received: boolean[] = [];
    const spy = jest.fn((v: boolean) => received.push(v));
    renderHeader({ onToggleMenu: spy });

    fireEvent.click(getToggleBtn()); // closed → open  : expect true
    fireEvent.click(getToggleBtn()); // open   → close : expect false
    fireEvent.click(getToggleBtn()); // closed → open  : expect true

    expect(received).toEqual([true, false, true]);
  });

  it('4.5 callback is not called on initial render', () => {
    const spy = jest.fn();
    renderHeader({ onToggleMenu: spy });
    expect(spy).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// 5. Accessibility Attributes
// ---------------------------------------------------------------------------

describe('5. Accessibility Attributes', () => {
  it('5.1 toggle button has aria-label "Toggle Navigation Menu"', () => {
    renderHeader();
    expect(getToggleBtn()).toHaveAttribute('aria-label', 'Toggle Navigation Menu');
  });

  it('5.2 toggle button aria-expanded reflects closed state initially', () => {
    renderHeader();
    expect(getToggleBtn()).toHaveAttribute('aria-expanded', 'false');
  });

  it('5.3 toggle button aria-expanded reflects open state after click', () => {
    renderHeader();
    fireEvent.click(getToggleBtn());
    expect(getToggleBtn()).toHaveAttribute('aria-expanded', 'true');
  });
});

// ---------------------------------------------------------------------------
// 6. Edge Cases
// ---------------------------------------------------------------------------

describe('6. Edge Cases', () => {
  it('6.1 three rapid toggles leaves the menu open', () => {
    renderHeader();
    fireEvent.click(getToggleBtn()); // open
    fireEvent.click(getToggleBtn()); // close
    fireEvent.click(getToggleBtn()); // open
    expect(getToggleBtn()).toHaveAttribute('aria-expanded', 'true');
  });

  it('6.2 four rapid toggles leaves the menu closed', () => {
    renderHeader();
    fireEvent.click(getToggleBtn()); // open
    fireEvent.click(getToggleBtn()); // close
    fireEvent.click(getToggleBtn()); // open
    fireEvent.click(getToggleBtn()); // close
    expect(getToggleBtn()).toHaveAttribute('aria-expanded', 'false');
  });

  it('6.3 component renders without error when only required prop is supplied', () => {
    expect(() =>
      render(<FrontendHeaderResponsive isWalletConnected={false} />),
    ).not.toThrow();
  });

  it('6.4 component renders without error with all props supplied', () => {
    expect(() =>
      render(
        <FrontendHeaderResponsive
          isWalletConnected={true}
          onToggleMenu={jest.fn()}
        />,
      ),
    ).not.toThrow();
  it('displays the correct wallet status when disconnected', () => {
    render(<FrontendHeaderResponsive isWalletConnected={false} />);
    const statusText = screen.getByText('Disconnected');
    expect(statusText).toBeTruthy();
    
    // Testing visual styling applied via dynamic properties
    const container = statusText.closest('.wallet-status');
    expect(container).toHaveStyle({ border: '1px solid #FF3B30' });
  });

  it('1.4 renders the wallet status badge', () => {
    renderHeader();
    // Badge is present in either connected or disconnected state
    expect(
      screen.getByText('Disconnected') || screen.queryByText('Connected'),
    ).toBeTruthy();
  });

  it('1.5 nav links are anchor elements with correct hrefs', () => {
    renderHeader();
    expect(screen.getByRole('link', { name: 'Dashboard' })).toHaveAttribute('href', '/dashboard');
    expect(screen.getByRole('link', { name: 'Invest' })).toHaveAttribute('href', '/invest');
    expect(screen.getByRole('link', { name: 'Docs' })).toHaveAttribute('href', '/docs');
  });
});

// ---------------------------------------------------------------------------
// 2. Wallet Status Badge
// ---------------------------------------------------------------------------

describe('2. Wallet Status Badge', () => {
  it('2.1 shows "Disconnected" label when isWalletConnected is false', () => {
    renderHeader({ isWalletConnected: false });
    expect(screen.getByText('Disconnected')).toBeTruthy();
  });

  it('2.2 shows "Connected" label when isWalletConnected is true', () => {
    renderHeader({ isWalletConnected: true });
    expect(screen.getByText('Connected')).toBeTruthy();
  });

  it('2.3 applies red border style when wallet is disconnected', () => {
    renderHeader({ isWalletConnected: false });
    const badge = screen.getByText('Disconnected').closest('.wallet-status');
    expect(badge).toHaveStyle({ border: '1px solid #FF3B30' });
  });

  it('2.4 applies green border style when wallet is connected', () => {
    renderHeader({ isWalletConnected: true });
    const badge = screen.getByText('Connected').closest('.wallet-status');
    expect(badge).toHaveStyle({ border: '1px solid #00C853' });
  });

  it('2.5 applies red background tint when wallet is disconnected', () => {
    renderHeader({ isWalletConnected: false });
    const badge = screen.getByText('Disconnected').closest('.wallet-status');
    expect(badge).toHaveStyle({ backgroundColor: 'rgba(255, 59, 48, 0.1)' });
  });

  it('2.6 applies green background tint when wallet is connected', () => {
    renderHeader({ isWalletConnected: true });
    const badge = screen.getByText('Connected').closest('.wallet-status');
    expect(badge).toHaveStyle({ backgroundColor: 'rgba(0, 200, 83, 0.1)' });
  });
});

// ---------------------------------------------------------------------------
// 3. Mobile Menu Toggle – State
// ---------------------------------------------------------------------------

describe('3. Mobile Menu Toggle – State', () => {
  it('3.1 menu starts closed: aria-expanded is "false"', () => {
    renderHeader();
    expect(getToggleBtn()).toHaveAttribute('aria-expanded', 'false');
  });

  it('3.2 shows hamburger icon (☰) when menu is closed', () => {
    renderHeader();
    expect(getToggleBtn()).toHaveTextContent('☰');
  });

  it('3.3 clicking toggle opens the menu: aria-expanded becomes "true"', () => {
    renderHeader();
    fireEvent.click(getToggleBtn());
    expect(getToggleBtn()).toHaveAttribute('aria-expanded', 'true');
  });

  it('3.4 shows close icon (✖) when menu is open', () => {
    renderHeader();
    fireEvent.click(getToggleBtn());
    expect(getToggleBtn()).toHaveTextContent('✖');
  });

  it('3.5 clicking toggle again closes the menu: aria-expanded returns "false"', () => {
    renderHeader();
    fireEvent.click(getToggleBtn()); // open
    fireEvent.click(getToggleBtn()); // close
    expect(getToggleBtn()).toHaveAttribute('aria-expanded', 'false');
  });

  it('3.6 hamburger icon is restored after menu is closed', () => {
    renderHeader();
    fireEvent.click(getToggleBtn()); // open
    fireEvent.click(getToggleBtn()); // close
    expect(getToggleBtn()).toHaveTextContent('☰');
  });

  it('3.7 nav acquires "block" class when menu is open', () => {
    renderHeader();
    fireEvent.click(getToggleBtn());
    const nav = document.querySelector('.nav-links');
    expect(nav?.className).toContain('block');
  });

  it('3.8 nav acquires "hidden" class when menu is closed', () => {
    renderHeader();
    const nav = document.querySelector('.nav-links');
    expect(nav?.className).toContain('hidden');
  });
});

// ---------------------------------------------------------------------------
// 4. onToggleMenu Callback
// ---------------------------------------------------------------------------

describe('4. onToggleMenu Callback', () => {
  it('4.1 fires once with `true` when menu is opened', () => {
    const spy = jest.fn();
    renderHeader({ onToggleMenu: spy });
    fireEvent.click(getToggleBtn());
    expect(spy).toHaveBeenCalledTimes(1);
    expect(spy).toHaveBeenCalledWith(true);
  });

  it('4.2 fires once with `false` when menu is closed', () => {
    const spy = jest.fn();
    renderHeader({ onToggleMenu: spy });
    fireEvent.click(getToggleBtn()); // open  → spy(true)
    fireEvent.click(getToggleBtn()); // close → spy(false)
    expect(spy).toHaveBeenCalledTimes(2);
    expect(spy).toHaveBeenNthCalledWith(2, false);
  });

  it('4.3 does not throw when onToggleMenu is not provided', () => {
    expect(() => {
      renderHeader(); // no onToggleMenu prop
      fireEvent.click(getToggleBtn());
    }).not.toThrow();
  });

  it('4.4 callback always receives the NEW state (stale-closure safety)', () => {
    // Security note: confirms the setState-updater pattern is working correctly.
    // The callback must receive the value *after* the toggle, not before.
    const received: boolean[] = [];
    const spy = jest.fn((v: boolean) => received.push(v));
    renderHeader({ onToggleMenu: spy });

    fireEvent.click(getToggleBtn()); // closed → open  : expect true
    fireEvent.click(getToggleBtn()); // open   → close : expect false
    fireEvent.click(getToggleBtn()); // closed → open  : expect true

    expect(received).toEqual([true, false, true]);
  });

  it('4.5 callback is not called on initial render', () => {
    const spy = jest.fn();
    renderHeader({ onToggleMenu: spy });
    expect(spy).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// 5. Accessibility Attributes
// ---------------------------------------------------------------------------

describe('5. Accessibility Attributes', () => {
  it('5.1 toggle button has aria-label "Toggle Navigation Menu"', () => {
    renderHeader();
    expect(getToggleBtn()).toHaveAttribute('aria-label', 'Toggle Navigation Menu');
  });

  it('5.2 toggle button aria-expanded reflects closed state initially', () => {
    renderHeader();
    expect(getToggleBtn()).toHaveAttribute('aria-expanded', 'false');
  });

  it('5.3 toggle button aria-expanded reflects open state after click', () => {
    renderHeader();
    fireEvent.click(getToggleBtn());
    expect(getToggleBtn()).toHaveAttribute('aria-expanded', 'true');
  });
});

// ---------------------------------------------------------------------------
// 6. Edge Cases
// ---------------------------------------------------------------------------

describe('6. Edge Cases', () => {
  it('6.1 three rapid toggles leaves the menu open', () => {
    renderHeader();
    fireEvent.click(getToggleBtn()); // open
    fireEvent.click(getToggleBtn()); // close
    fireEvent.click(getToggleBtn()); // open
    expect(getToggleBtn()).toHaveAttribute('aria-expanded', 'true');
  });

  it('6.2 four rapid toggles leaves the menu closed', () => {
    renderHeader();
    fireEvent.click(getToggleBtn()); // open
    fireEvent.click(getToggleBtn()); // close
    fireEvent.click(getToggleBtn()); // open
    fireEvent.click(getToggleBtn()); // close
    expect(getToggleBtn()).toHaveAttribute('aria-expanded', 'false');
  });

  it('6.3 component renders without error when only required prop is supplied', () => {
    expect(() =>
      render(<FrontendHeaderResponsive isWalletConnected={false} />),
    ).not.toThrow();
  });

  it('6.4 component renders without error with all props supplied', () => {
    expect(() =>
      render(
        <FrontendHeaderResponsive
          isWalletConnected={true}
          onToggleMenu={jest.fn()}
        />,
      ),
    ).not.toThrow();
  });
});
