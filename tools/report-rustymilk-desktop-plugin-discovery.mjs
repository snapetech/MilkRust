#!/usr/bin/env node
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const argv = process.argv.slice(2);

const outputArgIndex = argv.findIndex((value) => value === '--output' || value.startsWith('--output='));
const outputDirectory = outputArgIndex >= 0
  ? (() => {
    const explicit = argv[outputArgIndex].startsWith('--output=')
      ? argv[outputArgIndex].slice('--output='.length)
      : argv[outputArgIndex + 1];
    if (!explicit) {
      throw new Error('--output requires a directory path');
    }
    return resolve(repoRoot, explicit);
  })()
  : join(repoRoot, 'content/generated/desktop-plugin-reports');

const includeCommunity = argv.includes('--include-community');
const writeAllFromCatalog = argv.includes('--catalog') || argv.includes('--include-catalog');
const includeNoAssertion = argv.includes('--include-no-assertion');
const writeMarkdown = argv.includes('--write-markdown');
const rawFrameCount = (() => {
  const value = argv.find((value) => value.startsWith('--frames='));
  if (value) {
    return Number.parseInt(value.slice('--frames='.length), 10);
  }
  const index = argv.indexOf('--frames');
  return index >= 0 && argv[index + 1] ? Number.parseInt(argv[index + 1], 10) : 4;
})();
const frameCountArg = Number.isNaN(rawFrameCount) || rawFrameCount <= 0 ? 4 : rawFrameCount;

const explicitPacks = argv
  .map((value, index) => {
    if (value === '--pack') {
      return argv[index + 1];
    }
    if (value.startsWith('--pack=')) {
      return value.slice('--pack='.length);
    }
    return null;
  })
  .filter((value) => value);

const sanitizeName = (value) =>
  value
    .replace(/^[./\\]+/, '')
    .replace(/[\\/]+/g, '__')
    .replace(/[^a-zA-Z0-9._-]/g, '_');

const readCatalogPacks = () => {
  if (!writeAllFromCatalog) {
    return [];
  }
  const catalogPath = join(repoRoot, 'content', 'catalog.json');
  const catalog = JSON.parse(readFileSync(catalogPath, 'utf8'));
  if (!Array.isArray(catalog.entries)) {
    return [];
  }
  return catalog.entries
    .filter((entry) => {
      if (typeof entry !== 'object' || entry === null) {
        return false;
      }
      const source = entry.source;
      if (!source || source.type !== 'local' || typeof source.path !== 'string') {
        return false;
      }
      const copyPolicy = entry.copyPolicy || 'link';
      if (copyPolicy === 'include') {
        return true;
      }
      if (includeCommunity && copyPolicy === 'community-unlicensed') {
        return true;
      }
      if (includeNoAssertion && copyPolicy === 'review') {
        return true;
      }
      return false;
    })
    .map((entry) => entry.source.path);
};

const selectedPacks = explicitPacks.length > 0 ? explicitPacks : readCatalogPacks();
const candidatePacks = selectedPacks.length > 0
  ? selectedPacks
  : [join('examples', 'sample-pack')];

if (candidatePacks.length === 0) {
  throw new Error('No pack candidates selected. Pass --pack or --catalog paths.');
}

const ensureDirectory = (directory) => {
  mkdirSync(directory, { recursive: true });
};

const runCommand = (command, args, reportPath, modeLabel) => {
  const result = spawnSync('cargo', args, {
    cwd: repoRoot,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });

  if (result.status !== 0) {
    const stderr = (result.stderr || '').trim();
    const stdout = (result.stdout || '').trim();
    throw new Error(
      `desktop ${modeLabel} command failed (${result.status}). ${command} ${args.join(' ')}\n${stdout}\n${stderr}`
    );
  }

  const report = JSON.parse(readFileSync(reportPath, 'utf8'));
  return {
    mode: modeLabel,
    pluginCount: report.packPlugins?.count ?? 0,
    report,
  };
};

const runPack = (packPath) => {
  const safeName = sanitizeName(packPath);
  const playerReportPath = join(outputDirectory, `player-${safeName}.json`);
  const probeReportPath = join(outputDirectory, `probe-${safeName}.json`);
  const studioReportPath = join(outputDirectory, `studio-${safeName}.json`);

  return {
    pack: packPath,
    player: runCommand(
      'player',
      [
        'run',
        '-p',
        'milkrust-desktop',
        '--bin',
        'player',
        '--',
        '--pack',
        packPath,
        '--frames',
        String(frameCountArg),
        '--plugin-report',
        playerReportPath,
        '--json',
      ],
      playerReportPath,
      'player',
    ),
    probe: runCommand(
      'probe',
      [
        'run',
        '-p',
        'milkrust-desktop',
        '--bin',
        'milkrust-desktop',
        '--',
        '--pack',
        packPath,
        '--frames',
        String(frameCountArg),
        '--fps',
        '30',
        '--plugin-report',
        probeReportPath,
        '--json',
      ],
      probeReportPath,
      'probe',
    ),
    studio: runCommand(
      'studio',
      [
        'run',
        '-p',
        'milkrust-desktop',
        '--bin',
        'studio',
        '--',
        '--pack',
        packPath,
        '--plugin-report',
        studioReportPath,
        '--json',
      ],
      studioReportPath,
      'studio',
    ),
    reports: {
      player: playerReportPath,
      probe: probeReportPath,
      studio: studioReportPath,
    },
  };
};

ensureDirectory(outputDirectory);

const packReports = candidatePacks.map((packPath) => runPack(packPath));

const aggregateReport = {
  generatedAt: new Date().toISOString(),
  includeCommunity,
  includeNoAssertion,
  frameCount: frameCountArg,
  packs: packReports,
};

const outputPath = join(outputDirectory, 'desktop-plugin-discovery-summary.json');
writeFileSync(outputPath, JSON.stringify(aggregateReport, null, 2));

if (writeMarkdown) {
  const markdownPath = join(outputDirectory, 'desktop-plugin-discovery-summary.md');
  const rows = packReports.flatMap((packReport) => {
    const entries = (packReport.player.report?.packPlugins?.entries || []);
    const pluginIds = entries.map((entry) => entry.id).filter(Boolean).sort();
    const label = pluginIds.length > 0 ? pluginIds.join(', ') : 'none';

    return [
      `- **${packReport.pack}**`,
      `  - player plugins: ${packReport.player.pluginCount}`,
      `  - probe plugins: ${packReport.probe.pluginCount}`,
      `  - studio plugins: ${packReport.studio.pluginCount}`,
      `  - plugin ids: ${label}`,
      '',
    ];
  });

  const markdown = [
    '# Desktop Plugin Discovery Report',
    '',
    `Generated: ${aggregateReport.generatedAt}`,
    `Frame count: ${frameCountArg}`,
    `Include community packs: ${includeCommunity ? 'yes' : 'no'}`,
    `Include review/no-assertion packs: ${includeNoAssertion ? 'yes' : 'no'}`,
    '',
    '## Pack Summary',
    '',
    ...rows,
  ].join('\n');

  writeFileSync(markdownPath, markdown);
}

const counts = packReports.map((pack) => ({
  pack: pack.pack,
  player: pack.player.pluginCount,
  probe: pack.probe.pluginCount,
  studio: pack.studio.pluginCount,
}));
console.log(JSON.stringify({
  outputDirectory,
  summaryPath: outputPath,
  counts,
}, null, 2));
