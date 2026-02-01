#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const version = process.argv[2];

if (!version) {
  console.error('Usage: node update-version.js <version>');
  process.exit(1);
}

console.log(`Updating version to ${version}`);

// Update main package.json
const mainPkgPath = path.join(__dirname, '..', 'package.json');
const mainPkg = JSON.parse(fs.readFileSync(mainPkgPath, 'utf8'));
mainPkg.version = version;

// Update optionalDependencies versions
for (const dep of Object.keys(mainPkg.optionalDependencies || {})) {
  mainPkg.optionalDependencies[dep] = version;
}
fs.writeFileSync(mainPkgPath, JSON.stringify(mainPkg, null, 2) + '\n');
console.log(`Updated ${mainPkgPath}`);

// Update platform package.json files
const platforms = ['darwin-arm64', 'darwin-x64', 'linux-x64', 'linux-arm64', 'win32-x64'];
for (const platform of platforms) {
  const pkgPath = path.join(__dirname, '..', 'npm', platform, 'package.json');
  if (fs.existsSync(pkgPath)) {
    const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
    pkg.version = version;
    fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n');
    console.log(`Updated ${pkgPath}`);
  }
}

// Update Cargo.toml
const cargoPath = path.join(__dirname, '..', 'Cargo.toml');
if (fs.existsSync(cargoPath)) {
  let cargo = fs.readFileSync(cargoPath, 'utf8');
  cargo = cargo.replace(/^version\s*=\s*"[^"]*"/m, `version = "${version}"`);
  fs.writeFileSync(cargoPath, cargo);
  console.log(`Updated ${cargoPath}`);
}

// Update package-lock.json to reflect new optionalDependencies versions
const { execSync } = require('child_process');
console.log('Updating package-lock.json...');
execSync('npm install --package-lock-only --ignore-scripts', {
  cwd: path.join(__dirname, '..'),
  stdio: 'inherit'
});
console.log('Updated package-lock.json');

console.log('Version update complete');
