#!/usr/bin/env node
import fs from 'node:fs/promises';
import path from 'node:path';
import { spawn } from 'node:child_process';

import { createRustyMilkAppServer } from './serve-rustymilk-app.mjs';

const repoRoot = path.resolve(new URL('..', import.meta.url).pathname);
const errors = [];

const run = (command, args) => new Promise((resolve, reject) => {
  const child = spawn(command, args, {
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
      resolve({ stdout, stderr });
    } else {
      reject(new Error(`${command} ${args.join(' ')} exited with ${code}: ${stderr || stdout}`));
    }
  });
});

const readJson = async (file) => JSON.parse(await fs.readFile(path.join(repoRoot, file), 'utf8'));

const exists = async (file) => {
  try {
    await fs.access(path.join(repoRoot, file));
    return true;
  } catch {
    return false;
  }
};

const assert = (condition, message) => {
  if (!condition) errors.push(message);
};

const listFiles = async (root, files = []) => {
  const entries = await fs.readdir(root, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      await listFiles(fullPath, files);
    } else if (entry.isFile()) {
      files.push(path.relative(repoRoot, fullPath));
    }
  }
  return files;
};

const auditPackageMetadata = async () => {
  const rootPackage = await readJson('package.json');
  const webPackage = await readJson('packages/rustymilk-web/package.json');
  const lock = await readJson('package-lock.json');

  assert(rootPackage.license === 'AGPL-3.0-only', 'root package.json must declare AGPL-3.0-only');
  assert(webPackage.license === 'AGPL-3.0-only', 'packages/rustymilk-web/package.json must declare AGPL-3.0-only');
  assert(lock.packages?.['']?.license === 'AGPL-3.0-only', 'package-lock root license must be AGPL-3.0-only');
  assert(rootPackage.files?.includes('LICENSE'), 'npm files must include LICENSE');
  assert(rootPackage.files?.includes('LICENSE-SCOPE.md'), 'npm files must include LICENSE-SCOPE.md');
  assert(await exists('LICENSE'), 'LICENSE must exist');
  assert(await exists('LICENSE-SCOPE.md'), 'LICENSE-SCOPE.md must exist');
};

const auditCargoMetadata = async () => {
  const { stdout } = await run('cargo', ['metadata', '--no-deps', '--format-version', '1']);
  const metadata = JSON.parse(stdout);
  for (const pkg of metadata.packages || []) {
    if (String(pkg.manifest_path || '').includes('/crates/rustymilk-')) {
      assert(pkg.license === 'AGPL-3.0-only', `${pkg.name} Cargo license must be AGPL-3.0-only`);
    }
  }
};

const auditNpmPack = async () => {
  const { stdout } = await run('npm', ['pack', '--dry-run', '--json']);
  const [pack] = JSON.parse(stdout);
  const files = (pack?.files || []).map((file) => file.path);
  assert(files.includes('LICENSE'), 'npm package must include LICENSE');
  assert(files.includes('LICENSE-SCOPE.md'), 'npm package must include LICENSE-SCOPE.md');
  assert(
    !files.some((file) => file.startsWith('content/community-unlicensed/')),
    'npm package must not include content/community-unlicensed',
  );
  assert(
    !files.some((file) => file.startsWith('content/third-party/')),
    'npm package must not include vendored content unless explicitly added to package files',
  );
};

const auditContentPolicy = async () => {
  await run('node', ['./tools/validate-rustymilk-content-catalog.mjs']);

  const catalog = await readJson('content/catalog.json');
  const communityEntries = catalog.entries.filter((entry) => entry.copyPolicy === 'community-unlicensed');
  for (const entry of communityEntries) {
    assert(entry.license === 'NOASSERTION', `${entry.id}: community-unlicensed catalog entry must be NOASSERTION`);
    assert(entry.defaultBuild === false, `${entry.id}: community-unlicensed catalog entry must set defaultBuild false`);
    assert(entry.source?.path?.startsWith('content/community-unlicensed/'), `${entry.id}: community path must stay under content/community-unlicensed`);
    assert(await exists(path.join(entry.source.path, 'manifest.json')), `${entry.id}: community manifest is missing`);
    assert(await exists(path.join(entry.source.path, 'RUSTYMILK-COMMUNITY-NOTICE.md')), `${entry.id}: community notice is missing`);
    const manifest = await readJson(path.join(entry.source.path, 'manifest.json'));
    assert(manifest.license === 'NOASSERTION', `${entry.id}: community manifest license must be NOASSERTION`);
    assert(
      String(manifest.description || '').includes('RUSTYMILK-COMMUNITY-NOTICE.md'),
      `${entry.id}: community manifest should point to the community notice`,
    );
  }

  const audioExtensions = new Set(['.aac', '.flac', '.m4a', '.mp3', '.ogg', '.wav', '.wma']);
  const communityRoot = path.join(repoRoot, 'content/community-unlicensed');
  const communityFiles = await listFiles(communityRoot);
  for (const file of communityFiles) {
    assert(!audioExtensions.has(path.extname(file).toLowerCase()), `community-unlicensed must not include audio: ${file}`);
  }

  const thirdPartyEntries = catalog.entries.filter((entry) =>
    entry.copyPolicy === 'include' && entry.source?.path?.startsWith('content/third-party/'));
  for (const entry of thirdPartyEntries) {
    assert(entry.license !== 'NOASSERTION', `${entry.id}: included third-party content must have an explicit license`);
    assert(await exists(path.join(entry.source.path, 'RUSTYMILK-PROVENANCE.md')), `${entry.id}: third-party provenance is missing`);
  }
};

const auditDefaultServer = async () => {
  const { server } = createRustyMilkAppServer({ app: 'player' });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const { port } = server.address();
  try {
    const response = await fetch(`http://127.0.0.1:${port}/content/generated/community-pack-summary.json`);
    assert(response.status === 404, 'default app server must not serve community pack summary');
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
};

await auditPackageMetadata();
await auditCargoMetadata();
await auditNpmPack();
await auditContentPolicy();
await auditDefaultServer();

if (errors.length) {
  console.error(errors.join('\n'));
  process.exit(1);
}

console.log('RustyMilk compliance audit passed');
