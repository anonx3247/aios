import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const outputDir = path.resolve(__dirname, '../../desktop/src-tauri/binaries');
const backendDir = path.resolve(__dirname, '..');

const targets = [
  { pkg: 'node18-macos-arm64', suffix: 'aarch64-apple-darwin' },
];

fs.mkdirSync(outputDir, { recursive: true });

for (const { pkg, suffix } of targets) {
  console.log(`Building for ${pkg}...`);
  execSync(
    `npx pkg dist/bundle.cjs --target ${pkg} --output ${outputDir}/backend-${suffix}`,
    { stdio: 'inherit', cwd: backendDir }
  );
  fs.chmodSync(`${outputDir}/backend-${suffix}`, 0o755);
  console.log(`Created backend-${suffix}`);
}
