import fs from 'node:fs/promises';
import path from 'node:path';
import { spawn } from 'node:child_process';

const repoRoot = path.resolve(new URL('..', import.meta.url).pathname);
const outputRoot = path.join(repoRoot, 'content/community-unlicensed');
const packs = [
  {
    id: 'projectm-cream-of-the-crop',
    name: 'Milkdrop Cream of the Crop Presets',
    url: 'https://github.com/projectM-visualizer/presets-cream-of-the-crop.git',
    branch: 'master',
  },
  {
    id: 'projectm-milkdrop-original',
    name: 'Original MilkDrop Preset Pack',
    url: 'https://github.com/projectM-visualizer/presets-milkdrop-original.git',
    branch: 'master',
  },
  {
    id: 'projectm-milkdrop-texture-pack',
    name: 'MilkDrop Texture Pack',
    url: 'https://github.com/projectM-visualizer/presets-milkdrop-texture-pack.git',
    branch: 'master',
  },
  {
    id: 'projectm-classic',
    name: 'Classic projectM Preset Pack',
    url: 'https://github.com/projectM-visualizer/presets-projectm-classic.git',
    branch: 'master',
  },
  {
    id: 'projectm-en-d',
    name: 'En D Presets',
    url: 'https://github.com/projectM-visualizer/presets-en-d.git',
    branch: 'master',
  },
];
const presetExtensions = new Set(['.milk', '.milk2']);
const textureExtensions = new Set(['.jpg', '.jpeg', '.png', '.bmp', '.jfif', '.webp', '.tga']);
const excludedExtensions = new Set(['.mp3', '.wav', '.flac', '.ogg', '.m4a']);

const run = (command, args, options = {}) => new Promise((resolve, reject) => {
  const child = spawn(command, args, {
    cwd: repoRoot,
    stdio: 'inherit',
    ...options,
  });
  child.on('exit', (code) => {
    if (code === 0) resolve();
    else reject(new Error(`${command} ${args.join(' ')} exited with ${code}`));
  });
});

const removeIfExists = async (target) => {
  await fs.rm(target, { recursive: true, force: true });
};

const countFiles = async (root, extensionCounts = {}) => {
  let total = 0;
  const entries = await fs.readdir(root, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.name === '.git') continue;
    const fullPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      const child = await countFiles(fullPath, extensionCounts);
      total += child.total;
      continue;
    }
    if (!entry.isFile()) continue;
    total += 1;
    const extension = path.extname(entry.name).toLowerCase() || '<none>';
    extensionCounts[extension] = (extensionCounts[extension] || 0) + 1;
  }
  return { extensionCounts, total };
};

const walkFiles = async (root, files = []) => {
  const entries = await fs.readdir(root, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.name === '.git') continue;
    const fullPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      await walkFiles(fullPath, files);
    } else if (entry.isFile()) {
      files.push(fullPath);
    }
  }
  return files;
};

const pruneExcludedFiles = async (destination) => {
  const files = await walkFiles(destination);
  const excluded = [];
  for (const file of files) {
    const extension = path.extname(file).toLowerCase();
    if (!excludedExtensions.has(extension)) continue;
    await fs.rm(file, { force: true });
    excluded.push(path.relative(destination, file).replaceAll(path.sep, '/'));
  }
  return excluded;
};

const makeId = (relativePath) => relativePath
  .replace(/\.[^.]+$/, '')
  .toLowerCase()
  .replace(/[^a-z0-9]+/g, '-')
  .replace(/^-+|-+$/g, '')
  .slice(0, 160) || 'content';

const titleFromFile = (relativePath) => path.basename(relativePath, path.extname(relativePath));

const writeManifest = async (pack, destination) => {
  const files = await walkFiles(destination);
  const presets = [];
  const textures = [];
  for (const file of files) {
    const relativePath = path.relative(destination, file).replaceAll(path.sep, '/');
    if (relativePath === 'manifest.json' || relativePath === 'RUSTYMILK-COMMUNITY-NOTICE.md') {
      continue;
    }
    const extension = path.extname(file).toLowerCase();
    if (presetExtensions.has(extension)) {
      presets.push({
        id: makeId(relativePath),
        title: titleFromFile(relativePath),
        file: relativePath,
        sourceFormat: extension === '.milk2' ? 'milk2' : 'milk',
        tags: ['community', 'projectm'],
        thumbnail: '',
      });
    } else if (textureExtensions.has(extension)) {
      textures.push({
        id: makeId(relativePath),
        file: relativePath,
        aliases: [
          path.basename(relativePath),
          path.basename(relativePath, extension),
        ],
      });
    }
  }
  presets.sort((left, right) => left.file.localeCompare(right.file));
  textures.sort((left, right) => left.file.localeCompare(right.file));
  const manifest = {
    schemaVersion: 1,
    id: pack.id,
    name: pack.name,
    version: `community-${new Date().toISOString().slice(0, 10)}`,
    author: 'MilkDrop/projectM community',
    description: 'Community-imported public MilkDrop/projectM content. License status is NOASSERTION; see RUSTYMILK-COMMUNITY-NOTICE.md.',
    license: 'NOASSERTION',
    requiredRustyMilkVersion: '0.1.0',
    sourceUrls: [pack.url.replace(/\.git$/, '')],
    presets,
    textures,
    fragments: [],
    plugins: [],
    playlist: presets.map((preset) => preset.id),
    automationDefaults: {
      transitionSeconds: 8,
      randomizeOnBeat: false,
    },
  };
  await fs.writeFile(path.join(destination, 'manifest.json'), `${JSON.stringify(manifest, null, 2)}\n`);
  return { presetCount: presets.length, textureCount: textures.length };
};

const writeNotice = async (pack, destination, counts) => {
  const byExtension = Object.entries(counts.extensionCounts)
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([extension, count]) => `- \`${extension}\`: ${count}`)
    .join('\n');
  const notice = `# RustyMilk Community Content Notice

Pack: ${pack.name}

Source: ${pack.url.replace(/\.git$/, '')}

Branch: ${pack.branch}

This directory contains public community MilkDrop/projectM content imported for RustyMilk compatibility testing and user-accessible preset libraries.

License status: NOASSERTION. The upstream projectM preset repositories document that many historical MilkDrop presets were freely released but did not carry explicit per-author licenses. RustyMilk keeps this content separate from vetted redistributable third-party content.

Default build status: excluded unless a RustyMilk distribution explicitly opts into community-unlicensed content.

Removal/contact policy: if you are an original preset, texture, or asset author and want your work removed from RustyMilk-hosted community content, open an issue or contact the RustyMilk maintainers with the file path(s). The affected files will be removed from future releases.

Imported file count: ${counts.total}

Imported file types:

${byExtension || '- none'}
`;
  await fs.writeFile(path.join(destination, 'RUSTYMILK-COMMUNITY-NOTICE.md'), notice);
};

const main = async () => {
  await fs.mkdir(outputRoot, { recursive: true });
  const results = [];
  for (const pack of packs) {
    const destination = path.join(outputRoot, pack.id);
    await removeIfExists(destination);
    await run('git', [
      'clone',
      '--depth',
      '1',
      '--branch',
      pack.branch,
      pack.url,
      destination,
    ]);
    await removeIfExists(path.join(destination, '.git'));
    const excluded = await pruneExcludedFiles(destination);
    const manifestCounts = await writeManifest(pack, destination);
    const counts = await countFiles(destination);
    await writeNotice(pack, destination, counts);
    if (excluded.length) {
      await fs.appendFile(
        path.join(destination, 'RUSTYMILK-COMMUNITY-NOTICE.md'),
        `\nExcluded during RustyMilk import:\n\n${excluded.map((file) => `- \`${file}\`: file type outside the preset/texture/shader import scope.`).join('\n')}\n`,
      );
    }
    results.push({ ...pack, counts, manifestCounts });
  }
  console.log(JSON.stringify({
    outputRoot: path.relative(repoRoot, outputRoot),
    packs: results.map((pack) => ({
      id: pack.id,
      files: pack.counts.total,
      byExtension: pack.counts.extensionCounts,
      presets: pack.manifestCounts.presetCount,
      textures: pack.manifestCounts.textureCount,
    })),
  }, null, 2));
};

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
