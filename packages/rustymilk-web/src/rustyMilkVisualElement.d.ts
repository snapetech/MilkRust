import type { RustyMilkAutomation, RustyMilkEngine, RustyMilkRendererBackend } from './rustyMilkEngine.d.ts';

export interface RustyMilkVisualizerElementCreateOptions {
  tagName?: string;
  createEngine?: typeof import('./rustyMilkEngine.js').createRustyMilkEngine;
  baseElement?: CustomElementConstructor | null;
  customElements?: CustomElementsRegistry;
  documentRef?: Document;
}

export interface RustyMilkVisualizerElement extends HTMLElement {
  status: 'idle' | 'initializing' | 'ready' | 'error' | 'stopped';
  error: string;
  engine: RustyMilkEngine | null;
  presetText: string;
  presetFileName: string;
  modulePath: string;
  rendererBackend: RustyMilkRendererBackend | string;
  autoStart: boolean;
  start: () => Promise<void>;
  stop: () => void;
  dispose: () => void;
  setAudioSource: (audioContext: AudioContext, audioNode: AudioNode) => void;
  loadPresetText: (source: string, fileName?: string, options?: Record<string, unknown>) => unknown;
  setPresetAutomation: (nextAutomation: RustyMilkAutomation) => void;
  setMouseState: (mouseState: Partial<{
    mouse_down: number;
    mouse_x: number;
    mouse_y: number;
    mouse_dx: number;
    mouse_dy: number;
  }>) => void;
  render: () => unknown;
  onStatusChange?: (status: string, error: string) => void;
}

export function createRustyMilkVisualizerElement(
  options?: RustyMilkVisualizerElementCreateOptions,
): {
  new (): RustyMilkVisualizerElement;
};

export function defineRustyMilkVisualizerElement(
  options?: RustyMilkVisualizerElementCreateOptions,
): {
  new (): RustyMilkVisualizerElement;
};
