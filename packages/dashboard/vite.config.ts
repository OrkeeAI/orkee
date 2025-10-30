import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { fileURLToPath } from 'url'
import { dirname } from 'path'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

const apiPort = process.env.ORKEE_API_PORT || '4001'
const uiPort = parseInt(process.env.ORKEE_UI_PORT || process.env.VITE_PORT || '5173', 10)

export default defineConfig({
  plugins: [react()],
  envDir: `${__dirname}/../../`,
  base: './', // Use relative paths for assets - required for Tauri
  resolve: {
    alias: {
      '@': `${__dirname}/src`,
      '@orkee/tasks': `${__dirname}/../tasks/src`,
    },
  },
  define: {
    // Pass the dynamic API port to the frontend
    'import.meta.env.VITE_ORKEE_API_PORT': JSON.stringify(apiPort),
    'import.meta.env.VITE_ORKEE_UI_PORT': JSON.stringify(process.env.ORKEE_UI_PORT || ''),
  },
  server: {
    port: uiPort,
    strictPort: false, // Allow fallback to next available port
    host: 'localhost',
    proxy: {
      '/api': {
        target: `http://localhost:${apiPort}`,
        changeOrigin: true,
      },
    },
  },
})