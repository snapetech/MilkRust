import { createMilkRustEngine } from './milkrustEngine.js';

const FRAME_DELAY_FALLBACK_MS = 16;

const boolAttr = (value) => !(value === null || value === false || value === '0' || value === 'false');

const parseNumber = (value, fallback = 0) => {
  const numeric = Number(value);
  return Number.isFinite(numeric) ? numeric : fallback;
};

const parseAutomation = (value) => {
  if (value === null || value === undefined) return undefined;
  const stringValue = String(value).trim();
  if (!stringValue) return undefined;
  try {
    return JSON.parse(stringValue);
  } catch {
    return undefined;
  }
};

const toFrameScheduler = () => {
  if (typeof globalThis?.requestAnimationFrame === 'function') {
    return {
      requestFrame: globalThis.requestAnimationFrame.bind(globalThis),
      cancelFrame: globalThis.cancelAnimationFrame.bind(globalThis),
    };
  }
  return {
    requestFrame: (callback) => setTimeout(() => callback(Date.now()), FRAME_DELAY_FALLBACK_MS),
    cancelFrame: (id) => clearTimeout(id),
  };
};

export const createMilkRustVisualizerElement = ({
  tagName = 'milkrust-visualizer',
  createEngine = createMilkRustEngine,
  baseElement = globalThis.HTMLElement,
  customElements = globalThis.customElements,
  documentRef = globalThis.document,
} = {}) => {
  if (!documentRef || typeof documentRef.createElement !== 'function') {
    throw new Error('milkrustVisualElement requires a DOM document context');
  }
  const scheduler = toFrameScheduler();

  class MilkRustVisualizerElement extends (baseElement || class {}) {
    static get observedAttributes() {
      return [
        'module-path',
        'preset-text',
        'preset-file-name',
        'automation',
        'renderer-backend',
        'frame-limit-fps',
        'auto-start',
      ];
    }

    #audioContext;
    #audioNode;
    #canvas;
    #destroyed = false;
    #engine;
    #frameHandle = 0;
    #lastFrameMs = 0;
    #frameStarted = false;
    #modulePath = '/pkg/milkrust_wasm.js';
    #pendingPreset = null;
    #presetFileName = 'web-component.milk';
    #presetText = '';
    #rendererBackend = 'webgl';
    #frameLimitFps = 0;
    #status = 'idle';
    #error = '';
    #automation = undefined;
    #onStatus = null;
    #canvasSize = {
      width: 300,
      height: 150,
    };

    constructor() {
      super();
      this._setupCanvas();
    }

    _setupCanvas() {
      this.#canvas = documentRef.createElement('canvas');
      this.#canvas.style.width = '100%';
      this.#canvas.style.height = '100%';
      this.#canvas.style.display = 'block';
      if (!this.#canvas.className) {
        this.#canvas.className = 'milkrust-visualizer-canvas';
      }
      this.appendChild(this.#canvas);
    }

    #syncCanvasSize() {
      if (!this.#canvas) return this.#canvasSize;
      const width = Math.max(1, Math.floor(this.#canvas.clientWidth || this.#canvas.width || 300));
      const height = Math.max(1, Math.floor(this.#canvas.clientHeight || this.#canvas.height || 150));
      this.#canvas.width = width;
      this.#canvas.height = height;
      this.#canvasSize = { width, height };
      return this.#canvasSize;
    }

    connectedCallback() {
      this.#destroyed = false;
      if (this.autoStart) {
        void this.start();
      }
    }

    disconnectedCallback() {
      this.dispose();
    }

    attributeChangedCallback(name, _oldValue, newValue) {
      if (name === 'module-path') {
        this.#modulePath = newValue || '/pkg/milkrust_wasm.js';
        return;
      }
      if (name === 'preset-text') {
        this.#presetText = String(newValue || '');
        this._applyPendingPreset();
        return;
      }
      if (name === 'preset-file-name') {
        this.#presetFileName = String(newValue || 'web-component.milk');
        return;
      }
      if (name === 'automation') {
        this.#automation = parseAutomation(newValue);
        if (this.#engine) {
          this.#engine.setPresetAutomation?.(this.#automation);
        }
        return;
      }
      if (name === 'renderer-backend') {
        this.#rendererBackend = String(newValue || 'webgl');
        return;
      }
      if (name === 'frame-limit-fps') {
        this.#frameLimitFps = Math.max(0, Math.floor(parseNumber(newValue, 0)));
        return;
      }
      if (name === 'auto-start') {
        if (boolAttr(newValue) && !this.#frameStarted && !this.#engine && this.#audioContext && this.#audioNode) {
          void this.start();
        }
      }
    }

    get status() {
      return this.#status;
    }

    get error() {
      return this.#error;
    }

    get engine() {
      return this.#engine || null;
    }

    get autoStart() {
      return boolAttr(this.getAttribute('auto-start'));
    }

    set autoStart(nextValue) {
      if (boolAttr(nextValue)) {
        this.setAttribute('auto-start', '');
      } else {
        this.removeAttribute('auto-start');
      }
    }

    get presetText() {
      return this.#presetText;
    }

    set presetText(value) {
      this.#presetText = String(value || '');
      this.setAttribute('preset-text', this.#presetText);
      this._applyPendingPreset();
    }

    get presetFileName() {
      return this.#presetFileName;
    }

    set presetFileName(value) {
      this.#presetFileName = String(value || 'web-component.milk');
      this.setAttribute('preset-file-name', this.#presetFileName);
    }

    get modulePath() {
      return this.#modulePath;
    }

    set modulePath(nextPath) {
      this.#modulePath = String(nextPath || '/pkg/milkrust_wasm.js');
      this.setAttribute('module-path', this.#modulePath);
    }

    get rendererBackend() {
      return this.#rendererBackend;
    }

    set rendererBackend(nextBackend) {
      const value = String(nextBackend || 'webgl');
      this.#rendererBackend = value;
      this.setAttribute('renderer-backend', value);
    }

    set onStatusChange(handler) {
      this.#onStatus = typeof handler === 'function' ? handler : null;
    }

    setAudioSource(audioContext, audioNode) {
      this.#audioContext = audioContext;
      this.#audioNode = audioNode;
      if (!this.#destroyed && this.autoStart && !this.#engine && this.isConnected) {
        void this.start();
      }
    }

    async start() {
      if (this.#destroyed) return;
      if (this.#frameStarted || !this.#audioContext || !this.#audioNode || !this.#canvas) {
        return;
      }
      this.#setStatus('initializing');
      try {
        if (typeof this.#audioContext.resume === 'function') {
          await this.#audioContext.resume();
        }
        this.#engine = await createEngine({
          audioContext: this.#audioContext,
          audioNode: this.#audioNode,
          canvas: this.#canvas,
          modulePath: this.#modulePath,
          rendererBackend: this.#rendererBackend,
        });
        if (this.#destroyed) {
          this.#engine?.dispose?.();
          this.#engine = null;
          return;
        }
        if (this.#automation) {
          this.#engine?.setPresetAutomation?.(this.#automation);
        }
        if (this.#pendingPreset) {
          this._applyPendingPreset();
        } else if (this.#presetText) {
          this.#engine?.loadPresetText?.(this.#presetText, this.#presetFileName);
        }
        const { width, height } = this.#syncCanvasSize();
        this.#engine?.resize?.(width, height);
        this.#frameStarted = true;
        this.#setStatus('ready');
        this.dispatchEvent(new CustomEvent('milkrust:ready'));
        this.#loop();
      } catch (error) {
        this.#setStatus('error', error?.message || String(error));
      }
    }

    stop() {
      this.#frameStarted = false;
      this.#cancelLoop();
      if (this.#engine) {
        this.#engine.dispose?.();
      }
      this.#engine = null;
      this.#setStatus('stopped');
      this.dispatchEvent(new CustomEvent('milkrust:stopped'));
    }

    dispose() {
      this.#destroyed = true;
      this.stop();
    }

    _applyPendingPreset() {
      if (!this.#engine || !this.#pendingPreset) return;
      const { source, fileName, options } = this.#pendingPreset;
      const nextSource = this.#presetText || source;
      if (!nextSource) return;
      this.#engine.loadPresetText(nextSource, fileName || this.#presetFileName, options || {});
      this.#pendingPreset = null;
    }

    loadPresetText(source, fileName, options = {}) {
      const next = {
        source: String(source || ''),
        fileName: String(fileName || this.#presetFileName),
        options,
      };
      this.#pendingPreset = next;
      if (this.#engine) {
        this.#engine.loadPresetText(next.source, next.fileName, next.options);
        this.#pendingPreset = null;
      }
      this.#presetText = next.source;
      this.setAttribute('preset-text', this.#presetText);
      return this.#engine ? this.#engine.getPresetDebugSnapshot?.() : null;
    }

    setPresetAutomation(nextAutomation = {}) {
      this.#automation = nextAutomation;
      if (this.#engine?.setPresetAutomation) {
        this.#engine.setPresetAutomation(nextAutomation);
      }
      this.setAttribute('automation', JSON.stringify(nextAutomation));
    }

    setMouseState(nextState) {
      if (this.#engine?.setMouseState) {
        this.#engine.setMouseState(nextState || {});
      }
    }

    render() {
      return this.#engine?.render?.();
    }

    #cancelLoop() {
      if (this.#frameHandle) {
        scheduler.cancelFrame(this.#frameHandle);
        this.#frameHandle = 0;
      }
    }

    #loop() {
      if (!this.#frameStarted || this.#destroyed || !this.#engine) return;
      this.#frameHandle = scheduler.requestFrame(() => {
        if (!this.#frameStarted || this.#destroyed || !this.#engine) return;
        const now = globalThis.performance ? globalThis.performance.now() : Date.now();
        const targetMs = this.#frameLimitFps > 0 ? (1000 / this.#frameLimitFps) : 0;
        if (targetMs > 0 && this.#lastFrameMs && now - this.#lastFrameMs < targetMs) {
          this.#loop();
          return;
        }
        this.#lastFrameMs = now;
        try {
          this.#engine.render?.();
        } catch (error) {
          this.#setStatus('error', error?.message || String(error));
          this.#frameStarted = false;
        }
        this.#loop();
      });
    }

    #setStatus(status, error = '') {
      this.#status = status;
      this.#error = error || '';
      if (this.#onStatus) {
        this.#onStatus(status, error);
      }
      this.dispatchEvent(new CustomEvent('milkrust:status', { detail: { status, error } }));
    }
  }

  if (customElements && typeof customElements.get === 'function' && !customElements.get(tagName)) {
    customElements.define(tagName, MilkRustVisualizerElement);
  }

  return MilkRustVisualizerElement;
};

export const defineMilkRustVisualizerElement = (options = {}) => {
  const ElementClass = createMilkRustVisualizerElement(options);
  return ElementClass;
};
