/**
 * @notice Comprehensive tests for optimized ReactSubmitButton.
 * @dev Covers pending/script states, txHash/scriptOutput, double-submit, a11y. Target: ≥98% cov.
 * Run: cd frontend && npm test react_submit_button.test.tsx
 */

import React from 'react';
import { render, screen, fireEvent, act, waitFor } from '@testing-library/react';
import ReactSubmitButton, {
  isValidTransition, resolveSafeState, resolveLabel, isInteractionBlocked,
  isBusy, normalizeText, submitButtonReducer, DEFAULT_LABELS, ALLOWED_TRANSITIONS,
  type SubmitButtonState, type ReactSubmitButtonProps
} from './react_submit_button';

const renderBtn = (props: Partial<ReactSubmitButtonProps> = {}) => {
  const { container } = render(<ReactSubmitButton state=\"idle\" {...props} />);
  return container.querySelector('button')!;
};

const STATES: SubmitButtonState[] = ['idle', 'pending', 'success', 'error', 'disabled'];

// ── Reducer ──
describe('submitButtonReducer', () => {
  it('START_PENDING sets isPending true', () => {
    expect(submitButtonReducer({ isPending: false }, { type: 'START_PENDING' })).toEqual({ isPending: true });
  });
  it('END_PENDING sets isPending false', () => {
    expect(submitButtonReducer({ isPending: true }, { type: 'END_PENDING' })).toEqual({ isPending: false });
  });
});

// ── Helpers ──
describe('normalizeText', () => {
  it('sanitizes non-strings to fallback', () => {
    expect(normalizeText(null, 'Fallback')).toBe('Fallback');
    expect(normalizeText(123, 'Fallback')).toBe('Fallback');
  });
  it('strips control chars/whitespace', () => {
    expect(normalizeText('\\u0000Test\\n\\t', 'Fallback')).toBe('Test');
  });
  it('truncates long text', () => {
    const long = 'A'.repeat(90);
    expect(normalizeText(long, 'Short')).toHaveLength(80);
  });
});

describe('resolveLabel', () => {
  it('defaults match DEFAULT_LABELS', () => {
    STATES.forEach(s => expect(resolveLabel(s)).toBe(DEFAULT_LABELS[s]));
  });
  it('uses custom sanitized labels', () => {
    expect(resolveLabel('success', { success: ' Funded! ' })).toBe('Funded!');
  });
});

describe('state transitions', () => {
  it('validates allowed transitions', () => {
    expect(isValidTransition('idle', 'pending')).toBe(true);
    expect(isValidTransition('pending', 'success')).toBe(true);
  });
  it('blocks invalid', () => {
    expect(isValidTransition('idle', 'success')).toBe(false);
  });
  it('resolves safe state strict', () => {
    expect(resolveSafeState('success', 'idle')).toBe('idle');
  });
});

// ── Component ──
describe('ReactSubmitButton', () => {
  it('renders button', () => {
    expect(renderBtn().tagName).toBe('BUTTON');
  });

  it('shows resolved label', () => {
    renderBtn();
    expect(screen.getByText('Execute Script')).toBeInTheDocument();
  });

  it('shows subtext for txHash', () => {
    renderBtn({ txHash: 'abc123def456' });
    expect(screen.getByText('(Tx: …def456)')).toBeInTheDocument();
  });

  it('shows sanitized scriptOutput', () => {
    renderBtn({ scriptOutput: 'Deployed\\nSuccess' });
    expect(screen.getByText('(Deployed Success)')).toBeInTheDocument();
  });

  it('shows spinner in pending', () => {
    renderBtn({ state: 'pending' });
    expect(screen.getByRole('img', { hidden: true })).toBeInTheDocument();
  });

  it('disabled/blocked in pending/success/disabled', () => {
    ['pending', 'success', 'disabled'].forEach(s => {
      expect(renderBtn({ state: s as SubmitButtonState }).disabled).toBe(true);
    });
  });

  it('click fires in idle/error', async () => {
    const onClick = jest.fn(Promise.resolve);
    const btn = renderBtn({ onClick });
    await act(() => fireEvent.click(btn));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('double-submit blocked', async () => {
    const slow = jest.fn(() => new Promise(r => setTimeout(r, 100)));
    const btn = renderBtn({ onClick: slow });
    fireEvent.click(btn);
    fireEvent.click(btn); // ignored
    await waitFor(() => expect(slow).toHaveBeenCalledTimes(1));
  });

  it('isMounted guard unmount async', async () => {
    const slow = jest.fn(() => new Promise(r => setTimeout(r, 10)));
    const { unmount } = render(<ReactSubmitButton state='idle' onClick={slow} />);
    const btn = screen.getByRole('button');
    fireEvent.click(btn);
    unmount();
    await waitFor(() => expect(slow).toHaveBeenCalledTimes(1));
  });

  it('a11y: aria-busy pending', () => {
    const btn = renderBtn({ state: 'pending' });
    expect(btn.getAttribute('aria-busy')).toBe('true');
  });

  it('strict transitions fallback', () => {
    const btn = renderBtn({ state: 'success', previousState: 'idle' });
    expect(btn.dataset.state).toBe('idle');
  });
});

