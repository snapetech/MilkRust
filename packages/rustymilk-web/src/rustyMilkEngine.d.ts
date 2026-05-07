export type RustyMilkRendererBackend = 'webgl' | 'webgpu' | 'headless';

export type RustyMilkPluginKind = 'data' | 'js';

export interface RustyMilkPresetSource {
  id: string;
  title: string;
  file: string;
  name?: string;
  sourceFormat?: string;
  tags?: string[];
  thumbnail?: string;
  source?: string;
  url?: string;
}

export interface RustyMilkPackTexture {
  id: string;
  file: string;
  aliases?: string[];
  url?: string;
}

export interface RustyMilkPackFragment {
  id: string;
  kind?: string;
  file: string;
  tags?: string[];
  url?: string;
}

export interface RustyMilkPackPluginManifest {
  id: string;
  kind: RustyMilkPluginKind;
  entry: string;
  source?: string;
  url?: string;
}

export interface RustyMilkPackManifest {
  schemaVersion: number;
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  license: string;
  requiredRustyMilkVersion: string;
  sourceUrls: string[];
  presets: RustyMilkPresetSource[];
  textures: RustyMilkPackTexture[];
  fragments: RustyMilkPackFragment[];
  plugins: RustyMilkPackPluginManifest[];
  playlist: string[];
  automationDefaults?: Record<string, unknown>;
}

export interface RustyMilkPackValidation {
  valid: boolean;
  errors: string[];
  warnings: string[];
  manifest: RustyMilkPackManifest;
  manifestUrl?: string;
}

export interface RustyMilkPackLoadResult extends RustyMilkPackValidation {
  presets: RustyMilkPresetSource[];
}

export interface RustyMilkPluginDescriptor {
  id: string;
  kind: RustyMilkPluginKind | string;
  source: string;
  entry: string;
  url?: string;
  payload?: unknown;
  api?: RustyMilkPluginHooks | ((context: RustyMilkPluginContext) => RustyMilkPluginContext);
  module?: unknown;
}

export interface RustyMilkPluginLoadResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
  plugins: RustyMilkPluginDescriptor[];
  manifest: RustyMilkPackManifest;
}

export interface RustyMilkPluginContext {
  preset?: { name?: string };
  presetName?: string;
  source?: string;
  presetIndex?: number;
  timestamp?: number;
  now?: number;
  frame?: {
    bands: { bass: number; mid: number; treble: number };
    spectrum: number[];
    samples: number[];
  };
  automation?: RustyMilkAutomation;
  [key: string]: unknown;
}

export interface RustyMilkPluginState {
  activePresetName: string;
  lastPresetIndex: number;
  pluginCount: number;
  automation?: RustyMilkAutomation;
}

export interface RustyMilkPluginHooks {
  onPresetLoad?: (context: RustyMilkPluginContext) => RustyMilkPluginContext | void;
  onPresetLoaded?: (context: RustyMilkPluginContext) => RustyMilkPluginContext | void;
  onPresetChange?: (context: RustyMilkPluginContext) => RustyMilkPluginContext | void;
  onFrameStart?: (context: RustyMilkPluginContext) => RustyMilkPluginContext | void;
  onAudioFrame?: (context: RustyMilkPluginContext) => RustyMilkPluginContext | void;
  onBeat?: (context: RustyMilkPluginContext) => RustyMilkPluginContext | void;
  onAutomationStep?: (context: RustyMilkPluginContext) => RustyMilkPluginContext | void;
  onRenderFrame?: (context: RustyMilkPluginContext) => RustyMilkPluginContext | void;
  onInput?: (context: RustyMilkPluginContext) => RustyMilkPluginContext | void;
  onExport?: (context: RustyMilkPluginContext) => RustyMilkPluginContext | void;
}

export interface RustyMilkAutomation {
  beatSensitivity?: number;
  beatsPerPreset?: number;
  minBeatIntervalSeconds?: number;
  mode?: 'off' | 'beat' | 'time';
  transitionMode?: 'crossfade' | 'fade_through_black' | 'overlay' | 'cut' | string;
  timedIntervalSeconds?: number;
  transitionSeconds?: number;
  randomizeOnBeat?: boolean;
}

export interface RustyMilkEngineConfig {
  audioContext: AudioContext;
  audioNode: AudioNode;
  canvas: HTMLCanvasElement;
  modulePath?: string;
  rendererBackend?: RustyMilkRendererBackend;
  transitionSeconds?: number;
  defaultPresetName?: string;
}

export interface RustyMilkPresetResult {
  title?: string;
  source?: string;
  values?: Record<string, unknown>;
  fileName?: string;
}

export interface RustyMilkFrame {
  bands: { bass: number; mid: number; treble: number };
  samples: number[];
  spectrum: number[];
}

export interface RustyMilkRenderState {
  presetName?: string;
  presetIndex?: number;
}

export interface RustyMilkEngine {
  name: string;
  presetName: string;
  getPluginState(): RustyMilkPluginState;
  dispose(): void;
  exportPresetFragment(type?: 'shape' | 'wave', index?: number): RustyMilkPresetResult;
  exportPresetText(): RustyMilkPresetResult;
  getPresetDebugSnapshot(): unknown;
  getPresetFragmentSummary(): unknown;
  getPresetParameterSummary(): unknown;
  inspectPresetText(source: string, fileName?: string): RustyMilkPresetResult;
  loadPresetFragmentText(
    source: string,
    fileName: string,
    options?: { type?: 'shape' | 'wave'; textureAssets?: Record<string, string> },
  ): RustyMilkPresetResult;
  loadPresetText(source: string, fileName?: string, options?: { textureAssets?: Record<string, string> }): string;
  loadPresetPack(
    pack: { presets?: RustyMilkPresetSource[]; plugins?: RustyMilkPluginDescriptor[] },
    options?: { index?: number; textureAssets?: Record<string, string>; autoplay?: boolean },
  ): string | null;
  nextPreset(options?: { autoplay?: boolean; index?: number }): string | null;
  randomizePresetParameters(options?: { textureAssets?: Record<string, string> }): RustyMilkPresetResult;
  loadPlugins(plugins: RustyMilkPluginDescriptor[]): {
    pluginCount: number;
    pluginHooks: Record<string, number>;
    plugins: RustyMilkPluginDescriptor[];
  };
  removePresetFragment(
    type: 'shape' | 'wave',
    index?: number,
    options?: { textureAssets?: Record<string, string> },
  ): RustyMilkPresetResult;
  render(): RustyMilkRenderState | null;
  resize(width: number, height: number): void;
  setMouseState(nextMouseState: Partial<{
    mouse_down: number;
    mouse_x: number;
    mouse_y: number;
    mouse_dx: number;
    mouse_dy: number;
  }>): Record<string, number>;
  setPresetAutomation(nextAutomation: RustyMilkAutomation): RustyMilkAutomation;
  updatePresetBaseValue(key: string, value: number, options?: { textureAssets?: Record<string, string> }): RustyMilkPresetResult;
}

export function normalizeStringArray(values?: unknown): string[];
export function normalizeRustyMilkPackManifest(
  manifest?: unknown,
  manifestUrl?: string,
): RustyMilkPackManifest;
export function validateRustyMilkPackManifest(
  manifest?: unknown,
  manifestUrl?: string,
): RustyMilkPackValidation;
export function loadRustyMilkPackManifest(
  packUrl: string,
  options?: { fetchImpl?: typeof fetch },
): Promise<RustyMilkPackValidation>;
export function loadRustyMilkPackPresetSource(
  preset: Pick<RustyMilkPresetSource, 'url' | 'file'>,
  options?: { fetchImpl?: typeof fetch },
): Promise<string>;
export function loadRustyMilkPackPlugins(
  manifest: { manifest?: RustyMilkPackManifest; manifestUrl?: string } | RustyMilkPackManifest,
  options?: { fetchImpl?: typeof fetch },
): Promise<RustyMilkPluginLoadResult>;
export function loadRustyMilkPack(
  packUrl: string,
  options?: { fetchImpl?: typeof fetch },
): Promise<RustyMilkPackLoadResult & RustyMilkPluginLoadResult>;

export function getRustyMilkTransitionProgress(
  frameInterval: number,
  frameIndex: number,
  totalFrames: number,
): number;
export function getRustyMilkTransitionAlphas(
  transitionProgress: number,
  transitionMode?: RustyMilkAutomation['transitionMode'],
): { incoming: number; outgoing: number };

export function getRustyMilkBeatUpdate(
  baseline: Record<string, unknown> | null,
  spectrum: number[],
  now: number,
  automation?: RustyMilkAutomation,
): {
  isBeat: boolean;
  beatCount: number;
  beatSensitivity?: number;
};

export function createRustyMilkEngine(config: RustyMilkEngineConfig): Promise<RustyMilkEngine>;
