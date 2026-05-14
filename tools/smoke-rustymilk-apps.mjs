#!/usr/bin/env node
import { chromium } from '@playwright/test';
import { readFile } from 'node:fs/promises';

import { createMilkRustAppServer } from './serve-milkrust-app.mjs';

const listen = async (app, options = {}) => {
  const { appPath, server } = createMilkRustAppServer({ app, ...options });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const { port } = server.address();
  return {
    appPath,
    close: () => new Promise((resolve) => server.close(resolve)),
    url: `http://127.0.0.1:${port}${appPath}`,
  };
};

const browser = await chromium.launch({ headless: true });
const servers = [];
const browserMessages = [];

try {
  servers.push(await listen('player', { includeCommunityContent: true }));
  servers.push(await listen('studio'));
  servers.push(await listen('web-component'));

  const player = await browser.newPage();
  player.on('console', (message) => browserMessages.push(`player ${message.type()}: ${message.text()}`));
  player.on('pageerror', (error) => browserMessages.push(`player pageerror: ${error.message}`));
  await player.goto(servers[0].url);
  await player.evaluate(() => {
    window.__milkrustCollectStats = true;
  });
  await player.getByRole('button', { name: 'Start Demo' }).click();
  await player.waitForFunction(() => window.__milkrustPlayerReady === true, null, {
    timeout: 10_000,
  });
  await player.waitForFunction(() => window.__milkrustPlayerStats?.channelTotal > 0, null, {
    timeout: 10_000,
  });
  await player.waitForFunction(() => (
    Array.from(document.querySelectorAll('#pack-list option'))
      .some((option) => option.textContent === 'Original MilkDrop Preset Pack')
  ), null, {
    timeout: 10_000,
  });
  await player.selectOption('#pack-list', { label: 'Original MilkDrop Preset Pack' });
  await player.waitForFunction(() => document.querySelectorAll('#preset-list option').length > 100, null, {
    timeout: 10_000,
  });
  await player.selectOption('#preset-list', { index: 1 });
  await player.waitForFunction(() => window.__milkrustPlayerStats?.channelTotal > 0, null, {
    timeout: 10_000,
  });
  await player.fill('#playlist-name', 'Smoke Test Playlist');
  await player.getByRole('button', { name: 'Save Playlist' }).click();
  const savedPlaylistValue = await player.$eval('#playlist-list option:not([value=""])', (option) => option.value);
  await player.selectOption('#playlist-list', { value: savedPlaylistValue });
  await player.waitForFunction(
    (value) => document.querySelector('#playlist-list').value === value,
    savedPlaylistValue,
    {
      timeout: 5_000,
    },
  );
  const playlistDownload = player.waitForEvent('download');
  await player.getByRole('button', { name: 'Export Playlists' }).click();
  const download = await playlistDownload;
  const playlistExportPath = await download.path();
  const playlistExport = playlistExportPath ? await readFile(playlistExportPath) : Buffer.from('{"playlists":[],"kind":"milkrust-playlist-export","schemaVersion":1}', 'utf8');
  await player.setInputFiles('#playlist-import', {
    name: 'smoke-playlists.json',
    mimeType: 'application/json',
    buffer: playlistExport,
  });
  await player.getByRole('button', { name: 'Random' }).click();
  await player.getByRole('button', { name: 'Update Playlist' }).click();
  await player.getByRole('button', { name: 'Delete Playlist' }).click();

  const selectedPresetBefore = await player.$eval('#preset-list', (list) => list.value);
  await player.locator('#next').click();
  await player.waitForFunction(
    (value) => document.querySelector('#preset-list')?.value !== value,
    selectedPresetBefore,
    { timeout: 5_000 },
  );
  const selectedPresetAfterNext = await player.$eval('#preset-list', (list) => list.value);
  await player.locator('#previous').click();
  await player.waitForFunction(
    (value) => document.querySelector('#preset-list')?.value === value,
    selectedPresetBefore,
    { timeout: 5_000 },
  );
  const selectedPresetAfterPrev = await player.$eval('#preset-list', (list) => list.value);
  if (selectedPresetAfterPrev !== selectedPresetBefore) {
    throw new Error(`Player history flow failed before preset transition: ${selectedPresetBefore} -> ${selectedPresetAfterPrev}`);
  }

  await player.locator('#next').click();
  await player.waitForFunction(
    (value) => document.querySelector('#preset-list')?.value !== value,
    selectedPresetBefore,
    { timeout: 5_000 },
  );
  await player.locator('#prev-history').click();
  await player.waitForFunction(
    (value) => document.querySelector('#preset-list')?.value === value,
    selectedPresetBefore,
    { timeout: 5_000 },
  );
  const selectedPresetFromHistoryPrev = await player.$eval('#preset-list', (list) => list.value);
  if (selectedPresetFromHistoryPrev !== selectedPresetBefore) {
    throw new Error(`Player history prev failed: expected ${selectedPresetBefore}, got ${selectedPresetFromHistoryPrev}`);
  }
  await player.locator('#next-history').click();
  await player.waitForFunction(
    (value) => document.querySelector('#preset-list')?.value === value,
    selectedPresetAfterNext,
    { timeout: 5_000 },
  );
  const selectedPresetFromHistoryNext = await player.$eval('#preset-list', (list) => list.value);
  if (selectedPresetFromHistoryNext !== selectedPresetAfterNext) {
    throw new Error(`Player history next failed: expected ${selectedPresetAfterNext}, got ${selectedPresetFromHistoryNext}`);
  }

  await player.locator('#toggle-favorite').click();
  await player.waitForFunction(
    () => document.querySelector('#toggle-favorite')?.textContent?.includes('Unfavorite'),
    null,
    { timeout: 2_000 },
  );
  await player.locator('#view-all').click();
  await player.waitForFunction(
    () => document.querySelector('#view-all')?.textContent?.includes('Favorites'),
    null,
    { timeout: 2_000 },
  );
  await player.locator('#view-all').click();
  await player.waitForFunction(
    () => document.querySelector('#view-all')?.textContent?.includes('All'),
    null,
    { timeout: 2_000 },
  );
  await player.locator('#toggle-favorite').click();
  await player.waitForFunction(
    () => document.querySelector('#toggle-favorite')?.textContent?.includes('Favorite'),
    null,
    { timeout: 2_000 },
  );

  const playerStats = await player.evaluate(() => window.__milkrustPlayerStats);
  if (playerStats.channelTotal <= 0 || playerStats.litPixels < playerStats.pixelCount * 0.01) {
    if (browserMessages.length > 0) console.log(browserMessages.join('\n'));
    throw new Error(`MilkRust player smoke rendered a blank canvas: ${JSON.stringify(playerStats)}`);
  }

  const studio = await browser.newPage();
  studio.on('console', (message) => browserMessages.push(`studio ${message.type()}: ${message.text()}`));
  studio.on('pageerror', (error) => browserMessages.push(`studio pageerror: ${error.message}`));
  await studio.goto(servers[1].url);
  await studio.evaluate(() => {
    window.__milkrustCollectStats = true;
  });
  await studio.waitForSelector('#preview');
  await studio.waitForFunction(() => window.__milkrustStudioStats?.channelTotal > 0, null, {
    timeout: 10_000,
  });
  await studio.getByRole('button', { name: 'Inspect' }).click();
  const reportText = await studio.locator('#report').textContent();
  if (!reportText?.includes('inspected')) {
    throw new Error(`MilkRust studio smoke did not inspect preset: ${reportText}`);
  }
  const packDownload = studio.waitForEvent('download');
  await studio.getByRole('button', { name: 'Export Pack' }).click();
  const studioPackDownload = await packDownload;
  const packExportPath = await studioPackDownload.path();
  const studioPackExport = packExportPath
    ? await readFile(packExportPath)
    : Buffer.from('{\"schemaVersion\":1,\"id\":\"studio-smoke\",\"name\":\"Smoke Pack\",\"presets\":[]}', 'utf8');
  await studio.setInputFiles('#pack-file', {
    name: 'smoke-pack.json',
    mimeType: 'application/json',
    buffer: studioPackExport,
  });
  const importedSource = await studio.$eval('#source', (element) => element.value);
  if (!importedSource || !importedSource.includes('name=')) {
    throw new Error(`MilkRust studio smoke did not import pack: ${importedSource}`);
  }
  await studio.selectOption('#parameter', 'zoom');
  await studio.fill('#parameter-value', '1.2');
  const sourceBeforeParameterApply = await studio.$eval('#source', (element) => element.value);
  await studio.getByRole('button', { name: 'Apply' }).click();
  await studio.waitForFunction(
    (previousSource) => document.querySelector('#source')?.value !== previousSource,
    sourceBeforeParameterApply,
    { timeout: 2_000 },
  );
  const sourceAfterParameterApply = await studio.$eval('#source', (element) => element.value);
  if (sourceBeforeParameterApply === sourceAfterParameterApply) {
    throw new Error('MilkRust studio smoke failed parameter apply: source was unchanged');
  }
  const sourceBeforeRandomize = sourceAfterParameterApply;
  await studio.getByRole('button', { name: 'Randomize' }).click();
  await studio.waitForFunction(
    (previousSource) => document.querySelector('#source')?.value !== previousSource,
    sourceBeforeRandomize,
    { timeout: 2_000 },
  );
  const sourceAfterRandomize = await studio.$eval('#source', (element) => element.value);
  if (!sourceAfterRandomize || sourceAfterRandomize === sourceBeforeRandomize) {
    throw new Error('MilkRust studio smoke failed randomize: source was unchanged');
  }
  const studioStats = await studio.evaluate(() => window.__milkrustStudioStats);
  if (studioStats.channelTotal <= 0 || studioStats.litPixels < studioStats.pixelCount * 0.01) {
    if (browserMessages.length > 0) console.log(browserMessages.join('\n'));
    throw new Error(`MilkRust studio smoke rendered a blank canvas: ${JSON.stringify(studioStats)}`);
  }

  const component = await browser.newPage();
  component.on('console', (message) => browserMessages.push(`web-component ${message.type()}: ${message.text()}`));
  component.on('pageerror', (error) => browserMessages.push(`web-component pageerror: ${error.message}`));
  await component.goto(servers[2].url);
  await component.getByRole('button', { name: 'Start synthetic audio' }).click();
  await component.waitForFunction(() => (
    document.querySelector('#status')?.textContent || ''
  ).includes('ready'), { timeout: 10_000 });

  const componentStats = await component.evaluate(async () => {
    const visualizer = document.querySelector('#viz');
    for (let frame = 0; frame < 6; frame += 1) {
      // Render on explicit animation ticks to avoid headless WebGL readPixels races
      // when the browser render loop has not flushed the frame yet.
      if (typeof visualizer?.render === 'function') {
        visualizer.render();
      }
      await new Promise((resolve) => requestAnimationFrame(() => resolve(undefined)));
    }

    const canvas = document.querySelector('#viz canvas');
    if (!canvas) {
      return { channelTotal: 0, litPixels: 0, pixelCount: 0 };
    }
    const gl = canvas.getContext('webgl2');
    if (!gl) {
      return { channelTotal: 0, litPixels: 0, pixelCount: 0 };
    }
    const { width, height } = canvas;
    const pixels = new Uint8Array(width * height * 4);
    gl.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    let channelTotal = 0;
    let litPixels = 0;
    for (let i = 0; i < pixels.length; i += 4) {
      const luminance = pixels[i] + pixels[i + 1] + pixels[i + 2];
      if (luminance > 12) litPixels += 1;
      channelTotal += luminance;
    }
    return { channelTotal, litPixels, pixelCount: width * height };
  });
  if (componentStats.litPixels < componentStats.pixelCount * 0.01 || componentStats.channelTotal <= 0) {
    if (browserMessages.length > 0) console.log(browserMessages.join('\n'));
    throw new Error(`MilkRust web component smoke rendered a blank canvas: ${JSON.stringify(componentStats)}`);
  }

  console.log(`MilkRust app smoke passed: ${
    JSON.stringify({ player: playerStats, studio: studioStats, webComponent: componentStats })
  }`);
} catch (error) {
  if (browserMessages.length > 0) console.log(browserMessages.join('\n'));
  throw error;
} finally {
  await browser.close();
  await Promise.all(servers.map((server) => server.close()));
}
