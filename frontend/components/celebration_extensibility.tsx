import React, { useState, useEffect, useCallback } from 'react';

/**
 * @title Milestone Celebration Extensibility Component
 * @notice Provides extensible celebration UI for campaign milestones (roadmap, stretch goals).
 * @dev Supports customizable animations, messages, and callbacks. Secure against XSS via input validation.
 *      Designed for high performance with minimal re-renders and efficient state management.
 *
 * @security
 * - All user-provided strings are validated for length and control characters
 * - No dangerouslySetInnerHTML usage
 * - Event handlers are memoized to prevent unnecessary re-renders
 */

// ── Types ─────────────────────────────────────────────────────────────────────

/**
 * @notice Supported celebration types for different milestone achievements.
 */
export type CelebrationType = 'stretch_goal' | 'roadmap_milestone' | 'campaign_success';

/**
 * @notice Configuration for celebration appearance and behavior.
 */
export interface CelebrationConfig {
  /** Celebration message (max 200 chars, no control chars) */
  message: string;
  /** Optional subtitle (max 100 chars) */
  subtitle?: string;
  /** Animation duration in milliseconds (100-5000) */
  duration?: number;
  /** Celebration color theme */
  theme?: 'success' | 'celebration' | 'achievement';
  /** Enable confetti animation */
  enableConfetti?: boolean;
  /** Custom CSS class for styling */
  className?: string;
}

/**
 * @notice Props for CelebrationExtensibility component.
 */
export interface CelebrationExtensibilityProps {
  /** Type of celebration to display */
  type: CelebrationType;
  /** Celebration configuration */
  config: CelebrationConfig;
  /** Callback fired when celebration starts */
  onCelebrateStart?: () => void;
  /** Callback fired when celebration completes */
  onCelebrateEnd?: () => void;
  /** Whether to auto-start celebration on mount */
  autoStart?: boolean;
  /** Render prop for custom celebration content */
  children?: (state: CelebrationState) => React.ReactNode;
}

// ── Constants ─────────────────────────────────────────────────────────────────

const MAX_MESSAGE_LENGTH = 200;
const MAX_SUBTITLE_LENGTH = 100;
const MIN_DURATION = 100;
const MAX_DURATION = 5000;
const DEFAULT_DURATION = 2000;

const CONTROL_CHAR_RE = /[\u0000-\u001F\u007F]/g;

/**
 * @notice Celebration state machine.
 */
export type CelebrationState = 'idle' | 'celebrating' | 'completed';

/**
 * @notice Default configurations per celebration type.
 */
const DEFAULT_CONFIGS: Record<CelebrationType, Required<CelebrationConfig>> = {
  stretch_goal: {
    message: '🎉 Stretch Goal Unlocked!',
    subtitle: 'Campaign milestone achieved',
    duration: DEFAULT_DURATION,
    theme: 'celebration',
    enableConfetti: true,
    className: '',
  },
  roadmap_milestone: {
    message: '🚀 Roadmap Milestone Reached!',
    subtitle: 'Progress update',
    duration: DEFAULT_DURATION,
    theme: 'achievement',
    enableConfetti: false,
    className: '',
  },
  campaign_success: {
    message: '🎊 Campaign Successful!',
    subtitle: 'Goal achieved',
    duration: DEFAULT_DURATION,
    theme: 'success',
    enableConfetti: true,
    className: '',
  },
};

// ── Validation Functions ──────────────────────────────────────────────────────

/**
 * @notice Validates celebration configuration for security and correctness.
 * @param config The configuration to validate.
 * @return True if valid, throws error if invalid.
 */
function validateConfig(config: CelebrationConfig): void {
  if (!config.message || typeof config.message !== 'string') {
    throw new Error('Celebration message is required and must be a string');
  }
  if (config.message.length > MAX_MESSAGE_LENGTH) {
    throw new Error(`Message too long (max ${MAX_MESSAGE_LENGTH} chars)`);
  }
  if (CONTROL_CHAR_RE.test(config.message)) {
    throw new Error('Message contains invalid control characters');
  }

  if (config.subtitle && typeof config.subtitle !== 'string') {
    throw new Error('Subtitle must be a string');
  }
  if (config.subtitle && config.subtitle.length > MAX_SUBTITLE_LENGTH) {
    throw new Error(`Subtitle too long (max ${MAX_SUBTITLE_LENGTH} chars)`);
  }
  if (config.subtitle && CONTROL_CHAR_RE.test(config.subtitle)) {
    throw new Error('Subtitle contains invalid control characters');
  }

  if (config.duration !== undefined) {
    if (typeof config.duration !== 'number' || config.duration < MIN_DURATION || config.duration > MAX_DURATION) {
      throw new Error(`Duration must be between ${MIN_DURATION} and ${MAX_DURATION}ms`);
    }
  }

  if (config.theme && !['success', 'celebration', 'achievement'].includes(config.theme)) {
    throw new Error('Invalid theme');
  }

  if (config.enableConfetti !== undefined && typeof config.enableConfetti !== 'boolean') {
    throw new Error('enableConfetti must be boolean');
  }
}

// ── Component ─────────────────────────────────────────────────────────────────

/**
 * @notice Extensible celebration component for campaign milestones.
 * @dev Merges user config with defaults, validates inputs, manages celebration lifecycle.
 */
export const CelebrationExtensibility: React.FC<CelebrationExtensibilityProps> = ({
  type,
  config,
  onCelebrateStart,
  onCelebrateEnd,
  autoStart = false,
  children,
}) => {
  const [state, setState] = useState<CelebrationState>('idle');

  // Merge config with defaults
  const mergedConfig: Required<CelebrationConfig> = {
    ...DEFAULT_CONFIGS[type],
    ...config,
  };

  // Validate on mount and config changes
  useEffect(() => {
    validateConfig(mergedConfig);
  }, [mergedConfig]);

  // Celebration lifecycle
  const startCelebration = useCallback(() => {
    if (state !== 'idle') return;

    setState('celebrating');
    onCelebrateStart?.();

    setTimeout(() => {
      setState('completed');
      onCelebrateEnd?.();
    }, mergedConfig.duration);
  }, [state, mergedConfig.duration, onCelebrateStart, onCelebrateEnd]);

  // Auto-start if enabled
  useEffect(() => {
    if (autoStart && state === 'idle') {
      startCelebration();
    }
  }, [autoStart, state, startCelebration]);

  // Render confetti (simplified animation)
  const renderConfetti = () => {
    if (!mergedConfig.enableConfetti || state !== 'celebrating') return null;

    return (
      <div style={styles.confetti}>
        {Array.from({ length: 20 }, (_, i) => (
          <div
            key={i}
            style={{
              ...styles.confettiPiece,
              left: `${Math.random() * 100}%`,
              animationDelay: `${Math.random() * 2}s`,
            }}
          />
        ))}
      </div>
    );
  };

  // Theme-based styles
  const getThemeStyles = (): React.CSSProperties => {
    const base: React.CSSProperties = {
      ...styles.container,
      backgroundColor: mergedConfig.theme === 'success' ? '#10b981' :
                      mergedConfig.theme === 'celebration' ? '#f59e0b' : '#3b82f6',
    };
    return base;
  };

  // Custom render or default
  if (children) {
    return <>{children(state)}</>;
  }

  return (
    <div style={getThemeStyles()} className={mergedConfig.className}>
      {renderConfetti()}
      <div style={styles.content}>
        <h2 style={styles.message}>{mergedConfig.message}</h2>
        {mergedConfig.subtitle && (
          <p style={styles.subtitle}>{mergedConfig.subtitle}</p>
        )}
        {state === 'idle' && (
          <button style={styles.button} onClick={startCelebration}>
            Celebrate!
          </button>
        )}
        {state === 'celebrating' && (
          <div style={styles.celebrating}>🎉 Celebrating... 🎉</div>
        )}
        {state === 'completed' && (
          <div style={styles.completed}>✓ Celebration Complete</div>
        )}
      </div>
    </div>
  );
};

// ── Styles ────────────────────────────────────────────────────────────────────

const styles: Record<string, React.CSSProperties> = {
  container: {
    position: 'relative',
    padding: '2rem',
    borderRadius: '12px',
    color: 'white',
    textAlign: 'center',
    overflow: 'hidden',
    minHeight: '200px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    animation: 'celebrationFadeIn 0.5s ease-out',
  },
  content: {
    zIndex: 1,
    position: 'relative',
  },
  message: {
    fontSize: '1.5rem',
    fontWeight: 'bold',
    margin: '0 0 0.5rem 0',
  },
  subtitle: {
    fontSize: '1rem',
    margin: '0 0 1rem 0',
    opacity: 0.9,
  },
  button: {
    backgroundColor: 'rgba(255, 255, 255, 0.2)',
    border: '1px solid rgba(255, 255, 255, 0.3)',
    color: 'white',
    padding: '0.5rem 1rem',
    borderRadius: '6px',
    cursor: 'pointer',
    fontWeight: '600',
    transition: 'background-color 0.2s ease',
  },
  celebrating: {
    fontSize: '1.25rem',
    animation: 'celebrationPulse 1s infinite',
  },
  completed: {
    fontSize: '1rem',
    opacity: 0.8,
  },
  confetti: {
    position: 'absolute',
    top: 0,
    left: 0,
    width: '100%',
    height: '100%',
    pointerEvents: 'none',
  },
  confettiPiece: {
    position: 'absolute',
    width: '8px',
    height: '8px',
    backgroundColor: 'white',
    borderRadius: '50%',
    animation: 'celebrationFall 3s linear infinite',
  },
};

// CSS animations as inline styles (for better SSR compatibility)
const animationStyles = `
  @keyframes celebrationFadeIn {
    from { opacity: 0; transform: scale(0.9); }
    to { opacity: 1; transform: scale(1); }
  }
  @keyframes celebrationPulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.7; transform: scale(1.05); }
  }
  @keyframes celebrationFall {
    0% { transform: translateY(-100px) rotate(0deg); opacity: 1; }
    100% { transform: translateY(100vh) rotate(360deg); opacity: 0; }
  }
`;

// Inject styles once (avoid duplicate style tags)
if (typeof document !== 'undefined' && !document.getElementById('celebration-styles')) {
  const styleSheet = document.createElement('style');
  styleSheet.id = 'celebration-styles';
  styleSheet.textContent = animationStyles;
  document.head.appendChild(styleSheet);
}

export default CelebrationExtensibility;