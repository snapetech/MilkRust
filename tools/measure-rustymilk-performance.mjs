import { spawnSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const result = spawnSync('cargo', [
  'test',
  '-p',
  'rustymilk-core',
  'rustymilk_core_exports_webgpu_batch_summary_json',
  '--',
  '--nocapture',
], {
  cwd: repoRoot,
  stdio: 'inherit',
});

process.exit(result.status ?? 1);

