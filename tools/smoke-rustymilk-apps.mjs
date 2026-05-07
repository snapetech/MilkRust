#!/usr/bin/env node
import { chromium } from '@playwright/test';
import { readFile } from 'node:fs/promises';

import { createRustyMilkAppServer } from './serve-rustymilk-app.mjs';

const listen = async (app, options = {}) => {
  const { appPath, server } = createRustyMilkAppServer({ app, ...options });
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

  const player = await browser.newPage();
  player.on('console', (message) => browserMessages.push(`player ${message.type()}: ${message.text()}`));
  player.on('pageerror', (error) => browserMessages.push(`player pageerror: ${error.message}`));
  await player.goto(servers[0].url);
  await player.evaluate(() => {
    window.__rustyMilkCollectStats = true;
  });
  await player.getByRole('button', { name: 'Start Demo' }).click();
  await player.waitForFunction(() => window.__rustyMilkPlayerReady === true, null, {
    timeout: 10_000,
  });
  await player.waitForFunction(() => window.__rustyMilkPlayerStats?.channelTotal > 0, null, {
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
  await player.waitForFunction(() => window.__rustyMilkPlayerStats?.channelTotal > 0, null, {
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
  const playlistExport = playlistExportPath ? await readFile(playlistExportPath) : Buffer.from('{"playlists":[],"kind":"rustymilk-playlist-export","schemaVersion":1}', 'utf8');
  await player.setInputFiles('#playlist-import', {
    name: 'smoke-playlists.json',
    mimeType: 'application/json',
    buffer: playlistExport,
  });
  await player.getByRole('button', { name: 'Random' }).click();
  await player.getByRole('button', { name: 'Update Playlist' }).click();
  await player.getByRole('button', { name: 'Delete Playlist' }).click();
  const playerStats = await player.evaluate(() => window.__rustyMilkPlayerStats);
  if (playerStats.channelTotal <= 0 || playerStats.litPixels < playerStats.pixelCount * 0.01) {
    if (browserMessages.length > 0) console.log(browserMessages.join('\n'));
    throw new Error(`RustyMilk player smoke rendered a blank canvas: ${JSON.stringify(playerStats)}`);
  }

  const studio = await browser.newPage();
  studio.on('console', (message) => browserMessages.push(`studio ${message.type()}: ${message.text()}`));
  studio.on('pageerror', (error) => browserMessages.push(`studio pageerror: ${error.message}`));
  await studio.goto(servers[1].url);
  await studio.evaluate(() => {
    window.__rustyMilkCollectStats = true;
  });
  await studio.waitForSelector('#preview');
  await studio.waitForFunction(() => window.__rustyMilkStudioStats?.channelTotal > 0, null, {
    timeout: 10_000,
  });
  await studio.getByRole('button', { name: 'Inspect' }).click();
  const reportText = await studio.locator('#report').textContent();
  if (!reportText?.includes('inspected')) {
    throw new Error(`RustyMilk studio smoke did not inspect preset: ${reportText}`);
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
    throw new Error(`RustyMilk studio smoke did not import pack: ${importedSource}`);
  }
  const studioStats = await studio.evaluate(() => window.__rustyMilkStudioStats);
  if (studioStats.channelTotal <= 0 || studioStats.litPixels < studioStats.pixelCount * 0.01) {
    if (browserMessages.length > 0) console.log(browserMessages.join('\n'));
    throw new Error(`RustyMilk studio smoke rendered a blank canvas: ${JSON.stringify(studioStats)}`);
  }

  console.log(`RustyMilk app smoke passed: ${JSON.stringify({ player: playerStats, studio: studioStats })}`);
} catch (error) {
  if (browserMessages.length > 0) console.log(browserMessages.join('\n'));
  throw error;
} finally {
  await browser.close();
  await Promise.all(servers.map((server) => server.close()));
}
