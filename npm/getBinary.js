const { Binary } = require("binary-install");
const pkg = require("../package.json");

const PLATFORM_MAP = {
  darwin: "macos",
  linux: "linux",
  win32: "windows",
};

const ARCH_MAP = {
  x64: "x86_64",
  arm64: "aarch64",
  ia32: "i686",
};

function getPlatform() {
  const platform = PLATFORM_MAP[process.platform];
  const arch = ARCH_MAP[process.arch];

  if (!platform || !arch) {
    throw new Error(`Unsupported platform: ${process.platform} ${process.arch}`);
  }

  return { platform, arch };
}

function getBinary() {
  const { platform, arch } = getPlatform();
  const version = pkg.version;
  const url = pkg.binary.url
    .replace("{{version}}", version)
    .replace("{{platform}}", platform)
    .replace("{{arch}}", arch);

  return new Binary(pkg.binary.name, url);
}

module.exports = getBinary;
