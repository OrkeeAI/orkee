import { defineConfig } from 'tsup'

export default defineConfig({
  entry: ['src/index.ts'],
  format: ['cjs', 'esm'],
  dts: false, // Skip TypeScript declarations for now due to React type issues
  external: ['react', 'react-dom'],
  clean: true,
  minify: false,
  splitting: false,
  sourcemap: true
})