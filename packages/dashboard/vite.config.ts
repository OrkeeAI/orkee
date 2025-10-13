import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  envDir: path.resolve(__dirname, '../../'),
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@orkee/tasks': path.resolve(__dirname, '../tasks/src'),
    },
  },
  define: {
    // Pass the dynamic API port to the frontend
    'import.meta.env.VITE_ORKEE_API_PORT': JSON.stringify(process.env.ORKEE_API_PORT || ''),
    'import.meta.env.VITE_ORKEE_UI_PORT': JSON.stringify(process.env.ORKEE_UI_PORT || ''),
  },
  build: {
    rollupOptions: {
      external: [
        // Mark Tauri packages as external - they're only available in Tauri builds
        // and should not be bundled in the web version
        '@tauri-apps/api/core',
        '@tauri-apps/plugin-http',
      ],
    },
  },
  server: {
    port: parseInt(process.env.ORKEE_UI_PORT || process.env.VITE_PORT || '5173'),
    strictPort: false, // Allow fallback to next available port
    host: 'localhost',
    proxy: {
      '/api': {
        target: `http://localhost:${process.env.ORKEE_API_PORT || '4001'}`,
        changeOrigin: true,
      },
    },
  },
})