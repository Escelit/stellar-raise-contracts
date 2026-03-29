# ReactSubmitButton - Script-Optimized

Typed React button for script execution (e.g. wasm_build_pipeline, deploy.sh) and Stellar tx.

## States (Script Flow)

| State      | Description               | Clickable |
| ---------- | ------------------------- | --------- |
| `idle`     | Ready                     | ✅        |
| `pending`  | Script/tx in-flight       | ❌        |
| `success`  | Complete                  | ❌        |
| `error`    | Failed, retry possible    | ✅        |
| `disabled` | Locked (goal met, paused) | ❌        |

Transitions: idle→pending→success/error; strict enforcement.

## Props

| Prop           | Type    | Description       |
| -------------- | ------- | ----------------- |
| `state`        | State   | Required          |
| `scriptOutput` | unknown | Sanitized display |
| `txHash`       | string  | Truncated         |
| `labels`       | object  | Per-state         |
| `onClick`      | fn      | Handler           |

## Usage - Scripts

```tsx
<ReactSubmitButton
  state={scriptState}
  scriptOutput={deployResult}
  txHash={tx.hash}
  labels={{ idle: "Deploy WASM" }}
  onClick={runDeployScript}
/>
```

Example with wasm_build_pipeline:

```tsx
const [state, setState] = useState("idle");
const runScript = async () => {
  setState("pending");
  const result = await wasmBuildPipeline();
  setState(result.success ? "success" : "error");
};
```

## Security

- Double-script prevention (inFlightRef)
- Sanitized output/labels (no XSS)
- Truncated txHash (no full exposure)
- Strict transitions (no invalid UI)

## Tests

66+ cases, ≥98% coverage. Includes scriptOutput sanitization, double-click block.

See `react_submit_button.test.tsx`.
