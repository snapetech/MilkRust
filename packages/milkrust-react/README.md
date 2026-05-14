# @milkrust/react

`@milkrust/react` provides optional React bindings for the MilkRust web SDK.

The package is intentionally lightweight and does not import React directly.
Instead, create bindings by passing your React namespace into
`createMilkRustReactBindings(...)`.

## Install

```bash
npm i @milkrust/web @milkrust/react react
```

## Usage

```js
import * as React from 'react';
import { createMilkRustReactBindings } from '@milkrust/react';

const {
  useMilkRustEngine,
  MilkRustCanvas,
} = createMilkRustReactBindings(React);

function MilkRustPlayer({ audioContext, audioNode }) {
  const canvasRef = React.useRef(null);
  const { status, engine } = useMilkRustEngine({
    audioContext,
    audioNode,
    canvas: canvasRef.current,
    autoStart: true,
    onFrame: (state) => {
      // Optional per-frame diagnostics.
      void state;
    },
  });

  return (
    <div>
      <MilkRustCanvas ref={canvasRef} width={960} height={540} />
      <p>{status}</p>
    </div>
  );
}
```

## Public API

- `createMilkRustReactBindings(react, options?)`
  - Returns `{ useMilkRustEngine, useMilkRustPack, MilkRustCanvas }`.
- `useMilkRustEngine(config)`
  - Creates/destroys a MilkRust engine tied to the hook lifecycle.
- `useMilkRustPack(packUrl, options?)`
  - Fetches and validates pack manifests into simple state.
- `MilkRustCanvas`
  - Minimal canvas component wrapper.

For richer engine control, import `createMilkRustEngine` directly from
`@milkrust/web` and compose your own host layer.
