import init, { RustyMilkEngine } from '/pkg/rustymilk_wasm.js';

const canvas = document.querySelector('#preview');
const source = document.querySelector('#source');
const report = document.querySelector('#report');
const parameter = document.querySelector('#parameter');
const parameterValue = document.querySelector('#parameter-value');

let engine;
let animationFrame = 0;

const defaultSource = `name=RustyMilk Studio Draft
decay=0.9
wave_r=0.2
wave_g=0.7
wave_b=0.95
wave_a=0.85
wave_scale=1.25
zoom=1
rot=0
per_frame_1=rot=0.02*sin(time*0.5);
shape00_enabled=1
shape00_sides=5
shape00_rad=0.18
shape00_a=0.32
wavecode_0_enabled=1
wavecode_0_samples=96
wavecode_0_per_point1=x=i;
wavecode_0_per_point2=y=0.5+sample*0.35;
`;

const resize = () => {
  const width = Math.max(1, Math.floor(canvas.clientWidth * window.devicePixelRatio));
  const height = Math.max(1, Math.floor(canvas.clientHeight * window.devicePixelRatio));
  if (canvas.width !== width || canvas.height !== height) {
    canvas.width = width;
    canvas.height = height;
    engine?.resize(width, height);
  }
};

const show = (value) => {
  report.textContent = typeof value === 'string' ? value : JSON.stringify(value, null, 2);
};

const inspect = () => {
  try {
    const inspected = JSON.parse(engine.inspectPresetText(source.value, 'studio.milk'));
    const parameters = JSON.parse(engine.getPresetParameterSummaryJson());
    const fragments = JSON.parse(engine.getPresetFragmentSummaryJson());
    const debug = JSON.parse(engine.getPresetDebugSnapshotJson('{}'));
    show({ inspected, parameters, fragments, debug });
  } catch (error) {
    show(error.message || String(error));
  }
};

const loadPreview = () => {
  try {
    const title = engine.loadPresetText(source.value, 'studio.milk', '{}');
    show(JSON.parse(engine.getPresetDebugSnapshotJson('{}')));
    return title;
  } catch (error) {
    show(error.message || String(error));
    return '';
  }
};

const render = (time = 0) => {
  resize();
  const bass = Math.sin(time * 0.004) * 0.5 + 0.5;
  const mid = Math.sin(time * 0.0027 + 1.2) * 0.5 + 0.5;
  const treble = Math.sin(time * 0.006 + 2.4) * 0.5 + 0.5;
  const waveform = Array.from({ length: 128 }, (_, index) =>
    Math.sin(index * 0.21 + time * 0.006) * 0.7);
  const spectrum = Array.from({ length: 128 }, (_, index) =>
    Math.max(0, Math.sin(index * 0.08 + time * 0.004)));
  engine.render(
    time / 1000,
    bass,
    mid,
    treble,
    waveform.join(','),
    spectrum.join(','),
    0,
    0.5,
    0.5,
    0,
    0,
  );
  animationFrame = requestAnimationFrame(render);
};

const exportText = (payload, fallbackName) => {
  if (!payload?.source) return;
  const link = document.createElement('a');
  link.href = URL.createObjectURL(new Blob([payload.source], { type: 'text/plain' }));
  link.download = payload.fileName || fallbackName;
  link.click();
  URL.revokeObjectURL(link.href);
};

await init({ module_or_path: '/pkg/rustymilk_wasm_bg.wasm' });
engine = new RustyMilkEngine(canvas);
source.value = defaultSource;
resize();
loadPreview();
render();

document.querySelector('#preset-file').addEventListener('change', async (event) => {
  const [file] = event.target.files || [];
  if (!file) return;
  source.value = await file.text();
  loadPreview();
});

document.querySelector('#inspect').addEventListener('click', inspect);
document.querySelector('#load-preview').addEventListener('click', loadPreview);

document.querySelector('#randomize').addEventListener('click', () => {
  const result = JSON.parse(engine.randomizePresetParameters('{}'));
  if (result?.source) {
    source.value = result.source;
    loadPreview();
  }
});

document.querySelector('#apply-parameter').addEventListener('click', () => {
  const result = JSON.parse(engine.updatePresetBaseValue(
    parameter.value,
    Number(parameterValue.value),
    '{}',
  ));
  if (result?.source) {
    source.value = result.source;
    loadPreview();
  }
});

document.querySelector('#fragment-file').addEventListener('change', async (event) => {
  const [file] = event.target.files || [];
  if (!file) return;
  const result = JSON.parse(engine.loadPresetFragmentText(
    await file.text(),
    file.name,
    file.name.toLowerCase().endsWith('.wave') ? 'wave' : 'shape',
    '{}',
  ));
  if (result?.source) {
    source.value = result.source;
    loadPreview();
  }
});

document.querySelector('#export-shape').addEventListener('click', () => {
  exportText(JSON.parse(engine.exportPresetFragment('shape', 0)), 'shape.shape');
});

document.querySelector('#export-wave').addEventListener('click', () => {
  exportText(JSON.parse(engine.exportPresetFragment('wave', 0)), 'wave.wave');
});

window.addEventListener('resize', resize);
window.addEventListener('beforeunload', () => {
  cancelAnimationFrame(animationFrame);
  engine?.free?.();
});
