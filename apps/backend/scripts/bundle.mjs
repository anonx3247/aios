import * as esbuild from 'esbuild';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const backendDir = path.resolve(__dirname, '..');

await esbuild.build({
  entryPoints: [path.join(backendDir, 'src/index.ts')],
  bundle: true,
  platform: 'node',
  target: 'node18',
  format: 'cjs',
  outfile: path.join(backendDir, 'dist/bundle.cjs'),
  external: [],
  minify: false,
  sourcemap: false,
});

console.log('Bundle created: dist/bundle.cjs');
