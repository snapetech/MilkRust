#!/usr/bin/env node
import { createReadStream, existsSync } from 'node:fs';
import { createServer } from 'node:http';
import { extname, join, normalize, resolve } from 'node:path';

const repoRoot = resolve(import.meta.dirname, '..');
const appName = process.argv[2] === 'studio' ? 'rustymilk-studio' : 'rustymilk-player';
const appPath = `/apps/${appName}/`;
const port = Number(process.env.PORT || 4173);

const mimeTypes = {
  '.css': 'text/css; charset=utf-8',
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.wasm': 'application/wasm',
};

const sendFile = (response, filePath) => {
  response.writeHead(200, {
    'Content-Type': mimeTypes[extname(filePath)] || 'application/octet-stream',
  });
  createReadStream(filePath).pipe(response);
};

const server = createServer((request, response) => {
  const url = new URL(request.url || '/', `http://${request.headers.host || 'localhost'}`);
  if (url.pathname === '/') {
    response.writeHead(302, { Location: appPath });
    response.end();
    return;
  }

  const pathname = decodeURIComponent(url.pathname);
  const relativePath = pathname === appPath
    ? join('apps', appName, 'index.html')
    : pathname.replace(/^\/+/, '');
  const filePath = normalize(resolve(repoRoot, relativePath));

  if (!filePath.startsWith(repoRoot) || !existsSync(filePath)) {
    response.writeHead(404, { 'Content-Type': 'text/plain; charset=utf-8' });
    response.end('Not found');
    return;
  }

  sendFile(response, filePath);
});

server.listen(port, '127.0.0.1', () => {
  console.log(`RustyMilk ${appName.replace('rustymilk-', '')} running at http://127.0.0.1:${port}${appPath}`);
});

