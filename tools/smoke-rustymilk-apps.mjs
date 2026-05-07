#!/usr/bin/env node
import { chromium } from '@playwright/test';

import { createRustyMilkAppServer } from './serve-rustymilk-app.mjs';

const listen = async (app) => {
  const { appPath, server } = createRustyMilkAppServer({ app });
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
  servers.push(await listen('player'));
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
