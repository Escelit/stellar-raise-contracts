# React Submit Button Component States

## Purpose

`react_submit_button.tsx` provides a scalable submit-button state model that standardizes behavior across forms and workflows.

It focuses on:

- predictable state mapping
- accessibility defaults
- safer label handling
- easy extensibility for future states

## File locations

- Component: `frontend/components/react_submit_button.tsx`
- Tests: `frontend/components/react_submit_button.test.tsx`

## State model

The component supports a strict state union:

- `idle`
- `submitting`
- `success`
- `error`
- `disabled`

This ensures only approved states are used in consuming code and avoids ad-hoc string behavior.

## Security assumptions and safeguards

### Assumptions

- Labels may originate from untrusted sources (for example, API-driven copy or admin configuration).
- Consumers should not pass raw HTML into UI APIs.

### Safeguards implemented

1. **Text-only rendering path**  
   Labels are rendered as normal React string children. React escapes these values by default, reducing XSS risk when strings include markup-like text.

2. **Label normalization and fallback**  
   Empty or whitespace-only labels are rejected and replaced with known defaults, preventing blank CTA states.

3. **Label length bounding**  
   Labels are capped to 80 characters to prevent visual abuse and accidental layout breaks.

4. **State-based disable guard**  
   Click handling is removed when state is `submitting` or `disabled`, reducing duplicate submissions.

5. **Accessibility signaling**  
   `aria-busy` is enabled only while submitting; `aria-live="polite"` allows assistive technologies to announce state text changes.

## Usage example

```tsx
import ReactSubmitButton from "../components/react_submit_button";

<ReactSubmitButton
  state="submitting"
  type="submit"
  labels={{ idle: "Create Campaign", submitting: "Creating..." }}
  onClick={handleCreate}
/>;
```

## Testing coverage

`react_submit_button.test.tsx` validates:

- default labels per state
- custom label overrides
- fallback behavior for invalid labels
- long-label truncation edge case
- hostile label string handling assumptions
- disabled-state logic
- busy-state logic

## Review notes

- The component exports pure helper functions (`resolveSubmitButtonLabel`, `isSubmitButtonDisabled`, `isSubmitButtonBusy`) to keep tests deterministic and lightweight.
- Styling is state-mapped via a single lookup table to make future variants easy to add and review.
