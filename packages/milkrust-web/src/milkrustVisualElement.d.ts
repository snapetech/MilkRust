import type { MilkRustAutomation, MilkRustEngine, MilkRustRendererBackend } from './milkrustEngine.d.ts';

export interface MilkRustVisualizerElementCreateOptions {
  tagName?: string;
  createEngine?: typeof import('./milkrustEngine.js').createMilkRustEngine;
  baseElement?: CustomElementConstructor | null;
  customElements?: CustomElementsRegistry;
  documentRef?: Document;
}

export interface MilkRustVisualizerElement extends HTMLElement {
  status: 'idle' | 'initializing' | 'ready' | 'error' | 'stopped';
  error: string;
  engine: MilkRustEngine | null;
  presetText: string;
  presetFileName: string;
  modulePath: string;
  rendererBackend: MilkRustRendererBackend | string;
  autoStart: boolean;
  start: () => Promise<void>;
  stop: () => void;
  dispose: () => void;
  setAudioSource: (audioContext: AudioContext, audioNode: AudioNode) => void;
  loadPresetText: (source: string, fileName?: string, options?: Record<string, unknown>) => unknown;
  setPresetAutomation: (nextAutomation: MilkRustAutomation) => void;
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

export function createMilkRustVisualizerElement(
  options?: MilkRustVisualizerElementCreateOptions,
): {
  new (): MilkRustVisualizerElement;
};

export function defineMilkRustVisualizerElement(
  options?: MilkRustVisualizerElementCreateOptions,
): {
  new (): MilkRustVisualizerElement;
};
