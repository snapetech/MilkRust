import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import {
  createRustyMilkReactBindings,
} from './index.js';

const createFakeReact = (onUseEffect = () => {}) => {
  const cleanups = [];
  return {
    cleanups,
    useCallback: (fn) => fn,
    useMemo: (factory) => factory(),
    useEffect: (setup) => {
      const cleanup = setup();
      if (typeof cleanup === 'function') {
        cleanups.push(cleanup);
      }
      onUseEffect();
    },
    useRef: (value) => ({ current: value }),
    useState: (value) => {
      let state = value;
      return [state, (next) => {
        state = typeof next === 'function' ? next(state) : next;
      }];
    },
    createElement: (...args) => ({ tag: args[0], props: args[1] }),
    forwardRef: (render) => (props) => render(props, null),
  };
};

describe('createRustyMilkReactBindings', () => {
  it('requires React hooks', () => {
    assert.throws(
      () => createRustyMilkReactBindings({}),
      /missing required React hooks/i,
    );
  });

  it('builds reusable React bindings with default hook contract', async () => {
    let engineLoaded = false;
    const fakeEngine = {
      render: () => ({ presetName: 'sample' }),
      dispose: () => { engineLoaded = false; },
      setPresetAutomation: () => {},
      loadPresetText: () => 'ok',
      getPresetDebugSnapshot: () => ({ presetName: 'ok' }),
      getPresetParameterSummary: () => ({}),
      getPresetFragmentSummary: () => ({}),
      loadPresetFragmentText: () => ({ source: 'ok' }),
      updatePresetBaseValue: () => ({ source: 'ok' }),
      randomizePresetParameters: () => ({ source: 'ok' }),
      removePresetFragment: () => ({ source: 'ok' }),
      inspectPresetText: () => ({ title: 'ok' }),
      exportPresetText: () => ({ source: 'ok' }),
      exportPresetFragment: () => ({ source: 'ok' }),
      setMouseState: () => {},
      setPresetAutomation: () => {},
    };
    const fakeCreateEngine = async () => {
      engineLoaded = true;
      return fakeEngine;
    };

    const fakeReact = createFakeReact();
    const { useRustyMilkEngine, RustyMilkCanvas, useRustyMilkPack } =
      createRustyMilkReactBindings(fakeReact, {
        createEngine: fakeCreateEngine,
        packLoader: async () => ({ status: 'ok', plugins: [], presets: [], manifest: {} }),
      });

    const hookState = useRustyMilkEngine({
      audioContext: { currentTime: 0, createAnalyser: () => ({}) },
      audioNode: { connect: () => {}, disconnect: () => {} },
      canvas: { getContext: () => ({}) },
      autoStart: false,
    });

    assert.equal(typeof hookState.start, 'function');
    assert.equal(typeof hookState.dispose, 'function');
    assert.equal(typeof hookState.render, 'function');

    await hookState.start();
    assert.equal(engineLoaded, true);
    assert.equal(typeof RustyMilkCanvas, 'function');
    hookState.dispose();

    fakeReact.cleanups.length = 0;

    const packState = useRustyMilkPack('http://localhost/test', {
      fetchImpl: async () => ({ ok: true, json: async () => ({}) }),
    });
    assert.equal(packState.status === 'idle' || packState.status === 'loading', true);
  });
});
