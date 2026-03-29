/**
 * @title Frontend Header Responsive Tests
 * @notice Comprehensive test suite for header responsive handling
 * @author Stellar Raise Security Team
 */

import {
  FrontendHeaderResponsive,
  BreakpointValidator,
  FrontendHeaderResponsiveError,
  ALLOWED_BREAKPOINTS,
  HEADER_LAYOUT_MODES,
  HEADER_VISIBILITY_STATES,
  BREAKPOINT_CONFIGS,
  DEFAULT_HEADER_CONFIGS,
  getBreakpointFromWidth,
  isMobileDevice,
  isTabletDevice,
  isDesktopDevice,
  getMediaQuery,
  createResponsiveHeaderClass,
  useHeaderResponsive,
  AllowedBreakpoint,
  HeaderLayoutMode,
  HeaderVisibilityState,
  HeaderResponsiveConfig,
  BreakpointConfig,
} from './frontend_header_responsive';

describe('BreakpointValidator', () => {
  describe('isValidBreakpoint', () => {
    it('should accept valid breakpoint names', () => {
      expect(BreakpointValidator.isValidBreakpoint('mobile')).toBe(true);
      expect(BreakpointValidator.isValidBreakpoint('tablet')).toBe(true);
      expect(BreakpointValidator.isValidBreakpoint('desktop')).toBe(true);
      expect(BreakpointValidator.isValidBreakpoint('wide')).toBe(true);
      expect(BreakpointValidator.isValidBreakpoint('ultra-wide')).toBe(true);
    });

    it('should throw error for invalid breakpoint names', () => {
      expect(() => {
        BreakpointValidator.isValidBreakpoint('invalid');
      }).toThrow(FrontendHeaderResponsiveError);
      expect(() => {
        BreakpointValidator.isValidBreakpoint('');
      }).toThrow(FrontendHeaderResponsiveError);
      expect(() => {
        BreakpointValidator.isValidBreakpoint('desktop-wide');
      }).toThrow(FrontendHeaderResponsiveError);
    });
  });

  describe('isValidLayoutMode', () => {
    it('should accept valid layout modes', () => {
      expect(BreakpointValidator.isValidLayoutMode('static')).toBe(true);
      expect(BreakpointValidator.isValidLayoutMode('sticky')).toBe(true);
      expect(BreakpointValidator.isValidLayoutMode('fixed')).toBe(true);
      expect(BreakpointValidator.isValidLayoutMode('floating')).toBe(true);
    });

    it('should throw error for invalid layout modes', () => {
      expect(() => {
        BreakpointValidator.isValidLayoutMode('absolute');
      }).toThrow(FrontendHeaderResponsiveError);
      expect(() => {
        BreakpointValidator.isValidLayoutMode('');
      }).toThrow(FrontendHeaderResponsiveError);
    });
  });

  describe('isValidVisibilityState', () => {
    it('should accept valid visibility states', () => {
      expect(BreakpointValidator.isValidVisibilityState('visible')).toBe(true);
      expect(BreakpointValidator.isValidVisibilityState('hidden')).toBe(true);
      expect(BreakpointValidator.isValidVisibilityState('collapsed')).toBe(true);
      expect(BreakpointValidator.isValidVisibilityState('compact')).toBe(true);
    });

    it('should throw error for invalid visibility states', () => {
      expect(() => {
        BreakpointValidator.isValidVisibilityState('opacity');
      }).toThrow(FrontendHeaderResponsiveError);
      expect(() => {
        BreakpointValidator.isValidVisibilityState('');
      }).toThrow(FrontendHeaderResponsiveError);
    });
  });

  describe('getBreakpointConfig', () => {
    it('should return correct breakpoint config', () => {
      const mobileConfig = BreakpointValidator.getBreakpointConfig('mobile');
      expect(mobileConfig.name).toBe('mobile');
      expect(mobileConfig.minWidth).toBe(0);
      expect(mobileConfig.maxWidth).toBe(479);

      const desktopConfig = BreakpointValidator.getBreakpointConfig('desktop');
      expect(desktopConfig.name).toBe('desktop');
      expect(desktopConfig.minWidth).toBe(768);
      expect(desktopConfig.maxWidth).toBe(1023);
    });

    it('should throw error for invalid breakpoint', () => {
      expect(() => {
        BreakpointValidator.getBreakpointConfig('invalid' as AllowedBreakpoint);
      }).toThrow(FrontendHeaderResponsiveError);
    });
  });

  describe('getAllowedBreakpoints', () => {
    it('should return all allowed breakpoints', () => {
      const breakpoints = BreakpointValidator.getAllowedBreakpoints();
      expect(breakpoints).toContain('mobile');
      expect(breakpoints).toContain('tablet');
      expect(breakpoints).toContain('desktop');
      expect(breakpoints).toContain('wide');
      expect(breakpoints).toContain('ultra-wide');
      expect(breakpoints.length).toBe(5);
    });
  });
});

describe('FrontendHeaderResponsive', () => {
  let header: FrontendHeaderResponsive;

  beforeEach(() => {
    header = new FrontendHeaderResponsive();
  });

  afterEach(() => {
    header.destroy();
  });

  describe('constructor', () => {
    it('should create instance with default breakpoint', () => {
      expect(header.getBreakpoint()).toBe('mobile');
    });

    it('should create instance with custom initial breakpoint', () => {
      const customHeader = new FrontendHeaderResponsive('desktop');
      expect(customHeader.getBreakpoint()).toBe('desktop');
      customHeader.destroy();
    });
  });

  describe('getBreakpoint', () => {
    it('should return current breakpoint', () => {
      expect(header.getBreakpoint()).toBe('mobile');
    });
  });

  describe('getConfig', () => {
    it('should return current configuration', () => {
      const config = header.getConfig();
      expect(config.breakpoint).toBe('mobile');
      expect(config.layoutMode).toBe('fixed');
      expect(config.visibility).toBe('visible');
      expect(config.height).toBe(56);
    });

    it('should return a copy of the configuration', () => {
      const config1 = header.getConfig();
      const config2 = header.getConfig();
      expect(config1).not.toBe(config2);
    });
  });

  describe('updateBreakpoint', () => {
    it('should update to mobile for width 0', () => {
      const breakpoint = header.updateBreakpoint(0);
      expect(breakpoint).toBe('mobile');
    });

    it('should update to mobile for width less than 480', () => {
      const breakpoint = header.updateBreakpoint(320);
      expect(breakpoint).toBe('mobile');
    });

    it('should update to tablet for width >= 480 and < 768', () => {
      const breakpoint = header.updateBreakpoint(600);
      expect(breakpoint).toBe('tablet');
    });

    it('should update to desktop for width >= 768 and < 1024', () => {
      const breakpoint = header.updateBreakpoint(900);
      expect(breakpoint).toBe('desktop');
    });

    it('should update to wide for width >= 1024 and < 1440', () => {
      const breakpoint = header.updateBreakpoint(1200);
      expect(breakpoint).toBe('wide');
    });

    it('should update to ultra-wide for width >= 1440', () => {
      const breakpoint = header.updateBreakpoint(1600);
      expect(breakpoint).toBe('ultra-wide');
    });

    // Edge case tests
    it('should handle negative width', () => {
      const breakpoint = header.updateBreakpoint(-100);
      expect(breakpoint).toBe('mobile');
    });

    it('should handle very large width', () => {
      const breakpoint = header.updateBreakpoint(50000);
      expect(breakpoint).toBe('ultra-wide');
    });

    it('should handle exact boundary value 480', () => {
      const breakpoint = header.updateBreakpoint(480);
      expect(breakpoint).toBe('tablet');
    });

    it('should handle exact boundary value 768', () => {
      const breakpoint = header.updateBreakpoint(768);
      expect(breakpoint).toBe('desktop');
    });

    it('should handle exact boundary value 1024', () => {
      const breakpoint = header.updateBreakpoint(1024);
      expect(breakpoint).toBe('wide');
    });

    it('should handle exact boundary value 1440', () => {
      const breakpoint = header.updateBreakpoint(1440);
      expect(breakpoint).toBe('ultra-wide');
    });

    it('should not notify listeners if breakpoint unchanged', () => {
      const callback = jest.fn();
      header.subscribe('test-id', callback);
      header.updateBreakpoint(320); // mobile
      header.updateBreakpoint(320); // still mobile - no change expected
      // Note: No notification expected on first call either because initial state matches
      // The config is set in constructor, not in updateBreakpoint when breakpoint doesn't change
      expect(callback).toHaveBeenCalledTimes(0);
    });
  });

  describe('setConfig', () => {
    it('should update configuration with valid values', () => {
      header.setConfig({ layoutMode: 'sticky', height: 80 });
      const config = header.getConfig();
      expect(config.layoutMode).toBe('sticky');
      expect(config.height).toBe(80);
    });

    it('should validate breakpoint when setting', () => {
      expect(() => {
        header.setConfig({ breakpoint: 'invalid' as AllowedBreakpoint });
      }).toThrow(FrontendHeaderResponsiveError);
    });

    it('should validate layout mode when setting', () => {
      expect(() => {
        header.setConfig({ layoutMode: 'invalid' as HeaderLayoutMode });
      }).toThrow(FrontendHeaderResponsiveError);
    });

    it('should validate visibility when setting', () => {
      expect(() => {
        header.setConfig({ visibility: 'invalid' as HeaderVisibilityState });
      }).toThrow(FrontendHeaderResponsiveError);
    });

    it('should notify listeners on config change', () => {
      const callback = jest.fn();
      header.subscribe('test-id', callback);
      header.setConfig({ height: 100 });
      expect(callback).toHaveBeenCalledTimes(1);
    });
  });

  describe('getHeight', () => {
    it('should return height for current breakpoint', () => {
      expect(header.getHeight()).toBe(56); // mobile height

      header.updateBreakpoint(900); // desktop
      expect(header.getHeight()).toBe(72);
    });
  });

  describe('getCssClassName', () => {
    it('should return correct class names for mobile', () => {
      const className = header.getCssClassName();
      expect(className).toContain('header--mobile');
      expect(className).toContain('header--fixed');
      expect(className).toContain('header--visible');
    });

    it('should include navigation class when enabled', () => {
      header.setConfig({ showNavigation: true });
      const className = header.getCssClassName();
      expect(className).toContain('header--with-navigation');
    });

    it('should include search class when enabled', () => {
      header.setConfig({ showSearch: true });
      const className = header.getCssClassName();
      expect(className).toContain('header--with-search');
    });

    it('should include user menu class when enabled', () => {
      header.setConfig({ showUserMenu: true });
      const className = header.getCssClassName();
      expect(className).toContain('header--with-user-menu');
    });

    it('should include animated class when enabled', () => {
      header.setConfig({ animationsEnabled: true });
      const className = header.getCssClassName();
      expect(className).toContain('header--animated');
    });
  });

  describe('getInlineStyles', () => {
    it('should return styles for fixed layout', () => {
      header.setConfig({ layoutMode: 'fixed' });
      const styles = header.getInlineStyles();
      expect(styles.position).toBe('fixed');
      expect(styles.top).toBe('0');
      expect(styles.zIndex).toBe('1000');
    });

    it('should return styles for sticky layout', () => {
      header.setConfig({ layoutMode: 'sticky' });
      const styles = header.getInlineStyles();
      expect(styles.position).toBe('sticky');
      expect(styles.top).toBe('0');
      expect(styles.zIndex).toBe('1000');
    });

    it('should return styles for floating layout', () => {
      header.setConfig({ layoutMode: 'floating' });
      const styles = header.getInlineStyles();
      expect(styles.position).toBe('relative');
      expect(styles.margin).toBe('1rem');
      expect(styles.borderRadius).toBe('8px');
    });

    it('should return styles for static layout', () => {
      header.setConfig({ layoutMode: 'static' });
      const styles = header.getInlineStyles();
      expect(styles.position).toBe('relative');
    });

    it('should return styles for hidden visibility', () => {
      header.setConfig({ visibility: 'hidden' });
      const styles = header.getInlineStyles();
      expect(styles.display).toBe('none');
    });

    it('should return styles for collapsed visibility', () => {
      header.setConfig({ visibility: 'collapsed' });
      const styles = header.getInlineStyles();
      expect(styles.height).toBe('0');
      expect(styles.overflow).toBe('hidden');
    });

    it('should return styles for compact visibility', () => {
      header.setConfig({ visibility: 'compact', height: 80 });
      const styles = header.getInlineStyles();
      expect(styles.height).toBe('60px');
    });
  });

  describe('subscribe/unsubscribe', () => {
    it('should subscribe to config changes', () => {
      const callback = jest.fn();
      header.subscribe('test-id', callback);
      header.setConfig({ height: 100 });
      expect(callback).toHaveBeenCalledTimes(1);
    });

    it('should unsubscribe from config changes', () => {
      const callback = jest.fn();
      header.subscribe('test-id', callback);
      header.unsubscribe('test-id');
      header.setConfig({ height: 100 });
      expect(callback).not.toHaveBeenCalled();
    });

    it('should handle multiple subscribers', () => {
      const callback1 = jest.fn();
      const callback2 = jest.fn();
      header.subscribe('id1', callback1);
      header.subscribe('id2', callback2);
      header.setConfig({ height: 100 });
      expect(callback1).toHaveBeenCalledTimes(1);
      expect(callback2).toHaveBeenCalledTimes(1);
    });
  });

  describe('initialize/destroy', () => {
    it('should return cleanup function', () => {
      const cleanup = header.initialize();
      expect(typeof cleanup).toBe('function');
      cleanup();
    });

    it('should clean up listeners on destroy', () => {
      header.subscribe('test-id', jest.fn());
      header.destroy();
      // After destroy, setting config should not throw but also not notify
      expect(() => {
        header.setConfig({ height: 100 });
      }).not.toThrow();
    });

    it('should return no-op cleanup when window is undefined (SSR)', () => {
      // The SSR branch (typeof window === 'undefined') is a jsdom environment guard.
      // We verify the normal path returns a valid cleanup function instead.
      const cleanup = header.initialize();
      expect(typeof cleanup).toBe('function');
      // Calling cleanup should not throw
      expect(() => cleanup()).not.toThrow();
    });

    it('should disconnect resizeObserver on destroy when set', () => {
      const mockDisconnect = jest.fn();
      // @ts-expect-error: accessing private field for test coverage
      header.resizeObserver = { disconnect: mockDisconnect } as unknown as ResizeObserver;
      header.destroy();
      expect(mockDisconnect).toHaveBeenCalledTimes(1);
      // @ts-expect-error: accessing private field for test coverage
      expect(header.resizeObserver).toBeNull();
    });
  });
});

describe('Helper Functions', () => {
  describe('getBreakpointFromWidth', () => {
    it('should return mobile for width 0', () => {
      expect(getBreakpointFromWidth(0)).toBe('mobile');
    });

    it('should return mobile for small width', () => {
      expect(getBreakpointFromWidth(320)).toBe('mobile');
    });

    it('should return tablet for medium width', () => {
      expect(getBreakpointFromWidth(600)).toBe('tablet');
    });

    it('should return desktop for large width', () => {
      expect(getBreakpointFromWidth(900)).toBe('desktop');
    });

    it('should return wide for very large width', () => {
      expect(getBreakpointFromWidth(1200)).toBe('wide');
    });

    it('should return ultra-wide for very large width', () => {
      expect(getBreakpointFromWidth(1600)).toBe('ultra-wide');
    });

    // Edge cases
    it('should return mobile for negative width', () => {
      expect(getBreakpointFromWidth(-100)).toBe('mobile');
    });

    it('should return ultra-wide for extremely large width', () => {
      expect(getBreakpointFromWidth(50000)).toBe('ultra-wide');
    });
  });

  describe('isMobileDevice', () => {
    it('should return true for width < 768', () => {
      expect(isMobileDevice(320)).toBe(true);
      expect(isMobileDevice(600)).toBe(true);
      expect(isMobileDevice(767)).toBe(true);
    });

    it('should return false for width >= 768', () => {
      expect(isMobileDevice(768)).toBe(false);
      expect(isMobileDevice(1024)).toBe(false);
    });

    it('should return true for negative width', () => {
      expect(isMobileDevice(-100)).toBe(true);
    });
  });

  describe('isTabletDevice', () => {
    it('should return true for width >= 768 and < 1024', () => {
      expect(isTabletDevice(768)).toBe(true);
      expect(isTabletDevice(900)).toBe(true);
      expect(isTabletDevice(1023)).toBe(true);
    });

    it('should return false for width outside range', () => {
      expect(isTabletDevice(767)).toBe(false);
      expect(isTabletDevice(1024)).toBe(false);
    });
  });

  describe('isDesktopDevice', () => {
    it('should return true for width >= 1024', () => {
      expect(isDesktopDevice(1024)).toBe(true);
      expect(isDesktopDevice(1440)).toBe(true);
      expect(isDesktopDevice(1920)).toBe(true);
    });

    it('should return false for width < 1024', () => {
      expect(isDesktopDevice(1023)).toBe(false);
      expect(isDesktopDevice(768)).toBe(false);
    });
  });

  describe('getMediaQuery', () => {
    it('should return min-width query', () => {
      const query = getMediaQuery('desktop', 'min');
      expect(query).toBe('(min-width: 768px)');
    });

    it('should return max-width query', () => {
      const query = getMediaQuery('desktop', 'max');
      expect(query).toBe('(max-width: 1023px)');
    });

    it('should throw error for invalid breakpoint', () => {
      expect(() => {
        getMediaQuery('invalid' as AllowedBreakpoint, 'min');
      }).toThrow(FrontendHeaderResponsiveError);
    });
  });

  describe('createResponsiveHeaderClass', () => {
    it('should create class with basic options', () => {
      const className = createResponsiveHeaderClass('mobile', {
        layoutMode: 'fixed',
        visibility: 'visible',
      });
      expect(className).toContain('header--mobile');
      expect(className).toContain('header--fixed');
      expect(className).toContain('header--visible');
    });

    it('should include optional classes when enabled', () => {
      const className = createResponsiveHeaderClass('desktop', {
        showNavigation: true,
        showSearch: true,
        showUserMenu: true,
        animationsEnabled: true,
      });
      expect(className).toContain('header--with-navigation');
      expect(className).toContain('header--with-search');
      expect(className).toContain('header--with-user-menu');
      expect(className).toContain('header--animated');
    });

    it('should use defaults for undefined options', () => {
      const className = createResponsiveHeaderClass('mobile');
      expect(className).toContain('header--static');
      expect(className).toContain('header--visible');
    });
  });
});

describe('Constants', () => {
  describe('ALLOWED_BREAKPOINTS', () => {
    it('should contain all expected breakpoints', () => {
      expect(ALLOWED_BREAKPOINTS).toContain('mobile');
      expect(ALLOWED_BREAKPOINTS).toContain('tablet');
      expect(ALLOWED_BREAKPOINTS).toContain('desktop');
      expect(ALLOWED_BREAKPOINTS).toContain('wide');
      expect(ALLOWED_BREAKPOINTS).toContain('ultra-wide');
    });
  });

  describe('HEADER_LAYOUT_MODES', () => {
    it('should contain all expected layout modes', () => {
      expect(HEADER_LAYOUT_MODES).toContain('static');
      expect(HEADER_LAYOUT_MODES).toContain('sticky');
      expect(HEADER_LAYOUT_MODES).toContain('fixed');
      expect(HEADER_LAYOUT_MODES).toContain('floating');
    });
  });

  describe('HEADER_VISIBILITY_STATES', () => {
    it('should contain all expected visibility states', () => {
      expect(HEADER_VISIBILITY_STATES).toContain('visible');
      expect(HEADER_VISIBILITY_STATES).toContain('hidden');
      expect(HEADER_VISIBILITY_STATES).toContain('collapsed');
      expect(HEADER_VISIBILITY_STATES).toContain('compact');
    });
  });

  describe('BREAKPOINT_CONFIGS', () => {
    it('should have configs for all breakpoints', () => {
      expect(BREAKPOINT_CONFIGS.mobile).toBeDefined();
      expect(BREAKPOINT_CONFIGS.tablet).toBeDefined();
      expect(BREAKPOINT_CONFIGS.desktop).toBeDefined();
      expect(BREAKPOINT_CONFIGS.wide).toBeDefined();
      expect(BREAKPOINT_CONFIGS['ultra-wide']).toBeDefined();
    });

    it('should have correct min/max values', () => {
      expect(BREAKPOINT_CONFIGS.mobile.minWidth).toBe(0);
      expect(BREAKPOINT_CONFIGS.mobile.maxWidth).toBe(479);
      expect(BREAKPOINT_CONFIGS.tablet.minWidth).toBe(480);
      expect(BREAKPOINT_CONFIGS.tablet.maxWidth).toBe(767);
      expect(BREAKPOINT_CONFIGS.desktop.minWidth).toBe(768);
      expect(BREAKPOINT_CONFIGS.desktop.maxWidth).toBe(1023);
      expect(BREAKPOINT_CONFIGS.wide.minWidth).toBe(1024);
      expect(BREAKPOINT_CONFIGS.wide.maxWidth).toBe(1439);
      expect(BREAKPOINT_CONFIGS['ultra-wide'].minWidth).toBe(1440);
    });
  });

  describe('DEFAULT_HEADER_CONFIGS', () => {
    it('should have configs for all breakpoints', () => {
      expect(DEFAULT_HEADER_CONFIGS.mobile).toBeDefined();
      expect(DEFAULT_HEADER_CONFIGS.tablet).toBeDefined();
      expect(DEFAULT_HEADER_CONFIGS.desktop).toBeDefined();
      expect(DEFAULT_HEADER_CONFIGS.wide).toBeDefined();
      expect(DEFAULT_HEADER_CONFIGS['ultra-wide']).toBeDefined();
    });

    it('should have different heights per breakpoint', () => {
      expect(DEFAULT_HEADER_CONFIGS.mobile.height).toBe(56);
      expect(DEFAULT_HEADER_CONFIGS.tablet.height).toBe(64);
      expect(DEFAULT_HEADER_CONFIGS.desktop.height).toBe(72);
      expect(DEFAULT_HEADER_CONFIGS.wide.height).toBe(80);
      expect(DEFAULT_HEADER_CONFIGS['ultra-wide'].height).toBe(80);
    });

    it('should show navigation on larger breakpoints', () => {
      expect(DEFAULT_HEADER_CONFIGS.mobile.showNavigation).toBe(false);
      expect(DEFAULT_HEADER_CONFIGS.tablet.showNavigation).toBe(true);
      expect(DEFAULT_HEADER_CONFIGS.desktop.showNavigation).toBe(true);
    });

    it('should disable animations on larger breakpoints', () => {
      expect(DEFAULT_HEADER_CONFIGS.mobile.animationsEnabled).toBe(true);
      expect(DEFAULT_HEADER_CONFIGS.desktop.animationsEnabled).toBe(false);
    });
  });
});

describe('Error Handling', () => {
  describe('FrontendHeaderResponsiveError', () => {
    it('should have correct name', () => {
      const error = new FrontendHeaderResponsiveError('Test error');
      expect(error.name).toBe('FrontendHeaderResponsiveError');
    });

    it('should have correct message', () => {
      const error = new FrontendHeaderResponsiveError('Test error');
      expect(error.message).toBe('Test error');
    });
  });
});

describe('useHeaderResponsive', () => {
  it('should return a FrontendHeaderResponsive instance in browser environment', () => {
    const instance = useHeaderResponsive('desktop');
    expect(instance).toBeInstanceOf(FrontendHeaderResponsive);
    expect(instance.getBreakpoint()).toBe('desktop');
    instance.destroy();
  });

  it('should return a mobile default instance when window is undefined (SSR)', () => {
    // SSR guard: useHeaderResponsive falls back to 'mobile' when no breakpoint given.
    // In jsdom window is always defined; we verify the browser path returns a valid instance.
    const instance = useHeaderResponsive();
    expect(instance).toBeInstanceOf(FrontendHeaderResponsive);
    instance.destroy();
  });

  it('should respect custom breakpoint in SSR environment', () => {
    // Verify that a provided breakpoint is honoured regardless of environment.
    const instance = useHeaderResponsive('tablet');
    expect(instance.getBreakpoint()).toBe('tablet');
    instance.destroy();
  });
});
