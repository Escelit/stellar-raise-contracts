/**
 * @title Frontend Header Responsive Styling Utility
 * @notice Secure utility for responsive header styling with edge case handling
 * @dev Provides type-safe responsive header behavior with validation and optimization
 * @author Stellar Raise Security Team
 * @notice SECURITY: All header responsive operations must go through this utility to prevent
 *         layout shifts and ensure consistent cross-device experience.
 */

/**
 * @notice Simple CSS properties type for inline styles
 */
type CssProperties = Record<string, string | number>;

/**
 * @notice Predefined list of allowed breakpoint names
 * @dev Only breakpoints defined in this list can be used
 */
export const ALLOWED_BREAKPOINTS = [
  'mobile',
  'tablet',
  'desktop',
  'wide',
  'ultra-wide',
] as const;

/**
 * @notice Predefined header layout modes
 */
export const HEADER_LAYOUT_MODES = [
  'static',
  'sticky',
  'fixed',
  'floating',
] as const;

/**
 * @notice Predefined header visibility states
 */
export const HEADER_VISIBILITY_STATES = [
  'visible',
  'hidden',
  'collapsed',
  'compact',
] as const;

/**
 * @notice Type for allowed breakpoint names
 */
export type AllowedBreakpoint = typeof ALLOWED_BREAKPOINTS[number];

/**
 * @notice Type for header layout modes
 */
export type HeaderLayoutMode = typeof HEADER_LAYOUT_MODES[number];

/**
 * @notice Type for header visibility states
 */
export type HeaderVisibilityState = typeof HEADER_VISIBILITY_STATES[number];

/**
 * @notice Breakpoint configuration interface
 */
export interface BreakpointConfig {
  /** Breakpoint name */
  name: AllowedBreakpoint;
  /** Minimum width in pixels */
  minWidth: number;
  /** Maximum width in pixels */
  maxWidth: number;
  /** Whether this is the default breakpoint */
  isDefault: boolean;
}

/**
 * @notice Header responsive configuration interface
 */
export interface HeaderResponsiveConfig {
  /** Current breakpoint */
  breakpoint: AllowedBreakpoint;
  /** Layout mode */
  layoutMode: HeaderLayoutMode;
  /** Visibility state */
  visibility: HeaderVisibilityState;
  /** Whether to show navigation */
  showNavigation: boolean;
  /** Whether to show search */
  showSearch: boolean;
  /** Whether to show user menu */
  showUserMenu: boolean;
  /** Header height in pixels */
  height: number;
  /** Whether to enable animations */
  animationsEnabled: boolean;
}

/**
 * @notice Default breakpoint configurations
 */
export const BREAKPOINT_CONFIGS: Record<AllowedBreakpoint, BreakpointConfig> = {
  mobile: {
    name: 'mobile',
    minWidth: 0,
    maxWidth: 479,
    isDefault: false,
  },
  tablet: {
    name: 'tablet',
    minWidth: 480,
    maxWidth: 767,
    isDefault: false,
  },
  desktop: {
    name: 'desktop',
    minWidth: 768,
    maxWidth: 1023,
    isDefault: false,
  },
  wide: {
    name: 'wide',
    minWidth: 1024,
    maxWidth: 1439,
    isDefault: false,
  },
  'ultra-wide': {
    name: 'ultra-wide',
    minWidth: 1440,
    maxWidth: Number.MAX_SAFE_INTEGER,
    isDefault: false,
  },
};

/**
 * @notice Default header configurations per breakpoint
 */
export const DEFAULT_HEADER_CONFIGS: Record<AllowedBreakpoint, HeaderResponsiveConfig> = {
  mobile: {
    breakpoint: 'mobile',
    layoutMode: 'fixed',
    visibility: 'visible',
    showNavigation: false,
    showSearch: false,
    showUserMenu: true,
    height: 56,
    animationsEnabled: true,
  },
  tablet: {
    breakpoint: 'tablet',
    layoutMode: 'fixed',
    visibility: 'visible',
    showNavigation: true,
    showSearch: true,
    showUserMenu: true,
    height: 64,
    animationsEnabled: true,
  },
  desktop: {
    breakpoint: 'desktop',
    layoutMode: 'sticky',
    visibility: 'visible',
    showNavigation: true,
    showSearch: true,
    showUserMenu: true,
    height: 72,
    animationsEnabled: false,
  },
  wide: {
    breakpoint: 'wide',
    layoutMode: 'sticky',
    visibility: 'visible',
    showNavigation: true,
    showSearch: true,
    showUserMenu: true,
    height: 80,
    animationsEnabled: false,
  },
  'ultra-wide': {
    breakpoint: 'ultra-wide',
    layoutMode: 'sticky',
    visibility: 'visible',
    showNavigation: true,
    showSearch: true,
    showUserMenu: true,
    height: 80,
    animationsEnabled: false,
  },
};

/**
 * @title FrontendHeaderResponsiveError
 * @notice Custom error class for header responsive operations
 */
export class FrontendHeaderResponsiveError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'FrontendHeaderResponsiveError';
  }
}

/**
 * @title BreakpointValidator
 * @notice Static validator class for breakpoint operations
 */
export class BreakpointValidator {
  /**
   * @notice Validates if a breakpoint name is allowed
   * @param breakpoint The breakpoint name to validate
   * @returns True if the breakpoint is valid
   * @throws FrontendHeaderResponsiveError if validation fails
   */
  static isValidBreakpoint(breakpoint: string): boolean {
    if (!ALLOWED_BREAKPOINTS.includes(breakpoint as AllowedBreakpoint)) {
      throw new FrontendHeaderResponsiveError(
        `Invalid breakpoint "${breakpoint}". Allowed values: ${ALLOWED_BREAKPOINTS.join(', ')}`
      );
    }
    return true;
  }

  /**
   * @notice Validates if a layout mode is allowed
   * @param layoutMode The layout mode to validate
   * @returns True if the layout mode is valid
   * @throws FrontendHeaderResponsiveError if validation fails
   */
  static isValidLayoutMode(layoutMode: string): boolean {
    if (!HEADER_LAYOUT_MODES.includes(layoutMode as HeaderLayoutMode)) {
      throw new FrontendHeaderResponsiveError(
        `Invalid layout mode "${layoutMode}". Allowed values: ${HEADER_LAYOUT_MODES.join(', ')}`
      );
    }
    return true;
  }

  /**
   * @notice Validates if a visibility state is allowed
   * @param visibility The visibility state to validate
   * @returns True if the visibility state is valid
   * @throws FrontendHeaderResponsiveError if validation fails
   */
  static isValidVisibilityState(visibility: string): boolean {
    if (!HEADER_VISIBILITY_STATES.includes(visibility as HeaderVisibilityState)) {
      throw new FrontendHeaderResponsiveError(
        `Invalid visibility state "${visibility}". Allowed values: ${HEADER_VISIBILITY_STATES.join(', ')}`
      );
    }
    return true;
  }

  /**
   * @notice Gets the breakpoint configuration
   * @param breakpoint The breakpoint name
   * @returns The breakpoint configuration
   * @throws FrontendHeaderResponsiveError if breakpoint is invalid
   */
  static getBreakpointConfig(breakpoint: AllowedBreakpoint): BreakpointConfig {
    BreakpointValidator.isValidBreakpoint(breakpoint);
    return BREAKPOINT_CONFIGS[breakpoint];
  }

  /**
   * @notice Gets all allowed breakpoints
   * @returns Readonly array of allowed breakpoint names
   */
  static getAllowedBreakpoints(): readonly string[] {
    return ALLOWED_BREAKPOINTS;
  }
}

/**
 * @title FrontendHeaderResponsive
 * @notice Main utility class for responsive header operations
 */
export class FrontendHeaderResponsive {
  private currentBreakpoint: AllowedBreakpoint;
  private config: HeaderResponsiveConfig;
  private listeners: Map<string, Set<(config: HeaderResponsiveConfig) => void>>;
  private resizeObserver: ResizeObserver | null;
  private lastWidth: number;

  /**
   * @notice Creates a new FrontendHeaderResponsive instance
   * @param initialBreakpoint The initial breakpoint (defaults to 'mobile' for SSR safety)
   */
  constructor(initialBreakpoint?: AllowedBreakpoint) {
    this.currentBreakpoint = initialBreakpoint || 'mobile';
    this.config = { ...DEFAULT_HEADER_CONFIGS[this.currentBreakpoint] };
    this.listeners = new Map();
    this.resizeObserver = null;
    this.lastWidth = 0;
  }

  /**
   * @notice Gets the current breakpoint
   * @returns The current breakpoint name
   */
  getBreakpoint(): AllowedBreakpoint {
    return this.currentBreakpoint;
  }

  /**
   * @notice Gets the current header configuration
   * @returns The current header responsive configuration
   */
  getConfig(): HeaderResponsiveConfig {
    return { ...this.config };
  }

  /**
   * @notice Updates the current breakpoint based on window width
   * @param width The current window width in pixels
   * @returns The new breakpoint
   */
  updateBreakpoint(width: number): AllowedBreakpoint {
    // Edge case: Handle negative or zero width
    if (width <= 0) {
      width = 0;
    }

    // Edge case: Handle very large widths
    if (width > 10000) {
      width = 10000;
    }

    let newBreakpoint: AllowedBreakpoint = 'mobile';

    for (const [name, config] of Object.entries(BREAKPOINT_CONFIGS)) {
      if (width >= config.minWidth && width <= config.maxWidth) {
        newBreakpoint = name as AllowedBreakpoint;
        break;
      }
    }

    // Edge case: Handle exact boundary values
    if (width === 480) {
      newBreakpoint = 'tablet';
    } else if (width === 768) {
      newBreakpoint = 'desktop';
    } else if (width === 1024) {
      newBreakpoint = 'wide';
    } else if (width === 1440) {
      newBreakpoint = 'ultra-wide';
    }

    if (newBreakpoint !== this.currentBreakpoint) {
      this.currentBreakpoint = newBreakpoint;
      this.config = { ...DEFAULT_HEADER_CONFIGS[newBreakpoint] };
      this.notifyListeners();
    }

    this.lastWidth = width;
    return this.currentBreakpoint;
  }

  /**
   * @notice Sets a custom header configuration
   * @param config The configuration to set
   */
  setConfig(config: Partial<HeaderResponsiveConfig>): void {
    // Validate breakpoint if provided
    if (config.breakpoint) {
      BreakpointValidator.isValidBreakpoint(config.breakpoint);
    }

    // Validate layout mode if provided
    if (config.layoutMode) {
      BreakpointValidator.isValidLayoutMode(config.layoutMode);
    }

    // Validate visibility if provided
    if (config.visibility) {
      BreakpointValidator.isValidVisibilityState(config.visibility);
    }

    this.config = { ...this.config, ...config };
    this.notifyListeners();
  }

  /**
   * @notice Gets the header height for the current breakpoint
   * @returns The header height in pixels
   */
  getHeight(): number {
    return this.config.height;
  }

  /**
   * @notice Gets the CSS class name for the current state
   * @returns The CSS class name
   */
  getCssClassName(): string {
    const classes: string[] = [
      'header',
      `header--${this.config.layoutMode}`,
      `header--${this.config.visibility}`,
      `header--${this.currentBreakpoint}`,
    ];

    if (this.config.showNavigation) {
      classes.push('header--with-navigation');
    }

    if (this.config.showSearch) {
      classes.push('header--with-search');
    }

    if (this.config.showUserMenu) {
      classes.push('header--with-user-menu');
    }

    if (this.config.animationsEnabled) {
      classes.push('header--animated');
    }

    return classes.join(' ');
  }

  /**
   * @notice Gets the inline styles for the header
   * @returns The CSS properties as an object
   */
  getInlineStyles(): CssProperties {
    const styles: CssProperties = {
      height: `${this.config.height}px`,
    };

    switch (this.config.layoutMode) {
      case 'sticky':
        styles.position = 'sticky';
        styles.top = '0';
        styles.zIndex = '1000';
        break;
      case 'fixed':
        styles.position = 'fixed';
        styles.top = '0';
        styles.left = '0';
        styles.right = '0';
        styles.zIndex = '1000';
        break;
      case 'floating':
        styles.position = 'relative';
        styles.margin = '1rem';
        styles.borderRadius = '8px';
        styles.boxShadow = '0 4px 6px -1px rgba(0, 0, 0, 0.1)';
        break;
      case 'static':
      default:
        styles.position = 'relative';
        break;
    }

    switch (this.config.visibility) {
      case 'hidden':
        styles.display = 'none';
        break;
      case 'collapsed':
        styles.height = '0';
        styles.overflow = 'hidden';
        styles.padding = '0';
        break;
      case 'compact':
        styles.height = `${this.config.height * 0.75}px`;
        break;
      case 'visible':
      default:
        break;
    }

    return styles;
  }

  /**
   * @notice Subscribes to configuration changes
   * @param id Unique identifier for the listener
   * @param callback Function to call when configuration changes
   */
  subscribe(id: string, callback: (config: HeaderResponsiveConfig) => void): void {
    if (!this.listeners.has(id)) {
      this.listeners.set(id, new Set());
    }
    this.listeners.get(id)!.add(callback);
  }

  /**
   * @notice Unsubscribes from configuration changes
   * @param id Unique identifier for the listener
   */
  unsubscribe(id: string): void {
    this.listeners.delete(id);
  }

  /**
   * @notice Initializes the responsive behavior with window resize detection
   * @returns Cleanup function to remove event listeners
   */
  initialize(): () => void {
    if (typeof window === 'undefined') {
      return () => {};
    }

    const handleResize = () => {
      this.updateBreakpoint(window.innerWidth);
    };

    window.addEventListener('resize', handleResize);
    
    // Initialize with current width
    this.updateBreakpoint(window.innerWidth);

    return () => {
      window.removeEventListener('resize', handleResize);
    };
  }

  /**
   * @notice Cleans up resources
   */
  destroy(): void {
    this.listeners.clear();
    if (this.resizeObserver) {
      this.resizeObserver.disconnect();
      this.resizeObserver = null;
    }
  }

  /**
   * @notice Notifies all listeners of configuration changes
   */
  private notifyListeners(): void {
    for (const callbacks of this.listeners.values()) {
      for (const callback of callbacks) {
        callback(this.getConfig());
      }
    }
  }
}

/**
 * @title Header Responsive Hooks for React
 * @notice React hooks for header responsive operations
 */

/**
 * @notice Gets the current header responsive configuration as a React hook
 * @param initialBreakpoint The initial breakpoint
 * @returns The header responsive instance
 */
export function useHeaderResponsive(initialBreakpoint?: AllowedBreakpoint): FrontendHeaderResponsive {
  if (typeof window === 'undefined') {
    // Return a mock instance for SSR
    return new FrontendHeaderResponsive(initialBreakpoint || 'mobile');
  }

  return new FrontendHeaderResponsive(initialBreakpoint);
}

/**
 * @title Helper Functions
 */

/**
 * @notice Gets the breakpoint name from window width
 * @param width The window width in pixels
 * @returns The breakpoint name
 */
export function getBreakpointFromWidth(width: number): AllowedBreakpoint {
  // Edge case: Handle negative or zero width
  if (width <= 0) {
    return 'mobile';
  }

  // Edge case: Handle very large widths
  if (width > 10000) {
    return 'ultra-wide';
  }

  if (width >= 1440) {
    return 'ultra-wide';
  } else if (width >= 1024) {
    return 'wide';
  } else if (width >= 768) {
    return 'desktop';
  } else if (width >= 480) {
    return 'tablet';
  }
  return 'mobile';
}

/**
 * @notice Checks if the current device is a mobile device
 * @param width The window width in pixels
 * @returns True if the device is mobile
 */
export function isMobileDevice(width: number): boolean {
  return width < 768;
}

/**
 * @notice Checks if the current device is a tablet
 * @param width The window width in pixels
 * @returns True if the device is a tablet
 */
export function isTabletDevice(width: number): boolean {
  return width >= 768 && width < 1024;
}

/**
 * @notice Checks if the current device is a desktop
 * @param width The window width in pixels
 * @returns True if the device is a desktop
 */
export function isDesktopDevice(width: number): boolean {
  return width >= 1024;
}

/**
 * @notice Gets the media query string for a breakpoint
 * @param breakpoint The breakpoint name
 * @param type The type of media query ('min' or 'max')
 * @returns The media query string
 */
export function getMediaQuery(breakpoint: AllowedBreakpoint, type: 'min' | 'max' = 'min'): string {
  BreakpointValidator.isValidBreakpoint(breakpoint);
  const config = BREAKPOINT_CONFIGS[breakpoint];

  if (type === 'min') {
    return `(min-width: ${config.minWidth}px)`;
  }
  return `(max-width: ${config.maxWidth}px)`;
}

/**
 * @notice Creates a responsive header class name
 * @param breakpoint The current breakpoint
 * @param options Additional options
 * @returns The CSS class name
 */
export function createResponsiveHeaderClass(
  breakpoint: AllowedBreakpoint,
  options: {
    layoutMode?: HeaderLayoutMode;
    visibility?: HeaderVisibilityState;
    showNavigation?: boolean;
    showSearch?: boolean;
    showUserMenu?: boolean;
    animationsEnabled?: boolean;
  } = {}
): string {
  const classes: string[] = [
    'header',
    `header--${options.layoutMode || 'static'}`,
    `header--${options.visibility || 'visible'}`,
    `header--${breakpoint}`,
  ];

  if (options.showNavigation) {
    classes.push('header--with-navigation');
  }

  if (options.showSearch) {
    classes.push('header--with-search');
  }

  if (options.showUserMenu) {
    classes.push('header--with-user-menu');
  }

  if (options.animationsEnabled) {
    classes.push('header--animated');
  }

  return classes.join(' ');
}

// Default export for convenience
export default FrontendHeaderResponsive;
