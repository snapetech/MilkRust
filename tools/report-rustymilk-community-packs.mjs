import fs from 'node:fs/promises';
import path from 'node:path';
import { spawn } from 'node:child_process';

const repoRoot = path.resolve(new URL('..', import.meta.url).pathname);
const communityRoot = path.join(repoRoot, 'content/community-unlicensed');
const outputPath = path.join(repoRoot, 'content/generated/community-pack-summary.json');
const markdownOutputPath = path.join(repoRoot, 'content/generated/COMMUNITY_PACK_SUMMARY.md');
const args = process.argv.slice(2);
const write = args.includes('--write');
const compat = args.includes('--compat');
const limitArg = args.find((arg) => arg.startsWith('--limit='));
const limit = limitArg ? Number(limitArg.split('=')[1]) : 0;

const runJson = (command, commandArgs) => new Promise((resolve, reject) => {
  const child = spawn(command, commandArgs, {
    cwd: repoRoot,
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  let stdout = '';
  let stderr = '';
  child.stdout.on('data', (data) => {
    stdout += data;
  });
  child.stderr.on('data', (data) => {
    stderr += data;
  });
  child.on('exit', (code) => {
    if (code === 0) {
      try {
        resolve(JSON.parse(stdout));
      } catch (error) {
        reject(new Error(`failed to parse JSON from ${command}: ${error.message}`));
      }
    } else {
      reject(new Error(`${command} ${commandArgs.join(' ')} exited with ${code}: ${stderr || stdout}`));
    }
  });
});

const readJson = async (file) => JSON.parse(await fs.readFile(file, 'utf8'));

const listPackDirs = async () => {
  const entries = await fs.readdir(communityRoot, { withFileTypes: true });
  return entries
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join(communityRoot, entry.name))
    .sort();
};

const extensionCounts = async (root, counts = {}) => {
  const entries = await fs.readdir(root, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      await extensionCounts(fullPath, counts);
    } else if (entry.isFile()) {
      const extension = path.extname(entry.name).toLowerCase() || '<none>';
      counts[extension] = (counts[extension] || 0) + 1;
    }
  }
  return counts;
};

const writeCompatSubset = async (packDir, manifest, maxFiles) => {
  if (!maxFiles || !manifest.presets.length) return null;
  const subsetDir = await fs.mkdtemp(path.join('/tmp', 'rustymilk-community-compat-'));
  try {
    const selected = manifest.presets.slice(0, maxFiles);
    for (const preset of selected) {
      const source = path.join(packDir, preset.file);
      const target = path.join(subsetDir, `${preset.id}${path.extname(preset.file) || '.milk'}`);
      await fs.copyFile(source, target);
    }
    const report = await runJson('cargo', [
      'run',
      '-q',
      '-p',
      'rustymilk-cli',
      '--',
      'compat',
      subsetDir,
    ]);
    return {
      limit: maxFiles,
      presetCount: report.presetCount,
      supportedCount: report.supportedCount,
      unsupportedCount: report.unsupportedCount,
      webGpuSupportedCount: report.webGpuSupportedCount,
      webGpuUnsupportedCount: report.webGpuUnsupportedCount,
      unsupportedFunctions: report.unsupportedFunctions,
      unsupportedShaderSections: report.unsupportedShaderSections,
      webGpuUnsupportedShaderSections: report.webGpuUnsupportedShaderSections,
    };
  } finally {
    await fs.rm(subsetDir, { recursive: true, force: true });
  }
};

const main = async () => {
  const packDirs = await listPackDirs();
  const packs = [];
  for (const packDir of packDirs) {
    const manifestPath = path.join(packDir, 'manifest.json');
    const manifest = await readJson(manifestPath);
    const counts = await extensionCounts(packDir);
    const pack = {
      id: manifest.id,
      name: manifest.name,
      path: path.relative(repoRoot, packDir),
      license: manifest.license,
      sourceUrls: manifest.sourceUrls || [],
      presetCount: manifest.presets?.length || 0,
      textureCount: manifest.textures?.length || 0,
      playlistCount: manifest.playlist?.length || 0,
      byExtension: Object.fromEntries(Object.entries(counts).sort(([left], [right]) => left.localeCompare(right))),
    };
    if (compat) {
      pack.compatibility = await writeCompatSubset(packDir, manifest, limit);
    }
    packs.push(pack);
  }

  const summary = {
    generatedAt: new Date().toISOString(),
    packCount: packs.length,
    totalPresets: packs.reduce((total, pack) => total + pack.presetCount, 0),
    totalTextures: packs.reduce((total, pack) => total + pack.textureCount, 0),
    compatibilityLimit: compat ? limit : 0,
    packs,
  };

  if (write) {
    await fs.mkdir(path.dirname(outputPath), { recursive: true });
    await fs.writeFile(outputPath, `${JSON.stringify(summary, null, 2)}\n`);
    await fs.writeFile(markdownOutputPath, renderMarkdown(summary));
  }
  process.stdout.write(`${JSON.stringify(summary, null, 2)}\n`);
};

const renderList = (values = []) => values.length
  ? values.map((value) => `\`${value}\``).join(', ')
  : '';

function renderMarkdown(summary) {
  const rows = summary.packs.map((pack) => {
    const compatibility = pack.compatibility
      ? `${pack.compatibility.supportedCount}/${pack.compatibility.presetCount} sampled supported`
      : 'not sampled';
    const blockers = pack.compatibility
      ? renderList([
        ...pack.compatibility.unsupportedFunctions,
        ...pack.compatibility.unsupportedShaderSections,
        ...pack.compatibility.webGpuUnsupportedShaderSections,
      ])
      : '';
    return `| ${pack.id} | ${pack.presetCount} | ${pack.textureCount} | ${compatibility} | ${blockers || 'none'} |`;
  }).join('\n');

  return `# Community Pack Summary

Generated by \`node tools/report-rustymilk-community-packs.mjs\`.

Community-unlicensed packs are imported into \`content/community-unlicensed\` for compatibility work and opt-in builds. They are excluded from default builds by catalog policy.

## Totals

- Packs: ${summary.packCount}
- Presets: ${summary.totalPresets}
- Textures: ${summary.totalTextures}
- Compatibility sample size: ${summary.compatibilityLimit || 'not sampled'}

## Packs

| Pack | Presets | Textures | Compatibility | Blockers |
| --- | ---: | ---: | --- | --- |
${rows}
`;
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
