import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import {
  createMilkRustVisualizerElement,
} from './milkrustVisualElement.js';

const createFakeAnimationFrameScheduler = () => {
  let nextHandle = 1;
  const timers = new Map();
  const requestFrame = (callback) => {
    const id = nextHandle++;
    const timeout = setTimeout(() => callback(Date.now()), 0);
    timers.set(id, timeout);
    return id;
  };
  const cancelFrame = (id) => {
    const timeout = timers.get(id);
    if (timeout) {
      clearTimeout(timeout);
      timers.delete(id);
    }
  };
  return { requestFrame, cancelFrame };
};

const createFakeElementBase = () => class {
  constructor() {
    this.children = [];
    this.style = {};
    this.className = '';
    this._attributes = {};
    this._listeners = new Map();
    this._dispatched = [];
  }

  appendChild(child) {
    this.children.push(child);
    return child;
  }

  setAttribute(name, value) {
    const oldValue = this.getAttribute(name);
    this._attributes[name] = String(value);
    if (typeof this.attributeChangedCallback === 'function') {
      this.attributeChangedCallback(name, oldValue, String(value));
    }
  }

  removeAttribute(name) {
    const oldValue = this.getAttribute(name);
    delete this._attributes[name];
    if (typeof this.attributeChangedCallback === 'function') {
      this.attributeChangedCallback(name, oldValue, null);
    }
  }

  getAttribute(name) {
    return Object.prototype.hasOwnProperty.call(this._attributes, name)
      ? this._attributes[name]
      : null;
  }

  addEventListener(name, listener) {
    if (!this._listeners.has(name)) {
      this._listeners.set(name, []);
    }
    this._listeners.get(name).push(listener);
  }

  dispatchEvent(event) {
    this._dispatched.push(event);
    const listeners = this._listeners.get(event.type);
    if (listeners) {
      for (const listener of listeners) {
        listener(event);
      }
    }
    return true;
  }

  get isConnected() {
    return true;
  }
};

const createFakeDocument = () => ({
  createElement: () => ({ style: {}, setAttribute() {}, removeAttribute() {} }),
});

const createFakeCustomElements = () => {
  const map = new Map();
  return {
    get(name) {
      return map.get(name);
    },
    define(name, elementClass) {
      map.set(name, elementClass);
    },
  };
};

const installEnvironment = () => {
  const originalRAF = globalThis.requestAnimationFrame;
  const originalCAF = globalThis.cancelAnimationFrame;
  const originalCustomEvent = globalThis.CustomEvent;
  const originalPerformance = globalThis.performance;

  const scheduler = createFakeAnimationFrameScheduler();
  globalThis.requestAnimationFrame = scheduler.requestFrame;
  globalThis.cancelAnimationFrame = scheduler.cancelFrame;
  globalThis.CustomEvent = class {
    constructor(type, details = {}) {
      this.type = type;
      this.detail = details.detail;
    }
  };
  globalThis.performance = {
    now: () => Date.now(),
  };

  return () => {
    globalThis.requestAnimationFrame = originalRAF;
    globalThis.cancelAnimationFrame = originalCAF;
    globalThis.CustomEvent = originalCustomEvent;
    globalThis.performance = originalPerformance;
  };
};

const createFakeEngine = () => {
  const state = {
    presetLoaded: null,
    automation: null,
    mouseState: null,
    renderCount: 0,
    stopped: false,
    status: 'created',
    disposed: false,
  };

  return {
    state,
    methods: {
      loadPresetText: (source, fileName) => {
        state.presetLoaded = { source, fileName };
        return fileName;
      },
      setPresetAutomation: (automation) => {
        state.automation = automation;
      },
      setMouseState: (mouseState) => {
        state.mouseState = mouseState;
      },
      render: () => {
        state.renderCount += 1;
      },
      dispose: () => {
        state.disposed = true;
      },
    },
  };
};

describe('createMilkRustVisualizerElement', () => {
  it('defines and creates a connected visualizer element', async () => {
    const cleanup = installEnvironment();
    const restoreElements = createFakeCustomElements();
    const fakeDocument = createFakeDocument();
    const fakeEngine = createFakeEngine();

    const createEngine = async () => fakeEngine.methods;
    const Element = createMilkRustVisualizerElement({
      tagName: 'milkrust-visualizer-test',
      createEngine,
      baseElement: createFakeElementBase(),
      customElements: restoreElements,
      documentRef: fakeDocument,
    });

    const element = new Element();
    element.setAttribute('preset-text', 'name=Ready');
    element.setAttribute('preset-file-name', 'ready.milk');
    element.setAttribute('automation', JSON.stringify({ mode: 'demo' }));

    element.setAudioSource({ currentTime: 1 }, { id: 'audio-node' });
    await element.start();

    assert.equal(restoreElements.get('milkrust-visualizer-test'), Element);
    assert.equal(element.status, 'ready');
    assert.equal(fakeEngine.state.presetLoaded?.source, 'name=Ready');
    assert.equal(fakeEngine.state.presetLoaded?.fileName, 'ready.milk');
    assert.equal(fakeEngine.state.automation?.mode, 'demo');
    assert.equal(element.children.length, 1);

    element.stop();
    assert.equal(element.status, 'stopped');
    cleanup();
  });

  it('supports auto-start behavior through attribute/property', async () => {
    const cleanup = installEnvironment();
    const fakeElementBaseClass = createFakeElementBase();
    const fakeDocument = createFakeDocument();
    const restoreElements = createFakeCustomElements();
    const fakeEngine = createFakeEngine();
    const createEngine = async () => fakeEngine.methods;

    const Element = createMilkRustVisualizerElement({
      tagName: 'milkrust-visualizer-autostart',
      createEngine,
      baseElement: fakeElementBaseClass,
      customElements: restoreElements,
      documentRef: fakeDocument,
    });
    const element = new Element();

    element.autoStart = true;
    element.setAudioSource({ currentTime: 2 }, { id: 'audio-node' });
    element.connectedCallback();

    await new Promise((resolve) => setTimeout(resolve, 0));

    assert.equal(element.status, 'ready');
    element.dispose();
    cleanup();
  });

  it('queues preset and configuration before engine creation', async () => {
    const cleanup = installEnvironment();
    const fakeDocument = createFakeDocument();
    const fakeEngine = createFakeEngine();
    const createEngine = async () => fakeEngine.methods;
    const element = new (createMilkRustVisualizerElement({
      tagName: 'milkrust-visualizer-preset-queue',
      createEngine,
      baseElement: createFakeElementBase(),
      customElements: createFakeCustomElements(),
      documentRef: fakeDocument,
    }))();

    element.loadPresetText('name=Queued', 'queued.milk');
    element.setPresetAutomation({ mode: 'queued' });
    element.setAudioSource({ currentTime: 7 }, { id: 'audio-node' });
    await element.start();
    element.setMouseState({
      mouse_down: 1,
      mouse_dx: 0.5,
      mouse_dy: -0.2,
      mouse_x: 0.3,
      mouse_y: 0.6,
    });

    assert.equal(fakeEngine.state.presetLoaded?.source, 'name=Queued');
    assert.equal(fakeEngine.state.presetLoaded?.fileName, 'queued.milk');
    assert.equal(fakeEngine.state.automation?.mode, 'queued');
    assert.equal(fakeEngine.state.mouseState?.mouse_x, 0.3);
    assert.equal(element.status, 'ready');
    element.stop();
    cleanup();
  });

  it('dispatches web component status events and stops rendering on stop()', async () => {
    const cleanup = installEnvironment();
    const fakeDocument = createFakeDocument();
    const fakeElementBaseClass = createFakeElementBase();
    const fakeCustomElements = createFakeCustomElements();
    const fakeEngine = createFakeEngine();
    const createEngine = async () => ({
      ...fakeEngine.methods,
      render: () => {
        fakeEngine.methods.render();
      },
    });
    fakeEngine.methods.render = () => {
      fakeEngine.state.renderCount += 1;
    };

    const element = new (createMilkRustVisualizerElement({
      tagName: 'milkrust-visualizer-events',
      createEngine,
      baseElement: fakeElementBaseClass,
      customElements: fakeCustomElements,
      documentRef: fakeDocument,
    }))();

    element.setAudioSource({ currentTime: 3 }, { id: 'audio-node' });
    element.setAttribute('auto-start', '');
    element.setAttribute('frame-limit-fps', '60');
    const events = [];
    element.addEventListener('milkrust:ready', () => events.push('ready'));
    element.addEventListener('milkrust:status', (event) => events.push(`status:${event.detail.status}`));
    await element.start();

    await new Promise((resolve) => setTimeout(resolve, 0));
    element.stop();

    assert.equal(events.includes('ready'), true);
    assert.equal(events.includes('status:ready'), true);
    assert.equal(events.includes('status:stopped'), true);
    assert.equal(fakeEngine.state.disposed, true);
    cleanup();
  });

  it('applies status listener callbacks when status changes', async () => {
    const cleanup = installEnvironment();
    const fakeDocument = createFakeDocument();
    const fakeEngine = createFakeEngine();
    const createEngine = async () => fakeEngine.methods;

    const element = new (createMilkRustVisualizerElement({
      tagName: 'milkrust-visualizer-callbacks',
      createEngine,
      baseElement: createFakeElementBase(),
      customElements: createFakeCustomElements(),
      documentRef: fakeDocument,
    }))();
    const entries = [];
    element.onStatusChange = (status, error) => entries.push(`${status}:${error || ''}`);

    element.setAudioSource({ currentTime: 4 }, { id: 'audio-node' });
    await element.start();
    element.stop();

    assert.equal(entries[0].startsWith('initializing'), true);
    assert.equal(entries[1], 'ready:');
    assert.equal(entries[2], 'stopped:');
    cleanup();
  });
});
