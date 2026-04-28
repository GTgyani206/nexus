const fs = require('fs');
const http = require('http');
const https = require('https');
const path = require('path');
const zlib = require('zlib');
const { spawnSync } = require('child_process');
const pkg = require('../package.json');

const PLATFORM_MAP = { darwin: 'macos', linux: 'linux', win32: 'windows' };
const ARCH_MAP = { x64: 'x86_64', arm64: 'aarch64' };
const REPO = 'GTgyani206/nexus';

function getInfo() {
  const platform = PLATFORM_MAP[process.platform];
  const arch = ARCH_MAP[process.arch];
  if (!platform || !arch) {
    throw new Error('Unsupported platform: ' + process.platform + ' ' + process.arch);
  }
  const version = (process.env.NEXUS_BINARY_VERSION || pkg.nexusBinaryVersion || pkg.version).replace(/^v/, '');
  const name = 'nexus_' + version + '_' + platform + '_' + arch + '.tar.gz';
  const baseUrl = (process.env.NEXUS_BINARY_BASE_URL || ('https://github.com/' + REPO + '/releases/download/v' + version)).replace(/\/$/, '');
  const url = process.env.NEXUS_BINARY_URL || (baseUrl + '/' + name);
  const binDir = path.join(__dirname, '..', 'bin');
  const binName = process.platform === 'win32' ? 'nexus.exe' : 'nexus';
  const binPath = path.join(binDir, binName);
  return { url, binDir, binName, binPath };
}

function download(url, cb, redirects) {
  redirects = redirects || 0;
  if (redirects > 5) {
    cb(new Error('Too many redirects while downloading ' + url));
    return;
  }

  let parsed;
  try {
    parsed = new URL(url);
  } catch (err) {
    cb(err);
    return;
  }

  if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
    cb(new Error('Unsupported URL protocol: ' + parsed.protocol));
    return;
  }

  const client = parsed.protocol === 'http:' ? http : https;
  const req = client.get(parsed, { headers: { 'User-Agent': 'nexus-install' } }, function(res) {
    if ([301, 302, 303, 307, 308].includes(res.statusCode)) {
      const location = res.headers.location;
      res.resume();
      if (!location) {
        cb(new Error('Redirect without a location while downloading ' + url));
        return;
      }
      download(new URL(location, parsed).toString(), cb, redirects + 1);
      return;
    }

    if (res.statusCode !== 200) {
      res.resume();
      cb(new Error('HTTP ' + res.statusCode + ' while downloading ' + url));
      return;
    }

    const chunks = [];
    res.on('data', function(chunk) { chunks.push(chunk); });
    res.on('end', function() { cb(null, Buffer.concat(chunks)); });
    res.on('error', cb);
  });

  req.on('error', cb);
}

function readTarString(block, start, end) {
  return block.subarray(start, end).toString('utf8').replace(/\0.*$/, '').trim();
}

function readTarSize(block) {
  const raw = readTarString(block, 124, 136);
  return raw ? parseInt(raw, 8) : 0;
}

function isEmptyTarBlock(block) {
  for (let i = 0; i < block.length; i++) {
    if (block[i] !== 0) return false;
  }
  return true;
}

function tarEntryName(block) {
  const name = readTarString(block, 0, 100);
  const prefix = readTarString(block, 345, 500);
  return prefix ? prefix + '/' + name : name;
}

function extractBinary(buffer, info) {
  const tar = zlib.gunzipSync(buffer);
  let offset = 0;
  let fallback = null;
  const seenFiles = [];

  while (offset + 512 <= tar.length) {
    const header = tar.subarray(offset, offset + 512);
    if (isEmptyTarBlock(header)) break;

    const name = tarEntryName(header);
    const size = readTarSize(header);
    const type = String.fromCharCode(header[156] || 0);
    const dataStart = offset + 512;
    const dataEnd = dataStart + size;

    if (dataEnd > tar.length) {
      throw new Error('Corrupt tar archive: entry ' + name + ' is truncated');
    }

    if (type === '0' || type === '\0' || type === '') {
      const baseName = path.basename(name.replace(/\\/g, '/'));
      const entry = { name: name, data: tar.subarray(dataStart, dataEnd) };
      seenFiles.push(name);
      if (baseName === info.binName) {
        fallback = entry;
        break;
      }
      if (!fallback) fallback = entry;
    }

    offset = dataStart + Math.ceil(size / 512) * 512;
  }

  if (!fallback) {
    throw new Error('No binary file found in archive. Files seen: ' + (seenFiles.join(', ') || 'none'));
  }

  fs.mkdirSync(info.binDir, { recursive: true });
  fs.writeFileSync(info.binPath, fallback.data, { mode: 0o755 });
  try {
    fs.chmodSync(info.binPath, 0o755);
  } catch (err) {
    if (process.platform !== 'win32') throw err;
  }
}

function install() {
  const info = getInfo();
  console.log('Downloading:', info.url);

  download(info.url, function(err, buffer) {
    if (err) {
      console.error('Download failed:', err.message);
      process.exit(1);
    }

    try {
      extractBinary(buffer, info);
      console.log('Installed to:', info.binPath);
    } catch (extractErr) {
      console.error('Extraction failed:', extractErr.message);
      process.exit(1);
    }
  });
}

function run() {
  const info = getInfo();
  if (!fs.existsSync(info.binPath)) {
    console.error('nexus binary not found at ' + info.binPath + '. Re-run: npm install -g nexus-lang');
    process.exit(1);
  }
  const result = spawnSync(info.binPath, process.argv.slice(2), { stdio: 'inherit' });
  if (result.error) {
    console.error(result.error.message);
    process.exit(1);
  }
  process.exit(result.status !== null ? result.status : 1);
}

module.exports = { install: install, run: run };
