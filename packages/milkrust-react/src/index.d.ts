import type {
  MilkRustAutomation,
  MilkRustPackLoadResult,
  MilkRustEngine,
} from '@milkrust/web';

export interface MilkRustReactDependency {
  useCallback: <T>(fn: T) => T;
  useEffect: (effect: () => (void | (() => void)), deps?: unknown[] | undefined) => void;
  useMemo: <T>(factory: () => T, deps?: unknown[] | undefined) => T;
  useRef: <T>(initialValue: T) => { current: T };
  useState: <T>(initialValue: T) => [T, (next: T | ((previous: T) => T)) => void];
  createElement?: (...args: any[]) => unknown;
  forwardRef?: <T>(render: (props: any, ref: any) => unknown) => unknown;
}

export interface MilkRustCanvasProps {
  className?: string;
  style?: Record<string, unknown>;
  [key: string]: unknown;
}

export interface MilkRustEngineHookResult {
  status: 'idle' | 'initializing' | 'ready' | 'error';
  error: string;
  engine: MilkRustEngine | null;
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
  setPresetAutomation: (next: MilkRustAutomation) => unknown;
  setTextureAssets: (nextTextureAssets: Record<string, string>) => void;
}

export interface MilkRustEngineHookConfig {
  audioContext: AudioContext;
  audioNode: AudioNode;
  canvas: HTMLCanvasElement;
  modulePath?: string;
  rendererBackend?: 'webgl' | 'webgpu' | 'headless';
  presetText?: string;
  presetFileName?: string;
  textureAssets?: Record<string, string>;
  automation?: MilkRustAutomation;
  onFrame?: (next: unknown) => void;
  onError?: (error: unknown) => void;
  frameLimitFps?: number;
  autoStart?: boolean;
  [key: string]: unknown;
}

export type MilkRustPackHookState =
  | { status: 'idle'; pack: null; error: '' }
  | { status: 'loading'; pack: null; error: '' }
  | { status: 'ready'; pack: MilkRustPackLoadResult; error: '' }
  | { status: 'error'; pack: null; error: string };

export interface MilkRustBindings {
  useMilkRustEngine: (config: MilkRustEngineHookConfig) => MilkRustEngineHookResult;
  useMilkRustPack: (
    packUrl: string | null | undefined,
    options?: {
      fetchImpl?: typeof fetch;
    },
  ) => MilkRustPackHookState;
  MilkRustCanvas: (props: MilkRustCanvasProps) => unknown;
}

export interface MilkRustReactBindingFactory {
  createMilkRustEngine?: typeof import('@milkrust/web').createMilkRustEngine;
  loadMilkRustPack?: typeof import('@milkrust/web').loadMilkRustPack;
}

export function createMilkRustReactBindings(
  react?: MilkRustReactDependency,
  options?: MilkRustReactBindingFactory,
): MilkRustBindings;

export const createMilkRustReact: typeof createMilkRustReactBindings;
