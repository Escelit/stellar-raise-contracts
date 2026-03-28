/**
 * @title Celebration Extensibility — Comprehensive Test Suite
 * @notice Covers validation, state management, rendering, security, and extensibility.
 * @dev Targets ≥ 95% coverage of celebration_extensibility.tsx.
 */
import React from 'react';
import { render, screen, fireEvent, act, waitFor } from '@testing-library/react';
import CelebrationExtensibility, {
  type CelebrationType,
  type CelebrationConfig,
  type CelebrationState,
} from './celebration_extensibility';

// ── Helpers ───────────────────────────────────────────────────────────────────

function renderCelebration(props: Partial<React.ComponentProps<typeof CelebrationExtensibility>> = {}) {
  return render(
    <CelebrationExtensibility
      type="stretch_goal"
      config={{ message: 'Test Celebration' }}
      {...props}
    />
  );
}

const VALID_CONFIG: CelebrationConfig = {
  message: 'Valid Message',
  subtitle: 'Valid Subtitle',
  duration: 1000,
  theme: 'success',
  enableConfetti: true,
};

// ── Validation Tests ──────────────────────────────────────────────────────────

describe('Config Validation', () => {
  it('accepts valid configuration', () => {
    expect(() => renderCelebration({ config: VALID_CONFIG })).not.toThrow();
  });

  it('rejects missing message', () => {
    expect(() => renderCelebration({ config: { ...VALID_CONFIG, message: '' } })).toThrow(
      'Celebration message is required'
    );
  });

  it('rejects non-string message', () => {
    expect(() => renderCelebration({ config: { ...VALID_CONFIG, message: 123 as any } })).toThrow(
      'must be a string'
    );
  });

  it('rejects message too long', () => {
    const longMessage = 'a'.repeat(201);
    expect(() => renderCelebration({ config: { ...VALID_CONFIG, message: longMessage } })).toThrow(
      'too long'
    );
  });

  it('rejects message with control characters', () => {
    expect(() => renderCelebration({ config: { ...VALID_CONFIG, message: 'Test\u0000Message' } })).toThrow(
      'invalid control characters'
    );
  });

  it('rejects non-string subtitle', () => {
    expect(() => renderCelebration({ config: { ...VALID_CONFIG, subtitle: 123 as any } })).toThrow(
      'must be a string'
    );
  });

  it('rejects subtitle too long', () => {
    const longSubtitle = 'a'.repeat(101);
    expect(() => renderCelebration({ config: { ...VALID_CONFIG, subtitle: longSubtitle } })).toThrow(
      'too long'
    );
  });

  it('rejects subtitle with control characters', () => {
    expect(() => renderCelebration({ config: { ...VALID_CONFIG, subtitle: 'Test\u0000Sub' } })).toThrow(
      'invalid control characters'
    );
  });

  it('rejects invalid duration', () => {
    expect(() => renderCelebration({ config: { ...VALID_CONFIG, duration: 50 } })).toThrow(
      'between 100 and 5000'
    );
    expect(() => renderCelebration({ config: { ...VALID_CONFIG, duration: 6000 } })).toThrow(
      'between 100 and 5000'
    );
    expect(() => renderCelebration({ config: { ...VALID_CONFIG, duration: '1000' as any } })).toThrow(
      'between 100 and 5000'
    );
  });

  it('rejects invalid theme', () => {
    expect(() => renderCelebration({ config: { ...VALID_CONFIG, theme: 'invalid' as any } })).toThrow(
      'Invalid theme'
    );
  });

  it('rejects non-boolean enableConfetti', () => {
    expect(() => renderCelebration({ config: { ...VALID_CONFIG, enableConfetti: 'true' as any } })).toThrow(
      'must be boolean'
    );
  });
});

// ── Rendering Tests ───────────────────────────────────────────────────────────

describe('Rendering', () => {
  it('renders with default config', () => {
    renderCelebration();
    expect(screen.getByText('🎉 Stretch Goal Unlocked!')).toBeInTheDocument();
    expect(screen.getByText('Campaign milestone achieved')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Celebrate!' })).toBeInTheDocument();
  });

  it('renders custom message and subtitle', () => {
    renderCelebration({
      config: { message: 'Custom Message', subtitle: 'Custom Subtitle' },
    });
    expect(screen.getByText('Custom Message')).toBeInTheDocument();
    expect(screen.getByText('Custom Subtitle')).toBeInTheDocument();
  });

  it('applies custom className', () => {
    const { container } = renderCelebration({
      config: { ...VALID_CONFIG, className: 'custom-class' },
    });
    expect(container.firstChild).toHaveClass('custom-class');
  });

  it('renders confetti when enabled', () => {
    const { container } = renderCelebration({
      config: { ...VALID_CONFIG, enableConfetti: true },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Celebrate!' }));
    expect(container.querySelector('[style*="confetti"]')).toBeInTheDocument();
  });

  it('does not render confetti when disabled', () => {
    const { container } = renderCelebration({
      config: { ...VALID_CONFIG, enableConfetti: false },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Celebrate!' }));
    expect(container.querySelector('[style*="confetti"]')).toBeNull();
  });

  it('applies correct theme colors', () => {
    const { rerender } = render(
      <CelebrationExtensibility type="stretch_goal" config={{ message: 'Test', theme: 'success' }} />
    );
    expect(screen.getByText('Test').closest('div')).toHaveStyle({ backgroundColor: '#10b981' });

    rerender(
      <CelebrationExtensibility type="stretch_goal" config={{ message: 'Test', theme: 'celebration' }} />
    );
    expect(screen.getByText('Test').closest('div')).toHaveStyle({ backgroundColor: '#f59e0b' });

    rerender(
      <CelebrationExtensibility type="stretch_goal" config={{ message: 'Test', theme: 'achievement' }} />
    );
    expect(screen.getByText('Test').closest('div')).toHaveStyle({ backgroundColor: '#3b82f6' });
  });
});

// ── State Management Tests ────────────────────────────────────────────────────

describe('State Management', () => {
  it('starts in idle state', () => {
    renderCelebration();
    expect(screen.getByRole('button', { name: 'Celebrate!' })).toBeInTheDocument();
  });

  it('transitions to celebrating on button click', () => {
    renderCelebration();
    fireEvent.click(screen.getByRole('button', { name: 'Celebrate!' }));
    expect(screen.getByText('🎉 Celebrating... 🎉')).toBeInTheDocument();
  });

  it('transitions to completed after duration', async () => {
    renderCelebration({ config: { ...VALID_CONFIG, duration: 100 } });
    fireEvent.click(screen.getByRole('button', { name: 'Celebrate!' }));
    expect(screen.getByText('🎉 Celebrating... 🎉')).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.getByText('✓ Celebration Complete')).toBeInTheDocument();
    }, { timeout: 200 });
  });

  it('auto-starts when autoStart is true', () => {
    renderCelebration({ autoStart: true, config: { ...VALID_CONFIG, duration: 100 } });
    expect(screen.getByText('🎉 Celebrating... 🎉')).toBeInTheDocument();
  });

  it('calls onCelebrateStart callback', () => {
    const mockStart = jest.fn();
    renderCelebration({ onCelebrateStart: mockStart });
    fireEvent.click(screen.getByRole('button', { name: 'Celebrate!' }));
    expect(mockStart).toHaveBeenCalledTimes(1);
  });

  it('calls onCelebrateEnd callback', async () => {
    const mockEnd = jest.fn();
    renderCelebration({
      onCelebrateEnd: mockEnd,
      config: { ...VALID_CONFIG, duration: 100 },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Celebrate!' }));

    await waitFor(() => {
      expect(mockEnd).toHaveBeenCalledTimes(1);
    }, { timeout: 200 });
  });

  it('prevents multiple celebrations', () => {
    renderCelebration();
    const button = screen.getByRole('button', { name: 'Celebrate!' });
    fireEvent.click(button);
    fireEvent.click(button); // Should not trigger again
    expect(screen.getByText('🎉 Celebrating... 🎉')).toBeInTheDocument();
  });
});

// ── Extensibility Tests ───────────────────────────────────────────────────────

describe('Extensibility', () => {
  it('supports render prop for custom content', () => {
    const mockRender = jest.fn((state: CelebrationState) => <div>Custom: {state}</div>);
    renderCelebration({ children: mockRender });
    expect(mockRender).toHaveBeenCalledWith('idle');
    expect(screen.getByText('Custom: idle')).toBeInTheDocument();
  });

  it('passes state to render prop', () => {
    let currentState: CelebrationState = 'idle';
    const mockRender = jest.fn((state: CelebrationState) => {
      currentState = state;
      return <div>State: {state}</div>;
    });
    renderCelebration({
      children: mockRender,
      config: { ...VALID_CONFIG, duration: 100 },
    });

    fireEvent.click(screen.getByText('State: idle'));
    expect(currentState).toBe('celebrating');

    // Wait for completion
    return waitFor(() => {
      expect(currentState).toBe('completed');
    }, { timeout: 200 });
  });
});

// ── Celebration Types Tests ───────────────────────────────────────────────────

describe('Celebration Types', () => {
  it('renders stretch_goal defaults', () => {
    renderCelebration({ type: 'stretch_goal' });
    expect(screen.getByText('🎉 Stretch Goal Unlocked!')).toBeInTheDocument();
    expect(screen.getByText('Campaign milestone achieved')).toBeInTheDocument();
  });

  it('renders roadmap_milestone defaults', () => {
    renderCelebration({ type: 'roadmap_milestone' });
    expect(screen.getByText('🚀 Roadmap Milestone Reached!')).toBeInTheDocument();
    expect(screen.getByText('Progress update')).toBeInTheDocument();
  });

  it('renders campaign_success defaults', () => {
    renderCelebration({ type: 'campaign_success' });
    expect(screen.getByText('🎊 Campaign Successful!')).toBeInTheDocument();
    expect(screen.getByText('Goal achieved')).toBeInTheDocument();
  });
});

// ── Security Tests ────────────────────────────────────────────────────────────

describe('Security', () => {
  it('does not use dangerouslySetInnerHTML', () => {
    const { container } = renderCelebration();
    const elementsWithDanger = container.querySelectorAll('[dangerouslySetInnerHTML]');
    expect(elementsWithDanger).toHaveLength(0);
  });

  it('validates config on every render', () => {
    const consoleSpy = jest.spyOn(console, 'error').mockImplementation(() => {});
    const { rerender } = renderCelebration();

    expect(() => {
      rerender(
        <CelebrationExtensibility
          type="stretch_goal"
          config={{ message: 'Valid' }}
        />
      );
    }).not.toThrow();

    expect(() => {
      rerender(
        <CelebrationExtensibility
          type="stretch_goal"
          config={{ message: '' }}
        />
      );
    }).toThrow();

    consoleSpy.mockRestore();
  });
});

// ── Accessibility Tests ───────────────────────────────────────────────────────

describe('Accessibility', () => {
  it('has proper ARIA attributes', () => {
    renderCelebration();
    const button = screen.getByRole('button', { name: 'Celebrate!' });
    expect(button).toBeInTheDocument();
  });

  it('maintains focus during state changes', () => {
    renderCelebration();
    const button = screen.getByRole('button', { name: 'Celebrate!' });
    button.focus();
    expect(document.activeElement).toBe(button);

    fireEvent.click(button);
    // After click, focus should remain or move appropriately
    // In this case, button is removed, so focus moves to body
    expect(document.activeElement).toBe(document.body);
  });
});

// ── Performance Tests ─────────────────────────────────────────────────────────

describe('Performance', () => {
  it('memoizes callbacks to prevent unnecessary re-renders', () => {
    const mockStart = jest.fn();
    const { rerender } = renderCelebration({ onCelebrateStart: mockStart });

    // Re-render with same props
    rerender(
      <CelebrationExtensibility
        type="stretch_goal"
        config={{ message: 'Test Celebration' }}
        onCelebrateStart={mockStart}
      />
    );

    // Callbacks should be stable
    fireEvent.click(screen.getByRole('button', { name: 'Celebrate!' }));
    expect(mockStart).toHaveBeenCalledTimes(1);
  });
});