// npm/getBinary.js
const { Binary } = require('binary-install');
const pkg = require('../package.json');

const PLATFORM_MAP = {
  darwin: 'macos',
  linux:  'linux',
  win32:  'windows',
};

const ARCH_MAP = {
  x64:   'x86_64',
  arm64: 'aarch64',
};

function getBinary() {
  const platform = PLATFORM_MAP[process.platform];
  const arch     = ARCH_MAP[process.arch];

  if (!platform || !arch) {
    throw new Error(`Unsupported platform: ${process.platform} ${process.arch}`);
  }

  const version  = pkg.version;  // "0.1.0"
  const name     = `nexus_${version}_${platform}_${arch}.tar.gz`;
  const url      = `https://github.com/GTgyani206/nexus/releases/download/v${version}/${name}`;

  console.log(`Downloading: ${url}`);
  return new Binary('nexus', url);
}

module.exports = getBinary;
