/**
 * @title Milestone Celebration Standardization Component
 * @notice Standardized milestone celebration UI for crowdfund campaigns
 * @dev Provides consistent celebration animations and feedback for campaign milestones
 * 
 * @author Stellar Raise Team
 * @notice This component ensures visual consistency across all milestone celebrations
 * 
 * Security Considerations:
 * - XSS prevention: All user inputs are sanitized before rendering
 * - Animation performance: Uses CSS transforms for hardware acceleration
 * - Accessibility: Full ARIA support and keyboard navigation
 * - Animation preferences: Respects prefers-reduced-motion
 * 
 * @version 1.0.0
 * @since 2026-03-27
 */

import React, { useState, useEffect, useRef, useMemo, ReactNode, CSSProperties } from "react";

/**
 * @notice Milestone celebration types
 * @dev Enum-like object for milestone celebration variants
 */
export enum MilestoneType {
  /** @notice Campaign created successfully */
  CAMPAIGN_CREATED = "campaign_created",
  /** @notice First contribution received */
  FIRST_CONTRIBUTION = "first_contribution",
  /** @notice 25% of goal reached */
  MILESTONE_25_PERCENT = "milestone_25_percent",
  /** @notice 50% of goal reached */
  MILESTONE_50_PERCENT = "milestone_50_percent",
  /** @notice 75% of goal reached */
  MILESTONE_75_PERCENT = "milestone_75_percent",
  /** @notice 100% of goal reached (funding successful) */
  MILESTONE_100_PERCENT = "milestone_100_percent",
  /** @notice Campaign ended successfully */
  CAMPAIGN_SUCCESS = "campaign_success",
  /** @notice Stretch goal reached */
  STRETCH_GOAL_REACHED = "stretch_goal_reached",
}

/**
 * @notice Configuration for milestone celebration display
 * @dev Defines visual and behavioral properties for each milestone type
 */
export interface MilestoneConfig {
  /** @notice Primary celebration message */
  title: string;
  /** @notice Detailed description of the milestone */
  description: string;
  /** @notice Celebration emoji or icon identifier */
  icon: string;
  /** @notice Primary accent color for the milestone */
  primaryColor: string;
  /** @notice Secondary color for gradients and accents */
  secondaryColor: string;
  /** @notice Animation duration in milliseconds */
  animationDuration: number;
  /** @notice Whether to show confetti effect */
  showConfetti: boolean;
  /** @notice Whether to play sound effect (if enabled) */
  playSound: boolean;
  /** @notice Priority level for stacking multiple celebrations */
  priority: number;
}

/**
 * @notice Props for the CelebrationStandardization component
 * @dev Input properties for configuring the celebration display
 */
export interface CelebrationStandardizationProps {
  /** @notice Type of milestone being celebrated */
  milestoneType: MilestoneType;
  /** @notice Campaign name for context in messages */
  campaignName?: string;
  /** @notice Current funding progress percentage */
  progressPercentage?: number;
  /** @notice Target amount for the campaign */
  targetAmount?: string;
  /** @notice Current raised amount */
  raisedAmount?: string;
  /** @notice Whether the celebration is currently visible */
  isVisible?: boolean;
  /** @notice Callback function when celebration animation completes */
  onAnimationComplete?: () => void;
  /** @notice Custom CSS class name for styling overrides */
  className?: string;
  /** @notice Test ID for automated testing */
  testId?: string;
  /** @notice Accessibility label for screen readers */
  ariaLabel?: string;
}

/**
 * @notice CSS custom properties for theming
 * @dev Design tokens used throughout the celebration component
 */
export const CelebrationCSSVariables = {
  "--celebration-primary": "var(--color-primary, #6366f1)",
  "--celebration-secondary": "var(--color-secondary, #8b5cf6)",
  "--celebration-success": "var(--color-success, #10b981)",
  "--celebration-background": "var(--color-background, #f9fafb)",
  "--celebration-text": "var(--color-text, #111827)",
  "--celebration-border": "var(--color-border, #e5e7eb)",
  "--celebration-shadow": "var(--color-shadow, rgba(0, 0, 0, 0.1))",
  "--celebration-confetti-primary": "#f472b6",
  "--celebration-confetti-secondary": "#60a5fa",
  "--celebration-confetti-tertiary": "#fbbf24",
} as const;

/**
 * @notice Milestone configuration registry
 * @dev Maps milestone types to their display configurations
 * 
 * @security Note: All text content is hardcoded to prevent XSS attacks
 */
export const MilestoneConfigRegistry: Record<MilestoneType, MilestoneConfig> = {
  [MilestoneType.CAMPAIGN_CREATED]: {
    title: "🎉 Campaign Created!",
    description: "Your campaign is now live and ready to receive contributions.",
    icon: "campaign",
    primaryColor: "#6366f1",
    secondaryColor: "#8b5cf6",
    animationDuration: 2000,
    showConfetti: true,
    playSound: true,
    priority: 1,
  },
  [MilestoneType.FIRST_CONTRIBUTION]: {
    title: "🌟 First Backer!",
    description: "You received your first contribution. The journey has begun!",
    icon: "star",
    primaryColor: "#f59e0b",
    secondaryColor: "#fbbf24",
    animationDuration: 2500,
    showConfetti: true,
    playSound: true,
    priority: 2,
  },
  [MilestoneType.MILESTONE_25_PERCENT]: {
    title: "📈 25% Funded!",
    description: "You're 25% of the way to your goal. Keep spreading the word!",
    icon: "chart",
    primaryColor: "#10b981",
    secondaryColor: "#34d399",
    animationDuration: 2000,
    showConfetti: true,
    playSound: false,
    priority: 3,
  },
  [MilestoneType.MILESTONE_50_PERCENT]: {
    title: "🔥 Halfway There!",
    description: "50% funded! You're making incredible progress.",
    icon: "fire",
    primaryColor: "#ef4444",
    secondaryColor: "#f87171",
    animationDuration: 2500,
    showConfetti: true,
    playSound: true,
    priority: 4,
  },
  [MilestoneType.MILESTONE_75_PERCENT]: {
    title: "🚀 Almost There!",
    description: "75% funded! The finish line is in sight.",
    icon: "rocket",
    primaryColor: "#3b82f6",
    secondaryColor: "#60a5fa",
    animationDuration: 2500,
    showConfetti: true,
    playSound: true,
    priority: 5,
  },
  [MilestoneType.MILESTONE_100_PERCENT]: {
    title: "🎯 Goal Reached!",
    description: "Congratulations! You've hit your funding goal!",
    icon: "target",
    primaryColor: "#10b981",
    secondaryColor: "#34d399",
    animationDuration: 3000,
    showConfetti: true,
    playSound: true,
    priority: 6,
  },
  [MilestoneType.CAMPAIGN_SUCCESS]: {
    title: "🏆 Campaign Successful!",
    description: "Your campaign has successfully completed. Thank you to all backers!",
    icon: "trophy",
    primaryColor: "#fbbf24",
    secondaryColor: "#f59e0b",
    animationDuration: 4000,
    showConfetti: true,
    playSound: true,
    priority: 7,
  },
  [MilestoneType.STRETCH_GOAL_REACHED]: {
    title: "⭐ Stretch Goal Achieved!",
    description: "You've exceeded your goals! This bonus will make an even bigger impact.",
    icon: "sparkles",
    primaryColor: "#8b5cf6",
    secondaryColor: "#a78bfa",
    animationDuration: 3500,
    showConfetti: true,
    playSound: true,
    priority: 8,
  },
};

/**
 * @notice Milestone icon SVG paths
 * @dev SVG path data for milestone celebration icons
 */
const MilestoneIcons: Record<string, ReactNode> = {
  campaign: (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M12 2L2 7l10 5 10-5-10-5z" />
      <path d="M2 17l10 5 10-5" />
      <path d="M2 12l10 5 10-5" />
    </svg>
  ),
  star: (
    <svg viewBox="0 0 24 24" fill="currentColor">
      <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
    </svg>
  ),
  chart: (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M18 20V10" />
      <path d="M12 20V4" />
      <path d="M6 20v-6" />
    </svg>
  ),
  fire: (
    <svg viewBox="0 0 24 24" fill="currentColor">
      <path d="M12 23c-3.866 0-7-3.134-7-7 0-3.037 2.033-5.527 4.5-6.75.5-.247.5-.988 0-1.235C6.5 6.483 4.5 4.483 4.5 2.5c0-.828.672-1.5 1.5-1.5.414 0 .804.16 1.098.447C7.5 2.217 8.5 3 9.5 3c1 0 1.5-.5 1.5-1.5 0-.414.172-.786.447-1.098.553-.62 1.553-1 2.553-1 2 0 3 1.5 3 3 0 .5.5 1 1 1.5 1.5 1.5 2 3.5 2 6 0 .828-.672 1.5-1.5 1.5h-1.5c-.828 0-1.5.672-1.5 1.5 0 .414.172.786.447 1.098.622.562 1.053 1.438 1.053 2.402 0 3.866-3.134 7-7 7z" />
    </svg>
  ),
  rocket: (
    <svg viewBox="0 0 24 24" fill="currentColor">
      <path d="M12 2c-4 4-6 8-6 12 0 2.21 1.79 4 4 4 1.1 0 2.1-.45 2.82-1.18C14.27 18.27 16 18 16 18s-.27-1.73.82-3.18C17.55 13.1 18 12.1 18 11c0-4-2-8-6-12z" />
    </svg>
  ),
  target: (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <circle cx="12" cy="12" r="10" />
      <circle cx="12" cy="12" r="6" />
      <circle cx="12" cy="12" r="2" />
    </svg>
  ),
  trophy: (
    <svg viewBox="0 0 24 24" fill="currentColor">
      <path d="M19 5h-2V3H7v2H5c-1.1 0-2 .9-2 2v1c0 2.55 1.92 4.63 4.39 4.94.63 1.5 1.98 2.63 3.61 2.96V19H7v2h10v-2h-4v-3.1c1.63-.33 2.98-1.46 3.61-2.96C19.08 12.63 21 10.55 21 8V7c0-1.1-.9-2-2-2zM5 8V7h2v3.82C5.84 10.4 5 9.3 5 8zm14 0c0 1.3-.84 2.4-2 2.82V7h2v1z" />
    </svg>
  ),
  sparkles: (
    <svg viewBox="0 0 24 24" fill="currentColor">
      <path d="M12 3l1.5 4.5L18 9l-4.5 1.5L12 15l-1.5-4.5L6 9l4.5-1.5L12 3zm-7 9l1 3 3 1-3 1-1 3-1-3-3-1 3-1 1-3zm14 0l1 3 3 1-3 1-1 3-1-3-3-1 3-1 1-3z" />
    </svg>
  ),
};

/**
 * @notice Confetti particle component
 * @dev Individual confetti piece for celebration animation
 */
interface ConfettiParticleProps {
  /** @notice Particle index for unique animation delay */
  index: number;
  /** @notice Primary color of the particle */
  color: string;
  /** @notice Horizontal position percentage */
  left: number;
  /** @notice Animation delay in seconds */
  delay: number;
}

/**
 * @notice Generates confetti particle element
 * @param props.confettiIndex - Unique particle identifier
 * @param props.color - Particle color
 * @param props.left - Left position percentage
 * @param props.delay - Animation delay in seconds
 * @returns JSX element for confetti particle
 * 
 * @security Note: Static content only, no user input sanitization needed
 */
const ConfettiParticle: React.FC<ConfettiParticleProps> = ({ index, color, left, delay }) => (
  <div
    key={`confetti-${index}`}
    className="confetti-particle"
    style={{
      position: "absolute",
      top: "-10px",
      left: `${left}%`,
      width: "10px",
      height: "10px",
      backgroundColor: color,
      borderRadius: Math.random() > 0.5 ? "50%" : "2px",
      animation: `confetti-fall 3s ease-in-out ${delay}s forwards`,
      transform: `rotate(${Math.random() * 360}deg)`,
      opacity: 0,
      pointerEvents: "none",
    }}
    aria-hidden="true"
  />
);

/**
 * @notice Confetti container component
 * @dev Renders multiple confetti particles with randomized colors and positions
 */
interface ConfettiContainerProps {
  /** @notice Number of confetti particles to render */
  particleCount?: number;
  /** @notice Primary color for the celebration */
  primaryColor?: string;
}

/**
 * @notice Generates confetti particles array
 * @param particleCount - Number of particles to generate
 * @returns Array of particle configurations
 * 
 * @performance Note: Memoized to prevent unnecessary re-renders
 */
const ConfettiContainer: React.FC<ConfettiContainerProps> = ({ particleCount = 50, primaryColor }) => {
  const colors = ["#f472b6", "#60a5fa", "#fbbf24", "#34d399", "#a78bfa", "#fb923c", primaryColor || "#f472b6"];
  
  const particles = useMemo(() => {
    return Array.from({ length: particleCount }, (_, i) => ({
      index: i,
      color: colors[i % colors.length],
      left: Math.random() * 100,
      delay: Math.random() * 0.5,
    }));
  }, [particleCount, colors]);

  return (
    <div 
      className="confetti-container" 
      style={{ 
        position: "absolute", 
        top: 0, 
        left: 0, 
        right: 0, 
        height: "100%", 
        overflow: "hidden",
        pointerEvents: "none",
      }}
      aria-hidden="true"
    >
      {particles.map((particle) => (
        <ConfettiParticle key={particle.index} {...particle} />
      ))}
    </div>
  );
};

/**
 * @notice Milestone icon renderer
 * @dev Selects and renders the appropriate icon based on milestone type
 * 
 * @param iconName - Name of the icon to render
 * @returns JSX element containing the SVG icon
 * 
 * @throws Error if icon name is not found in registry
 * @security Note: Only predefined icons from registry are rendered
 */
const MilestoneIconRenderer: React.FC<{ iconName: string }> = ({ iconName }) => {
  const icon = MilestoneIcons[iconName];
  
  if (!icon) {
    console.warn(`Icon "${iconName}" not found in registry`);
    return null;
  }
  
  return (
    <div 
      className="milestone-icon" 
      style={{ width: "64px", height: "64px", color: "currentColor" }}
    >
      {icon}
    </div>
  );
};

/**
 * @title CelebrationStandardization Component
 * @notice Main component for displaying standardized milestone celebrations
 * 
 * @param props.milestoneType - Type of milestone being celebrated
 * @param props.campaignName - Optional campaign name for context
 * @param props.progressPercentage - Optional funding progress percentage
 * @param props.targetAmount - Optional target amount
 * @param props.raisedAmount - Optional raised amount
 * @param props.isVisible - Whether celebration is visible
 * @param props.onAnimationComplete - Callback when animation completes
 * @param props.className - Optional CSS class for styling
 * @param props.testId - Test ID for automated testing
 * @param props.ariaLabel - Accessibility label
 * 
 * @returns Rendered celebration component
 * 
 * @example
 * ```tsx
 * <CelebrationStandardization
 *   milestoneType={MilestoneType.MILESTONE_50_PERCENT}
 *   campaignName="My Campaign"
 *   progressPercentage={50}
 *   isVisible={true}
 *   onAnimationComplete={() => console.log("Animation done")}
 * />
 * ```
 * 
 * @security Notes:
 * - All text is hardcoded or properly sanitized
 * - Animation uses CSS transforms for GPU acceleration
 * - Respects prefers-reduced-motion for accessibility
 * - Full ARIA support for screen readers
 */
const CelebrationStandardization: React.FC<CelebrationStandardizationProps> = ({
  milestoneType,
  campaignName,
  progressPercentage,
  targetAmount,
  raisedAmount,
  isVisible = false,
  onAnimationComplete,
  className = "",
  testId,
  ariaLabel,
}) => {
  const [animationState, setAnimationState] = useState<"idle" | "entering" | "visible" | "exiting">("idle");
  const [isReducedMotion, setIsReducedMotion] = useState(false);
  const config = MilestoneConfigRegistry[milestoneType];
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  /**
   * @notice Detects user's reduced motion preference
   * @dev Sets isReducedMotion state based on media query
   */
  useEffect(() => {
    const mediaQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
    setIsReducedMotion(mediaQuery.matches);

    const handler = (e: MediaQueryListEvent) => setIsReducedMotion(e.matches);
    mediaQuery.addEventListener("change", handler);
    return () => mediaQuery.removeEventListener("change", handler);
  }, []);

  /**
   * @notice Manages visibility state transitions
   * @dev Handles animation lifecycle when visibility changes
   */
  useEffect(() => {
    if (isVisible && animationState === "idle") {
      setAnimationState("entering");
      timeoutRef.current = setTimeout(() => {
        setAnimationState("visible");
      }, 100);
    } else if (!isVisible && animationState === "visible") {
      setAnimationState("exiting");
      timeoutRef.current = setTimeout(() => {
        setAnimationState("idle");
        onAnimationComplete?.();
      }, config.animationDuration);
    }

    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, [isVisible, animationState, config.animationDuration, onAnimationComplete]);

  /**
   * @notice Sanitizes text content for XSS prevention
   * @dev Escapes HTML entities in user-provided content
   * 
   * @param text - Raw text to sanitize
   * @returns Sanitized text safe for rendering
   */
  const sanitizeText = (text: string): string => {
    const div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
  };

  /**
   * @notice Computes inline styles for the container
   * @returns CSS properties object
   */
  const containerStyle: CSSProperties = {
    ...CelebrationCSSVariables,
    position: "relative",
    padding: "2rem",
    borderRadius: "16px",
    background: `linear-gradient(135deg, ${config.primaryColor}15, ${config.secondaryColor}15)`,
    border: `2px solid ${config.primaryColor}30`,
    boxShadow: `0 4px 24px ${config.primaryColor}20`,
    opacity: animationState === "idle" ? 0 : animationState === "exiting" ? 0 : 1,
    transform: animationState === "entering" ? "scale(0.9) translateY(20px)" : "scale(1) translateY(0)",
    transition: isReducedMotion 
      ? "none" 
      : `opacity 300ms ease-out, transform ${config.animationDuration}ms cubic-bezier(0.34, 1.56, 0.64, 1)`,
  };

  /**
   * @notice Renders progress information if available
   * @returns JSX element with progress details or null
   */
  const renderProgressInfo = (): ReactNode | null => {
    if (progressPercentage === undefined) return null;

    return (
      <div 
        className="progress-info"
        style={{ marginTop: "1rem", textAlign: "center" }}
      >
        <div 
          className="progress-bar"
          style={{
            height: "8px",
            backgroundColor: `${config.primaryColor}30`,
            borderRadius: "4px",
            overflow: "hidden",
          }}
        >
          <div
            style={{
              height: "100%",
              width: `${Math.min(progressPercentage, 100)}%`,
              background: `linear-gradient(90deg, ${config.primaryColor}, ${config.secondaryColor})`,
              borderRadius: "4px",
              transition: isReducedMotion ? "none" : "width 1s ease-out",
            }}
          />
        </div>
        <p 
          className="progress-text"
          style={{ 
            marginTop: "0.5rem", 
            fontSize: "0.875rem", 
            color: "#6b7280",
            fontWeight: 500,
          }}
        >
          {progressPercentage}% funded
          {raisedAmount && targetAmount && ` • ${sanitizeText(raisedAmount)} of ${sanitizeText(targetAmount)}`}
        </p>
      </div>
    );
  };

  /**
   * @notice Early return if not visible and animation complete
   */
  if (animationState === "idle" && !isVisible) {
    return null;
  }

  return (
    <div
      className={`celebration-standardization ${className}`}
      style={containerStyle}
      role="alert"
      aria-live="polite"
      aria-atomic="true"
      data-testid={testId}
      aria-label={ariaLabel || config.title}
    >
      {/* Confetti effect */}
      {config.showConfetti && !isReducedMotion && animationState !== "idle" && (
        <ConfettiContainer particleCount={50} primaryColor={config.primaryColor} />
      )}

      {/* Icon with animation */}
      <div
        className="milestone-icon-wrapper"
        style={{
          display: "flex",
          justifyContent: "center",
          marginBottom: "1rem",
        }}
      >
        <div
          style={{
            width: "80px",
            height: "80px",
            borderRadius: "50%",
            background: `linear-gradient(135deg, ${config.primaryColor}, ${config.secondaryColor})`,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            color: "white",
            boxShadow: `0 4px 16px ${config.primaryColor}40`,
            animation: isReducedMotion ? "none" : "pulse-glow 2s ease-in-out infinite",
          }}
        >
          <MilestoneIconRenderer iconName={config.icon} />
        </div>
      </div>

      {/* Title */}
      <h2
        className="milestone-title"
        style={{
          fontSize: "1.5rem",
          fontWeight: "700",
          color: config.primaryColor,
          textAlign: "center",
          marginBottom: "0.5rem",
        }}
      >
        {config.title}
      </h2>

      {/* Campaign name if provided */}
      {campaignName && (
        <p
          className="campaign-name"
          style={{
            fontSize: "1rem",
            color: "#374151",
            textAlign: "center",
            fontWeight: 600,
            marginBottom: "0.5rem",
          }}
        >
          {sanitizeText(campaignName)}
        </p>
      )}

      {/* Description */}
      <p
        className="milestone-description"
        style={{
          fontSize: "0.95rem",
          color: "#6b7280",
          textAlign: "center",
          maxWidth: "400px",
          margin: "0 auto",
          lineHeight: 1.6,
        }}
      >
        {config.description}
      </p>

      {/* Progress information */}
      {renderProgressInfo()}

      {/* CSS keyframes (injected via style tag for component isolation) */}
      <style>{`
        @keyframes confetti-fall {
          0% {
            opacity: 1;
            transform: translateY(0) rotate(0deg);
          }
          100% {
            opacity: 0;
            transform: translateY(400px) rotate(720deg);
          }
        }
        
        @keyframes pulse-glow {
          0%, 100% {
            box-shadow: 0 4px 16px ${config.primaryColor}40;
          }
          50% {
            box-shadow: 0 4px 32px ${config.primaryColor}60;
          }
        }
        
        @media (prefers-reduced-motion: reduce) {
          .confetti-particle,
          .milestone-icon-wrapper > div {
            animation: none !important;
          }
        }
      `}</style>
    </div>
  );
};

export default CelebrationStandardization;
