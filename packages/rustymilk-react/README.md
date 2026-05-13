# @rustymilk/react

`@rustymilk/react` provides optional React bindings for the RustyMilk web SDK.

The package is intentionally lightweight and does not import React directly.
Instead, create bindings by passing your React namespace into
`createRustyMilkReactBindings(...)`.

## Install

```bash
npm i @rustymilk/web @rustymilk/react react
```

## Usage

```js
import * as React from 'react';
import { createRustyMilkReactBindings } from '@rustymilk/react';

const {
  useRustyMilkEngine,
  RustyMilkCanvas,
} = createRustyMilkReactBindings(React);

function RustyMilkPlayer({ audioContext, audioNode }) {
  const canvasRef = React.useRef(null);
  const { status, engine } = useRustyMilkEngine({
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
      <RustyMilkCanvas ref={canvasRef} width={960} height={540} />
      <p>{status}</p>
    </div>
  );
}
```

## Public API

- `createRustyMilkReactBindings(react, options?)`
  - Returns `{ useRustyMilkEngine, useRustyMilkPack, RustyMilkCanvas }`.
- `useRustyMilkEngine(config)`
  - Creates/destroys a RustyMilk engine tied to the hook lifecycle.
- `useRustyMilkPack(packUrl, options?)`
  - Fetches and validates pack manifests into simple state.
- `RustyMilkCanvas`
  - Minimal canvas component wrapper.

For richer engine control, import `createRustyMilkEngine` directly from
`@rustymilk/web` and compose your own host layer.
