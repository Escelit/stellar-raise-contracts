# SubmitButton

A reusable submit button for crowdfunding actions (contribute, withdraw, refund) that surfaces transaction state to the user and prevents double-submission.

## Location

`frontend/components/forms/SubmitButton.tsx`

## States

| State      | Visual                        | Clickable | Use case                                      |
| :--------- | :---------------------------- | :-------- | :-------------------------------------------- |
| `idle`     | Primary blue, label           | Yes       | Ready to submit                               |
| `loading`  | Primary blue + spinner        | No        | Transaction in-flight                         |
| `success`  | Primary blue, success label   | No        | Transaction confirmed; prevents re-submission |
| `error`    | Danger red, error label       | Yes       | Transaction failed; user can retry            |
| `disabled` | Greyed out (via `disabled` prop) | No     | Deadline passed, goal already met, etc.       |

## Props

| Prop           | Type                                          | Default          | Description                                      |
| :------------- | :-------------------------------------------- | :--------------- | :----------------------------------------------- |
| `label`        | `string`                                      | —                | **Required.** Label shown in idle state.         |
| `state`        | `"idle" \| "loading" \| "success" \| "error"` | `"idle"`         | Current button state.                            |
| `loadingLabel` | `string`                                      | `"Processing…"`  | Label shown while loading.                       |
| `successLabel` | `string`                                      | `"Success!"`     | Label shown on success.                          |
| `errorLabel`   | `string`                                      | `"Failed – try again"` | Label shown on error.                      |
| `disabled`     | `boolean`                                     | `false`          | Externally disable (e.g. deadline passed).       |
| `onClick`      | `() => void`                                  | —                | Fired when state is `idle` or `error`.           |
| `fullWidth`    | `boolean`                                     | `false`          | Render as full-width block.                      |
| `type`         | `"submit" \| "button" \| "reset"`             | `"submit"`       | HTML button type.                                |
| `className`    | `string`                                      | `""`             | Additional CSS class.                            |

## Usage

```tsx
import React, { useState } from "react";
import SubmitButton, { SubmitButtonState } from "../components/forms/SubmitButton";

function ContributeForm() {
  const [state, setState] = useState<SubmitButtonState>("idle");

  async function handleContribute() {
    setState("loading");
    try {
      await invokeContractContribute(/* ... */);
      setState("success");
    } catch {
      setState("error");
    }
  }

  return (
    <SubmitButton
      label="Contribute"
      state={state}
      onClick={handleContribute}
      fullWidth
    />
  );
}
```

## Edge Cases

- **Deadline passed / goal met** – pass `disabled={true}` to lock the button regardless of state.
- **Double-submission** – the button is automatically disabled during `loading` and `success`, so rapid clicks cannot trigger multiple transactions.
- **Retry after error** – the button remains enabled in `error` state so the user can retry without refreshing the page.
- **Custom labels** – all state labels are overridable via props for localisation or action-specific copy.

## Accessibility

- `aria-busy="true"` is set during `loading` so screen readers announce the in-progress state.
- `aria-live="polite"` on the button element announces label changes without interrupting the user.
- `aria-disabled` mirrors the `disabled` attribute for assistive technology compatibility.
- The spinner SVG is `aria-hidden` to avoid redundant announcements.

## Security Notes

- The component never submits data itself; all transaction logic lives in the parent's `onClick` handler.
- Input validation must be performed by the parent before calling `onClick`.
- The `loading` / `success` disabled states prevent double-submission at the UI layer, but the Soroban contract enforces idempotency on-chain.
