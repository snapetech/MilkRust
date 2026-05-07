import { createRustyMilkEngine } from '/packages/rustymilk-web/src/rustyMilkEngine.js';

const canvas = document.querySelector('#visualizer');
const status = document.querySelector('#status');
const debug = document.querySelector('#debug');
const presetList = document.querySelector('#preset-list');
const automation = document.querySelector('#automation');
const interval = document.querySelector('#interval');
const beats = document.querySelector('#beats');

const builtInPresets = [
  {
    id: crypto.randomUUID(),
    name: 'RustyMilk Grid Smoke',
    source: 'name=RustyMilk Grid Smoke\ndecay=0.91\nwave_r=0.12\nwave_g=0.64\nwave_b=0.88\nwave_a=0.86\nwave_scale=1.2\nzoom=1\nrot=0\nshape00_enabled=1\nshape00_sides=5\nshape00_rad=0.18\nwavecode_0_enabled=1\nwavecode_0_samples=96\nwavecode_0_per_point1=x=i;\nwavecode_0_per_point2=y=0.08+sample*0.55;',
  },
  {
    id: crypto.randomUUID(),
    name: 'RustyMilk Amber Tunnel',
    source: 'name=RustyMilk Amber Tunnel\ndecay=0.86\nwave_r=0.92\nwave_g=0.52\nwave_b=0.18\nwave_a=0.82\nwave_scale=1.55\nzoom=1.05\nrot=-0.018\nshape00_enabled=1\nshape00_sides=3\nshape01_enabled=1\nshape01_sides=6\nwavecode_0_enabled=1',
  },
];

let audioContext;
let audioNode;
let oscillator;
let engine;
let animationFrame = 0;
let activeIndex = 0;
let presets = [...builtInPresets];
let textureAssets = {};

const setStatus = (value) => {
  status.textContent = value;
};

const refreshPresetList = () => {
  presetList.replaceChildren(...presets.map((preset, index) => {
    const option = document.createElement('option');
    option.value = String(index);
    option.textContent = preset.name;
    return option;
  }));
  presetList.value = String(activeIndex);
};

const resize = () => {
  const width = Math.max(1, Math.floor(canvas.clientWidth * window.devicePixelRatio));
  const height = Math.max(1, Math.floor(canvas.clientHeight * window.devicePixelRatio));
  if (canvas.width !== width || canvas.height !== height) {
    canvas.width = width;
    canvas.height = height;
    engine?.resize(width, height);
  }
};

const loadActivePreset = () => {
  if (!engine) return;
  const preset = presets[activeIndex];
  const title = engine.loadPresetText(preset.source, preset.name, { textureAssets });
  setStatus(title);
  debug.textContent = JSON.stringify(engine.getPresetDebugSnapshot(), null, 2);
  refreshPresetList();
};

const render = () => {
  resize();
  const update = engine?.render();
  if (update?.presetName) setStatus(update.presetName);
  animationFrame = requestAnimationFrame(render);
};

const stopEngine = () => {
  cancelAnimationFrame(animationFrame);
  engine?.dispose();
  oscillator?.stop?.();
  engine = null;
  oscillator = null;
};

const startWithNode = async (context, node) => {
  stopEngine();
  audioContext = context;
  audioNode = node;
  engine = await createRustyMilkEngine({
    audioContext,
    audioNode,
    canvas,
    modulePath: '/pkg/rustymilk_wasm.js',
  });
  engine.setPresetAutomation({
    beatsPerPreset: Number(beats.value) || 8,
    mode: automation.value,
    timedIntervalSeconds: Number(interval.value) || 30,
  });
  loadActivePreset();
  render();
};

document.querySelector('#start-demo').addEventListener('click', async () => {
  const context = new AudioContext();
  const gain = context.createGain();
  gain.gain.value = 0.0001;
  oscillator = context.createOscillator();
  oscillator.frequency.value = 96;
  oscillator.connect(gain);
  gain.connect(context.destination);
  oscillator.start();
  await startWithNode(context, gain);
});

document.querySelector('#start-mic').addEventListener('click', async () => {
  const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
  const context = new AudioContext();
  await startWithNode(context, context.createMediaStreamSource(stream));
});

presetList.addEventListener('change', () => {
  activeIndex = Number(presetList.value) || 0;
  loadActivePreset();
});

document.querySelector('#previous').addEventListener('click', () => {
  activeIndex = (activeIndex + presets.length - 1) % presets.length;
  loadActivePreset();
});

document.querySelector('#next').addEventListener('click', () => {
  activeIndex = (activeIndex + 1) % presets.length;
  loadActivePreset();
});

document.querySelector('#random').addEventListener('click', () => {
  activeIndex = Math.floor(Math.random() * presets.length);
  loadActivePreset();
});

automation.addEventListener('change', () => {
  engine?.setPresetAutomation({
    beatsPerPreset: Number(beats.value) || 8,
    mode: automation.value,
    timedIntervalSeconds: Number(interval.value) || 30,
  });
});

document.querySelector('#preset-files').addEventListener('change', async (event) => {
  const files = Array.from(event.target.files || []);
  for (const file of files) {
    const source = await file.text();
    presets.push({
      id: crypto.randomUUID(),
      name: file.name,
      source,
    });
  }
  activeIndex = Math.max(0, presets.length - files.length);
  refreshPresetList();
  loadActivePreset();
});

document.querySelector('#texture-files').addEventListener('change', async (event) => {
  const files = Array.from(event.target.files || []);
  const entries = await Promise.all(files.map((file) => new Promise((resolve) => {
    const reader = new FileReader();
    reader.onload = () => resolve([file.name, String(reader.result || '')]);
    reader.readAsDataURL(file);
  })));
  textureAssets = Object.fromEntries(entries);
  loadActivePreset();
});

canvas.addEventListener('pointermove', (event) => {
  const rect = canvas.getBoundingClientRect();
  engine?.setMouseState({
    mouse_x: (event.clientX - rect.left) / rect.width,
    mouse_y: (event.clientY - rect.top) / rect.height,
    mouse_dx: event.movementX / Math.max(1, rect.width),
    mouse_dy: event.movementY / Math.max(1, rect.height),
  });
});

canvas.addEventListener('pointerdown', () => engine?.setMouseState({ mouse_down: 1 }));
canvas.addEventListener('pointerup', () => engine?.setMouseState({ mouse_down: 0 }));
window.addEventListener('resize', resize);
refreshPresetList();
