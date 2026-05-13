import type {
  RustyMilkAutomation,
  RustyMilkPackLoadResult,
  RustyMilkEngine,
} from '@rustymilk/web';

export interface RustyMilkReactDependency {
  useCallback: <T>(fn: T) => T;
  useEffect: (effect: () => (void | (() => void)), deps?: unknown[] | undefined) => void;
  useMemo: <T>(factory: () => T, deps?: unknown[] | undefined) => T;
  useRef: <T>(initialValue: T) => { current: T };
  useState: <T>(initialValue: T) => [T, (next: T | ((previous: T) => T)) => void];
  createElement?: (...args: any[]) => unknown;
  forwardRef?: <T>(render: (props: any, ref: any) => unknown) => unknown;
}

export interface RustyMilkCanvasProps {
  className?: string;
  style?: Record<string, unknown>;
  [key: string]: unknown;
}

export interface RustyMilkEngineHookResult {
  status: 'idle' | 'initializing' | 'ready' | 'error';
  error: string;
  engine: RustyMilkEngine | null;
  start: () => Promise<void>;
  dispose: () => void;
  render: () => unknown;
  getPresetDebugSnapshot: () => unknown;
  getPresetParameterSummary: () => unknown;
  getPresetFragmentSummary: () => unknown;
  loadPresetText: (source: string, fileName?: string, options?: unknown) => unknown;
  loadPresetFragmentText: (source: string, fileName?: string, options?: unknown) => unknown;
  updatePresetBaseValue: (key: string, value: number, options?: unknown) => unknown;
  randomizePresetParameters: (options?: unknown) => unknown;
  removePresetFragment: (type: 'shape' | 'wave', index?: number, options?: unknown) => unknown;
  inspectPresetText: (source: string, fileName?: string) => unknown;
  exportPresetText: () => unknown;
  exportPresetFragment: (type?: 'shape' | 'wave', index?: number) => unknown;
  setMouseState: (next: Record<string, number>) => unknown;
  setPresetAutomation: (next: RustyMilkAutomation) => unknown;
  setTextureAssets: (nextTextureAssets: Record<string, string>) => void;
}

export interface RustyMilkEngineHookConfig {
  audioContext: AudioContext;
  audioNode: AudioNode;
  canvas: HTMLCanvasElement;
  modulePath?: string;
  rendererBackend?: 'webgl' | 'webgpu' | 'headless';
  presetText?: string;
  presetFileName?: string;
  textureAssets?: Record<string, string>;
  automation?: RustyMilkAutomation;
  onFrame?: (next: unknown) => void;
  onError?: (error: unknown) => void;
  frameLimitFps?: number;
  autoStart?: boolean;
  [key: string]: unknown;
}

export type RustyMilkPackHookState =
  | { status: 'idle'; pack: null; error: '' }
  | { status: 'loading'; pack: null; error: '' }
  | { status: 'ready'; pack: RustyMilkPackLoadResult; error: '' }
  | { status: 'error'; pack: null; error: string };

export interface RustyMilkBindings {
  useRustyMilkEngine: (config: RustyMilkEngineHookConfig) => RustyMilkEngineHookResult;
  useRustyMilkPack: (
    packUrl: string | null | undefined,
    options?: {
      fetchImpl?: typeof fetch;
    },
  ) => RustyMilkPackHookState;
  RustyMilkCanvas: (props: RustyMilkCanvasProps) => unknown;
}

export interface RustyMilkReactBindingFactory {
  createRustyMilkEngine?: typeof import('@rustymilk/web').createRustyMilkEngine;
  loadRustyMilkPack?: typeof import('@rustymilk/web').loadRustyMilkPack;
}

export function createRustyMilkReactBindings(
  react?: RustyMilkReactDependency,
  options?: RustyMilkReactBindingFactory,
): RustyMilkBindings;

export const createRustyMilkReact: typeof createRustyMilkReactBindings;
