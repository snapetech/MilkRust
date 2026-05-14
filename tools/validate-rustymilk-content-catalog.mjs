import fs from 'node:fs/promises';
import path from 'node:path';

const repoRoot = path.resolve(new URL('..', import.meta.url).pathname);
const catalogPath = path.join(repoRoot, 'content/catalog.json');
const allowedCopyPolicies = new Set([
  'include',
  'optional-download',
  'link',
  'review',
  'reject',
  'community-unlicensed',
]);
const vendorableLicenses = new Set([
  '0BSD',
  'Apache-2.0',
  'BSD-2-Clause',
  'BSD-3-Clause',
  'CC-BY-3.0',
  'CC-BY-4.0',
  'CC-BY-SA-3.0',
  'CC-BY-SA-4.0',
  'CC0-1.0',
  'GPL-2.0-only',
  'GPL-2.0-or-later',
  'GPL-3.0-only',
  'GPL-3.0-or-later',
  'ISC',
  'LGPL-2.1-only',
  'LGPL-2.1-or-later',
  'LGPL-3.0-only',
  'LGPL-3.0-or-later',
  'MIT',
  'Unlicense',
  'Zlib',
]);

const catalog = JSON.parse(await fs.readFile(catalogPath, 'utf8'));
const errors = [];
const ids = new Set();

if (catalog.schemaVersion !== 1) {
  errors.push('catalog.schemaVersion must be 1');
}
if (!Array.isArray(catalog.entries)) {
  errors.push('catalog.entries must be an array');
}

for (const [index, entry] of (catalog.entries || []).entries()) {
  const label = entry.id || `entries[${index}]`;
  if (!entry.id) errors.push(`${label}: id is required`);
  if (ids.has(entry.id)) errors.push(`${label}: duplicate id`);
  ids.add(entry.id);
  if (!entry.name) errors.push(`${label}: name is required`);
  if (!entry.kind) errors.push(`${label}: kind is required`);
  if (!entry.status) errors.push(`${label}: status is required`);
  if (!allowedCopyPolicies.has(entry.copyPolicy)) {
    errors.push(`${label}: copyPolicy must be one of ${[...allowedCopyPolicies].join(', ')}`);
  }
  if (!entry.license) errors.push(`${label}: license is required`);
  if (!entry.source?.path && !entry.source?.url) {
    errors.push(`${label}: source.path or source.url is required`);
  }
  if (entry.copyPolicy === 'include' && !vendorableLicenses.has(entry.license)) {
    errors.push(`${label}: included content must use an explicitly vendorable license`);
  }
  if ((entry.copyPolicy === 'include' || entry.copyPolicy === 'community-unlicensed') && entry.source?.path) {
    const sourcePath = path.resolve(repoRoot, entry.source.path);
    try {
      await fs.access(sourcePath);
    } catch {
      errors.push(`${label}: local source path does not exist: ${entry.source.path}`);
    }
  }
  if (
    entry.license === 'NOASSERTION'
    && !['link', 'review', 'community-unlicensed'].includes(entry.copyPolicy)
  ) {
    errors.push(`${label}: NOASSERTION content must be link-only, review, or community-unlicensed`);
  }
  if (entry.copyPolicy === 'community-unlicensed' && entry.defaultBuild !== false) {
    errors.push(`${label}: community-unlicensed content must set defaultBuild to false`);
  }
}

if (errors.length) {
  console.error(errors.join('\n'));
  process.exit(1);
}

console.log(`MilkRust content catalog valid: ${catalog.entries.length} entries`);
